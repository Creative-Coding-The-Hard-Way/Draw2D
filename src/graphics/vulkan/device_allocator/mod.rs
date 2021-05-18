//! This module defines traits and implementations for managing device (gpu)
//! memory.
//!
//! Big Idea: Allocators should compose.
//!
//! Allocator Implementations (brainstorm):
//! - passthrough allocator -> directly allocates a new block of device memory
//!   - done!
//! - metric-gathering allocator -> decorates an allocator with metrics
//!   - done!
//! - pooling allocator -> something something, gpu memory pools

mod allocation;
mod forced_offset;
mod mem_unit;
mod metrics;
mod page_allocator;
mod passthrough;
mod pool;
mod shared_ref;
mod size_selector;
mod suballocator;
mod type_index;

#[cfg(test)]
mod stub_allocator;

use anyhow::Result;
use ash::vk;

pub use self::{
    forced_offset::ForcedOffsetAllocator,
    mem_unit::MemUnit,
    metrics::{ConsoleMarkdownReport, MetricsAllocator},
    page_allocator::PageAllocator,
    passthrough::PassthroughAllocator,
    pool::PoolAllocator,
    shared_ref::SharedRefAllocator,
    size_selector::SizeSelector,
    suballocator::Suballocator,
    type_index::TypeIndexAllocator,
};

/// A single allocated piece of device memory.
#[derive(Clone, Debug)]
pub struct Allocation {
    pub memory: vk::DeviceMemory,
    pub offset: vk::DeviceSize,
    pub byte_size: vk::DeviceSize,
    memory_type_index: u32,
}

/// The external device memory allocation interface. This is the api used by
/// applications to allocate and free memory on the gpu.
pub trait DeviceAllocator {
    /// Allocate device memory with the provided type index and size.
    ///
    /// # unsafe because
    ///
    /// - it is the responsibility of the caller to free the returned memory
    ///   when it is no longer in use
    /// - implementations do not generally check that the memory type index in
    ///   allocate_info is the correct memory type index, the arguments are
    ///   assumed to be correct
    unsafe fn allocate(
        &mut self,
        allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation>;

    /// Free an allocated piece of device memory.
    ///
    /// # unsafe because
    ///
    /// - it is the responsibility of the caller to know when the GPU is no
    ///   longer using the allocation
    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()>;
}

/// Build the standard allocator implementation.
///
/// The return is a boxed impl so that consumers are not dependent on the
/// specific implementation (often the full type is unwieldy because it is a
/// composition of DeviceAllocator implementations).
///
/// The caller is responsible for keeping the ash instance, logical device, and
/// physical device alive for at least as long as the allocator exists.
pub fn build_standard_allocator(
    ash_instance: ash::Instance,
    logical_device: ash::Device,
    physical_device: ash::vk::PhysicalDevice,
) -> Box<impl DeviceAllocator> {
    let device_allocator = SharedRefAllocator::new(MetricsAllocator::new(
        "Device Allocator",
        ConsoleMarkdownReport::new(ash_instance.clone(), physical_device),
        PassthroughAllocator::create(logical_device),
    ));

    let typed_allocator = PageAllocator::new(
        TypeIndexAllocator::new(
            &ash_instance,
            physical_device,
            |_memory_type_index, _memory_type| {
                SizeSelector::new(
                    // For allocations below 512KiB
                    PoolAllocator::new(
                        device_allocator.clone(),
                        MemUnit::MiB(1),
                    ),
                    MemUnit::KiB(512),
                    // for allocations above 512KiB
                    SizeSelector::new(
                        // for allocations below 256MiB
                        PoolAllocator::new(
                            device_allocator.clone(),
                            MemUnit::MiB(512),
                        ),
                        MemUnit::MiB(256),
                        // for allocations above 256MiB
                        device_allocator.clone(),
                    ),
                )
            },
        ),
        MemUnit::KiB(1),
    );

    Box::new(typed_allocator)
}
