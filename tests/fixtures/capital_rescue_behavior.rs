fn capital_rescue_case()->(Value,Value){
    let world=obj!{"seconds"=>3,"logistics"=>obj!{},"units"=>arr![
        obj!{"id"=>12,"owner"=>1,"type"=>"miner","at"=>4,"to"=>-1,"doing"=>"gather","build"=>obj!{"carry"=>5,"speed"=>3}},
        obj!{"id"=>13,"owner"=>1,"type"=>"miner","at"=>0,"to"=>-1,"doing"=>"gather","build"=>obj!{"carry"=>5,"speed"=>3}},
        obj!{"id"=>14,"owner"=>1,"type"=>"hauler","at"=>0,"to"=>-1,"build"=>obj!{},"rts"=>obj!{"cargo"=>obj!{"cycles"=>20000}}},
        obj!{"id"=>15,"owner"=>2,"type"=>"miner","at"=>0,"to"=>-1,"doing"=>"gather"},
        obj!{"id"=>16,"owner"=>1,"type"=>"miner","at"=>0,"to"=>-1,"doing"=>"gather","build"=>obj!{"wings"=>1}},
        obj!{"id"=>17,"owner"=>1,"type"=>"miner","at"=>0,"to"=>-1,"doing"=>"gather","rts"=>obj!{"sea"=>obj!{"ship"=>true}}}
    ]};
    let me=obj!{"id"=>1,"capital"=>0,"units"=>obj!{"total"=>30,"stalled"=>20,"upkeepRate"=>1.5},
        "costs"=>obj!{"draw"=>obj!{"worker"=>60}},"rts"=>obj!{"costs"=>obj!{"priceLevel"=>1,
            "upkeep"=>obj!{"base"=>100,"stat"=>30,"module"=>40}},
        "cpu"=>obj!{"income"=>20000,"bank"=>1000,"limits"=>obj!{"upkeep"=>0.8,"research"=>0.25}},
        "structureCosts"=>obj!{"maintenancePeriod"=>600},"structures"=>arr![
            obj!{"owner"=>1,"state"=>"ready","hp"=>100,"buildCost"=>60000,"progress"=>1}
        ]}};
    (world,me)
}

#[test]
fn a_starved_capital_funds_its_actual_delivery_crew_and_buildings(){
    let (world,me)=capital_rescue_case();let args=views(&[world,me]);
    let out=common::govern(args[0],args[1],0.4,||panic!("capital receipts precede new strategy"));
    let cpu=out.get("compute").get("cpu");
    assert_eq!(cpu.get("upkeepUnits"),&arr![13,14],"remote, foreign, flying and hull units do not enter the land crew");
    // Miner 220 and hauler 100, times the 1.5 over-cap rate, plus 120 artwork and 100 building upkeep.
    assert!(cpu.get("limits").get("upkeep").num()>=700.0);
    assert!(cpu.get("limits").get("upkeep").num()<800.0);
    assert!(cpu.get("reserve").is_empty());
    assert_eq!(cpu.get("limits").get("drive").num(),8000.0);
    assert_eq!(out.get("compute").get("types").get("hauler").get("cycles").num(),10000.0);
    assert!(out.get("ram").is_empty()&&out.get("disband").is_empty(),"standing jobs and cargo remain intact");
}

#[test]
fn a_funded_or_mildly_stalled_house_keeps_normal_planning(){
    for healthy in [true,false]{
        let (world,mut me)=capital_rescue_case();
        if healthy{me.entry("rts").entry("cpu").set("bank",40000);}else{me.entry("units").set("stalled",1);}
        let args=views(&[world,me]);
        let out=common::govern(args[0],args[1],0.4,||obj!{});
        assert!(out.get("compute").get("cpu").get("upkeepUnits").is_empty());
    }
}

#[test]
fn rescue_preserves_its_policy_between_reviews_and_releases_a_funded_bank(){
    for funded in [false,true]{
        let (mut world,mut me)=capital_rescue_case();world.set("seconds",4);
        let cpu=me.entry("rts").entry("cpu");cpu.set("upkeepUnits",arr![13,14]);
        if funded{cpu.set("bank",40000);}
        let args=views(&[world,me]);
        assert_eq!(common::capital_rescue_wait(args[0],args[1]),!funded);
        if !funded{
            let out=common::govern(args[0],args[1],0.4,||panic!("the paid capital jobs keep their policy"));
            assert!(out.is_empty(),"an empty response preserves the complete policy and standing jobs");
        }
    }
}
