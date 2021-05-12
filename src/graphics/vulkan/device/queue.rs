use super::Device;

use anyhow::Result;
use ash::vk;

/// A wrapper of the raw vulkan queue handle which prevents different threads
/// from accessing the queue concurrently using a mutex.
#[derive(Debug, Clone, Copy)]
pub struct Queue {
    queue: vk::Queue,
    pub family_id: u32,
    pub index: u32,
}

impl Queue {
    /// Build a queue wrapper from the raw queue handle.
    pub fn from_raw(queue: vk::Queue, family_id: u32, index: u32) -> Self {
        Self {
            queue,
            family_id,
            index,
        }
    }

    /// The raw queue handle.
    pub fn raw(&self) -> vk::Queue {
        self.queue
    }

    /// Returns true if this instance and another represent the same device
    /// queue
    pub fn is_same(&self, queue: &Queue) -> bool {
        self.family_id == queue.family_id && self.index == queue.index
    }

    /// Assign a name to this queue which wil show up in debug messages.
    pub fn name_vulkan_object<Name>(
        &self,
        name: Name,
        device: &Device,
    ) -> Result<()>
    where
        Name: Into<String>,
    {
        device.name_vulkan_object(name, vk::ObjectType::QUEUE, &self.queue)?;
        Ok(())
    }
}
