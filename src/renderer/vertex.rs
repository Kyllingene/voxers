use bytemuck::{Pod, Zeroable};

use std::hash;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub normal: [f32; 3],
}

// If we have NaN's, we have bigger issues
impl std::cmp::Eq for Vertex {}
impl hash::Hash for Vertex {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        for c in self
            .position
            .into_iter()
            .chain(self.color)
            .chain(self.normal)
        {
            hasher.write_u32(c.to_bits());
        }
    }
}

pub const VERTEX_SIZE: usize = std::mem::size_of::<Vertex>();

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x3];

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: VERTEX_SIZE as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
