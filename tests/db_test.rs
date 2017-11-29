#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_must_use)]

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
use std::collections::HashSet;
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

    // With FileInfo values like this structure
    // { path, lines, extension }

    // Write these to the database and seek through them to aggregate results.

    // dir might be our configured path from coded.toml
    let dir = "/tmp";

    let time_a = Utc.ymd(2010, 04, 20);
    let time_b = Utc.ymd(2011, 04, 20);
    let time_c = Utc.ymd(2012, 04, 20);

    let src_a = project::FileInfo { path: String::from("/tmp/main.c"), lines: 100, extension: String::from("c") };
    let hdr_a = project::FileInfo { path: String::from("/tmp/main.h"), lines: 50, extension: String::from("h") };

    let src_b = project::FileInfo { path: String::from("/tmp/main.c"), lines: 110, extension: String::from("c") };
    let hdr_b = project::FileInfo { path: String::from("/tmp/main.h"), lines: 40, extension: String::from("h") };

    let src_c = project::FileInfo { path: String::from("/tmp/main.c"), lines: 105, extension: String::from("c") };
    let hdr_c = project::FileInfo { path: String::from("/tmp/main.h"), lines: 45, extension: String::from("h") };

    let src_a_key = make_key!("projects!", dir, "!", time_a, "!", src_a.path.clone());
    let hdr_a_key = make_key!("projects!", dir, "!", time_a, "!", hdr_a.path.clone());

    let src_b_key = make_key!("projects!", dir, "!", time_b, "!", src_b.path.clone());
    let hdr_b_key = make_key!("projects!", dir, "!", time_b, "!", hdr_b.path.clone());

    let src_c_key = make_key!("projects!", dir, "!", time_c, "!", src_c.path.clone());
    let hdr_c_key = make_key!("projects!", dir, "!", time_c, "!", hdr_c.path.clone());

    let writes = [
        (src_a_key, src_a),
        (hdr_a_key, hdr_a),
        (src_b_key, src_b),
        (hdr_b_key, hdr_b),
        (src_c_key, src_c),
        (hdr_c_key, hdr_c),
    ];

    let mut batch = WriteBatch::default();
    for &(ref k, ref v) in writes.iter() {
        let encoded = serialize(&v, Infinite).unwrap();
        batch.put(k.0.as_slice(), encoded.as_slice()).unwrap();
    }
    db.write(batch);

    // we can "re-use" this iterator
    let mut iter = db.iterator(IteratorMode::Start);

    // expected keys in the db: 4
    let all_keys = iter.count();
    assert_eq!(all_keys, 6);

    // We do an explicit seek to Time B, scan until end, expecting 4 keys.
    iter = db.iterator(IteratorMode::From(
        make_key!("projects!", dir, "!", time_b).0.as_slice(), Direction::Forward));

    let b_keys = iter.count();
    assert_eq!(b_keys, 4);

    // Here we extract the timestamp portions of our keys
    iter = db.iterator(IteratorMode::Start);
    let timestamps: Vec<String> = iter.map(project::get_timestamps).collect();
    assert_eq!(timestamps.get(0).unwrap(), "2010-04-20T00:00:00+00:00");

    // But we can get rid of dupes by collecting a HashSet
    iter = db.iterator(IteratorMode::Start);
    let unique_timestamps: HashSet<String> = iter.map(project::get_timestamps).collect();
    assert_eq!(unique_timestamps.len(), 3);

    // we pick a day that lands "between" the keyspaces, e.g. "all activity since 2011-01-01".
    let before_b = make_key!("projects", dir, "!", Utc.ymd(2011, 01, 01));

    // seek to first key that matches, which we expect to be under Time B
    iter = db.iterator(IteratorMode::From(
        make_key!("projects!", dir, "!", before_b).0.as_slice(), Direction::Forward));

    // find total lines in each segment
    // find diff over total

    // find diff on single file
}

#[test]
fn test_equality() {
    // TODO: try and break make_key's serialization
}


/*
Query styles:

- map/reduce
to map, we need segments to map over. Segment:
project!/foo/baz!{ts} - we block our queries by ts

1. get list of ts
2. map ts:
*/


