use crate::utils::str_to_p_const_c_char as stpc3;
use std::sync::Arc;

#[path = "../data.rs"]
mod data;

impl data::VkDevices {
    const REQUIRED_DEVICE_EXTENSIONS: &[*const c_char] = &[
        // Swapchain Creation
        stpc3!("VK_KHR_swapchain"),
        // Added Dynamic Rendering, Flexible Pipelines
        stpc3!("VK_KHR_dynamic_rendering"),
        // Graph Frame-friendly Sync
        stpc3!("VK_KHR_timeline_semaphore"),
        // Cleaner, less error-prone sync API
        stpc3!("VK_KHR_synchronization2"),
        // Change Cull Mode, Topology, Blend State, etc. at draw time.
        stpc3!("VK_EXT_extended_dynamic_state"),
        stpc3!("VK_EXT_extended_dynamic_state2"),
        stpc3!("VK_EXT_extended_dynamic_state3"),
        // Dynamic Vertex Bindings/Attibutes
        stpc3!("VK_EXT_vertex_input_dynamic_state"),
        // Per-attachment color write control
        stpc3!("VK_EXT_color_write_enable"),
        // Bindless textures/buffers, sparse descriptor arrays
        stpc3!("VK_EXT_descriptor_indexing"),
        // GPU pointers for bindless/resource-descriptor-less acces
        stpc3!("VK_KHR_buffer_device_address"),
        // Query VRAM Usage
        stpc3!("VK_EXT_memory_budget"),
        // Lower-percision math for bandwidth saving
        stpc3!("VK_KHR_shader_float16_int8"),
        // Integer dot products for ML/physics.
        stpc3!("VK_KHR_shader_integer_dot_product"),
        // Negative viewport height, minor fixes
        stpc3!("VK_KHR_maintenance1"),
        // New image layouts, subpass dependency improvements
        stpc3!("VK_KHR_maintenance2"),
        // Descriptor query improvements
        stpc3!("VK_KHR_maintenance3"),
        // Device memory null handles, improved queries,
        stpc3!("VK_KHR_maintenance4"),
        // Pipeline creation feedback, descriptor indexing improvements
        stpc3!("VK_KHR_maintenance5"),
        // SPIR-V 1.4 Support
        stpc3!("VK_KHR_spirv_1_4"),
        // Extended Renderpass Creation
        stpc3!("VK_KHR_create_renderpass2"),
    ];

    pub(crate) fn new(vk_start: &data::VkStart) -> Self {}

    fn pick_physical_device(instance: &ash::Instance) -> Arc<vk::PhysicalDevice> {
        // Get Physical Devices
        let devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to find Physical Devices")
        };
        // Assert that there are devices
        assert!(!devices.is_empty());
        // Iterate over all avaliable devices & find a good one.
        // Panic if none were suitable
        let physical_device = devices.into_iter().find(|&device| {
            let mut return_items: (u32, u32, Arc<vk::PhysicalDevice>);
            // gets physical device properties
            let props = unsafe { instance.get_physical_device_properties(device) };
            // skip the device if the vulkan version is less than 1.3.286.0
            if props.api_version < vk::make_api_version(0, 1, 3, 286) {
                return false;
            }
            // Collect Queue Families
            let queue_Families =
                unsafe { instance.get_physical_device_queue_family_properties(device) };
            // Skip the device if it doesn't support
        });
    }
}

impl Drop for data::VkDevices {
    fn drop(&mut self) {
        unsafe {
            self.logical.destroy_device(None);
        }
    }
}
