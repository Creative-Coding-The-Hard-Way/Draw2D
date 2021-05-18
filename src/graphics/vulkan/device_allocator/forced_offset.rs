use super::{Allocation, DeviceAllocator, MemUnit};

use anyhow::Result;
use ash::vk;

/// An allocator which forces all allocations to have a fixed offset.
///
/// This has little practical use, but is convenient when verifying that other
/// parts of the code properly handle allocation offsets.
pub struct ForcedOffsetAllocator<Alloc: DeviceAllocator> {
    alignment: u64,
    allocator: Alloc,
}

impl<Alloc: DeviceAllocator> ForcedOffsetAllocator<Alloc> {
    pub fn new(allocator: Alloc, alignment: MemUnit) -> Self {
        Self {
            allocator,
            alignment: alignment.to_bytes(),
        }
    }

    fn offset(&self) -> u64 {
        self.alignment * 100
    }
}

impl<Alloc: DeviceAllocator> DeviceAllocator for ForcedOffsetAllocator<Alloc> {
    /// Use the underlying allocator implementation to allocate an oversized
    /// piece of memory, then set an offset to compensate.
    ///
    /// This has no practical use other than proving that code properly handles
    /// memory offsets.
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        let expanded_allocate_info = vk::MemoryAllocateInfo {
            memory_type_index: allocate_info.memory_type_index,
            allocation_size: allocate_info.allocation_size + self.offset(),
            ..Default::default()
        };
        let mut allocation = self.allocator.allocate(expanded_allocate_info)?;
        allocation.offset += self.offset();
        allocation.byte_size = allocate_info.allocation_size;
        Ok(allocation)
    }

    /// Undo the offset+size adjustments which were applied by [Self::allocate],
    /// then use the underlying allocator to actually free the memory.
    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        if allocation.is_null() {
            Ok(())
        } else {
            let mut adjusted = allocation.clone();
            adjusted.offset -= self.offset();
            adjusted.byte_size += self.offset();
            self.allocator.free(&adjusted)
        }
    }
}
