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
