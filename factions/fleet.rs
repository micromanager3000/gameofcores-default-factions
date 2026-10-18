// The Quiet Fleet mines, trades and fights by land and sea.

use gameofcores::*;
use shared::{air, capital, common, theme};
use shared::common::{hauler, freighter, escort};

program!(think, hauler, freighter, escort, miner, runner, lancer, shade, sentry, spire, needle, kite, scout, warjunk, picket, corsair, carrier, reflex, research, duel, challenge, look, build3d, live3d);

// Bodies spend ten points across combat, carry and vision.
fn miner_body() -> Value {
    obj! { "hp" => 2, "speed" => 2, "strength" => 0, "carry" => 5, "vision" => 1 }
}
fn runner_body() -> Value {
    obj! { "hp" => 2, "speed" => 3, "strength" => 2, "carry" => 0, "vision" => 3 }
}
fn shade_body() -> Value {
    obj! { "hp" => 1, "speed" => 3, "strength" => 2, "carry" => 0, "vision" => 2, "range" => 1 }
}
fn sentry_body() -> Value {
    obj! { "hp" => 3, "speed" => 1, "strength" => 2, "carry" => 0, "vision" => 4 }
}
fn needle_body() -> Value {
    obj! { "hp" => 2, "speed" => 3, "strength" => 2, "carry" => 0, "vision" => 1, "range" => 1 }
}
fn scout_body() -> Value {
    obj! { "hp" => 1, "speed" => 4, "strength" => 0, "carry" => 0, "vision" => 4 }
}
fn spire_body() -> Value {
    obj! { "hp" => 1, "speed" => 1, "strength" => 2, "carry" => 0, "vision" => 2, "range" => 2 }
}
// Flight unlocks kites; Air weapons let them attack rival miners.
fn kite_body() -> Value {
    obj! { "hp" => 2, "speed" => 3, "strength" => 2, "carry" => 0, "vision" => 0, "wings" => 1 }
}
fn kite_shot_body() -> Value {
    obj! { "hp" => 2, "speed" => 2, "strength" => 1, "carry" => 0, "vision" => 0, "wings" => 1, "range" => 1 }
}

// Junks spend ten points on hull and module; range costs two each.
fn warjunk_body() -> Value {
    obj! { "hp" => 3, "speed" => 2, "strength" => 2, "carry" => 0, "vision" => 1, "range" => 1 }
}
fn picket_body(guns: bool) -> Value {
    if guns {
        obj! { "hp" => 3, "speed" => 1, "strength" => 2, "carry" => 0, "vision" => 2, "range" => 1 }
    } else {
        obj! { "hp" => 5, "speed" => 2, "strength" => 1, "carry" => 0, "vision" => 2 }
    }
}
fn corsair_body(cargo: bool) -> Value {
    if cargo { return obj! { "hp"=>2,"speed"=>2,"strength"=>2,"carry"=>1,"vision"=>1,"range"=>1 }; }
    obj! { "hp" => 2, "speed" => 4, "strength" => 2, "carry" => 0, "vision" => 0, "range" => 1 }
}
// One carrier transports two kites and gains armor when researched.
fn carrier_body() -> Value {
    obj! { "hp" => 3, "speed" => 2, "strength" => 1, "carry" => 2, "vision" => 1 }
}

const FAITHS: [&str; 4] = ["The Silent Cache", "The Order of the Dark Fibre", "The Church of the Idle Loop", "The Brotherhood of the Hidden Bit"];

const TECHS: [&str; 21] = [
    "extraction", "seaTraversal", "turbines", "ranged", "armor", "dash", "cloak", "scan", "influence", "faith",
    "pierce", "artillery", "antiAir", "repair", "flight", "airWeapons", "mountainTraversal", "storage", "transmission",
    "subseaCable", "gate",
];

const FIRM_GRID_TECHS: [&str; 21] = [
    "extraction", "seaTraversal", "turbines", "ranged", "armor", "dash", "cloak", "scan", "influence", "faith",
    "pierce", "artillery", "antiAir", "repair", "flight", "storage", "airWeapons", "mountainTraversal", "transmission",
    "subseaCable", "gate",
];

// A dim grid gets Storage only with average variable generation to charge it.
fn storage_can_firm(world: View, me: View) -> bool {
    let energy = me.get("energy");
    if !energy.truthy() || energy.get("powered").num_or(1.0) >= 0.9 || me.get("techs").get("storage").truthy() {
        return false;
    }
    let Some(home) = energy.get("islands").iter().find(|i| i.get("capital").truthy()) else { return false };
    let demand = home.get("demand").num_or(0.0);
    if demand < 0.5 || home.get("storage").num_or(0.0) >= demand * 48.0 {
        return false;
    }
    let mut average = if home.get("capital").truthy() { energy.get("house").num_or(3.0) } else { 0.0 };
    let mut variable = 0.0;
    for plant in energy.get("plants").iter().filter(|p| p.get("state").str() == "ready" && home.get("provinces").has_num(p.get("p").num())) {
        let Some(province) = world.get("provinces").iter().find(|q| q.get("id").num() == plant.get("p").num()) else { continue };
        let site = province.get("power");
        for port in plant.get("ports").iter() {
            let kind = port.get("kind").str();
            let capacity = port.get("capacity").num_or(0.0);
            let duty = match kind {
                "solar" => site.get("sunFactor").num_or(1.0) * site.get("lat").num_or(0.0).cos() / std::f64::consts::PI,
                "wind" => site.get("windFactor").num_or(1.0) * 0.6,
                "tidal" => 0.55,
                "hydro" => 0.9,
                "boiler" => 0.85,
                "battery" => 0.0,
                _ => 1.0,
            };
            let output = capacity * duty;
            average += output;
            if matches!(kind, "solar" | "wind" | "tidal") { variable += output; }
        }
    }
    // Require near-demand average output; batteries cannot fix an undersized grid.
    home.get("powered").num_or(1.0) < 0.9 && variable >= demand * 0.35 && average >= demand * 0.9
}

fn research_techs(world: View, me: View) -> &'static [&'static str] {
    if storage_can_firm(world, me) { &FIRM_GRID_TECHS } else { &TECHS }
}

// ---- JavaScript arithmetic compatibility ----

/// `Math.max(a, b)`: NaN when either is.
fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}

/// `Math.min(a, b)`: NaN when either is.
fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.min(b)
    }
}

/// `Math.round`: halves go up.
fn round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// `Number(v)` for a value read back from RAM (null is 0).
fn num(v: &Value) -> f64 {
    match v {
        Value::Num(x) => *x,
        Value::Null => 0.0,
        Value::Bool(b) => f64::from(u8::from(*b)),
        _ => f64::NAN,
    }
}

/// `x || d` for a number read back from RAM.
fn or(v: &Value, d: f64) -> f64 {
    if v.truthy() {
        num(v)
    } else {
        d
    }
}

/// `a === b` for a value the game gives and a number, null or undefined.
fn same(a: View, b: &Value) -> bool {
    match b {
        Value::Undef => a.is_undef(),
        Value::Null => a.is_null(),
        Value::Num(x) => a.is_num() && a.num() == *x,
        _ => false,
    }
}

/// A number as text, as JavaScript writes it.
fn text(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n.is_infinite() {
        (if n > 0.0 { "Infinity" } else { "-Infinity" }).into()
    } else if n.fract() == 0.0 && n.abs() < 1e21 {
        format!("{}", n as i128)
    } else {
        format!("{n}")
    }
}

/// `orders[key] = (orders[key] || []).concat(items)`.
fn concat(orders: &mut Value, key: &str, items: Vec<Value>) {
    match orders.entry(key) {
        Value::Arr(a) => a.extend(items),
        slot => *slot = Value::Arr(items),
    }
}

/// `me.rts`, the CPU RTS state: whether the world runs on a shared CPU bank.
fn rts(me: View) -> bool {
    me.get("rts").truthy()
}

/// The unit whose turn it is: the list starts one unit further along each second, so every unit gets to go first.
fn nth(units: View, turn: usize, world: View) -> View {
    units.key((turn as f64 + world.get("seconds").or(0.0).floor()) % units.len() as f64)
}

/// The control lane's cycles left this second (NaN when the game doesn't say).
fn control(me: View) -> f64 {
    me.get("rts").get("cpu").get("available").get("control").num()
}

/// A unit type's call can't pay for the plan lookup and an order: under `cycles` cycles, or the control lane under `lane`.
fn short(me: View, cycles: f64, lane: f64) -> bool {
    rts(me) && (cycles_left() < cycles || (me.get("rts").get("cpu").truthy() && control(me) < lane))
}

// ---- Clocks ----

// Trade and sway timers are jittered and stored on RAM page 224.
const CLOCK: u32 = 224;

struct Clock {
    at: Value,
    changed: bool,
}

fn due(clock: &mut Clock, name: &str, base: f64, now: f64) -> bool {
    let next = clock.at.get(name).clone();
    if !next.is_undef() && now < num(&next) {
        return false;
    }
    let gap = if next.is_undef() { rand() } else { 0.4 + 1.2 * rand() };
    clock.at.set(name, now + round(base * gap));
    clock.changed = true;
    !next.is_undef()
}

// ---- think ----

/// The world's tick (the unit a ship's volley is stamped in).
fn now_tick(world: View) -> f64 {
    world.get("tick").num_or(0.0)
}

/// Rivals' units in province `q` (one when it's a rival's and the game doesn't say).
fn rival_in(q: View, me: View) -> f64 {
    let me_id = me.get("id").num();
    if !q.truthy() || !q.get("units").truthy() {
        let owner = q.get("owner").num();
        return if q.truthy() && owner >= 0.0 && owner != me_id { 1.0 } else { 0.0 };
    }
    let allies = me.get("allies");
    let mut n = 0.0;
    for (f, k) in q.get("units").entries() {
        let f: f64 = f.parse().unwrap_or(f64::NAN);
        if f != me_id && !allies.has_num(f) {
            n += k.num();
        }
    }
    n
}

/// Whether the army can hold on to province `q` once it takes it (common::can_hold, cheaper): it's on our land border
/// or over a pass, or it's the landing the fleet planned last second and a junk of ours keeps station off it (on the
/// water the plan named, or one next to it), so the capital's orders cross to the marines there.
fn fleet_hold(world: View, me: View) -> impl Fn(View) -> bool {
    let me_id = me.get("id").num();
    let by_id = realm::index(world);
    let nv = mem::get(me.get("ramCapital"), 0).get("nv").clone();
    let (lt, lw) = (nv.get("lt").num_or(-1.0), nv.get("lw").num_or(-1.0));
    let covered = lt >= 0.0 && lw >= 0.0 && {
        let near = sea::index(world).get(lw).get("adj");
        world.get("units").iter().any(|u| {
            u.get("owner").num() == me_id && {
                let s = u.get("rts").get("sea");
                s.get("ship").truthy() && (s.get("water").num() == lw || near.has_num(s.get("water").num_or(-1.0)))
            }
        })
    };
    move |q: View| {
        let ours = |id: View| by_id.at(id).get("owner").num() == me_id;
        q.get("adj").iter().any(ours) || q.get("routes").get("pass").iter().any(ours) || (covered && q.get("id").num() == lt)
    }
}

// Prioritize cores, rival land and prizes we can hold.
fn frontline(world: View, me: View, bonus: &[f64]) -> Option<View> {
    let me_id = me.get("id").num();
    let hold = fleet_hold(world, me);
    let by_id = realm::index(world);
    let mut best = None;
    let mut score = -1e9;
    for q in common::frontier(world, me) {
        let owner = q.get("owner").num();
        let hostile = rival_in(q, me);
        let enemy = owner >= 0.0 && owner != me_id;
        let held = hold(q);
        let landing = held && !q.get("adj").iter().any(|a| by_id.at(a).get("owner").num() == me_id);
        let s = q.get("cores").or(0.0) * 3.0 + if enemy { 9.0 } else { 2.0 } + hostile * 2.0 - if q.get("capital").truthy() { 1.0 } else { 0.0 }
            - if held { 0.0 } else { common::OVER_WATER }
            + if landing { 2.0 } else { 0.0 }
            + if bonus.contains(&q.get("id").num()) { 4.0 } else { 0.0 };
        if s > score {
            score = s;
            best = Some(q);
        }
    }
    best
}

