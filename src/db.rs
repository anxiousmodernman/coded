use chrono::prelude::{Date, DateTime, Utc, TimeZone};
use serde::de::Deserialize;
use rocksdb::{DB, DBVector};
use bincode::{deserialize, serialize, Infinite};
use std::convert::From;

pub trait GetAs {
    fn get_as<T>(&self, key: Key) -> Result<T, String> where for <'a>  T: Deserialize<'a>;
}

impl GetAs for DB {
    fn get_as<T>(&self, key: Key) -> Result<T, String> where for <'a> T: Deserialize<'a> {
        match self.get(key.0.as_slice()) {
            Ok(None) => Err(format!("Could not find key '{:?}' in DB", key)),
            Ok(Some(db_vec)) => deserialize(&db_vec[..]).map_err(|e|
                format!("Error deserializing key '{:?}' from DB: {:?}", key, e)
            ),
            Err(e) => Err(format!("Error fetching key '{:?}' from DB: {:?}", key, e)),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Key(pub Vec<u8>);

impl Key {
    pub fn join(&self, mut k: Key) -> Key {
        let mut left: Vec<u8> = self.0.clone();
        let mut right: Vec<u8> = k.0;
        left.append(&mut right);
        Key(left)
    }

    pub fn empty() -> Key {
        let v: Vec<u8> = Vec::new();
        Key(v)
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
        Key(val.to_rfc3339().as_bytes().to_vec())
    }
}

impl From<Date<Utc>> for Key {
    /// Here from adds in zero-values for hour, minute, second to let us work with a DateTime.
    ///
    /// We require a DateTime for it's rfc3339 representation.
    fn from(val: Date<Utc>) -> Self {
        Key(val.and_hms(0, 0, 0).to_rfc3339().as_bytes().to_vec())
    }
}

// Users of this macro must have db::Key in scope.
#[macro_export]
macro_rules! make_key {
    () => (
        Key::empty()
    );
    ($x:expr) => (
        Key::from($x)
    );
    ($x:expr $(, $more:expr)*) => (
        Key::join(&Key::from($x), make_key!($($more),*))
    );
}


#[cfg(test)]
mod test {
    #[test]
    fn test_make_key() {

        use super::*;
        use std::convert::From;

        // &str -> Key
        let mut key = make_key!("hello", String::from("world"));
        let mut expected: Vec<u8> = Vec::from("helloworld".as_bytes());
        assert_eq!(key, Key(expected));

        // NOTE: In the case of Date/DateTime, unable to get proper key equality without macro. We
        // will probably need to get stricter about how we serialize Key. We need a predictable,
        // joinable byte-level representation of a sequence of allowed types.

        // Date -> Key
        let  date = Utc.ymd(2016, 11, 8);
        let key2 = make_key!(date, "hello");
        let expected2 = make_key!(date.and_hms(0,0,0), "hello");
        assert_eq!(key2, expected2);
    }
}