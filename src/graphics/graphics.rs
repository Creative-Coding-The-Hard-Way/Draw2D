use super::{layer::LayerStack, Graphics};

use crate::graphics::{
    draw2d::Draw2d,
    layer::{Layer, LayerHandle},
    texture_atlas::{CachedAtlas, GpuAtlas, TextureAtlas, TextureHandle},
    vulkan::{Device, Swapchain, WindowSurface},
    FrameContext, SwapchainState,
};

use anyhow::Result;

impl Graphics {
    /// Instantiate the graphics subsystem.
    pub fn new(window_surface: &dyn WindowSurface) -> Result<Self> {
        let device = Device::new(window_surface)?;
        let swapchain = Swapchain::new(device.clone(), window_surface, None)?;

        let frame_context =
            FrameContext::new(device.clone(), swapchain.clone())?;
        let draw2d = Draw2d::new(device.clone(), swapchain.clone())?;
        let texture_atlas = CachedAtlas::new(GpuAtlas::new(device.clone())?);
        let layer_stack = LayerStack::new();

        Ok(Self {
            texture_atlas,
            draw2d,
            frame_context,
            layer_stack,
            device,
        })
    }

    /// Add a texture, the returned handle can be bound to a layer for
    /// rendering.
    pub fn add_texture(
        &mut self,
        path: impl Into<String>,
    ) -> Result<TextureHandle> {
        self.texture_atlas.add_texture(path)
    }

    pub fn add_layer_to_top(&mut self) -> LayerHandle {
        self.layer_stack.add_layer_to_top()
    }

    pub fn add_layer_to_bottom(&mut self) -> LayerHandle {
        self.layer_stack.add_layer_to_bottom()
    }

    /// Return a mutable reference to the layer referenced by the handle
    ///
    /// PANICs if the layer handle doesn't refer to an actual layer.
    pub fn get_layer_mut(&mut self, layer_handle: &LayerHandle) -> &mut Layer {
        self.layer_stack
            .get_layer_mut(&layer_handle)
            .expect("the provided layer handle doesn't refer to a real layer!")
    }

    /// Render a single frame to the screen.
    pub fn render(&mut self, window_surface: &dyn WindowSurface) -> Result<()> {
        let swapchain_state = self.frame_context.draw_frame(
            &self.draw2d,
            &self.texture_atlas,
            &self.layer_stack,
        )?;
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

impl Drop for Graphics {
    /// Block until the vulkan device idles.
    fn drop(&mut self) {
        use ash::version::DeviceV1_0;
        unsafe {
            self.device
                .logical_device
                .device_wait_idle()
                .expect("error while waiting for the graphics device to idle!")
        }
    }
}
