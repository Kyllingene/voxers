use super::block::Block;

pub const CHUNK_SIZE: usize = 32;

#[derive(Clone)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    Cached,
    Greedy,
    Remesh,
}

pub fn random_chunk() -> Chunk {
    use rand::seq::SliceRandom;

    let mut blocks = [[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
    let mut rng = rand::thread_rng();

    let kinds = [Block::Air, Block::Grass, Block::Dirt, Block::Stone];
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                blocks[y][z][x] = *kinds.choose(&mut rng).unwrap();
            }
        }
    }

    Chunk::new(blocks)
}

#[allow(clippy::needless_range_loop)]
pub fn base_chunk() -> Chunk {
    let mut blocks = [[[Block::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

    blocks[CHUNK_SIZE - 1] = [[Block::Grass; CHUNK_SIZE]; CHUNK_SIZE];
    blocks[CHUNK_SIZE - 1][0][0] = Block::Air;
    blocks[CHUNK_SIZE - 1][0][CHUNK_SIZE - 1] = Block::Air;
    blocks[CHUNK_SIZE - 1][CHUNK_SIZE - 1][0] = Block::Air;
    blocks[CHUNK_SIZE - 1][CHUNK_SIZE - 1][CHUNK_SIZE - 1] = Block::Air;

    for i in 2..6 {
        blocks[CHUNK_SIZE - i] = [[Block::Dirt; CHUNK_SIZE]; CHUNK_SIZE];
    }

    for i in 6..=CHUNK_SIZE {
        blocks[CHUNK_SIZE - i] = [[Block::Stone; CHUNK_SIZE]; CHUNK_SIZE];
    }

    for i in (20..=CHUNK_SIZE).step_by(4) {
        blocks[CHUNK_SIZE - i] = [[Block::Air; CHUNK_SIZE]; CHUNK_SIZE];
    }

    blocks[0] = [[Block::Stone; CHUNK_SIZE]; CHUNK_SIZE];

    Chunk::new(blocks)
}

pub const WATER_CHUNK: Chunk = Chunk::new([[[Block::Water; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
