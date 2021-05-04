use super::CachedAtlas;

use crate::graphics::texture_atlas::{
    AtlasVersion, TextureAtlas, TextureHandle,
};

use anyhow::Result;
use std::{collections::HashMap, path::Path};

impl<Atlas: TextureAtlas> CachedAtlas<Atlas> {
    /// Wrap another atlas with caching behavior.
    pub fn new(wrapped: Atlas) -> Self {
        Self {
            atlas: wrapped,
            cache: HashMap::new(),
        }
    }
}

impl<Atlas: TextureAtlas> TextureAtlas for CachedAtlas<Atlas> {
    fn version(&self) -> AtlasVersion {
        self.atlas.version()
    }

    fn build_descriptor_image_info(&self) -> Vec<ash::vk::DescriptorImageInfo> {
        self.atlas.build_descriptor_image_info()
    }

    fn add_texture(
        &mut self,
        path_to_texture_file: impl Into<String>,
    ) -> Result<TextureHandle> {
        let canonical_path =
            Path::new(&path_to_texture_file.into()).canonicalize()?;
        if self.cache.contains_key(&canonical_path) {
            Ok(*self.cache.get(&canonical_path).unwrap())
        } else {
            let handle =
                self.atlas.add_texture(canonical_path.to_str().unwrap())?;
            self.cache.insert(canonical_path, handle);
            Ok(handle)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::graphics::texture_atlas::{
        cached_atlas::{stub_atlas::StubAtlas, CachedAtlas},
        TextureAtlas,
    };

    use anyhow::Result;

    #[test]
    fn create_cached() {
        let _cached = CachedAtlas::new(StubAtlas::new());
    }

    #[test]
    fn texture_handles_should_be_created() -> Result<()> {
        let mut cached = CachedAtlas::new(StubAtlas::new());

        // these are not actual texture files, but they ARE files which will
        // definitely exist when these tests are run
        let handle = cached.add_texture("./src/lib.rs")?;
        let handle2 = cached.add_texture("./src/../src/graphics/mod.rs")?;

        assert_ne!(handle, handle2);

        Ok(())
    }

    #[test]
    fn texture_handles_should_be_cached() -> Result<()> {
        let mut cached = CachedAtlas::new(StubAtlas::new());

        // these are not actual texture files, but they ARE files which will
        // definitely exist when these tests are run
        let handle = cached.add_texture("./src/lib.rs")?;
        let handle2 = cached.add_texture("./src/../src/lib.rs")?;

        assert_eq!(handle, handle2);

        Ok(())
    }
}
