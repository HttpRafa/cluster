use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct BMetadata {
    pub pagination: BPagination,
}

#[derive(Deserialize)]
pub struct BPagination {
    pub total_pages: u32,
}

#[derive(Deserialize)]
pub struct BBody<T> {
    pub data: T,
    pub meta: Option<BMetadata>,
}

#[derive(Deserialize, Serialize)]
pub struct BObject<T> {
    pub attributes: T,
}

pub type BList<T> = BBody<Vec<BObject<T>>>;
