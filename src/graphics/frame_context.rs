use crate::graphics::{
    vulkan::{Device, Swapchain},
    Draw2d, Frame,
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// An enum used by the frame context to signal when the swapchain needs to be
/// rebuilt.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SwapchainState {
    Ok,
    NeedsRebuild,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// This struct is responsible for requesting a framebuffer from the swapchain,
/// rendering, and presenting the buffer for presentation.
///
/// This app associates resources with each framebuffer to minimize sharing and
/// synchronization between frames.
///
pub struct FrameContext {
    ///! There is one frame per swapchain framebuffer.
    frames_in_flight: Vec<Frame>,

    ///! The index of the last frame presented via the swapchain.
    previous_frame: usize,

    ///! An enum indicating when the swapchain needs to be reconstructed.
    swapchain_state: SwapchainState,

    ///! An owning reference to the application swapchain.
    swapchain: Arc<Swapchain>,

    ///! An owning reference to the application's vulkan device resources.
    device: Arc<Device>,
}

impl FrameContext {
    /// Create a new Frame context.
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Self> {
        Ok(Self {
            frames_in_flight: Frame::create_n_frames(
                &device,
                &swapchain.framebuffers,
            )?,
            swapchain_state: SwapchainState::Ok,
            previous_frame: 0, // always 'start' on frame 0
            swapchain,
            device,
        })
    }

    /// Render a single application frame.
    /// Synchronization between frames is kept to a minimum because each frame
    /// maintains its own resources.
    pub fn draw_frame(&mut self, draw2d: &Draw2d) -> Result<SwapchainState> {
        if self.swapchain_state == SwapchainState::NeedsRebuild {
            return Ok(SwapchainState::NeedsRebuild);
        }

        // Use the previous frame's semaphore because the current frame's
        // index cannot be known until *after* acquiring the image.
        let acquired_semaphore = self.frames_in_flight[self.previous_frame]
            .sync
            .image_available_semaphore;

        let result = unsafe {
            self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                acquired_semaphore,
                vk::Fence::null(),
            )
        };
        if let Err(vk::Result::ERROR_OUT_OF_DATE_KHR) = result {
            return Ok(SwapchainState::NeedsRebuild);
        }
        if let Ok((_, true)) = result {
            return Ok(SwapchainState::NeedsRebuild);
        }

        let (index, _) = result?;
        self.previous_frame = index as usize;

        let render_finished_semaphore = {
            let current_frame = &mut self.frames_in_flight[index as usize];
            current_frame.begin_frame()?;
            draw2d.draw_frame(current_frame)?;
            current_frame.finish_frame(acquired_semaphore)?
        };

        let render_finished_semaphores = &[render_finished_semaphore];
        let swapchains = [self.swapchain.swapchain];
        let indices = [index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(render_finished_semaphores)
            .swapchains(&swapchains)
            .image_indices(&indices);

        let result = unsafe {
            let present_queue = self.device.present_queue.acquire();
            self.swapchain
                .swapchain_loader
                .queue_present(*present_queue, &present_info)
        };
        if Err(vk::Result::ERROR_OUT_OF_DATE_KHR) == result {
            return Ok(SwapchainState::NeedsRebuild);
        }

        Ok(SwapchainState::Ok)
    }

    /// Wait for all rendering operations to complete on every frame, then
    /// rebuild the swapchain.
    ///
    /// Returns a clone of the swapchain which can be used by other systems.
    pub fn rebuild_swapchain(&mut self) -> Result<Arc<Swapchain>> {
        unsafe {
            self.device.logical_device.device_wait_idle()?;
            self.frames_in_flight.clear();
        }
        self.swapchain = self.swapchain.rebuild()?;
        self.frames_in_flight =
            Frame::create_n_frames(&self.device, &self.swapchain.framebuffers)?;
        self.swapchain_state = SwapchainState::Ok;

        Ok(self.swapchain.clone())
    }
}

impl Drop for FrameContext {
    fn drop(&mut self) {
        unsafe {
            // don't delete anything until the GPU has stoped using our
            // resources
            self.device
                .logical_device
                .device_wait_idle()
                .expect("wait for device to idle");

            self.frames_in_flight.clear();
        }
    }
}
