// Note:
//   Documentation Needed
//   Documentation will be Required later on.
//   Best to get into that habit now.
//   If lack of documentation becomes an issue, we will stop development to Document.

use {
    crate::utils::{AppState, RawWindowingHandles},
    raw_window_handle::{RawDisplayHandle, RawWindowHandle},
    std::sync::{Arc, Mutex, mpsc::channel},
    tokio::sync::{mpsc, oneshot},
    winit::dpi::PhysicalSize,
};

#[path = "winit.rs"]
mod window;

#[cfg(feature = "vulkan")]
#[path = "vk/mod.rs"]
mod vk;

#[path = "utils.rs"]
mod utils;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub use winit::window::WindowAttributes as WindowSettings;

pub fn init(
    app_name: &'static str,
    // Variant, Major, Minor, Patch
    app_version: (u32, u32, u32, u32),
    app_window_settings: Option<WindowSettings>,
) {
    #[cfg(any(
        all(feature = "debug", not(debug_assertions)),
        all(not(feature = "debug"), debug_assertions)
    ))]
    {
        panic!("Debug feature enabled but debug assertions isn't (or vice versa)")
    }
    let (tx, rx) = channel::<AppState>();
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<RawWindowingHandles>();
    let (tokio_tx, tokio_rx) = mpsc::channel::<PhysicalSize<u32>>(16);

    let mut app_window = Box::new(window::AppWindow::default());

    match app_window_settings {
        Some(attr) => app_window.modify_window_attrs(attr),
        None => {}
    }

    app_window.init_render_communicator(tx);

    let renderer_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on((async move || {
            let surface_handles = oneshot_rx.await.unwrap().unpack();

            #[cfg(feature = "vulkan")]
            {
                let mut vulkan_setup =
                    vk::VulkanSetup::new(surface_handles, app_name, app_version, tokio_rx)
                        .expect("Unable to create Vulkan Setup");
                vulkan_setup.init_window_communicator(rx);
            }
        })());
    });

    app_window.start(oneshot_tx, tokio_tx);

    // Minifb Code
    /*
    let mut game_window =
        window::GameWindow::new(app_name, app_window_size, app_window_settings, app_fps);
    let surface_handles = game_window.surface_handles();
    #[cfg(feature = "vulkan")]
    {
        let vulkan_setup = vulkan::VulkanSetup::new(surface_handles, app_name, app_version)
            .expect("Unable to create Vulkan Setup");
    }
    // Last Piece of Code, dont put anything after it
    game_window.update();
    */
}
