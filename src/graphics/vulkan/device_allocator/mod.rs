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
mod passthrough;
mod shared_ref;
mod suballocator;

#[cfg(test)]
mod stub_allocator;

use anyhow::Result;
use ash::vk;

pub use self::{
    forced_offset::ForcedOffsetAllocator,
    mem_unit::MemUnit,
    metrics::{ConsoleMarkdownReport, MetricsAllocator},
    passthrough::PassthroughAllocator,
    shared_ref::SharedRefAllocator,
    suballocator::Suballocator,
};

/// A single allocated piece of device memory.
#[derive(Clone)]
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

    let mut system_allocator = MetricsAllocator::new(
        "Application Allocator Interface",
        ConsoleMarkdownReport::new(ash_instance.clone(), physical_device),
        ForcedOffsetAllocator::new(device_allocator, MemUnit::MiB(1)),
    );

    let mut sub = Suballocator::new(unsafe {
        system_allocator
            .allocate(vk::MemoryAllocateInfo {
                memory_type_index: 7,
                allocation_size: 1024,
                ..Default::default()
            })
            .expect("ahhhhh!!!!")
    });
    unsafe {
        sub.free_all(&mut system_allocator)
            .expect("free the suballocator!");
    }

    Box::new(system_allocator)
}
