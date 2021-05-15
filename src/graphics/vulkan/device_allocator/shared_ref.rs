use super::{Allocation, DeviceAllocator, MemoryTypeAllocator};

use anyhow::Result;
use ash::vk;
use std::{cell::RefCell, rc::Rc};

/// A device allocator implementation which represents a shared reference to an
/// underlying allocator implementation.
///
/// This is useful because multiple other allocators often compose over the
/// passthrough. When this occurs, they often need to use the *same* instance
/// of the passthrough allocator.
pub struct SharedRefAllocator<Alloc: DeviceAllocator> {
    allocator: Rc<RefCell<Alloc>>,
}

impl<Alloc: DeviceAllocator> SharedRefAllocator<Alloc> {
    pub fn new(allocator: Alloc) -> Self {
        Self {
            allocator: Rc::new(RefCell::new(allocator)),
        }
    }
}

impl<Alloc: DeviceAllocator> Clone for SharedRefAllocator<Alloc> {
    fn clone(&self) -> Self {
        Self {
            allocator: self.allocator.clone(),
        }
    }
}

impl<Alloc: DeviceAllocator> DeviceAllocator for SharedRefAllocator<Alloc> {
    unsafe fn allocate(
        &mut self,
        memory_requirements: vk::MemoryRequirements,
        memory_property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation> {
        self.allocator
            .borrow_mut()
            .allocate(memory_requirements, memory_property_flags)
    }

    unsafe fn free(
        &mut self,
        allocation: &super::Allocation,
    ) -> anyhow::Result<()> {
        self.allocator.borrow_mut().free(allocation)
    }

    fn managed_by_me(&self, allocation: &super::Allocation) -> bool {
        self.allocator.borrow().managed_by_me(allocation)
    }
}

impl<Alloc: MemoryTypeAllocator + DeviceAllocator> MemoryTypeAllocator
    for SharedRefAllocator<Alloc>
{
    unsafe fn allocate_by_info(
        &mut self,
        memory_requirements: vk::MemoryRequirements,
        memory_property_flags: vk::MemoryPropertyFlags,
        allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        self.allocator.borrow_mut().allocate_by_info(
            memory_requirements,
            memory_property_flags,
            allocate_info,
        )
    }
}

#[cfg(test)]
mod test {
    use crate::graphics::vulkan::device_allocator::{
        stub_allocator::StubAllocator, SharedRefAllocator,
    };

    #[test]
    pub fn can_clone() {
        let allocator = SharedRefAllocator::new(StubAllocator {});
        let _alloc2 = allocator.clone();
    }

    #[should_panic]
    #[test]
    pub fn the_stub_allocator_cannot_clone() {
        let stub = StubAllocator {};
        let _cloned = stub.clone();
    }

    impl Clone for StubAllocator {
        fn clone(&self) -> Self {
            panic!("The Stub Allocator does not support cloning")
        }
    }
}
