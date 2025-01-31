/*
All the storage related functions are implemented here.
This makes it easier to change them in the future
*/

use std::path::PathBuf;

/* Logs */
const LOGS_DIRECTORY: &str = "logs";
const LATEST_LOG_FILE: &str = "latest.log";

/* Cloudlets */
const CLOUDLETS_DIRECTORY: &str = "cloudlets";

/* Deployments */
const DEPLOYMENTS_DIRECTORY: &str = "deployments";

/* Auth */
const AUTH_DIRECTORY: &str = "auth";
const USERS_DIRECTORY: &str = "users";

/* Configs */
const CONFIG_DIRECTORY: &str = "configs";
const PRIMARY_CONFIG_FILE: &str = "config.toml";

/* Drivers */
const DRIVERS_DIRECTORY: &str = "drivers";
const DATA_DIRECTORY: &str = "data";

pub struct Storage;

impl Storage {
    /* Logs */
    pub fn get_latest_log_file() -> PathBuf {
        PathBuf::from(LOGS_DIRECTORY).join(LATEST_LOG_FILE)
    }

    /* Cloudlets */
    pub fn get_cloudlets_folder() -> PathBuf {
        PathBuf::from(CLOUDLETS_DIRECTORY)
    }
    pub fn get_cloudlet_file(name: &str) -> PathBuf {
        Storage::get_cloudlets_folder().join(format!("{}.toml", name))
    }

    /* Deployments */
    pub fn get_deployments_folder() -> PathBuf {
        PathBuf::from(DEPLOYMENTS_DIRECTORY)
    }
    pub fn get_deployment_file(name: &str) -> PathBuf {
        Storage::get_deployments_folder().join(format!("{}.toml", name))
    }

    /* Auth */
    pub fn get_users_folder() -> PathBuf {
        PathBuf::from(AUTH_DIRECTORY).join(USERS_DIRECTORY)
    }
    pub fn get_user_file(name: &str) -> PathBuf {
        Storage::get_users_folder().join(format!("{}.toml", name))
    }

    /* Configs */
    pub fn get_configs_folder() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn get_primary_config_file() -> PathBuf {
        Storage::get_configs_folder().join(PRIMARY_CONFIG_FILE)
    }

    /* Drivers */
    pub fn get_drivers_folder() -> PathBuf {
        PathBuf::from(DRIVERS_DIRECTORY)
    }
    pub fn get_data_folder_for_driver(name: &str) -> PathBuf {
        PathBuf::from(DATA_DIRECTORY).join(name)
    }
    pub fn get_config_folder_for_driver(name: &str) -> PathBuf {
        Storage::get_configs_folder().join(name)
    }
}