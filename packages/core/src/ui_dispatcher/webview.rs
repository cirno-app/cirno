use crate::{AppState, ui_dispatcher::Dispatcher};
use anyhow::{Context, Error, Ok, Result};
use std::{
    collections::HashMap,
    sync::{
        Arc, Weak,
        mpsc::{Receiver, SyncSender, sync_channel},
    },
};
use tao::window::{Window, WindowBuilder, WindowId};
use thiserror::Error;
use wry::{WebView, WebViewBuilder};

#[derive(Debug, Error)]
enum WebViewManagerError {
    #[error("No window found for ID: {0}")]
    WindowNotFound(usize),
}

pub struct WebViewInstance {
    pub id: usize,
    wid: WindowId,
}

struct WebViewInstanceIntl {
    app: Option<Arc<crate::daemon::process::AppProc>>,
    window: Window,
    webview: WebView,
}

pub struct WebViewManager {
    pub dispatcher: Dispatcher,
    app_weak: Weak<AppState>,
    reg: HashMap<WindowId, WebViewInstanceIntl>,
    idreg: Vec<Option<WindowId>>,
}

#[derive(Clone)]
pub struct WebViewCreateOptions {
    pub title: String,
    pub url: String,
}

impl WebViewManager {
    pub fn new(dispatcher: Dispatcher, app_weak: Weak<AppState>) -> Self {
        Self {
            dispatcher,
            app_weak,
            reg: HashMap::new(),
            idreg: Vec::new(),
        }
    }

    pub fn create(
        &self,
        app: Option<Arc<crate::daemon::process::AppProc>>,
        options: WebViewCreateOptions,
    ) -> Result<WebViewInstance> {
        Ok(self
            .app_weak
            .upgrade()
            .unwrap()
            .dispatcher
            .dispatcher
            .dispatch(move |event_loop| -> core::result::Result<_, Error> {
                let window = WindowBuilder::new()
                    .with_title(options.title.clone())
                    .build(event_loop)
                    .context("Failed to create window")?;

                let builder = WebViewBuilder::new().with_url(options.url);

                #[cfg(not(target_os = "linux"))]
                let webview = builder.build(&window).context("Failed to create webview")?;
                #[cfg(target_os = "linux")]
                let webview = builder
                    .build_gtk(state.window.gtk_window())
                    .context("Failed to create webview")?;

                if self
                    .reg
                    .insert(
                        window.id(),
                        WebViewInstanceIntl {
                            app,
                            window,
                            webview,
                        },
                    )
                    .is_some()
                {
                    panic!("Duplicated window id detected");
                }

                let id = self.idreg.len();
                self.idreg[id] = Some(window.id());

                Ok(WebViewInstance {
                    id,
                    wid: window.id(),
                })
            })??)
    }

    pub fn get(&self, id: usize) -> Result<WebViewInstance> {
        self.dispatcher
            .dispatch(move |_event_loop| -> Result<WebViewInstance> {
                let wid = self.idreg.get(id);

                match wid {
                    Some(wid) => match wid {
                        Some(wid) => Ok(WebViewInstance { id, wid: *wid }),
                        None => Err(WebViewManagerError::WindowNotFound(id).into()),
                    },
                    None => Err(WebViewManagerError::WindowNotFound(id).into()),
                }
            })?
    }

    pub fn destroy(&self, id: usize) -> Result<()> {
        self.dispatcher.dispatch(move |_event_loop| -> Result<()> {
            let wid = self.idreg.get(id);

            if wid.is_none() {
                return Err(WebViewManagerError::WindowNotFound(id).into());
            }

            self.idreg[id] = None;

            Ok(())
        })?
    }
}
