extern crate rocksdb;

use serde::de::Deserialize;
use rocksdb::{DB, DBVector};
use bincode::{deserialize, serialize, Infinite};

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
    pub fn new(bytes: &[u8]) -> Key {
        Key(bytes.to_vec())
    }
}

//impl KeyComponent for Key {
//    fn join(&self, c: KeyComponent) -> Key {
//        unimplemented!()
//    }
//
//    fn as_bytes(&self) -> &[u8] {
//        unimplemented!()
//    }
//}
//
//pub trait KeyComponent {
//    fn join(&self, c: KeyComponent) -> Key;
//    fn as_bytes(&self) -> &[u8];
//}