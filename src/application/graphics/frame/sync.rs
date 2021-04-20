use crate::application::Device;

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// Synchronization primitives used to coordinate rendering each frame without
/// accidentally sharing resources.
pub struct FrameSync {
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub graphics_finished_fence: vk::Fence,
}

impl FrameSync {
    /// Create the synchronization primitives used for each frame.
    ///
    pub fn new<Name>(device: &Device, name: Name) -> Result<Self>
    where
        Name: Into<String>,
    {
        let owned_name = name.into();
        let image_available_semaphore = unsafe {
            device
                .logical_device
                .create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)?
        };
        device.name_vulkan_object(
            format!("{} Swapchain Image Available", &owned_name),
            vk::ObjectType::SEMAPHORE,
            &image_available_semaphore,
        )?;

        let render_finished_semaphore = unsafe {
            device
                .logical_device
                .create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)?
        };
        device.name_vulkan_object(
            format!("{} Render Finished", &owned_name),
            vk::ObjectType::SEMAPHORE,
            &render_finished_semaphore,
        )?;

        let graphics_finished_fence = unsafe {
            device.logical_device.create_fence(
                &vk::FenceCreateInfo::builder()
                    .flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )?
        };
        device.name_vulkan_object(
            format!("{} Graphics Finished", &owned_name),
            vk::ObjectType::FENCE,
            &graphics_finished_fence,
        )?;

        Ok(Self {
            image_available_semaphore,
            render_finished_semaphore,
            graphics_finished_fence,
        })
    }

    /// Called by the owner when all sync resources should be destroyed.
    pub unsafe fn destroy(&mut self, device: &Device) {
        //! This function does no checking that the semaphores are done being used,
        //! that is up to the owner. (for example, wait for the device to idle)
        device
            .logical_device
            .destroy_semaphore(self.image_available_semaphore, None);
        device
            .logical_device
            .destroy_semaphore(self.render_finished_semaphore, None);
        device
            .logical_device
            .destroy_fence(self.graphics_finished_fence, None);
    }
}
