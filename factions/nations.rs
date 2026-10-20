// Compute of Nations funds its army through trade, then invests in land.

use gameofcores::*;
use shared::{air, capital, common, theme};
use shared::common::{hauler, freighter, escort};

program!(think, hauler, freighter, escort, miner, runner, lancer, shade, sentry, spire, needle, kite, scout, reflex, research, duel, challenge, look, build3d, live3d);

// The bodies: points in hp, speed, strength, carry and vision, and for some in range and wings.
fn miner_body() -> Value {
    obj! { "hp" => 2, "speed" => 2, "strength" => 0, "carry" => 5, "vision" => 1 }
}
fn runner_body() -> Value {
    obj! { "hp" => 3, "speed" => 2, "strength" => 2, "carry" => 0, "vision" => 3 }
}
fn shade_body() -> Value {
    obj! { "hp" => 2, "speed" => 2, "strength" => 2, "carry" => 0, "vision" => 2, "range" => 1 }
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
fn kite_body() -> Value {
    obj! { "hp" => 2, "speed" => 3, "strength" => 2, "carry" => 0, "vision" => 0, "wings" => 1 }
}
fn kite_shot_body() -> Value {
    obj! { "hp" => 2, "speed" => 2, "strength" => 1, "carry" => 0, "vision" => 0, "wings" => 1, "range" => 1 }
}

// Research what nobody sells; buy what another faction holds.
const TECHS: [&str; 24] = [
    "extraction", "seaTraversal", "turbines", "armor", "ranged", "transmission", "repair", "influence", "capture",
    "dash", "cloak", "antiAir", "scan", "pierce", "artillery", "storage", "drilling", "fission", "computeBoost",
    "mountainTraversal", "flight", "airWeapons", "stealth", "gate",
];

// ---- JavaScript's arithmetic, where this program follows it ----

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

// Market timers use a random ±60% gap. Their next times live in capital RAM page 224.
const CLOCK: u32 = 224;
const CLOCK_PAGES: u32 = 6;

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

// Sea-station ranking waits until its searches and the remaining planner can fit.
fn rank_hold(world: View, me: View) -> impl Fn(View, f64) -> bool {
    let land = common::can_hold(world, me, false);
    let naval = common::can_hold(world, me, true);
    let work = 300.0 * world.get("waters").len() as f64;
    let cpu = rts(me);
    move |q, left| land(q) || ((!cpu || left > 18000.0 + work
        * (q.path("routes.sea").len() + q.path("routes.air").len()).max(1) as f64) && naval(q))
}

// Trade funds a campaign through valuable land. Existing leases keep their protection.
fn frontline(world: View, me: View) -> Vec<View> {
    let me_id = me.get("id").num();
    let hold = rank_hold(world, me);
    let campaign = mem::get(me.get("ramCapital"), 0).get("campaign").num_or(-1.0);
    let fighters = common::fighters(me);
    let by_id = realm::index(world);
    let candidates: Vec<View> = common::frontier(world, me).into_iter()
        .filter(|q| !q.get("leased").iter().any(|l| l.get("to").num() == me_id)).collect();
    let ranked = realm::rank(world, me, candidates, |_, _, q| {
        let owner = q.get("owner").num();
        let hostile = rival_in(q, me);
        let enemy = owner >= 0.0 && owner != me_id;
        let capital = enemy && q.get("capital").truthy() && common::muster_for(q) <= fighters;
        let approach = enemy && q.get("adj").iter().any(|id| {
            let next = by_id.at(id);
            next.get("owner").num_or(-2.0) == owner && next.get("capital").truthy()
        });
        q.get("cores").or(0.0) * 3.0 + if enemy { 9.0 } else { 2.0 } - hostile * 0.6
            + if capital { 12.0 } else if q.get("capital").truthy() { -5.0 } else { 0.0 }
            + if approach { 5.0 } else { 0.0 }
            + if enemy && owner == campaign { 4.0 } else { 0.0 }
            - if hold(q, cycles_left()) { 0.0 } else { common::OVER_WATER }
    });
    common::rally(world, me, ranked)
}

/// What think saw of its rivals' units: fliers, shooters and all, and their miners at work by province.
#[derive(Default)]
struct Threats {
    air: f64,
    ranged: f64,
    total: f64,
    /// (province, miners at work there), in the order first seen.
    miners: Vec<(f64, f64)>,
}

// Visible enemy composition and miners; targets remain reflex-local under fog.
fn seen_threats(world: View, me: View) -> Threats {
    let me_id = me.get("id").num();
    let mut t = Threats::default();
    for u in world.get("units").iter() {
        let owner = u.get("owner").num();
        if owner == me_id || !realm::is_rival(world, me, owner) {
            continue;
        }
        t.total += 1.0;
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

// Keep a rebuilding force of twelve fighters; larger armies must fit 60% of income.
fn army_room(me: View) -> f64 {
    let u = me.get("units");
    max(12.0 - common::army(me) - u.get("queued").num() - u.get("hiredOut").or(0.0), common::room(me, 0.6))
}

thread_local! {
    /// ◆ the spawns leave in the bank this second: what the market is saving for (market::saving).
    static SAVING: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}

fn add_spawn(orders: &mut Value, me: View, ty: &str, build: Value, modules: &[&str]) -> bool {
    if army_room(me) <= 0.0 || me.get("bank").num() < me.get("costs").get("unit").num() + 4.0 + SAVING.with(std::cell::Cell::get) {
        return false;
    }
    // Every CPU-RTS body has fabrication work. Keep a buffer because this view
    // is one simulation second old and reflexes may have spent since it.
    let fabrication = common::fabrication(me, modules.len() as f64);
    let available = me.get("rts").get("cpu").get("available");
    let available = if available.truthy() { available.get("fabrication").num() } else { me.get("rts").get("bank").num() };
    if available < fabrication + 300.0 * common::price_level(me) {
        return false;
    }
    if !orders.get("spawn").truthy() {
        orders.set("spawn", Value::arr());
    }
    let queued = orders.get("spawn").len();
    if queued >= 2 {
        return false;
    }
    let units = me.get("units");
    let n = units.get("spawned").num() + units.get("queued").num() + queued as f64 + 1.0;
    let s = obj! { "type" => ty, "name" => format!("{ty} {}", text(n)), "build" => build, "modules" => modules.to_vec() };
    orders.entry("spawn").push(s);
    true
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

// Remove spawns the bank cannot fund after accepted deals, sends, and bribes.
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
    for s in orders.get("send").items() {
        let coin = s.get("give").get("coin");
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

// Keep spawns payable after think and RAM writes, which draw on the shared bank first.
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
    let mut room = min(limit, c.get("bank").or(0.0) - used - held) - 300.0;
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

// Share one costly route field across unit types; reuse it until the destination
// changes, and skip searches unless their cost and `after` runway fit.
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

fn cpu_policy(world: View, me: View, offer: f64, under_attack: bool) -> Value {
    let c = me.get("rts").get("cpu");
    let old = c.get("reserve").get("research").or(0.0);
    let income = c.get("income").or(14000.0);
    // An offer larger than three seconds of income waits on savings (the thresholds here were tuned on a store that size).
    let capacity = income * 3.0;
    // Keep a planner/reflex runway rather than reserving the whole balance and
    // waiting forever for a research call that cannot start yet.
    let research_cap = max(0.0, capacity - 18000.0);
    // Save toward affordable research while quiet; threats release the shared bank to combat.
    // Oversized offers reserve nothing, so research cannot starve planning or miners.
    let offered = offer != 0.0 && !offer.is_nan();
    let research = if under_attack || !offered || offer > research_cap {
        0.0
    } else {
        min(min(offer, income * 0.6 + c.get("bank").or(0.0) * 0.18), old + max(2500.0, income * 0.18))
    };
    // Scale the think limit with army size; unused room stays in the shared bank.
    let think = round(min(
        max(64000.0, income * 0.5),
        max(max(24000.0 + me.get("units").get("total").or(0.0) * 200.0, income * 0.4), me.get("compute").get("thinkPeak").or(0.0) * 1.5),
    ));
    let think=think.max((planner_map_need(world,me)+6000.0)
        .min((c.get("spendCap").num_or(f64::INFINITY)-cycles_reserved()).max(0.0)));
    obj! {
        "limits" => obj! {
            "think" => think, "control" => round(max(50000.0, income * 0.5)), "reflex" => 50000,
            "research" => 0.3,
            "upkeep" => 0.72, "drive" => 0.34, "armor" => 0.26, "cloak" => 0.18, "weapon" => 0.48,
            // fabrication: a third of income, and never less than a two-module body and its margin
            "abilities" => 0.28, "fabrication" => round(max(income * 0.34, common::fabrication(me, 2.0) + 300.0 * common::price_level(me))), "gate" => 0.08, "duel" => 0.14, "challenge" => 0.08, "visuals" => 0.03,
        },
        "reserve" => obj! { "think" => 3500, "reflex" => 7000, "research" => research },
        "upkeepOrder" => vec!["miner", "sentry", "spire", "shade", "kite", "needle", "scout", "runner"],
    }
}

// A damaged faction keeps accepted orders and local reflexes rather than
// consuming its last shared CPU on map searches and new fabrication.
fn conserve(world: View, me: View) -> Value {
    common::recovery(world, me)
}

fn think(world: View, me: View) -> Value {
    if cycles_left()<800.0 {return obj!{};}
    common::govern(world, me, 0.4, || {
        let mut orders = strategy(world, me);
        // A recruit needs a complete upkeep review before the governor and ABI tail.
        if rts(me) && !orders.get("spawn").is_empty()
            && cycles_left() < 4000.0 + 220.0 * world.get("units").len() as f64 {
            orders.set("spawn", Value::arr());
        }
        orders
    })
}

enum PlannerAdmission { Full, Raise(Value), Lean }

fn planner_map_need(world: View, me: View) -> f64 {
    let base=if me.path("units.total").num_or(0.0)<20.0 {15000.0}else{20000.0};
    // Frontier ranking, survey, roads and shared-plan publication each revisit the map.
    base+400.0*world.get("provinces").len() as f64+100.0*world.get("units").len() as f64
}

// Repair a planner allowance before opening the map and market. A capped call runs a separate local pass.
fn planner_admission(world: View, me: View, remaining: f64) -> PlannerAdmission {
    let cpu = me.path("rts.cpu");
    let income = cpu.get("income").num_or(0.0);
    let bank = cpu.get("bank").num_or(0.0);
    let cap = (cpu.get("spendCap").num_or(0.0)-cycles_reserved()).max(0.0);
    if income <= 0.0 || cap <= 0.0 { return PlannerAdmission::Full; }
    let remaining = remaining.min(cpu.path("available.think").num_or(remaining)).min(cap);
    let amount = |n: f64| if n <= 1.0 { n * income } else { n };
    let configured = (amount(cpu.path("limits.think").num_or(cap))-cycles_reserved()).max(0.0);
    let peak = me.path("compute.thinkPeak").num_or(0.0).max(me.path("compute.used.think").num_or(0.0));
    let exhausted = peak >= configured * 0.97;
    let observed = if exhausted { peak.max(configured) } else { peak };
    let base: f64 = if me.path("units.total").num_or(0.0) < 20.0 { 15000.0 } else { 20000.0 };
    // Cheap recovery calls erase the recent peak. A full map, census, policy and return still need room.
    let shape=planner_map_need(world,me);
    let required = shape.max(observed * 1.2 + 4000.0);
    if remaining >= required { return PlannerAdmission::Full; }
    if bank < income * 2.0 || me.path("units.stalled").num_or(0.0) > 0.0 { return PlannerAdmission::Lean; }
    let protected: f64 = cpu.get("reserve").entries().filter(|(lane, _)| *lane != "think")
        .map(|(_, value)| amount(value.num_or(0.0))).sum();
    let ceiling = cap.min((bank - protected - 4000.0).max(0.0)).floor();
    if ceiling < required { return PlannerAdmission::Lean; }
    let next = (required + 4000.0).min(ceiling).floor();
    if next <= configured + 2000.0 || next < base + 4000.0 { return PlannerAdmission::Lean; }
    let mut policy = obj! {
        "limits"=>if cpu.get("limits").truthy() {cpu.get("limits").value()} else {obj! {}},
        "reserve"=>if cpu.get("reserve").truthy() {cpu.get("reserve").value()} else {obj! {}},
        "upkeepOrder"=>if cpu.get("upkeepOrder").is_arr() {cpu.get("upkeepOrder").value()} else {arr!["miner"]},
    };
    policy.entry("limits").set("think",if next <= income {next / income} else {next});
    PlannerAdmission::Raise(obj! {"compute"=>obj! {"cpu"=>policy}})
}

// Rotate a bounded frontier window, keeping the capital first and time to publish.
fn lean_plan(world: View, me: View) -> (Value, f64) {
    let by = realm::index(world);
    let ours = me.get("id").num();
    let capital = me.get("capital").num();
    let last = mem::get(me.get("ramCapital"), 0);
    let old_attack = last.get("attack").num_or(-1.0);
    let campaign = last.get("campaign").num_or(-1.0);
    let hosts: Vec<f64> = me.get("contracts").iter().take(32)
        .filter(|c| !c.get("ended").truthy() && c.path("get.passage").truthy())
        .map(|c| c.get("with").num()).collect();
    let provinces = world.get("provinces");
    let count = provinces.len();
    let offset = (world.get("seconds").num_or(0.0) / 3.0) as usize;
    let (mut attack, mut stage, mut need, mut best) = (-1.0, capital, 0.0, f64::NEG_INFINITY);
    let (mut defend, mut threat, mut mine_room) = (-1.0, 0.0, 0.0);
    let mut mines = Vec::new();
    for i in 0..count.min(48) + 1 {
        if cycles_left() < 10000.0 { break; }
        let p = if i == 0 { by.get(capital) } else { provinces.at((offset + i - 1) % count) };
        let id = p.get("id").num_or(-1.0);
        if p.get("owner").num_or(-1.0) != ours || (i > 0 && id == capital) { continue; }
        let enemies = rival_in(p, me);
        if (defend != capital && enemies > threat) || (id == capital && enemies > 0.0) { defend = id; threat = enemies; }
        for (k, value) in p.get("mines").iter().enumerate().take(8) {
            if value.num_or(0.0) <= 0.0 || realm::worker(p, k as f64) != ours { continue; }
            mine_room += 2.0;
            let staff = p.get("staff").at(k).num_or(0.0).max(0.0) as usize;
            for _ in staff..2 { if mines.len() < 12 { mines.push(arr![id, k]); } }
        }
        for adjacent in p.get("adj").iter().take(12) {
            if cycles_left() < 10000.0 { break; }
            let q = by.at(adjacent);
            let owner = q.get("owner").num_or(-1.0);
            if !q.truthy() || owner == ours || (owner >= 0.0 && (!q.get("visible").truthy()
                || me.get("contracts").len() > 32 || !realm::is_rival(world, me, owner))) || hosts.contains(&owner)
                || q.get("leased").iter().take(16).any(|l| l.get("to").num_or(-1.0) == ours)
                || common::hopeless(me, q) { continue; }
            let target = q.get("id").num_or(-1.0);
            let score = q.get("cores").num_or(0.0) * 3.0 - rival_in(q, me) * 0.6
                + if owner >= 0.0 { 9.0 } else { 2.0 }
                + if target == old_attack { 10000.0 } else { 0.0 }
                + if campaign >= 0.0 && owner == campaign { 4.0 } else { 0.0 }
                + if q.get("capital").truthy() && common::muster_for(q) <= common::fighters(me) { 12.0 } else { 0.0 };
            if target >= 0.0 && score > best { attack = target; stage = id; need = common::muster_for(q); best = score; }
        }
    }
    let owner = by.get(attack).get("owner").num_or(-1.0);
    (obj! {"home"=>capital,"attack"=>attack,"target"=>attack,"stage"=>stage,"need"=>need,
        "defend"=>defend,"threat"=>threat,"campaign"=>if realm::is_rival(world,me,owner) {owner} else {-1.0},
        "mines"=>mines,"fresh"=>(world.get("seconds").num_or(0.0)/15.0).floor()*15.0}, mine_room)
}

fn lean_research(orders: &mut Value, world: View, me: View) {
    let all = world.get("techs");
    let mine = me.get("techs");
    if !me.path("research.tech").str().is_empty() { return; }
    for &want in &TECHS {
        if cycles_left() < 7000.0 { break; }
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
            orders.set("research", pick); break;
        }
    }
}

fn strategy(world: View, me: View) -> Value {
    // Short calls still set a CPU policy so the next call can recover.
    if rts(me) && cycles_left() < 3000.0 {
        return common::recovery(world, me);
    }
    let start = cycles_left();
    let full_plan = if rts(me) { match planner_admission(world, me, start) {
        PlannerAdmission::Full => true,
        PlannerAdmission::Raise(orders) => return orders,
        PlannerAdmission::Lean => false,
    } } else { true };
    // Keep standing orders when the full census and market won't fit. Young
    // realms use a lower runway so miners can still be recruited.
    let available = me.get("rts").get("cpu").get("available");
    let full = if full_plan && me.get("units").get("total").or(0.0) >= 20.0 { 20000.0 } else { 15000.0 };
    if rts(me) && (cycles_left() < full || (available.truthy() && available.get("think").num() < 5000.0)) {
        return conserve(world, me);
    }
    let me_id = me.get("id").num();
    let techs = me.get("techs");
    let has = |t: &str| techs.get(t).truthy();
    let have = me.get("units").get("byType");
    let ranked = if full_plan { frontline(world, me) } else { Vec::new() };
    let (mut plan, local_mine_room) = if full_plan { (common::survey(world, me, &ranked, 2.0), 0.0) } else { lean_plan(world, me) };
    let target_id = plan.get("attack").num_or(-1.0);
    let target = if full_plan { ranked.iter().copied().find(|q| q.get("id").num() == target_id) }
        else { (target_id >= 0.0).then(|| realm::index(world).get(target_id)) };
    let owner = target.map_or(-1.0, |q| q.get("owner").num_or(-1.0));
    plan.set("target", target_id);
    plan.set("campaign", if realm::is_rival(world, me, owner) { owner } else { -1.0 });
    let defend = plan.get("defend").num_or(-1.0);
    let threatened = plan.get("threat").num_or(0.0) > 0.0;
    // unseen land the game said a partner holds (common::partner_land), remembered past the second it was refused
    let partner = if full_plan { common::partner_land(world, me) } else { std::rc::Rc::new(Vec::new()) };
    if !partner.is_empty() {
        plan.set("partner", Value::Arr(partner.iter().map(|&id| Value::Num(id)).collect()));
    }
    if full_plan {
        if let Some(field) = road(world, me, if defend >= 0.0 { defend } else { target_id }, 20000.0) {
            plan.set("field", field);
        }
    }
    // the census of rivals' units runs only when think can pay for it and still finish (the rest costs about 14,000)
    let census = full_plan && (!rts(me) || cycles_left() > 55.0 * world.get("units").len() as f64 + 14000.0);
    let enemy = if census { seen_threats(world, me) } else { Threats::default() };
    if full_plan && has("flight") {
        plan.set("prey", prey(world, me, &enemy.miners));
    }
    let mut orders = obj! {
        "policy" => obj! { "extraction" => 0.42, "autonomy" => "direct", "integrate" => "tolerate", "invest" => true },
        "provinces" => Value::arr(),
    };
    common::doctrine(&mut plan,world,me);
    if full_plan { common::share(&mut orders, &plan, world, me); }
    else { plan::share(&mut orders, &plan, me); }

    // Save for sold techs, except under attack or during the first four minutes.
    let sold = if full_plan { market::sold(world, me) } else { Vec::new() };
    let now = world.get("seconds").or(0.0).floor();
    let at = if full_plan { mem::get(me.get("ramCapital"), CLOCK) } else { Value::obj() };
    let mut clock = Clock { at: if at.truthy() { at } else { Value::obj() }, changed: false };
    SAVING.with(|s| s.set(if !full_plan || threatened || now < 240.0 { 0.0 } else { market::saving(world, me, &sold, &clock, now) }));
    // Fabrication is paid from the combat bank.  Leave it alone until it can
    // afford a useful field unit instead of repeatedly queuing a rejected spawn.
    let tactical_bank = if rts(me) { me.get("rts").get("bank").num() } else { 0.0 };
    let miners = have.get("miner").or(0.0);
    let room = if full_plan { realm::mine_room(world, me) } else { local_mine_room };
    let shades = have.get("shade").or(0.0);
    let sentries = have.get("sentry").or(0.0);
    let needles = have.get("needle").or(0.0);
    let spires = have.get("spire").or(0.0);
    let shade_ready = has("dash") && has("cloak") && has("ranged");
    let specialist_bank = tactical_bank > 2450.0 * common::price_level(me);
    let total = me.get("units").get("total").num();
    let guards = common::fighters(me);
    let guard_floor = if threatened { max(4.0, plan.get("threat").num_or(0.0) * 1.5) } else { 2.0 };
    if miners >= 2.0 && guards < guard_floor {
        add_spawn(&mut orders, me, "runner", runner_body(), &[]);
    } else if miners < room.min((me.get("provinces").num_or(1.0)*2.0).clamp(4.0,12.0)) {
        add_spawn(&mut orders, me, "miner", miner_body(), &[]);
    } else if !shade_ready {
        add_spawn(&mut orders, me, "runner", runner_body(), &[]);
    }
    // A sighted air wing takes precedence, then a scan anchor if opponents can
    // cloak, then short-range cloaked pressure against ranged screens.
    else if specialist_bank && has("ranged") && has("dash") && has("antiAir") && enemy.air > 0.0 && needles < max(2.0, enemy.air) {
        add_spawn(&mut orders, me, "needle", needle_body(), &["dash", "antiAir"]);
    } else if specialist_bank && has("scan") && has("armor") && sentries < max(1.0, ((enemy.ranged + shades) / 5.0).ceil()) {
        add_spawn(&mut orders, me, "sentry", sentry_body(), &["armor", "scan"]);
    } else if specialist_bank && has("artillery") && has("scan") && enemy.total >= 4.0 && spires < max(1.0, (enemy.ranged / 3.0).ceil()) {
        add_spawn(&mut orders, me, "spire", spire_body(), &["artillery", "scan"]);
    } else if specialist_bank && has("flight") && have.get("kite").or(0.0) < min(6.0, max(2.0, (total / 10.0).floor())) {
        add_spawn(&mut orders, me, "kite", if has("airWeapons") { kite_shot_body() } else { kite_body() }, if has("dash") { &["dash"][..] } else { &[][..] });
    } else if specialist_bank && has("pierce") && have.get("lancer").or(0.0) < max(2.0, (shades / 3.0).ceil()) {
        add_spawn(&mut orders, me, "lancer", shade_body(), &["dash", "pierce"]);
    } else if specialist_bank && shades < max(5.0, min(18.0, miners + (enemy.ranged / 2.0).ceil())) {
        add_spawn(&mut orders, me, "shade", shade_body(), &["dash", "cloak"]);
    } else if specialist_bank && has("dash") && has("scan") && have.get("scout").or(0.0) < 2.0 {
        add_spawn(&mut orders, me, "scout", scout_body(), &["dash", "scan"]);
    }

    let name = target.map_or_else(|| "the frontier".to_string(), |t| t.get("name").str().to_string());
    // the army's line, for a second the market doesn't run (when it does, it says what it's doing instead)
    orders.set("memo", format!("Kirkcaldy's guard: {} shades and {} sentries, marching on {name}", text(shades), text(sentries)));
    // Fund research early; later reserve combat cycles for fabrication and reflexes.
    let available_control = if available.truthy() { available.get("control").num() * 0.8 } else { f64::INFINITY };
    let types = common::calls(
        world,
        // Standing orders persist; each controller keeps route room.
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
        ],
        me.get("rts").get("cpu").get("income").or(0.0),
        available_control,
        me.get("units").get("byType"),
    );
    // Research what nobody sells; buy the rest.
    if full_plan {
        let research = market::research_order(&sold);
        common::budget(&mut orders, world, me, &common::Budget { research: 0.12, code: 0.10, techs: &research, types: Some(&types) });
    } else {
        orders.set("compute", obj! {"types"=>types});
        lean_research(&mut orders, world, me);
    }
    if rts(me) {
        // the tech under study: common::budget names it in orders.research only on the second it changes
        let studying = match orders.get("research") {
            Value::Str(s) if !s.is_empty() => s.clone(),
            _ => me.get("research").get("tech").str().to_string(),
        };
        let tech = if studying.is_empty() { View::undefined() } else { world.get("techs").get(&studying) };
        let offer = if tech.truthy() { tech.get("offer").num() } else { 0.0 };
        // Contacts release research reserves for combat.
        let mut policy = cpu_policy(world, me, offer, threatened || enemy.total >= 2.0);
        common::fit_cpu(&mut policy, me);
        orders.entry("compute").set("cpu", policy);
    }
    if !full_plan { return affordable(orders, me, start); }
    let pact_leader = realm::leader(world).map(|l| l.get("id").num());
    // Keep accepted orders on a short or exhausted second; CPU limits and
    // market timers need a completed think to persist.
    let limits = me.get("rts").get("cpu").get("limits");
    let limit = if limits.truthy() { limits.get("think").or(0.0) } else { 0.0 };
    let ran_out = limit > 1.0 && me.get("compute").get("used").get("think").or(0.0) >= limit * 0.97;
    if cycles_left() < 6000.0 || (rts(me) && ran_out) {
        return affordable(orders, me, start);
    }
    // Court large non-leaders outside pacts.
    let no_pacts = me.get("pacts").is_empty();
    // Avoid duplicates while one awaits an answer or within 10 s of the last proposal.
    if no_pacts && !me.get("inbox").iter().any(|m| m.get("pact").truthy()) && clock.at.get("ct").num_or(-1.0) <= now {
        for c in common::partners(world, me).iter().take(2) {
            let id = c.get("id").num();
            if common::court(&mut orders, world, me, id, |o| diplomacy::propose(o, me, vec![id], (), "a common market, against the leader"), &common::Court::default()) {
                clock.at.set("ct", now + 10.0);
                clock.changed = true;
                break;
            }
        }
    }
    // Reserve the governor and ABI tail before optional market searches.
    let seen = 10.0 * world.get("units").len() as f64;
    if !rts(me) || cycles_left() > 30000.0 + seen {
        let mood = market::Mood { threatened, fight: threatened || enemy.total >= 4.0, shade_ready, target: target_id, army: name };
        market::run(&mut orders, world, me, &mut clock, now, &mood, &sold);
    }
    if !rts(me) || cycles_left() > 2000.0 {
        let pact = move |m: View| no_pacts && pact_leader.is_none_or(|l| m.get("from").num() != l);
        market::answer(&mut orders, me, world, pact, !threatened);
    }
    if clock.changed {
        concat(&mut orders, "ram", mem::put(CLOCK, &clock.at, "local", None, CLOCK_PAGES).unwrap_or_default());
    }
    affordable(orders, me, start)
}

// ---- The market ----
// Timed trades use prices from common::worth and common::price.
mod market {
    use super::*;

    /// What think saw that the market needs.
    pub struct Mood {
        /// Invaders in our land.
        pub threatened: bool,
        /// A fight on: invaders, or four or more rivals' units in sight.
        pub fight: bool,
        /// The shades can be built (dash, cloak and ranged), so the runners are spare.
        pub shade_ready: bool,
        /// The army's target province (-1 for none), and its name.
        pub target: f64,
        pub army: String,
    }

    /// The tolls on our border, and on our crossings by sea or pass, in ◆ a unit (at most `rules.trade.tollMax`, 50).
    const TOLL: f64 = 30.0;
    const TOLL_CROSSING: f64 = 45.0;
    /// What we pay for a tech (the house programs' rule prices one at 150◆), and what we ask for one of ours.
    const TECH_BUY: f64 = 155.0;
    const TECH_SELL: f64 = 140.0;

    /// The research order: the wanted techs no other faction standing holds first (somebody has to make them), then the
    /// ones we can buy (`sold`), in case nobody sells.
    pub fn research_order(sold: &[&'static str]) -> Vec<&'static str> {
        let mut out: Vec<&'static str> = TECHS.iter().copied().filter(|t| !sold.contains(t)).collect();
        out.extend(TECHS.iter().copied().filter(|t| sold.contains(t)));
        out
    }

    /// Wanted techs held by another living faction.
    pub fn sold(world: View, me: View) -> Vec<&'static str> {
        let me_id = me.get("id").num();
        let mut out: Vec<&'static str> = Vec::new();
        for f in world.get("factions").iter() {
            if !f.get("alive").truthy() || f.get("id").num() == me_id {
                continue;
            }
            for t in f.get("techs").iter() {
                if let Some(&w) = TECHS.iter().find(|w| **w == t.str()) {
                    if !out.contains(&w) {
                        out.push(w);
                    }
                }
            }
        }
        out
    }

    /// How long a tech we offered to buy waits before we offer again (a seller that hasn't answered holds our ◆ for up
    /// to 300 s, so the next offer goes elsewhere or waits).
    const TECH_RETRY: f64 = 150.0;

    /// The first tech we'd buy now: one we want and lack, whose prerequisites we hold, that another faction holds, and
    /// that we haven't offered to buy in the last TECH_RETRY seconds (the clock's "b<index>").
    fn wanted(world: View, me: View, sold: &[&'static str], clock: &Clock, now: f64) -> Option<&'static str> {
        let mine = me.get("techs");
        let all = world.get("techs");
        TECHS.iter().enumerate().find_map(|(i, &t)| {
            let x = all.get(t);
            let tried = clock.at.get(&format!("b{i}")).num_or(-1e9);
            (sold.contains(&t)
                && now - tried >= TECH_RETRY
                && !mine.get(t).truthy()
                && x.truthy()
                && x.get("requires").iter().all(|q| mine.get(q.str()).truthy()))
            .then_some(t)
        })
    }

    /// What the spawns should leave in the bank: a tech's price while one is for sale.
    pub fn saving(world: View, me: View, sold: &[&'static str], clock: &Clock, now: f64) -> f64 {
        if wanted(world, me, sold, clock, now).is_some() {
            TECH_BUY + 10.0
        } else {
            0.0
        }
    }

    /// The ◆ the market may spend this second. `first`: for what comes before a unit (a tech bought for the price of a
    /// unit saves minutes of research, and free land bought is land): the bank less this second's bribes and the ◆ held
    /// by deals sent, and a few ◆ to spare. Otherwise this second's spawns come first too, and a reserve: half a unit's
    /// price, and a minute of the streams we pay.
    fn purse(orders: &Value, me: View, first: bool) -> f64 {
        let unit = me.get("costs").get("unit").num();
        let mut left = me.get("bank").num();
        for b in orders.get("bribe").items() {
            left -= b.get("coin").num_or(0.0);
        }
        for s in orders.get("send").items() {
            left -= s.get("give").get("coin").num_or(0.0);
        }
        let streams: f64 = running(me).iter().map(|c| c.get("give").get("coinPerSecond").or(0.0)).sum();
        if first {
            return left - 10.0 - streams * 20.0;
        }
        left - orders.get("spawn").len() as f64 * (unit + 4.0) - unit * 0.5 - streams * 60.0
    }

    /// Our contracts still running.
    fn running(me: View) -> Vec<View> {
        me.get("contracts").iter().filter(|c| !c.get("ended").truthy()).collect()
    }

    /// Whether a running contract with faction `f` (any, for -1) gives (`side` "give") or gets ("get") `good`.
    fn holds(me: View, f: f64, side: &str, good: &str) -> bool {
        running(me).iter().any(|c| (f < 0.0 || c.get("with").num() == f) && c.get(side).get(good).truthy())
    }

    /// The factions whose land touches ours, by land or a crossing, each with how many of its provinces do: those
    /// who could march on us.
    fn neighbours(world: View, me: View) -> Vec<(f64, f64)> {
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        let mut out: Vec<(f64, f64)> = Vec::new();
        let mut seen: Vec<f64> = Vec::new();
        for p in world.get("provinces").iter() {
            if p.get("owner").num() != me_id {
                continue;
            }
            let routes = p.get("routes");
            for a in p.get("adj").iter().chain(routes.get("sea").iter()).chain(routes.get("pass").iter()) {
                let id = a.num();
                if seen.contains(&id) {
                    continue;
                }
                seen.push(id);
                let o = by_id.get(id).get("owner").num_or(-1.0);
                if o < 0.0 || o == me_id {
                    continue;
                }
                match out.iter_mut().find(|e| e.0 == o) {
                    Some(e) => e.1 += 1.0,
                    None => out.push((o, 1.0)),
                }
            }
        }
        out
    }

    /// A faction's entry in `world.factions`.
    fn faction(world: View, f: f64) -> View {
        world.get("factions").key(f)
    }

    /// A faction's name, for the memo.
    fn called(world: View, f: f64) -> String {
        faction(world, f).get("name").str().to_string()
    }

    /// A tech's name, for the memo.
    fn tech_name(world: View, t: &str) -> String {
        let x = world.get("techs").get(t);
        if x.truthy() && x.get("name").is_str() {
            x.get("name").str().to_string()
        } else {
            t.to_string()
        }
    }

    /// The factions with units in our land (not partners'), the ones at war with us now.
    fn invaders(world: View, me: View) -> Vec<f64> {
        let me_id = me.get("id").num();
        let mut out = Vec::new();
        for p in world.get("provinces").iter() {
            if p.get("owner").num() != me_id || !p.get("units").truthy() {
                continue;
            }
            for (f, k) in p.get("units").entries() {
                let f: f64 = f.parse().unwrap_or(f64::NAN);
                if f != me_id && k.num() > 0.0 && !realm::is_ally(me, f) && !out.contains(&f) {
                    out.push(f);
                }
            }
        }
        out
    }

    /// The factions with units in sight capturing or raiding in our land or next to it: an army on its way to us (its
    /// passage would end the moment it bought it).
    fn marching(world: View, me: View) -> Vec<f64> {
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        let ours = |id: f64| {
            let q = by_id.get(id);
            q.get("owner").num() == me_id || q.get("adj").iter().any(|a| by_id.at(a).get("owner").num() == me_id)
        };
        let mut out = Vec::new();
        for u in world.get("units").iter() {
            let owner = u.get("owner").num();
            if owner == me_id || out.contains(&owner) || !matches!(u.get("doing").str(), "capture" | "raid") {
                continue;
            }
            if ours(u.get("at").num()) || ours(u.get("to").num_or(-1.0)) {
                out.push(owner);
            }
        }
        out
    }

    /// One second of the market: whatever falls due, then a line for the memo.
    pub fn run(orders: &mut Value, world: View, me: View, clock: &mut Clock, now: f64, mood: &Mood, sold: &[&'static str]) {
        let mut doing: Option<String> = None;
        remember(clock, me, now);
        // tolls stand until changed: posted again as our border moves
        if due(clock, "tl", 25.0, now) {
            tolls(orders, world, me);
        }
        if due(clock, "pa", 12.0, now) {
            doing = passage(orders, world, me, clock, now).or(doing);
        }
        // Buy an affordable tech every 12–20 seconds.
        if clock.at.get("bt").num_or(-1.0) <= now {
            let mut p = purse(orders, me, true);
            if let Some(d) = buy_tech(orders, world, me, &mut p, clock, now, sold) {
                clock.at.set("bt", now + 12.0 + rand_int(9.0));
                clock.changed = true;
                doing = Some(d);
            }
        }
        if due(clock, "st", 30.0, now) {
            doing = sell_tech(orders, world, me).or(doing);
        }
        if due(clock, "fr", 10.0, now) {
            let mut p = purse(orders, me, true);
            doing = free_land(orders, world, me, &mut p).or(doing);
        }
        if due(clock, "br", 20.0, now) {
            let mut p = purse(orders, me, false);
            doing = bribe(orders, world, me, &mut p).or(doing);
        }
        if due(clock, "lg", 10.0, now) {
            let mut p = purse(orders, me, true);
            largesse(orders, world, me, &mut p);
        }
        if me.get("techs").get("influence").truthy() && common::every(now, 8.0, me.get("id").num()) {
            propaganda(orders, world, me);
        }
        if due(clock, "mi", 70.0, now) && !mood.threatened {
            // A lease needs cash for its first payments.
            let mut p = purse(orders, me, true);
            doing = lease_in(orders, world, me, &mut p, mood.target).or(doing);
        }
        if due(clock, "mo", 60.0, now) {
            doing = lease_out(orders, world, me).or(doing);
        }
        if due(clock, "ho", 150.0, now) && !mood.fight && mood.shade_ready {
            doing = hire_out(orders, world, me).or(doing);
        }
        // soldiers hired in: when there's a fight, or whenever the purse runs deep (two hires at a time then)
        let spare = purse(orders, me, false);
        let hires = running(me).iter().filter(|c| c.get("get").get("hire").truthy()).count();
        let want = if spare >= 500.0 { 2 } else if mood.fight && purse(orders, me, mood.threatened) >= 170.0 { 1 } else { 0 };
        if hires < want && due(clock, "hi", 90.0, now) {
            doing = hire_in(orders, world, me, if spare >= 900.0 { 6.0 } else { 4.0 }).or(doing);
        }
        if due(clock, "ai", 60.0, now) {
            let mut p = purse(orders, me, true);
            doing = aid(orders, world, me, &mut p).or(doing);
        }
        if due(clock, "si", 200.0, now) {
            let mut p = purse(orders, me, false);
            doing = buy_sight(orders, world, me, &mut p, mood.target).or(doing);
        }
        if due(clock, "ld", 90.0, now) {
            let mut p = purse(orders, me, false);
            doing = buy_land(orders, world, me, &mut p).or(doing);
        }
        memo(orders, me, doing, mood);
        // the ledger in the console every five minutes, for whoever reads it
        if now % 300.0 == 0.0 && now > 0.0 {
            let t = me.get("tolls");
            log(format!(
                "Ledger at {} min: {} contracts running, {}◆ in hand, {}◆ earned in tolls, {}◆ paid",
                text(now / 60.0),
                running(me).len(),
                text(me.get("bank").num().floor()),
                text(t.get("earned").or(0.0)),
                text(t.get("paid").or(0.0))
            ));
        }
    }

    /// Grudges: a faction that broke a deal with us by attacking (its contract stays listed through the grace) is sold
    /// no passage for five minutes. Kept in the clock as "x<id>": the second the grudge ends.
    fn remember(clock: &mut Clock, me: View, now: f64) {
        let me_id = me.get("id").num();
        for c in me.get("contracts").iter() {
            let e = c.get("ended");
            if !e.truthy() || e.get("by").num() == me_id || e.get("why").str() != "broke its deal by attacking" {
                continue;
            }
            let key = format!("x{}", text(e.get("by").num()));
            let until = e.get("t").num() + 300.0;
            if clock.at.get(&key).num_or(-1.0) < until {
                clock.at.set(&key, until);
                clock.changed = true;
            }
        }
        // forget the grudges that are over, and the offers to buy a tech that have waited their time
        let over: Vec<String> = match &clock.at {
            Value::Obj(o) => o
                .keys()
                .filter(|k| {
                    let v = o.get(k).map_or(0.0, |v| v.num_or(0.0));
                    (k.starts_with('x') && v < now) || (k.starts_with('b') && *k != "br" && now - v >= TECH_RETRY)
                })
                .map(str::to_string)
                .collect(),
            _ => Vec::new(),
        };
        if let Value::Obj(o) = &mut clock.at {
            for k in &over {
                o.remove(k);
                clock.changed = true;
            }
        }
    }

    /// The memo: the market's ledger, and the deal just made or the army's march.
    fn memo(orders: &mut Value, me: View, doing: Option<String>, mood: &Mood) {
        let deals = running(me);
        let passages = deals.iter().filter(|c| c.get("give").get("passage").truthy()).count();
        let tolls = me.get("tolls").get("earned").or(0.0);
        let what = doing.unwrap_or_else(|| if mood.threatened { "the guard turns out".to_string() } else { format!("the army marches on {}", mood.army) });
        let ledger = format!("{} contracts, {passages} passages, {}◆ in tolls", deals.len(), text(round(tolls)));
        let line = format!("Kirkcaldy: {what}. Ledger: {ledger}");
        // (a memo holds 90 characters: the ledger goes first when the line is too long)
        orders.set("memo", if line.chars().count() <= 90 { line } else { format!("Kirkcaldy: {what}").chars().take(90).collect() });
    }

    /// Tolls on every province of ours on the border (land not ours or a partner's next to it) or with a crossing.
    fn tolls(orders: &mut Value, world: View, me: View) {
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        orders.set("toll", Value::arr());
        let mut n = 0;
        for p in world.get("provinces").iter() {
            if p.get("owner").num() != me_id {
                continue;
            }
            let routes = p.get("routes");
            let crossing = !routes.get("sea").is_empty() || !routes.get("pass").is_empty();
            let border = p.get("adj").iter().any(|a| {
                let o = by_id.at(a).get("owner").num_or(-1.0);
                o != me_id && !(o >= 0.0 && realm::is_ally(me, o))
            });
            if crossing || border {
                trade::toll(orders, p.get("id").num(), if crossing { TOLL_CROSSING } else { TOLL });
                n += 1;
                if n >= 64 {
                    break;
                }
            }
        }
    }

    /// Passage through our land, sold to a neighbour that holds none from us (or less than a minute and a quarter of
    /// it): ten minutes for up to 24 of its units, for a little ◆. A faction holding passage through a land leaves it off
    /// its targets, so the neighbours that buy keep the peace with us. The neighbour with the longest border first, with a
    /// little chance in close calls; not one besieging our land (it would turn it down), nor one with units in it.
    fn passage(orders: &mut Value, world: View, me: View, clock: &Clock, now: f64) -> Option<String> {
        let me_id = me.get("id").num();
        let deals = running(me);
        // not to a faction in our land now (its units there may be on their way to take it: the deal would end at once),
        // nor to one that broke a deal with us lately (remember)
        let mut foes = invaders(world, me);
        foes.extend(marching(world, me));
        let grudge = |f: f64| clock.at.get(&format!("x{}", text(f))).num_or(-1.0) >= now;
        let held = |f: f64| {
            deals.iter().any(|c| c.get("with").num() == f && c.get("give").get("passage").truthy() && c.get("endsIn").or(0.0) > 75.0)
        };
        let mut sieges: Vec<f64> = Vec::new();
        for p in world.get("provinces").iter() {
            if p.get("owner").num() == me_id && p.get("siege").truthy() {
                sieges.push(p.get("siege").get("by").num());
            }
        }
        let mut best: Option<f64> = None;
        let mut score = 0.0;
        for (f, border) in neighbours(world, me) {
            if !faction(world, f).get("alive").truthy() || realm::is_ally(me, f) || held(f) || sieges.contains(&f) || foes.contains(&f) || grudge(f) {
                continue;
            }
            let s = border * (0.7 + 0.6 * rand());
            if s > score {
                score = s;
                best = Some(f);
            }
        }
        let f = best?;
        let secs = 600.0 + 60.0 * rand_int(3.0);
        let coin = 30.0 + rand_int(15.0);
        trade::passage(orders, f, 24.0, secs, obj! { "coin" => coin }, "passage through our markets, for peace and trade");
        Some(format!("passage sold to {}", called(world, f)))
    }

    /// A tech we want and can take (its prerequisites held), bought from a faction that holds it (`wanted`): a partner
    /// first, else the smaller seller, with a little chance in close calls. One at a time.
    fn buy_tech(orders: &mut Value, world: View, me: View, purse: &mut f64, clock: &mut Clock, now: f64, sold: &[&'static str]) -> Option<String> {
        if *purse < TECH_BUY {
            return None;
        }
        let t = wanted(world, me, sold, clock, now)?;
        let me_id = me.get("id").num();
        let mut best: Option<View> = None;
        let mut score = -1e9;
        for f in world.get("factions").iter() {
            if !f.get("alive").truthy() || f.get("id").num() == me_id || !f.get("techs").has_str(t) {
                continue;
            }
            let s = if realm::is_ally(me, f.get("id").num()) { 2.0 } else { 0.0 } - f.get("share").or(0.0) + 0.3 * rand();
            if s > score {
                score = s;
                best = Some(f);
            }
        }
        let f = best?;
        let name = tech_name(world, t);
        diplomacy::send(
            orders,
            f.get("id").value(),
            format!("{name} for {}◆, paid on delivery", text(TECH_BUY)),
            obj! { "give" => obj! { "coin" => TECH_BUY }, "ask" => obj! { "techs" => vec![t] } },
        );
        *purse -= TECH_BUY;
        if let Some(i) = TECHS.iter().position(|&x| x == t) {
            clock.at.set(&format!("b{i}"), now);
            clock.changed = true;
        }
        Some(format!("buying {name} from {}", f.get("name").str()))
    }

    /// One of our techs for a faction that lacks it and could take it (its prerequisites held): not the leader, not one
    /// in our land, nor one under 2% of the world. A first-tier tech is sold for good; a dearer one leased for ten
    /// minutes, paid by the second.
    fn sell_tech(orders: &mut Value, world: View, me: View) -> Option<String> {
        let me_id = me.get("id").num();
        let lead = realm::leader(world).map(|l| l.get("id").num()).filter(|&l| l != me_id);
        let foes = invaders(world, me);
        let all = world.get("techs");
        let ours: Vec<&'static str> = me.get("techs").keys().collect();
        let mut best: Option<(View, &str)> = None;
        let mut score = 0.0;
        for f in world.get("factions").iter() {
            let id = f.get("id").num();
            if !f.get("alive").truthy() || id == me_id || lead == Some(id) || foes.contains(&id) || f.get("share").or(0.0) < 0.02 {
                continue;
            }
            let theirs = f.get("techs");
            for &t in &ours {
                let x = all.get(t);
                if !x.truthy() || theirs.has_str(t) || !x.get("requires").iter().all(|q| theirs.has_str(q.str())) {
                    continue;
                }
                let s = (if realm::is_ally(me, id) { 2.0 } else { 1.0 }) * (0.5 + rand());
                if s > score {
                    score = s;
                    best = Some((f, t));
                }
            }
        }
        let (f, t) = best?;
        let name = tech_name(world, t);
        let id = f.get("id").num();
        if all.get(t).get("tier").or(1.0) <= 1.0 {
            trade::sell_tech(orders, id, t, obj! { "coin" => TECH_SELL }, 0.0, format!("{name}, yours for good"));
            Some(format!("selling {name} to {}", f.get("name").str()))
        } else {
            trade::sell_tech(orders, id, t, obj! { "coinPerSecond" => 0.24, "seconds" => 600 }, 600.0, format!("{name} on lease, ten minutes"));
            Some(format!("leasing {name} to {}", f.get("name").str()))
        }
    }

    /// Free land on our border, won over with ◆: favour of 150 or more makes it likelier to join us each second
    /// (`rules.sway.favourFree`), so a bribe that takes ours there to 170 buys it in a minute or so. The one with the most
    /// cores.
    fn free_land(orders: &mut Value, world: View, me: View, purse: &mut f64) -> Option<String> {
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        let mut best: Option<View> = None;
        for p in world.get("provinces").iter() {
            if !p.get("visible").truthy() || !p.get("owner").is_num() || p.get("owner").num() >= 0.0 {
                continue;
            }
            if sway::favour(me, p.get("id").num()) >= 160.0 || !p.get("adj").iter().any(|a| by_id.at(a).get("owner").num() == me_id) {
                continue;
            }
            if best.is_none_or(|b| p.get("cores").or(0.0) > b.get("cores").or(0.0)) {
                best = Some(p);
            }
        }
        let p = best?;
        let coin = (170.0 - sway::favour(me, p.get("id").num())).ceil();
        if *purse < coin {
            return None;
        }
        sway::bribe(orders, p.get("id").num(), coin);
        *purse -= coin;
        Some(format!("buying the favour of {}", p.get("name").str()))
    }

    /// A rival's restive or uneasy province on our border (common::sway_targets), two when the purse allows: a bribe,
    /// so when it revolts it joins us. Not one with three or more of its owner's units in it: they'd take it straight
    /// back.
    fn bribe(orders: &mut Value, world: View, me: View, purse: &mut f64) -> Option<String> {
        let mut said = None;
        let targets = common::sway_targets(world, me);
        for t in targets.into_iter().filter(|q| matches!(q.get("loyaltyBand").str(), "restive" | "uneasy") && q.get("garrison").or(0.0) < 3.0).take(2) {
            let coin = min(200.0, *purse).floor();
            if coin < 60.0 {
                break;
            }
            sway::bribe(orders, t.get("id").num(), coin);
            *purse -= coin;
            said = said.or_else(|| Some(format!("paying {}'s malcontents", t.get("name").str())));
        }
        said
    }

    /// Largesse (60◆ each) for our three most restless provinces, so land we hold doesn't revolt away.
    fn largesse(orders: &mut Value, world: View, me: View, purse: &mut f64) {
        let me_id = me.get("id").num();
        let mut restless: Vec<(f64, f64)> = Vec::new();
        for p in world.get("provinces").iter() {
            let risk = p.get("risk").or(0.0);
            if p.get("owner").num() == me_id && risk > 0.03 {
                restless.push((risk, p.get("id").num()));
            }
        }
        restless.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, id) in restless.into_iter().take(3) {
            if *purse < 60.0 {
                break;
            }
            sway::bribe(orders, id, 60.0);
            *purse -= 60.0;
        }
    }

    /// Propaganda, a standing order worked out again every 9 seconds: in the four rival provinces on our border likeliest
    /// to turn, where it makes our bribes go a quarter further.
    fn propaganda(orders: &mut Value, world: View, me: View) {
        let income = me.get("rts").get("cpu").get("income").or(me.get("compute").get("cycles").or(14000.0));
        let each = round(income * 0.04);
        orders.set("propaganda", Value::arr());
        for t in common::sway_targets(world, me).iter().take(4) {
            sway::propaganda(orders, t.get("id").num(), each);
        }
    }

    /// Mines leased in: up to two slots of a neighbour's province on our border for ten minutes, paid by the second at a
    /// tenth over what the house programs think fair. Our miners work them (they're in realm::open_mines), and the
    /// cycles are ours. At most two leases at a time; never in the army's target (a lessee can't capture there).
    fn lease_in(orders: &mut Value, world: View, me: View, purse: &mut f64, target: f64) -> Option<String> {
        if *purse < 40.0 || running(me).iter().filter(|c| c.get("get").get("mines").truthy()).count() >= 2 {
            return None;
        }
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        let mut best: Option<(View, f64)> = None;
        let mut score = 0.0;
        for p in world.get("provinces").iter() {
            let o = p.get("owner").num_or(-1.0);
            let id = p.get("id").num();
            if !p.get("visible").truthy() || o < 0.0 || o == me_id || id == target || p.get("siege").truthy() {
                continue;
            }
            let f = faction(world, o);
            if !f.get("alive").truthy() || f.get("shielded").truthy() || !p.get("adj").iter().any(|a| by_id.at(a).get("owner").num() == me_id) {
                continue;
            }
            let mines = p.get("mines");
            let free = mines.len() as f64 - p.get("leased").len() as f64;
            if free < 1.0 {
                continue;
            }
            let mut cores: Vec<f64> = mines.iter().map(|m| m.or(0.0)).collect();
            cores.sort_by(|a, b| b.total_cmp(a));
            let s = cores.iter().take(2).sum::<f64>() * (0.8 + 0.4 * rand());
            if s > score {
                score = s;
                best = Some((p, min(2.0, free)));
            }
        }
        let (p, slots) = best?;
        let pay = (slots * 0.1834 * 100.0).ceil() / 100.0;
        diplomacy::send(
            orders,
            p.get("owner").value(),
            format!("{} mines in {} for ten minutes, paid by the second", text(slots), p.get("name").str()),
            obj! {
                "give" => obj! { "coinPerSecond" => pay, "seconds" => 600 },
                "ask" => obj! { "mines" => obj! { "province" => p.get("id").num(), "slots" => slots, "seconds" => 600 } },
            },
        );
        *purse -= pay * 60.0;
        Some(format!("leasing mines in {}", p.get("name").str()))
    }

    /// Mines leased out: each idle slot is productive for a neighbour, up to two at a time for five minutes, paid by
    /// the second. Partners go first, then the smallest neighbour.
    fn lease_out(orders: &mut Value, world: View, me: View) -> Option<String> {
        let idle = trade::idle_mines(world, me);
        if idle.iter().map(|m| m.slots).sum::<f64>() < 1.0 || running(me).iter().filter(|c| c.get("give").get("mines").truthy()).count() >= 2 {
            return None;
        }
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        let lead = realm::leader(world).map(|l| l.get("id").num());
        for m in idle {
            let q = by_id.get(m.province);
            let mut best: Option<View> = None;
            let mut score = -1e9;
            for a in q.get("adj").iter() {
                let o = by_id.at(a).get("owner").num_or(-1.0);
                let f = if o >= 0.0 && o != me_id { faction(world, o) } else { View::undefined() };
                if !f.truthy() || !f.get("alive").truthy() || lead == Some(o) || f.get("share").or(0.0) < 0.03 {
                    continue;
                }
                let s = if realm::is_ally(me, o) { 1.0 } else { 0.0 } - f.get("share").or(0.0) + 0.02 * rand();
                if s > score {
                    score = s;
                    best = Some(f);
                }
            }
            let Some(f) = best else { continue };
            let slots = min(2.0, m.slots);
            let ask = obj! { "coinPerSecond" => (slots * 0.16 * 100.0).floor() / 100.0, "seconds" => 300 };
            trade::lease_mines(orders, f.get("id").num(), m.province, slots, 300.0, ask, format!("idle seams in {}, five minutes", q.get("name").str()));
            return Some(format!("leasing seams in {} to {}", q.get("name").str(), f.get("name").str()));
        }
        None
    }

    /// Our spare militia on hire: once the shades can be built, four runners for five minutes to a partner, else to the
    /// smallest faction that borders the leader (not the leader, nor one with a fifth of the world): the pin-makers
    /// earn their keep, and the hirer pays their upkeep.
    fn hire_out(orders: &mut Value, world: View, me: View) -> Option<String> {
        if holds(me, -1.0, "give", "hire") {
            return None;
        }
        let me_id = me.get("id").num();
        let spare: Vec<Value> = world
            .get("units")
            .iter()
            .filter(|u| u.get("owner").num() == me_id && u.get("type").str() == "runner" && !u.get("hired").truthy() && u.get("doing").str() != "fight")
            .take(4)
            .map(|u| u.get("id").value())
            .collect();
        if spare.len() < 4 {
            return None;
        }
        let lead = realm::leader(world).map(|l| l.get("id").num()).filter(|&l| l != me_id);
        let by_id = realm::index(world);
        let mut near: Vec<f64> = Vec::new();
        if let Some(l) = lead {
            for p in world.get("provinces").iter() {
                if p.get("owner").num() != l {
                    continue;
                }
                for a in p.get("adj").iter() {
                    let o = by_id.at(a).get("owner").num_or(-1.0);
                    if o >= 0.0 && o != l && o != me_id && !near.contains(&o) {
                        near.push(o);
                    }
                }
            }
        }
        let mut best: Option<View> = None;
        let mut score = -1e9;
        for f in world.get("factions").iter() {
            let id = f.get("id").num();
            let share = f.get("share").or(0.0);
            let ally = realm::is_ally(me, id);
            if !f.get("alive").truthy() || id == me_id || lead == Some(id) || share >= 0.2 || share < 0.02 || !(ally || near.contains(&id)) {
                continue;
            }
            let s = if ally { 1.0 } else { 0.0 } - share;
            if s > score {
                score = s;
                best = Some(f);
            }
        }
        let f = best?;
        trade::hire(orders, f.get("id").num(), spare, 300.0, obj! { "coin" => 115 }, "four of Kirkcaldy's militia for five minutes");
        Some(format!("hiring militia out to {}", f.get("name").str()))
    }

    /// Soldiers hired in: `count` for five minutes from the faction with the most units in sight that isn't the leader,
    /// isn't in our land and holds less than a fifth of the world (partners and neighbours count more, being nearer),
    /// paid a quarter over the house programs' price. They take our orders (retrained as our types) and march with our
    /// army.
    fn hire_in(orders: &mut Value, world: View, me: View, count: f64) -> Option<String> {
        let me_id = me.get("id").num();
        let lead = realm::leader(world).map(|l| l.get("id").num());
        let foes = invaders(world, me);
        let near = neighbours(world, me);
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
        let mut best: Option<View> = None;
        let mut score = 0.0;
        for f in world.get("factions").iter() {
            let id = f.get("id").num();
            let count = n.iter().find(|e| e.0 == id).map_or(0.0, |e| e.1);
            if !f.get("alive").truthy() || id == me_id || lead == Some(id) || foes.contains(&id) || f.get("share").or(0.0) >= 0.2 || count < 8.0 {
                continue;
            }
            let s = count
                * if realm::is_ally(me, id) { 2.0 } else { 1.0 }
                * if near.iter().any(|e| e.0 == id) { 1.5 } else { 1.0 }
                * (0.8 + 0.4 * rand());
            if s > score {
                score = s;
                best = Some(f);
            }
        }
        let f = best?;
        // (the house programs price a hire at a tenth of a ◆ a unit a second: we pay a quarter more)
        let pay = (count * 300.0 / 10.0 * 1.25).round();
        trade::hire_from(orders, f.get("id").num(), count, 300.0, obj! { "coin" => pay }, format!("{} soldiers for five minutes, paid in coin", text(count)));
        Some(format!("hiring soldiers from {}", f.get("name").str()))
    }

    /// A subsidy for a partner at war (rivals' units in its land): half a ◆ a second for four minutes, one at a time.
    fn aid(orders: &mut Value, world: View, me: View, purse: &mut f64) -> Option<String> {
        if me.get("pacts").is_empty() || *purse < 120.0 {
            return None;
        }
        let me_id = me.get("id").num();
        let mut at_war = -1.0;
        for p in world.get("provinces").iter() {
            let owner = p.get("owner").num();
            if owner < 0.0 || owner.is_nan() || owner == me_id || !realm::is_ally(me, owner) || !p.get("units").truthy() {
                continue;
            }
            if p.get("units").entries().any(|(f, _)| {
                let f: f64 = f.parse().unwrap_or(f64::NAN);
                f != owner && f != me_id && !realm::is_ally(me, f)
            }) {
                at_war = owner;
                break;
            }
        }
        if at_war < 0.0 || holds(me, at_war, "give", "coinPerSecond") {
            return None;
        }
        diplomacy::send(orders, at_war, "a subsidy for the war", obj! { "give" => obj! { "coinPerSecond" => 0.5, "seconds" => 240 } });
        *purse -= 30.0;
        Some(format!("subsidising {}'s war", called(world, at_war)))
    }

    /// Intelligence bought on the market: the sight of the faction that holds the army's target, for five minutes.
    fn buy_sight(orders: &mut Value, world: View, me: View, purse: &mut f64, target: f64) -> Option<String> {
        if target < 0.0 || *purse < 30.0 {
            return None;
        }
        let o = realm::index(world).get(target).get("owner").num_or(-1.0);
        if o < 0.0 || o == me.get("id").num() || !faction(world, o).get("alive").truthy() || holds(me, o, "get", "sight") {
            return None;
        }
        diplomacy::send(orders, o, "a look at your land, for coin", obj! { "give" => obj! { "coin" => 12 }, "ask" => obj! { "sight" => obj! { "seconds" => 300 } } });
        *purse -= 12.0;
        Some(format!("buying {}'s sight", called(world, o)))
    }

    /// Land bought outright once the bank is deep: a neighbour's province on our border at the house programs' price
    /// (common::price), the most cores for the ◆, never a capital, never a partner's, and only from a faction that keeps
    /// four or more provinces (the rule won't sell below that).
    fn buy_land(orders: &mut Value, world: View, me: View, purse: &mut f64) -> Option<String> {
        if *purse < 1200.0 {
            return None;
        }
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        if realm::leader(world).is_some_and(|l| l.get("id").num() == me_id) {
            return None;
        }
        let mut best: Option<(View, f64)> = None;
        let mut score = 0.0;
        for p in world.get("provinces").iter() {
            let o = p.get("owner").num_or(-1.0);
            if !p.get("visible").truthy() || o < 0.0 || o == me_id || realm::is_ally(me, o) || p.get("capital").truthy() || p.get("siege").truthy() {
                continue;
            }
            let f = faction(world, o);
            if !f.get("alive").truthy() || f.get("provinces").or(0.0) < 6.0 || !p.get("adj").iter().any(|a| by_id.at(a).get("owner").num() == me_id) {
                continue;
            }
            let price = common::price(world, f, &[p.get("id").num()], me_id);
            if !price.is_finite() || price > *purse {
                continue;
            }
            let s = (p.get("cores").or(0.0) + 0.5) / price;
            if s > score {
                score = s;
                best = Some((p, price));
            }
        }
        let (p, price) = best?;
        diplomacy::send(
            orders,
            p.get("owner").value(),
            format!("{}◆ for {}", text(price + 1.0), p.get("name").str()),
            obj! { "give" => obj! { "coin" => price + 1.0 }, "ask" => obj! { "provinces" => vec![p.get("id").value()] } },
        );
        *purse -= price + 1.0;
        Some(format!("buying {} outright", p.get("name").str()))
    }

    /// Answers the inbox as common::answer does (deals the house rule calls fair, a pact when `pact` says yes and nobody
    /// in it is in one yet), but counts the ◆ this second's sent deals hold, and lends no units on hire while invaded.
    pub fn answer(orders: &mut Value, me: View, world: View, pact: impl Fn(View) -> bool, calm: bool) {
        let me_id = me.get("id").num();
        let mut joined = !me.get("pacts").is_empty();
        let free = |f: View| world.get("factions").key(f.num()).get("pacts").is_empty();
        let mu = me.get("units");
        let mut coin = me.get("bank").num();
        for s in orders.get("send").items() {
            coin -= s.get("give").get("coin").num_or(0.0);
        }
        let limit = if mu.get("max").truthy() { mu.get("max").num() } else { mu.get("cap").num() };
        let mut room = limit - mu.get("total").or(0.0) - mu.get("queued").or(0.0) - mu.get("hiredOut").or(0.0);
        for m in me.get("inbox").iter() {
            if m.get("accepted").has_num(me_id) {
                continue;
            }
            let yes = if m.get("pact").truthy() {
                let ok = !joined && m.get("pact").get("members").iter().all(free) && pact(m);
                joined |= ok;
                ok
            } else if m.get("give").truthy() || m.get("ask").truthy() {
                let (give, ask) = (m.get("give"), m.get("ask"));
                let pay = ask.get("coin").or(0.0) + ask.get("coinPerSecond").or(0.0) * ask.get("seconds").or(1.0);
                let hire = give.get("hire");
                let units = give.get("units").len() as f64 + if hire.get("units").truthy() { hire.get("units").len() as f64 } else { hire.get("count").or(0.0) };
                let ok = pay <= coin && units <= room.max(0.0) && (calm || !ask.get("hire").truthy()) && common::fair(m, me, world, &TECHS);
                if ok {
                    coin -= pay;
                    room -= units;
                }
                ok
            } else {
                true
            };
            orders.entry(if yes { "accept" } else { "reject" }).push(m.get("id").value());
        }
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

fn carry_order(o: Option<Value>, u: View, pl: &std::rc::Rc<Value>, me: View) -> Option<Value> {
    let Some(ops) = common::carry(u, pl, me) else { return o };
    let mut o = o.unwrap_or_else(|| obj! { "id" => u.get("id") });
    o.set("ram", ops);
    Some(o)
}

fn infantry_plan(units: View, me: View) -> Option<std::rc::Rc<Value>> {
    if !rts(me) {return Some(plan::for_units(me,units,View::undefined()));}
    let mut core=None;
    for u in units.iter() {
        let home=plan::here(me,u);
        if !home&&core.is_some(){continue;}
        let bytes=me.get("ramProvince").key(u.get("at").num()).key(0.0).get("bytes");
        let read=if plan::known(me,u).is_none()&&bytes.is_bytes()&&bytes.len()>=2 {
            let b=bytes.bytes();4.0*f64::from(u16::from_le_bytes([b[0],b[1]]))
        }else{0.0};
        if cycles_left()<read+2200.0{return None;}
        let pl=plan::of(me,u);
        if home&&(!pl.get("attack").is_undef()||pl.get("mines").truthy()||!pl.get("raid").is_undef()){return Some(pl);}
        if !home&&!pl.get("attack").is_undef(){core=Some(pl);}
    }
    Some(core.unwrap_or_else(||std::rc::Rc::new(Value::obj())))
}

// Cold plans must leave room for an order; standing orders survive a deferred lookup.
fn infantry(units: View, world: View, me: View, cloak: bool) -> Value {
    if short(me, 3000.0, 1200.0) {
        return Value::arr();
    }
    let Some(pl)=infantry_plan(units,me) else {return Value::arr();};
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        let u = nth(units, turn, world);
        // those the plan sends to defend (common::defends), the rest on the target
        if common::received_ship(u,world,me) { continue; }
        let defend = Value::Num(common::defends(&pl, u));
        let goal = if num(&defend) >= 0.0 { &defend } else { pl.get("attack") };
        let verb = if num(&defend) >= 0.0 || u.get("air").truthy() || u.path("build.wings").truthy() { "fight" } else { "capture" };
        // a beaten army falls back to the stage, and reinforcements wait there for a wave (common::front)
        if verb == "capture" {
            if let Some(o) = common::front(u, world, me, &pl) {
                if let Some(o) = carry_order(o, u, &pl, me) {
                    order(o);
                }
                continue;
            }
        }
        // A route is expensive code.  The engine already retains an accepted
        // order, so recalculate only when the strategic destination changes.
        let doing = u.get("doing").str();
        if (doing == verb || doing == "move") && same(u.get("target"), goal) {
            if let Some(o) = carry_order(None, u, &pl, me) { order(o); }
            continue;
        }
        let o = if num(goal) >= 0.0 { orders::to(u, verb, num(goal), world, me, ()) } else { None };
        if let Some(mut o) = carry_order(o, u, &pl, me) {
            if cloak {
                o.set("cloak", true);
            }
            o.set("power", obj! { "drive" => 120, "armor" => 0, "weapon" => 260 });
            order(o);
        } else if cloak {
            order(obj! { "id" => u.get("id"), "cloak" => true, "power" => obj! { "drive" => 0, "armor" => 0, "weapon" => 240 }, "alert" => 6 });
        }
    }
    Value::Undef
}


// Interrupt useful local generation; leave batteries and distant sites to the campaign.
fn energy_sabotage(u: View, world: View, me: View) -> Option<Option<Value>> {
    if cycles_left()<2800.0 || !world.get("energy").truthy() || !u.get("hidden").truthy()
        || !u.get("seenBy").is_empty() || u.get("stalled").truthy()
        || me.get("bank").num_or(0.0)<150.0 || u.get("to").num_or(-1.0)>=0.0 {return None;}
    let mut pick=None;let mut mw=2.0;
    let sites=world.get("structures");
    let start=(u.get("id").num_or(0.0)+world.get("seconds").num_or(0.0)/3.0) as usize;
    for turn in 0..sites.len().min(24) {
        if cycles_left()<2500.0 {break;}
        let s=sites.at((start+turn)%sites.len());
        let w=s.get("power");
        if s.get("p").num_or(-2.0)!=u.get("at").num_or(-1.0) || s.get("state").str()!="ready"
            || !realm::is_rival(world,me,s.get("owner").num_or(-1.0))
            || w.get("disruptedFor").num_or(0.0)>0.0 || w.get("disruptionCooldown").num_or(0.0)>0.0
            || !w.get("ports").iter().any(|p|p.get("kind").str()!="battery") {continue;}
        let value=w.get("mw").num_or(0.0);
        if value>=mw {mw=value;pick=Some(s);}
    }
    let s=pick?;
    if let Some(o)=spy::sabotage_structure(u,world,me,s) {return Some(Some(o));}
    (u.get("doing").str()=="sabotage" && u.get("sabotageStructure").num_or(-1.0)==s.get("id").num()).then_some(None)
}

fn shade(units: View, world: View, me: View) {
    // A scaled shade batch needs room to decode its plan before infantry can retain each standing order.
    if short(me, 3000.0, 1200.0) { return; }
    if world.get("energy").truthy() && cycles_left()>4500.0 {
        for u in units.iter().take(8) {
            if cycles_left()<4000.0 {break;}
            let doing=u.get("doing").str();
            if doing!="idle" && !(matches!(doing,"capture"|"fight"|"sabotage") && u.get("target").num_or(-1.0)==u.get("at").num_or(-2.0)) {continue;}
            if let Some(hit)=energy_sabotage(u,world,me) {
                if let Some(mut hit)=hit {hit.set("cloak",true);order(hit);}
                return;
            }
        }
    }
    infantry(units, world, me, true);
}

// Where the kites go: the rival province in sight with the most miners at work (n, from seen_threats), else -1 (the
// plan's target).
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

// Kites defend at home, else raid the prey (a raid takes ◆, not land), else join the push on the target.
fn kite(units: View, world: View, me: View) {
    // a flier a minute up comes down on your nearest land before anything else, even short of cycles: its burn climbs
    // the longer it flies, and an unpaid one crashes
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
        // a kite on its way down (above) stays on it, and one down stays until it has cooled off
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
        // a call left with almost nothing stops here rather than run out (the rest come down next call)
        if cycles_left() < 150.0 {
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

struct SentryBudget {
    next: f64,
    read: Vec<(f64, f64)>,
}

fn sentry_has_runway(available: f64, next: f64, read: f64) -> bool {
    available >= next.max(2400.0) + read + 1800.0
}

impl SentryBudget {
    fn new(me: View) -> Self {
        if rts(me) {
            orders::set_reserve(1100.0);
            orders::set_first(1400.0);
        }
        Self { next: 2400.0, read: Vec::new() }
    }

    fn admits(&mut self, u: View, me: View) -> bool {
        if !rts(me) { return true; }
        let province = u.get("at").num();
        let page = me.get("ramProvince").key(province).key(0.0);
        let stamp = page.get("t").num_or(-1.0);
        let cold = page.truthy() && !self.read.contains(&(province, stamp));
        let bytes = page.get("bytes");
        let read = if cold && bytes.is_bytes() && bytes.len() >= 2 {
            let b = bytes.bytes();
            2.0 * f64::from(u16::from_le_bytes([b[0], b[1]]))
        } else { 0.0 };
        if !sentry_has_runway(cycles_left(), self.next, read) { return false; }
        if cold { self.read.push((province, stamp)); }
        true
    }

    fn spent(&mut self, cycles: f64) {
        // Include plan decoding, the route, core/front RAM copies and committing the order.
        self.next = self.next.max(cycles * 2.0 + 1200.0);
    }
}

fn sentry(units: View, world: View, me: View) -> Value {
    if short(me, 3500.0, 1600.0) {
        return Value::arr();
    }
    let by_id = realm::index(world);
    let mut budget = SentryBudget::new(me);
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 4200.0 { break; }
        let u = nth(units, turn, world);
        if !budget.admits(u, me) { break; }
        let began = cycles_left();
        let pl = plan::of(me, u);
        if common::received_ship(u,world,me) { continue; }
        let defend = common::defends(&pl, u);
        let attack = pl.get("attack").num_or(-1.0);
        let q = by_id.get(attack);
        let owner = q.get("owner").num_or(-1.0);
        // Armor and artillery follow a committed wave instead of staying behind after every capture.
        let committed = attack >= 0.0 && !realm::is_ally(me, owner)
            && (q.get("units").key(me.get("id").num()).or(0.0) >= 2.0
                || q.get("siege").get("by").num_or(-1.0) == me.get("id").num());
        let at = if defend >= 0.0 { defend } else if committed { attack }
            else { pl.get("stage").num_or(pl.get("home").num_or(u.get("at").num())) };
        let mut o = if defend < 0.0 && committed {
            match common::front(u, world, me, &pl) {
                Some(o) => o,
                None => orders::to(u, if u.get("type").str() == "spire" || owner == me.get("id").num() { "fight" } else { "capture" }, at, world, me, ()),
            }
        } else {
            orders::hold(u, Point::new(0.0, 0.0), world, me, Some(at))
        };
        if let Some(ops) = common::carry(u, &pl, me) {
            o.get_or_insert_with(|| obj! { "id" => u.get("id") }).set("ram", ops);
        }
        if let Some(mut o) = o {
            if rts(me) && cycles_left() < 600.0 { break; }
            o.set("power", obj! { "drive" => 120, "armor" => 0, "weapon" => 180 });
            o.set("alert", 7);
            order(o);
        }
        budget.spent(began - cycles_left());
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

// ---- reflex ----

fn dodge(u: View) -> Option<Value> {
    let hs = u.get("rts").get("hazards");
    let pos = u.get("local");
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

// Start a shot while moving out of melee or marked artillery range; focus uses visible nearby units.
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
    // Search the public damage/CPU curve. The planning itself also consumes CPU.
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
    bank: f64,
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
        if *left < cost || self.bank < cost {
            return false;
        }
        *left -= cost;
        self.bank -= cost;
        true
    }
}

// Pick a walkable local direction that keeps progress toward the requested step.
fn terrain_step(u: View, x: f64, y: f64) -> Option<Value> {
    let length = x.hypot(y);
    if length < 0.01 { return None; }
    let terrain = u.get("terrain");
    if terrain.is_empty() { return Some(obj! { "x" => x / length, "y" => y / length }); }
    let (mut best, mut score) = (None, -1e9);
    for t in terrain.iter().take(8) {
        let progress = (t.get("x").num() * x + t.get("y").num() * y) / length;
        let walk = t.get("walk").or(0.0);
        if walk <= 0.0 || progress < 0.1 { continue; }
        let value = progress * walk + (1.0 - t.get("cover").or(1.0)) * 0.2;
        if value > score {
            score = value;
            best = Some(obj! { "x" => t.get("x"), "y" => t.get("y") });
        }
    }
    best
}

fn reflex(units: View, near: View) -> Value {
    // Divide the pre-billed lane snapshot locally: affordable engagements reach all contacted units.
    let n = units.len();
    let b = if n > 0 { units.at(0).get("rts").get("budget") } else { View::undefined() };
    let level = if n > 0 { common::price_level(units.at(0)) } else { 1.0 };
    let mut pool = Pool {
        bank: max(0.0, units.at(0).get("rts").get("bank").or(0.0) - cycles_left() - 700.0),
        drive: max(0.0, b.get("drive").or(0.0) - 120.0 * level),
        armor: max(0.0, b.get("armor").or(0.0) - 80.0 * level),
        cloak: max(0.0, b.get("cloak").or(0.0) - 70.0 * level),
        weapon: max(0.0, b.get("weapon").or(0.0) - 240.0 * level),
        abilities: max(0.0, b.get("abilities").or(0.0) - 180.0 * level),
        level,
    };
    let count = min(n as f64, max(1.0, (b.get("reflex").or(0.0) / 850.0).floor())) as usize;
    let start = if n > 0 { ((units.at(0).get("rts").get("bank").or(0.0) / 175.0).floor() % n as f64) as usize } else { 0 };
    let mut turn_cost = 1800.0_f64;
    for turn in 0..count {
        if cycles_left() < turn_cost {
            break;
        }
        let began = cycles_left();
        let u = units.at((start + turn) % n);
        let ns = near.key(u.get("id").num());
        let r = u.get("rts");
        let modules = r.get("modules");
        let (mut best, mut melee): (Option<View>, Option<View>) = (None, None);
        let (mut best_score, mut rivals) = (1e9, 0);
        for t in ns.iter().take(12) {
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
            // Finish damaged targets, but also prefer exposed ranged and air threats
            // over a healthy nearby melee screen.
            let score = t.get("hp").or(1.0) + d * 0.12 - if t.get("range").truthy() { 0.32 } else { 0.0 } - if air { 0.20 } else { 0.0 };
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
        let flee = flee.and_then(|f| terrain_step(u, f.get("x").num(), f.get("y").num()));
        let drive_scale = r.get("driveScale").or(1.0);
        let driving = flee.is_some() && pool.take("drive", 164.0 * drive_scale);
        let armored = modules.has_str("armor") && (melee.is_some() || hurt) && pool.take("armor", 110.0);
        let penetration = pierce_mode(u, best);
        let armed = best.is_some() && r.get("readyIn").or(0.0) <= 0.0 && (r.get("windupTarget").is_null() || r.get("windupTarget").is_undef()) && pool.take("weapon", (540.0 * (1.0 + 1.4 * penetration * penetration)).ceil());
        let cloaking = ty == "shade" && best.is_none() && flee.is_none() && pool.take("cloak", 90.0);
        let mut o = obj! {
            "id" => u.get("id"),
            "guard" => guard_facing(ns),
            "pierce" => penetration,
            "power" => obj! { "drive" => if driving { 760 } else { 0 }, "armor" => if armored { 520 } else { 0 }, "weapon" => if armed { 420 } else { 0 } },
        };
        if let (true, Some(t)) = (armed, best) {
            o.set("hit", t.get("id"));
            // Circle a guarded line instead of trading into its strongest face. Unit parity splits the flanks.
            if flee.is_none() && t.get("rts").get("guard").truthy() {
                let side = if u.get("id").num() % 2.0 != 0.0 { 1.0 } else { -1.0 };
                if let Some(direction) = terrain_step(u, -t.get("dy").num() * side, t.get("dx").num() * side) {
                    if pool.take("drive", 100.0 * drive_scale) {
                        o.set("step", direction);
                        o.entry("power").set("drive", 440);
                    }
                }
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
        if driving && artillery { o.set("deploy", false); }
        if artillery && !driving && best.is_some() && !r.get("deployed").truthy() && !r.get("deploying").truthy() {
            o.set("deploy", true);
        }
        let shell = best.filter(|t| artillery && !driving && r.get("deployed").truthy()
            && t.get("d").num() > 1.8 && t.get("d").num() <= 14.0 && ready("shell")
            && !ns.iter().any(|f| !f.get("rival").truthy()
                && (f.get("dx").num()-t.get("dx").num()).hypot(f.get("dy").num()-t.get("dy").num()) < 1.8))
            .filter(|t| pool.take("abilities", (250.0 * (1.0 + max(0.0, t.get("d").num()-5.0)*0.12)).ceil()));
        if let Some(t) = shell {
            let pos = u.get("local");
            o.set("ability", "shell");
            o.set("radius", 1);
            o.set("aim", obj! { "x" => pos.get("x").num() + t.get("dx").num(), "y" => pos.get("y").num() + t.get("dy").num() });
        } else if driving && modules.has_str("dash") && ready("dash") && pool.take("abilities", 900.0) {
            o.set("ability", "dash");
        }
        // Scan only after contact has established that a fight is worth revealing;
        // this avoids paying its cooldown repeatedly while merely patrolling.
        if !o.get("ability").truthy() && rivals >= 2 && modules.has_str("scan") && ready("scan") && pool.take("abilities", 700.0) {
            o.set("ability", "scan");
        }
        if modules.has_str("cloak") { o.set("cloak", cloaking); }
        if armed || driving || armored || o.get("cloak").truthy() || o.get("ability").truthy() {
            order(o);
        }
        turn_cost = turn_cost.max((began - cycles_left()) * 1.5);
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
    theme::of(info.get("seat"), 0.0).map_or(Value::Null, |t| t.look(budget.num(), info))
}

fn build3d(budget: View, info: View) -> Value {
    theme::of(info.get("seat"), 0.0).map_or(Value::Null, |t| t.scene(info, budget.num()))
}

fn live3d(state: View, now: View) -> Value {
    let mut s = if state.truthy() { state.value() } else { Value::obj() };
    let spin = capital::ring(&mut s, now);
    obj! { "state" => s, "set" => obj! { "wonder" => spin } }
}
