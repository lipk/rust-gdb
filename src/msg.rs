use std::str;

#[derive(Debug)]
pub struct Message {
    pub token: Option<String>,
    pub class: MessageClass,
    pub content: Vec<Variable>,
}

#[derive(Debug, PartialEq)]
pub enum MessageClass {
    Done,
    Running,
    Connected,
    Error,
    Exit,
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

impl str::FromStr for MessageClass {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "done" => Ok(MessageClass::Done),
            "running" => Ok(MessageClass::Running),
            "connected" => Ok(MessageClass::Connected),
            "error" => Ok(MessageClass::Error),
            "exit" => Ok(MessageClass::Exit),
            _ => Err("unrecognized result class".to_string()),
        }
    }
}
