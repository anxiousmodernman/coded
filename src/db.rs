extern crate rocksdb;

use serde::de::DeserializeOwned;
use rocksdb::{DB, DBVector};
use bincode::{deserialize, serialize, Infinite};

pub trait GetAs {
    fn get_as<'a, T>(&self, key: &str) -> Result<T, String> where T: DeserializeOwned;
}

impl GetAs for DB {
    fn get_as<'a, T>(&self, key: &str) -> Result<T, String> where T: DeserializeOwned {
        let found_dbvec = self.get(key.as_bytes())
            .map_err(|e| format!("Error fetching key '{}' from DB: {:?}", key, e))?;
        let dbvec = found_dbvec
            .ok_or(format!("Could not find key '{}' in DB", key))?;
        deserialize(&dbvec[..])
            .map_err(|e| format!("Error deserializing key '{}' from DB: {:?}", key, e))?
    }
}
