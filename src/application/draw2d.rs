mod graphics_pipeline;
mod impl_render_target;
mod uniforms;
mod vertex;

pub use self::{uniforms::UniformBufferObject, vertex::Vertex};

use self::graphics_pipeline::GraphicsPipeline;
use crate::rendering::{
    buffer::{Buffer, CpuBuffer},
    command_pool::TransientCommandPool,
    Device, Swapchain,
};

use anyhow::{bail, Result};
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

type Mat4 = nalgebra::Matrix4<f32>;

struct TextureImage {
    device: Arc<Device>,

    image: vk::Image,
    memory: vk::DeviceMemory,
}

/// Resources used to render triangles
pub struct Draw2d {
    pub vertices: Vec<Vertex>,

    image_buffer: CpuBuffer,
    texture_image: TextureImage,

    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
    projection: Mat4,
}

#[derive(Clone, Copy)]
enum TransitionType {
    READ,
    WRITE,
}

impl TextureImage {
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

        Ok(Self {
            device,
            image,
            memory,
        })
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
        self.device.logical_device.begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;

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

        self.device
            .logical_device
            .end_command_buffer(command_buffer)?;
        self.device.submit_and_wait_idle(
            &self.device.graphics_queue,
            command_buffer,
        )?;
        Ok(())
    }

    pub unsafe fn copy_buffer_to_image(
        &mut self,
        command_buffer: vk::CommandBuffer,
        src_buffer: vk::Buffer,
        width: u32,
        height: u32,
    ) -> Result<()> {
        self.device.logical_device.begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;

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

        self.device
            .logical_device
            .end_command_buffer(command_buffer)?;
        self.device.submit_and_wait_idle(
            &self.device.graphics_queue,
            command_buffer,
        )?;

        Ok(())
    }
}

impl Drop for TextureImage {
    fn drop(&mut self) {
        unsafe {
            self.device.logical_device.destroy_image(self.image, None);
            self.image = vk::Image::null();
            self.device.logical_device.free_memory(self.memory, None);
            self.memory = vk::DeviceMemory::null();
        }
    }
}

impl Draw2d {
    /// Create a new Triangle subsystem which knows how to render itself to a
    /// single frame.
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Self> {
        let example = image::open("assets/example.png")?.into_rgba8();
        log::info!(
            "opened image with dims: {}, {}",
            example.width(),
            example.height()
        );
        let width = example.width();
        let height = example.height();

        let mut texture_image = TextureImage::new(
            device.clone(),
            vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .format(vk::Format::R8G8B8A8_SRGB)
                .tiling(vk::ImageTiling::OPTIMAL)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usage(
                    vk::ImageUsageFlags::TRANSFER_DST
                        | vk::ImageUsageFlags::SAMPLED,
                )
                .samples(vk::SampleCountFlags::TYPE_1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        device.name_vulkan_object(
            "Example Texture",
            vk::ObjectType::IMAGE,
            &texture_image.image,
        )?;

        let mut image_buffer =
            CpuBuffer::new(device.clone(), vk::BufferUsageFlags::TRANSFER_SRC)?;

        unsafe {
            image_buffer.write_data(&example.into_raw())?;
        }

        device.name_vulkan_object(
            "Image Data",
            vk::ObjectType::BUFFER,
            &unsafe { image_buffer.raw() },
        )?;

        log::info!("gpu image buffer size: {}", image_buffer.size_in_bytes());

        unsafe {
            let mut command_pool = TransientCommandPool::new(
                device.clone(),
                "texture setup pool",
            )?;
            texture_image.transition_image_layout(
                command_pool.request_command_buffer()?,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            )?;
            texture_image.copy_buffer_to_image(
                command_pool.request_command_buffer()?,
                image_buffer.raw(),
                width,
                height,
            )?;
            texture_image.transition_image_layout(
                command_pool.request_command_buffer()?,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            )?;
        }

        let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain)?;
        Ok(Self {
            image_buffer,
            texture_image,
            vertices: vec![],
            graphics_pipeline,
            projection: Self::ortho(2.0, swapchain.extent),
            swapchain,
            device,
        })
    }

    /// Replace the swapchain and all dependent resources in the Triangle
    /// subsystem.
    pub fn replace_swapchain(
        &mut self,
        swapchain: Arc<Swapchain>,
    ) -> Result<()> {
        self.swapchain = swapchain;
        self.graphics_pipeline =
            GraphicsPipeline::new(&self.device, &self.swapchain)?;
        self.projection = Self::ortho(2.0, self.swapchain.extent);
        Ok(())
    }

    /// Build a orthographic projection using an extent to compute the aspect
    /// ratio for the screen. The height is fixed and the width varies to account
    /// for the aspect ratio.
    fn ortho(screen_height: f32, extent: vk::Extent2D) -> Mat4 {
        let aspect = extent.width as f32 / extent.height as f32;
        Mat4::new_orthographic(
            -aspect * screen_height / 2.0,
            aspect * screen_height / 2.0,
            -screen_height / 2.0,
            screen_height / 2.0,
            1.0,
            -1.0,
        )
    }
}

impl Drop for Draw2d {
    fn drop(&mut self) {
        unsafe {
            self.device.logical_device.device_wait_idle().expect(
                "error while waiting for the device to complete all work",
            );
        }
    }
}
