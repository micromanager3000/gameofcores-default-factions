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
            ], "exportLinks"=>arr![obj! { "with"=>2, "at"=>0 }], "deals"=>deals }
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
fn a_first_mountain_route_selects_its_own_island_for_power_sales() {
    // The engine reports its first `deal_link` result: province 0's pass to faction 2 precedes province 1's land
    // border. Island 0 is short, so the rich second island must not create an offer.
    let (mut world, mut me) = power_market(4.0, 4.0, 20.0, true, arr![]);
    let Value::Arr(provinces) = world.entry("provinces") else { panic!("provinces") };
    provinces[0].set("adj", arr![]);
    provinces[0].set("routes", obj! { "pass"=>arr![2] });
    provinces[1].set("adj", arr![2]);
    me.entry("energy").set("exportLinks", arr![obj! { "with"=>2, "at"=>0 }]);
    let args = views(&[world, me]);
    let mut orders = obj! {};
    shared::energy::apply(&mut orders, args[0], args[1]);
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

#[test]
fn only_a_trade_doctrine_sells_from_one_point_five_to_three_megawatts_spare() {
    // 7 MW of native generation less the island's 4 MW draw plus its 15% reserve leaves 2.4 MW.
    // Nations (seat 7) may offer it; another house must wait until it has 3 MW spare.
    let (nations, world, _) = power_orders(7.0, 4.0, 0.0, true, arr![]);
    assert_eq!(nations.get("send").len(), 1, "{nations:?}");

    let (_, mut other) = power_market(7.0, 4.0, 0.0, true, arr![]);
    other.set("seat", 0);
    let args = views(&[world, other]);
    let mut orders = obj! {};
    shared::energy::apply(&mut orders, args[0], args[1]);
    assert!(orders.get("send").is_empty(), "{orders:?}");
}

#[test]
fn export_spare_is_only_the_linked_islands_native_power_after_demand_and_exports() {
    let (_, me) = power_market(12.0, 4.0, 0.0, true, arr![]);
    let args = views(&[me]);
    assert!((shared::energy::export_spare(args[0], 2.0) - 7.4).abs() < 1e-9);

    let (_, committed) = power_market(12.0, 4.0, 0.0, true, arr![obj! { "give"=>true, "with"=>3, "mw"=>2 }]);
    let args = views(&[committed]);
    assert!((shared::energy::export_spare(args[0], 2.0) - 5.4).abs() < 1e-9);

    let (_, mut disconnected) = power_market(12.0, 4.0, 0.0, true, arr![]);
    disconnected.entry("energy").set("exportLinks", arr![]);
    let args = views(&[disconnected]);
    assert_eq!(shared::energy::export_spare(args[0], 2.0), 0.0);
}

#[test]
fn power_requests_are_accepted_only_when_the_seller_can_deliver_native_spare() {
    let request = |mw| obj! { "from"=>2, "give"=>obj! { "coin"=>90 },
        "ask"=>obj! { "power"=>obj! { "mw"=>mw, "seconds"=>180 } } };
    let (world, mut seller) = power_market(12.0, 4.0, 0.0, true, arr![]);
    seller.set("bank", 1_000);
    seller.set("contracts", arr![]);
    seller.set("units", obj! { "total"=>0, "queued"=>0, "cap"=>20 });
    let request = request(4.0);
    let args = views(&[request, seller, world]);
    assert!(common::fair(args[0], args[1], args[2], &[]));
}

#[test]
fn power_requests_reject_disconnected_or_non_native_or_already_committed_supply() {
    let request = |mw| obj! { "from"=>2, "give"=>obj! { "coin"=>180 },
        "ask"=>obj! { "power"=>obj! { "mw"=>mw, "seconds"=>180 } } };
    let rejects = |mut seller: Value, world: Value, mw: f64| {
        seller.set("bank", 1_000);
        seller.set("contracts", arr![]);
        seller.set("units", obj! { "total"=>0, "queued"=>0, "cap"=>20 });
        let message = request(mw);
        let args = views(&[message, seller, world]);
        assert!(!common::fair(args[0], args[1], args[2], &[]));
    };

    let (world, mut disconnected) = power_market(12.0, 4.0, 0.0, true, arr![]);
    disconnected.entry("energy").set("exportLinks", arr![]);
    rejects(disconnected, world, 1.0);

    let (world, beyond_spare) = power_market(12.0, 4.0, 0.0, true, arr![]);
    rejects(beyond_spare, world, 8.0);

    let (world, imported) = power_market(3.0, 4.0, 10.0, true, arr![obj! { "give"=>false, "with"=>2, "mw"=>10, "delivered"=>10 }]);
    rejects(imported, world, 1.0);

    for kind in ["battery", "boiler"] {
        let (world, mut plant) = power_market(12.0, 4.0, 0.0, true, arr![]);
        plant.entry("energy").set("plants", arr![
            obj! { "p"=>0, "mw"=>9, "ports"=>arr![obj! { "kind"=>kind, "capacity"=>9 }] },
            obj! { "p"=>1, "mw"=>0, "ports"=>arr![obj! { "kind"=>"wind", "capacity"=>0 }] }
        ]);
        rejects(plant, world, 1.0);
    }

    let (world, exported) = power_market(12.0, 4.0, 0.0, true, arr![obj! { "give"=>true, "with"=>3, "mw"=>8 }]);
    rejects(exported, world, 1.0);
}

fn power_buyer() -> (Value, Value, Value) {
    let (mut world, mut seller) = power_market(12.0, 4.0, 0.0, true, arr![]);
    world.set("seconds", 33);
    world.set("factions", arr![obj! { "id"=>7, "alive"=>true, "powered"=>1, "techs"=>arr!["transmission"] },
        obj! { "id"=>2, "alive"=>true, "powered"=>0.5, "techs"=>arr![] }]);
    seller.set("bank", 1000);
    seller.set("contracts", arr![]);
    seller.set("units", obj! { "total"=>0, "queued"=>0, "cap"=>20 });
    let buyer = obj! { "id"=>2, "seat"=>2, "capital"=>2, "bank"=>1000, "techs"=>obj! {},
        "compute"=>obj! { "cycles"=>15000 }, "rts"=>obj! { "cpu"=>obj! { "income"=>40000, "bank"=>300000 }, "structures"=>arr![] },
        "energy"=>obj! { "powered"=>0.5, "supply"=>3, "demand"=>6, "met"=>3, "deals"=>arr![],
            "islands"=>arr![obj! { "provinces"=>arr![2], "supply"=>3, "demand"=>6, "met"=>3, "capital"=>true }] } };
    (world, buyer, seller)
}

#[test]
fn a_short_grid_requests_affordable_power_that_a_connected_seller_can_accept() {
    let (world, buyer, seller) = power_buyer();
    let args = views(&[world.clone(), buyer]);
    let mut orders = obj! {};
    shared::energy::apply(&mut orders, args[0], args[1]);
    assert_eq!(orders.get("send").len(), 1, "{orders:?}");
    let mut request = orders.get("send").at(0).clone();
    assert_eq!(request.get("to").num(), 7.0);
    assert_eq!(request.get("ask").get("power").get("mw").num(), 3.0);
    assert_eq!(request.get("give").get("coinPerSecond").num(), 0.375);
    request.set("from", 2);
    let args = views(&[request, seller, world]);
    assert!(common::fair(args[0], args[1], args[2], &[]));
}

#[test]
fn power_buyers_preserve_savings_and_avoid_duplicate_or_unroutable_imports() {
    for case in ["healthy", "poor", "imported", "split", "disconnected", "unresearched", "short-seller"] {
        let (mut world, mut buyer, _) = power_buyer();
        match case {
            "healthy" => buyer.entry("energy").set("powered", 0.95),
            "poor" => buyer.set("bank", 100),
            "imported" => buyer.entry("energy").set("deals", arr![obj! { "give"=>false, "mw"=>2, "with"=>7 }]),
            "split" => buyer.entry("energy").entry("islands").push(obj! { "provinces"=>arr![3] }),
            "disconnected" => world.set("provinces", arr![obj! { "id"=>2, "owner"=>2, "adj"=>arr![] }]),
            "unresearched" => world.set("factions", arr![obj! { "id"=>7, "alive"=>true, "powered"=>1, "techs"=>arr![] }]),
            _ => world.set("factions", arr![obj! { "id"=>7, "alive"=>true, "powered"=>0.8, "techs"=>arr!["transmission"] }]),
        };
        let args = views(&[world, buyer]);
        let mut orders = obj! {};
        shared::energy::apply(&mut orders, args[0], args[1]);
        assert!(orders.get("send").is_empty(), "{case}: {orders:?}");
    }
}
