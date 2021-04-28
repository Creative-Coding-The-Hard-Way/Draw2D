//! The main application state.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

use draw2d::{
    camera::{default_camera_controls, OrthoCamera},
    GlfwWindow, Graphics, Layer, LayerHandle, Vertex,
};

use anyhow::Result;

/// The main application.
///
/// The Application has a window, a render context, and one or more systems
/// which can render to a frame when presented by the render context.
pub struct Application {
    graphics: Graphics,
    window_surface: GlfwWindow,
    layer: Option<LayerHandle>,
    camera: OrthoCamera,
}

impl Application {
    /// Build a new instance of the application.
    pub fn new() -> Result<Self> {
        let mut window_surface = GlfwWindow::windowed("Draw2D", 1366, 768)?;
        window_surface.window.set_resizable(true);
        window_surface.window.set_key_polling(true);
        window_surface.window.set_size_polling(true);
        window_surface.window.set_scroll_polling(true);
        let (iw, ih) = window_surface.window.get_size();
        Ok(Self {
            graphics: Graphics::new(&window_surface)?,
            window_surface,
            layer: None,
            camera: OrthoCamera::with_viewport(
                ih as f32,
                iw as f32 / ih as f32,
            ),
        })
    }

    fn init(&mut self) -> Result<()> {
        self.graphics.set_projection(&self.camera.as_matrix());

        let texture_handle = self.graphics.add_texture("assets/example.png")?;

        // background
        {
            let layer_handle = self.graphics.add_layer_to_bottom();
            let layer = self.graphics.get_layer_mut(&layer_handle).unwrap();
            layer.set_texture(texture_handle);
            layer.add_square(200.0, 1.0);
        }

        // foreground
        {
            let layer_handle = self.graphics.add_layer_to_top();
            let layer = self.graphics.get_layer_mut(&layer_handle).unwrap();
            layer.add_square(128.0, 0.5);
        }

        // (even more) foreground
        {
            let layer_handle = self.graphics.add_layer_to_top();
            self.layer = Some(layer_handle);
            let layer = self.graphics.get_layer_mut(&layer_handle).unwrap();
            layer.set_texture(texture_handle);
            layer.add_square(40.0, 0.4);
        }

        Ok(())
    }

    fn update(&mut self) {}

    /// Run the application, blocks until the main event loop exits.
    pub fn run(mut self) -> Result<()> {
        self.init()?;
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
        use glfw::{Action, Key, WindowEvent};
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.window_surface.window.set_should_close(true);
            }
            _ => {}
        }

        if default_camera_controls(&mut self.camera, &event) {
            self.graphics.set_projection(&self.camera.as_matrix());
        }

        Ok(())
    }
}

trait Quads {
    fn add_square(&mut self, size: f32, alpha: f32);
}

impl Quads for Layer {
    fn add_square(&mut self, size: f32, alpha: f32) {
        self.push_vertices(&[
            // top left
            Vertex {
                pos: [-size, size],
                uv: [0.0, 0.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // top right
            Vertex {
                pos: [size, size],
                uv: [1.0, 0.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // bottom right
            Vertex {
                pos: [size, -size],
                uv: [1.0, 1.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // top left
            Vertex {
                pos: [-size, size],
                uv: [0.0, 0.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // bottom right
            Vertex {
                pos: [size, -size],
                uv: [1.0, 1.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // bottom left
            Vertex {
                pos: [-size, -size],
                uv: [0.0, 1.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
        ]);
    }
}
