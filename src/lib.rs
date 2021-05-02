//! A bare-minimum set of tools for rendering 2-d graphics with vulkan in rust.

pub mod camera;
pub mod geometry;

mod glfw_window;
mod graphics;

pub use self::{
    glfw_window::{EventReceiver, GlfwWindow},
    graphics::{
        draw2d::{
            layer::{Layer, LayerHandle},
            Vertex,
        },
        texture_atlas, Graphics,
    },
};
