extern crate regex;

mod dbg;
mod parser;
mod msg;

#[test]
fn start_debugger() {
    let mut dbg = dbg::Debugger::start().unwrap();
    let resp = dbg.send_cmd_raw("-break-info\n").unwrap();
    assert_eq!(msg::MessageClass::Done, resp.class);
}

pub use dbg::*;
pub use msg::*;
