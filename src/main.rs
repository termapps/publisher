mod repositories;

mod error;

mod check;
mod publish;

use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use concolor_clap::{color_choice, Color};

/// Tool to publish & distribute CLI tools
#[derive(Debug, Parser)]
#[clap(name = "publisher", version)]
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
    Check(check::Check),
    Publish(publish::Publish),
}

fn main() {
    let program = App::parse();

    program.color.apply();

    env_logger::Builder::from_default_env()
        .format_target(false)
        .format_timestamp(None)
        .filter_level(program.verbose.log_level_filter())
        .init();

    let result = match program.cmd {
        Subcommands::Check(x) => x.run(),
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
