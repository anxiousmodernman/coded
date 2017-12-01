#![feature(plugin)]
#![plugin(rocket_codegen)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(dead_code)]
#![feature(proc_macro)]

extern crate bincode;
extern crate maud;
extern crate chrono;

#[macro_use]
extern crate coded;
extern crate rocket;
extern crate rocksdb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;

use std::path::{Path, PathBuf};
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::thread;

use bincode::{deserialize, serialize, Infinite};
use maud::{html, Markup};
use serde::de::Deserialize;
use rocksdb::{DB, DBIterator, IteratorMode, Direction};
use rocket::State;
use walkdir::{DirEntry, WalkDir};
use chrono::{Utc, DateTime, Duration};

use coded::project::{analyze_go, guess_type, Project, FileInfo};
use coded::project;
use coded::config;
use coded::db::{GetAs, Key};

mod background;


fn main() {
    let conf = config::load();
    let conf_arc = Arc::new(Mutex::new(conf));

    // Open the DB, wrap in atomic referenced-counted pointer,
    // clone the pointer twice: once for the background thread,
    // and again for our call to manage.
    let mut db = DB::open_default(".coded.db").unwrap();
    let db_arc = Arc::new(db);
    let db_background = db_arc.clone();
    let db_managed = db_arc.clone();

    thread::spawn(move || {
        background::watch(db_background, conf_arc);
    });

    let routes = routes![index];
    rocket::ignite()
        .mount("/", routes)
        .manage(db_managed)
        .launch();
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Entity {
    name: String,
    age: i32,
}


struct ProjectView {
    lines: i32
}


#[get("/")]
fn index(db: State<Arc<DB>>) -> Markup {
    let mut iter = db.iterator(IteratorMode::Start);
    let mut projects: Vec<String> = iter.map(project::get_projects).collect();
    projects.dedup();

    let now = Utc::now();
//    let then = now - Duration::hours(2);
    let then = now - Duration::minutes(5);

    let mut project_diffs: Vec<(String, i32)> = Vec::new();

    // TODO: styles
    html! {
        h2 "Projects"
        ol {
            @for (p, d) in project_diffs {
                li { p {(p)} p {(d)} }
            }
        }
    }
}


// write batches!
// use rocksdb::{DB, WriteBatch};
//
// let db = DB::open_default("path/for/rocksdb/storage1").unwrap();
// {
//     let mut batch = WriteBatch::default();
//     batch.put(b"my key", b"my value");
//     batch.put(b"key2", b"value2");
//     batch.put(b"key3", b"value3");
//     db.write(batch); // Atomically commits the batch
// }
