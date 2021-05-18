use super::{Allocation, DeviceAllocator, MemUnit};

use anyhow::Result;
use ash::vk;

pub struct SizeSelector<
    SmallAllocator: DeviceAllocator,
    LargeAllocator: DeviceAllocator,
> {
    small_allocator: SmallAllocator,
    large_allocator: LargeAllocator,
    size: u64,
}

impl<SmallAllocator: DeviceAllocator, LargeAllocator: DeviceAllocator>
    SizeSelector<SmallAllocator, LargeAllocator>
{
    /// Create a new allocator which defers to the small allocator for
    /// allocations below size bytes, and otherwise uses the large allocator.
    pub fn new(
        small_allocator: SmallAllocator,
        size: MemUnit,
        large_allocator: LargeAllocator,
    ) -> Self {
        Self {
            small_allocator,
            large_allocator,
            size: size.to_bytes(),
        }
    }
}

impl<SmallAllocator: DeviceAllocator, LargeAllocator: DeviceAllocator>
    DeviceAllocator for SizeSelector<SmallAllocator, LargeAllocator>
{
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        if allocate_info.allocation_size < self.size {
            self.small_allocator.allocate(allocate_info)
        } else {
            self.large_allocator.allocate(allocate_info)
        }
    }

    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        if allocation.byte_size < self.size {
            self.small_allocator.free(allocation)
        } else {
            self.large_allocator.free(allocation)
        }
    }
}
