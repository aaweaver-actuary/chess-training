use std::convert::Infallible;
use std::num::NonZeroU8;

use review_domain::StoredCardState;
use review_domain::card_state::bridge::{BridgeError, Sm2Runtime, StoredSnapshot};

use super::Sm2State;

/// Convert a persisted [`StoredCardState`] plus runtime counters into an [`Sm2State`].
pub fn hydrate_sm2_state(stored: StoredCardState, runtime: Sm2Runtime) -> Sm2State {
    Sm2State::try_from((stored, runtime))
        .expect("conversion from stored state should be infallible")
}

/// Convert an [`Sm2State`] back into a [`StoredCardState`] for persistence.
pub fn persist_sm2_state(
    sm2: &Sm2State,
    snapshot: StoredSnapshot,
) -> Result<StoredCardState, BridgeError> {
    StoredCardState::try_from((sm2, snapshot))
}

impl TryFrom<(StoredCardState, Sm2Runtime)> for Sm2State {
    type Error = Infallible;

    fn try_from(value: (StoredCardState, Sm2Runtime)) -> Result<Self, Self::Error> {
        let (stored, runtime) = value;
        Ok(Sm2State {
            stage: runtime.stage,
            ease_factor: stored.ease_factor,
            interval_days: u32::from(stored.interval.get()),
            due: stored.due_on,
            lapses: runtime.lapses,
            reviews: runtime.reviews,
        })
    }
}

impl TryFrom<(&Sm2State, StoredSnapshot)> for StoredCardState {
    type Error = BridgeError;

    fn try_from(value: (&Sm2State, StoredSnapshot)) -> Result<Self, Self::Error> {
        let (sm2, snapshot) = value;
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
        let sm2 = hydrate_sm2_state(stored.clone(), runtime);
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
        let persisted = persist_sm2_state(&sm2, snapshot).expect("conversion should succeed");
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
        let err = persist_sm2_state(&sm2, snapshot).expect_err("interval overflow");
        match err {
            BridgeError::IntervalOverflow { interval_days, max } => {
                assert_eq!(interval_days, 512);
                assert_eq!(max, u8::MAX);
            }
            other => panic!("unexpected error: {other:?}"),
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
        let err = persist_sm2_state(&sm2, snapshot).expect_err("zero interval");
        assert!(matches!(err, BridgeError::IntervalTooSmall));
    }
}
