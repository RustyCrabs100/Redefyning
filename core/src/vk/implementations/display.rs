#[path = "../data.rs"]
mod data;

impl data::VkDisplay {}

impl data::VkSwapchain {}

impl Drop for data::VkSwapchain {
    fn drop(&mut self) {}
}

impl data::VkSurface {
    fn new() -> Self {}
}

impl Drop for data::VkSurface {
    fn drop(&mut self) {}
}
