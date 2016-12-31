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
    IOError,
    ParseError,
    IgnoredOutput
}

pub type Result<T> = result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error::IOError
    }
}

impl Debugger {
    fn read_sequence(&mut self) -> Result<Vec<msg::Record>> {
        let mut result = Vec::new();
        let mut line = String::new();
        try!(self.stdout.read_line(&mut line));
        while line != "(gdb) \n" {
            match parser::parse_line(line.as_str()) {
                Ok(resp) => result.push(resp),
                Err(err) => return Err(err),
            }
            line.clear();
            try!(self.stdout.read_line(&mut line));
        }
        Ok(result)
    }

    fn read_result_record(&mut self) -> Result<msg::MessageRecord<msg::ResultClass>> {
        loop {
            let sequence = try!(self.read_sequence());
            for record in sequence.into_iter() {
                match record {
                    msg::Record::Result(msg) => return Ok(msg),
                    _ => {}
                }
            }
        }
    }

    pub fn send_cmd_raw(&mut self, cmd: &str) -> Result<msg::MessageRecord<msg::ResultClass>> {
        try!(self.stdin.write_all(cmd.as_ref()));
        try!(self.stdin.flush());
        self.read_result_record()
    }

    pub fn start() -> Result<Self> {
        let mut child = try!(process::Command::new("gdb")
            .args(&["--interpreter=mi"])
            .stdout(process::Stdio::piped())
            .stdin(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn());
        let mut result = Debugger {
            stdin: BufWriter::new(child.stdin.take().expect("broken stdin")),
            stdout: BufReader::new(child.stdout.take().expect("broken stdout")),
        };
        try!(result.read_sequence());
        Ok(result)
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        let _ = self.stdin.write_all(b"-gdb-exit\n");
    }
}
