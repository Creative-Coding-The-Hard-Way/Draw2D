use super::Mat4;

use ash::vk;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    projection: [[f32; 4]; 4],
}

impl UniformBufferObject {
    /// Create a new vertex buffer object instance using a projection matrix.
    pub fn new(projection: Mat4) -> Self {
        Self {
            projection: projection.into(),
        }
    }

    /// Create the binding structure which describes how buffer contents map
    /// to data within the shaders.
    pub fn descriptor_set_layout_binding() -> vk::DescriptorSetLayoutBinding {
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()
    }
}
