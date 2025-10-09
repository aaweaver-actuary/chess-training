use chrono::NaiveDate;
use scheduler_core::{
    Card, CardKind, CardState, CardStore, ReviewGrade, Scheduler, SchedulerConfig, UnlockRecord,
    build_queue_for_day,
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("invalid date")
}

#[derive(Debug, Clone)]
struct TimedStore {
    cards: BTreeMap<Uuid, Card>,
    unlock_log: Vec<UnlockRecord>,
    availability: BTreeMap<Uuid, NaiveDate>,
    current_day: NaiveDate,
}

impl TimedStore {
    fn new(current_day: NaiveDate) -> Self {
        Self {
            cards: BTreeMap::new(),
            unlock_log: Vec::new(),
            availability: BTreeMap::new(),
            current_day,
        }
    }

    fn insert_with_availability(&mut self, card: Card, available_from: NaiveDate) {
        self.availability.insert(card.id, available_from);
        self.cards.insert(card.id, card);
    }

    fn set_day(&mut self, day: NaiveDate) {
        self.current_day = day;
    }
}

impl CardStore for TimedStore {
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
            .filter(|card| {
                self.availability
                    .get(&card.id)
                    .map(|available| *available <= self.current_day)
                    .unwrap_or(true)
            })
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

#[test]
fn shared_first_move_is_unlocked_only_once() {
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let day1 = date(2024, 1, 1);
    let mut store = TimedStore::new(day1);

    let first_line = Card::new(
        owner,
        CardKind::Opening {
            parent_prefix: "start".to_string(),
        },
        day1,
        &config,
    );
    let alternate_line = Card::new(
        owner,
        CardKind::Opening {
            parent_prefix: "start".to_string(),
        },
        day1,
        &config,
    );

    store.insert_with_availability(first_line.clone(), day1);
    store.insert_with_availability(alternate_line.clone(), day1);

    let queue = build_queue_for_day(&mut store, &config, owner, day1);

    assert_eq!(queue.len(), 1, "only one opening should unlock per prefix");
    let unlocked = queue.first().expect("queue should contain a card");
    assert_eq!(unlocked.id, first_line.id.min(alternate_line.id));
    assert_eq!(unlocked.state, CardState::Learning);

    let remaining = store
        .get_card(first_line.id.max(alternate_line.id))
        .expect("second card should remain in store");
    assert_eq!(remaining.state, CardState::New);
}

#[test]
fn responses_unlock_after_first_move_review() {
    let config = SchedulerConfig::default();
    let owner = Uuid::new_v4();
    let day1 = date(2024, 1, 1);
    let day2 = date(2024, 1, 2);
    let mut store = TimedStore::new(day1);

    let first_move = Card::new(
        owner,
        CardKind::Opening {
            parent_prefix: "start".to_string(),
        },
        day1,
        &config,
    );
    let first_move_id = first_move.id;

    let scandinavian = Card::new(
        owner,
        CardKind::Opening {
            parent_prefix: "start e4 vs d5".to_string(),
        },
        day1,
        &config,
    );
    let open_game = Card::new(
        owner,
        CardKind::Opening {
            parent_prefix: "start e4 vs e5".to_string(),
        },
        day1,
        &config,
    );

    store.insert_with_availability(first_move.clone(), day1);
    store.insert_with_availability(scandinavian.clone(), day2);
    store.insert_with_availability(open_game.clone(), day2);

    let day_one_queue = build_queue_for_day(&mut store, &config, owner, day1);
    assert_eq!(day_one_queue.len(), 1);
    assert_eq!(day_one_queue[0].id, first_move_id);
    assert_eq!(day_one_queue[0].state, CardState::Learning);

    let mut scheduler = Scheduler::new(store, config.clone());
    scheduler
        .review(first_move_id, ReviewGrade::Good, day1)
        .expect("review should succeed");
    let mut store = scheduler.into_store();

    store.set_day(day2);
    let day_two_queue = build_queue_for_day(&mut store, &config, owner, day2);

    assert_eq!(day_two_queue.len(), 3, "review plus two new responses");
    let due_ids: BTreeSet<Uuid> = day_two_queue.iter().map(|card| card.id).collect();
    assert!(due_ids.contains(&first_move_id), "first move should be due");
    assert!(
        due_ids.contains(&scandinavian.id),
        "Scandinavian Defense response unlocks"
    );
    assert!(
        due_ids.contains(&open_game.id),
        "Open Game response unlocks"
    );
}
