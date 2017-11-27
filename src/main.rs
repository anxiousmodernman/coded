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

#[macro_use]
extern crate coded;
extern crate rocket;
extern crate rocksdb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;

use walkdir::{DirEntry, WalkDir};

use maud::{html, Markup};
use serde::de::Deserialize;
use rocksdb::{DB, DBIterator, IteratorMode};
use rocket::State;

use std::path::{Path, PathBuf};
use std::ops::Add;
use std::sync::{Arc, Mutex};

use bincode::{deserialize, serialize, Infinite};

use coded::project::{analyze_go, guess_type, Project, FileInfo};
use coded::project;
use coded::config;
use std::thread;


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


#[get("/random")]
fn random(db: State<Arc<DB>>) -> Markup {
    let mut iter = db.iterator(IteratorMode::Start);

    html! {
        h2 "Projects"
        ol {
            @for (k, v) in iter {

                @let file_info: FileInfo = deserialize(&v[..]).unwrap_or(
                    FileInfo::blank()
                );

                li (file_info.path)
        }
    }
}
}


// rocksdb docstrings
// An iterator over a database or column family, with specifiable
// ranges and direction.
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