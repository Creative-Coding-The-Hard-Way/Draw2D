mod frame;
mod render_target;

pub use self::{frame::Frame, render_target::RenderTarget};
use crate::rendering::{Device, Swapchain};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SwapchainState {
    Ok,
    NeedsRebuild,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The RenderContext is responsible for requesting an image from the
/// swapchain, picking the corresponding Frame instance, and dispatching to the
/// RenderTarget.
///
/// # Sequence Diagram
/// ```mermaid
/// sequenceDiagram
///     participant app as Application
///     participant rc as Render Context
///     participant rt as Render Target
///     participant frame as Frame <br> (Index)
///     participant swap as Swapchain
///
///     app ->>+ rc: Draw Frame(RenderTarget)
///
///     rc ->> swap: Acquire Frame
///     activate swap
///     swap -->> rc: (Index, Acquired)
///
///     rc ->>+frame: Begin Frame (Index)
///     frame ->> frame: wait for fences from last submit
///     frame ->> frame: reset resources
///
///
///     rc ->>+ rt: draw_to_frame(Acquired, Current Frame)
///
///     rt ->> frame: request <br> command buffer
///     rt ->> rt: fill command buffer
///     rt ->> frame: submit <br> command buffer
///     rt -->>- rc: finish frame
///
///
///     deactivate frame
///
///     rc ->> swap: Present
///     deactivate swap
///
///     rc -->>- app: Ok()
/// ```
pub struct RenderContext {
    frames_in_flight: Vec<Frame>,
    previous_frame: usize,
    swapchain_state: SwapchainState,
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
}

impl RenderContext {
    pub fn new(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
    ) -> Result<Self> {
        Ok(Self {
            frames_in_flight: Frame::create_n_frames(
                &device,
                &swapchain.framebuffers,
            )?,
            swapchain_state: SwapchainState::Ok,
            previous_frame: 0, // always 'start' on frame 0
            swapchain: swapchain.clone(),
            device: device.clone(),
        })
    }

    /// Signal that the swapchain needs to be rebuilt before the next frame
    /// is rendered.
    pub fn needs_rebuild(&mut self) {
        self.swapchain_state = SwapchainState::NeedsRebuild;
    }

    /// Render a single application frame.
    pub fn draw_frame<Target>(
        &mut self,
        render_target: &mut Target,
    ) -> Result<SwapchainState>
    where
        Target: RenderTarget,
    {
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
            let mut current_frame = &mut self.frames_in_flight[index as usize];
            current_frame.begin_frame()?;
            render_target
                .render_to_frame(acquired_semaphore, &mut current_frame)?
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

impl Drop for RenderContext {
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
