

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs::File;
use rocksdb::DB;
use walkdir::{DirEntry, WalkDir};
use std::io;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::ffi::OsStr;
use bincode::{deserialize, serialize, Infinite};
use maud::{html, Render, Markup};
use db::Key;

use chrono::{Utc, DateTime};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub lines: i32,
    pub extension: String,
}

impl FileInfo {
    pub fn from_path(path: &Path) -> Result<FileInfo, io::Error> {
        println!("GETTING ONE FILE NOW {:?}", path);
        let file = File::open(path)?;
        // more dumbass heuristics to avoid scanning binaries
        println!("len {}", file.metadata().unwrap().len());
        if file.metadata().unwrap().len() > 5000000 {
            return Ok(FileInfo::blank())
        }

        let mut count = 0;
        let mut reader = BufReader::new(file);
        for line in reader.lines() {
            // line is a Result<String, Error>
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

    pub fn blank() -> FileInfo {
        FileInfo{ path: String::from("error"), lines: 0, extension: String::new() }
    }

}

/// FileType marks what we determine a file to be.
pub enum FileType {
    Unknown,
    Rust,
    Go,
    ELF,
    Text,
    Markdown,
}

impl Render for FileInfo {
    fn render(&self) -> Markup {
        html! {

        }
    }

}
pub fn analyze_go(path: &mut PathBuf, db: Arc<DB>) {
    println!("analysis time");

    // proj is our common prefix for all analysis
    let proj = path.clone();
    // batch_id is a DateTime, and will serve as our common prefix
    let batch_id = Utc::now();
    // count number of .go files and the len of each
    let walker = WalkDir::new(path).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e) && golang_files(e)) {
        // only walking Go files now...
        println!("something happening");
        let entry = entry.expect("could not get dir entry");
        if entry.file_type().is_dir() {
            println!("skipping analysis for dir");
            continue;
        }
        println!("more stuff");
        let k = make_key!(proj.to_str().expect("proj to string failed"), batch_id, entry.path().to_str().expect("path to string failed"));

        let fi = FileInfo::from_path(entry.path()).expect("could not make fileinfo");
        println!("fileinfo made");
        let encoded: Vec<u8> = serialize(&fi, Infinite).unwrap();
        db.put(k.0.as_slice(), encoded.as_slice());
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
