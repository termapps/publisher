use clap::Parser;

/// Tool to publish & distribute CLI tools
#[derive(Debug, Parser)]
#[clap(name = "publisher")]
struct App {
    #[clap(subcommand)]
    cmd: Subcommands,
}

#[derive(Debug, Parser)]
enum Subcommands {}

fn main() {
    let program = App::parse();

    match program.cmd {
        // Subcommands::Pie(x) => x.run(),
    }
}
