

mod test {

    use azula::benchmark::benches::*;

    #[test]
    fn benchmark() {
        let mut benchmarks = Benchmark::init();
        let mut test_timer = NamedTimer::start("test");
        std::thread::sleep(std::time::Duration::from_millis(100));
        test_timer.end();
        benchmarks.push(test_timer);
        benchmarks.push(NamedTimer::start("only_start"));
        assert!(benchmarks.summary().contains("\nAzula Benchmark Summary\ntest.         | 0."));
        assert!(benchmarks.summary().contains("only_start"))
    }

}