use anyhow::Result;
use cirno_core::Cirno;
use clap::Args;

use crate::EnvArgs;

#[derive(Debug, Args)]
pub struct List {
    #[clap(long, help = "Output in JSON format")]
    json: bool,
}

impl EnvArgs for List {
    async fn main(self, cirno: Cirno) -> Result<()> {
        if self.json {
            let json = serde_json::to_string(&cirno.manifest.apps)?;
            println!("{json}");
            return Ok(());
        }
        if cirno.manifest.apps.is_empty() {
            println!("No applications found.");
            return Ok(());
        }
        let len = cirno.manifest.apps.len();
        println!("Found {len} applications:");
        for (i, app) in cirno.manifest.apps.iter().enumerate() {
            let prefix = if i == len - 1 { "└" } else { "├" };
            println!("{}── {}\t{}", prefix, app.id, app.name);
            for (j, backup) in app.backups.iter().enumerate() {
                let prefix = if j == app.backups.len() - 1 { "└" } else { "├" };
                println!("    {}── {}", prefix, backup.id);
            }
        }
        Ok(())
    }
}
