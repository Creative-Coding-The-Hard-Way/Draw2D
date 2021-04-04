//! This module contains functions and structures for handling vulkan
//! resources.

pub mod buffer;
pub mod command_pool;
pub mod device;
pub mod instance;
pub mod shader_module;
pub mod swapchain;
pub mod window_surface;

pub use self::{
    buffer::{CpuBuffer, StaticBuffer},
    device::{Device, Queue},
    instance::Instance,
    shader_module::ShaderModule,
    swapchain::Swapchain,
    window_surface::glfw_window,
    window_surface::WindowSurface,
};
