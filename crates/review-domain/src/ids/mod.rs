//! Type-safe identifier wrappers shared across review domain modules.
pub mod id_conversion_error;
pub mod id_kind;

pub mod card_id;
pub mod edge_id;
pub mod learner_id;
pub mod move_id;
pub mod position_id;
pub mod tactic_id;
pub mod unlock_id;

pub use id_conversion_error::IdConversionError;
pub use id_kind::IdKind;

pub use card_id::CardId;
pub use edge_id::EdgeId;
pub use learner_id::LearnerId;
pub use move_id::MoveId;
pub use position_id::PositionId;
pub use tactic_id::TacticId;
pub use unlock_id::UnlockId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_representation_matches_display() {
        let position = PositionId::new(91);
        assert_eq!(format!("{position}"), format!("{position:?}"));
    }

    #[test]
    fn default_is_zero() {
        let edge = EdgeId::default();
        assert_eq!(edge.get(), 0);

        let tactic = TacticId::default();
        assert_eq!(tactic.get(), 0);
    }

    #[test]
    fn try_from_u128_succeeds_within_range() {
        assert_eq!(PositionId::try_from(1_u128).unwrap().get(), 1);
        assert_eq!(EdgeId::try_from(2_u128).unwrap().get(), 2);
        assert_eq!(MoveId::try_from(3_u128).unwrap().get(), 3);
        assert_eq!(CardId::try_from(4_u128).unwrap().get(), 4);
        assert_eq!(LearnerId::try_from(5_u128).unwrap().get(), 5);
        assert_eq!(UnlockId::try_from(6_u128).unwrap().get(), 6);
    }

    #[test]
    fn try_from_u128_reports_overflow() {
        let overflow_value = u128::from(u64::MAX) + 1;

        assert_eq!(
            PositionId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Position,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            EdgeId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Edge,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            MoveId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Move,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            CardId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Card,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            LearnerId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Learner,
                value: overflow_value,
                max: u64::MAX,
            }
        );
        assert_eq!(
            UnlockId::try_from(overflow_value).unwrap_err(),
            IdConversionError::Overflow {
                kind: IdKind::Unlock,
                value: overflow_value,
                max: u64::MAX,
            }
        );
    }

    #[test]
    fn try_from_i64_reports_negative_values() {
        assert_eq!(
            PositionId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Position,
                value: -1,
            }
        );
        assert_eq!(
            EdgeId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Edge,
                value: -1,
            }
        );
        assert_eq!(
            MoveId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Move,
                value: -1,
            }
        );
        assert_eq!(
            CardId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Card,
                value: -1,
            }
        );
        assert_eq!(
            LearnerId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Learner,
                value: -1,
            }
        );
        assert_eq!(
            UnlockId::try_from(-1_i64).unwrap_err(),
            IdConversionError::Negative {
                kind: IdKind::Unlock,
                value: -1,
            }
        );
    }
}
