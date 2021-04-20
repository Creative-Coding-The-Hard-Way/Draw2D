use crate::{
    application::graphics::{Draw2d, Frame},
    rendering::buffer::Buffer,
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// Use Frame resources to record a one-time use CommandBuffer which actually
/// renders the draw2d render pass.
pub fn record(draw2d: &Draw2d, frame: &mut Frame) -> Result<vk::CommandBuffer> {
    let command_buffer = frame.command_pool.request_command_buffer()?;
    unsafe {
        // begin the command buffer
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty());

        draw2d
            .device
            .logical_device
            .begin_command_buffer(command_buffer, &begin_info)?;

        // begin the render pass
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(draw2d.swapchain.render_pass)
            .framebuffer(frame.framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: draw2d.swapchain.extent,
            })
            .clear_values(&clear_values);
        draw2d.device.logical_device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        );

        // bind the graphics pipeline
        draw2d.device.logical_device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            draw2d.graphics_pipeline.pipeline,
        );

        let buffers = [frame.vertex_buffer.raw()];
        let offsets = [0];
        draw2d.device.logical_device.cmd_bind_vertex_buffers(
            command_buffer,
            0,
            &buffers,
            &offsets,
        );

        let descriptor_sets = [frame.descriptor.raw_descriptor_set()];
        draw2d.device.logical_device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            draw2d.graphics_pipeline.pipeline_layout,
            0,
            &descriptor_sets,
            &[],
        );

        // draw
        draw2d.device.logical_device.cmd_draw(
            command_buffer,
            draw2d.vertices.len() as u32, // vertex count
            1,                            // instance count
            0,                            // first vertex
            0,                            // first instance
        );

        // end the render pass
        draw2d
            .device
            .logical_device
            .cmd_end_render_pass(command_buffer);

        // end the buffer
        draw2d
            .device
            .logical_device
            .end_command_buffer(command_buffer)?;
    }
    Ok(command_buffer)
}