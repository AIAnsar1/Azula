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
    pub(crate)  start: u16,
    pub(crate)  end: u16
}



#[derive(Parser, Debug, Clone)]
#[command(name = "azula", version = env!("CARGO_PKG_VERSION"), max_term_width = 120, help_template = "{bin} {version}\n{about}\n\nUSAGE:\n {usage}\n\nOPTIONS:\n{options}")]
#[allow(clippy::struct_excessive_bools)]
pub struct Opts {
    #[arg(short, long, value_delimiter = ',')]
    pub(crate)  addresses: Vec<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub(crate)  ports: Option<Vec<u16>>,
    #[arg(short, long, conflicts_with = "ports", value_parser = parse_range)]
    pub(crate)  range: Option<PortRange>,
    #[arg(long, short)]
    pub(crate)  no_config: bool,
    #[arg(long, short, value_parser)]
    pub(crate)  config_path: Option<PathBuf>,
    #[arg(long, short)]
    pub(crate)  greppable: bool,
    #[arg(long)]
    pub(crate)  accessible: bool,
    #[arg(long)]
    pub(crate)  resolver: Option<String>,
    #[arg(long, short, default_value = "4500")]
    pub(crate)  batch_size: u16,
    #[arg(long, short, default_value = "1500")]
    pub(crate)  timeout: u32,
    #[arg(long, default_value = "1")]
    pub(crate)  tries: u8,
    #[arg(long, short)]
    pub(crate)  ulimit: Option<u64>,
    #[arg(long, value_enum, ignore_case = true, default_value = "serial")]
    pub(crate)  scan_order: ScanOrder,
    #[arg(long, value_enum, ignore_case = true, default_value = "default")]
    pub(crate)  scripts: ScriptRequired,
    #[arg(long)]
    pub(crate)  top: bool,
    #[arg(last = true)]
    pub(crate)  command: Vec<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub(crate)  exclude_ports: Option<Vec<u16>>,
    #[arg(long)]
    pub(crate)  udp: bool,
}


pub struct RangeIterator {
    pub(crate) active: bool,
    pub(crate) normalized_end: u32,
    pub(crate) normalized_first_pick: u32,
    pub(crate) normalized_pick: u32,
    pub(crate) actual_start: u32,
    pub(crate) step: u32,
}

#[cfg(not(tarpaulin_include))]
#[derive(Debug, Deserialize)]
pub struct Config {
    pub(crate) address: Option<Vec<String>>,
    pub(crate) ports: Option<HashMap<String, u16>>,
    pub(crate) range: Option<PortRange>,
    pub(crate) greppable: Option<bool>,
    pub(crate) accessible: Option<bool>,
    pub(crate) batch_size: Option<u16>,
    pub(crate) timeout: Option<u32>,
    pub(crate) tries: Option<u8>,
    pub(crate) ulimit: Option<u64>,
    pub(crate) resolver: Option<String>,
    pub(crate) scan_order: Option<ScanOrder>,
    pub(crate) command: Option<Vec<String>>,
    pub(crate) scripts: Option<ScriptRequired>,
    pub(crate) exclude_ports: Option<Vec<u16>>,
    pub(crate) udp: Option<bool>,
}