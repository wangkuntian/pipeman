use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ports {
    pub ports: Vec<PortDetail>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Port {
    pub port: PortDetail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PortDetail {
    pub id: String,
    pub name: String,
    pub status: String,
    pub fixed_ips: Vec<FixedIps>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FixedIps {
    pub ip_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PortUpdate {
    pub port: PortUpdateFields,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PortUpdateFields {
    pub port_security_enabled: bool,
    security_groups: Vec<String>,
}

impl PortUpdate {
    pub fn empty() -> Self {
        Self {
            port: PortUpdateFields {
                port_security_enabled: false,
                security_groups: vec![],
            },
        }
    }
}
