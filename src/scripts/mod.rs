#![allow(clippy::module_name_repetitions)]

use crate::input::ScriptsRequired;
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

static DEFAULT: &str = r#"tags = ["core_approved", "RustScan", "default"]
developer = [ "RustScan", "https://github.com/RustScan" ]
ports_separator = ","
call_format = "nmap -vvv -p {{port}} -{{ipversion}} {{ip}}"
"#;

#[cfg(not(tarpaulin_include))]
pub fn init_scripts(scripts: &ScriptsRequired) -> Result<Vec<ScriptFile>> {
    let mut scripts_to_run: Vec<ScriptFile> = Vec::new();

    match scripts {
        ScriptsRequired::None => {}
        ScriptsRequired::Default => {
            let default_script =
                toml::from_str::<ScriptFile>(DEFAULT).expect("Failed to parse Script file.");
            scripts_to_run.push(default_script);
        }
        ScriptsRequired::Custom => {
            let scripts_dir_base =
                dirs::home_dir().ok_or_else(|| anyhow!("Could not infer scripts path."))?;
            let script_paths = find_scripts(scripts_dir_base)?;
            debug!("Scripts paths \n{:?}", script_paths);

            let parsed_scripts = parse_scripts(script_paths);
            debug!("Scripts parsed \n{:?}", parsed_scripts);

            let script_config = ScriptConfig::read_config()?;
            debug!("Script config \n{:?}", script_config);

            // Only Scripts that contain all the tags found in ScriptConfig will be selected.
            if let Some(config_hashset) = script_config.tags {
                for script in parsed_scripts {
                    if let Some(script_hashset) = &script.tags {
                        if script_hashset
                            .iter()
                            .all(|tag| config_hashset.contains(tag))
                        {
                            scripts_to_run.push(script);
                        } else {
                            debug!(
                                "\nScript tags does not match config tags {:?} {}",
                                &script_hashset,
                                script.path.unwrap().display()
                            );
                        }
                    }
                }
            }
            debug!("\nScript(s) to run {:?}", scripts_to_run);
        }
    }

    Ok(scripts_to_run)
}

