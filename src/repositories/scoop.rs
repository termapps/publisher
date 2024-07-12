use xshell::Shell;

use super::{get_checksums, Repository};
use crate::{
    check::{check_curl, check_git, check_repo, CheckResults},
    error::{Error, Result},
    publish::{commit_and_push, prepare_git_repo, write_and_add, PublishInfo},
    targets::Target,
};

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ScoopInfo {
    pub name: Option<String>,
    pub repository: String,
}

#[derive(Debug, Clone)]
pub(super) struct Scoop;

impl Repository for Scoop {
    fn name(&self) -> &'static str {
        "Scoop"
    }

    fn check(&self, results: &mut CheckResults, info: &PublishInfo) -> Result {
        let sh = Shell::new()?;

        check_git(&sh, results);
        check_curl(&sh, results);

        if let Some(scoop) = &info.scoop {
            check_repo(
                &sh,
                &format!("git@github.com:{}", scoop.repository),
                "master",
                results,
                false,
            )?;
        } else {
            results.add_result("config", Some("No configuration found for scoop"));
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

        let scoop = info.scoop.as_ref().ok_or(Error::NoScoopConfig)?;

        let name = get_name(info);
        let (sh, dir) = prepare_git_repo(self, &format!("git@github.com:{}", scoop.repository))?;

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
                format!("      \"url\": \"https://github.com/{repository}/releases/download/v{version}/{cli_name}-v{version}-x86_64-pc-windows-msvc.zip\","),
                format!("      \"hash\": {:?}", checksums.get(&Target::X86_64PcWindowsMsvc).unwrap()),
                format!("    }},"),
                format!("    \"32bit\": {{"),
                format!("      \"url\": \"https://github.com/{repository}/releases/download/v{version}/{cli_name}-v{version}-i686-pc-windows-msvc.zip\","),
                format!("      \"hash\": {:?}", checksums.get(&Target::I686PcWindowsMsvc).unwrap()),
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
}

fn get_name(info: &PublishInfo) -> String {
    info.scoop
        .as_ref()
        .and_then(|info| info.name.clone())
        .unwrap_or_else(|| info.name.clone())
}