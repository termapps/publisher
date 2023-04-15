use std::io::Write;

use crate::error::{Error, Result};

use anstream::{println, stdout};
use clap::Parser;
use owo_colors::OwoColorize;
use tracing::{debug, instrument};

/// Say hello to someone
#[derive(Debug, Parser)]
pub struct Hello {
    /// The name of the person to greet
    name: String,
}

impl Hello {
    #[instrument(name = "hello", skip_all)]
    pub fn run(self) -> Result {
        if self.name == "world" {
            return Err(Error::World);
        }

        println!("Hello, {}!", self.name.yellow());

        debug!("flushing stdout");
        stdout().flush()?;

        Ok(())
    }
}
