#![allow(dead_code)]

mod gdb;

#[test]
fn start_debugger() {
    let mut dbg = gdb::Debugger::start().unwrap();
    let resp = dbg.send_cmd("-break-info\n").unwrap();
    assert!(resp.starts_with("^done"));
}

