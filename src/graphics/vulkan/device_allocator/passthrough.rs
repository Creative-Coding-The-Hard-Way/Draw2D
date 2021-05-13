use super::{Allocation, DeviceAllocator};

use anyhow::Result;
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};

/// An allocator implementation which just directly allocates a new piece of
/// device memory on each call.
///
/// This allocator is stateles and can be cheaply cloned when building
/// allocators which decorate this behavior.
#[derive(Clone)]
pub struct PassthroughAllocator {
    /// A non-owning copy of the ash instance. e.g. Whoever creates this struct
    /// must make sure that the ash instance outlives this struct.
    ash_instance: ash::Instance,

    /// A non-owning copy of the vulkan logical device.
    logical_device: ash::Device,

    /// A non-owning copy of the vulkan phypsical device.
    physical_device: ash::vk::PhysicalDevice,
}

impl PassthroughAllocator {
    /// Create a new stateless passthrough allocator which just directly
    /// allocates and deallocates memory using the vulkan logical device.
    pub fn create(
        ash_instance: ash::Instance,
        logical_device: ash::Device,
        physical_device: ash::vk::PhysicalDevice,
    ) -> Self {
        Self {
            ash_instance,
            logical_device,
            physical_device,
        }
    }
}

impl DeviceAllocator for PassthroughAllocator {
    unsafe fn allocate(
        &mut self,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation> {
        use anyhow::Context;

        let memory_properties = self
            .ash_instance
            .get_physical_device_memory_properties(self.physical_device);

        let memory_type_index = memory_properties
            .memory_types
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                let type_supported =
                    memory_requirements.memory_type_bits & (1 << i) != 0;
                let properties_supported =
                    memory_type.property_flags.contains(property_flags);
                type_supported & properties_supported
            })
            .map(|(i, _memory_type)| i as u32)
            .with_context(|| {
                "unable to find a suitable memory type for this allocation!"
            })?;

        let allocate_info = vk::MemoryAllocateInfo {
            memory_type_index,
            allocation_size: memory_requirements.size,
            ..Default::default()
        };

        let memory =
            self.logical_device.allocate_memory(&allocate_info, None)?;

        Ok(Allocation {
            memory,
            offset: 0,
            byte_size: allocate_info.allocation_size,
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

    /// The passthrough allocator assumes that all memory is owned by itself.
    fn managed_by_me(&self, _allocation: &super::Allocation) -> bool {
        true
    }
}
