use crate::{AppState, ui_dispatcher::Dispatcher};
use anyhow::{Context, Error, Ok, Result};
use std::{
    cell::SyncUnsafeCell,
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

struct SyncWebView {
    webview: WebView,
}

unsafe impl Send for SyncWebView {}
unsafe impl Sync for SyncWebView {}

struct WebViewInstanceIntl {
    app: Option<Arc<crate::daemon::process::AppProc>>,
    window: Window,
    webview: SyncWebView,
}

struct WebViewManagerReg {
    map: HashMap<WindowId, WebViewInstanceIntl>,
    id: Vec<Option<WindowId>>,
}

pub struct WebViewManager {
    pub dispatcher: Dispatcher,
    app_weak: Weak<AppState>,
    reg: SyncUnsafeCell<WebViewManagerReg>,
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
            reg: SyncUnsafeCell::new(WebViewManagerReg {
                map: HashMap::new(),
                id: Vec::new(),
            }),
        }
    }

    pub fn create(
        &self,
        app: Option<Arc<crate::daemon::process::AppProc>>,
        options: WebViewCreateOptions,
    ) -> Result<WebViewInstance> {
        let app_state = self.app_weak.upgrade().unwrap();

        Ok(self
            .dispatcher
            .dispatch(move |event_loop| -> core::result::Result<_, Error> {
                let app_state = app_state.clone();
                let wv_manager = &app_state.dispatcher;

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

                let wid = window.id();

                let id;

                unsafe {
                    let reg = wv_manager.reg.get().as_mut_unchecked();

                    if reg
                        .map
                        .insert(
                            wid,
                            WebViewInstanceIntl {
                                app,
                                window,
                                webview: SyncWebView { webview },
                            },
                        )
                        .is_some()
                    {
                        panic!("Duplicated window id detected");
                    }

                    id = reg.id.len();
                    reg.id.push(Some(wid));
                }

                Ok(WebViewInstance { id, wid })
            })??)
    }

    pub fn get(&self, id: usize) -> Result<WebViewInstance> {
        let app_state = self.app_weak.upgrade().unwrap();

        self.dispatcher
            .dispatch(move |_event_loop| -> Result<WebViewInstance> {
                let app_state = app_state.clone();
                let wv_manager = &app_state.dispatcher;

                let wid;

                unsafe {
                    wid = wv_manager.reg.get().as_ref_unchecked().id.get(id);
                }

                match wid {
                    Some(Some(wid)) => Ok(WebViewInstance { id, wid: *wid }),
                    _ => Err(WebViewManagerError::WindowNotFound(id).into()),
                }
            })?
    }

    pub fn destroy(&self, id: usize) -> Result<()> {
        let app_state = self.app_weak.upgrade().unwrap();

        self.dispatcher.dispatch(move |_event_loop| -> Result<()> {
            let app_state = app_state.clone();
            let wv_manager = &app_state.dispatcher;

            let wid_option;

            unsafe {
                wid_option = wv_manager.reg.get().as_ref_unchecked().id.get(id);
            }

            let wid = match wid_option {
                Some(Some(wid)) => wid,
                _ => {
                    return Err(WebViewManagerError::WindowNotFound(id).into());
                }
            };

            unsafe {
                let reg = wv_manager.reg.get().as_mut_unchecked();
                reg.id[id] = None;
                reg.map
                    .remove(wid)
                    .expect("reg.map should contain WebViewInstanceIntl");
            }

            Ok(())
        })?
    }
}
