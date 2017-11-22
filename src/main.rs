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
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;

use walkdir::{DirEntry, WalkDir};

use serde::de::Deserialize;
use rocksdb::DB;
use rocket::State;
use db::GetAs;

use std::path::{Path, PathBuf};
use std::ops::Add;
use std::thread;
use std::sync::{Arc, Mutex};

use bincode::{deserialize, serialize, Infinite};

mod config;
mod controllers;
mod models;
mod db;

use models::ProjectType;

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

    let routes = routes![controllers::index];
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
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn golang_files(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s != "vendor")
        .unwrap_or(false)
}


// TODO test this in /tests as an integration test
pub fn project_heuristic(mut p: PathBuf) -> ProjectType {
    // TODO: make this better...
    p.push("Cargo.toml");
    if p.exists() {
        ProjectType::Rust
    } else {
        ProjectType::Go
    }
}


unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}
