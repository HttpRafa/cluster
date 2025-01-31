use std::{
    cell::UnsafeCell,
    rc::Rc,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::Result;
use common::{allocator::NumberAllocator, name::TimedName, tick::TickResult};
use unit::LocalUnit;

use crate::{
    cloudlet::driver::types::{ErrorMessage, ScopedError, ScopedErrors},
    error,
    exports::cloudlet::driver::bridge::{
        Address, Capabilities, GuestGenericCloudlet, RemoteController, Retention, Unit,
        UnitProposal,
    },
    info,
    storage::Storage,
};

use super::{config::Config, template::Templates, LocalCloudletWrapper};

pub mod unit;

impl LocalCloudlet {
    pub fn tick(&self) -> Result<(), ScopedErrors> {
        let mut units = self.get_units_mut();
        let mut errors = ScopedErrors::new();
        units.retain_mut(|unit| match unit.tick() {
            Ok(result) => result == TickResult::Ok,
            Err(err) => {
                errors.push(ScopedError {
                    scope: unit.name.get_raw_name().to_string(),
                    message: err.to_string(),
                });
                true
            }
        });
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl GuestGenericCloudlet for LocalCloudletWrapper {
    fn new(
        _cloud_identifier: String,
        _name: String,
        _id: Option<u32>,
        _capabilities: Capabilities,
        controller: RemoteController,
    ) -> Self {
        Self {
            inner: Rc::new(LocalCloudlet {
                _name,
                config: UnsafeCell::new(None),
                controller,
                templates: UnsafeCell::new(None),
                port_allocator: UnsafeCell::new(None),
                units: RwLock::new(vec![]),
            }),
        }
    }

    fn tick(&self) -> Result<(), ScopedErrors> {
        self.inner.tick()
    }

    fn allocate_addresses(&self, unit: UnitProposal) -> Result<Vec<Address>, ErrorMessage> {
        let amount = unit.resources.addresses;

        let mut ports = Vec::with_capacity(amount as usize);
        let mut allocator = self
            .inner
            .get_port_allocator()
            .write()
            .expect("Failed to lock port allocator");
        for _ in 0..amount {
            if let Some(port) = allocator.allocate() {
                ports.push(Address {
                    host: self.inner.get_config().address.clone(),
                    port,
                });
            } else {
                return Err("Failed to allocate ports".to_string());
            }
        }

        Ok(ports)
    }

    fn deallocate_addresses(&self, addresses: Vec<Address>) {
        let mut allocator = self
            .inner
            .get_port_allocator()
            .write()
            .expect("Failed to lock port allocator");
        for address in addresses {
            allocator.release(address.port);
        }
    }

    fn start_unit(&self, unit: Unit) {
        let spec = &unit.allocation.spec;
        let name =
            TimedName::new_no_identifier(&unit.name, spec.disk_retention == Retention::Permanent);

        let template = match self
            .inner
            .get_templates()
            .read()
            .expect("Failed to lock templates")
            .get_template_by_name(&spec.image)
        {
            Some(template) => template,
            None => {
                error!(
                    "Template <blue>{}</> not found for unit <blue>{}</>",
                    &spec.image,
                    name.get_name()
                );
                return;
            }
        };

        let folder = Storage::get_unit_folder(&name, &spec.disk_retention);
        if !folder.exists() {
            if let Err(err) = template.copy_to_folder(&folder) {
                error!(
                    "Failed to copy template for unit <blue>{}</>: <red>{}</>",
                    name.get_name(),
                    err
                );
                return;
            }
        }

        let mut local_unit = LocalUnit::new(self, unit, &name, template);
        if let Err(err) = local_unit.start() {
            error!(
                "Failed to start unit <blue>{}</>: <red>{}</>",
                name.get_raw_name(),
                err
            );
            return;
        }

        info!(
            "Successfully <green>created</> child process for unit <blue>{}</>",
            name.get_raw_name()
        );
        self.inner.get_units_mut().push(local_unit);
    }

    fn restart_unit(&self, unit: Unit) {
        let mut units = self.inner.get_units_mut();
        if let Some(local_unit) = units
            .iter_mut()
            .find(|u| u.name.get_raw_name() == unit.name)
        {
            if let Err(err) = local_unit.restart() {
                error!(
                    "<red>Failed</> to restart unit <blue>{}</>: <red>{}</>",
                    unit.name, err
                );
                return;
            }
            info!(
                "Child process of unit <blue>{}</> is <yellow>restarting</>",
                unit.name
            );
        } else {
            error!("<red>Failed</> to restart unit <blue>{}</>: Unit was <red>never started</> by this driver", unit.name);
        }
    }

    fn stop_unit(&self, unit: Unit) {
        let mut units = self.inner.get_units_mut();
        if let Some(local_unit) = units
            .iter_mut()
            .find(|u| u.name.get_raw_name() == unit.name)
        {
            if unit.allocation.spec.disk_retention == Retention::Temporary {
                if let Err(err) = local_unit.kill() {
                    error!(
                        "<red>Failed</> to stop unit <blue>{}</>: <red>{}</>",
                        unit.name, err
                    );
                    return;
                }
                info!(
                    "Child process of unit <blue>{}</> was <red>killed</>",
                    unit.name
                );
            } else {
                if let Err(err) = local_unit.stop() {
                    error!(
                        "<red>Failed</> to stop unit <blue>{}</>: <red>{}</>",
                        unit.name, err
                    );
                    return;
                }
                info!(
                    "Child process of unit <blue>{}</> is <red>stopping</>",
                    unit.name
                );
            }
        } else {
            error!("<red>Failed</> to stop unit <blue>{}</>: Unit was <red>never started</> by this driver", unit.name);
        }
    }
}

pub struct LocalCloudlet {
    /* Informations about the cloudlet */
    _name: String,
    pub config: UnsafeCell<Option<Rc<Config>>>,
    controller: RemoteController,

    /* Templates */
    pub templates: UnsafeCell<Option<Rc<RwLock<Templates>>>>,

    /* Dynamic Resources */
    pub port_allocator: UnsafeCell<Option<Rc<RwLock<NumberAllocator<u16>>>>>,
    units: RwLock<Vec<LocalUnit>>,
}

impl LocalCloudlet {
    /* Dispose */
    pub fn try_exit(&self, force: bool) -> Result<TickResult, ScopedErrors> {
        if force {
            let mut units = self.get_units_mut();
            let mut errors = ScopedErrors::new();
            for unit in units.iter_mut() {
                if let Err(error) = unit.kill() {
                    errors.push(ScopedError {
                        scope: unit.name.get_raw_name().to_string(),
                        message: error.to_string(),
                    });
                }
            }
            if !errors.is_empty() {
                return Err(errors);
            }
        }
        match self.tick() {
            Ok(()) => {
                if self.get_units().is_empty() {
                    Ok(TickResult::Drop)
                } else {
                    Ok(TickResult::Ok)
                }
            }
            Err(errors) => Err(errors),
        }
    }

    fn get_config(&self) -> &Rc<Config> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.config.get() }.as_ref().unwrap()
    }
    fn get_templates(&self) -> &Rc<RwLock<Templates>> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.templates.get() }.as_ref().unwrap()
    }
    fn get_port_allocator(&self) -> &Rc<RwLock<NumberAllocator<u16>>> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.port_allocator.get() }.as_ref().unwrap()
    }
    fn get_units(&self) -> RwLockReadGuard<Vec<LocalUnit>> {
        // Safe as we are only run on the same thread
        self.units.read().unwrap()
    }
    fn get_units_mut(&self) -> RwLockWriteGuard<Vec<LocalUnit>> {
        // Safe as we are only run on the same thread
        self.units.write().unwrap()
    }
}
