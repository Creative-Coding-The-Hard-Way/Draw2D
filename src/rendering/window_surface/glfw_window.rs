use crate::rendering::Instance;

use anyhow::{bail, Context, Result};
use ash::{extensions::khr::Surface, version::InstanceV1_0, vk, vk::Handle};
use std::{
    cell::RefCell,
    ptr::null,
    sync::{mpsc::Receiver, Arc},
};

pub type EventReceiver = Receiver<(f64, glfw::WindowEvent)>;

/// Resources required for rendering to a single GLFW window.
pub struct GlfwWindow {
    /// The raw vulkan surface handle
    surface: vk::SurfaceKHR,

    /// Extension functions for interacting with the surface
    surface_loader: Surface,

    /// The glfw library instance
    pub glfw: RefCell<glfw::Glfw>,

    /// The glfw window
    pub window: RefCell<glfw::Window>,

    /// The event reciever. Usually consumed by the application's main loop.
    pub event_receiver: RefCell<Option<EventReceiver>>,

    /// The instance must not be destroyed before the WindowSurface
    pub instance: Arc<Instance>,
}

impl GlfwWindow {
    /// Create a new application window and vulkan surface.
    ///
    /// It's safe to clone the the resulting window, but it is not safe to use
    /// glfw window functions from any thread but the main thread. (the thread
    /// where this `new` function was invoked).
    ///
    /// # Example
    ///
    /// ```
    /// GlfwWindow::new(|&mut glfw| {
    ///     let (mut window, event_receiver) = glfw
    ///         .create_window(
    ///             1366,
    ///             768,
    ///             "My GLFW Window",
    ///             glfw::WindowMode::Windowed,
    ///         )
    ///         .context("unable to create the glfw window")?;
    ///     Ok((window, event_receiver))
    /// })?;
    /// ```
    pub fn new<F>(create_window: F) -> Result<Arc<Self>>
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

        Ok(Arc::new(Self {
            surface,
            surface_loader,

            glfw: RefCell::new(glfw),
            window: RefCell::new(window),
            event_receiver: RefCell::new(Some(event_receiver)),

            instance,
        }))
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

impl super::WindowSurface for GlfwWindow {
    /// clone the instance created by this window surface
    fn clone_vulkan_instance(&self) -> Arc<Instance> {
        self.instance.clone()
    }

    /// Yield the window's current framebuffer size.
    ///
    /// The size is in physical pixels and is meant to be used directly in the
    /// swapchain extent.
    fn framebuffer_size(&self) -> (u32, u32) {
        let (iwidth, iheight) = self.window.borrow().get_framebuffer_size();
        (iwidth as u32, iheight as u32)
    }

    /// Get the raw surface handle.
    ///
    /// Unsafe because the the WindowSurface itself is responsible for the
    /// lifetime of the real surface object. The caller should not retain this
    /// handle except for the creation of other vulkan resources which will
    /// not outlive the window surface.
    unsafe fn get_surface_handle(&self) -> vk::SurfaceKHR {
        self.surface
    }

    /// Check that a physical device supports rendering to this surface.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying for queue presentation support.
    ///
    //                )
    unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: &vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool> {
        self.surface_loader
            .get_physical_device_surface_support(
                *physical_device,
                queue_family_index,
                self.surface,
            )
            .context("unable to check for queue family support!")
    }

    /// Returns the set of all supported formats for this device.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the surface formats.
    unsafe fn supported_formats(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::SurfaceFormatKHR> {
        self.surface_loader
            .get_physical_device_surface_formats(*physical_device, self.surface)
            .unwrap_or_else(|_| vec![])
    }

    /// Returns the set of all supported presentation modes for this device.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the presentation modes.
    unsafe fn supported_presentation_modes(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Vec<vk::PresentModeKHR> {
        self.surface_loader
            .get_physical_device_surface_present_modes(
                *physical_device,
                self.surface,
            )
            .unwrap_or_else(|_| vec![])
    }

    /// Returns the set of all supported surface capabilities.
    ///
    /// Unsafe because the device's supported extensions must be checked prior
    /// to querying the surface capabilities.
    unsafe fn surface_capabilities(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR> {
        self.surface_loader
            .get_physical_device_surface_capabilities(
                *physical_device,
                self.surface,
            )
            .context("unable to get surface capabiliities for this device")
    }
}

impl Drop for GlfwWindow {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}
