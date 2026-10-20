fn approach_case() -> (Value, Value, Value) {
    let (mut world, me, mut units) = fixture();
    item(world.entry("provinces"),0).set("mines",arr![]);
    item(world.entry("provinces"),0).set("map","o0".repeat(217));
    let u=item(&mut units,0);
    u.set("type","hauler");u.set("to",-1);u.set("doing","idle");
    u.set("pos",obj!{"p"=>0,"x"=>0,"y"=>-1.7});
    u.entry("rts").set("sea",Value::obj());
    u.entry("rts").set("cargo",obj!{"cycles"=>20000,"capacity"=>50000,
        "job"=>obj!{"status"=>"approaching","operation"=>obj!{"unload"=>obj!{"p"=>0,"dock"=>false}},"remaining"=>20000}});
    (world,me,units)
}
fn cargo_orders(world:Value,me:Value,units:Value)->Vec<Value>{
    let args=views(&[units,world,me]);
    common::haul_cargo(args[0],args[1],args[2]);
    abi::TEST_ORDERS.with(|o|o.take()).iter().map(|s|Value::parse(s)).collect()
}
#[test]
fn resting_land_approach_releases_only_its_job_then_retries_the_loaded_delivery(){
    let (world,me,mut units)=approach_case();
    let orders=cargo_orders(world.clone(),me.clone(),units.clone());
    assert_eq!(orders,vec![obj!{"id"=>107,"cargo"=>Value::Null,"do"=>"move","target"=>0,
        "at"=>obj!{"p"=>0,"x"=>0,"y"=>-1.7}}],"keep the hold and clear the completed way by targeting the carrier's current position");
    item(&mut units,0).entry("rts").entry("cargo").set("job",Value::Null);
    let orders=cargo_orders(world,me,units);
    assert_eq!(orders.len(),1);
    assert_eq!(orders[0].get("do").str(),"move");
    assert_eq!(orders[0].get("cargo").get("unload").get("p").num(),0.0);
    assert!(!orders[0].get("at").is_empty(),"the retry uses a real local route");
}
#[test]
fn active_approaches_and_unpaid_or_ship_jobs_keep_their_standing_work(){
    for kind in ["crossing","way","walking","ship","unpaid","unloading"]{
        let (world,me,mut units)=approach_case();let u=item(&mut units,0);
        match kind{
            "crossing"=>u.set("to",1),
            "way"=>u.set("way",arr![obj!{"x"=>0,"y"=>0}]),
            "walking"=>u.set("doing","move"),
            "ship"=>u.entry("rts").entry("sea").set("ship",true),
            status=>u.entry("rts").entry("cargo").entry("job").set("status",status),
        }
        assert!(cargo_orders(world,me,units).is_empty(),"{kind}");
    }
}
#[test]
fn recovery_funds_a_loaded_home_hauler_before_idle_miners_without_reserving_income_for_optional_code(){
    let (mut world,mut me,units)=approach_case();
    world.set("seconds",3);world.set("units",units.clone());
    me.entry("units").set("stalled",1);me.entry("rts").entry("cpu").set("bank",1000);
    let args=views(&[world,me]);
    let out=common::govern(args[0],args[1],0.4,||panic!("the unpaid army needs recovery"));
    let cpu=out.get("compute").get("cpu");
    assert_eq!(cpu.get("upkeepOrder").at(0).str(),"hauler");
    assert_eq!(cpu.get("limits").get("upkeep").num(),0.65);
    assert_eq!(cpu.get("reserve").get("upkeep").num(),200.0/40000.0);
    assert!(cpu.get("reserve").get("think").is_undef());
    assert!(cpu.get("reserve").get("duel").is_undef());
    assert_eq!(out.get("compute").get("types").get("miner").get("share").num(),0.0);
    assert_eq!(out.get("compute").get("types").get("hauler").get("cycles").num(),10000.0);
    assert!(out.get("disband").is_empty(),"the cargo and its carriers stay intact");
}
#[test]
fn a_healthy_cargo_policy_keeps_its_normal_reserves_and_miner_controllers(){
    let (mut world,mut me,units)=approach_case();world.set("units",units);world.set("seconds",3);
    me.entry("rts").entry("cpu").set("bank",400000);
    let args=views(&[world,me]);let out=common::govern(args[0],args[1],0.4,||obj!{});
    let cpu=out.get("compute").get("cpu");
    assert_eq!(cpu.get("reserve").get("think").num(),0.25);
    assert_eq!(cpu.get("reserve").get("duel").num(),0.05);
    assert_eq!(cpu.get("upkeepOrder").at(0).str(),"miner");
    assert_eq!(out.get("compute").get("types").get("miner").get("cycles").num(),6000.0);
}
#[test]
fn a_funded_planner_retry_keeps_normal_reserves_and_working_recovery_keeps_miner_priority(){
    for rich in [false,true]{
        let (mut world,mut me,units)=approach_case();world.set("units",units);world.set("seconds",3);
        me.entry("rts").entry("cpu").set("bank",if rich {400000}else{1000});
        me.entry("units").set("stalled",if rich {0}else{1});
        if !rich {me.entry("units").entry("doing").set("gather",1);}
        let args=views(&[world,me]);
        let out=common::govern(args[0],args[1],0.4,||common::recovery(args[0],args[1]));
        let cpu=out.get("compute").get("cpu");
        assert_eq!(cpu.get("upkeepOrder").at(0).str(),"miner");
        assert_eq!(out.get("compute").get("types").get("miner").get("cycles").num(),6000.0);
        if rich {
            assert_eq!(cpu.get("reserve").get("upkeep").num(),0.1);
            assert_eq!(cpu.get("reserve").get("think").num(),0.25);
        }
    }
}

