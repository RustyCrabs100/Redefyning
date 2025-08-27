// Note: 
//   Documentation Needed
//   Documentation will be Required later on.
//   Best to get into that habit now.
//   If lack of documentation becomes an issue, we will stop development to Document.


#[path = "window.rs"]
mod window;

#[cfg(feature = "mimalloc")]
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
    let surface_handles = game_window.surface_handles();
    // Last Piece of Code, dont put anything after it
    game_window.update();
}