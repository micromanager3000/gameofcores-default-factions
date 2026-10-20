fn recovery_timer_view(stalled:f64,bank:f64)->Value{
    let mut pages=Value::obj();
    for op in mem::put(244,&obj!{"r"=>true,"s"=>10},"local",None,1).expect("recovery history"){
        pages.set(&op.get("write").num().to_string(),obj!{"bytes"=>op.get("bytes").clone(),"t"=>10});
    }
    obj!{"id"=>1,"capital"=>0,"bank"=>1000,"ramCapital"=>pages,"types"=>obj!{},
        "units"=>obj!{"total"=>30,"stalled"=>stalled,"cap"=>40,"queued"=>0,"byType"=>obj!{"miner"=>8}},
        "costs"=>obj!{"unit"=>100,"upkeep"=>100},"compute"=>obj!{"thinkPeak"=>12000},
        "rts"=>obj!{"logistics"=>obj!{"deliveryRate"=>90000},"cpu"=>obj!{"income"=>100000,"bank"=>bank,
            "last"=>obj!{"upkeep"=>1000},"limits"=>obj!{"upkeep"=>0.8}}}}
}

#[test]
fn a_cargo_receipt_does_not_restart_an_unpaid_crews_wait(){
    for stalled in [0.0,10.0]{
        let world=obj!{"seconds"=>123,"logistics"=>obj!{},"units"=>arr![]};
        let args=views(&[world,recovery_timer_view(stalled,300000.0)]);
        let out=common::govern(args[0],args[1],0.4,||panic!("this bank still needs recovery"));
        let page=ram_pages(&out);let decoded=mem::get(views(&[page])[0],244);
        assert_eq!(decoded.get("s").num(),if stalled>0.0{10.0}else{123.0},
            "a healthy-looking receipt cannot erase elapsed unpaid time; a paid crew starts a fresh interval");
    }
}

#[test]
fn normal_planning_keeps_recovery_history_until_all_units_are_paid(){
    for stalled in [0.0,1.0]{
        let world=obj!{"seconds"=>126,"logistics"=>obj!{},"units"=>arr![]};
        let args=views(&[world,recovery_timer_view(stalled,1000000.0)]);
        let out=common::govern(args[0],args[1],0.4,||obj!{"memo"=>"normal planning"});
        assert_eq!(out.get("memo").str(),"normal planning","savings allow the strategy to resume");
        let cleared=out.get("ram").items().iter().any(|op|op.get("free").num_or(-1.0)==244.0);
        assert_eq!(cleared,stalled==0.0,"one unpaid unit keeps the existing wait history");
    }
}
