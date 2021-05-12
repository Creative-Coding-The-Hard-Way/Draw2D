use super::{descriptor_sets, Pipeline2d};

use crate::graphics::{
    texture_atlas::MAX_SUPPORTED_TEXTURES,
    vertex::Vertex2d,
    vulkan::{ffi, shader_module::ShaderModule, Device, Swapchain},
};

use anyhow::{Context, Result};
use ash::{version::DeviceV1_0, vk};
use std::{
    ffi::{c_void, CString},
    sync::Arc,
};

impl Pipeline2d {
    pub fn new(device: Arc<Device>, swapchain: &Swapchain) -> Result<Self> {
        let vertex_module = ShaderModule::new(
            &device,
            "Vertex Shader",
            std::include_bytes!("../../../shaders/sprv/texture2d.vert.sprv"),
        )?;
        let fragment_module = ShaderModule::new(
            &device,
            "Fragment Shader",
            std::include_bytes!("../../../shaders/sprv/texture2d.frag.sprv"),
        )?;

        // Dynamic parts of the pipeline

        let entry = CString::new("main").unwrap();
        let vertex_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: vertex_module.shader_module,
            p_name: entry.as_ptr(),
            ..Default::default()
        };

        let specialization_map_entries = [vk::SpecializationMapEntry {
            constant_id: 0,
            offset: 0,
            size: std::mem::size_of::<u32>(),
            ..Default::default()
        }];
        let specialization_data =
            unsafe { ffi::any_as_u8_slice(&MAX_SUPPORTED_TEXTURES) };
        let fragment_specialization_info = vk::SpecializationInfo {
            p_map_entries: specialization_map_entries.as_ptr(),
            map_entry_count: specialization_map_entries.len() as u32,
            p_data: specialization_data.as_ptr() as *const c_void,
            data_size: specialization_data.len(),
        };
        let fragment_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: fragment_module.shader_module,
            p_specialization_info: &fragment_specialization_info,
            p_name: entry.as_ptr(),
            ..Default::default()
        };

        // Fixed Function Configuration

        let (binding_descriptions, attribute_descriptions) =
            Vertex2d::binding_description();
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
            p_vertex_binding_descriptions: binding_descriptions.as_ptr(),
            vertex_binding_description_count: binding_descriptions.len() as u32,
            p_vertex_attribute_descriptions: attribute_descriptions.as_ptr(),
            vertex_attribute_description_count: attribute_descriptions.len()
                as u32,
            ..Default::default()
        };

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: 0,
            ..Default::default()
        };

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain.extent.width as f32,
            height: swapchain.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        }];

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            p_viewports: viewports.as_ptr(),
            viewport_count: 1,
            p_scissors: scissors.as_ptr(),
            scissor_count: 1,
            ..Default::default()
        };

        let raster_state = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: 0,
            rasterizer_discard_enable: 0,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: 0,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            ..Default::default()
        };

        let multisample_state = vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: 0,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            p_sample_mask: std::ptr::null(),
            min_sample_shading: 1.0,
            alpha_to_coverage_enable: 0,
            alpha_to_one_enable: 0,
            ..Default::default()
        };

        let blend_attachments = [vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
            blend_enable: 1,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        }];

        let blend_state = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: 0,
            logic_op: vk::LogicOp::COPY,
            blend_constants: [0.0, 0.0, 0.0, 0.0],
            p_attachments: blend_attachments.as_ptr(),
            attachment_count: blend_attachments.len() as u32,
            ..Default::default()
        };

        let (descriptor_set_layout, _bindings) =
            unsafe { descriptor_sets::create_descriptor_set_layout(&device)? };
        device.name_vulkan_object(
            "Graphics Pipeline Descriptor Set Layout",
            vk::ObjectType::DESCRIPTOR_SET_LAYOUT,
            &descriptor_set_layout,
        )?;

        let layouts = [descriptor_set_layout];
        let push_constant_ranges =
            vec![descriptor_sets::create_push_constant_range()];
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            p_set_layouts: layouts.as_ptr(),
            set_layout_count: layouts.len() as u32,
            p_push_constant_ranges: push_constant_ranges.as_ptr(),
            push_constant_range_count: push_constant_ranges.len() as u32,
            ..Default::default()
        };

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

        let stages = [vertex_create_info, fragment_create_info];
        let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            p_stages: stages.as_ptr(),
            stage_count: stages.len() as u32,
            p_vertex_input_state: &vertex_input_state,
            p_input_assembly_state: &input_assembly_state,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &raster_state,
            p_multisample_state: &multisample_state,
            p_color_blend_state: &blend_state,

            p_tessellation_state: std::ptr::null(),
            p_dynamic_state: std::ptr::null(),
            p_depth_stencil_state: std::ptr::null(),

            layout: pipeline_layout,
            render_pass: swapchain.render_pass,
            subpass: 0,
            base_pipeline_index: -1,
            base_pipeline_handle: vk::Pipeline::null(),

            ..Default::default()
        };

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

        Ok(Self {
            descriptor_set_layout,
            pipeline_layout,
            pipeline,
            device: device.clone(),
        })
    }

    /// Borrow the raw vulkan pipeline handle.
    pub fn raw_pipeline(&self) -> &vk::Pipeline {
        &self.pipeline
    }

    /// Borrow the pipeline layout handle.
    pub fn raw_pipeline_layout(&self) -> &vk::PipelineLayout {
        &self.pipeline_layout
    }
}

impl Drop for Pipeline2d {
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
