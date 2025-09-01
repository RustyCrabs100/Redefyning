#![cfg(feature = "vulkan")]

use {
    std::{
        ffi::{
            CString,
            c_char,
        },
        sync::Arc,    
    },    
    ash::{
        prelude::VkResult,
        Entry,
        vk::{
            ApplicationInfo,
            InstanceCreateInfo,
            InstanceCreateFlags,
            make_api_version,
            DebugUtilsMessengerEXT,
        },
        Instance,
        ext::debug_utils,
    },
    raw_window_handle::{
        WindowHandle, 
        DisplayHandle,
    }
};

#[path = "debug_vk.rs"]
mod debug;

pub struct VulkanSetup {
    entry: Arc<Entry>,
    instance: Arc<Instance>,
    debug_utils_loader: Arc<debug_utils::Instance>,
    debug_messenger: Option<DebugUtilsMessengerEXT>,
} 

impl VulkanSetup {
    pub(crate) fn new(
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
        let debug_utils_loader = Arc::new(debug_utils::Instance::new(&entry, &instance));
        let debug_messenger = {
            #[cfg(feature = "debug")]
            {
                Some(debug::vk_setup_debug_messenger(&debug_utils_loader))
            }
            #[cfg(not(feature = "debug"))] {
                None
            }
        };
        Ok(Self {
            entry, instance,
            debug_utils_loader,
            #[cfg(feature = "debug")]
            debug_messenger
        })
    }

    fn instance(
        entry: &Entry,
        application_name: &str,
        // Variant, Major, Minor, Patch
        application_version: (u32, u32, u32, u32),
    ) -> VkResult<Arc<Instance>> {
        let layer_names: &[*const c_char] = &Self::instance_layers(entry);
        let ext_names: &[*const c_char] = &Self::instance_extensions(entry);
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
            .flags(InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
            .application_info(&app_info)
            .enabled_layer_names(layer_names)
            .enabled_extension_names(ext_names);
        Ok(unsafe { entry.create_instance(&inst_create_info, None)?.into()})
    }

    fn instance_extensions(entry: &Entry) -> Vec<*const c_char> {
        let extensions = unsafe { entry
            .enumerate_instance_extension_properties(None)
            .expect("Failed to enumerate instance extensions")};
        #[cfg(feature = "debug")]
        let cstrings: Vec<CString> = extensions
            .iter()
            .map(|ext| {
                let name = ext.extension_name_as_c_str().unwrap();
                CString::new(name.to_str().unwrap()).unwrap()
            })
            .collect();
        cstrings
            .into_iter()
            .map(|cs| cs.into_raw() as *const c_char)
            .collect()
    }

    fn instance_layers(entry: &Entry) -> Vec<*const c_char> {
        let layers = unsafe { entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layers")};
        let cstrings: Vec<CString> = layers
            .iter()
            .map(|layer| {
                let name = layer.layer_name_as_c_str().unwrap();
                CString::new(name.to_str().unwrap()).unwrap()
            })
            .collect();

        cstrings
            .into_iter()
            .map(|cs| cs.into_raw() as *const c_char)
            .collect()
    }
}

impl Drop for VulkanSetup {
    // Drop everything in Order in this, or else there's going to be segmentation faults.
    fn drop(&mut self) {
        unsafe {
            if let Some(x) = self.debug_messenger {
                self.debug_utils_loader.destroy_debug_utils_messenger(x, None);
            }
            // Last Line, nothing comes after this (for your own good)
            self.instance.destroy_instance(None);
        }
    }
}