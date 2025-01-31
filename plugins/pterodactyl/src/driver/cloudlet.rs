use common::name::TimedName;
use std::{
    cell::UnsafeCell,
    rc::Rc,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    vec,
};
use unit::PanelUnit;

use crate::{
    cloudlet::driver::types::ScopedErrors,
    error,
    exports::cloudlet::driver::bridge::{
        Address, Capabilities, GuestGenericCloudlet, RemoteController, Retention, Unit,
        UnitProposal,
    },
    info, warn,
};

use super::{
    backend::{
        allocation::BAllocation,
        server::{BServerEgg, BServerFeatureLimits},
        Backend,
    },
    PterodactylCloudletWrapper,
};

pub mod unit;

impl GuestGenericCloudlet for PterodactylCloudletWrapper {
    fn new(
        cloud_identifier: String,
        _name: String,
        id: Option<u32>,
        _capabilities: Capabilities,
        controller: RemoteController,
    ) -> Self {
        Self {
            inner: Rc::new(PterodactylCloudlet {
                cloud_identifier,
                backend: UnsafeCell::new(None),
                id: id.unwrap(),
                //name,
                //capabilities,
                controller,
                allocations: RwLock::new(vec![]),
                units: RwLock::new(vec![]),
            }),
        }
    }

    fn tick(&self) -> Result<(), ScopedErrors> {
        Ok(())
    }

    /* This method expects that the Pterodactyl Allocations are only accessed by one atomic cloud instance */
    fn allocate_addresses(&self, unit: UnitProposal) -> Result<Vec<Address>, String> {
        let amount = unit.resources.addresses;

        let mut used = self.inner.get_allocations_mut();
        if unit.spec.disk_retention == Retention::Permanent {
            let name = TimedName::new(&self.inner.cloud_identifier, &unit.name, true);

            // Check if a unit with the same name is already exists
            if let Some(backend_unit) = self.inner.get_backend().get_server_by_name(&name) {
                // Get the allocations that are already used by this server
                let mut allocations = self
                    .inner
                    .get_backend()
                    .get_allocations_by_server(&backend_unit.identifier);

                if (allocations.1.len() + 1) as u32 != unit.resources.addresses {
                    warn!("The unit {} has a different amount of addresses than the panel has allocated. This may cause issues.", unit.name);
                    // TODO: Add a way to fix this
                }

                allocations.1.insert(0, allocations.0); // Add primary allocation to the list
                allocations
                    .1
                    .iter()
                    .for_each(|address| used.push(address.into()));
                return Ok(allocations.1.into_iter().map(|x| x.into()).collect());
            }
        }

        let allocations = self
            .inner
            .get_backend()
            .get_free_allocations(&used, self.inner.id, amount)
            .iter()
            .map(|allocation| {
                used.push(allocation.clone());
                Address {
                    host: allocation.get_host().clone(),
                    port: allocation.port,
                }
            })
            .collect();
        Ok(allocations)
    }

    fn deallocate_addresses(&self, addresses: Vec<Address>) {
        self.inner.get_allocations_mut().retain(|x| {
            !addresses
                .iter()
                .any(|address| *x.get_host() == address.host && x.port == address.port)
        });
    }

    fn start_unit(&self, unit: Unit) {
        let spec = &unit.allocation.spec;
        let name = TimedName::new(
            &self.inner.cloud_identifier,
            &unit.name,
            spec.disk_retention == Retention::Permanent,
        );

        let allocations = unit
            .allocation
            .addresses
            .iter()
            .map_while(|address| match self.inner.find_allocation(address) {
                Some(allocation) => Some(allocation),
                None => {
                    error!(
                        "Allocation({:?}) not found for unit {}",
                        &unit.allocation.addresses[0], unit.name
                    );
                    None
                }
            })
            .collect::<Vec<_>>();

        // Check if a unit with the same name is already exists
        if let Some(backend_unit) = self.inner.get_backend().get_server_by_name(&name) {
            if spec.disk_retention == Retention::Temporary {
                error!(
                    "Unit <blue>{}</> already exists on the panel, but the disk retention is temporary",
                    unit.name
                );
                return;
            }
            // Just use the existing unit and change its settings
            info!(
                "Unit <blue>{}</> already exists on the panel, updating settings and starting...",
                unit.name
            );
            self.inner
                .get_backend()
                .update_settings(self, allocations[0].id, &backend_unit, &unit);
            self.inner
                .get_backend()
                .start_server(&backend_unit.identifier);
            self.inner.get_units_mut().push(PanelUnit::new(
                backend_unit.id,
                backend_unit.identifier,
                name,
            ));
        } else {
            let mut egg = None;
            let mut startup = None;
            for value in &spec.settings {
                match value.key.as_str() {
                    "egg" => match value.value.parse::<u32>() {
                        Ok(id) => {
                            egg = Some(id);
                        }
                        Err(_) => {
                            error!("The egg setting must be a number!");
                        }
                    },
                    "startup" => {
                        startup = Some(value.value.clone());
                    }
                    _ => {}
                }
            }

            let mut missing = vec![];
            if egg.is_none() {
                missing.push("egg");
            }
            if startup.is_none() {
                missing.push("startup");
            }
            if !missing.is_empty() {
                error!(
                    "The following required settings to start the unit are missing: <red>{}</>",
                    missing.join(", ")
                );
                return;
            }

            // Create a new unit
            if let Some(unit) = self.inner.get_backend().create_server(
                &name,
                &unit,
                self,
                &allocations,
                BServerEgg {
                    id: egg.unwrap(),
                    startup: startup.unwrap(),
                },
                BServerFeatureLimits {
                    databases: 0,
                    backups: 0,
                },
            ) {
                info!(
                    "Unit <blue>{}</> successfully <green>created</> on the panel",
                    unit.name,
                );
                self.inner
                    .get_units_mut()
                    .push(PanelUnit::new(unit.id, unit.identifier, name));
            }
        }
    }

