use serde::{Deserialize, Serialize};
use xshell::{cmd, Shell};

use super::get_checksums;
use crate::{
    check::{check_curl, check_git, check_nix, check_repo, CheckResults},
    config::AppConfig,
    error::Result,
    publish::{commit_and_push, prepare_git_repo, write_and_add},
    repositories::Repository,
    targets::Target,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NixConfig {
    pub name: Option<String>,
    pub repository: Option<String>,
    pub path: Option<String>,
    pub lockfile: Option<bool>,
}

#[derive(Debug, Clone)]
pub(super) struct Nix;

impl Repository for Nix {
    fn name(&self) -> &'static str {
        "Nix"
    }

    fn check(&self, results: &mut CheckResults, info: &AppConfig) -> Result {
        let sh = Shell::new()?;

        check_git(&sh, results);
        check_curl(&sh, results);

        let repository = get_repository(info);

        check_repo(
            &sh,
            &format!("git@github.com:{repository}"),
            "master",
            results,
            false,
        )?;

        let lockfile = get_lockfile(info);

        if lockfile {
            check_nix(&sh, results);
        }

        Ok(())
    }

    fn publish(&self, info: &AppConfig, version: &str, dry_run: bool) -> Result {
        let AppConfig {
            name: cli_name,
            description,
            homepage,
            repository,
            ..
        } = info;

        let name = get_name(info);
        let pkg_repo = get_repository(info);
        let path = get_path(info, &name);
        let lockfile = get_lockfile(info);
        let (sh, dir) = prepare_git_repo(self, &format!("git@github.com:{}", pkg_repo))?;

        let checksums = get_checksums(
            info,
            version,
            vec![
                Target::Aarch64AppleDarwin,
                Target::X86_64AppleDarwin,
                Target::X86_64UnknownLinuxGnu,
                Target::I686UnknownLinuxGnu,
            ],
        )?;

        write_and_add(&sh, &dir, path, || {
            vec![
                format!("{{"),
                format!("  description = {description:?};"),
                format!(""),
                format!("  inputs = {{"),
                format!("    nixpkgs.url = \"github:NixOS/nixpkgs\";"),
                format!("    flake-utils.url = \"github:numtide/flake-utils\";"),
                format!("  }};"),
                format!(""),
                format!("  outputs = {{ self, nixpkgs, flake-utils }}:"),
                format!("    with flake-utils.lib;"),
                format!("    with nixpkgs.lib;"),
                format!(""),
                format!("    let"),
                format!("      systems = {{"),
                format!("        aarch64-darwin = {{"),
                format!("          target = \"aarch64-apple-darwin\";"),
                format!("          sha256 = {:?};", checksums.get(&Target::Aarch64AppleDarwin).unwrap()),
                format!("        }};"),
                format!("        x86_64-darwin = {{"),
                format!("          target = \"x86_64-apple-darwin\";"),
                format!("          sha256 = {:?};", checksums.get(&Target::X86_64AppleDarwin).unwrap()),
                format!("        }};"),
                format!("        x86_64-linux = {{"),
                format!("          target = \"x86_64-unknown-linux-gnu\";"),
                format!("          sha256 = {:?};", checksums.get(&Target::X86_64UnknownLinuxGnu).unwrap()),
                format!("        }};"),
                format!("        i686-linux = {{"),
                format!("          target = \"i686-unknown-linux-gnu\";"),
                format!("          sha256 = {:?};", checksums.get(&Target::I686UnknownLinuxGnu).unwrap()),
                format!("        }};"),
                format!("      }};"),
                format!("    in eachSystem (mapAttrsToList (n: v: n) systems) (system: {{"),
                format!("      packages.default = with import nixpkgs {{ inherit system; }};"),
                format!(""),
                format!("        stdenv.mkDerivation rec {{"),
                format!("          name = \"{name}-${{version}}\";"),
                format!("          version = {version:?};"),
                format!(""),
                format!("          nativeBuildInputs = [ unzip ];"),
                format!(""),
                format!("          src = pkgs.fetchurl {{"),
                format!("            url = \"https://github.com/{repository}/releases/download/v${{version}}/{cli_name}-v${{version}}-${{systems.${{system}}.target}}.zip\";"),
                format!("            inherit (systems.${{system}}) sha256;"),
                format!("          }};"),
                format!(""),
                format!("          sourceRoot = \".\";"),
                format!(""),
                format!("          installPhase = ''"),
                format!("            install -Dm755 {cli_name} $out/bin/{cli_name}"),
                format!("            install -Dm755 LICENSE $out/share/licenses/{cli_name}/LICENSE"),
                format!("          '';"),
                format!(""),
                format!("          meta = {{"),
                format!("            description = {description:?};"),
                format!("            homepage = {homepage:?};"),
                format!("            platforms = [ system ];"),
                format!("          }};"),
                format!("        }};"),
                format!("    }});"),
                format!("}}"),
            ]
        })?;

        if lockfile {
            cmd!(
                sh,
                "nix --extra-experimental-features 'nix-command flakes' flake update"
            )
            .run()?;

            cmd!(sh, "git add flake.lock").quiet().run()?;
        }

        if !dry_run {
            commit_and_push(&sh, &name, version)?;
        }

        Ok(())
    }

    fn instructions(&self, info: &AppConfig) -> Result<Vec<String>> {
        let name = get_name(info);
        let repository = get_repository(info);
        let path = get_path(info, &name);

        let mut contents = format!("nix profile install github:{}", repository);

        if path != "flake.nix" {
            contents = format!("{contents}#{name}")
        }

        Ok(vec![
            format!("With [Nix](https://nixos.org)"),
            format!(""),
            format!("```"),
            contents,
            format!("```"),
        ])
    }
}

fn get_name(info: &AppConfig) -> String {
    info.nix
        .as_ref()
        .and_then(|nix| nix.name.clone())
        .unwrap_or_else(|| info.name.clone())
}

fn get_repository(info: &AppConfig) -> String {
    info.nix
        .as_ref()
        .and_then(|nix| nix.repository.clone())
        .unwrap_or_else(|| info.repository.clone())
}

fn get_path(info: &AppConfig, name: &String) -> String {
    info.nix
        .as_ref()
        .and_then(|nix| nix.path.clone())
        .map(|path| path.replace("%n", name))
        .unwrap_or_else(|| "flake.nix".into())
}

fn get_lockfile(info: &AppConfig) -> bool {
    info.nix
        .as_ref()
        .and_then(|nix| nix.lockfile)
        .unwrap_or(true)
}
