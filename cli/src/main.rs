use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod init;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init(init::Init),
}

#[tokio::main]
pub async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init(args) => args.main().await,
    }
}
