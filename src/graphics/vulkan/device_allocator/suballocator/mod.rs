mod region;

use super::{Allocation, DeviceAllocator};

use self::region::{MergeResult, Region};

use anyhow::Result;
use ash::vk;

/// A pool suballocator can divvy up a single large allocation - the 'block' -
/// into multiple suballocations which take up a subset of the block.
pub struct Suballocator {
    block: Allocation,
    free_regions: Vec<Region>,
}

impl Suballocator {
    pub fn new(allocation: Allocation) -> Self {
        Self {
            free_regions: vec![Region::new(
                allocation.offset,
                allocation.byte_size,
            )],
            block: allocation,
        }
    }

    pub unsafe fn free_all(
        &mut self,
        allocator: &mut impl DeviceAllocator,
    ) -> Result<()> {
        allocator.free(&self.block)?;
        self.block = Allocation::null();
        Ok(())
    }

    fn find_free_region(&mut self, size: u64) -> Option<Region> {
        for i in 0..self.free_regions.len() {
            if size == self.free_regions[i].size {
                return Some(self.free_regions.remove(i));
            } else if size < self.free_regions[i].size {
                return Some(self.free_regions[i].take_subregion(size));
            }
        }
        None
    }
}

//
// impl DeviceAllocator for PoolSuballocator {
//     unsafe fn allocate(
//         &mut self,
//         memory_allocate_info: vk::MemoryAllocateInfo,
//     ) -> Result<Allocation> {
//         use anyhow::Context;
//
//         if self.block.memory_type_index
//             != memory_allocate_info.memory_type_index
//         {
//             anyhow::bail!(
//                 "memory type is not supported for this suballocator!"
//             );
//         }
//
//         let region = self
//             .find_free_region(memory_allocate_info.allocation_size)
//             .with_context(|| "unable to find a free region of memory")?;
//
//         Ok(Allocation {
//             memory: self.block.memory,
//             memory_type_index: self.block.memory_type_index,
//             offset: region.offset,
//             byte_size: region.size,
//         })
//     }
//
//     unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
//         todo!()
//     }
// }
