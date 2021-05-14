//! This module defines traits and implementations for managing device (gpu)
//! memory.
//!
//! Big Idea: Allocators should compose.
//!
//! Allocator Implementations (brainstorm):
//! - passthrough allocator -> directly allocates a new block of device memory
//!   - done!
//! - null allocator -> always returns an error
//! - metric-gathering allocator -> decorates an allocator with metrics
//! - pooling allocator -> something something, gpu memory pools
//! - freelist? does this make sense for device memory?

mod allocation;
mod forced_offset;
mod passthrough;

use anyhow::Result;
use ash::vk;

pub use self::{
    forced_offset::ForcedOffsetAllocator, passthrough::PassthroughAllocator,
};

/// A single allocated piece of device memory.
#[derive(Clone)]
pub struct Allocation {
    pub memory: vk::DeviceMemory,
    pub offset: vk::DeviceSize,
    pub byte_size: vk::DeviceSize,
}

pub trait DeviceAllocator {
    /// Allocate a piece of device memory given the requirements and usage.
    ///
    /// UNSAFE: because it is the responsibility of the caller to free the
    /// returned memory when it is no longer in use.
    unsafe fn allocate(
        &mut self,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation>;

    /// Free an allocated piece of device memory.
    ///
    /// # unsafe because
    ///
    /// - it is the responsibility of the caller to know when the GPU is no
    ///   longer using the allocation.
    unsafe fn free(&mut self, allocation: &Allocation) -> Result<()>;

    /// True when this specific allocator implementation knows how to manage
    /// allocation.
    fn managed_by_me(&self, allocation: &Allocation) -> bool;
}
