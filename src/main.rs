#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocksdb;
extern crate bincode;
#[macro_use]
extern crate serde_derive;

use std::ops::Add;

use rocksdb::DB;
use rocket::State;

use bincode::{serialize, deserialize, Infinite};

fn main() {
    use config::load;
    use std::path::Path;
    
    let mut db = DB::open_default(".coded.db").unwrap();
    let routes = routes![index];
    rocket::ignite().mount("/", routes).manage(db).launch();
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Entity  {
    name: String, 
    age: i32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World(Vec<Entity>);

#[get("/")]
fn index(db: State<DB>) -> String {

    // Now we know how to serialize/deserialize.

    let me = Entity{name: String::from("coleman"), age: 33};
    let encoded: Vec<u8> = serialize(&me, Infinite).unwrap();
    let k = "k2";
    db.put(k.as_bytes(), encoded.as_slice());

    let name = {
    match db.get(k.as_bytes()) {
        Ok(Some(db_vec)) => {
            let decoded: Entity = deserialize(&db_vec[..]).unwrap();
            let name = decoded.name.as_str();
            String::from("Hello, world! ").add(name).add(" specifically")
        },
        _ => String::from("error!!!!")
    }

    };
    name
}


unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}


