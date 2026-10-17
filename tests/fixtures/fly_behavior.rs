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
    let args = views(&[units, world, me]);
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
    let args = views(&[units, world, me]);
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
        {"id":201,"owner":2,"air":false,"hp":5,"pos":{"p":7,"x":0,"y":0}}]}"#);
    let me = Value::parse(r#"{"id":1,"seat":1,"units":{"stalled":0},"compute":{"thinkPeak":0},
        "rts":{"structureCosts":{"base":1},"cpu":{"income":10000,"bank":100000,"last":{"upkeep":0}},
        "structures":[{"id":9,"owner":2,"state":"ready","p":5,"x":0,"y":0,"radius":1,"hp":100,"power":true}]}}"#);
    let args = views(&[world, me]);
    let mut orders = obj! {"memo"=>""};
    shared::structures::apply(&mut orders, args[0], args[1]);
    assert_eq!(orders.get("structureAttacks").len(), 1, "{orders:?}");
    assert_eq!(orders.get("structureAttacks").at(0).get("unit").num(), 101.0);
    assert_eq!(orders.get("structureAttacks").at(0).get("target").num(), 9.0);
}
