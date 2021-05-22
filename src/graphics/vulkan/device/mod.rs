//! This module provides functions for picking a physical device and creating
//! the logical device.

mod ext;
mod physical_device;
mod queue;
mod queue_family_indices;

pub use self::{queue::Queue, queue_family_indices::QueueFamilyIndices};

use crate::graphics::vulkan::{
    device_allocator::{self, Allocation},
    Instance, WindowSurface,
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::{
    ffi::CString,
    sync::{Arc, Mutex},
};

use super::device_allocator::DeviceAllocator;

/// This struct holds all device-specific resources, the physical device and
/// logical device for interacting with it, and the associated queues.
pub struct Device {
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,
    pub graphics_queue: Queue,
    pub present_queue: Queue,

    allocator: Mutex<Box<dyn DeviceAllocator>>,

    instance: Arc<Instance>,
}

impl Device {
    /// Create a new device based on this application's required features and
    /// properties.
    pub fn new(window_surface: &dyn WindowSurface) -> Result<Arc<Device>> {
        let instance = window_surface.clone_vulkan_instance();
        let physical_device =
            physical_device::find_optimal(&instance, window_surface)?;
        let queue_family_indices = QueueFamilyIndices::find(
            &physical_device,
            instance.raw(),
            window_surface,
        )?;
        let logical_device = instance.create_logical_device(
            &physical_device,
            physical_device::required_features(),
            &physical_device::required_extensions(),
            &queue_family_indices.as_queue_create_infos(),
        )?;

        let (graphics_queue, present_queue) =
            queue_family_indices.get_queues(&logical_device)?;

        let allocator = device_allocator::build_standard_allocator(
            instance.ash.clone(),
            logical_device.clone(),
            physical_device,
        );

        let device = Arc::new(Self {
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
            allocator: Mutex::new(allocator),
            instance,
        });

        device.name_vulkan_object(
            "Application Logical Device",
            vk::ObjectType::DEVICE,
            &device.logical_device.handle(),
        )?;

        if device.graphics_queue.is_same(&device.present_queue) {
            device
                .graphics_queue
                .name_vulkan_object("graphics/present queue", &device)?;
        } else {
            device
                .graphics_queue
                .name_vulkan_object("graphics queue", &device)?;
            device
                .present_queue
                .name_vulkan_object("present queue", &device)?;
        }

        Ok(device)
    }

    /// Allocate a a chunk of memory for use in a buffer or texture.
    ///
    /// # unsafe because
    ///
    /// - the caller is responsible for eventually calling 'free memory' before
    ///   the application quits
    ///
    pub unsafe fn allocate_memory(
        &self,
        memory_requirements: vk::MemoryRequirements,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Result<Allocation> {
        use anyhow::Context;
        use ash::version::InstanceV1_0;

        let memory_properties = self
            .instance
            .ash
            .get_physical_device_memory_properties(self.physical_device);

        let memory_type_index = memory_properties
            .memory_types
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                let type_supported =
                    memory_requirements.memory_type_bits & (1 << i) != 0;
                let properties_supported =
                    memory_type.property_flags.contains(property_flags);
                type_supported & properties_supported
            })
            .map(|(i, _memory_type)| i as u32)
            .with_context(|| {
                "unable to find a suitable memory type for this allocation!"
            })?;

        self.allocator
            .lock()
            .unwrap()
            .allocate(vk::MemoryAllocateInfo {
                memory_type_index,
                allocation_size: memory_requirements.size,
                ..Default::default()
            })
    }

    /// Free a memory allocation.
    ///
    /// # unsafe because
    ///
    /// - the caller is responsible for ensuring that the memory is no longer
    ///   in use by the gpu.
    ///
    pub unsafe fn free_memory(&self, allocation: &Allocation) -> Result<()> {
        self.allocator.lock().unwrap().free(allocation)
    }

    /// Give a debug name for a vulkan object owned by this device.
    ///
    /// Whatever name is provided here will show up in the debug logs if there
    /// are any issues detected by the validation layers.
    pub fn name_vulkan_object<Name, Handle>(
        &self,
        name: Name,
        object_type: vk::ObjectType,
        handle: &Handle,
    ) -> Result<()>
    where
        Handle: vk::Handle + Copy,
        Name: Into<String>,
    {
        let cname = CString::new(name.into()).unwrap();
        let name_info = vk::DebugUtilsObjectNameInfoEXT {
            object_type,
            p_object_name: cname.as_ptr(),
            object_handle: handle.as_raw(),
            ..Default::default()
        };

        unsafe {
            self.instance.debug.debug_utils_set_object_name(
                self.logical_device.handle(),
                &name_info,
            )?;
        }

        Ok(())
    }

    /// Submit a command buffer to the specified queue, then wait for it to
    /// idle.
    pub unsafe fn submit_and_wait_idle(
        &self,
        queue: &Queue,
        command_buffer: vk::CommandBuffer,
    ) -> Result<()> {
        let command_buffers = &[command_buffer];
        self.logical_device.queue_submit(
            queue.raw(),
            &[vk::SubmitInfo {
                p_command_buffers: command_buffers.as_ptr(),
                command_buffer_count: 1,
                ..Default::default()
            }],
            vk::Fence::null(),
        )?;
        self.logical_device.queue_wait_idle(queue.raw())?;
        Ok(())
    }

    /// Create a new swapchain loader which will be owned by the caller.
    pub fn create_swapchain_loader(&self) -> ash::extensions::khr::Swapchain {
        ash::extensions::khr::Swapchain::new(
            &self.instance.ash,
            &self.logical_device,
        )
    }
}

impl Drop for Device {
    /// Destroy the logical device.
    ///
    /// Device owns an Arc<Instance> so it's guaranteed that the instance will
    /// not be destroyed until the logical device has been dropped.
    fn drop(&mut self) {
        unsafe {
            self.logical_device.destroy_device(None);
        }
    }
}
