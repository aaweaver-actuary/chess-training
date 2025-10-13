use review_domain::ids::{CardId, EdgeId, LearnerId, MoveId, PositionId, UnlockId};

#[test]
fn id_round_trips_preserve_raw_value() {
    let raw = 42_u64;
    let position = PositionId::new(raw);
    assert_eq!(position.get(), raw);
    assert_eq!(u64::from(position), raw);

    let edge = EdgeId::from(raw + 1);
    assert_eq!(edge.get(), raw + 1);
    assert_eq!(u64::from(edge), raw + 1);

    let mv = MoveId::from(raw + 2);
    assert_eq!(mv.get(), raw + 2);
    assert_eq!(u64::from(mv), raw + 2);

    let card = CardId::from(raw + 3);
    assert_eq!(card.get(), raw + 3);
    assert_eq!(u64::from(card), raw + 3);

    let learner = LearnerId::from(raw + 4);
    assert_eq!(learner.get(), raw + 4);
    assert_eq!(u64::from(learner), raw + 4);

    let unlock = UnlockId::from(raw + 5);
    assert_eq!(unlock.get(), raw + 5);
    assert_eq!(u64::from(unlock), raw + 5);
}

#[test]
fn ids_have_human_readable_display() {
    assert_eq!(PositionId::new(7).to_string(), "PositionId(7)");
    assert_eq!(EdgeId::from(8).to_string(), "EdgeId(8)");
    assert_eq!(MoveId::from(9).to_string(), "MoveId(9)");
    assert_eq!(CardId::from(10).to_string(), "CardId(10)");
    assert_eq!(LearnerId::from(11).to_string(), "LearnerId(11)");
    assert_eq!(UnlockId::from(12).to_string(), "UnlockId(12)");
}

#[test]
fn ids_are_copy_and_eq() {
    fn assert_copy_eq<T: Copy + Eq>() {}

    assert_copy_eq::<PositionId>();
    assert_copy_eq::<EdgeId>();
    assert_copy_eq::<MoveId>();
    assert_copy_eq::<CardId>();
    assert_copy_eq::<LearnerId>();
    assert_copy_eq::<UnlockId>();
}
