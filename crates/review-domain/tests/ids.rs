use review_domain::ids::{
    CardId, EdgeId, IdConversionError, LearnerId, MoveId, PositionId, TacticId, UnlockId,
};

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

    let tactic = TacticId::from(raw + 5);
    assert_eq!(tactic.get(), raw + 5);
    assert_eq!(u64::from(tactic), raw + 5);

    let unlock = UnlockId::from(raw + 6);
    assert_eq!(unlock.get(), raw + 6);
    assert_eq!(u64::from(unlock), raw + 6);
}

#[test]
fn ids_have_human_readable_display() {
    assert_eq!(PositionId::new(7).to_string(), "PositionId(7)");
    assert_eq!(EdgeId::from(8).to_string(), "EdgeId(8)");
    assert_eq!(MoveId::from(9).to_string(), "MoveId(9)");
    assert_eq!(CardId::from(10).to_string(), "CardId(10)");
    assert_eq!(LearnerId::from(11).to_string(), "LearnerId(11)");
    assert_eq!(TacticId::from(12).to_string(), "TacticId(12)");
    assert_eq!(UnlockId::from(13).to_string(), "UnlockId(13)");
}

#[test]
fn ids_are_copy_and_eq() {
    fn assert_copy_eq<T: Copy + Eq>() {}

    assert_copy_eq::<PositionId>();
    assert_copy_eq::<EdgeId>();
    assert_copy_eq::<MoveId>();
    assert_copy_eq::<CardId>();
    assert_copy_eq::<LearnerId>();
    assert_copy_eq::<TacticId>();
    assert_copy_eq::<UnlockId>();
}

#[test]
fn ids_report_overflow_conversion_errors() {
    let overflow =
        LearnerId::try_from(u128::from(u64::MAX) + 1).expect_err("overflow should emit error");
    assert!(matches!(
        overflow,
        IdConversionError::Overflow { value } if value == u128::from(u64::MAX) + 1
    ));

    let tactic_overflow = TacticId::try_from(u128::from(u64::MAX) + 2)
        .expect_err("tactic overflow should emit error");
    assert!(matches!(
        tactic_overflow,
        IdConversionError::Overflow { value } if value == u128::from(u64::MAX) + 2
    ));
}
