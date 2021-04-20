use crate::rendering::Device;

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// The UniformBufferObject structure used by the vertex shader for holding
/// transform matricies.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    /// The worldspace -> ndc space projection matrix
    pub projection: [[f32; 4]; 4],
}

/// Create a descriptor set layout instance which describes the bindings used by
/// Draw2d.
///
/// Unsafe:  the returned descriptor set is unowned. The caller is responsible
///          destroying it when it is no longer being used.
pub unsafe fn create_descriptor_set_layout(
    device: &Device,
) -> Result<vk::DescriptorSetLayout> {
    let bindings = [ubo_layout_binding(), sampler_layout_binding()];
    let descriptor_set_layout =
        device.logical_device.create_descriptor_set_layout(
            &vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings),
            None,
        )?;

    Ok(descriptor_set_layout)
}

/// The uniform buffer layout binding
fn ubo_layout_binding() -> vk::DescriptorSetLayoutBinding {
    vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .build()
}

/// the combined image sampler layout binding
fn sampler_layout_binding() -> vk::DescriptorSetLayoutBinding {
    vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .build()
}
