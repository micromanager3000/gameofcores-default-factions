fn power_market(supply: f64, demand: f64, remote_supply: f64, transmission: bool, deals: Value) -> (Value, Value) {
    let world = Value::parse(r#"{"seconds":3,"energy":{"kinds":{}},"provinces":[
        {"id":0,"owner":7,"adj":[2]}, {"id":1,"owner":7,"adj":[]}, {"id":2,"owner":2,"adj":[0]}],
        "factions":[{"id":7,"alive":true,"powered":1},{"id":2,"alive":true,"powered":0.5}]}"#);
    let me = obj! {
        "id"=>7, "seat"=>7, "capital"=>0, "techs"=>obj! { "transmission"=>transmission },
        "compute"=>obj! { "cycles"=>15000 },
        "rts"=>obj! { "cpu"=>obj! { "income"=>40000, "bank"=>300000 }, "structures"=>arr![] },
        "energy"=>obj! { "house"=>3, "supply"=>supply + remote_supply, "demand"=>demand,
            "islands"=>arr![
                obj! { "provinces"=>arr![0], "supply"=>supply, "demand"=>demand, "capital"=>true },
                obj! { "provinces"=>arr![1], "supply"=>remote_supply, "demand"=>0, "capital"=>false }
            ], "plants"=>arr![
                obj! { "p"=>0, "mw"=>(supply - 3.0).max(0.0), "ports"=>arr![obj! { "kind"=>"wind", "capacity"=>(supply - 3.0).max(0.0) }] },
                obj! { "p"=>1, "mw"=>remote_supply, "ports"=>arr![obj! { "kind"=>"wind", "capacity"=>remote_supply }] }
            ], "deals"=>deals }
    };
    (world, me)
}

fn power_orders(supply: f64, demand: f64, remote_supply: f64, transmission: bool, deals: Value) -> (Value, Value, Value) {
    let (world, me) = power_market(supply, demand, remote_supply, transmission, deals);
    let args = views(&[world.clone(), me.clone()]);
    let mut orders = obj! {};
    shared::energy::apply(&mut orders, args[0], args[1]);
    (orders, world, me)
}

#[test]
fn a_locally_surplus_grid_offers_power_at_the_default_fair_price() {
    let (orders, world, _) = power_orders(12.0, 4.0, 0.0, true, arr![]);
    let offer = orders.get("send").at(0).clone();
    assert_eq!(orders.get("send").len(), 1, "{orders:?}");
    assert_eq!(offer.get("to").num(), 2.0);
    assert_eq!(offer.get("give").get("power").get("mw").num(), 4.0);
    assert_eq!(offer.get("ask").get("coinPerSecond").num(), 0.5);

    // Recipients use this exact default acceptance rule. The buyer can pay and values 4 MW for 180 seconds at 90 ◆.
    let mut message = offer;
    message.set("from", 7);
    let buyer = obj! { "id"=>2, "bank"=>1000, "units"=>obj! { "total"=>0, "queued"=>0, "cap"=>20 },
        "techs"=>obj! {}, "contracts"=>arr![] };
    let args = views(&[message, buyer, world]);
    assert!(common::fair(args[0], args[1], args[2], &[]));
}

#[test]
fn remote_surplus_cannot_be_sold_through_a_short_local_grid() {
    // Province 0 is the first and only border link to faction 2. Its island is short, even though island 1 is rich.
    let (orders, _, _) = power_orders(4.0, 4.0, 20.0, true, arr![]);
    assert!(orders.get("send").is_empty(), "{orders:?}");
}

#[test]
fn imported_power_is_not_resold() {
    // The snapshot's 13 MW includes a 10 MW import. The local island only owns its 3 MW house generator.
    let deals = arr![obj! { "give"=>false, "with"=>2, "mw"=>10, "delivered"=>10 }];
    let (orders, _, _) = power_orders(3.0, 4.0, 0.0, true, deals);
    assert!(orders.get("send").is_empty(), "{orders:?}");
}

#[test]
fn an_existing_power_contract_prevents_a_duplicate_offer() {
    let deals = arr![obj! { "give"=>true, "with"=>2, "mw"=>2 }];
    let (orders, _, _) = power_orders(12.0, 4.0, 0.0, true, deals);
    assert!(orders.get("send").is_empty(), "{orders:?}");
}

#[test]
fn transmission_is_required_before_the_market_opens() {
    let (orders, _, _) = power_orders(12.0, 4.0, 0.0, false, arr![]);
    assert!(orders.get("send").is_empty(), "{orders:?}");
}
