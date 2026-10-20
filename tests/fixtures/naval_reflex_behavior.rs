fn naval_case(price: f64, x: f64, y: f64) -> (Value, Value) {
    let u = obj! {"id"=>1,"owner"=>0,"type"=>"escort","range"=>1,"focus"=>2,
        "rts"=>obj! {"priceLevel"=>price,"sea"=>obj! {"ship"=>true,"water"=>0,
            "head"=>obj! {"x"=>1,"y"=>0}},"cargo"=>obj! {"cycles"=>0}}};
    let t = obj! {"id"=>2,"owner"=>1,"rival"=>true,"range"=>1,"air"=>false,
        "d"=>x.hypot(y),"dx"=>x,"dy"=>y,"rts"=>obj! {"sea"=>obj! {"ship"=>true,"water"=>0}}};
    (u, arr![t])
}

#[test]
fn escort_broadside_prices_the_volley_and_preserves_a_paid_gun_order() {
    let (u, near) = naval_case(3.5, 0.0, 3.0);
    let args = views(&[u, near]);
    let mut out = obj! {"id"=>1,"hit"=>2,"power"=>obj! {"weapon"=>300}};
    let (mut bank, mut abilities) = (1575.0, 1575.0);
    assert!(common::naval_reflex(args[0], args[1], &mut out, &mut bank, &mut abilities));
    assert_eq!(out.get("ability").str(), "broadside");
    assert_eq!(out.get("side").str(), "port");
    assert_eq!(out.get("hit").num(), 2.0);
    assert_eq!(out.get("power").get("weapon").num(), 300.0);
    assert_eq!((bank, abilities), (0.0, 0.0));
}

#[test]
fn escorts_share_the_remaining_ability_allowance() {
    let (u, near) = naval_case(1.0, 0.0, 3.0);
    let mut second = u.clone();
    second.set("id", 4);
    let args = views(&[arr![u, second], near]);
    let (mut bank, mut abilities) = (10000.0, 450.0);
    let mut first = obj! {"id"=>1};
    let mut second = obj! {"id"=>4};
    assert!(common::naval_reflex(args[0].at(0), args[1], &mut first, &mut bank, &mut abilities));
    assert!(!common::naval_reflex(args[0].at(1), args[1], &mut second, &mut bank, &mut abilities));
    assert_eq!(first.get("ability").str(), "broadside");
    assert!(second.get("ability").is_undef());
    assert_eq!((bank, abilities), (9550.0, 0.0));
}

#[test]
fn an_unfunded_escort_preserves_its_existing_order() {
    let (u, near) = naval_case(1.0, 0.0, 3.0);
    let args = views(&[u, near]);
    let mut out = obj! {"id"=>1,"hit"=>2};
    let before = out.clone();
    let (mut bank, mut abilities) = (449.0, 1000.0);
    assert!(!common::naval_reflex(args[0], args[1], &mut out, &mut bank, &mut abilities));
    assert_eq!(out, before);
    assert_eq!((bank, abilities), (449.0, 1000.0));
}

#[test]
fn a_friendly_hull_masks_the_escort_volley() {
    let (u, mut near) = naval_case(1.0, 0.0, 3.0);
    near.push(obj! {"id"=>3,"owner"=>0,"rival"=>false,"air"=>false,"d"=>1,"dx"=>0,"dy"=>1,
        "rts"=>obj! {"sea"=>obj! {"ship"=>true,"water"=>0}}});
    let args = views(&[u, near]);
    let mut out = obj! {"id"=>1};
    let (mut bank, mut abilities) = (10000.0, 1000.0);
    assert!(!common::naval_reflex(args[0], args[1], &mut out, &mut bank, &mut abilities));
    assert!(out.get("ability").is_undef() && out.get("face").is_undef());
    assert_eq!((bank, abilities), (10000.0, 1000.0));
}

