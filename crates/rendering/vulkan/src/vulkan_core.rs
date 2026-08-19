//! Vulkan core initialization and device management

use std::ffi::CStr;
use std::os::raw::c_char;
use ash::vk;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VulkanError {
    #[error("Vulkan error: {0}")]
    Vk(#[from] vk::Result),
    #[error("Loading error: {0}")]
    Loading(#[from] ash::LoadingError),
    #[error("Nul error: {0}")]
    Nul(#[from] std::ffi::NulError),
    #[error("Instance creation failed: {0}")]
    Instance(String),
    #[error("Device creation failed: {0}")]
    Device(String),
    #[error("Swapchain creation failed: {0}")]
    Swapchain(String),
    #[error("Surface not supported")]
    SurfaceNotSupported,
    #[error("Required extension not available: {0}")]
    MissingExtension(String),
    #[error("Required feature not available: {0}")]
    MissingFeature(String),
}

/// Required instance extensions
pub const REQUIRED_INSTANCE_EXTENSIONS: &[&CStr] = &[
    #[cfg(target_os = "linux")]
    ash::extensions::khr::XlibSurface::name(),
    #[cfg(target_os = "linux")]
    ash::extensions::khr::WaylandSurface::name(),
    #[cfg(target_os = "windows")]
    ash::extensions::khr::Win32Surface::name(),
    ash::extensions::khr::Surface::name(),
    ash::extensions::ext::DebugUtils::name(),
];

/// Required device extensions
pub const REQUIRED_DEVICE_EXTENSIONS: &[&CStr] = &[
    ash::extensions::khr::Swapchain::name(),
    ash::extensions::khr::DynamicRendering::name(),
    ash::extensions::khr::Synchronization2::name(),
    ash::extensions::khr::BufferDeviceAddress::name(),
];

/// Validation layers
pub const VALIDATION_LAYERS: &[&CStr] = &[
    c"VK_LAYER_KHRONOS_validation",
];

/// Vulkan context holding instance, device, and core resources
pub struct VulkanContext {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub graphics_queue: vk::Queue,
    pub compute_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    pub graphics_queue_family: u32,
    pub compute_queue_family: u32,
    pub transfer_queue_family: u32,
    pub properties: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    pub debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,
}

impl VulkanContext {
    /// Create new Vulkan context with validation support
    pub fn new(enable_validation: bool, app_name: &str, engine_name: &str) -> Result<Self, VulkanError> {
        // Load Vulkan entry point
        let entry = unsafe { ash::Entry::load()? };

        // Create instance
        let instance = Self::create_instance(&entry, enable_validation, app_name, engine_name)?;

        // Setup debug messenger if validation enabled
        let (debug_utils_loader, debug_messenger) = if enable_validation {
            let (loader, messenger) = Self::setup_debug_messenger(&entry, &instance)?;
            (Some(loader), Some(messenger))
        } else {
            (None, None)
        };

        // Pick physical device
        let physical_device = Self::pick_physical_device(&instance)?;

        // Get device properties and features
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.get_physical_device_features(physical_device) };
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

        // Find queue families
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let (graphics_queue_family, compute_queue_family, transfer_queue_family) = 
            Self::find_queue_families(&queue_families)?;

        // Create logical device
        let device = Self::create_logical_device(
            &instance,
            physical_device,
            graphics_queue_family,
            compute_queue_family,
            transfer_queue_family,
            enable_validation,
        )?;

        // Get queues
        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family, 0) };
        let compute_queue = unsafe { device.get_device_queue(compute_queue_family, 0) };
        let transfer_queue = unsafe { device.get_device_queue(transfer_queue_family, 0) };

        Ok(Self {
            entry,
            instance,
            physical_device,
            device,
            graphics_queue,
            compute_queue,
            transfer_queue,
            graphics_queue_family,
            compute_queue_family,
            transfer_queue_family,
            properties,
            features,
            memory_properties,
            debug_messenger,
            debug_utils_loader,
        })
    }

    fn create_instance(
        entry: &ash::Entry,
        enable_validation: bool,
        app_name: &str,
        engine_name: &str,
    ) -> Result<ash::Instance, VulkanError> {
        let app_name_c = std::ffi::CString::new(app_name)?;
        let engine_name_c = std::ffi::CString::new(engine_name)?;

        // Check available extensions
        let available_extensions = unsafe { entry.enumerate_instance_extension_properties(None)? };
        for required in REQUIRED_INSTANCE_EXTENSIONS {
            let found = available_extensions.iter().any(|ext| {
                let ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                ext_name == *required
            });
            if !found {
                return Err(VulkanError::MissingExtension(
                    required.to_string_lossy().into_owned()
                ));
            }
        }

        // Check validation layers if enabled
        let mut enabled_layers = Vec::new();
        if enable_validation {
            let available_layers = unsafe { entry.enumerate_instance_layer_properties()? };
            for layer in VALIDATION_LAYERS {
                let found = available_layers.iter().any(|l| {
                    let layer_name = unsafe { CStr::from_ptr(l.layer_name.as_ptr()) };
                    layer_name == *layer
                });
                if found {
                    enabled_layers.push(layer.as_ptr());
                } else {
                    return Err(VulkanError::MissingExtension(
                        format!("Validation layer not found: {}", layer.to_string_lossy())
                    ));
                }
            }
        }

        let app_info = vk::ApplicationInfo {
            p_application_name: app_name_c.as_ptr(),
            application_version: vk::make_api_version(0, 0, 1, 0),
            p_engine_name: engine_name_c.as_ptr(),
            engine_version: vk::make_api_version(0, 0, 1, 0),
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };

        let extension_names: Vec<*const c_char> = REQUIRED_INSTANCE_EXTENSIONS
            .iter()
            .map(|s| s.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_layer_count: enabled_layers.len() as u32,
            pp_enabled_layer_names: if enabled_layers.is_empty() { std::ptr::null() } else { enabled_layers.as_ptr() },
            ..Default::default()
        };

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        Ok(instance)
    }

    fn setup_debug_messenger(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Result<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT), VulkanError> {
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

        let create_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING,
            pfn_user_callback: Some(vulkan_debug_callback),
            ..Default::default()
        };

        let debug_messenger = unsafe { debug_utils_loader.create_debug_utils_messenger(&create_info, None)? };

        Ok((debug_utils_loader, debug_messenger))
    }

    fn pick_physical_device(instance: &ash::Instance) -> Result<vk::PhysicalDevice, VulkanError> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        
        if devices.is_empty() {
            return Err(VulkanError::Device("No physical devices found".to_string()));
        }

        // Score and pick the best device (prefer discrete GPU)
        let mut best_device = None;
        let mut best_score = 0;

        for device in devices {
            let props = unsafe { instance.get_physical_device_properties(device) };
            let features = unsafe { instance.get_physical_device_features(device) };

            // Check required features
            if features.geometry_shader == 0 {
                continue;
            }

            // Check device extensions
            let extensions = unsafe { instance.enumerate_device_extension_properties(device)? };
            let has_required = REQUIRED_DEVICE_EXTENSIONS.iter().all(|required| {
                extensions.iter().any(|ext| {
                    let ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                    ext_name == *required
                })
            });
            if !has_required {
                continue;
            }

            // Score device
            let mut score = 0;
            if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                score += 1000;
            } else if props.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
                score += 100;
            }
            score += props.limits.max_image_dimension2_d as u32;

            if score > best_score {
                best_score = score;
                best_device = Some(device);
            }
        }

        best_device.ok_or(VulkanError::Device("No suitable physical device found".to_string()))
    }

    fn find_queue_families(
        queue_families: &[vk::QueueFamilyProperties],
    ) -> Result<(u32, u32, u32), VulkanError> {
        let mut graphics_family = None;
        let mut compute_family = None;
        let mut transfer_family = None;

        for (i, family) in queue_families.iter().enumerate() {
            let flags = family.queue_flags;
            
            if flags.contains(vk::QueueFlags::GRAPHICS) && graphics_family.is_none() {
                graphics_family = Some(i as u32);
            }
            if flags.contains(vk::QueueFlags::COMPUTE) && compute_family.is_none() && !flags.contains(vk::QueueFlags::GRAPHICS) {
                compute_family = Some(i as u32);
            }
            if flags.contains(vk::QueueFlags::TRANSFER) && transfer_family.is_none() && 
               !flags.contains(vk::QueueFlags::GRAPHICS) && !flags.contains(vk::QueueFlags::COMPUTE) {
                transfer_family = Some(i as u32);
            }
        }

        // Fallback: use graphics for compute/transfer if dedicated not found
        let graphics = graphics_family.ok_or(VulkanError::Device("No graphics queue family".to_string()))?;
        let compute = compute_family.unwrap_or(graphics);
        let transfer = transfer_family.unwrap_or(graphics);

        Ok((graphics, compute, transfer))
    }

    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        graphics_queue_family: u32,
        compute_queue_family: u32,
        transfer_queue_family: u32,
        enable_validation: bool,
    ) -> Result<ash::Device, VulkanError> {
        let mut unique_queue_families = vec![graphics_queue_family, compute_queue_family, transfer_queue_family];
        unique_queue_families.sort();
        unique_queue_families.dedup();

        let queue_priorities = [1.0f32];
        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_families
            .iter()
            .map(|&family| {
                vk::DeviceQueueCreateInfo {
                    queue_family_index: family,
                    queue_count: 1,
                    p_queue_priorities: &queue_priorities[0],
                    ..Default::default()
                }
            })
            .collect();

        // Required features
        let mut features = vk::PhysicalDeviceFeatures::default();
        features.geometry_shader = 1;
        features.sampler_anisotropy = 1;
features.sample_rate_shading = 1;

        // Vk 1.3 features
        let mut features_13 = vk::PhysicalDeviceVulkan13Features {
            dynamic_rendering: 1,
            synchronization2: 1,
            maintenance4: 1,
            ..Default::default()
        };

        // Buffer device address
        let mut buffer_device_address = vk::PhysicalDeviceBufferDeviceAddressFeatures {
            buffer_device_address: 1,
            ..Default::default()
        };

        // Chain the feature structs: DeviceCreateInfo -> Vk13Features -> BufferDeviceAddressFeatures
        buffer_device_address.p_next = std::ptr::null_mut();
        features_13.p_next = &mut buffer_device_address as *mut _ as *mut std::ffi::c_void;

        let extension_names: Vec<*const c_char> = REQUIRED_DEVICE_EXTENSIONS
            .iter()
            .map(|s| s.as_ptr())
            .collect();

        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            // Device layers are deprecated - use 0
            enabled_layer_count: 0,
            pp_enabled_layer_names: std::ptr::null(),
            p_enabled_features: &features,
            p_next: &mut features_13 as *mut _ as *mut std::ffi::c_void,
            ..Default::default()
        };

        let device = unsafe { instance.create_device(physical_device, &create_info, None)? };
        Ok(device)
    }

    /// Wait for device to be idle
    pub fn wait_idle(&self) -> Result<(), VulkanError> {
        unsafe { self.device.device_wait_idle()? };
        Ok(())
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            if let (Some(loader), Some(messenger)) = (self.debug_utils_loader.take(), self.debug_messenger.take()) {
                loader.destroy_debug_utils_messenger(messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

/// Debug callback function
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message = if callback_data.p_message.is_null() {
        "".to_string()
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy().into_owned()
    };

    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "VERBOSE",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "INFO",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "WARNING",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "ERROR",
        _ => "UNKNOWN",
    };

    let msg_type = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "GENERAL",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "VALIDATION",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "PERFORMANCE",
        vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => "DEVICE_ADDRESS_BINDING",
        _ => "UNKNOWN",
    };

    match severity {
        "ERROR" => tracing::error!("[Vulkan {} {}] {}", msg_type, severity, message),
        "WARNING" => tracing::warn!("[Vulkan {} {}] {}", msg_type, severity, message),
        "INFO" => tracing::info!("[Vulkan {} {}] {}", msg_type, severity, message),
        _ => tracing::debug!("[Vulkan {} {}] {}", msg_type, severity, message),
    }

    vk::FALSE
}

