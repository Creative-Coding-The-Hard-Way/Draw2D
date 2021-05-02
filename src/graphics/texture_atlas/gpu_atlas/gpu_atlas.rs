use super::GpuAtlas;

use crate::graphics::{
    texture_atlas::{
        AtlasVersion, TextureAtlas, TextureHandle, MAX_SUPPORTED_TEXTURES,
    },
    vulkan::{
        buffer::CpuBuffer,
        command_pool::TransientCommandPool,
        texture::{MipmapExtent, TextureImage},
        Device,
    },
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use image::ImageBuffer;
use std::{path::Path, sync::Arc};

impl GpuAtlas {
    /// Create a new texture atlas which loads image data into GPU memory.
    pub fn new(device: Arc<Device>) -> Result<Self> {
        let sampler = unsafe {
            let sampler_create_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::REPEAT)
                .address_mode_v(vk::SamplerAddressMode::REPEAT)
                .address_mode_w(vk::SamplerAddressMode::REPEAT)
                .anisotropy_enable(false)
                .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                .unnormalized_coordinates(false)
                .compare_enable(false)
                .compare_op(vk::CompareOp::ALWAYS)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .mip_lod_bias(0.0)
                .min_lod(0.0)
                .max_lod(vk::LOD_CLAMP_NONE);
            device
                .logical_device
                .create_sampler(&sampler_create_info, None)?
        };
        let mut atlas = Self {
            transfer_buffer: CpuBuffer::new(
                device.clone(),
                vk::BufferUsageFlags::TRANSFER_SRC,
            )?,
            command_pool: TransientCommandPool::new(
                device.clone(),
                "Texture Atlas Command Pool",
            )?,
            textures: vec![],
            version: AtlasVersion { revision_count: 1 },
            sampler,
            device,
        };

        let mut default_texture =
            atlas.create_empty_2d_texture("default texture", 1, 1, 1)?;
        unsafe {
            // SAFE: because the texture was just created and is not being used
            //       elsewhere.
            let white_pixel: [u8; 4] = [255, 255, 255, 255];
            atlas.transfer_buffer.write_data(&white_pixel)?;
            default_texture.upload_from_buffer(
                atlas.command_pool.request_command_buffer()?,
                &atlas.transfer_buffer,
            )?;
            atlas.command_pool.reset()?;
        }
        atlas.textures.push(default_texture);

        Ok(atlas)
    }

    /// Directly create an empty 2d texture.
    fn create_empty_2d_texture<Name>(
        &self,
        name: Name,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> Result<TextureImage>
    where
        Name: Into<String>,
    {
        let (format, bytes_per_pixel) = (vk::Format::R8G8B8A8_SRGB, 4 as u64);
        let texture = TextureImage::new(
            self.device.clone(),
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
        self.device.name_vulkan_object(
            format!("{} - Image", owned_name.clone()),
            vk::ObjectType::IMAGE,
            unsafe { &texture.raw_image() },
        )?;
        self.device.name_vulkan_object(
            format!("{} - Image View", owned_name.clone()),
            vk::ObjectType::IMAGE_VIEW,
            unsafe { &texture.raw_view() },
        )?;

        Ok(texture)
    }

    fn read_file_mipmaps<P: AsRef<Path>>(
        &self,
        path: &P,
    ) -> Result<Vec<ImageBuffer<image::Rgba<u8>, Vec<u8>>>> {
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
}

impl TextureAtlas for GpuAtlas {
    fn version(&self) -> AtlasVersion {
        self.version
    }

    /// Add a texture to the atlas and return a texture handle.
    ///
    /// Texture handles can be used when drawing to get the texture_index which
    /// the shader uses to select this texture from the global array.
    fn add_texture(
        &mut self,
        path_to_texture_file: impl Into<String>,
    ) -> Result<TextureHandle> {
        if self.textures.len() >= MAX_SUPPORTED_TEXTURES {
            anyhow::bail!(
                "only a maximum of {} textures are supported!",
                MAX_SUPPORTED_TEXTURES
            );
        }
        let path_string = path_to_texture_file.into();
        let mipmaps = self.read_file_mipmaps(&path_string)?;
        let mut texture = self.create_empty_2d_texture(
            path_string,
            mipmaps[0].width(),
            mipmaps[0].height(),
            mipmaps.len() as u32,
        )?;

        unsafe {
            // SAFE: the transfer buffer is only used/considered valid within
            //       the scope of this function.
            let data: Vec<&[u8]> = mipmaps
                .iter()
                .map(|mipmap| mipmap.as_raw() as &[u8])
                .collect();
            self.transfer_buffer.write_data_arrays(&data)?;
        }

        unsafe {
            let mipmap_sizes: Vec<MipmapExtent> = mipmaps
                .iter()
                .map(|mipmap| MipmapExtent {
                    width: mipmap.width(),
                    height: mipmap.height(),
                })
                .collect();

            texture.upload_mipmaps_from_buffer(
                self.command_pool.request_command_buffer()?,
                &self.transfer_buffer,
                &mipmap_sizes,
            )?;

            self.command_pool.reset()?;
        }

        self.textures.push(texture);
        let index = (self.textures.len() - 1) as u32;

        self.version.revision_count += 1;

        Ok(TextureHandle(index))
    }

    /// Build a vector of descriptor image info entries. This can be used when
    /// updating a descriptor set with specific image bindings.
    fn build_descriptor_image_info(&self) -> Vec<vk::DescriptorImageInfo> {
        let mut bindings: Vec<vk::DescriptorImageInfo> = self
            .textures
            .iter()
            .map(|texture| vk::DescriptorImageInfo {
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image_view: unsafe { texture.raw_view() },
                sampler: self.sampler,
            })
            .collect();
        for _ in self.textures.len()..MAX_SUPPORTED_TEXTURES {
            bindings.push(vk::DescriptorImageInfo {
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image_view: unsafe { self.textures[0].raw_view() },
                sampler: self.sampler,
            });
        }
        bindings
    }
}

impl Drop for GpuAtlas {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_sampler(self.sampler, None);
        }
    }
}