/// What think saw of its rivals' units: fliers, shooters and all, and their miners at work by province.
#[derive(Default)]
struct Threats {
    air: f64,
    ranged: f64,
    total: f64,
    /// (province, miners at work there), in the order first seen.
    miners: Vec<(f64, f64)>,
    /// Our junks afloat, and the broadsides they fired since tick `since` (each ship's last volley).
    ships: f64,
    volleys: f64,
    /// The waters our junks are on.
    waters: Vec<f64>,
    /// Crossings rivals are rowing over now: (from, to) province pairs, the lower id first.
    rowing: Vec<(f64, f64)>,
}

// Visible enemy counts guide composition; reflex picks targets.
fn seen_threats(world: View, me: View, since: f64) -> Threats {
    let me_id = me.get("id").num();
    let mut t = Threats::default();
    for u in world.get("units").iter() {
        let owner = u.get("owner").num();
        if owner == me_id {
            let sea = u.get("rts").get("sea");
            if sea.get("ship").truthy() {
                t.ships += 1.0;
                t.waters.push(sea.get("water").num_or(-1.0));
                let v = sea.get("volley");
                if v.get("kind").str() == "broadside" && v.get("t").num_or(0.0) > since {
                    t.volleys += 1.0;
                }
            }
            continue;
        }
        if !realm::is_rival(world, me, owner) {
            continue;
        }
        t.total += 1.0;
        // rowing: on its way between two provinces a sea route joins
        let (at, to) = (u.get("at").num(), u.get("to").num_or(-1.0));
        if to >= 0.0 && to != at && t.rowing.len() < 6 {
            let pair = (at.min(to), at.max(to));
            if !t.rowing.contains(&pair) && realm::index(world).get(at).get("routes").get("sea").has_num(to) {
                t.rowing.push(pair);
            }
        }
        let build = u.get("build");
        if build.get("wings").truthy() {
            t.air += 1.0;
            continue;
        }
        if build.get("range").truthy() {
            t.ranged += 1.0;
        }
        if u.get("doing").str() == "gather" || build.get("carry").num() >= 3.0 {
            let at = u.get("at").num();
            match t.miners.iter_mut().find(|m| m.0 == at) {
                Some(m) => m.1 += 1.0,
                None => t.miners.push((at, 1.0)),
            }
        }
    }
    t
}

// Grow the force while upkeep stays within the income share.
fn army_room(me: View) -> f64 {
    let u = me.get("units");
    max(u.get("cap").num() - u.get("total").num() - u.get("queued").num() - u.get("hiredOut").or(0.0), common::room(me, 0.4))
}

fn add_spawn(orders: &mut Value, me: View, ty: &str, build: Value, modules: &[&str]) -> bool {
    if army_room(me) <= 0.0 || me.get("bank").num() < me.get("costs").get("unit").num() + 4.0 {
        return false;
    }
    // `compute.cpu` applies before same-think fabrication. Its final policy is
    // checked by `affordable` below, rather than rejecting a zero lane left by
    // the recovery policy from the preceding second.
    if !orders.get("spawn").truthy() {
        orders.set("spawn", Value::arr());
    }
    let queued = orders.get("spawn").len();
    if queued >= 2 {
        return false;
    }
    let units = me.get("units");
    let n = units.get("spawned").num() + units.get("queued").num() + queued as f64 + 1.0;
    let title = if build.get("wings").truthy() {
        theme::of(me.get("seat"), 0.0).map(|t|t.airframe(air::Role::of(&build)).name).unwrap_or(ty)
    } else if modules.contains(&"hull") {
        theme::of(me.get("seat"), 0.0).and_then(|t| t.naval_name(ty)).unwrap_or(ty)
    } else {
        ty
    };
    let s = obj! { "type" => ty, "name" => format!("{title} {}", text(n)), "build" => build, "modules" => modules.to_vec() };
    orders.entry("spawn").push(s);
    true
}

/// Establish a small Flight wing once the opening mines can sustain the house.
/// The first two aircraft unlock the carrier order in `naval_spawn`; limiting
/// replacements to this same pair avoids filling the yard with unsupported air.
fn starter_kite(orders: &mut Value, me: View, miners: f64, room: f64, enemy_air: f64, specialist_bank: bool) -> bool {
    let have = me.get("units").get("byType");
    if !specialist_bank
        || !me.path("techs.flight").truthy()
        || enemy_air > 0.0
        || miners < min(room, 4.0)
        || have.get("kite").or(0.0) >= 2.0
    {
        return false;
    }
    let frame = common::airframe(me, have.get("kite").or(0.0));
    add_spawn(orders, me, "kite", frame.build(), if me.path("techs.dash").truthy() { &["dash"][..] } else { &[][..] })
}

/// `bytes.length`: a RAM op's bytes, as a byte string or a list.
fn byte_len(b: &Value) -> f64 {
    match b {
        Value::Bytes(x) => x.len() as f64,
        Value::Arr(x) => x.len() as f64,
        Value::Str(s) => s.chars().count() as f64,
        _ => f64::NAN,
    }
}

// Spawn within the bank left after deals and bribes.
fn payable(mut orders: Value, me: View) -> Value {
    let spawn = orders.get("spawn");
    if !spawn.truthy() || spawn.is_empty() {
        return orders;
    }
    let yes = orders.get("accept").items();
    let mut left = me.get("bank").num();
    let mut price = me.get("costs").get("unit").num();
    for m in me.get("inbox").iter() {
        let id = m.get("id").num();
        if m.get("ask").truthy() && yes.iter().any(|y| y.num() == id) {
            left -= m.get("ask").get("coin").or(0.0);
        }
    }
    for b in orders.get("bribe").items() {
        let coin = b.get("coin");
        left -= if coin.truthy() { coin.num() } else { 0.0 };
    }
    let kept: Vec<Value> = spawn
        .items()
        .iter()
        .filter(|_| {
            if left < price {
                return false;
            }
            left -= price;
            price += 4.0;
            true
        })
        .cloned()
        .collect();
    orders.set("spawn", kept);
    orders
}

// Fabrication follows think, RAM and other lane reserves.
fn affordable(mut orders: Value, me: View, start: f64) -> Value {
    orders = payable(orders, me);
    let c = me.get("rts").get("cpu");
    let policy = orders.get("compute").get("cpu").clone();
    let spawn = orders.get("spawn");
    if !spawn.truthy() || spawn.is_empty() || !c.truthy() || !policy.truthy() {
        return orders;
    }
    let income = c.get("income").or(0.0);
    let amount = |v: f64| if v <= 1.0 { v * income } else { v };
    let spent = c.get("spent");
    let mut used = start - cycles_left() + 1000.0;
    for op in orders.get("ram").items() {
        let b = op.get("bytes");
        if b.truthy() {
            used += byte_len(b) / 2.0;
        }
    }
    let mut held = 0.0;
    if let Value::Obj(reserve) = policy.get("reserve") {
        for (lane, v) in &reserve.0 {
            if lane != "fabrication" {
                held += max(0.0, amount(num(v)) - spent.get(lane).or(0.0) - if lane == "think" { used } else { 0.0 });
            }
        }
    }
    let limits = policy.get("limits");
    let limit = if limits.truthy() && !limits.get("fabrication").is_undef() {
        amount(num(limits.get("fabrication"))) - spent.get("fabrication").or(0.0)
    } else {
        f64::INFINITY
    };
    // Leave a margin for unit calls after think.
    let mut room = min(limit, c.get("bank").or(0.0) - used - held) - 1500.0;
    let kept: Vec<Value> = orders
        .get("spawn")
        .items()
        .iter()
        .filter(|s| {
            let cost = common::fabrication(me, s.get("modules").len() as f64);
            if room < cost {
                return false;
            }
            room -= cost;
            true
        })
        .cloned()
        .collect();
    orders.set("spawn", kept);
    orders
}

// Reuse the route field; search only when its cost and runway fit.
fn road(world: View, me: View, to: f64, after: f64) -> Option<Value> {
    if to.is_nan() || to < 0.0 {
        return None;
    }
    let by_id = realm::index(world);
    let ok = path::friendly(world, me);
    let me_id = me.get("id").num();
    let mut from: Vec<f64> = Vec::new();
    let mut apart = false;
    for p in world.get("provinces").iter() {
        if p.get("owner").num() != me_id {
            continue;
        }
        from.push(p.get("id").num());
        let hops = p.get("hops").num();
        if hops > 0.0
            && !p.get("adj").iter().any(|n| {
                let q = by_id.at(n);
                q.truthy() && q.get("owner").num() == me_id && q.get("hops").num() == hops - 1.0
            })
        {
            apart = true;
        }
        for n in realm::reach(p, me) {
            let q = by_id.get(n);
            if q.truthy() && q.get("owner").num() != me_id && ok(q) {
                from.push(n);
            }
        }
    }
    if !apart && path::home(world, me, me.get("capital").num(), to).is_some() {
        return None;
    }
    let now = world.get("seconds").or(0.0).floor();
    let last = mem::get(me.get("ramCapital"), 0);
    let kept = if last.truthy() && last.get("field").truthy() && num(last.get("field").get("to")) == to { Some(last.get("field").clone()) } else { None };
    if let Some(k) = &kept {
        if now - or(k.get("at"), 0.0) < 10.0 {
            return kept;
        }
    }
    if cycles_left() < 300.0 * world.get("provinces").len() as f64 + after {
        return kept;
    }
    let mut field = path::field(world, me, to, &path::Opts { ok: Some(&ok), from: Some(&from), keep: Some(after), ..path::Opts::default() });
    field.set("at", now);
    Some(field)
}

fn cpu_policy(me: View, offer: f64, under_attack: bool) -> Value {
    let c = me.get("rts").get("cpu");
    let old = c.get("reserve").get("research").or(0.0);
    let income = c.get("income").or(14000.0);
    // Save larger research offers before starting them.
    let capacity = income * 3.0;
    // Keep enough for planning and reflexes while research waits for funds.
    let research_cap = max(0.0, capacity - 18000.0);
    // Reserve research cycles while quiet; threats release them for combat.
    let offered = offer != 0.0 && !offer.is_nan();
    let research = if under_attack || !offered || offer > research_cap {
        0.0
    } else {
        min(min(offer, income * 0.6 + c.get("bank").or(0.0) * 0.18), old + max(2500.0, income * 0.18))
    };
    // Scale planning allocation with the army; leave unused cycles in the bank.
    let think = round(min(
        max(64000.0, income * 0.5),
        max(max(24000.0 + me.get("units").get("total").or(0.0) * 200.0, income * 0.4), me.get("compute").get("thinkPeak").or(0.0) * 1.5),
    ));
    obj! {
        "limits" => obj! {
            "think" => think, "control" => round(max(50000.0, income * 0.5)), "reflex" => 50000,
            "research" => 0.3,
            "upkeep" => 0.72, "drive" => 0.34, "armor" => 0.26, "cloak" => 0.18, "weapon" => 0.48,
            // Fabrication gets a third of income or a two-module body budget.
            "abilities" => 0.28, "fabrication" => round(max(income * 0.34, common::fabrication(me, 2.0) + 300.0 * common::price_level(me))), "gate" => 0.08, "duel" => 0.14, "challenge" => 0.08, "visuals" => 0.03,
        },
        "reserve" => obj! { "think" => 3500, "reflex" => 7000, "research" => research },
        "upkeepOrder" => vec!["miner", "picket", "sentry", "warjunk", "carrier", "spire", "shade", "corsair", "kite", "needle", "scout", "runner"],
    }
}

// Damaged factions keep orders and reflexes instead of planning.
fn full_think_need(world: View, me: View) -> f64 {
    // Scale the planning floor with map size and the last peak.
    let size = 40000.0 + world.get("provinces").len() as f64 * 180.0 + world.get("units").len() as f64 * 90.0;
    size.max(me.get("compute").get("thinkPeak").or(0.0) * 1.25 + 6000.0)
}

fn lean_think_need(world: View, me: View) -> f64 {
    // Optional census, route and naval scans have separate guards below.
    10000.0 + world.get("provinces").len() as f64 * 40.0 + me.get("units").get("total").or(0.0) * 60.0
}

fn budgeted_think_need(world: View, me: View, spend_cap: f64) -> f64 {
    let full = full_think_need(world, me);
    if spend_cap >= full + PLAN_REMAINING_MARGIN + PLAN_CALLBACK_OVERHEAD {
        full
    } else {
        lean_think_need(world, me)
    }
}

fn plan_fuel_ready(allowance: f64, remaining: f64, need: f64) -> bool {
    allowance >= need && remaining >= need + 4000.0
}

const PLAN_REMAINING_MARGIN: f64 = 4000.0;
const PLAN_CALLBACK_OVERHEAD: f64 = 2500.0;

