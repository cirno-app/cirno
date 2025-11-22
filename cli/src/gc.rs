use anyhow::Result;
use cirno_core::Cirno;
use clap::Args;

use crate::EnvArgs;

#[derive(Debug, Args)]
pub struct Gc {}

impl EnvArgs for Gc {
    async fn main(self, cirno: Cirno) -> Result<()> {
        cirno.gc().await
    }
}
