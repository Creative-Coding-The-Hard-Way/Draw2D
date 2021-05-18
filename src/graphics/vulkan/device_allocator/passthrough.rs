use super::{Allocation, DeviceAllocator};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// An allocator implementation which just directly allocates a new piece of
/// device memory on each call.
///
/// This allocator is stateles and can be cheaply cloned when building
/// allocators which decorate this behavior.
#[derive(Clone)]
pub struct PassthroughAllocator {
    /// A non-owning copy of the vulkan logical device.
    logical_device: ash::Device,
}

impl PassthroughAllocator {
    /// Create a new stateless passthrough allocator which just directly
    /// allocates and deallocates memory using the vulkan logical device.
    pub fn create(logical_device: ash::Device) -> Self {
        Self { logical_device }
    }
}

impl DeviceAllocator for PassthroughAllocator {
    /// Directly allocate device memory onto the heap indicated by the
    /// memory type index of the `allocate_info` struct.
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        Ok(Allocation {
            memory: self
                .logical_device
                .allocate_memory(&allocate_info, None)?,
            offset: 0,
            byte_size: allocate_info.allocation_size,
            memory_type_index: allocate_info.memory_type_index,
        })
    }

    /// Free the allocation's underlying memory.
    ///
    /// # unsafe because
    ///
    /// - it is the responsibility of the caller to ensure the allocation's
    ///   device memory is no longer in use
    /// - this *includes* other allocations which reference the same piece of
    ///   memory! Don't double-free!
    ///
    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        self.logical_device.free_memory(allocation.memory, None);
        Ok(())
    }
}
