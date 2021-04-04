//! This module provides structures for managing a collection of command
//! buffers for a given command pool.

use crate::rendering::Device;

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// This struct holds a command pool and tracks which buffers have been
/// allocated.
///
/// It allows easy reuse of transient command buffer allocations between
/// frames.
///
/// It is the responsibility of the caller to synchronize resets and
/// destruction.
pub struct TransientCommandPool {
    command_pool: vk::CommandPool,
    allocated_command_buffers: Vec<vk::CommandBuffer>,
    available_command_buffers: Vec<vk::CommandBuffer>,
    device: Arc<Device>,
}

impl TransientCommandPool {
    pub fn new<Name>(device: Arc<Device>, name: Name) -> Result<Self>
    where
        Name: Into<String>,
    {
        Ok(Self {
            command_pool: Self::create_command_pool(&device, name)?,
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
        self.device
            .logical_device
            .reset_command_pool(
                self.command_pool,
                vk::CommandPoolResetFlags::empty(),
            )
            .with_context(|| {
                "unable to reset the command pool for this frame!"
            })?;
        self.available_command_buffers = self.allocated_command_buffers.clone();
        Ok(())
    }

    /// Allocate a new command buffer
    fn allocate_command_buffer(&mut self) -> Result<vk::CommandBuffer> {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe {
            self.device
                .logical_device
                .allocate_command_buffers(&create_info)
                .context("unable to allocate command buffer")?
        };
        self.allocated_command_buffers.push(command_buffer[0]);
        Ok(command_buffer[0])
    }

    /// Create the command buffer pool.
    ///
    /// The caller is responsible for destroying the pool.
    fn create_command_pool<Name>(
        device: &Device,
        name: Name,
    ) -> Result<vk::CommandPool>
    where
        Name: Into<String>,
    {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(device.graphics_queue.family_id)
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);
        let command_pool = unsafe {
            device
                .logical_device
                .create_command_pool(&create_info, None)
                .context("unable to create the command pool")?
        };
        device.name_vulkan_object(
            format!("{} Command Pool", name.into()),
            vk::ObjectType::COMMAND_POOL,
            &command_pool,
        )?;
        Ok(command_pool)
    }
}

impl Drop for TransientCommandPool {
    /// The owner of the TransientCommandPool must ensure that all usage of
    /// the command buffers has completed prior to dropping.
    fn drop(&mut self) {
        unsafe {
            self.available_command_buffers.clear();
            self.device.logical_device.free_command_buffers(
                self.command_pool,
                &self.allocated_command_buffers,
            );
            self.allocated_command_buffers.clear();
            self.device
                .logical_device
                .destroy_command_pool(self.command_pool, None);
        }
    }
}
