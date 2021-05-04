pub mod frame;
pub mod frame_context;
pub mod layer;
pub mod texture_atlas;
pub mod vertex;
pub mod vulkan;

mod graphics;
mod graphics_commands;
mod pipeline2d;

use self::{
    frame_context::FrameContext,
    layer::LayerStack,
    pipeline2d::Pipeline2d,
    texture_atlas::{CachedAtlas, GpuAtlas},
    vulkan::Device,
};

use std::sync::Arc;

/// The application's graphics subsystem.
pub struct Graphics {
    /// The graphics pipeline for rendering 2d geometry.
    pipeline2d: Pipeline2d,

    /// The graphics subsystem's texture atlas.
    texture_atlas: CachedAtlas<GpuAtlas>,

    /// The graphics subsystem's visual layers.
    layer_stack: LayerStack,

    /// This object owns the swapchain and all per-frame resources.
    frame_context: FrameContext,

    /// The vulkan device used by all resources in the graphics subsystem.
    device: Arc<Device>,
}
