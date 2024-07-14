use std::fs::{read_to_string, write};

use crate::{
    error::{Error, Result},
    publish::PublishInfo,
    repositories::build,
};

use clap::Parser;
use tracing::instrument;

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
    pub fn run(self, info: &PublishInfo) -> Result {
        let PublishInfo {
            name,
            repository,
            exclude,
            ..
        } = info;

        let content = build(&vec![], &exclude.clone().unwrap_or_default())
            .into_iter()
            .map(|repo| repo.instructions(info).and_then(|v| Ok(v.join("\n"))))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|section| format!("{}{section}", self.prefix))
            .collect::<Vec<_>>()
            .join("\n\n");

        let mut file_content = read_to_string(&self.file)?;

        let start_index = file_content
            .find(&self.start_marker)
            .ok_or(Error::StartMarkerNotFound)?;

        let end_index = file_content
            .find(&self.end_marker)
            .ok_or(Error::EndMarkerNotFound)?;

        let content = vec![
            self.start_marker.clone(),
            format!("## Install"),
            format!(""),
            format!("`{name}` is available on Linux, macOS & Windows"),
            format!(""),
            content,
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