/// Swapchain management
pub struct Swapchain {
    pub loader: ash::extensions::khr::Swapchain,
    pub handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub present_mode: vk::PresentModeKHR,
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        entry: &ash::Entry,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
        window_size: vk::Extent2D,
        old_swapchain: Option<vk::SwapchainKHR>,
        vsync: bool,
    ) -> Result<Self, VulkanError> {
        let loader = ash::extensions::khr::Swapchain::new(instance, device);
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        // Get surface capabilities
        let surface_caps = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
        };

        // Get surface formats
        let surface_formats = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)?
        };

        // Get present modes
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
        };

        // Choose format
        let format = Self::choose_swapchain_format(&surface_formats);
        
        // Choose present mode
        let present_mode = Self::choose_present_mode(&present_modes, vsync);

        // Choose extent
        let extent = Self::choose_extent(&surface_caps, window_size);

        // Determine image count
        let mut image_count = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
            image_count = surface_caps.max_image_count;
        }

        // Create swapchain
        let create_info = vk::SwapchainCreateInfoKHR {
            surface,
            min_image_count: image_count,
            image_format: format,
            image_color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: surface_caps.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: 1,
            old_swapchain: old_swapchain.unwrap_or(vk::SwapchainKHR::null()),
            ..Default::default()
        };

        let handle = unsafe { loader.create_swapchain(&create_info, None)? };

        // Get swapchain images
        let images = unsafe { loader.get_swapchain_images(handle)? };

        // Create image views
        let image_views = images.iter().map(|&image| {
            let view_info = vk::ImageViewCreateInfo {
                image,
                view_type: vk::ImageViewType::TYPE_2D,
                format,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };
            unsafe { device.create_image_view(&view_info, None) }.unwrap()
        }).collect();

        Ok(Self {
            loader,
            handle,
            images,
            image_views,
            format,
            extent,
            present_mode,
        })
    }

    fn choose_swapchain_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::Format {
        // Prefer SRGB format
        for format in available_formats {
            if format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                return format.format;
            }
        }
        available_formats[0].format
    }

    fn choose_present_mode(available_modes: &[vk::PresentModeKHR], vsync: bool) -> vk::PresentModeKHR {
        if vsync {
            for mode in available_modes {
                if *mode == vk::PresentModeKHR::FIFO {
                    return *mode;
                }
            }
        } else {
            for mode in available_modes {
                if *mode == vk::PresentModeKHR::MAILBOX {
                    return *mode;
                }
            }
            for mode in available_modes {
                if *mode == vk::PresentModeKHR::IMMEDIATE {
                    return *mode;
                }
            }
        }
        vk::PresentModeKHR::FIFO
    }

    fn choose_extent(capabilities: &vk::SurfaceCapabilitiesKHR, window_size: vk::Extent2D) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: window_size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: window_size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    pub fn acquire_next_image(
        &self,
        device: &ash::Device,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
        timeout: u64,
    ) -> Result<(u32, bool), VulkanError> {
        let result = unsafe { self.loader.acquire_next_image(self.handle, timeout, semaphore, fence) };
        match result {
            Ok((index, suboptimal)) => Ok((index, suboptimal)),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Ok((0, true)),
            Err(e) => Err(VulkanError::Vk(e)),
        }
    }

    pub fn present(
        &self,
        queue: vk::Queue,
        image_index: u32,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<bool, VulkanError> {
        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: &self.handle,
            p_image_indices: &image_index,
            ..Default::default()
        };

        let result = unsafe { self.loader.queue_present(queue, &present_info) };
        match result {
            Ok(_) => Ok(false),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => Ok(true),
            Err(e) => Err(VulkanError::Vk(e)),
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        // Note: In practice, device reference needed for cleanup
    }
}

