mod graphics_pipeline;
mod uniforms;
mod vertex;

pub use self::{uniforms::UniformBufferObject, vertex::Vertex};

type Mat4 = nalgebra::Matrix4<f32>;

use self::graphics_pipeline::GraphicsPipeline;
use crate::{
    application::render_context::{Frame, RenderTarget},
    rendering::{buffer::Buffer, Device, Swapchain},
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// Resources used to render triangles
pub struct Triangles {
    pub vertices: Vec<Vertex>,

    graphics_pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
    projection: Mat4,
}

impl RenderTarget for Triangles {
    /// Render the triangle to a single frame.
    fn render_to_frame(
        &mut self,
        image_available: vk::Semaphore,
        frame: &mut Frame,
    ) -> Result<vk::Semaphore> {
        unsafe {
            // safe because this method is only invoked after these resources
            // are done being used by the gpu
            frame
                .uniform_buffer
                .write_data(&[UniformBufferObject::new(self.projection)])?;
            frame.vertex_buffer.write_data(&self.vertices)?;
        }

        let render_buffer = self.record_buffer_commands(
            frame.request_command_buffer()?,
            &frame.framebuffer,
            unsafe { frame.vertex_buffer.raw() },
            frame.descriptor_set,
        )?;

        frame.submit_command_buffers(image_available, &[render_buffer])
    }
}

impl Triangles {
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

    fn record_buffer_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer: &vk::Framebuffer,
        vertex_buffer: vk::Buffer,
        descriptor_set: vk::DescriptorSet,
    ) -> Result<vk::CommandBuffer> {
        // begin the command buffer
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty());

        // begin the render pass
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.swapchain.render_pass)
            .framebuffer(*framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);

        unsafe {
            // begin the command buffer
            self.device
                .logical_device
                .begin_command_buffer(command_buffer, &begin_info)?;

            // begin the render pass
            self.device.logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            // bind the graphics pipeline
            self.device.logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline.pipeline,
            );

            let buffers = [vertex_buffer];
            let offsets = [0];
            self.device.logical_device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &buffers,
                &offsets,
            );

            let descriptor_sets = [descriptor_set];
            self.device.logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline.pipeline_layout,
                0,
                &descriptor_sets,
                &[],
            );

            // draw
            self.device.logical_device.cmd_draw(
                command_buffer,
                self.vertices.len() as u32, // vertex count
                1,                          // instance count
                0,                          // first vertex
                0,                          // first instance
            );

            // end the render pass
            self.device
                .logical_device
                .cmd_end_render_pass(command_buffer);

            // end the buffer
            self.device
                .logical_device
                .end_command_buffer(command_buffer)?;
        }

        Ok(command_buffer)
    }
}

impl Drop for Triangles {
    fn drop(&mut self) {
        unsafe {
            self.device.logical_device.device_wait_idle().expect(
                "error while waiting for the device to complete all work",
            );
        }
    }
}
