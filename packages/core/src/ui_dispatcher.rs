use std::{pin::Pin, sync::Arc};
use tao::{
    event::Event,
    event_loop::{ControlFlow::Wait, EventLoop, EventLoopBuilder, EventLoopProxy},
};

enum DispatcherEvent {
    Dispatch(Box<dyn FnOnce(Event<'_, Self>) -> ()>),
    DispatchWait(Box<dyn FnOnce(Event<'_, Self>) -> ()>),
    DispatchAwait(Box<dyn FnOnce(Event<'_, Self>) -> Pin<Box<dyn Future<Output = ()>>>>),
}

struct DispatcherIntl {
    proxy: EventLoopProxy<DispatcherEvent>,
}

pub struct Dispatcher {
    event_loop: Option<EventLoop<DispatcherEvent>>,
    intl: Arc<DispatcherIntl>,
}

pub struct EventLoopClosed;

impl Clone for Dispatcher {
    fn clone(&self) -> Self {
        Self {
            event_loop: None,
            intl: self.intl.clone(),
        }
    }
}

impl Dispatcher {
    pub fn new() -> Dispatcher {
        let event_loop = EventLoopBuilder::<DispatcherEvent>::with_user_event().build();

        let proxy = event_loop.create_proxy();

        Dispatcher {
            event_loop: Some(event_loop),
            intl: Arc::new(DispatcherIntl { proxy }),
        }
    }

    pub fn run(self) -> ! {
        match self.event_loop {
            None => panic!("Do not call run() on cloned Dispatcher"),

            Some(event_loop) => event_loop.run(|event, event_loop, control_flow| {
                *control_flow = Wait;

                match event {
                    tao::event::Event::UserEvent(manager_event) => match manager_event {
                        DispatcherEvent::Dispatch(fn_once) => todo!(),
                        DispatcherEvent::DispatchWait(fn_once) => todo!(),
                        DispatcherEvent::DispatchAwait(fn_once) => todo!(),
                    },
                    _ => {}
                }
            }),
        }
    }

    pub fn dispatch<F: FnOnce(Event<'_, DispatcherEvent>) -> () + 'static>(
        &self,
        f: F,
    ) -> Result<(), EventLoopClosed> {
        match self
            .intl
            .proxy
            .send_event(DispatcherEvent::Dispatch(Box::new(f)))
        {
            Ok(_) => Ok(()),
            Err(err) => Err(match err.0 {
                DispatcherEvent::Dispatch(fn_once) => EventLoopClosed {},
                _ => panic!(),
            }),
        }
    }

    pub fn dispatch_wait<R, F: FnOnce(Event<'_, DispatcherEvent>) -> R>(&self, f: F) {
        match self
            .intl
            .proxy
            .send_event(DispatcherEvent::Dispatch(Box::new(f)))
        {
            Ok(_) => Ok(()),
            Err(err) => Err(match err.0 {
                DispatcherEvent::Dispatch(fn_once) => EventLoopClosed {},
                _ => panic!(),
            }),
        }
    }

    pub async fn dispatch_await<R, F: AsyncFnOnce(Event<'_, DispatcherEvent>) -> R>(&self, f: F) {
        match self
            .intl
            .proxy
            .send_event(DispatcherEvent::Dispatch(Box::new(f)))
        {
            Ok(_) => Ok(()),
            Err(err) => Err(match err.0 {
                DispatcherEvent::Dispatch(fn_once) => EventLoopClosed {},
                _ => panic!(),
            }),
        }
    }
}
