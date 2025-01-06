use backend::Backend;
use cloudlet::PterodactylCloudlet;
use std::cell::UnsafeCell;
use std::rc::Rc;
use std::sync::RwLock;

use crate::exports::cloudlet::driver::bridge::{
    Capabilities, GenericCloudlet, GuestGenericCloudlet, GuestGenericDriver, Information,
    RemoteController,
};
use crate::{error, info};

pub mod cloudlet;

mod backend;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];

pub struct Pterodactyl {
    /* Cloud Identification */
    cloud_identifier: String,

    /* Backend */
    backend: UnsafeCell<Option<Rc<Backend>>>,

    /* Cloudlets that this driver handles */
    cloudlets: RwLock<Vec<Rc<PterodactylCloudlet>>>,
}

impl Pterodactyl {
    fn get_backend(&self) -> &Rc<Backend> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.backend.get() }.as_ref().unwrap()
    }
}

impl GuestGenericDriver for Pterodactyl {
    fn new(cloud_identifier: String) -> Self {
        Self {
            cloud_identifier,
            backend: UnsafeCell::new(None),
            cloudlets: RwLock::new(Vec::new()),
        }
    }

    fn init(&self) -> Information {
        let backend = Backend::new_filled_and_resolved();
        if let Err(error) = &backend {
            error!(
                "Failed to initialize Pterodactyl driver: <red>{}</>",
                error.to_string()
            );
        }
        unsafe { *self.backend.get() = backend.ok().map(Rc::new) };
        Information {
            authors: AUTHORS.iter().map(|&author| author.to_string()).collect(),
            version: VERSION.to_string(),
            ready: unsafe { &*self.backend.get() }.is_some(),
        }
    }

    fn init_cloudlet(
        &self,
        name: String,
        capabilities: Capabilities,
        controller: RemoteController,
    ) -> Result<GenericCloudlet, String> {
        if let Some(value) = capabilities.child.as_ref() {
            if let Some(cloudlet) = self.get_backend().get_node_by_name(value) {
                let wrapper = PterodactylCloudletWrapper::new(
                    self.cloud_identifier.clone(),
                    name.clone(),
                    Some(cloudlet.id),
                    capabilities,
                    controller,
                );
                unsafe { *wrapper.inner.backend.get() = Some(self.get_backend().clone()) }
                // Add cloudlet to cloudlets list
                let mut cloudlets = self
                    .cloudlets
                    .write()
                    .expect("Failed to get lock on cloudlets");
                cloudlets.push(wrapper.inner.clone());
                info!(
                    "Cloudlet <blue>{}</>[<blue>{}</>] was <green>added</>",
                    name, cloudlet.id
                );
                Ok(GenericCloudlet::new(wrapper))
            } else {
                Err("Node does not exist in the Pterodactyl panel".to_string())
            }
        } else {
            Err("Cloudlet lacks the required child capability".to_string())
        }
    }
}

pub struct PterodactylCloudletWrapper {
    pub inner: Rc<PterodactylCloudlet>,
}

impl PterodactylCloudletWrapper {
    fn get_backend(&self) -> &Rc<Backend> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.inner.backend.get() }.as_ref().unwrap()
    }
}
