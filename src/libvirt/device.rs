use serde::Serialize;
use crate::libvirt::enumerations::{DiskBus, DiskDeviceTypes};

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Devices {
    Disk {
        #[serde(rename = "@type")]
        disk_type: String,
        #[serde(rename = "@device")]
        device_type: DiskDeviceTypes,
        driver: DiskDeviceDriver,
        source: DiskDeviceSource,
        target: DiskDeviceTarget,
        boot: DiskDeviceBoot,
    },
    Network,
    Serial {
        #[serde(rename = "@type")]
        serial_type: String,
    },
    Graphics {
        #[serde(rename = "@type")]
        graphics_type: String,
        #[serde(rename = "@port")]
        port: String,
        #[serde(rename = "@listen")]
        listen: String,
    },
    Video {
        model: VideoDeviceModel,
    },
}

impl Devices {
    pub fn new_disk(
        disk_type: String,
        device_type: DiskDeviceTypes,
        disk_driver_type: String,
        source: String,
        target: String,
        boot_order: String,
    ) -> Self {
        Self::Disk {
            disk_type,
            device_type,
            driver: DiskDeviceDriver {
                name: "qemu".to_string(),
                driver_type: disk_driver_type,
            },
            source: DiskDeviceSource { file: source },
            target: DiskDeviceTarget { dev: target, bus: DiskBus::VirtIO.to_string() },
            boot: DiskDeviceBoot { order: boot_order },
        }
    }

    pub fn new_raw_disk(
        device_type: DiskDeviceTypes,
        source: String,
        target: String,
        boot_order: String,
    ) -> Self {
        Self::new_disk("file".to_string(), device_type, "raw".to_string(),
                       source, target, boot_order)
    }

    pub fn new_serial() -> Self {
        Self::Serial { serial_type: "pty".to_string() }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DiskDeviceConfig {
    pub driver: DiskDeviceDriver,
    pub source: DiskDeviceSource,
    pub target: DiskDeviceTarget,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DiskDeviceDriver {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@type")]
    pub driver_type: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DiskDeviceSource {
    #[serde(rename = "@file")]
    pub file: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DiskDeviceTarget {
    #[serde(rename = "@dev")]
    pub dev: String,
    #[serde(rename = "@bus")]
    pub bus: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct DiskDeviceBoot {
    #[serde(rename = "@order")]
    pub order: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct VideoDeviceModel {
    #[serde(rename = "@type")]
    pub model_type: String,
}