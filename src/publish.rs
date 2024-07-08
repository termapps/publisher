use std::{fmt::Debug, fs::write};

use crate::{
    error::Result,
    init::CONFIG_FILE,
    repositories::{
        aur::AurInfo, aur_bin::AurBinInfo, build, homebrew::HomebrewInfo, Repositories, Repository,
    },
};

use clap::Parser;
use config::{Config, Environment, File, FileFormat};
use owo_colors::OwoColorize;
use tracing::{info, instrument, warn};
use xshell::{cmd, Shell};

/// Publish the tool to package repositories
#[derive(Debug, Parser)]
pub struct Publish {
    /// Version to publish
    version: String,

    /// The name(s) of the package repository
    repositories: Vec<Repositories>,

    /// Confirm the publish action
    #[clap(long)]
    yes: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PublishInfo {
    pub name: String,
    pub description: String,
    pub repository: String,
    pub license: String,
    pub homepage: String,
    pub exclude: Option<Vec<String>>,
    pub aur: Option<AurInfo>,
    pub aur_bin: Option<AurBinInfo>,
    pub homebrew: Option<HomebrewInfo>,
}

impl Publish {
    #[instrument(name = "publish", skip_all)]
    pub fn run(self) -> Result {
        let config = read_config()?;

        let repositories = build(
            &self.repositories,
            config.exclude.as_deref().unwrap_or_default(),
        );

        for repository in repositories {
            info!("{}", repository.name().yellow());
            repository.publish(&config, &self.version, !self.yes)?;
        }

        warn!(
            "{}",
            "Not published because dry-run mode was enabled".cyan()
        );

        Ok(())
    }
}

pub fn read_config() -> Result<PublishInfo> {
    Ok(Config::builder()
        .add_source(Environment::with_prefix("PUBLISHER").separator("_"))
        .add_source(File::new(CONFIG_FILE, FileFormat::Toml))
        .build()?
        .try_deserialize::<PublishInfo>()?)
}

pub fn prepare_tmp_dir(id: &str) -> Result<(Shell, String)> {
    let sh = Shell::new()?;
    let dir = format!("/tmp/publisher/{id}");

    cmd!(sh, "rm -rf {dir}").quiet().run()?;
    cmd!(sh, "mkdir -p {dir}").quiet().run()?;

    sh.change_dir(&dir);

    Ok((sh, dir))
}

pub fn prepare_git_repo(repository: &dyn Repository, remote: &str) -> Result<(Shell, String)> {
    let id = repository.name();
    let (sh, dir) = prepare_tmp_dir(id)?;

    cmd!(sh, "git init").quiet().ignore_stdout().run()?;
    cmd!(sh, "git remote add origin {remote}")
        .quiet()
        .ignore_stdout()
        .run()?;
    cmd!(sh, "git fetch origin").quiet().ignore_stderr().run()?;

    if let Ok(_) = cmd!(sh, "git ls-remote --exit-code --heads origin master")
        .quiet()
        .ignore_stdout()
        .run()
    {
        cmd!(sh, "git checkout master")
            .quiet()
            .ignore_stdout()
            .ignore_stderr()
            .run()?;
    }

    Ok((sh, dir))
}

pub fn write_and_add<P, F>(sh: &Shell, dir: &str, path: P, writer: F) -> Result
where
    P: AsRef<str> + Debug,
    F: FnOnce() -> Vec<String>,
{
    let path = path.as_ref();

    info!("  {} {}", "writing".magenta(), path.yellow());
    let lines = writer();

    write(format!("{dir}/{path}"), format!("{}\n", lines.join("\n")))?;
    cmd!(sh, "git add {path}").quiet().run()?;

    Ok(())
}

pub fn commit_and_push(sh: &Shell, name: &str, version: &str) -> Result {
    let message = format!("{name}: {version}");

    cmd!(sh, "git commit -m {message}")
        .quiet()
        .ignore_stdout()
        .run()?;
    cmd!(sh, "git push origin master")
        .quiet()
        .ignore_stderr()
        .run()?;

    Ok(())
}
