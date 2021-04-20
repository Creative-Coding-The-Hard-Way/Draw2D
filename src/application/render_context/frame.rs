mod descriptor;
mod sync;

use self::{descriptor::FrameDescriptor, sync::FrameSync};

use crate::rendering::{
    buffer::CpuBuffer, command_pool::TransientCommandPool, Device,
};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// All per-frame resources and synchronization for this application.
pub struct Frame {
    pub sync: FrameSync,
    pub descriptor: FrameDescriptor,

    pub vertex_buffer: CpuBuffer,

    pub command_pool: TransientCommandPool,

    pub framebuffer: vk::Framebuffer,
    device: Arc<Device>,
}

impl Frame {
    /// Create a collection of frames with resource debug names based on the
    /// frame index.
    pub fn create_n_frames(
        device: &Arc<Device>,
        framebuffers: &[vk::Framebuffer],
    ) -> Result<Vec<Self>> {
        let mut result = vec![];
        for (i, framebuffer) in framebuffers.iter().enumerate() {
            result.push(Self::new(
                device.clone(),
                *framebuffer,
                format!("Frame {}", i),
            )?);
        }
        Ok(result)
    }

    /// Create a new frame.
    ///
    /// Frames do not own framebuffers, it is the responsibility of the
    /// application to ensure no Frame instances are used after the swapchain
    /// has been dropped.
    pub fn new<Name>(
        device: Arc<Device>,
        framebuffer: vk::Framebuffer,
        name: Name,
    ) -> Result<Self>
    where
        Name: Into<String> + Clone,
    {
        Ok(Self {
            sync: FrameSync::new(&device, name.clone())?,
            framebuffer,
            command_pool: TransientCommandPool::new(
                device.clone(),
                name.clone(),
            )?,
            vertex_buffer: CpuBuffer::new(
                device.clone(),
                vk::BufferUsageFlags::VERTEX_BUFFER,
            )?,
            descriptor: FrameDescriptor::new(device.clone(), name.clone())?,

            device,
        })
    }

    /// Begin the frame's rendering operations.
    ///
    /// Blocks until the previous render with this frame has finished.
    /// Resets the command pool used by this frame.
    pub fn begin_frame(&mut self) -> Result<()> {
        unsafe {
            self.wait_for_graphics_to_complete()?;
            self.command_pool.reset()?;
        }
        Ok(())
    }

    /// Submit all graphics commands for this frame.
    ///
    /// The submission signals the `sync.graphics_finished_fence` which will
    /// block the next call to `begin_frame`.
    ///
    /// This command yields a semaphore which can be used for scheduling work
    /// on the GPU - like presenting the swapchain image.
    pub fn submit_graphics_commands(
        &mut self,
        wait_semaphore: vk::Semaphore,
        command_buffers: &[vk::CommandBuffer],
    ) -> Result<vk::Semaphore> {
        let wait_semaphores = [wait_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let render_finished_signal_semaphores =
            [self.sync.render_finished_semaphore];
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&render_finished_signal_semaphores)
            .build()];

        unsafe {
            let graphics_queue = self.device.graphics_queue.acquire();
            self.device
                .logical_device
                .queue_submit(
                    *graphics_queue,
                    &submit_info,
                    self.sync.graphics_finished_fence,
                )
                .with_context(|| "unable to submit graphics commands!")?;
        }
        Ok(self.sync.render_finished_semaphore)
    }

    /// Called at the beginning of each frame.
    ///
    /// Block until this frame's prior graphics submission has completed, then
    /// reset the fences. Unsafe because this function must be considered in
    /// the context of a full frame and how rendering commansd are submitted.
    unsafe fn wait_for_graphics_to_complete(&mut self) -> Result<()> {
        self.device
            .logical_device
            .wait_for_fences(
                &[self.sync.graphics_finished_fence],
                true,
                u64::MAX,
            )
            .with_context(|| {
                "error while waiting for the graphics fence to complete!"
            })?;
        self.device
            .logical_device
            .reset_fences(&[self.sync.graphics_finished_fence])
            .with_context(|| "unable to reset the graphics fence!")?;
        Ok(())
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            self.wait_for_graphics_to_complete()
                .expect("error while waiting for resources to clear");
            self.sync.destroy(&self.device);
        }
    }
}
