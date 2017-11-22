
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Entity {
    pub name: String,
    pub age: i32,
}

pub enum ProjectType {
    Rust,
    Go,
}
