fn line_pages(value: &Value) -> Value {
    let mut pages = Value::obj();
    for op in mem::put(0,value,"shared",None,16).expect("authored plan fits") {
        pages.set(&op.get("write").num().to_string(),obj! {"bytes"=>op.get("bytes").clone(),"t"=>120});
    }
    pages
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
