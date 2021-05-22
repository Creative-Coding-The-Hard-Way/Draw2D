/// A unique identifier for a texture managed by the texture atlas.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextureHandle(u32);

impl TextureHandle {
    /// Return the raw index which can be passed to the shader for selecting a
    /// texture.
    pub(crate) fn texture_index(&self) -> u32 {
        let TextureHandle(index) = self;
        *index
    }
}

impl Default for TextureHandle {
    /// Return a texture handle which will always refer to a all-white texture
    fn default() -> Self {
        TextureHandle(0)
    }
}
