#[cfg(feature = "vulkan")]
pub mod vulkan {
    use {
        ash::{
            vk,
            Entry,
            Instance,
        },
        std::sync::{mpsc, AtomicBool},
    };
    #[derive(Debug)]
    pub struct VulkanInit {
        pub reciever: mpsc::Reciever<AtomicBool>, 
        pub instance: Instance,
    }

    impl VulkanInit {
        pub fn new(rx: mpsc::Reciever<AtomicBool>) -> Self {
            let entry = Entry::linked();
            let instance = Self::create_instance(&entry).unwrap();
            Self {
                reciever: rx,
                instance
            }   
        }

        fn create_instance(entry: &Entry) -> VkResult<Instance> {
            let app_info = ApplicationInfo::default()
                .application_name(&c"Redefyning Game Engine")
                .application_version(make_api_version(0, 0, 0, 0))
                .engine_name(&c"Redefyning Game Engine")
                .engine_version(make_api_version(0, 0, 0, 0))
                .api_version(make_api_version(0, 1, 3, 281));

            let create_info = InstanceCreateInfo::default()
                .flags(ENUMERATE_PORTABILITY_KHR)
                .application_info(&app_info);

            unsafe { entry.create_instance(&create_info, None)?}
        }
    }
}