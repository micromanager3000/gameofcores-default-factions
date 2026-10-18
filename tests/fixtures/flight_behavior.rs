#[test]
fn arjuna_replaces_an_affordable_ground_pick_with_flight_after_its_crossings() {
    let (world, mut me, _) = fixture();
    me.entry("techs").set("seaTraversal", true);
    me.entry("techs").set("mountainTraversal", true);
    // `common::budget` may have chosen a smaller ground upgrade because Flight's offer is above capacity.
    let mut orders = obj! { "research"=>"armor", "compute"=>obj! { "cpu"=>obj! { "limits"=>obj! { "research"=>0.12 } } } };
    let args = views(&[world, me]);
    super::flight_progress(&mut orders, args[0], args[1]);
    assert_eq!(orders.get("research").str(), "flight");
    assert_eq!(orders.get("compute").get("cpu").get("limits").get("research").num(), 0.12);
}

#[test]
fn arjuna_keeps_a_solved_research_problem_instead_of_replacing_it_with_flight() {
    let (world, mut me, _) = fixture();
    me.entry("techs").set("seaTraversal", true);
    me.entry("techs").set("mountainTraversal", true);
    me.set("research", obj! { "tech"=>"artillery" });
    let mut orders = obj! { "research"=>"artillery", "compute"=>obj! { "cpu"=>obj! { "limits"=>obj! { "research"=>0.12 } } } };
    let args = views(&[world, me]);
    super::flight_progress(&mut orders, args[0], args[1]);
    assert_eq!(orders.get("research").str(), "artillery");
}

#[test]
fn budget_leaves_an_active_flight_order_unchanged() {
    let (mut world, mut me, _) = fixture();
    me.entry("techs").set("seaTraversal", true);
    me.entry("techs").set("mountainTraversal", true);
    me.set("research", obj! { "tech"=>"flight" });
    // Flight is a 600,000-cycle offer. The 100,000-cycle bank capacity would otherwise favor smaller upgrades.
    world.entry("techs").entry("flight").set("offer", 600_000);
    me.entry("rts").entry("cpu").set("capacity", 100_000);
    let args = views(&[world, me]);
    let mut orders = obj! {};
    common::budget(&mut orders, args[0], args[1], &common::Budget { techs: &TECHS, ..Default::default() });
    // The engine keeps the standing order until Flight completes; emitting no replacement preserves its solved puzzle.
    assert!(orders.get("research").is_undef());
}

#[test]
fn arjuna_continues_the_sky_path_with_artillery_after_flight() {
    let (mut world, mut me, _) = fixture();
    me.entry("techs").set("flight", true);
    world.entry("techs").set("artillery", obj! { "requires"=>arr!["ranged"], "offer"=>300_000 });
    let mut orders = obj! { "research"=>"armor", "compute"=>obj! { "cpu"=>obj! { "limits"=>obj! { "research"=>0.12 } } } };
    let args = views(&[world, me]);
    super::flight_progress(&mut orders, args[0], args[1]);
    assert_eq!(orders.get("research").str(), "artillery");
}

#[test]
fn air_weapons_leads_to_artillery_then_strike_craft_after_flight() {
    let (mut world, mut me, _) = fixture();
    me.entry("techs").set("flight", true);
    // Match the engine prerequisite graph: Air weapons requires Flight and Artillery; Artillery requires Ranged.
    world.entry("techs").set("artillery", obj! { "requires"=>arr!["ranged"], "offer"=>300_000 });
    world.entry("techs").set("airWeapons", obj! { "requires"=>arr!["flight", "artillery"], "offer"=>600_000 });
    // With capacity for the published offers, Arjuna's order chooses Air weapons' Artillery prerequisite before Armor.
    me.entry("rts").entry("cpu").set("capacity", 1_000_000);
    let args = views(&[world, me]);
    let mut orders = obj! {};
    common::budget(&mut orders, args[0], args[1], &common::Budget { techs: &TECHS, ..Default::default() });
    assert_eq!(orders.get("research").str(), "artillery");
}
