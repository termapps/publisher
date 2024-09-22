use std::io::Write;

use anstream::{eprintln, stderr, stdout};
use eyre::Result as EyreResult;
use owo_colors::OwoColorize;
use proc_exit::Code;

pub type Result<T = ()> = EyreResult<T>;

pub fn finish(result: Result) {
    let code = if let Some(e) = result.err() {
        // Use `e.is::<Error>()` to check for a specific error
        // in order to wrap all instances of it.
        let err = e;

        eprintln!("{}: {err}", "error".red().bold());
        Code::FAILURE
    } else {
        Code::SUCCESS
    };

    exit(code);
}

pub fn exit(code: Code) {
    stdout().flush().unwrap();
    stderr().flush().unwrap();

    code.process_exit();
}
