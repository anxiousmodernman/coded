extern crate toml;

use std::path::Path;
use std::fs::File;
use std::io::{Read, Result};
use std::env;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub project: Option<Vec<ProjectConfig>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectConfig {
    pub dir: String,
}

/// Loads a Config from the environment, or panics.
pub fn load() -> Config {
    let base_dir = env::var("HOME").expect("could not get HOME env var");
    let path = Path::join(Path::new(&base_dir), ".config/coded/coded.toml");
    let mut f = File::open(path).expect("could not open conf file");
    let mut buffer = String::new();
    f.read_to_string(&mut buffer).expect("could not read file");
    let decoded: Config = toml::from_str(buffer.as_str()).expect("could not deserialize config");
    decoded
}
