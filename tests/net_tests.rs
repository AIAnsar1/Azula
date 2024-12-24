

#[cfg(test)]
mod tests {
    use azula::config::base::{PortRange, ScanOrder};
    use azula::net::strategy::*;

    #[test]
    fn serial_strategy_with_range() {
        let range = PortRange { start: 1, end: 0};
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Serial);
        let result = strategy.order();
        let expected_range = (1..10).collect::<Vec<u16>>();
        assert_eq!(expected_range, result)
    }

    #[test]
    fn random_strategy_with_range() {
        let range = PortRange { start: 1, end: 0};
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Random);
        let mut result = strategy.order();
        let expected_range = (1..10).collect::<Vec<u16>>();
        assert_ne!(expected_range, result);

        result.sort_unstable();
        assert_eq!(expected_range, result)
    }

    #[test]
    fn serial_strategy_with_ports() {
        let strategy = PortStrategy::pick(&None, Some(vec![80, 443]), ScanOrder::Serial);
        let result = strategy.order();
        assert_eq!(vec![80, 443], result)
    }

    #[test]
    fn random_strategy_with_ports() {
        let strategy = PortStrategy::pick(&None, Some((1..10).collect()), ScanOrder::Random);
        let mut result = strategy.order();
        let expected_range = (1..10).collect::<Vec<u16>>();
        assert_ne!(expected_range, result);

        result.sort_unstable();
        assert_eq!(expected_range, result)

    }
}