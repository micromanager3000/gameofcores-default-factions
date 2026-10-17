#[test]
fn a_ground_picket_selects_soldiers_instead_of_workers_ships_or_aircraft() {
    let (mut world, mut me, _) = fixture();
    me.entry("units").set("total", 30);
    item(world.entry("provinces"), 1).set("units", obj! {"1"=>5,"2"=>1});
    world.set("units", Value::parse(r#"[
        {"id":1,"owner":1,"type":"hauler","doing":"idle","at":1},
        {"id":2,"owner":1,"type":"freighter","doing":"idle","at":1},
        {"id":3,"owner":1,"type":"scout","doing":"idle","at":1},
        {"id":4,"owner":1,"type":"warjunk","doing":"idle","at":1,"rts":{"sea":{"ship":true}}},
        {"id":5,"owner":1,"type":"wing","doing":"idle","at":1,"build":{"wings":1}},
        {"id":6,"owner":1,"type":"line","doing":"idle","at":2},
        {"id":7,"owner":1,"type":"archer","doing":"idle","at":2},
        {"id":8,"owner":1,"type":"guard","doing":"idle","at":2}]"#));
    let args = views(&[world, me]);
    let mut plan = obj! {};
    common::defense(args[0], args[1], &mut plan);
    assert_eq!(plan.get("picket").num(), 1.0, "{plan:?}");
    assert_eq!(plan.get("guard"), &arr![6,7,8], "{plan:?}");
}

#[test]
fn a_starved_recovery_preserves_its_policy_between_reviews() {
    let (mut world, mut me, units) = fixture();
    world.set("units", units);
    world.set("seconds", 4);
    me.entry("rts").entry("cpu").set("bank", 1000);
    me.entry("units").set("stalled", 5);
    let args = views(&[world.clone(), me.clone()]);
    let entering = common::govern(args[0], args[1], 0.4, || panic!("starvation must enter recovery"));
    assert!(entering.get("compute").truthy());
    me.set("ramCapital", ram_pages(&entering));
    world.set("seconds", 5);
    let args = views(&[world.clone(), me.clone()]);
    assert!(common::govern(args[0], args[1], 0.4, || panic!("standing policy should rest")).is_empty());
    world.set("seconds", 6);
    let args = views(&[world, me]);
    let reviewed = common::govern(args[0], args[1], 0.4, || panic!("still short"));
    assert!(reviewed.get("compute").truthy(), "a short faction still reviews its recovery every three seconds");
}

fn escort_orders(armed: bool, privateer: bool) -> Vec<Value> {
    let (mut world, me, mut convoy) = fixture();
    item(&mut convoy, 0).entry("rts").entry("cargo").set("cycles", 40000);
    world.set("units", convoy);
    let mut me = me;
    me.entry("rts").entry("sea").set("contacts", arr![obj! {
        "id"=>201,"owner"=>2,"ship"=>true,"water"=>1,"hp"=>8,
        "build"=>obj! {"range"=>if armed {1}else{0},"carry"=>2},
        "cargo"=>obj! {"load"=>3}
    }]);
    let guard = obj! {"id"=>108,"owner"=>1,"type"=>"escort","focus"=>201,
        "rts"=>obj! {"sea"=>obj! {"ship"=>true,"water"=>0,"course"=>obj! {"chase"=>201}}}};
    let mut units = vec![guard.clone()];
    if privateer {
        let mut raider = guard;
        raider.set("id", 109);
        raider.set("focus", -1);
        raider.entry("rts").entry("sea").set("course", Value::Null);
        units.push(raider);
    }
    let args = views(&[Value::Arr(units), world, me]);
    abi::TEST_ORDERS.with(|o| o.borrow_mut().clear());
    common::guard_shipping(args[0], args[1], args[2]);
    abi::TEST_ORDERS.with(|o| o.take()).iter().map(|s| Value::parse(s)).collect()
}

#[test]
fn a_convoy_guard_breaks_off_a_merchant_chase_to_stay_with_its_cargo() {
    let orders = escort_orders(false, false);
    assert!(orders.iter().any(|o| o.get("id").num() == 108.0 && o.get("sail").num_or(-1.0) == 0.0), "{orders:?}");
    assert!(!orders.iter().any(|o| matches!(o.get("sail").get("chase"), Value::Num(_))), "{orders:?}");
}

#[test]
fn a_convoy_guard_keeps_intercepting_a_raider_that_threatens_its_cargo() {
    // The standing chase already names this raider; the callback must preserve it.
    assert!(escort_orders(true, false).is_empty());
}

#[test]
fn the_privateer_can_raid_while_the_convoy_guard_stays_with_the_freighter() {
    let orders = escort_orders(false, true);
    assert!(orders.iter().any(|o| o.get("id").num() == 108.0 && o.get("sail").num_or(-1.0) == 0.0), "{orders:?}");
    assert!(orders.iter().any(|o| o.get("id").num() == 109.0 && o.get("focus").num_or(-1.0) == 201.0), "{orders:?}");
}
