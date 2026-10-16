// New Atlantis: Salomon's House. Science is the strategy, on the Bulwark Doctrine's combined-arms army.
//
// Research takes a share of income the army can live with: 30% while the faction is small, 40% once its income passes
// 150,000 a second, 50% while the bank holds cycles the army isn't spending, 20% while the bank is thin or invaders
// press. The research order serves the army first (Extraction, Armor, Dash, then Repair, Artillery and Scan whoever
// holds them), then Compute boost, Flight, Gate, Enhanced stealth and Air weapons, with a ready tier-2 tech no one
// holds taken on the way as a world first. The Merchants of Light sell and lease the techs of the economy, the map and
// the mind to the factions under half New Atlantis's share, allies first, never the runaway leader; the army's techs
// stay home. The army, bodies, look and capital are the Bulwark Doctrine's.
//
// Built on the SDK in the gameofcores crate (docs part sdk), with the strategy the founders share in the `common`
// module at the end: targets, spawning room, cycles and research, deals and pacts, scouting and the puzzle solver.

use std::rc::Rc;

use gameofcores::*;
use shared::{air, capital, common, theme};
use shared::common::{hauler, freighter};

program!(think, hauler, freighter, miner, line, bulwark, artillery, medic, interceptor, piercer, scout, aeronaut, reflex, research, duel, challenge, look, build3d, live3d);

// The bodies: points in hp, speed, strength, carry and vision, and for some in range.
fn miner_body() -> Value {
    obj! { "hp" => 3, "speed" => 2, "strength" => 0, "carry" => 5, "vision" => 0 }
}
fn line_body() -> Value {
    obj! { "hp" => 4, "speed" => 2, "strength" => 3, "carry" => 0, "vision" => 1 }
}
fn gun_body() -> Value {
    obj! { "hp" => 2, "speed" => 1, "strength" => 2, "carry" => 0, "vision" => 1, "range" => 2 }
}
fn medic_body() -> Value {
    obj! { "hp" => 5, "speed" => 2, "strength" => 0, "carry" => 0, "vision" => 3 }
}
fn interceptor_body() -> Value {
    obj! { "hp" => 3, "speed" => 2, "strength" => 2, "carry" => 0, "vision" => 1, "range" => 1 }
}
fn scout_body() -> Value {
    obj! { "hp" => 2, "speed" => 3, "strength" => 0, "carry" => 0, "vision" => 5 }
}

// The research order (atlantis_ext::choose_research): the seven first-tier techs, 12 small puzzles each, Extraction
// and the army's Armor and Dash first; the tier-2 techs the army fields ([`ARMY`]) whoever holds them; Compute boost
// (25% more cycles a core); then Flight, Gate, Enhanced stealth and Air weapons. A ready tier-2 tech that no faction
// holds yet (Mountain traversal, Influence, Capture, Cloak, Faith) jumps the queue once the army's techs are in: a
// world first for 60 puzzles.
const TECHS: [&str; 20] = [
    "extraction", "armor", "dash", "ranged", "pierce", "antiAir", "seaTraversal",
    "repair", "artillery", "scan", "mountainTraversal", "computeBoost", "influence", "capture", "cloak", "faith",
    "flight", "gate", "stealth", "airWeapons",
];

/// What the Merchants of Light trade: the techs of the economy, the map and the mind. The army's techs (Armor, Dash,
/// Ranged fire, Pierce, Anti-air, Repair, Artillery, Scan, Cloak, Gate, Air weapons) stay in Bensalem: leased out
/// every minute to every small faction, they came back as armies with our own guns.
const GOODS: [&str; 8] = ["extraction", "seaTraversal", "mountainTraversal", "influence", "capture", "faith", "computeBoost", "flight"];

/// The tier-2 techs the army fields (medics, artillery, scanners): researched in this order whoever holds them, before
/// any world first.
const ARMY: [&str; 3] = ["repair", "artillery", "scan"];

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

