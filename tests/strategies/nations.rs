#[test]
fn frontline_prefers_an_affordable_rival_capital_from_foreign_land() {
    let world = Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true,"capital":0},{"id":2,"alive":true,"capital":1},{"id":3,"alive":true,"capital":2}],"provinces":[
        {"id":0,"owner":1,"adj":[1,2],"hops":0},
        {"id":1,"owner":2,"capital":true,"cores":1,"adj":[0],"townGarrison":0},
        {"id":2,"owner":3,"cores":4,"adj":[0],"townGarrison":0}],"units":[]}"#);
    let me = obj! { "id"=>1, "capital"=>0, "allies"=>arr![], "contracts"=>arr![],
        "units"=>obj! { "total"=>8, "byType"=>obj! {}, "doing"=>obj! {} }, "ramCapital"=>obj! {} };
    let args = views(&[world, me]);
    let ranked = frontline(args[0], args[1]);
    assert_eq!(ranked.first().map(|p| p.get("id").num()), Some(1.0));
}

#[test]
fn frontline_keeps_pressing_the_recorded_campaign_when_scores_are_close() {
    let campaign = pages(&obj! { "campaign"=>3 });
    let world = Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true,"capital":0},{"id":2,"alive":true,"capital":1},{"id":3,"alive":true,"capital":2}],"provinces":[
        {"id":0,"owner":1,"adj":[1,2],"hops":0},
        {"id":1,"owner":2,"cores":5,"adj":[0],"townGarrison":0},
        {"id":2,"owner":3,"cores":4,"adj":[0],"townGarrison":0}],"units":[]}"#);
    let me = obj! { "id"=>1, "capital"=>0, "allies"=>arr![], "contracts"=>arr![],
        "units"=>obj! { "total"=>8, "byType"=>obj! {}, "doing"=>obj! {} }, "ramCapital"=>campaign };
    let args = views(&[world, me]);
    let ranked = frontline(args[0], args[1]);
    assert_eq!(ranked.first().map(|p| p.get("id").num()), Some(2.0));
}

#[test]
fn sentry_advances_with_a_committed_capture_wave() {
    let plan = pages(&obj! { "attack"=>1, "stage"=>0, "defend"=>-1, "home"=>0 });
    let world = Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true,"capital":0},{"id":2,"alive":true,"capital":1}],"provinces":[
        {"id":0,"owner":1,"adj":[1]},{"id":1,"owner":2,"adj":[0],"units":{"1":2}}],"units":[]}"#);
    let me = obj! { "id"=>1, "capital"=>0, "allies"=>arr![], "ramProvince"=>obj! { "0"=>plan } };
    let units = Value::parse(r#"[{"id":10,"owner":1,"type":"sentry","at":0,"to":-1,"doing":"idle","build":{}}]"#);
    let args = views(&[units, world, me]);
    sentry(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("target").num(), 1.0, "{orders:?}");
    assert_eq!(orders[0].get("do").str(), "capture", "{orders:?}");
}

#[test]
fn spire_admission_keeps_room_for_a_whole_body_and_its_ram_copy() {
    let mut budget = SentryBudget { next: 2400.0, read: Vec::new() };
    assert!(sentry_has_runway(5824.0,budget.next,300.0),"the measured call can admit its first body");
    budget.spent(2912.0);
    assert!(!sentry_has_runway(5824.0-2912.0,budget.next,0.0),"the remaining half cannot begin another plan/route/carry pass");
    assert!(!sentry_has_runway(5824.0,2400.0,2000.0),"a large cold plan is charged before decoding it");
    assert!(sentry_has_runway(12000.0,budget.next,300.0),"a later funded call can accept another body");
}

