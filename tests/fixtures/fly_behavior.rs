// Build the air callback view: unit callbacks receive contacts and decks, never world.units.
fn flight_views(values: &[Value]) -> Vec<View> {
    if values.len() != 3 { return views(values); }
    let mut v=values.to_vec();
    let mut contacts=Value::arr();
    let mut decks=Value::arr();
    for u in v[1].get("units").items() {
        if u.get("owner").num()!=v[2].get("id").num() && u.get("air").truthy() {contacts.push(u.clone());}
        let deck=u.get("rts").get("sea").get("deck");
        if u.get("owner").num()==v[2].get("id").num() && deck.get("room").num_or(0.0)>0.0 {
            decks.push(obj!{"id"=>u.get("id").clone(),"at"=>u.get("at").clone(),"room"=>deck.get("room").clone(),"free"=>deck.get("free").num_or(deck.get("room").num_or(0.0)-deck.get("riders").len() as f64),"riders"=>deck.get("riders").clone()});
        }
    }
    v[1].set("units",Value::arr());
    v[2].entry("rts").set("air",obj!{"contacts"=>contacts,"decks"=>decks});
    views(&v)
}

#[test]
fn a_standing_fighter_acquires_air_focus_while_scouts_do_not() {
    let plan = obj! {"attack"=>5,"stage"=>5,"defend"=>-1,"need"=>1,"threat"=>0};
    let mut pages = Value::obj();
    for op in mem::put(0, &plan, "local", None, 1).expect("small plan") {
        pages.set(&op.get("write").num().to_string(), obj! {"bytes"=>op.get("bytes").clone(),"t"=>1});
    }
    let world = Value::parse(r#"{"seconds":1,"factions":[null,null,{"id":2,"alive":true,"shielded":false}],"provinces":[{"id":5,"owner":1}],"units":[
        {"id":201,"owner":2,"air":true,"at":5,"pos":{"p":5,"x":0,"y":0},"hp":4,"hpMax":5}]}"#);
    let me = obj! {"id"=>1,"ramProvince"=>obj! {"5"=>pages}};
    let units = Value::parse(r#"[
        {"id":101,"owner":1,"air":true,"at":5,"doing":"fight","target":5,"build":{"vision":0,"range":0},"rts":{"air":{"heat":0,"aloft":true}}},
        {"id":102,"owner":1,"air":true,"at":5,"doing":"move","target":5,"build":{"vision":3,"range":0},"rts":{"air":{"heat":0,"aloft":true}}}
    ]"#);
    let args = flight_views(&[units, world, me]);
    common::fly(args[0], args[1], args[2]);
    let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o| Value::parse(o)).collect();
    assert_eq!(orders.len(), 1, "{orders:?}");
    let fighter = &orders[0];
    assert_eq!(fighter.get("id").num(), 101.0);
    assert_eq!(fighter.get("do").str(), "fight");
    assert_eq!(fighter.get("target").num(), 5.0);
    assert_eq!(fighter.get("focus").num(), 201.0);
}

#[test]
fn a_standing_fighter_clears_focus_when_the_aircraft_has_left() {
    let plan = obj! {"attack"=>5,"stage"=>5,"defend"=>-1,"need"=>1,"threat"=>0};
    let mut pages = Value::obj();
    for op in mem::put(0, &plan, "local", None, 1).expect("small plan") {
        pages.set(&op.get("write").num().to_string(), obj! {"bytes"=>op.get("bytes").clone(),"t"=>1});
    }
    let world = Value::parse(r#"{"seconds":1,"provinces":[{"id":5,"owner":1}],"units":[]}"#);
    let me = obj! {"id"=>1,"ramProvince"=>obj! {"5"=>pages}};
    let units = Value::parse(r#"[{"id":101,"owner":1,"air":true,"at":5,"doing":"fight","target":5,"focus":201,
        "build":{"vision":0,"range":0},"rts":{"air":{"heat":0,"aloft":true}}}]"#);
    let args = flight_views(&[units, world, me]);
    common::fly(args[0], args[1], args[2]);
    let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o| Value::parse(o)).collect();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("do").str(), "fight");
    assert_eq!(orders[0].get("target").num(), 5.0);
    assert!(orders[0].get("focus").is_undef(), "the full order clears the stale focus");
}

