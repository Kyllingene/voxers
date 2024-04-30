use log::{debug, trace};
use vek::Vec3;
use winit::{
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    keyboard::{Key, NamedKey},
    window::Window,
};

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crate::renderer::{cached::CachedMesh, camera::Camera, light::LightUniform, Renderer};

pub mod block;
pub mod chunk;
mod mesh;
pub mod voxel;

use chunk::{Chunk, ChunkState, CHUNK_SIZE};
use mesh::BgMesher;

pub struct ApplicationState {
    pub renderer: Renderer,
    pub exit: bool,
    pub changed: bool,

    chunks: HashMap<[i32; 3], Chunk>,
    chunk_cache: HashMap<[i32; 3], CachedMesh>,
    bg_mesher: BgMesher,
    next: [i32; 3],

    // sun: LightUniform,
    controller: CameraController,
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

        let mut chunks = HashMap::new();

        let mut rng = rand::thread_rng();
        use rand::Rng;

        let r = 6i32;
        for y in -3..0 {
            for z in -r..r {
                for x in -r..r {
                    if rng.gen() {
                        chunks.insert([x, y, z], chunk::base_chunk());
                    } else {
                        chunks.insert([x, y, z], chunk::random_chunk());
                    }
                }
            }
        }
        debug!("Generated {} chunks", (r * 2).pow(2) * 3);

        Self {
            renderer,
            exit: false,
            changed: true,

            chunks,
            chunk_cache: HashMap::new(),
            bg_mesher: BgMesher::new(),
            next: [r, 0, 0],

            controller: CameraController::new(30.0, 200.0),
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
        } else {
            for (pos, chunk) in &self.chunks {
                if chunk.state == ChunkState::Greedy {
                    if self.bg_mesher.mesh(*pos, &self.chunks) {
                        trace!("Greedy meshing chunk {pos:?}");
                    }
                    break;
                }
            }
        }

        self.renderer.render(self.chunk_cache.values()).unwrap();
    }

    pub fn update(&mut self, dt: Duration) {
        self.renderer.update_camera(&mut self.controller, dt);
    }

    #[allow(unused)]
    pub fn key_input(&mut self, event: &KeyEvent, dt: Duration) {
        if event.repeat {
            return;
        }

        if !self.controller.process_events(event) && event.state == ElementState::Pressed {
            match &event.logical_key {
                Key::Named(NamedKey::Escape) => self.exit = true,
                Key::Character(ch) => match ch.as_str() {
                    "t" | "T" => self.renderer.next_pipeline(),
                    "n" | "N" => {
                        self.chunks.insert(self.next, chunk::base_chunk());
                        self.next[0] += 1;
                        self.changed = true;
                    }
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
        if !self.controller.process_mouse(event) {
            match event {
                DeviceEvent::MouseMotion { delta: (dx, dy) } => {}
                _ => {}
            }
        }
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f32,

    rotation_horizontal: f32,
    rotation_vertical: f32,

    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_space_pressed: bool,
    is_shift_pressed: bool,
}

impl CameraController {
    fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,

            rotation_horizontal: 0.0,
            rotation_vertical: 0.0,

            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_space_pressed: false,
            is_shift_pressed: false,
        }
    }

    fn process_mouse(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                self.rotation_horizontal += *dx as f32;
                self.rotation_vertical += *dy as f32;

                true
            }
            _ => false,
        }
    }

    fn process_events(&mut self, event: &KeyEvent) -> bool {
        let is_pressed = event.state == ElementState::Pressed;
        match &event.logical_key {
            Key::Named(NamedKey::ArrowUp) => {
                self.is_forward_pressed = is_pressed;
                true
            }
            Key::Named(NamedKey::ArrowLeft) => {
                self.is_left_pressed = is_pressed;
                true
            }
            Key::Named(NamedKey::ArrowDown) => {
                self.is_backward_pressed = is_pressed;
                true
            }
            Key::Named(NamedKey::ArrowRight) => {
                self.is_right_pressed = is_pressed;
                true
            }
            Key::Named(NamedKey::Space) => {
                self.is_space_pressed = is_pressed;
                true
            }
            Key::Named(NamedKey::Shift) => {
                self.is_shift_pressed = is_pressed;
                true
            }
            Key::Named(NamedKey::Control) => {
                if is_pressed {
                    self.speed *= 2.0;
                } else {
                    self.speed /= 2.0;
                }
                true
            }
            Key::Character(ch) => match ch.as_str() {
                "w" | "W" => {
                    self.is_forward_pressed = is_pressed;
                    true
                }
                "a" | "A" => {
                    self.is_left_pressed = is_pressed;
                    true
                }
                "s" | "S" => {
                    self.is_backward_pressed = is_pressed;
                    true
                }
                "d" | "D" => {
                    self.is_right_pressed = is_pressed;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let speed = self.speed * dt.as_secs_f32();
        let sens = self.sensitivity * dt.as_secs_f32();

        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalized();

        let up = Vec3::unit_y();
        let right = -forward_norm.cross(camera.up);

        if self.is_forward_pressed {
            camera.eye += forward_norm * speed;
            camera.target += forward_norm * speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * speed;
            camera.target -= forward_norm * speed;
        }
        if self.is_left_pressed {
            camera.eye += right * speed;
            camera.target += right * speed;
        }
        if self.is_right_pressed {
            camera.eye -= right * speed;
            camera.target -= right * speed;
        }
        if self.is_space_pressed {
            let dir = Vec3::unit_y() * speed;
            camera.eye += dir;
            camera.target += dir;
        }
        if self.is_shift_pressed {
            let dir = Vec3::unit_y() * speed;
            camera.eye -= dir;
            camera.target -= dir;
        }

        camera.target = camera.eye
            - ((-forward + right * (self.rotation_horizontal * sens))
                + (-forward + up * (self.rotation_vertical * sens)))
                .normalized();

        self.rotation_horizontal = 0.0;
        self.rotation_vertical = 0.0;
    }
}
