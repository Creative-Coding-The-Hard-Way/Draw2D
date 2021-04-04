mod cpu_buffer;
mod static_buffer;

pub mod transfer;

pub use self::{cpu_buffer::CpuBuffer, static_buffer::StaticBuffer};

use ash::vk;

pub trait Buffer {
    /// The raw vulkan handle for the buffer. Should not be copied because
    /// implementations are allowed to invalidate the raw buffer value.
    ///
    /// Unsafe because it is the responsibility of the caller to ensure that
    /// the handle will live for the duration of it's usage. (by checking the
    /// specific implementation)
    unsafe fn raw(&self) -> vk::Buffer;

    /// The raw vulkan handle to the underlying buffer memory. Should not be
    /// copied because implementations are allowed to invalidate the raw buffer
    /// value.
    ///
    /// Unsafe because it is the responsibility of the caller to ensure that
    /// the handle will live for the duration of it's usage. (by checking the
    /// specific implementation)
    unsafe fn memory(&self) -> vk::DeviceMemory;

    /// The size of the underlying gpu memory in bytes.
    fn size_in_bytes(&self) -> u64;
}
