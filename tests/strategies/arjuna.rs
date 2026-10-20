fn flight_world() -> Value {
    Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true,"shielded":false},{"id":2,"alive":true,"shielded":false}],"provinces":[
        {"id":0,"owner":1,"adj":[1],"visible":true,"units":{"1":1}},
        {"id":1,"owner":2,"adj":[0],"visible":true,"units":{"2":2}}]}"#)
}

fn flight_me() -> Value {
    Value::parse(r#"{"id":1,"allies":[],"ramProvince":{},"rts":{"cpu":{"limits":{"fabrication":1200}},"air":{"contacts":[]}}}"#)
}

fn fighting_wing() -> Value {
    Value::parse(r#"[{"id":101,"owner":1,"type":"wing","air":true,"at":1,"to":-1,"target":1,"doing":"fight",
        "range":0,"build":{"wings":1,"range":0},"ramUnit":{},"rts":{"air":{"aloft":true,"heat":5,"glide":null}}}]"#)
}

#[test]
fn defended_sortie_keeps_fighting_with_province_counts_and_no_world_units() {
    let world = flight_world();
    assert!(world.get("units").is_undef(), "the real unit callback omits world.units");
    let args = views(&[fighting_wing(), world, flight_me()]);
    let (foes, defenders, prey) = super::sky::hostiles(args[1], args[2], 1.0);
    assert_eq!((foes, defenders), (2.0, 2.0));
    assert_eq!(prey, None, "province counts supply no target positions");
    super::wing(args[0], args[1], args[2]);
    let orders = emitted();
    assert!(orders.is_empty(), "the defended standing fight must remain active: {orders:?}");
}

#[test]
fn aircraft_abroad_uses_its_standing_target_to_intercept_a_visible_contact() {
    let mut me = flight_me();
    me.entry("rts").entry("air").set("contacts", arr![obj! {
        "id"=>401,"owner"=>2,"air"=>true,"at"=>1,"hp"=>3
    }]);
    let args = views(&[fighting_wing(), flight_world(), me]);
    super::wing(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("target").num(), 1.0);
    assert_eq!(orders[0].get("do").str(), "fight");
    assert_eq!(orders[0].get("focus").num(), 401.0);
}

#[test]
fn wing_full_path_runway_covers_a_four_aircraft_forty_five_province_call() {
    let mut units = Value::arr();
    for id in 0..4 { units.push(obj! { "id"=>id, "type"=>"wing" }); }
    let mut provinces = Value::arr();
    for id in 0..45 { provinces.push(obj! { "id"=>id }); }
    let args = views(&[units, obj! { "provinces"=>provinces }, obj! {}]);
    let need = super::wing_full_path_need(args[0], args[1]);
    assert_eq!(args[1].get("provinces").len(), 45);
    assert_eq!(args[0].len(), 4);
    assert_eq!(need, 7500.0);
    assert!(need > 7000.0, "rejects the metered failing range");
    assert!(need < 8000.0, "keeps the tested 8,000-cycle full path available");
}

#[test]
fn sky_routes_can_be_built_before_the_return_home_distances() {
    let args = views(&[flight_world(), flight_me()]);
    let (world, me) = (args[0], args[1]);
    let mut geo = super::sky::Geo::route_only(world);
    assert!(geo.route(0.0, 1.0).is_some());
    assert!(geo.home_d(0.0).is_infinite());
    geo.ensure_home(world, me, 12.0);
    assert_eq!(geo.home_d(0.0), 0.0);
    assert_eq!(geo.home_d(1.0), 1.0);
}

#[test]
fn fresh_home_launch_carries_the_sdk_core_plan_with_its_sky_sortie() {
    let mut me = flight_me();
    let plan = obj! {"attack"=>2,"sky"=>1,"stage"=>0,"defend"=>-1};
    me.entry("ramProvince").set("0", pages(&plan));
    let units = Value::parse(r#"[{"id":101,"owner":1,"type":"wing","air":true,"at":0,"to":-1,"target":-1,"doing":"idle",
        "range":0,"build":{"wings":1,"range":0},"ramUnit":{},"rts":{"air":{"aloft":false,"heat":0,"glide":null}}}]"#);
    let args = views(&[units, flight_world(), me]);
    super::wing(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("target").num(), 1.0, "the sky sortie can differ from the ground attack");
    assert!(!orders[0].get("ram").is_empty(), "a home launch must carry the SDK's core plan");
}

