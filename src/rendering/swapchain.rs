//! This module defines the Swapchain abstraction and related Vulkan resources.
//!
//! The Swapchain is inherently tied to the display surface and the Window
//! which provides it. As such, only the main application thread should ever
//! directly interact with the swapchain.

mod images;
mod render_pass;
mod selection;

use crate::rendering::{Device, WindowSurface};

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

    pub window_surface: Arc<dyn WindowSurface>,

    device: Arc<Device>,
}

impl Swapchain {
    /// Create a new swapchain based on the surface, physical device, and the
    /// current size of the framebuffer.
    pub fn new(
        device: Arc<Device>,
        window_surface: Arc<dyn WindowSurface>,
        previous: Option<&Swapchain>,
    ) -> Result<Arc<Self>> {
        let image_format = selection::choose_surface_format(
            window_surface.as_ref(),
            &device.physical_device,
        );
        let present_mode = selection::choose_present_mode(
            window_surface.as_ref(),
            &device.physical_device,
        );
        let extent = selection::choose_swap_extent(
            window_surface.as_ref(),
            &device.physical_device,
        )?;
        let image_count = selection::choose_image_count(
            window_surface.as_ref(),
            &device.physical_device,
        )?;

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            // set the surface
            .surface(unsafe { window_surface.get_surface_handle() })
            // image settings
            .image_format(image_format.format)
            .image_color_space(image_format.color_space)
            .image_extent(extent)
            .min_image_count(image_count)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            // window system presentation settings
            .present_mode(present_mode)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .old_swapchain(if let Some(old_swapchain) = previous {
                old_swapchain.swapchain
            } else {
                vk::SwapchainKHR::null()
            })
            .clipped(true);

        let indices = &[
            device.graphics_queue.family_id,
            device.present_queue.family_id,
        ];

        let with_sharing_mode =
            if device.present_queue.is_same(&device.graphics_queue) {
                create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            } else {
                create_info
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(indices)
            };

        let swapchain_loader =
            khr::Swapchain::new(&device.instance.ash, &device.logical_device);
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&with_sharing_mode, None)?
        };

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
            window_surface,
            device,
        }))
    }

    /// Rebuild a new swapchain using this swapchain as a reference.
    pub fn rebuild(&self) -> Result<Arc<Self>> {
        Self::new(
            self.device.clone(),
            self.window_surface.clone(),
            Some(&self),
        )
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        let graphics_queue = self.device.graphics_queue.acquire();
        let present_queue = self.device.present_queue.acquire();
        unsafe {
            self.device
                .logical_device
                .queue_wait_idle(*graphics_queue)
                .expect("wait for graphics queue to drain");
            self.device
                .logical_device
                .queue_wait_idle(*present_queue)
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
