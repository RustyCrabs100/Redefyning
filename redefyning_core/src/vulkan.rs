#![cfg(feature = "vulkan")]

use {
    std::ffi::CString,
    std::sync::Arc,
    ash::{
        prelude::VkResult,
        Entry,
        vk::{
            ApplicationInfo,
            InstanceCreateInfo,
            InstanceCreateFlags,
            make_api_version
        },
        Instance
    },
    raw_window_handle::{
        WindowHandle, 
        DisplayHandle,
    }
};

pub struct VulkanSetup {
    entry: Arc<Entry>,
    instance: Arc<Instance>,
} 

impl VulkanSetup {
    pub fn new(
        surface_handles: (
            WindowHandle<'_>, DisplayHandle<'_>
        ),
        application_name: &str,
        // Variant, Major, Minor, Patch
        application_version: (u32, u32, u32, u32),
    ) -> Result<Self, Box<dyn std::error::Error>> {
        println!("{:#?}", surface_handles);
        let entry = Arc::new(unsafe{Entry::load()?});
        let instance = Self::instance(&entry, application_name, application_version)?;
        Ok(Self {
            entry, instance
        })
    }

    fn instance(
        entry: &Entry,
        application_name: &str,
        // Variant, Major, Minor, Patch
        application_version: (u32, u32, u32, u32),
        /* 
        Used for Vulkan Instance Layer's & Extension's
        Commented out until querying for these is done
        layer_names: &[*const i8],
        extension_names: &[*const i8]
        */
    ) -> VkResult<Arc<Instance>> {
        let (v, ma, mi, p) = application_version;
        let app_name = CString::new(application_name).expect("Unable to make a CString out of this &str");
        let app_info = ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(make_api_version(v, ma, mi, p))
            .engine_name(c"Redefyning")
            .engine_version(make_api_version(0, 0, 0, 0))
            // Vulkan API Version (1.3.286.0 is the Max ash supports)
            .api_version(make_api_version(0, 1, 3, 286));
        let inst_create_info = InstanceCreateInfo::default()
            .application_info(&app_info)
            // .enabled_layer_names(layer_names)
            // .enabled_extension_names(extension_names)
            .flags(InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR);
        Ok(unsafe { entry.create_instance(&inst_create_info, None)?.into()})
    }
}

impl Drop for VulkanSetup {
    fn drop(&mut self) {
        unsafe {
            // Last Line, nothing comes after this (for your own good)
            self.instance.destroy_instance(None);
        }
    }
}