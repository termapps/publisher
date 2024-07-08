use xshell::{cmd, Shell};

use crate::{
    check::{check_curl, check_git, check_repo, CheckResults},
    error::Result,
    publish::{commit_and_push, prepare_git_repo, write_and_add, PublishInfo},
    repositories::Repository,
};

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AurInfo {
    pub repository: Option<String>,
    pub conflicts: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub(super) struct Aur;

impl Repository for Aur {
    fn name(&self) -> &'static str {
        "AUR"
    }

    fn check(&self, results: &mut CheckResults, info: &PublishInfo) -> Result {
        let sh = Shell::new()?;

        check_git(&sh, results);
        check_curl(&sh, results);

        let output = cmd!(sh, "ssh aur@aur.archlinux.org")
            .quiet()
            .ignore_status()
            .read_stderr()?;

        results.add_result(
            "ssh",
            (!output.contains("Interactive shell is disabled."))
                .then_some("AUR SSH access is not configured"),
        );

        let name = get_name(&info);

        check_repo(
            &sh,
            &format!("ssh://aur@aur.archlinux.org/{name}.git"),
            "master",
            results,
            true,
        )?;

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
        } = &info;

        let name = get_name(&info);
        let (sh, dir) = prepare_git_repo(self, &format!("ssh://aur@aur.archlinux.org/{name}.git"))?;

        let github_repo_name = repository.split('/').last().unwrap();
        let download_url = format!("https://github.com/{repository}");

        let checksum = cmd!(
            sh,
            "curl -L {download_url}/releases/download/v{version}/{cli_name}-v{version}_sha256sum.txt"
        )
        .ignore_stderr()
        .read()?;

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
                format!("pkgdesc={description:?}"),
                format!("pkgver={version}"),
                format!("pkgrel=0"),
                format!("url={homepage:?}"),
                format!("arch=('x86_64' 'i686')"),
                format!("license=({license:?})"),
                format!("provides=({cli_name:?})"),
                format!("conflicts=({conflicts_pkgbuild})"),
                format!("makedepends=('cargo')"),
                format!("source=($pkgname-$pkgver.zip::{download_url}/archive/refs/tags/v$pkgver.zip)"),
                format!("sha256sums=({checksum:?})"),
                format!(""),
                format!("build() {{"),
                format!("    cd {github_repo_name}-$pkgver"),
                format!("    cargo build --release --locked"),
                format!("}}"),
                format!(""),
                format!("package() {{"),
                format!("    install -Dm755 \"$srcdir/{github_repo_name}-$pkgver/target/release/{cli_name}\" \"$pkgdir/usr/bin/{cli_name}\""),
                format!("    install -Dm644 \"$srcdir/{github_repo_name}-$pkgver/LICENSE\" \"$pkgdir/usr/share/licenses/{cli_name}/LICENSE\""),
                format!("}}"),
            ]
        })?;

        write_and_add(&sh, &dir, ".SRCINFO", || {
            vec![
                format!("pkgbase = {name}"),
                format!("\tpkgdesc = {description}"),
                format!("\tpkgver = {version}"),
                format!("\tpkgrel = 0"),
                format!("\turl = {homepage}"),
                format!("\tarch = x86_64"),
                format!("\tarch = i686"),
                format!("\tlicense = {license}"),
                format!("\tprovides = {cli_name}"),
                conflicts_srcinfo,
                format!(
                    "\tsource = {name}-{version}.zip::{download_url}/archive/refs/tags/v{version}.zip"
                ),
                format!("\tsha256sums = {checksum}"),
                format!(""),
                format!("pkgname = {name}"),
            ]
        })?;

        if !dry_run {
            commit_and_push(&sh, &name, version)?;
        }

        Ok(())
    }
}

pub fn get_name(info: &PublishInfo) -> String {
    info.aur
        .as_ref()
        .and_then(|info| info.repository.clone())
        .unwrap_or_else(|| info.name.clone())
}
