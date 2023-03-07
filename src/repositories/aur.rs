use xshell::{cmd, Shell};

use crate::{
    check::{check_git, CheckResults},
    error::Result,
    repositories::Repository,
};

#[derive(Debug, Clone)]
pub struct Aur;

impl Repository for Aur {
    fn name(&self) -> &'static str {
        "AUR"
    }

    fn check(&self, results: &mut CheckResults) -> Result {
        let sh = Shell::new().unwrap();

        check_git(&sh, results);

        let output = cmd!(sh, "ssh aur@aur.archlinux.org")
            .quiet()
            .ignore_status()
            .read_stderr()
            .unwrap();

        results.add_result(
            "ssh",
            (!output.contains("Interactive shell is disabled."))
                .then_some("AUR SSH access is not configured"),
        );

        Ok(())
    }
}
