fn crew_recruit_case(haulers: usize, away: usize, queued: usize, cap: usize, bill: f64, draw: f64, loans: Option<Value>, planned: Value) -> Value {
    crew_case(600000.0, haulers, away, queued, cap, bill, draw, loans, planned)
}
/// As [`crew_recruit_case`], with `stock` cycles in the fullest remote depot (20,000 sit in another).
fn crew_case(stock: f64, haulers: usize, away: usize, queued: usize, cap: usize, bill: f64, draw: f64, loans: Option<Value>, planned: Value) -> Value {
    let (mut world, mut me, _) = fixture();
    item(me.entry("rts").entry("logistics").entry("depots"), 2).set("cycles", stock);
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
    if let Some(loans)=loans {me.entry("units").set("hiredOutByType",loans);}
    let args=views(&[world,me]);
    common::govern(args[0],args[1],0.4,||planned)
}

#[test]
fn logistics_does_not_replace_a_crew_away_on_hire() {
    let orders=crew_recruit_case(5,4,0,30,2000.0,0.0,None,obj! {});
    assert!(orders.get("spawn").is_empty(),"returning hired bodies already fill the conservative crew allowance");
}

#[test]
fn affordable_initial_haulers_still_recruit() {
    let orders=crew_recruit_case(2,0,0,30,2000.0,0.0,None,obj! {});
    assert_eq!(orders.get("spawn").at(0).get("type").str(),"hauler");
    assert!(orders.get("compute").get("cpu").get("limits").get("fabrication").num()>0.0);
}

#[test]
fn logistics_respects_capacity_queues_and_upkeep_even_with_a_rich_bank() {
    for (away,queued,cap,bill,draw) in [(0,0,6,2000.0,0.0),(4,0,10,2000.0,0.0),(0,1,30,2000.0,0.0),(0,0,30,16000.0,0.0),(0,0,30,15000.0,1000.0)] {
        assert!(crew_recruit_case(2,away,queued,cap,bill,draw,None,obj! {}).get("spawn").is_empty(),
            "away={away}, queued={queued}, cap={cap}, upkeep={bill}");
    }
}

#[test]
fn logistics_preserves_funded_combat_instead_of_replacing_it() {
    for haulers in [2,6] {
        let spawn=arr![obj! {"type"=>"screen","build"=>obj! {"hp"=>2,"speed"=>2,"strength"=>2,"carry"=>0,"vision"=>2}}];
        let orders=crew_recruit_case(haulers,0,0,30,2000.0,0.0,None,obj! {"spawn"=>spawn,
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

#[test]
fn typed_outgoing_hires_leave_room_for_a_needed_hauler() {
    let loans=obj! {"hauler"=>3,"screen"=>4};
    let orders=crew_recruit_case(2,7,0,30,2000.0,0.0,Some(loans.clone()),obj! {});
    assert_eq!(orders.get("spawn").at(0).get("type").str(),"hauler");
    assert!(crew_recruit_case(6,7,0,30,2000.0,0.0,Some(loans),obj! {}).get("spawn").is_empty());
    assert!(crew_recruit_case(2,7,0,30,2000.0,0.0,None,obj! {}).get("spawn").is_empty(),
        "older views conservatively count every outgoing body against the crew allowance");
    let empty=crew_recruit_case(2,7,0,30,2000.0,0.0,Some(obj! {}),obj! {});
    assert_eq!(empty.get("spawn").at(0).get("type").str(),"hauler");
}

#[test]
fn with_cargo_stranded_a_short_crew_hires_a_hauler_before_other_recruits() {
    let spawn=arr![obj! {"type"=>"screen","build"=>obj! {"hp"=>2,"speed"=>2,"strength"=>2,"carry"=>0,"vision"=>2}}];
    let planned=obj! {"spawn"=>spawn,"compute"=>obj! {"cpu"=>obj! {"limits"=>obj! {"fabrication"=>12000}}}};
    // 4M stranded: the hauler replaces the soldier, even with a recruit queued
    let orders=crew_case(4.0e6,2,0,1,30,2000.0,0.0,None,planned.clone());
    assert_eq!(orders.get("spawn").len(),1);
    assert_eq!(orders.get("spawn").at(0).get("type").str(),"hauler");
    // a full crew leaves the soldier alone
    let orders=crew_case(4.0e6,12,0,0,30,2000.0,0.0,None,planned);
    assert_eq!(orders.get("spawn").at(0).get("type").str(),"screen");
}

#[test]
fn rival_buildings_in_sight_do_not_stop_a_short_crew_hiring() {
    // 40 rival sites in sight and 2 of the house's own, with 20,000 cycles left in the call: the bill is scanned in
    // the cycles there are, not reserved at 800 a site listed
    let case=|structures:usize|{
        let (mut world,mut me,_)=fixture();
        item(me.entry("rts").entry("logistics").entry("depots"),2).set("cycles",600000.0);
        world.set("seconds",3);
        let mut units=Value::arr();
        for id in 0..6 {
            units.push(obj! {"id"=>id,"owner"=>1,"at"=>0,"to"=>-1,"type"=>if id<4 {"miner"} else {"hauler"},
                "build"=>obj! {"hp"=>2,"speed"=>2,"strength"=>0,"carry"=>5,"vision"=>1}});
        }
        world.set("units",units);
        let mut sites=Value::arr();
        for i in 0..structures {
            sites.push(obj! {"id"=>i,"owner"=>if i<2 {1} else {2},"p"=>0,"state"=>"built","hp"=>100,"buildCost"=>6000});
        }
        me.entry("rts").set("structures",sites);
        me.set("provinces",12);
        me.entry("techs").set("seaTraversal",false);
        me.entry("rts").entry("cpu").set("bank",1000000);
        me.entry("rts").entry("cpu").entry("last").set("upkeep",2000.0);
        me.set("units",obj! {"total"=>6,"hiredOut"=>0,"queued"=>0,"cap"=>30,"stalled"=>0,"byType"=>obj! {"miner"=>4,"hauler"=>2}});
        let args=views(&[world,me]);
        gameofcores::with_cycle_reserve(gameofcores::cycles_left()-20000.0,||common::govern(args[0],args[1],0.4,||obj! {}))
    };
    assert_eq!(case(2).get("spawn").at(0).get("type").str(),"hauler");
    assert_eq!(case(42).get("spawn").at(0).get("type").str(),"hauler");
}
