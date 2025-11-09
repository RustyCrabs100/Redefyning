#[path = "../data.rs"]
mod data;

use {ash, ash::vk};

impl data::VkStart {
    fn new() -> Result<Self, vk::Result> {
        let entry = ash::Entry::load()?;
    }

    fn create_instance() -> Arc<ash::Instance> {}
}

impl Drop for data::VkStart {
    fn drop(&mut self) {}
}
