use std::sync::Arc;

use crate::rendering::{buffer::Buffer, Device};

use anyhow::{bail, Result};
use ash::{version::DeviceV1_0, vk};

pub struct TextureImage {
    image: vk::Image,
    extent: vk::Extent3D,

    view: vk::ImageView,
    memory: vk::DeviceMemory,

    device: Arc<Device>,
}

#[derive(Clone, Copy)]
enum TransitionType {
    READ,
    WRITE,
}

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

    /// Create a new texture image which manages all of the resources needed to
    /// use an image as a gpu texture.
    pub fn new(
        device: Arc<Device>,
        image_create_info: vk::ImageCreateInfo,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        let image = unsafe {
            device
                .logical_device
                .create_image(&image_create_info, None)?
        };

        let memory = unsafe {
            let memory_requirements =
                device.logical_device.get_image_memory_requirements(image);
            device
                .allocate_memory(memory_requirements, memory_property_flags)?
        };

        unsafe {
            device.logical_device.bind_image_memory(image, memory, 0)?;
        }

        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image_create_info.format)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            });

        let view = unsafe {
            device
                .logical_device
                .create_image_view(&view_create_info, None)?
        };

        Ok(Self {
            image,
            extent: image_create_info.extent,

            view,
            memory,

            device,
        })
    }

    pub unsafe fn upload_from_buffer<Buf>(
        &mut self,
        command_buffer: vk::CommandBuffer,
        src: Buf,
    ) -> Result<()>
    where
        Buf: Buffer,
    {
        self.device.logical_device.begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;

        self.transition_image_layout(
            command_buffer,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;
        self.copy_buffer_to_image(
            command_buffer,
            src.raw(),
            self.extent.width,
            self.extent.height,
        )?;
        self.transition_image_layout(
            command_buffer,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        self.device
            .logical_device
            .end_command_buffer(command_buffer)?;
        self.device.submit_and_wait_idle(
            &self.device.graphics_queue,
            command_buffer,
        )?;

        Ok(())
    }

    /// Transition this command buffer's layout.
    /// Commands are executed synchronously. The provided command buffer must
    /// be new and can safely be discarded when this function returns.
    ///
    /// Unsafe because the image must not otherwise be in use when this method
    /// is invoked.
    pub unsafe fn transition_image_layout(
        &mut self,
        command_buffer: vk::CommandBuffer,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> Result<()> {
        let transition_type = if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            TransitionType::WRITE
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            TransitionType::READ
        } else {
            bail!("invalid layout combinations!")
        };

        let (src_access_mask, src_stage_mask) = match transition_type {
            TransitionType::WRITE => (
                vk::AccessFlags::empty(),
                vk::PipelineStageFlags::TOP_OF_PIPE,
            ),
            TransitionType::READ => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TRANSFER,
            ),
        };

        let (dst_access_mask, dst_stage_mask) = match transition_type {
            TransitionType::WRITE => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            TransitionType::READ => (
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
        };

        let image_memory_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .image(self.image)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .build();

        self.device.logical_device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[image_memory_barrier],
        );

        Ok(())
    }

    pub unsafe fn copy_buffer_to_image(
        &mut self,
        command_buffer: vk::CommandBuffer,
        src_buffer: vk::Buffer,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .build();

        self.device.logical_device.cmd_copy_buffer_to_image(
            command_buffer,
            src_buffer,
            self.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );

        Ok(())
    }
}

impl Drop for TextureImage {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_image_view(self.view, None);
            self.device.logical_device.destroy_image(self.image, None);
            self.image = vk::Image::null();
            self.device.logical_device.free_memory(self.memory, None);
            self.memory = vk::DeviceMemory::null();
        }
    }
}
