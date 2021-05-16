mod region;
mod suballocator;

use super::{Allocation, DeviceAllocator};

use self::region::{MergeResult, Region};

use anyhow::Result;
use ash::vk;

/// A suballocator can divvy up a single allocation into multiple
/// non-overlapping allocations.
pub struct Suballocator {
    block: Allocation,
    free_regions: Vec<Region>,
}

impl DeviceAllocator for Suballocator {
    unsafe fn allocate(
        &mut self,
        memory_allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        use anyhow::Context;

        if self.block.memory_type_index
            != memory_allocate_info.memory_type_index
        {
            anyhow::bail!("Attempted to allocate incompatible memory!");
        }
        let region = self
            .allocate_region(memory_allocate_info.allocation_size)
            .with_context(|| {
                "not enough memory for an allocation of the requested size"
            })?;

        Ok(Allocation {
            memory_type_index: self.block.memory_type_index,
            memory: self.block.memory,
            offset: region.offset + self.block.offset,
            byte_size: region.size,
        })
    }

    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        self.free_region(Region::new(
            allocation.offset - self.block.offset,
            allocation.byte_size,
        ));
        Ok(())
    }
}
