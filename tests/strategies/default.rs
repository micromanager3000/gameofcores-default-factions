#[test]
fn classic_reflex_returns_no_duplicate_after_committing_the_order() {
    let units = Value::parse(r#"[{"id":1,"doing":"fight","reach":1,"range":0,"air":false}]"#);
    let near = Value::parse(r#"{"1":[{"id":2,"rival":true,"air":false,"d":0.5}]}"#);
    let args = views(&[units, near]);
    let returned = super::reflex(args[0], args[1]);
    let committed = emitted();
    assert!(returned.is_empty(), "classic order() commits survive independently of a returned list");
    assert_eq!(committed.len(), 1, "the committed list contains the unit's action");
    assert_eq!(committed[0].get("hit").num(), 2.0);
}

#[test]
fn funded_guard_keeps_a_capital_order_after_the_short_callback_entry_guard() {
    let units=Value::parse(r#"[{"id":1,"at":0,"doing":"idle","target":0,"to":-1,"way":[],"air":false,"build":{"wings":0,"range":0}}]"#);
    let world=Value::parse(r#"{"tick":1,"seconds":1,"provinces":[{"id":0,"owner":1,"adj":[1],"units":{"1":1},"mines":[1],"staff":[0]},{"id":1,"owner":-1,"adj":[0],"units":{},"mines":[1],"staff":[0]}],"factions":[null,{"id":1,"alive":true}]}"#);
    let me=Value::parse(r#"{"id":1,"capital":0,"ramCapital":{},"ramProvince":{},"units":{"total":1,"byType":{"guard":1}},"rts":{"cpu":{"income":15000}},"allies":[],"contracts":[]}"#);
    let args=views(&[units,world,me]);
    super::guard(args[0],args[1],args[2]);
    let orders=emitted();
    assert_eq!(orders.len(),1,"a funded guard keeps the capital protected");
    assert_eq!(orders[0].get("do").str(),"fight");
    assert!(super::GUARD_CALLBACK_RUNWAY < 2174.0,
        "the uniform world's funded guard allocation remains above the entry guard");
}