fn arjuna_research_choice(powered: f64) -> String {
    let mut techs = Value::obj();
    for t in ["extraction", "dash", "ranged", "antiAir", "turbines", "flight", "artillery", "airWeapons"] {
        techs.set(t, obj! { "offer" => 1000, "requires" => arr![] });
    }
    let world = obj! { "provinces" => arr![], "techs" => techs };
    let me = obj! {
        "id" => 1,
        "energy" => obj! { "powered" => powered },
        "techs" => obj! { "extraction"=>true,"dash"=>true,"ranged"=>true,"antiAir"=>true },
        "compute" => obj! { "cycles"=>100000,"thinkPeak"=>1000,"used"=>obj!{} },
        "units" => obj! { "total"=>0,"queued"=>0,"cap"=>100 },
        "costs" => obj! { "unit"=>100,"upkeep"=>1 }, "bank"=>0
    };
    let args = views(&[world, me]);
    let (world_view, me_view) = (args[0], args[1]);
    let list = super::doctrine(me_view, false);
    let mut orders = obj! {};
    super::common::budget(&mut orders, world_view, me_view, &super::common::Budget {
        research: 0.12, code: 0.72, techs: list, types: None,
    });
    orders.get("research").str().to_string()
}

#[test]
fn turbines_take_priority_only_during_a_real_power_shortfall() {
    assert_eq!(arjuna_research_choice(0.8), "turbines");
    assert_eq!(arjuna_research_choice(0.96), "flight");
}

#[test]
fn gun_callback_keeps_its_standing_order_below_the_c4_route_runway() {
    assert!(super::GUN_CALLBACK_RUNWAY > 2317.0, "the captured C4 artillery callback must not enter its route work");
    assert!(super::GUN_CALLBACK_RUNWAY <= 2500.0, "the measured 3k funded gun callback remains available");
}

fn bounded_planner_case(seconds:f64)->(Value,Value) {
    let world=obj!{"seconds"=>seconds,"logistics"=>obj!{"enabled"=>true},"factions"=>arr![obj!{"id"=>0,"alive"=>true}],
        "provinces"=>arr![obj!{"id"=>0,"owner"=>0,"hops"=>0,"adj"=>arr![1],"mines"=>arr![2],"staff"=>arr![2],"units"=>obj!{"0"=>8}},
            obj!{"id"=>1,"owner"=>-1,"visible"=>true,"cores"=>2,"adj"=>arr![0],"townGarrison"=>0}],
        "techs"=>obj!{"flight"=>obj!{"offer"=>1000,"requires"=>arr![]}},
        "units"=>(0..8).map(|id|obj!{"id"=>id,"owner"=>0,"type"=>if id<4{"miner"}else if id<6{"screen"}else{"skirm"},
            "at"=>0,"to"=>-1,"hp"=>4,"doing"=>if id<4{"gather"}else{"idle"},"build"=>obj!{"hp"=>2,"strength"=>2}}).collect::<Vec<_>>()};
    let me=obj!{"id"=>0,"seat"=>1,"capital"=>0,"provinces"=>1,"bank"=>10000,"ramCapital"=>obj!{},"contracts"=>arr![],"allies"=>arr![],
        "techs"=>obj!{"extraction"=>true,"dash"=>true,"ranged"=>true},"research"=>obj!{"tech"=>"flight"},
        "costs"=>obj!{"unit"=>100,"upkeep"=>100},"units"=>obj!{"total"=>8,"cap"=>32,"queued"=>0,"spawned"=>8,"stalled"=>0,
            "doing"=>obj!{"gather"=>4},"byType"=>obj!{"miner"=>4,"screen"=>2,"skirm"=>2}},
        "compute"=>obj!{"cycles"=>40000,"thinkPeak"=>80000,"used"=>obj!{}},
        "rts"=>obj!{"bank"=>300000,"cpu"=>obj!{"income"=>40000,"bank"=>300000,"capacity"=>1000000,"spendCap"=>50000,
            "last"=>obj!{"upkeep"=>1000},"limits"=>obj!{"think"=>50000,"fabrication"=>24000},
            "available"=>obj!{"think"=>50000,"control"=>40000,"fabrication"=>30000}}}};
    (world,me)
}