#[test]
fn an_affordable_ranged_bomber_is_given_a_structure_attack() {
    let world = Value::parse(r#"{"seconds":6,"factions":[null,null,{"id":2,"alive":true,"shielded":false}],"units":[
        {"id":101,"owner":1,"air":true,"hp":5,"pos":{"p":5,"x":0,"y":0},"build":{"range":1}},
        {"id":201,"owner":2,"air":false,"hp":5,"pos":{"p":5,"x":0.5,"y":0}}]}"#);
    let me = Value::parse(r#"{"id":1,"seat":1,"units":{"stalled":0},"compute":{"thinkPeak":0},
        "rts":{"structureCosts":{"base":1},"cpu":{"income":10000,"bank":100000,"last":{"upkeep":0}},
        "structures":[{"id":9,"owner":2,"state":"ready","p":5,"x":0,"y":0,"radius":1,"hp":100,"power":true}]}}"#);
    let args = flight_views(&[world, me]);
    let mut orders = obj! {"memo"=>""};
    shared::structures::apply(&mut orders, args[0], args[1]);
    assert_eq!(orders.get("structureAttacks").len(), 1, "{orders:?}");
    assert_eq!(orders.get("structureAttacks").at(0).get("unit").num(), 101.0);
    assert_eq!(orders.get("structureAttacks").at(0).get("target").num(), 9.0);
}

#[test]
fn passage_covered_bomber_does_not_target_its_host_shell() {
    let world = Value::parse(r#"{"seconds":6,"factions":[null,null,{"id":2,"alive":true,"shielded":false}],
        "provinces":[{"id":5,"owner":2}],"units":[
        {"id":101,"owner":1,"air":true,"at":5,"hp":5,"pos":{"p":5,"x":0,"y":0},"build":{"range":1}}]}"#);
    let me = Value::parse(r#"{"id":1,"seat":1,"units":{"stalled":0},"compute":{"thinkPeak":0},
        "contracts":[{"with":2,"ended":false,"get":{"passage":{"units":1,"seconds":60}}}],
        "rts":{"structureCosts":{"base":1},"cpu":{"income":10000,"bank":100000,"last":{"upkeep":0}},
        "structures":[{"id":9,"owner":2,"state":"ready","p":5,"x":0,"y":0,"radius":1,"hp":100,"power":true}]}}"#);
    let args = flight_views(&[world, me]);
    let mut orders = obj! {"memo"=>""};
    shared::structures::apply(&mut orders, args[0], args[1]);
    assert!(orders.get("structureAttacks").is_empty(), "{orders:?}");
}

#[test]
fn fighters_intercept_local_or_adjacent_rivals_but_not_allies_or_scouts() {
    let plan = obj! {"attack"=>5,"stage"=>5,"defend"=>-1,"need"=>1,"threat"=>0};
    let mut pages = Value::obj();
    for op in mem::put(0, &plan, "local", None, 1).expect("small plan") {
        pages.set(&op.get("write").num().to_string(), obj! {"bytes"=>op.get("bytes").clone(),"t"=>1});
    }
    let world = Value::parse(r#"{"seconds":1,"factions":[null,null,{"id":2,"alive":true,"shielded":false},{"id":3,"alive":true,"shielded":false}],
        "provinces":[{"id":5,"owner":1,"adj":[6]},{"id":6,"owner":2,"adj":[5]}],"units":[
        {"id":201,"owner":2,"air":true,"at":5,"hp":5,"hpMax":5,"pos":{"p":5,"x":0,"y":0}},
        {"id":202,"owner":2,"air":true,"at":6,"hp":0.1,"hpMax":5,"pos":{"p":6,"x":0,"y":0}},
        {"id":203,"owner":3,"air":true,"at":5,"hp":0.1,"hpMax":5,"pos":{"p":5,"x":0,"y":0}}]}"#);
    let me = obj! {"id"=>1,"allies"=>arr![3],"ramProvince"=>obj! {"5"=>pages}};
    let units = Value::parse(r#"[
        {"id":101,"owner":1,"air":true,"at":5,"doing":"fight","target":5,"build":{"vision":0,"range":0},"rts":{"air":{"heat":0,"aloft":true}}},
        {"id":102,"owner":1,"air":true,"at":5,"doing":"move","target":5,"build":{"vision":3,"range":0},"rts":{"air":{"heat":0,"aloft":true}}}
    ]"#);
    let args = flight_views(&[units, world, me]);
    common::fly(args[0], args[1], args[2]);
    let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o| Value::parse(o)).collect();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("id").num(), 101.0);
    assert_eq!(orders[0].get("target").num(), 5.0, "the local rival wins over the wounded adjacent rival");
    assert_eq!(orders[0].get("focus").num(), 201.0, "allied aircraft are not interception targets");
}

