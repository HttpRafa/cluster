use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct BUser {
    pub id: u32,
    pub username: String,
}
