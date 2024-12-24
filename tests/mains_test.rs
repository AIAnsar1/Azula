


mod test {
    #[cfg(unix)]
    use azula::main::{adjust_ulimit_size, infer_batch_size, print_opening};
    use azula::config::base::{Opts};


    #[test]
    #[cfg(unix)]
    fn batch_size_lowered() {
        let opts = Opts {
            batch_size: 50000,
            ..Default::default()
        };
        let batch_size = infer_batch_size(&opts, 120);
        assert!(batch_size < opts.batch_size);
    }

    #[test]
    #[cfg(unix)]
    fn batch_size_lowered_average_size() {
        let opts = Opts {
            batch_size: 50000,
            ..Default::default()
        };
        let batch_size = infer_batch_size(&opts, 9000);
        assert!(batch_size < 3000);
    }

    #[test]
    #[cfg(unix)]
    fn batch_size_equals_ulimit_lowered() {
        let opts = Opts {
            batch_size: 50000,
            ..Default::default()
        };
        let batch_size = infer_batch_size(&opts, 5000);
        assert!(batch_size < 4900);
    }

    #[test]
    #[cfg(unix)]
    fn batch_size_adjusted_2000() {
        let opts = Opts {
            batch_size: 50000,
            ulimit: Some(2000),
            ..Default::default()
        };
        let batch_size = adjust_ulimit_size(&opts);
        assert!(batch_size < 2000);
    }

    #[test]
    #[cfg(unix)]
    fn test_high_ulimit_no_greppable_mode() {
        let opts = Opts {
            batch_size: 10,
            greppable: false,
            ..Default::default()
        };
        let batch_size = infer_batch_size(&opts, 1000000);
        assert!(batch_size == opts.batch_size);
    }

    #[test]
    fn test_print_opening_no_panic() {
        let opts = Opts {
            ulimit: Some(2000),
            ..Default::default()
        };
        print_opening(&opts);
    }
}