use heck::ToUpperCamelCase;
use xshell::Shell;

use crate::{
    check::{check_curl, check_git, check_repo, CheckResults},
    error::{Error, Result},
    publish::{commit_and_push, prepare_git_repo, write_and_add, PublishInfo},
    repositories::Repository,
};

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct HomebrewInfo {
    pub repository: String,
}

#[derive(Debug, Clone)]
pub(super) struct Homebrew;

impl Repository for Homebrew {
    fn name(&self) -> &'static str {
        "Homebrew"
    }

    fn check(&self, results: &mut CheckResults, info: &PublishInfo) -> Result {
        let sh = Shell::new()?;

        check_git(&sh, results);
        check_curl(&sh, results);

        if info.homebrew.is_none() {
            results.add_result("config", Some("No configuration found for homebrew"));
        }

        if let Some(homebrew) = &info.homebrew {
            let repository = &homebrew.repository;

            check_repo(
                &sh,
                &format!("git@github.com:{repository}"),
                "master",
                results,
                false,
            )?;
        } else {
            results.add_result("config", Some("No configuration found for homebrew"));
        }

        Ok(())
    }

    fn publish(&self, info: &PublishInfo, version: &str, dry_run: bool) -> Result {
        let PublishInfo {
            name,
            description,
            homepage,
            ..
        } = &info;

        let homebrew = info.homebrew.as_ref().ok_or(Error::NoHomebrewConfig)?;

        let (sh, dir) = prepare_git_repo(self, &format!("git@github.com:{}", homebrew.repository))?;

        write_and_add(&sh, &dir, format!("Formula/{name}.rb"), || {
            vec![
                format!("class {} < Formula", name.to_upper_camel_case()),
                format!("  version {version:?}"),
                format!("  description {description:?}"),
                format!("  homepage {homepage:?}"),
                format!("end"),
            ]
        })?;

        if !dry_run {
            commit_and_push(&sh, name, version)?;
        }

        Ok(())
    }
}
