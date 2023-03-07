use std::io::{stdout, Write};

use crate::error::{Error, Result};

use clap::Parser;

/// Say hello to someone
#[derive(Debug, Parser)]
pub struct Hello {
    /// The name of the person to greet
    name: String,
}

impl Hello {
    pub fn run(self) -> Result {
        if self.name == "world" {
            return Err(Error::World);
        }

        println!("Hello, {}!", self.name);
        stdout().flush()?;

        Ok(())
    }
}
