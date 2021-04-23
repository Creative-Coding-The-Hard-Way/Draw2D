//! The main application state.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

mod glfw_window;

use crate::graphics::{Graphics, Vertex};
use glfw_window::GlfwWindow;

use anyhow::Result;

/// The main application.
///
/// The Application has a window, a render context, and one or more systems
/// which can render to a frame when presented by the render context.
pub struct Application {
    graphics: Graphics,
    window_surface: GlfwWindow,
}

impl Application {
    /// Build a new instance of the application.
    pub fn new() -> Result<Self> {
        let mut window_surface = GlfwWindow::windowed("Draw2D", 1366, 768)?;
        window_surface.window.set_resizable(true);
        window_surface.window.set_key_polling(true);
        window_surface.window.set_size_polling(true);
        Ok(Self {
            graphics: Graphics::new(&window_surface)?,
            window_surface,
        })
    }

    fn init(&mut self) {}

    fn update(&mut self) {
        self.graphics.draw2d.vertices.clear();

        // top left
        self.graphics.draw2d.vertices.push(Vertex {
            pos: [-0.75, -0.75],
            uv: [0.0, 0.0],
            ..Default::default()
        });

        // top right
        self.graphics.draw2d.vertices.push(Vertex {
            pos: [0.75, -0.75],
            uv: [1.0, 0.0],
            ..Default::default()
        });

        // bottom right
        self.graphics.draw2d.vertices.push(Vertex {
            pos: [0.75, 0.75],
            uv: [1.0, 1.0],
            ..Default::default()
        });

        // top left
        self.graphics.draw2d.vertices.push(Vertex {
            pos: [-0.75, -0.75],
            uv: [0.0, 0.0],
            ..Default::default()
        });

        // bottom right
        self.graphics.draw2d.vertices.push(Vertex {
            pos: [0.75, 0.75],
            uv: [1.0, 1.0],
            ..Default::default()
        });

        // bottom left
        self.graphics.draw2d.vertices.push(Vertex {
            pos: [-0.75, 0.75],
            uv: [0.0, 1.0],
            ..Default::default()
        });
    }

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        self.init();
        while !self.window_surface.window.should_close() {
            for (_, event) in self.window_surface.poll_events() {
                self.handle_event(event)?;
            }
            self.update();
            self.graphics.render(&self.window_surface)?;
        }
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
                self.window_surface.window.set_should_close(true);
            }

            glfw::WindowEvent::FramebufferSize(_, _) => {
                self.graphics.rebuild_swapchain(&self.window_surface)?;
            }

            _ => {}
        }

        Ok(())
    }
}
