use std::{io, process::exit};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to parse the configuration file: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Unable to run a command: {0}")]
    Xshell(#[from] xshell::Error),
    #[error("{0}")]
    Fmt(#[from] std::fmt::Error),
    #[error("{0}")]
    Io(#[from] io::Error),
}

impl Error {
    fn print(self) -> io::Result<()> {
        eprintln!("error: {self}");

        Ok(())
    }

    fn code(&self) -> i32 {
        1
    }
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub fn finish(result: Result) {
    let code = if let Some(e) = result.err() {
        let code = e.code();

        e.print().unwrap();
        code
    } else {
        0
    };

    // TODO: Flush stdout and stderr

    exit(code);
}
