use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, KeyEvent};

use std::time::Duration;

use crate::renderer::camera::Camera;

mod controller;
use controller::CameraController;

pub struct Player {
    pub position: [f32; 3],
    pub camera: Camera,
    pub controller: CameraController,
}

impl Player {
    pub fn new([x, y, z]: [f32; 3], size: PhysicalSize<u32>) -> Self {
        let aspect = size.width as f32 / size.height as f32;
        Self {
            position: [x, y, z],
            camera: Camera::new([0.0, 10.0, 0.0], aspect),
            controller: CameraController::new(25.0, 120.0),
        }
    }

    pub fn update(&mut self, size: PhysicalSize<u32>, dt: Duration) {
        self.controller.update_camera(&mut self.camera, dt);
        self.camera.aspect = size.width as f32 / size.height as f32;
    }

    pub fn process_events(&mut self, ev: &KeyEvent) -> bool {
        self.controller.process_events(ev)
    }

    pub fn process_mouse(&mut self, ev: &DeviceEvent) -> bool {
        self.controller.process_mouse(ev)
    }
}
