



mod test {
    use std::net::Ipv4Addr;
    use azula::scanner::address::{parse_addresses, get_resolver};

    #[test]
    fn parse_correct_addresses() {
        let opts = Opts {
            addresses: vec!["127.0.0.1".to_owned(), "192.168.0.0/30".to_owned()],
            ..Default::default()
        };
        let ips = parse_addresses(&opts);
        assert_eq!(ips, [
                Ipv4Addr::new(127, 0, 0, 1),
                Ipv4Addr::new(192, 168, 0, 0),
                Ipv4Addr::new(192, 168, 0, 1),
                Ipv4Addr::new(192, 168, 0, 2),
                Ipv4Addr::new(192, 168, 0, 3)
            ]);
    }

    #[test]
    fn parse_correct_host_addresses() {
        let opts = Opts {
            addresses: vec!["google.com".to_owned()],
            ..Default::default()
        };
        let ips = parse_addresses(&opts);
        assert_eq!(ips.len(), 1);
    }

    #[test]
    fn parse_correct_and_incorrect_addresses() {
        let opts = Opts {
            addresses: vec!["127.0.0.1".to_owned(), "im_wrong".to_owned()],
            ..Default::default()
        };
        let ips = parse_addresses(&opts);
        assert_eq!(ips, [Ipv4Addr::new(127, 0, 0, 1),]);
    }

    #[test]
    fn parse_incorrect_addresses() {
        let opts = Opts {
            addresses: vec!["im_wrong".to_owned(), "300.10.1.1".to_owned()],
            ..Default::default()
        };
        let ips = parse_addresses(&opts);
        assert!(ips.is_empty());
    }
    #[test]
    fn parse_hosts_file_and_incorrect_hosts() {
        let opts = Opts {
            addresses: vec!["fixtures/hosts.txt".to_owned()],
            ..Default::default()
        };
        let ips = parse_addresses(&opts);
        assert_eq!(ips.len(), 3);
    }

    #[test]
    fn parse_empty_hosts_file() {
        let opts = Opts {
            addresses: vec!["fixtures/empty_hosts.txt".to_owned()],
            ..Default::default()
        };
        let ips = parse_addresses(&opts);
        assert_eq!(ips.len(), 0);
    }

    #[test]
    fn parse_naughty_host_file() {
        // Host file contains IP, Hosts, incorrect IPs, incorrect hosts
        let opts = Opts {
            addresses: vec!["fixtures/naughty_string.txt".to_owned()],
            ..Default::default()
        };
        let ips = parse_addresses(&opts);
        assert_eq!(ips.len(), 0);
    }

    #[test]
    fn parse_duplicate_cidrs() {
        let opts = Opts {
            addresses: vec!["79.98.104.0/21".to_owned(), "79.98.104.0/24".to_owned()],
            ..Default::default()
        };
        let ips = parse_addresses(&opts);
        assert_eq!(ips.len(), 2_048);
    }

    #[test]
    fn resolver_default_cloudflare() {
        let opts = Opts::default();
        let resolver = get_resolver(&opts.resolver);
        let lookup = resolver.lookup_ip("www.example.com.").unwrap();
        assert!(opts.resolver.is_none());
        assert!(lookup.iter().next().is_some());
    }

    #[test]
    fn resolver_args_google_dns() {
        let opts = Opts {
            resolver: Some("8.8.8.8,8.8.4.4".to_owned()),
            ..Default::default()
        };
        let resolver = get_resolver(&opts.resolver);
        let lookup = resolver.lookup_ip("www.example.com.").unwrap();
        assert!(lookup.iter().next().is_some());
    }
}