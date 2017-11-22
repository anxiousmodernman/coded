extern crate rocksdb;

use serde::de::DeserializeOwned;
use rocksdb::{DB, DBVector};
use bincode::{deserialize, serialize, Infinite};

pub trait GetAs {
    fn get_as<'a, T>(&self, key: &str) -> Result<T, String> where T: DeserializeOwned;
}

impl GetAs for DB {
    fn get_as<'a, T>(&self, key: &str) -> Result<T, String> where T: DeserializeOwned {
        match self.get(key.as_bytes()) {
            Ok(None) => Err(format!("Could not find key '{}' in DB", key)),
            Ok(Some(db_vec)) => deserialize(&db_vec[..]).map_err(|e|
                format!("Error deserializing key '{}' from DB: {:?}", key, e)
            ),
            Err(e) => Err(format!("Error fetching key '{}' from DB: {:?}", key, e)),
        }
    }
}
