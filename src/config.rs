extern crate toml;

#[macro_use]
extern crate serde_derive;

use std::path::Path;
use std::fs::File;
use std::io::Result;


#[derive(Serialize, Deserialize)]
struct Config {
    projects: Option<Vec<Project>>
}


#[derive(Serialize, Deserialize)]
struct ProjectConfig {
    dir: String,
}

fn load(path: Path) -> io::Result<Config>{
    let f = File::open(path)?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    let decoded: Config = toml::from_str(buffer)?;
    decoded
}
