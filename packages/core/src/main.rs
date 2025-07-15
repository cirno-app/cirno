use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunArgs),

    Start(StartArgs),

    Stop(StopArgs),
}

#[derive(Args)]
struct RunArgs {}

#[derive(Args)]
struct StartArgs {}

#[derive(Args)]
struct StopArgs {}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run(_args) => {
        }

        Commands::Start(_args) => {
        }

        Commands::Stop(_args) => {
        }
    }

    Ok(())
}
