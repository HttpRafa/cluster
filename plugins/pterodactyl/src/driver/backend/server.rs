use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::exports::cloudlet::driver::bridge::Resources;

use super::allocation::BAllocation;

pub struct BServerEgg {
    pub id: u32,
    pub startup: String,
}

/* Create Server on panel */
#[derive(Serialize, Clone)]
pub struct BCServer {
    pub name: String,
    pub node: u32,
    pub user: u32,
    pub egg: u32,
    pub docker_image: String,
    pub startup: String,
    pub environment: HashMap<String, String>,
    pub limits: BServerLimits,
    pub feature_limits: BServerFeatureLimits,
    pub allocation: BCServerAllocation,
    pub start_on_completion: bool,
}

#[derive(Serialize, Clone)]
pub struct BCServerAllocation {
    pub default: u32,
    pub additional: Vec<u32>,
}

impl BCServerAllocation {
    pub fn from(allocations: &[BAllocation]) -> BCServerAllocation {
        let mut additional = Vec::with_capacity(allocations.len() - 1);
        for item in allocations.iter().skip(1) {
            additional.push(item.id);
        }
        Self {
            default: allocations[0].id,
            additional,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct BSignal {
    pub signal: String,
}

#[derive(Serialize, Clone)]
pub struct BKeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Clone)]
pub struct BServer {
    pub id: u32,
    pub identifier: String,
    pub name: String,
}

/* Generic Data of Server */
#[derive(Deserialize, Serialize, Clone)]
pub struct BServerLimits {
    pub memory: u32,
    pub swap: u32,
    pub disk: u32,
    pub io: u32,
    pub cpu: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BServerFeatureLimits {
    pub databases: u32,
    pub backups: u32,
}

impl From<Resources> for BServerLimits {
    fn from(resources: Resources) -> Self {
        Self {
            memory: resources.memory,
            swap: resources.swap,
            disk: resources.disk,
            io: resources.io,
            cpu: resources.cpu,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct BUpdateBuild {
    pub allocation: u32,
    pub memory: u32,
    pub swap: u32,
    pub disk: u32,
    pub io: u32,
    pub cpu: u32,
    pub threads: Option<()>, // Used to generate null in the JSON
    pub feature_limits: BServerFeatureLimits,
}
