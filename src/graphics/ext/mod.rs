use crate::graphics::Graphics;

use anyhow::Result;
use ash::vk;

/// Types which implement this trait can create and destroy raw vulkan samplers.
pub trait SamplerFactory {
    /// Create a new sampler.
    ///
    /// # Unsafe Because
    ///
    /// - the caller must remember to destroy the sampler when they're done
    ///   with it.
    unsafe fn create_sampler(
        &self,
        debug_name: impl Into<String>,
        sampler_create_info: vk::SamplerCreateInfo,
    ) -> Result<vk::Sampler>;

    /// Destroy a sampler.
    ///
    /// # Unsafe Because
    ///
    /// - the caller must ensure that the sampler is not in use by the GPU when
    ///   it is destroyed.
    unsafe fn destroy_sampler(&self, sampler: vk::Sampler);
}

impl SamplerFactory for Graphics {
    /// Create a new sampler using the Graphics subsystem's logical device.
    unsafe fn create_sampler(
        &self,
        debug_name: impl Into<String>,
        sampler_create_info: vk::SamplerCreateInfo,
    ) -> Result<vk::Sampler> {
        self.device.create_sampler(debug_name, sampler_create_info)
    }

    unsafe fn destroy_sampler(&self, sampler: vk::Sampler) {
        self.device.destroy_sampler(sampler)
    }
}
