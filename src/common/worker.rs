use futures::future::join_all;
use log::{error, info};
use tokio::join;

use crate::common::args::{Arches, DeployArgs, DeployMode};
use crate::common::config::CONF;
use crate::common::remote::RemoteClient;
use crate::common::{command, utils};
use crate::openstack::models::network::Ports;
use crate::openstack::models::server::Server;
use crate::openstack::models::volume::{Volume, VolumeAttachment, VolumeSnapshot};
use crate::openstack::request::OpenStackClient;

#[derive(Debug, Default, Clone)]
pub struct Worker {
    pub client: OpenStackClient,
    pub remote: RemoteClient,
}

impl Worker {
    pub async fn new() -> Worker {
        Self {
            client: OpenStackClient::new().await,
            remote: RemoteClient::get_available_remote_client(
                CONF::global().default.user.as_str(),
                CONF::global().default.password.as_str(),
                CONF::global().default.host.as_str().to_string(),
                CONF::global().default.port,
            )
            .await,
        }
    }

    pub async fn create_iso_image_on_remote(&self, image_name: &str) -> String {
        info!("creating image {image_name}");
        let work_dir = CONF::global().default.work_dir.as_str();
        let image_file = format!("{work_dir}/{image_name}");
        let image_id = self
            .remote
            .create_image("iso", image_file.as_str(), image_name);
        info!("image id is {image_id}");
        self.client
            .wait_image_become(image_id.as_str(), "active")
            .await;
        image_id
    }

    pub async fn create_available_volume(&self, name: &str, size: i32) -> Volume {
        let volume = self.client.create_volume(name, size, "nova").await;
        self.client
            .wait_volume_become(volume.volume.id.as_str(), "available")
            .await;
        volume
    }

    pub async fn create_active_server(
        &self,
        name: &str,
        flavor: &str,
        image: String,
        az: &str,
        snapshot_id: Option<String>,
        multi_port: bool,
    ) -> Server {
        let server = self
            .client
            .create_server(name, flavor, image.as_str(), az, snapshot_id, multi_port)
            .await;
        let server = self
            .client
            .wait_server_to_become(server.server.id.as_str(), "ACTIVE")
            .await;
        server
    }

    pub async fn attach_volume(&self, server_id: &str, volume_id: &str) -> VolumeAttachment {
        let attachment = self
            .client
            .add_volume_to_server(server_id, volume_id, true)
            .await;
        attachment
    }

    pub async fn create_available_volume_snapshot(
        &self,
        volume_id: &str,
        snapshot_name: &str,
    ) -> VolumeSnapshot {
        let snapshot = self
            .client
            .create_volume_snapshot(snapshot_name, volume_id)
            .await;
        let snapshot_id = snapshot.snapshot.id.as_str();
        self.client
            .wait_volume_snapshot_become(snapshot_id, "available")
            .await;
        info!("volume snapshot {snapshot_id} create success");
        snapshot
    }

    pub async fn remove_volume_from_server(&self, server_id: &str, volume_id: &str) {
        self.client
            .remove_volume_from_server(server_id, volume_id)
            .await;
        self.client.wait_volume_become(volume_id, "available").await;
    }

    pub async fn set_volume_bootable(&self, volume_id: &str) {
        self.client.set_volume_bootable(volume_id, true).await;
    }

    pub async fn clear_port_security_groups(&self, ports: Ports) {
        let tasks: Vec<_> = ports
            .ports
            .iter()
            .map(|port| self.client.clear_port_security_groups(port.id.as_str()))
            .collect();
        let _ = join_all(tasks).await;
    }

    pub async fn create_new_server(
        &self,
        server_name: String,
        volume_name: String,
        flavor: &str,
        az: &str,
        snapshot_id: &str,
        volume_size: i32,
    ) -> Server {
        let server = self
            .create_active_server(
                server_name.as_str(),
                flavor,
                String::new(),
                az,
                Some(snapshot_id.to_string()),
                true,
            )
            .await;
        let ports = self.client.get_ports(server.server.id.as_str()).await;
        self.clear_port_security_groups(ports).await;
        let volume = self
            .create_available_volume(volume_name.as_str(), volume_size)
            .await;
        self.attach_volume(server.server.id.as_str(), volume.volume.id.as_str())
            .await;
        server
    }

    pub async fn create_new_servers(
        &self,
        name_prefix: String,
        flavor: &str,
        az: &str,
        snapshot_id: &str,
        count: i32,
        volume_size: i32,
    ) -> Vec<Server> {
        info!("creating servers");
        let tasks: Vec<_> = (1..count + 1)
            .collect::<Vec<i32>>()
            .iter()
            .map(|x| {
                let server_name = format!("{name_prefix}-{x}");
                let volume_name = format!("{name_prefix}-volume-{x}");
                self.create_new_server(
                    server_name,
                    volume_name,
                    flavor,
                    az,
                    snapshot_id,
                    volume_size,
                )
            })
            .collect();
        let servers = join_all(tasks).await;
        info!("creating servers");
        servers
    }

