use std::io::stdout;

mod repositories;
mod targets;

mod error;
mod styles;

mod check;
mod generate;
mod publish;

use anstream::{AutoStream, ColorChoice};
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use colorchoice_clap::Color;
use tracing_log::AsTrace;
use tracing_subscriber::prelude::*;

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
                .with_filter(program.verbose.log_level_filter().as_trace()),
        )
        .init();

    let result = match program.cmd {
        Subcommands::Check(x) => x.run(),
        Subcommands::Generate(x) => x.run(),
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
