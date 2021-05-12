use anyhow::Result;
use ash::{
    extensions::ext::DebugUtils,
    vk,
    vk::{
        DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT, DebugUtilsMessengerEXT,
    },
    Entry,
};
use std::{borrow::Cow, ffi::CStr};

/// Create the vulkan debug callback for validation.
pub fn create_debug_logger(
    entry: &Entry,
    instance: &ash::Instance,
) -> Result<(DebugUtils, DebugUtilsMessengerEXT)> {
    let debug_utils = DebugUtils::new(entry, instance);

    let create_info = vk::DebugUtilsMessengerCreateInfoEXT {
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        pfn_user_callback: Some(debug_callback),
        ..Default::default()
    };

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
