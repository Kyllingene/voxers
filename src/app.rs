use log::{debug, trace};
use winit::{
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    keyboard::{Key, NamedKey},
    window::Window,
};

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::renderer::{cached::CachedMesh, light::LightUniform, Renderer};

mod block;
mod chunk;
mod mesh;
mod player;
mod voxel;
mod worldgen;

use chunk::{Chunk, ChunkState, CHUNK_SIZE};
use mesh::BgMesher;
use player::Player;

pub struct ApplicationState {
    pub renderer: Renderer,
    pub exit: bool,
    pub changed: bool,

    chunks: HashMap<[i32; 3], Chunk>,
    chunk_cache: HashMap<[i32; 3], CachedMesh>,
    bg_mesher: BgMesher,

    player: Player,
}

impl ApplicationState {
    /// Flag a chunk for remeshing.
    ///
    /// Also necessitates remeshing each neighbor.
    pub fn flag(&mut self, [x, y, z]: [i32; 3]) {
        for chunk in [
            [x, y, z],
            [x - 1, y, z],
            [x + 1, y, z],
            [x, y - 1, z],
            [x, y + 1, z],
            [x, y, z - 1],
            [x, y, z + 1],
        ] {
            if let Some(chunk) = self.chunks.get_mut(&chunk) {
                chunk.state = ChunkState::Remesh;
            }
        }
    }

    /// Insert a chunk into the world.
    ///
    /// Queues the chunk for meshing, as well as its neighbors.
    pub fn insert_chunk(&mut self, at: [i32; 3], mut chunk: Chunk) {
        chunk.state = ChunkState::Remesh;
        self.chunks.insert(at, chunk);
        self.flag(at);
    }

    pub async fn new(window: &'static Window) -> Self {
        let sun = LightUniform::new([200.0, 200.0, 200.0], [1.0, 1.0, 1.0]);
        let mut renderer = Renderer::new(window).await;
        renderer.light(sun);

        let chunks = worldgen::gen(0, [0, 0, 0], [5, 2, 5]);

        let size = renderer.size;
        Self {
            renderer,
            exit: false,
            changed: true,

            chunks,
            chunk_cache: HashMap::new(),
            bg_mesher: BgMesher::new(),

            player: Player::new([0.0, 2.0, 0.0], size),
        }
    }

    pub fn draw(&mut self) {
        if let Some((pos, mesh)) = self.bg_mesher.query() {
            let old = self.chunk_cache.get_mut(&pos).unwrap();
            self.renderer.update_cache(old, mesh);
            self.chunks.get_mut(&pos).unwrap().state = ChunkState::Cached;
        }

        if self.changed {
            let start = std::time::Instant::now();

            let mut updated = vec![None; self.chunks.len()];
            let mut updated_chunks = updated.chunks_mut(1).map(|c| &mut c[0]);

            thread::scope(|s| {
                for (&pos, chunk) in &self.chunks {
                    let out = updated_chunks.next().unwrap();

                    match chunk.state {
                        ChunkState::Remesh => {
                            let chunks = &self.chunks;
                            s.spawn(move || {
                                let mesh = mesh::fast(chunk, pos, chunks);
                                *out = Some((pos, mesh));
                            });
                        }
                        ChunkState::Greedy => {
                            if self.bg_mesher.mesh(pos, &self.chunks) {
                                trace!("Greedy meshing chunk {pos:?}");
                            }
                        }
                        ChunkState::Cached => {}
                    }
                }
            });

            let mut num_updated = 0;
            for (pos, mesh) in updated.into_iter().flatten() {
                if mesh.vertices.len() < 15000 {
                    // TODO: keep tabs on this number
                    self.chunks.get_mut(&pos).unwrap().state = ChunkState::Greedy;
                } else {
                    self.chunks.get_mut(&pos).unwrap().state = ChunkState::Cached;
                }

                if let Some(old) = self.chunk_cache.get_mut(&pos) {
                    self.renderer.update_cache(old, mesh);
                } else {
                    let cached = self.renderer.cache(mesh);
                    self.chunk_cache.insert(pos, cached);
                }

                num_updated += 1;
            }

            let end = std::time::Instant::now();
            debug!(
                "Meshed {num_updated} chunks ({} blocks) in {} secs",
                num_updated * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE,
                (end - start).as_secs_f64()
            );

            self.changed = false;
        } else if let Some((pos, _)) = self
            .chunks
            .iter()
            .find(|(_, c)| c.state == ChunkState::Greedy)
        {
            if self.bg_mesher.mesh(*pos, &self.chunks) {
                trace!("Greedy meshing chunk {pos:?}");
            }
        }

        self.renderer.render(self.chunk_cache.values()).unwrap();
    }

    pub fn update(&mut self, dt: Duration) {
        self.player.update(self.renderer.size, dt);
        self.renderer.update_camera(&self.player.camera);
    }

    #[allow(unused)]
    pub fn key_input(&mut self, event: &KeyEvent, dt: Duration) {
        if event.repeat {
            return;
        }

        if !self.player.process_events(event) && event.state == ElementState::Pressed {
            match &event.logical_key {
                Key::Named(NamedKey::Escape) => self.exit = true,
                Key::Character(ch) => match ch.as_str() {
                    "t" | "T" => self.renderer.next_pipeline(),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    #[allow(unused)]
    pub fn mouse_input(&mut self, event: &WindowEvent, dt: Duration) -> bool {
        // TODO: handle mouse input
        matches!(
            event,
            WindowEvent::MouseInput { .. } | WindowEvent::MouseWheel { .. }
        )
    }

    #[allow(unused, clippy::single_match)]
    pub fn mouse_movement(&mut self, event: &DeviceEvent, dt: Duration) {
        if !self.player.process_mouse(event) {
            match event {
                DeviceEvent::MouseMotion { delta: (dx, dy) } => {}
                _ => {}
            }
        }
    }
}
