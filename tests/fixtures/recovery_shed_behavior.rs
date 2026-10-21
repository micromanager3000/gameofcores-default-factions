/// A cargo world short of upkeep: five miners at the capital and four guards, one at the capital, one a hop out,
/// two two hops out (one of them fighting). `debt` is the upkeep debt in seconds of income.
fn shed_case(debt: f64, rivals_at_capital: bool) -> (Value, Value) {
    let (mut world, mut me, _) = fixture();
    world.set("seconds", 40);
    for (i, hops) in [0, 1, 2].into_iter().enumerate() {
        let p = item(world.entry("provinces"), i);
        p.set("hops", hops);
        p.set("adj", if i == 0 { arr![1] } else if i == 1 { arr![0, 2] } else { arr![1] });
    }
    if rivals_at_capital { item(world.entry("provinces"), 0).set("units", obj! {"2"=>3}); }
    let mut units = Value::arr();
    for id in 0..5 {
        units.push(obj! {"id"=>id,"owner"=>1,"type"=>"miner","at"=>0,"to"=>-1,"doing"=>"gather","build"=>obj!{}});
    }
    for (id, at, fighting) in [(10, 0, false), (11, 1, false), (12, 2, false), (13, 2, true)] {
        units.push(obj! {"id"=>id,"owner"=>1,"type"=>"guard","at"=>at,"to"=>-1,"doing"=>"fight","fighting"=>fighting,"build"=>obj!{}});
    }
    world.set("units", units);
    me.set("units", obj! {"total"=>9,"stalled"=>0,"queued"=>0,"cap"=>20,"byType"=>obj! {"miner"=>5,"guard"=>4}});
    let cpu = me.entry("rts").entry("cpu");
    cpu.set("income", 40000);
    cpu.set("bank", 100000);
    cpu.set("debt", 40000.0 * debt);
    cpu.entry("last").set("upkeep", 30000);
    (world, me)
}

fn shed(world: &Value, me: &Value) -> Value {
    let args = views(&[world.clone(), me.clone()]);
    common::govern(args[0], args[1], 0.4, || obj! {})
}

#[test]
fn shortage_sheds_once_debt_passes_half_the_credit_farthest_idle_units_first() {
    let (world, me) = shed_case(5.0, false);
    assert!(shed(&world, &me).get("disband").is_empty(), "a short debt waits out a gap between cargo loads");
    let (world, me) = shed_case(16.0, false);
    assert_eq!(shed(&world, &me).get("disband"), &arr![12, 11], "idle before fighting, far before near, the capital last");
}

#[test]
fn a_pressed_capital_keeps_its_garrison_and_pays_it_before_the_miners() {
    let (world, me) = shed_case(16.0, true);
    let out = shed(&world, &me);
    assert!(!out.get("disband").items().iter().any(|id| id.num() == 10.0), "the capital's garrison stays while rivals are at its gates");
    assert_eq!(out.get("compute").get("cpu").get("upkeepOrder").at(0).str(), "guard");
    let (world, me) = shed_case(0.0, false);
    assert_eq!(shed(&world, &me).get("compute").get("cpu").get("upkeepOrder").at(0).str(), "miner", "miners first in peace");
}

#[test]
fn the_full_recovery_review_sheds_soldiers_before_the_haulers_that_pay_for_them() {
    let (mut world, mut me) = shed_case(16.0, false);
    // two idle haulers out at the far depot, and the army unpaid
    for id in [20, 21] {
        world.entry("units").push(obj! {"id"=>id,"owner"=>1,"type"=>"hauler","at"=>2,"to"=>-1,"doing"=>"idle","build"=>obj!{"carry"=>5}});
    }
    me.entry("units").set("total", 11);
    me.entry("units").set("stalled", 2);
    me.entry("units").entry("byType").set("hauler", 2);
    me.entry("rts").entry("cpu").set("bank", 1000);
    let out = shed(&world, &me);
    let gone: Vec<f64> = out.get("disband").items().iter().map(|v| v.num()).collect();
    assert!(!gone.is_empty(), "an unpaid army past half the credit is shed");
    assert!(!gone.contains(&20.0) && !gone.contains(&21.0), "haulers stay while soldiers can go: {gone:?}");
}

#[test]
fn shedding_before_any_stall_still_keeps_six_at_the_capital() {
    let (mut world, mut me) = shed_case(16.0, false);
    // every soldier at home: eight guards at the capital
    let mut units = Value::arr();
    for id in 0..5 {
        units.push(obj! {"id"=>id,"owner"=>1,"type"=>"miner","at"=>0,"to"=>-1,"doing"=>"gather","build"=>obj!{}});
    }
    for id in 10..18 {
        units.push(obj! {"id"=>id,"owner"=>1,"type"=>"guard","at"=>0,"to"=>-1,"doing"=>"fight","fighting"=>false,"build"=>obj!{}});
    }
    world.set("units", units);
    me.entry("units").set("total", 13);
    me.entry("units").entry("byType").set("guard", 8);
    let gone = shed(&world, &me).get("disband").items().len();
    assert!(gone > 0 && 8 - gone >= 6, "sheds only past a six-unit garrison: {gone} gone");
}
