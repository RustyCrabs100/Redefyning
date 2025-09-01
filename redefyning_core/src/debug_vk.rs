// This file is only intended to contain debug utils for vulkan.
// This file is only inteded to be used by vulkan.rs or any related rust files.
#![cfg(feature = "debug")]
#![cfg(feature = "vulkan")]

use ash::vk;
use std::ffi::c_void;

/// Constant Function to check if the debug feature is enabled, if so, return true
const fn debug() -> bool {
    #[cfg(feature = "debug")] {
        return true;
    }
    return false;
}
// Validation Supported? 
const VALIDATION: bool = debug();
// Validation Layer Name
const VALIDATION_LAYER: &'static str = "VK_LAYER_KHRONOS_validation";

unsafe extern "system" fn vk_debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    validation_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void
) -> vk::Bool32 {
    match severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => return vk::FALSE,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => return vk::FALSE,
        _ => {},
    };
    match validation_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => return vk::FALSE,
        _ => {},
    };
    println!(
        "Validation Layer: Type: {:?}, Severity: {:?} Message: {:?}",
        validation_type, severity, (*p_callback_data).p_message,
    );


    return vk::FALSE;
}

pub(crate) fn vk_setup_debug_messenger(
    debug_utils_loader: &ash::ext::debug_utils::Instance
) -> vk::DebugUtilsMessengerEXT {
    if !VALIDATION {
        panic!("Validation not allowed but validation messenger setup function called.");
    }

    let severity_flags = vk::DebugUtilsMessageSeverityFlagsEXT::WARNING 
        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
    let type_flags = vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION 
        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(severity_flags)
        .message_type(type_flags)
        .pfn_user_callback(Some(vk_debug_callback));   
    unsafe {debug_utils_loader.create_debug_utils_messenger(&debug_info, None).unwrap()}
}