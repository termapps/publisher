use std::{
    collections::VecDeque,
    fs::{read_to_string, write},
};

use clap::Parser;
use eyre::eyre;
use tracing::instrument;

use crate::{config::AppConfig, error::Result, repositories::build};

/// Generates installation instructions
#[derive(Debug, Parser)]
pub struct Instructions {
    /// File in which to place the instructions
    file: String,

    /// Marks the beginning of the file content to replace with instructions
    #[clap(long, default_value = "<!-- publisher install start -->")]
    start_marker: String,

    /// Marks the end of the file content to replace with instructions
    #[clap(long, default_value = "<!-- publisher install end -->")]
    end_marker: String,

    /// Prefix for each section in the installation instructions
    #[clap(long, default_value = "#### ")]
    prefix: String,
}

impl Instructions {
    #[instrument(name = "install", skip_all)]
    pub fn run(self, info: &AppConfig) -> Result {
        let AppConfig {
            name,
            repository,
            exclude,
            cargo,
            ..
        } = info;

        let mut repo_content = build(&vec![], &exclude.clone().unwrap_or_default())
            .into_iter()
            .map(|repo| repo.instructions(info).and_then(|v| Ok(v.join("\n"))))
            .collect::<Result<VecDeque<_>>>()?;

        if let Some(cargo) = cargo {
            repo_content.push_front(
                vec![
                    format!("With [Cargo](https://crates.io)"),
                    format!(""),
                    format!("```"),
                    format!("cargo install {cargo}"),
                    format!("```"),
                ]
                .join("\n"),
            );
        };

        let content = repo_content
            .into_iter()
            .map(|section| format!("{}{section}", self.prefix))
            .collect::<Vec<_>>();

        let mut file_content = read_to_string(&self.file)?;

        let start_index = file_content
            .find(&self.start_marker)
            .ok_or(eyre!("Unable to find start marker to place instructions"))?;

        let end_index = file_content
            .find(&self.end_marker)
            .ok_or(eyre!("Unable to find end marker to place instructions"))?;

        let content = vec![
            self.start_marker.clone(),
            format!("## Install"),
            format!(""),
            format!("`{name}` is available on Linux, macOS & Windows"),
            format!(""),
            content.join("\n\n"),
            format!(""),
            format!("{}Direct", self.prefix),
            format!(""),
            format!("Pre-built binary executables are available at [releases page](https://github.com/{repository}/releases)."),
            format!(""),
            format!("Download, unarchive the binary, and then put the executable in `$PATH`."),
            format!(""),
            format!(""),
        ].join("\n");

        file_content.replace_range(start_index..=end_index - 1, &content);
        write(&self.file, file_content)?;

        Ok(())
    }
}
