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

use std::str;

#[derive(Debug)]
pub enum Record {
    Result(MessageRecord<ResultClass>),
    Async(AsyncRecord),
    Stream(StreamRecord)
}

#[derive(Debug)]
pub struct MessageRecord<ClassT> {
    pub token: Option<String>,
    pub class: ClassT,
    pub content: Vec<Variable>,
}

#[derive(Debug, PartialEq)]
pub enum ResultClass {
    Done,
    Running,
    Connected,
    Error,
    Exit
}

#[derive(Debug, PartialEq)]
pub enum AsyncClass {
    Stopped,
    Other
}

#[derive(Debug)]
pub enum AsyncRecord {
    Exec(MessageRecord<AsyncClass>),
    Status(MessageRecord<AsyncClass>),
    Notify(MessageRecord<AsyncClass>),
}

#[derive(Debug)]
pub enum StreamRecord {
    Console(Constant),
    Target(Constant),
    Log(Constant),
}

#[derive(Debug)]
pub struct Variable {
    pub name: VarName,
    pub value: Value
}

#[derive(Debug)]
pub enum Value {
    String(Constant),
    VariableList(Vec<Variable>),
    ValueList(Vec<Value>),
}

pub type VarName = String;
pub type Constant = String;

impl str::FromStr for ResultClass {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "done" => Ok(ResultClass::Done),
            "running" => Ok(ResultClass::Running),
            "connected" => Ok(ResultClass::Connected),
            "error" => Ok(ResultClass::Error),
            "exit" => Ok(ResultClass::Exit),
            _ => Err("unrecognized result class".to_string()),
        }
    }
}

impl str::FromStr for AsyncClass {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stopped" => Ok(AsyncClass::Stopped),
            _ => Ok(AsyncClass::Other),
        }
    }
}
