use crate::graphics::{vulkan::texture::TextureImage, Device, Graphics};

use anyhow::Result;
use ash::vk;
use std::sync::Arc;

/// Types which implement this trait can easily construct new texture images
/// which represent 2d rgba textures.
pub trait Texture2dFactory {
    /// Create a new 2d texture image and view.
    fn create_empty_2d_texture(
        &self,
        name: impl Into<String>,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> Result<TextureImage>;
}

impl Texture2dFactory for Graphics {
    /// Create an empty 2d texture object using the Graphic subsystem's logical
    /// device.
    fn create_empty_2d_texture(
        &self,
        name: impl Into<String>,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> Result<TextureImage> {
        self.device
            .create_empty_2d_texture(name, width, height, mip_levels)
    }
}

impl Texture2dFactory for Arc<Device> {
    fn create_empty_2d_texture(
        &self,
        name: impl Into<String>,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> Result<TextureImage> {
        let (format, bytes_per_pixel) = (vk::Format::R8G8B8A8_SRGB, 4 as u64);
        let texture = TextureImage::new(
            self.clone(),
            vk::ImageCreateInfo {
                image_type: vk::ImageType::TYPE_2D,
                extent: vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                },
                mip_levels,
                array_layers: 1,
                format,
                tiling: vk::ImageTiling::OPTIMAL,
                initial_layout: vk::ImageLayout::UNDEFINED,
                usage: vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::SAMPLED,
                samples: vk::SampleCountFlags::TYPE_1,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            },
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            bytes_per_pixel,
        )?;

        let owned_name = name.into();
        self.name_vulkan_object(
            format!("{} - Image", owned_name.clone()),
            vk::ObjectType::IMAGE,
            unsafe { &texture.raw_image() },
        )?;
        self.name_vulkan_object(
            format!("{} - Image View", owned_name.clone()),
            vk::ObjectType::IMAGE_VIEW,
            unsafe { &texture.raw_view() },
        )?;

        Ok(texture)
    }
}
