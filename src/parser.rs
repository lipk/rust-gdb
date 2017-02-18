/*
 * This file is part of rust-gdb.
 *
 * rust-gdb is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * rust-gdb is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with rust-gdb.  If not, see <http://www.gnu.org/licenses/>.
 */

use regex;
use std::str;
use msg;
use dbg;

macro_rules! static_regex {
    ($id:ident = $val:expr) => {
        lazy_static! {
            static ref $id: regex::Regex = regex::Regex::new($val).unwrap();
        }
    }
}

pub fn parse_line(line: &str) -> Result<msg::Record, dbg::Error> {
    if let Some(result) = parse_result_line(line) {
        Ok(msg::Record::Result(result))
    } else if let Some(async) = parse_async_line(line) {
        Ok(msg::Record::Async(async))
    } else if let Some(stream) = parse_stream_line(line) {
        Ok(msg::Record::Stream(stream))
    } else {
        Err(dbg::Error::ParseError)
    }
}

pub fn parse_result_line(mut line: &str) -> Option<msg::MessageRecord<msg::ResultClass>> {
    let mut token = None;
    if let Some((tok, rest)) = parse_token(line) {
        token = Some(tok);
        line = rest;
    }
    if !line.starts_with("^") {
        return None;
    }
    line = line.split_at(1).1;
    let class = if let Some((class, rest)) = parse_result_class(line) {
        line = rest;
        class
    } else {
        return None
    };
    let mut result = Vec::new();
    if line.starts_with("\n") {
        return Some(msg::MessageRecord::<msg::ResultClass> {
            token: token,
            class: class,
            content: result
        });
    } else if !line.starts_with(",") {
        return None;
    }
    line = line.split_at(1).1;
    if let Some((variable, rest)) = parse_variable(line) {
        line = rest;
        result.push(variable);
    } else {
        return None;
    }
    while !line.starts_with("\n") {
        if !line.starts_with(",") {
            return None;
        }
        line = line.split_at(1).1;
        if let Some((variable, rest)) = parse_variable(line) {
            line = rest;
            result.push(variable);
        } else {
            return None;
        }
    }
    Some(msg::MessageRecord::<msg::ResultClass> {
        token: token,
        class: class,
        content: result
    })
}

pub fn parse_async_line(mut line: &str) -> Option<msg::AsyncRecord> {
    let mut token = None;
    if let Some((tok, rest)) = parse_token(line) {
        token = Some(tok);
        line = rest;
    }
    let async_type = if let Some(first) = line.chars().nth(0) {
        match first {
            '=' | '+' | '*' => first,
            _ => return None
        }
    } else {
        return None
    };
    line = line.split_at(1).1;
    let class = if let Some((class, rest)) = parse_async_class(line) {
        line = rest;
        class
    } else {
        return None
    };
    let mut result = Vec::new();
    if line.starts_with("\n") {
        let msg = msg::MessageRecord::<msg::AsyncClass> {
            token: token,
            class: class,
            content: result
        };
        return Some(match async_type {
            '=' => msg::AsyncRecord::Notify(msg),
            '+' => msg::AsyncRecord::Status(msg),
            '*' => msg::AsyncRecord::Exec(msg),
            _ => panic!("unrecognized async type ???!!!")
        });
    } else if !line.starts_with(",") {
        return None;
    }
    line = line.split_at(1).1;
    if let Some((variable, rest)) = parse_variable(line) {
        line = rest;
        result.push(variable);
    } else {
        return None;
    }
    while !line.starts_with("\n") {
        if !line.starts_with(",") {
            return None;
        }
        line = line.split_at(1).1;
        if let Some((variable, rest)) = parse_variable(line) {
            line = rest;
            result.push(variable);
        } else {
            return None;
        }
    }
    let msg = msg::MessageRecord::<msg::AsyncClass> {
        token: token,
        class: class,
        content: result
    };
    Some(match async_type {
        '=' => msg::AsyncRecord::Notify(msg),
        '+' => msg::AsyncRecord::Status(msg),
        '*' => msg::AsyncRecord::Exec(msg),
        _ => panic!("unrecognized async type ???!!!")
    })
}

