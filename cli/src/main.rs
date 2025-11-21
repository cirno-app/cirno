use std::path::PathBuf;
use std::process::ExitCode;

use cirno_core::{Cirno, OpenError};
use clap::{Args, Parser, Subcommand};
use owo_colors::OwoColorize;

mod gc;
mod init;

#[derive(Debug, Subcommand)]
enum Commands {
    Init(init::Init),
    Gc(gc::Gc),
}

#[derive(Debug, Args)]
struct SharedArgs {
    #[arg(long, default_value = ".")]
    pub cwd: PathBuf,
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[command(flatten)]
    shared_args: SharedArgs,
}

impl Cli {
    async fn main(self) -> ExitCode {
        let result = match self.command {
            Commands::Init(args) => return args.main().await,
            Commands::Gc(args) => match Cirno::open(self.shared_args.cwd).await {
                Ok(cirno) => args.main(cirno).await,
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
                Err(OpenError::Other(error)) => Err(error),
            },
        };
        match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                if self.shared_args.verbose {
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

#[tokio::main]
pub async fn main() -> ExitCode {
    Cli::parse().main().await
}
