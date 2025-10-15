#[derive(Clone, Debug, PartialEq)]
pub struct CardState {
    pub ease: f32,
    pub interval_days: u32,
    pub lapses: u32,
}

impl CardState {
    #[must_use]
    pub fn new(ease: f32, interval_days: u32, lapses: u32) -> Self {
        Self {
            ease,
            interval_days,
            lapses,
        }
    }
}
