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
    let mut me = obj! { "id"=>0, "capital"=>25, "allies"=>arr![], "ramProvince"=>obj! {} };
    me.entry("ramProvince").set("11", pages(&obj! { "attack"=>11, "stage"=>25 }));
    let fixture = vec![
        Value::parse(r#"[{"id":733,"owner":0,"type":"gun","at":11,"to":-1,"doing":"fight","target":11,"rts":{"deployed":true}}]"#),
        Value::parse(r#"{"provinces":[{"id":11,"owner":1,"units":{"0":1}}]}"#), me,
    ];
    let args = views(&fixture);
    super::gun(args[0], args[1], args[2]);
    assert!(emitted().is_empty(), "a deployed gun already at its target keeps firing its standing order");
}

#[test]
fn at_target_gun_can_still_set_up_without_route_fuel() {
    let mut me = obj! { "id"=>0, "capital"=>25, "allies"=>arr![], "ramProvince"=>obj! {} };
    me.entry("ramProvince").set("11", pages(&obj! { "attack"=>11, "stage"=>25 }));
    let fixture = vec![
        Value::parse(r#"[{"id":733,"owner":0,"type":"gun","at":11,"to":-1,"doing":"fight","target":11,"rts":{"deployed":false}}]"#),
        Value::parse(r#"{"provinces":[{"id":11,"owner":1,"units":{"0":1}}]}"#), me,
    ];
    let args = views(&fixture);
    super::gun(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "the target gun needs only its setup order: {orders:?}");
    assert!(orders[0].get("deploy").truthy());
}

#[test]
fn a_funded_gun_reads_a_cold_home_plan_and_starts_its_route() {
    let mut me=obj!{"id"=>0,"capital"=>25,"allies"=>arr![],"rts"=>obj!{},"ramProvince"=>obj!{}};
    let mut cold=Value::obj();
    for op in mem::put(0,&obj!{"attack"=>11,"stage"=>25,"home"=>25,"detail"=>"x".repeat(512)},"local",None,16).unwrap(){
        cold.set(&op.get("write").num().to_string(),obj!{"bytes"=>op.get("bytes").clone(),"t"=>1});
    }
    me.entry("ramProvince").set("25",cold);
    let units=arr![obj!{"id"=>733,"owner"=>0,"type"=>"gun","at"=>25,"to"=>-1,"doing"=>"idle",
        "target"=>-1,"ramUnit"=>obj!{},"rts"=>obj!{"deployed"=>false}}];
    let world=obj!{"seconds"=>1,"factions"=>arr![obj!{"id"=>0,"alive"=>true},obj!{"id"=>1,"alive"=>true}],
        "provinces"=>arr![obj!{"id"=>25,"owner"=>0,"adj"=>arr![11]},obj!{"id"=>11,"owner"=>1,"adj"=>arr![25]}]};
    let args=views(&[units,world,me]);
    super::gun(args[0],args[1],args[2]);
    let orders=emitted();
    assert_eq!(orders.len(),1);
    assert_eq!(orders[0].get("do").str(),"fight");
    assert_eq!(orders[0].get("target").num(),11.0);
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

#[test]
fn kublai_wing_recovery_skips_the_cold_planner_when_fabrication_is_zero() {
    let fixture = [
        Value::parse(r#"[{"id":1034,"owner":0,"type":"wing","at":1,"to":-1,"doing":"idle","target":-1,"rts":{"air":{"aloft":false,"heat":0}}}]"#),
        Value::parse(r#"{"provinces":[{"id":1,"owner":0,"units":{"0":1}}]}"#),
        Value::parse(r#"{"id":0,"rts":{"cpu":{"limits":{"fabrication":0}}}}"#),
    ];
    let args = views(&fixture);
    super::wing(args[0], args[1], args[2]);
    assert!(emitted().is_empty(), "the grounded recovery wing keeps its standing order");
}

#[test]
fn road_phase_leaves_the_standing_field_when_a_new_plan_cannot_publish() {
    let field=obj!{"to"=>3,"at"=>100,"next"=>obj!{"0"=>3}};
    let world=obj!{"seconds"=>101};
    let me=obj!{"rts"=>obj!{"cpu"=>obj!{}},"ramCapital"=>pages(&obj!{"field"=>field.clone()})};
    let a=views(&[world,me]);
    let mut plan=obj!{"attack"=>3};
    road_phase(&mut plan,a[0],a[1],3.0,8999.0);
    assert!(plan.get("field").is_undef(),"a short turn keeps the existing shared plan instead of decoding an unpublished field");
    road_phase(&mut plan,a[0],a[1],3.0,9000.0);
    assert_eq!(plan.get("field").get("to").num(),3.0);
    assert_eq!(plan.get("field").get("next").get("0").num(),3.0);
}

#[test]
fn kublai_wing_preserves_urgent_landings_and_a_funded_sortie() {
    let mut me=obj!{"id"=>0,"capital"=>1,"allies"=>arr![],"rts"=>obj!{"cpu"=>obj!{"limits"=>obj!{"fabrication"=>20488}}},"ramProvince"=>obj!{}};
    me.entry("ramProvince").set("1",pages(&obj!{"attack"=>14,"hunt"=>14,"defend"=>-1,"stage"=>1}));
    let unit=obj!{"id"=>1365,"owner"=>0,"type"=>"wing","at"=>1,"to"=>-1,"doing"=>"idle","target"=>-1,
        "build"=>obj!{"wings"=>1,"vision"=>0},"rts"=>obj!{"air"=>obj!{"aloft"=>false,"heat"=>0}}};
    let world=obj!{"seconds"=>15015,"factions"=>arr![obj!{"id"=>0,"alive"=>true},obj!{"id"=>1,"alive"=>true}],
        "provinces"=>arr![obj!{"id"=>1,"owner"=>0,"adj"=>arr![14]},obj!{"id"=>14,"owner"=>1,"adj"=>arr![1],"units"=>obj!{"1"=>1}}]};
    let v=views(&[arr![unit.clone()],world.clone(),me.clone()]);
    super::wing(v[0],v[1],v[2]);
    let out=emitted();
    assert!(out.iter().any(|o|o.get("id").num()==1365.0 && o.get("target").num()==14.0),"funded wing keeps its sortie: {out:?}");
    let mut hot=unit;
    hot.entry("rts").entry("air").set("aloft",true);
    hot.entry("rts").entry("air").set("heat",65);
    let v=views(&[arr![hot],world,me]);
    super::wing(v[0],v[1],v[2]);
    let out=emitted();
    assert_eq!(out.len(),1,"hot aircraft land before optional route work");
    assert!(out[0].get("land").truthy());
}
#[test]
fn a_hired_hull_retrained_as_skirm_guards_shipping_instead_of_marching_ashore() {
    let u=obj! {"id"=>77,"type"=>"skirm","owner"=>0,"at"=>1,"to"=>-1,"doing"=>"idle","build"=>obj! {"carry"=>0,"strength"=>2},
        "rts"=>obj! {"sea"=>obj! {"ship"=>true,"water"=>1,"course"=>"stop"}}};
    let world=obj! {"seconds"=>30,"logistics"=>obj! {"docks"=>vec![obj! {"p"=>0,"water"=>0},obj! {"p"=>1,"water"=>1}]},
        "waters"=>vec![obj! {"id"=>0,"adj"=>vec![1],"coast"=>vec![0]},obj! {"id"=>1,"adj"=>vec![0],"coast"=>vec![1]}],
        "provinces"=>vec![obj! {"id"=>0,"owner"=>0,"hops"=>0,"adj"=>vec![1]},obj! {"id"=>1,"owner"=>0,"hops"=>1,"adj"=>vec![0]}]};
    let me=obj! {"id"=>0,"capital"=>0,"ramCapital"=>obj! {},"rts"=>obj! {"cpu"=>obj! {"income"=>30000},"sea"=>obj! {"contacts"=>Vec::<Value>::new()}}};
    let args=views(&[u.clone(),world,me]);
    assert!(super::war_order(args[0],args[1],args[2]).is_none());
    let orders=emitted();
    assert_eq!(orders.len(),1);
    assert_eq!(orders[0].get("sail").num(),0.0,"the acquired ship returns to the receiving dock");
    assert!(orders[0].get("do").is_undef(),"a hull receives no land verb");
    let mut land=u;land.entry("rts").entry("sea").set("ship",false);
    let ground=View::of(&land);
    assert_eq!(super::war_order(ground,args[1],args[2]).expect("ground defence").get("do").str(),"fight");
}
#[test]
fn a_short_kublai_plan_rebuilds_capturers_beside_the_working_mining_crew() {
    let args=views(&productive_local_fixture());
    let out=super::local_strategy(args[0],args[1]);
    assert!(out.get("spawn").items().iter().any(|s|s.get("type").str()=="screen"));
    assert!(out.get("ram").len()>0,"the next reachable target is published");
    assert!(out.get("compute").get("cpu").truthy());
}
#[test]
fn a_local_kublai_army_develops_the_core_it_just_held() {
    let args=views(&frontier_mining_fixture("screen"));
    let out=super::local_strategy(args[0],args[1]);
    assert!(out.get("spawn").items().iter().any(|s|s.get("type").str()=="miner"));
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
