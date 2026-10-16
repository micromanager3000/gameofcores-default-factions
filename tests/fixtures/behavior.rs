use super::*;
fn item(v:&mut Value,i:usize)->&mut Value { let Value::Arr(a)=v else {panic!("array")}; &mut a[i] }

fn fixture() -> (Value, Value, Value) {
    let world = Value::parse(r#"{"seconds":1,"logistics":{"docks":[{"p":0,"water":0},{"p":1,"water":1},{"p":2,"water":2}]},
        "provinces":[{"id":0,"owner":1,"adj":[],"hops":0},{"id":1,"owner":1,"adj":[]},{"id":2,"owner":1,"adj":[]}],
        "waters":[{"id":0,"adj":[1]},{"id":1,"adj":[0]},{"id":2,"adj":[]}],
        "techs":{"extraction":{"requires":[],"offer":1000},"dash":{"requires":[],"offer":1000},"ranged":{"requires":[],"offer":1000},
          "armor":{"requires":[],"offer":1000},"seaTraversal":{"requires":[],"offer":1000},
          "mountainTraversal":{"requires":[],"offer":1000},"flight":{"requires":["seaTraversal","mountainTraversal"],"offer":1000}}}"#);
    let me = Value::parse(r#"{"id":1,"seat":1,"capital":0,"provinces":3,"bank":1000,"ramCapital":{},
        "techs":{"extraction":true,"dash":true,"ranged":true,"seaTraversal":true,"mountainTraversal":true},
        "units":{"total":12,"stalled":0,"queued":0,"cap":20,"byType":{"miner":6,"hauler":4,"freighter":2}},
        "compute":{"cycles":40000,"thinkPeak":12000,"rentMax":0,"rentRate":1},"costs":{"unit":50,"upkeep":100},
        "rts":{"cpu":{"income":40000,"bank":120000,"capacity":1000000,"last":{"upkeep":12000}},
            "logistics":{"depots":[{"p":0,"dock":true,"cycles":0},{"p":1,"dock":true,"cycles":20000},{"p":2,"dock":true,"cycles":1000000}]}}}"#);
    let units = Value::parse(r#"[{"id":107,"owner":1,"type":"freighter","at":0,"to":-1,"doing":"idle","pos":{"p":0,"x":0,"y":0},
        "rts":{"sea":{"ship":true,"water":0,"deck":{"riders":[]}},"cargo":{"cycles":0,"capacity":50000}}}]"#);
    (world, me, units)
}
fn views(values: &[Value]) -> Vec<View> {
    reset_library();
    View::args_json(&Value::Arr(values.to_vec()).to_json())
}
fn ram_pages(orders: &Value) -> Value {
    let mut pages = Value::obj();
    for op in orders.get("ram").items() {
        if matches!(op.get("write"), Value::Num(_)) {
            pages.set(&op.get("write").num().to_string(), obj! {"bytes"=>op.get("bytes").clone(),"t"=>5});
        }
    }
    pages
}
fn maintain(world: &mut Value, me: &mut Value, units: &Value, now: f64) -> Value {
    world.set("seconds", now);
    world.set("units", units.clone());
    let args = views(&[world.clone(), me.clone()]);
    let orders = common::govern(args[0], args[1], 0.4, || obj! {});
    me.set("ramCapital", ram_pages(&orders));
    orders
}

#[test]
fn empty_ship_retires_after_two_minutes_without_progress() {
    let (mut world, mut me, units) = fixture();
    assert!(maintain(&mut world,&mut me,&units,1.0).get("disband").is_empty());
    let orders = maintain(&mut world,&mut me,&units,121.0);
    assert_eq!(orders.get("disband"), &arr![107]);
    assert!(mem::get(views(&[me.get("ramCapital").clone()])[0],245).get("until").num() > 121.0);
}
#[test]
fn movement_resets_the_idle_timer() {
    let (mut world, mut me, mut units) = fixture();
    maintain(&mut world,&mut me,&units,1.0);
    item(&mut units,0).entry("pos").set("x",1.0);
    assert!(maintain(&mut world,&mut me,&units,61.0).get("disband").is_empty());
    assert!(maintain(&mut world,&mut me,&units,121.0).get("disband").is_empty());
    assert_eq!(maintain(&mut world,&mut me,&units,181.0).get("disband"),&arr![107]);
}
#[test]
fn loaded_hired_and_passenger_ships_survive_idle_cleanup() {
    for kind in ["loaded","hired","passenger"] {
        let (mut world, mut me, mut units) = fixture();
        maintain(&mut world,&mut me,&units,1.0);
        let u = item(&mut units,0);
        match kind {
            "loaded" => u.entry("rts").entry("cargo").set("cycles",1),
            "hired" => u.set("hired",obj! {"from"=>2}),
            _ => u.entry("rts").entry("sea").entry("deck").set("riders",arr![12]),
        }
        assert!(maintain(&mut world,&mut me,&units,121.0).get("disband").is_empty(),"{kind}");
    }
}
#[test]
fn freighters_choose_reachable_stock_and_do_not_double_book_it() {
    let (world,me,mut units) = fixture();
    let mut second = units.at(0).clone();second.set("id",114);units.push(second);
    let args=views(&[units,world,me]);
    common::freight(args[0],args[1],args[2]);
    let orders:Vec<Value>=abi::TEST_ORDERS.with(|o|o.take()).iter().map(|o|Value::parse(o)).collect();
    assert_eq!(orders.len(),1,"{orders:?}");
    assert_eq!(orders[0].get("dock").num(),1.0,"larger unreachable dock must not hide a useful route");
    assert_eq!(orders[0].get("cargo").get("amount").num(),20000.0);
}
#[test]
fn loaded_ship_can_use_another_receiving_dock_in_its_sea_basin() {
    let (mut world,me,mut units)=fixture();
    item(world.entry("provinces"),1).set("adj",arr![0]);
    item(world.entry("logistics").entry("docks"),0).set("water",2);
    item(&mut units,0).entry("rts").entry("cargo").set("cycles",5000);
    let args=views(&[units,world,me]);
    common::freight(args[0],args[1],args[2]);
    let orders=abi::TEST_ORDERS.with(|o|o.take());
    assert_eq!(Value::parse(&orders[0]).get("dock").num(),1.0);
}
#[test]
fn arjuna_prioritizes_flight_before_optional_ground_upgrades() {
    let (world,me,_)=fixture();let args=views(&[world,me]);let mut orders=obj!{};
    common::budget(&mut orders,args[0],args[1],&common::Budget{techs:&TECHS,..Default::default()});
    assert_eq!(orders.get("research").str(),"flight");
}
#[test]
fn healthy_recovery_keeps_flight_research_moving() {
    let (world,me,_)=fixture();let args=views(&[world,me]);
    let orders=think(args[0],args[1]);
    assert_eq!(orders.get("research").str(),"flight");
    assert_eq!(orders.get("compute").get("cpu").get("limits").get("research").num(),0.12);
}

