use anyhow::Result;

pub mod manager;
mod runtime;

pub trait GenericPlugin {
    fn init(&self) -> Result<Information>;
}

pub struct Information {
    authors: Vec<String>,
    version: String,
    ready: bool,
}