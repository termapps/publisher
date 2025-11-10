use serde::{Deserialize, Serialize};
use xshell::{Shell, cmd};

use crate::{
    check::{CheckResults, check_git, check_program, check_repo},
    config::AppConfig,
    error::Result,
    publish::{commit_and_push, prepare_git_repo, write_and_add},
    repositories::{Repository, get_checksums},
    targets::Target,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DebianConfig {
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct Debian;

impl Repository for Debian {
    fn name(&self) -> &'static str {
        "Debian"
    }

    fn check(&self, results: &mut CheckResults, info: &AppConfig) -> Result {
        let sh = Shell::new()?;

        check_git(&sh, results);
        check_program(&sh, results, "curl", "curl --version", "curl ");

        Ok(())
    }

    fn publish(&self, info: &AppConfig, version: &str, dry_run: bool) -> Result {
        let AppConfig { name: cli_name, .. } = info;

        let name = get_name(info);

        let checksums = get_checksums(
            info,
            version,
            vec![Target::X86_64UnknownLinuxGnu, Target::I686UnknownLinuxGnu],
        )?;

        Ok(())
    }

    fn instructions(&self, info: &AppConfig) -> Result<Vec<String>> {
        let name = get_name(info);

        Ok(vec![
            format!("With [Debian]()"),
            format!(""),
            format!("```"),
            format!(
                "sudo apt-add-repository https://raw.githubusercontent.com/termapps/ppa/master"
            ),
            format!("sudo apt-get update"),
            format!("sudo apt-get install {name}"),
            format!("```"),
            format!(""),
        ])
    }
}

fn get_name(info: &AppConfig) -> String {
    info.debian
        .as_ref()
        .and_then(|debian| debian.name.clone())
        .unwrap_or_else(|| info.name.clone())
}
