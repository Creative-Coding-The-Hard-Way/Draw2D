//! The main application state.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

use super::text_renderer::TextRenderer;

use ab_glyph::{Font, FontArc, PxScaleFont};
use draw2d::{
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
    world_layer: LayerHandle,
    graphics: Graphics,
    window_surface: GlfwWindow,

    title_renderer: TextRenderer<FontArc, PxScaleFont<FontArc>>,
    body_renderer: TextRenderer<FontArc, PxScaleFont<FontArc>>,
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

        let font_bytes = include_bytes!(
            "../../assets/Architects_Daughter/ArchitectsDaughter-Regular.ttf"
        );
        let font =
            ab_glyph::FontArc::try_from_slice(font_bytes)?.into_scaled(64.0);
        let title_renderer = TextRenderer::new(font, &mut graphics)?;

        let font_bytes =
            include_bytes!("../../assets/Montserrat/Montserrat-Regular.ttf");
        let font =
            ab_glyph::FontArc::try_from_slice(font_bytes)?.into_scaled(32.0);
        let body_renderer = TextRenderer::new(font, &mut graphics)?;

        Ok(Self {
            graphics,
            window_surface,
            world_layer,
            title_renderer,
            body_renderer,
        })
    }

    fn init(&mut self) -> Result<()> {
        self.update_projection();

        Ok(())
    }

    fn update(&mut self) {
        let title = self
            .title_renderer
            .layout_text("Hello World!", [200.0, 50.0]);
        let body = self.body_renderer.layout_text(
            indoc::indoc!(
                r#"
                Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do
                eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut
                enim ad minim veniam, quis nostrud exercitation ullamco laboris
                nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor
                in reprehenderit in voluptate velit esse cillum dolore eu
                fugiat nulla pariatur. Excepteur sint occaecat cupidatat non
                proident, sunt in culpa qui officia deserunt mollit anim id est
                laborum.
                "#
            ),
            [200.0, 120.0],
        );

        let layer = self.graphics.get_layer_mut(&self.world_layer);
        layer.clear();
        layer.push_batches(&[body, title]);
    }

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
        match event {
            glfw::WindowEvent::Key(
                glfw::Key::Escape,
                _,
                glfw::Action::Press,
                _,
            ) => {
                self.window_surface.window.set_should_close(true);
            }

            glfw::WindowEvent::Size(_, _) => {
                self.update_projection();
            }

            _ => {}
        }

        Ok(())
    }

    fn update_projection(&mut self) {
        let (iwidth, iheight) = self.window_surface.window.get_size();
        self.graphics
            .get_layer_mut(&self.world_layer)
            .set_projection(nalgebra::Matrix4::<f32>::new_orthographic(
                0.0,
                iwidth as f32,
                0.0,
                iheight as f32,
                -1.0,
                1.0,
            ));
    }
}

struct Quad {
    top_left: Vertex2d,
    top_right: Vertex2d,
    bottom_left: Vertex2d,
    bottom_right: Vertex2d,
}

impl Quad {
    pub fn add_to_batch(&self, batch: &mut Batch) {
        batch.vertices.extend_from_slice(&[
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.top_left,
            self.bottom_right,
            self.bottom_left,
        ]);
    }
}
