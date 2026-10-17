#[test]
fn interceptors_and_anti_air_choose_a_visible_aircraft_first() {
    // The ground decoy is nearly destroyed, so this verifies the sky priority rather than ordinary wounded-target
    // selection. The fixture uses one nearby contact list per friendly unit, as the RTS reflex receives it.
    let fighter = Value::parse(
        r#"{"id":101,"owner":1,"type":"wing","air":true,"range":0,"reach":3,"hp":5,"hpMax":5,
        "pos":{"x":0,"y":0},"rts":{"bank":100000,"modules":[],"budget":{},"air":{"heat":0},"abilityIn":{}}}"#,
    );
    let gun = Value::parse(
        r#"{"id":102,"owner":1,"type":"gun","air":false,"range":1,"reach":4,"hp":5,"hpMax":5,
        "pos":{"x":0,"y":0},"rts":{"bank":100000,"modules":["antiAir"],"budget":{},"abilityIn":{}}}"#,
    );
    let contacts = Value::parse(
        r#"[{"id":201,"rival":true,"air":false,"range":0,"hp":0.1,"hpMax":5,"d":1,"dx":1,"dy":0,"doing":"idle","type":"line"},
        {"id":202,"rival":true,"air":true,"range":0,"hp":5,"hpMax":5,"d":1,"dx":1,"dy":0,"doing":"idle","type":"wing"}]"#,
    );
    let near = obj! {"101"=>contacts.clone(),"102"=>contacts};
    let args = views(&[arr![fighter, gun], near]);
    reflex(args[0], args[1]);
    let orders: Vec<Value> = abi::TEST_ORDERS
        .with(|o| o.take())
        .iter()
        .map(|o| Value::parse(o))
        .collect();
    assert_eq!(orders.len(), 2, "{orders:?}");
    assert!(
        orders.iter().all(|o| o.get("hit").num() == 202.0),
        "{orders:?}"
    );
}


#[test]
fn arjuna_funds_an_active_flight_without_replacing_its_topic() {
    let (world, mut me, _) = fixture();
    me.set("research", obj! { "tech"=>"flight" });
    // Model the standing recovery policy that paused research while the bank recovered.
    let mut orders = obj! { "compute"=>obj! { "cpu"=>obj! {
        "limits"=>obj! { "research"=>0 }, "reserve"=>obj! { "research"=>0 }
    } } };
    let args = views(&[world, me]);
    super::flight_progress(&mut orders, args[0], args[1]);
    assert!(orders.get("research").is_undef(), "the existing Flight topic stands in the engine");
    assert_eq!(orders.get("compute").get("cpu").get("limits").get("research").num(), 0.12);
    assert_eq!(orders.get("compute").get("cpu").get("reserve").get("research").num(), 0.02);
}
