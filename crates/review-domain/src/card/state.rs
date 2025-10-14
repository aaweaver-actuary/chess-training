
    #[derive(Clone, Debug, PartialEq)]
    struct CardState {
        ease: f32,
        interval_days: u32,
        lapses: u32,
    }

    impl CardState {
        fn new(ease: f32, interval_days: u32, lapses: u32) -> Self {
            Self {
                ease,
                interval_days,
                lapses,
            }
        }
    }
