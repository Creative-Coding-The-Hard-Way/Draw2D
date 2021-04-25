pub mod vulkan;

mod draw2d;
mod frame;
mod frame_context;

pub use self::{
    draw2d::{
        layer::{Layer, LayerHandle},
        texture_atlas::TextureHandle,
        Draw2d, Vertex,
    },
    frame::Frame,
    frame_context::{FrameContext, SwapchainState},
};

use self::vulkan::{Device, Swapchain, WindowSurface};

use anyhow::Result;

/// The application's graphics subsystem.
pub struct Graphics {
    /// This object manages resources and logic for rendering 2d graphics.
    draw2d: Draw2d,

    /// This object owns the swapchain and all per-frame resources.
    frame_context: FrameContext,
}

impl Graphics {
    /// Instantiate the graphics subsystem.
    pub fn new(window_surface: &dyn WindowSurface) -> Result<Self> {
        let device = Device::new(window_surface)?;
        let swapchain = Swapchain::new(device.clone(), window_surface, None)?;

        let frame_context =
            FrameContext::new(device.clone(), swapchain.clone())?;
        let draw2d = Draw2d::new(device.clone(), swapchain.clone())?;

        Ok(Self {
            draw2d,
            frame_context,
        })
    }

    /// Add a texture, the returned handle can be bound to a layer for
    /// rendering.
    pub fn add_texture<P>(&mut self, path: P) -> Result<TextureHandle>
    where
        P: AsRef<std::path::Path>,
    {
        self.draw2d.texture_atlas.add_texture(path)
    }

    pub fn add_layer_to_top(&mut self) -> LayerHandle {
        self.draw2d.layer_stack.add_layer_to_top()
    }

    pub fn add_layer_to_bottom(&mut self) -> LayerHandle {
        self.draw2d.layer_stack.add_layer_to_bottom()
    }

    /// Return a mutable reference to the layer referenced by the handle
    pub fn get_layer_mut(
        &mut self,
        layer_handle: &LayerHandle,
    ) -> Option<&mut Layer> {
        self.draw2d.layer_stack.get_layer_mut(&layer_handle)
    }

    /// Render a single frame to the screen.
    pub fn render(&mut self, window_surface: &dyn WindowSurface) -> Result<()> {
        let swapchain_state = self.frame_context.draw_frame(&self.draw2d)?;
        if swapchain_state == SwapchainState::NeedsRebuild {
            self.rebuild_swapchain(window_surface)?;
        }
        Ok(())
    }

    /// Replace the swapchain and all dependent resources in the Triangle
    /// subsystem.
    pub fn rebuild_swapchain(
        &mut self,
        window_surface: &dyn WindowSurface,
    ) -> Result<()> {
        let swapchain = self.frame_context.rebuild_swapchain(window_surface)?;
        self.draw2d.replace_swapchain(swapchain)?;
        Ok(())
    }
}
