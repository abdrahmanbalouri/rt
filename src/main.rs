use std::env;
use std::f64::consts::PI;

const EPSILON: f64 = 1e-4;
const MAX_BOUNCES: usize = 3;

fn main() {
    let config = Config::from_args(env::args().skip(1).collect());
    let scene = build_scene(&config.scene, config.brightness);
    let image = render(&scene, &config);
    print_ppm(&image, config.width, config.height);
}

#[derive(Clone, Copy, Debug, Default)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            self
        } else {
            self / len
        }
    }

    fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn hadamard(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    fn clamp(self, min: f64, max: f64) -> Self {
        Self::new(
            self.x.clamp(min, max),
            self.y.clamp(min, max),
            self.z.clamp(min, max),
        )
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl std::ops::Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

#[derive(Clone, Copy)]
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn at(self, t: f64) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[derive(Clone, Copy)]
struct Material {
    color: Vec3,
    reflectivity: f64,
}

#[derive(Clone, Copy)]
struct Hit {
    distance: f64,
    point: Vec3,
    normal: Vec3,
    material: Material,
}

trait Hittable {
    fn intersect(&self, ray: Ray) -> Option<Hit>;
}

struct Sphere {
    center: Vec3,
    radius: f64,
    material: Material,
}

impl Hittable for Sphere {
    fn intersect(&self, ray: Ray) -> Option<Hit> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return None;
        }
        let sqrt_disc = disc.sqrt();
        let mut t = (-b - sqrt_disc) / (2.0 * a);
        if t < EPSILON {
            t = (-b + sqrt_disc) / (2.0 * a);
            if t < EPSILON {
                return None;
            }
        }
        let point = ray.at(t);
        Some(Hit {
            distance: t,
            point,
            normal: (point - self.center).normalize(),
            material: self.material,
        })
    }
}

struct Plane {
    point: Vec3,
    normal: Vec3,
    material: Material,
}

impl Hittable for Plane {
    fn intersect(&self, ray: Ray) -> Option<Hit> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() < EPSILON {
            return None;
        }
        let t = (self.point - ray.origin).dot(self.normal) / denom;
        if t < EPSILON {
            return None;
        }
        Some(Hit {
            distance: t,
            point: ray.at(t),
            normal: if denom < 0.0 { self.normal } else { -self.normal },
            material: self.material,
        })
    }
}

struct Cube {
    min: Vec3,
    max: Vec3,
    material: Material,
}

impl Hittable for Cube {
    fn intersect(&self, ray: Ray) -> Option<Hit> {
        let inv = Vec3::new(
            safe_inverse(ray.direction.x),
            safe_inverse(ray.direction.y),
            safe_inverse(ray.direction.z),
        );

        let tx1 = (self.min.x - ray.origin.x) * inv.x;
        let tx2 = (self.max.x - ray.origin.x) * inv.x;
        let ty1 = (self.min.y - ray.origin.y) * inv.y;
        let ty2 = (self.max.y - ray.origin.y) * inv.y;
        let tz1 = (self.min.z - ray.origin.z) * inv.z;
        let tz2 = (self.max.z - ray.origin.z) * inv.z;

        let tmin = tx1.min(tx2).max(ty1.min(ty2)).max(tz1.min(tz2));
        let tmax = tx1.max(tx2).min(ty1.max(ty2)).min(tz1.max(tz2));

        if tmax < tmin || tmax < EPSILON {
            return None;
        }

        let t = if tmin > EPSILON { tmin } else { tmax };
        let point = ray.at(t);
        let normal = cube_normal(point, self.min, self.max);

        Some(Hit {
            distance: t,
            point,
            normal,
            material: self.material,
        })
    }
}

struct Cylinder {
    center: Vec3,
    radius: f64,
    min_y: f64,
    max_y: f64,
    material: Material,
}

