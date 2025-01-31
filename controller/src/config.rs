use std::{net::SocketAddr, str::FromStr, time::Duration};

use common::config::{LoadFromTomlFile, SaveToTomlFile};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage::Storage;

const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(150);
const DEFAULT_RESTART_TIMEOUT: Duration = Duration::from_secs(120);
const DEFAULT_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(15);
const DEFAULT_TRANSFER_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_EMPTY_INSTANCE_TIMEOUT: Duration = Duration::from_secs(60);

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:8080";

#[derive(Deserialize, Serialize)]
struct Network {
    bind: SocketAddr,
}

#[derive(Deserialize, Serialize)]
struct Timeouts {
    startup: Duration,
    restart: Duration,
    heartbeat: Duration,
    transfer: Duration,
    empty_instance: Duration,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    identifier: String,
    network: Network,
    timeouts: Timeouts,
}

impl Config {
    pub fn parse() -> Self {
        let path = Storage::get_primary_config_file();
        if path.exists() {
            Self::from_file(&path).expect("Failed to load configuration file")
        } else {
            let default = Self::default();
            default.write(&path, true).expect("Failed to write default configuration file");
            default
        }
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    pub fn get_network_bind(&self) -> &SocketAddr {
        &self.network.bind
    }

    pub fn get_startup_timeout(&self) -> &Duration {
        &self.timeouts.startup
    }

    pub fn get_restart_timeout(&self) -> &Duration {
        &self.timeouts.restart
    }

    pub fn get_heartbeat_timeout(&self) -> &Duration {
        &self.timeouts.heartbeat
    }

    pub fn get_transfer_timeout(&self) -> &Duration {
        &self.timeouts.transfer
    }

    pub fn get_empty_instance_timeout(&self) -> &Duration {
        &self.timeouts.empty_instance
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            identifier: Uuid::new_v4().to_string(),
            network: Network {
                bind: SocketAddr::from_str(DEFAULT_BIND_ADDRESS).expect("Invalid default bind address"),
            },
            timeouts: Timeouts {
                startup: DEFAULT_STARTUP_TIMEOUT,
                restart: DEFAULT_RESTART_TIMEOUT,
                heartbeat: DEFAULT_HEARTBEAT_TIMEOUT,
                transfer: DEFAULT_TRANSFER_TIMEOUT,
                empty_instance: DEFAULT_EMPTY_INSTANCE_TIMEOUT,
            },
        }
    }
}

impl LoadFromTomlFile for Config {}
impl SaveToTomlFile for Config {}