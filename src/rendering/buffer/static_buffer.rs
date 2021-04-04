use super::Buffer;
use crate::rendering::Device;

use anyhow::{Context, Result};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use std::sync::Arc;

/// A static chunk of real GPU memory. Each instance is backed by a GPU
/// allocation.
pub struct StaticBuffer {
    raw: vk::Buffer,
    memory: vk::DeviceMemory,
    size: u64,

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
            memory: vk::DeviceMemory::null(),
            size: 0,
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
        let create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let raw =
            unsafe { device.logical_device.create_buffer(&create_info, None)? };

        let buffer_memory_requirements = unsafe {
            device.logical_device.get_buffer_memory_requirements(raw)
        };

        let memory_properties = unsafe {
            device
                .instance
                .ash
                .get_physical_device_memory_properties(device.physical_device)
        };

        let memory_type_index = memory_properties
            .memory_types
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                let type_supported =
                    buffer_memory_requirements.memory_type_bits & (1 << i) != 0;
                let properties_supported =
                    memory_type.property_flags.contains(properties);
                type_supported & properties_supported
            })
            .map(|(i, _memory_type)| i as u32)
            .with_context(|| {
                "unable to find a suitable memory type for this gpu buffer!"
            })?;

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(buffer_memory_requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe {
            let memory = device
                .logical_device
                .allocate_memory(&allocate_info, None)?;
            device.logical_device.bind_buffer_memory(raw, memory, 0)?;
            memory
        };

        Ok(Self {
            raw,
            memory,
            size: buffer_memory_requirements.size,
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
    unsafe fn memory(&self) -> vk::DeviceMemory {
        self.memory
    }

    /// The size, in bytes, of the allocated device memory.
    fn size_in_bytes(&self) -> u64 {
        self.size
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

            if self.memory != vk::DeviceMemory::null() {
                self.device.logical_device.free_memory(self.memory, None);
                self.memory = vk::DeviceMemory::null();
            }
        }
    }
}
