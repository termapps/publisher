use std::{fmt::Write, fs::write};

use xshell::{cmd, Shell};

use crate::{
    check::{check_curl, check_git, CheckResults},
    error::Result,
    publish::PublishInfo,
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
        let sh = Shell::new()?;

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

        cmd!(sh, "rm -rf /tmp/aur").run()?;
        cmd!(sh, "mkdir -p /tmp/aur").run()?;
        sh.change_dir("/tmp/aur");

        cmd!(sh, "git init").run()?;
        cmd!(
            sh,
            "git remote add aur ssh://aur@aur.archlinux.org/{name}.git"
        )
        .run()?;
        cmd!(sh, "git fetch aur").run()?;
        cmd!(sh, "git checkout master").run()?;

        let mut pkgbuild = String::new();

        writeln!(pkgbuild, "pkgname={name}")?;
        writeln!(pkgbuild, "pkgver={version}")?;
        writeln!(pkgbuild, "pkgrel=0")?;
        writeln!(pkgbuild, "pkgdesc={description:?}")?;
        writeln!(pkgbuild, "arch=('x86_64' 'i686')")?;
        writeln!(pkgbuild, "url={homepage:?}")?;
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

        write("/tmp/aur/PKGBUILD", pkgbuild)?;

        let srcinfo = cmd!(sh, "makepkg --printsrcinfo").read()?;
        write("/tmp/aur/.SRCINFO", srcinfo)?;

        cmd!(sh, "git add PKGBUILD .SRCINFO").run()?;
        cmd!(sh, "git commit -m 'Release '{version}").run()?;
        cmd!(sh, "git push aur master").run()?;

        Ok(())
    }
}
