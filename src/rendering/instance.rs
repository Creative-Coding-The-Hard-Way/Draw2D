//! Functions for creating an instance with extensions.
//!
//! The Instance struct holds the ash entry and ash instance along with the
//! debug callback. This is convenient because the application needs to hold
//! references to all of this data, but it's unwieldy to have separate handles
//! to each constantly floating around.
//!
//! # Example
//!
//! ```
//! let instance = Instance::new(
//!     &glfw.get_required_instance_extensions().unwrap()
//! )?;
//! ```

mod extensions;
mod layers;

use crate::ffi::to_os_ptrs;
use anyhow::Result;
use ash::{
    extensions::ext::DebugUtils,
    version::{EntryV1_0, InstanceV1_0},
    vk,
    vk::{
        DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerEXT,
    },
    Entry,
};
use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    sync::Arc,
};

/// Hold all of the instance-related objects and drop them in the correct order.
pub struct Instance {
    pub entry: Entry,
    pub ash: ash::Instance,
    pub debug: DebugUtils,
    pub debug_messenger: DebugUtilsMessengerEXT,
    pub enabled_layer_names: Vec<String>,
}

impl Instance {
    /// Create a new ash instance with the required extensions.
    ///
    /// Debug and validation layers are automatically setup along with the
    /// debug callback.
    pub fn new(required_extensions: &Vec<String>) -> Result<Arc<Self>> {
        let layers = required_layers();
        let (instance, entry) = create_instance(required_extensions, &layers)?;
        let (debug, debug_messenger) =
            create_debug_callback(&entry, &instance)?;

        Ok(Arc::new(Self {
            ash: instance,
            entry,
            debug,
            debug_messenger,
            enabled_layer_names: layers,
        }))
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

/// The validation layers required by this application.
fn required_layers() -> Vec<String> {
    vec![
        "VK_LAYER_KHRONOS_validation".to_owned(),
        // "VK_LAYER_LUNARG_api_dump".to_owned(),
    ]
}

/// Create a Vulkan instance with the required extensions.
/// Returns an `Err()` if any required extensions are unavailable.
fn create_instance(
    required_extensions: &Vec<String>,
    required_layers: &Vec<String>,
) -> Result<(ash::Instance, Entry)> {
    let entry = Entry::new()?;

    let mut required_with_debug = required_extensions.clone();
    required_with_debug.push(DebugUtils::name().to_str()?.to_owned());

    extensions::check_extensions(&entry, &required_with_debug)?;
    layers::check_layers(&entry, &required_layers)?;

    log::debug!("Required Extensions {:?}", required_extensions);

    let app_name = CString::new("ash starter").unwrap();
    let engine_name = CString::new("no engine").unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(&engine_name)
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 1, 0));

    let (_layer_names, layer_ptrs) = unsafe { to_os_ptrs(&required_layers) };
    let (_ext_names, ext_ptrs) = unsafe { to_os_ptrs(&required_with_debug) };

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&ext_ptrs)
        .enabled_layer_names(&layer_ptrs);

    let instance = unsafe { entry.create_instance(&create_info, None)? };

    Ok((instance, entry))
}

/// Create the vulkan debug callback for validation.
fn create_debug_callback(
    entry: &Entry,
    instance: &ash::Instance,
) -> Result<(DebugUtils, DebugUtilsMessengerEXT)> {
    let debug_utils = DebugUtils::new(entry, instance);

    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(debug_callback));

    let debug_messenger = unsafe {
        debug_utils.create_debug_utils_messenger(&create_info, None)?
    };

    Ok((debug_utils, debug_messenger))
}

unsafe extern "system" fn debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message_number = callback_data.message_id_number;

    let raw_message = std::format!(
        "Vulkan Debug Callback - {:?} :: {:?} [{} ({})]\n{}",
        message_severity,
        message_type,
        message_id_name,
        message_number,
        message
    );

    let full_message = raw_message.replace("; ", ";\n\n");

    match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!("{}", full_message);
        }

        DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("{}", full_message);
        }

        DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("{}", full_message);
        }

        DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("{}", full_message);
        }

        _ => {
            log::warn!("?? {}", full_message);
        }
    }
    return vk::FALSE;
}
