use std::sync::Arc;

use crate::graphics::vulkan::{buffer::Buffer, Device};

use anyhow::{bail, Result};
use ash::{version::DeviceV1_0, vk};

pub struct TextureImage {
    image: vk::Image,
    extent: vk::Extent3D,

    view: vk::ImageView,
    memory: vk::DeviceMemory,

    device: Arc<Device>,
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
        src: &Buf,
    ) -> Result<()>
    where
        Buf: Buffer,
    {
        self.device.logical_device.begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;

        let required_size = (self.extent.width * self.extent.height * 4) as u64;
        if required_size > src.size_in_bytes() {
            bail!(
                "The texture expects {:?} bytes, but the provided buffer includes only {:?} bytes of data!",
                required_size,
                src.size_in_bytes()
            );
        }

        self.write_barrier(command_buffer);
        self.copy_buffer_to_image(
            command_buffer,
            src.raw(),
            self.extent.width,
            self.extent.height,
        );
        self.read_barrier(command_buffer);

        self.device
            .logical_device
            .end_command_buffer(command_buffer)?;
        self.device.submit_and_wait_idle(
            &self.device.graphics_queue,
            command_buffer,
        )?;

        Ok(())
    }

    /// Transition the image memory layout such that it is an optimal transfer
    /// target.
    pub unsafe fn write_barrier(&self, command_buffer: vk::CommandBuffer) {
        let write_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
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
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .build();
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
    unsafe fn read_barrier(&self, command_buffer: vk::CommandBuffer) {
        let read_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
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
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .build();
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

    /// Copy a buffer's memory into the image memory.
    unsafe fn copy_buffer_to_image(
        &self,
        command_buffer: vk::CommandBuffer,
        src_buffer: vk::Buffer,
        width: u32,
        height: u32,
    ) {
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
