#[test]
fn kublai_skips_control_orders_without_a_full_route_budget() {
    assert!(!super::control_has_runway(2381.0, 800.0, 0.0));
    assert!(super::control_has_runway(4200.0, 800.0, 0.0));
    assert!(!super::control_has_runway(5000.0, 2400.0, 900.0));
}
