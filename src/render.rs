use crate::camera::Camera;
use crate::config::Config;
use crate::math::Vec3;
use crate::object::{Hit, EPSILON};
use crate::ray::Ray;
use crate::scene::Scene;

const MAX_BOUNCES: usize = 3;

pub fn render(scene: &Scene, config: &Config) -> Vec<Vec3> {
    let mut pixels = Vec::with_capacity(config.width * config.height);
    let aspect_ratio = config.width as f64 / config.height as f64;
    let camera = Camera::new(
        scene.camera.origin,
        scene.camera.target,
        scene.camera.fov_degrees,
        aspect_ratio,
    );

    for y in 0..config.height {
        for x in 0..config.width {
            let u = (x as f64 + 0.5) / config.width as f64;
            let v = 1.0 - (y as f64 + 0.5) / config.height as f64;
            let ray = camera.ray(u, v);
            pixels.push(trace(scene, ray, 0).clamp(0.0, 1.0));
        }
    }

    pixels
}

fn trace(scene: &Scene, ray: Ray, depth: usize) -> Vec3 {
    if depth > MAX_BOUNCES {
        return sky(scene, ray.direction);
    }

    let mut closest: Option<Hit> = None;
    for object in &scene.objects {
        if let Some(hit) = object.intersect(ray) {
            if closest
                .map(|current| hit.distance < current.distance)
                .unwrap_or(true)
            {
                closest = Some(hit);
            }
        }
    }

    let Some(hit) = closest else {
        return sky(scene, ray.direction);
    };

    shade(scene, ray, hit, depth)
}

fn shade(scene: &Scene, ray: Ray, hit: Hit, depth: usize) -> Vec3 {
    let light_dir = (scene.light.position - hit.point).normalize();
    let light_distance = (scene.light.position - hit.point).length();
    let shadow_ray = Ray {
        origin: hit.point + hit.normal * EPSILON * 10.0,
        direction: light_dir,
    };

    let in_shadow = scene.objects.iter().any(|object| {
        object
            .intersect(shadow_ray)
            .map(|shadow_hit| shadow_hit.distance < light_distance)
            .unwrap_or(false)
    });

    let ambient = hit.material.color * scene.ambient;
    let diffuse_strength = if in_shadow {
        0.0
    } else {
        hit.normal.dot(light_dir).max(0.0) * scene.light.intensity
    };
    let diffuse = hit.material.color * diffuse_strength;

    let view_dir = -ray.direction;
    let reflection_dir = (-light_dir).reflect(hit.normal).normalize();
    let specular_strength = if in_shadow {
        0.0
    } else {
        view_dir.dot(reflection_dir).max(0.0).powf(32.0) * 0.35 * scene.light.intensity
    };
    let specular = Vec3::new(1.0, 1.0, 1.0) * specular_strength;

    let local = ambient + diffuse + specular;

    if hit.material.reflectivity <= 0.0 {
        return local;
    }

    let reflected_ray = Ray {
        origin: hit.point + hit.normal * EPSILON * 10.0,
        direction: ray.direction.reflect(hit.normal).normalize(),
    };
    let reflected = trace(scene, reflected_ray, depth + 1);
    local * (1.0 - hit.material.reflectivity) + reflected * hit.material.reflectivity
}

fn sky(scene: &Scene, direction: Vec3) -> Vec3 {
    let t = 0.5 * (direction.y + 1.0);
    scene.sky_bottom * (1.0 - t) + scene.sky_top * t
}

pub fn print_ppm(pixels: &[Vec3], width: usize, height: usize) {
    println!("P3");
    println!("{} {}", width, height);
    println!("255");
    for pixel in pixels {
        let rgb = pixel.hadamard(Vec3::new(255.0, 255.0, 255.0));
        println!(
            "{} {} {}",
            rgb.x.round() as i32,
            rgb.y.round() as i32,
            rgb.z.round() as i32
        );
    }
}