#[test]
fn a_resting_arjuna_does_not_replace_its_cpu_policy_with_a_research_fragment() {
    let (world,me)=bounded_planner_case(601.0);let args=views(&[world,me]);
    assert!(think(args[0],args[1]).is_empty());
}

#[test]
fn capped_arjuna_publishes_a_local_plan_and_funds_its_ground_column() {
    let (world,me)=bounded_planner_case(600.0);let args=views(&[world,me]);
    assert!(!super::full_planning_ready(args[0],args[1],1000000.0),"the earned peak cannot fit this hard cap");
    let out=think(args[0],args[1]);let cpu=out.get("compute").get("cpu");
    for lane in ["control","upkeep","fabrication","weapon","research"] {
        assert!(cpu.get("limits").get(lane).num_or(0.0)>0.0,"{lane}");
    }
    assert!(cpu.get("limits").get("think").num()<=50000.0);
    assert!(out.get("compute").get("types").get("screen").get("share").num()>0.0);
    assert!(!out.get("spawn").is_empty(),"an affordable local army still recruits");
    let mut ram=Value::obj();
    for op in out.get("ram").items(){if matches!(op.get("write"),Value::Num(_)){ram.set(&op.get("write").num().to_string(),obj!{"bytes"=>op.get("bytes").clone(),"t"=>600});}}
    let saved=views(&[ram]);let plan=mem::get(saved[0],0);
    assert_eq!(plan.get("attack").num(),1.0);
    assert!(plan.get("field").is_undef()&&plan.get("front").is_undef());
    for key in ["send","hire","bribe","rent"]{assert!(out.get(key).is_undef(),"lean planning skips {key}");}
}

#[test]
fn absent_hard_cap_keeps_the_full_planner_available() {
    let (world,mut me)=bounded_planner_case(600.0);me.entry("rts").entry("cpu").set("spendCap",Value::Undef);
    let args=views(&[world,me]);assert!(super::full_planning_ready(args[0],args[1],140000.0));
}

#[test]
fn a_decayed_recovery_peak_does_not_admit_a_large_world_scan() {
    let (mut world,mut me)=bounded_planner_case(600.0);
    for id in 2..45 { world.entry("provinces").push(obj!{"id"=>id,"owner"=>-1}); }
    me.entry("compute").set("thinkPeak",8949);
    me.entry("rts").entry("cpu").set("spendCap",27136);
    let args=views(&[world,me]);
    assert!(!super::full_planning_ready(args[0],args[1],21346.0));
    let out=think(args[0],args[1]);
    assert!(out.get("compute").get("cpu").get("limits").get("fabrication").num()>0.0);
    assert!(out.get("compute").get("types").get("screen").get("share").num()>0.0);
    assert!(!out.get("spawn").is_empty());
}

#[test]
fn the_bounded_planner_keeps_insolvent_crews_in_shared_recovery() {
    let (world,mut me)=bounded_planner_case(600.0);
    me.entry("rts").entry("cpu").set("bank",1000);
    me.entry("units").set("stalled",1);
    let args=views(&[world,me]);let out=think(args[0],args[1]);
    assert_eq!(out.get("memo").str(),"Recovering production");
    assert_eq!(out.get("compute").get("cpu").get("limits").get("fabrication").num(),0.0);
    assert!(out.get("spawn").is_empty());
}

#[test]
fn an_active_funded_topic_needs_no_flight_progress_rewrite() {
    let (world,me)=bounded_planner_case(600.0);let args=views(&[world,me]);
    let mut orders=obj!{"compute"=>obj!{"cpu"=>obj!{"limits"=>obj!{
        "think"=>50000,"research"=>0.25,"fabrication"=>24000,"control"=>40000},
        "reserve"=>obj!{"research"=>0.05}}}};
    let before=orders.clone();super::flight_progress(&mut orders,args[0],args[1]);
    assert_eq!(orders,before,"the complete standing policy already funds the active topic");
}

