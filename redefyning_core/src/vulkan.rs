#![cfg(feature = "vulkan")]

use {
    std::{
        ffi::{
            CString,
            CStr,
            c_char,
        },
        sync::Arc,    
    },    
    ash::{
        prelude::VkResult,
        Entry,
        vk::{
            TRUE,
            FALSE,
            ApplicationInfo,
            InstanceCreateInfo,
            InstanceCreateFlags,
            make_api_version,
            DebugUtilsMessengerEXT,
            PhysicalDevice,
            QueueFlags,
            QueueFamilyProperties,
            DeviceCreateInfo,
            DeviceQueueCreateInfo,
            PhysicalDeviceFeatures,
            PhysicalDeviceVulkan12Features,
            PhysicalDeviceVulkan13Features,
            PhysicalDeviceExtendedDynamicStateFeaturesEXT,
            PhysicalDeviceExtendedDynamicState2FeaturesEXT,
            PhysicalDeviceExtendedDynamicState3FeaturesEXT,
            PhysicalDeviceColorWriteEnableFeaturesEXT,
            Bool32,
            Queue
        },
        Instance,
        ext::debug_utils,
        Device,
    },
    raw_window_handle::{
        WindowHandle, 
        DisplayHandle,
    }
};

#[path = "debug_vk.rs"]
mod debug;

// Don't give this a string with a Null terminator
macro_rules! str_to_p_const_c_char {
    ($s:expr) => {{
        const BYTES: &[u8] = concat!($s, "\0").as_bytes();
        const REF_CSTR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(BYTES)};
        REF_CSTR.as_ptr()
    }};
}


pub struct VulkanSetup {
    entry: Arc<Entry>,
    instance: Arc<Instance>,
    debug_utils_loader: Arc<debug_utils::Instance>,
    debug_messenger: Option<DebugUtilsMessengerEXT>,
    physical_device: Arc<PhysicalDevice>,
    logical_device: Arc<Device>, 
    graphics_queue: Arc<Queue>,
} 

/// Merge all the other impl VulkanSetup's
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
        let physical_device = Self::pick_physical_device(&instance);
        let logical_device = Self::create_logical_device(&instance, &physical_device);
        let graphics_index = Self::find_graphics_queue_family(&instance, &physical_device);
        let graphics_queue = Self::create_graphics_queue(&logical_device, &graphics_index);
        Ok(Self {
            entry, instance,
            debug_utils_loader,
            #[cfg(feature = "debug")]
            debug_messenger,
            physical_device,
            logical_device,
            graphics_queue
        })
    }
}

/// Vulkan Instance
impl VulkanSetup {
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

/// Device And Queue Families
impl VulkanSetup {
    // Some of these extensions are automatically included in Core, but i'm adding them here
    // For improved readability
    const REQUIRED_DEVICE_EXTENSIONS: &[*const c_char] = &[
        // Swapchain Creation
        str_to_p_const_c_char!("VK_KHR_swapchain"),
        // Added Dynamic Rendering, Flexible Pipelines
        str_to_p_const_c_char!("VK_KHR_dynamic_rendering"),
        // Graph Frame-friendly Sync
        str_to_p_const_c_char!("VK_KHR_timeline_semaphore"),
        // Cleaner, less error-prone sync API
        str_to_p_const_c_char!("VK_KHR_synchronization2"),
        // Change Cull Mode, Topology, Blend State, etc. at draw time.
        str_to_p_const_c_char!("VK_EXT_extended_dynamic_state"),
        str_to_p_const_c_char!("VK_EXT_extended_dynamic_state2"),
        str_to_p_const_c_char!("VK_EXT_extended_dynamic_state3"),
        // Dynamic Vertex Bindings/Attibutes
        str_to_p_const_c_char!("VK_EXT_vertex_input_dynamic_state"),
        // Per-attachment color write control
        str_to_p_const_c_char!("VK_EXT_color_write_enable"),
        // Bindless textures/buffers, sparse descriptor arrays
        str_to_p_const_c_char!("VK_EXT_descriptor_indexing"),
        // GPU pointers for bindless/resource-descriptor-less acces
        str_to_p_const_c_char!("VK_KHR_buffer_device_address"),
        // Query VRAM Usage
        str_to_p_const_c_char!("VK_EXT_memory_budget"),
        // Lower-percision math for bandwidth saving
        str_to_p_const_c_char!("VK_KHR_shader_float16_int8"),
        // Integer dot products for ML/physics.
        str_to_p_const_c_char!("VK_KHR_shader_integer_dot_product"),
        // Negative viewport height, minor fixes
        str_to_p_const_c_char!("VK_KHR_maintenance1"),
        // New image layouts, subpass dependency improvements
        str_to_p_const_c_char!("VK_KHR_maintenance2"),
        // Descriptor query improvements
        str_to_p_const_c_char!("VK_KHR_maintenance3"),
        // Device memory null handles, improved queries,
        str_to_p_const_c_char!("VK_KHR_maintenance4"),
        // Pipeline creation feedback, descriptor indexing improvements
        str_to_p_const_c_char!("VK_KHR_maintenance5"),
        // SPIR-V 1.4 Support
        str_to_p_const_c_char!("VK_KHR_spirv_1_4"),
        // Extended Renderpass Creation
        str_to_p_const_c_char!("VK_KHR_create_renderpass2"),
    ];

