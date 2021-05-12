//! This module defines traits and implementations for managing device (gpu)
//! memory.
//!
//! Big Idea: Allocators should compose.
//!
//! Allocator Implementations (brainstorm):
//! - null allocator -> always returns an error
//! - passthrough allocator -> directly allocates a new block of device memory
//! - metric-gathering allocator -> decorates an allocator with metrics
//! - pooling allocator -> something something, gpu memory pools
//! - freelist? does this make sense for device memory?

use anyhow::Result;
use ash::vk;

/// A single allocated piece of device memory.
pub struct Allocation {
    memory: vk::DeviceMemory,
    offset: vk::DeviceSize,
    byte_size: vk::DeviceSize,
}

pub trait DeviceAllocator {
    /// Allocate a piece of device memory given the requirements and usage.
    fn allocate(
        &mut self,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation>;

    /// Free an allocated piece of device memory.
    fn free(&mut self, allocation: Allocation) -> Result<()>;

    /// True when this specific allocator implementation knows how to manage
    /// allocation.
    fn managed_by_me(&self, allocation: &Allocation) -> bool;
}