#[test]
fn flight_progress_still_selects_a_topic_and_reopens_recovery_research() {
    let (world,mut me)=bounded_planner_case(600.0);
    me.set("research",Value::Undef);
    let args=views(&[world.clone(),me.clone()]);
    let mut normal=obj!{"compute"=>obj!{"cpu"=>obj!{"limits"=>obj!{"research"=>0.25,"fabrication"=>24000}}}};
    super::flight_progress(&mut normal,args[0],args[1]);
    assert_eq!(normal.get("research").str(),"flight");
    assert_eq!(normal.get("compute").get("cpu").get("limits").get("research").num(),0.25);
    me.set("research",obj!{"tech"=>"flight"});
    let args=views(&[world,me]);
    let mut recovery=obj!{"memo"=>"Recovering production","compute"=>obj!{"cpu"=>obj!{"limits"=>obj!{"research"=>0,"fabrication"=>0}}}};
    super::flight_progress(&mut recovery,args[0],args[1]);
    assert_eq!(recovery.get("compute").get("cpu").get("limits").get("research").num(),0.12);
    assert_eq!(recovery.get("compute").get("cpu").get("limits").get("fabrication").num(),0.0);
}

#[test]
fn lean_recruitment_avoids_only_proven_upkeep_rejections() {
    let (mut world,mut me)=bounded_planner_case(600.0);
    me.entry("rts").entry("cpu").entry("last").set("upkeep",20000);
    let args=views(&[world.clone(),me.clone()]);let unaffordable=think(args[0],args[1]);
    assert!(unaffordable.get("spawn").is_empty());
    assert_eq!(unaffordable.get("slots").num(),0.0);
    assert!(unaffordable.get("compute").get("cpu").get("limits").get("fabrication").num()>0.0);
    // A mixed ownership view cannot establish the same lower bound for the governor's own-body census.
    let units=world.get("units").items().iter().enumerate().map(|(i,u)|{
        let mut u=u.clone();if i>=2 {u.set("owner",99);}u
    }).collect::<Vec<_>>();world.set("units",units);
    let args=views(&[world,me]);let unproved=think(args[0],args[1]);
    assert!(unproved.get("slots").num()>0.0);
    assert!(!unproved.get("spawn").is_empty());
}

#[test]
fn captured_c6_full_plan_allowance_uses_the_bounded_local_plan() {
    let (mut world, mut me) = bounded_planner_case(4668.0);
    for id in 2..44 { world.entry("provinces").push(obj! { "id"=>id, "owner"=>-1, "adj"=>arr![] }); }
    for id in 8..77 { world.entry("units").push(obj! { "id"=>id, "owner"=>2, "type"=>"screen", "at"=>1 }); }
    me.entry("compute").set("thinkPeak", 20856);
    me.entry("rts").entry("cpu").set("spendCap", 70000);
    let args = views(&[world.clone(), me.clone()]);
    assert!(!super::full_planning_ready(args[0], args[1], 45116.0));
    assert!(super::full_planning_ready(args[0], args[1], 65000.0));
    me.entry("rts").entry("cpu").set("spendCap", 45116);
    let args = views(&[world, me]);
    let out = think(args[0], args[1]);
    let plan = out.get("ram").items().iter().find_map(|op| {
        if op.get("write").num_or(-1.0) == 0.0 { Some(op) } else { None }
    }).expect("lean planning writes its local plan");
    assert!(plan.get("bytes").truthy());
    assert!(out.get("compute").get("types").get("screen").get("share").num() > 0.0);
    assert!(out.get("compute").get("cpu").get("limits").get("fabrication").num() > 0.0);
}


