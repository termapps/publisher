use crate::error::Result;

use clap::Parser;

/// Say hello to someone
#[derive(Debug, Parser)]
pub struct Hello {
    /// The name of the person to greet
    name: String,
}

impl Hello {
    pub fn run(self) -> Result {
        println!("Hello, {}!", self.name);

        Ok(())
    }
}