    pub async fn get_remote_clients(
        &self,
        hosts: &Vec<String>,
        user: &str,
        password: &str,
    ) -> Vec<RemoteClient> {
        let tasks: Vec<_> = hosts
            .iter()
            .map(|x| RemoteClient::get_available_remote_client(user, password, x.to_string(), 22))
            .collect();
        join_all(tasks).await
    }

    pub async fn get_vip(&self, port_id: &str) -> String {
        let port = self.client.show_port(port_id).await;
        let fixed_ips = port.port.fixed_ips;
        if !fixed_ips.is_empty() {
            fixed_ips[0].ip_address.to_string()
        } else {
            error!("port {port_id} have no ip");
            panic!("port {port_id} have no ip");
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Deployment {
    pub arch: Arches,
    pub mode: DeployMode,
    pub count: i32,
    pub user: String,
    pub password: String,
    pub az: String,
    pub image_id: String,
    pub image_name: String,
    pub volume_snapshot_id: String,
    pub config_file: String,
    pub worker: Worker,
    pub timestamp: String,
    pub volume_size: i32,
    pub iso_volume_size: i32,
    pub vip: String,
    pub hosts: Vec<String>,
    pub keepalived_router_id: String,
}

impl Deployment {
    pub async fn new(args: &DeployArgs) -> Self {
        let mut hosts: Vec<String> = Vec::new();
        if let Some(host) = &args.hosts {
            hosts = host.split(",").map(|x| x.to_string()).collect();
        }
        let (
            image_id,
            image_name,
            az,
            volume_snapshot_id,
            user,
            password,
            vip_port_id,
            keepalived_router_id,
        ) = match args.arch {
            Arches::Amd64 => (
                CONF::global().amd64.image_id.as_str(),
                CONF::global().amd64.image_name.as_str(),
                "nova",
                CONF::global().amd64.volume_snapshot_id.as_str(),
                CONF::global().amd64.user.as_str(),
                CONF::global().amd64.password.as_str(),
                CONF::global().amd64.vip_port_id.as_str(),
                CONF::global().amd64.keepalived_router_id.as_str(),
            ),
            Arches::Arm64 => (
                CONF::global().arm64.image_id.as_str(),
                CONF::global().arm64.image_name.as_str(),
                "nova-arm",
                CONF::global().arm64.volume_snapshot_id.as_str(),
                CONF::global().arm64.user.as_str(),
                CONF::global().arm64.password.as_str(),
                CONF::global().arm64.vip_port_id.as_str(),
                CONF::global().arm64.keepalived_router_id.as_str(),
            ),
        };
        let (count, config_file) = match args.mode {
            DeployMode::AllInOne => {
                if hosts.len() != 1 {
                    panic!("all in one mode need only one host")
                } else {
                    (
                        1,
                        format!("{}/configs/all-in-one.ini", CONF::global().default.work_dir),
                    )
                }
            }
            DeployMode::MultiNode => {
                if hosts.len() != 3 {
                    panic!("multi node mode need 3 hosts")
                } else {
                    (
                        3,
                        format!("{}/configs/multinode.ini", CONF::global().default.work_dir),
                    )
                }
            }
        };
        let worker = Worker::new().await;
        let vip = worker.get_vip(vip_port_id).await;
        Self {
            arch: args.arch,
            mode: args.mode,
            count,
            user: user.to_string(),
            password: password.to_string(),
            az: az.to_string(),
            image_id: image_id.to_string(),
            image_name: image_name.to_string(),
            volume_snapshot_id: volume_snapshot_id.to_string(),
            config_file,
            worker,
            timestamp: utils::get_timestamp(),
            volume_size: CONF::global().default.server_volume_size,
            iso_volume_size: CONF::global().default.iso_volume_size,
            vip,
            hosts,
            keepalived_router_id: keepalived_router_id.to_string(),
        }
    }

    pub fn snapshot_name(&self, volume_id: &str) -> String {
        format!("{}-volume-{volume_id}-snapshot", self.timestamp)
    }

    pub async fn get_remote_client(&self, host: &str) -> RemoteClient {
        RemoteClient::get_available_remote_client(
            self.user.as_str(),
            self.password.as_str(),
            host.to_string(),
            22,
        )
        .await
    }

    pub async fn install_uswift(&self, host: &str, device: &str) {
        let work_dir = CONF::global().default.work_dir.as_str();
        let ignition_file = CONF::global().ignition.file.as_str();
        let file = format!("{work_dir}/configs/{ignition_file}");
        let remote = self.get_remote_client(host).await;
        info!("upload {file} to {host}");
        remote.upload(file.as_str(), "/root/uswift.ign");
        info!("upload {file} to {host} success");
        info!("start installation");
        match remote.exec_long_command(command::install_uswift(device).as_str(), true) {
            Ok(_) => {}
            Err(_) => {
                error!("install failed");
                info!("start installation again");
                match remote.exec_long_command(command::install_uswift(device).as_str(), true) {
                    Ok(_) => {}
                    Err(_) => {
                        error!("install failed");
                        panic!("install failed");
                    }
                };
            }
        };
        info!("install success");
        match remote.exec_command("reboot") {
            Ok(_) => {}
            Err(_) => {}
        };
        let _ = self.get_remote_client(host).await;
    }

    pub async fn setup_uswift_server(&self, image: String) -> (String, String) {
        let flavor = CONF::global().openstack.empty_disk_flavor.as_str();
        let iso_server_name = format!(
            "{}-{}",
            self.timestamp,
            CONF::global().default.iso_server_name.as_str()
        );
        let volume_name = format!(
            "{}-{}",
            self.timestamp,
            CONF::global().default.iso_volume_name.as_str()
        );
        let (server, volume) = join!(
            self.worker.create_active_server(
                iso_server_name.as_str(),
                flavor,
                image,
                self.az.as_str(),
                None,
                false,
            ),
            self.worker
                .create_available_volume(volume_name.as_str(), self.iso_volume_size)
        );
        let server_id = server.server.id.as_str();
        let volume_id = volume.volume.id.as_str();
        let host = server.server.addr();
        let attachment = self.worker.attach_volume(server_id, volume_id).await;
        self.worker.set_volume_bootable(volume_id).await;
        self.install_uswift(host.as_str(), attachment.volume_attachment.device.as_str())
            .await;
        (server.server.id, volume.volume.id)
    }

    pub async fn deploy_on_specific_hosts(&self) {
        info!("deploy ustack on {:?}", self.hosts);
        if self.hosts.is_empty() {
            error!("hosts is empty");
            panic!("hosts is empty")
        }
        let clients = self
            .worker
            .get_remote_clients(&self.hosts, self.user.as_str(), self.password.as_str())
            .await;
        let master = &clients[0];
        if self.count == 1 {
            let _ = utils::generate_single_node_config_file(
                master,
                self.config_file.as_str(),
                self.password.as_str(),
            )
            .await;
        } else {
            let _ = utils::generate_multi_node_config_file(
                &clients,
                self.config_file.as_str(),
                self.password.as_str(),
                self.vip.as_str(),
                self.keepalived_router_id.as_str(),
            )
            .await;
        }
        master.upload(self.config_file.as_str(), "/etc/ustack-deploy/config.ini");
        master.deploy_ustack(command::deploy(self.mode));
    }

    async fn deploy(&mut self) {
        info!(
            "creating new servers with volume snapshot {}",
            self.volume_snapshot_id
        );
        let flavor = CONF::global().openstack.flavor.as_str();
        let server_name_prefix = format!(
            "{}-{}",
            self.timestamp,
            CONF::global().default.server_prefix
        );
        let servers = self
            .worker
            .create_new_servers(
                server_name_prefix,
                flavor,
                self.az.as_str(),
                self.volume_snapshot_id.as_str(),
                self.count,
                self.volume_size,
            )
            .await;
        let hosts: Vec<String> = servers.iter().map(|x| x.server.addr()).collect();
        info!("deploy ustack on {:?}", hosts);
        self.hosts = hosts;
        self.deploy_on_specific_hosts().await;
    }

    pub async fn execute(&mut self) -> i32 {
        if !self.hosts.is_empty() {
            self.deploy_on_specific_hosts().await;
            return 0;
        }
        if self.volume_snapshot_id.is_empty() {
            if self.image_id.is_empty() {
                self.image_id = self
                    .worker
                    .create_iso_image_on_remote(self.image_name.as_str())
                    .await;
            }
            let (server_id, volume_id) = self.setup_uswift_server(self.image_id.to_string()).await;
            println!("{server_id}, {volume_id}");
            let volume_snapshot = self
                .worker
                .create_available_volume_snapshot(
                    volume_id.as_str(),
                    self.snapshot_name(volume_id.as_str()).as_str(),
                )
                .await;
            self.volume_snapshot_id = volume_snapshot.snapshot.id;
        }
        self.deploy().await;
        0
    }
}
