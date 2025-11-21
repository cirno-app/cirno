use std::path::PathBuf;
use std::process::ExitCode;

use cirno_core::{Cirno, InitError};
use clap::Args;
use owo_colors::OwoColorize;

#[derive(Debug, Args)]
pub struct Init {
    #[arg(long, default_value = ".")]
    pub cwd: PathBuf,
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
}

impl Init {
    pub async fn main(self) -> ExitCode {
        match Cirno::init(&self.cwd, self.force).await {
            Ok(cwd) => {
                println!(
                    "{:>12} Cirno environment initialized at {}.",
                    "Success".bold().bright_green(),
                    cwd.display()
                );
                ExitCode::SUCCESS
            }
            Err(InitError::NotEmpty) => {
                println!(
                    "{:>12} Target directory is not empty. Use `cirno init -f` to overwrite.",
                    "Error".bold().bright_red()
                );
                ExitCode::FAILURE
            }
            Err(InitError::Other(error)) => {
                println!(
                    "{:>12} Failed to initialize Cirno environment: {}",
                    "Error".bold().bright_red(),
                    error
                );
                ExitCode::FAILURE
            }
        }
    }
}
