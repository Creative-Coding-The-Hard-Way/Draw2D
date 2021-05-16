use super::{Allocation, DeviceAllocator};

use anyhow::Result;
use ash::vk;

pub struct StubAllocator {}

impl DeviceAllocator for StubAllocator {
    unsafe fn free(&mut self, _allocation: &Allocation) -> Result<()> {
        todo!()
    }

    unsafe fn allocate(
        &mut self,
        _allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        todo!()
    }
}
