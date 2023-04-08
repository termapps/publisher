use std::{
    io::{stderr, stdout, Error as IoError, Write},
    process::exit,
    result::Result as StdResult,
};

use anstream::eprintln;
use owo_colors::OwoColorize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("You cannot use cliche")]
    World,
    #[error("{0}")]
    Io(#[from] IoError),
}

impl Error {
    fn print(&self) {
        eprintln!("{}: {self}", "error".red().bold());
    }

    fn code(&self) -> i32 {
        1
    }
}

pub type Result<T = ()> = StdResult<T, Error>;

pub fn finish(result: Result) {
    let code = if let Some(e) = result.err() {
        e.print();
        e.code()
    } else {
        0
    };

    stdout().flush().unwrap();
    stderr().flush().unwrap();

    exit(code);
}
