use super::Graphics;

use crate::graphics::{
    frame::Frame,
    frame_context::FrameContext,
    layer::{Layer, LayerHandle, LayerStack},
    pipeline2d::Pipeline2d,
    texture_atlas::GpuAtlas,
    vulkan::{Device, Swapchain, WindowSurface},
};

use anyhow::Result;

impl Graphics {
    /// Instantiate the graphics subsystem.
    pub fn new(window_surface: &dyn WindowSurface) -> Result<Self> {
        let device = Device::new(window_surface)?;
        let swapchain = Swapchain::new(device.clone(), window_surface, None)?;

        let frame_context =
            FrameContext::new(device.clone(), swapchain.clone())?;
        let pipeline2d = Pipeline2d::new(device.clone(), &swapchain)?;
        let texture_atlas = GpuAtlas::new(device.clone())?;
        let layer_stack = LayerStack::new();

        Ok(Self {
            pipeline2d,
            texture_atlas,
            frame_context,
            layer_stack,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            device,
        })
    }

    /// Add a new graphics layer to the top of the rendering stack.
    ///
    /// This layer will be rendered above all other existing layers.
    pub fn add_layer_to_top(&mut self) -> LayerHandle {
        self.layer_stack.add_layer_to_top()
    }

    /// Add a new graphics layer to the bottom of the rendering stack.
    ///
    /// This layer will be rendered below all other existing layers.
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
        if let Ok(mut frame) = self.frame_context.acquire_frame() {
            self.draw_to_frame(&mut frame)?;
            self.frame_context.return_frame(frame)?
        } else {
            self.rebuild_swapchain(window_surface)?;
        }
        Ok(())
    }

    fn draw_to_frame(&mut self, frame: &mut Frame) -> Result<()> {
        let all_vertices = self.layer_stack.vertices();
        if all_vertices.len() == 0 {
            let graphics_commands = self.record_no_op_commands(frame)?;
            frame.submit_graphics_commands(&[graphics_commands]);
        } else {
            // Fill per-frame gpu resources with the relevant data.
            // SAFE: because resources are not shared between frames.
            unsafe {
                frame.descriptor.update_texture_atlas(&self.texture_atlas);
                frame.vertex_buffer.write_data_arrays(&all_vertices)?;
            }

            let graphics_commands = self.record_layer_draw_commands(frame)?;
            frame.submit_graphics_commands(&[graphics_commands]);
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
        self.pipeline2d = Pipeline2d::new(self.device.clone(), &swapchain)?;
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
