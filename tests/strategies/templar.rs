#[test]
fn campaign_ranking_keeps_all_frontier_options_and_promotes_a_viable_capital() {
    let campaign = pages(&obj! {"campaign"=>3});
    let world = Value::parse(r#"{
        "seconds":1,
        "factions":[null,
            {"id":1,"alive":true,"shielded":false},
            {"id":2,"alive":true,"shielded":false},
            {"id":3,"alive":true,"shielded":false}
        ],
        "provinces":[
            {"id":0,"owner":1,"adj":[1,2]},
            {"id":1,"owner":2,"adj":[0],"cores":4,"townGarrison":0},
            {"id":2,"owner":3,"adj":[0],"cores":1,"townGarrison":0,"capital":true}
        ]
    }"#);
    let me = obj! {"id"=>1,"capital"=>0,"ramCapital"=>campaign,
        "units"=>obj! {"total"=>20,"byType"=>obj! {}}};
    let args = views(&[world, me]);
    let ranked = super::targets(args[0], args[1]);
    let ids: Vec<f64> = ranked.iter().map(|q| q.get("id").num()).collect();
    assert_eq!(ids, vec![2.0, 1.0], "the viable campaign capital leads without discarding the other frontier target");
}

#[test]
fn peak_aware_admission_recovers_before_an_unfunded_full_templar_plan() {
    let world = Value::parse(r#"{"seconds":6,"provinces":[{}, {}, {}, {}, {}, {}, {}, {}, {}, {}],"units":[
        {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}]}"#);
    let me = Value::parse(r#"{"id":1,"compute":{"thinkPeak":120000},"units":{"total":20},
        "rts":{"cpu":{"income":100000,"bank":300000,"spendCap":180000,"available":{"think":150000}}}}"#);
    let args = views(&[world, me]);
    let need = super::budgeted_think_need(args[0], args[1]);
    assert_eq!(need, super::full_think_need(args[0], args[1]));
    assert!(!super::plan_fuel_ready(150000.0, 180000.0, need));
    let recovery = super::recovery_for_plan(args[0], args[1], need);
    assert_eq!(recovery.get("memo").str(), "Recovering production");
    assert!(recovery.get("compute").get("cpu").get("limits").get("think").num() <= 180000.0);
}

#[test]
fn capped_templar_keeps_a_funded_lean_retry() {
    let world = Value::parse(r#"{"seconds":6,"provinces":[{}, {}, {}, {}, {}, {}, {}, {}, {}, {}],"units":[
        {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}]}"#);
    let me = Value::parse(r#"{"id":1,"compute":{"thinkPeak":120000},"units":{"total":20},
        "rts":{"cpu":{"income":50000,"bank":150000,"spendCap":40000,"available":{"think":9000}}}}"#);
    let args = views(&[world, me]);
    let need = super::budgeted_think_need(args[0], args[1]);
    assert_eq!(need, super::lean_think_need(args[0], args[1]));
    let recovery = super::recovery_for_plan(args[0], args[1], need);
    let retry = recovery.get("compute").get("cpu").get("limits").get("think").num();
    assert!(retry >= need + super::PLAN_REMAINING_MARGIN + super::PLAN_CALLBACK_OVERHEAD, "{recovery:?}");
    assert!(retry <= 40000.0, "{recovery:?}");
    assert!(super::plan_fuel_ready(retry, retry - super::PLAN_CALLBACK_OVERHEAD, need));
    assert!(!super::full_plan_selected(args[0], args[1]), "the lean retry must omit optional world scans");
}

#[test]
fn capped_templar_executes_a_faithful_local_plan_and_keeps_its_standing_sermons() {
    let world=Value::parse(r#"{"seconds":600,"factions":[null,{"id":1,"alive":true},{"id":2,"alive":true},
        {"id":3,"alive":true},{"id":4,"alive":true},{"id":5,"alive":true}],
        "provinces":[{"id":0,"owner":1,"adj":[1,2,3,4],"hops":0,"mines":[2],"staff":[0],"units":{"1":8}},
          {"id":1,"owner":2,"visible":true,"cores":1,"name":"Chapel","adj":[0],"townGarrison":0,"faith":{"by":1,"zeal":120}},
          {"id":2,"owner":3,"visible":true,"cores":4,"name":"Plain","adj":[0],"townGarrison":0},
          {"id":3,"owner":4,"visible":true,"cores":100,"adj":[0]},
          {"id":4,"owner":5,"visible":true,"cores":100,"adj":[0]}],
        "techs":{"turbines":{"offer":1000,"requires":["armor"]}},
        "units":[{"id":99,"owner":2,"air":true,"at":1,"build":{"wings":1,"range":2}}]}"#);
    let me=obj! {"id"=>1,"capital"=>0,"provinces"=>1,"bank"=>10000,
        "ramCapital"=>pages(&obj! {"pilgrims"=>arr![2]}),"allies"=>arr![4],
        "contracts"=>arr![obj! {"with"=>5,"get"=>obj! {"passage"=>true}}],
        "costs"=>obj! {"unit"=>100,"upkeep"=>100},
        "compute"=>obj! {"cycles"=>50000,"thinkPeak"=>120000},
        "techs"=>obj! {"extraction"=>true,"armor"=>true,"dash"=>true,"influence"=>true,"faith"=>true},
        "sway"=>obj! {"faith"=>obj! {"name"=>"Rite","upkeep"=>100,"provinces"=>1},
            "preach"=>arr![obj! {"p"=>1,"cycles"=>2000}],"propaganda"=>arr![obj! {"p"=>1,"cycles"=>1000}]},
        "units"=>obj! {"total"=>14,"cap"=>40,"queued"=>0,"spawned"=>14,
            "byType"=>obj! {"miner"=>6,"bulwark"=>2},"doing"=>obj! {"gather"=>6}},
        "rts"=>obj! {"bank"=>1000000,"cpu"=>obj! {"income"=>50000,"bank"=>1000000,"capacity"=>1000000,
            "spendCap"=>40000,"limits"=>obj! {"think"=>150000},
            "available"=>obj! {"think"=>30000,"control"=>50000,"fabrication"=>50000}}}};
    let args=views(&[world,me]);
    assert!(!super::full_plan_selected(args[0],args[1]));
    let orders=super::strategy(args[0],args[1]);
    let mut ram=Value::obj();
    for op in orders.get("ram").items() {
        if matches!(op.get("write"),Value::Num(_)) {
            ram.set(&op.get("write").num().to_string(),obj! {"bytes"=>op.get("bytes").clone(),"t"=>600});
        }
    }
    let saved=views(&[ram]);let plan=mem::get(saved[0],0);
    assert_eq!(plan.get("attack").num(),1.0,"the local crusade prefers land its faith already softened, avoiding allies and passage hosts");
    assert_eq!(plan.get("stage").num(),0.0);
    assert_eq!(plan.get("pilgrims"),&arr![2],"the scouts retain their existing stations");
    assert_eq!(plan.get("mines").len(),2);
    for key in ["field","front","convoy"] {assert!(plan.get(key).is_undef(),"lean planning omits {key}");}
    assert_eq!(orders.get("spawn").at(0).get("type").str(),"bulwark");
    assert!(orders.get("compute").get("cpu").get("limits").get("fabrication").num()>0.0);
    assert!(orders.get("compute").get("types").get("bulwark").get("share").num()>0.0);
    assert_eq!(orders.get("research").str(),"turbines");
    assert!(orders.get("compute").get("cpu").get("limits").get("influence").num()>=3100.0,"the standing faith and sermons remain funded");
    for key in ["preach","propaganda","faith","send","accept"] {assert!(orders.get(key).is_undef(),"the lean turn preserves standing faith orders without recomputing {key}");}
}

