//! Functions for creating an instance with extensions.
//!
//! The Instance struct holds the ash entry and ash instance along with the
//! debug callback. This is convenient because the application needs to hold
//! references to all of this data, but it's unwieldy to have separate handles
//! to each constantly floating around.

mod debug_callback;
mod extensions;
mod layers;

use super::ffi::to_os_ptrs;

use anyhow::Result;
use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    version::{EntryV1_0, InstanceV1_0},
    vk, Entry,
};
use std::{ffi::CString, sync::Arc};

/// Hold all of the instance-related objects and drop them in the correct order.
pub struct Instance {
    /// The Ash Vulkan library entrypoint.
    pub ash: ash::Instance,

    /// The Debug entrypoint, used to set debug names for vulkan objects.
    pub debug: DebugUtils,

    layers: Vec<String>,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    entry: Entry,
}

impl Instance {
    fn debug_layers() -> Vec<String> {
        vec![
            "VK_LAYER_KHRONOS_validation".to_owned(),
            // "VK_LAYER_LUNARG_api_dump".to_owned(),
        ]
    }

    /// Create a new ash instance with the required extensions.
    ///
    /// Debug and validation layers are automatically setup along with the
    /// debug callback.
    pub fn new(required_extensions: &Vec<String>) -> Result<Arc<Self>> {
        let (instance, entry) = Self::create_instance(required_extensions)?;
        let (debug, debug_messenger) =
            debug_callback::create_debug_logger(&entry, &instance)?;

        Ok(Arc::new(Self {
            ash: instance,
            entry,
            debug,
            debug_messenger,
            layers: Self::debug_layers(),
        }))
    }

    /// A non-owning borrow of the ash library instance.
    pub fn raw(&self) -> &ash::Instance {
        &self.ash
    }

    /// Create a khr surface loader.
    ///
    /// The caller is responsible for destroying the loader when it is no
    /// longer needed.
    pub fn create_surface_loader(&self) -> Surface {
        Surface::new(&self.entry, &self.ash)
    }

    /// Create a new logical device for use by this application. The caller is
    /// responsible for destroying the device when done.
    pub fn create_logical_device(
        &self,
        physical_device: &vk::PhysicalDevice,
        physical_device_features: vk::PhysicalDeviceFeatures,
        physical_device_extensions: &[String],
        queue_create_infos: &[vk::DeviceQueueCreateInfo],
    ) -> Result<ash::Device> {
        use anyhow::Context;

        let (_c_names, layer_name_ptrs) = unsafe { to_os_ptrs(&self.layers) };
        let (_c_ext_names, ext_name_ptrs) =
            unsafe { to_os_ptrs(physical_device_extensions) };

        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: &physical_device_features,
            pp_enabled_layer_names: layer_name_ptrs.as_ptr(),
            enabled_layer_count: layer_name_ptrs.len() as u32,
            pp_enabled_extension_names: ext_name_ptrs.as_ptr(),
            enabled_extension_count: physical_device_extensions.len() as u32,
            ..Default::default()
        };

        let logical_device = unsafe {
            self.ash
                .create_device(*physical_device, &create_info, None)
                .context("unable to create the logical device")?
        };

        Ok(logical_device)
    }

    /// Create a Vulkan instance with the required extensions.
    /// Returns an `Err()` if any required extensions are unavailable.
    fn create_instance(
        required_extensions: &Vec<String>,
    ) -> Result<(ash::Instance, Entry)> {
        let entry = Entry::new()?;

        let mut required_with_debug = required_extensions.clone();
        required_with_debug.push(DebugUtils::name().to_str()?.to_owned());

        extensions::check_extensions(&entry, &required_with_debug)?;
        layers::check_layers(&entry, &Self::debug_layers())?;

        log::debug!("Required Extensions {:?}", required_extensions);

        let app_name = CString::new("ash starter").unwrap();
        let engine_name = CString::new("no engine").unwrap();

        let app_info = vk::ApplicationInfo {
            p_engine_name: engine_name.as_ptr(),
            p_application_name: app_name.as_ptr(),
            application_version: vk::make_version(1, 0, 0),
            engine_version: vk::make_version(1, 0, 0),
            api_version: vk::make_version(1, 1, 0),
            ..Default::default()
        };

        let (_layer_names, layer_ptrs) =
            unsafe { to_os_ptrs(&Self::debug_layers()) };
        let (_ext_names, ext_ptrs) =
            unsafe { to_os_ptrs(&required_with_debug) };

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            pp_enabled_layer_names: layer_ptrs.as_ptr(),
            enabled_layer_count: layer_ptrs.len() as u32,
            pp_enabled_extension_names: ext_ptrs.as_ptr(),
            enabled_extension_count: ext_ptrs.len() as u32,
            ..Default::default()
        };

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        Ok((instance, entry))
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.debug
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.ash.destroy_instance(None);
        }
    }
}
