//! Daily queue construction that merges due reviews with newly unlocked cards.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::config::SchedulerConfig;
use crate::domain::{Card, CardKind, CardState, SchedulerUnlockDetail, UnlockRecord};
use crate::store::CardStore;

#[must_use]
pub fn build_queue_for_day<S: CardStore>(
    store: &mut S,
    config: &SchedulerConfig,
    owner_id: Uuid,
    today: NaiveDate,
) -> Vec<Card> {
    let mut queue = store.due_cards(owner_id, today);
    let prior_unlocks = store.unlocked_on(owner_id, today);
    let mut unlocked = ExistingUnlocks::from_records(&prior_unlocks);
    extend_queue_with_unlocks(store, config, owner_id, today, &mut queue, &mut unlocked);
    queue.sort_by(|a, b| (a.state.due, a.id).cmp(&(b.state.due, b.id)));
    queue
}

struct ExistingUnlocks {
    prefixes: BTreeSet<String>,
    ids: BTreeSet<Uuid>,
}

impl ExistingUnlocks {
    fn from_records(records: &[UnlockRecord]) -> Self {
        let prefixes = records
            .iter()
            .filter_map(|record| record.detail.parent_prefix.clone())
            .collect();
        let ids = records.iter().map(|record| record.detail.card_id).collect();
        Self { prefixes, ids }
    }

    fn contains_prefix(&self, prefix: &str) -> bool {
        self.prefixes.contains(prefix)
    }

    fn contains_card(&self, id: &Uuid) -> bool {
        self.ids.contains(id)
    }

    fn track_new_unlock(&mut self, prefix: Option<String>, id: Uuid) {
        if let Some(prefix) = prefix {
            self.prefixes.insert(prefix);
        }
        self.ids.insert(id);
    }
}

fn extend_queue_with_unlocks<S: CardStore>(
    store: &mut S,
    config: &SchedulerConfig,
    owner_id: Uuid,
    today: NaiveDate,
    queue: &mut Vec<Card>,
    unlocked: &mut ExistingUnlocks,
) {
    for mut candidate in store.unlock_candidates(owner_id) {
        if skip_candidate(&candidate, unlocked) {
            continue;
        }
        let parent_prefix = extract_prefix(&candidate);
        unlock_card(&mut candidate, config, today);
        store.record_unlock(UnlockRecord {
            owner_id,
            detail: SchedulerUnlockDetail {
                card_id: candidate.id,
                parent_prefix: parent_prefix.clone(),
            },
            unlocked_on: today,
        });
        unlocked.track_new_unlock(parent_prefix, candidate.id);
        store.upsert_card(candidate.clone());
        queue.push(candidate);
    }
}

fn skip_candidate(candidate: &Card, unlocked: &ExistingUnlocks) -> bool {
    if unlocked.contains_card(&candidate.id) {
        return true;
    }
    match &candidate.kind {
        CardKind::Opening(opening) => unlocked.contains_prefix(&opening.parent_prefix),
        CardKind::Tactic(_) => true,
    }
}

fn unlock_card(card: &mut Card, config: &SchedulerConfig, today: NaiveDate) {
    card.state.stage = CardState::Learning;
    card.state.interval_days = 0;
    card.state.due = today;
    card.state.ease_factor = config.initial_ease_factor;
}

fn extract_prefix(card: &Card) -> Option<String> {
    match &card.kind {
        CardKind::Opening(opening) => Some(opening.parent_prefix.clone()),
        CardKind::Tactic(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        CardKind, SchedulerOpeningCard, SchedulerTacticCard, SchedulerUnlockDetail, new_card,
    };
    use crate::store::InMemoryStore;

    fn naive_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn sample_opening(owner: Uuid, prefix: &str) -> Card {
        let config = SchedulerConfig::default();
        new_card(
            owner,
            CardKind::Opening(SchedulerOpeningCard::new(prefix)),
            naive_date(2023, 1, 1),
            &config,
        )
    }

    #[test]
    fn build_queue_skips_already_unlocked_prefixes() {
        let mut store = InMemoryStore::new();
        let config = SchedulerConfig::default();
        let owner = Uuid::new_v4();
        let existing = sample_opening(owner, "e4");
        store.upsert_card(existing.clone());
        store.record_unlock(UnlockRecord {
            owner_id: owner,
            detail: SchedulerUnlockDetail {
                card_id: existing.id,
                parent_prefix: Some("e4".into()),
            },
            unlocked_on: naive_date(2023, 1, 1),
        });
        let new_candidate = sample_opening(owner, "e4");
        store.upsert_card(new_candidate.clone());

        let queue = build_queue_for_day(&mut store, &config, owner, naive_date(2023, 1, 1));
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn build_queue_unlocks_new_opening() {
        let mut store = InMemoryStore::new();
        let config = SchedulerConfig::default();
        let owner = Uuid::new_v4();
        let candidate = sample_opening(owner, "c4");
        store.upsert_card(candidate.clone());

        let queue = build_queue_for_day(&mut store, &config, owner, naive_date(2023, 1, 1));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].state.stage, CardState::Learning);
    }

    #[test]
    fn skip_candidate_blocks_previously_seen_card() {
        let mut store = InMemoryStore::new();
        let config = SchedulerConfig::default();
        let owner = Uuid::new_v4();
        let mut queue = Vec::new();
        let prior_unlock = sample_opening(owner, "d4");
        let duplicate = sample_opening(owner, "d4");
        store.upsert_card(prior_unlock.clone());
        store.upsert_card(duplicate);
        store.record_unlock(UnlockRecord {
            owner_id: owner,
            detail: SchedulerUnlockDetail {
                card_id: prior_unlock.id,
                parent_prefix: Some("d4".into()),
            },
            unlocked_on: naive_date(2023, 1, 2),
        });

        let mut existing =
            ExistingUnlocks::from_records(&store.unlocked_on(owner, naive_date(2023, 1, 2)));
        extend_queue_with_unlocks(
            &mut store,
            &config,
            owner,
            naive_date(2023, 1, 3),
            &mut queue,
            &mut existing,
        );

        assert!(queue.is_empty());
    }

    #[test]
    fn skip_candidate_ignores_tactic_cards() {
        let owner = Uuid::new_v4();
        let mut store = InMemoryStore::new();
        let config = SchedulerConfig::default();
        let mut queue = Vec::new();
        let tactic_card = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &config,
        );
        store.upsert_card(tactic_card.clone());

        let mut existing = ExistingUnlocks::from_records(&[]);
        extend_queue_with_unlocks(
            &mut store,
            &config,
            owner,
            naive_date(2023, 1, 2),
            &mut queue,
            &mut existing,
        );

        assert!(queue.is_empty());
        assert!(!existing.contains_card(&tactic_card.id));
    }

    #[test]
    fn extract_prefix_returns_none_for_tactic_cards() {
        let owner = Uuid::new_v4();
        let config = SchedulerConfig::default();
        let tactic_card = new_card(
            owner,
            CardKind::Tactic(SchedulerTacticCard::new()),
            naive_date(2023, 1, 1),
            &config,
        );

        assert!(extract_prefix(&tactic_card).is_none());
    }
}
