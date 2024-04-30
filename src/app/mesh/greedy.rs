use super::ChunkRegion;
use crate::app::block::Block;
use crate::app::chunk::CHUNK_SIZE;
use crate::renderer::mesh::Mesh;
use crate::renderer::vertex::Vertex;

pub fn greedy([ax, ay, az]: [i32; 3], chunks: ChunkRegion) -> Mesh {
    let mut mesh = Mesh::new();

    let chunk = chunks.center;
    let chunk_above = chunks
        .top
        .map(|neighbor| {
            let plane = neighbor.blocks[0];
            let mut above = [0u32; CHUNK_SIZE];
            for (z, row) in plane.iter().enumerate() {
                for block in row {
                    above[z] <<= 1;
                    above[z] |= block.is_transparent() as u32;
                }
            }
            above
        })
        .unwrap_or([!0; CHUNK_SIZE]);

    let chunk_below = chunks
        .bot
        .map(|neighbor| {
            let plane = neighbor.blocks[CHUNK_SIZE - 1];
            let mut below = [0u32; CHUNK_SIZE];
            for (z, row) in plane.iter().enumerate() {
                for block in row {
                    below[z] <<= 1;
                    below[z] |= block.is_transparent() as u32;
                }
            }
            below
        })
        .unwrap_or([!0; CHUNK_SIZE]);

    let chunk_back = chunks
        .front
        .map(|neighbor| {
            let blocks = &neighbor.blocks;
            let mut front = [0u32; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    front[y] <<= 1;
                    front[y] |= blocks[y][CHUNK_SIZE - 1][x].is_transparent() as u32;
                }
            }
            front
        })
        .unwrap_or([!0; CHUNK_SIZE]);

    let chunk_front = chunks
        .back
        .map(|neighbor| {
            let blocks = &neighbor.blocks;
            let mut back = [0u32; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    back[y] <<= 1;
                    back[y] |= blocks[y][0][x].is_transparent() as u32;
                }
            }
            back
        })
        .unwrap_or([!0; CHUNK_SIZE]);

    let chunk_left = chunks
        .left
        .map(|neighbor| {
            let blocks = &neighbor.blocks;
            let mut left = [0u32; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    left[z] <<= 1;
                    left[z] |= blocks[y][z][CHUNK_SIZE - 1].is_transparent() as u32;
                }
            }
            left
        })
        .unwrap_or([!0; CHUNK_SIZE]);

    let chunk_right = chunks
        .right
        .map(|neighbor| {
            let blocks = &neighbor.blocks;
            let mut right = [0u32; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    right[z] <<= 1;
                    right[z] |= blocks[y][z][0].is_transparent() as u32;
                }
            }
            right
        })
        .unwrap_or([!0; CHUNK_SIZE]);

    for kind in [Block::Water, Block::Grass, Block::Dirt, Block::Stone] {
        let mut above = chunk_above;
        for (y, plane) in chunk.blocks.iter().enumerate().rev() {
            let mut bitmap = [0u32; CHUNK_SIZE];
            let mut transparent = [0u32; CHUNK_SIZE];
            for (z, row) in plane.iter().enumerate() {
                for block in row {
                    bitmap[z] <<= 1;
                    bitmap[z] |= (*block == kind) as u32;

                    transparent[z] <<= 1;
                    transparent[z] |= (block.is_transparent() && !kind.is_transparent()) as u32;
                }
            }

            mesh_face(
                &mut mesh,
                [ax, az, ay],
                bitmap,
                kind,
                0,
                above,
                y as f32 + 1.0,
                [0, 2, 1],
                [0.0, -1.0],
                [0.0, 1.0, 0.0],
                false,
            );

            above = transparent;
        }

        let mut below = chunk_below;
        for (y, plane) in chunk.blocks.iter().enumerate() {
            let mut bitmap = [0u32; CHUNK_SIZE];
            let mut transparent = [0u32; CHUNK_SIZE];
            for (z, row) in plane.iter().enumerate() {
                for block in row {
                    bitmap[z] <<= 1;
                    bitmap[z] |= (*block == kind) as u32;

                    transparent[z] <<= 1;
                    transparent[z] |= (block.is_transparent() && !kind.is_transparent()) as u32;
                }
            }

            mesh_face(
                &mut mesh,
                [ax, az, ay],
                bitmap,
                kind,
                2,
                below,
                y as f32,
                [0, 2, 1],
                [0.0, -1.0],
                [0.0, -1.0, 0.0],
                true,
            );

            below = transparent;
        }

        let blocks = &chunk.blocks;

        let mut back = chunk_back;
        for z in 0..CHUNK_SIZE {
            let mut bitmap = [0u32; CHUNK_SIZE];
            let mut transparent = [0u32; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = blocks[y][z][x];
                    bitmap[y] <<= 1;
                    bitmap[y] |= (block == kind) as u32;

                    transparent[y] <<= 1;
                    transparent[y] |= (block.is_transparent() && !kind.is_transparent()) as u32;
                }
            }

            mesh_face(
                &mut mesh,
                [ax, ay, az],
                bitmap,
                kind,
                1,
                back,
                z as f32 - 1.0,
                [0, 1, 2],
                [0.0, 0.0],
                [0.0, 0.0, -1.0],
                false,
            );

            back = transparent;
        }

        let mut front = chunk_front;
        for z in (0..CHUNK_SIZE).rev() {
            let mut bitmap = [0u32; CHUNK_SIZE];
            let mut transparent = [0u32; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = blocks[y][z][x];
                    bitmap[y] <<= 1;
                    bitmap[y] |= (block == kind) as u32;

                    transparent[y] <<= 1;
                    transparent[y] |= (block.is_transparent() && !kind.is_transparent()) as u32;
                }
            }

            mesh_face(
                &mut mesh,
                [ax, ay, az],
                bitmap,
                kind,
                1,
                front,
                z as f32,
                [0, 1, 2],
                [0.0, 0.0],
                [0.0, 0.0, 1.0],
                true,
            );

            front = transparent;
        }

        let mut left = chunk_left;
        for x in 0..CHUNK_SIZE {
            let mut bitmap = [0u32; CHUNK_SIZE];
            let mut transparent = [0u32; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block = blocks[y][z][x];
                    bitmap[z] <<= 1;
                    bitmap[z] |= (block == kind) as u32;

                    transparent[z] <<= 1;
                    transparent[z] |= (block.is_transparent() && !kind.is_transparent()) as u32;
                }
            }

            mesh_face(
                &mut mesh,
                [ay, az, ax],
                bitmap,
                kind,
                1,
                left,
                x as f32,
                [1, 2, 0],
                [0.0, -1.0],
                [-1.0, 0.0, 0.0],
                false,
            );

            left = transparent;
        }

        let mut right = chunk_right;
        for xx in (0..CHUNK_SIZE).rev() {
            let mut bitmap = [0u32; CHUNK_SIZE];
            let mut transparent = [0u32; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let block = blocks[y][z][xx];
                    bitmap[z] <<= 1;
                    bitmap[z] |= (block == kind) as u32;

                    transparent[z] <<= 1;
                    transparent[z] |= (block.is_transparent() && !kind.is_transparent()) as u32;
                }
            }

            mesh_face(
                &mut mesh,
                [ay, az, ax],
                bitmap,
                kind,
                1,
                right,
                xx as f32 + 1.0,
                [1, 2, 0],
                [0.0, -1.0],
                [1.0, 0.0, 0.0],
                true,
            );

            right = transparent;
        }
    }

    mesh
}

