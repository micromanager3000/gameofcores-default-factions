#[test]
fn classic_reflex_returns_no_duplicate_after_committing_the_order() {
    let units = Value::parse(r#"[{"id":1,"doing":"fight","reach":1,"range":0,"air":false}]"#);
    let near = Value::parse(r#"{"1":[{"id":2,"rival":true,"air":false,"d":0.5}]}"#);
    let args = views(&[units, near]);
    let returned = super::reflex(args[0], args[1]);
    let committed = emitted();
    assert!(returned.is_empty(), "classic order() commits survive independently of a returned list");
    assert_eq!(committed.len(), 1, "the committed list contains the unit's action");
    assert_eq!(committed[0].get("hit").num(), 2.0);
}

#[test]
fn funded_guard_keeps_a_capital_order_after_the_short_callback_entry_guard() {
    let units=Value::parse(r#"[{"id":1,"at":0,"doing":"idle","target":0,"to":-1,"way":[],"air":false,"build":{"wings":0,"range":0}}]"#);
    let world=Value::parse(r#"{"tick":1,"seconds":1,"provinces":[{"id":0,"owner":1,"adj":[1],"units":{"1":1},"mines":[1],"staff":[0]},{"id":1,"owner":-1,"adj":[0],"units":{},"mines":[1],"staff":[0]}],"factions":[null,{"id":1,"alive":true}]}"#);
    let me=Value::parse(r#"{"id":1,"capital":0,"ramCapital":{},"ramProvince":{},"units":{"total":1,"byType":{"guard":1}},"rts":{"cpu":{"income":15000}},"allies":[],"contracts":[]}"#);
    let args=views(&[units,world,me]);
    super::guard(args[0],args[1],args[2]);
    let orders=emitted();
    assert_eq!(orders.len(),1,"a funded guard keeps the capital protected");
    assert_eq!(orders[0].get("do").str(),"fight");
    assert!(super::GUARD_CALLBACK_RUNWAY < 2174.0,
        "the uniform world's funded guard allocation remains above the entry guard");
}

#[test]
fn recovery_keeps_cargo_controllers_and_upkeep_priority_in_logistics_worlds() {
    for cargo in [false, true] {
        let world=obj! {"seconds"=>600,"provinces"=>Vec::<Value>::new(),"units"=>Vec::<Value>::new()};
        let mut me=obj! {"id"=>0,"capital"=>0,"ramCapital"=>obj! {},
            "types"=>obj! {"miner"=>obj! {},"hauler"=>obj! {},"freighter"=>obj! {},"guard"=>obj! {}},
            "units"=>obj! {"total"=>0,"stalled"=>0,"byType"=>obj! {}},
            "rts"=>obj! {"cpu"=>obj! {"income"=>20000,"bank"=>1000}}};
        if cargo {me.entry("rts").set("logistics",obj! {"depots"=>Vec::<Value>::new()});}
        let args=views(&[world,me]);
        let out=shared::common::recovery(args[0],args[1]);
        let compute=out.get("compute");
        for (ty,allocation) in [("hauler",10000.0),("freighter",12000.0)] {
            assert_eq!(compute.get("types").get(ty).get("cycles").num_or(0.0),if cargo {allocation}else{0.0});
            assert_eq!(compute.get("cpu").get("upkeepOrder").items().iter().any(|v|v.str()==ty),cargo);
        }
        assert_eq!(compute.get("types").get("guard").get("share").num(),0.0);
    }
}

#[test]
fn local_planning_uses_a_newly_held_core_as_its_next_stage_and_keeps_sea_assignments() {
    let world=obj! {"seconds"=>30,"factions"=>vec![obj! {"id"=>0,"alive"=>true}],
        "provinces"=>vec![obj! {"id"=>0,"owner"=>0,"adj"=>vec![1],"units"=>obj! {}},
            obj! {"id"=>1,"owner"=>0,"adj"=>vec![0,2],"units"=>obj! {},"mines"=>vec![1],"staff"=>vec![0]},
            obj! {"id"=>2,"owner"=>-1,"adj"=>vec![1],"cores"=>3,"townGarrison"=>0,"units"=>obj! {}}]};
    let me=obj! {"id"=>0,"capital"=>0,"allies"=>Vec::<Value>::new(),"contracts"=>Vec::<Value>::new(),
        "units"=>obj! {"total"=>8,"byType"=>obj! {"miner"=>4}}};
    let nv=obj! {"st"=>vec![arr![0,1,7]],"lt"=>3};
    let previous=obj! {"attack"=>1,"target"=>1,"stage"=>0,"field"=>obj! {"to"=>1},"nv"=>nv.clone()};
    let args=views(&[world,me]);
    let plan=shared::common::local_plan(args[0],args[1],&previous,14000.0);
    assert_eq!(plan.get("attack").num(),2.0);
    assert_eq!(plan.get("stage").num(),1.0);
    assert!(matches!(plan.get("field"),Value::Null),"a route to the captured province cannot route the next attack");
    assert!(plan.get("mines").items().iter().any(|m|m.at(0).num()==1.0),"the held core receives mining jobs");
    assert_eq!(plan.get("nv"),&nv);
}

#[test]
fn local_planning_protects_the_capital_and_respects_allies_and_passage() {
    let world=obj! {"seconds"=>30,"factions"=>vec![obj! {"id"=>0,"alive"=>true},obj! {"id"=>1,"alive"=>true},obj! {"id"=>2,"alive"=>true}],
        "provinces"=>vec![obj! {"id"=>0,"owner"=>0,"adj"=>vec![1,2,3],"units"=>obj! {"1"=>2}},
            obj! {"id"=>1,"owner"=>0,"adj"=>vec![0],"units"=>obj! {"1"=>5}},
            obj! {"id"=>2,"owner"=>2,"visible"=>true,"adj"=>vec![0],"units"=>obj! {},"cores"=>8},
            obj! {"id"=>3,"owner"=>1,"visible"=>true,"adj"=>vec![0],"units"=>obj! {},"cores"=>8}]};
    let me=obj! {"id"=>0,"capital"=>0,"allies"=>vec![2],"contracts"=>vec![obj! {"with"=>1,"get"=>obj! {"passage"=>true}}],
        "units"=>obj! {"total"=>10,"byType"=>obj! {"miner"=>4}}};
    let args=views(&[world,me]);
    let plan=shared::common::local_plan(args[0],args[1],&obj! {"attack"=>2,"stage"=>0},14000.0);
    assert_eq!(plan.get("defend").num(),0.0,"capital defense takes priority over a larger remote incursion");
    assert_eq!(plan.get("threat").num(),2.0);
    assert_eq!(plan.get("attack").num(),-1.0,"allied and passage land remain protected");
}
#[test]
fn a_rebuilding_local_army_takes_free_core_land_before_resuming_its_capital_campaign() {
    for rebuilding in [false,true] {
        let world=obj! {"seconds"=>30,"factions"=>vec![obj! {"id"=>0,"alive"=>true},obj! {"id"=>1,"alive"=>true}],
            "provinces"=>vec![obj! {"id"=>0,"owner"=>0,"adj"=>vec![1,2],"units"=>obj! {}},
                obj! {"id"=>1,"owner"=>1,"adj"=>vec![0],"visible"=>true,"capital"=>true,"cores"=>5,"townGarrison"=>0,"units"=>obj! {}},
                obj! {"id"=>2,"owner"=>-1,"adj"=>vec![0],"visible"=>true,"cores"=>2,"townGarrison"=>0,"units"=>obj! {}}]};
        let me=obj! {"id"=>0,"capital"=>0,"allies"=>Vec::<Value>::new(),"contracts"=>Vec::<Value>::new(),
            "units"=>obj! {"total"=>10,"stalled"=>if rebuilding {3}else{0},"byType"=>obj! {"miner"=>4}},
            "rts"=>obj! {"cpu"=>obj! {"income"=>20000,"bank"=>if rebuilding {50000}else{200000}}}};
        let args=views(&[world,me]);
        let plan=shared::common::local_plan(args[0],args[1],&obj! {"attack"=>1,"stage"=>0},14000.0);
        assert_eq!(plan.get("attack").num(),if rebuilding {2.0}else{1.0});
    }
}
#[test]
fn local_musters_count_different_ground_controllers_and_exclude_the_working_crew() {
    let world=obj! {"seconds"=>30,"factions"=>vec![obj! {"id"=>0,"alive"=>true}],
        "provinces"=>vec![obj! {"id"=>0,"owner"=>0,"adj"=>vec![1],"units"=>obj! {"0"=>7}},
            obj! {"id"=>1,"owner"=>-1,"adj"=>vec![0],"visible"=>true,"cores"=>2,"townGarrison"=>0,"units"=>obj! {}}],
        "units"=>vec![obj! {"id"=>1,"owner"=>0,"at"=>0,"type"=>"miner","doing"=>"gather"},
            obj! {"id"=>2,"owner"=>0,"at"=>0,"type"=>"hauler"},
            obj! {"id"=>3,"owner"=>0,"at"=>0,"type"=>"wing","air"=>true},
            obj! {"id"=>4,"owner"=>0,"at"=>0,"type"=>"screen","rts"=>obj! {"sea"=>obj! {"ship"=>true}}},
            obj! {"id"=>5,"owner"=>0,"at"=>0,"type"=>"gun","doing"=>"study"},
            obj! {"id"=>6,"owner"=>0,"at"=>0,"type"=>"screen"},obj! {"id"=>7,"owner"=>0,"at"=>0,"type"=>"skirm"}]};
    let me=obj! {"id"=>0,"capital"=>0,"allies"=>Vec::<Value>::new(),"contracts"=>Vec::<Value>::new(),
        "units"=>obj! {"total"=>7,"byType"=>obj! {"miner"=>1,"hauler"=>1},"doing"=>obj! {"gather"=>1,"study"=>1}}};
    let args=views(&[world,me]);
    let plan=shared::common::local_plan(args[0],args[1],&obj! {},12000.0);
    assert_eq!(plan.get("need").num(),2.0);
    assert_eq!(plan.get("ready").num(),2.0,"separate controllers can launch the same supplied wave");
}
#[test]
fn paid_reserves_keep_the_normal_planning_cadence_during_an_upkeep_shortfall() {
    for second in [31,32,33] {
        let world=obj! {"seconds"=>second,"logistics"=>obj! {}};
        let me=obj! {"ramCapital"=>obj! {},"units"=>obj! {"total"=>6,"stalled"=>2},
            "rts"=>obj! {"cpu"=>obj! {"income"=>20000,"bank"=>100000}}};
        let args=views(&[world,me]);
        let out=shared::common::govern(args[0],args[1],0.5,||obj! {"planned"=>true});
        assert_eq!(out.get("planned").truthy(),second==33);
    }
}

#[test]
fn a_short_upkeep_bridge_preserves_jobs_and_limits_the_bill_to_available_savings() {
    let world=obj! {"seconds"=>31,"logistics"=>obj! {}};
    let me=obj! {"ramCapital"=>obj! {},"units"=>obj! {"total"=>6,"stalled"=>2},
        "rts"=>obj! {"cpu"=>obj! {"income"=>20000,"bank"=>100000,"last"=>obj! {"upkeep"=>16000},
            "limits"=>obj! {"upkeep"=>0.4,"control"=>22000},"reserve"=>obj! {"upkeep"=>0.1},"upkeepOrder"=>vec!["miner","hauler"]}}};
    let args=views(&[world,me]);
    let out=shared::common::govern(args[0],args[1],0.5,||panic!("the full planner must wait for its beat"));
    let policy=out.get("compute").get("cpu");
    assert_eq!(policy.get("limits").get("upkeep").num(),24000.0);
    assert_eq!(policy.get("limits").get("control").num(),22000.0);
    assert_eq!(policy.get("reserve").get("upkeep").num(),0.1);
    for field in ["ram","spawn","disband","policy"] {assert!(out.get(field).is_undef(),"standing jobs survive: {field}");}
}
#[test]
fn a_fresh_planner_allowance_includes_return_cycles_once_and_copied_policies_do_not_grow() {
    let mut fixture=productive_local_fixture();
    let args=views(&fixture);
    let first=shared::common::govern(args[0],args[1],0.5,||obj! {"compute"=>obj! {"cpu"=>obj! {
        "limits"=>obj! {"think"=>20000,"upkeep"=>0.8},"reserve"=>obj! {},"upkeepOrder"=>vec!["miner"]}}});
    let policy=first.get("compute").get("cpu");
    assert_eq!(policy.get("limits").get("think").num(),28000.0,"20,000 planner cycles plus the 8,000-cycle governor return reserve");
    fixture[0].set("seconds",31);fixture[1].entry("units").set("stalled",2);
    fixture[1].entry("rts").entry("cpu").set("limits",policy.get("limits").clone());
    let args=views(&fixture);
    let next=shared::common::govern(args[0],args[1],0.5,||panic!("between planning beats"));
    assert_eq!(next.get("compute").get("cpu").get("limits").get("think").num(),28000.0,"a copied policy must not add the return reserve again");
}
