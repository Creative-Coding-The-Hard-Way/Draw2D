use crate::graphics::{
    frame::Frame,
    vulkan::{Device, Swapchain, WindowSurface},
};

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

/// An enum used by the frame context to signal when the swapchain needs to be
/// rebuilt.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    frames_in_flight: Vec<Option<Frame>>,

    ///! The index of the last frame presented via the swapchain.
    current_frame_index: usize,

    ///! An enum indicating when the swapchain needs to be reconstructed.
    swapchain_state: SwapchainState,

    /// Populated automatically when a frame is started, and consumed
    /// automatically when the frame is completed.
    current_image_acquired_semaphore: vk::Semaphore,

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
            current_image_acquired_semaphore: vk::Semaphore::null(),
            current_frame_index: 0,
            swapchain,
            device,
        })
    }

    /// Borrow the swapchain owned by this frame context.
    pub fn swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    /// Acquire the next swapchain image and select the frame-specific
    /// resources which are now ready to be used.
    pub fn acquire_frame(&mut self) -> Result<Frame, SwapchainState> {
        if self.swapchain_state == SwapchainState::NeedsRebuild {
            return Err(SwapchainState::NeedsRebuild);
        }

        self.current_image_acquired_semaphore = self.frames_in_flight
            [self.current_frame_index]
            .as_ref()
            .expect("the current frame was never ended!")
            .sync
            .image_available_semaphore;

        let result = unsafe {
            self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                self.current_image_acquired_semaphore,
                vk::Fence::null(),
            )
        };
        if let Err(vk::Result::ERROR_OUT_OF_DATE_KHR) = result {
            return Err(SwapchainState::NeedsRebuild);
        }
        if let Ok((_, true)) = result {
            return Err(SwapchainState::NeedsRebuild);
        }

        let (index, _) = result.ok().unwrap();
        self.current_frame_index = index as usize;

        let mut current_frame = self.frames_in_flight[self.current_frame_index]
            .take()
            .expect("the current frame was never returned!");

        current_frame
            .begin_frame()
            .expect("unable to begin the current frame!");

        Ok(current_frame)
    }

    /// Complete the current frame and present the framebuffer.
    pub fn return_frame(&mut self, mut frame: Frame) -> Result<()> {
        let image_acquired_semaphore = self.current_image_acquired_semaphore;
        let render_finished_semaphore =
            frame.finish_frame(image_acquired_semaphore)?;
        self.frames_in_flight[self.current_frame_index] = Some(frame);

        let render_finished_semaphores = &[render_finished_semaphore];
        let swapchains = [self.swapchain.swapchain];
        let indices = [self.current_frame_index as u32];
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
            self.swapchain_state = SwapchainState::NeedsRebuild;
        }

        Ok(())
    }

    /// Wait for all rendering operations to complete on every frame, then
    /// rebuild the swapchain.
    ///
    /// Returns a clone of the swapchain which can be used by other systems.
    pub fn rebuild_swapchain(
        &mut self,
        window_surface: &dyn WindowSurface,
    ) -> Result<Arc<Swapchain>> {
        unsafe {
            self.device.logical_device.device_wait_idle()?;
            self.frames_in_flight.clear();
        }
        self.swapchain = self.swapchain.rebuild(window_surface)?;
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
