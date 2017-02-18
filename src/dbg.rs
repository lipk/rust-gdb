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

use std::process;
use std::io::{Write, BufReader, BufWriter, BufRead};
use std::io;
use std::error;
use std::fmt;
use std::convert::From;
use std::result;
use std::str;
use parser;
use msg;

pub struct Debugger {
    stdin: BufWriter<process::ChildStdin>,
    stdout: BufReader<process::ChildStdout>,
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    ParseError,
    IgnoredOutput
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = self as &error::Error;
        write!(f, "{}", err.description())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::IOError(ref err) => err.description(),
            &Error::ParseError => "cannot parse response from gdb",
            &Error::IgnoredOutput => "ignored output"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::IOError(ref err) => Some(err),
            _ => None
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOError(err)
    }
}

impl Debugger {
    fn read_sequence(&mut self) -> Result<Vec<msg::Record>> {
        let mut result = Vec::new();
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        while line != "(gdb) \n" {
            match parser::parse_line(line.as_str()) {
                Ok(resp) => result.push(resp),
                Err(err) => return Err(err),
            }
            line.clear();
            self.stdout.read_line(&mut line)?;
        }
        Ok(result)
    }

    fn read_result_record(&mut self) -> Result<msg::MessageRecord<msg::ResultClass>> {
        loop {
            let sequence = self.read_sequence()?;
            for record in sequence.into_iter() {
                match record {
                    msg::Record::Result(msg) => return Ok(msg),
                    _ => {}
                }
            }
        }
    }

    pub fn send_cmd_raw(&mut self, cmd: &str) -> Result<msg::MessageRecord<msg::ResultClass>> {
        if cmd.ends_with("\n") {
            write!(self.stdin, "{}", cmd)?;
        } else {
            writeln!(self.stdin, "{}", cmd)?;
        }
        self.stdin.flush()?;
        self.read_result_record()
    }

    pub fn start() -> Result<Self> {
        let mut child = process::Command::new("gdb")
            .args(&["--interpreter=mi"])
            .stdout(process::Stdio::piped())
            .stdin(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()?;
        let mut result = Debugger {
            stdin: BufWriter::new(child.stdin.take().expect("broken stdin")),
            stdout: BufReader::new(child.stdout.take().expect("broken stdout")),
        };
        result.read_sequence()?;
        Ok(result)
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        let _ = self.stdin.write_all(b"-gdb-exit\n");
    }
}
