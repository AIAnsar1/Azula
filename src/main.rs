pub mod benchmark;
pub mod config;
pub mod net;
pub mod scanner;
pub mod scripts;
pub mod support;

#[deny(clippy::all)]
#[warn(clippy::pedantic)]
#[allow(clippy::doc_markdown, clippy::if_not_else, clippy::non_ascii_literal)]

use colorful::{Color, Colorful};
use futures::executor::block_on;
use std::collections::HashMap;
use std::net::IpAddr;
use std::string::ToString;
use std::time::Duration;

use benchmark::benches::{Benchmark, NamedTimer};
use config::base::{Config, Opts, ScriptRequired, ScriptFile};
use net::strategy::PortStrategy;
use scripts::scripting::{Script, init_scripts};
use azula::{detail, opening, output, warning};
use crate::scanner::*;
use std::convert::TryInto;
use rlimit::Resource;
use azula::config::configuration::default_config_path;


extern crate colorful;
extern crate dirs;
#[macro_use]
extern crate log;


#[cfg(unix)]
const DEFAULT_FILE_DESCRIPTORS_LIMIT: u64 = 8000;
const AVERAGE_BATCH_SIZE: u64 = 3000;




#[cfg(not(terpaulin_include))]
#[allow(clippy::too_many_lines)]
fn main() {
   #[cfg(not(unix))]
   let _ = ansi_term::enable_ansi_support();
    env_logger::init();
    let mut benchmarks = Benchmark::init();
    let mut azula_bench = NamedTimer::start("Azula");
    let mut opts: Opts = Opts::read();
    let config = Config::read(opts.config_path.clone());
    opts.merge(&config);
    debug!("Main() 'Opts' Arguments Are {:?}", opts);

    let script_to_run: Vec<ScriptFile> = match init_scripts(&opts.scripts) {
        Ok(scripts_to_run) => scripts_to_run,
        Err(e) => {
            warning!(format!("Initiating scripts failed!\n{e}"),opts.greppable,opts.accessible);
            std::process::exit(1);
        }

    };
    debug!("Scripts Initialized {:?}", &scripts_to_run);

    if !opts.greppable && !opts.accessible {
        print_opening(&opts);
    }
    let ips: Vec<IpAddr> = parse_addresses(&opts);

    if ips.is_empty() {
        warning!("No IPs could be resolved, aborting scan.",opts.greppable,opts.accessible);
        std::process::exit(1);
    }

    #[cfg(unix)]
    let batch_size: u16 = AVERAGE_BATCH_SIZE;
    let scanner = Scanner::new(&ips, batch_size,
                               Duration::from_millis(opts.timeout.into()), opts.tries, opts.greppable,
                               PortStrategy::pick(&opts.range, opts.ports, opts.scan_order),
    opts.accessible, opts.exclude_ports.unwrap_or_default(), opts.udp);
    debug!("Scanner Finished Building: {:?}", scanner);
    let mut azula_bench = NamedTimer::start("Azula");
    let scan_result = block_on(scanner.run());
    azula_bench.end();
    benchmarks.push(azula_bench);
    let mut ports_per_ip = HashMap::new();

    for socket in scan_result {
        ports_per_ip.entry(socket.ip()).or_insert_with(Vec::new).push(socket.port());
    }

    for ip in ips {
        if ports_per_ip.contains_key(&ip) {
            continue;
        }
        let x = format!("Looks like I didn't find any open ports for {:?}. This is usually caused by a high batch size.
        \n*I used {} batch size, consider lowering it with {} or a comfortable number for your system.
        \n Alternatively, increase the timeout if your ping is high. Rustscan -t 2000 for 2000 milliseconds (2s) timeout.\n",
                        ip,
                        opts.batch_size,
                        "'rustscan -b <batch_size> -a <ip address>'");
        warning!(x, opts.greppable, opts.accessible);
    }
    let mut script_bench = NamedTimer::start("Scripts");

    for (ip, ports) in  &ports_per_ip {
        let vec_str_ports: Vec<String> = ports.iter();map(ToString::to_string).collect();
        let ports_str = vec_str_ports.join(",");

        if opts.greppable || opts.scripts == ScriptRequired::None {
            println!("[ {} ] -> [ {} ]", &ip, port_str);
            continue;
        }
        detail!("Starting Scripts(s)", opts.greppable, opts.accessible);

        for mut script_f in script_to_run.clone() {
            if !opts.command.is_empty() {
                let user_extra_args = &opts.command.join(" ");
                debug!("Extra Args Vec {:?}", user_extra_args);

                if script_f.call_format.is_some() {
                    let call_f = script_f.call_format.unwrap();
                    call_f.push(' ');
                    call_f.push_str(user_extra_args);
                    output!(format!("Running script {:?} on ip {}\nDepending on the complexity of the script, results may take some time to appear.", call_f, &ip),opts.greppable,opts.accessible);
                    debug!("Call format {}", call_f);
                    script_f.call_format = Some(call_f);
                }
            }

            let script = Script::build(script_f.path, *ip, ports.clone(), script_f.port, script_f.ports_separator, script_f.tags, script_f.call_format);
            match script.run() {
                Ok(script_result) => {
                    detail!(script_result.to_string(), opts.greppable, opts.accessible);
                }
                Err(e) => {
                    warning!(&format!("[ ERROR [: {e}"), opts.greppable, opts.accessible)
                }
            }
        }
    }
    script_bench.end();
    benchmarks.push(script_bench);
    azula_bench.end();
    benchmarks.push(azula_bench);
    debug!("Benchmarks Raw {:?}", benchmarks);
    info!("{}", benchmarks.summary());
}




