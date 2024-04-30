use vek::Vec3;
use winit::event::{DeviceEvent, ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};

use std::time::Duration;

use crate::renderer::camera::Camera;

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
    pub fn new(speed: f32, sensitivity: f32) -> Self {
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

    pub fn process_mouse(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                self.rotation_horizontal += *dx as f32;
                self.rotation_vertical += *dy as f32;

                true
            }
            _ => false,
        }
    }

    pub fn process_events(&mut self, event: &KeyEvent) -> bool {
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
