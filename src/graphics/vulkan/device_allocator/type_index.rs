use super::{Allocation, DeviceAllocator};

use anyhow::Result;
use ash::vk;

/// The type index allocator creates a separate memory allocator for each memory
/// type index, then dispatches allocations and frees.
pub struct TypeIndexAllocator<Allocator: DeviceAllocator> {
    allocators: Vec<Allocator>,
}

impl<Allocator: DeviceAllocator> TypeIndexAllocator<Allocator> {
    pub fn new<Factory: FnMut(u32, vk::MemoryType) -> Allocator>(
        ash_instance: &ash::Instance,
        physical_device: ash::vk::PhysicalDevice,
        mut factory: Factory,
    ) -> Self {
        use ash::version::InstanceV1_0;

        let memory_properties = unsafe {
            ash_instance.get_physical_device_memory_properties(physical_device)
        };

        let mut allocators = Vec::new();
        allocators.reserve(memory_properties.memory_type_count as usize);

        for i in 0..memory_properties.memory_type_count {
            allocators
                .push(factory(i, memory_properties.memory_types[i as usize]));
        }

        Self { allocators }
    }
}

impl<Allocator: DeviceAllocator> DeviceAllocator
    for TypeIndexAllocator<Allocator>
{
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        self.allocators[allocate_info.memory_type_index as usize]
            .allocate(allocate_info)
    }

    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()> {
        if allocation.is_null() {
            Ok(())
        } else {
            self.allocators[allocation.memory_type_index as usize]
                .free(allocation)
        }
    }
}
