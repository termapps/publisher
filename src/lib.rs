use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use colorchoice_clap::Color;

use crate::{commands::Subcommands, error::Result};

pub mod error;
mod styles;

pub mod commands;

mod config;
pub mod repositories;
mod targets;

/// Tool to publish & distribute CLI tools
#[derive(Debug, Parser)]
#[clap(name = "publisher", version)]
#[command(styles = styles::styles())]
pub struct App {
    #[command(subcommand)]
    pub cmd: Subcommands,

    #[command(flatten)]
    pub color: Color,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}

impl App {
    pub fn run(self) -> Result {
        self.cmd.run()
    }

    pub fn new(cmd: Subcommands) -> Self {
        App {
            cmd,
            color: Color::default(),
            verbose: Verbosity::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use clap::CommandFactory;

    #[test]
    fn verify_app() {
        App::command().debug_assert();
    }
}
