#[path = "../data.rs"]
mod data;

use {ash, ash::vk, std::ffi, std::sync::Arc};

impl data::VkStart {
    fn new() -> Result<Self, vk::Result> {
        let entry = Arc::new(ash::Entry::load()?);
        let instance = Self::create_instance();
        Self { entry, instance }
    }

    fn create_instance(
        entry: &Entry,
        application_name: &str,
        // Variant Major Minor Patch
        application_version: (u32, u32, u32, u32),
    ) -> Arc<ash::Instance> {
        let layer_names: &[*const ffi::c_char] = &[];
        let ext_names: &[*const ffi::c_char] = &[];
        let (v, ma, mi, p) = application_version;
        let app_name = match ffi::CString::new(application_name) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("{:?}", e);
            }
        };
        todo!("create_instance has not yet been fully recoded.")
    }
}

impl Drop for data::VkStart {
    fn drop(&mut self) {}
}
