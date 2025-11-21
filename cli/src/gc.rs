use anyhow::Result;
use cirno_core::Cirno;
use clap::Args;

#[derive(Debug, Args)]
pub struct Gc {}

impl Gc {
    pub async fn main(self, cirno: Cirno) -> Result<()> {
        // let cirno = Cirno::open(self.cwd).await.unwrap();
        cirno.gc().await
    }
}
