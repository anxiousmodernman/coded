use std::path::{Path, PathBuf};

// TODO test this in /tests as an integration test
pub fn guess_type(mut p: PathBuf) -> ProjectType {
    // TODO: make this better...
    p.push("Cargo.toml");
    if p.exists() {
        ProjectType::Rust
    } else {
        ProjectType::Go
    }
}

pub enum ProjectType {
    Rust,
    Go,
}
