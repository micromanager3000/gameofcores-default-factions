fn cleared_front_case() -> (Value, Value, Value, Value) {
    let world = obj! {"seconds"=>86400,"provinces"=>arr![
        obj! {"id"=>24,"owner"=>6},
        obj! {"id"=>35,"owner"=>-1,"visible"=>true,"cores"=>4,"adj"=>arr![39],"units"=>obj! {"6"=>5}},
        obj! {"id"=>39,"owner"=>-1,"visible"=>true,"cores"=>1,"adj"=>arr![35],"units"=>obj! {}}
    ]};
    let me = obj! {"id"=>6,"capital"=>38,"allies"=>arr![]};
    let unit = obj! {"id"=>1584,"owner"=>6,"at"=>35,"to"=>-1,"doing"=>"idle","target"=>35,
        "build"=>obj! {"strength"=>2},"ramUnit"=>obj! {}};
    let plan = obj! {"attack"=>24,"stage"=>28,"defend"=>-1,"at"=>69692};
    (world, me, unit, plan)
}

#[test]
fn an_idle_column_recaptures_its_clear_neutral_land_after_the_carried_target_falls() {
    let (world, me, unit, plan) = cleared_front_case();
    let args = views(&[world, me, unit]);
    let order = cleared_front(args[2], args[0], args[1], &plan).expect("local capture");
    assert_eq!(order.get("do").str(), "capture");
    assert_eq!(order.get("target").num(), 35.0);
    assert!(order.get("via").is_empty(), "the column already stands there");
    let bytes = order.get("ram").at(0).get("bytes");
    let Value::Arr(bytes) = bytes else { panic!("carried plan bytes") };
    let local = plan::unpack(&bytes.iter().map(|v| v.num() as u8).collect::<Vec<_>>());
    assert_eq!(local.get("attack").num(), 35.0);
    assert_eq!(local.get("at").num(), 86400.0);
}

#[test]
fn a_cleared_local_column_can_continue_to_visible_empty_neutral_neighbors() {
    let (mut world, me, unit, plan) = cleared_front_case();
    let Value::Arr(provinces) = world.entry("provinces") else { panic!("provinces") };
    provinces[1].set("owner", 6);
    let args = views(&[world, me, unit]);
    let order = cleared_front(args[2], args[0], args[1], &plan).expect("local advance");
    assert_eq!(order.get("target").num(), 39.0);
}

#[test]
fn local_advances_preserve_fresh_orders_defense_and_occupied_land() {
    for reason in ["fresh", "defense", "moving", "working", "air", "ship", "enemy", "ally", "lease"] {
        let (mut world, me, mut unit, mut plan) = cleared_front_case();
        match reason {
            "fresh" => plan.set("at", 86300),
            "defense" => plan.set("defend", 35),
            "moving" => unit.set("to", 39),
            "working" => unit.set("doing", "capture"),
            "air" => unit.entry("build").set("wings", 1),
            "ship" => unit.entry("rts").entry("sea").set("ship", true),
            reason => {
                let Value::Arr(provinces) = world.entry("provinces") else { panic!("provinces") };
                match reason {
                    "enemy" => provinces[1].entry("units").set("2", 1),
                    "ally" => provinces[1].set("owner", 2),
                    _ => provinces[1].set("leased", arr![obj! {"to"=>6}]),
                }
            }
        }
        let args = views(&[world, me, unit]);
        assert!(cleared_front(args[2], args[0], args[1], &plan).is_none(), "{reason}");
    }
}

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
    // Eleven haulers at 1,600 cycles each.
    assert_eq!(orders.get("compute").get("types").get("hauler").get("cycles").num(), 17600.0);
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

