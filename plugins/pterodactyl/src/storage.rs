/*
All the storage related functions are implemented here.
This makes it easier to change them in the future
*/

use std::path::PathBuf;

/* Configs */
const CONFIG_DIRECTORY: &str = "/configs";
const BACKEND_CONFIG_FILE: &str = "backend.toml";

pub struct Storage;

impl Storage {
    /* Configs */
    pub fn get_configs_folder() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn get_backend_config_file() -> PathBuf {
        Storage::get_configs_folder().join(BACKEND_CONFIG_FILE)
    }
}
