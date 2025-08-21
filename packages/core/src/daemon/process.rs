use crate::app;
use anyhow::{Result, bail};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    spawn,
    sync::{Mutex, MutexGuard},
};

struct DaemonProc {}

struct ProcessDaemonIntl {
    reg: [Option<DaemonProc>; 256],
    name_reg: HashMap<String, u8>,
}

pub struct ProcessDaemon {
    intl: Arc<Mutex<ProcessDaemonIntl>>,
}

impl ProcessDaemon {
    pub async fn init(&self) -> Result<()> {
        let daemon_intl = self.intl.lock().await;

        Ok(())
    }

    pub fn new() -> ProcessDaemon {
        ProcessDaemon {
            intl: Arc::new(Mutex::new(ProcessDaemonIntl {
                reg: [const { None }; 256],
                name_reg: HashMap::new(),
            })),
        }
    }

    pub async fn start(&self, name: &String) -> Result<()> {
        let exists = app::exists(name).await?;

        if !exists {
            bail!("App {} does not exist", name);
        }

        let daemon_intl = self.intl.lock().await;

        self.start_intl(daemon_intl, name).await
    }

    async fn start_intl(
        &self,
        mut daemon_intl: MutexGuard<'_, ProcessDaemonIntl>,
        name: &str,
    ) -> Result<()> {
        let index = daemon_intl.get_index(name);
        let index_usize = index as usize;

        if daemon_intl.reg[index_usize].is_some() {
            bail!("Instance {} already started", name);
        }

        daemon_intl.reg[index_usize] = Some(DaemonProc {});
        let dp = &daemon_intl.reg[index_usize];

        spawn(async {
            loop {
            }

            {
            }

            {
            }
        });

        Ok(())
    }
}

impl ProcessDaemonIntl {
    fn get_index(&self, name: &str) -> u8 {
        let mut index: u8 = 0;

        for (n, i) in &self.name_reg {
            if name == n {
                return *i;
            }
            index += 1;
        }

        index
    }
}