fn affordable_plan_limit(me: View, need: f64) -> Option<f64> {
    let cpu = me.path("rts.cpu");
    let funded = cpu.get("bank").or(0.0) + cpu.get("income").or(20000.0).max(1.0);
    let spend_cap = cpu.get("spendCap").or(cpu.get("burst").or(funded));
    let required = need + PLAN_REMAINING_MARGIN + PLAN_CALLBACK_OVERHEAD;
    let ceiling = required.min(funded).min(spend_cap);
    (ceiling >= required).then_some(ceiling)
}

fn recovery_for_plan(world: View, me: View, need: f64) -> Value {
    let mut orders = common::recovery(world, me);
    let cpu = me.path("rts.cpu");
    let income = cpu.get("income").or(20000.0).max(1.0);
    let bank = cpu.get("bank").or(0.0);
    let spend_cap = cpu.get("spendCap").or(cpu.get("burst").or(bank + income));
    // Respect the second’s hard spend cap.
    let cheap_ceiling = orders.get("compute").get("cpu").get("limits").get("think").num().min(spend_cap);
    orders.entry("compute").entry("cpu").entry("limits").set("think", cheap_ceiling);
    // Use recovery when the selected planning pass does not fit.
    if bank >= income * 2.0 {
        if let Some(ceiling) = affordable_plan_limit(me, need) {
            orders.entry("compute").entry("cpu").entry("limits").set("think", ceiling);
        }
    }
    orders
}

// A house can have enough income to keep its crew and an active study, yet not
// enough of this second's capped spend to run Fleet's map planner. Keep that
// productive work alive without attempting routes, census, or naval scans.
fn lean_rebuild(world: View, me: View) -> Value {
    let mut orders = common::recovery(world, me);
    let studying = me.path("research.tech").str();
    if studying.is_empty() {
        let all = world.get("techs");
        let mine = me.get("techs");
        for &want in research_techs(world, me) {
            if cycles_left() < 4000.0 { break; }
            let mut pick = want;
            for _ in 0..8 {
                let tech = all.get(pick);
                let missing = tech.get("requires").iter().find(|t| !mine.get(t.str()).truthy());
                match missing { Some(t) => pick = t.str(), None => break }
            }
            let tech = all.get(pick);
            if !mine.get(pick).truthy() && tech.truthy()
                && tech.get("requires").iter().all(|t| mine.get(t.str()).truthy())
                && tech.get("offer").num_or(0.0) <= me.path("rts.cpu.capacity").num_or(f64::INFINITY) {
                orders.set("research", pick);
                break;
            }
        }
    }
    let mut policy = cpu_policy(me, 0.0, false);
    common::fit_cpu(&mut policy, me);
    orders.entry("compute").set("cpu", policy);
    orders.set("memo", "Fleet rebuilding crew and research");
    orders
}

fn rich_rebuild_ready(me: View) -> bool {
    let cpu = me.path("rts.cpu");
    cpu.truthy() && me.path("units.stalled").num_or(0.0)==0.0
        && cpu.get("bank").num_or(0.0)>=cpu.get("income").num_or(0.0)*2.0
}

// This is about the per-second hard cap, not merely the callback's temporary
// allowance. If that cap can buy the selected planner, recovery posts its
// normal retry rather than replacing it with bounded work.
fn rich_rebuild_is_capped(world: View, me: View) -> bool {
    if !rich_rebuild_ready(me) { return false; }
    let cap = me.path("rts.cpu.spendCap").or(me.path("rts.cpu.burst").or(f64::INFINITY));
    affordable_plan_limit(me, budgeted_think_need(world, me, cap)).is_none()
}

fn retirement_ready(world: View, me: View, remaining: f64) -> bool {
    if !world.get("logistics").truthy() || me.path("units.byType.corsair").num_or(0.0) <= 0.0 || remaining < 5000.0 { return false; }
    let bytes = me.get("ramCapital").key(0.0).get("bytes");
    let read = if bytes.is_bytes() && bytes.len() >= 2 {
        let b=bytes.bytes(); 4.0*f64::from(u16::from_le_bytes([b[0],b[1]]))
    } else {0.0};
    remaining >= 5000.0 + read
}

fn think(world: View, me: View) -> Value {
    if cycles_left()<800.0 {return obj!{};}
    // A rich, paid crew can use a capped second to maintain its active study
    // and fabrication policy even when that cap is too small for the planner.
    if cycles_left()>=6000.0 && rich_rebuild_is_capped(world, me) {
        return lean_rebuild(world, me);
    }
    let mut hunting=None;
    let mut out=common::govern(world, me, 0.4, || strategy(world, me, &mut hunting));
    // Only a fresh naval decision can retire a hull; recovery and resting preserve it.
    if hunting!=Some(false) || !retirement_ready(world,me,cycles_left()) {return out;}
    if cycles_left()>5000.0 {
        for u in world.get("units").iter() {
            if cycles_left()<3500.0 {break;}
            if u.get("owner").num()!=me.get("id").num() || u.get("type").str()!="corsair" {continue;}
            let cargo=u.path("rts.cargo");
            if cargo.get("cycles").num_or(0.0)==0.0 && !u.get("hired").truthy() && !cargo.get("job").truthy()
                && quarry(u,me,world,2).is_none() && common::salvage_target(u,world,me).is_none() && !out.get("disband").items().iter().any(|v|v.num()==u.get("id").num()) {
                out.entry("disband").push(u.get("id").value());
            }
        }
    }
    out
}

fn strategy(world: View, me: View, hunting: &mut Option<bool>) -> Value {
    // Leave this callback runway for recovery policy.
    let cpu = me.path("rts.cpu");
    let spend_cap = cpu.get("spendCap").or(cpu.get("burst").or(0.0));
    let need = budgeted_think_need(world, me, spend_cap);
    if rts(me) && cycles_left() < 3000.0 {
        return recovery_for_plan(world, me, need);
    }
    let start = cycles_left();
    // Skip over-budget plans; optional scans retain their own guards.
    let available = me.get("rts").get("cpu").get("available");
    if rts(me) {
        let allowance = if available.truthy() { available.get("think").num_or(0.0) } else { cycles_left() };
        if !plan_fuel_ready(allowance, cycles_left(), need) {
            return if rich_rebuild_is_capped(world, me) { lean_rebuild(world, me) } else { recovery_for_plan(world, me, need) };
        }
    }
    let me_id = me.get("id").num();
    let techs = me.get("techs");
    let has = |t: &str| techs.get(t).truthy();
    let have = me.get("units").get("byType");
    let logbook = mem::get(me.get("ramCapital"), LOG);
    let intel = prize_intel(world, me, &logbook);
    let target = frontline(world, me, &intel.bonus);
    let stage = match target.and_then(realm::from) {
        Some(from) => Value::Num(from),
        None => me.get("capital").value(),
    };
    let target_id = target.map_or(-1.0, |t| t.get("id").num());
    let mines: Vec<Value> = realm::open_mines(world, me, 2.0, 12.0).into_iter().map(|[p, k]| arr![p, k]).collect();
    let mut plan = obj! { "target" => target_id, "stage" => stage, "home" => me.get("capital"), "mines" => mines };
    // Defend the capital or major landing against visible invaders.
    common::defense(world, me, &mut plan);
    let defend = plan.get("defend").num_or(-1.0);
    let threatened = plan.get("threat").num_or(0.0) > 0.0;
    // Remember partner land beyond a refusal second.
    let partner = common::partner_land(world, me);
    if !partner.is_empty() {
        plan.set("partner", Value::Arr(partner.iter().map(|&id| Value::Num(id)).collect()));
    }
    if let Some(field) = road(world, me, if defend >= 0.0 { defend } else { target_id }, 20000.0) {
        plan.set("field", field);
    }
    if intel.warn >= 0.0 {
        plan.set("warn", intel.warn);
    }
    // Run the rival census only when think has its full runway.
    let census = cycles_left() > 55.0 * world.get("units").len() as f64 + 20000.0;
    let enemy = if census { seen_threats(world, me, logbook.get("tk").num_or(0.0)) } else { Threats::default() };
    // Keep standing orders when little think fuel remains.
    let naval = if cycles_left() > 26000.0 {
        let naval=naval_plan(world, me, &mut plan, target_id, defend, &enemy.rowing);
        *hunting=Some(naval.raid>0);
        naval
    } else {
        let last = mem::get(me.get("ramCapital"), 0).get("nv").clone();
        let n = Naval {
            stations: last.get("st").len(),
            landing: last.get("lt").num_or(-1.0),
            landing_name: String::new(),
            fleet: last.get("fw").num_or(-1.0),
            blockade: last.get("bx").len(),
            raid: last.get("rw").len(),
        };
        if last.truthy() {
            plan.set("nv", last);
        }
        n
    };
    if has("flight") {
        plan.set("prey", prey(world, me, &enemy.miners));
    }
    let mut orders = obj! {
        "policy" => obj! { "extraction" => 0.42, "autonomy" => "direct", "integrate" => "tolerate", "invest" => true },
        "provinces" => Value::arr(),
    };
    common::share(&mut orders, &plan, world, me);

    // Preserve fabrication until a useful field unit is affordable.
    let tactical_bank = if rts(me) { me.get("rts").get("bank").num() } else { 0.0 };
    let miners = have.get("miner").or(0.0);
    let room = realm::mine_room(world, me);
    let shades = have.get("shade").or(0.0);
    let sentries = have.get("sentry").or(0.0);
    let needles = have.get("needle").or(0.0);
    let spires = have.get("spire").or(0.0);
    let shade_ready = has("dash") && has("cloak") && has("ranged");
    let specialist_bank = tactical_bank > 2450.0 * common::price_level(me);
    let total = me.get("units").get("total").num();
    // Once mines have porters, build one junk per second toward fleet needs.
    let rival_ships = me.path("rts.sea.contacts").iter().filter(|c| c.get("ship").truthy()).count() as f64;
    // Launch at most one junk per six seconds.
    let now_s = world.get("seconds").num_or(0.0).floor();
    let slip = now_s - logbook.get("js").num_or(-1e9) >= 6.0;
    let launched = slip
        && miners >= min(room, 4.0)
        && (miners >= room || shade_ready || have.get("runner").or(0.0) >= 3.0)
        && naval_spawn(&mut orders, world, me, &naval, rival_ships, threatened);
    let urgent_air_defense = specialist_bank && has("ranged") && has("dash") && has("antiAir") && enemy.air > 0.0 && needles < max(2.0, enemy.air);
    if urgent_air_defense {
        add_spawn(&mut orders, me, "needle", needle_body(), &["dash", "antiAir"]);
    } else if starter_kite(&mut orders, me, miners, room, enemy.air, specialist_bank) {
        // Keep a two-aircraft opening while the rest of the mining quota is built.
    } else if miners < room {
        add_spawn(&mut orders, me, "miner", miner_body(), &[]);
    } else if !shade_ready {
        add_spawn(&mut orders, me, "runner", runner_body(), &[]);
    }
    // Prioritize air defense, scan coverage, then cloaked pressure.
    else if specialist_bank && has("scan") && has("armor") && sentries < max(1.0, ((enemy.ranged + shades) / 5.0).ceil()) {
        add_spawn(&mut orders, me, "sentry", sentry_body(), &["armor", "scan"]);
    } else if specialist_bank && has("artillery") && has("scan") && enemy.total >= 4.0 && spires < max(1.0, (enemy.ranged / 3.0).ceil()) {
        add_spawn(&mut orders, me, "spire", spire_body(), &["artillery", "scan"]);
    } else if specialist_bank && has("flight") && have.get("carrier").or(0.0) >= 1.0 && have.get("kite").or(0.0) < min(6.0, max(2.0, (total / 10.0).floor())) {
        let frame=common::airframe(me,have.get("kite").or(0.0));
        add_spawn(&mut orders, me, "kite", frame.build(), if has("dash") { &["dash"][..] } else { &[][..] });
    } else if specialist_bank && has("pierce") && have.get("lancer").or(0.0) < max(2.0, (shades / 3.0).ceil()) {
        add_spawn(&mut orders, me, "lancer", shade_body(), &["dash", "pierce"]);
    } else if specialist_bank && shades < max(5.0, min(18.0, miners + (enemy.ranged / 2.0).ceil())) {
        add_spawn(&mut orders, me, "shade", shade_body(), &["dash", "cloak"]);
    } else if specialist_bank && has("dash") && has("scan") && have.get("scout").or(0.0) < 2.0 {
        add_spawn(&mut orders, me, "scout", scout_body(), &["dash", "scan"]);
    }

    let name = target.map_or_else(|| "frontier".to_string(), |t| t.get("name").str().to_string());
    // Update broadside and prize counts during the census.
    let broadsides = logbook.get("b").num_or(0.0) + enemy.volleys;
    let prizes = logbook.get("p").num_or(0.0) + intel.fresh;
    if launched || (census && (enemy.volleys > 0.0 || intel.fresh > 0.0 || logbook.get("tk").is_undef() || now_tick(world) - logbook.get("tk").num_or(0.0) > 600.0)) {
        let js = if launched { now_s } else { logbook.get("js").num_or(-1e9) };
        // Without the census, retain prior counts and tick.
        let tk = if census { now_tick(world) } else { logbook.get("tk").num_or(0.0) };
        let book = obj! { "b" => broadsides, "p" => prizes, "pt" => intel.newest, "tk" => tk, "js" => js };
        concat(&mut orders, "ram", mem::put(LOG, &book, "local", None, 1).unwrap_or_default());
    }
    let junks = have.get("warjunk").or(0.0) + have.get("picket").or(0.0) + have.get("corsair").or(0.0) + have.get("carrier").or(0.0);
    let doing = if let Some(l) = &intel.line {
        l.clone()
    } else if naval.landing >= 0.0 && naval.landing == target_id {
        format!("marines landing at {name}")
    } else if naval.blockade > 0 {
        format!("junks blockade {} straits; target {name}", naval.blockade)
    } else if naval.stations > 0 {
        format!("relays hold {} crossings; target {name}", naval.stations)
    } else if naval.landing >= 0.0 {
        format!("scouting a landing at {}; target {name}", naval.landing_name)
    } else {
        format!("target {name}")
    };
    let memo = format!("Quiet Fleet: {} junks, {} broadsides, {} prizes; {doing}", text(junks), text(broadsides), text(prizes));
    orders.set("memo", memo.chars().take(90).collect::<String>());
    let _ = naval.fleet;
    // Fund research early; later reserve combat cycles for fabrication and reflexes.
    let available_control = if available.truthy() { available.get("control").num() * 0.8 } else { f64::INFINITY };
    let types = common::calls(
        world,
        // Standing orders persist; reserve control cycles for routes as units wake.
        &[
            ("miner", 6000.0, 6.0),
            ("runner", 5000.0, 4.0),
            ("lancer", 4200.0, 4.0),
            ("shade", 7000.0, 3.0),
            ("sentry", 4500.0, 5.0),
            ("spire", 6000.0, 4.0),
            ("needle", 5500.0, 3.0),
            ("kite", 5000.0, 3.0),
            ("scout", 5000.0, 8.0),
            ("warjunk", 8000.0, 3.0),
            ("picket", 6000.0, 6.0),
            ("corsair", 8000.0, 3.0),
            ("carrier", 8000.0, 3.0),
        ],
        me.get("rts").get("cpu").get("income").or(0.0),
        available_control,
        me.get("units").get("byType"),
    );
    let techs = research_techs(world, me);
    common::budget(&mut orders, world, me, &common::Budget { research: 0.12, code: 0.10, techs, types: Some(&types) });
    if rts(me) {
        // common::budget reports research only when it changes.
        let studying = match orders.get("research") {
            Value::Str(s) if !s.is_empty() => s.clone(),
            _ => me.get("research").get("tech").str().to_string(),
        };
        let tech = if studying.is_empty() { View::undefined() } else { world.get("techs").get(&studying) };
        let offer = if tech.truthy() { tech.get("offer").num() } else { 0.0 };
        // Visible contact releases research reserves for combat.
        let mut policy = cpu_policy(me, offer, threatened || enemy.total >= 2.0);
        common::fit_cpu(&mut policy, me);
        orders.entry("compute").set("cpu", policy);
    }
    let pact_leader = realm::leader(world).map(|l| l.get("id").num());
    let now = world.get("seconds").or(0.0).floor();
    // Complete think so CPU limits and trade timers persist.
    let limits = me.get("rts").get("cpu").get("limits");
    let limit = if limits.truthy() { limits.get("think").or(0.0) } else { 0.0 };
    let ran_out = limit > 1.0 && me.get("compute").get("used").get("think").or(0.0) >= limit * 0.97;
    if cycles_left() < 6000.0 || (rts(me) && ran_out) {
        return affordable(orders, me, start);
    }
    let at = mem::get(me.get("ramCapital"), CLOCK);
    let mut clock = Clock { at: if at.truthy() { at } else { Value::obj() }, changed: false };
    // Court two non-leaders and occasionally fund a partner under pressure.
    let no_pacts = me.get("pacts").is_empty();
    if no_pacts {
        for c in common::partners(world, me).iter().take(2) {
            let id = c.get("id").num();
            if common::court(&mut orders, world, me, id, |o| diplomacy::propose(o, me, vec![id], (), "together against the leader"), &common::Court::default()) {
                break;
            }
        }
    } else if me.get("bank").num() > 300.0 && due(&mut clock, "gift", 30.0, now) {
        let mut at_war = -1.0;
        for p in world.get("provinces").iter() {
            let owner = p.get("owner").num();
            if owner < 0.0 || owner.is_nan() || owner == me_id || !realm::is_ally(me, owner) || !p.get("units").truthy() {
                continue;
            }
            for (f, _) in p.get("units").entries() {
                let f: f64 = f.parse().unwrap_or(f64::NAN);
                if f != owner && f != me_id && !realm::is_ally(me, f) {
                    at_war = owner;
                    break;
                }
            }
            if at_war >= 0.0 {
                break;
            }
        }
        if at_war >= 0.0 {
            common::court(
                &mut orders,
                world,
                me,
                at_war,
                |o| diplomacy::send(o, at_war, "for the war", obj! { "give" => obj! { "coin" => 80 } }),
                &common::Court::default(),
            );
        }
    }
    // Inspect the border and visible units when a bribe is due.
    let seen = 10.0 * world.get("units").len() as f64;
    if cycles_left() > 6000.0 + seen {
        intrigue(&mut orders, world, me, &mut clock, now);
    }
    if cycles_left() > 5000.0 + seen {
        mercenaries(&mut orders, world, me, &mut clock, now, threatened || enemy.total >= 4.0);
    }
    if cycles_left() > 3000.0 {
        faith(&mut orders, world, me, now);
    }
    if cycles_left() > 2000.0 {
        let pact = move |m: View| no_pacts && pact_leader.is_none_or(|l| m.get("from").num() != l);
        common::answer(&mut orders, me, world, pact, techs);
    }
    if clock.changed {
        concat(&mut orders, "ram", mem::put(CLOCK, &clock.at, "local", None, 1).unwrap_or_default());
    }
    affordable(orders, me, start)
}

