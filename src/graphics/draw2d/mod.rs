pub(crate) mod descriptor_sets;
pub(crate) mod layer;

mod commands;
mod graphics_pipeline;
mod vertex;

pub use self::{
    descriptor_sets::UniformBufferObject,
    layer::{Layer, LayerHandle, StackedLayers},
    vertex::Vertex,
};

use self::graphics_pipeline::GraphicsPipeline;
use super::Frame;

use crate::graphics::{
    texture_atlas::TextureAtlas,
    vulkan::{Device, Swapchain},
};

use anyhow::Result;
use ash::version::DeviceV1_0;
use std::sync::Arc;

type Mat4 = nalgebra::Matrix4<f32>;

/// Resources used to render triangles
pub struct Draw2d {
    pub(super) layer_stack: StackedLayers,
    pub(super) projection: Mat4,

    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,

    device: Arc<Device>,
}

impl Draw2d {
    /// Create a new Triangle subsystem which knows how to render itself to a
    /// single frame.
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Self> {
        let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain)?;
        Ok(Self {
            layer_stack: StackedLayers::default(),
            graphics_pipeline,
            projection: Mat4::identity(),
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
        Ok(())
    }

    /// Render to a single application frame.
    pub fn draw_frame(
        &self,
        frame: &mut Frame,
        texture_atlas: &impl TextureAtlas,
    ) -> Result<()> {
        // Fill per-frame gpu resources with the relevant data.
        // SAFE: because resources are not shared between frames.
        unsafe {
            frame.descriptor.update_ubo(&UniformBufferObject {
                projection: self.projection.into(),
            })?;
            frame.descriptor.update_texture_atlas(texture_atlas);
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
