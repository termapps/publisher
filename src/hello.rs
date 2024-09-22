use std::io::Write;

use anstream::{println, stdout};
use clap::Parser;
use eyre::eyre;
use owo_colors::OwoColorize;
use tracing::{debug, instrument};

use crate::error::Result;

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
            return Err(eyre!("You cannot use cliche"));
        }

        println!("Hello, {}!", self.name.yellow());

        debug!("flushing stdout");
        stdout().flush()?;

        Ok(())
    }
}
