#[test]
fn fleet_recruits_one_deck_carrier_after_two_kites_are_ready() {
    let world = Value::parse(r#"{"waters":[{"id":0,"p":0,"coast":[0],"adj":[]}]}"#);
    let me = Value::parse(r#"{"id":1,"seat":4,"capital":0,"bank":1000,"costs":{"unit":1},
        "techs":{"seaTraversal":true,"flight":true},"units":{"cap":20,"total":10,"queued":0,"spawned":0,
        "byType":{"kite":2,"carrier":0,"picket":0,"warjunk":0,"corsair":0,"freighter":0}},
        "build":{"max":{"hp":5,"speed":5,"strength":5,"carry":2,"vision":5},"price":{},"points":100},
        "rts":{"sea":{"ports":[0],"contacts":[]},"cpu":{"income":60000,"available":{"fabrication":100000}},"licenses":["hull"]}}"#);
    let args = views(&[world, me]);
    let naval = super::Naval { stations: 0, landing: -1.0, landing_name: String::new(), fleet: -1.0, blockade: 0, raid: 0 };
    let mut orders = Value::obj();
    assert!(super::naval_spawn(&mut orders, args[0], args[1], &naval, 0.0, false), "carrier conditions should recruit a deck");
    assert_eq!(orders.get("spawn").len(), 1, "{orders:?}");
    let carrier = orders.get("spawn").at(0);
    assert_eq!(carrier.get("type").str(), "carrier");
    assert_eq!(carrier.get("build").get("carry").num(), 2.0, "the Fleet carrier must carry both kites");
    assert!(carrier.get("modules").items().iter().any(|m| m.str() == "hull"));
}

#[test]
fn fleet_does_not_recruit_a_carrier_before_a_two_kite_flight_exists() {
    let world = Value::parse(r#"{"waters":[{"id":0,"p":0,"coast":[0],"adj":[]}]}"#);
    let me = Value::parse(r#"{"id":1,"seat":4,"capital":0,"bank":1000,"costs":{"unit":1},
        "techs":{"seaTraversal":true,"flight":true},"units":{"cap":20,"total":10,"queued":0,"spawned":0,
        "byType":{"kite":1,"carrier":0,"picket":0,"warjunk":0,"corsair":0,"freighter":0}},
        "build":{"max":{"hp":5,"speed":5,"strength":5,"carry":2,"vision":5},"price":{},"points":100},
        "rts":{"sea":{"ports":[0],"contacts":[]},"cpu":{"income":60000,"available":{"fabrication":100000}},"licenses":["hull"]}}"#);
    let args = views(&[world, me]);
    let naval = super::Naval { stations: 0, landing: -1.0, landing_name: String::new(), fleet: -1.0, blockade: 0, raid: 0 };
    let mut orders = Value::obj();
    assert!(!super::naval_spawn(&mut orders, args[0], args[1], &naval, 0.0, false));
    assert!(orders.get("spawn").is_empty(), "{orders:?}");
}

#[test]
fn fleet_kite_callback_routes_a_hot_wing_to_an_adjacent_carrier_deck() {
    let units = Value::parse(r#"[{"id":101,"owner":1,"type":"kite","air":true,"at":5,
        "build":{"vision":0,"range":0,"wings":1},"rts":{"air":{"heat":30,"aloft":true},"sea":{}}}]"#);
    let world = Value::parse(r#"{"seconds":1,"provinces":[{"id":5,"owner":1,"adj":[6]},{"id":6,"owner":1,"adj":[5]}],"units":[
        {"id":501,"owner":1,"type":"carrier","at":6,"hp":5,"rts":{"sea":{"ship":true,"deck":{"room":2,"riders":[]}}}}]}"#);
    let me = Value::parse(r#"{"id":1,"units":{"byType":{"carrier":1}}}"#);
    let args = flight_views(&[units, world, me]);
    super::kite(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("id").num(), 101.0);
    assert_eq!(orders[0].get("board").num(), 501.0);
}

#[test]
fn fleet_kite_callback_keeps_a_paid_carrier_return_course() {
    let units = Value::parse(r#"[{"id":101,"owner":1,"type":"kite","air":true,"at":5,
        "build":{"vision":0,"range":0,"wings":1},"rts":{"air":{"heat":30,"aloft":true},"sea":{"course":{"board":501}}}}]"#);
    let world = Value::parse(r#"{"seconds":1,"units":[
        {"id":501,"owner":1,"type":"carrier","at":5,"hp":5,"rts":{"sea":{"ship":true,"deck":{"room":2,"riders":[]}}}}]}"#);
    let me = Value::parse(r#"{"id":1,"units":{"byType":{"carrier":1}}}"#);
    let args = flight_views(&[units, world, me]);
    super::kite(args[0], args[1], args[2]);
    assert!(emitted().is_empty(), "the kite must not replace an already-paid boarding route");
}

