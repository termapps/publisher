use std::{fmt::Debug, fs::write};

use clap::Parser;
use owo_colors::OwoColorize;
use tracing::{info, instrument};

use crate::{config::read_config, error::Result};

pub mod ci;
pub mod instructions;

/// Generates things related to publishing
#[derive(Debug, Parser)]
pub struct Generate {
    #[clap(subcommand)]
    pub cmd: Subcommands,
}

#[derive(Debug, Parser)]
pub enum Subcommands {
    CI(ci::CI),
    Instructions(instructions::Instructions),
}

impl Generate {
    #[instrument(name = "generate", skip_all)]
    pub(crate) fn run(&self) -> Result {
        let config = read_config()?;

        match &self.cmd {
            Subcommands::CI(x) => x.run(&config),
            Subcommands::Instructions(x) => x.run(&config),
        }
    }
}

fn write_lines<P, F>(path: P, writer: F) -> Result
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
