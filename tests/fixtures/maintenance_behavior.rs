#[test]
fn plant_upkeep_forecast_splits_shell_and_energy_ports_at_their_periods() {
    let me = obj! { "id"=>1, "rts"=>obj! {
        "costs"=>obj! { "priceLevel"=>2 },
        "structureCosts"=>obj! { "maintenancePeriod"=>600, "plantMaintenancePeriod"=>1800 },
        "structures"=>arr![
            obj! { "owner"=>1, "state"=>"ready", "hp"=>100, "progress"=>0.5, "buildCost"=>6000, "energyCost"=>3600 },
            obj! { "owner"=>2, "state"=>"ready", "hp"=>100, "progress"=>1, "buildCost"=>6000, "energyCost"=>3600 }
        ]
    }};
    let me = views(&[me])[0];
    // (2,400 / 600 + 3,600 / 1,800) × half-built × price level 2 = 6 cycles/s.
    assert_eq!(common::building_upkeep(me), 6.0);
}
