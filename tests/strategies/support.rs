use super::*;

fn views(values: &[Value]) -> Vec<View> {
    reset_library();
    View::args_json(&Value::Arr(values.to_vec()).to_json())
}

fn pages(plan: &Value) -> Value {
    let mut pages = Value::obj();
    for op in mem::put(0, plan, "local", None, 1).expect("fixture plan fits") {
        pages.set(&op.get("write").num().to_string(), obj! {"bytes"=>op.get("bytes").clone(), "t"=>1});
    }
    pages
}

fn emitted() -> Vec<Value> {
    abi::TEST_ORDERS.with(|o| o.take()).iter().map(|o| Value::parse(o)).collect()
}

fn productive_local_fixture() -> Vec<Value> {
    let world=obj! {"tick"=>150,"seconds"=>30,"logistics"=>obj! {"docks"=>Vec::<Value>::new()},
        "factions"=>vec![obj! {"id"=>0,"alive"=>true},obj! {"id"=>1,"alive"=>true}],
        "provinces"=>vec![obj! {"id"=>0,"owner"=>0,"adj"=>vec![1],"hops"=>0,"units"=>obj! {"0"=>6},"mines"=>vec![1,1],"staff"=>vec![2,2]},
            obj! {"id"=>1,"owner"=>-1,"adj"=>vec![0],"visible"=>true,"cores"=>3,"townGarrison"=>0,"units"=>obj! {}}],
        "units"=>(0..6).map(|id|obj! {"id"=>id,"owner"=>0,"at"=>0,"type"=>if id<4 {"miner"}else{"hauler"},
            "doing"=>if id<4 {"gather"}else{"idle"},"to"=>-1,"build"=>obj! {"hp"=>2,"speed"=>2,"strength"=>1,"carry"=>4,"vision"=>1}}).collect::<Vec<_>>()};
    let me=obj! {"id"=>0,"seat"=>4,"capital"=>0,"provinces"=>1,"bank"=>1000,"allies"=>Vec::<Value>::new(),
        "contracts"=>Vec::<Value>::new(),"ramCapital"=>obj! {},"techs"=>obj! {},"research"=>obj! {"tech"=>"extraction"},
        "types"=>obj! {"miner"=>obj! {},"hauler"=>obj! {},"freighter"=>obj! {},"runner"=>obj! {},"screen"=>obj! {}},
        "costs"=>obj! {"unit"=>1,"upkeep"=>100,"draw"=>obj! {}},"compute"=>obj! {"cycles"=>40000,"thinkPeak"=>1000},
        "units"=>obj! {"total"=>6,"queued"=>0,"spawned"=>6,"stalled"=>0,"cap"=>32,"byType"=>obj! {"miner"=>4,"hauler"=>2},"doing"=>obj! {"gather"=>4}},
        "rts"=>obj! {"bank"=>200000,"sea"=>obj! {"ports"=>Vec::<Value>::new()},"costs"=>obj! {"priceLevel"=>1,"fabrication"=>obj! {"base"=>1200,"module"=>600}},
            "cpu"=>obj! {"income"=>40000,"bank"=>200000,"spendCap"=>50000,"reserve"=>obj! {},"spent"=>obj! {},
                "last"=>obj! {"upkeep"=>600},"available"=>obj! {"control"=>40000,"think"=>25000,"fabrication"=>30000}}}};
    vec![world,me]
}

fn frontier_mining_fixture(army:&str)->Vec<Value> {
    let mut fixture=productive_local_fixture();
    if let Value::Arr(p)=fixture[0].entry("provinces") {
        p[0].set("units",obj! {"0"=>8});
        p[1].set("owner",0);p[1].set("adj",vec![0,2]);p[1].set("mines",vec![1]);p[1].set("staff",vec![0]);
        p.push(obj! {"id"=>2,"owner"=>-1,"adj"=>vec![1],"visible"=>true,"cores"=>2,"townGarrison"=>0,"units"=>obj! {}});
    }
    for id in 6..8 {fixture[0].entry("units").push(obj! {"id"=>id,"owner"=>0,"at"=>0,"type"=>army,
        "doing"=>"idle","to"=>-1,"hp"=>4,"build"=>obj! {"hp"=>4,"speed"=>2,"strength"=>3}});}
    let units=fixture[1].entry("units");units.set("total",8);units.set("spawned",8);units.entry("byType").set(army,2);
    units.entry("doing").set("idle",4);
    fixture[1].set("provinces",2);
    fixture[1].entry("rts").entry("cpu").entry("last").set("upkeep",800);
    fixture
}

// Build the air callback view: unit callbacks receive contacts and decks, never world.units.
fn flight_views(values: &[Value]) -> Vec<View> {
    if values.len() != 3 { return views(values); }
    let mut v=values.to_vec();
    let mut contacts=Value::arr();
    let mut decks=Value::arr();
    for u in v[1].get("units").items() {
        if u.get("owner").num()!=v[2].get("id").num() && u.get("air").truthy() {contacts.push(u.clone());}
        let deck=u.get("rts").get("sea").get("deck");
        if u.get("owner").num()==v[2].get("id").num() && deck.get("room").num_or(0.0)>0.0 {
            decks.push(obj!{"id"=>u.get("id").clone(),"at"=>u.get("at").clone(),"room"=>deck.get("room").clone(),"free"=>deck.get("free").num_or(deck.get("room").num_or(0.0)-deck.get("riders").len() as f64),"riders"=>deck.get("riders").clone()});
        }
    }
    v[1].set("units",Value::arr());
    v[2].entry("rts").set("air",obj!{"contacts"=>contacts,"decks"=>decks});
    views(&v)
}
