use std::{fmt::Write, fs::write};

use tracing::info;
use xshell::{cmd, Shell};

use crate::{
    check::{check_curl, check_git, CheckResults},
    error::Result,
    publish::{commit_and_push, prepare_git_repo, PublishInfo},
    repositories::Repository,
};

#[derive(Debug, Clone)]
pub struct Aur;

impl Repository for Aur {
    fn name(&self) -> &'static str {
        "AUR"
    }

    fn check(&self, results: &mut CheckResults) -> Result {
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

        info!("Writing PKGBUILD");
        let mut pkgbuild = String::new();

        writeln!(pkgbuild, "pkgname={name}")?;
        writeln!(pkgbuild, "pkgdesc={description:?}")?;
        writeln!(pkgbuild, "pkgver={version}")?;
        writeln!(pkgbuild, "pkgrel=0")?;
        writeln!(pkgbuild, "url={homepage:?}")?;
        writeln!(pkgbuild, "arch=('x86_64' 'i686')")?;
        writeln!(pkgbuild, "license=({license:?})")?;
        writeln!(pkgbuild, "provides=({name:?})")?;
        writeln!(
            pkgbuild,
            "source_x86_64=({name}-{version}.zip::{download_url}-x86_64-unknown-linux-gnu.zip)"
        )?;
        writeln!(
            pkgbuild,
            "source_i686=({name}-{version}.zip::{download_url}-i686-unknown-linux-gnu.zip)"
        )?;
        writeln!(pkgbuild, "sha256sums_x86_64=({x86_64_checksum:?})")?;
        writeln!(pkgbuild, "sha256sums_i686=({i686_checksum:?})")?;
        writeln!(pkgbuild, "")?;
        writeln!(pkgbuild, "package() {{")?;
        writeln!(
            pkgbuild,
            "    install -Dm755 \"$srcdir/$pkgname\" \"$pkgdir/usr/bin/$pkgname\""
        )?;
        writeln!(pkgbuild, "    install -Dm644 \"$srcdir/LICENSE\" \"$pkgdir/usr/share/licenses/$pkgname/LICENSE\"")?;
        writeln!(pkgbuild, "}}")?;

        write(format!("{dir}/PKGBUILD"), pkgbuild)?;

        info!("Writing SRCINFO");
        let mut srcinfo = String::new();

        writeln!(srcinfo, "pkgbase = {name}")?;
        writeln!(srcinfo, "\tpkgdesc = {description}")?;
        writeln!(srcinfo, "\tpkgver = {version}")?;
        writeln!(srcinfo, "\tpkgrel = 0")?;
        writeln!(srcinfo, "\turl = {homepage}")?;
        writeln!(srcinfo, "\tarch = x86_64")?;
        writeln!(srcinfo, "\tarch = i686")?;
        writeln!(srcinfo, "\tlicense = {license}")?;
        writeln!(srcinfo, "\tprovides = {name}")?;
        writeln!(
            srcinfo,
            "\tsource_x86_64 = {name}-{version}.zip::{download_url}-x86_64-unknown-linux-gnu.zip"
        )?;
        writeln!(srcinfo, "\tsha256sums_x86_64 = {x86_64_checksum}")?;
        writeln!(
            srcinfo,
            "\tsource_i686 = {name}-{version}.zip::{download_url}-i686-unknown-linux-gnu.zip"
        )?;
        writeln!(srcinfo, "\tsha256sums_i686 = {i686_checksum}")?;
        writeln!(srcinfo, "")?;
        writeln!(srcinfo, "pkgname = {name}")?;

        write(format!("{dir}/.SRCINFO"), srcinfo)?;

        cmd!(sh, "git add PKGBUILD .SRCINFO").run()?;

        commit_and_push(self, &sh, version)?;

        Ok(())
    }
}
