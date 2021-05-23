mod gpu_atlas;

use crate::graphics::{
    texture_atlas::{AtlasVersion, SamplerHandle},
    vulkan::{buffer::CpuBuffer, texture::TextureImage, Device},
};

use ash::vk;
use std::sync::Arc;

struct Binding {
    texture: TextureImage,
    sampler_handle: SamplerHandle,
}

/// The GPU Atlas is responsible for actually loading texture data into gpu
/// memory.
pub struct GpuAtlas {
    /// The collection of all textures owned by this atlas.
    textures: Vec<Option<Binding>>,

    /// The samplers used by textures owned by this atlas.
    samplers: Vec<vk::Sampler>,

    /// This buffer is used to transfer data to the GPU when a texture is first
    /// added to the atlas.
    transfer_buffer: CpuBuffer,

    /// The version be used to determine when a shader's descriptors need to
    /// be updated.
    version: AtlasVersion,

    /// A handle to the vulkan device.
    device: Arc<Device>,
}
