#[path = "window.rs"]
mod window;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub use minifb::WindowOptions as WindowSettings;

pub fn init(
    app_window_name: &str,
    app_window_size: (usize, usize),
    app_window_settings: Option<WindowSettings>,
    app_fps: Option<usize>
) {
    let mut game_window = window::GameWindow::new(
        app_window_name,
        app_window_size,
        app_window_settings,
        app_fps,
    );
    game_window.update()
}