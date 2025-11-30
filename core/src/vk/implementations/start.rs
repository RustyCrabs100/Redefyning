#[path = "../data.rs"]
mod data;

use {ash, ash::vk, std::ffi, std::sync::Arc};

impl data::VkStart {
    pub(crate) fn new() -> Result<Self, vk::Result> {
        let entry = Arc::new(ash::Entry::load()?);
        let instance = Self::create_instance()?;
        Self { entry, instance }
    }

    fn create_instance(
        entry: &Entry,
        application_name: &str,
        // Variant Major Minor Patch
        application_version: (u32, u32, u32, u32),
    ) -> VkResult<Arc<ash::Instance>> {
        let layer_names: &[*const ffi::c_char] = &Self::instance_layers((entry));
        let ext_names: &[*const ffi::c_char] = &Self::instance_extensions(entry);
        let (v, ma, mi, p) = application_version;
        let sanatized_app_name = application_name.replace('\0', "");
        let app_name = ffi::CString::new(sanatized_app_name)
            .expect("Sanatized String somehow wasn't sanatized.");
        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(make_api_version(v, ma, mi, p))
            .engine_name(c"Redefyning")
            .engine_version(make_api_version(0, 0, 0, 0))
            // Vulkan API Version (1.3.286.0) is the max ash Supports
            .api_version(make_api_version(0, 1, 3, 286));
        let create_info = vk::InstanceCreateInfo::default()
            .flags(InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
            .application_info(&app_info)
            .enabled_extension_names(ext_names)
            .enabled_layer_names(layer_names);
        Ok(unsafe { entry.create_instance(&inst_create_info)?.into() })
    }

    fn instance_extensions(entry: &Entry) -> Vec<*const ffi::c_char> {
        let extensions = unsafe {
            entry
                .enumerate_device_extension_properties(None)
                .expect("Failed to enumerate instance extensions")
        };
        let c_strings = extensions
            .iter()
            .map(|ext| {
                let name = ext
                    .extension_name_as_c_str()
                    .expect("Failed to convert Extension name to &CStr");
                ffi::CString::new(
                    name.to_str()
                        .expect("Failed to convert CStr to str")
                        .replace('\0', ""),
                )
                .unwrap()
            })
            .collect::<Vec<CString>>();
        c_strings
            .into_iter()
            .map(|cs| cs.into_raw() as *const ffi::c_char)
            .collect()
    }

    fn instance_layers(entry: &Entry) -> Vec<*const ffi::c_char> {
        let layers = unsafe {
            entry
                .enumerate_instance_layer_properties(None)
                .expect("Failed to enumerate instance layers")
        };
        let cstrings = layers.iter().map(|layer| {
            let name = layer
                .layer_name_as_c_str()
                .expect("Failed to convert Layer Name to CStr");
            CString::new(
                name.to_str()
                    .expect("Failed to convert CStr to str")
                    .replace('\0', ""),
            )
            .unwrap()
        });

        cstrings
            .into_iter()
            .map(|cs| cs.into_raw() as *const ffi::c_char)
            .collect()
    }
}

impl Drop for data::VkStart {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
