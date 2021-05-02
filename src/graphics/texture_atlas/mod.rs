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
mod cached_atlas;
mod gpu_atlas;
mod texture_handle;

pub use self::{cached_atlas::CachedAtlas, gpu_atlas::GpuAtlas};

use anyhow::Result;
use ash::vk;

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

    /// Load a texture file into the atlas.
    fn add_texture(
        &mut self,
        path_to_texture_file: impl Into<String>,
    ) -> Result<TextureHandle>;
}

/// At atlas's version changes any time that the loaded textures are changed
/// in some way.
///
/// Typically this is used to detect when the atlas needs to update as shader's
/// descriptor sets.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AtlasVersion {
    revision_count: u32,
}

/// A handle which can provide the texture index for a push constant.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextureHandle(u32);
