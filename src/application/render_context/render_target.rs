use super::Frame;

use anyhow::Result;
use ash::vk;

/// Types which implement this interface can be passed to the frame's draw
/// callback.
pub trait RenderTarget {
    /// Render a single complete frame.
    ///
    /// # Arguments
    /// * image_available - a semaphore which is signaled when the swapchain
    ///                     image, and therefore the framebuffer, is available
    ///                     for rendering.
    /// * frame - a mutable reference to all frame resources. Synchronization
    ///           is handled per-frame internally.
    ///
    /// # Return
    ///
    /// Returns a Result which should contain a semaphore which gets signaled
    /// when all rendering operations are complete.
    fn render_to_frame(
        &mut self,
        image_available: vk::Semaphore,
        frame: &mut Frame,
    ) -> Result<vk::Semaphore>;
}

impl<T> RenderTarget for T
where
    T: FnMut(vk::Semaphore, &mut Frame) -> Result<vk::Semaphore>,
{
    fn render_to_frame(
        &mut self,
        image_available: vk::Semaphore,
        frame: &mut Frame,
    ) -> Result<vk::Semaphore> {
        self(image_available, frame)
    }
}
