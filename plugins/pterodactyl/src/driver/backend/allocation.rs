use serde::Deserialize;

use crate::exports::cloudlet::driver::bridge::Address;

#[derive(Deserialize, Clone)]
pub struct BAllocation {
    pub id: u32,
    pub port: u16,
    pub assigned: bool,
    ip: String,
    alias: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct BCAllocation {
    pub id: u32,
    pub port: u16,
    pub is_default: bool,
    ip: String,
    ip_alias: Option<String>,
}

impl BAllocation {
    pub fn get_host(&self) -> &String {
        self.alias.as_ref().unwrap_or(&self.ip)
    }
}

impl BCAllocation {
    pub fn get_host(&self) -> &String {
        self.ip_alias.as_ref().unwrap_or(&self.ip)
    }
}

impl From<BCAllocation> for Address {
    fn from(val: BCAllocation) -> Self {
        Address {
            host: val.ip,
            port: val.port,
        }
    }
}

impl From<&BCAllocation> for BAllocation {
    fn from(val: &BCAllocation) -> Self {
        BAllocation {
            id: val.id,
            ip: val.ip.clone(),
            alias: val.ip_alias.clone(),
            port: val.port,
            assigned: true,
        }
    }
}
