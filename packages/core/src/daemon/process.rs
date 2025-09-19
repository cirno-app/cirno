use crate::{AppState, app, proc::CirnoProc};
use anyhow::{Context, Result, bail};
use log::warn;
use std::process::ExitStatusError;
use std::sync::Weak;
use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::{Arc, LazyLock},
};
use thiserror::Error;
use tokio::{
    spawn,
    sync::{Mutex, MutexGuard},
};

static ARG_START: LazyLock<Vec<&OsStr>> = LazyLock::new(|| vec![OsStr::new("start")]);

pub struct AppProc {}

struct ProcessDaemonIntl {
    reg: [Option<Arc<AppProc>>; 256],
    name_reg: HashMap<String, u8>,
}

pub struct ProcessDaemon {
    app_weak: Weak<AppState>,
    intl: Arc<Mutex<ProcessDaemonIntl>>,
}

impl ProcessDaemon {
    pub async fn init(&self) -> Result<()> {

        let daemon_intl = self.intl.lock().await;

        Ok(())
    }

    pub fn new(app_weak: Weak<AppState>) -> ProcessDaemon {
        ProcessDaemon {
            app_weak,
            intl: Arc::new(Mutex::new(ProcessDaemonIntl {
                reg: [const { None }; 256],
                name_reg: HashMap::new(),
            })),
        }
    }

    pub async fn start(&self, name: &String) -> Result<()> {
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
        &self,
        mut daemon_intl: MutexGuard<'_, ProcessDaemonIntl>,
        name: &str,
    ) -> Result<()> {
        let index = daemon_intl.get_index(name)?;
        let index_usize = index as usize;

        if daemon_intl.reg[index_usize].is_some() {
            bail!("Instance {} already started", name);
        }

        daemon_intl.reg[index_usize] = Some(Arc::new(AppProc {}));
        let app_proc = &daemon_intl.reg[index_usize];

        let name = name.to_owned();

        let app = self.app_weak.upgrade().unwrap();

        spawn(async move {
            loop {
                let mut cp =
                    CirnoProc::new_yarn(&app.env, &ARG_START, app.env.apps_dir.join(name.clone()));

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
                // This is the only place to drop AppProc; only when the process ends should AppProc be cleaned up.
                // This will actually drop app_proc instantly
                let app = app.clone();
                let mut daemon_intl = app.process_daemon.intl.lock().await;
                daemon_intl.reg[index_usize] = None;
            }
        });

        Ok(())
    }

    pub async fn claim(&self, token: &str) -> Option<Arc<AppProc>> {
        let index: u8 = match token.parse() {
            Ok(index) => index,
            Err(_) => {
                return None;
            }
        };

        self.intl.lock().await.reg[index as usize].clone()
    }
}

impl ProcessDaemonIntl {
    fn get_index(&self, name: &str) -> core::result::Result<u8, ProcessDaemonError> {
        let mut index: u8 = 0;

        for (n, i) in &self.name_reg {
            if name == n {
                return core::result::Result::Ok(*i);
            }
            index += 1;
        }

        if self.name_reg.len() > 256 {
            return core::result::Result::Err(ProcessDaemonError::RegistryFull);
        }

        core::result::Result::Ok(index)
    }
}

#[derive(Error, Debug)]
enum ProcessDaemonError {
    #[error("process daemon registry is full")]
    RegistryFull,
}
