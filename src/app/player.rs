use vek::Vec3;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta};
use winit::keyboard::{Key, NamedKey};

use std::time::Duration;

use crate::renderer::camera::Camera;

pub struct Player {
    pub position: [f32; 3],
    pub camera: Camera,
    pub controller: CameraController,
}

impl Player {
    pub fn new([x, y, z]: [f32; 3]) -> Self {
        Self {
            position: [x, y, z],
            camera: Camera::new([0.0, 10.0, 0.0], 0.0, 0.0),
            controller: CameraController::new(4.0, 4.0),
        }
    }

    pub fn handle(&mut self, ev: &KeyEvent) -> bool {
        self.controller.handle(ev)
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.controller.process_mouse(mouse_dx, mouse_dy)
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.controller.process_scroll(delta)
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn handle(&mut self, ev: &KeyEvent) -> bool {
        let amount = if ev.state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        match &ev.logical_key {
            Key::Character(ch) => match ch.as_str() {
                "w" | "W" => {
                    self.amount_forward = amount;
                    true
                }
                "s" | "S" => {
                    self.amount_backward = amount;
                    true
                }
                "a" | "A" => {
                    self.amount_left = amount;
                    true
                }
                "d" | "D" => {
                    self.amount_right = amount;
                    true
                }
                _ => false,
            },
            Key::Named(NamedKey::Space) => {
                self.amount_up = amount;
                true
            }
            Key::Named(NamedKey::Shift) => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalized();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalized();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scrollward =
            Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalized();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch += -self.rotate_vertical * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }
    }
}

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;
