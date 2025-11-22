use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use cirno_core::{Cirno, OpenError};
use clap::{Args, Parser, Subcommand};
use owo_colors::OwoColorize;

mod gc;
mod init;
mod list;

#[derive(Debug, Subcommand)]
enum Commands {
    Init(init::Init),
    #[command(alias = "prune")]
    Gc(EnvCommand<gc::Gc>),
    #[command(alias = "ls", alias = "tree")]
    List(EnvCommand<list::List>),
}

#[derive(Debug, Args)]
struct EnvCommand<T: EnvArgs> {
    #[command(flatten)]
    inner: T,
    #[arg(long, default_value = ".")]
    pub cwd: PathBuf,
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

impl<T: EnvArgs> EnvCommand<T> {
    async fn main(self) -> ExitCode {
        let cirno = match Cirno::open(&self.cwd).await {
            Ok(cirno) => cirno,
            Err(OpenError::Empty) => {
                println!(
                    "{:>12} No Cirno environment found at the specified directory.",
                    "Error".bold().bright_red()
                );
                return ExitCode::FAILURE;
            }
            Err(OpenError::Version(version)) => {
                println!(
                    "{:>12} Incompatible Cirno manifest version: {}.",
                    "Error".bold().bright_red(),
                    version
                );
                return ExitCode::FAILURE;
            }
            Err(OpenError::Other(error)) => {
                if self.verbose {
                    println!(
                        "{:>12} Failed to open cirno environment: {:?}",
                        "Error".bold().bright_red(),
                        error
                    );
                } else {
                    println!(
                        "{:>12} Failed to open cirno environment: {}\n{} Use --verbose for more details.",
                        "Error".bold().bright_red(),
                        error,
                        " ".repeat(12)
                    );
                }
                return ExitCode::FAILURE;
            }
        };
        match self.inner.main(cirno).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                if self.verbose {
                    println!("{:>12} Operation failed: {:?}", "Error".bold().bright_red(), error);
                } else {
                    println!(
                        "{:>12} Operation failed: {}\n{} Use --verbose for more details.",
                        "Error".bold().bright_red(),
                        error,
                        " ".repeat(12)
                    );
                }
                ExitCode::FAILURE
            }
        }
    }
}

trait EnvArgs: Args {
    async fn main(self, cirno: Cirno) -> Result<()>;
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    async fn main(self) -> ExitCode {
        match self.command {
            Commands::Init(args) => args.main().await,
            Commands::Gc(args) => args.main().await,
            Commands::List(args) => args.main().await,
        }
    }
}

#[tokio::main]
pub async fn main() -> ExitCode {
    Cli::parse().main().await
}
