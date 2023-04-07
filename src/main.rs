mod error;

mod hello;

use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use concolor::{get, Stream};
use concolor_clap::{color_choice, Color};
use tracing_log::AsTrace;
use tracing_subscriber::prelude::*;

/// A simple CLI application using clap
#[derive(Debug, Parser)]
#[clap(name = "cli-clap", version)]
#[clap(color = color_choice())]
struct App {
    #[clap(subcommand)]
    cmd: Subcommands,

    #[clap(flatten)]
    color: Color,

    #[clap(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[derive(Debug, Parser)]
enum Subcommands {
    Hello(hello::Hello),
}

fn main() {
    let program = App::parse();

    program.color.apply();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_ansi(get(Stream::Stdout).color())
                .with_filter(program.verbose.log_level_filter().as_trace()),
        )
        .init();

    let result = match program.cmd {
        Subcommands::Hello(x) => x.run(),
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
