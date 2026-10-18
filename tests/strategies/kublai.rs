#[test]
fn kublai_skips_control_orders_without_a_full_route_budget() {
    assert!(!super::control_has_runway(2381.0, 800.0, 0.0));
    assert!(super::control_has_runway(4200.0, 800.0, 0.0));
    assert!(!super::control_has_runway(5000.0, 2400.0, 900.0));
}

#[test]
fn kublai_repairs_a_stale_planning_cap_even_after_cheap_calls_lower_the_peak() {
    let me = obj! { "compute"=>obj! { "thinkPeak"=>12000 }, "ramCapital"=>obj! {},
        "units"=>obj! { "total"=>6, "queued"=>0, "doing"=>obj! {} },
        "types"=>obj! { "miner"=>obj! {}, "runner"=>obj! {} },
        "rts"=>obj! { "cpu"=>obj! { "income"=>36424, "bank"=>4000000, "capacity"=>4370880,
            "available"=>obj! { "think"=>23531 } } } };
    let args = views(&[me.clone()]);
    assert!(!super::full_planning_fits(args[0], 23531.0), "a decayed peak must not admit the stale cap");
    assert!(!super::full_planning_fits(args[0], 50000.0), "the public available allowance also bounds admission");
    let recovery = super::recovery_policy(args[0]);
    assert_eq!(recovery.get("memo").str(), "Recovering production");
    assert!(recovery.get("compute").get("types").get("miner").get("cycles").num() >= 4000.0);
    let requested = recovery.get("compute").get("cpu").get("limits").get("think").num();
    let allowance = if requested <= 1.0 { requested * 36424.0 } else { requested };
    assert!(allowance >= super::planning_allowance(args[0]), "recovery must fund admission plus the governor");
    let mut resumed = me;
    resumed.entry("rts").entry("cpu").entry("available").set("think", allowance);
    let args = views(&[resumed]);
    assert!(super::full_planning_fits(args[0], allowance - 4000.0), "the returned policy can resume the full court");
    let policy = super::cpu_policy(args[0], &obj! { "defend"=>-1 }, &obj! {});
    assert!(policy.get("limits").get("think").num() >= allowance, "a resumed pass must not restore the stale cap");
}