#[test]
fn a_bomber_besieges_despite_a_nearby_ground_enemy_while_ground_forces_keep_fighting() {
    let world = Value::parse(r#"{"seconds":6,"factions":[null,null,{"id":2,"alive":true,"shielded":false}],"units":[
        {"id":101,"owner":1,"air":true,"hp":5,"pos":{"p":5,"x":0,"y":0},"build":{"range":1}},
        {"id":102,"owner":1,"air":false,"hp":5,"pos":{"p":5,"x":0,"y":1},"build":{"range":1}},
        {"id":201,"owner":2,"air":false,"hp":5,"pos":{"p":5,"x":1,"y":0}}]}"#);
    let me = Value::parse(r#"{"id":1,"seat":1,"units":{"stalled":0},"compute":{"thinkPeak":0},
        "rts":{"structureCosts":{"base":1},"cpu":{"income":10000,"bank":100000,"last":{"upkeep":0}},
        "structures":[{"id":9,"owner":2,"state":"ready","p":5,"x":0,"y":0,"radius":1,"hp":100,"power":true}]}}"#);
    let args = flight_views(&[world, me]);
    let mut orders = obj! {"memo"=>""};
    shared::structures::apply(&mut orders, args[0], args[1]);
    assert_eq!(orders.get("structureAttacks").len(), 1, "{orders:?}");
    assert_eq!(orders.get("structureAttacks").at(0).get("unit").num(), 101.0);
    assert_eq!(orders.get("structureAttacks").at(0).get("target").num(), 9.0);
}

#[test]
fn hot_wings_board_a_nearby_free_carrier_and_reserve_its_last_deck_slot() {
    let world = Value::parse(r#"{"seconds":1,"provinces":[{"id":5,"owner":1,"adj":[6]},{"id":6,"owner":1,"adj":[5]}],"units":[
        {"id":501,"owner":1,"type":"carrier","at":6,"hp":5,"rts":{"sea":{"ship":true,"deck":{"room":1,"riders":[]}}}}]}"#);
    let me = obj! {"id"=>1,"units"=>obj! {"byType"=>obj! {"carrier"=>1}}};
    let units = Value::parse(r#"[
        {"id":101,"owner":1,"air":true,"at":5,"build":{"vision":0,"range":0},"rts":{"air":{"heat":30,"aloft":true},"sea":{}}},
        {"id":102,"owner":1,"air":true,"at":5,"build":{"vision":0,"range":0},"rts":{"air":{"heat":30,"aloft":true},"sea":{}}}
    ]"#);
    let args = flight_views(&[units, world, me]);
    common::fly(args[0], args[1], args[2]);
    let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o| Value::parse(o)).collect();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("id").num(), 101.0);
    assert_eq!(orders[0].get("board").num(), 501.0);
}

