mod graphics_pipeline;
mod impl_render_target;
mod uniforms;
mod vertex;

pub use self::{uniforms::UniformBufferObject, vertex::Vertex};

use self::graphics_pipeline::GraphicsPipeline;
use crate::rendering::{
    buffer::{Buffer, CpuBuffer},
    Device, Swapchain,
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

type Mat4 = nalgebra::Matrix4<f32>;

/// Resources used to render triangles
pub struct Draw2d {
    pub vertices: Vec<Vertex>,

    image_buffer: CpuBuffer,

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

        let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain)?;
        Ok(Self {
            image_buffer,
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
