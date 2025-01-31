use std::{ops::Range, time::Duration};

use common::config::{LoadFromTomlFile, SaveToTomlFile};
use serde::{Deserialize, Serialize};

use crate::{error, storage::Storage, warn};

/* Timeouts */
pub const UNIT_STOP_TIMEOUT: Duration = Duration::from_secs(30);
pub const CLEANUP_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Serialize, Deserialize)]
pub struct Config {
    /* Network */
    pub address: String,
    pub ports: Range<u16>,
}

impl Config {
    fn new_empty() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            ports: 27000..28000,
        }
    }

    pub fn new_filled() -> Self {
        let path = Storage::get_primary_config_file();
        if path.exists() {
            Self::from_file(&path).unwrap_or_else(|err| {
                warn!("Failed to read configuration from file: {}", err);
                Self::new_empty()
            })
        } else {
            let config = Self::new_empty();
            if let Err(error) = config.write(&path, false) {
                error!("Failed to save default configuration to file: {}", &error);
            }
            config
        }
    }
}

impl LoadFromTomlFile for Config {}
impl SaveToTomlFile for Config {}
