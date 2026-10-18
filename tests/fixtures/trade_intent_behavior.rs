fn trade_intent_case(attack:f64)->(Value,Value,Value) {
    let world=obj!{"seconds"=>1000,"factions"=>arr![Value::Null,obj!{"id"=>1,"alive"=>true},obj!{"id"=>2,"alive"=>true}],
        "provinces"=>arr![obj!{"id"=>0,"owner"=>1,"hops"=>0,"adj"=>arr![1]},
            obj!{"id"=>1,"owner"=>2,"visible"=>true,"adj"=>arr![0],"cores"=>2,"mines"=>arr![1,1],"staff"=>arr![0,0],"units"=>obj!{}}],
        "units"=>arr![]};
    let ram=ram_pages(&obj!{"ram"=>mem::put(0,&obj!{"attack"=>attack,"stage"=>0},"shared",None,16).unwrap()});
    let me=obj!{"id"=>1,"capital"=>0,"bank"=>1000,"allies"=>arr![],"contracts"=>arr![],"ramCapital"=>ram,
        "units"=>obj!{"total"=>8,"queued"=>0,"cap"=>24},"techs"=>obj!{}};
    let offer=obj!{"id"=>7,"from"=>2,"give"=>obj!{"mines"=>arr![obj!{"province"=>1,"slots"=>2,"seconds"=>300}]},"ask"=>obj!{"coin"=>10}};
    (world,me,offer)
}
fn frontier_ids(world:&Value,me:&Value)->Vec<f64> {
    let args=views(&[world.clone(),me.clone()]);
    common::frontier(args[0],args[1]).iter().map(|p|p.get("id").num()).collect()
}
fn trade_is_fair(offer:&Value,world:&Value,me:&Value)->bool {
    let args=views(&[offer.clone(),me.clone(),world.clone()]);common::fair(args[0],args[1],args[2],&[])
}

#[test]
fn incoming_mine_leases_hide_targets_until_even_ended_contract_grace_expires() {
    let (mut world,mut me,_)=trade_intent_case(-1.0);
    item(world.entry("provinces"),1).set("leased",arr![obj!{"slot"=>0,"to"=>1}]);
    assert!(frontier_ids(&world,&me).is_empty(),"the public active lease alone protects the target");
    item(world.entry("provinces"),1).set("leased",arr![]);
    me.set("contracts",arr![obj!{"with"=>2,"ended"=>obj!{"t"=>990,"by"=>2},"graceIn"=>20,
        "mines"=>arr![obj!{"province"=>1,"from"=>2,"to"=>1,"endsIn"=>0}]}]);
    assert!(frontier_ids(&world,&me).is_empty(),"safe conduct survives an ended lease");
    item(me.entry("contracts"),0).set("graceIn",0);
    assert_eq!(frontier_ids(&world,&me),vec![1.0]);
    item(me.entry("contracts"),0).set("graceIn",20);
    item(item(me.entry("contracts"),0).entry("mines"),0).set("to",2);
    assert_eq!(frontier_ids(&world,&me),vec![1.0],"leasing mines out does not cover our attackers");
}

#[test]
fn passage_eligibility_uses_incoming_rights_and_their_actual_grace() {
    let (world,mut me,_)=trade_intent_case(-1.0);
    me.set("contracts",arr![obj!{"with"=>2,"get"=>obj!{"passage"=>true},"ended"=>obj!{"by"=>2},"graceIn"=>10}]);
    assert!(frontier_ids(&world,&me).is_empty());
    item(me.entry("contracts"),0).set("graceIn",0);
    assert_eq!(frontier_ids(&world,&me),vec![1.0]);
    item(me.entry("contracts"),0).set("graceIn",10);
    item(me.entry("contracts"),0).set("ended",obj!{"by"=>1,"why"=>"broke its deal by attacking"});
    assert_eq!(frontier_ids(&world,&me),vec![1.0],"attacking passage holders have no grace");
    me.set("contracts",arr![obj!{"with"=>2,"give"=>obj!{"passage"=>true},"graceIn"=>10}]);
    assert_eq!(frontier_ids(&world,&me),vec![1.0],"selling passage does not cover our own force");
}

#[test]
fn a_saved_capture_plan_declines_protection_but_keeps_productive_trade() {
    let (world,me,offer)=trade_intent_case(1.0);
    assert!(!trade_is_fair(&offer,&world,&me),"a lease cannot freeze a planned siege before it starts");
    let incoming=obj!{"from"=>2,"give"=>obj!{"passage"=>obj!{"units"=>10,"seconds"=>300}},"ask"=>obj!{"coin"=>1}};
    assert!(!trade_is_fair(&incoming,&world,&me));
    let outgoing=obj!{"from"=>2,"give"=>obj!{"coin"=>100},"ask"=>obj!{"passage"=>obj!{"units"=>10,"seconds"=>60}}};
    assert!(trade_is_fair(&outgoing,&world,&me),"outgoing passage does not protect our capturers");
    let lease_out=obj!{"from"=>2,"give"=>obj!{"coin"=>100},"ask"=>obj!{"mines"=>arr![obj!{"province"=>0,"slots"=>1,"seconds"=>60}]}};
    assert!(trade_is_fair(&lease_out,&world,&me),"selling a mine lease does not cover our own force");
    let (_,idle_me,_)=trade_intent_case(-1.0);
    assert!(trade_is_fair(&offer,&world,&idle_me),"an affordable productive lease remains useful without capture intent");
}

#[test]
fn standing_capture_intent_declines_renewal_when_the_frontier_hides_it() {
    let (mut world,mut me,offer)=trade_intent_case(-1.0);
    world.set("units",arr![obj!{"id"=>9,"owner"=>1,"at"=>1,"target"=>1,"doing"=>"capture","stalled"=>false}]);
    me.set("contracts",arr![obj!{"with"=>2,"get"=>obj!{"mines"=>arr![obj!{"province"=>1,"slots"=>2,"seconds"=>300}]},"endsIn"=>200,"graceIn"=>230,
        "mines"=>arr![obj!{"province"=>1,"from"=>2,"to"=>1,"endsIn"=>200}]}]);
    assert!(frontier_ids(&world,&me).is_empty());
    assert!(!trade_is_fair(&offer,&world,&me),"covered capture orders retain intent even without a saved attack or live siege");
    item(me.entry("contracts"),0).set("ended",obj!{"by"=>2,"t"=>990});
    item(me.entry("contracts"),0).set("graceIn",10);
    assert!(frontier_ids(&world,&me).is_empty());
    item(me.entry("contracts"),0).set("graceIn",0);
    assert_eq!(frontier_ids(&world,&me),vec![1.0],"the objective is eligible after grace if the lease is not renewed");
}
