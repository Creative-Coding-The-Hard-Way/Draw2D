use super::{UniformBufferObject, Vertex};
use crate::rendering::{Device, ShaderModule, Swapchain};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::{ffi::CString, sync::Arc};

/// All vulkan resources related to the graphics pipeline.
pub struct GraphicsPipeline {
    descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,

    device: Arc<Device>,

    #[allow(dead_code)]
    swapchain: Arc<Swapchain>,
}

impl GraphicsPipeline {
    pub fn new(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
    ) -> Result<Arc<Self>> {
        let vertex_module = ShaderModule::new(
            device,
            "Vertex Shader",
            std::include_bytes!("../../../shaders/sprv/passthrough.vert.spv"),
        )?;
        let fragment_module = ShaderModule::new(
            device,
            "Fragment Shader",
            std::include_bytes!("../../../shaders/sprv/passthrough.frag.spv"),
        )?;

        // Dynamic parts of the pipeline

        let entry = CString::new("main").unwrap();
        let vertex_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_module.shader_module)
            .name(&entry);
        let fragment_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_module.shader_module)
            .name(&entry);

        // Fixed Function Configuration

        let (binding_descriptions, attribute_descriptions) =
            Vertex::binding_description();
        let vertex_input_state =
            vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_binding_descriptions(&binding_descriptions)
                .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly =
            vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

        let viewports = &[vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build()];

        let scissors = &[vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent)
            .build()];

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .viewports(viewports)
            .scissor_count(1)
            .scissors(scissors);

        let raster_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0);

        let multisample_state =
            vk::PipelineMultisampleStateCreateInfo::builder()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0)
                .sample_mask(&[])
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false);

        let blend_attachments =
            &[vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(
                    vk::ColorComponentFlags::R
                        | vk::ColorComponentFlags::G
                        | vk::ColorComponentFlags::B
                        | vk::ColorComponentFlags::A,
                )
                .blend_enable(false)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ZERO)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD)
                .build()];

        let blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .attachments(blend_attachments);

        let descriptor_set_layout_bindings =
            [UniformBufferObject::descriptor_set_layout_binding()];
        let descriptor_set_layout_create_info =
            vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&descriptor_set_layout_bindings);

        let descriptor_set_layout = unsafe {
            device.logical_device.create_descriptor_set_layout(
                &descriptor_set_layout_create_info,
                None,
            )?
        };

        let layouts = [descriptor_set_layout];
        let push_constant_ranges = vec![];
        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&layouts)
                .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = unsafe {
            device
                .logical_device
                .create_pipeline_layout(&pipeline_layout_create_info, None)?
        };
        device.name_vulkan_object(
            "Graphics Pipeline Layout",
            vk::ObjectType::PIPELINE_LAYOUT,
            &pipeline_layout,
        )?;

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&[vertex_create_info.build(), fragment_create_info.build()])
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&raster_state)
            .multisample_state(&multisample_state)
            //.depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&blend_state)
            //.dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(swapchain.render_pass)
            .subpass(0)
            .base_pipeline_index(-1)
            .base_pipeline_handle(vk::Pipeline::null())
            .build();

        let pipelines = unsafe {
            device
                .logical_device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_create_info],
                    None,
                )
                .map_err(|(_, err)| err)
                .context("unable to create graphics pipeline")?
        };
        let pipeline = pipelines[0];
        device.name_vulkan_object(
            "Application Graphics Pipeline",
            vk::ObjectType::PIPELINE,
            &pipeline,
        )?;

        // build pipeline object

        Ok(Arc::new(Self {
            descriptor_set_layout,
            pipeline_layout,
            pipeline,
            device: device.clone(),
            swapchain: swapchain.clone(),
        }))
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_pipeline(self.pipeline, None);
            self.device
                .logical_device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.logical_device.destroy_descriptor_set_layout(
                self.descriptor_set_layout,
                None,
            );
        }
    }
}
