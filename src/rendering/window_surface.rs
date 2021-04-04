//! This module defines the WindowSurface trait and common implementations.
//!
//! # Example
//!
//! ```
//! let window_surface = GlfwWindow::new(|glfw| {
//!     let (mut window, event_receiver) = glfw
//!         .create_window(
//!             1366,
//!             768,
//!             "Ash Starter",
//!             glfw::WindowMode::Windowed,
//!         )
//!         .context("unable to create the glfw window")?;
//!
//!     window.set_resizable(true);
//!     window.set_key_polling(true);
//!     window.set_size_polling(true);
//!
//!     Ok((window, event_receiver))
//! })?;
//! ```
//!

pub mod glfw_window;

use crate::rendering::Instance;

use anyhow::Result;
use ash::vk;
use std::sync::Arc;

/// The WindowSurface trait defines what other parts of the application require
/// of a window + Vulkan Surface pair.
pub trait WindowSurface {
    /// clone the instance created by this window surface
    fn clone_vulkan_instance(&self) -> Arc<Instance>;

    /// Yield the window's current framebuffer size.
    ///
    /// The size is in physical pixels and is meant to be used directly in the
    /// swapchain extent.
    fn framebuffer_size(&self) -> (u32, u32);

    /// Get the raw surface handle.
    ///
    /// Unsafe because the the WindowSurface itself is responsible for the
    /// lifetime of the real surface object. The caller should not retain this
    /// handle except for the creation of other vulkan resources which will
    /// not outlive the window surface.
    unsafe fn get_surface_handle(&self) -> vk::SurfaceKHR;

    /// Check that a physical device supports rendering to this surface.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying for queue presentation support.
    unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: &vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool>;

    /// Fetch the supported formats for this device.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the surface formats.
    unsafe fn supported_formats(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::SurfaceFormatKHR>;

    /// Fetch the supported presentation modes for this device.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the presentation modes.
    unsafe fn supported_presentation_modes(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::PresentModeKHR>;

    /// Returns the set of all supported surface capabilities.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the surface capabilities.
    unsafe fn surface_capabilities(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR>;
}
