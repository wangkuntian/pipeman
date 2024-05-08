use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::common::config::CONF;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerCreate {
    pub server: ServerCreateFields,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerCreateFields {
    pub name: String,
    pub networks: Vec<Network>,
    pub availability_zone: String,
    #[serde(rename(serialize = "imageRef"))]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub image_ref: String,
    #[serde(rename(serialize = "flavorRef"))]
    pub flavor_ref: String,
    pub max_count: i32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub block_device_mapping_v2: Vec<BlockDevice>,
}

impl ServerCreate {
    pub fn new(
        name: String,
        flavor: String,
        image: String,
        networks: Vec<Network>,
        availability_zone: String,
        count: i32,
    ) -> Self {
        Self {
            server: ServerCreateFields {
                name,
                networks,
                availability_zone,
                image_ref: image,
                flavor_ref: flavor,
                max_count: count,
                block_device_mapping_v2: vec![],
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockDevice {
    pub boot_index: i32,
    pub delete_on_termination: bool,
    pub source_type: String,
    pub destination_type: String,
    pub uuid: String,
}

impl BlockDevice {
    pub fn new(
        boot_index: i32,
        delete_on_termination: bool,
        source_type: String,
        destination_type: String,
        uuid: String,
    ) -> Self {
        Self {
            boot_index,
            delete_on_termination,
            source_type,
            destination_type,
            uuid,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
    pub uuid: String,
}

impl Network {
    pub fn new(multi_port: bool) -> Vec<Self> {
        let mut networks = vec![Self {
            uuid: CONF::global().openstack.external_network.to_string(),
        }];
        if multi_port {
            networks.push(Self {
                uuid: CONF::global().openstack.internal_network.to_string(),
            });
        }
        networks
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Servers {
    pub servers: Vec<ServerDetail>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Server {
    pub server: ServerDetail,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ServerDetail {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub addresses: HashMap<String, Vec<Address>>,
    #[serde(default)]
    #[serde(rename(
        deserialize = "os-extended-volumes:volumes_attached",
        serialize = "os-extended-volumes:volumes_attached"
    ))]
    pub volumes_attached: Vec<VolumeAttached>,
}
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct VolumeAttached {
    pub id: String,
}

impl ServerDetail {
    pub fn addr(&self) -> String {
        let addr = self
            .addresses
            .get(CONF::global().openstack.external_network_name.as_str())
            .unwrap();
        addr.get(0).unwrap().addr.to_string()
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Address {
    pub addr: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ServerISODetachAction {
    pub detach: ISODetachAction,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ISODetachAction {}

impl ServerISODetachAction {
    pub fn new() -> Self {
        Self {
            detach: ISODetachAction::default(),
        }
    }
}
