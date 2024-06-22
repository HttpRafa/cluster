use std::{fs, path::Path};

use anyhow::Result;
use colored::Colorize;
use list::BList;
use node::BNode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use user::BUser;

use crate::{
    config::{LoadFromTomlFile, SaveToTomlFile, CONFIG_DIRECTORY}, debug, error, node::driver::http::{send_http_request, Header, Method, Response}, warn
};

mod list;
mod node;
mod user;

const BACKEND_FILE: &str = "backend.toml";

/* Endpoints */
const APPLICATION_ENDPOINT: &str = "/api/application";

#[derive(Deserialize, Serialize)]
pub struct ResolvedValues {
    pub user: u32,
}

#[derive(Deserialize, Serialize)]
pub struct Backend {
    url: Option<String>,
    token: Option<String>,
    user: Option<String>,
    resolved: Option<ResolvedValues>,
}

impl ResolvedValues {
    fn new_resolved(backend: &Backend) -> Result<Self> {
        let user = backend.get_user_by_name(&backend.user.as_ref().unwrap()).ok_or_else(|| anyhow::anyhow!("The provided user {} does not exist in the Pterodactyl panel", &backend.user.as_ref().unwrap()))?.id;
        Ok(Self {
            user,
        })
    }
}

impl Backend {
    fn new_empty() -> Self {
        Self {
            url: Some("".to_string()),
            token: Some("".to_string()),
            user: Some("".to_string()),
            resolved: None,
        }
    }

    fn load_or_empty() -> Self {
        let path = Path::new(CONFIG_DIRECTORY).join(BACKEND_FILE);
        if path.exists() {
            Self::load_from_file(&path).unwrap_or_else(|err| {
                warn!("Failed to read backend configuration from file: {}", err);
                Self::new_empty()
            })
        } else {
            let backend = Self::new_empty();     
            if let Err(error) = backend.save_to_file(&path, false) {
                error!("Failed to save default backend configuration to file: {}", &error);
            }
            backend
        }
    }

    fn new_filled() -> Result<Self> {
        let mut backend = Self::load_or_empty();

        // Check config values are overridden by environment variables
        if let Ok(url) = std::env::var("PTERODACTYL_URL") {
            backend.url = Some(url);
        }
        if let Ok(token) = std::env::var("PTERODACTYL_TOKEN") {
            backend.token = Some(token);
        }
        if let Ok(user) = std::env::var("PTERODACTYL_USER") {
            backend.user = Some(user);
        }

        let mut missing = vec![];
        if backend.url.is_none() || backend.url.as_ref().unwrap().is_empty() {
            missing.push("url");
        }
        if backend.token.is_none() || backend.token.as_ref().unwrap().is_empty() {
            missing.push("token");
        }
        if backend.user.is_none() || backend.user.as_ref().unwrap().is_empty() {
            missing.push("user");
        }
        if !missing.is_empty() {
            error!("The following required configuration values are missing: {}", missing.join(", ").red());
            return Err(anyhow::anyhow!("Missing required configuration values"));
        }

        Ok(backend)
    }

    pub fn new_filled_and_resolved() -> Result<Self> {
        let mut backend = Self::new_filled()?;
        match ResolvedValues::new_resolved(&backend) {
            Ok(resolved) => backend.resolved = Some(resolved),
            Err(error) => return Err(error),
        }
        Ok(backend)
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<BUser> {
        if let Some(response) = self.pull_list::<BUser>(Method::Get, APPLICATION_ENDPOINT, "users") {
            return response.data.iter().find(|node| node.attributes.username == username).map(|node| node.attributes.clone());
        }
        None
    }

    // TODO: This function currently only supports up to 50 nodes because pterodactyl only sends 50 per page
    pub fn get_node_by_name(&self, name: &str) -> Option<BNode> {
        if let Some(response) = self.pull_list::<BNode>(Method::Get, APPLICATION_ENDPOINT, "nodes") {
            return response.data.iter().find(|node| node.attributes.name == name).map(|node| node.attributes.clone());
        }
        None
    }

    fn pull_list<T: DeserializeOwned>(&self, method: Method, endpoint: &str, target: &str) -> Option<BList<T>> {
        self.pull_api::<BList<T>>(method, endpoint, target)
    }

    fn pull_api<T: DeserializeOwned>(&self, method: Method, endpoint: &str, target: &str) -> Option<T> {
        let url = format!("{}{}/{}", &self.url.as_ref().unwrap(), endpoint, target);
        debug!("Sending request to the pterodactyl panel: {:?} {}", method, &url);
        let response = send_http_request(method, &url, &[Header {
            key: "Authorization".to_string(),
            value: format!("Bearer {}", &self.token.as_ref().unwrap()),
        }]);
        if let Some(response) = Self::handle_response::<T>(response, 200) {
            return Some(response);
        }
        None
    }

    fn handle_response<T: DeserializeOwned>(response: Option<Response>, expected_code: u32) -> Option<T> {
        response.as_ref()?;
        let response = response.unwrap();
        if response.status_code != expected_code {
            error!("Received {} status code {} from the pterodactyl panel: {}", "unexpected".red(), &response.status_code, &response.reason_phrase);
            debug!("Response body: {}", String::from_utf8_lossy(&response.bytes));
            return None;
        }
        let response = serde_json::from_slice::<T>(&response.bytes);
        if let Err(error) = response {
            error!("{} to parse response from the pterodactyl panel: {}", "Failed".red(), &error);
            return None;
        }
        Some(response.unwrap())
    }
}

impl SaveToTomlFile for Backend {}
impl LoadFromTomlFile for Backend {}