use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};

/// This structure holds resources for managing an owned command pool.
/// "Owned" means that the owner is responsible for destroying the contained
/// resources before this struct is dropped.
pub struct OwnedCommandPool {
    command_pool: vk::CommandPool,
}

impl OwnedCommandPool {
    /// Create the command buffer pool.
    ///
    /// The caller is responsible for destroying the pool.
    pub fn new(
        logical_device: &ash::Device,
        queue_family_index: u32,
    ) -> Result<Self> {
        let create_info = vk::CommandPoolCreateInfo {
            queue_family_index,
            flags: vk::CommandPoolCreateFlags::TRANSIENT,
            ..Default::default()
        };
        let command_pool = unsafe {
            logical_device
                .create_command_pool(&create_info, None)
                .context("unable to create the command pool")?
        };

        Ok(Self { command_pool })
    }

    /// The raw command pool handle.
    ///
    /// # Unsafe Because
    ///
    /// - The returned reference is still logically 'owned' by this struct and
    ///   must not be destroyed except via a call to [Self::destroy].
    pub unsafe fn raw(&self) -> &vk::CommandPool {
        &self.command_pool
    }

    /// Allocate a new command buffer.
    ///
    /// # Unsafe Because
    ///
    /// - the caller must eventually call [Self::reset] or else resources will
    ///   be leaked
    pub unsafe fn allocate_command_buffer(
        &self,
        logical_device: &ash::Device,
    ) -> Result<vk::CommandBuffer> {
        let create_info = vk::CommandBufferAllocateInfo {
            command_pool: self.command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        };
        let command_buffer = logical_device
            .allocate_command_buffers(&create_info)
            .context("unable to allocate command buffer")?;
        Ok(command_buffer[0])
    }

    /// Free a command buffer's resources to be used by the pool again.
    ///
    /// # Unsafe Because
    ///
    /// - the caller is responsible for ensuring that the command buffer is
    ///   not being used anymore
    /// - only command buffers created by previous calls to
    ///   [Self::allocate_command_buffer] on this instance can be passed to
    ///   this function
    pub unsafe fn free_command_buffer(
        &self,
        logical_device: &ash::Device,
        command_buffer: vk::CommandBuffer,
    ) {
        let buffers = [command_buffer];
        logical_device.free_command_buffers(self.command_pool, &buffers);
    }

    /// Reset all command buffers allocated by this pool.
    ///
    /// # Unsafe Because
    ///
    /// - the caller is responsible for ensuring that none of the command
    ///   buffers are still in use
    pub unsafe fn reset(&self, logical_device: &ash::Device) -> Result<()> {
        logical_device
            .reset_command_pool(
                self.command_pool,
                vk::CommandPoolResetFlags::empty(),
            )
            .with_context(|| "unable to reset the command pool!")
    }

    /// Destroy the command pool.
    ///
    /// # Unsafe Because
    ///
    /// - the caller must ensure that the command pool is not in use when it is
    ///   destroyed.
    pub unsafe fn destroy(&mut self, logical_device: &ash::Device) {
        logical_device.destroy_command_pool(self.command_pool, None);
    }
}
