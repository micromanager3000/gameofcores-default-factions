fn factions() -> Value {
    Value::parse(r#"[{"id":1,"alive":true,"shielded":false},{"id":2,"alive":true,"shielded":false}]"#)
}

#[test]
fn a_capped_science_house_refreshes_its_plan_and_recruits_without_optional_research_or_market_scans() {
    let mut provinces=arr![
        obj!{"id"=>0,"owner"=>1,"adj"=>arr![1],"hops"=>0,"mines"=>arr![2],"staff"=>arr![0],"units"=>obj!{"1"=>8}},
        obj!{"id"=>1,"owner"=>-1,"adj"=>arr![0],"visible"=>true,"cores"=>2,"townGarrison"=>0}
    ];
    for id in 2..44 {provinces.push(obj!{"id"=>id,"owner"=>2,"adj"=>arr![],"visible"=>true});}
    let world=obj!{"seconds"=>600,"provinces"=>provinces,"units"=>arr![],"factions"=>factions(),
        "techs"=>obj!{"turbines"=>obj!{"offer"=>1000,"requires"=>arr!["armor"]}}};
    let me=obj!{"id"=>1,"seat"=>3,"capital"=>0,"provinces"=>1,"bank"=>10000,"ramCapital"=>obj!{},
        "techs"=>obj!{"extraction"=>true,"armor"=>true,"dash"=>true},"research"=>obj!{"tech"=>"turbines"},
        "costs"=>obj!{"unit"=>100,"upkeep"=>100},"compute"=>obj!{"cycles"=>50000,"thinkPeak"=>8500},
        "units"=>obj!{"total"=>8,"cap"=>40,"queued"=>0,"spawned"=>8,"byType"=>obj!{"miner"=>6,"bulwark"=>1,"scout"=>1}},
        "rts"=>obj!{"cpu"=>obj!{"income"=>50000,"bank"=>1000000,"spendCap"=>62500,"last"=>obj!{"upkeep"=>1000},
            "available"=>obj!{"think"=>20000,"control"=>50000,"fabrication"=>50000}}}};
    let args=views(&[world,me]);let orders=super::strategy(args[0],args[1]);
    let mut ram=Value::obj();
    for op in orders.get("ram").items(){if matches!(op.get("write"),Value::Num(_)){
        ram.set(&op.get("write").num().to_string(),obj!{"bytes"=>op.get("bytes").clone(),"t"=>600});
    }}
    let saved=views(&[ram]);let plan=mem::get(saved[0],0);
    assert_eq!(plan.get("attack").num(),1.0);
    assert!(!plan.get("mines").is_empty());
    for key in ["field","front","convoy"] {assert!(plan.get(key).is_undef(),"bounded planning omits {key}");}
    assert!(!orders.get("spawn").is_empty());
    assert!(orders.get("compute").get("cpu").get("limits").get("fabrication").num()>0.0);
    for key in ["research","rent","send","accept"] {assert!(orders.get(key).is_undef(),"standing science and trade remain unchanged: {key}");}
}

#[test]
fn interceptor_moves_to_the_owned_province_under_air_pressure() {
    let plan = obj! {"attack"=>2,"stage"=>0,"defend"=>-1,"airGuard"=>1,"threat"=>0};
    let world = obj! {
        "seconds"=>1, "factions"=>factions(),
        "provinces"=>arr![
            obj! {"id"=>0,"owner"=>1,"adj"=>arr![1]},
            obj! {"id"=>1,"owner"=>1,"adj"=>arr![0,2]},
            obj! {"id"=>2,"owner"=>2,"adj"=>arr![1]},
        ],
    };
    let me = obj! {"id"=>1,"capital"=>0,"ramProvince"=>obj! {"0"=>pages(&plan)},
        "rts"=>obj! {"cpu"=>obj! {"income"=>10000,"bank"=>100000}}};
    let unit = obj! {"id"=>101,"owner"=>1,"type"=>"interceptor","at"=>0,"to"=>-1,
        "doing"=>"idle","ramUnit"=>obj! {}};
    let args = views(&[unit, world, me]);
    let order = super::tactical_order(args[0], args[1], args[2], super::Role::Fast).expect("air guard order");
    assert_eq!(order.get("do").str(), "fight");
    assert_eq!(order.get("target").num(), 1.0, "interceptors protect the owned air-guard province before the attack");
}

#[test]
fn ground_tactical_unit_captures_the_planned_attack_when_unpressed() {
    let plan = obj! {"attack"=>2,"stage"=>0,"defend"=>-1,"airGuard"=>-1,"threat"=>0};
    let world = obj! {
        "seconds"=>1, "factions"=>factions(),
        "provinces"=>arr![
            obj! {"id"=>0,"owner"=>1,"adj"=>arr![1]},
            obj! {"id"=>1,"owner"=>1,"adj"=>arr![0,2]},
            obj! {"id"=>2,"owner"=>2,"adj"=>arr![1]},
        ],
    };
    let me = obj! {"id"=>1,"capital"=>0,"ramProvince"=>obj! {"0"=>pages(&plan)},
        "rts"=>obj! {"cpu"=>obj! {"income"=>10000,"bank"=>100000}}};
    let unit = obj! {"id"=>102,"owner"=>1,"type"=>"bulwark","at"=>0,"to"=>-1,
        "doing"=>"idle","ramUnit"=>obj! {}};
    let args = views(&[unit, world, me]);
    let order = super::tactical_order(args[0], args[1], args[2], super::Role::Wall).expect("capture order");
    assert_eq!(order.get("do").str(), "capture");
    assert_eq!(order.get("target").num(), 2.0);
}

#[test]
fn artillery_commits_its_route_from_a_shared_plan() {
    let plan = obj! {"attack"=>1,"stage"=>0,"defend"=>-1};
    let world = obj! {"seconds"=>1,"factions"=>factions(),"provinces"=>arr![
        obj! {"id"=>0,"owner"=>1,"adj"=>arr![1]}, obj! {"id"=>1,"owner"=>2,"adj"=>arr![0]},
    ]};
    let me = obj! {"id"=>1,"capital"=>0,"ramProvince"=>obj! {"0"=>pages(&plan)},
        "rts"=>obj! {"cpu"=>obj! {"income"=>10000,"bank"=>100000}}};
    let guns = arr![obj! {"id"=>103,"owner"=>1,"type"=>"artillery","at"=>0,"to"=>-1,
        "doing"=>"idle","rts"=>obj! {"deployed"=>false,"deploying"=>false},"ramUnit"=>obj! {}}];
    let args = views(&[guns, world, me]);
    super::artillery(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].get("do").str(), "fight");
    assert_eq!(orders[0].get("target").num(), 1.0);
    assert!(!orders[0].get("deploy").truthy());
}
