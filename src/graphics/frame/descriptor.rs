use crate::graphics::{
    pipeline2d,
    texture_atlas::{AtlasVersion, TextureAtlas},
    vulkan::Device,
};

use std::sync::Arc;

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// All DescriptorSet-related resources required by this applications frames.
///
/// Each frame has it's own descriptor set, pool, and uniform buffers. Thus,
/// none of these resources are shared between frames. Not sharing is convenient
/// because things like the uniform buffer can be updated in the render loop
/// without any additional synchronization.
pub struct FrameDescriptor {
    atlas_version: AtlasVersion,

    ///! A Descriptor Pool is required for allocating a Descriptor Set.
    descriptor_pool: vk::DescriptorPool,

    ///! The Descriptor Set Layout tells Vulkan how the shader will access
    ///! descriptors.
    descriptor_set_layout: vk::DescriptorSetLayout,

    ///! The Descriptor Set actually binds shader uniforms to gpu resources.
    descriptor_set: vk::DescriptorSet,

    ///! A handle to the device, used to cleanup resources and for helper
    ///! methods.
    device: Arc<Device>,
}

impl FrameDescriptor {
    /// Create a new descriptor for an application frame.
    pub fn new<Name>(device: Arc<Device>, name: Name) -> Result<Self>
    where
        Name: Into<String>,
    {
        let owned_name = name.into();
        let (descriptor_set_layout, bindings) = unsafe {
            pipeline2d::descriptor_sets::create_descriptor_set_layout(&device)?
        };
        device.name_vulkan_object(
            format!("{} - DescriptorSetLayout", owned_name.clone()),
            vk::ObjectType::DESCRIPTOR_SET_LAYOUT,
            &descriptor_set_layout,
        )?;

        // create a descriptor pool which exactly matches the number of bindings
        let pool_sizes: Vec<vk::DescriptorPoolSize> = bindings
            .iter()
            .map(|binding| vk::DescriptorPoolSize {
                ty: binding.descriptor_type,
                descriptor_count: binding.descriptor_count,
            })
            .collect();
        let pool_create_info = vk::DescriptorPoolCreateInfo {
            p_pool_sizes: pool_sizes.as_ptr(),
            pool_size_count: pool_sizes.len() as u32,
            max_sets: 1,
            flags: vk::DescriptorPoolCreateFlags::empty(),
            ..Default::default()
        };
        let descriptor_pool = unsafe {
            device
                .logical_device
                .create_descriptor_pool(&pool_create_info, None)?
        };
        device.name_vulkan_object(
            format!("{} - DescriptorPool", owned_name.clone()),
            vk::ObjectType::DESCRIPTOR_POOL,
            &descriptor_pool,
        )?;

        let descriptor_set_layouts = [descriptor_set_layout];
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool,
            p_set_layouts: descriptor_set_layouts.as_ptr(),
            descriptor_set_count: descriptor_set_layouts.len() as u32,
            ..Default::default()
        };

        let descriptor_set = unsafe {
            device
                .logical_device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)?[0]
        };
        device.name_vulkan_object(
            format!("{} - DescriptorSet", owned_name.clone()),
            vk::ObjectType::DESCRIPTOR_SET,
            &descriptor_set,
        )?;

        Ok(Self {
            descriptor_pool,
            descriptor_set_layout,
            descriptor_set,
            atlas_version: AtlasVersion::new_out_of_date(),
            device,
        })
    }

    /// Update the combined image sampler descriptor based on a texture atlas.
    ///
    /// Unsafe:  it is up to the caller to make sure the image sampler is not
    ///          currently in use by the gpu. This should be safe to invoke in
    ///          the middle of a frame's draw call.
    pub unsafe fn update_texture_atlas(
        &mut self,
        texture_atlas: &impl TextureAtlas,
    ) {
        if texture_atlas.version().is_out_of_date(&self.atlas_version) {
            self.write_texture_descriptor(
                &texture_atlas.build_descriptor_image_info(),
            );
            self.atlas_version = texture_atlas.version();
        }
    }

    /// Update the combined image sampler descriptor.
    ///
    /// Unsafe:  it is up to the caller to make sure the image sampler is not
    ///          currently in use by the gpu. This should be safe to invoke in
    ///          the middle of a frame's draw call.
    unsafe fn write_texture_descriptor(
        &mut self,
        image_infos: &[vk::DescriptorImageInfo],
    ) {
        let descriptor_write = vk::WriteDescriptorSet {
            dst_set: self.descriptor_set,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: image_infos.as_ptr(),
            descriptor_count: image_infos.len() as u32,
            ..Default::default()
        };
        self.device
            .logical_device
            .update_descriptor_sets(&[descriptor_write], &[]);
    }

    /// Return a non-owning handle to the raw vulkan descriptor set object.
    ///
    /// Unsafe:  it is up to the caller to synchronize usage of the set.
    pub unsafe fn raw_descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }
}

impl Drop for FrameDescriptor {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical_device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.logical_device.destroy_descriptor_set_layout(
                self.descriptor_set_layout,
                None,
            );
        }
    }
}
