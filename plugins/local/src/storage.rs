/*
All the storage related functions are implemented here.
This makes it easier to change them in the future
*/

use std::path::PathBuf;

use common::name::TimedName;

use crate::{
    cloudlet::driver::types::{Directory, Reference},
    exports::cloudlet::driver::bridge::Retention,
};

/* Configs */
const CONFIG_DIRECTORY: &str = "/configs";
const PRIMARY_CONFIG_FILE: &str = "config.toml";

/* Data */
const DATA_DIRECTORY: &str = "/data";
//const TEMPLATES_DIRECTORY: &str = "templates";
const UNITS_DIRECTORY: &str = "units";

/* Templates */
const TEMPLATES_DIRECTORY: &str = "templates";
const TEMPLATE_DATA_FILE: &str = "template.toml";

/* Units */
const TEMPORARY_DIRECTORY: &str = "temporary";
const PERMANENT_DIRECTORY: &str = "permanent";

pub struct Storage;

impl Storage {
    /* Configs */
    pub fn get_configs_folder() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn get_primary_config_file() -> PathBuf {
        Storage::get_configs_folder().join(PRIMARY_CONFIG_FILE)
    }

    /* Data */
    pub fn get_data_folder() -> PathBuf {
        PathBuf::from(DATA_DIRECTORY)
    }
    pub fn get_units_folder() -> PathBuf {
        Self::get_data_folder().join(UNITS_DIRECTORY)
    }
    pub fn get_units_folder_host() -> PathBuf {
        PathBuf::from(UNITS_DIRECTORY)
    }

    /* Templates */
    pub fn get_templates_folder() -> PathBuf {
        Self::get_data_folder().join(TEMPLATES_DIRECTORY)
    }
    pub fn get_templates_folder_host() -> PathBuf {
        PathBuf::from(TEMPLATES_DIRECTORY)
    }
    pub fn get_template_folder(name: &str) -> PathBuf {
        Self::get_data_folder().join(TEMPLATES_DIRECTORY).join(name)
    }
    pub fn get_template_folder_host_converted(name: &str) -> Directory {
        Directory {
            reference: Reference::Data,
            path: Self::get_templates_folder_host()
                .join(name)
                .to_string_lossy()
                .to_string(),
        }
    }
    pub fn get_template_data_file(name: &str) -> PathBuf {
        Self::get_template_folder(name).join(TEMPLATE_DATA_FILE)
    }

    /* Units */
    pub fn get_temporary_folder() -> PathBuf {
        Self::get_units_folder().join(TEMPORARY_DIRECTORY)
    }
    pub fn get_temporary_folder_host() -> PathBuf {
        Self::get_units_folder_host().join(TEMPORARY_DIRECTORY)
    }
    pub fn get_temporary_folder_host_converted() -> Directory {
        Directory {
            reference: Reference::Data,
            path: Self::get_temporary_folder_host()
                .to_string_lossy()
                .to_string(),
        }
    }
    pub fn get_permanent_folder() -> PathBuf {
        Self::get_units_folder().join(PERMANENT_DIRECTORY)
    }
    pub fn get_permanent_folder_host() -> PathBuf {
        Self::get_units_folder_host().join(PERMANENT_DIRECTORY)
    }

    pub fn get_unit_folder(name: &TimedName, retention: &Retention) -> PathBuf {
        if retention == &Retention::Permanent {
            Self::get_permanent_folder().join(name.get_name())
        } else {
            Self::get_temporary_folder().join(name.get_name())
        }
    }

    pub fn get_unit_folder_host_converted(name: &TimedName, retention: &Retention) -> Directory {
        let path = if retention == &Retention::Permanent {
            Self::get_permanent_folder_host().join(name.get_name())
        } else {
            Self::get_temporary_folder_host().join(name.get_name())
        };
        Directory {
            reference: Reference::Data,
            path: path.to_string_lossy().to_string(),
        }
    }
}
