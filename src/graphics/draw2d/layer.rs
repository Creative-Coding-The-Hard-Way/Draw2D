//! A layer is a collection of vertices which are all rasterized with the same
//! texture.
//!
//! Layers are inherently ordered by Draw2D. Layers are drawn from lowest to
//! highest, so layer 5 will be drawn above layer 2.

use crate::graphics::texture_atlas::TextureHandle;

use super::Vertex;

use std::collections::HashMap;
use std::sync::atomic::{self, AtomicI64};

/// A layer handle is a unique reference to a layer. Handles can be used to
/// efficiently get a reference to a layer while rendering.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerHandle {
    id: i64,
}

impl LayerHandle {
    /// Generate a new LayerHandle which is known to be unique for this
    /// execution.
    fn generate() -> Self {
        static COUNTER: AtomicI64 = AtomicI64::new(0);
        let id = COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
        Self { id }
    }
}

/// Stacked Layers is a datastructure which owns all currently available layers.
/// It is responsible for maintaining the render-order for layers when they are
/// added or removed.
#[derive(Default, Debug, Clone)]
pub struct StackedLayers {
    layers: HashMap<LayerHandle, Layer>,
    render_order: Vec<LayerHandle>,
}

impl StackedLayers {
    /// Add a layer to be rendered on top of all existing layers.
    pub fn add_layer_to_top(&mut self) -> LayerHandle {
        let handle = LayerHandle::generate();
        self.layers.insert(handle, Layer::default());
        self.render_order.push(handle);
        handle
    }

    /// Add a layer to be rendered under all existing layers
    pub fn add_layer_to_bottom(&mut self) -> LayerHandle {
        let handle = LayerHandle::generate();
        self.layers.insert(handle, Layer::default());
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

    /// Return a layer, assuming that the handle is valid
    pub fn get_layer_mut(
        &mut self,
        handle: &LayerHandle,
    ) -> Option<&mut Layer> {
        self.layers.get_mut(handle)
    }
}

#[derive(Default, Clone, Debug)]
pub struct Layer {
    vertices: Vec<Vertex>,
    texture_handle: TextureHandle,
}

impl Layer {
    /// Clear all vertices from this layer.
    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    /// Set the texture for this layer
    pub fn set_texture(&mut self, texture_handle: TextureHandle) {
        self.texture_handle = texture_handle;
    }

    /// Clear the texture to the default
    pub fn clear_texture(&mut self) {
        self.texture_handle = TextureHandle::default();
    }

    /// Push vertices onto this layer's render buffer. Vertices will remain
    /// until 'clear' is called.
    pub fn push_vertices(&mut self, vertices: &[Vertex]) {
        self.vertices.extend_from_slice(vertices);
    }

    pub(super) fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub(super) fn texture_handle(&self) -> &TextureHandle {
        &self.texture_handle
    }
}
