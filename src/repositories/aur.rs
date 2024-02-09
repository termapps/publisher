use xshell::{cmd, Shell};

use crate::{
    check::{check_curl, check_git, CheckResults},
    error::Result,
    publish::{commit_and_push, prepare_git_repo, write_and_add, PublishInfo},
    repositories::Repository,
};

#[derive(Debug, Clone)]
pub(super) struct Aur;

impl Repository for Aur {
    fn name(&self) -> &'static str {
        "AUR"
    }

    fn check(&self, results: &mut CheckResults, _: &PublishInfo) -> Result {
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

        Ok(())
    }

    fn publish(&self, info: &PublishInfo, version: &str) -> Result {
        let PublishInfo {
            name,
            description,
            homepage,
            license,
            repository,
            ..
        } = &info;

        let download_url = format!(
            "https://github.com/{repository}/releases/download/v{version}/{name}-v{version}"
        );

        let (sh, dir) = prepare_git_repo(self, &format!("ssh://aur@aur.archlinux.org/{name}.git"))?;

        let x86_64_checksum = cmd!(
            sh,
            "curl -L {download_url}-x86_64-unknown-linux-gnu_sha256sum.txt"
        )
        .read()?;
        let i686_checksum = cmd!(
            sh,
            "curl -L {download_url}-i686-unknown-linux-gnu_sha256sum.txt"
        )
        .read()?;

        write_and_add(&sh, &dir, "PKGBUILD", || {
            vec![
                format!("pkgname={name}"),
                format!("pkgdesc={description:?}"),
                format!("pkgver={version}"),
                format!("pkgrel=0"),
                format!("url={homepage:?}"),
                format!("arch=('x86_64' 'i686')"),
                format!("license=({license:?})"),
                format!("provides=({name:?})"),
                format!("source_x86_64=({name}-{version}.zip::{download_url}-x86_64-unknown-linux-gnu.zip)"),
                format!("source_i686=({name}-{version}.zip::{download_url}-i686-unknown-linux-gnu.zip)"),
                format!("sha256sums_x86_64=({x86_64_checksum:?})"),
                format!("sha256sums_i686=({i686_checksum:?})"),
                format!(""),
                format!("package() {{"),
                format!("    install -Dm755 \"$srcdir/$pkgname\" \"$pkgdir/usr/bin/$pkgname\""),
                format!("    install -Dm644 \"$srcdir/LICENSE\" \"$pkgdir/usr/share/licenses/$pkgname/LICENSE\""),
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
                format!("\tprovides = {name}"),
                format!("\tsource_x86_64 = {name}-{version}.zip::{download_url}-x86_64-unknown-linux-gnu.zip"),
                format!("\tsha256sums_x86_64 = {x86_64_checksum}"),
                format!("\tsource_i686 = {name}-{version}.zip::{download_url}-i686-unknown-linux-gnu.zip"),
                format!("\tsha256sums_i686 = {i686_checksum}"),
                format!(""),
                format!("pkgname = {name}"),
            ]
        })?;

        commit_and_push(self, &sh, name, version)?;

        Ok(())
    }
}
