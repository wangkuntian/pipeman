use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, Debug, Display, PartialEq, EnumString, Serialize, Deserialize)]
pub enum OSArches {
    #[strum(serialize = "x86_64")]
    #[serde(rename(deserialize = "x86_64"))]
    X8664,
    #[strum(serialize = "aarch64")]
    #[serde(rename(deserialize = "aarch64"))]
    AARCH64,
}

#[derive(Clone, Debug, Display, PartialEq, EnumString, Serialize, Deserialize)]
pub enum CPUModes {
    #[strum(serialize = "custom")]
    #[serde(rename(deserialize = "custom"))]
    Custom,
    #[strum(serialize = "host-model")]
    #[serde(rename(deserialize = "host-model"))]
    HostModel,
    #[strum(serialize = "host-passthrough")]
    #[serde(rename(deserialize = "host-passthrough"))]
    HostPassthrough,
}

#[derive(Clone, Debug, Display, PartialEq, EnumString, Serialize, Deserialize)]
pub enum DiskDeviceTypes {
    #[strum(serialize = "cdrom")]
    #[serde(rename(serialize = "cdrom"))]
    #[serde(rename(deserialize = "cdrom"))]
    Cdrom,
    #[strum(serialize = "disk")]
    #[serde(rename(serialize = "disk"))]
    #[serde(rename(deserialize = "disk"))]
    Disk,
}

#[derive(Clone, Debug, Display, PartialEq, EnumString, Serialize, Deserialize)]
pub enum DiskBus {
    #[strum(serialize = "ide")]
    #[serde(rename(deserialize = "ide"))]
    Ide,
    #[strum(serialize = "virtio")]
    #[serde(rename(deserialize = "virtio"))]
    VirtIO,
}

#[derive(Clone, Debug, Display, PartialEq, EnumString, Serialize, Deserialize)]
pub enum CPUMatches {
    #[strum(serialize = "exact")]
    #[serde(rename(deserialize = "exact"))]
    Exact,
    #[strum(serialize = "none")]
    #[serde(rename(deserialize = "none"))]
    None,
}