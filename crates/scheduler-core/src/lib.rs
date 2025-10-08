//! Scheduler core library implementing SM-2 scheduling and unlock policy.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration, NaiveDate};
use thiserror::Error;
use uuid::Uuid;

/// Configuration for the scheduler.
#[derive(Debug, Clone, PartialEq)]
pub struct SchedulerConfig {
    /// Initial ease factor assigned to brand new cards.
    pub initial_ease_factor: f32,
    /// Minimum ease factor allowed.
    pub ease_minimum: f32,
    /// Maximum ease factor allowed.
    pub ease_maximum: f32,
    /// Learning steps in minutes for new cards.
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

/// Possible grades assigned to a review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewGrade {
    /// Repeat immediately.
    Again,
    /// Hard recall.
    Hard,
    /// Satisfactory recall.
    Good,
    /// Effortless recall.
    Easy,
}

/// Card state classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    /// Never seen.
    New,
    /// Currently in the learning queue.
    Learning,
    /// Graduated and scheduled via spaced repetition.
    Review,
    /// Relearning after a lapse.
    Relearning,
}

/// Different card kinds supported by the scheduler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardKind {
    /// Opening move referencing the shared parent prefix.
    Opening { parent_prefix: String },
    /// Tactic puzzle independent of the unlock policy.
    Tactic,
}

/// Record describing a card unlock event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlockRecord {
    /// Identifier of the unlocked card.
    pub card_id: Uuid,
    /// Card owner identifier.
    pub owner_id: Uuid,
    /// Shared prefix (if any) used to enforce unlock limits.
    pub parent_prefix: Option<String>,
    /// The day the unlock happened.
    pub day: NaiveDate,
}

/// Card data tracked by the scheduler.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    /// Unique identifier.
    pub id: Uuid,
    /// Card owner.
    pub owner_id: Uuid,
    /// Card kind (opening or tactic).
    pub kind: CardKind,
    /// SRS state.
    pub state: CardState,
    /// Ease factor used by SM-2 calculations.
    pub ease_factor: f32,
    /// Current interval expressed in whole days.
    pub interval_days: u32,
    /// Due date for the next review.
    pub due: NaiveDate,
    /// Number of lapses (Again responses).
    pub lapses: u32,
    /// Total reviews recorded.
    pub reviews: u32,
}

impl Card {
    /// Create a brand new card using the provided configuration.
    pub fn new(owner_id: Uuid, kind: CardKind, today: NaiveDate, config: &SchedulerConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner_id,
            kind,
            state: CardState::New,
            ease_factor: config.initial_ease_factor,
            interval_days: 0,
            due: today,
            lapses: 0,
            reviews: 0,
        }
    }
}

/// Outcome of a review operation, capturing the updated card and the diff applied.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewOutcome {
    /// Updated card state after applying the review.
    pub card: Card,
    /// Previous due date prior to the review.
    pub previous_due: NaiveDate,
    /// Grade assigned during the review.
    pub grade: ReviewGrade,
}

/// Errors that may occur while scheduling.
#[derive(Debug, Error)]
pub enum SchedulerError {
    /// Attempted to review a card that does not exist.
    #[error("card not found: {0}")]
    CardNotFound(Uuid),
}

/// Trait describing persistence needs for the scheduler.
pub trait CardStore {
    /// Fetch a card by identifier.
    fn get_card(&self, id: Uuid) -> Option<Card>;
    /// Insert or update a card.
    fn upsert_card(&mut self, card: Card);
    /// Retrieve cards that are due on or before the provided day.
    fn due_cards(&self, owner_id: Uuid, today: NaiveDate) -> Vec<Card>;
    /// Fetch unlock candidates (typically brand new opening moves).
    fn unlock_candidates(&self, owner_id: Uuid) -> Vec<Card>;
    /// Record that a card was unlocked today.
    fn record_unlock(&mut self, record: UnlockRecord);
    /// Fetch unlock log entries for a particular day.
    fn unlocked_on(&self, owner_id: Uuid, day: NaiveDate) -> Vec<UnlockRecord>;
}

