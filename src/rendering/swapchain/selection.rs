//! Functions for selecting correct swapchain properties for this application.

use crate::rendering::WindowSurface;

use anyhow::Result;
use ash::vk;

/// Choose the number of images to use in the swapchain based on the min and
/// max numbers of images supported by the device.
pub fn choose_image_count(
    window_surface: &dyn WindowSurface,
    physical_device: &vk::PhysicalDevice,
) -> Result<u32> {
    //! querying surface capabilities is safe in this context because the
    //! physical device will not be selected unless it supports the swapchain
    //! extension
    let capabilities =
        unsafe { window_surface.surface_capabilities(physical_device)? };
    let proposed_image_count = capabilities.min_image_count + 1;
    if capabilities.max_image_count > 0 {
        Ok(std::cmp::min(
            proposed_image_count,
            capabilities.max_image_count,
        ))
    } else {
        Ok(proposed_image_count)
    }
}

/// Choose a surface format for the swapchain based on the window and chosen
/// physical device.
///
pub fn choose_surface_format(
    window_surface: &dyn WindowSurface,
    physical_device: &vk::PhysicalDevice,
) -> vk::SurfaceFormatKHR {
    //! checking formats is safe because support for the swapchain extension is
    //! verified when picking a physical device
    let formats = unsafe { window_surface.supported_formats(physical_device) };

    log::info!("available formats {:?}", formats);

    let format = formats
        .iter()
        .cloned()
        .find(|format| {
            format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                && format.format == vk::Format::B8G8R8A8_SRGB
        })
        .unwrap_or_else(|| formats[0]);

    log::info!("chosen format {:?}", format);

    format
}

/// Choose a presentation mode for the swapchain based on the window and chosen
/// physical device.
///
pub fn choose_present_mode(
    window_surface: &dyn WindowSurface,
    physical_device: &vk::PhysicalDevice,
) -> vk::PresentModeKHR {
    //! checking presentation modes is safe because support for the swapchain
    //! extension is verified when picking a physical device
    let modes =
        unsafe { window_surface.supported_presentation_modes(physical_device) };

    log::info!("available presentation modes {:?}", modes);

    let mode = if modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        vk::PresentModeKHR::IMMEDIATE
    };

    log::info!("chosen presentation mode {:?}", mode);

    mode
}

/// Choose the swap extent for the swapchain based on the window's framebuffer
/// size.
pub fn choose_swap_extent(
    window_surface: &dyn WindowSurface,
    physical_device: &vk::PhysicalDevice,
) -> Result<vk::Extent2D> {
    //! Getting surface capabilities is safe because suppport for the swapchain
    //! extenstion is verified when picking a physical device
    let capabilities =
        unsafe { window_surface.surface_capabilities(physical_device)? };

    if capabilities.current_extent.width != u32::MAX {
        log::debug!("use current extent {:?}", capabilities.current_extent);
        Ok(capabilities.current_extent)
    } else {
        let (width, height) = window_surface.framebuffer_size();
        let extent = vk::Extent2D {
            width: clamp(
                width,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: clamp(
                height,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        };
        log::debug!("use computed extent {:?}", extent);
        Ok(extent)
    }
}

/// Clamp a value between a minimum and maximum bound.
fn clamp(x: u32, min: u32, max: u32) -> u32 {
    std::cmp::max(min, std::cmp::min(x, max))
}
