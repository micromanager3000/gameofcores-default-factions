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
