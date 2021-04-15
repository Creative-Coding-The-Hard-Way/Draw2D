mod graphics_pipeline;
mod impl_render_target;
mod uniforms;
mod vertex;

pub use self::{uniforms::UniformBufferObject, vertex::Vertex};

use self::graphics_pipeline::GraphicsPipeline;
use crate::rendering::{Device, Swapchain};

use anyhow::Result;
use ash::version::DeviceV1_0;
use std::sync::Arc;

type Mat4 = nalgebra::Matrix4<f32>;

/// Resources used to render triangles
pub struct Draw2d {
    pub vertices: Vec<Vertex>,

    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
    projection: Mat4,
}

impl Draw2d {
    /// Create a new Triangle subsystem which knows how to render itself to a
    /// single frame.
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Self> {
        let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain)?;

        let aspect =
            swapchain.extent.width as f32 / swapchain.extent.height as f32;
        let height = 2.0;
        let width = aspect * height;

        Ok(Self {
            vertices: vec![],
            graphics_pipeline,
            swapchain,
            projection: Mat4::new_orthographic(
                -width / 2.0,
                width / 2.0,
                -height / 2.0,
                height / 2.0,
                1.0,
                -1.0,
            ),
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

        let [uwidth, uheight] =
            [self.swapchain.extent.width, self.swapchain.extent.height];
        let aspect = uwidth as f32 / uheight as f32;
        let height = 2.0;
        let width = aspect * height;

        self.projection = Mat4::new_orthographic(
            -width / 2.0,
            width / 2.0,
            -height / 2.0,
            height / 2.0,
            1.0,
            -1.0,
        );
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
