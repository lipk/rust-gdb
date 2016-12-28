# Description

*rust-gdb* is a WIP library for controlling GDB from Rust programs. At the
moment, it can launch a GDB process, pass commands, and parse gdb's responses.
*rust-gdb* uses GDB's
[Machine Interface](https://sourceware.org/gdb/onlinedocs/gdb/GDB_002fMI.html).

Missing features:

* Handle asynchronous output from GDB (currently it's ignored)
* Better interface for executing commands
* Proper documentation

# Usage

## Launching the debugger

    use gdb;

    let debugger = gdb::Debugger::start().unwrap();

The library will look for the *gdb* binary in your path.

## Executing commands

    use gdb;

    let mut debugger = gdb::Debugger::start().unwrap();
    let response = debugger.send_cmd_raw("your-command-here\n").unwrap();

*send_cmd_raw* currently blocks until it gets a result record from GDB, so don't
use async commands :)

## Response format

Currently only result records are returned by *send_cmd_raw*. GDB/MI output
structure is described [here](https://sourceware.org/gdb/onlinedocs/gdb/GDB_002fMI-Output-Syntax.html),
*rust-gdb* practically transforms this into a syntax tree, as described in
*msg.rs*.
