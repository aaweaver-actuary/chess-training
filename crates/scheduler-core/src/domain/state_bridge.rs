use review_domain::StoredCardState;

use chrono::NaiveDate;

/// Error type for state bridge conversions.
#[derive(Debug, Clone, PartialEq)]
pub enum BridgeError {
    IntervalTooSmall,
    IntervalOverflow { interval_days: u32, max: u8 },
}

/// Runtime counters and stage for SM-2 scheduling.
#[derive(Debug, Clone, PartialEq)]
pub struct Sm2Runtime {
    pub stage: super::card_state::CardState,
    pub lapses: u32,
    pub reviews: u32,
}

/// Snapshot of stored review state for persistence.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredSnapshot {
    pub consecutive_correct: u32,
    pub last_reviewed_on: Option<NaiveDate>,
}

use super::Sm2State;

/// Convert a persisted [`StoredCardState`] plus runtime counters into an [`Sm2State`].
///
/// # Panics
/// Panics if the conversion from stored state is not infallible (should never happen).
#[must_use]
pub fn hydrate_sm2_state(stored: StoredCardState, runtime: Sm2Runtime) -> Sm2State {
    Sm2State::from((stored, runtime))
}

/// Convert an [`Sm2State`] back into a [`StoredCardState`] for persistence.
///
/// # Errors
/// Returns a [`BridgeError`] if the interval is zero or overflows u8.
pub fn persist_sm2_state(
    sm2: &Sm2State,
    snapshot: &StoredSnapshot,
) -> Result<StoredCardState, BridgeError> {
    use std::num::NonZeroU8;
    if sm2.interval_days == 0 {
        return Err(BridgeError::IntervalTooSmall);
    }
    let interval_u8 =
        u8::try_from(sm2.interval_days).map_err(|_| BridgeError::IntervalOverflow {
            interval_days: sm2.interval_days,
            max: u8::MAX,
        })?;
    let interval = NonZeroU8::new(interval_u8).ok_or(BridgeError::IntervalTooSmall)?;
    Ok(StoredCardState {
        due_on: sm2.due,
        interval,
        ease_factor: sm2.ease_factor,
        consecutive_correct: snapshot.consecutive_correct,
        last_reviewed_on: snapshot.last_reviewed_on,
    })
}

impl From<(StoredCardState, Sm2Runtime)> for Sm2State {
    fn from(value: (StoredCardState, Sm2Runtime)) -> Self {
        let (stored, runtime) = value;
        Self {
            stage: runtime.stage,
            ease_factor: stored.ease_factor,
            interval_days: u32::from(stored.interval.get()),
            due: stored.due_on,
            lapses: runtime.lapses,
            reviews: runtime.reviews,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CardState;
    use chrono::NaiveDate;
    use std::num::NonZeroU8;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn sample_stored_state() -> StoredCardState {
        StoredCardState {
            due_on: naive_date(2024, 1, 1),
            interval: NonZeroU8::new(5).expect("non-zero interval"),
            ease_factor: 2.4,
            consecutive_correct: 3,
            last_reviewed_on: Some(naive_date(2023, 12, 31)),
        }
    }

    #[test]
    fn round_trip_preserves_fields() {
        let stored = sample_stored_state();
        let runtime = Sm2Runtime {
            stage: CardState::Review,
            lapses: 2,
            reviews: 7,
        };
        let sm2 = hydrate_sm2_state(stored.clone(), runtime.clone());
        assert_eq!(sm2.stage, runtime.stage);
        assert_eq!(sm2.lapses, runtime.lapses);
        assert_eq!(sm2.reviews, runtime.reviews);
        assert_eq!(sm2.due, stored.due_on);
        assert_eq!(sm2.interval_days, u32::from(stored.interval.get()));
        assert!((sm2.ease_factor - stored.ease_factor).abs() < f32::EPSILON);

        let snapshot = StoredSnapshot {
            consecutive_correct: stored.consecutive_correct,
            last_reviewed_on: stored.last_reviewed_on,
        };
        let persisted = persist_sm2_state(&sm2, &snapshot).expect("conversion should succeed");
        assert_eq!(persisted, stored);
    }

    #[test]
    fn persist_sm2_state_rejects_large_interval() {
        let stored = sample_stored_state();
        let runtime = Sm2Runtime {
            stage: CardState::Review,
            lapses: 0,
            reviews: 0,
        };
        let mut sm2 = hydrate_sm2_state(stored, runtime);
        sm2.interval_days = 512;
        let snapshot = StoredSnapshot {
            consecutive_correct: 0,
            last_reviewed_on: None,
        };
        let err = persist_sm2_state(&sm2, &snapshot).expect_err("interval overflow");
        if let BridgeError::IntervalOverflow { interval_days, max } = err {
            assert_eq!(interval_days, 512);
            assert_eq!(max, u8::MAX);
        } else {
            panic!("unexpected error: {err:?}");
        }
    }

    #[test]
    fn persist_sm2_state_rejects_zero_interval() {
        let stored = sample_stored_state();
        let runtime = Sm2Runtime {
            stage: CardState::Review,
            lapses: 0,
            reviews: 0,
        };
        let mut sm2 = hydrate_sm2_state(stored, runtime);
        sm2.interval_days = 0;
        let snapshot = StoredSnapshot {
            consecutive_correct: 0,
            last_reviewed_on: None,
        };
        let err = persist_sm2_state(&sm2, &snapshot).expect_err("zero interval");
        assert!(matches!(err, BridgeError::IntervalTooSmall));
    }
}
