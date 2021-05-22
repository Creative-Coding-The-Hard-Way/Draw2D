use crate::graphics::{vulkan::texture::TextureImage, Graphics};

use anyhow::Result;

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
