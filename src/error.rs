use std::{io, process::exit};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("You cannot use cliche")]
    World,
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
