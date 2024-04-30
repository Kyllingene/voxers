use ahash::HashMapExt;
use fxhash::FxHashMap;

use crate::renderer::Vertex;

#[derive(Debug, Default, Clone)]
pub struct Mesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}

impl Mesh {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn vertex(&mut self, vertex: Vertex) -> u32 {
        let i = self.vertices.len() as u32;
        self.vertices.push(vertex);
        i
    }

    pub(crate) fn vertices(&mut self, vertices: impl IntoIterator<Item = Vertex>) {
        let vertices = vertices.into_iter();

        let (min, max) = vertices.size_hint();
        self.indices.reserve(max.unwrap_or(min));
        for vertex in vertices {
            let i = self.vertex(vertex);
            self.indices.push(i);
        }
    }

    pub(crate) fn finish(self) -> (Vec<Vertex>, Vec<u32>) {
        (self.vertices, self.indices)
    }
}

#[derive(Debug, Default, Clone)]
pub struct DedupMesh {
    pub(crate) vertices: FxHashMap<Vertex, u32>,
    pub(crate) indices: Vec<u32>,
    next: u32,
}

impl DedupMesh {
    pub(crate) fn new() -> Self {
        Self {
            vertices: FxHashMap::with_capacity(10_000),
            indices: Vec::with_capacity(5000),
            next: 0,
        }
    }

    pub(crate) fn vertex(&mut self, vertex: Vertex) -> u32 {
        *self.vertices.entry(vertex).or_insert_with(|| {
            let id = self.next;
            self.next += 1;
            id
        })
    }

    pub(crate) fn vertices(&mut self, vertices: impl IntoIterator<Item = Vertex>) {
        let vertices = vertices.into_iter();

        let (min, max) = vertices.size_hint();
        self.indices.reserve(max.unwrap_or(min));
        for vertex in vertices {
            let i = self.vertex(vertex);
            self.indices.push(i);
        }
    }

    pub(crate) fn into_mesh(self) -> Mesh {
        let mut vertices = vec![Vertex::default(); self.vertices.len()];

        for (vertex, idx) in self.vertices {
            vertices[idx as usize] = vertex;
        }

        Mesh {
            vertices,
            indices: self.indices,
        }
    }
}
