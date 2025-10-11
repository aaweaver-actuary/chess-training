use chrono::NaiveDate;
use scheduler_core::{InMemoryStore, Scheduler, SchedulerConfig};
use uuid::Uuid;

/// Core scheduler wrapper shared between Rust unit tests and the wasm bindings.
pub struct SchedulerFacade {
    inner: Scheduler<InMemoryStore>,
    config: SchedulerConfig,
}

impl SchedulerFacade {
    /// Construct a scheduler facade backed by a fresh in-memory store.
    #[must_use]
    pub fn new(config: SchedulerConfig) -> Self {
        let config_clone = config.clone();
        let inner = Scheduler::new(InMemoryStore::new(), config);
        Self {
            inner,
            config: config_clone,
        }
    }

    /// Returns the configuration used by the facade.
    #[must_use]
    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }

    /// Builds the queue for the supplied owner and returns its length.
    #[must_use]
    pub fn queue_length(&mut self, owner_id: Uuid, today: NaiveDate) -> usize {
        self.inner.build_queue(owner_id, today).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid test date")
    }

    #[test]
    fn queue_length_is_zero_without_cards() {
        let mut facade = SchedulerFacade::new(SchedulerConfig::default());
        let owner_id = Uuid::nil();
        let today = naive_date(2024, 1, 1);
        assert_eq!(facade.queue_length(owner_id, today), 0);
    }
}