fn stale_local_args(seconds: f64) -> (Value, Value, Value) {
    let world = obj! {
        "seconds"=>seconds,
        "provinces"=>arr![
            obj! {"id"=>0,"owner"=>1,"visible"=>true,"adj"=>arr![1,2,3],"units"=>obj!{"1"=>1}},
            obj! {"id"=>1,"owner"=>1,"visible"=>true,"adj"=>arr![0],"units"=>obj!{}},
            obj! {"id"=>2,"owner"=>-1,"visible"=>true,"cores"=>3,"adj"=>arr![0],"units"=>obj!{}},
            obj! {"id"=>3,"owner"=>-1,"visible"=>true,"cores"=>7,"adj"=>arr![0],"units"=>obj!{}}
        ]
    };
    let unit = obj! {"id"=>71,"type"=>"screen","at"=>0,"to"=>-1,"doing"=>"idle","ramUnit"=>obj!{}};
    let me = obj! {"id"=>1,"capital"=>0,"ramProvince"=>obj!{"0"=>pages(&obj!{"attack"=>1,"defend"=>-1})}};
    (unit, world, me)
}

fn stale_order(unit: Value, world: Value, me: Value) -> Option<Value> {
    let args = views(&[unit.clone(), arr![unit], world, me]);
    super::war_order(args[0], args[1], args[2], args[3])
}

#[test]
fn fulfilled_stale_attack_captures_the_best_visible_empty_neutral_and_carries_it() {
    let (unit, world, me) = stale_local_args(122.0);
    let order = stale_order(unit, world, me).expect("the stale local plan has a safe neighboring capture");
    assert_eq!(order.get("do").str(), "capture");
    assert_eq!(order.get("target").num(), 3.0, "the richer eligible neighbor wins");
    assert!(!order.get("ram").is_empty(), "the replacement local plan travels with the capture");
}

#[test]
fn fresh_defender_protected_and_moving_orders_do_not_use_stale_local_recovery() {
    let (unit, world, me) = stale_local_args(121.0);
    let fresh = stale_order(unit.clone(), world, me).expect("ordinary attack order remains");
    assert_eq!(fresh.get("target").num(), 1.0, "a plan exactly 120 seconds old stays on its order");

    let (defender, world, mut me) = stale_local_args(122.0);
    let defend_plan = obj! {"attack"=>1,"defend"=>0};
    me.entry("ramProvince").set("0", pages(&defend_plan));
    let defending = stale_order(defender, world, me).expect("defender keeps its assigned order");
    assert_eq!(defending.get("do").str(), "fight");
    assert_eq!(defending.get("target").num(), 0.0);

    let (mut unit, world, me) = stale_local_args(122.0);
    unit.set("to", 3);
    let moving = stale_order(unit, world, me).expect("moving unit keeps its normal order");
    assert_eq!(moving.get("target").num(), 1.0);

    let (mut unit, world, me) = stale_local_args(122.0);
    unit.set("conduct", obj! {"seconds"=>20,"from"=>-1});
    let protected = stale_order(unit, world, me).expect("protected neutral remains excluded");
    assert_eq!(protected.get("target").num(), 1.0);
}

#[test]
fn a_fresh_local_enemy_blocks_stale_expansion_and_live_defense_stays_assigned() {
    for case in ["idle", "assigned-defender", "standing-defense"] {
        let (mut unit, mut world, mut me) = stale_local_args(122.0);
        let mut provinces = world.get("provinces").items().to_vec();
        provinces[0].entry("units").set("2", 1);
        world.set("provinces", provinces);
        world.set("factions", arr![Value::Null, obj!{"id"=>1,"alive"=>true}, obj!{"id"=>2,"alive"=>true}]);
        me.set("allies", arr![]);
        me.entry("ramProvince").set("0", pages(&obj!{"attack"=>0,"defend"=>if case=="idle"{-1}else{0}}));
        if case=="standing-defense" { unit.set("doing", "fight"); unit.set("target", 0); }
        let args=views(&[arr![unit],world,me]);
        super::screen(args[0],args[1],args[2]);
        let out=emitted();
        assert_eq!(out.len(), 1, "{case}: {out:?}");
        if case=="standing-defense" {
            assert!(out[0].get("do").is_undef(), "RAM refresh preserves the standing fight");
        } else {
            assert_eq!(out[0].get("target").num(), 0.0, "the local rival prevents a neighboring capture");
            assert_eq!(out[0].get("do").str(), if case=="idle"{"capture"}else{"fight"});
        }
    }
}

