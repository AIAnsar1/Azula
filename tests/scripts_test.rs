



mod test {
    use azula::scripts::scripting::{find_scripts, parse_scripts, Script};
    use azula::config::base::ScriptFile;

    fn into_script(script_f: ScriptFile) -> Script {
        Script::build(script_f.path, "127.0.0.1".parse().unwrap(), vec![80, 8080], script_f.port, script_f.ports_separator, script_f.tags, script_f.call_format)
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
        let mut script_f = ScriptFile::new("fixtures/.azula_scripts/test_script.txt".into()).unwrap();
        script_f.call_format = Some("qwertyuiop".to_string());
        let script: Script = into_script(script_f);
        let _output = script.run().unwrap();
    }

    #[test]
    #[should_panic]
    fn open_script_file_missing_call_format() {
        let mut script_f = ScriptFile::new("fixtures/.azula_scripts/test_script.txt".into()).unwrap();
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
        let script_f = ScriptFile::new("fixtures/.rustscan_scripts/test_script.txt".into()).unwrap();
        assert_eq!(script_f.tags, Some(vec!["core_approved".to_string(), "example".to_string()]));
        assert_eq!(script_f.developer, Some(vec![
                "example".to_string(),
                "https://example.org".to_string()
            ])
        );
        assert_eq!(script_f.ports_separator, Some(",".to_string()));
        assert_eq!(script_f.call_format, Some("nmap -vvv -p {{port}} {{ip}}".to_string()));
    }

    #[test]
    #[cfg(unix)]
    fn run_bash_script() {
        let script_f = ScriptFile::new("fixtures/.rustscan_scripts/test_script.sh".into()).unwrap();
        let script: Script = into_script(script_f);
        let output = script.run().unwrap();
        assert_eq!(output.trim(), "127.0.0.1 80,8080");
    }

    #[test]
    fn run_python_script() {
        let script_f = ScriptFile::new("fixtures/.rustscan_scripts/test_script.py".into()).unwrap();
        let script: Script = into_script(script_f);
        let output = script.run().unwrap();
        assert_eq!(output.trim(), "Python script ran with arguments ['fixtures/.rustscan_scripts/test_script.py', '127.0.0.1', '80,8080']");
    }

    #[test]
    #[cfg(unix)]
    fn run_perl_script() {
        let script_f = ScriptFile::new("fixtures/.rustscan_scripts/test_script.pl".into()).unwrap();
        let script: Script = into_script(script_f);
        let output = script.run().unwrap();
        assert_eq!(output.trim(), "Total args passed to fixtures/.rustscan_scripts/test_script.pl : 2\nArg # 1 : 127.0.0.1\nArg # 2 : 80,8080");
    }
}