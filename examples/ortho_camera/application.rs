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
    graphics::{
        layer::{Batch, LayerHandle},
        vertex::Vertex2d,
        Graphics,
    },
    GlfwWindow,
};

use anyhow::Result;

/// The main application.
///
/// The Application has a window, a render context, and one or more systems
/// which can render to a frame when presented by the render context.
pub struct Application {
    layer: LayerHandle,
    camera: OrthoCamera,
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
        window_surface.window.set_scroll_polling(true);
        let (iw, ih) = window_surface.window.get_size();

        let mut graphics = Graphics::new(&window_surface)?;
        let layer = graphics.add_layer_to_bottom();

        Ok(Self {
            layer,
            camera: OrthoCamera::with_viewport(
                ih as f32,
                iw as f32 / ih as f32,
            ),
            graphics,
            window_surface,
        })
    }

    fn init(&mut self) -> Result<()> {
        self.graphics
            .get_layer_mut(&self.layer)
            .set_projection(self.camera.as_matrix());

        let texture_handle = self.graphics.add_texture("assets/example.png")?;

        let mut back = Batch::default();
        let mut middle = Batch::default();
        let mut front = Batch::default();

        back.texture_handle = texture_handle;
        back.add_square(200.0, 1.0);

        middle.add_square(128.0, 0.5);

        front.texture_handle = texture_handle;
        front.add_square(40.0, 0.4);

        self.graphics
            .get_layer_mut(&self.layer)
            .push_batches(&[back, middle, front]);

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
            self.graphics
                .get_layer_mut(&self.layer)
                .set_projection(self.camera.as_matrix());
        }

        Ok(())
    }
}

trait Quads {
    fn add_square(&mut self, size: f32, alpha: f32);
}

impl Quads for Batch {
    fn add_square(&mut self, size: f32, alpha: f32) {
        self.vertices.extend_from_slice(&[
            // top left
            Vertex2d {
                pos: [-size, size],
                uv: [0.0, 0.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // top right
            Vertex2d {
                pos: [size, size],
                uv: [1.0, 0.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // bottom right
            Vertex2d {
                pos: [size, -size],
                uv: [1.0, 1.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // top left
            Vertex2d {
                pos: [-size, size],
                uv: [0.0, 0.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // bottom right
            Vertex2d {
                pos: [size, -size],
                uv: [1.0, 1.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
            // bottom left
            Vertex2d {
                pos: [-size, -size],
                uv: [0.0, 1.0],
                rgba: [1.0, 1.0, 1.0, alpha],
            },
        ]);
    }
}
