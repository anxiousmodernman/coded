use std::path::{Path, PathBuf};
use std::sync::Arc;
use rocksdb::DB;
use walkdir::{WalkDir, DirEntry};

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

pub struct Project {
    files: Vec<File>
}

pub struct File {
    path: String,
    lines: i32,
}


pub fn analyze_go(path: &mut PathBuf, db: Arc<DB>) {
    // ignore vendor
    // count number of .go files and the len of each
    println!("wow!");
    let walker = WalkDir::new(path).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e) && golang_files(e)) {
        // only walking Go files now...
        let entry = entry.unwrap();
        println!("name: {:?}, path: {:?}", entry.file_name(), entry.path());

        // naive impl:
        // "get" file repr from database
    }
}


fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn golang_files(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s != "vendor")
        .unwrap_or(false)
}