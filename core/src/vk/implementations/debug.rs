#![cfg(all(feature = "debug", feature = "vulkan", debug_assertions))]

use {
    crate::{logln, utils::debug_logln, vk::data},
    ash::{ext, vk},
    error_stack::{IntoReport as _, Report},
    std::ffi::c_void,
};

const VK_DEBUG_LOCATION: &'static str = "VULKAN VALIDATION";

const VALIDATION: bool = const || {
    #[cfg(all(debug_assertions, feature = "debug"))]
    {
        return true;
    }
    false
};

const VALIDATION_LAYER: &'static str = "VK_LAYER_KHRONOS_validation";

impl data::VkDebug {
    unsafe extern "system" fn vk_debug_callback(
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        validation_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _: *mut c_void,
    ) -> vk::Bool32 {
        match severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => return vk::FALSE,
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => return vk::FALSE,
            _ => {}
        }
        match validation_type {
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => return vk::FALSE,
            _ => {}
        }

        debug_logln!(
            "Type: {:?}, Severity: {:?}, Message: {:?}",
            validation_type,
            severity,
            unsafe { (*p_callback_data).p_message },
            VK_DEBUG_LOCATION
        );

        vk::FALSE
    }

    fn new(entry: &data::VkStart) -> Result<Self, Report<std::error::Error>> {
        let debug_utils_loader = ext::debug_utils::Instance::new(*entry.entry, *entry.instance);
        if !VALIDATION {
            logln!(
                "VkDebug::new was called when debugging was disabled, how did you do that??",
                VK_DEBUG_LOCATION,
            );
            return Err(Report::new("Unnecessary VkDebug::new Call")
                .attach("debug_assertions was disabled")
                .attach("The \"debug\" feature was disabled"));
        }
        let severity_flags = vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
        let type_flags = vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(severity_flags)
            .message_type(type_flags)
            .pfn_user_callback(Some(vk_debug_callback));
        let messenger = match debug_utils_loader.create_debug_utils_messenger(&debug_info, None) {
            Ok(x) => x,
            Err(x) => Report::new(x).attach("Failed to create VkDebug.messenger."),
        };
        Self {
            loader: debug_utils_loader,
            messenger,
        }
    }
}

impl Drop for data::VkDebug {
    fn drop(&mut self) {
        if let Some(x) = self.messenger {
            self.loader.destroy_debug_utils_messenger(x, None);
        }
    }
}
