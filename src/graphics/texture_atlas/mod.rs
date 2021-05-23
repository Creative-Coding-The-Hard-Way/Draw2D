//! Traits and implementations for managing a texture atlas.
//!
//! # Big Idea
//!
//! A shader can define code like:
//!
//! ```glsl
//! layout(binding = 1) uniform sampler2D textures[MAX_TEXTURES];
//! ```
//!
//! Which describes an array of textures. Textures in the array can all have
//! different sizes and bindings. Individual draw calls can index into the
//! array using uniform buffers or push constants.
//!
//! The appeal is that the entire texture array only needs to be bound once
//! for the entire frame.

mod atlas_version;
mod gpu_atlas;
mod sampler_handle;
mod texture_handle;

pub use self::{
    atlas_version::AtlasVersion, gpu_atlas::GpuAtlas,
    sampler_handle::SamplerHandle, texture_handle::TextureHandle,
};

use crate::graphics::Graphics;

use anyhow::Result;
use ash::vk;

use super::vulkan::texture::TextureImage;

/// The maximum number of textures which can be managed by any given texture
/// atlas.
pub const MAX_SUPPORTED_TEXTURES: usize = 64;

/// A type which owns a collection of texture objects that can be bound once
/// per frame and individually accessed in calls to `vkDraw`.
pub trait TextureAtlas {
    /// The atlas's current version.
    fn version(&self) -> AtlasVersion;

    /// Build the array of descriptor image info objects which can be used to
    /// write all of this atlas's textures into a descriptor set.
    fn build_descriptor_image_info(&self) -> Vec<vk::DescriptorImageInfo>;

    /// Add a named sampler to the atlas. Samplers can be persistently bound to
    /// individual textures.
    fn add_sampler(&mut self, sampler: vk::Sampler) -> Result<SamplerHandle>;

    /// Add a texture to the atlas. The atlas owns the texture and will destroy
    /// it when the atlas is dropped.
    fn add_texture(&mut self, texture: TextureImage) -> Result<TextureHandle>;

    /// Bind a sampler to a texture. Binding are persistent - they do not change
    /// until this method is called again.
    fn bind_sampler_to_texture(
        &mut self,
        sampler_handle: SamplerHandle,
        texture_handle: TextureHandle,
    ) -> Result<()>;
}

impl TextureAtlas for Graphics {
    fn version(&self) -> AtlasVersion {
        self.texture_atlas.version()
    }

    fn build_descriptor_image_info(&self) -> Vec<vk::DescriptorImageInfo> {
        self.texture_atlas.build_descriptor_image_info()
    }

    fn add_sampler(&mut self, sampler: vk::Sampler) -> Result<SamplerHandle> {
        self.texture_atlas.add_sampler(sampler)
    }

    fn bind_sampler_to_texture(
        &mut self,
        sampler_handle: SamplerHandle,
        texture_handle: TextureHandle,
    ) -> Result<()> {
        self.texture_atlas
            .bind_sampler_to_texture(sampler_handle, texture_handle)
    }

    fn add_texture(&mut self, texture: TextureImage) -> Result<TextureHandle> {
        self.texture_atlas.add_texture(texture)
    }
}