impl Hittable for Cylinder {
    fn intersect(&self, ray: Ray) -> Option<Hit> {
        let oc = ray.origin - self.center;
        let a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z;
        let b = 2.0 * (oc.x * ray.direction.x + oc.z * ray.direction.z);
        let c = oc.x * oc.x + oc.z * oc.z - self.radius * self.radius;

        let mut best: Option<Hit> = None;

        if a.abs() > EPSILON {
            let disc = b * b - 4.0 * a * c;
            if disc >= 0.0 {
                let sqrt_disc = disc.sqrt();
                for t in [(-b - sqrt_disc) / (2.0 * a), (-b + sqrt_disc) / (2.0 * a)] {
                    if t < EPSILON {
                        continue;
                    }
                    let point = ray.at(t);
                    if point.y >= self.min_y && point.y <= self.max_y {
                        let normal =
                            Vec3::new(point.x - self.center.x, 0.0, point.z - self.center.z)
                                .normalize();
                        best = choose_closer(
                            best,
                            Hit {
                                distance: t,
                                point,
                                normal,
                                material: self.material,
                            },
                        );
                    }
                }
            }
        }

        for (cap_y, normal) in [
            (self.min_y, Vec3::new(0.0, -1.0, 0.0)),
            (self.max_y, Vec3::new(0.0, 1.0, 0.0)),
        ] {
            if ray.direction.y.abs() < EPSILON {
                continue;
            }
            let t = (cap_y - ray.origin.y) / ray.direction.y;
            if t < EPSILON {
                continue;
            }
            let point = ray.at(t);
            let dx = point.x - self.center.x;
            let dz = point.z - self.center.z;
            if dx * dx + dz * dz <= self.radius * self.radius {
                best = choose_closer(
                    best,
                    Hit {
                        distance: t,
                        point,
                        normal,
                        material: self.material,
                    },
                );
            }
        }

        best
    }
}

fn choose_closer(current: Option<Hit>, candidate: Hit) -> Option<Hit> {
    match current {
        Some(existing) if existing.distance <= candidate.distance => Some(existing),
        _ => Some(candidate),
    }
}

fn safe_inverse(value: f64) -> f64 {
    if value.abs() < EPSILON {
        f64::INFINITY
    } else {
        1.0 / value
    }
}

fn cube_normal(point: Vec3, min: Vec3, max: Vec3) -> Vec3 {
    let faces = [
        ((point.x - min.x).abs(), Vec3::new(-1.0, 0.0, 0.0)),
        ((point.x - max.x).abs(), Vec3::new(1.0, 0.0, 0.0)),
        ((point.y - min.y).abs(), Vec3::new(0.0, -1.0, 0.0)),
        ((point.y - max.y).abs(), Vec3::new(0.0, 1.0, 0.0)),
        ((point.z - min.z).abs(), Vec3::new(0.0, 0.0, -1.0)),
        ((point.z - max.z).abs(), Vec3::new(0.0, 0.0, 1.0)),
    ];
    faces
        .into_iter()
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, normal)| normal)
        .unwrap_or(Vec3::new(0.0, 1.0, 0.0))
}

struct Light {
    position: Vec3,
    intensity: f64,
}

struct Camera {
    origin: Vec3,
    target: Vec3,
    fov_degrees: f64,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    viewport_height: f64,
    viewport_width: f64,
}

impl Camera {
    fn new(origin: Vec3, target: Vec3, fov_degrees: f64, aspect_ratio: f64) -> Self {
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

    fn ray(&self, u: f64, v: f64) -> Ray {
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

struct Scene {
    camera: Camera,
    light: Light,
    ambient: f64,
    sky_top: Vec3,
    sky_bottom: Vec3,
    objects: Vec<Box<dyn Hittable>>,
}

#[derive(Clone)]
struct Config {
    width: usize,
    height: usize,
    scene: String,
    brightness: f64,
}

impl Config {
    fn from_args(args: Vec<String>) -> Self {
        let mut config = Self {
            width: 800,
            height: 600,
            scene: "sphere".to_string(),
            brightness: 1.0,
        };

        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--width" => {
                    if let Some(value) = args.get(index + 1) {
                        config.width = value.parse().unwrap_or(config.width);
                    }
                    index += 2;
                }
                "--height" => {
                    if let Some(value) = args.get(index + 1) {
                        config.height = value.parse().unwrap_or(config.height);
                    }
                    index += 2;
                }
                "--scene" => {
                    if let Some(value) = args.get(index + 1) {
                        config.scene = value.to_lowercase();
                    }
                    index += 2;
                }
                "--brightness" => {
                    if let Some(value) = args.get(index + 1) {
                        config.brightness = value.parse().unwrap_or(config.brightness);
                    }
                    index += 2;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    index += 1;
                }
            }
        }

        config
    }
}

fn print_help() {
    eprintln!("Usage: cargo run -- [--scene NAME] [--width N] [--height N] [--brightness F] > image.ppm");
    eprintln!("Scenes: sphere, plane-cube, all-objects, all-objects-alt");
}

fn build_scene(name: &str, brightness: f64) -> Scene {
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

fn render(scene: &Scene, config: &Config) -> Vec<Vec3> {
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

fn print_ppm(pixels: &[Vec3], width: usize, height: usize) {
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
