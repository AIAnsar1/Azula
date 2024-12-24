use async_std::task::block_on;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::net::IpAddr;
use std::time::Duration;
use azula::config::base::{PortRange, ScanOrder};
use azula::net::strategy::PortStrategy;
use azula::scanner::Scanner;



fn azula_tcp(scanner: &Scanner) {
    let _scan_result = block_on(scanner.run());
}

fn azula_udp(scanner: &Scanner) {
    let _scan_result = block_on(scanner.run());
}

fn bench_address() {
    let _address = ["127.0.0.1".parse::<IpAddr>().unwrap()];
}

fn bench_port_strategy() {
    let range = PortRange {
        start: 1,
        end: 1_000,
    };
    let _strategy = PortStrategy::pick(&Some(range.clone()), None, ScanOrder::Serial);
}

fn criterion_benchmark(c: &mut Criterion) {
    let address = vec!["127.0.0.1".parse::<IpAddr>().unwrap()];
    let range = PortRange {
        start: 1, end: 1_000,
    };
    let strategy_tcp = PortStrategy::pick(&Some(range.clone()), None, ScanOrder::Serial);
    let strategy_udp = PortStrategy::pick(&Some(range.clone()), None, ScanOrder::Serial);
    let scanner_tco = Scanner::new(&address, 10, Duration::from_millis(10), 1, false, strategy_tcp, true, vec![], false);
    c.banch_function("azula tcp", |b| {
        b.iter(|| azula_tcp(black_box(&scanner_tco)))
    });
    let scanner_udp = Scanner::new(&address, 10, Duration::from_millis(10), 1, false, strategy_udp, true, vec!{}, true);
    let mut udp_group = c.benchmark_group("azula udp");
    udp_group.measurement_time(Duration::from_secs(20));
    udp_group.bench_function("azula udp", |b| {
        b.iter(|| azula_udp(black_box(&scanner_udp)))
    });
    udp_group.finish();
    c.bench_function("parse address", |b| b.iter(bench_address));
    c.bench_function("parse strategy", |b| b.iter(bench_port_strategy));
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);