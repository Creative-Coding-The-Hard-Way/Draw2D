//! This module provides a structure for finding queue families which support
//! this application.

use crate::rendering::WindowSurface;

use super::Queue;

use anyhow::{Context, Result};
use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use std::sync::Arc;

/// This struct holds all of the queue indices required by this application.
pub struct QueueFamilyIndices {
    /// the index for the graphics queue
    graphics_family_index: u32,

    /// the index for the presentation queue
    present_family_index: u32,
}

impl QueueFamilyIndices {
    /// Find all of the queue families required by this application.
    ///
    /// Yields an Err if any of the queues cannot be found.
    ///
    /// The implementation is greedy, e.g. the same queue will be used for
    /// multiple operations where possible.
    pub fn find(
        physical_device: &vk::PhysicalDevice,
        ash: &ash::Instance,
        window_surface: &dyn WindowSurface,
    ) -> Result<Self> {
        let queue_families = unsafe {
            ash.get_physical_device_queue_family_properties(*physical_device)
        };

        let mut graphics_family = None;
        let mut present_family = None;

        queue_families.iter().enumerate().for_each(|(i, family)| {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family = Some(i as u32);
            }

            let present_support = unsafe {
                window_surface.get_physical_device_surface_support(
                    physical_device,
                    i as u32,
                )
            };
            match present_support {
                Ok(true) => {
                    present_family = Some(i as u32);
                }
                _ => {}
            }
        });

        let graphics_family_index = graphics_family
            .context("unable to find queue family which supports graphics")?;

        let present_family_index = present_family
            .context("unable to find a queue which supports presentation")?;

        Ok(Self {
            graphics_family_index,
            present_family_index,
        })
    }

    /// Create a vector of queue create info structs based on the indices.
    ///
    /// Automatically handles duplicate indices
    pub fn as_queue_create_infos(&self) -> Vec<vk::DeviceQueueCreateInfo> {
        let mut create_infos = vec![vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(self.graphics_family_index)
            .queue_priorities(&[1.0])
            .build()];

        if self.graphics_family_index != self.present_family_index {
            create_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(self.present_family_index)
                    .queue_priorities(&[1.0])
                    .build(),
            );
        }

        create_infos
    }

    /// Return a tuple of the actual vulkan queues.
    ///
    /// Handles duplicate queue family indices automatically.
    pub fn get_queues(
        &self,
        logical_device: &ash::Device,
    ) -> Result<(Arc<Queue>, Arc<Queue>)> {
        let raw_graphics_queue = unsafe {
            logical_device.get_device_queue(self.graphics_family_index, 0)
        };
        let graphics_queue =
            Queue::from_raw(raw_graphics_queue, self.graphics_family_index, 0);

        let is_same = self.graphics_family_index == self.present_family_index;
        let present_queue = if is_same {
            graphics_queue.clone()
        } else {
            let raw_present_queue = unsafe {
                logical_device.get_device_queue(self.present_family_index, 0)
            };
            Queue::from_raw(raw_present_queue, self.present_family_index, 0)
        };

        Ok((graphics_queue, present_queue))
    }
}
