mod window_surface;

use crate::graphics::vulkan::Instance;

use anyhow::{bail, Context, Result};
use ash::{extensions::khr::Surface, version::InstanceV1_0, vk, vk::Handle};
use std::{
    ptr::null,
    sync::{mpsc::Receiver, Arc},
};

pub type EventReceiver = Receiver<(f64, glfw::WindowEvent)>;

/// Resources required for rendering to a single GLFW window.
pub struct GlfwWindow {
    /// The glfw library instance
    pub glfw: glfw::Glfw,

    /// The glfw window
    pub window: glfw::Window,

    /// The event reciever. Usually consumed by the application's main loop.
    pub event_receiver: EventReceiver,

    /// The raw vulkan surface handle
    surface: vk::SurfaceKHR,

    /// Extension functions for interacting with the surface
    surface_loader: Surface,

    /// The instance must not be destroyed before the WindowSurface
    instance: Arc<Instance>,
}

impl GlfwWindow {
    /// Create a new application window and vulkan surface.
    ///
    /// It's safe to clone the the resulting window, but it is not safe to use
    /// glfw window functions from any thread but the main thread. (the thread
    /// where this `new` function was invoked).
    ///
    pub fn new<F>(create_window: F) -> Result<Self>
    where
        F: FnOnce(&mut glfw::Glfw) -> Result<(glfw::Window, EventReceiver)>,
    {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
            .context("unable to setup glfw for this application")?;

        let (window, event_receiver) =
            Self::build_vulkan_window(&mut glfw, create_window)?;

        let instance =
            Instance::new(&glfw.get_required_instance_extensions().context(
                "unable to get required vulkan extensions for this platform",
            )?)?;

        let surface = Self::create_surface(&instance, &window)?;
        let surface_loader = Surface::new(&instance.entry, &instance.ash);

        Ok(Self {
            surface,
            surface_loader,

            glfw,
            window,
            event_receiver,

            instance,
        })
    }

    /// Create a new fullscreen window using the primary monitor.
    #[allow(dead_code)]
    pub fn fullscreen(title: &str) -> Result<Self> {
        GlfwWindow::new(|glfw| {
            let (window, event_receiver) = glfw
                .with_primary_monitor(|glfw, main_monitor| {
                    if let Some(monitor) = main_monitor {
                        let (width, height) = monitor.get_physical_size();
                        let (sw, sh) = monitor.get_content_scale();
                        let (w, h) = (width as f32 * sw, height as f32 * sh);
                        glfw.create_window(
                            w as u32,
                            h as u32,
                            title,
                            glfw::WindowMode::FullScreen(monitor),
                        )
                    } else {
                        glfw.create_window(
                            1366,
                            768,
                            title,
                            glfw::WindowMode::Windowed,
                        )
                    }
                })
                .context("unable to create the glfw window")?;
            Ok((window, event_receiver))
        })
    }

    /// Create a new non-fullscreen window.
    #[allow(dead_code)]
    pub fn windowed(title: &str, width: u32, height: u32) -> Result<Self> {
        GlfwWindow::new(|glfw| {
            glfw.create_window(width, height, title, glfw::WindowMode::Windowed)
                .context("unable to create glfw window")
        })
    }

    /// Poll glfw for window events
    pub fn poll_events(&mut self) -> Vec<(f64, glfw::WindowEvent)> {
        self.glfw.poll_events();
        glfw::flush_messages(&self.event_receiver)
            .into_iter()
            .collect()
    }

    /// Build a vulkan-enabled glfw window, using the provided create_window
    /// function.
    fn build_vulkan_window<F>(
        glfw: &mut glfw::Glfw,
        create_window: F,
    ) -> Result<(glfw::Window, EventReceiver)>
    where
        F: FnOnce(&mut glfw::Glfw) -> Result<(glfw::Window, EventReceiver)>,
    {
        if !glfw.vulkan_supported() {
            bail!("vulkan is not supported on this device!");
        }
        glfw.window_hint(glfw::WindowHint::ClientApi(
            glfw::ClientApiHint::NoApi,
        ));
        create_window(glfw)
    }

    /// Create a vulkan surface using the glfw to handle the platform-specific
    /// setup.
    fn create_surface(
        instance: &Instance,
        window: &glfw::Window,
    ) -> Result<vk::SurfaceKHR> {
        let mut surface_handle: u64 = 0;
        let result = window.create_window_surface(
            instance.ash.handle().as_raw() as usize,
            null(),
            &mut surface_handle,
        );
        if result != vk::Result::SUCCESS.as_raw() as u32 {
            bail!("unable to create the vulkan surface");
        }
        Ok(vk::SurfaceKHR::from_raw(surface_handle))
    }
}

impl Drop for GlfwWindow {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}
