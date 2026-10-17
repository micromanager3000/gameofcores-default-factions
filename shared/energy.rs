// Power, in worlds with energy. A house plans its grid from what every program reads: `me.energy` (its grid last
// second: supply, demand, islands and plants), `world.energy` (the sky, and what each plant kind needs and costs) and
// each province's `power` (where it lies, the sun, wind and tide there now, and the room it has for water, rift and
// reactor plants). The planner builds plants where a builder stands, names the land to darken first, keeps a battery
// reserve for the dark, and sells what it has spare. Research and war stay with each house: `prize` lets a house's
// targeting value a rival's weak grid, its plants and its rich sites.
use std::cell::RefCell;
use std::f64::consts::PI;

use gameofcores::*;

/// A house's way with power.
pub struct Doctrine {
    /// Plant kinds it builds, the ones it likes best first.
    pub kinds: &'static [&'static str],
    /// The supply it plans for, as a multiple of its draw.
    pub margin: f64,
    /// Seconds of its draw its batteries hold, once sun, wind and tide make much of its power.
    pub store: f64,
    /// The share of its batteries it keeps back for the dark.
    pub reserve: f64,
    /// Whether it darkens its outlying land first in a shortfall (otherwise every province dims alike).
    pub shed: bool,
    /// Whether it sells spare power to its neighbours.
    pub sell: bool,
}

/// Each house's doctrine, by founding seat.
pub fn doctrine(seat: usize) -> Doctrine {
    let d = |kinds, margin, store, reserve, shed, sell| Doctrine { kinds, margin, store, reserve, shed, sell };
    match seat {
        // Kublai: dams where the steppe meets the range, boilers in conquered towns
        0 => d(&["hydro", "boiler", "wind", "geo", "solar"], 1.15, 0.0, 0.1, true, false),
        // Arjuna: solar fields and batteries, a reserve before every dusk
        1 => d(&["solar", "wind", "tidal"], 1.3, 240.0, 0.45, true, false),
        // Templars: wind and water mills beside the chapter houses
        2 => d(&["wind", "hydro", "solar", "tidal"], 1.25, 90.0, 0.25, true, false),
        // New Atlantis: geothermal, then reactors, then the race to fusion
        3 => d(&["fusion", "reactor", "geo", "hydro", "solar"], 1.2, 60.0, 0.2, true, false),
        // Quiet Fleet: tidal barrages at its ports, boilers its freighters feed
        4 => d(&["tidal", "boiler", "wind", "solar"], 1.2, 60.0, 0.2, true, false),
        // Kernel Manifesto: dams and boilers everywhere, and every province lit alike
        5 => d(&["hydro", "boiler", "wind", "geo", "solar"], 1.35, 0.0, 0.0, false, false),
        // Null Pointer Society: a lean grid on batteries
        6 => d(&["solar", "wind", "geo"], 1.05, 150.0, 0.3, true, false),
        // Compute of Nations: spare capacity, batteries, and power for sale
        7 => d(&["wind", "hydro", "solar", "tidal", "reactor"], 1.4, 120.0, 0.2, true, true),
        _ => d(&["solar", "wind", "hydro", "boiler", "geo", "reactor"], 1.2, 60.0, 0.2, true, false),
    }
}

/// What a rated MW of `kind` makes in province `p` over a day, on average: sun after the ground and the latitude
/// (the sun stands over the equator), the wind's mean, the tide's, and steady plants in full.
pub fn duty(kind: &str, p: View) -> f64 {
    let w = p.get("power");
    match kind {
        "solar" => w.get("sunFactor").num_or(1.0) * w.get("lat").num_or(0.0).cos() / PI,
        "wind" => w.get("windFactor").num_or(1.0) * 0.6,
        "tidal" => 0.55,
        "hydro" => 0.9,
        "boiler" => 0.85,
        "battery" => 0.0,
        _ => 1.0,
    }
}
fn variable(kind: &str) -> bool { matches!(kind, "solar" | "wind" | "tidal") }

