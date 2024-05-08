use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VolumeCreate {
    pub volume: VolumeCreateFields,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct VolumeCreateFields {
    pub size: i32,
    pub availability_zone: String,
    pub name: String,
}

impl VolumeCreate {
    pub fn new(name: String, size: i32, availability_zone: String) -> Self {
        Self {
            volume: VolumeCreateFields {
                name,
                size,
                availability_zone,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Volume {
    pub volume: VolumeDetail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VolumeDetail {
    pub id: String,
    pub name: String,
    pub size: i32,
    pub availability_zone: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VolumeAttachment {
    #[serde(rename(serialize = "volumeAttachment", deserialize = "volumeAttachment"))]
    pub volume_attachment: VolumeAttachmentDetail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VolumeAttachmentDetail {
    #[serde(rename(serialize = "volumeId", deserialize = "volumeId"))]
    pub volume_id: String,
    pub device: String,
    pub delete_on_termination: bool,
}

impl VolumeAttachment {
    pub fn new(volume_id: String, device: String, delete_on_termination: bool) -> Self {
        Self {
            volume_attachment: VolumeAttachmentDetail {
                volume_id,
                device,
                delete_on_termination,
            },
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct VolumeUpload {
    #[serde(rename(serialize = "os-volume_upload_image"))]
    pub volume_upload: VolumeUploadFields,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct VolumeUploadFields {
    pub image_name: String,
    pub disk_format: String,
    pub container_format: String,
    pub visibility: String,
    pub protected: bool,
    #[serde(default)]
    #[serde(skip_serializing)]
    pub image_id: String,
}

impl VolumeUpload {
    pub fn new(image_name: String) -> Self {
        Self {
            volume_upload: VolumeUploadFields {
                image_name,
                disk_format: "raw".to_string(),
                container_format: "bare".to_string(),
                visibility: "public".to_string(),
                protected: false,
                ..Default::default()
            },
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct VolumeSnapshots {
    pub snapshots: Vec<Snapshot>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct VolumeSnapshot {
    pub snapshot: Snapshot,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Snapshot {
    #[serde(skip_serializing)]
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub status: String,
    pub volume_id: String,
    #[serde(skip_deserializing)]
    pub force: bool,
}

impl VolumeSnapshot {
    pub fn new(name: String, volume_id: String, force: bool) -> Self {
        Self {
            snapshot: Snapshot {
                name,
                volume_id,
                force,
                ..Default::default()
            },
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct VolumeBootableAction {
    #[serde(rename(serialize = "os-set_bootable"))]
    pub os_set_bootable: Bootable,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Bootable {
    pub bootable: bool,
}

impl VolumeBootableAction {
    pub fn new(bootable: bool) -> Self {
        Self {
            os_set_bootable: Bootable { bootable },
        }
    }
}
