use anyhow::Result;

pub struct ProcessDaemon {}

impl ProcessDaemon {
    pub async fn init(&self) -> Result<()> {
        Ok(())
    }

    pub fn new() -> ProcessDaemon {
        ProcessDaemon {}
    }
}
