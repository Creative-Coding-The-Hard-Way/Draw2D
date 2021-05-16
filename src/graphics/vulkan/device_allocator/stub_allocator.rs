use super::{Allocation, DeviceAllocator};

use anyhow::Result;
use ash::vk;

pub struct StubAllocator {}

impl DeviceAllocator for StubAllocator {
    unsafe fn allocate(
        &mut self,
        _: vk::MemoryRequirements,
        _: vk::MemoryPropertyFlags,
    ) -> Result<Allocation> {
        todo!()
    }

    unsafe fn free(&mut self, _allocation: &Allocation) -> Result<()> {
        todo!()
    }
}
