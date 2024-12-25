



mod test {
    use azula::config::base::SocketIterator;
    use azula::net::strategy::PortStrategy;
    use azula::config::base::{PortRange, ScanOrder};


    #[test]
    fn range_iterator_iterates_through_the_entire_range() {
        let result = generate_sorted_range(1, 10);
        let expected_range = (1..=10).collect::<Vec<u16>>();
        assert_eq!(expected_range, result);
        let result = generate_sorted_range(1, 100);
        let expected_range = (1..=100).collect::<Vec<u16>>();
        assert_eq!(expected_range, result);
        let result = generate_sorted_range(1, 1000);
        let expected_range = (1..=1000).collect::<Vec<u16>>();
        assert_eq!(expected_range, result);
        let result = generate_sorted_range(1, 65_535);
        let expected_range = (1..=65_535).collect::<Vec<u16>>();
        assert_eq!(expected_range, result);
        let result = generate_sorted_range(1000, 2000);
        let expected_range = (1000..=2000).collect::<Vec<u16>>();
        assert_eq!(expected_range, result);
    }

    fn generate_sorted_range(start: u32, end: u32) -> Vec<u16> {
        let range = RangeIterator::new(start, end);
        let mut result = range.into_iter().collect::<Vec<u16>>();
        result.sort_unstable();
        result
    }

    #[test]
    fn serial_strategy_with_range() {
        let range = PortRange { start: 1, end: 100 };
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Serial);
        let result = strategy.order();
        let expected_range = (1..=100).collect::<Vec<u16>>();
        assert_eq!(expected_range, result);
    }
    #[test]
    fn random_strategy_with_range() {
        let range = PortRange { start: 1, end: 100 };
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Random);
        let mut result = strategy.order();
        let expected_range = (1..=100).collect::<Vec<u16>>();
        assert_ne!(expected_range, result);
        result.sort_unstable();
        assert_eq!(expected_range, result);
    }

    #[test]
    fn serial_strategy_with_ports() {
        let strategy = PortStrategy::pick(&None, Some(vec![80, 443]), ScanOrder::Serial);
        let result = strategy.order();
        assert_eq!(vec![80, 443], result);
    }

    #[test]
    fn random_strategy_with_ports() {
        let strategy = PortStrategy::pick(&None, Some((1..10).collect()), ScanOrder::Random);
        let mut result = strategy.order();
        let expected_range = (1..10).collect::<Vec<u16>>();
        assert_ne!(expected_range, result);
        result.sort_unstable();
        assert_eq!(expected_range, result);
    }
}