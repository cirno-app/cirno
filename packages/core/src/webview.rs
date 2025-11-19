use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, RwLock, Weak};

use anyhow::{Context, Error, Ok, Result};
use tao::window::{Window, WindowBuilder};
use thiserror::Error;
use wry::{WebView, WebViewBuilder};

use crate::AppState;

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
    pub id: u8,
    app: Option<Arc<crate::daemon::process::AppProc>>,
    window: Window,
    tx: SyncSender<WvEvent>,
}

pub struct WryStateRegistry {
    app_weak: Weak<AppState>,
    intl: RwLock<WryStateRegistryIntl>,
}

struct WryStateRegistryIntl {
    map: u64,
    reg: [Option<Arc<WryState>>; 64],
}

#[derive(Clone)]
pub struct WryCreateOptions {
    pub title: String,
    pub url: String,
}

impl WryStateRegistry {
    pub fn new(app_weak: Weak<AppState>) -> Self {
        Self {
            app_weak,
            intl: RwLock::new(WryStateRegistryIntl {
                map: 0,
                reg: [(); 64].map(|_| None),
            }),
        }
    }

    pub fn create(&self, app: Option<Arc<crate::daemon::process::AppProc>>, options: WryCreateOptions) -> Result<Weak<WryState>> {
        let mut intl = self.intl.write().unwrap();
        let free_bit = (0..64).find(|i| (intl.map & (1 << i)) == 0);

        match free_bit {
            Some(id) => {
                if (intl.map & (1 << id)) != 0 {
                    return Err(WryStateRegistryError::RegistryFull.into());
                }
                intl.map |= 1 << id;

                let state =
                    self.app_weak
                        .upgrade()
                        .unwrap()
                        .dispatcher
                        .dispatch(move |event_loop| -> core::result::Result<_, Error> {
                            let window = WindowBuilder::new()
                                .with_title(options.title.clone())
                                .build(event_loop)
                                .context("Failed to create window")?;

                            let (tx, rx) = sync_channel(0);

                            let state = Arc::new(WryState {
                                id: id as u8,
                                app,
                                window,
                                tx,
                            });

                            let builder = WebViewBuilder::new().with_url(options.url);

                            #[cfg(not(target_os = "linux"))]
                            let webview = builder.build(&state.window).context("Failed to create webview")?;
                            #[cfg(target_os = "linux")]
                            let webview = builder.build_gtk(state.window.gtk_window()).context("Failed to create webview")?;

                            Box::leak(Box::new(webview));

                            // let wv_state = state.clone();

                            // let wv_options = options.clone();

                            // spawn(|| match wv_run(wv_state, wv_options, rx) {
                            //     core::result::Result::Ok(_) => (),
                            //     Err(err) => {
                            //         error!("{err:?}");
                            //     }
                            // });

                            Ok(state)
                        })??;

                intl.reg[id] = Some(state.clone());

                Ok(Arc::downgrade(&state))
            }
            None => Err(WryStateRegistryError::RegistryFull.into()),
        }
    }

    pub fn get(&self, id: u8) -> core::result::Result<Arc<WryState>, WryStateRegistryError> {
        let intl = self.intl.read().unwrap();

        if id >= 64 {
            return Err(WryStateRegistryError::InvalidId(id));
        }

        intl.reg[id as usize]
            .as_ref()
            .map(Arc::clone)
            .ok_or(WryStateRegistryError::WindowNotFound(id))
    }

    pub fn destroy(&self, id: u8) -> core::result::Result<(), WryStateRegistryError> {
        let mut intl = self.intl.write().unwrap();

        if id >= 64 {
            return Err(WryStateRegistryError::InvalidId(id));
        }

        let state = intl.reg[id as usize].take();

        let Some(state) = state else {
            return Err(WryStateRegistryError::WindowNotFound(id));
        };

        intl.map &= (!(1 << id));

        core::result::Result::Ok(())
    }
}

impl WryState {
    fn dispatch<R: 'static + Send, F: 'static + FnOnce(&WebView) -> R + Send>(
        &self,
        f: F,
    ) -> core::result::Result<R, EventLoopClosed> {
        let (dispatch_tx, dispatch_rx) = sync_channel(0);

        match self.tx.send(WvEvent::Dispatch(Box::new(move |webview| {
            dispatch_tx.send(f(webview)).unwrap();
        }))) {
            core::result::Result::Ok(_) => core::result::Result::Ok(()),
            Err(_) => Err(EventLoopClosed {}),
        }?;

        core::result::Result::Ok(dispatch_rx.recv().unwrap())
    }

    fn close(&self) -> core::result::Result<(), EventLoopClosed> {
        // TODO: Should this become impl of Drop trait?
        match self.tx.send(WvEvent::Close) {
            core::result::Result::Ok(_) => core::result::Result::Ok(()),
            Err(_) => Err(EventLoopClosed {}),
        }
    }
}

#[derive(Error, Debug)]
#[error("event loop closed")]
struct EventLoopClosed;

enum WvEvent {
    Dispatch(Box<dyn FnOnce(&WebView) -> () + Send>),
    Close,
}

fn wv_run(state: Arc<WryState>, options: WryCreateOptions, rx: Receiver<WvEvent>) -> Result<()> {
    let builder = WebViewBuilder::new().with_url(options.url);

    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&state.window).context("Failed to create webview")?;
    #[cfg(target_os = "linux")]
    let webview = builder.build_gtk(state.window.gtk_window()).context("Failed to create webview")?;

    while let WvEvent::Dispatch(fn_once) = rx.recv()? {
        fn_once(&webview)
    }

    Ok(())
}