/// Frame synchronization primitives
pub struct FrameSync {
    pub image_available: Vec<vk::Semaphore>,
    pub render_finished: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
    pub current_frame: usize,
}

impl FrameSync {
    pub fn new(device: &ash::Device, frames_in_flight: usize) -> Result<Self, VulkanError> {
        let mut image_available = Vec::with_capacity(frames_in_flight);
        let mut render_finished = Vec::with_capacity(frames_in_flight);
        let mut in_flight_fences = Vec::with_capacity(frames_in_flight);
        let mut images_in_flight = Vec::with_capacity(frames_in_flight);

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        for _ in 0..frames_in_flight {
            image_available.push(unsafe { device.create_semaphore(&semaphore_info, None)? });
            render_finished.push(unsafe { device.create_semaphore(&semaphore_info, None)? });
            in_flight_fences.push(unsafe { device.create_fence(&fence_info, None)? });
            images_in_flight.push(vk::Fence::null());
        }

        Ok(Self {
            image_available,
            render_finished,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
        })
    }

    pub fn wait_for_frame(&self, device: &ash::Device) -> Result<(), VulkanError> {
        unsafe {
            device.wait_for_fences(
                &[self.in_flight_fences[self.current_frame]],
                true,
                u64::MAX,
            )?;
        }
        Ok(())
    }

