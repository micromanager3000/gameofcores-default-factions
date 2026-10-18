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