#[test]
fn lean_defense_keeps_the_threatened_capital_ahead_of_a_larger_remote_invasion() {
    for capital_threat in [1,0] {
        let world=obj! {"seconds"=>6,"factions"=>arr![Value::Null,obj! {"id"=>1,"alive"=>true},obj! {"id"=>2,"alive"=>true}],
            "provinces"=>arr![obj! {"id"=>0,"owner"=>1,"adj"=>arr![1],"units"=>obj! {"2"=>capital_threat}},
                obj! {"id"=>1,"owner"=>1,"adj"=>arr![0],"units"=>obj! {"2"=>2}}]};
        let me=obj! {"id"=>1,"capital"=>0,"allies"=>arr![],"contracts"=>arr![],"ramCapital"=>obj! {}};
        let args=views(&[world,me]);let plan=super::lean_plan(args[0],args[1]);
        assert_eq!(plan.get("defend").num(),if capital_threat>0 {0.0} else {1.0},
            "one invader at the capital outranks two elsewhere; without a capital threat the remote invasion is defended");
        assert_eq!(plan.get("threat").num(),if capital_threat>0 {1.0} else {2.0});
    }
}

// Metered fixtures: templar-think-endpoint.json is the candidate1 seed101 t3600
// public own observation. templar-think-naval.json is an authored counterfactual
// of that endpoint, changing only cpu income=48216.707, spendCap=60270,
// limits.think=53880, available.think=53775 and spent.think=0. These match the
// recorded t3271 resource window; its original observation was not captured.
fn naval_target_case(waters: usize, remaining: f64, land_border: bool) -> bool {
    let mut sea = Value::arr();
    for id in 0..waters {
        sea.push(obj! {"id"=>id,"adj"=>if id == 0 {arr![1]} else if id == 1 {arr![0]} else {arr![]},
            "coast"=>if id == 0 {arr![0]} else if id == 1 {arr![1]} else {arr![]} });
    }
    let world = obj! {"waters"=>sea,"provinces"=>arr![
        obj! {"id"=>0,"owner"=>2},
        obj! {"id"=>1,"owner"=>-1,"adj"=>if land_border {arr![0]} else {arr![]},
            "routes"=>obj! {"sea"=>arr![0],"air"=>arr![],"pass"=>arr![]}}],
        "units"=>arr![obj! {"owner"=>2,"rts"=>obj! {"sea"=>obj! {"ship"=>true,"water"=>0}}}]};
    let args = views(&[world,obj! {"id"=>2,"rts"=>obj! {"cpu"=>obj! {"income"=>40000}}}]);
    super::target_hold(args[0],args[1])(realm::index(args[0]).get(1.0),remaining)
}

#[test]
fn a_funded_naval_target_can_use_its_ship_on_station() {
    assert!(naval_target_case(2,20000.0,false));
    assert!(naval_target_case(84,50000.0,false));
}

#[test]
fn a_short_naval_rank_preserves_land_without_assuming_a_crossing_is_held() {
    assert!(!naval_target_case(84,20000.0,false));
    assert!(naval_target_case(84,20000.0,true));
}

#[test]
fn logistics_miner_keeps_a_bounded_cargo_retry_below_the_c4_full_assignment_runway() {
    assert!(super::common::LOGISTICS_MINER_FULL_RUNWAY > 4324.0,
        "the captured loaded-miner callback must not enter full local assignment");
    assert!(super::common::LOGISTICS_MINER_FULL_RUNWAY <= 5000.0,
        "the measured funded full assignment remains available at five thousand cycles");
}