// Bribe and sway likely border provinces; lease scan and cloak.
fn intrigue(orders: &mut Value, world: View, me: View, clock: &mut Clock, now: f64) {
    let income = me.get("rts").get("cpu").get("income").or(me.get("compute").get("cycles").or(14000.0));
    let techs = me.get("techs");
    // Refresh propaganda targets every nine seconds.
    if techs.get("influence").truthy() && common::every(now, 8.0, me.get("id").num()) {
        let each = round(income * if me.get("sway").get("faith").truthy() { 0.04 } else { 0.05 });
        orders.set("propaganda", Value::arr());
        for t in common::sway_targets(world, me).iter().take(4) {
            sway::propaganda(orders, t.get("id").num(), each);
        }
    }
    let bank = me.get("bank").num();
    if bank > 500.0 && due(clock, "bribe", 20.0, now) {
        if let Some(t) = common::sway_targets(world, me).first() {
            sway::bribe(orders, t.get("id").num(), min(250.0, bank - 350.0));
        }
    }
    // Lease scan or cloak to a solvent faction that lacks it.
    if (techs.get("cloak").truthy() || techs.get("scan").truthy()) && due(clock, "lease", 150.0, now) {
        let leased: Vec<View> = me.get("contracts").iter().filter(|c| !c.get("ended").truthy()).map(|c| c.get("with")).collect();
        let lead = realm::leader(world).map(|l| l.get("id").num());
        let me_id = me.get("id").num();
        let mut best: Option<(f64, &str)> = None;
        let mut score = 0.0;
        for f in world.get("factions").iter() {
            let id = f.get("id").num();
            let share = f.get("share").num();
            if !f.get("alive").truthy() || id == me_id || lead == Some(id) || leased.iter().any(|w| w.is_num() && w.num() == id) || share < 0.03 {
                continue;
            }
            let lacks: Vec<&str> = ["cloak", "scan"].into_iter().filter(|t| techs.get(t).truthy() && !f.get("techs").has_str(t)).collect();
            let s = lacks.len() as f64 * share * (0.8 + 0.4 * rand());
            if s > score {
                score = s;
                best = Some((id, lacks[rand_int(lacks.len() as f64) as usize]));
            }
        }
        if let Some((id, tech)) = best {
            trade::sell_tech(orders, id, tech, obj! { "coinPerSecond" => 0.2, "seconds" => 600 }, 600.0, format!("our {tech} for ten minutes"));
        }
    }
}

// Hire four soldiers from a non-leader with under a fifth of the world.
fn mercenaries(orders: &mut Value, world: View, me: View, clock: &mut Clock, now: f64, fight: bool) {
    if !fight
        || me.get("bank").num() < 700.0
        || me.get("contracts").iter().any(|c| !c.get("ended").truthy() && c.get("get").get("hire").truthy())
        || !due(clock, "hire", 180.0, now)
    {
        return;
    }
    let lead = realm::leader(world).map(|l| l.get("id").num());
    let me_id = me.get("id").num();
    let mut n: Vec<(f64, f64)> = Vec::new();
    for u in world.get("units").iter() {
        let owner = u.get("owner").num();
        if owner == me_id {
            continue;
        }
        match n.iter_mut().find(|e| e.0 == owner) {
            Some(e) => e.1 += 1.0,
            None => n.push((owner, 1.0)),
        }
    }
    let count = |id: f64| n.iter().find(|e| e.0 == id).map_or(0.0, |e| e.1);
    let (mut best, mut score) = (None, 0.0);
    for f in world.get("factions").iter() {
        let id = f.get("id").num();
        if !f.get("alive").truthy() || id == me_id || lead == Some(id) || f.get("share").num() >= 0.2 || count(id) < 8.0 {
            continue;
        }
        let s = count(id) * if me.get("allies").has_num(id) { 2.0 } else { 1.0 } * (0.8 + 0.4 * rand());
        if s > score {
            score = s;
            best = Some(id);
        }
    }
    if let Some(id) = best {
        trade::hire_from(orders, id, 4.0, 300.0, obj! { "coin" => 240 }, "four soldiers for five minutes, paid up front");
    }
}

// Found faith once unlocked and preach at the weakest border provinces.
fn faith(orders: &mut Value, world: View, me: View, now: f64) {
    if !me.get("techs").get("faith").truthy() {
        return;
    }
    let me_id = me.get("id").num();
    if !me.get("sway").get("faith").truthy() {
        let taken: Vec<&str> = world.get("faiths").iter().map(|f| f.get("name").str()).collect();
        for i in 0..FAITHS.len() {
            let name = FAITHS[((me_id + i as f64) % FAITHS.len() as f64) as usize];
            if !taken.contains(&name) {
                orders.set("faith", obj! { "found" => name });
                break;
            }
        }
        return;
    }
    if !common::every(now, 8.0, me_id) {
        return;
    }
    let income = me.get("rts").get("cpu").get("income").or(me.get("compute").get("cycles").or(14000.0));
    let by_id = realm::index(world);
    let mut cands: Vec<(f64, f64)> = Vec::new();
    for p in world.get("provinces").iter() {
        if !p.get("visible").truthy() || p.get("owner").is_undef() {
            continue;
        }
        let mine = p.get("owner").num() == me_id;
        // the faith works on rivals' and free land, never a partner's
        if !mine && realm::is_ally(me, p.get("owner").num()) {
            continue;
        }
        let border = p.get("adj").iter().any(|a| {
            let q = by_id.at(a);
            let owner = q.get("owner").num();
            q.truthy() && if mine { owner != me_id && owner >= 0.0 } else { owner == me_id }
        });
        if !border {
            continue;
        }
        let pf = p.get("faith");
        let zeal = if pf.truthy() && pf.get("by").num() == me_id { pf.get("zeal").num() } else { -(if pf.truthy() { pf.get("zeal").num() } else { 0.0 }) };
        if zeal >= 80.0 {
            continue;
        }
        cands.push((p.get("id").num(), if mine { 0.0 } else { 15.0 } - zeal + p.get("cores").or(0.0) * 4.0));
    }
    cands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    orders.set("preach", Value::arr());
    for c in cands.iter().take(if income >= 60000.0 { 2 } else { 1 }) {
        sway::preach(orders, c.0, round(income * 0.025));
    }
}

