#![feature(plugin)]
#![plugin(rocket_codegen)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(dead_code)]

extern crate bincode;
#[allow(dead_code)]
extern crate rocket;
extern crate rocksdb;
#[macro_use]
extern crate serde_derive;

extern crate walkdir;

use walkdir::{DirEntry, WalkDir};

use rocksdb::DB;
use rocket::State;

use std::path::{Path, PathBuf};
use std::ops::Add;
use std::thread;
use std::sync::{Arc, Mutex};

use bincode::{deserialize, serialize, Infinite};

mod config;

fn main() {
    let conf = config::load();
    println!("config: {:?}", conf);
    let conf_arc = Arc::new(Mutex::new(conf));

    // Open the DB, wrap in atomic referenced-counted pointer,
    // clone the pointer twice: once for the background thread,
    // and again for our call to manage.
    let mut db = DB::open_default(".coded.db").unwrap();
    let db_arc = Arc::new(db);
    let db_background = db_arc.clone();
    let db_managed = db_arc.clone();

    // background thread
    thread::spawn(move || {
        println!("okay!!!");
        watch(db_background, conf_arc);
    });

    let routes = routes![index];
    rocket::ignite()
        .mount("/", routes)
        .manage(db_managed)
        .launch();
}

fn watch(db: Arc<DB>, conf: Arc<Mutex<config::Config>>) {
    use std::time::Duration;
    // We must clone() here.
    println!("don't believe me jus watch");
    let projects = conf.lock()
        .expect("could not unlock conf")
        .clone()
        .project
        .expect("could not unlock projects");
    println!("projects: {:?}", projects);
    loop {
        thread::sleep(Duration::from_secs(3));
        // I was using map() but I'm not "collecting" yet.
        for proj in projects.iter() {
            let mut path = PathBuf::from(proj.dir.as_str());
            println!("path: {}", path.display());
            if path.exists() {
                // continue...
                match project_heuristic(path.clone()) {
                    ProjectType::Go => {
                        analyze_go(&mut path, db.clone());
                    }
                    ProjectType::Rust => {}
                };
            };
        }
    }
}

pub enum ProjectType {
    Rust,
    Go,
}

fn analyze_go(path: &mut PathBuf, db: Arc<DB>) {
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


// TODO test this in /tests as an integration test
pub fn project_heuristic(mut p: PathBuf) -> ProjectType {
    // TODO: make this better
    p.push("Cargo.toml");
    if p.exists() {
        ProjectType::Rust
    } else {
        ProjectType::Go
    }
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Entity {
    name: String,
    age: i32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World(Vec<Entity>);

#[get("/")]
fn index(db: State<Arc<DB>>) -> String {
    let me = Entity {
        name: String::from("coleman"),
        age: 33,
    };
    let encoded: Vec<u8> = serialize(&me, Infinite).unwrap();
    let k = "k2";
    db.put(k.as_bytes(), encoded.as_slice());

    let name = {
        match db.get(k.as_bytes()) {
            Ok(Some(db_vec)) => {
                let decoded: Entity = deserialize(&db_vec[..]).unwrap();
                let name = decoded.name.as_str();
                String::from("Hello, world! ")
                    .add(name)
                    .add(" specifically")
            }
            _ => String::from("error!!!!"),
        }
    };
    name
}


unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}
