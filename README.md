# rt ray tracer

`rt` is a small dependency-free Rust ray tracer that renders ASCII `P3` PPM images to standard output.

It supports:

- Sphere intersections
- Plane intersections
- Axis-aligned cube intersections
- Finite cylinder intersections with top and bottom caps
- Movable camera
- Adjustable light brightness
- Shadows
- Basic reflections

## Build and render

```bash
cargo run --release -- --scene sphere --width 800 --height 600 > sphere.ppm
```

Available scenes:

- `sphere`
- `plane-cube`
- `all-objects`
- `all-objects-alt`

Other flags:

- `--width 800`
- `--height 600`
- `--brightness 1.0`

## Required sample renders

```bash
cargo run --release -- --scene sphere --width 800 --height 600 > scene_1_sphere.ppm
cargo run --release -- --scene plane-cube --width 800 --height 600 > scene_2_plane_cube.ppm
cargo run --release -- --scene all-objects --width 800 --height 600 > scene_3_all_objects.ppm
cargo run --release -- --scene all-objects-alt --width 800 --height 600 > scene_4_all_objects_alt.ppm
```

The second scene is intentionally darker than the first one.

## How the ray tracer works

For each pixel, the camera emits one ray into the scene. The nearest object hit by that ray is shaded with:

- Ambient light
- Diffuse lighting from a point light
- Specular highlight
- Shadow testing with a secondary ray
- Optional recursive reflection

If no object is hit, a gradient sky color is used.

## Code examples

### Create a sphere

```rust
let sphere = Sphere {
    center: Vec3::new(0.0, 1.0, 0.5),
    radius: 1.0,
    material: Material {
        color: Vec3::new(0.82, 0.22, 0.18),
        reflectivity: 0.25,
    },
};
```

### Create a cube

```rust
let cube = Cube {
    min: Vec3::new(-1.0, 0.0, -0.1),
    max: Vec3::new(1.0, 2.0, 1.9),
    material: Material {
        color: Vec3::new(0.18, 0.44, 0.78),
        reflectivity: 0.10,
    },
};
```

### Create a flat plane

```rust
let plane = Plane {
    point: Vec3::new(0.0, 0.0, 0.0),
    normal: Vec3::new(0.0, 1.0, 0.0),
    material: Material {
        color: Vec3::new(0.85, 0.85, 0.88),
        reflectivity: 0.05,
    },
};
```

### Create a cylinder

```rust
let cylinder = Cylinder {
    center: Vec3::new(3.0, 0.0, 2.3),
    radius: 0.8,
    min_y: 0.0,
    max_y: 2.4,
    material: Material {
        color: Vec3::new(0.84, 0.74, 0.22),
        reflectivity: 0.18,
    },
};
```

### Change brightness

Brightness is controlled from the command line and scales the light intensity:

```bash
cargo run -- --scene all-objects --brightness 0.6 > darker.ppm
cargo run -- --scene all-objects --brightness 1.4 > brighter.ppm
```

### Change camera position and angle

The sample scenes build a camera with an origin, a target point, and a field of view:

```rust
let camera = Camera::new(
    Vec3::new(-4.5, 2.5, -6.5),
    Vec3::new(0.3, 1.0, 1.3),
    46.0,
    4.0 / 3.0,
);
```

- The first vector is the camera position.
- The second vector is the point the camera looks at.
- The third value is the field of view in degrees.
- The last value is the aspect ratio.

To produce the same scene from another angle, change the origin and target, or use the built-in alternate preset:

```bash
cargo run -- --scene all-objects-alt > alt_view.ppm
```
