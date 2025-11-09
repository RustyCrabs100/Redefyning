#![cfg(feature = "vulkan")]

use {
    crate::utils::AppState,
    ash,
    ash::{ext, khr, vk},
    std::sync::{Arc, mpsc::Receiver},
};

pub struct VkCore {
    pub communicator: WindowCommunication,
    pub core: VkStart,
    #[cfg(all(debug_assertions, feature = "debug"))]
    pub debug: Option<VkDebug>,
    pub devices: VkDevices,
    pub display: VkDisplay,
}

pub struct WindowCommunication(Option<Receiver<AppState>>);

pub struct VkStart {
    pub entry: Arc<ash::Entry>,
    pub instance: Arc<ash::Instance>,
}

#[cfg(all(debug_assertions, feature = "debug"))]
pub struct VkDebug {
    pub loader: Arc<ext::debug_utils::Instance>,
    pub messenger: Option<vk::DebugUtilsMessengerEXT>,
}

pub struct VkDevices {
    pub physical: vk::PhysicalDevice,
    pub logical: Arc<ash::Device>,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

pub struct VkDisplay {
    pub surface: VkSurface,
    pub swapchain: VkSwapchain,
}

pub struct VkSurface {
    pub handle: vk::SurfaceKHR,
    pub functions: Arc<khr::surface::Instance>,
}

pub struct VkSwapchain {
    pub handle: vk::SwapchainKHR,
    pub device: Arc<khr::swapchain::Device>,
    pub images: Arc<Vec<vk::Image>>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
}
