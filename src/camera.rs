use std::f64::consts::PI;

use crate::math::Vec3;
use crate::ray::Ray;

pub struct Camera {
    pub origin: Vec3,
    pub target: Vec3,
    pub fov_degrees: f64,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    viewport_height: f64,
    viewport_width: f64,
}

impl Camera {
    pub fn new(origin: Vec3, target: Vec3, fov_degrees: f64, aspect_ratio: f64) -> Self {
        let forward = (target - origin).normalize();
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward).normalize();
        let theta = fov_degrees * PI / 180.0;
        let viewport_height = 2.0 * (theta / 2.0).tan();
        let viewport_width = viewport_height * aspect_ratio;
        Self {
            origin,
            target,
            fov_degrees,
            forward,
            right,
            up,
            viewport_height,
            viewport_width,
        }
    }

    pub fn ray(&self, u: f64, v: f64) -> Ray {
        let horizontal = self.right * self.viewport_width;
        let vertical = self.up * self.viewport_height;
        let lower_left = self.origin + self.forward - horizontal / 2.0 - vertical / 2.0;
        let direction = (lower_left + horizontal * u + vertical * v - self.origin).normalize();
        Ray {
            origin: self.origin,
            direction,
        }
    }
}
