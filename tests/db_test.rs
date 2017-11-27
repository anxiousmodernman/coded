#![allow(unused_variables)]
#![allow(unused_imports)]

#[macro_use]
extern crate coded;
extern crate iso8601;
extern crate chrono;
extern crate rocksdb;
extern crate tempdir;
extern crate bincode;

#[macro_use]
extern crate serde_derive;

use rocksdb::*;
use std::time;
use std::str;
use chrono::prelude::*;
use tempdir::TempDir;
use bincode::{deserialize, serialize, Infinite};
use std::path::{PathBuf, Path};
use coded::*;
use coded::db::Key;

#[derive(Serialize, Deserialize)]
struct Thing {
    id: i32
}

#[test]
fn test_sortable_key() {
    // TempDir type is auto-removed.
    let db = DB::open_default(TempDir::new("test").unwrap()).unwrap();

    let oct_rev = make_key!(Utc.ymd(1917, 10, 25));
    let bday = make_key!(Utc.ymd(1984, 09, 21));
    let nine_eleven = make_key!(Utc.ymd(2001, 09, 11));

    // make 3 private data structures that indicate their expected order via id field
    let _val1 = Thing { id: 1 };
    let _val2 = Thing { id: 2 };
    let _val3 = Thing { id: 3 };

    let first = oct_rev.clone();

    // insert them out of order; we will rely on RocksDB native key sorting
    let vals = [(bday, _val2), (oct_rev, _val1), (nine_eleven, _val3)];

    for &(ref k, ref v) in vals.iter() {
        let encoded: Vec<u8> = serialize(&v, Infinite).unwrap();
        db.put(k.0.as_slice(), encoded.as_slice()).unwrap();
    }

    // start at the earliest; iterators must be mut
    let mut iter = db.iterator(
        IteratorMode::From(first.0.as_slice(), Direction::Forward));

    // iter gives us
    // `std::option::Option<(std::boxed::Box<[u8]>, std::boxed::Box<[u8]>)>`
    let db_vec1 = iter.next().unwrap();
    // note we access second field of tuple here with .1
    // the & gets us into the Box
    let decoded1: Thing = deserialize(&db_vec1.1[..]).unwrap();

    let db_vec2 = iter.next().unwrap();
    let decoded2: Thing = deserialize(&db_vec2.1[..]).unwrap();

    let db_vec3 = iter.next().unwrap();
    let decoded3: Thing = deserialize(&db_vec3.1[..]).unwrap();

    assert_eq!(decoded1.id, 1);
    assert_eq!(decoded2.id, 2);
    assert_eq!(decoded3.id, 3);
}

#[test]
fn test_aggregation() {
    let db = DB::open_default(TempDir::new("test").unwrap()).unwrap();

    // Make keys with this structure
    // projects!{proj_path}!{batch_ts}!{path}

    // With values like this structure
    // { path, line_count }

    // Write these to the database and seek through them to aggregate results.

    // procedure:
    // 1. write a batch of line counts for two distinct files at Time A
    // 2. write another batch at Time B
    // 3. given only a project dir path (what config gives us), compute
    //    the growth/delta of both files over the history

    // for example, dir might be our configured path from coded.toml
    let dir = "/tmp";

    let time_a = Utc.ymd(2010, 04, 20);
    let time_b = Utc.ymd(2011, 04, 20);

    let src_a = project::FileInfo{ path: String::from("/tmp/main.c"), lines: 100, extension: String::from("c")};
    let header_a = project::FileInfo{ path: String::from("/tmp/main.h"), lines: 50, extension: String::from("h")};

    let src_b = project::FileInfo{ path: String::from("/tmp/main.c"), lines: 110, extension: String::from("c")};
    let header_b = project::FileInfo{ path: String::from("/tmp/main.h"), lines: 40, extension: String::from("h")};

    let src_a_key = make_key!("projects!", dir, "!", time_a, "!", src_a.path.clone());
    let header_a_key = make_key!("projects!", dir, "!", time_a, "!", header_a.path.clone());

    let src_b_key = make_key!("projects!", dir, "!", time_b, "!", src_b.path.clone());
    let header_b_key = make_key!("projects!", dir, "!", time_b, "!", header_b.path.clone());

    let writes = [
        (src_a_key, src_a),
        (header_a_key, header_a),
        (src_b_key, src_b),
        (header_b_key, header_b),
    ];

    for &(ref k, ref v) in writes.iter() {
        let encoded = serialize(&v, Infinite).unwrap();
        db.put(k.0.as_slice(), encoded.as_slice()).unwrap();
    }
    // we can "re-use" this iterator
    let mut iter = db.iterator(IteratorMode::Start);

    // expected keys in the db: 4
    let all_keys = iter.count();
    assert_eq!(all_keys, 4);

    // We do an explicit seek to Time A, but we know that it's still the first key in the db
    iter = db.iterator(IteratorMode::From(
        make_key!("projects!", dir, "!", time_a).0.as_slice(), Direction::Forward));

    let a_and_b_keys = iter.count();
    assert_eq!(a_and_b_keys, 4);

    iter = db.iterator(IteratorMode::From(
        make_key!("projects!", dir, "!", time_b).0.as_slice(), Direction::Forward));

    let b_keys = iter.count();
    assert_eq!(b_keys, 2);

    // Here we extract the timestamp portions of our keys
    iter = db.iterator(IteratorMode::Start);
    let timestamps: Vec<String> = iter.map(get_timestamps).collect();
    assert_eq!(timestamps.get(0).unwrap(), "2010-04-20T00:00:00+00:00");



}

fn get_timestamps(kv: (Box<[u8]>, Box<[u8]>)) -> String {
    // We use ref to take the Box contents as a reference. This matters for &[u8].
    let ref st = *kv.0;
    let s = str::from_utf8(st).unwrap();
    let splitted: Vec<&str> = s.split("!").collect();
    let ts = splitted.get(2).expect("index 2 get");
    String::from(*ts)
}


