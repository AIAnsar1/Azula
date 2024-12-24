#[allow(clippy::module_name_repetitions)]
use crate::config::base::{ScriptRequired, ScriptFile};
use anyhow::{anyhow, Result};
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, prelude::*};
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::string::ToString;
use text_placeholder::Template;



#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use crate::config::base::ScriptConfig;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Script {
    pub (crate) path: Option<PathBuf>,
    pub (crate) ip: IpAddr,
    pub (crate) open_ports: Vec<u16>,
    pub (crate) trigger_ports: Option<String>,
    pub (crate) ports_separator: Option<String>,
    pub (crate) tags: Option<Vec<String>>,
    pub (crate) call_format: Option<String>,
}


#[derive(Serialize)]
struct ExecPartsScript {
    script: String,
    ip: String,
    port: String,
    ipversion: String,
}

#[derive(Serialize)]
struct ExecParts {
    ip: String,
    port: String,
    ipversion: String,
}

static DEFAULT: &str = r#"tags = ["core_approved", "azula", "default"]
ports_separator = ","
call_format = "nmap -vvv -p {{port}} -{{ipversion}} {{ip}}"
"#;


pub fn init_scripts(scripts: &ScriptRequired) -> Result<Vec<ScriptFile>> {
    let mut scripts_to_run: Vec<ScriptFile> = Vec::new();

    match scripts {
        ScriptRequired::None => {}
        ScriptRequired::Default => {
            let default_script = toml::from_str::<ScriptFile>(DEFAULT).expect("[ ERROR ]: Failed To Parse Script File.");
            scripts_to_run.push(default_script);
        }
        ScriptRequired::Custom => {
            let scripts_dir_base = dirs::home_dir().ok_or_else(|| anyhow!("Could Not Infer Scripts Path."))?;
            let scripts_paths = find_scripts(scripts_dir_base)?;
            debug!("Scripts Paths \n{:?}", scripts_paths);
            let parsed_scripts = parse_scripts(scripts_paths);
            debug!("Scripts Parsed \n{:?}", parsed_scripts);
            let script_config = ScriptConfig::read_config()?;
            debug!("Scripts Config \n{:?}", script_config);

            if let Some(config_hashset) = script_config.tags {
                for script in parsed_scripts {
                    if let Some(script_hashset) = &script.tags {
                        if script_hashset.iter().all(|tag| config_hashset.contains(tag)) {
                            scripts_to_run.push(script);
                        } else {
                            debug!("\nScript Tags Does not Match Config Tags {:?}", &script_hashset, script.path.unwrap().display());
                        }
                    }
                }
            }
            debug!("\nScript(s) To Run {:?}",scripts_to_run);

        }

    }
    Ok(scripts_to_run)
}


pub fn parse_scripts(scripts: Vec<PathBuf>) -> Vec<ScriptFile> {
    let mut parsed_scripts: Vec<ScriptFile> = Vec::with_capacity(scripts.len());

    for script in scripts {
        debug!("Parsing Script {}", &script.display());

        if let Some(script_file) = ScriptFile::new(script) {
            parsed_scripts.push(script_file);
        }
    }
    parsed_scripts
}


impl Script {
    pub fn build(path: Option<PathBuf>, ip: IpAddr, open_ports: Vec<u16>, trigger_ports: Option<String>, ports_separator: Option<String>, tags: Option<Vec<String>>, call_format: Option<String>,) -> Self {
        Self { path, ip, open_ports, trigger_ports, ports_separator, tags, call_format }
    }

    pub fn run(self) -> Result<String> {
        debug!("Run Self {:?}", &self);
        let separator = self.ports_separator.unwrap_or_else(|| ".".into());
        let mut ports_str = self.open_ports.iter().map(ToString::to_string).collect::<Vec<String>>().join(&separator);

        if let Some(ports) = self.trigger_ports {
            ports_str = ports
        }
        let mut final_call_format = String::new();

        if let Some(call_format) = self.call_format {
            final_call_format = call_format
        } else {
            return Err(anyhow!("[ ERROR ]: Failed To Parse Execution Format."))
        }
        let default_tempalte: Template = Template::new(&final_call_format);
        let mut tp_run = String::new();

        if final_call_format.contains("{{script}}") {
            let exec_ports_script: ExecPartsScript = ExecPartsScript {
                script: self.path.unwrap().to_str().unwrap().to_string(),
                ip: self.ip.to_string(),
                port: ports_str,
                ipversion: match &self.ip {
                    IpAddr::V4(_) => String::from("4"),
                    IpAddr::V6(_) => String::from("6"),
                },
            };
            to_run = default_tempalte.fill_with_struct(&exec_ports_script)?;
        } else {
            let exec_parts: ExecParts = ExecParts {
                ip: self.ip.to_string(),
                port: ports_str,
                ipversion: match &self.ip {
                    IpAddr::V4(_) => String::from("4"),
                    IpAddr::V6(_) => String::from("6"),
                },
            };
            to_run = default_tempalte.fill_with_struct(&exec_ports_script)?;
        }
        debug!("\nScript Format To Run {}", to_run);
        execute_script(&to_run)
    }
}

#[cfg(not(tarpaulin_include))]
fn execute_script(script: &str) -> Result<String> {
    debug!("\nScripts Arguments {}", script);

    let (cmd, arg) = if cfg!(unix) {
        ("sh", "-c")
    } else {
        ("cmd.exe", "/c")
    };

    match Command::new(cmd).args([arg, script]).stdin(Stdio::piped()).stderr(Stdio::piped()).output() {
        Ok(output) => {
            let status = output.status;
            let es = match status.code() {
                Some(code) => code,
                _ => {
                    #[cfg(unix)]
                    {
                        status.signal().unwrap();
                    }
                    #[cfg(windows)]
                    {
                        return Err(anyhow!("Unknown Exit Status"));
                    }
                }
            };

            if es != 0 {
                return Err(anyhow!("Exit Code: = {}", es));
            }
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        }
        Err(error) => {
            debug!("Command Error {}", error.to_string());
            Err(anyhow!(error.to_string()))
        }
    }
}


pub fn find_scripts(mut path: PathBuf) -> Result<Vec<PathBuf>> {
    path.push(".azula_scripts");

    if path.is_dir() {
        debug!("Scrits Folder Found {}", &path.display());
        let mut files_vec: Vec<PathBuf> = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            files_vec.push(entry.path());
        }
        Ok(files_vec)
    } else {
        Err(anyhow!("Can't Find Scripts Folder {}", path.display()))
    }
}






























