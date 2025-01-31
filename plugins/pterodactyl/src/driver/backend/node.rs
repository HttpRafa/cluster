use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct BNode {
    pub id: u32,
    pub name: String,
}
