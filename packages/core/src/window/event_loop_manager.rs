use tao::event_loop::EventLoop;

pub struct EventLoopInit {
    event_loop: EventLoop<()>,
}

pub struct EventLoopManager {}

impl EventLoopManager {
    pub fn new() -> (EventLoopInit, EventLoopManager) {
        let event_loop = EventLoop::new();

        (EventLoopInit { event_loop }, EventLoopManager {})
    }
}

impl EventLoopInit {
    pub fn run(self) -> ! {
        self.event_loop.run(|event, event_loop, control_flow| {
            *control_flow = tao::event_loop::ControlFlow::Wait;
        })
    }
}
