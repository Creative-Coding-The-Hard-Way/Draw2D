mod gpu_atlas;

use crate::graphics::{
    texture_atlas::AtlasVersion,
    vulkan::{
        buffer::CpuBuffer, command_pool::TransientCommandPool,
        texture::TextureImage, Device,
    },
};

use ash::vk;
use std::sync::Arc;

/// The GPU Atlas is responsible for actually loading texture data into gpu
/// memory.
pub struct GpuAtlas {
    /// The collection of all textures owned by this atlas.
    textures: Vec<TextureImage>,

    /// The samplers used by textures owned by this atlas.
    samplers: Vec<vk::Sampler>,

    /// A pool of command buffers used by the atlas for assorted GPU operations.
    command_pool: TransientCommandPool,

    /// This buffer is used to transfer data to the GPU when a texture is first
    /// added to the atlas.
    transfer_buffer: CpuBuffer,

    /// The version be used to determine when a shader's descriptors need to
    /// be updated.
    version: AtlasVersion,

    /// A handle to the vulkan device.
    device: Arc<Device>,
}