#[test]
fn spires_advance_and_carry_combat_totals_when_the_callback_has_room() {
    let plan=pages(&obj! {"attack"=>1,"stage"=>0,"front"=>arr![1,0,0,20,10,1]});
    let world=Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true},{"id":2,"alive":true}],"provinces":[
        {"id":0,"owner":1,"adj":[1]},{"id":1,"owner":2,"adj":[0],"units":{"1":2}}]}"#);
    let me=obj! {"id"=>1,"capital"=>0,"allies"=>arr![],"ramProvince"=>obj! {"0"=>plan},
        "rts"=>obj! {"cpu"=>obj! {"available"=>obj! {"control"=>100000}}}};
    let units=Value::parse(r#"[{"id":10,"owner":1,"type":"spire","at":0,"to":-1,"doing":"idle","build":{}},
        {"id":11,"owner":1,"type":"spire","at":0,"to":-1,"doing":"idle","build":{}}]"#);
    let args=views(&[units,world,me]);
    spire(args[0],args[1],args[2]);
    let orders=emitted();
    assert_eq!(orders.len(),2,"{orders:?}");
    for order in orders {
        assert_eq!(order.get("target").num(),1.0);
        assert_eq!(order.get("do").str(),"fight");
        assert!(order.get("ram").items().iter().any(|op|op.get("write").num_or(-1.0)==93.0),"spire orders include the combat snapshot copy");
    }
}

fn planner_fixture() -> Value {
    Value::parse(r#"{"id":1,"units":{"total":22,"stalled":0},"compute":{"thinkPeak":31893,"used":{"think":31893}},
        "rts":{"cpu":{"income":106000,"bank":11000000,"spendCap":132500,
        "limits":{"think":32000,"control":50000,"fabrication":20000,"research":0.3,"upkeep":0.72},
        "reserve":{"think":3500,"reflex":7000},"upkeepOrder":["miner","sentry","spire"]}}}"#)
}

fn planner_allowance(policy: &Value, income: f64) -> f64 {
    let value=policy.get("limits").get("think").num();
    if value<=1.0 {value*income} else {value}
}

#[test]
fn exhausted_planner_repairs_its_next_allowance_before_reentering_expensive_work() {
    let mut me=planner_fixture();
    let args=views(&[me.clone()]);
    let PlannerAdmission::Raise(orders)=planner_admission(View::undefined(),args[0],31893.0) else { panic!("a funded exhausted planner repairs its allowance") };
    let policy=orders.get("compute").get("cpu");
    let next=planner_allowance(policy,106000.0);
    assert!(next>32000.0*1.2+4000.0,"exhaustion is a lower bound with a further invocation margin");
    assert!(next<=132500.0);
    assert_eq!(policy.get("limits").get("upkeep").num(),0.72);
    assert_eq!(policy.get("reserve"),me.get("rts").get("cpu").get("reserve"));
    me.entry("rts").entry("cpu").set("limits",policy.get("limits").clone());
    let args=views(&[me]);
    assert!(matches!(planner_admission(View::undefined(),args[0],next-3000.0),PlannerAdmission::Full),"the outgoing policy must admit the following call, including wrapper costs");
}

#[test]
fn planner_repair_respects_hard_caps_without_creating_a_recovery_loop() {
    let mut me=planner_fixture();
    me.entry("rts").entry("cpu").set("spendCap",45000);
    let args=views(&[me.clone()]);
    let PlannerAdmission::Raise(orders)=planner_admission(View::undefined(),args[0],31893.0) else { panic!("the hard cap still permits a useful allowance increase") };
    let policy=orders.get("compute").get("cpu");
    let next=planner_allowance(policy,106000.0);
    assert!(next<=45000.0);
    me.entry("rts").entry("cpu").set("limits",policy.get("limits").clone());
    let args=views(&[me]);
    assert!(matches!(planner_admission(View::undefined(),args[0],next-3000.0),PlannerAdmission::Lean),"a binding cap selects local planning rather than silently admitting the full pass");
}

#[test]
fn a_low_income_planner_keeps_its_viable_lean_admission() {
    let mut me=planner_fixture();
    me.entry("units").set("total",8);
    me.entry("compute").set("thinkPeak",10000);
    me.entry("compute").entry("used").set("think",10000);
    let cpu=me.entry("rts").entry("cpu");
    cpu.set("income",20000);cpu.set("spendCap",25000);cpu.entry("limits").set("think",20000);
    let args=views(&[me]);
    assert!(matches!(planner_admission(View::undefined(),args[0],17000.0),PlannerAdmission::Full),"a cheap 15k planner need not wait for a large-world allowance");
}

