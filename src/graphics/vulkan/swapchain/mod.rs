//! This module defines the Swapchain abstraction and related Vulkan resources.
//!
//! The Swapchain is inherently tied to the display surface and the Window
//! which provides it. As such, only the main application thread should ever
//! directly interact with the swapchain.

mod images;
mod render_pass;
mod selection;

use crate::graphics::vulkan::{Device, WindowSurface};

use anyhow::{Context, Result};
use ash::{extensions::khr, version::DeviceV1_0, vk};
use std::sync::Arc;

/// Manage the swapchain and all dependent resources.
pub struct Swapchain {
    pub swapchain_loader: khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,

    pub framebuffers: Vec<vk::Framebuffer>,
    swapchain_image_views: Vec<vk::ImageView>,

    pub render_pass: vk::RenderPass,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
    pub color_space: vk::ColorSpaceKHR,

    device: Arc<Device>,
}

impl Swapchain {
    /// Create a new swapchain based on the surface, physical device, and the
    /// current size of the framebuffer.
    pub fn new(
        device: Arc<Device>,
        window_surface: &dyn WindowSurface,
        previous: Option<&Swapchain>,
    ) -> Result<Arc<Self>> {
        let image_format = selection::choose_surface_format(
            window_surface,
            &device.physical_device,
        );
        let present_mode = selection::choose_present_mode(
            window_surface,
            &device.physical_device,
        );
        let extent = selection::choose_swap_extent(
            window_surface,
            &device.physical_device,
        )?;
        let image_count = selection::choose_image_count(
            window_surface,
            &device.physical_device,
        )?;

        let mut create_info = vk::SwapchainCreateInfoKHR {
            surface: unsafe { window_surface.get_surface_handle() },

            // image settings
            image_format: image_format.format,
            image_color_space: image_format.color_space,
            image_extent: extent,
            min_image_count: image_count,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,

            // window system presentation settings
            present_mode,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            old_swapchain: if let Some(old_swapchain) = previous {
                old_swapchain.swapchain
            } else {
                vk::SwapchainKHR::null()
            },
            clipped: 1,

            ..Default::default()
        };

        let indices = &[
            device.graphics_queue.family_id,
            device.present_queue.family_id,
        ];

        if device.present_queue.is_same(&device.graphics_queue) {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        } else {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.p_queue_family_indices = indices.as_ptr();
            create_info.queue_family_index_count = indices.len() as u32;
        };

        let swapchain_loader = device.create_swapchain_loader();
        let swapchain =
            unsafe { swapchain_loader.create_swapchain(&create_info, None)? };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .context("unable to get swapchain images")?
        };

        let render_pass = render_pass::create_render_pass(
            device.as_ref(),
            image_format.format,
        )?;

        let swapchain_image_views = images::create_image_views(
            device.as_ref(),
            image_format.format,
            &swapchain_images,
        )?;

        let framebuffers = images::create_framebuffers(
            device.as_ref(),
            &swapchain_image_views,
            render_pass,
            extent,
        )?;

        Ok(Arc::new(Self {
            swapchain_loader,
            swapchain,
            render_pass,
            swapchain_image_views,
            framebuffers,
            extent,
            format: image_format.format,
            color_space: image_format.color_space,
            device,
        }))
    }

    /// Rebuild a new swapchain using this swapchain as a reference.
    pub fn rebuild(
        &self,
        window_surface: &dyn WindowSurface,
    ) -> Result<Arc<Self>> {
        Self::new(self.device.clone(), window_surface, Some(&self))
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical_device
                .queue_wait_idle(self.device.graphics_queue.raw())
                .expect("wait for graphics queue to drain");
            self.device
                .logical_device
                .queue_wait_idle(self.device.present_queue.raw())
                .expect("wait for presentation queue to drain");
            self.device
                .logical_device
                .device_wait_idle()
                .expect("wait for device to idle");

            let logical_device = &self.device.logical_device;
            self.framebuffers.drain(..).for_each(|framebuffer| {
                logical_device.destroy_framebuffer(framebuffer, None);
            });
            self.swapchain_image_views.drain(..).for_each(|view| {
                logical_device.destroy_image_view(view, None);
            });
            self.device
                .logical_device
                .destroy_render_pass(self.render_pass, None);
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}
