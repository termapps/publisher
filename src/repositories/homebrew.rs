use eyre::eyre;
use heck::ToUpperCamelCase;
use serde::{Deserialize, Serialize};
use xshell::Shell;

use super::get_checksums;
use crate::{
    check::{CheckResults, check_curl, check_git, check_repo},
    config::AppConfig,
    error::Result,
    publish::{commit_and_push, prepare_git_repo, write_and_add},
    repositories::Repository,
    targets::Target,
};

const NO_HOMEBREW_CONFIG: &str = "No configuration found for homebrew";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HomebrewConfig {
    pub name: Option<String>,
    pub repository: String,
}

#[derive(Debug, Clone)]
pub(super) struct Homebrew;

impl Repository for Homebrew {
    fn name(&self) -> &'static str {
        "Homebrew"
    }

    fn check(&self, results: &mut CheckResults, info: &AppConfig) -> Result {
        let sh = Shell::new()?;

        check_git(&sh, results);
        check_curl(&sh, results);

        if let Some(homebrew) = &info.homebrew {
            check_repo(
                &sh,
                &format!("git@github.com:{}", homebrew.repository),
                "master",
                results,
                false,
            )?;
        } else {
            results.add_result("config", Some("No configuration found for homebrew"));
        }

        Ok(())
    }

    fn publish(&self, info: &AppConfig, version: &str, dry_run: bool) -> Result {
        let AppConfig {
            name: cli_name,
            description,
            homepage,
            license,
            repository,
            ..
        } = info;

        let homebrew = info.homebrew.as_ref().ok_or(eyre!(NO_HOMEBREW_CONFIG))?;

        let name = get_name(info);
        let (sh, dir) = prepare_git_repo(self, &format!("git@github.com:{}", homebrew.repository))?;

        let checksums = get_checksums(
            info,
            version,
            vec![
                Target::Aarch64AppleDarwin,
                Target::X86_64AppleDarwin,
                Target::X86_64UnknownLinuxGnu,
            ],
        )?;

        write_and_add(&sh, &dir, format!("Formula/{name}.rb"), || {
            vec![
                format!("class {} < Formula", name.to_upper_camel_case()),
                format!("  version {version:?}"),
                format!("  desc {description:?}"),
                format!("  homepage {homepage:?}"),
                format!("  license {license:?}"),
                format!(""),
                format!("  if OS.mac?"),
                format!("    if Hardware::CPU.arm?"),
                format!(
                    "      url \"https://github.com/{repository}/releases/download/v#{{version}}/{cli_name}-v#{{version}}-aarch64-apple-darwin.zip\""
                ),
                format!(
                    "      sha256 {:?}",
                    checksums.get(&Target::Aarch64AppleDarwin).unwrap()
                ),
                format!("    else"),
                format!(
                    "      url \"https://github.com/{repository}/releases/download/v#{{version}}/{cli_name}-v#{{version}}-x86_64-apple-darwin.zip\""
                ),
                format!(
                    "      sha256 {:?}",
                    checksums.get(&Target::X86_64AppleDarwin).unwrap()
                ),
                format!("    end"),
                format!("  elsif OS.linux?"),
                format!(
                    "     url \"https://github.com/{repository}/releases/download/v#{{version}}/{cli_name}-v#{{version}}-x86_64-unknown-linux-gnu.zip\""
                ),
                format!(
                    "     sha256 {:?}",
                    checksums.get(&Target::X86_64UnknownLinuxGnu).unwrap()
                ),
                format!("  end"),
                format!(""),
                format!("  def install"),
                format!("    bin.install {cli_name:?}"),
                format!("  end"),
                format!(""),
                format!("  test do"),
                format!("    system \"#{{bin}}/{cli_name} --version\""),
                format!("  end"),
                format!("end"),
            ]
        })?;

        if !dry_run {
            commit_and_push(&sh, &name, version)?;
        }

        Ok(())
    }

    fn instructions(&self, info: &AppConfig) -> Result<Vec<String>> {
        let homebrew = info.homebrew.as_ref().ok_or(eyre!(NO_HOMEBREW_CONFIG))?;

        let name = get_name(info);
        let tap_org_name = homebrew.repository.split('/').next().unwrap();
        let tap_name = homebrew.repository.split('/').next_back().unwrap();

        let contents = if tap_name.starts_with("homebrew-") {
            format!(
                "brew install {tap_org_name}/{}/{name}",
                tap_name.trim_start_matches("homebrew-")
            )
        } else {
            [
                format!(
                    "brew tap {tap_org_name}/{tap_name} https://github.com/{tap_org_name}/{tap_name}"
                ),
                format!("brew install {tap_org_name}/{tap_name}/{name}"),
            ]
            .join("\n")
        };

        Ok(vec![
            format!("With [Homebrew](https://brew.sh)"),
            format!(""),
            format!("```"),
            contents,
            format!("```"),
        ])
    }
}

fn get_name(info: &AppConfig) -> String {
    info.homebrew
        .as_ref()
        .and_then(|homebrew| homebrew.name.clone())
        .unwrap_or_else(|| info.name.clone())
}
