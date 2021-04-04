use super::Device;

use anyhow::Result;
use ash::vk;
use std::sync::{Arc, Mutex, MutexGuard};

/// A wrapper of the raw vulkan queue handle which prevents different threads
/// from accessing the queue concurrently using a mutex.
#[derive(Debug)]
pub struct Queue {
    queue: Mutex<vk::Queue>,
    pub family_id: u32,
    pub index: u32,
}

impl Queue {
    /// Build a queue wrapper from the raw queue handle.
    pub fn from_raw(queue: vk::Queue, family_id: u32, index: u32) -> Arc<Self> {
        Arc::new(Self {
            queue: Mutex::new(queue),
            family_id,
            index,
        })
    }

    /// Acquire a lock on the queue to do *some* operation.
    ///
    /// Can panic if the queue is acquired multiple times in the same scope.
    pub fn acquire(&self) -> MutexGuard<vk::Queue> {
        self.queue
            .lock()
            .expect("unable to lock the vulkan queue for use in this thread")
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
        let raw_queue = device.graphics_queue.acquire();
        device.name_vulkan_object(name, vk::ObjectType::QUEUE, &*raw_queue)?;
        Ok(())
    }
}
