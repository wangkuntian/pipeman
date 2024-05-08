use clap_derive::{Parser, Subcommand, ValueEnum};
use strum::{Display, EnumString};

#[derive(Parser)]
#[command(version, about = "Ustack deploy test tool", long_about)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Deploy Ustack
    Deploy(DeployArgs),
}

#[derive(Parser, Debug)]
pub struct DeployArgs {
    /// The architecture of deployment
    #[arg(short, long, value_enum)]
    pub arch: Arches,

    /// The mode of deployment
    #[arg(short, long, value_enum)]
    pub mode: DeployMode,

    /// The config file of deployment
    #[arg(short, long)]
    pub file: Option<String>,

    /// The ip of hosts to deploy, separated by ','
    #[arg(long)]
    pub hosts: Option<String>,

    /// disable stdout log, default is false
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,
}

#[derive(
    Default, Debug, Display, Copy, Clone, PartialEq, Eq, EnumString, PartialOrd, Ord, ValueEnum,
)]
pub enum Arches {
    /// x86_64
    #[strum(serialize = "amd64")]
    #[default]
    Amd64,
    /// aarch64
    #[strum(serialize = "arm64")]
    Arm64,
}

#[derive(
    Default, Debug, Display, Copy, Clone, PartialEq, Eq, EnumString, PartialOrd, Ord, ValueEnum,
)]
pub enum DeployMode {
    /// Single Node
    #[strum(serialize = "all_in_one")]
    #[default]
    AllInOne,
    /// Cluster
    #[strum(serialize = "multi_node")]
    MultiNode,
}
