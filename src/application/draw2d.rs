pub mod descriptor_sets;
mod graphics_pipeline;
mod impl_render_target;
mod vertex;

pub use self::{descriptor_sets::UniformBufferObject, vertex::Vertex};

use self::graphics_pipeline::GraphicsPipeline;
use crate::rendering::{
    buffer::{Buffer, CpuBuffer},
    command_pool::TransientCommandPool,
    texture::TextureImage,
    Device, Swapchain,
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

type Mat4 = nalgebra::Matrix4<f32>;

/// Resources used to render triangles
pub struct Draw2d {
    pub vertices: Vec<Vertex>,

    texture_image: TextureImage,
    sampler: vk::Sampler,

    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
    projection: Mat4,
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
            // SAFE:  because the image was only just created and no reference
            //        to the underlying handle is retained
            &unsafe { texture_image.raw_image() },
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
            texture_image.upload_from_buffer(
                command_pool.request_command_buffer()?,
                image_buffer,
            )?;
        }

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
            .max_lod(0.0);

        let sampler = unsafe {
            device
                .logical_device
                .create_sampler(&sampler_create_info, None)?
        };

        let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain)?;
        Ok(Self {
            texture_image,
            sampler,
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
            self.device
                .logical_device
                .destroy_sampler(self.sampler, None);
            self.sampler = vk::Sampler::null();
            self.device.logical_device.device_wait_idle().expect(
                "error while waiting for the device to complete all work",
            );
        }
    }
}
