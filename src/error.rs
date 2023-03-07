use std::{io, process::exit};

#[derive(Debug)]
pub enum Error {}

impl Error {
    pub fn print(&self) -> io::Result<()> {
        // TODO: print error message
        Ok(())
    }

    pub fn code(&self) -> i32 {
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
