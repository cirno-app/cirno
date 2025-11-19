use std::fmt::Debug;
use std::sync::Arc;
use std::sync::mpsc::sync_channel;

use tao::event_loop::ControlFlow::Wait;
use tao::event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};
use thiserror::Error;

pub enum DispatcherEvent {
    Dispatch(Box<dyn FnOnce(&EventLoopWindowTarget<DispatcherEvent>) + Send>),
}

struct DispatcherIntl {
    proxy: EventLoopProxy<DispatcherEvent>,
}

pub struct Dispatcher {
    intl: Arc<DispatcherIntl>,
}

pub struct DispatcherInit {
    event_loop: EventLoop<DispatcherEvent>,
}

#[derive(Error, Debug)]
#[error("event loop closed")]
pub struct EventLoopClosed;

impl Clone for Dispatcher {
    fn clone(&self) -> Self {
        Self { intl: self.intl.clone() }
    }
}

impl Dispatcher {
    pub fn new() -> (DispatcherInit, Dispatcher) {
        let event_loop = EventLoopBuilder::<DispatcherEvent>::with_user_event().build();

        let proxy = event_loop.create_proxy();

        (
            DispatcherInit { event_loop },
            Dispatcher {
                intl: Arc::new(DispatcherIntl { proxy }),
            },
        )
    }

    pub fn dispatch<R: 'static + Send, F: 'static + FnOnce(&EventLoopWindowTarget<DispatcherEvent>) -> R + Send>(
        &self,
        f: F,
    ) -> Result<R, EventLoopClosed> {
        let (tx, rx) = sync_channel(0);

        match self.intl.proxy.send_event(DispatcherEvent::Dispatch(Box::new(move |event_loop| {
            tx.send(f(event_loop)).unwrap();
        }))) {
            Ok(_) => Ok(()),
            Err(err) => Err(match err.0 {
                DispatcherEvent::Dispatch(_) => EventLoopClosed {},
            }),
        }?;

        Ok(rx.recv().unwrap())
    }
}

impl DispatcherInit {
    pub fn run(self) -> ! {
        self.event_loop.run(|event, event_loop, control_flow| {
            *control_flow = Wait;

            if let tao::event::Event::UserEvent(manager_event) = event {
                match manager_event {
                    DispatcherEvent::Dispatch(fn_once) => fn_once(event_loop),
                }
            }
        })
    }
}
