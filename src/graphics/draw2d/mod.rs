pub(crate) mod descriptor_sets;

mod commands;
mod graphics_pipeline;

use self::graphics_pipeline::GraphicsPipeline;
use super::{layer::LayerStack, Frame};

use crate::graphics::{
    texture_atlas::TextureAtlas,
    vulkan::{Device, Swapchain},
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// Resources used to render triangles
pub struct Draw2d {
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
            graphics_pipeline,
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
        layers: &LayerStack,
    ) -> Result<()> {
        let all_vertices = layers.vertices();
        if all_vertices.len() == 0 {
            let graphics_commands = self.no_op_commands(frame)?;
            frame.submit_graphics_commands(&[graphics_commands]);
        } else {
            // Fill per-frame gpu resources with the relevant data.
            // SAFE: because resources are not shared between frames.
            unsafe {
                frame.descriptor.update_texture_atlas(texture_atlas);
                frame.vertex_buffer.write_data_arrays(&all_vertices)?;
            }

            let graphics_commands = commands::record(self, frame, layers)?;
            frame.submit_graphics_commands(&[graphics_commands]);
        }

        Ok(())
    }

    fn no_op_commands(&self, frame: &mut Frame) -> Result<vk::CommandBuffer> {
        let command_buffer = frame.command_pool.request_command_buffer()?;
        unsafe {
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();
            self.device
                .logical_device
                .begin_command_buffer(command_buffer, &begin_info)?;

            // begin the render pass
            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.swapchain.render_pass)
                .framebuffer(frame.framebuffer)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain.extent,
                })
                .clear_values(&clear_values);
            self.device.logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            // end the render pass
            self.device
                .logical_device
                .cmd_end_render_pass(command_buffer);

            self.device
                .logical_device
                .end_command_buffer(command_buffer)?;
        }
        Ok(command_buffer)
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
