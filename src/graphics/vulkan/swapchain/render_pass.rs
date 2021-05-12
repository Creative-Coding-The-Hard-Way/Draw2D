//! Functions to create a proper render pass for this application's graphics
//! pipeline.

use crate::graphics::vulkan::Device;

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// Create a render pass for the graphics pipeline.
pub fn create_render_pass(
    device: &Device,
    format: vk::Format,
) -> Result<vk::RenderPass> {
    let attachments = [vk::AttachmentDescription {
        format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    }];

    let color_references = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];

    let subpasses = [vk::SubpassDescription {
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        p_color_attachments: color_references.as_ptr(),
        color_attachment_count: color_references.len() as u32,
        ..Default::default()
    }];

    let dependencies = [vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: vk::AccessFlags::empty(),
        dst_subpass: 0,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dependency_flags: vk::DependencyFlags::default(),
    }];

    let create_info = vk::RenderPassCreateInfo {
        p_attachments: attachments.as_ptr(),
        attachment_count: attachments.len() as u32,
        p_subpasses: subpasses.as_ptr(),
        subpass_count: subpasses.len() as u32,
        p_dependencies: dependencies.as_ptr(),
        dependency_count: dependencies.len() as u32,
        ..Default::default()
    };

    let render_pass = unsafe {
        device
            .logical_device
            .create_render_pass(&create_info, None)?
    };

    device.name_vulkan_object(
        "Application Render Pass",
        vk::ObjectType::RENDER_PASS,
        &render_pass,
    )?;

    Ok(render_pass)
}
