use std::sync::Arc;
use std::ops::Add;

use bincode::{deserialize, serialize, Infinite};
use rocket::State;
use rocksdb::DB;

use db::GetAs;

use models::Entity;


#[get("/")]
fn index(db: State<Arc<DB>>) -> String {
    let me = Entity {
        name: String::from("coleman"),
        age: 33,
    };
    let encoded: Vec<u8> = serialize(&me, Infinite).unwrap();
    let k = "k2";
    db.put(k.as_bytes(), encoded.as_slice());

    let decoded: Entity = db.get_as(k).unwrap();

    let name = {
        let name = decoded.name.as_str();
        String::from("Hello, world! ")
            .add(name)
            .add(" specifically")
    };
    name
}
