//! Domain model structs shared by card-store implementations.

use std::collections::{HashMap, HashSet};

/// Re-export shared review-domain types to simplify crate consumers.
pub use review_domain::{EdgeInput, OpeningCard, ReviewRequest, StoredCardState, TacticCard};
pub use scheduler_core::domain::{
    hydrate_sm2_state, persist_sm2_state, CardStateBridgeError, Sm2Runtime, StoredSnapshot,
};

use review_domain::{
    Card as GenericCard, CardKind as GenericCardKind, ChessPosition, OpeningEdge,
    UnlockDetail as GenericUnlockDetail, UnlockRecord as GenericUnlockRecord,
};

use crate::hash64;

/// Opening edge describing a transition between two positions.
pub type Edge = OpeningEdge;

/// Hash Map from an integer ID to an [`Edge`].
pub type EdgeMap = HashMap<u64, Edge>;

/// Classification of a card target.
pub type CardKind = GenericCardKind<OpeningCard, TacticCard>;

/// Hash Map from an integer ID to a [`Card`].
pub type CardMap = HashMap<u64, GenericCard<u64, String, CardKind, StoredCardState>>;

/// Hash Map from a position ID to a [`ChessPosition`]
pub type PositionMap = HashMap<u64, ChessPosition>;

/// Set of unlock records.
pub type UnlockSet = HashSet<UnlockRecord>;

/// Specialized review card type for the card-store service.
pub type Card = GenericCard<u64, String, CardKind, StoredCardState>;

/// Domain payload stored for each unlock record.
pub type UnlockDetail = GenericUnlockDetail;

/// Unlock ledger entry representing newly released opening moves.
pub type UnlockRecord = GenericUnlockRecord<String, UnlockDetail>;

/// Deterministically compute a card identifier for an opening edge.
#[must_use]
pub fn build_opening_card_id(owner_id: &str, edge_id: u64) -> u64 {
    hash64(&[owner_id.as_bytes(), &edge_id.to_be_bytes()])
}

/// Deterministically compute a card identifier for a tactic.
#[must_use]
pub fn build_tactic_card_id(owner_id: &str, tactic_id: u64) -> u64 {
    hash64(&[owner_id.as_bytes(), &tactic_id.to_be_bytes()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use review_domain::CardKind as GenericCardKind;
    use review_domain::{EdgeId, UnlockDetail};

    fn increment_opening(card: OpeningCard) -> OpeningCard {
        OpeningCard::new(EdgeId::new(card.edge_id.get() + 1))
    }

    fn increment_tactic(card: TacticCard) -> TacticCard {
        TacticCard::new(review_domain::TacticId::new(card.tactic_id.get() + 1))
    }

    fn tactic_identifier(card: TacticCard) -> review_domain::TacticId {
        review_domain::TacticId::new(card.tactic_id.get() + 1)
    }

    #[test]
    fn build_tactic_card_id_depends_on_inputs() {
        let base = build_tactic_card_id("owner", 42);
        assert_ne!(base, build_tactic_card_id("owner", 43));
        assert_ne!(base, build_tactic_card_id("other", 42));
    }

    #[test]
    fn build_opening_card_id_depends_on_inputs() {
        let base = build_opening_card_id("owner", 7);
        assert_ne!(base, build_opening_card_id("owner", 8));
        assert_ne!(base, build_opening_card_id("other", 7));
    }

    #[test]
    fn card_kind_helpers_cover_review_domain_types() {
        let opening = OpeningCard::new(EdgeId::new(7));
        let mapped_opening = CardKind::Opening(opening).map_opening(increment_opening);
        assert!(matches!(
            mapped_opening,
            CardKind::Opening(card) if card.edge_id == EdgeId::new(8)
        ));
        assert!(matches!(
            GenericCardKind::<OpeningCard, TacticCard>::Tactic(TacticCard::new(
                review_domain::TacticId::new(13)
            ))
                .map_opening(increment_opening),
            GenericCardKind::Tactic(tactic) if tactic.tactic_id == review_domain::TacticId::new(13)
        ));

        let tactic_kind = CardKind::Tactic(TacticCard::new(review_domain::TacticId::new(11)));
        assert!(matches!(
            tactic_kind.clone().map_tactic(tactic_identifier),
            GenericCardKind::Tactic(identifier)
                if identifier == review_domain::TacticId::new(12)
        ));
        assert!(matches!(
            tactic_kind.as_ref(),
            GenericCardKind::Tactic(payload)
                if payload.tactic_id == review_domain::TacticId::new(11)
        ));
        assert!(matches!(
            GenericCardKind::<OpeningCard, TacticCard>::Opening(OpeningCard::new(EdgeId::new(5)))
                .map_opening(increment_opening),
            GenericCardKind::Opening(card) if card.edge_id == EdgeId::new(6)
        ));
        assert!(matches!(
            GenericCardKind::<OpeningCard, TacticCard>::Opening(OpeningCard::new(EdgeId::new(5)))
                .map_tactic(increment_tactic),
            GenericCardKind::Opening(card) if card.edge_id == EdgeId::new(5)
        ));
        assert!(matches!(
            GenericCardKind::<OpeningCard, TacticCard>::Opening(OpeningCard::new(EdgeId::new(9))).as_ref(),
            GenericCardKind::Opening(reference) if reference.edge_id == EdgeId::new(9)
        ));
        assert!(matches!(
            GenericCardKind::<OpeningCard, TacticCard>::Tactic(TacticCard::new(
                review_domain::TacticId::new(21)
            ))
                .map_tactic(increment_tactic),
            GenericCardKind::Tactic(card)
                if card.tactic_id == review_domain::TacticId::new(22)
        ));
        assert!(matches!(
            GenericCardKind::<OpeningCard, TacticCard>::Tactic(TacticCard::new(
                review_domain::TacticId::new(17)
            ))
                .map_tactic(tactic_identifier),
            GenericCardKind::Tactic(identifier)
                if identifier == review_domain::TacticId::new(18)
        ));

        let edge = OpeningEdge::new(1, 2, 3, "e2e4", "e4");
        assert_eq!(edge.move_uci, "e2e4");
        assert_eq!(edge.move_san, "e4");

        let unlock = UnlockRecord {
            owner_id: String::from("owner"),
            detail: UnlockDetail::new(EdgeId::new(9)),
            unlocked_on: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).expect("valid date"),
        };
        let mapped_unlock = unlock.map_detail(|detail| detail.edge_id);
        assert_eq!(mapped_unlock.detail, EdgeId::new(9));
    }
}
