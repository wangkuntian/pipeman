mod common;
mod openstack;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use std::path::Path;

use crate::common::args::{Cli, Commands};
use crate::common::config::{CONF, CONFIG};
use crate::common::log::init_logger;
use crate::common::utils::get_default_config_file;
use crate::common::worker::Deployment;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.commands {
        Commands::Deploy(args) => {
            let file_path: String;
            if let Some(config_file) = &args.file {
                let path = Path::new(config_file.as_str());
                if !path.is_file() {
                    let mut cmd = Cli::command();
                    let message = format!("config file {:?} not exists", path);
                    cmd.error(ErrorKind::InvalidValue, message).exit();
                } else {
                    file_path = config_file.to_string();
                }
            } else {
                file_path = get_default_config_file();
            }
            let conf = CONF::init(file_path.as_str());
            CONFIG.set(conf).unwrap();
            init_logger(args);
            let mut deployment = Deployment::new(args).await;
            deployment.execute().await;
        }
    };
}