#[test]
fn a_wing_preserves_its_boarding_course_and_launches_after_cooling() {
    let world = Value::parse(r#"{"seconds":1,"units":[{"id":501,"owner":1,"type":"carrier","at":5,"hp":5,"rts":{"sea":{"ship":true,"deck":{"room":2,"riders":[]}}}}]}"#);
    let me = obj! {"id"=>1,"units"=>obj! {"byType"=>obj! {"carrier"=>1}}};
    let returning = Value::parse(r#"[{"id":101,"owner":1,"air":true,"at":5,"build":{"vision":0,"range":0},
        "rts":{"air":{"heat":30,"aloft":true},"sea":{"course":{"board":501}}}}]"#);
    let args = flight_views(&[returning, world.clone(), me.clone()]);
    common::fly(args[0], args[1], args[2]);
    assert!(abi::TEST_ORDERS.with(|o| o.take()).is_empty(), "an existing return course stands");

    let aboard = Value::parse(r#"[{"id":101,"owner":1,"air":true,"at":5,"build":{"vision":0,"range":0},
        "rts":{"air":{"heat":0,"aloft":false},"sea":{"aboard":501}}}]"#);
    let args = flight_views(&[aboard, world, me]);
    common::fly(args[0], args[1], args[2]);
    let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o| Value::parse(o)).collect();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("id").num(), 101.0);
    assert!(orders[0].get("launch").truthy());
}

#[test]
fn a_hot_wing_lands_when_no_nearby_carrier_has_a_free_deck() {
    for carrier in [
        r#"{"id":501,"owner":1,"type":"carrier","at":6,"hp":5,"rts":{"sea":{"ship":true,"deck":{"room":1,"riders":[77]}}}}"#,
        r#"{"id":501,"owner":1,"type":"carrier","at":8,"hp":5,"rts":{"sea":{"ship":true,"deck":{"room":2,"riders":[]}}}}"#,
    ] {
        let world = Value::parse(&format!(r#"{{"seconds":1,"provinces":[{{"id":5,"owner":1,"adj":[6]}},{{"id":6,"owner":1,"adj":[5]}},{{"id":8,"owner":1,"adj":[]}}],"units":[{carrier}]}}"#));
        let me = obj! {"id"=>1,"units"=>obj! {"byType"=>obj! {"carrier"=>1}}};
        let units = Value::parse(r#"[{"id":101,"owner":1,"air":true,"at":5,"doing":"fight","target":5,
            "build":{"vision":0,"range":0},"rts":{"air":{"heat":45,"aloft":true},"sea":{}}}]"#);
        let args = flight_views(&[units, world, me]);
        common::fly(args[0], args[1], args[2]);
        let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o| Value::parse(o)).collect();
        assert_eq!(orders.len(), 1, "{carrier}: {orders:?}");
        assert_eq!(orders[0].get("id").num(), 101.0);
        assert!(orders[0].get("land").truthy(), "{carrier}: {orders:?}");
    }
}

#[test]
fn fighters_redirect_to_an_adjacent_aircraft_before_it_reaches_the_objective() {
    let plan = obj! {"attack"=>5,"stage"=>5,"defend"=>-1,"need"=>1,"threat"=>0};
    let mut pages = Value::obj();
    for op in mem::put(0, &plan, "local", None, 1).expect("small plan") {
        pages.set(&op.get("write").num().to_string(), obj! {"bytes"=>op.get("bytes").clone(),"t"=>1});
    }
    let world = Value::parse(r#"{"seconds":1,"factions":[null,null,{"id":2,"alive":true,"shielded":false}],
        "provinces":[{"id":5,"owner":1,"adj":[6]},{"id":6,"owner":2,"adj":[5]}],"units":[
        {"id":202,"owner":2,"air":true,"at":6,"hp":5,"hpMax":5,"pos":{"p":6,"x":0,"y":0}}]}"#);
    let me = obj! {"id"=>1,"ramProvince"=>obj! {"5"=>pages}};
    let units = Value::parse(r#"[{"id":101,"owner":1,"air":true,"at":5,"doing":"fight","target":5,
        "build":{"vision":0,"range":0},"rts":{"air":{"heat":0,"aloft":true}}}]"#);
    let args = flight_views(&[units, world, me]);
    common::fly(args[0], args[1], args[2]);
    let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o| Value::parse(o)).collect();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("id").num(), 101.0);
    assert_eq!(orders[0].get("target").num(), 6.0);
    assert_eq!(orders[0].get("focus").num(), 202.0);
}