pub fn parse_scripts(scripts: Vec<PathBuf>) -> Vec<ScriptFile> {
    let mut parsed_scripts: Vec<ScriptFile> = Vec::with_capacity(scripts.len());
    for script in scripts {
        debug!("Parsing script {}", &script.display());
        if let Some(script_file) = ScriptFile::new(script) {
            parsed_scripts.push(script_file);
        }
    }
    parsed_scripts
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Script {
    // Path to the script itself.
    path: Option<PathBuf>,

    // Ip got from scanner.
    ip: IpAddr,

    // Ports found with portscan.
    open_ports: Vec<u16>,

    // Port found in ScriptFile, if defined only this will run with the ip.
    trigger_port: Option<String>,

    // Character to join ports in case we want to use a string format of them, for example nmap -p.
    ports_separator: Option<String>,

    // Tags found in ScriptFile.
    tags: Option<Vec<String>>,

    // The format how we want the script to run.
    call_format: Option<String>,
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

impl Script {
    pub fn build(
        path: Option<PathBuf>,
        ip: IpAddr,
        open_ports: Vec<u16>,
        trigger_port: Option<String>,
        ports_separator: Option<String>,
        tags: Option<Vec<String>>,
        call_format: Option<String>,
    ) -> Self {
        Self {
            path,
            ip,
            open_ports,
            trigger_port,
            ports_separator,
            tags,
            call_format,
        }
    }

    // Some variables get changed before read, and compiler throws warning on warn(unused_assignments)
    #[allow(unused_assignments)]
    pub fn run(self) -> Result<String> {
        debug!("run self {:?}", &self);

        let separator = self.ports_separator.unwrap_or_else(|| ",".into());

        let mut ports_str = self
            .open_ports
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(&separator);
        if let Some(port) = self.trigger_port {
            ports_str = port;
        }

        let mut final_call_format = String::new();
        if let Some(call_format) = self.call_format {
            final_call_format = call_format;
        } else {
            return Err(anyhow!("Failed to parse execution format."));
        }
        let default_template: Template = Template::new(&final_call_format);
        let mut to_run = String::new();

        if final_call_format.contains("{{script}}") {
            let exec_parts_script: ExecPartsScript = ExecPartsScript {
                script: self.path.unwrap().to_str().unwrap().to_string(),
                ip: self.ip.to_string(),
                port: ports_str,
                ipversion: match &self.ip {
                    IpAddr::V4(_) => String::from("4"),
                    IpAddr::V6(_) => String::from("6"),
                },
            };
            to_run = default_template.fill_with_struct(&exec_parts_script)?;
        } else {
            let exec_parts: ExecParts = ExecParts {
                ip: self.ip.to_string(),
                port: ports_str,
                ipversion: match &self.ip {
                    IpAddr::V4(_) => String::from("4"),
                    IpAddr::V6(_) => String::from("6"),
                },
            };
            to_run = default_template.fill_with_struct(&exec_parts)?;
        }
        debug!("\nScript format to run {}", to_run);
        execute_script(&to_run)
    }
}

#[cfg(not(tarpaulin_include))]
fn execute_script(script: &str) -> Result<String> {
    debug!("\nScript arguments {}", script);

    let (cmd, arg) = if cfg!(unix) {
        ("sh", "-c")
    } else {
        ("cmd.exe", "/c")
    };

    match Command::new(cmd)
        .args([arg, script])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => {
            let status = output.status;

            let es = match status.code() {
                Some(code) => code,
                _ => {
                    #[cfg(unix)]
                    {
                        status.signal().unwrap()
                    }

                    #[cfg(windows)]
                    {
                        return Err(anyhow!("Unknown exit status"));
                    }
                }
            };

            if es != 0 {
                return Err(anyhow!("Exit code = {}", es));
            }
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        }
        Err(error) => {
            debug!("Command error {}", error.to_string());
            Err(anyhow!(error.to_string()))
        }
    }
}

pub fn find_scripts(mut path: PathBuf) -> Result<Vec<PathBuf>> {
    path.push(".azula_scripts");
    if path.is_dir() {
        debug!("Scripts folder found {}", &path.display());
        let mut files_vec: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            files_vec.push(entry.path());
        }
        Ok(files_vec)
    } else {
        Err(anyhow!("Can't find scripts folder {}", path.display()))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScriptFile {
    pub path: Option<PathBuf>,
    pub tags: Option<Vec<String>>,
    pub developer: Option<Vec<String>>,
    pub port: Option<String>,
    pub ports_separator: Option<String>,
    pub call_format: Option<String>,
}

impl ScriptFile {
    fn new(script: PathBuf) -> Option<ScriptFile> {
        let real_path = script.clone();
        let mut lines_buf = String::new();
        if let Ok(file) = File::open(script) {
            for mut line in io::BufReader::new(file).lines().skip(1).flatten() {
                if line.starts_with('#') {
                    line.retain(|c| c != '#');
                    line = line.trim().to_string();
                    line.push('\n');
                    lines_buf.push_str(&line);
                } else {
                    break;
                }
            }
        } else {
            debug!("Failed to read file: {}", &real_path.display());
            return None;
        }
        debug!("ScriptFile {} lines\n{}", &real_path.display(), &lines_buf);

        match toml::from_str::<ScriptFile>(&lines_buf) {
            Ok(mut parsed) => {
                debug!("Parsed ScriptFile{} \n{:?}", &real_path.display(), &parsed);
                parsed.path = Some(real_path);
                // parsed_scripts.push(parsed);
                Some(parsed)
            }
            Err(e) => {
                debug!("Failed to parse ScriptFile headers {}", e.to_string());
                None
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScriptConfig {
    pub tags: Option<Vec<String>>,
    pub ports: Option<Vec<String>>,
    pub developer: Option<Vec<String>>,
}

#[cfg(not(tarpaulin_include))]
impl ScriptConfig {
    pub fn read_config() -> Result<ScriptConfig> {
        let Some(mut home_dir) = dirs::home_dir() else {
            return Err(anyhow!("Could not infer ScriptConfig path."));
        };
        home_dir.push(".rustscan_scripts.toml");

        let content = fs::read_to_string(home_dir)?;
        let config = toml::from_str::<ScriptConfig>(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::{find_scripts, parse_scripts, Script, ScriptFile};

    // Function for testing only, it inserts static values into ip and open_ports
    // Doesn't use impl in case it's implemented in the super module at some point
    fn into_script(script_f: ScriptFile) -> Script {
        Script::build(
            script_f.path,
            "127.0.0.1".parse().unwrap(),
            vec![80, 8080],
            script_f.port,
            script_f.ports_separator,
            script_f.tags,
            script_f.call_format,
        )
    }

    #[test]
    fn find_and_parse_scripts() {
        let scripts = find_scripts("fixtures/".into()).unwrap();
        let scripts = parse_scripts(scripts);
        assert_eq!(scripts.len(), 4);
    }

    #[test]
    #[should_panic]
    fn find_invalid_folder() {
        let _scripts = find_scripts("Cargo.toml".into()).unwrap();
    }

    #[test]
    #[should_panic]
    fn open_script_file_invalid_headers() {
        ScriptFile::new("fixtures/.azula_scripts/test_script_invalid_headers.txt".into())
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn open_script_file_invalid_call_format() {
        let mut script_f =
            ScriptFile::new("fixtures/.azula_scripts/test_script.txt".into()).unwrap();
        script_f.call_format = Some("qwertyuiop".to_string());
        let script: Script = into_script(script_f);
        let _output = script.run().unwrap();
    }

    #[test]
    #[should_panic]
    fn open_script_file_missing_call_format() {
        let mut script_f =
            ScriptFile::new("fixtures/.azula_scripts/test_script.txt".into()).unwrap();
        script_f.call_format = None;
        let script: Script = into_script(script_f);
        let _output = script.run().unwrap();
    }

    #[test]
    #[should_panic]
    fn open_nonexisting_script_file() {
        ScriptFile::new("qwertyuiop.txt".into()).unwrap();
    }

    #[test]
    fn parse_txt_script() {
        let script_f =
            ScriptFile::new("fixtures/.azula_scripts/test_script.txt".into()).unwrap();
        assert_eq!(
            script_f.tags,
            Some(vec!["core_approved".to_string(), "example".to_string()])
        );
        assert_eq!(
            script_f.developer,
            Some(vec![
                "example".to_string(),
                "https://example.org".to_string()
            ])
        );
        assert_eq!(script_f.ports_separator, Some(",".to_string()));
        assert_eq!(
            script_f.call_format,
            Some("nmap -vvv -p {{port}} {{ip}}".to_string())
        );
    }

    #[test]
    #[cfg(unix)]
    fn run_bash_script() {
        let script_f = ScriptFile::new("fixtures/.azula_scripts/test_script.sh".into()).unwrap();
        let script: Script = into_script(script_f);
        let output = script.run().unwrap();
        // output has a newline at the end by default, .trim() trims it
        assert_eq!(output.trim(), "127.0.0.1 80,8080");
    }

    #[test]
    fn run_python_script() {
        let script_f = ScriptFile::new("fixtures/.azula_scripts/test_script.py".into()).unwrap();
        let script: Script = into_script(script_f);
        let output = script.run().unwrap();
        // output has a newline at the end by default, .trim() trims it
        assert_eq!(
            output.trim(),
            "Python script ran with arguments ['fixtures/.azula_scripts/test_script.py', '127.0.0.1', '80,8080']"
        );
    }

    #[test]
    #[cfg(unix)]
    fn run_perl_script() {
        let script_f = ScriptFile::new("fixtures/.azula_scripts/test_script.pl".into()).unwrap();
        let script: Script = into_script(script_f);
        let output = script.run().unwrap();
        // output has a newline at the end by default, .trim() trims it
        assert_eq!(output.trim(), "Total args passed to fixtures/.azula_scripts/test_script.pl : 2\nArg # 1 : 127.0.0.1\nArg # 2 : 80,8080");
    }
}