// ---- Unit types ----

fn miner(units: View, world: View, me: View) -> Value {
    if common::recover_miners(units, world, me) { return Value::Undef; }
    if short(me, 2500.0, 800.0) {
        return Value::arr();
    }
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        let u = nth(units, turn, world);
        if u.get("doing").str() == "gather" {
            continue;
        }
        if let Some(g) = orders::gather(u, world, me) {
            order(g);
        }
    }
    Value::Undef
}

fn runner(units: View, world: View, me: View) {
    infantry(units, world, me, false);
}

fn needle(units: View, world: View, me: View) {
    infantry(units, world, me, false);
}

// Keep standing orders below the plan lookup runway.
fn infantry(units: View, world: View, me: View, cloak: bool) -> Value {
    if short(me, 2500.0, 1200.0) {
        return Value::arr();
    }
    let pl = plan::for_units(me, units, View::undefined());
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        march(nth(units, turn, world), &pl, world, me, cloak);
    }
    Value::Undef
}

/// One foot soldier's order from the plan: defend where the plan says (common::defends), else take its target.
fn march(u: View, pl: &Value, world: View, me: View, cloak: bool) {
    {
        // those the plan sends to defend (common::defends), the rest on the target
        let defend = Value::Num(common::defends(pl, u));
        let goal = if num(&defend) >= 0.0 { &defend } else { pl.get("target") };
        let owner=realm::index(world).get(num(goal)).get("owner").num_or(-1.0);
        let conduct=u.get("conduct");
        let safe=realm::is_ally(me,owner) || (conduct.get("seconds").num_or(0.0)>0.0 && conduct.get("from").num_or(-2.0)==owner);
        let verb = if num(&defend) >= 0.0 || safe || u.get("air").truthy() || u.path("build.wings").truthy() { "fight" } else { "capture" };
        // Retreat defeated armies to the stage and wait for reinforcements.
        if verb == "capture" {
            if let Some(o) = common::front(u, world, me, pl) {
                if let Some(o) = o {
                    order(o);
                }
                return;
            }
        }
        // Recalculate routes only after a destination change.
        let doing = u.get("doing").str();
        if (doing == verb || doing == "move") && same(u.get("target"), goal) {
            return;
        }
        let o = if num(goal) >= 0.0 { orders::to(u, verb, num(goal), world, me, ()) } else { None };
        if let Some(mut o) = o {
            if cloak {
                o.set("cloak", true);
            }
            o.set("power", obj! { "drive" => 420, "armor" => 0, "weapon" => 260 });
            order(o);
        } else if cloak {
            order(obj! { "id" => u.get("id"), "cloak" => true, "power" => obj! { "drive" => 0, "armor" => 0, "weapon" => 240 }, "alert" => 6 });
        }
    }
}

fn shade(units: View, world: View, me: View) {
    infantry(units, world, me, true);
}

// Strike visible provinces with the most miners, else the plan target.
fn prey(world: View, me: View, n: &[(f64, f64)]) -> f64 {
    let by_id = realm::index(world);
    let me_id = me.get("id").num();
    // provinces in order of id, as a JavaScript object keeps them
    let mut n = n.to_vec();
    n.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut best = -1.0;
    let mut most = 0.0;
    for (k, count) in n {
        let q = by_id.get(k);
        let owner = q.get("owner").num();
        if q.truthy() && owner != me_id && !realm::is_ally(me, owner) && count > most {
            most = count;
            best = k;
        }
    }
    best
}

// Kites defend, raid prey for ◆, or join the target attack.
fn kite(units: View, world: View, me: View) {
    // A starved callback cannot afford recovery's deck search.  Landing one hot
    // wing is still useful, but leave the rest on their standing orders.
    if cycles_left() < 3500.0 {
        land_hot(units);
        return;
    }
    if common::air_recovery(units,me) { return; }
    if !me.path("rts.air.decks").is_empty() {
        common::fly(units, world, me);
        return;
    }
    // Land a flier after a minute aloft or before its fuel runs short.
    land_hot(units);
    if short(me, 2500.0, 1200.0) {
        return;
    }
    let pl = plan::for_units(me, units, View::undefined());
    let by_id = realm::index(world);
    let goal = if num(pl.get("defend")) >= 0.0 {
        pl.get("defend")
    } else if num(pl.get("prey")) >= 0.0 {
        pl.get("prey")
    } else {
        pl.get("target")
    };
    if goal.is_undef() || num(goal) < 0.0 {
        return;
    }
    let goal = num(goal);
    let q = by_id.get(goal);
    let owner = q.get("owner").num();
    let raid = num(pl.get("defend")) < 0.0 && q.truthy() && owner >= 0.0 && owner != me.get("id").num() && realm::is_rival(world, me, owner);
    let verb = if raid { "raid" } else { "fight" };
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        let u = nth(units, turn, world);
        // Finish landing and cooling before assigning a new sortie.
        if cooling(u) {
            continue;
        }
        if u.get("doing").str() == verb && same(u.get("target"), &Value::Num(goal)) {
            continue;
        }
        if let Some(mut o) = orders::to(u, verb, goal, world, me, ()) {
            o.set("alert", 6);
            o.set("power", obj! { "drive" => 600, "armor" => 0, "weapon" => 420 });
            order(o);
        }
    }
}

/// Orders every flier a minute up (`u.rts.air.heat`) down to your nearest land, unless it's already moving.
fn land_hot(units: View) {
    for u in units.iter() {
        // Stop low-fuel calls before they run out.
        if cycles_left() < 500.0 {
            break;
        }
        let a = u.path("rts.air");
        let doing = u.get("doing");
        if a.truthy() && a.get("aloft").truthy() && heat(a) >= 60.0 && !(doing.is_str() && doing.str() == "move") {
            order(obj! { "id" => u.get("id"), "land" => true });
        }
    }
}

/// A flier coming down (a minute up) or down and not yet cooled off.
fn cooling(u: View) -> bool {
    let a = u.path("rts.air");
    a.truthy() && (heat(a) >= 60.0 || (heat(a) > 0.0 && !a.get("aloft").truthy()))
}

/// `air.heat` as a number (null is 0).
fn heat(a: View) -> f64 {
    let h = a.get("heat");
    if h.is_null() {
        0.0
    } else {
        h.num()
    }
}

fn spire(units: View, world: View, me: View) {
    sentry(units, world, me);
}

fn sentry(units: View, world: View, me: View) -> Value {
    if short(me, 2500.0, 1200.0) {
        return Value::arr();
    }
    let pl = plan::for_units(me, units, View::undefined());
    let at = if num(pl.get("defend")) >= 0.0 {
        pl.get("defend")
    } else if num(pl.get("warn")) >= 0.0 {
        pl.get("warn")
    } else if num(pl.get("stage")) >= 0.0 {
        pl.get("stage")
    } else {
        pl.get("home")
    };
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        let u = nth(units, turn, world);
        let doing = u.get("doing").str();
        if (doing == "hold" || doing == "move") && same(u.get("target"), at) {
            continue;
        }
        // with no place to hold, the province it stands in
        let p = if at.is_undef() { None } else { Some(num(at)) };
        if let Some(mut o) = orders::hold(u, Point::new(0.0, 0.0), world, me, p) {
            o.set("power", obj! { "drive" => 0, "armor" => 520, "weapon" => 180 });
            o.set("alert", 7);
            order(o);
        }
    }
    Value::Undef
}

fn scout(units: View, world: View, me: View) -> Value {
    if short(me, 2500.0, 1200.0) {
        return Value::arr();
    }
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        let u = nth(units, turn, world);
        if u.get("doing").str() == "move" {
            continue;
        }
        let to = common::wander(world, me, u.get("at").num());
        if to >= 0.0 {
            order(obj! { "id" => u.get("id"), "do" => "move", "target" => to, "power" => obj! { "drive" => 650, "armor" => 0, "weapon" => 0 }, "alert" => 8 });
        }
    }
    Value::Undef
}

fn lancer(units: View, world: View, me: View) -> Value {
    infantry(units, world, me, false)
}

// ---- The fleet ----

// Capital RAM page 226 stores broadsides, prizes and the last census tick.
const LOG: u32 = 226;

/// The water on province `p`'s shore that lies off its own town if any, else the first; -1 for an inland province.
fn shore_water(world: View, p: f64) -> f64 {
    if p.is_nan() || p < 0.0 {
        return -1.0;
    }
    let mut first = -1.0;
    for w in world.get("waters").iter() {
        if !w.get("coast").has_num(p) {
            continue;
        }
        let id = w.get("id").num();
        if w.get("p").num() == p {
            return id;
        }
        if first < 0.0 {
            first = id;
        }
    }
    first
}

/// What the fleet does this second: think works it out once and puts it in the plan as `nv`, small enough for every
/// province to hold, and each junk copies what it needs into its own RAM before it sails off our coast.
/// `st`: [a, b, water] for each crossing to land overseas a picket must hold; `lw`/`lt`: the water off the next landing
/// and the province the marines land on (-1: none); `fw`: where the battle fleet keeps; `hw`: home water; `rw`: waters
/// the corsairs hunt in.
struct Naval {
    stations: usize,
    landing: f64,
    landing_name: String,
    fleet: f64,
    blockade: usize,
    /// Waters the corsairs hunt in.
    raid: usize,
}

