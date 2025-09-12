use std::sync::Arc;
use tao::event_loop::{ControlFlow::Wait, EventLoop, EventLoopBuilder, EventLoopProxy};

enum EventLoopManagerEvent<T> {
    CreateWindow,
    User(T),
}

pub struct EventLoopInit<T: 'static> {
    event_loop: EventLoop<EventLoopManagerEvent<T>>,
}

pub struct EventLoopManager<T: 'static> {
    proxy: EventLoopProxy<EventLoopManagerEvent<T>>,
}

impl<T: 'static> EventLoopManager<T> {
    pub fn new() -> (EventLoopInit<T>, Arc<EventLoopManager<T>>) {
        let event_loop = EventLoopBuilder::<EventLoopManagerEvent<T>>::with_user_event().build();

        let proxy = event_loop.create_proxy();

        (
            EventLoopInit { event_loop },
            Arc::new(EventLoopManager::<T> { proxy }),
        )
    }
}

impl<T: 'static> EventLoopInit<T> {
    pub fn run(self) -> ! {
        self.event_loop.run(|event, event_loop, control_flow| {
            *control_flow = Wait;

            match event {
                tao::event::Event::UserEvent(manager_event) => match manager_event {
                    EventLoopManagerEvent::CreateWindow => todo!(),
                    EventLoopManagerEvent::User(user_event) => todo!(),
                },
                _ => {}
            }
        })
    }
}
