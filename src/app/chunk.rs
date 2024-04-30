use super::block::Block;

pub const CHUNK_SIZE: usize = 32;

#[derive(Clone, Default)]
pub struct Chunk {
    /// [[[x] z] y]
    pub blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    pub state: ChunkState,
}

impl Chunk {
    pub const fn new(blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]) -> Self {
        Chunk {
            blocks,
            state: ChunkState::Remesh,
        }
    }

    pub fn get(&self, [x, y, z]: [usize; 3]) -> Option<Block> {
        self.blocks
            .get(y)
            .and_then(|col| col.get(z))
            .and_then(|row| row.get(x))
            .copied()
    }

    pub fn get_mut(&mut self, [x, y, z]: [usize; 3]) -> Option<&mut Block> {
        self.blocks
            .get_mut(y)
            .and_then(|col| col.get_mut(z))
            .and_then(|row| row.get_mut(x))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    Cached,
    Greedy,
    #[default]
    Remesh,
}