fn naval_plan(world: View, me: View, plan: &mut Value, target_id: f64, defend: f64, rowing: &[(f64, f64)]) -> Naval {
    let me_id = me.get("id").num();
    let last = mem::get(me.get("ramCapital"), 0).get("nv").clone();
    let by_id = realm::index(world);
    // Retain RAM crossings and their station waters from the prior plan.
    let mut st: Vec<Value> = Vec::new();
    for l in me.path("rts.sea.links").iter() {
        if st.len() >= 6 {
            break;
        }
        let (a, b) = (l.at(0).num(), l.at(1).num());
        let cached = last.get("st").items().iter().find(|x| x.at(0).num() == a && x.at(1).num() == b).map(|x| x.at(2).num());
        let w = match cached {
            Some(w) => w,
            None if cycles_left() > 24000.0 => sea::station(world, a, b),
            None => -1.0,
        };
        if w >= 0.0 {
            st.push(arr![a, b, w]);
        }
    }
    // home water: off the capital, else off the first port
    let mut hw = last.get("hw").num_or(-1.0);
    if hw < 0.0 || last.get("hc").num_or(-1.0) != me.get("capital").num() {
        hw = shore_water(world, me.get("capital").num());
        if hw < 0.0 {
            hw = shore_water(world, me.path("rts.sea.ports").at(0).num_or(-1.0));
        }
    }
    // Pick the best sea-only landing and its picket station.
    let mut lt = last.get("lt").num_or(-1.0);
    let mut lw = last.get("lw").num_or(-1.0);
    let frontier = common::frontier(world, me);
    let still = frontier.iter().any(|q| q.get("id").num() == lt);
    if !still || lw < 0.0 {
        lt = -1.0;
        lw = -1.0;
        let mut best = -1e9;
        for q in &frontier {
            if q.get("adj").iter().any(|a| by_id.at(a).get("owner").num() == me_id) {
                continue;
            }
            let owner = q.get("owner").num();
            if owner >= 0.0 && realm::is_ally(me, owner) {
                continue;
            }
            let s = q.get("cores").or(0.0) * 3.0 + if owner < 0.0 { 3.0 } else { 1.0 } - q.get("garrison").or(0.0) * 2.0
                - if q.get("capital").truthy() { 8.0 } else { 0.0 };
            if s > best {
                best = s;
                lt = q.get("id").num();
            }
        }
        if lt >= 0.0 && cycles_left() > 24000.0 {
            let from = frontier.iter().find(|q| q.get("id").num() == lt).and_then(|q| realm::from(*q)).unwrap_or(-1.0);
            lw = if from >= 0.0 { sea::station(world, from, lt) } else { -1.0 };
            if lw < 0.0 {
                lt = -1.0;
            }
        } else {
            lt = -1.0;
        }
    }
    // Keep the battle fleet near defense, landing, target or home.
    let mut fw = -1.0;
    if defend >= 0.0 {
        fw = shore_water(world, defend);
    }
    if fw < 0.0 && lt >= 0.0 && lt == target_id {
        fw = lw;
    }
    if fw < 0.0 && target_id >= 0.0 {
        fw = shore_water(world, target_id);
    }
    if fw < 0.0 {
        fw = hw;
    }
    // Keep four observed straits for five minutes; prioritize routes touching our land.
    let now = world.get("seconds").num_or(0.0).floor();
    let mut bx: Vec<Value> = Vec::new();
    for x in last.get("bx").items() {
        let (a, b) = (x.at(0).num(), x.at(1).num());
        let seen = if rowing.contains(&(a, b)) { now } else { x.at(3).num_or(0.0) };
        if now - seen < 300.0 && bx.len() < 4 {
            bx.push(arr![a, b, x.at(2).num(), seen]);
        }
    }
    let ours = |p: f64| by_id.get(p).get("owner").num() == me_id;
    let mut fresh: Vec<(f64, f64)> = rowing.iter().copied().filter(|&(a, b)| !bx.iter().any(|y| y.at(0).num() == a && y.at(1).num() == b)).collect();
    fresh.sort_by_key(|&(a, b)| u8::from(!(ours(a) || ours(b))));
    if let Some(&(a, b)) = fresh.first() {
        if cycles_left() > 24000.0 {
            let w = sea::station(world, a, b);
            if w >= 0.0 {
                if bx.len() >= 4 && (ours(a) || ours(b)) {
                    // Replace the stalest strait when an invasion route needs space.
                    let old = (0..bx.len()).min_by(|&i, &j| bx[i].at(3).num().total_cmp(&bx[j].at(3).num())).unwrap_or(0);
                    bx.remove(old);
                }
                if bx.len() < 4 {
                    bx.push(arr![a, b, w, now]);
                }
            }
        }
    }
    // Hunt known shipping, straits and rival coasts.
    let mut rw: Vec<f64> = Vec::new();
    for c in me.path("rts.sea.contacts").iter() {
        let w = c.get("water").num_or(-1.0);
        if c.get("ship").truthy() && w >= 0.0 && !rw.contains(&w) && rw.len() < 3 {
            rw.push(w);
        }
    }
    // Patrol rival dock waters, nearest first.
    let mut lanes: Vec<(usize, f64)> = Vec::new();
    for d in world.path("logistics.docks").iter() {
        if cycles_left() < 24000.0 || hw < 0.0 { break; }
        let w = d.get("water").num_or(-1.0);
        let owner = by_id.get(d.get("p").num()).get("owner").num_or(-1.0);
        if w < 0.0 || rw.contains(&w) || !realm::is_rival(world, me, owner) { continue; }
        if let Some(r) = sea::route(world, hw, w) { lanes.push((r.len(), w)); }
    }
    lanes.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    for (_, w) in lanes.into_iter().take(2) { if rw.len() < 4 { rw.push(w); } }
    for x in &bx {
        let w = x.at(2).num();
        if !rw.contains(&w) && rw.len() < 4 {
            rw.push(w);
        }
    }
    for q in &frontier {
        if rw.len() >= 4 {
            break;
        }
        let owner = q.get("owner").num();
        if owner >= 0.0 && realm::is_rival(world, me, owner) {
            let w = shore_water(world, q.get("id").num());
            if w >= 0.0 && !rw.contains(&w) {
                rw.push(w);
            }
        }
    }
    let name = if lt >= 0.0 { by_id.get(lt).get("name").str().to_string() } else { String::new() };
    plan.set(
        "nv",
        obj! {
            "st" => st.clone(), "lw" => lw, "lt" => lt, "fw" => fw, "hw" => hw, "hc" => me.get("capital"), "bx" => bx.clone(),
            "rw" => rw.iter().map(|&w| Value::Num(w)).collect::<Vec<Value>>(),
        },
    );
    Naval { stations: st.len(), landing: lt, landing_name: name, fleet: fw, blockade: bx.len(), raid: rw.len() }
}

/// What the newest prize says: a rival's shared plan, read from the code pages of a ship we sank (me.rts.sea.prizes).
/// `warn`: our province it means to attack (-1: none); `bonus`: its own land it left (its home and staging ground),
/// worth more to our army; `line`: what the fleet says about it.
struct Intel {
    warn: f64,
    bonus: Vec<f64>,
    line: Option<String>,
    newest: f64,
    fresh: f64,
}

fn prize_intel(world: View, me: View, log: &Value) -> Intel {
    let mut out = Intel { warn: -1.0, bonus: Vec::new(), line: None, newest: log.get("pt").num_or(-1.0), fresh: 0.0 };
    let prizes = me.path("rts.sea.prizes");
    let seen = log.get("pt").num_or(-1.0);
    let me_id = me.get("id").num();
    let by_id = realm::index(world);
    let now = world.get("seconds").num_or(0.0);
    for pz in prizes.iter() {
        let t = pz.get("t").num_or(0.0);
        if t > seen {
            out.fresh += 1.0;
            if t > out.newest {
                out.newest = t;
            }
        }
        // Use only fresh prize intel.
        if now - t > 240.0 {
            continue;
        }
        let pl = mem::get(pz.get("pages"), 0);
        if !pl.truthy() {
            continue;
        }
        let attack = if pl.get("attack").num_or(-1.0) >= 0.0 { pl.get("attack").num() } else { pl.get("target").num_or(-1.0) };
        let from = pz.get("from").num();
        let who = world.get("factions").key(from).get("name").str().to_string();
        let q = by_id.get(attack);
        if attack >= 0.0 && q.get("owner").num() == me_id {
            out.warn = attack;
            out.line = Some(format!("Prize papers: {who} means to strike our {}. Lanterns lit there.", q.get("name").str()));
        } else {
            for k in ["home", "stage"] {
                let p = pl.get(k).num_or(-1.0);
                if p >= 0.0 && by_id.get(p).get("owner").num() == from && !out.bonus.contains(&p) {
                    out.bonus.push(p);
                }
            }
            if out.line.is_none() && attack >= 0.0 {
                out.line = Some(format!("Prize papers: {who} marches on {}. Its own coast is bare.", q.get("name").str()));
            }
        }
    }
    out
}

