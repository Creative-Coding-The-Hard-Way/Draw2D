use super::Allocation;

use ash::vk;

impl Allocation {
    /// Create a null allocation which has a size of zero and a null memory
    /// handle.
    pub fn null() -> Self {
        Self {
            offset: 0,
            byte_size: 0,
            memory: vk::DeviceMemory::null(),
            memory_type_index: 0,
        }
    }

    /// Returns true when the memory pointer is null.
    pub fn is_null(&self) -> bool {
        self.memory == vk::DeviceMemory::null()
    }
}
