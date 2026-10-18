#[test]
fn classic_reflex_reserves_a_measured_contact_turn_before_the_next_unit() {
    let first = super::classic_reflex_runway(120.0, 1);
    assert_eq!(first, 330.0, "the original single-contact estimate fits the 400-cycle callback budget");
    assert!(first <= 400.0, "the initial gate leaves the classic single-unit callback productive");
    assert_eq!(super::classic_reflex_runway(120.0, 4), 480.0,
        "additional visible contacts raise the first-turn runway");
    let learned = super::next_reflex_runway(120.0, 500.0, false);
    assert_eq!(learned, 1000.0, "classic callbacks leave twice the last unit's measured work");
    assert_eq!(super::next_reflex_runway(1800.0, 500.0, true), 1800.0,
        "the established CPU RTS runway stays unchanged");
}
