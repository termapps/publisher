mod aur;

use clap::ValueEnum;

use crate::{check::CheckResults, error::Result};

pub trait Repository {
    fn name(&self) -> &'static str;

    fn check(&self, check_result: &mut CheckResults) -> Result;
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Repositories {
    Aur,
}

impl Repositories {
    pub fn build_config(&self) -> impl Repository {
        match self {
            Repositories::Aur => aur::Aur,
        }
    }
}
