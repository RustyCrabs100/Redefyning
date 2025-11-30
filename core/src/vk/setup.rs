#![cfg(feature = "vulkan")]

use {
    crate::{str_to_p_const_c_char, utils::AppState},
    ash::{
        Device, Entry, Instance,
        ext::debug_utils,
        khr,
        khr::surface,
        prelude::VkResult,
        vk,
        vk::{
            ApplicationInfo, DebugUtilsMessengerEXT, DeviceCreateInfo, DeviceQueueCreateInfo,
            InstanceCreateFlags, InstanceCreateInfo, PhysicalDevice,
            PhysicalDeviceColorWriteEnableFeaturesEXT,
            PhysicalDeviceExtendedDynamicState2FeaturesEXT,
            PhysicalDeviceExtendedDynamicState3FeaturesEXT,
            PhysicalDeviceExtendedDynamicStateFeaturesEXT, PhysicalDeviceFeatures,
            PhysicalDeviceVulkan12Features, PhysicalDeviceVulkan13Features, Queue,
            QueueFamilyProperties, QueueFlags, SurfaceKHR, make_api_version,
        },
    },
    ash_window,
    raw_window_handle::{RawDisplayHandle, RawWindowHandle},
    std::{
        ffi::{CStr, CString, c_char},
        sync::{Arc, mpsc::Receiver},
    },
    tokio::sync::mpsc::Receiver as TokioReceiver,
    winit::dpi::PhysicalSize,
};

#[path = "debug_vk.rs"]
mod debug;

pub struct VulkanSetup {
    pub window_communicator: Option<Receiver<AppState>>,
    pub entry: Arc<Entry>,
    pub instance: Arc<Instance>,
    pub surface_functions: Arc<surface::Instance>,
    pub surface: Arc<SurfaceKHR>,
    pub debug_utils_loader: Arc<debug_utils::Instance>,
    pub debug_messenger: Option<DebugUtilsMessengerEXT>,
    pub physical_device: Arc<PhysicalDevice>,
    pub logical_device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
    pub swapchain: Arc<vk::SwapchainKHR>,
    pub swapchain_device: Arc<khr::swapchain::Device>,
}

/// Merge all the other impl VulkanSetup's
impl VulkanSetup {
    pub(crate) fn init_window_communicator(&mut self, communicator: Receiver<AppState>) {
        self.window_communicator = Some(communicator);
    }