    fn pick_physical_device(
        instance: &Instance,
    ) -> Arc<PhysicalDevice> {
        let devices = unsafe { instance.enumerate_physical_devices().unwrap()};
        if devices.is_empty() {
            panic!("Failed to find GPUs with Vulkan Support");
        }
        let physical_device = devices.into_iter()
            .find( |&device| unsafe {
                let props = instance.get_physical_device_properties(device);
                if props.api_version < make_api_version(0, 1, 3, 286) {
                    panic!(
                        "Your Vulkan API Version is not supported. Your current API version: {}, needed API version: {}",
                        props.api_version, make_api_version(0, 1, 3, 286)
                    );
                }
                let queue_families = instance.get_physical_device_queue_family_properties(device);
                if !(queue_families.iter().any(|qf| qf.queue_flags.contains(QueueFlags::GRAPHICS))) {
                    panic!("No queue family supports Graphics");
                }
                let extensions = instance.enumerate_device_extension_properties(device).unwrap();
                return Self::REQUIRED_DEVICE_EXTENSIONS.iter().all(|&req| {
                    extensions.iter().any(|ext| {
                        CStr::from_ptr(ext.extension_name.as_ptr()) == CStr::from_ptr(req)
                    })
                });
            }).expect("No suitable GPU found");
        Arc::new(physical_device)    
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: &PhysicalDevice,
    ) -> Arc<Device> {
        let priorities: &[f32] = &[0.0];
        let graphics_index = Self::find_graphics_queue_family(
            instance,
            physical_device
        );   
        let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(*physical_device)};
        let logical_device_queue_create_info = &[DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_index)
            .queue_priorities(priorities)];
        let physical_device_features: PhysicalDeviceFeatures = unsafe {
            instance.get_physical_device_features(*physical_device)
        }; 
        let mut physical_device_features_vulkan_12 = PhysicalDeviceVulkan12Features::default()
            // VK_KHR_timeline_semaphore
            .timeline_semaphore(true)
            // VK_EXT_descriptor_indexing
            .shader_input_attachment_array_dynamic_indexing(true)
            .shader_uniform_texel_buffer_array_dynamic_indexing(true)
            .shader_storage_texel_buffer_array_dynamic_indexing(true)
            .shader_uniform_buffer_array_non_uniform_indexing(true)
            .shader_sampled_image_array_non_uniform_indexing(true)
            .shader_storage_buffer_array_non_uniform_indexing(true)
            .shader_storage_image_array_non_uniform_indexing(true)
            .shader_input_attachment_array_non_uniform_indexing(true)
            .descriptor_binding_uniform_buffer_update_after_bind(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_storage_image_update_after_bind(true)
            .descriptor_binding_storage_buffer_update_after_bind(true)
            .descriptor_binding_uniform_texel_buffer_update_after_bind(true)
            .descriptor_binding_storage_texel_buffer_update_after_bind(true)
            .descriptor_binding_update_unused_while_pending(true)
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_variable_descriptor_count(true)
            .runtime_descriptor_array(true)
            // VK_KHR_buffer_device_address
            .buffer_device_address(true)
            // VK_KHR_shader_float16_int8
            .shader_float16(true)
            .shader_int8(true);
        let mut physical_device_features_vulkan_13 = PhysicalDeviceVulkan13Features::default()
            // VK_KHR_dynamic_rendering
            .dynamic_rendering(true)
            // VK_KHR_synchronization2
            .synchronization2(true)
            // VK_KHR_shader_integer_dot_product
            .shader_integer_dot_product(true);
        let mut physical_device_features_extended_dynamic_state = PhysicalDeviceExtendedDynamicStateFeaturesEXT::default()
            .extended_dynamic_state(true);
        let mut physical_device_features_extended_dynamic_state2 = PhysicalDeviceExtendedDynamicState2FeaturesEXT::default()
            .extended_dynamic_state2(true)
            .extended_dynamic_state2_logic_op(true)
            .extended_dynamic_state2_patch_control_points(true);
        let mut physical_device_features_extended_dynamic_state3 = PhysicalDeviceExtendedDynamicState3FeaturesEXT::default();
        let ptr = &mut physical_device_features_extended_dynamic_state3 as *mut PhysicalDeviceExtendedDynamicState3FeaturesEXT as *mut Bool32;
        let count = std::mem::size_of::<PhysicalDeviceExtendedDynamicState3FeaturesEXT>() / std::mem::size_of::<Bool32>();
        unsafe {
            for i in 0..count {
                *ptr.add(i) = TRUE;
            }
        }
        let mut physical_device_features_color_write_enable = PhysicalDeviceColorWriteEnableFeaturesEXT::default()
            .color_write_enable(true);
        let logical_device_create_info = DeviceCreateInfo::default()
            .push_next(&mut physical_device_features_vulkan_12)
            .push_next(&mut physical_device_features_vulkan_13)
            .push_next(&mut physical_device_features_extended_dynamic_state)
            .push_next(&mut physical_device_features_extended_dynamic_state2)
            .push_next(&mut physical_device_features_extended_dynamic_state3)
            .push_next(&mut physical_device_features_color_write_enable)
            .queue_create_infos(logical_device_queue_create_info)
            .enabled_features(&physical_device_features);
        unsafe { Arc::new(instance.create_device(*physical_device, &logical_device_create_info, None).expect("Failed to create a Vulkan Device"))}
    }

    fn create_graphics_queue(
        device: &Device,
        graphics_index: &u32
    ) -> Arc<Queue> {
        return Arc::new(unsafe {device.get_device_queue(*graphics_index, 0)});
    }

    fn find_graphics_queue_family(
        instance: &Instance,
        device: &PhysicalDevice
    ) -> u32 {
        let queue_family_properties: Vec<QueueFamilyProperties> = unsafe { instance.get_physical_device_queue_family_properties(*device)};
        queue_family_properties
            .iter()
            .position(|q| q.queue_flags.contains(QueueFlags::GRAPHICS))
            .expect("No Graphics Queue Family with Graphics Support found") as u32
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