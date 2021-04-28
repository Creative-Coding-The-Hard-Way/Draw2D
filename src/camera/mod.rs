mod ortho_camera;

use nalgebra as na;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct OrthoCamera {
    projection: na::Orthographic3<f32>,
    view: na::Translation2<f32>,
    viewport_height: f32,
    viewport_width: f32,
}

/// A really simple input handler for a camera which _just works_ for a demo.
///
/// Real applications will almost certainly prefer a more refined camera
/// controller which provides a more smooth experience.
///
/// Returns `true` when the camera has been changed in some way. Often this is
/// used to trigger a matrix update for the graphics subsystem.
pub fn default_camera_controls(
    camera: &mut OrthoCamera,
    event: &glfw::WindowEvent,
) -> bool {
    use glfw::{Action, Key, WindowEvent};

    let step_h = na::Vector2::new(camera.viewport_width() * 0.1, 0.0);
    let step_v = na::Vector2::new(0.0, camera.viewport_height() * 0.1);
    let pos = camera.world_position();

    match event {
        WindowEvent::Key(Key::Left, _, Action::Release, _)
        | WindowEvent::Key(Key::A, _, Action::Release, _) => {
            camera.set_world_position(&(pos - step_h));
            true
        }

        WindowEvent::Key(Key::Right, _, Action::Release, _)
        | WindowEvent::Key(Key::D, _, Action::Release, _) => {
            camera.set_world_position(&(pos + step_h));
            true
        }

        WindowEvent::Key(Key::Up, _, Action::Release, _)
        | WindowEvent::Key(Key::W, _, Action::Release, _) => {
            camera.set_world_position(&(pos + step_v));
            true
        }

        WindowEvent::Key(Key::Down, _, Action::Release, _)
        | WindowEvent::Key(Key::S, _, Action::Release, _) => {
            camera.set_world_position(&(pos - step_v));
            true
        }

        WindowEvent::Size(iwidth, iheight) => {
            camera.set_aspect_ratio(*iwidth as f32 / *iheight as f32);
            true
        }

        WindowEvent::Scroll(_xoffset, yoffset) => {
            if *yoffset < 0.0 {
                camera.set_viewport_height(camera.viewport_height() * 1.1);
            } else {
                camera.set_viewport_height(camera.viewport_height() * 0.9);
            }
            true
        }

        _ => false,
    }
}