#[test]
fn a_quiet_planning_second_preserves_other_lanes_and_raises_a_stale_limit() {
    let (mut world, mut me, _) = fixture();
    world.set("seconds", 101);
    me.entry("rts").entry("cpu").set("limits", obj! {"think"=>18000,"research"=>0.12,"upkeep"=>0.7});
    me.entry("rts").entry("cpu").set("reserve", obj! {"control"=>0.1});
    me.entry("rts").entry("cpu").set("upkeepOrder", arr!["miner","wing"]);
    me.entry("compute").set("thinkPeak",18000);
    let args=views(&[world,me]);
    let orders=common::govern(args[0],args[1],0.4,||panic!("standing plans should continue"));
    let cpu=orders.get("compute").get("cpu");
    assert!(cpu.get("limits").get("think").num()>=32000.0);
    assert_eq!(cpu.get("limits").get("research"),&Value::from(0.12));
    assert_eq!(cpu.get("reserve"),&obj!{"control"=>0.1});
    assert_eq!(cpu.get("upkeepOrder"),&arr!["miner","wing"]);
    assert!(orders.get("ram").is_empty() && orders.get("spawn").is_empty());
}

#[test]
fn distant_workers_do_not_hide_an_open_town_from_army_planning() {
    let args=views(&[obj!{"factions"=>arr![]},obj!{"id"=>0},
        obj!{"owner"=>1,"cores"=>4,"garrison"=>20,"townGarrison"=>0},
        obj!{"owner"=>1,"cores"=>3,"garrison"=>2,"townGarrison"=>2},
        obj!{"owner"=>1,"cores"=>4,"garrison"=>20}]);
    assert!(common::score(args[0],args[1],args[2])>common::score(args[0],args[1],args[3]));
    assert_eq!(common::town_defenders(args[4]),20.0,"older observations keep their fallback");
}
#[test]
fn flight_research_waits_for_recovery_and_preserves_emu_doctrine() {
    for state in ["stalled","poor","costly","emu"] {
        let (world,mut me,_)=fixture();
        match state {
            "stalled"=>me.entry("units").set("stalled",1),
            "poor"=>me.entry("rts").entry("cpu").set("bank",1000),
            "costly"=>me.entry("rts").entry("cpu").entry("last").set("upkeep",30000),
            _=>me.set("seat",8),
        }
        let args=views(&[world,me]);let orders=think(args[0],args[1]);
        assert!(orders.get("research").is_undef(),"{state}");
    }
}

#[test]
fn recruitment_requires_a_reachable_load_and_observes_retirement_cooldown() {
    let (mut world,mut me,_)=fixture();
    world.set("units",arr![]);
    me.entry("units").entry("byType").set("freighter",0);
    me.entry("rts").set("sea",obj!{"ports"=>arr![0]});
    let recruit=|world:&Value,me:&Value| {
        let args=views(&[world.clone(),me.clone()]);
        common::govern(args[0],args[1],0.4,||obj!{}).get("spawn").clone()
    };
    assert!(recruit(&world,&me).is_empty(),"unreachable stock must not recruit a hull");
    item(me.entry("rts").entry("logistics").entry("depots"),1).set("cycles",1000000);
    let spawn=recruit(&world,&me);
    assert_eq!(spawn.at(0).get("type").str(),"freighter");
    assert_eq!(spawn.at(0).get("port").num(),0.0);
    let saved=obj!{"ram"=>mem::put(245,&obj!{"t"=>1,"until"=>300,"ships"=>arr![]},"local",None,4).unwrap()};
    me.set("ramCapital",ram_pages(&saved));
    assert!(recruit(&world,&me).is_empty(),"idle retirement must not cause immediate replacement");
}
#[test]
fn ground_carriers_collect_cargo_from_an_alternate_receiving_dock() {
    let (mut world,me,mut units)=fixture();
    item(world.entry("provinces"),1).set("map","o0".repeat(217));
    item(world.entry("provinces"),0).set("adj",arr![1]);
    item(world.entry("provinces"),1).set("adj",arr![0]);
    item(&mut units,0).set("type","hauler");
    item(&mut units,0).entry("rts").entry("sea").set("ship",false);
    let args=views(&[units,world,me]);
    common::haul_cargo(args[0],args[1],args[2]);
    let orders:Vec<Value>=abi::TEST_ORDERS.with(|o|o.take()).iter().map(|s|Value::parse(s)).collect();
    assert!(orders.iter().any(|o|o.get("cargo").get("load").get("p").num()==1.0),"{orders:?}");
}