    pub fn reset_fence(&self, device: &ash::Device) -> Result<(), VulkanError> {
        unsafe {
            device.reset_fences(&[self.in_flight_fences[self.current_frame]])?;
        }
        Ok(())
    }

    pub fn advance_frame(&mut self, frames_in_flight: usize) {
        self.current_frame = (self.current_frame + 1) % frames_in_flight;
    }
}

impl Drop for FrameSync {
    fn drop(&mut self) {
        // Device reference needed for cleanup
    }
}

/// Command pool for graphics operations
pub struct CommandPool {
    pub handle: vk::CommandPool,
    pub queue_family: u32,
}

impl CommandPool {
    pub fn new(device: &ash::Device, queue_family: u32, transient: bool) -> Result<Self, VulkanError> {
        let mut flags = vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        if transient {
            flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }

        let create_info = vk::CommandPoolCreateInfo {
            queue_family_index: queue_family,
            flags,
            ..Default::default()
        };

        let handle = unsafe { device.create_command_pool(&create_info, None)? };

        Ok(Self { handle, queue_family })
    }

    pub fn allocate_buffers(
        &self,
        device: &ash::Device,
        count: u32,
        level: vk::CommandBufferLevel,
    ) -> Result<Vec<vk::CommandBuffer>, VulkanError> {
        let alloc_info = vk::CommandBufferAllocateInfo {
            command_pool: self.handle,
            level,
            command_buffer_count: count,
            ..Default::default()
        };

        let buffers = unsafe { device.allocate_command_buffers(&alloc_info)? };
        Ok(buffers)
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        // Device reference needed for cleanup
    }
}