//! The main application state.
//!
//! # Example
//!
//! ```
//! let mut app = Application::new()?;
//! app.run()?;
//! ```

use draw2d::{
    graphics::{
        ext::{SamplerFactory, TextureLoader},
        layer::{Batch, LayerHandle},
        texture_atlas::TextureAtlas,
        vertex::Vertex2d,
        Graphics,
    },
    GlfwWindow,
};

use ash::vk;

use anyhow::Result;

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

        let texture_handle_1 = self.graphics.add_texture(
            self.graphics.read_texture_file("assets/example.png")?,
        )?;
        let texture_handle_2 = self.graphics.add_texture(
            self.graphics.read_texture_file("assets/example.png")?,
        )?;

        let sampler = unsafe {
            self.graphics.create_sampler(
                "sampler 2",
                vk::SamplerCreateInfo {
                    mag_filter: vk::Filter::LINEAR,
                    min_filter: vk::Filter::LINEAR,
                    address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
                    address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
                    address_mode_w: vk::SamplerAddressMode::REPEAT,
                    anisotropy_enable: 0,
                    border_color: vk::BorderColor::FLOAT_OPAQUE_BLACK,
                    unnormalized_coordinates: 0,
                    compare_enable: 0,
                    compare_op: vk::CompareOp::ALWAYS,
                    mipmap_mode: vk::SamplerMipmapMode::LINEAR,
                    mip_lod_bias: 0.0,
                    min_lod: 0.0,
                    max_lod: vk::LOD_CLAMP_NONE,
                    ..Default::default()
                },
            )?
        };
        let sampler_handle = self.graphics.add_sampler(sampler)?;

        self.graphics
            .bind_sampler_to_texture(sampler_handle, texture_handle_2)?;

        let mut back = Batch::default();
        let mut middle = Batch::default();
        let mut front = Batch::default();

        back.texture_handle = texture_handle_1;
        back.add_square(200.0);

        middle.add_square(128.0);

        front.texture_handle = texture_handle_2;
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
                uv: [-1.0, -1.0],
                ..Default::default()
            },
            // top right
            Vertex2d {
                pos: [size, size],
                uv: [1.0, -1.0],
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
                uv: [-1.0, -1.0],
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
                uv: [-1.0, 1.0],
                ..Default::default()
            },
        ]);
    }
}