#[test]
fn rts_prefers_a_supported_land_front_to_a_richer_unstaffed_crossing() {
    let world = obj! { "waters"=>arr![], "units"=>arr![], "provinces"=>arr![
        obj! { "id"=>0, "owner"=>6 },
        obj! { "id"=>1, "owner"=>-1, "cores"=>1, "adj"=>arr![0], "routes"=>obj! { "sea"=>arr![], "air"=>arr![], "pass"=>arr![] } },
        obj! { "id"=>2, "owner"=>-1, "cores"=>20, "adj"=>arr![], "routes"=>obj! { "sea"=>arr![0], "air"=>arr![], "pass"=>arr![] } }] };
    let me = obj! { "id"=>6, "rts"=>obj! { "cpu"=>obj! { "income"=>40000 } } };
    let args = views(&[world, me]);
    let rim = vec![realm::index(args[0]).get(1.0), realm::index(args[0]).get(2.0)];
    let hold = rank_hold(args[0], args[1]);
    let supported: Vec<f64> = rim.iter().filter(|q| hold(**q, 30000.0)).map(|q| q.get("id").num()).collect();
    let preferred = prefer_supported(rim, true, &supported);
    assert_eq!(preferred.len(), 1);
    assert_eq!(preferred[0].get("id").num(), 1.0, "a reachable land front precedes a richer unsupported crossing");
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
fn ghosts_keep_escape_and_capital_theft_ahead_of_local_energy_sabotage() {
    let mut v=sabotage_case(false);v[0].set("type","ghost");
    let a=views(&[Value::Arr(vec![v[0].clone()]),v[1].clone(),v[2].clone()]);ghost(a[0],a[1],a[2]);
    assert!(emitted().iter().any(|o|o.get("structure").num_or(-1.0)==91.0));
    v[0].set("hidden",false);v[0].set("seenBy",arr![2]);
    let a=views(&[Value::Arr(vec![v[0].clone()]),v[1].clone(),v[2].clone()]);ghost(a[0],a[1],a[2]);
    let out=emitted();assert!(out.iter().any(|o|o.get("do").str()=="move"),"{out:?}");
    assert!(out.iter().all(|o|o.get("structure").is_undef()));
}

#[test]
fn a_generator_inside_a_capital_does_not_replace_valuable_technology_theft() {
    let mut v=sabotage_case(false);v[0].set("at",2);
    let mut site=v[1].get("structures").at(0).clone();site.set("p",2);v[1].set("structures",vec![site]);
    v[1].set("techs",obj!{"artillery"=>obj!{"need"=>100,"requires"=>arr![]}});
    let mut factions=v[1].get("factions").items().to_vec();factions[2].set("techs",arr!["artillery"]);v[1].set("factions",factions);
    let a=views(&[Value::Arr(vec![v[0].clone()]),v[1].clone(),v[2].clone()]);ghost(a[0],a[1],a[2]);
    let out=emitted();assert!(out.iter().any(|o|o.get("do").str()=="steal"),"{out:?}");
    assert!(out.iter().all(|o|o.get("structure").is_undef()));
}
#[test]
fn local_nullptr_planning_rearms_workers_without_waiting_for_a_full_spy_census() {
    let args=views(&productive_local_fixture());
    let out=super::local_strategy(args[0],args[1]);
    assert!(out.get("spawn").items().iter().any(|s|s.get("type").str()=="screen"),"the held mining core receives a combat unit");
    assert!(!out.get("ram").is_empty(),"the reachable target and its stage accompany the CPU policy");
    assert!(out.get("compute").get("cpu").truthy());
}
#[test]
fn a_local_nullptr_army_develops_the_core_it_just_held() {
    let args=views(&frontier_mining_fixture("screen"));
    let out=super::local_strategy(args[0],args[1]);
    assert!(out.get("spawn").items().iter().any(|s|s.get("type").str()=="miner"));
}

#[test]
fn artillery_orders_strike_walls_only_during_an_offensive_siege() {
    for (walls,defending,want) in [(false,false,"fight"),(true,false,"capture"),(true,true,"fight")] {
        let fixture=wall_artillery_fixture("gun",walls,defending,false,false);
        let args=views(&fixture);
        super::gun(args[0],args[1],args[2]);
        let orders=emitted();
        assert!(orders.iter().any(|o|o.get("do").str()==want && o.get("target").num()==1.0),
            "walls={walls}, defending={defending}: {orders:?}");
    }
}

#[test]
fn artillery_already_at_the_enemy_town_switches_its_standing_fight_to_the_walls() {
    for deployed in [false,true] {
        let fixture=wall_artillery_fixture("gun",true,false,true,deployed);
        let args=views(&fixture);
        super::gun(args[0],args[1],args[2]);
        let orders=emitted();
        assert!(orders.iter().any(|o|o.get("do").str()=="capture" && o.get("target").num()==1.0),
            "deployed={deployed}: {orders:?}");
        if deployed {assert!(!orders.iter().any(|o|matches!(o.get("deploy"),Value::Bool(false))),
            "a gun already at the siege keeps its deployment");}
    }
}