#[test]
fn fleet_low_think_allowance_recovers_and_funded_retry_can_plan() {
    let world = Value::parse(r#"{"seconds":6,"provinces":[{}, {}, {}, {}, {}],"units":[{}, {}, {}, {}, {}, {}, {}, {}, {}, {}]}"#);
    let me = Value::parse(r#"{"id":1,"compute":{"thinkPeak":22000},"units":{"total":10},
        "rts":{"cpu":{"income":50000,"bank":150000,"spendCap":60000,
        "available":{"think":9000}}}}"#);
    let args = views(&[world, me]);
    let need = super::full_think_need(args[0], args[1]);

    assert!(!super::plan_fuel_ready(9000.0, 15000.0, need), "the low allowance must skip the full planner");
    let fallback = super::recovery_for_plan(args[0], args[1], need);
    assert_eq!(fallback.get("memo").str(), "Recovering production");
    let next_limit = fallback.get("compute").get("cpu").get("limits").get("think").num();
    assert!(next_limit >= need + super::PLAN_REMAINING_MARGIN + super::PLAN_CALLBACK_OVERHEAD,
        "healthy recovery should budget the planner and its callback overhead: {fallback:?}");
    assert!(next_limit <= 60000.0, "the retry permission must stay under the spend cap");
    let remaining_after_callback = next_limit - super::PLAN_CALLBACK_OVERHEAD;
    assert!(super::plan_fuel_ready(next_limit, remaining_after_callback, need),
        "the saved recovery policy must pass admission after callback overhead");
}

#[test]
fn fleet_uses_lean_plan_when_the_full_estimate_exceeds_the_spend_cap() {
    let mut world = Value::parse(r#"{"seconds":6,"provinces":[],"units":[]}"#);
    let mut provinces = Value::arr();
    let mut units = Value::arr();
    for _ in 0..44 { provinces.push(obj! {}); }
    for _ in 0..180 { units.push(obj! {}); }
    world.set("provinces", provinces);
    world.set("units", units);
    let me = Value::parse(r#"{"id":1,"compute":{"thinkPeak":22000},"units":{"total":26},
        "rts":{"cpu":{"income":37900,"bank":150000,"spendCap":47400,
        "available":{"think":9000}}}}"#);
    let args = views(&[world, me]);
    let full_need = super::full_think_need(args[0], args[1]);
    let need = super::budgeted_think_need(args[0], args[1], 47400.0);
    assert!(need < full_need, "a capped house should receive the guarded lean estimate");
    assert!(need <= 14000.0, "small rich factions need an opening pass sized to their measured workload");
    let fallback = super::recovery_for_plan(args[0], args[1], need);
    let next_limit = fallback.get("compute").get("cpu").get("limits").get("think").num();

    assert!(next_limit >= need + super::PLAN_REMAINING_MARGIN + super::PLAN_CALLBACK_OVERHEAD,
        "the lean pass should fit inside the healthy 47.4k spend cap: {fallback:?}");
    assert!(next_limit <= 47400.0, "the retry permission must respect the spend cap");
    assert!(next_limit < 20000.0, "the saved opening policy should fit the observed 20k-class budget");
    assert!(super::plan_fuel_ready(next_limit, next_limit - super::PLAN_CALLBACK_OVERHEAD, need),
        "the saved lean policy should admit after callback overhead");

    let low_cap_me = Value::parse(r#"{"id":1,"compute":{"thinkPeak":22000},"units":{"total":26},
        "rts":{"cpu":{"income":37900,"bank":150000,"spendCap":14000,
        "available":{"think":9000}}}}"#);
    let low_cap_args = views(&[args[0].value(), low_cap_me]);
    let low_cap_need = super::budgeted_think_need(low_cap_args[0], low_cap_args[1], 14000.0);
    let cheap = super::recovery_for_plan(low_cap_args[0], low_cap_args[1], low_cap_need);
    let cheap_limit = cheap.get("compute").get("cpu").get("limits").get("think").num();
    assert!(cheap_limit <= 14000.0, "recovery must respect the lower spend cap: {cheap:?}");
    assert!(!super::plan_fuel_ready(cheap_limit, cheap_limit - super::PLAN_CALLBACK_OVERHEAD, low_cap_need),
        "when even the lean pass does not fit, stay on cheap recovery");
}
