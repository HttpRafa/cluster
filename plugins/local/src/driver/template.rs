use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{anyhow, Result};
use common::config::LoadFromTomlFile;
use serde::{Deserialize, Serialize};
use stored::StoredTemplate;
use walkdir::WalkDir;

use crate::{
    cloudlet::driver::{
        platform::{get_os, Os},
        process::{
            drop_process, read_line_direct, spawn_process, try_wait, write_stdin, KeyValue,
            ReaderMode, StdReader,
        },
        types::Directory,
    },
    info,
    storage::Storage,
    warn,
};

pub struct Templates {
    /* Templates */
    pub templates: Vec<Rc<Template>>,
}

impl Templates {
    pub fn new() -> Self {
        Self { templates: vec![] }
    }

    pub fn load_all(&mut self) {
        info!("Loading templates...");

        let templates_directory = Storage::get_templates_folder();
        if !templates_directory.exists() {
            if let Err(error) = fs::create_dir_all(&templates_directory) {
                warn!(
                    "<red>Failed</> to create templates directory: <red>{}</>",
                    error
                );
                return;
            }
        }

        let entries = match fs::read_dir(&templates_directory) {
            Ok(entries) => entries,
            Err(error) => {
                warn!(
                    "<red>Failed</> to read templates directory: <red>{}</>",
                    &error
                );
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    warn!(
                        "<red>Failed</> to read entry in templates directory: <red>{}</>",
                        &error
                    );
                    continue;
                }
            };

            let path = entry.path();
            if path.is_file() {
                continue;
            }

            let name: String = match path.file_stem() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            let data_file = Storage::get_template_data_file(&name);
            let template = match StoredTemplate::from_file(&data_file) {
                Ok(profile) => profile,
                Err(error) => {
                    warn!(
                        "<red>Failed</> to load template from file {}: <red>{}</>",
                        path.display(),
                        &error
                    );
                    continue;
                }
            };

            let template = Template::from(&name, &template);
            info!(
                "Loaded template <blue>{} {}</> by <blue>{}</>",
                name,
                template.version,
                template.authors.join(", ")
            );

            self.add_template(template);
        }

        info!("Loaded <blue>{} template(s)</>", self.templates.len());
    }

    pub fn prepare_all(&mut self) {
        info!("Running preperation scripts...");

        for template in &self.templates {
            info!(
                "Running preperation script for template <blue>{}</>",
                template.name
            );

            template.run_prepare();
        }
    }

    pub fn get_template_by_name(&self, name: &str) -> Option<Rc<Template>> {
        self.templates
            .iter()
            .find(|template| template.name == name)
            .cloned()
    }

    fn add_template(&mut self, template: Template) {
        self.templates.push(Rc::new(template));
    }
}

#[derive(Clone)]
pub struct Template {
    /* Template */
    pub name: String,
    //pub description: String,
    pub version: String,
    pub authors: Vec<String>,

    /* Files */
    pub exclusions: Vec<PathBuf>,

    /* Environment */
    pub environment: Vec<KeyValue>,

    /* Commands */
    pub shutdown: Option<String>,

    /* Scripts */
    pub prepare: PlatformScript,
    pub startup: PlatformScript,
}

impl Template {
    fn from(name: &str, stored: &StoredTemplate) -> Self {
        Self {
            name: name.to_string(),
            //description: stored.description.clone(),
            version: stored.version.clone(),
            authors: stored.authors.clone(),
            exclusions: stored.exclusions.clone(),
            environment: stored
                .environment
                .iter()
                .map(|value| KeyValue {
                    key: value.0.clone(),
                    value: value.1.clone(),
                })
                .collect(),
            shutdown: stored.shutdown.clone(),
            prepare: stored.prepare.clone(),
            startup: stored.startup.clone(),
        }
    }

