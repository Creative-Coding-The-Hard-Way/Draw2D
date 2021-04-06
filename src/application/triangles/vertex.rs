use ash::vk;
use memoffset::offset_of;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    /// Create a new vertex
    pub fn new(pos: [f32; 2], color: [f32; 4]) -> Self {
        Self { pos, color }
    }

    /// Build a binding description for this vertex type.
    pub fn binding_description() -> (
        Vec<vk::VertexInputBindingDescription>,
        Vec<vk::VertexInputAttributeDescription>,
    ) {
        let binding = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(Vertex, pos) as u32)
            .build();
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(offset_of!(Vertex, color) as u32)
            .build();
        (vec![binding], vec![pos, color])
    }
}