#[test]
fn unpaid_capital_deliveries_get_upkeep_even_when_miners_keep_gather_orders(){
    for ty in ["hauler","freighter"] {
        let (mut world,mut me,mut units)=approach_case();
        world.set("seconds",3);
        let u=item(&mut units,0);
        u.set("type",ty);u.set("at",1);u.set("to",0);u.set("stalled",true);
        u.entry("rts").entry("cargo").entry("job").set("status","unpaid");
        world.set("units",units);
        me.entry("units").set("stalled",1);me.entry("units").entry("doing").set("gather",12);
        me.entry("rts").entry("cpu").set("bank",1000);
        let args=views(&[world,me]);
        let out=common::govern(args[0],args[1],0.4,||panic!("blocked deliveries need recovery"));
        assert_eq!(out.get("compute").get("cpu").get("upkeepOrder"),&arr!["hauler","freighter","miner"]);
        assert!(out.get("disband").is_empty(),"loaded carriers must survive recovery");
        assert_eq!(out.get("compute").get("types").get("miner").get("share").num(),0.0);
    }
}

#[test]
fn empty_or_remote_carriers_do_not_displace_working_miners(){
    for case in ["empty","remote","paid","other-owner"] {
        let (mut world,mut me,mut units)=approach_case();
        world.set("seconds",3);
        let u=item(&mut units,0);u.set("at",1);u.set("to",0);u.set("stalled",true);
        u.entry("rts").entry("cargo").entry("job").set("status","unpaid");
        match case {
            "empty"=>u.entry("rts").entry("cargo").set("cycles",0),
            "remote"=>u.entry("rts").entry("cargo").entry("job").entry("operation").entry("unload").set("p",1),
            "paid"=>{u.set("stalled",false);u.entry("rts").entry("cargo").entry("job").set("status","approaching");},
            _=>u.set("owner",7),
        }
        world.set("units",units);
        me.entry("units").set("stalled",1);me.entry("units").entry("doing").set("gather",12);
        me.entry("rts").entry("cpu").set("bank",1000);
        let args=views(&[world,me]);
        let out=common::govern(args[0],args[1],0.4,||panic!("the unpaid army needs recovery"));
        assert_eq!(out.get("compute").get("cpu").get("upkeepOrder").at(0).str(),"miner","{case}");
    }
}

#[test]
fn a_delivery_bridge_preserves_fielded_aircraft_landing_control() {
    let (mut world,mut me,mut units)=approach_case();
    world.set("seconds",3);
    units.push(obj!{"id"=>901,"owner"=>me.get("id").clone(),"type"=>"interceptor","build"=>obj!{"wings"=>2}});
    units.push(obj!{"id"=>902,"owner"=>me.get("id").clone(),"type"=>"aeronaut","build"=>obj!{"wings"=>2}});
    units.push(obj!{"id"=>903,"owner"=>7,"type"=>"foreign-wing","build"=>obj!{"wings"=>2}});
    world.set("units",units);
    me.entry("units").set("stalled",1);me.entry("rts").entry("cpu").set("bank",1000);
    let args=views(&[world,me]);
    let out=common::govern(args[0],args[1],0.4,||panic!("the loaded delivery needs recovery"));
    let types=out.get("compute").get("types");
    for ty in ["interceptor","aeronaut"] {
        assert_eq!(types.get(ty),&obj!{"cycles"=>2400,"every"=>3},"{ty}");
    }
    assert!(types.get("foreign-wing").is_undef());
    assert_eq!(types.get("hauler").get("cycles").num(),10000.0);
    assert_eq!(types.get("freighter").get("cycles").num(),12000.0);
    assert_eq!(out.get("compute").get("cpu").get("upkeepOrder"),&arr!["hauler","freighter","miner"]);
}

