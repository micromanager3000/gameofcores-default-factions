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
    assert_eq!(orders,vec![obj!{"id"=>107,"cargo"=>Value::Null}],"keep the loaded hold; clear the stale job before rebuilding a route");
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
