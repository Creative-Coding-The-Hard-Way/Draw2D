use super::Device;

use crate::graphics::ext::SamplerFactory;

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};

impl SamplerFactory for Device {
    unsafe fn create_sampler(
        &self,
        debug_name: impl Into<String>,
        sampler_create_info: vk::SamplerCreateInfo,
    ) -> Result<vk::Sampler> {
        let owned_name = debug_name.into();
        let sampler = self
            .logical_device
            .create_sampler(&sampler_create_info, None)
            .with_context(|| {
                format!("unable to create sampler {:?}", owned_name.clone())
            })?;

        self.name_vulkan_object(owned_name, vk::ObjectType::SAMPLER, &sampler)?;

        Ok(sampler)
    }

    unsafe fn destroy_sampler(&self, sampler: vk::Sampler) {
        self.logical_device.destroy_sampler(sampler, None);
    }
}
