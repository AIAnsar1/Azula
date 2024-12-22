use clap::{Parser, ValueEnum};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const LOWEST_PORT_NUMBER: u16 = 1;
pub const TOP_PORT_NUMBER: u16 = 65535;


#[derive(Deserialize, Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum ScanOrder {
    Serial,
    Random,
}

#[derive(Deserialize, Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum ScriptRequired {
    None,
    Default,
    Custom,
}


#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PortRange {
    pub start: u16,
    pub end: u16
}



#[derive(Parser, Debug, Clone)]
#[command(name = "azula", version = env!("CARGO_PKG_VERSION"), max_term_width = 120, help_template = "{bin} {version}\n{about}\n\nUSAGE:\n {usage}\n\nOPTIONS:\n{options}")]
#[allow(clippy::struct_excessive_bools)]
pub struct Opts {
    #[arg(short, long, value_delimiter = ',')]
    pub addresses: Vec<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub ports: Option<Vec<u16>>,
    #[arg(short, long, conflicts_with = "ports", value_parser = parse_range)]
    pub range: Option<PortRange>,
    #[arg(long, short)]
    pub no_config: bool,
    #[arg(long, short, value_parser)]
    pub config_path: Option<PathBuf>,
    #[arg(long, short)]
    pub greppable: bool,
    #[arg(long)]
    pub accessible: bool,
    #[arg(long)]
    pub resolver: Option<String>,
    #[arg(long, short, default_value = "4500")]
    pub batch_size: u16,
    #[arg(long, short, default_value = "1500")]
    pub timeout: u32,
    #[arg(long, default_value = "1")]
    pub tries: u8,
    #[arg(long, short)]
    pub ulimit: Option<u64>,
    #[arg(long, value_enum, ignore_case = true, default_value = "serial")]
    pub scan_order: ScanOrder,
    #[arg(long, value_enum, ignore_case = true, default_value = "default")]
    pub scripts: ScriptRequired,
    #[arg(long)]
    pub top: bool,
    #[arg(last = true)]
    pub command: Vec<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub exclude_ports: Option<Vec<u16>>,
    #[arg(long)]
    pub udp: bool,
}


pub struct RangeIterator {
    active: bool,
    normalized_end: u32,
    normalized_first_pick: u32,
    normalized_pick: u32,
    actual_start: u32,
    step: u32,
}

#[cfg(not(tarpaulin_include))]
#[derive(Debug, Deserialize)]
pub struct Config {
    address: Option<Vec<String>>,
    ports: Option<HashMap<String, u16>>,
    range: Option<PortRange>,
    greppable: Option<bool>,
    accessible: Option<bool>,
    batch_size: Option<u16>,
    timeout: Option<u32>,
    tries: Option<u8>,
    ulimit: Option<u64>,
    resolver: Option<String>,
    scan_order: Option<ScanOrder>,
    command: Option<Vec<String>>,
    scripts: Option<ScriptRequired>,
    exclude_ports: Option<Vec<u16>>,
    udp: Option<bool>,
}