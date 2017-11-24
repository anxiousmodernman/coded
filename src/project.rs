use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs::File;
use rocksdb::DB;
use walkdir::{DirEntry, WalkDir};
use std::io;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::ffi::OsStr;


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

pub struct Project {}

pub struct FileInfo {
    pub path: String,
    pub lines: i32,
    pub extension: String,
}

impl FileInfo {
    pub fn from_path(path: &Path) -> Result<FileInfo, io::Error> {
        let file = File::open(path)?;
        let mut count = 0;
        let mut reader = BufReader::new(file);
        for line in reader.lines() {
            count += 1;
        }
        let p = path.to_str().expect("could not render path");
        let e = match path.extension() {
            Some(ext) => ext.to_os_string().into_string().unwrap(),
            _ => String::default(),
        };
        let file_info = FileInfo {
            path: String::from_str(p).unwrap(),
            lines: count,
            extension: e,
        };
        Ok(file_info)
    }
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
