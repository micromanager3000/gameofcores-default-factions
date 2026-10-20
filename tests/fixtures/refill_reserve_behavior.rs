fn refill_reserve_policy()->Value{
    obj!{"compute"=>obj!{"cpu"=>obj!{"limits"=>obj!{"think"=>40000,"upkeep"=>0.8},
        "reserve"=>obj!{"upkeep"=>0.3,"control"=>20000},"upkeepUnits"=>arr![12,13]},
        "types"=>obj!{"miner"=>obj!{"cycles"=>6000,"every"=>2}}}}
}

#[test]
fn cargo_recovery_cannot_reserve_more_than_its_next_automatic_refill(){
    let me=obj!{"compute"=>obj!{"rented"=>30000},"rts"=>obj!{"cpu"=>obj!{"income"=>100000,"bank"=>70000},
        "logistics"=>obj!{"deliveryRate"=>58000}}};
    let args=views(&[me]);let mut out=refill_reserve_policy();common::fit_refill_reserves(&mut out,args[0]);
    let cpu=out.get("compute").get("cpu");let reserve=cpu.get("reserve");
    let promised=reserve.get("upkeep").num()+reserve.get("control").num();
    assert!(promised<=6000.0,"estimated deliveries and rented cycles are not part of the 12,000-cycle automatic refill");
    assert!(12000.0-promised>=220.0+2000.0,"the next read and a bounded recovery callback remain possible");
    assert_eq!(cpu.get("limits"),refill_reserve_policy().get("compute").get("cpu").get("limits"));
    assert_eq!(cpu.get("upkeepUnits"),&arr![12,13]);
    assert_eq!(out.get("compute").get("types"),refill_reserve_policy().get("compute").get("types"));
}

#[test]
fn wealth_and_non_logistics_worlds_preserve_the_authored_reserves(){
    for cargo in [true,false]{
        let mut me=obj!{"rts"=>obj!{"cpu"=>obj!{"income"=>100000,"bank"=>if cargo{400000}else{70000}}}};
        if cargo{me.entry("rts").set("logistics",obj!{"deliveryRate"=>88000});}
        let args=views(&[me]);let mut out=refill_reserve_policy();let before=out.clone();
        common::fit_refill_reserves(&mut out,args[0]);assert_eq!(out,before);
    }
}

#[test]
fn recovery_after_a_recent_delivery_still_reserves_only_automatic_refills(){
    let me=obj!{"compute"=>obj!{"rented"=>0},"rts"=>obj!{"cpu"=>obj!{"income"=>100000,"bank"=>600000},
        "logistics"=>obj!{"deliveryRate"=>88000}}};
    let args=views(&[me]);let mut out=refill_reserve_policy();out.set("memo","Recovering production");
    let before=out.clone();common::fit_refill_reserves(&mut out,args[0]);
    let cpu=out.get("compute").get("cpu");let reserve=cpu.get("reserve");
    assert!(reserve.get("upkeep").num()+reserve.get("control").num()<=6000.0,
        "the recent cargo balance cannot promise future deliveries while recovery is active");
    assert_eq!(cpu.get("limits"),before.get("compute").get("cpu").get("limits"));
    assert_eq!(cpu.get("upkeepUnits"),before.get("compute").get("cpu").get("upkeepUnits"));
    assert_eq!(out.get("compute").get("types"),before.get("compute").get("types"));
}