#[test]
fn ordinary_miner_budget_can_route_one_idle_worker_to_its_planned_remote_mine() {
    let mut pages = Value::obj();
    for op in mem::put(0, &obj! { "mines" => arr![arr![18, 0]] }, "shared", None, 1).unwrap() {
        pages.set(&op.get("write").num().to_string(), obj! { "bytes" => op.get("bytes").clone(), "t" => 1 });
    }
    let units = arr![obj! { "id" => 1410, "owner" => 2, "type" => "miner", "at" => 0,
        "to" => -1, "doing" => "idle", "rts" => obj! { "cargo" => obj! { "cycles" => 0 } } }];
    let world = obj! { "tick" => 1, "logistics" => obj! {}, "provinces" => arr![
        obj! { "id" => 0, "owner" => 2, "adj" => arr![18], "mines" => arr![1], "staff" => arr![2] },
        obj! { "id" => 18, "owner" => 2, "adj" => arr![0], "mines" => arr![1], "staff" => arr![0] }
    ] };
    let me = obj! { "id" => 2, "capital" => 0, "ramProvince" => obj! { "0" => pages } };
    let args = views(&[units, world, me]);
    assert!(common::recover_miners(args[0], args[1], args[2]));
    let orders: Vec<Value> = abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o| Value::parse(o)).collect();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("id").num(), 1410.0);
    assert_eq!(orders[0].get("do").str(), "gather");
    assert_eq!(orders[0].get("target").num(), 18.0);
    assert_eq!(orders[0].get("via"), &arr![18]);
}

#[test]
fn a_funded_miner_routes_remote_work_without_replacing_existing_gather_jobs() {
    let mut pages=Value::obj();
    for op in mem::put(0,&obj! {"mines"=>arr![arr![1,0]]},"shared",None,1).unwrap() {
        pages.set(&op.get("write").num().to_string(),obj! {"bytes"=>op.get("bytes").clone(),"t"=>1});
    }
    let units=arr![obj! {"id"=>1,"owner"=>2,"at"=>0,"doing"=>"gather","target"=>0},
        obj! {"id"=>2,"owner"=>2,"at"=>0,"to"=>-1,"doing"=>"idle","rts"=>obj! {"cargo"=>obj! {"cycles"=>0}}}];
    let world=obj! {"tick"=>1,"logistics"=>obj! {},"provinces"=>arr![
        obj! {"id"=>0,"owner"=>2,"adj"=>arr![1],"mines"=>arr![1],"staff"=>arr![2]},
        obj! {"id"=>1,"owner"=>2,"adj"=>arr![0],"mines"=>arr![1],"staff"=>arr![0]}]};
    let me=obj! {"id"=>2,"capital"=>0,"ramProvince"=>obj! {"0"=>pages}};
    let args=views(&[units,world,me]);
    assert!(common::recover_miners(args[0],args[1],args[2]));
    let out:Vec<Value>=abi::TEST_ORDERS.with(|o|o.take()).iter().map(|o|Value::parse(o)).collect();
    assert_eq!(out.len(),1,"{out:?}");
    assert_eq!(out[0].get("id").num(),2.0);
    assert_eq!(out[0].get("target").num(),1.0);
}

#[test]
fn a_loaded_hauler_at_a_full_depot_uses_a_reachable_capital_instead_of_waiting_forever() {
    for reachable in [true,false] {
        let (mut world,mut me,mut units)=approach_case();
        let mut home=world.get("provinces").at(0).clone();home.set("id",1);
        home.set("owner",if reachable {1}else{7});home.set("adj",arr![0]);
        item(world.entry("provinces"),0).set("adj",arr![1]);world.entry("provinces").push(home);
        me.set("capital",1);
        let u=item(&mut units,0);
        u.entry("rts").entry("cargo").set("job",obj!{"status"=>"depot full",
            "operation"=>obj!{"unload"=>obj!{"p"=>0,"dock"=>true}},"remaining"=>20000});
        let orders=cargo_orders(world,me,units);
        if reachable {
            assert_eq!(orders.len(),1,"the full load gets one replacement delivery: {orders:?}");
            assert_eq!(orders[0].get("cargo").get("unload"),&obj!{"p"=>1,"dock"=>false});
            assert_eq!(orders[0].get("via"),&arr![1]);
        } else {
            assert!(orders.is_empty(),"keep the cargo job when the capital cannot be reached");
        }
    }
}