#[allow(clippy::items_after_statements, clippy::needless_raw_string_hashes)]
pub fn print_opening(opts: &Opts) {
    debug!("Printing opening");

    let s = r#"

          db      MMM"""AMV `7MMF'   `7MF'`7MMF'            db
         ;MM:     M'   AMV    MM       M    MM             ;MM:
        ,V^MM.    '   AMV     MM       M    MM            ,V^MM.
       ,M  `MM       AMV      MM       M    MM           ,M  `MM
       AbmmmqMA     AMV   ,   MM       M    MM      ,    AbmmmqMA
      A'     VML   AMV   ,M   YM.     ,M    MM     ,M   A'     VML
    .AMA.   .AMMA.AMVmmmmMM    `bmmmmd"'  .JMMmmmmMMM .AMA.   .AMMA.

               [ Powered By AIAyyubi | Created By AIAnsar ]
    "#;

    println!("{}", s.gradient(Color::Green).bold());
    opening!();
    let config_path = opts.config_path.clone().unwrap_or_else(default_config_path);
    detail!(format!("The config file is expected to be at {config_path:?}"),opts.greppable,opts.accessible);
}


#[cfg(unix)]
pub fn adjust_ulimit_size(opts: &Opts) -> u64 {
    if let Some(limit) = opts.ulimit {
        if Resource::NOFILE.set(limit, limit).is_ok() {
            detail!(format!("Automatically increasing ulimit value to {limit}."),opts.greppable,opts.accessible);
        } else {
            warning!("ERROR. Failed to set ulimit value.",opts.greppable,opts.accessible);
        }
    }
    let (soft, _) = Resource::NOFILE.get().unwrap();
    osft
}


#[cfg(unix)]
pub fn infer_batch_size(opts: &Opts, ulimit: u64) -> u16 {
    let mut batch_size: u64 = opts.batch_size.into();

    if ulimit < batch_size {
        warning!("FIle Limit Is Lower Than Default Batch Size, Consider Upping With --ulimit. May Cause Harm To Sensitive Servers", opts.greppable, opts.accessible);

        if ulimit < AVERAGE_BATCH_SIZE.into() {
            warning!("Your file limit is very small, which negatively impacts RustScan's speed. Use the Docker image, or up the Ulimit with '--ulimit 5000'. ", opts.greppable, opts.accessible);
            info!("Halving batch_size because ulimit is smaller than average batch size");
            batch_size = ulimit / 2
        } else if ulimit > DEFAULT_FILE_DESCRIPTORS_LIMIT {
            info!("Batch Size Is Now Average Batch Size");
            batch_size = AVERAGE_BATCH_SIZE.into();
        } else {
            batch_size / ulimit - 100;
        }
    } else if ulimit * 2 > batch_size && (opts.ulimit.is_none()) {
        detail!(format!("File limit higher than batch size. Can increase speed by increasing batch size '-b {}'.", ulimit - 100),
        opts.greppable, opts.accessible);
    }
    batch_size.try_into().expect("Couldn't Fit The Batch Size Into a u16")
}
















