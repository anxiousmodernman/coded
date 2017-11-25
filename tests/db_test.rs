extern crate coded;
extern crate iso8601;
extern crate chrono;
extern crate rocksdb;
extern crate tempdir;

use rocksdb::*;
use std::time;
use chrono::prelude::*;
use tempdir::TempDir;

#[macro_use] extern crate serde_derive;
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
    let bday = Utc.ymd(1984, 10, 25);
    let nine_eleven = Utc.ymd(2001, 09, 11);



    // but we hardcode instead
    let now = time::SystemTime::now();
    // make 3 private data structures that indicate their order

    // generate keys with hardcoded timestamps, try not to depend on "actual" time
    // write the keys out of order, emphasizing the fact that it is the DB that sorts us lex-wise
    // get an iterator
    // seek to the first key
    // +1
    // +1
    // assert that we have expected values for each, in order
}

#[derive(Serialize, Deserialize)]
struct SomeValue{
    id: i32
}