fn stale_field_defense_args() -> Value {
    let (mut unit, mut world, mut me) = stale_local_args(122.0);
    unit.set("doing", "fight"); unit.set("target", 0);
    let mut second = unit.clone(); second.set("id", 72);
    let mut third = unit.clone(); third.set("id", 73);
    let mut provinces = world.get("provinces").items().to_vec();
    provinces[2].entry("units").set("6", 1);
    provinces[3].set("owner", 6);
    world.set("provinces", provinces);
    let mut factions = vec![Value::Null; 7];
    factions[1] = obj!{"id"=>1,"alive"=>true}; factions[6] = obj!{"id"=>6,"alive"=>true};
    world.set("factions", factions);
    me.set("capital", 1); me.set("allies", arr![]);
    me.entry("ramProvince").set("0", pages(&obj!{"attack"=>3,"defend"=>0,"threat"=>1}));
    arr![arr![unit,second,third],world,me]
}

#[test]
fn a_cleared_stale_field_defense_can_contest_a_neutral_neighbor() {
    let fixture = stale_field_defense_args();
    let args = views(fixture.items());
    super::screen(args[0], args[1], args[2]);
    let out = emitted();
    assert!(out.iter().any(|o| o.get("do").str() == "capture" && o.get("target").num() == 2.0), "{out:?}");
    assert!(out.iter().filter(|o| o.get("do").str() == "capture").all(|o| !o.get("ram").is_empty()));
}

#[test]
fn stale_field_defenders_keep_current_threats_and_do_not_send_a_lone_unit() {
    for threat in [false, true] {
        let fixture = stale_field_defense_args();
        let mut args = fixture.items().to_vec();
        if threat {
            let mut provinces = args[1].get("provinces").items().to_vec();
            let here = provinces.iter_mut().find(|p| p.get("id").num() == 0.0).unwrap();
            here.entry("units").set("6", 1);
            args[1].set("provinces", provinces);
        } else {
            args[0] = arr![args[0].at(0).clone()];
        }
        let args = views(&args);
        super::screen(args[0], args[1], args[2]);
        let out = emitted();
        assert!(out.iter().all(|o| o.get("do").str() != "capture"), "threat={threat}: {out:?}");
    }
}

#[test]
fn a_newer_carried_advance_survives_stale_ram_in_captured_land() {
    for (fresh, enemy) in [(false, false), (true, false), (false, true)] {
        let (mut unit, mut world, mut me) = stale_local_args(244.0);
        unit.set("at", 2);
        unit.set("ramUnit", obj!{"0"=>obj!{"bytes"=>Value::Bytes(plan::pack(
            &obj!{"attack"=>2,"stage"=>0,"defend"=>-1,"need"=>2,"at"=>122}))}});
        let mut provinces = world.get("provinces").items().to_vec();
        provinces[2].set("owner", 1); provinces[2].set("adj", arr![0,3]);
        provinces[3].set("adj", arr![2]);
        if enemy { provinces[2].entry("units").set("6", 1); }
        world.set("provinces", provinces);
        let mut factions = vec![Value::Null; 7];
        factions[1] = obj!{"id"=>1,"alive"=>true}; factions[6] = obj!{"id"=>6,"alive"=>true};
        world.set("factions", factions);
        me.set("capital", 1); me.set("allies", arr![]);
        let mut shared = pages(&obj!{"attack"=>3,"defend"=>0,"threat"=>1});
        if fresh { shared.entry("0").set("t", 243); }
        me.entry("ramProvince").set("2", shared);
        let order = stale_order(unit, world, me);
        if !fresh && !enemy {
            let order = order.expect("continue from captured land");
            assert_eq!(order.get("do").str(), "capture");
            assert_eq!(order.get("target").num(), 3.0);
        } else {
            assert!(order.is_none_or(|o| o.get("do").str() != "capture"));
        }
    }
}


