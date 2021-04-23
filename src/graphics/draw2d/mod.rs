mod atlas;
mod commands;
pub mod descriptor_sets;
mod graphics_pipeline;
mod vertex;

pub use self::{descriptor_sets::UniformBufferObject, vertex::Vertex};

use self::{atlas::TextureAtlas, graphics_pipeline::GraphicsPipeline};
use super::Frame;

use crate::graphics::vulkan::{Device, Swapchain};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

type Mat4 = nalgebra::Matrix4<f32>;

/// Resources used to render triangles
pub struct Draw2d {
    projection: Mat4,
    pub vertices: Vec<Vertex>,

    texture_atlas: TextureAtlas,
    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
}

impl Draw2d {
    /// Create a new Triangle subsystem which knows how to render itself to a
    /// single frame.
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Self> {
        let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain)?;
        let mut texture_atlas = TextureAtlas::new(device.clone())?;
        texture_atlas.add_texture("assets/example.png")?;
        Ok(Self {
            texture_atlas,
            vertices: vec![],
            graphics_pipeline,
            projection: Self::ortho(2.0, swapchain.extent),
            swapchain,
            device,
        })
    }

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

    /// Render to a single application frame.
    pub fn draw_frame(&self, frame: &mut Frame) -> Result<()> {
        // Fill per-frame gpu resources with the relevant data.
        // SAFE: because resources are not shared between frames.
        unsafe {
            frame.descriptor.update_ubo(&UniformBufferObject {
                projection: self.projection.into(),
            })?;
            frame.descriptor.write_texture_descriptor(
                &self.texture_atlas.build_descriptor_image_info(),
            );
            frame.vertex_buffer.write_data(&self.vertices)?;
        }

        let graphics_commands = commands::record(self, frame)?;
        frame.submit_graphics_commands(&[graphics_commands]);

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