    fn restart_unit(&self, unit: Unit) {
        if let Some(backend_unit) = self.inner.find_unit(&unit.name) {
            self.inner.get_backend().restart_server(&backend_unit);
            info!(
                "Panel is <yellow>restarting</> the unit <blue>{}</>...",
                backend_unit.name.get_name(),
            );
        } else {
            error!(
                "<red>Failed</> to restart unit <blue>{}</> because the unit was <red>never started</> by this driver",
                unit.name,
            );
        }
    }

    fn stop_unit(&self, unit: Unit) {
        if let Some(backend_unit) = self.inner.find_unit(&unit.name) {
            if unit.allocation.spec.disk_retention == Retention::Temporary {
                self.inner.get_backend().delete_server(backend_unit.id);
                info!(
                    "Unit <blue>{}</> successfully <red>deleted</> from the panel",
                    backend_unit.name.get_name()
                );
            } else {
                self.inner.get_backend().stop_server(&backend_unit);
                info!(
                    "Panel is <red>stopping</> the unit <blue>{}</>...",
                    backend_unit.name.get_name(),
                );
            }
            self.inner.delete_unit(backend_unit.id);
        } else {
            error!(
                "<red>Failed</> to stop unit <blue>{}</> because the unit was <red>never started</> by this driver",
                unit.name
            );
        }
    }
}

pub struct PterodactylCloudlet {
    /* Cloud Identification */
    pub cloud_identifier: String,

    /* Informations about the cloudlet */
    pub backend: UnsafeCell<Option<Rc<Backend>>>,
    pub id: u32,
    //pub name: String,
    //pub capabilities: Capabilities,
    pub controller: RemoteController,

    /* Dynamic Resources */
    pub allocations: RwLock<Vec<BAllocation>>,
    pub units: RwLock<Vec<PanelUnit>>,
}

impl PterodactylCloudlet {
    fn get_backend(&self) -> &Rc<Backend> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.backend.get() }.as_ref().unwrap()
    }

    fn get_allocations(&self) -> RwLockReadGuard<Vec<BAllocation>> {
        // Safe as we are only run on the same thread
        self.allocations.read().unwrap()
    }
    fn get_allocations_mut(&self) -> RwLockWriteGuard<Vec<BAllocation>> {
        // Safe as we are only run on the same thread
        self.allocations.write().unwrap()
    }
    fn get_units(&self) -> RwLockReadGuard<Vec<PanelUnit>> {
        // Safe as we are only run on the same thread
        self.units.read().unwrap()
    }
    fn get_units_mut(&self) -> RwLockWriteGuard<Vec<PanelUnit>> {
        // Safe as we are only run on the same thread
        self.units.write().unwrap()
    }

    fn find_allocation(&self, address: &Address) -> Option<BAllocation> {
        self.get_allocations()
            .iter()
            .find(|allocation| {
                *allocation.get_host() == address.host && allocation.port == address.port
            })
            .cloned()
    }
    fn find_unit(&self, name: &str) -> Option<PanelUnit> {
        self.get_units()
            .iter()
            .find(|unit| unit.name.get_raw_name() == name)
            .cloned()
    }
    fn delete_unit(&self, id: u32) {
        self.get_units_mut().retain(|unit| unit.id != id);
    }
}
