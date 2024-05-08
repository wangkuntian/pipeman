use log::{debug, error, info};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Method, Response, StatusCode,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::time::Duration;

use crate::common::{
    config::CONF,
    exception::{Exception, Result},
};
use crate::openstack::models::auth::AuthPayload;
use crate::openstack::models::image::{Image, Images};
use crate::openstack::models::network::{Port, PortUpdate, Ports};
use crate::openstack::models::server::{
    BlockDevice, Network, Server, ServerCreate, ServerISODetachAction,
};
use crate::openstack::models::volume::{
    Volume, VolumeAttachment, VolumeBootableAction, VolumeCreate, VolumeSnapshot, VolumeSnapshots,
    VolumeUpload,
};

#[derive(Debug, Default, Clone)]
pub struct OpenStackClient {
    pub client: Client,
}

impl OpenStackClient {
    pub async fn new() -> Self {
        let token = match Self::auth_token().await {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                panic!("{e}");
            }
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Auth-Token",
            HeaderValue::from_str(token.as_str()).unwrap(),
        );
        headers.insert(
            "OpenStack-API-Version",
            HeaderValue::from_str("volume 3.62").unwrap(),
        );
        headers.insert(
            "X-OpenStack-Nova-API-Version",
            HeaderValue::from_str("2.79").unwrap(),
        );
        let client = Client::builder().default_headers(headers).build().unwrap();
        Self { client }
    }

    pub fn base_url(&self, port: i32) -> String {
        format!("http://{}:{port}", CONF::global().openstack.host)
    }
    fn neutron_url(&self, resource: &str) -> String {
        format!("{}/v2.0/{resource}", self.base_url(9696))
    }
    fn glance_url(&self, resource: &str) -> String {
        format!("{}/v2/{resource}", self.base_url(9292))
    }
    pub fn nova_url(&self, resource: &str) -> String {
        format!("{}/v2.1/{resource}", self.base_url(8774))
    }
    pub fn cinder_url(&self, resource: &str) -> String {
        format!(
            "{}/v3/{}/{resource}",
            self.base_url(8776),
            CONF::global().openstack.project_id
        )
    }

    async fn handle_response_status(
        response: Response,
        expected_code: StatusCode,
    ) -> Result<StatusCode> {
        if response.status() != expected_code {
            let error = response.text().await?;
            Err(Exception::Error(error))
        } else {
            Ok(response.status())
        }
    }

    async fn request_status(
        &self,
        url: String,
        method: Method,
        expected_code: StatusCode,
    ) -> Result<StatusCode> {
        let response = self.client.request(method, url).send().await?;
        Self::handle_response_status(response, expected_code).await
    }

    async fn request_status_with_data<D>(
        &self,
        url: String,
        method: Method,
        expected_code: StatusCode,
        data: D,
    ) -> Result<StatusCode>
    where
        D: Serialize,
    {
        let response = self.client.request(method, url).json(&data).send().await?;
        Self::handle_response_status(response, expected_code).await
    }

    async fn handle_response_json<T>(response: Response, expected_code: StatusCode) -> Result<T>
    where
        T: Debug + DeserializeOwned,
    {
        if response.status() != expected_code {
            let error = response.text().await?;
            Err(Exception::Error(error))
        } else {
            let data = response.json::<T>().await?;
            info!("{:?}", data);
            Ok(data)
        }
    }

    async fn request_json<T>(
        &self,
        url: String,
        method: Method,
        expected_code: StatusCode,
    ) -> Result<T>
    where
        T: Debug + DeserializeOwned,
    {
        let response = self.client.request(method, url).send().await?;
        Self::handle_response_json::<T>(response, expected_code).await
    }

    async fn request_json_with_data<T, D>(
        &self,
        url: String,
        method: Method,
        expected_code: StatusCode,
        data: D,
    ) -> Result<T>
    where
        T: Debug + DeserializeOwned,
        D: Serialize,
    {
        let response = self.client.request(method, url).json(&data).send().await?;
        Self::handle_response_json(response, expected_code).await
    }

    pub async fn list_images(&self) -> Images {
        info!("list images");
        self.request_json::<Images>(self.glance_url("images"), Method::GET, StatusCode::OK)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            })
    }

    pub async fn show_image(&self, image_id: &str) -> Image {
        info!("show image {image_id}");
        let url = format!("{}/{image_id}", self.glance_url("images"));
        self.request_json::<Image>(url, Method::GET, StatusCode::OK)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            })
    }
    pub async fn delete_image(&self, image_id: &str) {
        info!("delete image {image_id}");
        let url = format!("{}/{image_id}", self.glance_url("images"));
        let _ = self
            .request_status(url, Method::DELETE, StatusCode::OK)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}")
            });
    }

    pub async fn auth_token() -> Result<String> {
        let user = CONF::global().openstack.user.to_string();
        let password = CONF::global().openstack.password.to_string();
        let auth = AuthPayload::new(user, password);
        let url = format!("{}/auth/tokens", CONF::global().openstack.auth_url);
        let response = Client::new().post(url).body(auth).send().await?;
        let token = response
            .headers()
            .get("x-subject-token")
            .ok_or(Exception::AuthenticationException)?;
        let token = token.to_str().unwrap_or_default().to_string();
        info!("token is {token}");
        Ok(token)
    }

    pub async fn wait_image_become(&self, image_id: &str, expected_status: &str) {
        info!("wait image {image_id} become {expected_status}");
        loop {
            let image = self.show_image(image_id).await;
            if image.status == expected_status {
                info!("image {image_id} become {expected_status}");
                break;
            } else {
                info!("wait 3 seconds for image {image_id}");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }

    pub async fn show_server(&self, server_id: &str) -> Server {
        debug!("show server {server_id}");
        let url = format!("{}/{server_id}", self.nova_url("servers"));
        self.request_json::<Server>(url, Method::GET, StatusCode::OK)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}")
            })
    }
    pub async fn create_server(
        &self,
        name: &str,
        flavor: &str,
        image: &str,
        az: &str,
        snapshot_id: Option<String>,
        multi_port: bool,
    ) -> Server {
        info!("create server {name}");
        let mut payload = ServerCreate::new(
            name.to_string(),
            flavor.to_string(),
            image.to_string(),
            Network::new(multi_port),
            az.to_string(),
            1,
        );
        if let Some(snapshot) = snapshot_id {
            let block_device = BlockDevice::new(
                0,
                true,
                "snapshot".to_string(),
                "volume".to_string(),
                snapshot,
            );
            payload.server.block_device_mapping_v2 = vec![block_device];
        }
        let server = self
            .request_json_with_data::<Server, ServerCreate>(
                self.nova_url("servers"),
                Method::POST,
                StatusCode::ACCEPTED,
                payload,
            )
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            });
        server
    }

    pub async fn wait_server_to_become(&self, server_id: &str, expected_status: &str) -> Server {
        info!("wait server {server_id} become {expected_status}");
        let mut timeout = 5;
        loop {
            let server = self.show_server(server_id).await;
            if server.server.status == expected_status.to_string() {
                info!("server {server_id} become {}", expected_status);
                return server;
            } else {
                info!("wait {timeout} seconds for server {server_id}");
                tokio::time::sleep(Duration::from_secs(timeout)).await;
                timeout += 5;
            }
            if timeout >= 1200 {
                let e = format!(
                    "{timeout} seconds timeout to wait server {server_id} become {expected_status}",
                );
                error!("{e}");
                panic!("{e}");
            }
        }
    }

    pub async fn show_volume(&self, volume_id: &str) -> Volume {
        let url = format!("{}/{volume_id}", self.cinder_url("volumes"));
        self.request_json(url, Method::GET, StatusCode::OK)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}")
            })
    }

    pub async fn create_volume(&self, name: &str, size: i32, az: &str) -> Volume {
        info!("create volume {name}");
        let payload = VolumeCreate::new(name.to_string(), size, az.to_string());
        let volume = self
            .request_json_with_data::<Volume, VolumeCreate>(
                self.cinder_url("volumes"),
                Method::POST,
                StatusCode::ACCEPTED,
                payload,
            )
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            });
        volume
    }
    pub async fn wait_volume_become(&self, volume_id: &str, expected_status: &str) {
        info!("wait volume {volume_id} become {expected_status}");
        loop {
            let volume = self.show_volume(volume_id).await;
            if volume.volume.status == expected_status {
                info!("volume {volume_id} become {expected_status}");
                break;
            } else {
                info!("wait 3 seconds for volume {volume_id}");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }
    pub async fn add_volume_to_server(
        &self,
        server_id: &str,
        volume_id: &str,
        delete_on_termination: bool,
    ) -> VolumeAttachment {
        info!("add volume {volume_id} to server {server_id}");
        let url = format!(
            "{}/{server_id}/os-volume_attachments",
            self.nova_url("servers")
        );
        let data = VolumeAttachment::new(
            volume_id.to_string(),
            "/dev/vda".to_string(),
            delete_on_termination,
        );
        let attachment = self
            .request_json_with_data::<VolumeAttachment, VolumeAttachment>(
                url,
                Method::POST,
                StatusCode::OK,
                data,
            )
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            });
        attachment
    }

    pub async fn remove_volume_from_server(&self, server_id: &str, volume_id: &str) {
        info!("remove volume {volume_id} from server {server_id}");
        let url = format!(
            "{}/{server_id}/os-volume_attachments/{volume_id}",
            self.nova_url("servers")
        );
        let _ = self
            .request_status(url, Method::DELETE, StatusCode::ACCEPTED)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}")
            });
    }

    pub async fn set_volume_bootable(&self, volume_id: &str, bootable: bool) {
        info!("set volume {volume_id} bootable {bootable}");
        let url = format!("{}/{volume_id}/action", self.cinder_url("volumes"));
        let data = VolumeBootableAction::new(bootable);
        let _ = self
            .request_status_with_data(url, Method::POST, StatusCode::OK, data)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}")
            });
    }

    pub async fn upload_volume_to_image(&self, volume_id: &str, image_name: &str) -> String {
        let url = format!("{}/{volume_id}/action", self.cinder_url("volumes"));
        let data = VolumeUpload::new(image_name.to_string());
        let response = self
            .client
            .post(url)
            .json(&data)
            .send()
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            });
        let data = response.text().await.unwrap();
        info!("{data}");
        "123".to_string()
    }

    pub async fn create_volume_snapshot(&self, name: &str, volume_id: &str) -> VolumeSnapshot {
        info!("creating volume {volume_id} snapshot");
        let data = VolumeSnapshot::new(name.to_string(), volume_id.to_string(), true);
        let snapshot = self
            .request_json_with_data::<VolumeSnapshot, VolumeSnapshot>(
                self.cinder_url("snapshots"),
                Method::POST,
                StatusCode::ACCEPTED,
                data,
            )
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            });
        snapshot
    }

    pub async fn list_volume_snapshots(&self, volume_id: &str) -> VolumeSnapshots {
        let url = format!(
            "{}/detail?volume_id={volume_id}&status=available",
            self.cinder_url("snapshots")
        );
        let snapshots = self
            .request_json::<VolumeSnapshots>(url, Method::GET, StatusCode::OK)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            });
        snapshots
    }

    pub async fn show_volume_snapshot(&self, snapshot_id: &str) -> VolumeSnapshot {
        let url = format!("{}/{snapshot_id}", self.cinder_url("snapshots"));
        let snapshot = self
            .request_json::<VolumeSnapshot>(url, Method::GET, StatusCode::OK)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            });
        snapshot
    }
    pub async fn wait_volume_snapshot_become(&self, snapshot_id: &str, expected_status: &str) {
        info!("wait snapshot {snapshot_id} become {expected_status}");
        loop {
            let snapshot = self.show_volume_snapshot(snapshot_id).await;
            if snapshot.snapshot.status == expected_status {
                info!("snapshot {snapshot_id} become {expected_status}");
                break;
            } else {
                info!("wait 3 seconds for snapshot {snapshot_id}");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }

    pub async fn detach_iso_from_server(&self, server_id: &str) {
        info!("detach iso from server {server_id}");
        let data = ServerISODetachAction::new();
        let url = format!("{}/{server_id}/action", self.nova_url("servers"));
        let _ = self
            .request_status_with_data(url, Method::POST, StatusCode::ACCEPTED, data)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            });
    }

    pub async fn show_port(&self, port_id: &str) -> Port {
        info!("show port {port_id}");
        let url = format!("{}/{port_id}", self.neutron_url("ports"));
        self.request_json::<Port>(url, Method::GET, StatusCode::OK)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            })
    }

    pub async fn get_ports(&self, device_id: &str) -> Ports {
        info!("get port with device_id {device_id}");
        let url = format!("{}?device_id={device_id}", self.neutron_url("ports"));
        self.request_json::<Ports>(url, Method::GET, StatusCode::OK)
            .await
            .unwrap_or_else(|e| {
                error!("{e}");
                panic!("{e}");
            })
    }

    pub async fn clear_port_security_groups(&self, port_id: &str) -> Port {
        info!("clear port {port_id} security groups");
        let url = format!("{}/{port_id}", self.neutron_url("ports"));
        self.request_json_with_data::<Port, PortUpdate>(
            url,
            Method::PUT,
            StatusCode::OK,
            PortUpdate::empty(),
        )
        .await
        .unwrap_or_else(|e| {
            error!("{e}");
            panic!("{e}");
        })
    }
}
