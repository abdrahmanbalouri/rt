use crate::camera::Camera;
use crate::math::Vec3;
use crate::object::{Cube, Cylinder, Hittable, Material, Plane, Sphere};

pub struct Light {
    pub position: Vec3,
    pub intensity: f64,
}

pub struct Scene {
    pub camera: Camera,
    pub light: Light,
    pub ambient: f64,
    pub sky_top: Vec3,
    pub sky_bottom: Vec3,
    pub objects: Vec<Box<dyn Hittable>>,
}

pub fn build_scene(name: &str, brightness: f64) -> Scene {
    match name {
        "plane-cube" => scene_plane_cube(brightness * 0.7),
        "all-objects" => scene_all_objects(brightness, false),
        "all-objects-alt" => scene_all_objects(brightness, true),
        _ => scene_sphere(brightness),
    }
}

fn scene_sphere(brightness: f64) -> Scene {
    Scene {
        camera: Camera::new(Vec3::new(0.0, 1.0, -5.0), Vec3::new(0.0, 0.7, 0.0), 50.0, 4.0 / 3.0),
        light: Light {
            position: Vec3::new(-5.0, 8.0, -6.0),
            intensity: brightness,
        },
        ambient: 0.12,
        sky_top: Vec3::new(0.70, 0.82, 1.0),
        sky_bottom: Vec3::new(0.98, 0.98, 1.0),
        objects: vec![
            Box::new(Sphere {
                center: Vec3::new(0.0, 1.0, 0.5),
                radius: 1.0,
                material: Material {
                    color: Vec3::new(0.82, 0.22, 0.18),
                    reflectivity: 0.25,
                },
            }),
            Box::new(Plane {
                point: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                material: Material {
                    color: Vec3::new(0.85, 0.85, 0.88),
                    reflectivity: 0.05,
                },
            }),
        ],
    }
}

fn scene_plane_cube(brightness: f64) -> Scene {
    Scene {
        camera: Camera::new(Vec3::new(-1.8, 1.8, -6.0), Vec3::new(0.0, 1.0, 0.5), 48.0, 4.0 / 3.0),
        light: Light {
            position: Vec3::new(5.0, 7.0, -4.0),
            intensity: brightness,
        },
        ambient: 0.08,
        sky_top: Vec3::new(0.54, 0.60, 0.72),
        sky_bottom: Vec3::new(0.92, 0.93, 0.96),
        objects: vec![
            Box::new(Plane {
                point: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                material: Material {
                    color: Vec3::new(0.68, 0.70, 0.73),
                    reflectivity: 0.03,
                },
            }),
            Box::new(Cube {
                min: Vec3::new(-1.0, 0.0, -0.1),
                max: Vec3::new(1.0, 2.0, 1.9),
                material: Material {
                    color: Vec3::new(0.18, 0.44, 0.78),
                    reflectivity: 0.1,
                },
            }),
        ],
    }
}

fn scene_all_objects(brightness: f64, alternate_camera: bool) -> Scene {
    let (origin, target) = if alternate_camera {
        (Vec3::new(4.8, 2.6, -5.5), Vec3::new(0.2, 1.1, 1.6))
    } else {
        (Vec3::new(-4.5, 2.5, -6.5), Vec3::new(0.3, 1.0, 1.3))
    };

    Scene {
        camera: Camera::new(origin, target, 46.0, 4.0 / 3.0),
        light: Light {
            position: Vec3::new(2.0, 8.0, -3.0),
            intensity: brightness,
        },
        ambient: 0.1,
        sky_top: Vec3::new(0.60, 0.74, 0.95),
        sky_bottom: Vec3::new(0.97, 0.97, 1.0),
        objects: vec![
            Box::new(Plane {
                point: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                material: Material {
                    color: Vec3::new(0.84, 0.84, 0.86),
                    reflectivity: 0.04,
                },
            }),
            Box::new(Sphere {
                center: Vec3::new(-2.0, 1.0, 2.6),
                radius: 1.0,
                material: Material {
                    color: Vec3::new(0.84, 0.30, 0.24),
                    reflectivity: 0.35,
                },
            }),
            Box::new(Cube {
                min: Vec3::new(-0.3, 0.0, 0.2),
                max: Vec3::new(1.7, 2.0, 2.2),
                material: Material {
                    color: Vec3::new(0.20, 0.62, 0.56),
                    reflectivity: 0.12,
                },
            }),
            Box::new(Cylinder {
                center: Vec3::new(3.0, 0.0, 2.3),
                radius: 0.8,
                min_y: 0.0,
                max_y: 2.4,
                material: Material {
                    color: Vec3::new(0.84, 0.74, 0.22),
                    reflectivity: 0.18,
                },
            }),
        ],
    }
}
