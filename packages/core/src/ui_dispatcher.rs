use std::sync::Arc;
use tao::event_loop::{ControlFlow::Wait, EventLoop, EventLoopBuilder, EventLoopProxy};

enum DispatcherEvent<T> {
    CreateWindow,
    User(T),
}

struct DispatcherIntl<T: 'static> {
    proxy: EventLoopProxy<DispatcherEvent<T>>,
}

pub struct Dispatcher<T: 'static> {
    event_loop: Option<EventLoop<DispatcherEvent<T>>>,
    intl: Arc<DispatcherIntl<T>>,
}

pub struct EventLoopClosed<T>(T);

impl<T> Clone for Dispatcher<T> {
    fn clone(&self) -> Self {
        Self {
            event_loop: None,
            intl: self.intl.clone(),
        }
    }
}

impl<T: 'static> Dispatcher<T> {
    pub fn new() -> Dispatcher<T> {
        let event_loop = EventLoopBuilder::<DispatcherEvent<T>>::with_user_event().build();

        let proxy = event_loop.create_proxy();

        Dispatcher {
            event_loop: Some(event_loop),
            intl: Arc::new(DispatcherIntl::<T> { proxy }),
        }
    }

    pub fn run(self) -> ! {
        match self.event_loop {
            None => panic!("Do not call run() on cloned Dispatcher"),

            Some(event_loop) => event_loop.run(|event, event_loop, control_flow| {
                *control_flow = Wait;

                match event {
                    tao::event::Event::UserEvent(manager_event) => match manager_event {
                        DispatcherEvent::CreateWindow => todo!(),
                        DispatcherEvent::User(user_event) => todo!(),
                    },
                    _ => {}
                }
            }),
        }
    }

    pub fn send_event(&self, event: T) -> Result<(), EventLoopClosed<T>> {
        match self.intl.proxy.send_event(DispatcherEvent::User(event)) {
            Ok(_) => Ok(()),
            Err(err) => Err(match err.0 {
                DispatcherEvent::User(event) => EventLoopClosed(event),
                _ => panic!(),
            }),
        }
    }
}
