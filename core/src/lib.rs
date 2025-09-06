// Note: 
//   Documentation Needed
//   Documentation will be Required later on.
//   Best to get into that habit now.
//   If lack of documentation becomes an issue, we will stop development to Document.


#[path = "window.rs"]
mod window;

#[cfg(feature = "vulkan")]
#[path = "vulkan.rs"]
mod vulkan;

#[path = "utils.rs"]
mod utils;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub use minifb::WindowOptions as WindowSettings;

pub fn init(
    app_name: &str,
    // Variant, Major, Minor, Patch
    app_version: (u32, u32, u32, u32),
    app_window_size: (usize, usize),
    app_window_settings: Option<WindowSettings>,
    app_fps: Option<usize>
) {
    #[cfg(any(
        all(feature = "debug", not(debug_assertions)),
        all(not(feature = "debug"), debug_assertions)
    ))] {
        panic!("Debug feature enabled but debug assertions isn't (or vice versa)")
    }
    let mut game_window = window::GameWindow::new(
        app_name,
        app_window_size,
        app_window_settings,
        app_fps,
    );
    let surface_handles = game_window.surface_handles();
    #[cfg(feature = "vulkan")] {
        let vulkan_setup = vulkan::VulkanSetup::new(
            surface_handles,
            app_name,
            app_version,
        ).expect("Unable to create Vulkan Setup");
    }
    // Last Piece of Code, dont put anything after it
    game_window.update();
}