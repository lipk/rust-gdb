#![allow(dead_code)]

use std::process;
use std::io::{Write, BufReader, BufWriter, BufRead};
use std::io;
use std::convert::From;
use std::result;


pub struct Debugger {
    proc_handle: process::Child,
    stdin: BufWriter<process::ChildStdin>,
    stdout: BufReader<process::ChildStdout>,
    stderr: BufReader<process::ChildStderr>,
}

#[derive(Debug)]
pub enum Error {
    IOError,
}

type Result<T> = result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error::IOError
    }
}

impl Debugger {
    fn read_sequence(&mut self) -> Result<Vec<String>> {
        let mut result = Vec::new();
        let mut line = String::new();
        try!(self.stdout.read_line(&mut line));
        while line != "(gdb) \n" {
            result.push(line.clone());
            line.clear();
            try!(self.stdout.read_line(&mut line));
        }
        Ok(result)
    }

    fn read_response(&mut self) -> Result<String> {
        loop {
            let sequence = try!(self.read_sequence());
            for line in sequence.into_iter() {
                if line.starts_with("^") {
                    return Ok(line);
                }
            }
        }
    }

    pub fn send_cmd(&mut self, cmd: &str) -> Result<String> {
        try!(self.stdin.write_all(cmd.as_ref()));
        try!(self.stdin.flush());
        self.read_response()
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
            stderr: BufReader::new(child.stderr.take().expect("broken stderr")),
            proc_handle: child,
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
