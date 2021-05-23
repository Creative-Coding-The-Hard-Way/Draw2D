use crate::graphics::vulkan::Device;

use super::OwnedCommandPool;

use anyhow::Result;
use ash::vk;
use std::sync::Arc;

/// This struct holds a command pool and tracks which buffers have been
/// allocated.
///
/// It allows easy reuse of transient command buffer allocations between
/// frames.
///
/// It is the responsibility of the caller to synchronize resets and
/// destruction.
pub struct ReusableCommandPool {
    command_pool: OwnedCommandPool,
    allocated_command_buffers: Vec<vk::CommandBuffer>,
    available_command_buffers: Vec<vk::CommandBuffer>,
    device: Arc<Device>,
}

impl ReusableCommandPool {
    pub fn new(
        device: Arc<Device>,
        debug_name: impl Into<String>,
    ) -> Result<Self> {
        let command_pool = OwnedCommandPool::new(
            &device.logical_device,
            device.graphics_queue.family_id,
        )?;

        device.name_vulkan_object(
            format!("{} Command Pool", debug_name.into()),
            vk::ObjectType::COMMAND_POOL,
            unsafe { command_pool.raw() },
        )?;

        Ok(Self {
            command_pool,
            allocated_command_buffers: vec![],
            available_command_buffers: vec![],
            device,
        })
    }

    /// Request a command buffer.
    ///
    /// A command buffer is available if it has been allocated and has not been
    /// requested since the last call to `reset`.
    ///
    /// The returned buffer is owned by this pool, the caller should not retain
    /// a reference to the buffer beyond the next call to `reset`.
    pub fn request_command_buffer(&mut self) -> Result<vk::CommandBuffer> {
        if let Some(buffer) = self.available_command_buffers.pop() {
            Ok(buffer)
        } else {
            self.allocate_command_buffer()
        }
    }

    /// Reset the command pool and mark return all allocated as available for
    /// use again.
    ///
    /// Unsafe because the caller must ensure that the GPU is done with all of
    /// the allocated command buffers prior to calling this function.
    pub unsafe fn reset(&mut self) -> Result<()> {
        self.command_pool.reset(&self.device.logical_device)?;
        self.available_command_buffers = self.allocated_command_buffers.clone();
        Ok(())
    }

    /// Allocate a new command buffer
    fn allocate_command_buffer(&mut self) -> Result<vk::CommandBuffer> {
        let command_buffer = unsafe {
            self.command_pool
                .allocate_command_buffer(&self.device.logical_device)?
        };
        self.allocated_command_buffers.push(command_buffer);
        Ok(command_buffer)
    }
}

impl Drop for ReusableCommandPool {
    /// The owner of the TransientCommandPool must ensure that all usage of
    /// the command buffers has completed prior to dropping.
    fn drop(&mut self) {
        unsafe {
            for buffer in &self.allocated_command_buffers {
                self.command_pool
                    .free_command_buffer(&self.device.logical_device, *buffer);
            }
            self.available_command_buffers.clear();
            self.allocated_command_buffers.clear();
            self.command_pool.destroy(&self.device.logical_device);
        }
    }
}
