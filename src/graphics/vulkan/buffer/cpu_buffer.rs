use super::{Buffer, StaticBuffer};
use crate::graphics::vulkan::Device;

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// A CPU-accessible buffer.
///
/// Data is allocated directly, so every instance of this buffer contributes
/// to the driver-specified limit on the number of allocations supported by
/// the device.
pub struct CpuBuffer {
    buffer: StaticBuffer,
    written_size: u64,
}

impl CpuBuffer {
    /// Create an empty buffer which can be written from the CPU.
    pub fn new(
        device: Arc<Device>,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self> {
        Ok(Self {
            buffer: StaticBuffer::empty(
                device.clone(),
                usage,
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?,
            written_size: 0,
        })
    }

    /// Write the provided data into the vertex buffer.
    ///
    /// Unsafe because this method can replace both the buffer and the backing
    /// memory. It is the responsibility of the application to ensure that
    /// neither resource is being used when this method is called.
    ///
    /// No explicit flush is required because the memory is allocated as
    /// HOST_COHERENT.
    pub unsafe fn write_data<T>(&mut self, data: &[T]) -> Result<()>
    where
        T: Sized + Copy + std::fmt::Debug,
    {
        self.write_data_arrays(&[data])?;

        Ok(())
    }

    pub unsafe fn write_data_arrays<T>(
        &mut self,
        data_arrays: &[&[T]],
    ) -> Result<()>
    where
        T: Sized + Copy + std::fmt::Debug,
    {
        let entry_size = std::mem::size_of::<T>();
        let total_count: usize =
            data_arrays.iter().map(|entry| entry.len()).sum();
        let total_size = total_count * entry_size;

        self.resize(total_size as u64)?;

        let mut ptr = self.buffer.device.logical_device.map_memory(
            self.buffer.memory(),
            0,
            self.written_size,
            vk::MemoryMapFlags::empty(),
        )? as *mut T;

        for entry in data_arrays {
            let mapped_slice = std::slice::from_raw_parts_mut(ptr, entry.len());
            mapped_slice.copy_from_slice(entry);
            ptr = ptr.offset(entry.len() as isize);
        }

        self.buffer
            .device
            .logical_device
            .unmap_memory(self.buffer.memory());

        Ok(())
    }

    /// Update the written-size of the buffer.
    ///
    /// Reallocate the underlying GPU memory when needed.
    fn resize(&mut self, byte_size: u64) -> Result<()> {
        if byte_size > self.buffer.size_in_bytes() {
            self.buffer = self.buffer.allocate(byte_size)?;
        }
        self.written_size = byte_size;
        Ok(())
    }
}

impl Buffer for CpuBuffer {
    /// The raw buffer handle.
    ///
    /// Can be invalidated on calls to `write_data`.
    unsafe fn raw(&self) -> ash::vk::Buffer {
        self.buffer.raw()
    }

    /// The raw device memory handle.
    ///
    /// Can be invalidate on calls to `write_data`.
    unsafe fn memory(&self) -> vk::DeviceMemory {
        self.buffer.memory()
    }

    /// The size of the data written on the last call to `write_data`.
    fn size_in_bytes(&self) -> u64 {
        self.written_size
    }
}
