use std::{fmt::Write, fs::write};

use heck::ToUpperCamelCase;
use tracing::info;
use xshell::{cmd, Shell};

use crate::{
    check::{check_curl, check_git, CheckResults},
    error::Result,
    publish::{commit_and_push, prepare_git_repo, PublishInfo},
    repositories::Repository,
};

#[derive(Debug, Clone)]
pub struct Homebrew;

impl Repository for Homebrew {
    fn name(&self) -> &'static str {
        "Homebrew"
    }

    fn check(&self, results: &mut CheckResults) -> Result {
        let sh = Shell::new()?;

        check_git(&sh, results);
        check_curl(&sh, results);

        Ok(())
    }

    fn publish(&self, info: &PublishInfo, version: &str) -> Result {
        let PublishInfo {
            name,
            description,
            homepage,
            ..
        } = &info;

        // TODO: Remove this
        println!("{:?}", info);

        let (sh, dir) =
            prepare_git_repo(self, &format!("git@github.com:{}", "termapps/homebrew-tap"))?;

        info!("Writing formula");
        let mut formula = String::new();

        writeln!(formula, "class {} < Formula", name.to_upper_camel_case())?;
        writeln!(formula, "  version {version:?}")?;
        writeln!(formula, "  description {description:?}")?;
        writeln!(formula, "  homepage {homepage:?}")?;
        writeln!(formula, "end")?;

        write(format!("{dir}/Formula/{name}.rb"), formula)?;

        cmd!(sh, "git add Formula/{name}.rb").run()?;

        commit_and_push(self, &sh, version)?;

        Ok(())
    }
}
