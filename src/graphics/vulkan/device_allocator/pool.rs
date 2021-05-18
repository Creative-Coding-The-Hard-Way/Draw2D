use super::{Allocation, DeviceAllocator, MemUnit, Suballocator};

use anyhow::Result;
use ash::vk;
use std::collections::HashMap;

pub struct PoolAllocator<Allocator: DeviceAllocator> {
    parent: Allocator,
    block_size: u64,
    blocks: HashMap<vk::DeviceMemory, Suballocator>,
}

impl<Allocator: DeviceAllocator> PoolAllocator<Allocator> {
    /// Create a new pool allocator which suballocates memory from large
    /// blocks.
    pub fn new(allocator: Allocator, block_size: MemUnit) -> Self {
        Self {
            parent: allocator,
            block_size: block_size.to_bytes(),
            blocks: HashMap::new(),
        }
    }
}

impl<Allocator: DeviceAllocator> DeviceAllocator for PoolAllocator<Allocator> {
    unsafe fn allocate(
        &mut self,
        memory_allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        if memory_allocate_info.allocation_size > self.block_size {
            anyhow::bail!("This pool is unable to allocate a block that large!")
        }

        for (_, suballocator) in &mut self.blocks {
            if let Ok(allocation) = suballocator.allocate(memory_allocate_info)
            {
                return Ok(allocation);
            }
        }

        let new_block_allocation =
            self.parent.allocate(vk::MemoryAllocateInfo {
                memory_type_index: memory_allocate_info.memory_type_index,
                allocation_size: self.block_size,
                ..Default::default()
            })?;
        let mut suballocator = Suballocator::new(new_block_allocation.clone());

        let allocation = suballocator.allocate(memory_allocate_info)?;
        self.blocks
            .insert(new_block_allocation.memory, suballocator);

        Ok(allocation)
    }

    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        if allocation.is_null() {
            Ok(())
        } else if self.blocks.contains_key(&allocation.memory) {
            let suballocator = self.blocks.get_mut(&allocation.memory).unwrap();
            suballocator.free(allocation)?;
            if suballocator.is_empty() {
                suballocator.free_block(&mut self.parent)?;
                self.blocks.remove(&allocation.memory);
            }
            Ok(())
        } else {
            anyhow::bail!(format!(
                "this pool did not allocate that memory! {:#?}\n {:#?}",
                allocation, self.blocks
            ))
        }
    }
}
