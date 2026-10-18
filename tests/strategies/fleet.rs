#[test]
fn retirement_postamble_admits_the_cold_plan_only_for_a_funded_existing_corsair() {
    let world=obj! {"logistics"=>obj! {}};
    let mut me=obj! {"units"=>obj! {"byType"=>obj! {"corsair"=>0}},
        "ramCapital"=>obj! {"0"=>obj! {"bytes"=>Value::Bytes(vec![0,4]),"t"=>1}}};
    let args=views(&[world.clone(),me.clone()]);
    assert!(!super::retirement_ready(args[0],args[1],50000.0),"a faction with no corsairs does not decode its naval plan after recovery");
    me.entry("units").entry("byType").set("corsair",1);
    let args=views(&[world.clone(),me.clone()]);
    assert!(!super::retirement_ready(args[0],args[1],5000.0),"the cold 1024-byte plan needs its own decoding runway");
    assert!(super::retirement_ready(args[0],args[1],10000.0),"funded cleanup remains available");
    let args=views(&[obj! {},me]);
    assert!(!super::retirement_ready(args[0],args[1],50000.0),"ordinary worlds have no cargo retirement task");
}

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
fn fleet_starts_a_funded_two_kite_wing_before_finishing_its_mining_quota() {
    let me = Value::parse(r#"{"id":1,"seat":4,"bank":5000,"costs":{"unit":1},
        "techs":{"flight":true},"units":{"cap":40,"total":30,"queued":0,"spawned":0,
        "byType":{"miner":4,"kite":0,"carrier":0}},
        "rts":{"bank":30000,"costs":{"fabrication":{"base":1200,"module":600}},
        "cpu":{"available":{"fabrication":30000}}}}"#);
    let me = views(&[me]);
    let tactical_bank = me[0].path("rts.bank").num();
    let funded = tactical_bank > 2450.0 * super::common::price_level(me[0]);
    let mut orders = Value::obj();

    assert!(super::starter_kite(&mut orders, me[0], 4.0, 12.0, 0.0, funded), "four miners establish the minimum economy even with eight still missing");
    assert_eq!(orders.get("spawn").at(0).get("type").str(), "kite", "the opening wing is the selected production choice");
    assert!(orders.get("spawn").at(0).get("build").get("wings").truthy(), "the airframe comes from the production cost helper");

    let mut orders = Value::obj();
    assert!(!super::starter_kite(&mut orders, me[0], 4.0, 12.0, 0.0, false), "no tactical reserve means no airframe");
    assert!(orders.get("spawn").is_undef() || orders.get("spawn").is_empty());
}

#[test]
fn fleet_keeps_its_opening_air_wing_small_until_a_carrier_is_fielded() {
    let me = Value::parse(r#"{"id":1,"seat":4,"bank":5000,"costs":{"unit":1},
        "techs":{"flight":true},"units":{"cap":40,"total":30,"queued":0,"spawned":0,
        "byType":{"miner":4,"kite":2,"carrier":0}},
        "rts":{"bank":30000,"costs":{"fabrication":{"base":1200,"module":600}},
        "cpu":{"available":{"fabrication":30000}}}}"#);
    let me = views(&[me]);
    let mut orders = Value::obj();
    assert!(!super::starter_kite(&mut orders, me[0], 4.0, 12.0, 0.0, true));
    assert!(orders.get("spawn").is_undef() || orders.get("spawn").is_empty(), "two kites wait for the carrier order before expanding");
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

#[test]
fn fleet_corsair_spawn_rejects_the_same_unprofitable_or_escorted_prey_as_quarry() {
    let me = Value::parse(r#"{"rts":{"logistics":{"rules":{"shipHoldPerCarry":160000}},"sea":{"contacts":[]}}}"#);
    let prey = Value::parse(r#"{"id":9,"ship":true,"water":1,"build":{"range":0,"carry":2},"cargo":{"load":2}}"#);
    let empty=Value::arr();
    let args=views(&[me.clone(),prey,empty]);
    assert!(!super::paying_corsair_prey(args[1], args[2], args[0], 160000.0, true, 0.0), "a half-load cannot repay the existing corsair trip gate");
    let prey = Value::parse(r#"{"id":9,"ship":true,"water":1,"build":{"range":0,"carry":3},"cargo":{"load":3}}"#);
    let contacts=Value::parse(r#"[{"id":9,"ship":true,"water":1,"build":{"range":0,"carry":3},"cargo":{"load":3}},{"id":10,"ship":true,"water":1,"build":{"range":1}}]"#);
    let args=views(&[me,prey,contacts]);
    assert!(!super::paying_corsair_prey(args[1],args[2],args[0],160000.0,true,0.0),"an armed escort rejects the spawn just as it rejects a chase");
}

#[test]
fn fleet_corsair_spawn_keeps_a_profitable_unescorted_prey_eligible() {
    let me=Value::parse(r#"{"rts":{"logistics":{"rules":{"shipHoldPerCarry":160000}}}}"#);
    let prey=Value::parse(r#"{"id":9,"ship":true,"water":1,"build":{"range":0,"carry":3},"cargo":{"load":3}}"#);
    let empty=Value::arr();
    let args=views(&[me,prey,empty]);
    assert!(super::paying_corsair_prey(args[1],args[2],args[0],300000.0,true,0.0));
}

fn storage_research_pick(powered: f64, capacity: f64) -> String {
    let provinces = arr![obj! { "id"=>0,"owner"=>1,"power"=>obj! {"sunFactor"=>1,"lat"=>0,"windFactor"=>1} }];
    let mut world_techs = obj! {};
    for t in super::TECHS { world_techs.set(t, obj! {"offer"=>1000,"requires"=>arr![]}); }
    let world = obj! {"provinces"=>provinces,"techs"=>world_techs};
    let mut owned = obj! {};
    for t in ["extraction","seaTraversal","turbines","ranged","armor","dash","cloak","scan","influence","faith","pierce","artillery","antiAir","repair","flight"] {
        owned.set(t, true);
    }
    let plants = if capacity > 0.0 { arr![obj! {"state"=>"ready","p"=>0,"ports"=>arr![obj! {"kind"=>"tidal","capacity"=>capacity}]}] } else { Value::arr() };
    let me = obj! {
        "id"=>1,"capital"=>0,"techs"=>owned,
        "energy"=>obj! {"powered"=>powered,"house"=>3,"plants"=>plants,
            "islands"=>arr![obj! {"capital"=>true,"provinces"=>arr![0],"demand"=>8,"powered"=>powered,"storage"=>0}]},
        "compute"=>obj! {"cycles"=>100000,"thinkPeak"=>1000,"used"=>obj!{}},
        "units"=>obj! {"total"=>0,"queued"=>0,"cap"=>100},"costs"=>obj! {"unit"=>100,"upkeep"=>1},"bank"=>0
    };
    let args = views(&[world,me]);
    let techs = super::research_techs(args[0],args[1]);
    let mut orders = obj! {};
    super::common::budget(&mut orders,args[0],args[1],&super::common::Budget {research:0.12,code:0.10,techs,types:None});
    orders.get("research").str().to_string()
}

#[test]
fn fleet_prioritizes_storage_only_for_a_dim_grid_with_chargeable_variable_surplus() {
    assert_eq!(storage_research_pick(0.8,10.0),"storage");
    assert_eq!(storage_research_pick(0.96,10.0),"airWeapons","a stable grid keeps the air path ahead of storage");
    assert_eq!(storage_research_pick(0.8,2.0),"airWeapons","a low powered share without generation headroom is not a battery case");
}
