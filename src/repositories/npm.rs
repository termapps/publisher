use serde::{Deserialize, Serialize};
use xshell::{Shell, cmd};

use crate::{
    check::{CheckResults, check_program},
    config::AppConfig,
    error::Result,
    publish::{download_binary, prepare_tmp_dir, write_file},
    repositories::Repository,
    targets::Target,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NPMConfig {
    pub name: Option<String>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub(super) struct NPM;

impl Repository for NPM {
    fn name(&self) -> &'static str {
        "NPM"
    }

    fn check(&self, results: &mut CheckResults, _: &AppConfig) -> Result {
        let sh = Shell::new()?;

        check_program(&sh, results, "curl", "curl --version", "curl ");
        check_program(&sh, results, "unzip", "unzip -v", "UnZip ");
        check_program(&sh, results, "npm", "npm --version", "");

        // TODO: Check if all packages can be published to

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

        let (sh, dir) = prepare_tmp_dir(self)?;

        write_file(&dir, "main/package.json", || {
            vec![
                format!("{{"),
                format!("  \"name\": {name:?},"),
                format!("  \"version\": {version:?},"),
                format!("  \"description\": {description:?},"),
                format!("  \"homepage\": {homepage:?},"),
                format!("  \"license\": {license:?},"),
                format!("  \"repository\": {{"),
                format!("    \"type\": \"git\","),
                format!("    \"url\": \"git+https://github.com/{repository}.git\""),
                format!("  }},"),
                format!("  \"bin\": {{"),
                format!("    {cli_name:?}: \"cli.js\""),
                format!("  }},"),
                format!("  \"scripts\": {{"),
                format!("    \"postinstall\": \"node ./install.js\""),
                format!("  }},"),
                format!("  \"dependencies\": {{"),
                format!("    \"detect-libc\": \"^2.0.4\""),
                format!("  }},"),
                format!("  \"optionalDependencies\": {{"),
                format!("    \"{name}-darwin-arm64\": {version:?},"),
                format!("    \"{name}-darwin-x64\": {version:?},"),
                format!("    \"{name}-linux-x64-glibc\": {version:?},"),
                format!("    \"{name}-linux-ia32-glibc\": {version:?},"),
                format!("    \"{name}-linux-x64-musl\": {version:?},"),
                format!("    \"{name}-windows-x64\": {version:?},"),
                format!("    \"{name}-windows-ia32\": {version:?}"),
                format!("  }},"),
                format!("  \"publishConfig\": {{"),
                format!("    \"access\": \"public\""),
                format!("  }}"),
                format!("}}"),
            ]
        })?;

        write_file(&dir, "main/constants.js", || {
            vec![
                format!("const BINARY_DISTRIBUTION_PACKAGES = {{"),
                format!("  \"darwin-arm64\": \"{name}-darwin-arm64\","),
                format!("  \"darwin-x64\": \"{name}-darwin-x64\","),
                format!("  \"linux-x64-glibc\": \"{name}-linux-x64-glibc\","),
                format!("  \"linux-ia32-glibc\": \"{name}-linux-ia32-glibc\","),
                format!("  \"linux-x64-musl\": \"{name}-linux-x64-musl\","),
                format!("  \"win32-x64\": \"{name}-windows-x64\","),
                format!("  \"win32-ia32\": \"{name}-windows-ia32\","),
                format!("}};"),
                format!(""),
                format!("const BINARY_DISTRIBUTION_VERSION = {version:?};"),
                format!(""),
                format!(
                    "const BINARY_NAME = [\"win32\", \"cygwin\"].includes(process.platform) ? \"{cli_name}.exe\" : {cli_name:?};"
                ),
                format!(""),
                format!("module.exports = {{"),
                format!("  BINARY_DISTRIBUTION_PACKAGES,"),
                format!("  BINARY_DISTRIBUTION_VERSION,"),
                format!("  BINARY_NAME,"),
                format!("}};"),
            ]
        })?;

        write_file(&dir, "main/install.js", || {
            vec![include_str!("../templates/npm/install.js").into()]
        })?;

        write_file(&dir, "main/binary.js", || {
            vec![include_str!("../templates/npm/binary.js").into()]
        })?;

        write_file(&dir, "main/cli.js", || {
            vec![include_str!("../templates/npm/cli.js").into()]
        })?;

        let write_and_publish = |target: Target,
                                 os: &str,
                                 cpu: &str,
                                 libc: Option<&str>|
         -> Result {
            let mut suffix = format!("{os}-{cpu}");
            let mut libc_lines = String::new();

            if let Some(libc) = libc {
                suffix = format!("{suffix}-{libc}");
                libc_lines = format!("  \"libc\": [{libc:?}],",);
            }

            write_file(&dir, format!("{suffix}/package.json"), || {
                vec![
                    format!("{{"),
                    format!("  \"name\": \"{name}-{suffix}\","),
                    format!("  \"version\": {version:?},"),
                    format!("  \"description\": {description:?},"),
                    format!("  \"homepage\": {homepage:?},"),
                    format!("  \"license\": {license:?},"),
                    format!("  \"repository\": {{"),
                    format!("    \"type\": \"git\","),
                    format!("    \"url\": \"git+https://github.com/{repository}.git\""),
                    format!("  }},"),
                    format!("  \"os\": [{os:?}],"),
                    format!("  \"cpu\": [{cpu:?}],"),
                    libc_lines,
                    format!("  \"publishConfig\": {{"),
                    format!("    \"access\": \"public\""),
                    format!("  }}"),
                    format!("}}"),
                ]
            })?;

            download_binary(
                &sh,
                &dir,
                format!("{suffix}/bin"),
                &format!(
                    "https://github.com/{repository}/releases/download/v{version}/{cli_name}-v{version}-{target}.zip"
                ),
            )?;

            if !dry_run {
                sh.change_dir(format!("../{suffix}"));
                cmd!(sh, "npm publish")
                    .quiet()
                    .ignore_stderr()
                    .ignore_stdout()
                    .run()?;
            }

            Ok(())
        };

        if !dry_run {
            sh.change_dir("main");
            cmd!(sh, "npm publish")
                .quiet()
                .ignore_stderr()
                .ignore_stdout()
                .run()?;
        }

        write_and_publish(Target::Aarch64AppleDarwin, "darwin", "arm64", None)?;
        write_and_publish(Target::X86_64AppleDarwin, "darwin", "x64", None)?;
        write_and_publish(Target::X86_64UnknownLinuxGnu, "linux", "x64", Some("glibc"))?;
        write_and_publish(Target::I686UnknownLinuxGnu, "linux", "ia32", Some("glibc"))?;
        write_and_publish(Target::X86_64UnknownLinuxMusl, "linux", "x64", Some("musl"))?;
        write_and_publish(Target::X86_64PcWindowsMsvc, "windows", "x64", None)?;
        write_and_publish(Target::I686PcWindowsMsvc, "windows", "ia32", None)?;

        Ok(())
    }

    fn instructions(&self, info: &AppConfig) -> Result<Vec<String>> {
        let name = get_name(info);

        Ok(vec![
            format!("With [NPM](https://npmjs.com)"),
            format!(""),
            format!("```"),
            format!("npm install -g {}", name),
            format!("```"),
        ])
    }
}

fn get_name(info: &AppConfig) -> String {
    info.npm
        .as_ref()
        .and_then(|npm| npm.name.clone())
        .unwrap_or_else(|| info.name.clone())
}
