use xshell::{cmd, Shell};

use crate::{
    check::{check_curl, check_git, check_repo, CheckResults},
    error::Result,
    publish::{commit_and_push, prepare_git_repo, write_and_add, PublishInfo},
    repositories::Repository,
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AurBinInfo {
    pub repository: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct AurBin;

impl Repository for AurBin {
    fn name(&self) -> &'static str {
        "AUR (bin)"
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
        )?;

        Ok(())
    }

    fn publish(&self, info: &PublishInfo, version: &str) -> Result {
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

        let download_url = format!(
            "https://github.com/{repository}/releases/download/v{version}/{cli_name}-v{version}"
        );

        let x86_64_checksum = cmd!(
            sh,
            "curl -L {download_url}-x86_64-unknown-linux-gnu_sha256sum.txt"
        )
        .ignore_stderr()
        .read()?;
        let i686_checksum = cmd!(
            sh,
            "curl -L {download_url}-i686-unknown-linux-gnu_sha256sum.txt"
        )
        .ignore_stderr()
        .read()?;

        write_and_add(&sh, &dir, "PKGBUILD", || {
            vec![
                format!("pkgname={name}"),
                format!("pkgver={version}"),
                format!("pkgrel=0"),
                format!("pkgdesc={description:?}"),
                format!("arch=('x86_64' 'i686')"),
                format!("url={homepage:?}"),
                format!("license=({license:?})"),
                format!("provides=({cli_name:?})"),
                format!("source_x86_64=($pkgname-$pkgver.zip::{download_url}-x86_64-unknown-linux-gnu.zip)"),
                format!("source_i686=($pkgname-$pkgver.zip::{download_url}-i686-unknown-linux-gnu.zip)"),
                format!("sha256sums_x86_64=({x86_64_checksum:?})"),
                format!("sha256sums_i686=({i686_checksum:?})"),
                format!(""),
                format!("package() {{"),
                format!("    install -Dm755 \"$srcdir/{cli_name}\" \"$pkgdir/usr/bin/{cli_name}\""),
                format!("    install -Dm644 \"$srcdir/LICENSE\" \"$pkgdir/usr/share/licenses/{cli_name}/LICENSE\""),
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
                format!("\tsource_x86_64 = {name}-{version}.zip::{download_url}-x86_64-unknown-linux-gnu.zip"),
                format!("\tsha256sums_x86_64 = {x86_64_checksum}"),
                format!("\tsource_i686 = {name}-{version}.zip::{download_url}-i686-unknown-linux-gnu.zip"),
                format!("\tsha256sums_i686 = {i686_checksum}"),
                format!(""),
                format!("pkgname = {name}"),
            ]
        })?;

        commit_and_push(&sh, &name, version)?;

        Ok(())
    }
}

fn get_name(info: &PublishInfo) -> String {
    info.aur_bin
        .as_ref()
        .and_then(|info| info.repository.clone())
        .unwrap_or_else(|| format!("{}-bin", info.name))
}