fn armor_modules(me: View, module: &'static str) -> Vec<&'static str> {
    if me.path("rts.licenses").iter().any(|l| l.str() == module) { vec!["hull", module] } else { vec!["hull"] }
}

/// A local sea wreck pays for a new one-hold corsair's fabrication and round trip. This only uses wrecks already
/// observed by the faction and keeps the same travel and expiry estimates as `common::salvage_target`.
fn paying_recovery_wreck(world: View, me: View, hold: f64) -> bool {
    let home=shore_water(world,me.get("capital").num());
    if home<0.0 || hold<=0.0 { return false; }
    let adj=sea::index(world).get(home).get("adj");
    let now=world.get("seconds").num_or(0.0); let level=common::price_level(me);
    me.path("rts.logistics.wrecks").iter().any(|w| {
        if !w.get("sea").truthy(){return false;}
        let water=w.get("water").num_or(-1.0);
        let hops=if water==home {0.0}else if adj.has_num(water){1.0}else{return false};
        let leg=35.0+20.0*hops;
        let cost=common::fabrication(me,2.0)+300.0*level+2.0*leg*700.0*level*1.5;
        w.get("cycles").num_or(0.0).min(hold)>cost && w.get("until").num_or(0.0)>(now+2.0*leg)*5.0
    })
}

/// One junk a second at most, toward the fleet the plan needs: a picket for every crossing and the next landing, then
/// warjunks, then up to two corsairs when visible cargo can pay for a raid. A valuable nearby sea wreck can reserve
/// one cargo corsair before the ordinary fleet priorities. True when one was ordered.
fn naval_spawn(orders: &mut Value, world: View, me: View, naval: &Naval, rival_ships: f64, threatened: bool) -> bool {
    let techs = me.get("techs");
    let has = |t: &str| techs.get(t).truthy();
    if !has("seaTraversal") || me.path("rts.sea.ports").is_empty() {
        return false;
    }
    let have = me.get("units").get("byType");
    let (pickets, war, corsairs, carriers) = (have.get("picket").or(0.0), have.get("warjunk").or(0.0), have.get("corsair").or(0.0), have.get("carrier").or(0.0));
    let guns=has("ranged");
    let corsair_capacity=me.path("rts.logistics.rules.shipHoldPerCarry").num_or(160000.0*common::price_level(me));
    // A live or queued corsair already owns the recovery assignment. Do not recruit a second one.
    if guns && corsairs<1.0 && paying_recovery_wreck(world,me,corsair_capacity) {
        return add_spawn(orders,me,"corsair",corsair_body(true),&armor_modules(me,"dash"));
    }
    let income = me.get("rts").get("cpu").get("income").or(0.0);
    // Income and a crossing, escort or target bound hull recruitment.
    let fleet_max = max(3.0, (income / 20000.0).floor()).min(12.0);
    // (two junks put to sea whatever the army; past them, a soldier ashore for every two junks, since the land is what
    // feeds the yard)
    let junks = pickets + war + corsairs + carriers;
    let ashore = common::fighters(me) - junks;
    if junks >= fleet_max || (threatened && junks >= 2.0) || (junks >= 2.0 && ashore < 1.0 + (junks / 2.0).floor()) {
        return false;
    }
    let want_pick = (naval.stations as f64 + if naval.landing >= 0.0 { 1.0 } else { 0.0 }).min(5.0);
    // Replace raiders only when cargo prey is visible.
    let home=shore_water(world,me.get("capital").num());
    let adj=sea::index(world).get(home).get("adj");
    let nearby=|water:f64|water>=0.0&&(water==home||adj.has_num(water));
    let contacts=me.path("rts.sea.contacts");
    // A visible, unarmed loaded merchant justifies preparing a corsair. Its
    // controller keeps the stricter escort and round-trip profitability gate.
    let prospects=contacts.iter().filter(|c| c.get("ship").truthy()
        && nearby(c.get("water").num_or(-1.0)) && c.path("cargo.load").num_or(0.0)>=2.0
        && c.path("build.range").num_or(0.0)==0.0 && c.path("build.carry").num_or(0.0)>=2.0).count();
    let salvage=me.path("rts.logistics.wrecks").iter().any(|w|w.get("sea").truthy()&&nearby(w.get("water").num_or(-1.0))&&w.get("cycles").num_or(0.0)>160000.0*common::price_level(me));
    // Raid nearby cargo, salvage wrecks, or hunt rival shipping.
    let want_cors = if !guns { 0.0 } else if prospects>0 || salvage { (prospects as f64).clamp(1.0,2.0) } else if naval.raid > 0 { 1.0 } else { 0.0 };
    // Build warships for observed traffic or contested crossings.
    let escorts: f64 = if have.get("freighter").num_or(0.0) > 0.0 { 1.0 } else { 0.0 };
    let contested = rival_ships.min(3.0) + naval.blockade.min(1) as f64;
    let want_war = if guns { escorts.max(contested).max(if prospects > 0 { 1.0 } else { 0.0 }).min(4.0) } else { 0.0 };
    let want_carrier = has("flight") && have.get("kite").or(0.0) >= 2.0 && carriers < 1.0;
    let pick: Option<(&str, Value, Vec<&str>)> = if pickets < want_pick {
        Some(("picket", picket_body(guns), armor_modules(me,"armor")))
    } else if want_carrier {
        Some(("carrier", carrier_body(), armor_modules(me,"armor")))
    } else if war < want_war.min(1.0) {
        Some(("warjunk", warjunk_body(), armor_modules(me,"armor")))
    } else if corsairs < want_cors {
        Some(("corsair", corsair_body(me.path("rts.logistics").truthy()), armor_modules(me,"dash")))
    } else if war < want_war {
        Some(("warjunk", warjunk_body(), armor_modules(me,"armor")))
    } else {
        None
    };
    let Some((ty, body, mods)) = pick else { return false };
    let m: Value = mods.iter().map(|&x| Value::from(x)).collect::<Vec<Value>>().into();
    if army::check(&body, &m, me).is_some() {
        return false;
    }
    add_spawn(orders, me, ty, body, &mods)
}

// ---- The junks ----

/// The water junk `u` should keep to, from the fleet's orders (`nv`): role 0 warjunk, 1 picket, 2 corsair; `k` is its
/// place among the junks of its kind (by id), so each picket takes its own crossing.
/// The strait (a, b) a warjunk on the blockade keeps: the same pick as [`junk_goal`]; None for the rest.
fn junk_strait(nv: &Value, role: u8, k: usize) -> Option<(f64, f64)> {
    let bx = nv.get("bx").items();
    (role == 0 && k % 2 == 1 && !bx.is_empty()).then(|| {
        let x = &bx[(k / 2) % bx.len()];
        (x.at(0).num(), x.at(1).num())
    })
}

fn junk_goal(nv: &Value, role: u8, k: usize, now: f64) -> f64 {
    let fw = nv.get("fw").num_or(-1.0);
    match role {
        1 => {
            let st = nv.get("st").items();
            if k < st.len() {
                st[k].at(2).num_or(fw)
            } else if k == st.len() && nv.get("lw").num_or(-1.0) >= 0.0 {
                nv.get("lw").num()
            } else {
                fw
            }
        }
        2 => {
            let rw = nv.get("rw").items();
            if rw.is_empty() {
                fw
            } else {
                rw[(k + (now / 120.0).floor() as usize) % rw.len()].num_or(fw)
            }
        }
        _ => {
            // Divide warjunks between blockades, corsairs and the battle fleet.
            let bx = nv.get("bx").items();
            let rw = nv.get("rw").items();
            if k % 2 == 1 && !bx.is_empty() {
                bx[(k / 2) % bx.len()].at(2).num_or(fw)
            } else if k % 3 == 2 && !rw.is_empty() {
                rw[(k / 3 + (now / 120.0).floor() as usize) % rw.len()].num_or(fw)
            } else {
                fw
            }
        }
    }
}

/// True when a visible merchant can pay for a healthy corsair's round trip.  Spawn and chase
/// share this exact gate so Fleet never launches a hull that its controller immediately rejects.
fn paying_corsair_prey(c: View, contacts: View, me: View, capacity: f64, healthy: bool, hops: f64) -> bool {
    if !healthy || !c.get("ship").truthy() || c.get("build").get("range").num_or(0.0)>0.0 { return false; }
    let load=c.path("cargo.load").num_or(0.0);
    if load<2.0 || c.get("build").get("carry").num_or(0.0)<2.0 { return false; }
    let water=c.get("water").num_or(-1.0);
    if contacts.iter().any(|guard| guard.get("id").num()!=c.get("id").num() && guard.get("ship").truthy() && guard.get("water").num_or(-1.0)==water && guard.path("build.range").num_or(0.0)>0.0) { return false; }
    let prize=c.path("build.carry").num_or(0.0)*160000.0*common::price_level(me)*if load>=3.0 {1.0}else{0.5};
    (prize*0.7).min(capacity) >= 1.5*((55.0+hops*30.0)* (600.0*common::price_level(me)+1500.0))
}

/// A rival ship for junk `u` to go after (me.rts.sea.contacts): on its water or one next to it, the weakest first; a
/// corsair looks two waters out and prefers ships without guns (relays, pickets, carriers) and lone ones.
fn quarry(u: View, me: View, world: View, role: u8) -> Option<View> {
    let here = u.path("rts.sea.water").num_or(-1.0);
    if here < 0.0 {
        return None;
    }
    let waters = sea::index(world);
    let near = waters.get(here).get("adj");
    let mut best: Option<View> = None;
    let mut score = 1e9;
    for c in me.path("rts.sea.contacts").iter() {
        // Contact searches share fuel with routes and RAM writes.
        if cycles_left() < 3000.0 { break; }
        if !c.get("ship").truthy() {
            continue;
        }
        let w = c.get("water").num_or(-1.0);
        let hops = if w == here {
            0.0
        } else if near.has_num(w) {
            1.0
        } else if role == 2 && waters.get(w).get("adj").iter().any(|x| near.has_num(x.num())) {
            2.0
        } else {
            continue;
        };
        let armed = c.get("build").get("range").num_or(0.0) > 0.0;
        let mut value=0.0;
        if (role==2 || !armed) && world.get("logistics").truthy() {
            let healthy=u.get("hp").num()>=u.get("hpMax").num()*0.7;
            if armed || !paying_corsair_prey(c, me.path("rts.sea.contacts"), me, logistics::capacity(u), healthy, hops) {continue;}
            let prize=c.path("build.carry").num_or(0.0)*160000.0*common::price_level(me);
            value=prize.min(logistics::capacity(u))/1e6;
        }
        let s = c.get("hp").num_or(5.0) + hops * 1.5 + if role == 2 && armed { 3.0 } else { 0.0 } - value;
        if s < score {
            score = s;
            best = Some(c);
        }
    }
    best
}

/// Every junk: keep to its water, or go after a rival ship nearby; the reflex fires the guns. A junk standing off our
/// own coast hears the plan there, and copies its water into its own RAM (page 1) for when it sails off a rival's.
fn junks(units: View, world: View, me: View, role: u8) -> Value {
    // Route, contact, and ship-RAM scans need more than a thin control lane.
    // Below it, retain the standing course for the next funded callback.
    if short(me, 3200.0, 1200.0) {
        return Value::arr();
    }
    let pl = plan::for_units(me, units, View::undefined());
    let heard = pl.get("nv");
    let now = world.get("seconds").or(0.0).floor();
    // each junk's place among its kind, by id
    let mut ids: Vec<f64> = units.iter().map(|u| u.get("id").num()).collect();
    ids.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let contacts = me.path("rts.sea.contacts").len();
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 3500.0 {
            break;
        }
        let u = nth(units, turn, world);
        let sea = u.path("rts.sea");
        if !sea.get("ship").truthy() {
            // Hired units with a junk body fight on land.
            if sea.is_undef() {
                march(u, &pl, world, me, false);
            }
            continue;
        }
        if role != 1 && common::recover_ship(u,world,me) { continue; }
        let id = u.get("id").num();
        let k = ids.iter().position(|&x| x == id).unwrap_or(0);
        let carried = mem::get(u.get("ramUnit"), 1);
        let mut goal = carried.get("g").num_or(-1.0);
        let mut o = obj! { "id" => id };
        let mut send = false;
        if heard.truthy() {
            // Store target waters in the ship RAM page.
            let g = junk_goal(heard, role, k, now);
            let h = heard.get("hw").num_or(-1.0);
            let (a, b) = junk_strait(heard, role, k).unwrap_or((-1.0, -1.0));
            if g >= 0.0 && (g != goal || now - carried.get("t").num_or(-1e9) > 60.0) {
                goal = g;
                if let Some(ops) = mem::put(1, &obj! { "g" => g, "h" => h, "t" => now, "a" => a, "b" => b }, "local", Some("unit"), 1) {
                    o.set("ram", ops);
                    send = true;
                }
            }
        } else if role != 1 && now - carried.get("t").num_or(now) > 150.0 && carried.get("h").num_or(-1.0) >= 0.0 {
            // Return home from an idle rival coast.
            goal = carried.get("h").num();
        }
        let course = sea.get("course");
        let water = sea.get("water").num_or(-1.0);
        let chasing = course.get("chase").num_or(-1.0);
        // Close with nearby rival ships.
        let prey = if role != 1 && contacts > 0 { quarry(u, me, world, role) } else { None };
        if let Some(t) = prey {
            let tid = t.get("id").num();
            // The reflex turns and fires at nearby targets.
            let (up, tp) = (u.get("pos"), t.get("pos"));
            let close = up.get("p").num() == tp.get("p").num()
                && (up.get("x").num() - tp.get("x").num()).hypot(up.get("y").num() - tp.get("y").num()) <= 6.5;
            if chasing != tid && !close {
                o.set("sail", obj! { "chase" => tid, "keep" => if role == 2 { 3.5 } else { 4.5 } });
                send = true;
            }
        } else if role==2 && goal < 0.0 && world.get("logistics").truthy() {
            let home=heard.get("hw").num_or(carried.get("h").num_or(-1.0));
            if home>=0.0 && water!=home && chasing<0.0 && sea::route(world,water,home).is_some(){o.set("sail",home);send=true;}
        } else if goal >= 0.0 {
            // Return to station after a chase or detour.
            let on_way = match course {
                c if c.is_arr() => c.at(c.len().saturating_sub(1)).num() == goal,
                c if c.is_str() => c.str() != "heading" && c.str() != "lost" && water == goal,
                c => c.get("x").is_num() && water == goal,
            };
            // Drop unreachable waters; continue any chase begun by reflex.
            if !on_way && chasing < 0.0 && water >= 0.0 && sea::route(world, water, goal).is_some() {
                o.set("sail", goal);
                send = true;
            } else if on_way && water == goal && course.is_str() && course.str() == "stop" && carried.get("a").num_or(-1.0) >= 0.0 {
                // Face the strait so passing ships cross the beam.
                let (a, b) = (carried.get("a").num(), carried.get("b").num());
                let at = u.path("pos.p").num();
                let line = if at == b { realm::toward(realm::index(world).get(b), a, 1.0) } else { realm::toward(realm::index(world).get(a), b, 1.0) };
                let hd = sea.get("head");
                let cross = hd.get("x").num_or(0.0) * line.y - hd.get("y").num_or(0.0) * line.x;
                if cross.abs() > 0.34 && (line.x != 0.0 || line.y != 0.0) {
                    o.set("face", obj! { "x" => line.x, "y" => line.y });
                    send = true;
                }
            }
        }
        // Use armor only when rivals are present; it costs cycles.
        let armored = u.path("rts.modules").has_str("armor");
        let want_armor = if armored && prey.is_some_and(|t|t.path("build.range").num_or(0.0)>0.0) { 450.0 } else { 0.0 };
        let p = u.path("rts.power");
        if send || p.get("armor").num_or(0.0) != want_armor || u.get("alert").num_or(0.0) < 8.0 {
            o.set("power", obj! { "drive" => if role == 2 { 420 } else { 260 }, "armor" => want_armor, "weapon" => 300 });
            o.set("alert", 8);
            send = true;
        }
        if send {
            order(o);
        }
    }
    Value::Undef
}

fn warjunk(units: View, world: View, me: View) -> Value {
    junks(units, world, me, 0)
}

fn picket(units: View, world: View, me: View) -> Value {
    junks(units, world, me, 1)
}

fn corsair(units: View, world: View, me: View) -> Value {
    junks(units, world, me, 2)
}

fn carrier(units: View, world: View, me: View) -> Value {
    // Keep the carrier with the fleet; its air controller handles kites.
    junks(units, world, me, 0)
}

/// A junk in contact: a broadside at whatever is at sea in its arc (the side with more; a broadside with nothing at
/// sea in the arc costs nothing), the beam turned to the nearest rival ship, and its guns on the nearest rival it can
/// reach, ship or shore. A hurt corsair runs and dashes.
fn junk_reflex(u: View, ns: View, pool: &mut Pool) -> Option<Value> {
    let r = u.get("rts");
    let sea = r.get("sea");
    let modules = r.get("modules");
    let reach = u.get("reach").num_or(1.0);
    let merchant=u.get("type").str()=="corsair" && r.get("cargo").truthy();
    let carrying=logistics::cargo(u)>0.0;
    let transferring=r.path("cargo.job").truthy();
    let (mut port, mut star) = (0, 0);
    let mut ship: Option<View> = None;
    let mut shot: Option<View> = None;
    let mut foe: Option<View> = None;
    let mut near = 0;
    for t in ns.iter() {
        if !t.get("rival").truthy() || t.get("air").truthy() {
            continue;
        }
        let d = t.get("d").num();
        if merchant && (carrying || t.path("rts.cargo.load").num_or(0.0)<2.0)
            && t.get("range").num_or(0.0)==0.0 {continue;}
        if d <= 7.0 {
            near += 1;
        }
        let afloat = t.path("rts.sea.ship").truthy();
        if !afloat && foe.is_none() && d <= 7.0 {
            foe = Some(t);
        }
        // broadsides hit only ships and rowers at sea
        if afloat && d <= 6.0 && sea::in_arc(u, t, 44.0) {
            if sea::bearing(u, t).side == "port" {
                port += 1;
            } else {
                star += 1;
            }
        }
        if afloat && ship.is_none() {
            ship = Some(t);
        }
        if shot.is_none() && d <= reach {
            shot = Some(t);
        }
    }
    if near == 0 {
        return None;
    }
    let mut o = obj! { "id" => u.get("id") };
    let mut act = false;
    let ab = r.get("abilityIn");
    let ready = |a: &str| !ab.truthy() || !ab.get(a).truthy() || ab.get(a).num() <= 0.0;
    let hurt = u.get("hp").num() < u.get("hpMax").num() * 0.45;
    let course = sea.get("course");
    let stopped = (course.is_str() && course.str() != "heading") || course.get("chase").is_num();
    if u.get("range").num_or(0.0) > 0.0 && port + star > 0 && ready("broadside") && pool.take("abilities", 300.0 + 150.0 * f64::from(port.max(star))) {
        o.set("ability", "broadside");
        o.set("side", if star > port { "starboard" } else { "port" });
        act = true;
    } else if let Some(t) = ship {
        if u.get("type").str() == "corsair" && hurt && t.get("d").num() < 6.0 {
            // run: away from it, dashing if it can
            o.set("head", obj! { "x" => -t.get("dx").num(), "y" => -t.get("dy").num() });
            if transferring {o.set("cargo",Value::Null);}
            if modules.has_str("dash") && ready("dash") && pool.take("abilities", 900.0) {
                o.set("ability", "dash");
            }
            act = true;
        } else if !carrying && !transferring && t.get("d").num() <= 7.0 && !sea::in_arc(u, t, 30.0) {
            let b = sea::beam(u, t);
            o.set(if stopped { "face" } else { "head" }, obj! { "x" => b.x, "y" => b.y });
            act = true;
        }
    } else if let Some(t) = foe.filter(|_| stopped && !transferring && !carrying) {
        // Chase ships and shore troops outside picket gun range.
        if t.get("d").num() > 5.5 && u.get("type").str() != "picket" {
            o.set("sail", obj! { "chase" => t.get("id"), "keep" => 4 });
            act = true;
        } else if !sea::in_arc(u, t, 30.0) {
            let b = sea::beam(u, t);
            o.set("face", obj! { "x" => b.x, "y" => b.y });
            act = true;
        }
    }
    if let Some(t) = shot {
        if pool.take("weapon", 420.0) {
            o.set("hit", t.get("id"));
            act = true;
        }
    }
    act.then_some(o)
}

