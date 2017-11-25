// we don't need extern crate here
// we can do that in lib.rs instead

use chrono::prelude::*;
use serde::de::Deserialize;
use rocksdb::{DB, DBVector};
use bincode::{deserialize, serialize, Infinite};
use std::convert::From;

pub trait GetAs {
    fn get_as<T>(&self, key: &str) -> Result<T, String> where for <'a>  T: Deserialize<'a>;
}

impl GetAs for DB {
    fn get_as<T>(&self, key: &str) -> Result<T, String> where for <'a> T: Deserialize<'a> {
        match self.get(key.as_bytes()) {
            Ok(None) => Err(format!("Could not find key '{}' in DB", key)),
            Ok(Some(db_vec)) => deserialize(&db_vec[..]).map_err(|e|
                format!("Error deserializing key '{}' from DB: {:?}", key, e)
            ),
            Err(e) => Err(format!("Error fetching key '{}' from DB: {:?}", key, e)),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Key(Vec<u8>);

impl Key {
    pub fn join(&self, mut k: Key) -> Key {
        let mut left: Vec<u8> = self.0.clone();
        let mut right: Vec<u8> = k.0;
        left.append(&mut right);
        Key(left)
    }
}

impl Into<Vec<u8>> for Key  {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl<'a> From<&'a str> for Key {
    fn from(val: &str) -> Self {
        Key(val.as_bytes().to_vec())
    }
}

impl From<String> for Key {
    fn from(val: String) -> Self {
        Key(val.as_bytes().to_vec())
    }
}

impl From<DateTime<Utc>> for Key {
    fn from(val: DateTime<Utc>) -> Self {
        Key(val.to_string().as_bytes().to_vec())
    }
}

//macro_rules! make_list {
//    () => (
//        None
//    );
//    ($x:expr $(, $more:expr)*) => (
//        Node::new($x, make_list!($($more),*))
//    );
//}

