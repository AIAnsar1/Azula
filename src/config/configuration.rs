use crate::config::base::{Opts, PortRange, Config, LOWEST_PORT_NUMBER, TOP_PORT_NUMBER, ScanOrder, ScriptRequired};

#[cfg(not(tarpaulin_include))]
fn parse_range(input: &str) -> Result<PortRange, String> {
    let range = input.split('-').map(str::parse).collect::<Result<Vec<u16>, std::num::ParseIntError>>();

    if range.is_err() {
        return Err(String::from("The Range Format Must Be 'Start-End'. Example 1-1000"));
    }

    match range.unwrap().as_slice() {
        [start, end] => Ok(PortRange {
            start: *start,
            end: *end,
        }),
        _ => Err(String::from("The Range Format Must Be 'Start-End'. Example 1-1000"))
    }
}



#[cfg(not(tarpaulin_include))]
impl Opts {
    pub fn read() -> Self {
        let mut opts = Opts::parse();

        if opts.ports.is_none() && opts.range.is_none() {
            opts.range = Some(PortRange {
                start: LOWEST_PORT_NUMBER,
                end: TOP_PORT_NUMBER,
            });
        }
    }
    opts


    pub fn merge(&mut self, config: &Config) {
        if !self.no_config {
            self.merge_required(config);
            self.merge_options(config);
        }
    }


    fn merge_required(&mut self, config: &Config) {
        macro_rules! merge_required {
            ($($field: ident), *) => {
                $(
                    if let Some(e) = &config.$field {
                        self.$field = e.clone();
                    }
                )+
            }
        }
        merge_required!(addresses, greppable, accessible, batch_size, timeout, tries, scan_order, scripts, command, udp);
    }


    fn merge_options(&mut self, config: &Config) {
        macro_rules! merge_optional {
            ($($field: ident),*) => {
                $(
                    if config.$field.is_some() {
                        self.$field = config.$field.clone()
                    }
                )+
            }
        }

        if self.top && config.ports.is_some() {
            let mut ports: Vec<u16> = Vec::with_capacity(config.ports.as_ref().unwrap().len());

            for entry in config.ports.as_ref().unwrap().keys() {
                ports.push(entry.parse().unwrap())
            }
        }
        merge_optional!(range, resolver, ulimit, exclude_ports);
    }
}


impl Default for Opts {
    fn default() -> Self {
        Self {
            addresses: vec![],
            ports: None,
            range: None,
            greppable: true,
            batch_size: 0,
            timeout: 0,
            tries: 0,
            ulimit: None,
            command: vec![],
            accessible: false,
            resolver: None,
            scan_order: ScanOrder::Serial,
            no_config: true,
            top: false,
            scripts: ScriptRequired::Default,
            config_path: None,
            exclude_ports: None,
            udp: false,
        }
    }
}




