mod camera;
mod config;
mod math;
mod object;
mod ray;
mod render;
mod scene;

use std::env;

use config::Config;
use render::{print_ppm, render};
use scene::build_scene;

fn main() {
    let config = Config::from_args(env::args().skip(1).collect());
    let scene = build_scene(&config.scene, config.brightness);
    let image = render(&scene, &config);
    print_ppm(&image, config.width, config.height);
}
