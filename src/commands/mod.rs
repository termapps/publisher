use clap::Parser;

use crate::error::Result;

pub mod check;
pub mod generate;
pub mod init;
pub mod publish;

#[derive(Debug, Parser)]
pub enum Subcommands {
    Check(check::Check),
    Generate(generate::Generate),
    Init(init::Init),
    Publish(publish::Publish),
}

impl Subcommands {
    pub(crate) fn run(&self) -> Result {
        match self {
            Self::Check(x) => x.run(),
            Self::Generate(x) => x.run(),
            Self::Init(x) => x.run(),
            Self::Publish(x) => x.run(),
        }
    }
}