    pub fn run_prepare(&self) {
        let prepare = match get_os() {
            Os::Unix => &self.prepare.unix,
            Os::Windows => &self.prepare.windows,
        };

        let prepare = match prepare {
            Some(script) => script,
            None => {
                warn!(
                    "The template <blue>{}</> does not seem to support the current platform",
                    self.name
                );
                return;
            }
        };

        match spawn_process(
            &prepare.command,
            &prepare.args,
            &self.environment,
            &Storage::get_template_folder_host_converted(&self.name),
            ReaderMode::Direct,
        ) {
            Ok(pid) => {
                while try_wait(pid).ok().flatten().is_none() {
                    match read_line_direct(pid, StdReader::Stdout) {
                        Ok(read) => {
                            if read.0 > 0 {
                                let line = read.1.trim();
                                info!("<blue>[PREPARE]</> {}", line);
                            }
                        }
                        Err(error) => {
                            warn!(
                                "Failed to read stdout of process <blue>{}</>: <red>{}</>",
                                pid, error
                            );
                            break;
                        }
                    }
                }
                drop_process(pid).expect("Failed to drop process. This indicates that something is wrong with the controller");
            }
            Err(error) => {
                warn!(
                    "Failed to prepare template <blue>{}</>: <red>{}</>",
                    self.name, error
                );
            }
        }
    }

    pub fn run_startup(&self, directory: &Directory, environment: &[KeyValue]) -> Result<u32> {
        let startup = match get_os() {
            Os::Unix => &self.startup.unix,
            Os::Windows => &self.startup.windows,
        };

        let startup = match startup {
            Some(script) => script,
            None => {
                return Err(anyhow!(
                    "The template {} does not seem to support the current platform",
                    self.name
                ))
            }
        };
        let mut environment = environment.to_vec();
        environment.extend_from_slice(&self.environment);

        spawn_process(
            &startup.command,
            &startup.args,
            &environment,
            directory,
            ReaderMode::Async,
        )
        .map_err(|error| anyhow!("Failed to execute entrypoint {}: {}", self.name, error))
    }

    pub fn run_shutdown(&self, pid: u32) -> Result<()> {
        match &self.shutdown {
            Some(command) => {
                write_stdin(pid, command.as_bytes()).map_err(|error| {
                    anyhow!(
                        "Failed to send shutdown command to process {}: {}",
                        pid,
                        error
                    )
                })?;
            }
            None => {
                write_stdin(pid, b"^C").map_err(|error| {
                    anyhow!("Failed to send Ctl+C command to process {}: {}", pid, error)
                })?;
            }
        }
        Ok(())
    }

    pub fn copy_to_folder(&self, folder: &Path) -> Result<()> {
        let source = Storage::get_template_folder(&self.name);
        fs::create_dir_all(folder)?;

        for entry in WalkDir::new(&source) {
            let entry = entry?;
            let path = entry.path();

            // Skip directories as they'll be created automatically with files
            if path.is_dir() {
                continue;
            }

            // Calculate the relative path
            let relative_path = path.strip_prefix(&source).unwrap();

            // Check if the relative path is in the exclusions list
            if self
                .exclusions
                .iter()
                .any(|exclusion| exclusion == relative_path)
            {
                continue;
            }

            // Construct the destination path
            let destination = folder.join(relative_path);

            // Ensure the destination directory exists
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }

            // Copy the file
            fs::copy(path, &destination)?;
        }

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlatformScript {
    pub unix: Option<Script>,
    pub windows: Option<Script>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Script {
    pub command: String,
    pub args: Vec<String>,
}

mod stored {
    use std::{collections::HashMap, path::PathBuf};

    use common::config::{LoadFromTomlFile, SaveToTomlFile};
    use serde::{Deserialize, Serialize};

    use super::PlatformScript;

    #[derive(Serialize, Deserialize)]
    pub struct StoredTemplate {
        /* Template */
        pub description: String,
        pub version: String,
        pub authors: Vec<String>,

        /* Files */
        pub exclusions: Vec<PathBuf>,

        /* Environment */
        pub environment: HashMap<String, String>,

        /* Commands */
        pub shutdown: Option<String>,

        /* Scripts */
        pub prepare: PlatformScript,
        pub startup: PlatformScript,
    }

    impl LoadFromTomlFile for StoredTemplate {}
    impl SaveToTomlFile for StoredTemplate {}
}
