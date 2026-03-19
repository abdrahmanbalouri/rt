#[derive(Clone)]
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub scene: String,
    pub brightness: f64,
}

impl Config {
    pub fn from_args(args: Vec<String>) -> Self {
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

pub fn print_help() {
    eprintln!("Usage: cargo run -- [--scene NAME] [--width N] [--height N] [--brightness F] > image.ppm");
    eprintln!("Scenes: sphere, plane-cube, all-objects, all-objects-alt");
}