    pub(crate) fn new(
        window_handles: (RawDisplayHandle, RawWindowHandle),
        application_name: &str,
        // Variant, Major, Minor, Patch
        application_version: (u32, u32, u32, u32),
        inner_size_reciever: TokioReceiver<PhysicalSize<u32>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Loading Vulkan");
        // Create the Entry Point of Vulkan
        let entry = Arc::new(unsafe { Entry::load()? });
        // Create the vulkan instance
        let instance = Self::instance(&entry, application_name, application_version)?;
        let surface_functions = Self::create_surface_destructor(&entry, &instance);
        let surface = Self::create_surface(&entry, &instance, window_handles);
        let debug_utils_loader = Arc::new(debug_utils::Instance::new(&entry, &instance));
        let debug_messenger = {
            #[cfg(feature = "debug")]
            {
                Some(debug::vk_setup_debug_messenger(&debug_utils_loader))
            }
            #[cfg(not(feature = "debug"))]
            {
                None
            }
        };
        let physical_device = Self::pick_physical_device(&instance);
        let logical_device =
            Self::create_logical_device(&instance, &physical_device, &surface_functions, &surface);
        let graphics_index = Self::find_graphics_queue_family(&instance, &physical_device);
        let graphics_queue = Self::create_graphics_queue(&logical_device, &graphics_index);
        let (swapchain, swapchain_device) = Self::create_swapchain(
            &instance,
            &physical_device,
            &logical_device,
            &surface,
            &surface_functions,
            inner_size_reciever,
        );
        println!("Finished loading Vulkan");
        Ok(Self {
            window_communicator: None,
            entry,
            instance,
            surface_functions,
            surface,
            debug_utils_loader,
            debug_messenger,
            physical_device,
            logical_device,
            graphics_queue,
            swapchain,
            swapchain_device,
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
        let app_name =
            CString::new(application_name).expect("Unable to make a CString out of this &str");
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
        Ok(unsafe { entry.create_instance(&inst_create_info, None)?.into() })
    }

    fn instance_extensions(entry: &Entry) -> Vec<*const c_char> {
        let extensions = unsafe {
            entry
                .enumerate_instance_extension_properties(None)
                .expect("Failed to enumerate instance extensions")
        };
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
        let layers = unsafe {
            entry
                .enumerate_instance_layer_properties()
                .expect("Failed to enumerate instance layers")
        };
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

    fn pick_physical_device(instance: &Instance) -> Arc<PhysicalDevice> {
        // Collect all viable PhysicalDevice's
        let devices = unsafe { instance.enumerate_physical_devices().unwrap() };
        // Panic if no device was found
        if devices.is_empty() {
            panic!("Failed to find GPUs with Vulkan Support");
        }
        // Iterates over all avaliable devices and searches for a good one.
        // Panics if No PhysicalDevice was found
        let physical_device = devices
            .into_iter()
            .find(|&device| unsafe {
                // Collect Device Properties
                let props = instance.get_physical_device_properties(device);
                // Skip this Device if the Vulkan version is less than 1.3.286.0
                if props.api_version < make_api_version(0, 1, 3, 286) {
                    return false;
                }
                // Collect Queue Families
                let queue_families = instance.get_physical_device_queue_family_properties(device);
                // Skip this Device if the queue doesn't support Graphics
                if !(queue_families
                    .iter()
                    .any(|qf| qf.queue_flags.contains(QueueFlags::GRAPHICS)))
                {
                    return false;
                }
                // Collect Device Extensions
                let extensions = instance
                    .enumerate_device_extension_properties(device)
                    .unwrap();
                // Iterate over all the device's extensions.
                // If the device contains all required extensions, it passes.
                Self::REQUIRED_DEVICE_EXTENSIONS.iter().all(|&req| {
                    extensions.iter().any(|ext| {
                        CStr::from_ptr(ext.extension_name.as_ptr()) == CStr::from_ptr(req)
                    })
                })
            })
            .expect("No suitable GPU found");
        Arc::new(physical_device)
    }

    /*
    TODO: Fix this code so that it doesn't panic when the graphics queue doesn't support surface presenting
    Do this by checking if it supports both, and if not, check a different queue.
     */
    fn create_logical_device(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        surface_functions: &surface::Instance,
        surface: &SurfaceKHR,
    ) -> Arc<Device> {
        // Creates Queue Priorities
        let priorities: &[f32] = &[0.0];
        // Collect the Queue with the Graphics Queue
        let graphics_index = Self::find_graphics_queue_family(instance, physical_device);
        // See if the physical device supports Surface Presentation
        let present_support = unsafe {
            surface_functions
                .get_physical_device_surface_support(*physical_device, graphics_index, *surface)
                .expect("Failed to query surface support")
        };
        // Panic if the physical device does not support Surface presentation
        if !present_support {
            panic!("Selected queue family does not support presentation")
        }
        // Collect the physical device's queue family properties.
        let _queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };
        // Create the Logical Device's Queue Info
        let logical_device_queue_create_info = &[DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_index)
            .queue_priorities(priorities)];
        // Get the physical device features
        let physical_device_features: PhysicalDeviceFeatures =
            unsafe { instance.get_physical_device_features(*physical_device) };
        // Collect all good physical device features from Vulkan 1.2
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
        // Collect all good physical device features from Vulkan 1.3
        let mut physical_device_features_vulkan_13 = PhysicalDeviceVulkan13Features::default()
            // VK_KHR_dynamic_rendering
            .dynamic_rendering(true)
            // VK_KHR_synchronization2
            .synchronization2(true)
            // VK_KHR_shader_integer_dot_product
            .shader_integer_dot_product(true);
        // Collect all good physical device features for Extended Dynamic State
        let mut physical_device_features_extended_dynamic_state =
            PhysicalDeviceExtendedDynamicStateFeaturesEXT::default().extended_dynamic_state(true);
        // Collect all good physical device features for Extended Dynamic State 2
        let mut physical_device_features_extended_dynamic_state2 =
            PhysicalDeviceExtendedDynamicState2FeaturesEXT::default()
                .extended_dynamic_state2(true)
                .extended_dynamic_state2_logic_op(true)
                .extended_dynamic_state2_patch_control_points(true);
        // Collect all good physical device features for Extended Dynamic State 3
        let mut physical_device_features_extended_dynamic_state3 =
            PhysicalDeviceExtendedDynamicState3FeaturesEXT::default()
                .extended_dynamic_state3_tessellation_domain_origin(true)
                .extended_dynamic_state3_depth_clamp_enable(true)
                .extended_dynamic_state3_polygon_mode(true)
                .extended_dynamic_state3_rasterization_samples(true)
                .extended_dynamic_state3_sample_mask(true)
                .extended_dynamic_state3_alpha_to_coverage_enable(true)
                .extended_dynamic_state3_alpha_to_one_enable(true)
                .extended_dynamic_state3_logic_op_enable(true)
                .extended_dynamic_state3_color_blend_enable(true)
                .extended_dynamic_state3_color_blend_equation(true)
                .extended_dynamic_state3_color_write_mask(true)
                .extended_dynamic_state3_rasterization_stream(true)
                .extended_dynamic_state3_conservative_rasterization_mode(true)
                .extended_dynamic_state3_extra_primitive_overestimation_size(true)
                .extended_dynamic_state3_depth_clip_enable(true)
                .extended_dynamic_state3_sample_locations_enable(true)
                .extended_dynamic_state3_color_blend_advanced(true)
                .extended_dynamic_state3_provoking_vertex_mode(true)
                .extended_dynamic_state3_line_rasterization_mode(true)
                .extended_dynamic_state3_line_stipple_enable(true)
                .extended_dynamic_state3_depth_clip_negative_one_to_one(true)
                .extended_dynamic_state3_viewport_w_scaling_enable(true)
                .extended_dynamic_state3_viewport_swizzle(true)
                .extended_dynamic_state3_coverage_to_color_enable(true)
                .extended_dynamic_state3_coverage_to_color_location(true)
                .extended_dynamic_state3_coverage_modulation_mode(true)
                .extended_dynamic_state3_coverage_modulation_table_enable(true)
                .extended_dynamic_state3_coverage_modulation_table(true)
                .extended_dynamic_state3_coverage_reduction_mode(true)
                .extended_dynamic_state3_representative_fragment_test_enable(true)
                .extended_dynamic_state3_shading_rate_image_enable(true);
        // Enable the Device Feature for Color Writing
        let mut physical_device_features_color_write_enable =
            PhysicalDeviceColorWriteEnableFeaturesEXT::default().color_write_enable(true);
        // Create the Logical Device's info with all of the features inputed
        let logical_device_create_info = DeviceCreateInfo::default()
            .push_next(&mut physical_device_features_vulkan_12)
            .push_next(&mut physical_device_features_vulkan_13)
            .push_next(&mut physical_device_features_extended_dynamic_state)
            .push_next(&mut physical_device_features_extended_dynamic_state2)
            .push_next(&mut physical_device_features_extended_dynamic_state3)
            .push_next(&mut physical_device_features_color_write_enable)
            .queue_create_infos(logical_device_queue_create_info)
            .enabled_features(&physical_device_features);
        // Create the Logical Device
        unsafe {
            Arc::new(
                instance
                    .create_device(*physical_device, &logical_device_create_info, None)
                    .expect("Failed to create a Vulkan Device"),
            )
        }
    }

    fn create_graphics_queue(device: &Device, graphics_index: &u32) -> Arc<Queue> {
        Arc::new(unsafe { device.get_device_queue(*graphics_index, 0) })
    }

    fn find_graphics_queue_family(instance: &Instance, device: &PhysicalDevice) -> u32 {
        let queue_family_properties: Vec<QueueFamilyProperties> =
            unsafe { instance.get_physical_device_queue_family_properties(*device) };
        queue_family_properties
            .iter()
            .position(|q| q.queue_flags.contains(QueueFlags::GRAPHICS))
            .expect("No Graphics Queue Family with Graphics Support found") as u32
    }
}

/// Vulkan Surface + Swapchain
impl VulkanSetup {
    fn create_surface_destructor(entry: &Entry, instance: &Instance) -> Arc<surface::Instance> {
        Arc::new(surface::Instance::new(entry, instance))
    }

