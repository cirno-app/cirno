use tao::event_loop::{ControlFlow::Wait, EventLoop, EventLoopBuilder};

enum EventLoopManagerEvent<T> {
    CreateWindow,
    User(T),
}

pub struct EventLoopInit<T: 'static> {
    event_loop: EventLoop<EventLoopManagerEvent<T>>,
}

pub struct EventLoopManager<T: 'static> {}

impl<T: 'static> EventLoopManager<T> {
    pub fn new() -> (EventLoopInit<T>, EventLoopManager<T>) {
        let event_loop = EventLoopBuilder::<EventLoopManagerEvent<T>>::with_user_event().build();

        (EventLoopInit { event_loop }, EventLoopManager::<T> {})
    }
}

impl<T: 'static> EventLoopInit<T> {
    pub fn run(self) -> ! {
        self.event_loop.run(|event, event_loop, control_flow| {
            *control_flow = Wait;

            match event {
                tao::event::Event::NewEvents(start_cause) => todo!(),
                tao::event::Event::WindowEvent {
                    window_id, event, ..
                } => todo!(),
                tao::event::Event::DeviceEvent {
                    device_id, event, ..
                } => todo!(),
                tao::event::Event::UserEvent(manager_event) => match manager_event {
                    EventLoopManagerEvent::CreateWindow => todo!(),
                    EventLoopManagerEvent::User(user_event) => todo!(),
                },
                tao::event::Event::Suspended => todo!(),
                tao::event::Event::Resumed => todo!(),
                tao::event::Event::MainEventsCleared => todo!(),
                tao::event::Event::RedrawRequested(window_id) => todo!(),
                tao::event::Event::RedrawEventsCleared => todo!(),
                tao::event::Event::LoopDestroyed => todo!(),
                tao::event::Event::Opened { urls } => todo!(),
                tao::event::Event::Reopen {
                    has_visible_windows,
                    ..
                } => todo!(),
                _ => {}
            }
        })
    }
}
