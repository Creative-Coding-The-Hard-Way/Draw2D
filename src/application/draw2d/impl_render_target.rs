use super::UniformBufferObject;

use crate::{
    application::{
        render_context::{Frame, RenderTarget},
        Draw2d,
    },
    rendering::buffer::Buffer,
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

impl RenderTarget for Draw2d {
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

        let render_buffer = record_buffer_commands(
            &self,
            frame.request_command_buffer()?,
            &frame.framebuffer,
            unsafe { frame.vertex_buffer.raw() },
            frame.descriptor_set,
        )?;

        frame.submit_command_buffers(image_available, &[render_buffer])
    }
}

fn record_buffer_commands(
    draw2d: &Draw2d,
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
        .render_pass(draw2d.swapchain.render_pass)
        .framebuffer(*framebuffer)
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: draw2d.swapchain.extent,
        })
        .clear_values(&clear_values);

    unsafe {
        // begin the command buffer
        draw2d
            .device
            .logical_device
            .begin_command_buffer(command_buffer, &begin_info)?;

        // begin the render pass
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

        let buffers = [vertex_buffer];
        let offsets = [0];
        draw2d.device.logical_device.cmd_bind_vertex_buffers(
            command_buffer,
            0,
            &buffers,
            &offsets,
        );

        let descriptor_sets = [descriptor_set];
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
