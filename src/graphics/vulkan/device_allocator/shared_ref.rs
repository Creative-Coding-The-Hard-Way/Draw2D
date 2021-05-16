use super::{Allocation, DeviceAllocator};

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
        allocate_info: vk::MemoryAllocateInfo,
    ) -> Result<Allocation> {
        self.allocator.borrow_mut().allocate(allocate_info)
    }

    unsafe fn free(
        &mut self,
        allocation: &super::Allocation,
    ) -> anyhow::Result<()> {
        self.allocator.borrow_mut().free(allocation)
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
