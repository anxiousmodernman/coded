use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs::File;
use std::str;
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

pub struct Project {
    path: String,
    files: Vec<FileInfo>,
}

impl Project {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub lines: i32,
    pub extension: String,
}

impl FileInfo {
    /// from_path constructs a FileInfo by inspecting the contents of the file at path.
    pub fn from_path(path: &Path) -> Result<FileInfo, io::Error> {
        let file = File::open(path)?;
        if file.metadata().unwrap().len() > 5000000 {
            return Ok(FileInfo::blank());
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

    // TODO: this is silly.
    pub fn blank() -> FileInfo {
        FileInfo { path: String::from("error"), lines: 0, extension: String::new() }
    }
}

pub fn diff_since(then: DateTime<Utc>, db: &DB) -> Option<i32> {
    Some(0)
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
    let proj = path.clone();
    // batch_id is a DateTime
    let batch_id = Utc::now();
    // count number of .go files and the len of each
    let walker = WalkDir::new(path).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e) && golang_files(e)) {
        let entry = entry.expect("could not get dir entry");
        if entry.file_type().is_dir() {
            // skip direcotries
            continue;
        }
        // key: projects!{path}!{batch_id}!{entry_path}
        let k = make_key!(
            "projects!",
            proj.to_str().unwrap(), 
            "!",
            batch_id, 
            "!",
            entry.path().to_str().unwrap());
        let fi = FileInfo::from_path(entry.path()).expect("could not make FileInfo");
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


pub fn get_timestamps(kv: (Box<[u8]>, Box<[u8]>)) -> String {
    let ref st = *kv.0;
    let s = str::from_utf8(st).unwrap();
    let splitted: Vec<&str> = s.split("!").collect();
    let ts = splitted.get(2).unwrap();
    String::from(*ts)
}

pub fn get_projects(kv: (Box<[u8]>, Box<[u8]>)) -> String {
    let ref st = *kv.0;
    let s = str::from_utf8(st).unwrap();
    let splitted: Vec<&str> = s.split("!").collect();
    match splitted.get(1) {
        Some(ts) => String::from(*ts),
        _ => String::from("unknown project")
    }
}

pub fn yield_lines(kv: (Box<[u8]>, Box<[u8]>)) -> i32 {
    // we need index 1, the value
    let ref st = *kv.1;
    let fi: FileInfo = match deserialize(st) {
        Ok(x) => x,
        _ => FileInfo{extension: String::from("blah"), lines: -1, path: String::from("/foo/baz")}
    };
    fi.lines
}