#[test]
fn field_defenders_keep_fresh_orders_and_capital_defense() {
    for fresh in [false, true] {
        let fixture = stale_field_defense_args();
        let mut args = fixture.items().to_vec();
        if fresh {
            args[2].entry("ramProvince").entry("0").entry("0").set("t", 121);
        } else {
            args[2].set("capital", 0);
        }
        let args = views(&args);
        super::screen(args[0], args[1], args[2]);
        let out = emitted();
        assert!(out.iter().all(|o| o.get("do").str() != "capture"), "fresh={fresh}: {out:?}");
    }
}

#[test]
fn the_captured_field_advance_reserves_the_full_planning_tail() {
    let (mut world, mut me) = bounded_planner_case(21936.0);
    for id in 2..44 { world.entry("provinces").push(obj! { "id"=>id, "owner"=>-1, "adj"=>arr![] }); }
    for id in 8..56 { world.entry("units").push(obj! { "id"=>id, "owner"=>2, "type"=>"screen", "at"=>1 }); }
    me.entry("compute").set("thinkPeak", 19829);
    me.entry("rts").entry("cpu").set("spendCap", 70000);
    let args = views(&[world, me]);
    assert!(!super::full_planning_ready(args[0], args[1], 53010.0));
    assert!(super::full_planning_ready(args[0], args[1], 65000.0));
}

#[test]
fn artillery_orders_strike_walls_only_during_an_offensive_siege() {
    for (walls,defending,want) in [(false,false,"fight"),(true,false,"capture"),(true,true,"fight")] {
        let fixture=wall_artillery_fixture("gun",walls,defending,false,false);
        let args=views(&fixture);
        super::gun(args[0],args[1],args[2]);
        let orders=emitted();
        assert!(orders.iter().any(|o|o.get("do").str()==want && o.get("target").num()==1.0),
            "walls={walls}, defending={defending}: {orders:?}");
    }
}

#[test]
fn artillery_already_at_the_enemy_town_switches_its_standing_fight_to_the_walls() {
    for deployed in [false,true] {
        let fixture=wall_artillery_fixture("gun",true,false,true,deployed);
        let args=views(&fixture);
        super::gun(args[0],args[1],args[2]);
        let orders=emitted();
        assert!(orders.iter().any(|o|o.get("do").str()=="capture" && o.get("target").num()==1.0),
            "deployed={deployed}: {orders:?}");
        if deployed {assert!(!orders.iter().any(|o|matches!(o.get("deploy"),Value::Bool(false))),
            "a gun already at the siege keeps its deployment");}
    }
}

#[test]
fn generic_escorts_follow_the_program_catalogue_when_houses_change_seats() {
    for seat in [0,4,7] {
        for fleet in [false,true] {
            let mut fixture=productive_local_fixture();
            fixture[0].set("seconds",3);
            fixture[1].set("seat",seat);
            fixture[1].entry("units").entry("byType").set("freighter",1);
            fixture[1].entry("techs").set("seaTraversal",true);
            fixture[1].entry("techs").set("ranged",true);
            fixture[1].entry("rts").entry("sea").set("ports",arr![0]);
            fixture[1].entry("rts").set("licenses",arr!["hull"]);
            fixture[1].set("build",obj! {"points"=>10,"wingRange"=>0,
                "max"=>obj! {"hp"=>5,"speed"=>5,"strength"=>5,"carry"=>4,"vision"=>5,"range"=>2,"wings"=>0,"stealth"=>0},
                "standard"=>obj! {"hp"=>2,"speed"=>2,"strength"=>2,"carry"=>2,"vision"=>2,"range"=>0,"wings"=>0,"stealth"=>0},
                "price"=>obj! {"hp"=>1,"speed"=>1,"strength"=>1,"carry"=>1,"vision"=>1,"range"=>2,"wings"=>3,"stealth"=>2}});
            if fleet {fixture[1].entry("types").set("warjunk",obj! {"name"=>"Warjunk"});}
            let args=views(&fixture);
            let orders=common::govern(args[0],args[1],0.4,||obj! {});
            let escort=orders.get("spawn").items().iter().any(|s|s.get("type").str()=="escort");
            assert_eq!(escort,!fleet,"seat={seat}, dedicated fleet={fleet}: {orders:?}");
        }
    }
}