/// Whether the house has the research `kind` needs.
fn open(world: View, me: View, kind: &str) -> bool {
    let k = world.path("energy.kinds").get(kind);
    k.truthy() && k.get("research").iter().all(|t| me.get("techs").get(t.str()).truthy())
}

fn same(a: View, b: View) -> bool { a.is_num() && b.is_num() && a.num() == b.num() }

/// The plants a house can see, as (province, rated MW), worked out once a call.
fn plants_seen(world: View, me: View) -> Vec<(f64, f64)> {
    thread_local! { static SEEN: RefCell<Option<(View, Vec<(f64, f64)>)>> = const { RefCell::new(None) }; }
    if let Some(v) = SEEN.with(|s| s.borrow().as_ref().filter(|(w, _)| w.same(&world)).map(|(_, v)| v.clone())) {
        return v;
    }
    let mut out: Vec<(f64, f64)> = Vec::new();
    for s in me.path("rts.structures").iter().take(160) {
        if s.get("state").str() == "ruin" || same(s.get("owner"), me.get("id")) { continue; }
        let mw: f64 = s.path("power.ports").iter().filter(|q| q.get("kind").str() != "battery").map(|q| q.get("capacity").num_or(0.0)).sum();
        if mw <= 0.0 { continue; }
        let p = s.get("p").num();
        match out.iter_mut().find(|e| e.0 == p) {
            Some(e) => e.1 += mw,
            None => out.push((p, mw)),
        }
    }
    SEEN.with(|s| *s.borrow_mut() = Some((world, out.clone())));
    out
}

/// What a province of a rival's adds to a house's wish to take it, in the units of `common::score`: its owner's grid
/// running short (its miners and army slowed), the plants that would change hands with it, and its water, rift and
/// reactor room.
pub fn prize(world: View, me: View, q: View) -> f64 {
    if !world.get("energy").truthy() { return 0.0; }
    let o = q.get("owner");
    if !o.is_num() || o.num() < 0.0 || o.num() == me.get("id").num() { return 0.0; }
    let weak = world.get("factions").iter().find(|f| same(f.get("id"), o))
        .map_or(0.0, |f| (0.75 - f.get("powered").num_or(1.0)).max(0.0) * 4.0);
    let w = q.get("power");
    let room = w.get("hydro").num_or(0.0) + 0.5 * w.get("tidal").num_or(0.0) + 1.5 * w.get("geo").num_or(0.0);
    let plants = plants_seen(world, me).iter().find(|e| e.0 == q.get("id").num()).map_or(0.0, |e| e.1);
    weak + (room * 0.12).min(1.0) + (plants * 0.35).min(3.0)
}

