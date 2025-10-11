use scheduler_core::SchedulerConfig;
use serde::{Deserialize, Serialize};

/// Serializable snapshot of the scheduler configuration exposed to JavaScript consumers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SchedulerConfigDto {
    pub initial_ease_factor: f32,
    pub ease_minimum: f32,
    pub ease_maximum: f32,
    pub learning_steps_minutes: Vec<u32>,
}

impl From<&SchedulerConfig> for SchedulerConfigDto {
    fn from(config: &SchedulerConfig) -> Self {
        Self {
            initial_ease_factor: config.initial_ease_factor,
            ease_minimum: config.ease_minimum,
            ease_maximum: config.ease_maximum,
            learning_steps_minutes: config.learning_steps_minutes.clone(),
        }
    }
}

/// Partial configuration update supplied from JavaScript.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct SchedulerConfigPatch {
    pub initial_ease_factor: Option<f32>,
    pub ease_minimum: Option<f32>,
    pub ease_maximum: Option<f32>,
    pub learning_steps_minutes: Option<Vec<u32>>,
}

impl SchedulerConfigPatch {
    /// Applies the patch values to the provided configuration baseline.
    #[must_use]
    pub fn apply(self, mut base: SchedulerConfig) -> SchedulerConfig {
        if let Some(initial_ease_factor) = self.initial_ease_factor {
            base.initial_ease_factor = initial_ease_factor;
        }
        if let Some(ease_minimum) = self.ease_minimum {
            base.ease_minimum = ease_minimum;
        }
        if let Some(ease_maximum) = self.ease_maximum {
            base.ease_maximum = ease_maximum;
        }
        if let Some(learning_steps_minutes) = self.learning_steps_minutes {
            base.learning_steps_minutes = learning_steps_minutes;
        }
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> SchedulerConfig {
        SchedulerConfig {
            initial_ease_factor: 2.5,
            ease_minimum: 1.3,
            ease_maximum: 2.8,
            learning_steps_minutes: vec![1, 10],
        }
    }

    fn approx_eq(lhs: f32, rhs: f32) -> bool {
        (lhs - rhs).abs() <= f32::EPSILON
    }

    #[test]
    fn dto_reflects_configuration_values() {
        let config = baseline();
        let dto = SchedulerConfigDto::from(&config);
        assert!(approx_eq(
            dto.initial_ease_factor,
            config.initial_ease_factor
        ));
        assert!(approx_eq(dto.ease_minimum, config.ease_minimum));
        assert!(approx_eq(dto.ease_maximum, config.ease_maximum));
        assert_eq!(dto.learning_steps_minutes, config.learning_steps_minutes);
    }

    #[test]
    fn patch_overrides_selected_fields() {
        let patch = SchedulerConfigPatch {
            initial_ease_factor: Some(2.8),
            ease_minimum: None,
            ease_maximum: Some(3.0),
            learning_steps_minutes: Some(vec![1, 5, 10]),
        };
        let patched = patch.apply(baseline());
        assert!(approx_eq(patched.initial_ease_factor, 2.8));
        assert!(approx_eq(patched.ease_minimum, 1.3));
        assert!(approx_eq(patched.ease_maximum, 3.0));
        assert_eq!(patched.learning_steps_minutes, vec![1, 5, 10]);
    }
}
