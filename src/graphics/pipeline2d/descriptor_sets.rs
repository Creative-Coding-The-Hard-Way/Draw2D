use super::PushConsts;

use std::mem::size_of;

use crate::graphics::{texture_atlas::MAX_SUPPORTED_TEXTURES, vulkan::Device};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// Create a descriptor set layout instance which describes the bindings used by
/// Draw2d.
///
/// Unsafe:  the returned descriptor set is unowned. The caller is responsible
///          destroying it when it is no longer being used.
pub unsafe fn create_descriptor_set_layout(
    device: &Device,
) -> Result<(vk::DescriptorSetLayout, Vec<vk::DescriptorSetLayoutBinding>)> {
    let bindings = vec![sampler_layout_binding()];
    let descriptor_set_layout =
        device.logical_device.create_descriptor_set_layout(
            &vk::DescriptorSetLayoutCreateInfo {
                p_bindings: bindings.as_ptr(),
                binding_count: bindings.len() as u32,
                ..Default::default()
            },
            None,
        )?;
    Ok((descriptor_set_layout, bindings))
}

/// Create the push constant range definition for the graphics pipeline.
pub fn create_push_constant_range() -> vk::PushConstantRange {
    vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::FRAGMENT
            | vk::ShaderStageFlags::VERTEX,
        size: size_of::<PushConsts>() as u32,
        offset: 0,
    }
}

/// the combined image sampler layout binding
fn sampler_layout_binding() -> vk::DescriptorSetLayoutBinding {
    vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_count: MAX_SUPPORTED_TEXTURES as u32,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
        ..Default::default()
    }
}
