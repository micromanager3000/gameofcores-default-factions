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
    let args=views(&[world,me]);assert!(super::full_planning_ready(args[0],args[1],120000.0));
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
