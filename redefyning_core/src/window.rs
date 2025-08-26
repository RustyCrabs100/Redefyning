use minifb;

const DEFAULT_WIDTH: usize = 640;
const DEFAULT_HEIGHT: usize = 480;

#[derive(Debug)]
pub struct GameWindow {
    pub window: minifb::Window,
    pub window_settings: minifb::WindowOptions,
}

impl Default for GameWindow {
    /// The default implementation.
    /// This either returns Self or panicks on Window Creation.
    /// Perfer the new() function, you get much more control over Window Settings
    fn default() -> Self {
        let window_options = minifb::WindowOptions {
            resize: false,
            ..Default::default()
        };
        let window = minifb::Window::new(
                "Window",
                DEFAULT_WIDTH,
                DEFAULT_HEIGHT,
                window_options
            ).expect("Unable to Create a Window");
        Self {
            window,
            window_settings: window_options,
        }
    }
}

impl GameWindow {
    /// Creates a new GameWindow, with 
    ///   the Window Name = name = &str,
    ///   the Window Size = size = width & height = usize & usize,
    ///   the Window Settings = options = Option<minifb::WindowOptions>,
    ///   the Window's FPS = fps = Option<usize>,
    /// If fps == None, the target fps will be set to 60
    /// If options == None, the Window Settings = minifb::WindowOptions {
    ///     resize: false,
    ///     ..Default::default()
    /// }
    pub fn new(
        name: &str,
        size: (usize, usize),
        options: Option<minifb::WindowOptions>,
        fps: Option<usize>
    ) -> Self {
        let target_fps = match fps {
            Some(x) => x,
            None => 60
        };
        let window_options = match options {
            Some(x) => x,
            None => minifb::WindowOptions {
                resize: false,
                ..Default::default()
            }
        };
        let mut window = minifb::Window::new(
            name,
            size.0,
            size.1,
            window_options
        ).expect("Window Failed to be Made");
        window.set_target_fps(target_fps);
        Self {
            window,
            window_settings: window_options
        }
    }

    pub fn update(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape) {
            self.window
                .update();
        }
    }
}