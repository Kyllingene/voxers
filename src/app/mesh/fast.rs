use std::collections::HashMap;

use crate::app::block::Block;
use crate::app::chunk::{Chunk, CHUNK_SIZE};
use crate::renderer::mesh::{DedupMesh, Mesh};
use crate::renderer::vertex::Vertex;

pub fn fast(chunk: &Chunk, [ax, ay, az]: [i32; 3], chunks: &HashMap<[i32; 3], Chunk>) -> Mesh {
    let mut mesh = DedupMesh::new();

    let chunk_above = chunks
        .get(&[ax, ay + 1, az])
        .map(|neighbor| {
            let plane = neighbor.blocks[0];
            let mut above = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (z, row) in plane.iter().enumerate() {
                for (x, block) in row.iter().enumerate() {
                    above[z][x] = block.is_transparent();
                }
            }
            above
        })
        .unwrap_or([[true; CHUNK_SIZE]; CHUNK_SIZE]);

    let chunk_below = chunks
        .get(&[ax, ay - 1, az])
        .map(|neighbor| {
            let plane = neighbor.blocks[CHUNK_SIZE - 1];
            let mut below = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for (z, row) in plane.iter().enumerate() {
                for (x, block) in row.iter().enumerate() {
                    below[z][x] = block.is_transparent();
                }
            }
            below
        })
        .unwrap_or([[true; CHUNK_SIZE]; CHUNK_SIZE]);

    let chunk_left = chunks
        .get(&[ax - 1, ay, az])
        .map(|neighbor| {
            let mut left = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    left[y][z] = neighbor.blocks[y][z][CHUNK_SIZE - 1].is_transparent();
                }
            }
            left
        })
        .unwrap_or([[true; CHUNK_SIZE]; CHUNK_SIZE]);

    let chunk_right = chunks
        .get(&[ax + 1, ay, az])
        .map(|neighbor| {
            let mut right = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    right[y][z] = neighbor.blocks[y][z][0].is_transparent();
                }
            }
            right
        })
        .unwrap_or([[true; CHUNK_SIZE]; CHUNK_SIZE]);

    let chunk_back = chunks
        .get(&[ax, ay, az - 1])
        .map(|neighbor| {
            let mut front = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    front[x][y] = neighbor.blocks[y][CHUNK_SIZE - 1][x].is_transparent();
                }
            }
            front
        })
        .unwrap_or([[true; CHUNK_SIZE]; CHUNK_SIZE]);

    let chunk_front = chunks
        .get(&[ax, ay, az + 1])
        .map(|neighbor| {
            let mut back = [[false; CHUNK_SIZE]; CHUNK_SIZE];
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    back[x][y] = neighbor.blocks[y][0][x].is_transparent();
                }
            }
            back
        })
        .unwrap_or([[true; CHUNK_SIZE]; CHUNK_SIZE]);

    let pos = [ax, ay, az];

    mesh_face(
        &mut mesh,
        chunk,
        0,
        pos,
        chunk_above,
        [0, 2, 1],
        [0.0, 1.0, 0.0],
        false,
        false,
    );

    mesh_face(
        &mut mesh,
        chunk,
        2,
        pos,
        chunk_below,
        [0, 2, 1],
        [0.0, -1.0, 0.0],
        true,
        true,
    );

    mesh_face(
        &mut mesh,
        chunk,
        1,
        pos,
        chunk_back,
        [0, 1, 2],
        [0.0, 0.0, -1.0],
        true,
        false,
    );

    mesh_face(
        &mut mesh,
        chunk,
        1,
        pos,
        chunk_front,
        [0, 1, 2],
        [0.0, 0.0, 1.0],
        false,
        true,
    );

    mesh_face(
        &mut mesh,
        chunk,
        1,
        pos,
        chunk_left,
        [1, 2, 0],
        [-1.0, 0.0, 0.0],
        true,
        false,
    );

    mesh_face(
        &mut mesh,
        chunk,
        1,
        pos,
        chunk_right,
        [1, 2, 0],
        [1.0, 0.0, 0.0],
        false,
        true,
    );

    mesh.into_mesh()
}

fn mesh_face(
    mesh: &mut DedupMesh,
    chunk: &Chunk,
    voxel_face_idx: usize,
    [xc, yc, zc]: [i32; 3],
    neighbor: [[bool; CHUNK_SIZE]; CHUNK_SIZE],
    [ai, bi, ci]: [usize; 3],
    normal: [f32; 3],
    flip: bool,
    clockwise: bool,
) {
    for a in 0..CHUNK_SIZE {
        for b in 0..CHUNK_SIZE {
            let mut pos = [0; 3];
            pos[ai] = a;
            pos[bi] = b;

            let mut transparent = 0u64;
            transparent |= (flip & neighbor[a][b]) as u64;

            let mut blocks = 0u32;
            for c in 0..CHUNK_SIZE {
                pos[ci] = c;
                let [x, y, z] = pos;
                let block = chunk.blocks[y][z][x];

                blocks <<= 1;
                blocks |= (block != Block::Air) as u32;

                transparent <<= 1;
                transparent |= block.is_transparent() as u64;
            }

            if flip {
                transparent >>= 1;
            } else {
                transparent <<= 1;
                transparent |= neighbor[a][b] as u64;
            }

            blocks &= transparent as u32;

            if blocks == 0 {
                continue;
            }

            let mut c = CHUNK_SIZE - 1;
            while blocks != 0 {
                if blocks & 1 == 0 {
                    blocks >>= 1;
                    c -= 1;
                    continue;
                }

                pos[ci] = c;
                let [x, y, z] = pos;
                let color = chunk.blocks[y][z][x].voxel().faces[voxel_face_idx].color;

                let v = |p, q| {
                    let mut position = [0.0; 3];
                    position[ai] = a as f32 + p as f32;
                    position[bi] = b as f32 + q as f32;
                    position[ci] = c as f32 + !flip as u8 as f32;
                    let [x, y, z] = position;

                    Vertex {
                        position: [
                            x + xc as f32 * CHUNK_SIZE as f32,
                            y + yc as f32 * CHUNK_SIZE as f32,
                            z + zc as f32 * CHUNK_SIZE as f32,
                        ],
                        color,
                        normal,
                    }
                };

                if clockwise {
                    mesh.vertices([v(1, 1), v(0, 1), v(0, 0), v(0, 0), v(1, 0), v(1, 1)]);
                } else {
                    mesh.vertices([v(0, 0), v(0, 1), v(1, 1), v(1, 1), v(1, 0), v(0, 0)]);
                }

                blocks >>= 1;
                c -= 1;
            }
        }
    }
}
