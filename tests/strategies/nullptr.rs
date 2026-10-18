#[test]
fn a_lone_spy_in_a_rival_capital_steals_instead_of_treating_itself_as_support() {
    let world = Value::parse(r#"{"seconds":100,"techs":{"artillery":{"need":100,"requires":[]}},"factions":[null,
        {"id":1,"alive":true,"capital":0},{"id":2,"alive":true,"capital":2,"techs":["artillery"]}],"provinces":[
        {"id":0,"owner":1,"adj":[2]},{"id":2,"owner":2,"adj":[0],"units":{"1":1}}],"units":[]}"#);
    let me = obj! { "id"=>1, "capital"=>0, "allies"=>arr![], "bank"=>10000, "costs"=>obj! { "unit"=>100 },
        "techs"=>obj! {}, "ramCapital"=>obj! {} };
    let ghosts = Value::parse(r#"[{"id":10,"owner":1,"type":"ghost","at":2,"to":-1,"doing":"idle","hidden":true,
        "name":"exploit","build":{"stealth":1},"rts":{"modules":["cloak"]}}]"#);
    let args = views(&[ghosts, world, me]);
    nullptr_ext::spies(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("do").str(), "steal", "{orders:?}");
    assert_eq!(orders[0].get("tech").str(), "artillery");
}

fn recovery_recruit(bank: f64, control_reserve: f64, fabrication_spent: f64) -> (Value, bool) {
    let me = obj! { "id"=>1, "compute"=>obj! { "thinkPeak"=>20000 },
        "units"=>obj! { "total"=>6, "queued"=>0, "byType"=>obj! { "miner"=>4, "hauler"=>2 }, "doing"=>obj! {} },
        "rts"=>obj! { "costs"=>obj! { "priceLevel"=>2 }, "cpu"=>obj! {
            "income"=>40000, "bank"=>bank, "capacity"=>1000000,
            "limits"=>obj! { "fabrication"=>0 }, "available"=>obj! { "fabrication"=>0 },
            "reserve"=>obj! { "control"=>control_reserve }, "spent"=>obj! { "fabrication"=>fabrication_spent }
        } } };
    let args = views(&[me]);
    let mut policy = cpu_policy(args[0], &obj! { "defend"=>-1, "threat"=>0 }, &obj! {});
    common::fit_cpu(&mut policy, args[0]);
    let budget = fabrication_budget(args[0], &policy);
    let mut orders = obj! { "compute"=>obj! { "cpu"=>policy }, "spawn"=>Value::arr(),
        "fabrication"=>budget, "treasury"=>1000, "unitPrice"=>100, "slots"=>1, "priceLevel"=>2 };
    let recruited = spawn(&mut orders, "screen", SCREEN, vec!["dash", "armor"]);
    (orders, recruited)
}

#[test]
fn a_funded_recovery_exit_recruits_under_the_policy_it_emits() {
    let (orders, recruited) = recovery_recruit(1000000.0, 4000.0, 0.0);
    assert!(recruited, "the old zero fabrication allowance must not suppress the restored policy");
    assert_eq!(orders.get("spawn").at(0).get("type").str(), "screen");
    let limit = orders.get("compute").get("cpu").get("limits").get("fabrication").num();
    assert_eq!(limit, 10400.0);
    assert_eq!(orders.get("fabrication").num(), limit - 4800.0, "the proposed recruit consumes its fabrication budget");
}

#[test]
fn recovery_recruitment_preserves_think_control_and_fabrication_commitments() {
    for (bank, reserve, spent) in [(45000.0, 4000.0, 0.0), (200000.0, 180000.0, 0.0), (1000000.0, 4000.0, 6000.0)] {
        let (orders, recruited) = recovery_recruit(bank, reserve, spent);
        assert!(!recruited, "bank={bank}, control reserve={reserve}, fabrication spent={spent}");
        assert!(orders.get("spawn").is_empty());
    }
}

#[test]
fn rich_cargo_recovery_rearms_a_screen_before_a_full_plan_fits() {
    let mut recovery_pages = Value::obj();
    for op in mem::put(244, &obj! { "r"=>true }, "local", None, 1).expect("marker fits") {
        recovery_pages.set(&op.get("write").num().to_string(), obj! { "bytes"=>op.get("bytes").clone(), "t"=>1 });
    }
    let me = obj! { "id"=>6, "capital"=>36, "compute"=>obj! { "thinkPeak"=>15449 },
        "units"=>obj! { "total"=>33, "cap"=>52, "queued"=>0, "stalled"=>0,
            "byType"=>obj! { "miner"=>17, "hauler"=>11, "freighter"=>2, "escort"=>2, "sentry"=>1 } },
        "rts"=>obj! { "costs"=>obj! { "fabrication"=>obj! { "base"=>4800, "module"=>2400 } }, "cpu"=>obj! {
            "income"=>36694, "bank"=>379960, "limits"=>obj! { "fabrication"=>0, "control"=>22000 },
            "reserve"=>obj! {}, "spent"=>obj! {} } },
        "ramCapital"=>recovery_pages, "types"=>obj! { "miner"=>obj! {}, "screen"=>obj! {} } };
    let args = views(&[me]);
    assert!(productive_reentry(args[0]));
    let orders = think_reentry(args[0]);
    assert_eq!(orders.get("memo").str(), "Address Zero: rearming a screen for the carried front.");
    assert_eq!(orders.get("spawn").at(0).get("type").str(), "screen");
    assert_eq!(orders.get("compute").get("cpu").get("limits").get("fabrication").num(), 5000.0);
    assert_eq!(orders.get("compute").get("types").get("screen").get("cycles").num(), 6000.0);
    assert_eq!(orders.get("compute").get("types").get("hauler").get("cycles").num(), 10000.0);
}

fn naval_rank_case(waters: usize, remaining: f64, land_border: bool) -> bool {
    let mut sea = Value::arr();
    for id in 0..waters {
        sea.push(obj! {"id"=>id,"adj"=>if id == 0 {arr![1]} else if id == 1 {arr![0]} else {arr![]},
            "coast"=>if id == 0 {arr![0]} else if id == 1 {arr![1]} else {arr![]} });
    }
    let world = obj! {"waters"=>sea,"provinces"=>arr![
        obj! {"id"=>0,"owner"=>6},
        obj! {"id"=>1,"owner"=>-1,"adj"=>if land_border {arr![0]} else {arr![]},"routes"=>obj! {"sea"=>arr![0],"air"=>arr![],"pass"=>arr![]}}],
        "units"=>arr![obj! {"owner"=>6,"rts"=>obj! {"sea"=>obj! {"ship"=>true,"water"=>0}}}]};
    let args = views(&[world,obj! {"id"=>6,"rts"=>obj! {"cpu"=>obj! {"income"=>40000}}}]);
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