/// In-memory card store useful for tests and examples.
#[derive(Debug, Default)]
pub struct InMemoryStore {
    cards: BTreeMap<Uuid, Card>,
    unlock_log: Vec<UnlockRecord>,
}

impl InMemoryStore {
    /// Create an empty in-memory store.
    pub fn new() -> Self {
        Self {
            cards: BTreeMap::new(),
            unlock_log: Vec::new(),
        }
    }
}

impl CardStore for InMemoryStore {
    fn get_card(&self, id: Uuid) -> Option<Card> {
        self.cards.get(&id).cloned()
    }

    fn upsert_card(&mut self, card: Card) {
        self.cards.insert(card.id, card);
    }

    fn due_cards(&self, owner_id: Uuid, today: NaiveDate) -> Vec<Card> {
        let mut due: Vec<Card> = self
            .cards
            .values()
            .filter(|card| {
                card.owner_id == owner_id
                    && card.due <= today
                    && !matches!(card.state, CardState::New)
            })
            .cloned()
            .collect();
        due.sort_by(|a, b| (a.due, a.id).cmp(&(b.due, b.id)));
        due
    }

    fn unlock_candidates(&self, owner_id: Uuid) -> Vec<Card> {
        let mut candidates: Vec<Card> = self
            .cards
            .values()
            .filter(|card| card.owner_id == owner_id && matches!(card.state, CardState::New))
            .cloned()
            .collect();
        candidates.sort_by(|a, b| match (&a.kind, &b.kind) {
            (
                CardKind::Opening {
                    parent_prefix: a_prefix,
                },
                CardKind::Opening {
                    parent_prefix: b_prefix,
                },
            ) => (a_prefix, &a.id).cmp(&(b_prefix, &b.id)),
            (CardKind::Opening { .. }, _) => std::cmp::Ordering::Less,
            (_, CardKind::Opening { .. }) => std::cmp::Ordering::Greater,
            _ => a.id.cmp(&b.id),
        });
        candidates
    }

    fn record_unlock(&mut self, record: UnlockRecord) {
        self.unlock_log.push(record);
    }

    fn unlocked_on(&self, owner_id: Uuid, day: NaiveDate) -> Vec<UnlockRecord> {
        self.unlock_log
            .iter()
            .filter(|record| record.owner_id == owner_id && record.day == day)
            .cloned()
            .collect()
    }
}

/// Scheduler applying SM-2 updates and unlock policy.
#[derive(Debug)]
pub struct Scheduler<S: CardStore> {
    store: S,
    config: SchedulerConfig,
}

impl<S: CardStore> Scheduler<S> {
    /// Create a scheduler backed by the provided store and configuration.
    pub fn new(store: S, config: SchedulerConfig) -> Self {
        Self { store, config }
    }

    /// Apply a review for the specified card identifier.
    pub fn review(
        &mut self,
        card_id: Uuid,
        grade: ReviewGrade,
        today: NaiveDate,
    ) -> Result<ReviewOutcome, SchedulerError> {
        let mut card = self
            .store
            .get_card(card_id)
            .ok_or(SchedulerError::CardNotFound(card_id))?;
        let previous_due = card.due;
        apply_sm2(&mut card, grade, &self.config, today);
        self.store.upsert_card(card.clone());
        Ok(ReviewOutcome {
            card,
            previous_due,
            grade,
        })
    }

    /// Build a queue of cards due for the given day, including unlocks.
    pub fn build_queue(&mut self, owner_id: Uuid, today: NaiveDate) -> Vec<Card> {
        build_queue_for_day(&mut self.store, &self.config, owner_id, today)
    }

    /// Consume the scheduler and return the underlying store.
    pub fn into_store(self) -> S {
        self.store
    }
}

