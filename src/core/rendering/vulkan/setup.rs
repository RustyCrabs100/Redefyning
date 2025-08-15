#[cfg(feature = "vulkan")]
pub mod vulkan {
    use {
        ash::{
            vk,
            prelude::VkResult,
            Entry,
            Instance,
        },
        std::{
            sync::{
                mpsc, 
                atomic::AtomicBool
            },
            ops::Drop,
        }
    };

    pub struct VulkanInit {
        pub reciever: mpsc::Receiver<AtomicBool>, 
        pub instance: Instance,
    }

    impl Drop for VulkanInit {
        fn drop(&mut self) {   
            unsafe {
                // self.instance.destroy_instance(None);
            }
        }
    }

    impl VulkanInit {
        pub fn new(rx: mpsc::Receiver<AtomicBool>) -> Self {
            let entry = unsafe {Entry::load().expect("Failed to load Vulkan Entry Points")};
            let instance = Self::create_instance(&entry).unwrap();
            Self {
                reciever: rx,
                instance
            }   
        }

        fn create_instance(entry: &Entry) -> VkResult<Instance> {
            let app_info = vk::ApplicationInfo::default()
                .application_name(&c"Redefyning Game Engine")
                .application_version(vk::make_api_version(0, 0, 0, 0))
                .engine_name(&c"Redefyning Game Engine")
                .engine_version(vk::make_api_version(0, 0, 0, 0))
                .api_version(vk::make_api_version(0, 1, 3, 281));

            let create_info = vk::InstanceCreateInfo::default()
                .flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
                .application_info(&app_info);

            Ok(unsafe { entry.create_instance(&create_info, None).expect("Unable to create Instance")})
        }

        fn create_instance_layers(entry: &Entry) {
            
        }
    }
}