pub fn parse_stream_line(mut line: &str) -> Option<msg::StreamRecord> {
    let stream_type = match line.chars().nth(0) {
        Some(t@'~') | Some(t@'@') | Some(t@'&') => t,
        _ => return None
    };
    line = line.split_at(1).1;
    if let Some((msg::Value::String(content), rest)) = parse_constant(line) {
        if rest == "\n" {
            Some(match stream_type {
                '~' => msg::StreamRecord::Console(content),
                '@' => msg::StreamRecord::Target(content),
                '&' => msg::StreamRecord::Log(content),
                _ => panic!("this is weird"),
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn parse<T: str::FromStr>(data: &str, toklen: usize) -> (T, &str) {
    let (x, y) = data.split_at(toklen);
    (T::from_str(x).ok().unwrap(), y)
}

fn parse_token(data: &str) -> Option<(String, &str)> {
    static_regex!(RE = r"^[0-9]+");
    if let Some((_, count)) = RE.find(data) {
        Some(parse(data, count))
    } else {
        None
    }
}

fn parse_result_class(data: &str) -> Option<(msg::ResultClass, &str)> {
    static_regex!(RE = r"^(done|connected|running|error|exit)");
    if let Some((_, count)) = RE.find(data) {
        Some(parse(data, count))
    } else {
        None
    }
}

fn parse_async_class(data: &str) -> Option<(msg::AsyncClass, &str)> {
    static_regex!(RE = r"^[-a-zA-Z]+");
    if let Some((_, count)) = RE.find(data) {
        Some(parse(data, count))
    } else {
        None
    }
}

fn parse_varname(data: &str) -> Option<(msg::VarName, &str)> {
    static_regex!(RE = r"^[a-zA-Z_][a-zA-Z0-9_-]*");
    if let Some((_, count)) = RE.find(data) {
        Some(parse(data, count))
    } else {
        None
    }
}

fn parse_constant(data: &str) -> Option<(msg::Value, &str)> {
    static_regex!(RE = r#"^(".*?[^\\]"|"")"#);
    if let Some((_, count)) = RE.find(data) {
        let (value, rest) = parse(data, count);
        Some((msg::Value::String(value), rest))
    } else {
        None
    }
}

fn parse_variable_list(data: &str) -> Option<(msg::Value, &str)> {
    let mut end = "}";
    if data.starts_with("[") {
        end = "]";
    }
    if !data.starts_with("{") {
        return None;
    }
    let mut data = data.split_at(1).1;
    let mut result = Vec::new();
    if data.starts_with(end) {
        return Some((msg::Value::VariableList(result), data.split_at(1).1));
    }
    if let Some((variable, rest)) = parse_variable(data) {
        data = rest;
        result.push(variable);
    } else {
        return None;
    }
    while !data.starts_with(end) {
        if !data.starts_with(",") {
            return None;
        }
        data = data.split_at(1).1;
        if let Some((variable, rest)) = parse_variable(data) {
            data = rest;
            result.push(variable);
        } else {
            return None;
        }
    }
    Some((msg::Value::VariableList(result), data.split_at(1).1))
}

fn parse_value_list(data: &str) -> Option<(msg::Value, &str)> {
    if !data.starts_with("[") {
        return None;
    }
    let mut data = data.split_at(1).1;
    let mut result = Vec::new();
    if data.starts_with("]") {
        return Some((msg::Value::ValueList(result), data.split_at(1).1));
    }
    if let Some((value, rest)) = parse_value(data) {
        data = rest;
        result.push(value);
    } else {
        return None;
    }
    while !data.starts_with("]") {
        if !data.starts_with(",") {
            return None;
        }
        data = data.split_at(1).1;
        if let Some((value, rest)) = parse_value(data) {
            data = rest;
            result.push(value);
        } else {
            return None;
        }
    }
    Some((msg::Value::ValueList(result), data.split_at(1).1))
}

fn parse_value(data: &str) -> Option<(msg::Value, &str)> {
    parse_constant(data).or(parse_variable_list(data)).or(parse_value_list(data))
}

fn parse_variable(data: &str) -> Option<(msg::Variable, &str)> {
    if let Some((var, rest)) = parse_varname(data) {
        match rest.chars().nth(0) {
            Some('=') => if let Some((val, rest)) =
                    parse_value(rest.split_at(1).1) {
                        Some((msg::Variable { name: var, value: val }, rest))   
                    } else {
                        None
                    }
                ,
            _ => None
        }
    } else {
        None
    }
}