#[test]
fn a_cheap_recent_peak_does_not_admit_a_large_full_map_or_loop_above_the_code_cap() {
    let world=obj! {"provinces"=>vec![Value::Null;35],"units"=>vec![Value::Null;57]};
    let mut me=planner_fixture();
    me.entry("units").set("total",16);
    me.entry("compute").set("thinkPeak",6694);
    me.entry("compute").entry("used").set("think",6694);
    let cpu=me.entry("rts").entry("cpu");
    cpu.set("income",37453);cpu.set("bank",325256);cpu.set("spendCap",46816);
    cpu.entry("limits").set("think",21483);cpu.set("reserve",obj! {});
    let args=views(&[world.clone(),me.clone()]);
    let PlannerAdmission::Raise(orders)=planner_admission(args[0],args[1],21419.0) else {panic!("the captured cheap peak must repair the full-map allowance")};
    let next=planner_allowance(orders.get("compute").get("cpu"),37453.0);
    assert!(next>27700.0 && next<=46816.0);
    let cpu=me.entry("rts").entry("cpu");
    cpu.set("income",20000);cpu.set("spendCap",25000);
    let args=views(&[world,me]);
    assert!(matches!(planner_admission(args[0],args[1],25000.0),PlannerAdmission::Lean),"an impossible full-map allowance keeps the local planner available");
}

#[test]
fn a_large_merchant_map_funds_its_survey_and_keeps_the_next_full_allowance() {
    let world=obj! {"provinces"=>vec![Value::Null;44],"units"=>vec![Value::Null;74]};
    let mut me=planner_fixture();
    me.entry("units").set("total",18);
    me.entry("compute").set("thinkPeak",3887);
    me.entry("compute").entry("used").set("think",224);
    let cpu=me.entry("rts").entry("cpu");
    cpu.set("income",73033);cpu.set("bank",1733263);cpu.set("spendCap",91291);
    cpu.entry("limits").set("think",39347);cpu.set("available",obj! {"think"=>39226});
    with_cycle_reserve(5000.0,|| {
        let args=views(&[world.clone(),me.clone()]);
        let PlannerAdmission::Raise(orders)=planner_admission(args[0],args[1],32558.0) else {
            panic!("a cheap recent peak cannot admit the captured merchant map on 39k physical cycles");
        };
        let next=planner_allowance(orders.get("compute").get("cpu"),73033.0);
        me.entry("rts").entry("cpu").entry("limits").set("think",next+cycles_reserved());
        me.entry("rts").entry("cpu").set("available",obj! {"think"=>next+cycles_reserved()-120.0});
        let args=views(&[world,me]);
        assert!(matches!(planner_admission(args[0],args[1],next-2500.0),PlannerAdmission::Full));
        let policy=cpu_policy(args[0],args[1],0.0,false);
        assert!(planner_allowance(&policy,73033.0)>=planner_map_need(args[0],args[1])+6000.0,
            "a completed planner retains the allowance needed for the next map and wrapper");
    });
}

#[test]
fn planner_repair_leaves_other_lanes_reserved_cycles_in_the_shared_bank() {
    let mut me=planner_fixture();
    let cpu=me.entry("rts").entry("cpu");
    cpu.set("bank",240000);
    cpu.set("reserve",obj! {"think"=>3500,"control"=>190000});
    let args=views(&[me]);
    let PlannerAdmission::Raise(orders)=planner_admission(View::undefined(),args[0],31893.0) else { panic!("the remaining shared bank can fund one larger planner call") };
    let next=planner_allowance(orders.get("compute").get("cpu"),106000.0);
    assert!(next<=240000.0-190000.0-4000.0,"other lanes' reserves and the repair call are funded first");
}

#[test]
fn a_cap_below_observed_work_selects_lean_even_with_a_larger_configured_limit() {
    let mut me=planner_fixture();
    me.entry("compute").set("thinkPeak",114494);
    me.entry("rts").entry("cpu").set("spendCap",124276);
    me.entry("rts").entry("cpu").entry("limits").set("think",162588);
    me.entry("rts").entry("cpu").set("available",obj! {"think"=>119418});
    let args=views(&[me]);
    assert!(matches!(planner_admission(View::undefined(),args[0],119000.0),PlannerAdmission::Lean));
    assert!(matches!(planner_admission(View::undefined(),args[0],1.0e9),PlannerAdmission::Lean),"the public allowance also bounds a synthetic native callback");
}

