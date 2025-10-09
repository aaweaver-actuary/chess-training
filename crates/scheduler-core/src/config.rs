//! Scheduler configuration values governing SM-2 calculations and unlock policy.

#[derive(Debug, Clone, PartialEq)]
pub struct SchedulerConfig {
    pub initial_ease_factor: f32,
    pub ease_minimum: f32,
    pub ease_maximum: f32,
    pub learning_steps_minutes: Vec<u32>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            initial_ease_factor: 2.5,
            ease_minimum: 1.3,
            ease_maximum: 2.8,
            learning_steps_minutes: vec![1, 10],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_configuration_matches_expected_values() {
        let config = SchedulerConfig::default();
        assert!((config.initial_ease_factor - 2.5).abs() <= f32::EPSILON);
        assert!((config.ease_minimum - 1.3).abs() <= f32::EPSILON);
        assert!((config.ease_maximum - 2.8).abs() <= f32::EPSILON);
        assert_eq!(config.learning_steps_minutes, vec![1, 10]);
    }
}
