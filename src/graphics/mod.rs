pub mod ext;
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
    frame_context::FrameContext, layer::LayerStack, pipeline2d::Pipeline2d,
    texture_atlas::GpuAtlas, vulkan::Device,
};

use std::sync::Arc;

/// The application's graphics subsystem.
pub struct Graphics {
    /// The graphics pipeline for rendering 2d geometry.
    pipeline2d: Pipeline2d,

    /// The graphics subsystem's texture atlas.
    pub texture_atlas: GpuAtlas,

    /// The graphics subsystem's visual layers.
    layer_stack: LayerStack,

    /// This object owns the swapchain and all per-frame resources.
    frame_context: FrameContext,

    /// the color used to clear the screen
    pub clear_color: [f32; 4],

    /// The vulkan device used by all resources in the graphics subsystem.
    pub device: Arc<Device>,
}
