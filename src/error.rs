use std::{
    io::{Error as IoError, Write},
    result::Result as StdResult,
};

use anstream::{eprintln, stderr, stdout};
use owo_colors::OwoColorize;
use proc_exit::Code;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Some checks failed")]
    ChecksFailed,
    #[error("No configuration found for homebrew")]
    NoHomebrewConfig,
    #[error("No configuration found for scoop")]
    NoScoopConfig,
    #[error("Unable to find start marker to place instructions")]
    StartMarkerNotFound,
    #[error("Unable to find end marker to place instructions")]
    EndMarkerNotFound,
    #[error("{0}")]
    Regex(#[from] regex::Error),
    #[error("{0}")]
    Inquire(#[from] inquire::InquireError),
    #[error("{0}")]
    Toml(#[from] toml::ser::Error),
    #[error("Unable to parse the configuration file: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Unable to run a command: {0}")]
    Xshell(#[from] xshell::Error),
    #[error("{0}")]
    Fmt(#[from] std::fmt::Error),
    #[error("{0}")]
    Io(#[from] IoError),
}

impl Error {
    fn print(&self) {
        eprintln!("{}: {self}", "error".red().bold());
    }

    fn code(&self) -> Code {
        Code::FAILURE
    }
}

pub type Result<T = ()> = StdResult<T, Error>;

pub fn finish(result: Result) {
    let code = if let Some(e) = result.err() {
        e.print();
        e.code()
    } else {
        Code::SUCCESS
    };

    stdout().flush().unwrap();
    stderr().flush().unwrap();

    code.process_exit();
}
