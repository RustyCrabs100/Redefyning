use {
    crate::utils::{AppState, RawWindowingHandles},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle},
    std::{convert::From, ptr::NonNull, sync::mpsc::Sender},
    tokio::{
        runtime::Runtime,
        sync::{mpsc::Sender as TokioSender, oneshot},
        task,
    },
    winit::{
        application::ApplicationHandler,
        dpi::PhysicalSize,
        event::{DeviceEvent, KeyEvent, WindowEvent},
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        keyboard::{KeyCode, PhysicalKey},
        window::{Window, WindowAttributes, WindowId},
    },
};

enum Events {
    Window(WindowEvent),
    Device(DeviceEvent),
    Keyboard(KeyEvent),
}

impl From<WindowEvent> for Events {
    fn from(value: WindowEvent) -> Self {
        Self::Window(value)
    }
}

impl From<DeviceEvent> for Events {
    fn from(value: DeviceEvent) -> Self {
        Self::Device(value)
    }
}

impl From<KeyEvent> for Events {
    fn from(value: KeyEvent) -> Self {
        Self::Keyboard(value)
    }
}

pub struct AppWindow {
    window: Option<Window>,
    attr: WindowAttributes,
    app_state: AppState,
    render_communicator: Option<Sender<AppState>>,
    surface_handles_sender: Option<oneshot::Sender<RawWindowingHandles>>,
    inner_size_sender: Option<TokioSender<PhysicalSize<u32>>>,
}

impl AppWindow {
    fn send_app_state(&self) {
        if let Some(rc) = &self.render_communicator {
            rc.send(self.app_state.clone());
        } else {
            eprintln!("Render Communicator not Initalized yet...");
        }
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        println!("Close Requested!");
        if let Some(rc) = &self.render_communicator {
            rc.send(AppState::Closed);
            self.app_state = AppState::Closed;
        } else {
            panic!("Render Communicator failed to be initalized!");
        }
        event_loop.exit();
    }

    fn get_surface_handles(&self) -> RawWindowingHandles {
        RawWindowingHandles::from_raw_tuple(&(
            self.window
                .as_ref()
                .unwrap()
                .display_handle()
                .unwrap()
                .as_raw(),
            self.window
                .as_ref()
                .unwrap()
                .window_handle()
                .unwrap()
                .as_raw(),
        ))
    }

    pub(crate) fn modify_window_attrs(&mut self, attrs: &WindowAttributes) {
        self.attr = attrs.clone();
    }

    pub(crate) fn init_render_communicator(&mut self, communicator: Sender<AppState>) {
        self.render_communicator = Some(communicator);
    }

    pub(crate) fn start(
        &mut self,
        surface_handles_sender: oneshot::Sender<RawWindowingHandles>,
        inner_size_sender: TokioSender<PhysicalSize<u32>>,
    ) {
        let event_loop = EventLoop::<Events>::with_user_event().build().unwrap();

        {
            #[cfg(feature = "app_mode")]
            {
                event_loop.set_control_flow(ControlFlow::Wait);
            }
            #[cfg(not(feature = "app_mode"))]
            {
                event_loop.set_control_flow(ControlFlow::Poll);
            }
        }

        self.surface_handles_sender = Some(surface_handles_sender);
        self.inner_size_sender = Some(inner_size_sender);

        event_loop.run_app(self).expect("Event Loop Eror");
    }
}

impl ApplicationHandler<Events> for AppWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Resumed");
        self.window = Some(event_loop.create_window(self.attr.clone()).unwrap());
        self.app_state = AppState::Open;
        // We know at this point surface_handles_sender is Some()
        let sender = self.surface_handles_sender.take().unwrap();
        sender.send(self.get_surface_handles()).expect("Un oh");
        let inner_size_sender = self.inner_size_sender.take().unwrap();
        let size = self.window.as_ref().unwrap().inner_size();
        task::block_in_place(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on((async move || inner_size_sender.send(size).await)())
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.exit(event_loop),
            WindowEvent::RedrawRequested => {
                println!("Requested Redraw")
            }
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(KeyCode::Escape) => self.exit(event_loop),
                _ => {}
            },
            _ => self.send_app_state(),
        }
    }
}

impl Default for AppWindow {
    fn default() -> Self {
        let attr = WindowAttributes::default().with_resizable(false);
        Self {
            window: None,
            attr,
            app_state: AppState::default(),
            render_communicator: None,
            surface_handles_sender: None,
            inner_size_sender: None,
        }
    }
}
