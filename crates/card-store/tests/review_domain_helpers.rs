use review_domain::CardKind;

#[test]
fn card_kind_map_helpers_cover_all_variants() {
    let opening_variant: CardKind<&str, &str> = CardKind::Opening("ruy lopez");
    let mapped_opening = opening_variant.map_opening(str::to_uppercase);
    assert!(matches!(mapped_opening, CardKind::Opening(ref value) if value == "RUY LOPEZ"));

    let tactic_variant: CardKind<&str, &str> = CardKind::Tactic("fork");
    let mapped_tactic = tactic_variant.map_tactic(str::to_uppercase);
    assert!(matches!(mapped_tactic, CardKind::Tactic(ref value) if value == "FORK"));

    let untouched_tactic = CardKind::Tactic("pin").map_opening(str::to_uppercase);
    assert!(matches!(untouched_tactic, CardKind::Tactic("pin")));

    let untouched_opening = CardKind::Opening("sicilian").map_tactic(str::to_uppercase);
    assert!(matches!(untouched_opening, CardKind::Opening("sicilian")));

    let tactic_payload = String::from("skewer");
    match CardKind::<(), String>::Tactic(tactic_payload.clone()).as_ref() {
        CardKind::Tactic(reference) => assert_eq!(*reference, "skewer"),
        CardKind::Opening(()) => panic!("expected tactic reference"),
    }

    let opening_payload = String::from("london");
    match CardKind::<String, ()>::Opening(opening_payload.clone()).as_ref() {
        CardKind::Opening(reference) => assert_eq!(*reference, "london"),
        CardKind::Tactic(()) => panic!("expected opening reference"),
    }
}
