use super::{Batch, Layer};

use nalgebra as na;

impl Layer {
    /// Create a new empty layer.
    pub fn empty() -> Self {
        Self {
            projection: na::Matrix4::identity(),
            batches: vec![],
        }
    }

    /// Clear all batches from the layer.
    pub fn clear(&mut self) {
        self.batches.clear();
    }

    /// Set the layer's projection matrix.
    pub fn set_projection(&mut self, projection: na::Matrix4<f32>) {
        self.projection = projection;
    }

    /// Get a reference to the layer's projection.
    pub fn projection(&self) -> &na::Matrix4<f32> {
        &self.projection
    }

    /// Add a batch to the layer.
    ///
    /// Batches will persist until `clear` is called on this layer.
    pub fn push_batch(&mut self, batch: Batch) {
        self.batches.push(batch);
    }

    /// Push all of the batches into the layer.
    ///
    /// Batches will persist until `clear` is called on this layer.
    pub fn push_batches(&mut self, batches: &[Batch]) {
        self.batches.extend_from_slice(batches);
    }

    pub fn batches(&self) -> &[Batch] {
        &self.batches
    }
}
