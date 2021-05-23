use super::Texture2dFactory;

use crate::graphics::{
    vulkan::{
        buffer::CpuBuffer,
        texture::{MipmapExtent, TextureImage},
    },
    Graphics,
};

use anyhow::Result;
use ash::vk;
use image::ImageBuffer;
use std::path::Path;

/// Types which implement this trait can load 2d textures from files on the
/// disk.
pub trait TextureLoader {
    /// Read a file from the local filesystem into memory as a usable texture.
    fn read_texture_file(
        &self,
        file_path: impl Into<String>,
    ) -> Result<TextureImage>;
}

impl TextureLoader for Graphics {
    /// Read a file from the local filesystem into memory as a usable texture.
    fn read_texture_file(
        &self,
        file_path: impl Into<String>,
    ) -> Result<TextureImage> {
        let path_string = file_path.into();

        let mipmaps = read_file_mipmaps(&path_string)?;
        let packed_mipmap_data: Vec<&[u8]> = mipmaps
            .iter()
            .map(|mipmap| mipmap.as_raw() as &[u8])
            .collect();

        let mut texture = self.create_empty_2d_texture(
            path_string,
            mipmaps[0].width(),
            mipmaps[0].height(),
            mipmaps.len() as u32,
        )?;

        let mut transfer_buffer = CpuBuffer::new(
            self.device.clone(),
            vk::BufferUsageFlags::TRANSFER_SRC,
        )?;

        unsafe {
            transfer_buffer.write_data_arrays(&packed_mipmap_data)?;

            let mipmap_sizes: Vec<MipmapExtent> = mipmaps
                .iter()
                .map(|mipmap| MipmapExtent {
                    width: mipmap.width(),
                    height: mipmap.height(),
                })
                .collect();

            texture
                .upload_mipmaps_from_buffer(&transfer_buffer, &mipmap_sizes)?;
        }
        Ok(texture)
    }
}

type ImageBufferU8 = ImageBuffer<image::Rgba<u8>, Vec<u8>>;

/// Read a file as an rgba8 image. Mipmaps are automatically generated based
/// on the file size and a Gaussian filter. The returned list is the set of
/// all image mipmaps in a R8G8B8A8 format.
fn read_file_mipmaps(path: &impl AsRef<Path>) -> Result<Vec<ImageBufferU8>> {
    let image_file = image::open(path)?.into_rgba8();
    let (width, height) = (image_file.width(), image_file.height());
    let mip_levels = (height.max(width) as f32).log2().floor() as u32 + 1;

    let mut mipmaps = Vec::with_capacity(mip_levels as usize);
    mipmaps.push(image_file.clone());
    for mipmap_level in 1..mip_levels {
        use image::imageops;
        let mipmap = imageops::resize(
            &image_file,
            width >> mipmap_level,
            height >> mipmap_level,
            imageops::FilterType::Gaussian,
        );
        mipmaps.push(mipmap);
    }

    Ok(mipmaps)
}
