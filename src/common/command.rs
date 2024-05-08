use crate::common::args::DeployMode;

static PRE_COMMAND: &str = "source /root/admin-openrc.sh";
static IMAGE_CREATE_COMMAND: &str = "openstack image create -f value -c id";

pub fn create_image(disk_format: &str, path: &str, image_name: &str) -> String {
    let command = format!(
        "{} && {} --disk-format {} --container-format bare --public --file {} {}",
        PRE_COMMAND, IMAGE_CREATE_COMMAND, disk_format, path, image_name
    );
    command
}

pub fn install_uswift(device: &str) -> String {
    format!("sudo uswift-installer install -i uswift.ign {device} -n")
}

pub fn get_nics<'a>() -> &'a str {
    "ifconfig | grep -e ^en | awk -F ':' '{print $1}'"
}

pub fn set_hostname(host: String) -> String {
    format!("hostnamectl set-hostname {host}")
}

pub fn deploy<'a>(mode: DeployMode) -> &'a str {
    match mode {
        DeployMode::AllInOne => "cd /etc/ustack-deploy/ && python3 deploy.py all_in_one",
        DeployMode::MultiNode => "cd /etc/ustack-deploy/ && python3 deploy.py all",
    }
}
