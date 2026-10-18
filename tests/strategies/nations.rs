#[test]
fn frontline_prefers_an_affordable_rival_capital_from_foreign_land() {
    let world = Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true,"capital":0},{"id":2,"alive":true,"capital":1},{"id":3,"alive":true,"capital":2}],"provinces":[
        {"id":0,"owner":1,"adj":[1,2],"hops":0},
        {"id":1,"owner":2,"capital":true,"cores":1,"adj":[0],"townGarrison":0},
        {"id":2,"owner":3,"cores":4,"adj":[0],"townGarrison":0}],"units":[]}"#);
    let me = obj! { "id"=>1, "capital"=>0, "allies"=>arr![], "contracts"=>arr![],
        "units"=>obj! { "total"=>8, "byType"=>obj! {}, "doing"=>obj! {} }, "ramCapital"=>obj! {} };
    let args = views(&[world, me]);
    let ranked = frontline(args[0], args[1]);
    assert_eq!(ranked.first().map(|p| p.get("id").num()), Some(1.0));
}

#[test]
fn frontline_keeps_pressing_the_recorded_campaign_when_scores_are_close() {
    let campaign = pages(&obj! { "campaign"=>3 });
    let world = Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true,"capital":0},{"id":2,"alive":true,"capital":1},{"id":3,"alive":true,"capital":2}],"provinces":[
        {"id":0,"owner":1,"adj":[1,2],"hops":0},
        {"id":1,"owner":2,"cores":5,"adj":[0],"townGarrison":0},
        {"id":2,"owner":3,"cores":4,"adj":[0],"townGarrison":0}],"units":[]}"#);
    let me = obj! { "id"=>1, "capital"=>0, "allies"=>arr![], "contracts"=>arr![],
        "units"=>obj! { "total"=>8, "byType"=>obj! {}, "doing"=>obj! {} }, "ramCapital"=>campaign };
    let args = views(&[world, me]);
    let ranked = frontline(args[0], args[1]);
    assert_eq!(ranked.first().map(|p| p.get("id").num()), Some(2.0));
}

#[test]
fn sentry_advances_with_a_committed_capture_wave() {
    let plan = pages(&obj! { "attack"=>1, "stage"=>0, "defend"=>-1, "home"=>0 });
    let world = Value::parse(r#"{"seconds":1,"factions":[null,{"id":1,"alive":true,"capital":0},{"id":2,"alive":true,"capital":1}],"provinces":[
        {"id":0,"owner":1,"adj":[1]},{"id":1,"owner":2,"adj":[0],"units":{"1":2}}],"units":[]}"#);
    let me = obj! { "id"=>1, "capital"=>0, "allies"=>arr![], "ramProvince"=>obj! { "0"=>plan } };
    let units = Value::parse(r#"[{"id":10,"owner":1,"type":"sentry","at":0,"to":-1,"doing":"idle","build":{}}]"#);
    let args = views(&[units, world, me]);
    sentry(args[0], args[1], args[2]);
    let orders = emitted();
    assert_eq!(orders.len(), 1, "{orders:?}");
    assert_eq!(orders[0].get("target").num(), 1.0, "{orders:?}");
    assert_eq!(orders[0].get("do").str(), "capture", "{orders:?}");
}