#[test]
fn capped_strategy_refreshes_a_local_plan_recruits_and_researches_without_global_work() {
    let world=Value::parse(r#"{"seconds":600,"factions":[null,{"id":1,"alive":true},{"id":2,"alive":true,"techs":["turbines"]}],
        "provinces":[{"id":0,"owner":1,"adj":[1,2],"hops":0,"mines":[2],"staff":[2],"units":{"1":8}},
          {"id":1,"owner":2,"visible":true,"cores":4,"name":"Gate","adj":[0],"townGarrison":1,"garrison":1,"units":{"2":1}},
          {"id":2,"owner":-1,"visible":true,"cores":1,"adj":[0],"townGarrison":0}],
        "techs":{"turbines":{"offer":1000,"requires":["armor"]}},
        "units":[{"id":99,"owner":2,"air":true,"at":1,"doing":"gather","build":{"wings":1,"range":2}}]}"#);
    let mut me=planner_fixture();
    me.set("capital",0);me.set("provinces",1);me.set("bank",10000);me.set("ramCapital",obj! {});
    me.set("costs",obj! {"unit"=>100,"upkeep"=>100});me.set("contracts",arr![]);me.set("allies",arr![]);
    me.set("techs",obj! {"extraction"=>true,"seaTraversal"=>true,"armor"=>true,"ranged"=>true,
        "dash"=>true,"cloak"=>true,"antiAir"=>true,"scan"=>true,"flight"=>true});
    me.entry("units").set("cap",40);me.entry("units").set("queued",0);me.entry("units").set("spawned",22);
    me.entry("units").set("byType",obj! {"miner"=>6,"shade"=>4,"sentry"=>1,"kite"=>6});
    me.entry("units").set("doing",obj! {"gather"=>6});
    me.entry("compute").set("thinkPeak",80000);me.entry("compute").set("cycles",50000);
    let cpu=me.entry("rts").entry("cpu");cpu.set("spendCap",62500);cpu.set("capacity",1000000);
    cpu.entry("limits").set("think",120000);cpu.set("available",obj! {"think"=>60000,"control"=>50000,"fabrication"=>50000});
    me.entry("rts").set("bank",1000000);
    let args=views(&[world,me.clone()]);
    let orders=strategy(args[0],args[1]);
    let mut ram=Value::obj();
    for op in orders.get("ram").items() {
        if matches!(op.get("write"),Value::Num(_)) {ram.set(&op.get("write").num().to_string(),obj! {"bytes"=>op.get("bytes").clone(),"t"=>600});}
    }
    let saved=views(&[ram]);
    let plan=mem::get(saved[0],0);
    assert_eq!(plan.get("attack").num(),1.0,"the lean pass publishes a new legal local frontier");
    assert_eq!(plan.get("stage").num(),0.0);
    assert!(plan.get("field").is_undef() && plan.get("prey").is_undef() && plan.get("front").is_undef());
    assert_eq!(orders.get("spawn").at(0).get("type").str(),"shade","the unseen global air census cannot trigger an AA recruit");
    assert_eq!(orders.get("research").str(),"turbines","basic research continues even when another faction sells the tech");
    assert!(orders.get("compute").get("cpu").get("limits").get("fabrication").num()>0.0);
    for key in ["send","accept","hire","bribe","tolls","rent"] {assert!(orders.get(key).is_undef(),"lean planning skips {key}");}
}

#[test]
fn lean_frontier_keeps_allies_passage_hosts_and_leased_land_safe() {
    let world=Value::parse(r#"{"seconds":6,"factions":[null,{"id":1,"alive":true},{"id":2,"alive":true},{"id":3,"alive":true},{"id":4,"alive":true}],
        "provinces":[{"id":0,"owner":1,"adj":[1,2,3,4],"units":{},"mines":[1],"staff":[0]},
          {"id":1,"owner":2,"visible":true,"cores":10},{"id":2,"owner":3,"visible":true,"cores":10},
          {"id":3,"owner":4,"visible":true,"cores":10,"leased":[{"to":1}]},
          {"id":4,"owner":-1,"visible":true,"cores":1}]}"#);
    let me=obj! {"id"=>1,"capital"=>0,"allies"=>arr![2],"contracts"=>arr![obj! {"with"=>3,"get"=>obj! {"passage"=>true}}],
        "units"=>obj! {"total"=>10},"ramCapital"=>obj! {}};
    let args=views(&[world,me]);let (plan,room)=lean_plan(args[0],args[1]);
    assert_eq!(plan.get("attack").num(),4.0);
    assert_eq!(room,2.0);
    assert_eq!(plan.get("mines").len(),2);
}

