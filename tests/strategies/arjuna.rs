fn flight_world() -> Value {
    Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true,"shielded":false},{"id":2,"alive":true,"shielded":false}],"provinces":[
        {"id":0,"owner":1,"adj":[1],"visible":true,"units":{"1":1}},
        {"id":1,"owner":2,"adj":[0],"visible":true,"units":{"2":2}}]}"#)
}

fn flight_me() -> Value {
    Value::parse(r#"{"id":1,"allies":[],"ramProvince":{},"rts":{"cpu":{"limits":{"fabrication":1200}},"air":{"contacts":[]}}}"#)
}

fn fighting_wing() -> Value {
    Value::parse(r#"[{"id":101,"owner":1,"type":"wing","air":true,"at":1,"to":-1,"target":1,"doing":"fight",
        "range":0,"build":{"wings":1,"range":0},"ramUnit":{},"rts":{"air":{"aloft":true,"heat":5,"glide":null}}}]"#)
}

#[test]
fn defended_sortie_keeps_fighting_with_province_counts_and_no_world_units() {
    let world = flight_world();
    assert!(world.get("units").is_undef(), "the real unit callback omits world.units");
    let args = views(&[fighting_wing(), world, flight_me()]);
    let (foes, defenders, prey) = super::sky::hostiles(args[1], args[2], 1.0);
    assert_eq!((foes, defenders), (2.0, 2.0));
    assert_eq!(prey, None, "province counts supply no target positions");
    super::wing(args[0], args[1], args[2]);
    let orders = emitted();
    assert!(orders.is_empty(), "the defended standing fight must remain active: {orders:?}");
}

#[test]
fn aircraft_abroad_uses_its_standing_target_to_intercept_a_visible_contact() {
    let mut me = flight_me();
    me.entry("rts").entry("air").set("contacts", arr![obj! {
        "id"=>401,"owner"=>2,"air"=>true,"at"=>1,"hp"=>3
    }]);
    let args = views(&[fighting_wing(), flight_world(), me]);
    super::wing(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("target").num(), 1.0);
    assert_eq!(orders[0].get("do").str(), "fight");
    assert_eq!(orders[0].get("focus").num(), 401.0);
}

#[test]
fn fresh_home_launch_carries_the_sdk_core_plan_with_its_sky_sortie() {
    let mut me = flight_me();
    let plan = obj! {"attack"=>2,"sky"=>1,"stage"=>0,"defend"=>-1};
    me.entry("ramProvince").set("0", pages(&plan));
    let units = Value::parse(r#"[{"id":101,"owner":1,"type":"wing","air":true,"at":0,"to":-1,"target":-1,"doing":"idle",
        "range":0,"build":{"wings":1,"range":0},"ramUnit":{},"rts":{"air":{"aloft":false,"heat":0,"glide":null}}}]"#);
    let args = views(&[units, flight_world(), me]);
    super::wing(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("target").num(), 1.0, "the sky sortie can differ from the ground attack");
    assert!(!orders[0].get("ram").is_empty(), "a home launch must carry the SDK's core plan");
}
