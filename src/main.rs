#![allow(clippy::useless_format)]

use std::io::stdout;

use anstream::{AutoStream, ColorChoice};
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use colorchoice_clap::Color;
use tracing_subscriber::prelude::*;

mod error;
mod styles;

mod config;
mod repositories;
mod targets;

mod check;
mod generate;
mod init;
mod publish;

/// Tool to publish & distribute CLI tools
#[derive(Debug, Parser)]
#[clap(name = "publisher", version)]
#[command(styles = styles::styles())]
struct App {
    #[command(subcommand)]
    cmd: Subcommands,

    #[command(flatten)]
    color: Color,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[derive(Debug, Parser)]
enum Subcommands {
    Check(check::Check),
    Generate(generate::Generate),
    Init(init::Init),
    Publish(publish::Publish),
}

fn main() {
    let program = App::parse();

    program.color.write_global();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_ansi(!matches!(AutoStream::choice(&stdout()), ColorChoice::Never))
                .with_filter(program.verbose.tracing_level_filter()),
        )
        .init();

    let result = match program.cmd {
        Subcommands::Check(x) => x.run(),
        Subcommands::Generate(x) => x.run(),
        Subcommands::Init(x) => x.run(),
        Subcommands::Publish(x) => x.run(),
    };

    error::finish(result);
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