#[test]
fn lean_defense_keeps_the_threatened_capital_ahead_of_a_larger_remote_invasion() {
    for capital_threat in [1,0] {
        let world=obj! {"seconds"=>6,"factions"=>arr![Value::Null,obj! {"id"=>1,"alive"=>true},obj! {"id"=>2,"alive"=>true}],
            "provinces"=>arr![obj! {"id"=>0,"owner"=>1,"adj"=>arr![1],"units"=>obj! {"2"=>capital_threat}},
                obj! {"id"=>1,"owner"=>1,"adj"=>arr![0],"units"=>obj! {"2"=>2}}]};
        let me=obj! {"id"=>1,"capital"=>0,"allies"=>arr![],"contracts"=>arr![],"ramCapital"=>obj! {}};
        let args=views(&[world,me]);let (plan,_)=lean_plan(args[0],args[1]);
        assert_eq!(plan.get("defend").num(),if capital_threat>0 {0.0} else {1.0},
            "one invader at the capital outranks two elsewhere; without a capital threat the remote invasion is defended");
        assert_eq!(plan.get("threat").num(),if capital_threat>0 {1.0} else {2.0});
    }
}

fn naval_rank_case(waters: usize, remaining: f64, land_border: bool) -> bool {
    let mut sea = Value::arr();
    for id in 0..waters {
        sea.push(obj! {"id"=>id,"adj"=>if id == 0 {arr![1]} else if id == 1 {arr![0]} else {arr![]},
            "coast"=>if id == 0 {arr![0]} else if id == 1 {arr![1]} else {arr![]} });
    }
    let world = obj! {"waters"=>sea,"provinces"=>arr![
        obj! {"id"=>0,"owner"=>7},
        obj! {"id"=>1,"owner"=>-1,"adj"=>if land_border {arr![0]} else {arr![]},"routes"=>obj! {"sea"=>arr![0],"air"=>arr![],"pass"=>arr![]}}],
        "units"=>arr![obj! {"owner"=>7,"rts"=>obj! {"sea"=>obj! {"ship"=>true,"water"=>0}}}]};
    let args = views(&[world,obj! {"id"=>7,"rts"=>obj! {"cpu"=>obj! {"income"=>40000}}}]);
    rank_hold(args[0],args[1])(realm::index(args[0]).get(1.0),remaining)
}

#[test]
fn affordable_naval_ranking_recognizes_the_ship_holding_its_crossing() {
    assert!(naval_rank_case(2,20000.0,false));
    assert!(naval_rank_case(84,50000.0,false));
}

#[test]
fn a_short_naval_search_keeps_land_targets_without_claiming_an_unchecked_crossing() {
    assert!(!naval_rank_case(84,20000.0,false));
    assert!(naval_rank_case(84,20000.0,true));
}

#[test]
fn lancer_low_control_keeps_a_standing_fight_order() {
    let plan=pages(&obj! {"attack"=>1,"stage"=>0,"defend"=>-1,"home"=>0});
    let world=Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true},{"id":2,"alive":true}],"provinces":[{"id":0,"owner":1,"adj":[1]},{"id":1,"owner":2,"adj":[0]}]}"#);
    let me=obj! {"id"=>1,"capital"=>0,"allies"=>arr![],"ramProvince"=>obj! {"0"=>plan},
        "rts"=>obj! {"cpu"=>obj! {"available"=>obj! {"control"=>1199}}}};
    let units=Value::parse(r#"[{"id":10,"owner":1,"type":"lancer","at":0,"doing":"fight","target":1,"to":-1,"build":{}}]"#);
    let args=views(&[units,world,me]);
    lancer(args[0],args[1],args[2]);
    assert!(emitted().is_empty(),"an underfunded lancer must preserve its existing fight order");
}

