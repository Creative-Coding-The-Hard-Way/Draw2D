//! A bare-minimum set of tools for rendering 2-d graphics with vulkan in rust.

pub mod camera;
pub mod geometry;
pub mod graphics;

mod glfw_window;

pub use self::glfw_window::{EventReceiver, GlfwWindow};
