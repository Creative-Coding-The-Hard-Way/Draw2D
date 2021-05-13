mod mipmap_extent;
mod texture_image;

use crate::graphics::vulkan::Device;

use ash::vk;
use std::sync::Arc;

use super::device_allocator::Allocation;

/// The TextureImage maintains the image, view, and memory, which are required
/// when rendering with a texture.
pub struct TextureImage {
    bytes_per_pixel: u64,
    image: vk::Image,
    extent: vk::Extent3D,
    view: vk::ImageView,

    allocation: Allocation,

    device: Arc<Device>,
}

/// This struct defines the size of a mipmap level.
#[derive(Copy, Clone, Debug)]
pub struct MipmapExtent {
    /// The mipmap level's width, in pixels.
    pub width: u32,

    /// The mipmap level's height, in pixels.
    pub height: u32,
}
