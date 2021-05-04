use super::Graphics;

use crate::graphics::{
    frame::Frame, pipeline2d::PushConsts, vulkan::buffer::Buffer,
    vulkan::ffi::any_as_u8_slice,
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// Use Frame resources to record a one-time use CommandBuffer which actually
/// renders the draw2d render pass.
impl Graphics {
    /// Record a command buffer for rendering each graphics layer in a single
    /// pass.
    pub(super) fn record_layer_draw_commands(
        &mut self,
        frame: &mut Frame,
    ) -> Result<vk::CommandBuffer> {
        let command_buffer = self.begin_frame_commands(frame)?;
        unsafe {
            self.device.logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                *self.pipeline2d.raw_pipeline(),
            );

            let descriptor_sets = [frame.descriptor.raw_descriptor_set()];
            self.device.logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                *self.pipeline2d.raw_pipeline_layout(),
                0,
                &descriptor_sets,
                &[],
            );

            let buffers = [frame.vertex_buffer.raw()];
            let offsets = [0];
            self.device.logical_device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &buffers,
                &offsets,
            );

            let mut offset: u32 = 0;
            for layer in self.layer_stack.layers() {
                for batch in layer.batches() {
                    let consts = PushConsts {
                        projection: (*layer.projection()).into(),
                        texture_index: batch.texture_handle.texture_index(),
                    };
                    self.device.logical_device.cmd_push_constants(
                        command_buffer,
                        *self.pipeline2d.raw_pipeline_layout(),
                        vk::ShaderStageFlags::FRAGMENT
                            | vk::ShaderStageFlags::VERTEX,
                        0,
                        any_as_u8_slice(&consts),
                    );
                    self.device.logical_device.cmd_draw(
                        command_buffer,
                        batch.vertices.len() as u32, // vertex count
                        1,                           // instance count
                        offset,                      // first vertex
                        0,                           // first instance
                    );
                    offset += batch.vertices.len() as u32;
                }
            }
        }
        self.end_frame_commands(command_buffer)?;
        Ok(command_buffer)
    }

    pub(super) fn record_no_op_commands(
        &mut self,
        frame: &mut Frame,
    ) -> Result<vk::CommandBuffer> {
        let command_buffer = self.begin_frame_commands(frame)?;
        self.end_frame_commands(command_buffer)?;
        Ok(command_buffer)
    }

    fn begin_frame_commands(
        &mut self,
        frame: &mut Frame,
    ) -> Result<vk::CommandBuffer> {
        let command_buffer = frame.command_pool.request_command_buffer()?;
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty());
        unsafe {
            self.device
                .logical_device
                .begin_command_buffer(command_buffer, &begin_info)?;
        }
        // begin the render pass
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.frame_context.swapchain().render_pass)
            .framebuffer(frame.framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.frame_context.swapchain().extent,
            })
            .clear_values(&clear_values);
        unsafe {
            self.device.logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
        Ok(command_buffer)
    }

    fn end_frame_commands(
        &self,
        command_buffer: vk::CommandBuffer,
    ) -> Result<()> {
        unsafe {
            // end the render pass
            self.device
                .logical_device
                .cmd_end_render_pass(command_buffer);

            self.device
                .logical_device
                .end_command_buffer(command_buffer)?;
        }
        Ok(())
    }
}
