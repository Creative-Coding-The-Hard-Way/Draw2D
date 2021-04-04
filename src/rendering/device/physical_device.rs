//! Functions for picking a physical device with the features required by this
//! application.

use super::QueueFamilyIndices;
use crate::rendering::{Instance, WindowSurface};

use anyhow::{Context, Result};
use ash::{version::InstanceV1_0, vk};

/// Pick a physical device based on suitability criteria.
pub fn pick_physical_device(
    instance: &Instance,
    window_surface: &dyn WindowSurface,
) -> Result<vk::PhysicalDevice> {
    let physical_devices =
        unsafe { instance.ash.enumerate_physical_devices()? };
    let physical_device = physical_devices
        .iter()
        .find(|device| is_device_suitable(&instance, device, window_surface))
        .context("unable to pick a suitable device")?;
    Ok(*physical_device)
}

/// Return true when the device is suitable for this application.
fn is_device_suitable(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    window_surface: &dyn WindowSurface,
) -> bool {
    let queues_supported = QueueFamilyIndices::find(
        physical_device,
        &instance.ash,
        window_surface,
    )
    .is_ok();

    let features =
        unsafe { instance.ash.get_physical_device_features(*physical_device) };

    let extensions_supported =
        check_required_extensions(&instance, physical_device);

    let format_available = if extensions_supported {
        unsafe { !window_surface.supported_formats(physical_device).is_empty() }
    } else {
        false
    };

    let presentation_mode_available = if extensions_supported {
        unsafe {
            !window_surface
                .supported_presentation_modes(physical_device)
                .is_empty()
        }
    } else {
        false
    };

    queues_supported
        && extensions_supported
        && format_available
        && presentation_mode_available
        && features.geometry_shader == vk::TRUE
}

/// Fetch a vector of all missing device extensions based on the required
/// extensions.
fn check_required_extensions(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
) -> bool {
    let extensions = unsafe {
        instance
            .ash
            .enumerate_device_extension_properties(*physical_device)
            .unwrap_or_else(|_| vec![])
    };
    extensions
        .iter()
        .map(|extension| {
            String::from_utf8(
                extension.extension_name.iter().map(|c| *c as u8).collect(),
            )
            .unwrap()
        })
        .filter(|name| required_device_extensions().contains(name))
        .collect::<Vec<String>>()
        .is_empty()
}

/// Return the set of required device features for this application.
///
/// `is_device_suitable` should verify that all required features are supported
/// by the chosen physical device.
pub fn required_device_features() -> vk::PhysicalDeviceFeatures {
    vk::PhysicalDeviceFeatures::builder()
        .geometry_shader(true)
        .build()
}

/// Return the set of required device extensions for this application
pub fn required_device_extensions() -> Vec<String> {
    let swapchain = ash::extensions::khr::Swapchain::name()
        .to_owned()
        .into_string()
        .unwrap();
    vec![swapchain]
}
