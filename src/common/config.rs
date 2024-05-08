use config::builder::DefaultState;
use config::{ConfigBuilder, File};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

pub static CONFIG: OnceCell<CONF> = OnceCell::new();

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct DefaultConfig {
    pub work_dir: String,
    pub host: String,
    pub port: i32,
    pub user: String,
    pub password: String,
    pub iso_server_name: String,
    pub iso_volume_name: String,
    pub iso_volume_size: i32,
    pub server_volume_size: i32,
    pub server_prefix: String,
    pub host_prefix: String,
}
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct LogConfig {
    pub debug: bool,
    pub log_dir: String,
}

#[derive(Default, Deserialize, Debug, Serialize)]
pub struct OpenStackConfig {
    pub host: String,
    pub user: String,
    pub password: String,
    pub auth_url: String,
    pub project: String,
    pub project_id: String,
    pub domain: String,
    pub flavor: String,
    pub empty_disk_flavor: String,
    pub external_network: String,
    pub external_network_name: String,
    pub internal_network: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Amd64Config {
    pub user: String,
    pub password: String,
    pub image_id: String,
    pub image_name: String,
    pub volume_snapshot_id: String,
    pub vip_port_id: String,
    pub keepalived_router_id: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Arm64Config {
    pub host: String,
    pub user: String,
    pub password: String,
    pub image_id: String,
    pub image_name: String,
    pub volume_snapshot_id: String,
    pub vip_port_id: String,
    pub keepalived_router_id: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct IgnitionConfig {
    pub file: String,
    pub user: String,
    pub password: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct CONF {
    pub default: DefaultConfig,
    pub log: LogConfig,
    pub openstack: OpenStackConfig,
    pub amd64: Amd64Config,
    pub arm64: Arm64Config,
    pub ignition: IgnitionConfig,
}

impl CONF {
    pub fn init(config_file: &str) -> Self {
        let config = ConfigBuilder::<DefaultState>::default()
            .add_source(File::with_name(config_file))
            .build()
            .unwrap();
        Self {
            default: config.get::<DefaultConfig>("default").unwrap_or_default(),
            log: config.get::<LogConfig>("log").unwrap_or_default(),
            openstack: config
                .get::<OpenStackConfig>("openstack")
                .unwrap_or_default(),
            amd64: config.get::<Amd64Config>("amd64").unwrap_or_default(),
            arm64: config.get::<Arm64Config>("arm64").unwrap_or_default(),
            ignition: config.get::<IgnitionConfig>("ignition").unwrap_or_default(),
        }
    }
    pub fn global() -> &'static CONF {
        CONFIG.get().expect("CONF is not initialized")
    }
}
