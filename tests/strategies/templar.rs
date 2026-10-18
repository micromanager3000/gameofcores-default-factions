#[test]
fn campaign_ranking_keeps_all_frontier_options_and_promotes_a_viable_capital() {
    let campaign = pages(&obj! {"campaign"=>3});
    let world = Value::parse(r#"{
        "seconds":1,
        "factions":[null,
            {"id":1,"alive":true,"shielded":false},
            {"id":2,"alive":true,"shielded":false},
            {"id":3,"alive":true,"shielded":false}
        ],
        "provinces":[
            {"id":0,"owner":1,"adj":[1,2]},
            {"id":1,"owner":2,"adj":[0],"cores":4,"townGarrison":0},
            {"id":2,"owner":3,"adj":[0],"cores":1,"townGarrison":0,"capital":true}
        ]
    }"#);
    let me = obj! {"id"=>1,"capital"=>0,"ramCapital"=>campaign,
        "units"=>obj! {"total"=>20,"byType"=>obj! {}}};
    let args = views(&[world, me]);
    let ranked = super::targets(args[0], args[1]);
    let ids: Vec<f64> = ranked.iter().map(|q| q.get("id").num()).collect();
    assert_eq!(ids, vec![2.0, 1.0], "the viable campaign capital leads without discarding the other frontier target");
}
