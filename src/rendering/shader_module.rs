//! This module defines the ShadeModule abstraction which makes it easy to
//! create vulkan shader modules directly with the rust `include_bytes` macro.

use crate::{ffi, rendering::Device};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// A wrapper for the vulkan shader module handle which destroys the module
/// when dropped.
pub struct ShaderModule {
    pub shader_module: vk::ShaderModule,
    device: Arc<Device>,
}

impl ShaderModule {
    /// Create a new shader module using the provided source.
    ///
    /// Panics if the source array is not divisible evenly into u32 words.
    pub fn new<Name>(
        device: &Arc<Device>,
        name: Name,
        source: &'static [u8],
    ) -> Result<Self>
    where
        Name: Into<String>,
    {
        let source_u32 = ffi::copy_to_u32(source);
        let create_info =
            vk::ShaderModuleCreateInfo::builder().code(&source_u32);

        let shader_module = unsafe {
            device
                .logical_device
                .create_shader_module(&create_info, None)
                .context("unable to create shader module")?
        };

        device.name_vulkan_object(
            name,
            vk::ObjectType::SHADER_MODULE,
            &shader_module,
        )?;

        Ok(Self {
            shader_module,
            device: device.clone(),
        })
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_shader_module(self.shader_module, None);
        }
    }
}
