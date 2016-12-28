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

pub fn parse_line(mut line: &str) -> dbg::Result<msg::Message> {
    let mut token = None;
    if let Some((tok, rest)) = parse_token(line) {
        token = Some(tok);
        line = rest;
    }
    if !line.starts_with("^") {
        return Err(dbg::Error::IgnoredOutput);
    }
    line = line.split_at(1).1;
    let class = if let Some((class, rest)) = parse_message_class(line) {
        line = rest;
        class
    } else {
        return Err(dbg::Error::ParseError);
    };
    let mut result = Vec::new();
    if line.starts_with("\n") {
        return Ok(msg::Message {
            token: token,
            class: class,
            content: result
        });
    } else if !line.starts_with(",") {
        return Err(dbg::Error::ParseError);
    }
    line = line.split_at(1).1;
    if let Some((variable, rest)) = parse_variable(line) {
        line = rest;
        result.push(variable);
    } else {
        return Err(dbg::Error::ParseError);
    }
    while !line.starts_with("\n") {
        if !line.starts_with(",") {
            return Err(dbg::Error::ParseError);
        }
        line = line.split_at(1).1;
        if let Some((variable, rest)) = parse_variable(line) {
            line = rest;
            result.push(variable);
        } else {
            return Err(dbg::Error::ParseError);
        }
    }
    Ok(msg::Message {
        token: token,
        class: class,
        content: result
    })
}

fn parse<T: str::FromStr>(data: &str, toklen: usize) -> (T, &str) {
    let (x, y) = data.split_at(toklen);
    (T::from_str(x).ok().unwrap(), y)
}

fn parse_token(data: &str) -> Option<(String, &str)> {
    let reg = regex::Regex::new(r"^[0-9]+").unwrap();
    if let Some((_, count)) = reg.find(data) {
        Some(parse(data, count))
    } else {
        None
    }
}

fn parse_message_class(data: &str) -> Option<(msg::MessageClass, &str)> {
    let reg = regex::Regex::new(r"^(done|connected|running|error|exit)").unwrap();
    if let Some((_, count)) = reg.find(data) {
        Some(parse(data, count))
    } else {
        None
    }
}

fn parse_varname(data: &str) -> Option<(msg::VarName, &str)> {
    let reg = regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*").unwrap();
    if let Some((_, count)) = reg.find(data) {
        Some(parse(data, count))
    } else {
        None
    }
}

fn parse_constant(data: &str) -> Option<(msg::Value, &str)> {
    let reg = regex::Regex::new(r#"^(".*?[^\\]"|"")"#).unwrap();
    if let Some((_, count)) = reg.find(data) {
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
