use xshell::{cmd, Shell};

use super::{get_checksums, Repository};
use crate::{
    check::{check_curl, check_git, check_repo, CheckResults},
    error::Result,
    publish::{commit_and_push, prepare_git_repo, write_and_add, PublishInfo},
    targets::Target,
};

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AurBinInfo {
    pub name: Option<String>,
    pub conflicts: Option<Vec<String>>,
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

        let ssh_configured = cmd!(sh, "ssh aur@aur.archlinux.org")
            .quiet()
            .ignore_status()
            .read_stderr()?
            .contains("Interactive shell is disabled.");

        results.add_result(
            "ssh",
            (!ssh_configured).then_some("AUR SSH access is not configured"),
        );

        let name = get_name(&info);

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

    fn publish(&self, info: &PublishInfo, version: &str, dry_run: bool) -> Result {
        let PublishInfo {
            name: cli_name,
            description,
            homepage,
            license,
            repository,
            ..
        } = info;

        let name = get_name(info);
        let (sh, dir) = prepare_git_repo(self, &format!("ssh://aur@aur.archlinux.org/{name}.git"))?;

        let checksums = get_checksums(
            info,
            version,
            vec![Target::X86_64UnknownLinuxGnu, Target::I686UnknownLinuxGnu],
        )?;

        let conflicts = info
            .aur_bin
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
                format!("provides=({cli_name:?})"),
                format!("conflicts=({conflicts_pkgbuild})"),
                format!("source_x86_64=($pkgname-$pkgver.zip::https://github.com/{repository}/releases/download/v$pkgver/{cli_name}-v$pkgver-x86_64-unknown-linux-gnu.zip)"),
                format!("source_i686=($pkgname-$pkgver.zip::https://github.com/{repository}/releases/download/v$pkgver/{cli_name}-v$pkgver-i686-unknown-linux-gnu.zip)"),
                format!("sha256sums_x86_64=({:?})", checksums.get(&Target::X86_64UnknownLinuxGnu).unwrap()),
                format!("sha256sums_i686=({:?})", checksums.get(&Target::I686UnknownLinuxGnu).unwrap()),
                format!(""),
                format!("package() {{"),
                format!("    cd \"$srcdir\""),
                format!("    install -Dm755 \"{cli_name}\" \"$pkgdir/usr/bin/{cli_name}\""),
                format!("    install -Dm644 \"LICENSE\" \"$pkgdir/usr/share/licenses/{cli_name}/LICENSE\""),
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
                format!("\tprovides = {cli_name}"),
                conflicts_srcinfo,
                format!("\tsource_x86_64 = {name}-{version}.zip::https://github.com/{repository}/releases/download/v{version}/{cli_name}-v{version}-x86_64-unknown-linux-gnu.zip"),
                format!("\tsha256sums_x86_64 = {}", checksums.get(&Target::X86_64UnknownLinuxGnu).unwrap()),
                format!("\tsource_i686 = {name}-{version}.zip::https://github.com/{repository}/releases/download/v{version}/{cli_name}-v{version}-i686-unknown-linux-gnu.zip"),
                format!("\tsha256sums_i686 = {}", checksums.get(&Target::I686UnknownLinuxGnu).unwrap()),
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

pub(super) fn get_name(info: &PublishInfo) -> String {
    info.aur_bin
        .as_ref()
        .and_then(|info| info.name.clone())
        .unwrap_or_else(|| format!("{}-bin", info.name))
}
