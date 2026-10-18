#[test]
fn kublai_skips_control_orders_without_a_full_route_budget() {
    assert!(!super::control_has_runway(2381.0, 800.0, 0.0));
    assert!(super::control_has_runway(4200.0, 800.0, 0.0));
    assert!(!super::control_has_runway(5000.0, 2400.0, 900.0));
}

#[test]
fn kublai_gun_callback_leaves_a_route_for_a_funded_control_turn() {
    assert!(!super::gun_has_route_runway(2243.0), "the captured failing allowance keeps its standing gun order");
    assert!(!super::gun_has_route_runway(3500.0), "a route with little return room waits for the next turn");
    assert!(super::gun_has_route_runway(4000.0), "the measured 4k-class callback still controls artillery");
    assert!(super::gun_has_route_runway(6000.0), "funded artillery callbacks remain admitted");
}

#[test]
fn deployed_gun_at_its_target_keeps_its_standing_combat_posture() {
    // `fixtures/kublai-gun-deployed.json` is an authored public callback view
    // with the plan pages its unit would carry away from home.
    let fixture = Value::parse(include_str!("../fixtures/kublai-gun-deployed.json")).items().to_vec();
    let args = views(&fixture);
    super::gun(args[0], args[1], args[2]);
    assert!(emitted().is_empty(), "a deployed gun already at its target keeps firing its standing order");
}

#[test]
fn at_target_gun_can_still_set_up_without_route_fuel() {
    let mut fixture = Value::parse(include_str!("../fixtures/kublai-gun-deployed.json")).items().to_vec();
    let mut gun = fixture[0].at(0).clone();
    gun.entry("rts").set("deployed", false);
    fixture[0] = arr![gun];
    let args = views(&fixture);
    super::gun(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "the target gun needs only its setup order: {orders:?}");
    assert!(orders[0].get("deploy").truthy());
}

#[test]
fn short_budget_phase_keeps_combat_calls_and_standing_economic_choices() {
    let args = views(&[obj! {}, obj! { "rts"=>obj! { "cpu"=>obj! { "income"=>33583 } } }]);
    let types = obj! { "screen"=>obj! { "share"=>6000, "every"=>1 }, "skirm"=>obj! { "share"=>6000, "every"=>1 } };
    let budget = super::common::Budget { types: Some(&types), ..super::common::Budget::default() };
    let mut orders = obj! { "research"=>"flight", "rent"=>3, "spawn"=>arr![obj! { "type"=>"screen" }] };
    super::budget_phase(&mut orders, args[0], args[1], &budget, &types, 3350.0);
    assert_eq!(orders.get("research").str(), "flight");
    assert_eq!(orders.get("rent").num(), 3.0);
    assert_eq!(orders.get("spawn").at(0).get("type").str(), "screen");
    assert_eq!(orders.get("compute").get("types").get("screen").get("share").num(), 6000.0);
    assert_eq!(orders.get("compute").get("types").get("skirm").get("share").num(), 6000.0);
}

#[test]
fn road_reuses_its_matching_recent_field_before_scanning_the_world() {
    let field = obj! { "to"=>3, "at"=>100, "next"=>obj! {} };
    let args = views(&[obj! { "seconds"=>105 }, obj! { "ramCapital"=>pages(&obj! { "field"=>field.clone() }) }]);
    let road = super::road(args[0], args[1], 3.0, 16000.0).expect("the matching field stands");
    assert_eq!(road.to_json(), field.to_json());
}

#[test]
fn short_plan_publication_keeps_standing_ram_and_recruitment() {
    let args = views(&[obj! {}, obj! { "rts"=>obj! { "cpu"=>obj! { "income"=>33583 } } }]);
    let standing = arr![obj! { "write"=>224, "bytes"=>arr![1,2], "scope"=>"local" }];
    let mut orders = obj! { "ram"=>standing.clone(), "spawn"=>arr![obj! { "type"=>"screen" }] };
    super::share_phase(&mut orders, &obj! { "attack"=>3 }, args[0], args[1], 6500.0);
    assert_eq!(orders.get("ram").to_json(), standing.to_json(), "the standing plan and unrelated RAM writes survive");
    assert_eq!(orders.get("spawn").at(0).get("type").str(), "screen");
}
