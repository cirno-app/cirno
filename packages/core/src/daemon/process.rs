use crate::{AppState, app, proc::CirnoProc};
use anyhow::{Context, Result, bail};
use log::warn;
use std::process::ExitStatusError;
use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::{Arc, LazyLock},
};
use tokio::{
    spawn,
    sync::{Mutex, MutexGuard},
};

static ARG_START: LazyLock<Vec<&OsStr>> = LazyLock::new(|| vec![OsStr::new("start")]);

struct DaemonProc {}

struct ProcessDaemonIntl {
    reg: [Option<DaemonProc>; 256],
    name_reg: HashMap<String, u8>,
}

pub struct ProcessDaemon {
    app: Arc<AppState>,
    intl: Arc<Mutex<ProcessDaemonIntl>>,
}

impl ProcessDaemon {
    pub async fn init(&'static self) -> Result<()> {
        let daemon_intl = self.intl.lock().await;

        Ok(())
    }

    pub fn new(app: Arc<AppState>) -> ProcessDaemon {
        ProcessDaemon {
            app,
            intl: Arc::new(Mutex::new(ProcessDaemonIntl {
                reg: [const { None }; 256],
                name_reg: HashMap::new(),
            })),
        }
    }

    pub async fn start(&'static self, name: &String) -> Result<()> {
        let exists = app::exists(name)
            .await
            .context("Failed to check for existence")?;

        if !exists {
            bail!("App {} does not exist", name);
        }

        let daemon_intl = self.intl.lock().await;

        self.start_intl(daemon_intl, name).await
    }

    async fn start_intl(
        &'static self,
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

        let name = name.to_owned();

        spawn(async move {
            loop {
                let mut cp = CirnoProc::new_yarn(
                    &self.app.env,
                    &ARG_START,
                    self.app.env.apps_dir.join(name.clone()),
                );

                let result = cp.run().await;

                match result {
                    Err(err) => match err.downcast_ref::<ExitStatusError>() {
                        Some(exit_err) => {
                            if let Some(exit_code) = exit_err.code() {
                                if exit_code == 52 {
                                    warn!("App {} exited with code restart (52). Restarting", name);
                                } else {
                                    warn!("App {} exited with code {}", name, exit_code);
                                    break;
                                }
                            } else {
                                warn!("App {} exited with error {}", name, err);
                                break;
                            }
                        }
                        None => {
                            warn!("App {} exited with error {}", name, err);
                            break;
                        }
                    },
                    Ok(_) => {
                        warn!("App {} exited", name);
                        break;
                    }
                }
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
