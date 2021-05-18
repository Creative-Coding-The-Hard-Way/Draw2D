//! This module contains functions and structures for handling vulkan
//! resources.

pub mod buffer;
pub mod command_pool;
pub mod device;
pub mod ffi;
pub mod instance;
pub mod shader_module;
pub mod swapchain;
pub mod texture;
pub mod window_surface;
pub mod device_allocator;

pub use self::{
    device::Device, instance::Instance, swapchain::Swapchain,
    window_surface::WindowSurface,
};
