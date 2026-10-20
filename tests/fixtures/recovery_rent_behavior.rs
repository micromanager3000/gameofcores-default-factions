fn recovery_rental_view(coin:f64,quote:f64)->Value{
    obj!{"bank"=>coin,"costs"=>obj!{"unit"=>100},
        "compute"=>obj!{"rented"=>30000,"rentRate"=>1000,"rentMax"=>quote},
        "rts"=>obj!{"cpu"=>obj!{"income"=>100000,"bank"=>70000},
            "logistics"=>obj!{"deliveryRate"=>58000}}}
}

#[test]
fn new_recovery_rent_fits_currency_and_automatic_income(){
    for (coin,quote,expected) in [(50000.0,40000.0,6.0),(350.0,40000.0,50.0/120.0),(50000.0,3000.0,3.0)]{
        let args=views(&[recovery_rental_view(coin,quote)]);
        let mut out=refill_reserve_policy();let compute=out.get("compute").clone();
        common::recovery_rent(&mut out,args[0]);let pay=out.get("rent").num();
        assert!((pay-expected).abs()<1e-9);
        assert!(pay*120.0<=coin-300.0+1e-8,"keep two unit prices, 100 currency and 120 seconds of rent");
        assert!(pay*1000.0<=6000.0 && pay*1000.0<=quote,"cargo receipts and existing rent cannot raise the automatic-income cap");
        assert_eq!(out.get("compute"),&compute,"rental recovery keeps the complete CPU policy and type schedules");
    }
}

#[test]
fn recovery_rental_scope_preserves_existing_choices(){
    for case in ["wealthy","no-cargo","no-rate","no-policy","larger-request"]{
        let mut me=recovery_rental_view(50000.0,40000.0);let mut out=refill_reserve_policy();
        match case{
            "wealthy"=>{me.entry("rts").entry("cpu").set("bank",400000);},
            "no-cargo"=>{me.entry("rts").set("logistics",Value::Null);},
            "no-rate"=>{me.entry("compute").set("rentRate",0);},
            "no-policy"=>{out=Value::obj();},
            _=>{out.set("rent",12);},
        }
        let args=views(&[me]);let before=out.clone();common::recovery_rent(&mut out,args[0]);
        assert_eq!(out,before,"{case}");
    }
}
