fn line_pages(value: &Value) -> Value {
    let mut pages = Value::obj();
    for op in mem::put(0,value,"shared",None,16).expect("authored plan fits") {
        pages.set(&op.get("write").num().to_string(),obj! {"bytes"=>op.get("bytes").clone(),"t"=>120});
    }
    pages
}

#[test]
fn a_funded_cargo_house_rebuilds_two_ground_lines_before_ordinary_recruitment(){
    let world=obj!{"seconds"=>600,"logistics"=>obj!{},"units"=>arr![
        obj!{"owner"=>5,"type"=>"line","hp"=>2,"rts"=>obj!{"sea"=>obj!{"ship"=>true}}},
        obj!{"owner"=>5,"type"=>"line","hp"=>2,"build"=>obj!{"wings"=>1}},
        obj!{"owner"=>5,"type"=>"line","hp"=>2}]};
    let me=obj!{"id"=>5,"units"=>obj!{"total"=>3,"byType"=>obj!{"line"=>3}},
        "rts"=>obj!{"cpu"=>obj!{"income"=>10000,"bank"=>600000}}};
    let args=views(&[world.clone(),me.clone()]);
    assert_eq!(frontier_upkeep(args[0],args[1]),0.55,"ships and aircraft cannot replace capturers");
    let mut ready=world.clone();ready.entry("units").push(obj!{"owner"=>5,"type"=>"line","hp"=>2});
    let args=views(&[ready,me.clone()]);assert_eq!(frontier_upkeep(args[0],args[1]),0.4);
    for case in ["early","poor","stalled","queued","no-cargo"]{
        let mut w=world.clone();let mut m=me.clone();
        match case{
            "early"=>w.set("seconds",599),
            "poor"=>m.entry("rts").entry("cpu").set("bank",599999),
            "stalled"=>m.entry("units").set("stalled",1),
            "queued"=>m.entry("units").set("queued",1),
            _=>w.set("logistics",Value::Null),
        }
        let args=views(&[w,m]);assert_eq!(frontier_upkeep(args[0],args[1]),0.4,"{case}");
    }
}

#[test]
fn a_local_manifesto_plan_keeps_priority_over_a_carried_core() {
    let local = line_pages(&obj! {"attack"=>1,"stage"=>0,"hold"=>arr![0],"sway"=>arr![1]});
    let core = plan::pack(&obj! {"attack"=>2,"stage"=>2});
    let units = arr![obj! {"id"=>1,"at"=>2,"ramUnit"=>obj! {"0"=>obj! {"bytes"=>Value::Bytes(core)}}},
        obj! {"id"=>2,"at"=>0}];
    let args = views(&[units,obj! {"seconds"=>123},obj! {"rts"=>obj! {},"ramProvince"=>obj! {"0"=>local}}]);
    let p = line_plan(args[0],args[1],args[2]).expect("funded plan selection");
    assert_eq!(p.get("attack").num(),1.0);
    assert_eq!(p.get("hold").at(0).num(),0.0);
    assert!(!p.get("partial").truthy());
}

#[test]
fn troops_abroad_keep_their_carried_objective() {
    let core = plan::pack(&obj! {"attack"=>2,"stage"=>1,"defend"=>-1,"need"=>4});
    let args = views(&[arr![obj! {"id"=>1,"at"=>1,"ramUnit"=>obj! {"0"=>obj! {"bytes"=>Value::Bytes(core)}}}],
        obj! {"seconds"=>123},obj! {"rts"=>obj! {},"ramProvince"=>obj! {}}]);
    let p = line_plan(args[0],args[1],args[2]).expect("carried plan");
    assert_eq!(p.get("attack").num(),2.0);
    assert_eq!(p.get("need").num(),4.0);
    assert!(p.get("partial").truthy());
}

#[test]
fn funded_line_control_keeps_its_garrison_and_capture_roles() {
    let plan = obj! {"attack"=>1,"stage"=>0,"home"=>0,"defend"=>-1,"need"=>2,"ready"=>2,
        "hold"=>arr![0],"sway"=>arr![1],"front"=>arr![1,0,20,0,20,123]};
    let world = obj! {"seconds"=>123,"provinces"=>arr![obj! {"id"=>0,"owner"=>5,"adj"=>arr![1],"hops"=>0},
        obj! {"id"=>1,"owner"=>4,"adj"=>arr![0]}]};
    let me = obj! {"id"=>5,"capital"=>0,"allies"=>arr![],"ramProvince"=>obj! {"0"=>line_pages(&plan)},"rts"=>obj! {}};
    let units = arr![obj! {"id"=>3,"owner"=>5,"at"=>0,"to"=>-1,"doing"=>"idle","target"=>-1},
        obj! {"id"=>4,"owner"=>5,"at"=>0,"to"=>-1,"doing"=>"idle","target"=>-1}];
    let args = views(&[units,world,me]);
    line(args[0],args[1],args[2]);
    let out = emitted();
    assert!(out.iter().any(|o|o.get("id").num()==3.0 && o.get("do").str()=="fight" && o.get("target").num()==0.0),"{out:?}");
    assert!(out.iter().any(|o|o.get("id").num()==4.0 && o.get("do").str()=="capture" && o.get("target").num()==1.0),"{out:?}");
}

// manifesto-spark-cold.json reuses the authored 45-province line fixture's
// 11-page plan with one spark and the original distant sway marks23/24. It is
// not the candidate1 t3660 observation; it exercises cold reading and routing.
#[test]
fn funded_sparks_persuade_their_mark_and_keep_an_existing_assignment() {
    let p=obj! {"attack"=>1,"stage"=>0,"home"=>0,"sway"=>arr![1]};
    let world=obj! {"seconds"=>123,"provinces"=>arr![obj! {"id"=>0,"owner"=>5,"adj"=>arr![1]},
        obj! {"id"=>1,"owner"=>4,"adj"=>arr![0]}]};
    let me=obj! {"id"=>5,"capital"=>0,"allies"=>arr![],"techs"=>obj! {"influence"=>true},
        "ramProvince"=>obj! {"1"=>line_pages(&p)},"rts"=>obj! {}};
    let units=arr![obj! {"id"=>1,"owner"=>5,"at"=>1,"to"=>-1,"doing"=>"idle","target"=>-1}];
    let args=views(&[units.clone(),world.clone(),me.clone()]);spark(args[0],args[1],args[2]);
    let out=emitted();
    assert_eq!(out.len(),1);
    assert_eq!(out[0].get("do").str(),"persuade");
    assert_eq!(out[0].get("target").num(),1.0);
    let busy=arr![obj! {"id"=>1,"owner"=>5,"at"=>1,"to"=>-1,"doing"=>"persuade","target"=>1}];
    let args=views(&[busy,world,me]);spark(args[0],args[1],args[2]);
    assert!(emitted().is_empty(),"funded sparks retain their paid persuasion work");
}
