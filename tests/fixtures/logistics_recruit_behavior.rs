fn crew_recruit_case(haulers: usize, away: usize, queued: usize, cap: usize, bill: f64, draw: f64, planned: Value) -> Value {
    let (mut world, mut me, _) = fixture();
    world.set("seconds", 3);
    let total=haulers+4;
    let mut units=Value::arr();
    for id in 0..total {
        units.push(obj! {"id"=>id,"owner"=>1,"at"=>0,"to"=>-1,"type"=>if id<4 {"miner"} else {"hauler"},
            "build"=>obj! {"hp"=>2,"speed"=>2,"strength"=>0,"carry"=>5,"vision"=>1}});
    }
    world.set("units",units);
    me.set("provinces",12);
    me.entry("costs").set("draw",obj! {"worker"=>draw});
    me.entry("techs").set("seaTraversal",false);
    me.entry("rts").entry("cpu").set("bank",1000000);
    me.entry("rts").entry("cpu").entry("last").set("upkeep",bill);
    me.set("units",obj! {"total"=>total,"hiredOut"=>away,"queued"=>queued,"cap"=>cap,
        "stalled"=>0,"byType"=>obj! {"miner"=>4,"hauler"=>haulers}});
    let args=views(&[world,me]);
    common::govern(args[0],args[1],0.4,||planned)
}

#[test]
fn logistics_does_not_replace_a_crew_away_on_hire() {
    let orders=crew_recruit_case(4,4,0,30,2000.0,0.0,obj! {});
    assert!(orders.get("spawn").is_empty(),"returning hired bodies already fill the conservative crew allowance");
}

#[test]
fn affordable_initial_haulers_still_recruit() {
    let orders=crew_recruit_case(2,0,0,30,2000.0,0.0,obj! {});
    assert_eq!(orders.get("spawn").at(0).get("type").str(),"hauler");
    assert!(orders.get("compute").get("cpu").get("limits").get("fabrication").num()>0.0);
}

#[test]
fn logistics_respects_capacity_queues_and_upkeep_even_with_a_rich_bank() {
    for (away,queued,cap,bill,draw) in [(0,0,6,2000.0,0.0),(4,0,10,2000.0,0.0),(0,1,30,2000.0,0.0),(0,0,30,16000.0,0.0),(0,0,30,15000.0,1000.0)] {
        assert!(crew_recruit_case(2,away,queued,cap,bill,draw,obj! {}).get("spawn").is_empty(),
            "away={away}, queued={queued}, cap={cap}, upkeep={bill}");
    }
}

#[test]
fn logistics_preserves_funded_combat_instead_of_replacing_it() {
    for haulers in [2,6] {
        let spawn=arr![obj! {"type"=>"screen","build"=>obj! {"hp"=>2,"speed"=>2,"strength"=>2,"carry"=>0,"vision"=>2}}];
        let orders=crew_recruit_case(haulers,0,0,30,2000.0,0.0,obj! {"spawn"=>spawn,
            "compute"=>obj! {"cpu"=>obj! {"limits"=>obj! {"fabrication"=>12000}}}});
        assert_eq!(orders.get("spawn").len(),1);
        assert_eq!(orders.get("spawn").at(0).get("type").str(),"screen");
        assert_eq!(orders.get("compute").get("cpu").get("limits").get("fabrication").num(),12000.0);
    }
}

#[test]
fn logistics_preserves_the_recovery_miner_before_optional_crew() {
    let (mut world,mut me,_)=fixture();
    world.set("seconds",3);
    me.entry("rts").entry("cpu").set("income",20000);
    me.entry("rts").entry("cpu").set("bank",1000000);
    me.set("units",obj! {"total"=>1,"queued"=>0,"cap"=>20,"byType"=>obj! {"miner"=>1}});
    let args=views(&[world,me]);
    let orders=common::govern(args[0],args[1],0.4,||panic!("the starting crew uses recovery"));
    assert_eq!(orders.get("spawn").at(0).get("type").str(),"miner");
}
