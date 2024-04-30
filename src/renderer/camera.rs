use bytemuck::{Pod, Zeroable};
use vek::{Mat4, Vec3};

pub struct Camera {
    pub eye: Vec3<f32>,
    pub target: Vec3<f32>,
    pub up: Vec3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(pos: [f32; 3], aspect: f32) -> Self {
        let eye = pos.into();
        Self {
            eye,
            target: eye + vek::Vec3::new(0.0, 0.0, -1.0),
            up: vek::Vec3::unit_y(),
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4<f32> {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh_zo(self.fovy, self.aspect, self.znear, self.zfar);

        opengl_to_wgpu_matrix() * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::identity().into_col_arrays(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into_col_arrays();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

#[rustfmt::skip]
pub fn opengl_to_wgpu_matrix() -> Mat4<f32> {
    Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    )
}
