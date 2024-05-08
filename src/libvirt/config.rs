use quick_xml::se::Serializer;
use serde::Serialize;
use crate::libvirt::device::{Devices};
use crate::libvirt::enumerations::{CPUMatches, CPUModes, OSArches};

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename = "domain")]
pub struct Guest {
    #[serde(rename = "@type")]
    pub domain_type: String,
    pub name: String,
    pub vcpu: i32,
    pub memory: f64,
    pub os: OSConfig,
    pub cpu: CPUConfig,
    pub devices: DevicesConfig,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename = "os")]
pub struct OSConfig {
    pub r#type: OSTypeConfig,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename = "os")]
pub struct OSTypeConfig {
    #[serde(rename = "$value")]
    pub os_type: String,
    pub boot: Boot,
}

impl OSTypeConfig {
    pub fn new(os_type: String, dev: String) -> Self {
        Self { os_type, boot: Boot { dev } }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Boot {
    #[serde(rename = "@dev")]
    pub dev: String,
}


#[derive(Debug, PartialEq, Serialize)]
pub struct CPUConfig {
    #[serde(rename = "@mode")]
    pub mode: String,
    #[serde(rename = "@match")]
    pub r#match: String,
    pub topology: CPUTopology,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct CPUTopology {
    #[serde(rename = "@sockets")]
    pub sockets: String,
    #[serde(rename = "@cores")]
    pub cores: String,
    #[serde(rename = "@threads")]
    pub threads: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DevicesConfig {
    #[serde(rename = "$value")]
    pub device: Vec<Devices>,
}


impl Guest {
    pub fn new(name: String, vcpu: i32, memory: f64) -> Self {
        Self {
            name,
            domain_type: "kvm".to_string(),
            memory,
            vcpu,
            devices: DevicesConfig {
                device: vec![]
            },
            os: OSConfig {
                r#type: OSTypeConfig::new(
                    "hvm".to_string(),
                    "cdrom".to_string(),
                )
            },
            cpu: CPUConfig {
                mode: CPUModes::HostPassthrough.to_string(),
                r#match: CPUMatches::Exact.to_string(),
                topology: CPUTopology {
                    sockets: vcpu.to_string(),
                    cores: "1".to_string(),
                    threads: "1".to_string(),
                },
            },
        }
    }

    pub fn add_device(&mut self, device: Devices) {
        self.devices.device.push(device)
    }

    pub fn to_xml(&self) -> String {
        let mut buffer = String::new();
        let ser = Serializer::with_root(&mut buffer, None).unwrap();
        self.serialize(ser).unwrap();
        buffer
    }
}