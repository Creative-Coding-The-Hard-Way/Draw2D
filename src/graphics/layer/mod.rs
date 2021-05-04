mod batch;
mod layer;
mod layer_handle;
mod layer_stack;

use std::collections::HashMap;

use crate::graphics::{texture_atlas::TextureHandle, vertex::Vertex2d};

/// A layer handle is a unique reference to a layer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerHandle {
    id: i64,
}

/// Layers are ordered, back to front, and render a persistent collection of
/// vertex batches.
#[derive(Clone, Debug)]
pub struct Layer {
    projection: nalgebra::Matrix4<f32>,
    batches: Vec<Batch>,
}

/// A collection of ordered layers for rendering.
pub struct LayerStack {
    layers: HashMap<LayerHandle, Layer>,
    render_order: Vec<LayerHandle>,
}

/// Batches are persintent units of data which can be rendered.
///
/// These are comparable to 'meshes' in other rendering frameworks.
#[derive(Default, Clone, Debug)]
pub struct Batch {
    pub texture_handle: TextureHandle,
    pub vertices: Vec<Vertex2d>,
}