    fn create_surface(
        entry: &Entry,
        instance: &Instance,
        raw_handles: (RawDisplayHandle, RawWindowHandle),
    ) -> Arc<SurfaceKHR> {
        Arc::new(unsafe {
            ash_window::create_surface(entry, instance, raw_handles.0, raw_handles.1, None)
                .expect("Failed to create Vulkan Surface")
        })
    }

    fn create_swapchain(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        device: &Device,
        surface: &SurfaceKHR,
        surface_functions: &surface::Instance,
        inner_size_reciever: TokioReceiver<PhysicalSize<u32>>,
    ) -> (Arc<vk::SwapchainKHR>, Arc<khr::swapchain::Device>) {
        let swapchain_device = khr::swapchain::Device::new(instance, device);
        let swapchain: Arc<vk::SwapchainKHR>;
        unsafe {
            // Get the Surface Capabilities
            let surface_capabilities = surface_functions
                .get_physical_device_surface_capabilities(*physical_device, *surface)
                .expect("Failed to get Surface Capabilities");
            // Get the Surface Format
            let surface_formats = surface_functions
                .get_physical_device_surface_formats(*physical_device, *surface)
                .expect("Failed to get the Surface Formats");
            // Get the Surface Present Mode
            let surface_present_modes = surface_functions
                .get_physical_device_surface_present_modes(*physical_device, *surface)
                .expect("Failed to get the Surface Present Modes");
            // Get the BEST surface format for our needs
            let surface_format = Self::choose_swapchain_surface_format(surface_formats);
            // Choose the BEST present mode for our needs
            let present_mode = Self::choose_swapchain_present_mode(surface_present_modes);
            let swapchain_extent =
                Self::choose_swapchain_extent(&surface_capabilities, inner_size_reciever);
            let mut min_image_count =
                std::cmp::max::<u32>(3u32, surface_capabilities.min_image_count);
            if surface_capabilities.max_image_count > 0
                && min_image_count > surface_capabilities.max_image_count
            {
                min_image_count = surface_capabilities.min_image_count;
            }
            let mut image_count = surface_capabilities.min_image_count + 1;
            if surface_capabilities.max_image_count > 0
                && image_count > surface_capabilities.max_image_count
            {
                image_count = surface_capabilities.max_image_count;
            }

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
                .flags(vk::SwapchainCreateFlagsKHR::default())
                .surface(*surface)
                .min_image_count(image_count)
                .image_format(surface_format.format)
                .image_color_space(surface_format.color_space)
                .image_extent(swapchain_extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(surface_capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .old_swapchain(vk::SwapchainKHR::null());
            let swapchain_current = swapchain_device
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Vulkan Swapchain!");
            let _swapchain_images = swapchain_device.get_swapchain_images(swapchain_current);
            swapchain = Arc::new(swapchain_current);
        }

        (swapchain, Arc::new(swapchain_device))
    }

    fn choose_swapchain_surface_format(formats: Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
        // Iterate over the formats
        for format in &formats {
            /*
            Check if
                - A: The format is SRGB and provides 8 bits to all channels (RGBA)
                AND
                - B: The colorspace is SRGB and Non-linear
            */
            if (format.format == vk::Format::R8G8B8A8_SRGB)
                && (format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            {
                return *format;
            }
        }
        // If all formats fail the above, the first format is fine
        formats[0]
    }

    fn choose_swapchain_present_mode(present_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
        // Iterate over all Present Modes
        for present_mode in present_modes {
            /*
            Catchs all Modes, excluding VK_KHR_shared_presentable_image.
            We don't use this extension, so it's fine.
            We print out warnings based on the present mode avaliable.
            */
            match present_mode {
                vk::PresentModeKHR::MAILBOX => return present_mode,
                vk::PresentModeKHR::FIFO => {
                    println!("VSync Enabled Automatically due to lack of Mailbox Present Mode");
                    return present_mode;
                }
                vk::PresentModeKHR::FIFO_RELAXED => {
                    println!("VSync Enabled Automatically, but less efficient than regular.");
                    println!(
                        "This is due to lacking both Mailbox & FIFO Present Modes being unavaliable"
                    );
                    return present_mode;
                }
                vk::PresentModeKHR::IMMEDIATE => {
                    println!("All Present Modes unavaliable, defaulting to Immediate");
                    eprintln!("Lowest Latency but most screen tearing, be warned!");
                    return present_mode;
                }
                _ => {}
            }
        }

        // If the present mode was not found, it's either invalid or the vec is empty. Panic!
        panic!("No Present Mode Avaliable")
    }

    fn choose_swapchain_extent(
        capabilites: &vk::SurfaceCapabilitiesKHR,
        mut inner_size_reciever: TokioReceiver<PhysicalSize<u32>>,
    ) -> vk::Extent2D {
        if capabilites.current_extent != vk::Extent2D::default().width(u32::MAX).height(u32::MAX) {
            return capabilites.current_extent;
        }

        let size = inner_size_reciever.blocking_recv().unwrap();
        let (width, height) = (size.width, size.height);

        vk::Extent2D::default().width(width).height(height)
    }
}

impl Drop for VulkanSetup {
    // Drop everything in Order in this, or else there's going to be segmentation faults.
    fn drop(&mut self) {
        unsafe {
            self.swapchain_device
                .destroy_swapchain(*self.swapchain, None);
            self.surface_functions.destroy_surface(*self.surface, None);
            self.logical_device.destroy_device(None);
            if let Some(x) = self.debug_messenger {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(x, None);
            }
            // Last Line, nothing comes after this (for your own good)
            self.instance.destroy_instance(None);
        }
    }
}
