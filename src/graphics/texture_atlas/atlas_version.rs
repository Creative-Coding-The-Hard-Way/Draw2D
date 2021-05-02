use super::AtlasVersion;

impl AtlasVersion {
    /// A binding revision which will always be considered 'out_of_date'
    /// relative to the atlas.
    pub fn out_of_date() -> Self {
        Self { revision_count: 0 }
    }
}
