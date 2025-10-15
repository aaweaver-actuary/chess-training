//! Domain model structs shared by card-store implementations.

use std::collections::{HashMap, HashSet};

// Use canonical card types from review-domain
pub use review_domain::{
    CardKind as GenericCardKind, EdgeInput, OpeningCard, OpeningEdge, ReviewRequest,
    StoredCardState, TacticCard, UnlockDetail, UnlockRecord as GenericUnlockRecord,
};

/// `CardKind` type with concrete generics for this store.
pub type CardKind = GenericCardKind<OpeningCard, TacticCard>;
/// Card type with concrete generics for this store.
pub type Card = review_domain::Card<u64, String, CardKind, StoredCardState>;
/// `UnlockRecord` type with concrete generics for this store.
pub type UnlockRecord = review_domain::UnlockRecord<String, UnlockDetail>;

pub use scheduler_core::domain::{
    CardStateBridgeError, Sm2Runtime, StoredSnapshot, hydrate_sm2_state, persist_sm2_state,
};

use review_domain::hash_with_seed;

/// Opening edge describing a transition between two positions.
pub type Edge = OpeningEdge;

/// Hash Map from an integer ID to an [`Edge`].
pub type EdgeMap = HashMap<u64, Edge>;

// PositionMap is not defined because ChessPosition is not re-exported from review-domain.
// pub type PositionMap = HashMap<u64, ChessPosition>;

/// Set of unlock records.
pub type UnlockSet = HashSet<UnlockRecord>;

/// Deterministically compute a card identifier for an opening edge.
#[must_use]
pub fn build_opening_card_id(owner_id: &str, edge_id: u64) -> u64 {
    // Compose a string key for deterministic hashing
    let key = format!("{owner_id}:{edge_id}");
    hash_with_seed(&key)
}

/// Deterministically compute a card identifier for a tactic.
#[must_use]
pub fn build_tactic_card_id(owner_id: &str, tactic_id: u64) -> u64 {
    let key = format!("{owner_id}:{tactic_id}");
    hash_with_seed(&key)
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
