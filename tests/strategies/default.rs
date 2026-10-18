#[test]
fn classic_reflex_reserves_a_measured_contact_turn_before_the_next_unit() {
    let first = super::classic_reflex_runway(120.0, 4);
    assert_eq!(first, 730.0, "contact count sets a conservative first-turn runway");
    let learned = super::next_reflex_runway(120.0, 500.0, false);
    assert_eq!(learned, 1000.0, "classic callbacks leave twice the last unit's measured work");
    assert_eq!(super::next_reflex_runway(1800.0, 500.0, true), 1800.0,
        "the established CPU RTS runway stays unchanged");
}
