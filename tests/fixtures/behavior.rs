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

#[test]
fn classic_founder_bootstraps_a_miner_with_a_shared_mine_plan() {
    let world = Value::parse(r#"{"seconds":1,"provinces":[{"id":7,"owner":1,"mines":[1],"staff":[0]}]}"#);
    let me = Value::parse(r#"{"id":1,"capital":7,"bank":100,"costs":{"unit":10},"units":{"total":0,"queued":0},"ramCapital":{}}"#);
    let args = views(&[world, me]);
    let orders = common::govern(args[0], args[1], 0.4, || panic!("bootstrap must precede the house planner"));
    assert_eq!(orders.get("spawn").at(0).get("type").str(), "miner");
    let pages = ram_pages(&orders);
    let plan = mem::get(views(&[pages])[0], 0);
    assert_eq!(plan.get("mines").at(0), &arr![7, 0]);
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
    let orders = maintain(&mut world,&mut me,&units,123.0);
    assert_eq!(orders.get("disband"), &arr![107]);
    assert!(mem::get(views(&[me.get("ramCapital").clone()])[0],245).get("until").num() > 123.0);
}
#[test]
fn movement_resets_the_idle_timer() {
    let (mut world, mut me, mut units) = fixture();
    maintain(&mut world,&mut me,&units,1.0);
    item(&mut units,0).entry("pos").set("x",1.0);
    assert!(maintain(&mut world,&mut me,&units,63.0).get("disband").is_empty());
    assert!(maintain(&mut world,&mut me,&units,123.0).get("disband").is_empty());
    assert_eq!(maintain(&mut world,&mut me,&units,183.0).get("disband"),&arr![107]);
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
        assert!(maintain(&mut world,&mut me,&units,123.0).get("disband").is_empty(),"{kind}");
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
fn target_scoring_uses_capture_ring_defenders_and_preserves_old_observations() {
    let args=views(&[obj!{"factions"=>arr![]},obj!{"id"=>0},
        obj!{"owner"=>1,"cores"=>4,"garrison"=>20,"townGarrison"=>0},
        obj!{"owner"=>1,"cores"=>3,"garrison"=>2,"townGarrison"=>2},
        obj!{"owner"=>1,"cores"=>4,"garrison"=>20}]);
    assert!(common::score(args[0],args[1],args[2]) > common::score(args[0],args[1],args[3]),
        "distant workers do not make an open capture ring harder than two local defenders");
    assert_eq!(common::town_defenders(args[4]),20.0,"older observations fall back to the province garrison");
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
    front_case_observed(target_hp, max, stage_hp, rival_hp, 1.0, 5.0)
}
fn front_case_observed(target_hp: f64, max: f64, stage_hp: f64, rival_hp: f64, now: f64, attack: f64) -> (Option<Option<Value>>, Option<Option<Value>>) {
    let (mut world, mut me, _) = fixture();
    world.set("factions", Value::parse(r#"[null,{"id":1,"alive":true},{"id":2,"alive":true}]"#));
    world.set("provinces", Value::parse(r#"[{"id":0,"owner":1,"adj":[4]},{"id":4,"owner":1,"adj":[0,5]},{"id":5,"owner":2,"adj":[4]}]"#));
    world.set("units", Value::parse(&format!(r#"[
        {{"id":1,"owner":1,"type":"line","at":5,"to":-1,"doing":"capture","target":5,"hp":{target_hp},"hpMax":{max}}},
        {{"id":2,"owner":1,"type":"line","at":4,"to":5,"doing":"capture","target":5,"hp":{stage_hp}}},
        {{"id":3,"owner":2,"type":"line","at":5,"to":-1,"doing":"fight","target":5,"hp":{rival_hp}}},
        {{"id":4,"owner":2,"type":"miner","at":5,"to":-1,"doing":"gather","target":5,"hp":50}}]"#)));
    me.set("ramProvince", obj! {"0"=>arr![obj!{}],"4"=>arr![obj!{}]});
    let args = views(&[world.clone(), me.clone()]);
    let mut plan = obj! {"attack"=>5,"stage"=>4};
    common::front_snapshot(&mut plan, args[0], args[1]);
    assert_eq!(plan.get("front"), &arr![5,4,target_hp.floor(),rival_hp.ceil(),stage_hp.floor(),1]);
    let mut departing = world.get("units").at(0).clone();
    departing.set("at", 4);
    let args = views(&[departing, me.clone()]);
    let ops = common::carry(args[0], &Rc::new(plan.clone()), args[1]).expect("a departing unit carries the snapshot");
    let carried = ram_pages(&obj! {"ram"=>ops});
    let mut units = arr![world.get("units").at(0).clone(),world.get("units").at(1).clone()];
    for i in 0..2 { item(&mut units,i).set("ramUnit",carried.clone()); }
    item(&mut units,0).set("at",attack);
    world.set("seconds",now);
    if let Value::Obj(o) = &mut world { o.remove("units"); }
    let args = views(&[units, world, me]);
    assert!(args[1].get("units").is_undef(), "unit callbacks omit the THINK unit census");
    assert_eq!(mem::get(args[0].at(0).get("ramUnit"),93), plan.get("front").clone());
    let core = obj! {"attack"=>attack,"stage"=>4};
    (common::front(args[0].at(0), args[1], args[2], &core), common::front(args[0].at(1), args[1], args[2], &core))
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
fn front_snapshots_expire_and_never_apply_to_a_different_attack() {
    let (ahead, stage) = front_case_observed(3.0, 4.0, 10.0, 20.0, 122.0, 5.0);
    assert!(ahead.is_none() && stage.is_none(), "expired totals cannot change standing orders");
    let (ahead, stage) = front_case_observed(3.0, 4.0, 10.0, 20.0, 1.0, 6.0);
    assert!(ahead.is_none() && stage.is_none(), "a previous attack's totals cannot order retreat or regrouping");
}

#[test]
fn front_uses_the_full_plan_and_rejects_a_wrong_stage_or_future_snapshot() {
    let world = Value::parse(r#"{"seconds":10,"provinces":[{"id":4,"owner":1,"adj":[5]},{"id":5,"owner":2,"adj":[4]}]}"#);
    let me = obj! {"id"=>1,"ramProvince"=>obj! {"4"=>arr![obj!{}]}};
    let unit = obj! {"id"=>1,"owner"=>1,"at"=>5,"to"=>-1,"hp"=>3,"hpMax"=>4,"doing"=>"capture","target"=>5};
    let args = views(&[unit,world,me]);
    let mut plan = obj! {"attack"=>5,"stage"=>4,"front"=>arr![5,4,3,20,10,1]};
    let retreat = common::front(args[0],args[1],args[2],&plan).flatten().expect("the full plan can order retreat");
    assert_eq!(retreat.get("target").num(),4.0);
    plan.set("front",arr![5,0,3,20,10,1]);
    assert!(common::front(args[0],args[1],args[2],&plan).is_none(),"a different stage invalidates the snapshot");
    plan.set("front",arr![5,4,3,20,10,11]);
    assert!(common::front(args[0],args[1],args[2],&plan).is_none(),"future totals cannot change standing orders");
}

#[test]
fn front_snapshots_count_ground_fighters_and_exclude_workers_scouts_ships_and_aircraft() {
    let world = Value::parse(r#"{"seconds":10,"factions":[null,{"id":1,"alive":true},{"id":2,"alive":true},{"id":3,"alive":true}],"units":[
        {"owner":1,"type":"line","at":5,"hp":3.8},
        {"owner":3,"type":"line","at":5,"hp":2},
        {"owner":2,"type":"line","at":5,"hp":20.2},
        {"owner":1,"type":"line","at":4,"hp":10.6},
        {"owner":1,"type":"hauler","at":5,"hp":200},
        {"owner":2,"type":"miner","at":5,"doing":"gather","hp":200},
        {"owner":2,"type":"researcher","at":5,"doing":"study","hp":200},
        {"owner":1,"type":"scout","at":4,"hp":200},
        {"owner":2,"type":"frigate","at":5,"hp":200,"rts":{"sea":{"ship":true}}},
        {"owner":2,"type":"wing","at":5,"hp":200,"build":{"wings":1}}]}"#);
    let me = obj! {"id"=>1,"allies"=>arr![3]};
    let args = views(&[world,me]);
    let mut plan = obj! {"attack"=>5,"stage"=>4};
    common::front_snapshot(&mut plan,args[0],args[1]);
    assert_eq!(plan.get("front"),&arr![5,4,5,21,10,10]);
    assert!(plan.get("front").to_json().len()<=62,"the snapshot must fit one unit RAM page");
}
fn breach_fixture(force: usize, guards: usize, neutral: bool) -> (Value, Value) {
    let mut provinces = arr![
        obj! {"id"=>0,"owner"=>1,"adj"=>arr![1,2],"hops"=>0,"units"=>obj! {"1"=>force as f64}},
        obj! {"id"=>1,"owner"=>2,"adj"=>arr![0],"capital"=>true,"visible"=>true,
            "garrison"=>guards as f64,"townGarrison"=>guards as f64,"units"=>obj! {"2"=>guards as f64}},
    ];
    if neutral { provinces.push(obj! {"id"=>2,"owner"=>-1,"adj"=>arr![0],"visible"=>true,"garrison"=>0}); }
    let mut units = Value::arr();
    for i in 0..force { units.push(obj! {"id"=>i as f64+1.0,"owner"=>1,"type"=>"line","at"=>0,"to"=>-1,
        "doing"=>"idle","target"=>-1,"hp"=>5,"build"=>obj! {"strength"=>2}}); }
    for i in 0..guards { units.push(obj! {"id"=>i as f64+100.0,"owner"=>2,"type"=>"line","at"=>1,"to"=>-1,
        "doing"=>"fight","target"=>1,"hp"=>5,"build"=>obj! {"strength"=>2}}); }
    let world = obj! {"seconds"=>100,"provinces"=>provinces,"units"=>units,
        "factions"=>Value::parse(r#"[null,{"id":1,"alive":true},{"id":2,"alive":true}]"#)};
    let me = obj! {"id"=>1,"capital"=>0,"allies"=>arr![],"contracts"=>arr![],"ramCapital"=>obj! {},
        "units"=>obj! {"total"=>force as f64+4.0,"byType"=>obj! {"miner"=>4},"doing"=>obj! {"gather"=>4}}};
    (world,me)
}

fn breach_plan(world: &Value, me: &Value) -> Value {
    let args = views(&[world.clone(),me.clone()]);
    let targets = realm::frontier(args[0],args[1]);
    common::survey(args[0],args[1],&targets,2.0)
}

#[test]
fn an_assault_reaches_a_defended_capital_before_capture_odds_are_met() {
    let (world,me) = breach_fixture(14,7,false);
    let plan = breach_plan(&world,&me);
    assert_eq!(plan.get("attack").num(),1.0,"a plausible fight must remain an objective");
    assert_eq!(plan.get("need").num(),13.0,"combat needs a 50% advantage and two reserves, rather than 23 capturers");
    let own = Value::Arr(world.get("units").items().into_iter().filter(|u|u.get("owner").num()==1.0).cloned().collect());
    let mut callback_world = world.clone();
    if let Value::Obj(o) = &mut callback_world {o.remove("units");}
    let args = views(&[own,callback_world,me]);
    let units:Vec<View>=args[0].iter().collect();
    let plans:Vec<Rc<Value>>=units.iter().map(|_|Rc::new(plan.clone())).collect();
    let orders=common::muster(&units,args[1],args[2],"capture",&plans);
    assert!(orders.iter().any(|o|o.get("do").str()=="capture"&&o.get("target").num()==1.0),"the assault must leave its stage: {orders:?}");
}

#[test]
fn a_weak_army_waits_and_safe_expansion_keeps_priority_over_an_assault() {
    let (world,me)=breach_fixture(10,7,false);
    assert_eq!(breach_plan(&world,&me).get("attack").num(),-1.0,"a smaller army cannot afford this fight");
    let (world,me)=breach_fixture(5,3,false);
    assert_eq!(breach_plan(&world,&me).get("attack").num(),-1.0,"an assault requires at least six ground fighters");
    let (world,me)=breach_fixture(14,7,true);
    let plan=breach_plan(&world,&me);
    assert_eq!(plan.get("attack").num(),2.0,"ordinary capturable land has priority");
    assert_eq!(plan.get("need").num(),2.0);
}

#[test]
fn the_same_capital_returns_to_normal_capture_planning_when_defenders_fall() {
    let (world,me)=breach_fixture(14,2,false);
    let plan=breach_plan(&world,&me);
    assert_eq!(plan.get("attack").num(),1.0);
    assert_eq!(plan.get("need").num(),8.0,"ordinary three-to-one capture odds replace the assault threshold");
}

#[test]
fn assaults_respect_passage_and_do_not_count_aircraft_as_ground_replacements() {
    let (world,mut me)=breach_fixture(14,7,false);
    me.set("contracts",arr![obj! {"with"=>2,"get"=>obj! {"passage"=>true}}]);
    assert_eq!(breach_plan(&world,&me).get("attack").num(),-1.0,"a passage host stays protected");
    let (mut world,me)=breach_fixture(14,7,false);
    item(world.entry("provinces"),1).set("visible",false);
    assert_eq!(breach_plan(&world,&me).get("attack").num(),-1.0,"stale garrison observations cannot justify an assault");
    let (mut world,me)=breach_fixture(14,7,false);
    for i in 4..14 { item(world.entry("units"),i).entry("build").set("wings",1); }
    assert_eq!(breach_plan(&world,&me).get("attack").num(),-1.0,"ten aircraft cannot replace missing capturers");
}

#[test]
fn a_plausible_assault_keeps_its_existing_target() {
    let (mut world,mut me)=breach_fixture(14,7,false);
    world.entry("provinces").push(obj! {"id"=>2,"owner"=>3,"adj"=>arr![0],"capital"=>true,"visible"=>true,
        "garrison"=>6,"townGarrison"=>6,"units"=>obj! {"3"=>6}});
    world.entry("factions").push(obj! {"id"=>3,"alive"=>true});
    for i in 0..6 {world.entry("units").push(obj! {"id"=>i as f64+200.0,"owner"=>3,"type"=>"line","at"=>2,"to"=>-1,
        "doing"=>"fight","target"=>2,"hp"=>5,"build"=>obj! {"strength"=>2}});}
    me.set("ramCapital",ram_pages(&obj! {"ram"=>mem::put(0,&obj! {"attack"=>2},"local",None,1).unwrap()}));
    let plan=breach_plan(&world,&me);
    assert_eq!(plan.get("attack").num(),2.0,"fallback assaults use the existing sticky target selection");
    assert_eq!(plan.get("need").num(),11.0);
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
    // A newly short bank enters recovery without waiting for a scheduled review.
    let mut short = me.clone();
    short.entry("rts").entry("cpu").set("bank", 120000);
    let args = views(&[world.clone(), short.clone()]);
    assert!(!common::govern(args[0], args[1], 0.4, || obj! {}).is_empty(), "a newly short bank answers immediately");
    let mut recovering = me.clone();
    recovering.set("ramCapital", ram_pages(&obj! {"ram"=>mem::put(244,&obj!{"r"=>true,"s"=>0},"local",None,1).unwrap()}));
    assert!(!think_at(&mut world, &recovering, 4.0), "standing recovery leaves income for hauling");
    assert!(think_at(&mut world, &recovering, 6.0), "recovery is reviewed on the next full think");
    me.entry("units").set("stalled", 1);
    me.entry("rts").entry("cpu").set("limits",obj!{"upkeep"=>5000});
    world.set("seconds",4);
    let args=views(&[world.clone(),me]);let mut ran=false;
    let out=common::govern(args[0],args[1],0.4,||{ran=true;obj!{}});
    assert!(!ran,"a funded upkeep stall preserves the three-second strategy cadence");
    assert!(out.get("compute").get("cpu").get("limits").get("upkeep").num()>5000.0,
        "the intervening call raises upkeep to the estimated full bill");
}

#[test]
fn funded_recovery_resumes_below_current_income_and_clears_its_marker() {
    let (mut world,mut me,units)=fixture();
    world.set("seconds",6);world.set("units",units);
    me.entry("compute").set("thinkPeak",32000);
    me.set("ramCapital",ram_pages(&obj! {"ram"=>mem::put(244,&obj! {"r"=>true,"s"=>0},"local",None,1).unwrap()}));
    let cpu=me.entry("rts").entry("cpu");
    cpu.set("income",146177);cpu.set("bank",18600000);cpu.set("capacity",20000000);
    cpu.set("limits",obj! {"think"=>127693});
    cpu.set("available",obj! {"think"=>127000});
    cpu.set("last",obj! {"upkeep"=>62000});
    let args=views(&[world,me]);let mut ran=false;
    let orders=common::govern(args[0],args[1],0.4,|| {
        ran=true;
        obj! {"compute"=>obj! {"cpu"=>obj! {"limits"=>obj! {"think"=>127693,"fabrication"=>20000}}}}
    });
    assert!(ran,"a healthy funded callback reaches house admission even when its limit is below current cargo income");
    assert_ne!(orders.get("memo").str(),"Recovering production");
    assert!(orders.get("compute").get("cpu").get("limits").get("fabrication").num()>0.0);
    assert!(orders.get("ram").items().iter().any(|op|op.get("free").num_or(-1.0)==244.0),"the successful strategy clears the old recovery marker");
}

#[test]
fn recovery_raises_an_exhausted_allowance_before_retrying_the_house() {
    for (income,remaining,peak) in [(49534.0,23100.0,23000.0),(41140.0,23900.0,23979.0)] {
        let (mut world,mut me,units)=fixture();
        world.set("seconds",6);world.set("units",units);
        me.entry("compute").set("thinkPeak",peak);
        me.set("ramCapital",ram_pages(&obj! {"ram"=>mem::put(244,&obj! {"r"=>true,"s"=>0},"local",None,1).unwrap()}));
        let cpu=me.entry("rts").entry("cpu");
        cpu.set("income",income);cpu.set("bank",7000000);cpu.set("capacity",20000000);
        cpu.set("limits",obj! {"think"=>remaining});cpu.set("available",obj! {"think"=>remaining});
        let args=views(&[world.clone(),me.clone()]);
        let orders=common::govern(args[0],args[1],0.4,|| panic!("an exhausted allowance must be raised before retrying the house"));
        assert_eq!(orders.get("memo").str(),"Recovering production");
        let next=orders.get("compute").get("cpu").get("limits").get("think").num();
        assert!(next>remaining && next<=income*1.25,"recovery must fund a larger callback within the hard spend cap");
        assert!(!orders.get("ram").items().iter().any(|op|op.get("free").num_or(-1.0)==244.0));

        me.set("ramCapital",ram_pages(&orders));
        let cpu=me.entry("rts").entry("cpu");
        cpu.set("limits",orders.get("compute").get("cpu").get("limits").clone());
        cpu.set("available",obj! {"think"=>next});
        world.set("seconds",9);
        let args=views(&[world,me]);let mut ran=false;
        let resumed=common::govern(args[0],args[1],0.4,|| {ran=true;obj! {}});
        assert!(ran,"the returned allowance must permit another house attempt");
        assert!(resumed.get("ram").items().iter().any(|op|op.get("free").num_or(-1.0)==244.0));
    }
}

#[test]
fn recovery_admits_a_funded_lean_house_after_income_falls_below_its_peak() {
    let (mut world,mut me,units)=fixture();
    world.set("seconds",6);world.set("units",units);
    me.entry("compute").set("thinkPeak",32000);
    me.set("ramCapital",ram_pages(&obj! {"ram"=>mem::put(244,&obj! {"r"=>true,"s"=>0},"local",None,1).unwrap()}));
    let cpu=me.entry("rts").entry("cpu");
    cpu.set("income",19000);cpu.set("bank",7000000);cpu.set("capacity",20000000);
    cpu.set("limits",obj! {"think"=>23750});cpu.set("available",obj! {"think"=>23750});
    let args=views(&[world,me]);let mut ran=false;
    let orders=common::govern(args[0],args[1],0.4,|| {ran=true;obj! {}});
    assert!(ran,"a lower income must not make admission exceed a funded lean callback");
    assert!(orders.get("ram").items().iter().any(|op|op.get("free").num_or(-1.0)==244.0));
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
