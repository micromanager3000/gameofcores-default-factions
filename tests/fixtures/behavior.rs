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

/// Unit 1 in the target (5), hurt unless `max` is its hp, unit 2 at the stage (4) on its way there, and a rival
/// soldier and miner in the target.
fn front_case(target_hp: f64, stage_hp: f64, rival_hp: f64) -> (Option<Option<Value>>, Option<Option<Value>>) {
    front_case_max(target_hp, target_hp + 1.0, stage_hp, rival_hp)
}
fn front_case_max(target_hp: f64, max: f64, stage_hp: f64, rival_hp: f64) -> (Option<Option<Value>>, Option<Option<Value>>) {
    let (mut world, mut me, _) = fixture();
    world.set("provinces", Value::parse(r#"[{"id":0,"owner":1,"adj":[4]},{"id":4,"owner":1,"adj":[0,5]},{"id":5,"owner":2,"adj":[4]}]"#));
    world.set("units", Value::parse(&format!(r#"[
        {{"id":1,"owner":1,"type":"line","at":5,"to":-1,"doing":"capture","target":5,"hp":{target_hp},"hpMax":{max}}},
        {{"id":2,"owner":1,"type":"line","at":4,"to":5,"doing":"capture","target":5,"hp":{stage_hp}}},
        {{"id":3,"owner":2,"type":"line","at":5,"to":-1,"doing":"fight","target":5,"hp":{rival_hp}}},
        {{"id":4,"owner":2,"type":"miner","at":5,"to":-1,"doing":"gather","target":5,"hp":50}}]"#)));
    me.set("ramProvince", obj! {"0"=>arr![obj!{}],"4"=>arr![obj!{}]});
    let args = views(&[world, me]);
    let plan = obj! {"attack"=>5,"stage"=>4};
    let units: Vec<View> = args[0].get("units").iter().collect();
    (common::front(units[0], args[0], args[1], &plan), common::front(units[1], args[0], args[1], &plan))
}
#[test]
fn an_outweighed_army_falls_back_and_goes_back_in_as_one_wave() {
    let to_stage = |o: &Option<Option<Value>>| o.as_ref().and_then(|o| o.as_ref()).is_some_and(|o| o.get("do").str() == "fight" && o.get("target").num() == 4.0);
    // 3 hp against 20: the hurt unit in the target falls back, the one at the stage (10 + 3 < 1.15 × 20) waits there
    let (ahead, stage) = front_case(3.0, 10.0, 20.0);
    assert!(to_stage(&ahead), "{ahead:?}");
    assert!(to_stage(&stage), "{stage:?}");
    // a unit that hasn't been hurt yet fights on
    let (ahead, _) = front_case_max(3.0, 3.0, 10.0, 20.0);
    assert!(ahead.is_none(), "{ahead:?}");
    // with none of yours in the target, a first attack goes in on the plan
    let (_, stage) = front_case(0.0, 10.0, 20.0);
    assert!(stage.is_none(), "{stage:?}");
    // a wave at the stage that outweighs the rivals goes in on the plan
    let (_, stage) = front_case(3.0, 30.0, 20.0);
    assert!(stage.is_none(), "{stage:?}");
    // an even fight stands: 15 × 1.6 outweighs 20, and workers don't count
    let (ahead, stage) = front_case(15.0, 10.0, 20.0);
    assert!(ahead.is_none() && stage.is_none(), "{ahead:?} {stage:?}");
}
#[test]
fn a_full_think_runs_every_third_second_while_nothing_presses() {
    let (mut world, mut me, units) = fixture();
    me.entry("rts").entry("cpu").set("bank", 400000);
    world.set("units", units);
    let think_at = |world: &mut Value, me: &Value, now: f64| {
        world.set("seconds", now);
        let args = views(&[world.clone(), me.clone()]);
        let mut ran = false;
        common::govern(args[0], args[1], 0.4, || { ran = true; obj! {} });
        ran
    };
    assert!(think_at(&mut world, &me, 3.0));
    assert!(!think_at(&mut world, &me, 4.0));
    // under four seconds of income in the bank, the cheap recovery answers in place of the strategy, every second
    let mut short = me.clone();
    short.entry("rts").entry("cpu").set("bank", 120000);
    let args = views(&[world.clone(), short.clone()]);
    assert!(!common::govern(args[0], args[1], 0.4, || obj! {}).is_empty(), "a short bank answers every second");
    let mut recovering = me.clone();
    recovering.set("ramCapital", ram_pages(&obj! {"ram"=>mem::put(244,&obj!{"r"=>true,"s"=>0},"local",None,1).unwrap()}));
    assert!(think_at(&mut world, &recovering, 4.0), "a recovering house thinks every second");
    me.entry("units").set("stalled", 1);
    assert!(think_at(&mut world, &me, 4.0), "a stalled unit brings the think back every second");
}

#[test]
fn timed_jobs_come_due_on_full_thinks_for_every_house() {
    // a job every 8 s comes due every 9 s on a full-think second, whatever the stagger
    for id in 0..12 {
        let due: Vec<u32> = (1..=90).filter(|&t| common::every(t as f64, 8.0, id as f64)).collect();
        assert_eq!(due.len(), 10, "id {id}: {due:?}");
        assert!(due.iter().all(|t| t % 3 == 0) && due.windows(2).all(|w| w[1] - w[0] == 9), "id {id}: {due:?}");
    }
    // a 30 s job comes due once in 30 s for every house, and a job shorter than the spacing on every full think
    assert!((0..12).all(|id| (1..=30).filter(|&t| common::every(t as f64, 30.0, id as f64)).count() == 1));
    assert!((1..=9).filter(|&t| common::every(t as f64, 2.0, 0.0)).eq([3, 6, 9]));
}
