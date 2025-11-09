#[path = "../data.rs"]
mod data;

impl data::VkDevices {}

impl Drop for data::VkDevices {
    fn drop(&mut self) {}
}