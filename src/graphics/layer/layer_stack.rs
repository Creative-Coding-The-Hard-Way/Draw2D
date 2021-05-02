use std::collections::HashMap;

use crate::graphics::vertex::Vertex2d;

use super::{Layer, LayerHandle, LayerStack};

impl LayerStack {
    /// Create a new stack with zero visible layers.
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
            render_order: vec![],
        }
    }

    /// Add a layer to be rendered on top of all existing layers.
    pub fn add_layer_to_top(&mut self) -> LayerHandle {
        let handle = LayerHandle::generate();
        self.layers.insert(handle, Layer::empty());
        self.render_order.push(handle);
        handle
    }

    /// Add a layer to be rendered under all existing layers
    pub fn add_layer_to_bottom(&mut self) -> LayerHandle {
        let handle = LayerHandle::generate();
        self.layers.insert(handle, Layer::empty());
        if self.render_order.is_empty() {
            self.render_order.push(handle);
        } else {
            self.render_order.insert(0, handle);
        }
        handle
    }

    /// Return the set of all layer references in their render order.
    pub fn layers(&self) -> Vec<&Layer> {
        self.render_order
            .iter()
            .map(|handle| self.layers.get(handle).unwrap())
            .collect::<Vec<&Layer>>()
    }

    /// Get the layer referenced by a handle.
    ///
    /// Returns None if the handle is invalid.
    pub fn get_layer_mut(
        &mut self,
        handle: &LayerHandle,
    ) -> Option<&mut Layer> {
        self.layers.get_mut(handle)
    }

    /// Get the slice of all vertices for all layers and batches in order.
    ///
    /// This can be used to build a vertex buffer when rendering.
    ///
    /// The layout of the vertices is like the following:
    ///
    /// - Layer
    ///   - Batch vertices
    ///   - Batch vertices
    /// - Layer
    ///   - Batch vertices
    /// - Layer
    ///   - Batch vertices
    ///   - Batch vertices
    ///   - Batch vertices
    ///
    pub fn vertices(&self) -> Vec<&[Vertex2d]> {
        let layers: Vec<&Layer> = self
            .render_order
            .iter()
            .map(|handle| self.layers.get(handle).unwrap())
            .collect();

        let mut verts: Vec<&[Vertex2d]> = vec![];
        for layer in layers {
            verts.reserve(layer.batches.len());
            for batch in &layer.batches {
                verts.push(&batch.vertices);
            }
        }
        verts
    }
}
