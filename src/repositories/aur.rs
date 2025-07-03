use serde::{Deserialize, Serialize};
use xshell::{Shell, cmd};

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
pub struct AurConfig {
    pub name: Option<String>,
    pub conflicts: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub(super) struct Aur;

impl Repository for Aur {
    fn name(&self) -> &'static str {
        "AUR"
    }

    fn check(&self, results: &mut CheckResults, info: &AppConfig) -> Result {
        let sh = Shell::new()?;

        check_git(&sh, results);

        let ssh_configured = cmd!(sh, "ssh aur@aur.archlinux.org")
            .quiet()
            .ignore_status()
            .read_stderr()?
            .contains("Interactive shell is disabled.");

        results.add_result(
            "ssh",
            (!ssh_configured).then_some("AUR SSH access is not configured"),
        );

        let name = get_name(info);

        check_repo(
            &sh,
            &if ssh_configured {
                format!("ssh://aur@aur.archlinux.org/{name}.git")
            } else {
                format!("https://aur.archlinux.org/{name}.git")
            },
            "master",
            results,
            true,
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
        let (sh, dir) = prepare_git_repo(self, &format!("ssh://aur@aur.archlinux.org/{name}.git"))?;

        let github_repo_name = repository.split('/').next_back().unwrap();

        let checksums = get_checksums(info, version, vec![Target::Source])?;

        let conflicts = info
            .aur
            .as_ref()
            .and_then(|info| info.conflicts.clone())
            .unwrap_or_default();

        let conflicts_pkgbuild = conflicts
            .iter()
            .map(|c| format!("{c:?}"))
            .collect::<Vec<String>>()
            .join(" ");

        let conflicts_srcinfo = conflicts
            .iter()
            .map(|c| format!("\tconflicts = {c}"))
            .collect::<Vec<String>>()
            .join("\n");

        write_and_add(&sh, &dir, "PKGBUILD", || {
            vec![
                format!("pkgname={name}"),
                format!("pkgver={version}"),
                format!("pkgrel=0"),
                format!("pkgdesc={description:?}"),
                format!("arch=('x86_64' 'i686')"),
                format!("url={homepage:?}"),
                format!("license=({license:?})"),
                format!("makedepends=('cargo')"),
                format!("provides=({cli_name:?})"),
                format!("conflicts=({conflicts_pkgbuild})"),
                format!(
                    "source=($pkgname-$pkgver.zip::https://github.com/{repository}/archive/refs/tags/v$pkgver.zip)"
                ),
                format!("sha256sums=({:?})", checksums.get(&Target::Source).unwrap()),
                format!(""),
                format!("build() {{"),
                format!("    cd \"$srcdir/{github_repo_name}-$pkgver\""),
                format!("    cargo build --release --locked"),
                format!("}}"),
                format!(""),
                format!("package() {{"),
                format!("    cd \"$srcdir/{github_repo_name}-$pkgver\""),
                format!(
                    "    install -Dm755 \"target/release/{cli_name}\" \"$pkgdir/usr/bin/{cli_name}\""
                ),
                format!(
                    "    install -Dm644 \"LICENSE\" \"$pkgdir/usr/share/licenses/{cli_name}/LICENSE\""
                ),
                format!("}}"),
            ]
        })?;

        write_and_add(&sh, &dir, ".SRCINFO", || {
            vec![
                format!("pkgbase = {name}"),
                format!("\tpkgver = {version}"),
                format!("\tpkgrel = 0"),
                format!("\tpkgdesc = {description}"),
                format!("\turl = {homepage}"),
                format!("\tarch = x86_64"),
                format!("\tarch = i686"),
                format!("\tlicense = {license}"),
                format!("\tmakedepends = cargo"),
                format!("\tprovides = {cli_name}"),
                conflicts_srcinfo,
                format!(
                    "\tsource = {name}-{version}.zip::https://github.com/{repository}/archive/refs/tags/v{version}.zip"
                ),
                format!("\tsha256sums = {}", checksums.get(&Target::Source).unwrap()),
                format!(""),
                format!("pkgname = {name}"),
            ]
        })?;

        if !dry_run {
            commit_and_push(&sh, &name, version)?;
        }

        Ok(())
    }

    fn instructions(&self, info: &AppConfig) -> Result<Vec<String>> {
        let name = get_name(info);

        Ok(vec![
            format!("With [AUR](https://aur.archlinux.org)"),
            format!(""),
            format!("```"),
            format!("yay -S {name}"),
            format!("```"),
        ])
    }
}

pub(super) fn get_name(info: &AppConfig) -> String {
    info.aur
        .as_ref()
        .and_then(|aur| aur.name.clone())
        .unwrap_or_else(|| info.name.clone())
}
