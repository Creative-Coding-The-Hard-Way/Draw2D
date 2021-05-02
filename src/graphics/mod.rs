pub mod draw2d;
pub mod frame;
pub mod frame_context;
pub mod layer;
pub mod texture_atlas;
pub mod vertex;
pub mod vulkan;

mod graphics;

pub use self::{
    draw2d::Draw2d,
    frame::Frame,
    frame_context::{FrameContext, SwapchainState},
};

use self::{
    layer::LayerStack,
    texture_atlas::{CachedAtlas, GpuAtlas},
    vulkan::Device,
};

use std::sync::Arc;

/// The application's graphics subsystem.
pub struct Graphics {
    /// This object manages resources and logic for rendering 2d graphics.
    draw2d: Draw2d,

    /// The graphics subsystem's texture atlas.
    texture_atlas: CachedAtlas<GpuAtlas>,

    /// The graphics subsystem's visual layers.
    layer_stack: LayerStack,

    /// This object owns the swapchain and all per-frame resources.
    frame_context: FrameContext,

    /// The vulkan device used by all resources in the graphics subsystem.
    device: Arc<Device>,
}
