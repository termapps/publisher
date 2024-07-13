use std::{fmt::Debug, fs::write};

use crate::{error::Result, publish::read_config};

use clap::Parser;
use owo_colors::OwoColorize;
use tracing::{info, instrument};

mod ci;

/// Generates things related to publishing
#[derive(Debug, Parser)]
pub struct Generate {
    #[clap(subcommand)]
    cmd: Subcommands,
}

#[derive(Debug, Parser)]
enum Subcommands {
    CI(ci::CI),
}

impl Generate {
    #[instrument(name = "generate", skip_all)]
    pub fn run(self) -> Result {
        let config = read_config()?;

        match self.cmd {
            Subcommands::CI(x) => x.run(&config),
        }
    }
}

pub fn write_lines<P, F>(path: P, writer: F) -> Result
where
    P: AsRef<str> + Debug,
    F: FnOnce() -> Vec<String>,
{
    let path = path.as_ref();

    info!("{} {}", "writing".magenta(), path.cyan());
    let lines = writer();

    write(format!("{path}"), format!("{}\n", lines.join("\n")))?;

    Ok(())
}
