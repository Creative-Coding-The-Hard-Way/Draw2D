use super::{Allocation, DeviceAllocator, MemUnit};

use anyhow::Result;
use ash::vk;

/// Decorate an allocator such that all allocation requests are rounded up to
/// the nearest page.
pub struct PageAllocator<Allocator: DeviceAllocator> {
    parent: Allocator,
    page_size: u64,
}

impl<Allocator: DeviceAllocator> PageAllocator<Allocator> {
    /// Decorate an allocator such that all of the underlying allocations occur
    /// on page boundaries.
    pub fn new(parent: Allocator, page_size: MemUnit) -> Self {
        Self {
            parent,
            page_size: page_size.to_bytes(),
        }
    }

    fn round_up_to_nearest_page(&self, size: u64) -> u64 {
        let pages = (size as f64 / self.page_size as f64).ceil() as u64;
        pages * self.page_size
    }
}

impl<Allocator: DeviceAllocator> DeviceAllocator for PageAllocator<Allocator> {
    unsafe fn allocate(
        &mut self,
        memory_allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        let aligned_memory_allocate_info = vk::MemoryAllocateInfo {
            memory_type_index: memory_allocate_info.memory_type_index,
            allocation_size: self
                .round_up_to_nearest_page(memory_allocate_info.allocation_size),
            ..Default::default()
        };

        // used the aligned size when actually allocating
        let mut allocation =
            self.parent.allocate(aligned_memory_allocate_info)?;

        // but tell the caller the size they expect
        allocation.byte_size = memory_allocate_info.allocation_size;

        Ok(allocation)
    }

    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        let mut aligned_allocation = allocation.clone();
        aligned_allocation.byte_size =
            self.round_up_to_nearest_page(allocation.byte_size);
        self.parent.free(&aligned_allocation)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::graphics::vulkan::device_allocator::stub_allocator::StubAllocator;

    #[test]
    pub fn test_round_to_nearest_page() {
        let allocator = PageAllocator::new(StubAllocator {}, MemUnit::B(256));

        assert_eq!(allocator.round_up_to_nearest_page(0), 0);
        assert_eq!(allocator.round_up_to_nearest_page(256), 256);
        assert_eq!(allocator.round_up_to_nearest_page(20), 256);
        assert_eq!(allocator.round_up_to_nearest_page(257), 512);
        assert_eq!(
            allocator
                .round_up_to_nearest_page(MemUnit::GiB(1024).to_bytes() + 1),
            MemUnit::GiB(1024).to_bytes() + 256
        );
    }
}
