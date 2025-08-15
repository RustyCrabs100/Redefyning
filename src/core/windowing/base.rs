// This file is the base module for windowing functionality.
// Any specific windowing implementations should be placed in separate files.
pub mod windowing {
    use {
        std::sync::{atomic::AtomicBool, mpsc},
        winit::{
            application::ApplicationHandler,
            event::WindowEvent,
            event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
            window::{Window, WindowAttributes, WindowId},
        },
    };

    #[derive(Debug)]
    pub struct Windowing {
        window: Option<Window>,
        event_sender: mpsc::Sender<AtomicBool>,
    }
    
    impl ApplicationHandler for Windowing {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            self.window = Some(
                event_loop
                    .create_window(WindowAttributes::default())
                    .unwrap(),
            );
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _id: WindowId,
            event: WindowEvent,
        ) {
            match event {
                WindowEvent::CloseRequested => {
                    println!("Closing");
                    self.event_sender.send(AtomicBool::new(true)).unwrap();
                    event_loop.exit();
                }
                WindowEvent::Destroyed => {
                    println!("Window Destroyed");
                }
                WindowEvent::RedrawRequested => {
                    self.window.as_ref().unwrap().request_redraw();
                }
                _ => {}
            }
        }
    }

    impl Windowing {
        pub fn new(sender: mpsc::Sender<AtomicBool>) -> Self {
            Self {
                window: None,
                event_sender: sender,
            }
        }
        pub fn run(&mut self) {
            let event_loop = EventLoop::new().unwrap();
            event_loop.set_control_flow(ControlFlow::Poll);
            event_loop.run_app(self).unwrap();
        }
    }
}
