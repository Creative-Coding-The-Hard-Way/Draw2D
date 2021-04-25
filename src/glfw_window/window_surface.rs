use crate::{
    glfw_window::GlfwWindow,
    graphics::{vulkan, vulkan::Instance},
};

use anyhow::{Context, Result};
use ash::vk;
use std::sync::Arc;

impl vulkan::WindowSurface for GlfwWindow {
    /// clone the instance created by this window surface
    fn clone_vulkan_instance(&self) -> Arc<Instance> {
        self.instance.clone()
    }

    /// Yield the window's current framebuffer size.
    ///
    /// The size is in physical pixels and is meant to be used directly in the
    /// swapchain extent.
    fn framebuffer_size(&self) -> (u32, u32) {
        let (iwidth, iheight) = self.window.get_framebuffer_size();
        (iwidth as u32, iheight as u32)
    }

    /// Get the raw surface handle.
    ///
    /// Unsafe because the the WindowSurface itself is responsible for the
    /// lifetime of the real surface object. The caller should not retain this
    /// handle except for the creation of other vulkan resources which will
    /// not outlive the window surface.
    unsafe fn get_surface_handle(&self) -> vk::SurfaceKHR {
        self.surface
    }

    /// Check that a physical device supports rendering to this surface.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying for queue presentation support.
    ///
    //                )
    unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: &vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool> {
        self.surface_loader
            .get_physical_device_surface_support(
                *physical_device,
                queue_family_index,
                self.surface,
            )
            .context("unable to check for queue family support!")
    }

    /// Returns the set of all supported formats for this device.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the surface formats.
    unsafe fn supported_formats(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::SurfaceFormatKHR> {
        self.surface_loader
            .get_physical_device_surface_formats(*physical_device, self.surface)
            .unwrap_or_else(|_| vec![])
    }

    /// Returns the set of all supported presentation modes for this device.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the presentation modes.
    unsafe fn supported_presentation_modes(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::PresentModeKHR> {
        self.surface_loader
            .get_physical_device_surface_present_modes(
                *physical_device,
                self.surface,
            )
            .unwrap_or_else(|_| vec![])
    }

    /// Returns the set of all supported surface capabilities.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the surface capabilities.
    unsafe fn surface_capabilities(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR> {
        self.surface_loader
            .get_physical_device_surface_capabilities(
                *physical_device,
                self.surface,
            )
            .context("unable to get surface capabiliities for this device")
    }
}