fn mesh_face(
    mesh: &mut Mesh,
    at: [i32; 3],
    mut bitmap: [u32; CHUNK_SIZE],
    kind: Block,
    voxel_face_idx: usize,
    neighbor: [u32; CHUNK_SIZE],
    c: f32,
    [ai, bi, ci]: [usize; 3],
    [ao, bo]: [f32; 2],
    normal: [f32; 3],
    clockwise: bool,
) {
    let mut idx = 0;
    while idx < CHUNK_SIZE {
        // mask out any obscured faces
        let row = bitmap[idx] & neighbor[idx];

        // if this row is complete, move on to the next one
        if row == 0 {
            idx += 1;
            continue;
        }

        // determine the start and width of the quad
        let start = row.leading_zeros();
        let width = (row << start).leading_ones();

        let mask = (!0 << (CHUNK_SIZE as u32 - width)) >> start;

        // determine the end (height) of the quad
        let end = bitmap[idx + 1..]
            .iter()
            // we don't want to generate meshing for obscured faces, but we
            // also don't want to overcomplicate the meshes by being
            // pedantic; thus, `row` is not masked with `above`
            .position(|row| row & mask != mask)
            // + idx + 1: because we're starting from idx + 1
            .map(|end| end + idx + 1)
            .unwrap_or(CHUNK_SIZE);

        // create the quad
        let color = kind.voxel().faces[voxel_face_idx].color;
        let v = |a: u32, b: u32| {
            let mut position = [0.0; 3];
            position[ai] = a as f32 + ao + at[0] as f32 * CHUNK_SIZE as f32;
            position[bi] = b as f32 + bo + at[1] as f32 * CHUNK_SIZE as f32;
            position[ci] = c + at[2] as f32 * CHUNK_SIZE as f32;
            position[2] += 1.0;
            Vertex {
                position,
                color,
                normal,
            }
        };

        let depth = end as u32 - idx as u32;
        let a = start;
        let b = idx as u32;

        if clockwise {
            mesh.vertices([
                v(a + width, b + depth),
                v(a, b + depth),
                v(a, b),
                v(a, b),
                v(a + width, b),
                v(a + width, b + depth),
            ]);
        } else {
            mesh.vertices([
                v(a, b),
                v(a, b + depth),
                v(a + width, b + depth),
                v(a + width, b + depth),
                v(a + width, b),
                v(a, b),
            ]);
        }

        // clear the bits we already covered
        for row in &mut bitmap[idx..end] {
            *row ^= mask;
        }
    }
}