/// Build a queue for the provided day by combining due cards and new unlocks.
pub fn build_queue_for_day<S: CardStore>(
    store: &mut S,
    config: &SchedulerConfig,
    owner_id: Uuid,
    today: NaiveDate,
) -> Vec<Card> {
    let mut queue = store.due_cards(owner_id, today);
    let prior_unlocks = store.unlocked_on(owner_id, today);
    let mut unlocked_prefixes: BTreeSet<String> = prior_unlocks
        .iter()
        .filter_map(|record| record.parent_prefix.clone())
        .collect();
    let mut unlocked_ids: BTreeSet<Uuid> =
        prior_unlocks.iter().map(|record| record.card_id).collect();

    for mut candidate in store.unlock_candidates(owner_id) {
        if unlocked_ids.contains(&candidate.id) {
            continue;
        }
        let parent_prefix = match &candidate.kind {
            CardKind::Opening { parent_prefix } => parent_prefix.clone(),
            _ => continue,
        };
        if unlocked_prefixes.contains(&parent_prefix) {
            continue;
        }

        candidate.state = CardState::Learning;
        candidate.interval_days = 0;
        candidate.due = today;
        candidate.ease_factor = config.initial_ease_factor;
        let record = UnlockRecord {
            card_id: candidate.id,
            owner_id,
            parent_prefix: Some(parent_prefix.clone()),
            day: today,
        };
        store.record_unlock(record);
        unlocked_prefixes.insert(parent_prefix);
        unlocked_ids.insert(candidate.id);
        store.upsert_card(candidate.clone());
        queue.push(candidate);
    }

    queue.sort_by(|a, b| (a.due, a.id).cmp(&(b.due, b.id)));
    queue
}

fn apply_sm2(card: &mut Card, grade: ReviewGrade, config: &SchedulerConfig, today: NaiveDate) {
    let previous_reviews = card.reviews;
    let previous_interval = card.interval_days.max(1);
    let ease = update_ease(card.ease_factor, grade, config);

    // State transitions: only Again keeps/moves to Relearning; all other grades
    // graduate to Review state (including cards currently in Relearning).
    let interval = match grade {
        ReviewGrade::Again => {
            card.lapses = card.lapses.saturating_add(1);
            card.state = CardState::Relearning;
            1
        }
        ReviewGrade::Hard => {
            card.state = CardState::Review;
            if previous_reviews == 0 {
                1
            } else if previous_reviews == 1 {
                4
            } else {
                (((previous_interval as f32) * 1.2).round() as u32).max(1)
            }
        }
        ReviewGrade::Good => {
            card.state = CardState::Review;
            if previous_reviews == 0 {
                1
            } else if previous_reviews == 1 {
                6
            } else {
                (((previous_interval as f32) * ease).round() as u32).max(1)
            }
        }
        ReviewGrade::Easy => {
            card.state = CardState::Review;
            if previous_reviews == 0 {
                1
            } else if previous_reviews == 1 {
                6
            } else {
                (((previous_interval as f32) * (ease * 1.3)).round() as u32).max(1)
            }
        }
    };

    let due = today
        .checked_add_signed(Duration::days(i64::from(interval)))
        .unwrap_or(today);
    card.due = due;
    card.interval_days = interval;
    card.ease_factor = ease;
    card.reviews = card.reviews.saturating_add(1);
}

fn update_ease(current: f32, grade: ReviewGrade, config: &SchedulerConfig) -> f32 {
    let quality = match grade {
        ReviewGrade::Again => 0.0,
        ReviewGrade::Hard => 3.0,
        ReviewGrade::Good => 4.0,
        ReviewGrade::Easy => 5.0,
    };
    let delta = 0.1 - (5.0 - quality) * (0.08 + (5.0 - quality) * 0.02);
    let next = current + delta;
    next.clamp(config.ease_minimum, config.ease_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_ease_clamps_values() {
        let config = SchedulerConfig {
            initial_ease_factor: 2.0,
            ease_minimum: 1.4,
            ease_maximum: 2.3,
            learning_steps_minutes: vec![],
        };
        assert!((update_ease(2.5, ReviewGrade::Hard, &config) - 2.3).abs() < f32::EPSILON);
        assert!((update_ease(1.0, ReviewGrade::Again, &config) - 1.4).abs() < f32::EPSILON);
    }
}
