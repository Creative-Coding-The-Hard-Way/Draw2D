use super::{Allocation, DeviceAllocator};

use anyhow::Result;
use ash::vk;

/// An allocator which forces all allocations to have a fixed offset.
///
/// This has little practical use, but is convenient when verifying that other
/// parts of the code properly handle allocation offsets.
pub struct ForcedOffsetAllocator<Alloc: DeviceAllocator> {
    allocator: Alloc,
}

impl<Alloc: DeviceAllocator> ForcedOffsetAllocator<Alloc> {
    pub fn new(allocator: Alloc) -> Self {
        Self { allocator }
    }
}

impl<Alloc: DeviceAllocator> DeviceAllocator for ForcedOffsetAllocator<Alloc> {
    /// Use the underlying allocator implementation to allocate an oversized
    /// piece of memory, then set an offset to compensate.
    ///
    /// The offset will always be `memory_requirements.alignment * 100`.
    ///
    /// This has no practical use other than proving that code properly handles
    /// memory offsets.
    unsafe fn allocate(
        &mut self,
        memory_requirements: vk::MemoryRequirements,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation> {
        let alignment = memory_requirements.alignment;
        let fixed_offset = alignment * 100;
        let mut expanded_memory_requirements = memory_requirements.clone();
        expanded_memory_requirements.size += fixed_offset;

        let mut allocation = self
            .allocator
            .allocate(expanded_memory_requirements, memory_property_flags)?;

        allocation.offset += fixed_offset;
        allocation.byte_size -= fixed_offset;

        Ok(allocation)
    }

    /// Undo the offset+size adjustments which were applied by [Self::allocate],
    /// then use the underlying allocator to actually free the memory.
    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        let mut adjusted = allocation.clone();
        adjusted.byte_size += allocation.offset;
        adjusted.offset = 0;
        self.allocator.free(&adjusted)
    }

    fn managed_by_me(&self, allocation: &super::Allocation) -> bool {
        self.allocator.managed_by_me(allocation)
    }
}
