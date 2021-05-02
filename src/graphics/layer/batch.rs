use super::Batch;

impl Batch {
    /// Create a new empty batch.
    pub fn empty() -> Self {
        Self {
            ..Default::default()
        }
    }
}
