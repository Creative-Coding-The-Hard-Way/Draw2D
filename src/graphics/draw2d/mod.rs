pub(super) mod descriptor_sets;
pub(super) mod layer;
pub(super) mod texture_atlas;

mod commands;
mod graphics_pipeline;
mod vertex;

pub use self::{
    descriptor_sets::UniformBufferObject,
    layer::{Layer, LayerHandle, StackedLayers},
    texture_atlas::TextureHandle,
    vertex::Vertex,
};

use self::{graphics_pipeline::GraphicsPipeline, texture_atlas::TextureAtlas};
use super::Frame;

use crate::graphics::vulkan::{Device, Swapchain};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

type Mat4 = nalgebra::Matrix4<f32>;

/// Resources used to render triangles
pub struct Draw2d {
    pub layer_stack: StackedLayers,
    pub texture_atlas: TextureAtlas,

    projection: Mat4,

    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
}

impl Draw2d {
    /// Create a new Triangle subsystem which knows how to render itself to a
    /// single frame.
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Self> {
        let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain)?;
        let texture_atlas = TextureAtlas::new(device.clone())?;
        Ok(Self {
            texture_atlas,
            layer_stack: StackedLayers::default(),
            graphics_pipeline,
            projection: Self::ortho(swapchain.extent),
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
        self.projection = Self::ortho(self.swapchain.extent);
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
            frame.descriptor.update_texture_atlas(&self.texture_atlas);
            let all_vertices: Vec<&[Vertex]> = self
                .layer_stack
                .layers()
                .iter()
                .map(|layer| layer.vertices())
                .collect();
            frame.vertex_buffer.write_data_arrays(&all_vertices)?;
        }

        let graphics_commands = commands::record(self, frame)?;
        frame.submit_graphics_commands(&[graphics_commands]);

        Ok(())
    }

    /// Build a orthographic projection using an extent to compute the aspect
    /// ratio for the screen. The height is fixed and the width varies to account
    /// for the aspect ratio.
    fn ortho(extent: vk::Extent2D) -> Mat4 {
        let width = extent.width as f32;
        let height = extent.height as f32;
        Mat4::new_orthographic(
            -width / 2.0,
            width / 2.0,
            -height / 2.0,
            height / 2.0,
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