#[test]
fn a_loaded_hull_fires_an_existing_arc_without_changing_its_cargo_route() {
    let (mut u, near) = naval_case(1.0, 0.0, 3.0);
    u.entry("rts").entry("cargo").set("cycles", 1000);
    let args = views(&[u, near]);
    let mut out = obj! {"id"=>1,"cargo"=>obj! {"unload"=>obj! {"p"=>7}}};
    let cargo = out.get("cargo").clone();
    let (mut bank, mut abilities) = (10000.0, 1000.0);
    assert!(common::naval_reflex(args[0], args[1], &mut out, &mut bank, &mut abilities));
    assert_eq!(out.get("ability").str(), "broadside");
    assert_eq!(out.get("cargo"), &cargo);
    assert!(out.get("face").is_undef() && out.get("head").is_undef() && out.get("sail").is_undef());
}

#[test]
fn a_loaded_hull_keeps_sailing_when_the_target_is_off_its_beam() {
    let (mut u, near) = naval_case(1.0, 3.0, 0.0);
    u.entry("rts").entry("cargo").set("cycles", 1000);
    let args = views(&[u, near]);
    let mut out = obj! {"id"=>1,"cargo"=>obj! {"unload"=>obj! {"p"=>7}}};
    let before = out.clone();
    let (mut bank, mut abilities) = (10000.0, 1000.0);
    assert!(!common::naval_reflex(args[0], args[1], &mut out, &mut bank, &mut abilities));
    assert_eq!(out, before);
    assert_eq!((bank, abilities), (10000.0, 1000.0));
}

#[test]
fn an_escort_keeps_its_course_during_reload_without_buying_another_volley() {
    let (mut u, near) = naval_case(1.0, 3.0, 0.0);
    u.entry("rts").entry("abilityIn").set("broadside", 2);
    let args = views(&[u, near]);
    let mut out = obj! {"id"=>1,"hit"=>2,"sail"=>arr![3,4]};
    let before = out.clone();
    let (mut bank, mut abilities) = (10000.0, 1000.0);
    assert!(!common::naval_reflex(args[0], args[1], &mut out, &mut bank, &mut abilities));
    assert_eq!(out, before, "face would cancel the existing sailing course");
    assert_eq!((bank, abilities), (10000.0, 1000.0));
}

#[test]
fn an_aligned_escort_fires_without_replacing_its_sailing_course() {
    let (u, near) = naval_case(1.0, 0.0, 3.0);
    let args = views(&[u, near]);
    let mut out = obj! {"id"=>1,"sail"=>arr![3,4],"focus"=>2};
    let (mut bank, mut abilities) = (10000.0, 1000.0);
    assert!(common::naval_reflex(args[0], args[1], &mut out, &mut bank, &mut abilities));
    assert_eq!(out.get("ability").str(), "broadside");
    assert_eq!(out.get("sail"), &arr![3,4]);
    assert_eq!(out.get("focus").num(), 2.0);
    assert!(out.get("face").is_undef() && out.get("step").is_undef() && out.get("hold").is_undef());
    assert_eq!((bank, abilities), (9550.0, 550.0));
}

#[test]
fn a_ready_unloaded_escort_can_aim_an_affordable_bow_contact() {
    let (u, near) = naval_case(1.0, 3.0, 0.0);
    let args = views(&[u, near]);
    let mut out = obj! {"id"=>1,"step"=>obj! {"x"=>1,"y"=>0},"hold"=>true};
    let (mut bank, mut abilities) = (10000.0, 450.0);
    assert!(common::naval_reflex(args[0], args[1], &mut out, &mut bank, &mut abilities));
    assert!(out.get("face").truthy());
    assert_eq!(out.get("step"), &Value::Null);
    assert!(!out.get("hold").truthy() && out.get("ability").is_undef());
    assert_eq!((bank, abilities), (10000.0, 450.0), "aiming does not buy a volley before it can fire");
}

#[test]
fn naval_contacts_preserve_another_paid_ability() {
    let (u, near) = naval_case(1.0, 0.0, 3.0);
    let args = views(&[u, near]);
    let mut out = obj! {"id"=>1,"ability"=>"scan"};
    let before = out.clone();
    let (mut bank, mut abilities) = (10000.0, 1000.0);
    assert!(!common::naval_reflex(args[0], args[1], &mut out, &mut bank, &mut abilities));
    assert_eq!(out, before);
    assert_eq!((bank, abilities), (10000.0, 1000.0));
}
