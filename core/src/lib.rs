// Note:
//   Documentation Needed
//   Documentation will be Required later on.
//   Best to get into that habit now.
//   If lack of documentation becomes an issue, we will stop development to Document.

use {
    crate::utils::{AppState, RawWindowingHandles},
    once_cell::sync::Lazy,
    std::sync::mpsc::channel,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
/// This struct represents the application's version.
/// The numbers go in the order of
/// 1. Variant / Build
/// 2. Major
/// 3. Minor
/// 4. Patch
/// 5. Revision
/// This is printed as 2.3.4.1-5 (see the above for number definiton)
pub struct AppVersion(u32, u32, u32, u32, &'static str);

impl std::fmt::Display for AppVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{0}.{1}.{2}.{3}-{4}",
            self.0, self.1, self.2, self.3, self.4
        )
        .unwrap();
        Ok(())
    }
}

impl AppVersion {
    pub fn new(
        major: u32,
        minor: u32,
        patch: u32,
        variant: u32,
        revision: Option<&'static str>,
    ) -> Self {
        let final_revision = match revision {
            Some(x) => x,
            None => "stable",
        };
        AppVersion(variant, major, minor, patch, final_revision)
    }

    pub fn unpack_raw(&self) -> (u32, u32, u32, u32) {
        (self.0, self.1, self.2, self.3)
    }

    pub fn unpack(&self) -> (u32, u32, u32, u32, &'static str) {
        (self.0, self.1, self.2, self.3, self.4)
    }

    pub fn unpack_str(&self) -> &'static str {
        "{self.0}.{self.1}.{self.2}.{self.3}-{self.4}"
    }
}

pub struct App {
    scripts: Vec<Box<dyn Fn() + Send + 'static>>,
    name: &'static str,
    version: AppVersion,
    window_settings: WindowSettings,
}

impl App {
    pub fn new(
        name: &'static str,
        version: AppVersion,
        window_settings: Option<WindowSettings>,
    ) -> Self {
        Self {
            scripts: Vec::new(),
            name,
            version,
            window_settings: window_settings.unwrap_or_default(),
        }
    }

    pub fn add_script(mut self, script: Box<dyn Fn() + Send + 'static>) -> Self {
        self.scripts.push(script);
        self
    }

    pub fn run(self) {
        Lazy::force(&utils::TIMER);
        // Fix this later
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
        app_window.modify_window_attrs(&self.window_settings);
        app_window.init_render_communicator(tx.clone());

        let name = self.name;
        let version = self.version;
        let scripts = self.scripts;

        // Renderer thread
        let _renderer_thread = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let surface_handles = oneshot_rx.await.unwrap().unpack();

                #[cfg(feature = "vulkan")]
                {
                    let mut vulkan_setup = vk::setup::VulkanSetup::new(
                        surface_handles,
                        name,
                        version.unpack_raw(),
                        tokio_rx,
                    )
                    .expect("Unable to create Vulkan Setup");
                    vulkan_setup.init_window_communicator(rx);
                    let core = vk::Core::new(vulkan_setup);
                    core.main_loop();
                }
            });
        });

        // Scripting thread
        let _scripting_thread = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                for script in scripts {
                    script();
                }
            });
        });

        app_window.start(oneshot_tx, tokio_tx);
    }
}
