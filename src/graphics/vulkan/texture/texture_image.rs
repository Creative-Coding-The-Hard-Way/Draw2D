use super::{MipmapExtent, TextureImage};

use std::sync::Arc;

use crate::graphics::vulkan::{buffer::Buffer, Device};

use anyhow::{bail, Result};
use ash::{version::DeviceV1_0, vk};

impl TextureImage {
    /// The raw image handle used by this texture.
    ///
    /// Unsafe because it is up to the caller to synchronize access to the
    /// image.
    pub unsafe fn raw_image(&self) -> vk::Image {
        self.image
    }

    /// The raw image view used by this texture.
    ///
    /// Unsafe because it is up to the caller to synchronize access to the
    /// view.
    pub unsafe fn raw_view(&self) -> vk::ImageView {
        self.view
    }

    /// Create the image, allocate memory, create a view for the texture.
    ///
    /// Bytes per pixel is used by the various `upload_*` methods when copying
    /// data from a buffer into the image. For example, if the image format
    /// is R8G8B8A8_SRGB then the bytes per pixel is 4.
    pub fn new(
        device: Arc<Device>,
        image_create_info: vk::ImageCreateInfo,
        memory_property_flags: vk::MemoryPropertyFlags,
        bytes_per_pixel: u64,
    ) -> Result<Self> {
        let image = unsafe {
            device
                .logical_device
                .create_image(&image_create_info, None)?
        };

        let allocation = unsafe {
            let memory_requirements =
                device.logical_device.get_image_memory_requirements(image);
            device
                .allocate_memory(memory_requirements, memory_property_flags)?
        };

        unsafe {
            device.logical_device.bind_image_memory(
                image,
                allocation.memory,
                allocation.offset,
            )?;
        }

        let view_create_info = vk::ImageViewCreateInfo {
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: image_create_info.format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: image_create_info.mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            ..Default::default()
        };

        let view = unsafe {
            device
                .logical_device
                .create_image_view(&view_create_info, None)?
        };

        Ok(Self {
            bytes_per_pixel,
            image,
            extent: image_create_info.extent,
            view,
            allocation,
            device,
        })
    }

    /// Upload a texture's data from a buffer.
    ///
    /// This method is just an alias to [Self::upload_mipmaps_from_buffer]
    /// which only updates the first mipmap. It's particularly convenient for
    /// textures which only have a single mipmap level.
    pub unsafe fn upload_from_buffer<Buf>(&mut self, src: &Buf) -> Result<()>
    where
        Buf: Buffer,
    {
        let mipmap_extent = MipmapExtent {
            width: self.extent.width,
            height: self.extent.height,
        };
        self.upload_mipmaps_from_buffer(src, &[mipmap_extent])
    }

    /// Upload a texture's mipmaps from a buffer.
    ///
    /// * This method assumes that each mipmap has the same `bytes_per_pixel`
    ///   as the texture image.
    /// * Order is super important. The first entry in `mipmap_sizes`
    ///   corresponds to the first region of memory in the src bufer. The
    ///   mipmap extents are used to compute the byte offset and size of each
    ///   mipmap region.
    pub unsafe fn upload_mipmaps_from_buffer(
        &mut self,
        src: &impl Buffer,
        mipmap_sizes: &[MipmapExtent],
    ) -> Result<()> {
        let required_size: u64 = mipmap_sizes
            .iter()
            .map(|mipmap_size| mipmap_size.size_in_bytes(self.bytes_per_pixel))
            .sum();
        if required_size > src.size_in_bytes() {
            bail!(
                "The texture expects {:?} bytes, but the provided buffer includes only {:?} bytes of data!",
                required_size,
                src.size_in_bytes()
            );
        }

        self.device.sync_graphics_commands(|command_buffer| {
            let mut mip_level = 0;
            let mut offset: u64 = 0;

            for extent in mipmap_sizes {
                self.write_barrier(command_buffer, mip_level);
                self.copy_buffer_to_image(
                    command_buffer,
                    src.raw(),
                    offset,
                    extent,
                    mip_level,
                );
                self.read_barrier(command_buffer, mip_level);

                mip_level += 1;
                offset += extent.size_in_bytes(self.bytes_per_pixel);
            }

            Ok(())
        })
    }

    /// Transition the image memory layout such that it is an optimal transfer
    /// target.
    pub unsafe fn write_barrier(
        &self,
        command_buffer: vk::CommandBuffer,
        mip_level: u32,
    ) {
        let write_barrier = vk::ImageMemoryBarrier {
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            image: self.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: mip_level,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            ..Default::default()
        };
        self.device.logical_device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[write_barrier],
        );
    }

    /// Transition the image memory layout such that is is optimal for reading
    /// within the fragment shader.
    unsafe fn read_barrier(
        &self,
        command_buffer: vk::CommandBuffer,
        mip_level: u32,
    ) {
        let read_barrier = vk::ImageMemoryBarrier {
            old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image: self.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: mip_level,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            dst_access_mask: vk::AccessFlags::SHADER_READ,
            ..Default::default()
        };
        self.device.logical_device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[read_barrier],
        );
    }

    /// Copy a region of the buffer's memory into the image mipmap.
    unsafe fn copy_buffer_to_image(
        &self,
        command_buffer: vk::CommandBuffer,
        src_buffer: vk::Buffer,
        offset: u64,
        mipmap_extent: &MipmapExtent,
        mip_level: u32,
    ) {
        let region = vk::BufferImageCopy {
            buffer_offset: offset,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D {
                width: mipmap_extent.width,
                height: mipmap_extent.height,
                depth: 1,
            },
        };
        self.device.logical_device.cmd_copy_buffer_to_image(
            command_buffer,
            src_buffer,
            self.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }
}

impl Drop for TextureImage {
    fn drop(&mut self) {
        log::trace!("DESTROY TEXTURE");
        unsafe {
            self.device
                .logical_device
                .destroy_image_view(self.view, None);
            self.device.logical_device.destroy_image(self.image, None);
            self.image = vk::Image::null();
            self.device.free_memory(&self.allocation).unwrap();
        }
    }
}