// ---- reflex ----

fn dodge(u: View) -> Option<Value> {
    let hs = u.get("rts").get("hazards");
    let pos = u.get("pos");
    if !hs.truthy() || hs.is_empty() || !pos.truthy() {
        return None;
    }
    for h in hs.iter() {
        let dx = pos.get("x").num() - h.get("x").num();
        let dy = pos.get("y").num() - h.get("y").num();
        let d = dx.hypot(dy);
        if h.get("impactIn").num() <= 1.4 && d < h.get("radius").num() + 1.1 {
            return Some(obj! { "x" => if dx != 0.0 && !dx.is_nan() { dx } else { 1.0 }, "y" => if dy != 0.0 && !dy.is_nan() { dy } else { 0.0 } });
        }
    }
    None
}

// Withdraw from melee and artillery while firing at visible targets.
fn pierce_mode(u: View, target: Option<View>) -> f64 {
    let Some(t) = target else { return 0.0 };
    let tr = t.get("rts");
    if !u.get("rts").get("modules").has_str("pierce") || !tr.truthy() {
        return 0.0;
    }
    let mut armor = tr.get("armor").or(0.0);
    let guard = tr.get("guard");
    let d = t.get("d").num();
    if guard.truthy() && d > 0.01 {
        armor = min(0.8, armor * (0.85 - 0.5 * (guard.at(0).num() * t.get("dx").num() + guard.at(1).num() * t.get("dy").num()) / d));
    }
    // Search public damage and CPU data within the call budget.
    let mut best = 0.0;
    let mut efficiency = 1.0 - armor;
    for p in [0.2, 0.4, 0.6, 0.8, 1.0] {
        let value = (1.0 - 0.35 * p) * (1.0 - armor * (1.0 - p)) / (1.0 + 1.4 * p * p);
        if value > efficiency * 1.03 {
            efficiency = value;
            best = p;
        }
    }
    best
}

fn guard_facing(ns: View) -> Value {
    let (mut x, mut y, mut n) = (0.0, 0.0, 0.0);
    for t in ns.iter() {
        let d = t.get("d").num();
        if t.get("rival").truthy() && !t.get("air").truthy() && d > 0.0 {
            x += t.get("dx").num() / d;
            y += t.get("dy").num() / d;
            n += 1.0;
        }
    }
    if n != 0.0 && x.hypot(y) > n * 0.45 {
        obj! { "x" => x, "y" => y }
    } else {
        Value::Null
    }
}

/// What the units in contact may spend this call, lane by lane, and the world's price level.
struct Pool {
    drive: f64,
    armor: f64,
    cloak: f64,
    weapon: f64,
    abilities: f64,
    level: f64,
}

impl Pool {
    /// Spends `cost` (the rules' price, times the price level) from a lane when it holds that much.
    fn take(&mut self, lane: &str, cost: f64) -> bool {
        let cost = cost * self.level;
        let left = match lane {
            "drive" => &mut self.drive,
            "armor" => &mut self.armor,
            "cloak" => &mut self.cloak,
            "weapon" => &mut self.weapon,
            _ => &mut self.abilities,
        };
        if *left < cost {
            return false;
        }
        *left -= cost;
        true
    }
}

fn reflex(units: View, near: View) -> Value {
    // Share lane budgets across contacted units.
    let n = units.len();
    let b = if n > 0 { units.at(0).get("rts").get("budget") } else { View::undefined() };
    let level = if n > 0 { common::price_level(units.at(0)) } else { 1.0 };
    let mut pool = Pool {
        drive: max(0.0, b.get("drive").or(0.0) - 120.0 * level),
        armor: max(0.0, b.get("armor").or(0.0) - 80.0 * level),
        cloak: max(0.0, b.get("cloak").or(0.0) - 70.0 * level),
        weapon: max(0.0, b.get("weapon").or(0.0) - 240.0 * level),
        abilities: max(0.0, b.get("abilities").or(0.0) - 180.0 * level),
        level,
    };
    let count = min(n as f64, max(1.0, (b.get("reflex").or(0.0) / 850.0).floor())) as usize;
    let start = if n > 0 { ((units.at(0).get("rts").get("bank").or(0.0) / 175.0).floor() % n as f64) as usize } else { 0 };
    for turn in 0..count {
        if cycles_left() < 1800.0 {
            break;
        }
        let u = units.at((start + turn) % n);
        let ns = near.key(u.get("id").num());
        // a junk fights as a ship: broadsides, its beam and its guns
        if u.path("rts.sea.ship").truthy() {
            if let Some(o) = junk_reflex(u, ns, &mut pool) {
                order(o);
            }
            continue;
        }
        let r = u.get("rts");
        let modules = r.get("modules");
        let (mut best, mut melee): (Option<View>, Option<View>) = (None, None);
        let (mut best_score, mut rivals) = (1e9, 0);
        for t in ns.iter().take(4) {
            if !t.get("rival").truthy() {
                continue;
            }
            rivals += 1;
            let air = t.get("air").truthy();
            let d = t.get("d").num();
            if melee.is_none() && !air && t.get("range").is_num() && t.get("range").num() == 0.0 && d <= 2.4 {
                melee = Some(t);
            }
            if d > u.get("reach").num() || (air && !u.get("air").truthy() && !modules.has_str("antiAir")) {
                continue;
            }
            // Finish damaged targets and prioritize exposed ranged or air threats.
            let score = t.get("hp").or(1.0) + d * 0.12 - if t.get("range").truthy() { 0.32 } else { 0.0 }
                - if air { if common::air_defender(u) { 100.0 } else { 0.20 } } else { 0.0 };
            if score < best_score {
                best = Some(t);
                best_score = score;
            }
        }
        let hurt = u.get("hp").num() < u.get("hpMax").num() * 0.52;
        let ty = u.get("type").str();
        let flee = dodge(u).or_else(|| {
            let m = melee?;
            (m.get("d").num() > 0.0 && (u.get("range").num() > 0.0 || ty == "shade" || hurt))
                .then(|| obj! { "x" => -m.get("dx").num(), "y" => -m.get("dy").num() })
        });
        let driving = flee.is_some() && pool.take("drive", 150.0);
        let armored = modules.has_str("armor") && (melee.is_some() || hurt) && pool.take("armor", 110.0);
        let penetration = pierce_mode(u, best);
        let armed = best.is_some() && pool.take("weapon", (540.0 * (1.0 + 1.4 * penetration * penetration)).ceil());
        let cloaking = ty == "shade" && best.is_none() && flee.is_none() && pool.take("cloak", 90.0);
        let mut o = obj! {
            "id" => u.get("id"),
            "guard" => guard_facing(ns),
            "pierce" => penetration,
            "power" => obj! { "drive" => if driving { 760 } else { 0 }, "armor" => if armored { 520 } else { 0 }, "weapon" => if armed { 420 } else { 0 } },
        };
        if let (true, Some(t)) = (armed, best) {
            o.set("hit", t.get("id"));
            // Circle guarded lines and split flanks.
            if flee.is_none() && t.get("rts").get("guard").truthy() && pool.take("drive", 100.0) {
                let side = if u.get("id").num() % 2.0 != 0.0 { 1.0 } else { -1.0 };
                o.set("step", obj! { "x" => -t.get("dy").num() * side, "y" => t.get("dx").num() * side });
                o.entry("power").set("drive", 440);
            }
        }
        if driving {
            if let Some(f) = flee {
                o.set("step", f);
            }
        } else if armed && o.get("step").is_undef() && reflex::engaged(u) {
            o.set("hold", true);
        }
        let ability_in = r.get("abilityIn");
        let ready = |a: &str| !ability_in.truthy() || !ability_in.get(a).truthy() || ability_in.get(a).num() <= 0.0;
        let artillery = modules.has_str("artillery");
        if artillery && best.is_some() && !r.get("deployed").truthy() && !r.get("deploying").truthy() {
            o.set("deploy", true);
        }
        let shell = match best {
            Some(t) if artillery && r.get("deployed").truthy() && armed && t.get("d").num() > 1.4 && ready("shell") => pool.take("abilities", 1000.0).then_some(t),
            _ => None,
        };
        if let Some(t) = shell {
            let pos = u.get("pos");
            o.set("ability", "shell");
            o.set("aim", obj! { "x" => pos.get("x").num() + t.get("dx").num(), "y" => pos.get("y").num() + t.get("dy").num() });
        } else if driving && modules.has_str("dash") && ready("dash") && pool.take("abilities", 900.0) {
            o.set("ability", "dash");
        }
        // Scan only during fights, not while patrolling.
        if !o.get("ability").truthy() && rivals >= 2 && modules.has_str("scan") && ready("scan") && pool.take("abilities", 700.0) {
            o.set("ability", "scan");
        }
        if cloaking {
            o.set("cloak", true);
        }
        if armed || driving || armored || o.get("cloak").truthy() || o.get("ability").truthy() {
            order(o);
        }
    }
    Value::arr()
}

// ---- Puzzles ----

fn research(c: View) -> Value {
    common::solve(c)
}

fn duel(c: View) -> Value {
    common::solve(c)
}

fn challenge(foe: View, max: View) -> Value {
    common::challenge(foe, max)
}

// Each founder supplies its own body, weapon, projectile, anthem and capital.
fn look(budget: View, info: View) -> Value {
    let house = theme::of(info.get("seat"), 0.0);
    let mut l = house.map_or(Value::Null, |t| t.look(budget.num(), info));
    // At sea, ships use the Fleet junk body and name.
    if l.truthy() {
        for (ship, ashore) in [("warjunk", ["lancer", "shade", "runner"]), ("picket", ["sentry", "spire", "runner"]), ("corsair", ["needle", "shade", "runner"]), ("carrier", ["kite", "kite", "runner"])] {
            // Ashore, use the house body or its land counterpart.
            let body = ashore.iter().find(|t| l.get("units").get(t).truthy())
                .map_or_else(|| l.get("units").get("guard").clone(), |kin| l.get("units").get(kin).clone());
            l.entry("units").set(ship, body);
            if let Some(name) = house.and_then(|t| t.naval_name(ship)) {
                l.entry("names").set(ship, name);
            }
        }
        // Keep ship entries within the look type limit.
        for part in ["units", "names"] {
            if let Value::Obj(o) = l.entry(part) {
                o.remove("envoy");
                o.remove("sailor");
                while o.0.len() > 20 {
                    let unused = o.0.iter().position(|(k, _)| !matches!(k.as_str(), "miner" | "hauler" | "freighter" | "guard" | "scout" | "longbow" | "spy" | "runner" | "lancer" | "shade" | "sentry" | "spire" | "needle" | "kite" | "warjunk" | "picket" | "corsair" | "carrier"));
                    let Some(i) = unused else { break };
                    o.0.remove(i);
                }
            }
        }
    }
    l
}

fn build3d(budget: View, info: View) -> Value {
    theme::of(info.get("seat"), 0.0).map_or(Value::Null, |t| t.scene(info, budget.num()))
}

fn live3d(state: View, now: View) -> Value {
    let mut s = if state.truthy() { state.value() } else { Value::obj() };
    let spin = capital::ring(&mut s, now);
    obj! { "state" => s, "set" => obj! { "wonder" => spin } }
}
