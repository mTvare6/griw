use bytemuck::{Pod, Zeroable};

use crate::math::Vec3;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniforms {
    _pad0: u32,
    origin: Vec3,
    _pad1: u32,
    u: Vec3,
    _pad2: u32,
    v: Vec3,
    _pad3: u32,
    w: Vec3,
}

pub struct Camera {
    uniforms: CameraUniforms,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn new(origin: Vec3, center: Vec3, up: Vec3) -> Camera {
        let w = (center - origin).normalized();
        let u = w.cross(&up);
        let v = u.cross(&w);
        Camera {
            uniforms: CameraUniforms {
                origin,
                u,
                v,
                w,
                _pad0: 0,
                _pad1: 0,
                _pad2: 0,
                _pad3: 0,
            },
            yaw: -90.0,
            pitch: 0.0,
        }
    }

    pub fn uniforms(&self) -> &CameraUniforms {
        &self.uniforms
    }

    pub fn zoom(&mut self, displacement: f32) {
        self.uniforms.origin += displacement * self.uniforms.w;
    }

    pub fn move_along_w(&mut self, t: f32) {
        self.uniforms.origin += self.uniforms.w * t;
    }

    pub fn move_along_u(&mut self, t: f32) {
        self.uniforms.origin -= self.uniforms.u * t;
    }

    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw += dx;
        self.pitch += dy;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        let front = Vec3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        );
        self.uniforms.w = front.normalized();
        self.uniforms.u = self.uniforms.w.cross(&Vec3::Y).normalized();
        self.uniforms.v = self.uniforms.u.cross(&self.uniforms.w);
    }
}
