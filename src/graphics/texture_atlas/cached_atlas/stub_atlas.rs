use anyhow::Result;

use crate::texture_atlas::{AtlasVersion, TextureAtlas, TextureHandle};

pub struct StubAtlas {
    pub version: AtlasVersion,
    pub handle_counter: u32,
}

impl StubAtlas {
    pub fn new() -> Self {
        Self {
            version: AtlasVersion { revision_count: 0 },
            handle_counter: 0,
        }
    }
}

impl TextureAtlas for StubAtlas {
    fn version(&self) -> AtlasVersion {
        self.version
    }

    fn build_descriptor_image_info(&self) -> Vec<ash::vk::DescriptorImageInfo> {
        vec![ash::vk::DescriptorImageInfo {
            sampler: ash::vk::Sampler::null(),
            image_view: ash::vk::ImageView::null(),
            image_layout: ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        }]
    }

    fn add_texture(
        &mut self,
        _path_to_texture_file: impl Into<String>,
    ) -> Result<TextureHandle> {
        self.handle_counter += 1;
        Ok(TextureHandle(self.handle_counter))
    }
}