/// `Number(v)` for a value the game gives (null is 0, as `null >= 0` reads it).
fn number(v: View) -> f64 {
    if v.is_null() {
        0.0
    } else {
        v.num()
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

// ---- Clocks ----

// Leases and largesse run on timers with a random gap, so no two worlds repeat: each fires `base` seconds after the
// last, give or take 60%. The next second of each lives in capital RAM page 224, written only when one fires.
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

fn pick_target(world: View, me: View) -> Option<View> {
    // land the army couldn't hold on to (over the water, no ship of ours on station) counts for less: common::can_hold
    let hold = common::can_hold(world, me, false);
    let ranked = realm::rank(world, me, common::frontier(world, me), |w, m, q| {
        let owner = q.get("owner").num();
        let enemy = if owner >= 0.0 && realm::is_rival(w, m, owner) { 7.0 } else { 0.0 };
        enemy + q.get("cores").or(0.0) * 2.0 - common::town_defenders(q) * 1.6 - if q.get("capital").truthy() { 3.0 } else { 0.0 }
            - if hold(q) { 0.0 } else { common::OVER_WATER }
    });
    ranked.first().copied()
}

/// Sets type `t`'s share of the mix, in place when it's there, else last.
fn mix_set(mix: &mut Vec<(&'static str, f64)>, t: &'static str, share: f64) {
    match mix.iter_mut().find(|m| m.0 == t) {
        Some(m) => m.1 = share,
        None => mix.push((t, share)),
    }
}

// The type furthest below its share of the mix. Miners stop at `room`, what our mines can use (common::miner_room):
// past that a new one would only stand idle.
fn choose_spawn(me: View, mix: &[(&'static str, f64)], room: f64) -> Option<&'static str> {
    let units = me.get("units");
    let counts = units.get("byType");
    let total = units.get("total").num() + units.get("queued").num();
    let mut best = None;
    let mut gap = -999.0;
    let weight: f64 = mix.iter().map(|(_, share)| share).sum();
    for &(t, share) in mix {
        if t == "miner" && counts.get(t).or(0.0) >= room {
            continue;
        }
        let g = share / weight * (total + 1.0) - counts.get(t).or(0.0);
        if g > gap {
            gap = g;
            best = Some(t);
        }
    }
    best
}

// Room for more units: up to the allowance, then past it while their upkeep stays under 40% of income (common::room).
fn army_room(me: View) -> f64 {
    let u = me.get("units");
    max(u.get("cap").num() - u.get("total").num() - u.get("queued").num() - u.get("hiredOut").or(0.0), common::room(me, 0.4))
}

fn spawn_one(orders: &mut Value, me: View, ty: &str, build: Value, modules: &[&str]) {
    let fabrication = common::fabrication(me, modules.len() as f64);
    let tactical = if rts(me) { me.get("rts").get("bank").num() } else { 0.0 };
    // Do not burn a spawn order every second when the shared tactical bank cannot
    // pay its fabrication bill. Keeping this reserve is a strategic choice: a
    // finished screen can still dash or armor instead of a new body browning out.
    if army_room(me) <= 0.0 || me.get("bank").num() < me.get("costs").get("unit").num() || tactical < fabrication {
        return;
    }
    let units = me.get("units");
    let n = units.get("spawned").or(0.0) + units.get("queued").num() + 1.0;
    let title=if build.get("wings").truthy(){theme::of(me.get("seat"),3.0).map(|t|t.airframe(air::Role::of(&build)).name).unwrap_or(ty)}else{ty};
    let mut s = obj! { "type" => ty, "name" => format!("{title} {}", text(n)), "build" => build };
    if !modules.is_empty() {
        s.set("modules", modules.to_vec());
    }
    orders.set("spawn", arr![s]);
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

// Spawns the ◆ left can pay. Deals we accept that ask us for ◆, and bribes, are paid before spawns, and a spawn the
// bank can't pay is only refused, so it waits a second.
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

// The spawns the bank can still pay once think is over. The game charges fabrication after think and its RAM writes,
// under the new CPU policy: what think spent and the other lanes' reserves come off the bank first. A spawn it can't
// pay is only an error, so it waits a second. start: cycles_left() when think began.
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

// CPU is one shared balance.  This policy limits how quickly each lane can
// consume it and reserves enough for the next research offer in quiet periods.
// Every value below is a fraction of current income unless it exceeds one.
fn cpu_policy(world: View, me: View, orders: &Value, pressure: f64, need: f64) -> Value {
    let rts = me.get("rts");
    let c = rts.get("cpu");
    let income = max(1.0, c.get("income").or(me.get("compute").get("cycles").or(14000.0)));
    let bank = c.get("bank").or(if rts.truthy() { rts.get("bank").or(0.0) } else { 0.0 });
    // the thresholds below were tuned on a store of three seconds' income; the bank holds two minutes now
    let capacity = income * 3.0;
    // the tech under study: common::budget names it in orders.research only on the second it changes
    let research = match orders.get("research") {
        Value::Str(s) if !s.is_empty() => s.clone(),
        _ => me.get("research").get("tech").str().to_string(),
    };
    let tech = if !research.is_empty() && world.get("techs").truthy() { world.get("techs").get(&research) } else { View::undefined() };
    let offer = tech.get("offer").or(0.0);
    let urgent = pressure >= 2.0;
    // This grows as the shared bank grows.  It reaches the full offer only when
    // the faction can genuinely afford to protect a whole research attempt.
    // Do not reserve for an offer bigger than three seconds of income; the
    // reservation would only starve planning and fabrication.
    // The reserve leaves think what it needs (need, and the upkeep and reflex reserves
    // below): reserving the whole offer starved a poor faction's think every second.
    // (half a second's income at most: a reserve up to the offer, 600,000 cycles for a tier-3 tech, held the bank away
    // from the army's armor and shots)
    let study_reserve = if !urgent && offer != 0.0 && capacity >= offer {
        max(0.0, min(min(min(offer, income * 0.5), income * 0.6 + bank * 0.18), bank - need - income * 0.28 - 4000.0))
    } else {
        0.0
    };
    let strained = bank < capacity * 0.55 || (!c.get("available").is_undef() && c.get("available").num() < income * 0.35);
    let limits = obj! {
        // Think may spend half again what a full think costs (its census grows
        // with what it sees), from 24k up to 64k or half of income. A think that runs out sets no
        // new policy, so the limit has to clear the real cost, not an estimate.
        "think" => round(min(max(64000.0, income * 0.5), max(max(24000.0 + me.get("units").get("total").or(0.0) * 200.0, income * 0.4), need * 1.5))),
        // Type callbacks can arrive on events between their scheduled turns.
        // Keep a real absolute control pool so a whole battery can still accept
        // its first legal order instead of producing partial, failed callbacks.
        "control" => round(max(if urgent { 36000.0 } else { 32000.0 }, income * 0.45)),
        "reflex" => if urgent { 18000 } else { 14000 },
        // Research puts up to this into its savings each second: half of income, since an attempt costs a few
        // thousand cycles and three a second need 20,000 to 60,000 (atlantis_ext::research_limit); 12,000 cycles
        // at least, so the first tier-1 attempt comes within 10 seconds of the epoch; nothing once the tree is done.
        "research" => atlantis_ext::research_limit(world, me, pressure),
        "upkeep" => 0.96,
        "drive" => if urgent { 0.28 } else { 0.13 },
        "armor" => if urgent { 0.34 } else { 0.12 },
        "cloak" => 0.05,
        "weapon" => if urgent { 0.30 } else { 0.14 },
        "abilities" => if urgent { 0.34 } else { 0.16 },
        // A 2-module specialist needs 2,400 cycles at price level 1. Keep this above a
        // single viable build even during pressure, while still making a shallow bank choose.
        "fabrication" => common::price_level(me) * if strained || urgent { 3000.0 } else { 6000.0 },
        "gate" => 0.03,
        "duel" => if urgent { 0.16 } else { 0.08 },
        "challenge" => 0.08,
        "visuals" => 0.02,
    };
    let reserve = obj! {
        // Keep a larger planner floor only once the bank has enough room for it.
        "think" => if bank > capacity * 0.70 { 6000 } else { 3000 },
        "reflex" => if urgent { 9000 } else { 4000 },
        "research" => study_reserve,
        "upkeep" => if urgent { income * 0.45 } else { income * 0.28 },
        "armor" => if urgent { income * 0.18 } else { 0.0 },
        "abilities" => if urgent { income * 0.16 } else { 0.0 },
    };
    // During a siege the screen has to remain alive; at peace miners protect
    // future income, then specialized bodies follow.
    let order: Vec<&str> = if urgent {
        vec!["artillery", "bulwark", "medic", "interceptor", "miner", "line", "scout"]
    } else {
        vec!["miner", "artillery", "bulwark", "medic", "interceptor", "line", "scout"]
    };
    obj! { "limits" => limits, "reserve" => reserve, "upkeepOrder" => order }
}

// One route search in think instead of one per unit type each second: the plan carries a field toward where the army
// goes (the province to defend, else the attack), cut to the steps from our land and the free or allied land next to
// it (where a unit stepping out stands), and path::to routes along it. None when path::home already leads there from
// all our land; it can't from an exclave or land over the sea. The last plan's field to the same place serves for 10 s.
// A search costs up to about 300 cycles a province, so it runs only when think can pay for it and still have `after`
// cycles to finish; until then the last field to the same place serves, however old.
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

// What a full think costs: about 7,000 cycles plus 50 for each unit in sight (the census of rivals' units below), or
// the most a think took in the last 5 seconds when that's more. Think's CPU limit and the research reserve are sized
// from it, so a poor bank still pays for think.
fn think_need(world: View, me: View) -> f64 {
    let units = world.get("units");
    let seen = if units.truthy() { units.len() as f64 } else { 0.0 };
    max(7000.0 + 50.0 * seen, me.get("compute").get("thinkPeak").or(0.0))
}

fn think(world: View, me: View) -> Value {
    // A nearly empty callback only opens the next allowance. Existing type schedules and orders stand.
    if me.path("rts.cpu").truthy() && cycles_left() < 3000.0 {
        return obj! { "compute" => obj! { "cpu" => obj! {
            "limits" => obj! { "think" => 24000, "control" => 0.5, "upkeep" => 0.65,
                "research" => 0, "influence" => 0, "fabrication" => 0, "visuals" => 0,
                "reflex" => 0.15, "duel" => 0.08, "challenge" => 0.02,
                "drive" => 0.08, "armor" => 0, "weapon" => 0.08,
                "abilities" => 0, "cloak" => 0, "gate" => 0 },
            "reserve" => obj! {}, "upkeepOrder" => vec!["miner"]
        } } };
    }
    // Recovery's map and unit scans also need fuel when income falls sharply.
    // A policy-only turn lets the next call recover without repeating those scans.
    if me.path("rts.cpu").truthy() && cycles_left() < 18000.0 {
        let income = me.path("rts.cpu.income").num_or(20000.0).max(1.0);
        let mut types = obj! {};
        for ty in me.get("types").keys() { types.set(ty, obj! { "share" => 0, "every" => 8 }); }
        types.set("miner", obj! { "cycles" => (income * 0.4).max(4000.0), "every" => 1 });
        let mut cpu = obj! {
            "limits" => obj! {
                "think" => (me.path("compute.thinkPeak").num_or(0.0) * 1.5 + 2000.0).max(24000.0),
                "control" => 0.5, "upkeep" => 0.65, "research" => 0, "influence" => 0,
                "fabrication" => 0, "visuals" => 0, "reflex" => 0.15, "duel" => 0.08,
                "challenge" => 0.02, "drive" => 0.08, "armor" => 0.05, "weapon" => 0.08,
                "abilities" => 0, "cloak" => 0, "gate" => 0
            },
            "reserve" => obj! {}, "upkeepOrder" => vec!["miner"]
        };
        common::fit_cpu(&mut cpu, me);
        // Shared miner recovery reads this capital page before using local mining rights.
        let mut recovery = mem::get(me.get("ramCapital"), 244);
        if !matches!(recovery, Value::Obj(_)) { recovery = obj! {}; }
        recovery.set("r", true);
        return common::govern(world, me, 0.4, || obj! { "compute" => obj! { "cpu" => cpu, "types" => types },
            "ram" => mem::put(244, &recovery, "local", None, 1).unwrap_or_default(),
            "propaganda" => Value::arr(), "preach" => Value::arr(), "rent" => 0,
            "memo" => "Recovering production" });
    }
    common::govern(world, me, 0.4, || strategy(world, me))
}

fn strategy(world: View, me: View) -> Value {
    // Below this even the lean plan won't fit: keep the standing orders a second longer.
    if rts(me) && cycles_left() < 6000.0 {
        return common::recovery(world, me);
    }
    let start = cycles_left();
    // A short second (a poor bank, a crowded view) takes the lean path: it keeps the last plan's target, skips the census
    // of rivals' units and the trade, and still shares the plan, spawns and sets the CPU policy. Running out instead
    // would throw away every order and leave the army on stale ones.
    let need = think_need(world, me);
    // Under three quarters of a full think even the lean path may run out, and a think that runs out sets no policy:
    // its limit, a share of income, stays too small when income falls, every second after. Set only the policy then.
    if rts(me) && cycles_left() < need * 0.75 {
        return common::recovery(world, me);
    }
    let lean = rts(me) && cycles_left() < need + 3000.0;
    let me_id = me.get("id").num();
    let mut orders = obj! { "policy" => obj! { "extraction" => 0.47, "autonomy" => "direct", "integrate" => "tolerate", "invest" => true } };
    let prev = if lean { mem::get(me.get("ramCapital"), 0) } else { Value::Undef };
    let kept = if prev.truthy() && num(prev.get("attack")) >= 0.0 {
        let q = realm::index(world).get(num(prev.get("attack")));
        q.truthy().then_some(q)
    } else {
        None
    };
    let kept = kept.filter(|q| q.get("owner").num() != me_id);
    let target = kept.or_else(|| pick_target(world, me));
    let mut survey = common::survey(world, me, &target.into_iter().collect::<Vec<View>>(), 2.0);
    if kept.is_some() {
        survey.set("stage", prev.get("stage").clone());
    }
    survey.set("home", me.get("capital"));
    let invaded_n = or(survey.get("threat"), 0.0);
    // the lean path keeps the last plan's road to the same target rather than search again
    let goal = if num(survey.get("defend")) >= 0.0 { num(survey.get("defend")) } else { num(survey.get("attack")) };
    let field = if lean {
        let f = prev.get("field");
        (f.truthy() && num(f.get("to")) == goal).then(|| f.clone())
    } else {
        road(world, me, goal, need + 2000.0)
    };
    if let Some(f) = field {
        survey.set("field", f);
    }
    plan::share(&mut orders, &survey, me);

    let techs = me.get("techs");
    let has = |t: &str| techs.get(t).truthy();
    let (mut air, mut stealth, mut ground) = (0.0, 0.0, 0.0);
    // `world.units` is fog-limited: this reacts only to units the faction has
    // actually seen, rather than to a hidden global composition.
    if !lean {
        for u in world.get("units").iter() {
            let owner = u.get("owner").num();
            if owner == me_id || !realm::is_rival(world, me, owner) {
                continue;
            }
            let build = u.get("build");
            if build.get("wings").truthy() {
                air += 1.0;
            } else {
                ground += 1.0;
            }
            if build.get("stealth").truthy() {
                stealth += 1.0;
            }
        }
    }
    let pressure = invaded_n + min(2.0, ground);
    let cpu = me.get("rts").get("cpu");
    let income = max(1.0, cpu.get("income").or(me.get("compute").get("cycles").or(14000.0)));
    let capacity = income * 3.0;
    let strained = cpu.get("bank").or(0.0) < capacity * 0.55 || (!cpu.get("available").is_undef() && cpu.get("available").num() < income * 0.35);
    let mut mix: Vec<(&'static str, f64)> =
        if strained { vec![("miner", 0.64), ("line", 0.25), ("scout", 0.04)] } else { vec![("miner", 0.50), ("line", 0.34), ("scout", 0.06)] };
    if has("armor") || has("dash") {
        mix_set(&mut mix, "bulwark", 0.28);
        mix_set(&mut mix, "line", 0.08);
    }
    if has("artillery") && has("ranged") && ground >= 3.0 && !strained {
        mix_set(&mut mix, "artillery", 0.18);
    }
    if has("repair") && !strained {
        mix_set(&mut mix, "medic", if stealth != 0.0 && has("scan") { 0.13 } else { 0.07 });
    }
    if has("pierce") && has("ranged") && !strained {
        mix_set(&mut mix, "piercer", 0.12);
    }
    if has("antiAir") && has("ranged") && !strained {
        mix_set(&mut mix, "interceptor", if air != 0.0 { 0.17 } else { 0.06 });
    }
    // Salomon's House fields a small research wing after Flight, without displacing its mining/screen floor.
    let own_ground=me.path("units.total").num_or(0.0)-me.path("units.byType.miner").num_or(0.0)-me.path("units.byType.aeronaut").num_or(0.0);
    if has("flight")&&!strained&&own_ground>=4.0&&me.path("units.byType.aeronaut").num_or(0.0)<4.0 {
        mix_set(&mut mix,"aeronaut",0.12);
    }
    let pick = |a: (bool, &'static str), b: (bool, &'static str)| -> Vec<&'static str> { [a, b].into_iter().filter(|x| x.0).map(|x| x.1).collect() };
    match choose_spawn(me, &mix, common::miner_room(me, &survey)) {
        Some("miner") => spawn_one(&mut orders, me, "miner", miner_body(), &[]),
        Some("line") => spawn_one(&mut orders, me, "line", line_body(), &[]),
        Some("bulwark") => spawn_one(&mut orders, me, "bulwark", line_body(), &pick((has("armor"), "armor"), (has("dash"), "dash"))),
        Some("artillery") => spawn_one(&mut orders, me, "artillery", gun_body(), &pick((has("artillery"), "artillery"), (has("scan"), "scan"))),
        Some("medic") => spawn_one(&mut orders, me, "medic", medic_body(), &pick((has("repair"), "repair"), (has("scan"), "scan"))),
        Some("piercer") => spawn_one(&mut orders, me, "piercer", interceptor_body(), if has("dash") { &["pierce", "dash"][..] } else { &["pierce"][..] }),
        Some("interceptor") => spawn_one(&mut orders, me, "interceptor", interceptor_body(), &pick((has("antiAir"), "antiAir"), (has("dash"), "dash"))),
        Some("aeronaut") => spawn_one(&mut orders,me,"aeronaut",common::airframe(me,me.path("units.byType.aeronaut").num_or(0.0)).build(),&[]),
        _ => spawn_one(&mut orders, me, "scout", scout_body(), &[]),
    }

    // Compute chooses research/type schedules; shared-CPU policy below replaces
    // the old split and makes every execution lane trade against one bank.
    let available = cpu.get("available");
    let types = common::calls(
        world,
        // Controllers keep their last legal orders. Expensive route/deploy loops
        // therefore run less often, while artillery gets enough of the control
        // lane to route an entire battery when the plan changes.
        // Absolute allocations are deliberate: a fractional split made every
        // event wake-up too small to issue even one Order. Every type we field
        // gets its base first (common::calls); lines and scouts grow their calls
        // last, so the rest goes to the combined-arms bodies that can change the fight.
        &[
            ("miner", 6400.0, 3.0),
            ("bulwark", 6000.0, 4.0),
            ("artillery", 6000.0, 5.0),
            ("piercer", 6000.0, 5.0),
            ("medic", 6000.0, 6.0),
            ("interceptor", 6000.0, 5.0),
            ("aeronaut", 4200.0, 3.0),
            ("line", 6000.0, 4.0),
            ("scout", 4000.0, 6.0),
        ],
        cpu.get("income").or(0.0),
        if available.truthy() { available.get("control").num() * 0.8 } else { f64::INFINITY },
        me.get("units").get("byType"),
    );
    // common::budget sets the split, the types and the rent; the tech to research is Salomon's House's own choice
    common::budget(&mut orders, world, me, &common::Budget { research: 0.13, code: 0.66, techs: &[], types: Some(&types) });
    atlantis_ext::choose_research(&mut orders, world, me);
    let mut policy = cpu_policy(world, me, &orders, pressure, need);
    common::fit_cpu(&mut policy, me);
    orders.entry("compute").set("cpu", policy);
    orders.set("memo", atlantis_ext::memo(world, me));
    let pact_leader = realm::leader(world).map(|l| l.get("id").num());
    let no_pacts = me.get("pacts").is_empty();
    let pact = move |m: View| no_pacts && pact_leader.is_none_or(|l| m.get("from").num() != l);
    if lean || cycles_left() < 6000.0 {
        if cycles_left() > 2000.0 {
            common::answer(&mut orders, me, world, pact, &TECHS);
        }
        return affordable(orders, me, start);
    }
    // A pact against a runaway leader: ask the biggest partner in no pact, at most once per 2 minutes (common::court).
    if pact_leader.is_some_and(|lead| no_pacts && lead != me_id && common::dominant(world).is_some()) {
        if let Some(p) = common::partners(world, me).first() {
            let id = p.get("id").num();
            common::court(&mut orders, world, me, id, |o| diplomacy::propose(o, me, vec![id], (), "against the leader"), &common::Court::default());
        }
    }
    let at = mem::get(me.get("ramCapital"), CLOCK);
    let mut clock = Clock { at: if at.truthy() { at } else { Value::obj() }, changed: false };
    landlord(&mut orders, world, me, &mut clock, or(survey.get("threat"), 0.0) <= 6.0);
    atlantis_ext::merchants(&mut orders, world, me, &mut clock);
    common::answer(&mut orders, me, world, pact, &TECHS);
    if clock.changed {
        concat(&mut orders, "ram", mem::put(CLOCK, &clock.at, "local", None, 1).unwrap_or_default());
    }
    affordable(orders, me, start)
}

// The landlord's trade: tolls where its land leaves by sea or pass, a mine lease on a neighbour's border when its own
// miners leave slots idle, soldiers for hire against the leader, and largesse for its own restless provinces.
fn landlord(orders: &mut Value, world: View, me: View, clock: &mut Clock, calm: bool) {
    let now = world.get("seconds").or(0.0).floor();
    let by_id = realm::index(world);
    let me_id = me.get("id").num();
    let factions = world.get("factions");
    orders.set("toll", Value::arr());
    for p in world.get("provinces").iter() {
        let routes = p.get("routes");
        if p.get("owner").num() == me_id && routes.truthy() && (!routes.get("sea").is_empty() || !routes.get("pass").is_empty()) {
            trade::toll(orders, p.get("id").num(), 2.0);
        }
    }
    let contracts = me.get("contracts");
    if !contracts.iter().any(|c| !c.get("ended").truthy() && c.get("give").get("mines").truthy()) && due(clock, "lease", 60.0, now) {
        // the tenant: a neighbour on that border that isn't the leader and can pay (3% of the world or more), allies
        // first, then the smallest, since idle mines matter most to it
        let lead = realm::leader(world).map(|l| l.get("id").num());
        for m in trade::idle_mines(world, me) {
            let q = by_id.get(m.province);
            let mut best: Option<View> = None;
            let mut score = -1e9;
            for a in q.get("adj").iter() {
                let n = by_id.at(a);
                let owner = n.get("owner").num();
                let f = if n.truthy() && owner >= 0.0 && owner != me_id { factions.key(owner) } else { View::undefined() };
                let share = f.get("share").num();
                if !f.truthy() || !f.get("alive").truthy() || lead == Some(f.get("id").num()) || share < 0.03 {
                    continue;
                }
                let ally = if me.get("allies").has_num(f.get("id").num()) { 1.0 } else { 0.0 };
                let s = ally - share + 0.02 * rand();
                if s > score {
                    score = s;
                    best = Some(f);
                }
            }
            let Some(best) = best else { continue };
            // priced as common::worth prices a lease: a sixth of a ◆ a slot a second
            let slots = min(2.0, m.slots);
            trade::lease_mines(orders, best.get("id").num(), m.province, slots, 300.0, obj! { "coin" => round(slots * 300.0 / 6.0) }, "idle mines on our border, five minutes");
            break;
        }
    }
    // soldiers for hire: with six invaders or fewer in its land and line troops to spare, four of them serve the
    // smallest faction that borders the leader (not the leader), else the smallest of our other neighbours, and never one
    // with a fifth of the world, for five minutes, one hire at a time, every 2½ minutes or so
    if calm && !contracts.iter().any(|c| !c.get("ended").truthy() && c.get("give").get("hire").truthy()) && due(clock, "hire", 150.0, now) {
        // (units we hired in from someone else aren't ours to hire out)
        let spare: Vec<View> = world
            .get("units")
            .iter()
            .filter(|u| u.get("owner").num() == me_id && !u.get("hired").truthy() && matches!(u.get("type").str(), "line" | "bulwark"))
            .collect();
        if let Some(lead) = realm::leader(world).map(|l| l.get("id").num()) {
            if lead != me_id && spare.len() >= 6 {
                let (mut near, mut ours): (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
                for p in world.get("provinces").iter() {
                    let po = p.get("owner").num();
                    if po != lead && po != me_id {
                        continue;
                    }
                    for a in p.get("adj").iter() {
                        let n = by_id.at(a);
                        let owner = n.get("owner").num();
                        if n.truthy() && owner >= 0.0 && owner != lead && owner != me_id {
                            let set = if po == lead { &mut near } else { &mut ours };
                            if !set.contains(&owner) {
                                set.push(owner);
                            }
                        }
                    }
                }
                let smallest = |set: &[f64]| {
                    let mut to: Option<View> = None;
                    for f in factions.iter() {
                        let share = f.get("share").num();
                        if f.get("alive").truthy() && set.contains(&f.get("id").num()) && share >= 0.03 && share < 0.2 && to.is_none_or(|t| share < t.get("share").num()) {
                            to = Some(f);
                        }
                    }
                    to
                };
                let front = smallest(&near);
                if let Some(to) = front.or_else(|| smallest(&ours)) {
                    let ids: Vec<Value> = spare.iter().take(4).map(|u| u.get("id").value()).collect();
                    let body = if front.is_some() { "four soldiers for five minutes, to hold your border with the leader" } else { "four soldiers for five minutes" };
                    // priced as common::worth prices a hire: a tenth of a ◆ a unit a second
                    trade::hire(orders, to.get("id").num(), ids, 300.0, obj! { "coin" => 120 }, body);
                }
            }
        }
    }
    // largesse (60◆) is paid before spawns (payable): it requires at least 60◆ in the bank
    if me.get("bank").num() >= 60.0 && due(clock, "largesse", 10.0, now) {
        let mut worst: Option<View> = None;
        for p in world.get("provinces").iter() {
            let risk = p.get("risk").num();
            if p.get("owner").num() == me_id && risk > 0.04 && worst.is_none_or(|w| risk > w.get("risk").num()) {
                worst = Some(p);
            }
        }
        if let Some(w) = worst {
            sway::bribe(orders, w.get("id").num(), 60.0);
        }
    }
}

// ---- Unit types ----

// Unit types commit each order with order() as they make it, rather than return the list: the game applies the same
// orders, and what sending each costs counts before the next unit, so the 2,200 cycles kept back stay free. Returning
// the list would cost cycles after the last check.

fn miner(units: View, world: View, me: View) {
    if common::recover_miners(units, world, me) { return; }
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        if let Some(o) = orders::gather(nth(units, turn, world), world, me) {
            order(o);
        }
    }
}

// Route searches must leave room for plan carry, submission and the callback return.
// Admission measures that whole operation, including cold plan reads and RAM writes.
struct ControlBudget {
    next: f64,
    seen: Vec<f64>,
}

impl ControlBudget {
    fn new(me: View) -> Self {
        if me.get("rts").truthy() {
            orders::set_reserve(1100.0);
            orders::set_first(1400.0);
        }
        Self { next: 1800.0, seen: Vec::new() }
    }

    fn admits(&mut self, u: View, world: View, me: View) -> bool {
        if !me.get("rts").truthy() { return true; }
        let page = me.get("ramProvince").key(u.get("at").num()).key(0.0);
        let stamp = page.get("t").num_or(-1.0);
        let cold = page.truthy() && !self.seen.contains(&stamp);
        let bytes = page.get("bytes");
        let read = if cold && bytes.is_bytes() && bytes.len() >= 2 {
            let b = bytes.bytes();
            4.0 * f64::from(u16::from_le_bytes([b[0], b[1]]))
        } else { 0.0 };
        let index_cost = if realm::indexed(world) { 0.0 } else { 4.0 * world.get("provinces").len() as f64 };
        if cycles_left() < self.next + read + index_cost + 600.0 { return false; }
        let p = plan::of(me, u);
        if cold { self.seen.push(stamp); }
        let defend = common::defends(&p, u);
        let attack = p.get("attack").num_or(-1.0);
        let target = if defend >= 0.0 { defend } else if attack >= 0.0 { attack } else { me.get("capital").num() };
        let from = if u.get("to").num_or(-1.0) >= 0.0 { u.get("to").num() } else { u.get("at").num() };
        let index = realm::index(world);
        let here = index.get(from);
        // Home-route walking and serializing its steps also consume execution fuel.
        let steps = if from == target || realm::reach(here, me).contains(&target) { 0.0 } else {
            let q = index.get(target);
            let goal = if q.get("owner").num_or(-1.0) == me.get("id").num() { q.get("hops").num_or(0.0) } else {
                q.get("adj").iter().map(|id| index.at(id)).filter(|a| a.get("owner").num_or(-1.0) == me.get("id").num())
                    .map(|a| a.get("hops").num_or(0.0)).fold(f64::INFINITY, f64::min)
            };
            here.get("hops").num_or(0.0).max(0.0) + if goal.is_finite() { goal.max(0.0) + 1.0 } else { 0.0 }
        };
        cycles_left() >= self.next + 60.0 * steps + 600.0
    }

    fn has_room(&self, me: View) -> bool {
        !me.get("rts").truthy() || cycles_left() >= self.next + 600.0
    }

    fn finish(&mut self, began: f64) {
        self.next = self.next.max((began - cycles_left()) * 1.5 + 400.0);
    }

    fn submit(o: Value) -> bool {
        if cycles_left() < 600.0 { return false; }
        order(o);
        true
    }
}

// Transitional infantry only route when their job actually changes.  They stay
// useful as a small capture/defence reserve while the licensed bodies take over.
fn line(units: View, world: View, me: View) {
    // a call short of the plan lookup and an order keeps its units' standing orders a second longer
    if rts(me) && cycles_left() < 2200.0 {
        return;
    }
    let mut budget = ControlBudget::new(me);
    let mut first = None;
    for u in units.iter() {
        if !budget.has_room(me) { break; }
        if budget.admits(u, world, me) { first = Some(u); break; }
    }
    let Some(first) = first else { return; };
    let began = cycles_left();
    let p = plan::check(plan::of(me, first), world);
    budget.finish(began);
    let attack = num(p.get("attack"));
    for turn in 0..units.len() {
        let u = nth(units, turn, world);
        if !budget.has_room(me) { break; }
        let began = cycles_left();
        if !budget.admits(u, world, me) { continue; }
        // those the plan sends to defend (common::defends), the rest on the attack
        let defend = common::defends(&p, u);
        let (verb, target) = if defend >= 0.0 { ("fight", defend) } else { ("capture", attack) };
        if target.is_nan() || target < 0.0 || (u.get("doing").str() == verb && u.get("target").num() == target) {
            continue;
        }
        if let Some(o) = orders::to(u, verb, target, world, me, ()) {
            if !ControlBudget::submit(o) { break; }
        }
        budget.finish(began);
    }
}

// Abroad, a unit knows only the plan it carried out of your land (plan::of), so a unit at home carries the current one
// in its RAM (orders::carry) on its order, or on a RAM-only order when its order stands. Without it, a unit that
// crosses the border forgets its target and walks home.
fn carry(o: Option<Value>, u: View, p: &Rc<Value>, me: View) -> Option<Value> {
    let Some(ops) = orders::carry(u, p, me) else { return o };
    let mut x = o.unwrap_or_else(|| obj! { "id" => u.get("id") });
    x.set("ram", ops);
    Some(x)
}

#[derive(PartialEq, Clone, Copy)]
enum Role {
    Wall,
    Gun,
    Medic,
    Fast,
}

fn tactical_order(u: View, world: View, me: View, role: Role) -> Option<Value> {
    let p = plan::of(me, u);
    let defend = common::defends(&p, u);
    let attack = num(p.get("attack"));
    let o = if defend >= 0.0 {
        orders::to(u, "fight", defend, world, me, ())
    } else if attack >= 0.0 {
        orders::to(u, if role == Role::Gun { "fight" } else { "capture" }, attack, world, me, ())
    } else {
        orders::to(u, "fight", me.get("capital").num(), world, me, ())
    };
    // Unchanged route and power stay in the engine. Avoiding a fresh empty order
    // per unit is what leaves cycles for reactions and the shared combat bank.
    let Some(o) = o else { return carry(None, u, &p, me) };
    let mut o = carry(Some(o), u, &p, me)?;
    let cpu = me.get("rts").get("cpu");
    let income = max(1.0, cpu.get("income").or(me.get("compute").get("cycles").or(14000.0)));
    let tight = cpu.get("bank").num() < income * 3.0 * 0.55
        || (!cpu.get("available").is_undef() && cpu.get("available").num() < income * 0.35);
    let pressed = or(p.get("threat"), 0.0) > 0.0;
    let power = match role {
        Role::Wall => obj! { "drive" => if pressed { 360 } else { 180 }, "armor" => if !pressed { 0 } else if tight { 480 } else { 850 }, "weapon" => 220 },
        Role::Gun => obj! { "drive" => if tight { 100 } else { 180 }, "armor" => 160, "weapon" => if tight { 340 } else { 650 } },
        Role::Medic => obj! { "drive" => if tight { 260 } else if pressed { 620 } else { 420 }, "armor" => 0, "weapon" => 0 },
        Role::Fast => obj! { "drive" => if tight { 400 } else { 720 }, "armor" => 0, "weapon" => if tight { 180 } else { 360 } },
    };
    o.set("power", power);
    Some(o)
}

fn tactical(units: View, world: View, me: View, role: Role) {
    let mut budget = ControlBudget::new(me);
    for turn in 0..units.len() {
        let u = nth(units, turn, world);
        if !budget.has_room(me) { break; }
        let began = cycles_left();
        if !budget.admits(u, world, me) { continue; }
        if let Some(o) = tactical_order(u, world, me, role) {
            if !ControlBudget::submit(o) { break; }
        }
        budget.finish(began);
    }
}

fn bulwark(units: View, world: View, me: View) {
    tactical(units, world, me, Role::Wall)
}

fn artillery(units: View, world: View, me: View) {
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        let u = nth(units, turn, world);
        let r = u.get("rts");
        let p = plan::of(me, u);
        let target = if common::defends(&p, u) >= 0.0 {
            common::defends(&p, u)
        } else if num(p.get("attack")) >= 0.0 {
            num(p.get("attack"))
        } else {
            me.get("capital").num()
        };
        let travelling = u.get("at").num() != target || number(u.get("to")) >= 0.0;
        if travelling {
            // A deployed gun has zero movement. Pack it in the same order that gives
            // it the new route, including when the target changed under its feet.
            let o = carry(orders::to(u, "fight", target, world, me, ()), u, &p, me);
            if o.is_some() || r.get("deployed").truthy() || r.get("deploying").truthy() {
                let mut o = o.unwrap_or_else(|| obj! { "id" => u.get("id") });
                o.set("deploy", false);
                o.set("power", obj! { "drive" => 260, "armor" => 180, "weapon" => 650 });
                order(o);
            }
        } else if !r.get("deployed").truthy() && !r.get("deploying").truthy() {
            // The destination is reached: spend the one-second setup time, then let
            // reflex choose shell targets. Do not plant guns at their home province.
            order(obj! { "id" => u.get("id"), "deploy" => true, "power" => obj! { "drive" => 120, "armor" => 180, "weapon" => 650 } });
        }
    }
}

fn medic(units: View, world: View, me: View) {
    tactical(units, world, me, Role::Medic)
}

fn interceptor(units: View, world: View, me: View) {
    tactical(units, world, me, Role::Fast)
}

fn piercer(units: View, world: View, me: View) {
    interceptor(units, world, me)
}

fn scout(units: View, world: View, me: View) {
    // common::wander is a search: each scout that has stopped gets one while the call has cycles for it, from a
    // rotating start, so the rest wait a call rather than run it out
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        let u = nth(units, turn, world);
        if u.get("doing").str() == "move" {
            continue;
        }
        let next = common::wander(world, me, u.get("at").num());
        if next >= 0.0 {
            order(obj! { "id" => u.get("id"), "do" => "move", "target" => next, "power" => obj! { "drive" => 840, "armor" => 0, "weapon" => 0 } });
        }
    }
}

// ---- reflex ----

// Target selection, facing and support are faction code; the engine only prices and validates.
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

/// A unit to repair: its id, health and distance.
struct Patient {
    id: Value,
    hp: f64,
    hp_max: f64,
    d: f64,
}

fn repair_friend(u: View, ns: View, o: &mut Value, bank: f64, level: f64) -> f64 {
    let r = u.get("rts");
    if !r.get("modules").has_str("repair") || r.get("abilityIn").get("repair").or(0.0) > 0.0 {
        return 0.0;
    }
    let (hp, hp_max) = (u.get("hp").num(), u.get("hpMax").num());
    let mut target = (hp < hp_max - 0.1).then(|| Patient { id: u.get("id").value(), hp, hp_max, d: 0.0 });
    for t in ns.iter() {
        let (thp, tmax, d) = (t.get("hp").num(), t.get("hpMax").num(), t.get("d").num());
        let friend = (t.get("owner").is_num() && t.get("owner").num() == u.get("owner").num()) || t.get("ally").truthy();
        if friend && d <= 3.0 && thp < tmax - 0.1 && target.as_ref().is_none_or(|x| thp / tmax < x.hp / x.hp_max) {
            target = Some(Patient { id: t.get("id").value(), hp: thp, hp_max: tmax, d });
        }
    }
    let Some(target) = target else { return 0.0 };
    let amount = min(0.7, target.hp_max - target.hp);
    let cost = (level * 700.0 * (amount / 0.7) * (1.0 + 0.15 * target.d * target.d)).ceil();
    if cost > bank {
        return 0.0;
    }
    o.set("ability", "repair");
    o.set("repairTarget", target.id);
    o.set("amount", amount);
    cost
}

/// A lane's budget, or what's left of the bank when the budget doesn't name it.
fn lane(b: View, name: &str, left: f64) -> f64 {
    let v = b.get(name);
    if v.is_undef() {
        left
    } else if v.is_null() {
        0.0
    } else {
        v.num()
    }
}

#[allow(unused_assignments)] // the lanes left, kept as they're spent even after the last read
fn reflex(units: View, near: View) -> Vec<Value> {
    let n = units.len();
    let start = if n > 0 { ((units.at(0).get("rts").get("bank").or(0.0) / 211.0).floor() % n as f64) as usize } else { 0 };
    // A rich first reaction can exceed a short callback's entire allowance.
    // Submit each order once and leave room for the callback's return.
    let mut turn_cost = 4000.0_f64;
    for turn in 0..n {
        if cycles_left() < turn_cost + 400.0 {
            break;
        }
        let began = cycles_left();
        let u = units.at((start + turn) % n);
        let ns = near.key(u.get("id").num());
        if cycles_left() < 3400.0 + 160.0 * ns.len() as f64 {
            break;
        }
        let r = u.get("rts");
        let modules = r.get("modules");
        let (mut threat, mut best, mut shell_target): (Option<View>, Option<View>, Option<View>) = (None, None, None);
        let (mut score, mut ground) = (99.0, 0.0);
        for t in ns.iter() {
            if !t.get("rival").truthy() {
                continue;
            }
            let air = t.get("air").truthy();
            if !air {
                ground += 1.0;
            }
            let d = t.get("d").num();
            if !air && d <= 14.0 && shell_target.is_none_or(|old| d < old.get("d").num()) {
                shell_target = Some(t);
            }
            if threat.is_none() && t.get("range").is_num() && t.get("range").num() == 0.0 && d <= 2.2 {
                threat = Some(t);
            }
            if d > u.get("reach").num() || (air && !u.get("air").truthy() && !modules.has_str("antiAir")) {
                continue;
            }
            let s = t.get("hp").num() / t.get("hpMax").num() - if t.get("range").truthy() { 0.18 } else { 0.0 } - if air { 0.11 } else { 0.0 };
            if s < score {
                score = s;
                best = Some(t);
            }
        }
        best = common::windup_target(u, ns, best);
        let low = u.get("hp").num() < u.get("hpMax").num() * 0.40;
        // A unit sees a shared-bank snapshot. Spend a local estimate conservatively
        // and leave 700 CPU headroom for this code and other combat events.
        let mut left = max(0.0, r.get("bank").or(0.0) - 700.0);
        let b = r.get("budget");
        let mut weapon_left = lane(b, "weapon", left);
        let mut armor_left = lane(b, "armor", left);
        let mut drive_left = lane(b, "drive", left);
        let mut ability_left = lane(b, "abilities", left);
        // every price below is the rules' price times the world's price level
        let level = common::price_level(u);
        let ty = u.get("type").str();
        let (drive, mut armor, weapon) = match ty {
            "bulwark" => (300.0, 850.0, 300.0),
            "artillery" => (120.0, 180.0, 720.0),
            "interceptor" | "piercer" => (780.0, 0.0, 450.0),
            "medic" => (420.0, 0.0, 0.0),
            _ => (500.0, 0.0, 260.0),
        };
        if !modules.has_str("armor") || ground == 0.0 { armor = 0.0; }
        let armor_cost = level * armor * 0.2;
        if armor_cost > left || armor_cost > armor_left {
            armor = 0.0;
        } else {
            left -= armor_cost;
            armor_left -= armor_cost;
        }
        let mut o = obj! { "id" => u.get("id"), "power" => obj! { "drive" => drive, "armor" => armor, "weapon" => weapon }, "guard" => guard_facing(ns), "pierce" => 0 };
        let heal_cost = if ty == "medic" { repair_friend(u, ns, &mut o, min(left, ability_left), level) } else { 0.0 };
        left -= heal_cost;
        ability_left -= heal_cost;
        // Dodge a shell before it lands. Hazards are own-observation data only.
        let pos = if u.get("local").truthy() { u.get("local") } else { u.get("pos") };
        let mut hazard: Option<(f64, f64)> = None;
        for h in r.get("hazards").iter() {
            let x = if pos.truthy() { pos.get("x").num() - h.get("x").num() } else { 99.0 };
            let y = if pos.truthy() { pos.get("y").num() - h.get("y").num() } else { 99.0 };
            let reach = h.get("radius").num() + 0.8;
            if x * x + y * y < reach * reach && h.get("impactIn").num() < 1.4 {
                hazard = Some((x, y));
                break;
            }
        }
        let ability_in = r.get("abilityIn");
        let away = |t: View| obj! { "x" => -t.get("dx").num(), "y" => -t.get("dy").num() };
        if let Some((x, y)) = hazard {
            o.set("step", obj! { "x" => if x != 0.0 && !x.is_nan() { x } else { 1.0 }, "y" => if y != 0.0 && !y.is_nan() { y } else { 1.0 } });
            o.entry("power").set("drive", 900);
        } else if let (true, Some(t)) = (ty == "artillery" && ground >= 2.0 && pos.truthy() && r.get("deployed").truthy() && (!ability_in.truthy() || ability_in.get("shell").or(0.0) <= 0.0), shell_target) {
            let (bx, by) = (t.get("dx").num(), t.get("dy").num());
            let d = bx.hypot(by);
            let mut radius = 2.0;
            let mut clustered = 0;
            let mut friendly = false;
            for t in ns.iter() {
                let to_aim = (t.get("dx").num() - bx).hypot(t.get("dy").num() - by);
                if t.get("rival").truthy() && to_aim <= 3.2 {
                    clustered += 1;
                }
                if !t.get("rival").truthy() && to_aim <= 3.4 {
                    friendly = true;
                }
            }
            // Radius two is cheap. Increase it only for a safe, dense cluster.
            if !friendly && clustered >= 3 {
                radius = if clustered >= 5 { 4.0 } else { 3.0 };
            }
            let shell_cost = level * 1000.0 * (radius / 2.0_f64).powi(2) * (1.0 + max(d - 5.0, 0.0) * 0.12);
            if !friendly && d > radius + 0.7 && d <= 14.0 && shell_cost <= left && shell_cost <= ability_left {
                o.set("ability", "shell");
                o.set("aim", obj! { "x" => pos.get("x").num() + bx, "y" => pos.get("y").num() + by });
                o.set("radius", radius);
                o.set("hold", true);
                left -= shell_cost;
                ability_left -= shell_cost;
            }
        } else if ty == "artillery" && best.is_some() && !r.get("deployed").truthy() {
            o.set("deploy", true);
            o.set("hold", true);
        } else if heal_cost != 0.0 && !heal_cost.is_nan() {
            if let Some(t) = threat {
                o.set("step", away(t));
            }
        } else if matches!(ty, "bulwark" | "interceptor")
            && low
            && threat.is_some()
            && ability_left >= 900.0 * level
            && left >= 900.0 * level
            && modules.has_str("dash")
            && ability_in.get("dash").or(0.0) <= 0.0
        {
            if let Some(t) = threat {
                o.set("step", away(t));
            }
            o.set("ability", "dash");
            left -= 900.0 * level;
            ability_left -= 900.0 * level;
        } else if modules.truthy()
            && modules.has_str("scan")
            && ns.len() >= 2
            && ability_left >= 700.0 * level
            && left >= 700.0 * level
            && (!ability_in.truthy() || ability_in.get("scan").or(0.0) <= 0.0)
        {
            o.set("ability", "scan");
            left -= 700.0 * level;
            ability_left -= 700.0 * level;
        } else if let (Some(t), "artillery") = (threat, ty) {
            o.set("step", away(t));
        }
        let ability_before = o.get("ability").clone();
        if !matches!(ty, "miner" | "medic" | "scout") {
            common::close_melee(u, ns, &mut o);
            common::space_ranged(u, ns, &mut o);
        }
        if o.get("ability").str() == "dash" && ability_before.str() != "dash" {
            if ability_before.truthy() || 900.0 * level > min(left, ability_left) {
                o.set("ability", ability_before);
            } else {
                left -= 900.0 * level;
                ability_left -= 900.0 * level;
            }
        }
        common::steer_ground(u, ns, &mut o);
        if o.get("step").truthy() {
            let drive_cost = level * common::drive_tick(u, num(o.get("power").get("drive")));
            if drive_cost > left || drive_cost > drive_left {
                o.entry("power").set("drive", 0);
                if level * common::drive_tick(u, 0.0) > min(left, drive_left) {
                    o.set("step", Value::Null);
                    o.set("hold", true);
                } else {
                    left -= level * common::drive_tick(u, 0.0);
                    drive_left -= level * common::drive_tick(u, 0.0);
                }
            } else {
                left -= drive_cost;
                drive_left -= drive_cost;
            }
        }
        if let Some(t) = best {
            let pierce = pierce_mode(u, best);
            o.set("pierce", pierce);
            let shot_cost = (level * (120.0 + num(o.get("power").get("weapon"))) * (1.0 + 1.4 * pierce * pierce)).ceil();
            if shot_cost <= left && shot_cost <= weapon_left {
                o.set("hit", t.get("id"));
                left -= shot_cost;
                weapon_left -= shot_cost;
            }
            if !o.get("step").truthy() {
                o.set("hold", true);
            }
        }
        if best.is_some() || o.get("step").truthy() || o.get("ability").truthy() || o.get("deploy").truthy() {
            if cycles_left() < 700.0 { break; }
            order(o);
        }
        turn_cost = turn_cost.max(1.5 * (began - cycles_left()));
    }
    Vec::new()
}

// ---- Puzzles ----

// Salomon's House's solver: the founders' for the skill puzzles, and a shorter loop for a toll.
fn research(puzzle: View) -> Value {
    atlantis_ext::solve(puzzle)
}

fn duel(challenge: View) -> Value {
    atlantis_ext::solve(challenge)
}

fn challenge(foe: View, max: View) -> Value {
    common::challenge(foe, max)
}

// Each founder supplies its own body, weapon, projectile, anthem and capital.
fn look(budget: View, info: View) -> Value {
    let mut look=theme::of(info.get("seat"), 0.0).map_or(Value::Null, |t| t.look(budget.num(), info));
    if look.truthy(){let body=look.get("units").get("scout").clone();look.entry("units").set("aeronaut",body);look.entry("names").set("aeronaut","Salomon air wing");}
    look
}

fn aeronaut(units: View, world: View, me: View) { common::fly(units,world,me); }

fn build3d(budget: View, info: View) -> Value {
    theme::of(info.get("seat"), 0.0).map_or(Value::Null, |t| t.scene(info, budget.num()))
}

fn live3d(state: View, now: View) -> Value {
    let mut s = if state.truthy() { state.value() } else { Value::obj() };
    let spin = capital::ring(&mut s, now);
    obj! { "state" => s, "set" => obj! { "wonder" => spin } }
}

// ---- Salomon's House: research, its CPU lane, the Merchants of Light and the memo ----

mod atlantis_ext {
    use gameofcores::*;

    use super::common;
    use super::{due, text, Clock, ARMY, GOODS, TECHS};

    /// `Number(x)` for a value from the game: null is 0.
    fn n(v: View) -> f64 {
        if v.is_null() {
            0.0
        } else {
            v.num()
        }
    }

    /// Whether faction `f` holds tech `t` (`world.factions[i].techs` lists ids).
    fn holds(f: View, t: &str) -> bool {
        f.get("techs").iter().any(|x| x.str() == t)
    }

    /// Whether any faction but you holds `t`: a world first is gone.
    fn taken(world: View, me_id: f64, t: &str) -> bool {
        world.get("factions").iter().any(|f| f.get("id").num() != me_id && holds(f, t))
    }

    /// Whether you can start `t` now: you lack it, the world has it, and you hold what it requires.
    fn ready(all: View, mine: View, t: &str) -> bool {
        let x = all.get(t);
        !mine.get(t).truthy() && x.truthy() && x.get("requires").iter().all(|q| mine.get(q.str()).truthy())
    }

    /// A listed tech you can't start yet stands for its first missing prerequisite, and so on down the tree.
    fn first(all: View, mine: View, want: &str) -> String {
        let mut t = want.to_string();
        for _ in 0..8 {
            let x = all.get(&t);
            if !x.truthy() || mine.get(&t).truthy() {
                break;
            }
            match x.get("requires").iter().find(|r| !mine.get(r.str()).truthy()) {
                Some(q) => t = q.str().to_string(),
                None => break,
            }
        }
        t
    }

    /// Sets `orders.research` when the tech to study changes. A tier-1 or tier-2 tech under way goes on (12 or 60
    /// puzzles). Otherwise: the first ready tier-1 tech in the list; else the first ready tech of [`ARMY`]; else the
    /// first ready tier-2 tech no faction holds (a world first, ahead of a tier-3 tech under way, which keeps the
    /// puzzles it has solved); else the tech under way while it can go on; else the first tech in the list you can
    /// start, a listed tech you can't standing for its first missing prerequisite.
    pub fn choose_research(orders: &mut Value, world: View, me: View) {
        // Tabula builds a broad toolbox before specializing; Atlantis pursues world-first research below.
        if me.get("seat").num_or(-1.0)==9.0 {
            let mut choice=Value::obj();
            common::budget(&mut choice,world,me,&common::Budget{techs:&["extraction","ranged","armor","seaTraversal","repair","capture","influence","mountainTraversal","flight","artillery","antiAir","scan","dash","computeBoost","faith","cloak","pierce","airWeapons","stealth","gate"],..Default::default()});
            if choice.get("research").truthy(){orders.set("research",choice.get("research").clone());}
            return;
        }
        let (all, mine) = (world.get("techs"), me.get("techs"));
        let me_id = me.get("id").num();
        let now = me.path("research.tech").str().to_string();
        let tier = |t: &str| all.get(t).get("tier").num_or(9.0);
        let going = !now.is_empty() && ready(all, mine, &now);
        let pick = if going && tier(&now) <= 2.0 {
            now.clone()
        } else if let Some(t) = TECHS.iter().copied().find(|t| tier(t) <= 1.0 && ready(all, mine, t)) {
            t.to_string()
        } else if let Some(t) = ARMY.iter().copied().find(|t| ready(all, mine, t)) {
            t.to_string()
        } else if let Some(t) = TECHS.iter().copied().find(|t| tier(t) == 2.0 && ready(all, mine, t) && !taken(world, me_id, t)) {
            t.to_string()
        } else if going {
            now.clone()
        } else {
            let mut pick = String::new();
            for want in TECHS {
                let t = first(all, mine, want);
                if ready(all, mine, &t) {
                    pick = t;
                    break;
                }
            }
            pick
        };
        if !pick.is_empty() && pick != now {
            orders.set("research", pick);
        }
    }

    /// The research lane's limit for `compute.cpu`, in cycles a second (think sets it again every second). The CPU bank
    /// is the combat bank too, so research takes a share the army can live with: 30% of income while the faction is
    /// small (under 150,000 cycles a second, the founders' share); 40% once it's past that with four seconds of income
    /// in the bank (a tier-3 attempt costs 15,000 to 130,000 cycles, so three a second need about 230,000); 50% while
    /// the bank holds over ten seconds of income the army isn't spending; 20% while it holds under two seconds of
    /// income or invaders press (`pressure` 2 or more); 0 once every tech in the list is yours.
    pub fn research_limit(world: View, me: View, pressure: f64) -> Value {
        let (all, mine) = (world.get("techs"), me.get("techs"));
        if TECHS.iter().all(|t| mine.get(t).truthy() || !all.get(t).truthy()) {
            return Value::Num(0.0);
        }
        let income = me.path("rts.cpu.income").num_or(0.0).max(1.0);
        let bank = me.path("rts.cpu.bank").num_or(0.0);
        let now = me.path("research.tech").str();
        let offer = if now.is_empty() { 110_000.0 } else { all.get(now).get("offer").num_or(110_000.0) };
        let saved = me.path("research.bank").num_or(0.0);
        let tier1 = TECHS[..7].iter().any(|t| !mine.get(t).truthy());
        let fill = if pressure >= 2.0 || (!tier1 && bank < income * 2.0) {
            income * 0.2
        } else if bank > income * 10.0 && saved < offer * 1.4 {
            income * 0.5
        } else if income >= 150_000.0 && bank >= income * 4.0 {
            income * 0.4
        } else {
            income * 0.3
        };
        Value::Num(fill.round().max(2.0))
    }

    /// The memo spectators read: the tech under study and its count, the techs held, how many no one else has, and
    /// the tech deals running.
    pub fn memo(world: View, me: View) -> String {
        let (all, mine) = (world.get("techs"), me.get("techs"));
        let me_id = me.get("id").num();
        let held: Vec<&str> = TECHS.iter().copied().filter(|t| mine.get(t).truthy()).collect();
        let only = held.iter().filter(|t| !taken(world, me_id, t)).count();
        let deals = me
            .get("contracts")
            .iter()
            .filter(|c| !c.get("ended").truthy() && (c.get("give").get("techs").truthy() || c.get("give").get("techLease").truthy()))
            .count();
        let t = me.path("research.tech").str();
        let study = if t.is_empty() {
            "Salomon's House has researched every tech.".to_string()
        } else {
            let done = me.path("research.progress").get(t).num_or(0.0);
            format!("Salomon's House studies {}: {} of {} puzzles solved.", all.get(t).get("name").str(), text(done), text(n(all.get(t).get("need"))))
        };
        format!("{study} {} techs, {only} that no one else holds. {deals} tech deals running.", held.len())
    }

    /// The Merchants of Light: every minute or so, one tech from [`GOODS`] offered to one faction. Allies first, then
    /// the smallest; only factions under half your share, never the runaway leader, and never a faction with units in
    /// your land. The dearest such tech it lacks and can take (its prerequisites held): a first-tier tech is sold for
    /// good, 60 ◆; a tech above is leased for five minutes, 40 ◆. The founders' deal rule values a tech sold at 150 ◆
    /// and a lease at a quarter ◆ a second, so a buyer with the ◆ takes it.
    pub fn merchants(orders: &mut Value, world: View, me: View, clock: &mut Clock) {
        let now = world.get("seconds").or(0.0).floor();
        if !due(clock, "merchants", 60.0, now) {
            return;
        }
        let me_id = me.get("id").num();
        let (all, mine) = (world.get("techs"), me.get("techs"));
        let lead = common::dominant(world).map(|l| l.get("id").num());
        let ours: Vec<View> = world.get("provinces").iter().filter(|p| p.get("owner").num() == me_id).collect();
        let my_share = world.get("factions").key(me_id).get("share").num_or(0.0);
        let mut buyers: Vec<(f64, View)> = Vec::new();
        for f in world.get("factions").iter() {
            let id = f.get("id").num();
            if id == me_id
                || !f.get("alive").truthy()
                || lead == Some(id)
                || f.get("share").num_or(0.0) >= my_share * 0.5
                || ours.iter().any(|p| p.get("units").key(id).or(0.0) > 0.0)
            {
                continue;
            }
            let ally = if me.get("allies").has_num(id) { 1.0 } else { 0.0 };
            buyers.push((ally - f.get("share").num_or(0.0) * 3.0 + 0.1 * rand(), f));
        }
        buyers.sort_by(|a, b| b.0.total_cmp(&a.0));
        let tier = |t: &str| all.get(t).get("tier").num_or(9.0);
        for (_, f) in buyers {
            let can = |t: &str| mine.get(t).truthy() && !holds(f, t) && all.get(t).truthy() && all.get(t).get("requires").iter().all(|q| holds(f, q.str()));
            let Some(t) = GOODS.iter().copied().filter(|t| can(t)).max_by(|a, b| tier(a).total_cmp(&tier(b))) else { continue };
            let name = all.get(t).get("name").str().to_string();
            let to = f.get("id").num();
            if tier(t) >= 2.0 {
                trade::sell_tech(orders, to, t, obj! { "coin" => 40 }, 300.0, format!("Salomon's House lends {name} for five minutes"));
            } else {
                trade::sell_tech(orders, to, t, obj! { "coin" => 60 }, 0.0, format!("Salomon's House sells {name}"));
            }
            return;
        }
    }

    // ---- puzzles ----

    /// The answer to a research or duel puzzle: the founders' solver, except that a toll is answered by [`toll`], the
    /// same search in a shorter loop.
    pub fn solve(c: View) -> Value {
        if c.get("kind").str() == "toll" {
            return Value::Num(toll(c.get("seed").num(), c.get("bits").num()));
        }
        common::solve(c)
    }

    /// The game's `hash(a, b)` mixes each argument's bits with splitmix64's finaliser.
    fn mix(mut z: u64) -> u64 {
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A nonce whose `hash(seed, nonce)` starts with `bits` zero bits: the plain search, about 2^bits probes whatever
    /// the code, in a loop that mixes the seed once and tests the high word's zero bits with one shift (the founders'
    /// `common::toll` mixes the seed again and converts the hash on every probe). The nonce is a whole number below
    /// 2^32. Short of cycles it answers -1 (a wrong answer costs only what ran). The nonce is checked with the SDK's
    /// `hash` before it's returned; if the game's hash isn't this one, `common::toll` answers instead.
    pub fn toll(seed: f64, bits: f64) -> f64 {
        if !(1.0..=32.0).contains(&bits) || bits.fract() != 0.0 {
            return common::toll(seed, bits);
        }
        let shift = 64 - bits as u32;
        let x0 = mix(seed.to_bits() ^ 0x9E37_79B9_7F4A_7C15);
        let mut x = 0.0f64;
        let mut i: u32 = 0;
        loop {
            if mix(x0 ^ x.to_bits()) >> shift == 0 {
                break;
            }
            x += 1.0;
            i = i.wrapping_add(1);
            if (i & 4095) == 0 && (cycles_left() < 2000.0 || x >= 4_294_967_296.0) {
                return -1.0;
            }
        }
        if hash(seed, x).leading_zeros() >= bits as u32 {
            x
        } else {
            common::toll(seed, bits)
        }
    }
}
