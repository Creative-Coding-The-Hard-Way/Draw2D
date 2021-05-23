use super::GpuAtlas;

use crate::graphics::{
    ext::Texture2dFactory,
    texture_atlas::{
        gpu_atlas::Binding, AtlasVersion, SamplerHandle, TextureAtlas,
        TextureHandle, MAX_SUPPORTED_TEXTURES,
    },
    vulkan::{buffer::CpuBuffer, texture::MipmapExtent, Device},
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use image::ImageBuffer;
use std::{path::Path, sync::Arc};

impl GpuAtlas {
    /// Create a new texture atlas which loads image data into GPU memory.
    pub fn new(device: Arc<Device>) -> Result<Self> {
        let sampler = unsafe {
            use crate::graphics::ext::SamplerFactory;
            device.create_sampler(
                "default sampler",
                vk::SamplerCreateInfo {
                    mag_filter: vk::Filter::LINEAR,
                    min_filter: vk::Filter::LINEAR,
                    address_mode_u: vk::SamplerAddressMode::REPEAT,
                    address_mode_v: vk::SamplerAddressMode::REPEAT,
                    address_mode_w: vk::SamplerAddressMode::REPEAT,
                    anisotropy_enable: 0,
                    border_color: vk::BorderColor::INT_OPAQUE_BLACK,
                    unnormalized_coordinates: 0,
                    compare_enable: 0,
                    compare_op: vk::CompareOp::ALWAYS,
                    mipmap_mode: vk::SamplerMipmapMode::LINEAR,
                    mip_lod_bias: 0.0,
                    min_lod: 0.0,
                    max_lod: vk::LOD_CLAMP_NONE,
                    ..Default::default()
                },
            )?
        };

        let mut atlas = Self {
            transfer_buffer: CpuBuffer::new(
                device.clone(),
                vk::BufferUsageFlags::TRANSFER_SRC,
            )?,
            textures: vec![],
            version: AtlasVersion::new_out_of_date().increment(),
            samplers: vec![sampler],
            device,
        };

        let mut default_texture =
            atlas
                .device
                .create_empty_2d_texture("default texture", 1, 1, 1)?;

        unsafe {
            // SAFE: because the texture was just created and is not being used
            //       elsewhere.
            let white_pixel: [u8; 4] = [255, 255, 255, 255];
            atlas.transfer_buffer.write_data(&white_pixel)?;
            default_texture.upload_from_buffer(&atlas.transfer_buffer)?;
        }

        atlas.textures.push(Some(Binding {
            texture: default_texture,
            sampler_handle: SamplerHandle::default(),
        }));

        for _ in 1..MAX_SUPPORTED_TEXTURES {
            atlas.textures.push(None);
        }

        Ok(atlas)
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

    fn add_sampler(&mut self, sampler: vk::Sampler) -> Result<SamplerHandle> {
        self.samplers.push(sampler);
        let index = self.samplers.len() - 1;
        Ok(SamplerHandle::new(index as u32))
    }

    fn bind_sampler_to_texture(
        &mut self,
        sampler_handle: SamplerHandle,
        texture_handle: TextureHandle,
    ) -> Result<()> {
        if let Some(binding) =
            &mut self.textures[texture_handle.texture_index() as usize]
        {
            binding.sampler_handle = sampler_handle;
            Ok(())
        } else {
            anyhow::bail!("the provide texture handle does not match an existing texture!");
        }
    }

    /// Add a texture to the atlas and return a texture handle.
    ///
    /// Texture handles can be used when drawing to get the texture_index which
    /// the shader uses to select this texture from the global array.
    fn add_texture(
        &mut self,
        path_to_texture_file: impl Into<String>,
    ) -> Result<TextureHandle> {
        let path_string = path_to_texture_file.into();
        let mipmaps = self.read_file_mipmaps(&path_string)?;
        let mut texture = self.device.create_empty_2d_texture(
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
                &self.transfer_buffer,
                &mipmap_sizes,
            )?;
        }
        use anyhow::Context;

        let free_slot_index = self
            .textures
            .iter()
            .enumerate()
            .find(|(_i, entry)| entry.is_none())
            .with_context(|| "unable to find a free texture slot!")?
            .0;

        self.textures[free_slot_index] = Some(Binding {
            texture,
            sampler_handle: SamplerHandle::default(),
        });

        self.version = self.version.increment();

        Ok(TextureHandle::new(free_slot_index as u32))
    }

    /// Build a vector of descriptor image info entries. This can be used when
    /// updating a descriptor set with specific image bindings.
    fn build_descriptor_image_info(&self) -> Vec<vk::DescriptorImageInfo> {
        let default_view =
            unsafe { self.textures[0].as_ref().unwrap().texture.raw_view() };

        self.textures
            .iter()
            .map(|binding_option| match binding_option {
                Some(binding) => vk::DescriptorImageInfo {
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    image_view: unsafe { binding.texture.raw_view() },
                    sampler: self.samplers
                        [binding.sampler_handle.index() as usize],
                },

                None => vk::DescriptorImageInfo {
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    image_view: default_view,
                    sampler: self.samplers[0],
                },
            })
            .collect()
    }
}

impl Drop for GpuAtlas {
    fn drop(&mut self) {
        unsafe {
            for sampler in self.samplers.drain(0..) {
                self.device.logical_device.destroy_sampler(sampler, None);
            }
        }
    }
}