#[test]
fn funded_lancer_routes_a_fresh_capture() {
    let plan=pages(&obj! {"attack"=>1,"stage"=>0,"defend"=>-1,"home"=>0});
    let world=Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true},{"id":2,"alive":true}],"provinces":[{"id":0,"owner":1,"adj":[1]},{"id":1,"owner":2,"adj":[0]}]}"#);
    let me=obj! {"id"=>1,"capital"=>0,"allies"=>arr![],"ramProvince"=>obj! {"0"=>plan},
        "rts"=>obj! {"cpu"=>obj! {"available"=>obj! {"control"=>1200}}}};
    let units=Value::parse(r#"[{"id":10,"owner":1,"type":"lancer","at":0,"doing":"idle","target":-1,"to":-1,"build":{}}]"#);
    let args=views(&[units,world,me]);
    lancer(args[0],args[1],args[2]);
    let orders=emitted();
    assert_eq!(orders.len(),1,"the funded lancer still gets a fresh order: {orders:?}");
    assert_eq!(orders[0].get("do").str(),"capture");
    assert_eq!(orders[0].get("target").num(),1.0);
}

#[test]
fn funded_shades_keep_their_local_plan_and_resume_an_idle_guard() {
    let plan=pages(&obj! {"attack"=>1,"stage"=>0,"defend"=>-1,"picket"=>0,"guard"=>arr![10]});
    let world=Value::parse(r#"{"tick":1,"seconds":1,"factions":[null,{"id":1,"alive":true},{"id":2,"alive":true}],"provinces":[{"id":0,"owner":1,"adj":[1]},{"id":1,"owner":2,"adj":[0]}]}"#);
    let me=obj! {"id"=>1,"capital"=>0,"allies"=>arr![],"ramProvince"=>obj! {"0"=>plan},
        "rts"=>obj! {"cpu"=>obj! {"available"=>obj! {"control"=>10000}}}};
    let units=Value::parse(r#"[{"id":10,"owner":1,"type":"shade","at":0,"doing":"idle","target":-1,"to":-1,"build":{}}]"#);
    let args=views(&[units,world,me]);
    shade(args[0],args[1],args[2]);
    let orders=emitted();
    assert_eq!(orders.len(),1,"{orders:?}");
    assert_eq!(orders[0].get("do").str(),"fight");
    assert_eq!(orders[0].get("target").num(),0.0);
    assert!(orders[0].get("cloak").truthy());
}

fn sabotage_case(cloaked: bool) -> Vec<Value> {
    let u=obj!{"id"=>10,"owner"=>1,"type"=>"shade","at"=>1,"to"=>-1,"doing"=>"idle","target"=>-1,
        "hidden"=>true,"seenBy"=>arr![],"build"=>obj!{"stealth"=>if cloaked {0}else{1}},
        "rts"=>obj!{"cloaked"=>cloaked,"modules"=>arr!["cloak"]}};
    let world=Value::parse(r#"{"energy":true,"seconds":100,"factions":[null,{"id":1,"alive":true,"capital":0},{"id":2,"alive":true,"capital":2}],
        "provinces":[{"id":0,"owner":1,"x":0,"y":0,"adj":[1]},{"id":1,"owner":2,"x":1,"y":0,"adj":[0,2]},{"id":2,"owner":2,"x":2,"y":0,"adj":[1]}],
        "structures":[{"id":91,"p":1,"owner":2,"x":-4,"y":3,"radius":1,"hp":100,"state":"ready",
        "power":{"mw":2,"ports":[{"kind":"wind","capacity":3}],"disruptedFor":0,"disruptionCooldown":0}}]}"#);
    let me=obj!{"id"=>1,"capital"=>0,"bank"=>1000,"costs"=>obj!{"unit"=>100},"allies"=>arr![],"contracts"=>arr![],
        "techs"=>obj!{},"rts"=>obj!{"cpu"=>obj!{}},"ramCapital"=>obj!{}};
    vec![u,world,me]
}

#[test]
fn local_energy_sabotage_uses_the_physical_site_and_preserves_pending_orders() {
    let mut v=sabotage_case(true);let a=views(&v);
    let o=energy_sabotage(a[0],a[1],a[2]).unwrap().unwrap();
    assert_eq!(o.get("structure").num(),91.0);
    assert_eq!(o.get("target").num(),1.0);
    assert_eq!(o.get("at").get("x").num(),-4.0);
    assert_eq!(o.get("at").get("y").num(),3.0);
    assert_eq!(o.get("do").str(),"sabotage");
    v[0].set("doing","sabotage");v[0].set("target",1);v[0].set("sabotageStructure",91);
    let a=views(&v);assert!(matches!(energy_sabotage(a[0],a[1],a[2]),Some(None)));
}

#[test]
fn local_energy_sabotage_skips_recent_allied_poor_detected_and_battery_sites() {
    for case in 0..7 {
        let mut v=sabotage_case(true);
        let mut site=v[1].get("structures").at(0).clone();
        match case {
            0=>site.entry("power").set("disruptedFor",10),
            1=>site.entry("power").set("disruptionCooldown",40),
            2=>v[2].set("allies",arr![2]),
            3=>v[2].set("bank",24),
            4=>{v[0].set("hidden",false);v[0].set("seenBy",arr![2]);},
            5=>site.entry("power").set("ports",arr![obj!{"kind"=>"battery","capacity"=>100}]),
            _=>v[0].set("conduct",obj!{"from"=>2,"seconds"=>30}),
        }
        v[1].set("structures",vec![site]);
        let a=views(&v);assert!(energy_sabotage(a[0],a[1],a[2]).is_none(),"case {case}");
    }
}

#[test]
fn structure_sabotage_sdk_routes_to_discovered_geometry_and_respects_guest_rights() {
    let mut v=sabotage_case(false);v[0].set("at",0);
    let mut site=v[1].get("structures").at(0).clone();site.set("p",2);v[1].set("structures",vec![site]);
    let a=views(&v);let o=spy::sabotage_structure(a[0],a[1],a[2],a[1].get("structures").at(0)).unwrap();
    assert_eq!(o.get("via").items().iter().map(|x|x.num()).collect::<Vec<_>>(),vec![1.0,2.0]);
    v[2].set("contracts",arr![obj!{"with"=>2,"graceIn"=>10,"get"=>obj!{"passage"=>true}}]);
    let a=views(&v);assert!(spy::sabotage_structure(a[0],a[1],a[2],a[1].get("structures").at(0)).is_none());
}

#[test]
fn a_hidden_shade_interrupts_local_generation_without_replacing_detected_combat() {
    let v=sabotage_case(true);let a=views(&[Value::Arr(vec![v[0].clone()]),v[1].clone(),v[2].clone()]);
    shade(a[0],a[1],a[2]);
    let out=emitted();assert_eq!(out.len(),1,"{out:?}");
    assert_eq!(out[0].get("structure").num(),91.0);
    assert!(out[0].get("cloak").truthy());
    let mut v=sabotage_case(true);v[0].set("hidden",false);v[0].set("seenBy",arr![2]);
    let a=views(&[Value::Arr(vec![v[0].clone()]),v[1].clone(),v[2].clone()]);shade(a[0],a[1],a[2]);
    assert!(emitted().iter().all(|o|o.get("structure").is_undef()));
}

#[test]
fn bounded_energy_search_rotates_past_earlier_unrelated_sites() {
    let mut v=sabotage_case(true);
    let site=v[1].get("structures").at(0).clone();
    let mut sites=Vec::new();for id in 0..29 {let mut s=site.clone();s.set("id",id);s.set("owner",1);sites.push(s);}
    sites.push(site);v[1].set("structures",sites);
    let a=views(&v);let o=energy_sabotage(a[0],a[1],a[2]).unwrap().unwrap();
    assert_eq!(o.get("structure").num(),91.0);
}

#[test]
fn a_shade_keeps_its_committed_remote_military_destination() {
    let mut v=sabotage_case(true);v[0].set("doing","capture");v[0].set("target",2);
    let a=views(&[Value::Arr(vec![v[0].clone()]),v[1].clone(),v[2].clone()]);shade(a[0],a[1],a[2]);
    assert!(emitted().iter().all(|o|o.get("structure").is_undef()));
}
