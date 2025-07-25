use serde::{Deserialize, Serialize};
use xshell::Shell;

use super::get_checksums;
use crate::{
    check::{CheckResults, check_git, check_repo},
    config::AppConfig,
    error::Result,
    publish::{commit_and_push, prepare_git_repo, write_and_add},
    repositories::Repository,
    targets::Target,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScoopConfig {
    pub name: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct Scoop;

impl Repository for Scoop {
    fn name(&self) -> &'static str {
        "Scoop"
    }

    fn check(&self, results: &mut CheckResults, info: &AppConfig) -> Result {
        let sh = Shell::new()?;

        check_git(&sh, results);

        let repository = get_repository(info);

        check_repo(
            &sh,
            &format!("git@github.com:{repository}"),
            "master",
            results,
            false,
        )?;

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

        let name = get_name(info);
        let pkg_repo = get_repository(info);
        let (sh, dir) = prepare_git_repo(self, &format!("git@github.com:{pkg_repo}"))?;

        let checksums = get_checksums(
            info,
            version,
            vec![Target::I686PcWindowsMsvc, Target::X86_64PcWindowsMsvc],
        )?;

        write_and_add(&sh, &dir, format!("{name}.json"), || {
            vec![
                format!("{{"),
                format!("  \"version\": {version:?},"),
                format!("  \"description\": {description:?},"),
                format!("  \"homepage\": {homepage:?},"),
                format!("  \"license\": {license:?},"),
                format!("  \"architecture\": {{"),
                format!("    \"64bit\": {{"),
                format!(
                    "      \"url\": \"https://github.com/{repository}/releases/download/v{version}/{cli_name}-v{version}-x86_64-pc-windows-msvc.zip\","
                ),
                format!(
                    "      \"hash\": {:?}",
                    checksums.get(&Target::X86_64PcWindowsMsvc).unwrap()
                ),
                format!("    }},"),
                format!("    \"32bit\": {{"),
                format!(
                    "      \"url\": \"https://github.com/{repository}/releases/download/v{version}/{cli_name}-v{version}-i686-pc-windows-msvc.zip\","
                ),
                format!(
                    "      \"hash\": {:?}",
                    checksums.get(&Target::I686PcWindowsMsvc).unwrap()
                ),
                format!("    }}"),
                format!("  }},"),
                format!("  \"bin\": [\"{cli_name}.exe\"]"),
                format!("}}"),
            ]
        })?;

        if !dry_run {
            commit_and_push(&sh, &name, version)?;
        }

        Ok(())
    }

    fn instructions(&self, info: &AppConfig) -> Result<Vec<String>> {
        let name = get_name(info);
        let repository = get_repository(info);
        let bucket_org_name = repository.split('/').next().unwrap();

        Ok(vec![
            format!("With [Scoop](https://scoop.sh)"),
            format!(""),
            format!("```"),
            format!("scoop bucket add {bucket_org_name} https://github.com/{repository}"),
            format!("scoop install {name}"),
            format!("```"),
        ])
    }
}

fn get_name(info: &AppConfig) -> String {
    info.scoop
        .as_ref()
        .and_then(|scoop| scoop.name.clone())
        .unwrap_or_else(|| info.name.clone())
}

fn get_repository(info: &AppConfig) -> String {
    info.scoop
        .as_ref()
        .and_then(|scoop| scoop.repository.clone())
        .unwrap_or_else(|| info.repository.clone())
}
