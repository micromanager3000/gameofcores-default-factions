fn building_service_orders(stalled: f64, bank: f64, recovering: bool) -> Value {
    let world = obj! { "seconds"=>6, "provinces"=>arr![obj! {"id"=>0,"owner"=>1}],
        "units"=>arr![obj! {"id"=>20,"owner"=>1,"type"=>"line","at"=>0,"to"=>-1,
            "pos"=>obj! {"p"=>0,"x"=>1,"y"=>0},"hp"=>4,"hpMax"=>10,
            "build"=>obj! {"hp"=>2,"speed"=>2,"strength"=>2,"carry"=>2,"vision"=>2}}] };
    let me = obj! { "id"=>1,"seat"=>1,"units"=>obj! {"stalled"=>stalled},
        "rts"=>obj! {"structureCosts"=>obj! {"maintenancePeriod"=>600},
            "costs"=>obj! {"priceLevel"=>1,"upkeep"=>obj! {"base"=>100}},
            "cpu"=>obj! {"income"=>10000,"bank"=>bank,"last"=>obj! {"upkeep"=>2000}},
            "structures"=>arr![obj! {"id"=>10,"owner"=>1,"p"=>0,"x"=>0,"y"=>0,
                "state"=>"ready","hp"=>100,"maxHp"=>100,"ports"=>arr!["weapon","logistics","repair"]}] } };
    let args = views(&[world, me]);
    let mut orders = obj! {};
    if recovering { orders.set("memo", "Recovering production"); }
    shared::structures::apply(&mut orders, args[0], args[1]);
    orders
}

#[test]
fn one_stalled_unit_does_not_stop_affordable_existing_building_services() {
    let orders = building_service_orders(1.0, 100000.0, false);
    let job = orders.get("structureJobs").at(0);
    assert!(job.get("guard").truthy(), "a funded turret must still defend its province");
    assert_eq!(job.get("supply").num(), 1.0, "supply still reduces nearby troops' upkeep");
    assert_eq!(orders.get("structureService").at(0).get("target").num(), 20.0);
    assert!(job.get("budget").num() > 0.0 && job.get("budget").num() <= 250.0);
    assert!(orders.get("structures").is_empty());
}

#[test]
fn building_services_still_respect_cash_and_recovery_limits() {
    for (bank, recovering) in [(1000.0, false), (100000.0, true)] {
        let orders = building_service_orders(1.0, bank, recovering);
        let job = orders.get("structureJobs").at(0);
        assert!(!job.get("guard").truthy());
        assert_eq!(job.get("supply").num(), 0.0);
        assert_eq!(job.get("budget").num(), 0.0);
        assert!(orders.get("structureService").is_empty());
    }
}
