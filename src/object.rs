use crate::math::Vec3;
use crate::ray::Ray;

pub const EPSILON: f64 = 1e-4;

#[derive(Clone, Copy)]
pub struct Material {
    pub color: Vec3,
    pub reflectivity: f64,
}

#[derive(Clone, Copy)]
pub struct Hit {
    pub distance: f64,
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

pub trait Hittable {
    fn intersect(&self, ray: Ray) -> Option<Hit>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Material,
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

pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
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

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
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

pub struct Cylinder {
    pub center: Vec3,
    pub radius: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub material: Material,
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
