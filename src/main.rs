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

    let routes = routes![index, random];
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

#[get("/")]
fn index(db: State<Arc<DB>>) -> String {
    let me = Entity {
        name: String::from("coleman"),
        age: 33,
    };
    let encoded: Vec<u8> = serialize(&me, Infinite).unwrap();
    let k = make_key!("k2");
    db.put(k.0.as_slice(), encoded.as_slice());

    let decoded: Entity = db.get_as(k).unwrap();

    let name = {
        let name = decoded.name.as_str();
        String::from("Hello, world! ")
            .add(name)
            .add(" specifically")
    };
    name
}

struct ProjectView {
    lines: i32
}


#[get("/random")]
fn random(db: State<Arc<DB>>) -> Markup {
    let mut iter = db.iterator(IteratorMode::Start);
    let mut projects: Vec<String> = iter.map(project::get_projects).collect();
    projects.dedup();

    let now = Utc::now();
//    let then = now - Duration::hours(2);
    let then = now - Duration::minutes(5);


    let mut project_diffs: Vec<(String, i32)> = Vec::new();
    for proj in projects {
        println!("scanning project {:?}", proj);
        let mut sums = Vec::new();
        let scan_from = make_key!("projects!", proj.clone(), "!", then);
        iter = db.iterator(IteratorMode::From(scan_from.0.as_slice(), Direction::Forward));
        let mut batch_ids: Vec<String> = iter.map(project::get_timestamps).collect();
        batch_ids.dedup();
        for batch_id in batch_ids {
            let batch = make_key!("projects!", proj.clone(), "!", batch_id.clone());
            iter = db.iterator(IteratorMode::From(batch.0.as_slice(), Direction::Forward));
            let total: i32 = iter.map(project::yield_lines).sum();
            sums.push(total);
            println!("    batch_id {:?}   sum {:?}", batch_id, total);

        }
        let mut diffs = Vec::new();
        let mut prev = 0;
        for s in sums {
            let diff = s - prev;
            prev = s;
            diffs.push(diff);
        }
        let d = diffs.into_iter().sum();
        project_diffs.push((proj, d));
    }

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


// rocksdb docstrings
// An iterator over a database or column family, with specifiable
// ranges and direction.
//
// Note(cm): I think "column family" is like a bucket in boltdb
//
// ```
// use rocksdb::{DB, Direction, IteratorMode};
//
// let mut db = DB::open_default("path/for/rocksdb/storage2").unwrap();
// let mut iter = db.iterator(IteratorMode::Start); // Always iterates forward
// for (key, value) in iter {
//     println!("Saw {:?} {:?}", key, value);
// }
// iter = db.iterator(IteratorMode::End);  // Always iterates backward
// for (key, value) in iter {
//     println!("Saw {:?} {:?}", key, value);
// }
// iter = db.iterator(IteratorMode::From(b"my key", Direction::Forward)); // From a key in Direction::{forward,reverse}
// for (key, value) in iter {
//     println!("Saw {:?} {:?}", key, value);
// }
//
// // You can seek with an existing Iterator instance, too
// iter = db.iterator(IteratorMode::Start);
// iter.set_mode(IteratorMode::From(b"another key", Direction::Reverse));
// for (key, value) in iter {
//     println!("Saw {:?} {:?}", key, value);
// }
// ```
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
