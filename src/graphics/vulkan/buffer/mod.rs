mod cpu_buffer;
mod static_buffer;
mod transfer;

pub use self::{
    cpu_buffer::CpuBuffer, static_buffer::StaticBuffer,
    transfer::copy_full_buffer,
};

use ash::vk;

use super::device_allocator::Allocation;

pub trait Buffer {
    /// The raw vulkan handle for the buffer. Should not be copied because
    /// implementations are allowed to invalidate the raw buffer value.
    ///
    /// Unsafe because it is the responsibility of the caller to ensure that
    /// the handle will live for the duration of it's usage. (by checking the
    /// specific implementation)
    unsafe fn raw(&self) -> vk::Buffer;

    /// The raw handle to the buffer's underlying memory allocation.
    unsafe fn allocation(&self) -> &Allocation;

    /// The size of the underlying gpu memory in bytes.
    fn size_in_bytes(&self) -> u64;
}
