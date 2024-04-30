use wgpu::util::DeviceExt;

use super::{vertex, Mesh};

pub struct CachedMesh {
    pub(super) vertices: wgpu::Buffer,
    pub(super) indices: wgpu::Buffer,
    pub(super) num_indices: u32,
}

impl CachedMesh {
    pub fn new(mesh: Mesh, device: &mut wgpu::Device) -> Self {
        let (vertices, indices) = mesh.finish();
        let num_indices = indices.len() as u32;

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            vertices,
            indices,
            num_indices,
        }
    }

    pub fn update(&mut self, mesh: Mesh, device: &mut wgpu::Device, queue: &mut wgpu::Queue) {
        let (vertices, indices) = mesh.finish();
        self.num_indices = indices.len() as u32;

        if vertices.len() * vertex::VERTEX_SIZE > self.vertices.size() as usize {
            self.vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        } else {
            let start = self.vertices.as_entire_buffer_binding().offset;
            queue.write_buffer(&self.vertices, start, bytemuck::cast_slice(&vertices));
        }

        if indices.len() * 4 > self.indices.size() as usize {
            self.indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });
        } else {
            let start = self.indices.as_entire_buffer_binding().offset;
            queue.write_buffer(&self.indices, start, bytemuck::cast_slice(&indices));
        }
    }
}
