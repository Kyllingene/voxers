use fastnoise_lite::*;
use log::debug;

use std::collections::HashMap;
use std::time::Instant;

use super::block::Block;
use super::chunk::{Chunk, CHUNK_SIZE};

const DIRT_HEIGHT: usize = 3;
const STONE_HEIGHT: usize = CHUNK_SIZE - DIRT_HEIGHT - 1;

// TODO: figure out chunk variations
// TODO: figure out vertical terrain gen
pub fn gen(
    seed: i32,
    [center_x, center_y, center_z]: [i32; 3],
    [width, _height, depth]: [usize; 3],
) -> HashMap<[i32; 3], Chunk> {
    let start = Instant::now();
    let mut chunks = HashMap::new();

    let mut noise = FastNoiseLite::with_seed(seed);
    noise.set_noise_type(Some(NoiseType::Perlin));

    for z in 0..depth as i32 {
        for x in 0..width as i32 {
            let pos = [center_x + x, center_y, center_z + z];
            chunks.insert(pos, gen_chunk(pos, &mut noise));
        }
    }

    let end = Instant::now();
    debug!(
        "Generated {} chunks in {} seconds",
        width * depth,
        (end - start).as_secs_f32()
    );

    chunks
}

fn gen_chunk([chunk_x, _chunk_y, chunk_z]: [i32; 3], noise: &mut FastNoiseLite) -> Chunk {
    let mut chunk = Chunk::default();

    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let sample = noise.get_noise_2d(
                (x + chunk_x as usize * CHUNK_SIZE) as f32,
                (z + chunk_z as usize * CHUNK_SIZE) as f32,
            );
            let height = ((sample + 1.0) / 2.0) % 1.0;
            let height = (height * CHUNK_SIZE as f32).floor() as usize;

            let stone_height = STONE_HEIGHT.saturating_sub(CHUNK_SIZE - height);
            for y in 0..stone_height {
                chunk.blocks[y][z][x] = Block::Stone;
            }

            for y in stone_height..height {
                chunk.blocks[y][z][x] = Block::Dirt;
            }

            chunk.blocks[height][z][x] = Block::Grass;
        }
    }

    chunk
}
