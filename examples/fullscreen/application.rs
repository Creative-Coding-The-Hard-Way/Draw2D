//! The main application state.
//!
//! Press Space to toggle fullscreen.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

use draw2d::{
    graphics::{
        ext::TextureLoader,
        layer::{Batch, LayerHandle},
        texture_atlas::TextureAtlas,
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
    world_layer: LayerHandle,
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

        let mut graphics = Graphics::new(&window_surface)?;
        let world_layer = graphics.add_layer_to_bottom();

        Ok(Self {
            graphics,
            window_surface,
            world_layer,
        })
    }

    fn init(&mut self) -> Result<()> {
        self.update_projection();

        let texture_handle = self.graphics.add_texture(
            self.graphics.read_texture_file("assets/example.png")?,
        )?;

        let mut back = Batch::default();
        let mut middle = Batch::default();
        let mut front = Batch::default();

        back.texture_handle = texture_handle;
        back.add_square(200.0);

        middle.add_square(128.0);

        front.texture_handle = texture_handle;
        front.add_square(40.0);

        self.graphics
            .get_layer_mut(&self.world_layer)
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

            WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                log::info!("toggle fullscreen");
                self.toggle_fullscreen()?;
            }

            WindowEvent::Size(_, _) => {
                self.update_projection();
            }

            _ => {}
        }

        Ok(())
    }

    fn toggle_fullscreen(&mut self) -> Result<()> {
        use glfw::WindowMode;

        let glfw = &mut self.window_surface.glfw;
        let window = &mut self.window_surface.window;

        let is_fullscreen = window.with_window_mode(|mode| match mode {
            WindowMode::Windowed => false,
            WindowMode::FullScreen(_) => true,
        });

        if is_fullscreen {
            window.set_monitor(WindowMode::Windowed, 100, 100, 1366, 768, None);
        } else {
            glfw.with_primary_monitor_mut(|_, monitor_opt| {
                if let Some(monitor) = monitor_opt {
                    let current_mode = monitor.get_video_mode().unwrap();
                    let (x, y) = monitor.get_pos();
                    let (w, h) = (current_mode.width, current_mode.height);
                    let rate = current_mode.refresh_rate;

                    log::debug!("fullscreen scale: {:?}x{:?}", w, h);
                    window.set_monitor(
                        WindowMode::FullScreen(monitor),
                        x,
                        y,
                        w,
                        h,
                        Some(rate),
                    );
                }
            })
        }

        Ok(())
    }

    fn update_projection(&mut self) {
        let (iwidth, iheight) = self.window_surface.window.get_size();
        let half_width = iwidth as f32 / 2.0;
        let half_height = iheight as f32 / 2.0;
        self.graphics
            .get_layer_mut(&self.world_layer)
            .set_projection(nalgebra::Matrix4::<f32>::new_orthographic(
                -half_width,
                half_width,
                half_height,
                -half_height,
                -1.0,
                1.0,
            ));
    }
}

trait Quads {
    fn add_square(&mut self, size: f32);
}

impl Quads for Batch {
    fn add_square(&mut self, size: f32) {
        self.vertices.extend_from_slice(&[
            // top left
            Vertex2d {
                pos: [-size, size],
                uv: [0.0, 0.0],
                ..Default::default()
            },
            // top right
            Vertex2d {
                pos: [size, size],
                uv: [1.0, 0.0],
                ..Default::default()
            },
            // bottom right
            Vertex2d {
                pos: [size, -size],
                uv: [1.0, 1.0],
                ..Default::default()
            },
            // top left
            Vertex2d {
                pos: [-size, size],
                uv: [0.0, 0.0],
                ..Default::default()
            },
            // bottom right
            Vertex2d {
                pos: [size, -size],
                uv: [1.0, 1.0],
                ..Default::default()
            },
            // bottom left
            Vertex2d {
                pos: [-size, -size],
                uv: [0.0, 1.0],
                ..Default::default()
            },
        ]);
    }
}
