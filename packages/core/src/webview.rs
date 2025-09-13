use crate::AppState;
use anyhow::{Context, Error, Result};
use std::sync::{
    Arc, RwLock, Weak,
    atomic::{AtomicU64, Ordering},
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
}

pub struct WryStateRegistry {
    app_weak: Weak<AppState>,
    intl: RwLock<WryStateRegistryIntl>,
}

struct WryStateRegistryIntl {
    map: AtomicU64,
    reg: [Option<Arc<RwLock<WryState>>>; 64],
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

    pub fn create(&self) -> Result<(u8, Arc<RwLock<WryState>>)> {
        let mut intl = self.intl.write().unwrap();
        let bitmap = intl.map.load(Ordering::Acquire);
        let free_bit = (0..64).find(|i| (bitmap & (1 << i)) == 0);

        match free_bit {
            Some(id) => {
                let prev = intl.map.fetch_or(1 << id, Ordering::AcqRel);
                if (prev & (1 << id)) != 0 {
                    return Err(WryStateRegistryError::RegistryFull.into());
                }

                let window = self.app_weak.upgrade().unwrap().dispatcher.dispatch(
                    |event_loop| -> std::result::Result<_, Error> {
                        let window = WindowBuilder::new()
                            .build(event_loop)
                            .context("Failed to create window")?;

                        let builder = WebViewBuilder::new().with_url("https://tauri.app");

                        #[cfg(not(target_os = "linux"))]
                        let webview = builder.build(&window).context("Failed to create webview")?;
                        #[cfg(target_os = "linux")]
                        let webview = builder
                            .build_gtk(window.gtk_window())
                            .context("Failed to create webview")?;

                        Box::leak(Box::new(webview));

                        // Ok((window, webview))
                        Ok(window)
                    },
                )??;

                let arc = Arc::new(RwLock::new(WryState { window }));
                intl.reg[id] = Some(arc.clone());

                Ok((id as u8, arc))
            }
            None => Err(WryStateRegistryError::RegistryFull.into()),
        }
    }

    pub fn get(&self, id: u8) -> Result<Arc<RwLock<WryState>>, WryStateRegistryError> {
        let intl = self.intl.read().unwrap();

        if id >= 64 {
            return Err(WryStateRegistryError::InvalidId(id));
        }

        intl.reg[id as usize]
            .as_ref()
            .map(Arc::clone)
            .ok_or(WryStateRegistryError::WindowNotFound(id))
    }

    pub fn destroy(&self, id: u8) -> Result<(), WryStateRegistryError> {
        let mut intl = self.intl.write().unwrap();

        if id >= 64 {
            return Err(WryStateRegistryError::InvalidId(id));
        }

        let state = intl.reg[id as usize].take();

        let Some(state) = state else {
            return Err(WryStateRegistryError::WindowNotFound(id));
        };

        intl.map.fetch_and(!(1 << id), Ordering::AcqRel);

        Ok(())
    }
}
