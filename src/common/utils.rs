use chrono::prelude::*;
use ini::Ini;
use log::error;
use std::fs;
use std::path::Path;

use crate::common::exception::Result;
use crate::common::remote::RemoteClient;

pub fn get_file_size(path: &str) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

pub fn get_default_config_file() -> String {
    let mut path = Path::new("/opt/pipeman/configs/config.toml");
    if !path.is_file() {
        path = Path::new("./configs/config.toml");
        if !path.is_file() {
            let msg = "config file not exists";
            error!("{msg}");
            panic!("{msg}");
        }
    }
    path.to_string_lossy().to_string()
}

pub async fn generate_single_node_config_file(
    client: &RemoteClient,
    config_file: &str,
    password: &str,
) -> Result<()> {
    client.set_hostname("node1");
    let (nic_1, nic_2) = client.get_nics();
    let mut conf = Ini::load_from_file(config_file)?;
    let new_node = format!("node1,{},{password},{},{}", client.host, nic_1, nic_2,);
    conf.with_section(Some("node")).set("node", new_node);
    conf.write_to_file(config_file)?;
    Ok(())
}

pub async fn generate_multi_node_config_file(
    clients: &Vec<RemoteClient>,
    config_file: &str,
    password: &str,
    vip: &str,
    keepalived_router_id: &str,
) -> Result<()> {
    let mut conf = Ini::load_from_file(config_file)?;
    for (index, client) in clients.iter().enumerate() {
        let hostname = format!("node{}", index + 1);
        client.set_hostname(hostname.as_str());
        let (nic_1, nic_2) = client.get_nics();
        let new_node = format!("{hostname},{},{password},{},{}", client.host, nic_1, nic_2,);
        conf.with_section(Some("node")).set(hostname, new_node);
        conf.with_section(Some("global")).set("vip", vip);
        conf.with_section(Some("global"))
            .set("keepalived_router_id", keepalived_router_id);
        conf.write_to_file(config_file)?;
    }

    Ok(())
}

pub fn get_timestamp() -> String {
    let fmt = "%Y-%m-%d-%H-%M";
    let now = Local::now();
    now.format(fmt).to_string()
}
