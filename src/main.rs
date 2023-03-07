mod error;

mod hello;

use clap::Parser;
use clap_verbosity_flag::Verbosity;
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
    verbose: Verbosity,
}

#[derive(Debug, Parser)]
enum Subcommands {
    Hello(hello::Hello),
}

fn main() {
    let program = App::parse();

    program.color.apply();

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
