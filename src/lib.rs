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

#[macro_use]
extern crate lazy_static;
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
