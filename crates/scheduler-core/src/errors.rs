//! Error types returned by the scheduler.

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("card not found: {0}")]
    CardNotFound(Uuid),
}

#[cfg(test)]
mod tests {
    use super::SchedulerError;
    use uuid::Uuid;

    #[test]
    fn card_not_found_displays_identifier() {
        let id = Uuid::nil();
        let err = SchedulerError::CardNotFound(id);
        assert!(err.to_string().contains(&id.to_string()));
    }
}
