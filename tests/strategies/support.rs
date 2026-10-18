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
            decks.push(obj!{"id"=>u.get("id").clone(),"at"=>u.get("at").clone(),"room"=>deck.get("room").clone(),"riders"=>deck.get("riders").clone()});
        }
    }
    v[1].set("units",Value::arr());
    v[2].entry("rts").set("air",obj!{"contacts"=>contacts,"decks"=>decks});
    views(&v)
}

