//! The main application state.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

pub mod render_context;
mod triangles;

pub use self::{
    render_context::{RenderContext, SwapchainState},
    triangles::{Triangles, Vertex},
};
use crate::rendering::{glfw_window::GlfwWindow, Device, Swapchain};

use anyhow::{Context, Result};
use std::{sync::Arc, time::Instant};

/// The main application.
///
/// The Application has a window, a render context, and one or more systems
/// which can render to a frame when presented by the render context.
pub struct Application {
    window_surface: Arc<GlfwWindow>,
    render_context: RenderContext,
    swapchain: Arc<Swapchain>,
    triangle: Triangles,
}

impl Application {
    /// Build a new instance of the application.
    ///
    /// Returns `Err()` if anything goes wrong while building the app.
    pub fn new() -> Result<Self> {
        let window_surface = GlfwWindow::new(|glfw| {
            let (mut window, event_receiver) = glfw
                .with_primary_monitor(|glfw, main_monitor| {
                    //if let Some(monitor) = main_monitor {
                    //    let (width, height) = monitor.get_physical_size();
                    //    let (sw, sh) = monitor.get_content_scale();
                    //    let (w, h) = (width as f32 * sw, height as f32 * sh);
                    //    glfw.create_window(
                    //        w as u32,
                    //        h as u32,
                    //        "Ash Starter",
                    //        glfw::WindowMode::FullScreen(monitor),
                    //    )
                    //} else {
                    glfw.create_window(
                        1366,
                        768,
                        "Ash Starter",
                        glfw::WindowMode::Windowed,
                    )
                    //}
                })
                .context("unable to create the glfw window")?;
            window.set_resizable(true);
            window.set_key_polling(true);
            window.set_size_polling(true);
            Ok((window, event_receiver))
        })?;

        let device = Device::new(window_surface.clone())?;
        let swapchain =
            Swapchain::new(device.clone(), window_surface.clone(), None)?;
        let render_context = RenderContext::new(&device, &swapchain)?;
        let triangle = Triangles::new(device.clone(), swapchain.clone())?;

        Ok(Self {
            window_surface,
            render_context,
            swapchain: swapchain.clone(),
            triangle,
        })
    }

    fn init(&mut self) {
        self.triangle.vertices = vec![];
    }

    fn update(&mut self) {
        self.triangle.vertices.clear();

        // top left
        self.triangle
            .vertices
            .push(Vertex::new([-0.75, -0.75], [0.0, 0.0]));

        // top right
        self.triangle
            .vertices
            .push(Vertex::new([0.75, -0.75], [1.0, 0.0]));

        // bottom right
        self.triangle
            .vertices
            .push(Vertex::new([0.75, 0.75], [1.0, 1.0]));

        // top left
        self.triangle
            .vertices
            .push(Vertex::new([-0.75, -0.75], [0.0, 0.0]));

        // bottom right
        self.triangle
            .vertices
            .push(Vertex::new([0.75, 0.75], [1.0, 1.0]));

        // bottom left
        self.triangle
            .vertices
            .push(Vertex::new([-0.75, 0.75], [0.0, 1.0]));
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        let events = self
            .window_surface
            .event_receiver
            .borrow_mut()
            .take()
            .unwrap();
        self.init();
        while !self.window_surface.window.borrow().should_close() {
            self.window_surface.glfw.borrow_mut().poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                log::debug!("{:?}", event);
                self.handle_event(event)?;
            }
            self.update();
            let status = self.render_context.draw_frame(&mut self.triangle)?;
            match status {
                SwapchainState::Ok => {}
                SwapchainState::NeedsRebuild => {
                    self.replace_swapchain()?;
                }
            }
        }
        Ok(())
    }

    /// Update all systems which depend on the swapchain
    fn replace_swapchain(&mut self) -> Result<()> {
        self.swapchain = self.render_context.rebuild_swapchain()?;
        self.triangle.replace_swapchain(self.swapchain.clone())?;
        Ok(())
    }

    /// Handle window events and update the application state as needed.
    fn handle_event(&mut self, event: glfw::WindowEvent) -> Result<()> {
        match event {
            glfw::WindowEvent::Key(
                glfw::Key::Escape,
                _,
                glfw::Action::Press,
                _,
            ) => {
                self.window_surface
                    .window
                    .borrow_mut()
                    .set_should_close(true);
            }

            glfw::WindowEvent::FramebufferSize(_, _) => {
                log::info!("resized");
                self.render_context.needs_rebuild();
            }

            _ => {}
        }

        Ok(())
    }
}
