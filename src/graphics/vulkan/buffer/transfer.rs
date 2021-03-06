use super::Buffer;
use crate::graphics::vulkan::Device;

use anyhow::Result;
use ash::{version::DeviceV1_0, vk};

/// Write commands to copy the full source buffer to the destination buffer.
///
/// Unsafe because this method does not check the destination's size. It is
/// the responsibility of the application to ensure the destination buffer is
/// at least as large as the source buffer.
#[allow(dead_code)]
pub unsafe fn copy_full_buffer<Source, Destination>(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    src: &Source,
    dst: &Destination,
) -> Result<vk::CommandBuffer>
where
    Source: Buffer,
    Destination: Buffer,
{
    let begin_info = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };
    device
        .logical_device
        .begin_command_buffer(command_buffer, &begin_info)?;

    device.logical_device.cmd_copy_buffer(
        command_buffer,
        src.raw(),
        dst.raw(),
        &[vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: src.size_in_bytes(),
        }],
    );

    device.logical_device.end_command_buffer(command_buffer)?;

    Ok(command_buffer)
}
