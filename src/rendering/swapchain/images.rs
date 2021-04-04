use crate::rendering::Device;

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};

/// Create one framebuffer for each swapchain image view
///
/// The caller is responsible for destroying the framebuffers when they are
/// done being used.
pub fn create_framebuffers(
    device: &Device,
    swapchain_image_views: &Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>> {
    let mut framebuffers = vec![];
    framebuffers.reserve(swapchain_image_views.len());

    for (i, image_view) in swapchain_image_views.iter().enumerate() {
        let attachments = &[*image_view];
        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);
        let framebuffer = unsafe {
            device
                .logical_device
                .create_framebuffer(&create_info, None)?
        };
        device.name_vulkan_object(
            format!("Framebuffer {}", i),
            vk::ObjectType::FRAMEBUFFER,
            &framebuffer,
        )?;
        framebuffers.push(framebuffer);
    }

    Ok(framebuffers)
}

/// Create image views for each of the swapchain images
///
/// The caller is responsible for destroying the views when they are done
/// being used.
pub fn create_image_views(
    device: &Device,
    format: vk::Format,
    swapchain_images: &Vec<vk::Image>,
) -> Result<Vec<vk::ImageView>> {
    let mut image_views = vec![];
    for (i, image) in swapchain_images.iter().enumerate() {
        let create_info = vk::ImageViewCreateInfo::builder()
            .image(*image)
            .format(format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .components(
                vk::ComponentMapping::builder()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY)
                    .a(vk::ComponentSwizzle::IDENTITY)
                    .build(),
            );
        let view = unsafe {
            device
                .logical_device
                .create_image_view(&create_info, None)
                .context("unable to create image view for swapchain image")?
        };
        device.name_vulkan_object(
            format!("Swapchain Image View {}", i),
            vk::ObjectType::IMAGE_VIEW,
            &view,
        )?;
        image_views.push(view);
    }

    Ok(image_views)
}
