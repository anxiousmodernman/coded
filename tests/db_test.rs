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
use chrono::prelude::*;
use tempdir::TempDir;
use bincode::{deserialize, serialize, Infinite};
use std::path::PathBuf;
use coded::*;


#[test]
fn test_sortable_key() {
    // TempDir type is auto-removed.
    let mut db = DB::open_default(TempDir::new("test").unwrap()).unwrap();

    // we could do this...
    // let utc: DateTime<Utc> = Utc::now();

    // but we hardcode instead
    let oct_rev = Utc.ymd(1917, 10, 25);
    let bday = Utc.ymd(1984, 09, 21);
    let nine_eleven = Utc.ymd(2001, 09, 11);

    // make 3 private data structures that indicate their expected order via id field
    let _val1 = SomeValue { id: 1 };
    let _val2 = SomeValue { id: 2 };
    let _val3 = SomeValue { id: 3 };

    // insert them out of order; we will rely on RocksDB native key sorting
    let vals = [(bday, _val2), (oct_rev, _val1), (nine_eleven, _val3)];

    for &(ref k, ref v) in vals.iter() {
        let encoded: Vec<u8> = serialize(&v, Infinite).unwrap();
        db.put(k.to_string().as_bytes(), encoded.as_slice()).unwrap();
    }

    // start at the earliest; iterators must be mut
    let mut iter = db.iterator(
        IteratorMode::From(oct_rev.to_string().as_bytes(), Direction::Forward));

    // iter gives us
    // `std::option::Option<(std::boxed::Box<[u8]>, std::boxed::Box<[u8]>)>`
    let db_vec1 = iter.next().unwrap();
    // note we access second field of tuple here with .1
    // the & gets us into the Box
    let decoded1: SomeValue = deserialize(&db_vec1.1[..]).unwrap();

    let db_vec2 = iter.next().unwrap();
    let decoded2: SomeValue = deserialize(&db_vec2.1[..]).unwrap();

    let db_vec3 = iter.next().unwrap();
    let decoded3: SomeValue = deserialize(&db_vec3.1[..]).unwrap();

    assert_eq!(decoded1.id, 1);
    assert_eq!(decoded2.id, 2);
    assert_eq!(decoded3.id, 3);
}

#[derive(Serialize, Deserialize)]
struct SomeValue {
    id: i32
}