/// One plant the planner could order next.
struct Pick { p: View, kind: &'static str, cap: f64, cost: f64, score: f64 }

/// The grid for a second: settings every five seconds, a look for a plant to build every three, power for sale once a
/// minute. None of it runs in the second `structures` plans its buildings (every fifth), so the two never share a call.
pub fn apply(orders: &mut Value, world: View, me: View) {
    let e = me.get("energy");
    if !e.truthy() || !world.get("energy").truthy() || cycles_left() < 7000.0 { return; }
    let now = world.get("seconds").num_or(0.0);
    let seat = me.get("seat").num_or(9.0).max(0.0) as usize;
    let d = doctrine(seat);
    fund(orders, me);
    if now % 5.0 == 0.0 { return; }
    if now % 5.0 == 2.0 && cycles_left() > 10000.0 { settings(orders, world, me, &d); }
    if now % 3.0 == 1.0 && cycles_left() > 30000.0 { build(orders, world, me, &d, now); }
    if d.sell && now % 60.0 == 7.0 && cycles_left() > 10000.0 { sell(orders, world, me); }
}

/// A plant going up is charged a twelfth of its build cost a second through the fabrication lane, which recruits share.
/// A compute policy sent this second gets that much more room, so the house's own limit stays for its units.
fn fund(orders: &mut Value, me: View) {
    let fab = orders.get("compute").get("cpu").get("limits").get("fabrication").num_or(0.0);
    if fab <= 0.0 { return; }
    let id = me.get("id").num_or(-1.0);
    let rate = me.path("rts.structures").iter()
        .filter(|s| s.get("owner").num_or(-2.0) == id && s.get("state").str() == "building" && s.get("power").truthy())
        .map(|s| s.get("buildCost").num_or(0.0)).sum::<f64>() * price(me) / 12.0;
    if rate <= 0.0 { return; }
    // a limit of 1 or less is a share of income
    let base = if fab <= 1.0 { fab * me.path("rts.cpu.income").num_or(0.0) } else { fab };
    orders.entry("compute").entry("cpu").entry("limits").set("fabrication", (base + rate).round());
}

/// The land to darken first, and the reserve the batteries keep.
fn settings(orders: &mut Value, world: View, me: View, d: &Doctrine) {
    let e = me.get("energy");
    let id = me.get("id").num();
    let mut shed: Vec<(f64, f64)> = Vec::new();
    if d.shed {
        for p in world.get("provinces").iter().filter(|p| p.get("owner").num_or(-1.0) == id) {
            if same(p.get("id"), me.get("capital")) || p.path("power.draw").num_or(0.0) <= 0.0 { continue; }
            // outlying, restless land goes dark first; the core of the realm keeps its lights
            let rank = p.get("hops").num_or(0.0) * 10.0 + (100.0 - p.get("loyalty").num_or(50.0)) / 10.0;
            shed.push((rank, p.get("id").num()));
        }
        shed.sort_by(|a, b| b.0.total_cmp(&a.0));
    }
    let storage = e.get("storage").num_or(0.0);
    // with the sun up over a solar house's capital, it banks for the night
    let sunny = world.get("provinces").iter().find(|p| same(p.get("id"), me.get("capital")))
        .is_some_and(|p| p.path("power.sun").num_or(0.0) > 0.25);
    let reserve = storage * if sunny && d.store >= 200.0 { d.reserve } else { d.reserve * 0.4 };
    orders.set("grid", obj! { "shed" => shed.iter().take(24).map(|s| Value::Num(s.1)).collect::<Vec<Value>>(), "reserve" => reserve.round() });
}

/// Orders one plant (or battery) when the grid is short of the house's margin and the bank can stand it.
fn build(orders: &mut Value, world: View, me: View, d: &Doctrine, now: f64) {
    if !me.path("rts.structureCosts").truthy() { return; }
    let e = me.get("energy");
    let id = me.get("id");
    let income = me.path("rts.cpu.income").num_or(0.0);
    let bank = me.path("rts.cpu.bank").num_or(0.0);
    // Plants are paid from savings above the floor an army keeps (common::govern drops to its recovery policy below
    // four seconds of income). The floor comes down while the grid runs short, since a dim province mines less.
    let short = e.get("powered").num_or(1.0) < 0.92;
    let floor = income * if short { 2.0 } else { 5.0 };
    if me.path("units.stalled").num_or(0.0) > 0.0 || bank < floor + income { return; }
    let sites = me.path("rts.structures");
    let mine: Vec<View> = sites.iter().filter(|s| same(s.get("owner"), id) && s.get("state").str() != "ruin").collect();
    // two plants at a time at most, and the second only while the grid is well short
    let building = mine.iter().filter(|s| s.get("state").str() == "building" && s.get("power").truthy()).count();
    if building >= 2 { return; }
    let land: Vec<View> = world.get("provinces").iter().filter(|p| same(p.get("owner"), id) && p.get("power").truthy()).collect();
    if land.is_empty() { return; }
    let find = |pid: f64| land.iter().copied().find(|p| p.get("id").num() == pid);
    // what the plants it has make in a day, on average, and how much of that comes and goes with the sky
    let (mut expected, mut fickle) = (e.get("house").num_or(3.0), 0.0);
    for s in &mine {
        let Some(p) = find(s.get("p").num()) else { continue };
        for q in s.path("power.ports").iter() {
            let kind = q.get("kind").str();
            let mw = q.get("capacity").num_or(0.0) * duty(kind, p);
            expected += mw;
            if variable(kind) { fickle += mw; }
        }
    }
    let demand = e.get("demand").num_or(0.0);
    // the house generator carries a young realm; plants follow the draw once the mines outgrow it
    if demand < e.get("house").num_or(3.0) * 0.8 { return; }
    let target = demand * d.margin + 0.5;
    if building == 1 && expected + 1.5 >= target { return; }
    // The first construction second (a twelfth of the quote) comes out of the fabrication lane when the site is
    // accepted. A policy sent this second makes room for it; otherwise the plant fits what the lane has left. The
    // rest is charged over the next twelve seconds.
    let fab = orders.get("compute").get("cpu").get("limits").get("fabrication").num_or(0.0);
    let policy = fab > 1.0;
    let spare = (bank - floor) * 0.7;
    let lane = if policy { spare } else { me.path("rts.cpu.available.fabrication").num_or(0.0) * 0.6 * 12.0 };
    let spend = lane.min(spare);
    let units: Vec<View> = world.get("units").iter().filter(|u| same(u.get("owner"), id) && u.get("hp").num_or(0.0) > 0.0).collect();
    let kinds = world.path("energy.kinds");
    let ids: Vec<f64> = land.iter().map(|p| p.get("id").num()).collect();
    let interior = |p: View| p.get("adj").iter().all(|q| ids.contains(&q.num()));
    let mut picks: Vec<Pick> = Vec::new();
    if expected < target {
        let short = target - expected;
        for (rank, kind) in d.kinds.iter().copied().enumerate() {
            if cycles_left() < 8000.0 { break; }
            if !open(world, me, kind) { continue; }
            let k = kinds.get(kind);
            let (lo, hi, quote) = (k.get("min").num_or(0.1), k.get("max").num_or(8.0), k.get("quote").num_or(1e12));
            for p in land.iter().copied().take(24) {
                if cycles_left() < 9000.0 { break; }
                let w = p.get("power");
                let room = match kind {
                    "hydro" | "tidal" | "geo" | "reactor" => w.get(kind).num_or(0.0),
                    _ => hi,
                };
                let duty = duty(kind, p);
                if room < lo || duty <= 0.05 { continue; }
                // boilers burn the cargo a mining town stores
                if kind == "boiler" && !me.path("rts.logistics.depots").iter().any(|t| same(t.get("p"), p.get("id")) && t.get("cycles").num_or(0.0) > 0.0) { continue; }
                // several small plants rather than one big one: each goes up faster, and a raid takes less
                let most = if matches!(kind, "reactor" | "fusion") { 10.0 } else { 4.0 };
                let cap = (short / duty).min(room.min(hi).min(most)).min(spend / quote);
                let cap = (cap * 10.0).floor() / 10.0;
                if cap < lo { continue; }
                // cheapest MW for the house's taste, safer behind the border
                // (a boiler's fuel is cargo that would have been income: it counts thrice its build price)
                let fuel = if kind == "boiler" { 3.0 } else { 1.0 };
                let score = quote * fuel / duty * (1.0 + 0.4 * rank as f64) * if interior(p) { 1.0 } else { 1.3 };
                picks.push(Pick { p, kind, cap, cost: quote * cap, score });
            }
        }
    }
    // batteries once the sky makes much of the power
    if picks.is_empty() && d.store > 0.0 && fickle > 0.35 * expected && open(world, me, "battery") {
        let want = demand * d.store;
        let have = e.get("storage").num_or(0.0);
        let quote = kinds.path("battery.quote").num_or(1e12);
        // a bank holds a minute of draw; more banks follow
        let cap = (want - have).min(demand * 60.0).min(spend / quote).min(2400.0).round();
        if have < want * 0.8 && cap >= 10.0 {
            let capital = land.iter().copied().find(|p| same(p.get("id"), me.get("capital"))).unwrap_or(land[0]);
            picks.push(Pick { p: capital, kind: "battery", cap, cost: quote * cap, score: 0.0 });
        }
    }
    picks.sort_by(|a, b| a.score.total_cmp(&b.score));
    // the best site with room for the shell and a builder at hand
    for pick in picks.iter().take(4) {
        if cycles_left() < 25000.0 { return; }
        let r = (1.2 + 0.18 * pick.cap.min(10.0)).min(3.2);
        let Some((x, y)) = super::structures::spot(pick.p, sites, r, &units) else { continue };
        let pid = pick.p.get("id").num();
        let site = obj! { "key" => format!("power-{pid:.0}-{now:.0}"), "p" => pid, "x" => x, "y" => y,
            "shell" => obj! { "radius" => r, "height" => 2.0, "integrity" => 1.0 },
            "ports" => arr![obj! { "kind" => pick.kind, "capacity" => pick.cap }],
            "appearance" => look(pick.kind, pick.cap, seat_style(me)), "budget" => pick.cost * 1.2 + 4000.0 * price(me) };
        orders.entry("structures").push(site);
        if policy {
            let deposit = (pick.cost + 2000.0 * price(me)) / 12.0;
            orders.entry("compute").entry("cpu").entry("limits").set("fabrication", (fab + deposit).round());
        }
        return;
    }
}

fn price(me: View) -> f64 { me.path("rts.costs.priceLevel").num_or(1.0).max(0.01) }
fn seat_style(me: View) -> usize { me.get("seat").num_or(9.0).max(0.0) as usize }

/// Offers spare power to the neighbour that's shortest of it, for ◆ a second.
fn sell(orders: &mut Value, world: View, me: View) {
    let e = me.get("energy");
    if !me.get("techs").get("transmission").truthy() { return; }
    let spare = e.get("supply").num_or(0.0) - e.get("demand").num_or(0.0) * 1.1 - e.path("deals").iter().filter(|x| x.get("give").truthy()).map(|x| x.get("mw").num_or(0.0)).sum::<f64>();
    if spare < 1.5 { return; }
    let id = me.get("id").num();
    // neighbours: factions whose land borders ours
    let mut near: Vec<f64> = Vec::new();
    let by = realm::index(world);
    for p in world.get("provinces").iter().filter(|p| p.get("owner").num_or(-1.0) == id) {
        for q in p.get("adj").iter() {
            let o = by.at(q).get("owner").num_or(-1.0);
            if o >= 0.0 && o != id && !near.contains(&o) { near.push(o); }
        }
    }
    let buyer = world.get("factions").iter()
        .filter(|f| f.get("alive").truthy() && near.contains(&f.get("id").num()) && f.get("powered").num_or(1.0) < 0.9)
        .min_by(|a, b| a.get("powered").num_or(1.0).total_cmp(&b.get("powered").num_or(1.0)));
    let Some(buyer) = buyer else { return };
    let mw = (spare * 0.6).min(6.0).floor().max(1.0);
    let give = obj! { "power" => obj! { "mw" => mw, "seconds" => 180 } };
    let ask = obj! { "coinPerSecond" => (mw * 0.12 * 100.0).round() / 100.0, "seconds" => 180 };
    diplomacy::send(orders, buyer.get("id").num(), "power for sale", obj! { "give" => give, "ask" => ask });
}

// ---------------------------------------------------------------------------------------------------------------------
// How plants look. Pieces are in cells from the site's centre (x east, y north, z up), as in `structures`.

/// A plant's appearance: its kind's machinery, more of it the bigger the plant, on the house's ground.
pub fn look(kind: &str, cap: f64, style: usize) -> Value {
    let base = ["stone", "wood", "marble", "glass", "metal", "brick", "dark", "jade", "sand"][style.min(8)];
    let mut pieces: Vec<Value> = vec![obj! { "kind" => "platform", "x" => 0, "y" => 0, "z" => 0, "w" => 2.6, "h" => 0.12, "d" => 2.2, "tone" => base }];
    let mut add = |kind: &str, x: f64, y: f64, z: f64, w: f64, h: f64, d: f64, rot: f64, tone: &str| {
        let r = |v: f64| (v * 100.0).round() / 100.0;
        let mut o = obj! { "kind" => kind, "x" => r(x), "y" => r(y), "z" => r(z), "w" => r(w), "h" => r(h) };
        if d > 0.0 { o.set("d", r(d)); }
        if rot != 0.0 { o.set("rot", r(rot)); }
        if !tone.is_empty() { o.set("tone", tone); }
        pieces.push(o);
    };
    let n = |per: f64, most: usize| ((cap / per).ceil() as usize).clamp(1, most);
    let name = match kind {
        "solar" => {
            for i in 0..n(0.8, 8) {
                let (row, col) = ((i / 4) as f64, (i % 4) as f64);
                add("panel", -0.9 + col * 0.6, -0.55 + row * 0.9, 0.12, 0.52, 0.12, 0.7, 0.0, "");
            }
            add("accent", 1.0, 0.95, 0.12, 0.3, 0.3, 0.3, 0.0, "faction");
            "Solar field"
        }
        "wind" => {
            let m = n(1.5, 4);
            for i in 0..m {
                let a = i as f64 / m as f64 * 6.283 + 0.6;
                let rr = if m == 1 { 0.0 } else { 0.75 };
                add("turbine", rr * a.cos(), rr * a.sin(), 0.12, 0.6, 2.6, 0.0, 0.0, "");
            }
            "Wind farm"
        }
        "hydro" => {
            add("dam", 0.0, 0.2, 0.12, 2.4, 1.4, 0.6, 0.0, "stone");
            add("hall", 0.0, -0.6, 0.12, 1.2, 0.6, 0.6, 0.0, "faction");
            add("light", 0.9, -0.8, 0.12, 0.14, 0.7, 0.0, 0.0, "gold");
            "Dam"
        }
        "tidal" => {
            add("dam", 0.0, 0.0, 0.0, 2.6, 0.7, 0.5, 0.0, "metal");
            for i in 0..n(2.0, 3) { add("turbine", -0.8 + i as f64 * 0.8, 0.0, 0.7, 0.4, 0.9, 0.0, 0.0, "metal"); }
            "Tidal barrage"
        }
        "boiler" => {
            add("hall", -0.2, 0.3, 0.12, 1.5, 1.0, 1.0, 0.0, "brick");
            for i in 0..n(2.0, 3) { add("stack", 0.9 - i as f64 * 0.45, 0.6, 0.12, 0.28, 2.2, 0.0, 0.0, "dark"); }
            add("drum", -0.7, -0.7, 0.12, 0.6, 0.6, 0.0, 0.0, "metal");
            "Boiler house"
        }
        "geo" => {
            add("drum", 0.0, 0.2, 0.12, 1.1, 0.8, 0.0, 0.0, "metal");
            add("dome", 0.0, 0.2, 0.92, 1.1, 0.5, 0.0, 0.0, "faction");
            add("stack", -0.9, 0.7, 0.12, 0.25, 1.6, 0.0, 0.0, "metal");
            add("light", 0.9, -0.7, 0.12, 0.16, 0.6, 0.0, 0.0, "jade");
            "Geothermal well"
        }
        "reactor" => {
            for i in 0..n(4.0, 3) { add("cooling", -0.8 + i as f64 * 0.8, 0.6, 0.12, 0.8, 2.4, 0.0, 0.0, "marble"); }
            add("dome", 0.2, -0.6, 0.12, 1.2, 1.1, 0.0, 0.0, "metal");
            add("hall", -0.8, -0.6, 0.12, 0.7, 0.6, 0.7, 0.0, "faction");
            "Reactor"
        }
        "fusion" => {
            add("torus", 0.0, 0.0, 0.5, 2.2, 0.6, 0.0, 0.0, "metal");
            add("drum", 0.0, 0.0, 0.12, 0.5, 0.9, 0.0, 0.0, "faction");
            add("light", 0.0, 0.0, 1.0, 0.3, 0.3, 0.0, 0.0, "gold");
            "Fusion plant"
        }
        _ => {
            for i in 0..n(200.0, 6) {
                let (row, col) = ((i / 3) as f64, (i % 3) as f64);
                add("cell", -0.7 + col * 0.7, -0.4 + row * 0.8, 0.12, 0.5, 0.7, 0.5, 0.0, "");
            }
            "Battery bank"
        }
    };
    obj! { "name" => name, "pieces" => pieces }
}
