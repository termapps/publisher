use heck::ToUpperCamelCase;
use xshell::Shell;

use super::{get_checksums, Repository};
use crate::{
    check::{check_curl, check_git, check_repo, CheckResults},
    error::{Error, Result},
    publish::{commit_and_push, prepare_git_repo, write_and_add, PublishInfo},
    targets::Target,
};

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct HomebrewInfo {
    pub name: Option<String>,
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

    fn publish(&self, info: &PublishInfo, version: &str, dry_run: bool) -> Result {
        let PublishInfo {
            name: cli_name,
            description,
            homepage,
            license,
            repository,
            ..
        } = info;

        let homebrew = info.homebrew.as_ref().ok_or(Error::NoHomebrewConfig)?;

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
                format!("      url \"https://github.com/{repository}/releases/download/v#{{version}}/{cli_name}-v#{{version}}-aarch64-apple-darwin.zip\""),
                format!("      sha256 {:?}", checksums.get(&Target::Aarch64AppleDarwin).unwrap()),
                format!("    else"),
                format!("      url \"https://github.com/{repository}/releases/download/v#{{version}}/{cli_name}-v#{{version}}-x86_64-apple-darwin.zip\""),
                format!("      sha256 {:?}", checksums.get(&Target::X86_64AppleDarwin).unwrap()),
                format!("    end"),
                format!("  elsif OS.linux?"),
                format!("     url \"https://github.com/{repository}/releases/download/v#{{version}}/{cli_name}-v#{{version}}-x86_64-unknown-linux-gnu.zip\""),
                format!("     sha256 {:?}", checksums.get(&Target::X86_64UnknownLinuxGnu).unwrap()),
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
}

fn get_name(info: &PublishInfo) -> String {
    info.homebrew
        .as_ref()
        .and_then(|info| info.name.clone())
        .unwrap_or_else(|| info.name.clone())
}