#[test]
fn bomber_approaches_an_off_center_hostile_plant_and_keeps_the_paid_course() {
    let world=Value::parse(r#"{"factions":[null,{"id":1,"alive":true},{"id":2,"alive":true}]}"#);
    let me=Value::parse(r#"{"id":1,"rts":{"structures":[{"id":44,"owner":2,"p":5,"x":7,"y":2,"hp":100,"power":true,"state":"ready"}]}}"#);
    let mut bomber=Value::parse(r#"{"id":101,"air":true,"at":5,"to":-1,"doing":"fight","target":5,"spot":{"x":0,"y":0},"build":{"range":1}}"#);
    let args=views(&[bomber.clone(),world.clone(),me.clone()]);
    let o=common::bombing_run(args[0],args[1],args[2],5.0).expect("bombing objective").expect("new approach");
    assert_eq!(o.get("at").get("x").num(),7.0);
    assert_eq!(o.get("at").get("y").num(),2.0);
    bomber.set("spot",obj!{"x"=>7,"y"=>2});
    let args=views(&[bomber.clone(),world.clone(),me.clone()]);
    assert!(common::bombing_run(args[0],args[1],args[2],5.0).expect("same objective").is_none());
    bomber.set("conduct",obj!{"from"=>2,"seconds"=>30});
    let args=views(&[bomber,world,me]);
    assert!(common::bombing_run(args[0],args[1],args[2],5.0).is_none(),"safe passage protects the plant");
}

#[test]
fn an_aircraft_callback_lands_during_economic_recovery_and_waits_when_down() {
    let units=Value::parse(r#"[{"id":101,"air":true,"rts":{"air":{"aloft":true,"heat":12}}},{"id":102,"air":true,"rts":{"air":{"aloft":false,"heat":0}}}]"#);
    let me=Value::parse(r#"{"rts":{"cpu":{"limits":{"fabrication":0}}}}"#);
    let args=views(&[units,me]);
    assert!(common::air_recovery(args[0],args[1]));
    let orders:Vec<Value>=abi::TEST_ORDERS.with(|o|o.take()).iter().map(|o|Value::parse(o)).collect();
    assert_eq!(orders.len(),1);
    assert_eq!(orders[0].get("id").num(),101.0);
    assert!(orders[0].get("land").truthy());
}

#[test]
fn returning_aircraft_skip_cargo_filled_decks_and_land_when_none_can_accept_them() {
    let units = Value::parse(r#"[{"id":101,"owner":1,"air":true,"at":5,
        "rts":{"air":{"heat":45,"aloft":true},"sea":{"course":{"board":500}}}}]"#);
    let world = Value::parse(r#"{"seconds":1,"provinces":[{"id":5,"owner":1,"adj":[]}]}"#);
    let mut me = Value::parse(r#"{"id":1,"rts":{"air":{"decks":[
        {"id":500,"at":5,"room":3,"free":0,"riders":[]},
        {"id":501,"at":5,"room":2,"free":1,"riders":[102]}]}}}"#);
    let args = views(&[units.clone(),world.clone(),me.clone()]);
    common::fly(args[0],args[1],args[2]);
    let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o|Value::parse(o)).collect();
    assert_eq!(orders.len(),1,"{orders:?}");
    assert_eq!(orders[0].get("board").num(),501.0,"a rider already aboard is already deducted from free");
    me.entry("rts").entry("air").set("decks",vec![obj!{"id"=>500,"at"=>5,"room"=>3,"free"=>0,"riders"=>Value::arr()}]);
    let args = views(&[units,world,me]);
    common::fly(args[0],args[1],args[2]);
    let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o|Value::parse(o)).collect();
    assert_eq!(orders.len(),1,"{orders:?}");
    assert!(orders[0].get("land").truthy(),"an unusable paid return must not prevent emergency landing");
}
