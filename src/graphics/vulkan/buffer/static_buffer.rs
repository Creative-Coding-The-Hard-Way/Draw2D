use super::Buffer;
use crate::graphics::vulkan::{device_allocator::Allocation, Device};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// A static chunk of real GPU memory. Each instance is backed by a GPU
/// allocation.
pub struct StaticBuffer {
    raw: vk::Buffer,
    allocation: Allocation,

    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,

    /// the device used to create this buffer
    pub(super) device: Arc<Device>,
}

impl StaticBuffer {
    /// Create a new buffer with no memory allocated.
    pub fn empty(
        device: Arc<Device>,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        Ok(Self {
            raw: vk::Buffer::null(),
            allocation: Allocation::null(),
            usage,
            properties,
            device,
        })
    }

    /// Allocates another buffer with the same properties as the current buffer.
    pub fn allocate(&self, size: u64) -> Result<Self> {
        Self::create(self.device.clone(), self.usage, self.properties, size)
    }

    /// Create a new buffer with the specified size.
    pub fn create(
        device: Arc<Device>,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
        size: u64,
    ) -> Result<Self> {
        let create_info = vk::BufferCreateInfo {
            size,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let raw =
            unsafe { device.logical_device.create_buffer(&create_info, None)? };

        let buffer_memory_requirements = unsafe {
            device.logical_device.get_buffer_memory_requirements(raw)
        };

        let allocation = unsafe {
            device.allocate_memory(buffer_memory_requirements, properties)?
        };

        unsafe {
            device.logical_device.bind_buffer_memory(
                raw,
                allocation.memory,
                allocation.offset,
            )?;
        }

        Ok(Self {
            raw,
            allocation,
            usage,
            properties,
            device,
        })
    }
}

impl Buffer for StaticBuffer {
    /// The raw buffer handle. Valid for the lifetime of this buffer.
    unsafe fn raw(&self) -> ash::vk::Buffer {
        self.raw
    }

    /// The device memory handle. Valid for the lifetime of this buffer.
    unsafe fn allocation(&self) -> &Allocation {
        &self.allocation
    }

    /// The size, in bytes, of the allocated device memory.
    fn size_in_bytes(&self) -> u64 {
        self.allocation.byte_size
    }
}

impl Drop for StaticBuffer {
    /// Free the buffer and any memory which is allocated.
    ///
    /// It is the responsibility of the application to synchronize this drop
    /// with any ongoing GPU actions.
    fn drop(&mut self) {
        unsafe {
            if self.raw != vk::Buffer::null() {
                self.device.logical_device.destroy_buffer(self.raw, None);
                self.raw = vk::Buffer::null();
            }
            self.device.free_memory(&self.allocation).unwrap();
        }
    }
}
