use ash::vk;
use memoffset::offset_of;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex2d {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub rgba: [f32; 4],
}

impl Default for Vertex2d {
    /// A complete vertex, colored white.
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0],
            uv: [0.0, 0.0],
            rgba: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl Vertex2d {
    /// Build a binding description for this vertex type.
    pub fn binding_description() -> (
        Vec<vk::VertexInputBindingDescription>,
        Vec<vk::VertexInputAttributeDescription>,
    ) {
        let binding = vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        };
        let pos = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: offset_of!(Vertex2d, pos) as u32,
        };
        let uv = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32_SFLOAT,
            offset: offset_of!(Vertex2d, uv) as u32,
        };
        let rgba = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Vertex2d, rgba) as u32,
        };
        (vec![binding], vec![pos, uv, rgba])
    }
}
