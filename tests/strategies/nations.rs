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
    let orders=planner_recovery(args[0],31893.0).expect("a funded exhausted planner repairs its allowance");
    let policy=orders.get("compute").get("cpu");
    let next=planner_allowance(policy,106000.0);
    assert!(next>32000.0*1.2+4000.0,"exhaustion is a lower bound with a further invocation margin");
    assert!(next<=132500.0);
    assert_eq!(policy.get("limits").get("upkeep").num(),0.72);
    assert_eq!(policy.get("reserve"),me.get("rts").get("cpu").get("reserve"));
    me.entry("rts").entry("cpu").set("limits",policy.get("limits").clone());
    let args=views(&[me]);
    assert!(planner_recovery(args[0],next-3000.0).is_none(),"the outgoing policy must admit the following call, including wrapper costs");
}

#[test]
fn planner_repair_respects_hard_caps_without_creating_a_recovery_loop() {
    let mut me=planner_fixture();
    me.entry("rts").entry("cpu").set("spendCap",45000);
    let args=views(&[me.clone()]);
    let orders=planner_recovery(args[0],31893.0).expect("the hard cap still permits a useful allowance increase");
    let policy=orders.get("compute").get("cpu");
    let next=planner_allowance(policy,106000.0);
    assert!(next<=45000.0);
    me.entry("rts").entry("cpu").set("limits",policy.get("limits").clone());
    let args=views(&[me]);
    assert!(planner_recovery(args[0],next-3000.0).is_none(),"a binding hard cap must not request an impossible larger call forever");
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
    assert!(planner_recovery(args[0],17000.0).is_none(),"a cheap 15k planner need not wait for a large-world allowance");
}

#[test]
fn planner_repair_leaves_other_lanes_reserved_cycles_in_the_shared_bank() {
    let mut me=planner_fixture();
    let cpu=me.entry("rts").entry("cpu");
    cpu.set("bank",240000);
    cpu.set("reserve",obj! {"think"=>3500,"control"=>190000});
    let args=views(&[me]);
    let orders=planner_recovery(args[0],31893.0).expect("the remaining shared bank can fund one larger planner call");
    let next=planner_allowance(orders.get("compute").get("cpu"),106000.0);
    assert!(next<=240000.0-190000.0-4000.0,"other lanes' reserves and the repair call are funded first");
}
