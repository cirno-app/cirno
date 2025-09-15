use crate::AppState;
use anyhow::{Context, Error, Ok, Result};
use log::error;
use std::{
    sync::{
        Arc, RwLock, Weak,
        atomic::{AtomicU64, Ordering},
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    thread::spawn,
};
use tao::window::{Window, WindowBuilder};
use thiserror::Error;
use wry::WebViewBuilder;

#[derive(Debug, Error)]
enum WryStateRegistryError {
    #[error("Registry is full, no available IDs")]
    RegistryFull,
    #[error("Invalid ID: {0} (must be 0-63)")]
    InvalidId(u8),
    #[error("No window found for ID: {0}")]
    WindowNotFound(u8),
}

pub struct WryState {
    window: Window,
    tx: SyncSender<WvEvent>,
}

enum A {
    B,
    C,
}

pub struct WryStateRegistry {
    app_weak: Weak<AppState>,
    intl: RwLock<WryStateRegistryIntl>,
}

struct WryStateRegistryIntl {
    map: AtomicU64,
    reg: [Option<Arc<WryState>>; 64],
}

impl WryStateRegistry {
    pub fn new(app_weak: Weak<AppState>) -> Self {
        Self {
            app_weak,
            intl: RwLock::new(WryStateRegistryIntl {
                map: AtomicU64::new(0),
                reg: [(); 64].map(|_| None),
            }),
        }
    }

    pub fn create(&self) -> Result<(u8, Arc<WryState>)> {
        let mut intl = self.intl.write().unwrap();
        let bitmap = intl.map.load(Ordering::Acquire);
        let free_bit = (0..64).find(|i| (bitmap & (1 << i)) == 0);

        match free_bit {
            Some(id) => {
                let prev = intl.map.fetch_or(1 << id, Ordering::AcqRel);
                if (prev & (1 << id)) != 0 {
                    return Err(WryStateRegistryError::RegistryFull.into());
                }

                let state = self.app_weak.upgrade().unwrap().dispatcher.dispatch(
                    |event_loop| -> std::result::Result<_, Error> {
                        let window = WindowBuilder::new()
                            .build(event_loop)
                            .context("Failed to create window")?;

                        let (tx, rx) = sync_channel(0);

                        let state = Arc::new(WryState { window, tx });

                        let wv_state = state.clone();

                        spawn(|| match wv_run(wv_state, rx) {
                            std::result::Result::Ok(_) => (),
                            Err(err) => {
                                error!("{err:?}");
                            }
                        });

                        Ok(state)
                    },
                )??;

                intl.reg[id] = Some(state.clone());

                Ok((id as u8, state))
            }
            None => Err(WryStateRegistryError::RegistryFull.into()),
        }
    }

    pub fn get(&self, id: u8) -> Result<Arc<WryState>> {
        let intl = self.intl.read().unwrap();

        if id >= 64 {
            return Err(WryStateRegistryError::InvalidId(id).into());
        }

        intl.reg[id as usize]
            .as_ref()
            .map(Arc::clone)
            .ok_or(WryStateRegistryError::WindowNotFound(id).into())
    }

    pub fn destroy(&self, id: u8) -> Result<()> {
        let mut intl = self.intl.write().unwrap();

        if id >= 64 {
            return Err(WryStateRegistryError::InvalidId(id).into());
        }

        let state = intl.reg[id as usize].take();

        let Some(state) = state else {
            return Err(WryStateRegistryError::WindowNotFound(id).into());
        };

        intl.map.fetch_and(!(1 << id), Ordering::AcqRel);

        Ok(())
    }
}

enum WvEvent {}

fn wv_run(state: Arc<WryState>, rx: Receiver<WvEvent>) -> Result<()> {
    let builder = WebViewBuilder::new().with_url("https://tauri.app");

    #[cfg(not(target_os = "linux"))]
    let webview = builder
        .build(&state.window)
        .context("Failed to create webview")?;
    #[cfg(target_os = "linux")]
    let webview = builder
        .build_gtk(state.window.gtk_window())
        .context("Failed to create webview")?;

    loop {
        let a = rx.recv();
    }

    Ok(())
}
