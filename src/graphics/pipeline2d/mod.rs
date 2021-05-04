pub mod descriptor_sets;

mod pipeline2d;

use crate::graphics::Device;

use ash::vk;
use std::sync::Arc;

/// The 2d graphics vulkan pipeline.
pub struct Pipeline2d {
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_set_layout: vk::DescriptorSetLayout,
    device: Arc<Device>,
}

/// The push constants used by the pipeline.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PushConsts {
    pub projection: [[f32; 4]; 4],
    /// An index into the global texture array indicating which texture to
    /// sample for rendering.
    pub texture_index: u32,
}
