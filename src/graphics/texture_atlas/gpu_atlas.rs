use crate::graphics::{
    ext::Texture2dFactory,
    texture_atlas::{
        AtlasVersion, SamplerHandle, TextureAtlas, TextureHandle,
        MAX_SUPPORTED_TEXTURES,
    },
    vulkan::{buffer::CpuBuffer, texture::TextureImage, Device},
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
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

    /// The version be used to determine when a shader's descriptors need to
    /// be updated.
    version: AtlasVersion,

    /// A handle to the vulkan device.
    device: Arc<Device>,
}

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

        let default_texture = unsafe {
            let mut transfer_buffer = CpuBuffer::new(
                device.clone(),
                vk::BufferUsageFlags::TRANSFER_SRC,
            )?;

            let white_pixel: [u8; 4] = [255, 255, 255, 255];
            transfer_buffer.write_data(&white_pixel)?;

            let mut tex = device.create_empty_2d_texture("default", 1, 1, 1)?;
            tex.upload_from_buffer(&transfer_buffer)?;
            tex
        };

        let mut bindings = vec![];
        bindings.reserve(MAX_SUPPORTED_TEXTURES);

        bindings.push(Some(Binding {
            texture: default_texture,
            sampler_handle: SamplerHandle::default(),
        }));

        for _ in 1..MAX_SUPPORTED_TEXTURES {
            bindings.push(None);
        }

        Ok(Self {
            textures: bindings,
            version: AtlasVersion::new_out_of_date().increment(),
            samplers: vec![sampler],
            device,
        })
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
    fn add_texture(&mut self, texture: TextureImage) -> Result<TextureHandle> {
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

    /// # Unsafe Because
    ///
    /// - the caller must make sure the atlas is not in use when this method
    ///   is called
    unsafe fn take_texture(
        &mut self,
        texture_handle: TextureHandle,
    ) -> Result<TextureImage> {
        use anyhow::Context;

        let texture = self.textures[texture_handle.texture_index() as usize]
            .take()
            .context("no texture bound with that texture handle!")?
            .texture;

        self.version = self.version.increment();

        Ok(texture)
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
