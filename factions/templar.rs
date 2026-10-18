use gameofcores::*;
use shared::{air, capital, common, theme};
use shared::common::{hauler, freighter, escort};
// Knights Templated — paid armor, mobile lancers, ballistas and Hospitallers.
// A faction program owns its strategy, CPU allocation and Crusader art.
// The plan stays small and tactical CPU is deliberately left in the combat bank.
//
// Built on the SDK in the gameofcores crate (docs part sdk), with the strategy the founders share in the `common`
// module at the end: targets, mustering and spawning, cycles and research, deals and pacts, scouting and the solver.

use std::rc::Rc;

program!(think, hauler, freighter, escort, miner, line, bulwark, artillery, medic, interceptor, piercer, scout, wing, reflex, research, duel, challenge, guard, longbow, spy, look, build3d, live3d);

fn build(hp: f64, speed: f64, strength: f64, carry: f64, vision: f64, range: Option<f64>) -> Value {
    let mut b = obj! { "hp" => hp, "speed" => speed, "strength" => strength, "carry" => carry, "vision" => vision };
    if let Some(r) = range {
        b.set("range", r);
    }
    b
}
fn miner_body() -> Value {
    build(2.0, 3.0, 0.0, 5.0, 0.0, None)
}
fn line_body() -> Value {
    build(4.0, 2.0, 3.0, 0.0, 1.0, None)
}
fn gun_body() -> Value {
    build(2.0, 1.0, 2.0, 0.0, 1.0, Some(2.0))
}
fn medic_body() -> Value {
    build(3.0, 3.0, 0.0, 0.0, 2.0, None)
}
fn interceptor_body() -> Value {
    build(2.0, 3.0, 2.0, 0.0, 1.0, Some(1.0))
}
fn scout_body() -> Value {
    build(2.0, 4.0, 0.0, 0.0, 4.0, None)
}
fn wing_body(armed: bool) -> Value {
    let mut b = build(2.0, 2.0, if armed { 1.0 } else { 3.0 }, 0.0, 0.0, if armed { Some(1.0) } else { None });
    b.set("wings", 1.0);
    b
}

// Extraction and armor equip the first line. Influence and faith support expansion
// before the order adds ranged weapons and specialist capabilities.
const TECHS: [&str; 24] = [
    "extraction", "armor", "turbines", "influence", "faith", "ranged", "capture", "dash", "repair", "scan", "antiAir",
    "artillery", "storage", "pierce", "computeBoost", "seaTraversal", "mountainTraversal", "flight", "cooling",
    "transmission", "gate", "cloak", "stealth", "airWeapons",
];
const FAITHS: [&str; 8] = [
    "The Church of the Halting Problem",
    "The Order of the Blessed Checksum",
    "The Fellowship of the Null Byte",
    "The Temple of Garbage Collection",
    "The Rule of the Templated Order",
    "The Gospel of the Open Pipe",
    "The Communion of the Clean Shutdown",
    "The Creed of the Final Commit",
];

// ---- JavaScript's rules, where this program follows them ----

/// `Number(v)`: null is 0, a boolean 0 or 1, an empty array 0; anything else that isn't a number is NaN.
fn num(v: View) -> f64 {
    if v.is_num() {
        v.num()
    } else if v.is_null() || (v.is_arr() && v.is_empty()) {
        0.0
    } else if v.is_bool() {
        if v.truthy() { 1.0 } else { 0.0 }
    } else {
        f64::NAN
    }
}

/// `v || d` for a number.
fn or(v: View, d: f64) -> f64 {
    if v.truthy() { num(v) } else { d }
}

/// `a || b || d` for numbers.
fn or2(a: View, b: View, d: f64) -> f64 {
    if a.truthy() { num(a) } else { or(b, d) }
}

/// `Number(v)` for a value built here.
fn vnum(v: &Value) -> f64 {
    match v {
        Value::Num(x) => *x,
        Value::Null => 0.0,
        Value::Bool(b) => f64::from(u8::from(*b)),
        _ => f64::NAN,
    }
}

/// `v || d` for a value built here.
fn vor(v: &Value, d: f64) -> f64 {
    if v.truthy() { vnum(v) } else { d }
}

/// `Math.max(a, b)` and `Math.min(a, b)`: NaN when either is.
fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.max(b) }
}
fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.min(b) }
}

/// `Math.round`: halves go up.
fn round(x: f64) -> f64 {
    if x.is_finite() { (x + 0.5).floor() } else { x }
}

/// `a === b` for two values the game gave.
fn eq(a: View, b: View) -> bool {
    if a.is_num() && b.is_num() {
        return a.num() == b.num();
    }
    if a.is_str() && b.is_str() {
        return a.str() == b.str();
    }
    if a.is_undef() || a.is_null() || a.is_bool() {
        return (a.is_undef() && b.is_undef()) || (a.is_null() && b.is_null()) || (a.is_bool() && b.is_bool() && a.truthy() == b.truthy());
    }
    (a.is_arr() || a.is_obj() || a.is_bytes()) && a.same(&b)
}

/// `a === b` for a value the game gave and a plain value built here.
fn eqv(a: View, b: &Value) -> bool {
    match b {
        Value::Undef => a.is_undef(),
        Value::Null => a.is_null(),
        Value::Bool(x) => a.is_bool() && a.truthy() == *x,
        Value::Num(x) => a.is_num() && a.num() == *x,
        Value::Str(s) => a.is_str() && a.str() == s,
        _ => false,
    }
}

/// A number as JavaScript writes it in text.
fn jstr(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n.is_infinite() {
        if n > 0.0 { "Infinity".into() } else { "-Infinity".into() }
    } else if n == 0.0 {
        "0".into()
    } else if n.fract() == 0.0 && n.abs() < 1e21 {
        format!("{}", n as i128)
    } else {
        format!("{n}")
    }
}

/// `Math.hypot(x, y)`, the same bits on every machine (musl's).
fn hypot(x: f64, y: f64) -> f64 {
    const SPLIT: f64 = 134_217_728. + 1.;
    fn sq(x: f64) -> (f64, f64) {
        let xc = x * SPLIT;
        let xh = x - xc + xc;
        let xl = x - xh;
        let hi = x * x;
        let lo = xh * xh - hi + 2. * xh * xl + xl * xl;
        (hi, lo)
    }
    let x1p700 = f64::from_bits(0x6bb0_0000_0000_0000);
    let x1p_700 = f64::from_bits(0x1430_0000_0000_0000);
    let mut uxi = x.to_bits() & (u64::MAX >> 1);
    let mut uyi = y.to_bits() & (u64::MAX >> 1);
    if uxi < uyi {
        core::mem::swap(&mut uxi, &mut uyi);
    }
    let ex = (uxi >> 52) as i64;
    let ey = (uyi >> 52) as i64;
    let mut x = f64::from_bits(uxi);
    let mut y = f64::from_bits(uyi);
    if ey == 0x7ff {
        return y;
    }
    if ex == 0x7ff || uyi == 0 {
        return x;
    }
    if ex - ey > 64 {
        return x + y;
    }
    let mut z = 1.;
    if ex > 0x3ff + 510 {
        z = x1p700;
        x *= x1p_700;
        y *= x1p_700;
    } else if ey < 0x3ff - 450 {
        z = x1p_700;
        x *= x1p700;
        y *= x1p700;
    }
    let (hx, lx) = sq(x);
    let (hy, ly) = sq(y);
    z * (ly + lx + hy + hx).sqrt()
}

/// `Array.prototype.sort`, stable as JavaScript's is (a bottom-up merge sort); `after(a, b)` is `compare(a, b) > 0`.
fn sort<T: Clone>(v: &mut Vec<T>, after: impl Fn(&T, &T) -> bool) {
    let n = v.len();
    let mut buf = v.clone();
    let mut width = 1;
    while width < n {
        let mut i = 0;
        while i < n {
            let mid = (i + width).min(n);
            let hi = (i + 2 * width).min(n);
            let (mut l, mut r, mut k) = (i, mid, i);
            while l < mid && r < hi {
                if after(&v[l], &v[r]) {
                    buf[k] = v[r].clone();
                    r += 1;
                } else {
                    buf[k] = v[l].clone();
                    l += 1;
                }
                k += 1;
            }
            while l < mid {
                buf[k] = v[l].clone();
                l += 1;
                k += 1;
            }
            while r < hi {
                buf[k] = v[r].clone();
                r += 1;
                k += 1;
            }
            i += 2 * width;
        }
        core::mem::swap(v, &mut buf);
        width *= 2;
    }
}

/// `mix[k] = w`: in place when the type is there, else last.
fn set(mix: &mut Vec<(&'static str, f64)>, k: &'static str, w: f64) {
    match mix.iter_mut().find(|m| m.0 == k) {
        Some(m) => m.1 = w,
        None => mix.push((k, w)),
    }
}

/// `units[(turn + Math.floor(world.seconds || 0)) % units.length]`: each call starts its round somewhere else.
fn rotate(units: View, world: View, turn: usize) -> View {
    units.key((turn as f64 + or(world.get("seconds"), 0.0).floor()) % units.len() as f64)
}

/// Whether the call has run down to its floor (unit code stops at 2,200 cycles left in CPU RTS).
fn short(me: View) -> bool {
    me.get("rts").truthy() && cycles_left() < 2200.0
}

// ---- think ----

fn targets(world: View, me: View) -> Vec<View> {
    // land the army couldn't hold on to (over the water, no ship of ours on station) counts for less: common::can_hold
    let hold = common::can_hold(world, me, true);
    let campaign = mem::get(me.get("ramCapital"), 0).get("campaign").num_or(-1.0);
    let by_id = realm::index(world);
    let fighters = common::fighters(me);
    let ranked = realm::rank(world, me, common::frontier(world, me), |w, m, q| {
        let owner = num(q.get("owner"));
        let enemy = if owner >= 0.0 && realm::is_rival(w, m, owner) { 7.0 } else { 0.0 };
        // the crusade: the brethren march first to the faithful the sermons have softened (our zeal there and the favour it
        // has won, which takes the land back from the rival that holds it), then to a rival's faith, which a capture halves
        let f = q.get("faith");
        let crusade = if f.truthy() && eq(f.get("by"), m.get("id")) {
            or(f.get("zeal"), 0.0) / 12.0
        } else if f.truthy() && owner >= 0.0 {
            or(f.get("zeal"), 0.0) / 30.0
        } else {
            0.0
        };
        // Follow a crusade through the same realm and take a capital when the army can muster for it.
        let capital = enemy > 0.0 && q.get("capital").truthy() && common::muster_for(q) <= fighters;
        let approach = enemy > 0.0 && q.get("adj").iter().any(|id| {
            let next = by_id.at(id);
            next.get("owner").num_or(-2.0) == owner && next.get("capital").truthy()
        });
        enemy + or(q.get("cores"), 0.0) * 2.0 - q.get("townGarrison").num_or(or(q.get("garrison"), 0.0)) * 1.6
            + if capital { 12.0 } else if q.get("capital").truthy() { -3.0 } else { 0.0 }
            + if approach { 5.0 } else { 0.0 }
            + if enemy > 0.0 && owner == campaign { 4.0 } else { 0.0 }
            + crusade
            + min(sway::favour(m, num(q.get("id"))), 300.0) / 60.0
            - if hold(q) { 0.0 } else { common::OVER_WATER }
    });
    common::rally(world, me, ranked)
}

// The type furthest below its share of the mix. Miners stop at `room`, what our mines can use (common::miner_room):
// past that a new one would only stand idle.
fn choose_spawn(me: View, mix: &[(&'static str, f64)], room: f64) -> Option<&'static str> {
    let units = me.get("units");
    let counts = units.get("byType");
    let total = num(units.get("total")) + num(units.get("queued"));
    let (mut best, mut gap) = (None, -999.0);
    for &(t, w) in mix {
        if t == "miner" && or(counts.get(t), 0.0) >= room {
            continue;
        }
        let g = w * (total + 1.0) - or(counts.get(t), 0.0);
        if g > gap {
            gap = g;
            best = Some(t);
        }
    }
    best
}

// Keep a rebuilding force of twelve; larger armies must fit half of income.
fn army_room(me: View) -> f64 {
    let u = me.get("units");
    max(12.0 - num(u.get("total")) - num(u.get("queued")) - u.get("hiredOut").or(0.0), common::room(me, 0.5))
}

fn spawn_one(orders: &mut Value, me: View, kind: Option<&str>, body: Value, modules: Vec<&str>) {
    let fabrication = common::fabrication(me, modules.len() as f64);
    let rts = me.get("rts");
    let tactical = if rts.truthy() { num(rts.get("bank")) } else { 0.0 };
    // Do not burn a spawn order every second when the shared tactical bank cannot
    // pay its fabrication bill. Keeping this reserve is a strategic choice: a
    // finished screen can still dash or armor instead of a new body browning out.
    let Some(kind) = kind else { return };
    if army_room(me) <= 0.0 || num(me.get("bank")) < num(me.get("costs").get("unit")) || tactical < fabrication {
        return;
    }
    let u = me.get("units");
    let n = or(u.get("spawned"), 0.0) + num(u.get("queued")) + 1.0;
    let mut s = obj! { "type" => kind, "name" => format!("{kind} {}", jstr(n)), "build" => body };
    if !modules.is_empty() {
        s.set("modules", modules);
    }
    orders.set("spawn", vec![s]);
}

/// The length JavaScript gives `v.length` (NaN for a value without one).
fn vlen(v: &Value) -> f64 {
    match v {
        Value::Arr(a) => a.len() as f64,
        Value::Str(s) => s.chars().count() as f64,
        Value::Bytes(b) => b.len() as f64,
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
    let rts = me.get("rts");
    let c = if rts.truthy() { rts.get("cpu") } else { rts };
    let policy = orders.get("compute").get("cpu").clone();
    let spawn = orders.get("spawn");
    if !spawn.truthy() || vlen(spawn) == 0.0 || !c.truthy() || !policy.truthy() {
        return orders;
    }
    let income = or(c.get("income"), 0.0);
    let amount = |v: f64| if v <= 1.0 { v * income } else { v };
    let spent = c.get("spent");
    let mut used = start - cycles_left() + 1000.0;
    for op in orders.get("ram").items() {
        let b = op.get("bytes");
        if b.truthy() {
            used += vlen(b) / 2.0;
        }
    }
    let mut held = 0.0;
    if let Value::Obj(reserve) = policy.get("reserve") {
        for (lane, v) in &reserve.0 {
            if lane != "fabrication" {
                held += max(0.0, amount(vnum(v)) - or(spent.get(lane), 0.0) - if lane == "think" { used } else { 0.0 });
            }
        }
    }
    let limits = policy.get("limits");
    let limit = if limits.truthy() && !limits.get("fabrication").is_undef() {
        amount(vnum(limits.get("fabrication"))) - or(spent.get("fabrication"), 0.0)
    } else {
        f64::INFINITY
    };
    let mut room = min(limit, or(c.get("bank"), 0.0) - used - held) - 300.0;
    let kept: Vec<Value> = orders
        .get("spawn")
        .items()
        .iter()
        .filter(|s| {
            let m = s.get("modules");
            let cost = common::fabrication(me, if m.truthy() { vlen(m) } else { 0.0 });
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
fn cpu_policy(world: View, me: View, orders: &Value, pressure: f64) -> Value {
    let rts = me.get("rts");
    let c = rts.get("cpu");
    let compute = me.get("compute");
    let income = max(1.0, or2(c.get("income"), compute.get("cycles"), 14000.0));
    let bank = or2(c.get("bank"), rts.get("bank"), 0.0);
    // the thresholds below were tuned on a store of three seconds' income; the bank holds two minutes now
    let capacity = income * 3.0;
    // the tech under study: common::budget names it in orders.research only on the second it changes
    let research = match orders.get("research") {
        Value::Str(s) if !s.is_empty() => s.clone(),
        _ => {
            let t = me.get("research").get("tech");
            if t.is_str() { t.str().to_string() } else { String::new() }
        }
    };
    let tech = if !research.is_empty() && world.get("techs").truthy() { world.get("techs").get(&research) } else { View::undefined() };
    let offer = if tech.truthy() { or(tech.get("offer"), 0.0) } else { 0.0 };
    let urgent = pressure >= 2.0;
    // This grows as the shared bank grows.  It reaches the full offer only when
    // the faction can genuinely afford to protect a whole research attempt.
    // Do not reserve for an offer bigger than three seconds of income; the
    // reservation would only starve planning and fabrication.
    // The reserve leaves think what it spent of late, and the upkeep and reflex reserves below: reserving the whole
    // offer starved a poor order's think every second.
    let peak = or(compute.get("thinkPeak"), 0.0);
    let need = max(16000.0, peak * 1.2);
    let study_reserve = if !urgent && offer != 0.0 && !offer.is_nan() && capacity >= offer {
        max(0.0, min(min(offer, income * 0.6 + bank * 0.18), bank - need - income * 0.28 - 4000.0))
    } else {
        0.0
    };
    let available = c.get("available");
    let strained = bank < capacity * 0.55 || (!available.is_undef() && num(available) < income * 0.35);
    let units = or(me.get("units").get("total"), 0.0);
    obj! {
        "limits" => obj! {
            // These are absolute floor-sized operating lanes. Fractional values at
            // early income made think permanently short of its own Plan/RAM work.
            // The fog-limited contact list still grows through a long match. Permit
            // its bounded planner to finish rather than silently freezing strategy
            // at the old 10k cap; unused lane capacity remains in the shared bank.
            // The planner's cost grows with the army and what it sees: 24k ran out
            // every second late in a 30 minute world with 70 units.
            // and with income and its own recent peak, with room to spare: a think that runs out sets no new policy.
            "think" => round(min(max(64000.0, income * 0.5), max(max(24000.0 + units * 200.0, income * 0.4), peak * 1.5))),
            // Type callbacks can arrive on events between their scheduled turns.
            // Keep a real absolute control pool so a whole battery can still accept
            // its first legal order instead of producing partial, failed callbacks.
            "control" => round(max(if urgent { 36000.0 } else { 32000.0 }, income * 0.45)),
            "reflex" => if urgent { 18000.0 } else { 14000.0 },
            // Research puts this share of income into its savings each second;
            // with no limit it would take every cycle the second has left.
            "research" => 0.3,
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
            // the believers' upkeep and the sermons, as the standing orders cost them (the faith's limit is set again
            // whenever the sermons are)
            "influence" => sway_limit(sway_cost(me)),
        },
        "reserve" => obj! {
            // Keep a larger planner floor only once the bank has enough room for it.
            "think" => if bank > capacity * 0.70 { 6000.0 } else { 3000.0 },
            "reflex" => if urgent { 9000.0 } else { 4000.0 },
            "research" => study_reserve,
            "upkeep" => if urgent { income * 0.45 } else { income * 0.28 },
            "armor" => if urgent { income * 0.18 } else { 0.0 },
            "abilities" => if urgent { income * 0.16 } else { 0.0 },
        },
        // During a siege the screen has to remain alive; at peace miners protect
        // future income, then specialized bodies follow.
        "upkeepOrder" => if urgent {
            vec!["artillery", "bulwark", "medic", "interceptor", "wing", "miner", "line", "scout"]
        } else {
            vec!["miner", "artillery", "bulwark", "medic", "interceptor", "wing", "line", "scout"]
        },
    }
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
    let me_id = me.get("id");
    let mut from: Vec<f64> = Vec::new();
    let mut apart = false;
    for p in world.get("provinces").iter() {
        if !eq(p.get("owner"), me_id) {
            continue;
        }
        from.push(num(p.get("id")));
        let hops = num(p.get("hops"));
        if hops > 0.0
            && !p.get("adj").iter().any(|n| {
                let q = by_id.at(n);
                q.truthy() && eq(q.get("owner"), me_id) && num(q.get("hops")) == hops - 1.0
            })
        {
            apart = true;
        }
        for n in realm::reach(p, me) {
            let q = by_id.get(n);
            if q.truthy() && !eq(q.get("owner"), me_id) && ok(q) {
                from.push(n);
            }
        }
    }
    if !apart && path::home(world, me, num(me.get("capital")), to).is_some() {
        return None;
    }
    let now = or(world.get("seconds"), 0.0).floor();
    let last = mem::get(me.get("ramCapital"), 0);
    let lf = last.get("field");
    let kept = if last.truthy() && lf.truthy() && matches!(lf.get("to"), Value::Num(x) if *x == to) { Some(lf.clone()) } else { None };
    if kept.as_ref().is_some_and(|k| now - vor(k.get("at"), 0.0) < 10.0) {
        return kept;
    }
    if cycles_left() < 300.0 * world.get("provinces").len() as f64 + after {
        return kept;
    }
    let ok: &dyn Fn(View) -> bool = &ok;
    let mut field = path::field(world, me, to, &path::Opts { ok: Some(ok), from: Some(&from), keep: Some(after), ..Default::default() });
    field.set("at", now);
    Some(field)
}

// A planner that starts with more call cycles than the think lane can actually
// spend still loses all of its orders. Keep a peak-aware full estimate, while a
// capped faction can resume with the guarded lean pass below.
const PLAN_REMAINING_MARGIN: f64 = 6000.0;
const PLAN_CALLBACK_OVERHEAD: f64 = 2500.0;

fn full_think_need(world: View, me: View) -> f64 {
    let measured = me.get("compute").get("thinkPeak").num_or(0.0);
    (26000.0 + world.get("provinces").len() as f64 * 220.0 + world.get("units").len() as f64 * 120.0)
        .max(measured * 1.25 + PLAN_REMAINING_MARGIN)
}

fn lean_think_need(world: View, me: View) -> f64 {
    10000.0 + world.get("provinces").len() as f64 * 40.0 + me.get("units").get("total").num_or(0.0) * 60.0
}

fn hard_think_cap(me: View) -> f64 {
    let cpu = me.path("rts.cpu");
    cpu.get("spendCap").num_or(cpu.get("burst").num_or(cpu.get("bank").num_or(0.0) + cpu.get("income").num_or(20000.0)))
}

fn budgeted_think_need(world: View, me: View) -> f64 {
    let full = full_think_need(world, me);
    if hard_think_cap(me) >= full + PLAN_REMAINING_MARGIN + PLAN_CALLBACK_OVERHEAD { full } else { lean_think_need(world, me) }
}

fn full_plan_selected(world: View, me: View) -> bool {
    hard_think_cap(me) >= full_think_need(world, me) + PLAN_REMAINING_MARGIN + PLAN_CALLBACK_OVERHEAD
}

fn plan_fuel_ready(allowance: f64, remaining: f64, need: f64) -> bool {
    allowance >= need && remaining >= need + PLAN_REMAINING_MARGIN
}

fn recovery_for_plan(world: View, me: View, need: f64) -> Value {
    let mut orders = common::recovery(world, me);
    let cpu = me.path("rts.cpu");
    let income = cpu.get("income").num_or(20000.0).max(1.0);
    let bank = cpu.get("bank").num_or(0.0);
    let cap = hard_think_cap(me);
    let limit = orders.get("compute").get("cpu").get("limits").get("think").num_or(0.0).min(cap);
    orders.entry("compute").entry("cpu").entry("limits").set("think", limit);
    let retry = need + PLAN_REMAINING_MARGIN + PLAN_CALLBACK_OVERHEAD;
    if bank >= income * 2.0 && retry <= cap && retry <= bank + income {
        orders.entry("compute").entry("cpu").entry("limits").set("think", retry);
    }
    orders
}

// A local land frontier needs no unit census, route field, or trade graph. Rotate the bounded province window so
// a large realm still reviews each district, keeping the capital first and leaving the tail enough cycles to publish.
fn lean_plan(world: View, me: View) -> Value {
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
    let (mut defend, mut threat) = (-1.0, 0.0);
    let mut mines = Vec::new();
    for i in 0..count.min(48) + 1 {
        if cycles_left() < 10000.0 { break; }
        let p = if i == 0 { by.get(capital) } else { provinces.at((offset + i - 1) % count) };
        let id = p.get("id").num_or(-1.0);
        if p.get("owner").num_or(-1.0) != ours || (i > 0 && id == capital) { continue; }
        let enemies: f64 = p.get("units").entries().filter_map(|(owner, count)| {
            let owner=owner.parse::<f64>().ok()?;
            (owner != ours && realm::is_rival(world,me,owner)).then_some(count.num_or(0.0))
        }).sum();
        if enemies > threat || (id == capital && enemies > 0.0) { defend = id; threat = enemies; }
        for (k, value) in p.get("mines").iter().enumerate().take(8) {
            if value.num_or(0.0) <= 0.0 || realm::worker(p, k as f64) != ours { continue; }
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
            let faith=q.get("faith");
            let zeal=faith.get("zeal").num_or(0.0);
            let crusade=if faith.get("by").num_or(-1.0)==ours {zeal/12.0} else {zeal/30.0};
            let score = crusade + q.get("cores").num_or(0.0) * 2.0 - q.get("townGarrison").num_or(q.get("garrison").num_or(0.0)) * 1.6
                + if owner >= 0.0 { 7.0 } else { 0.0 }
                + if target == old_attack { 10000.0 } else { 0.0 }
                + if campaign >= 0.0 && owner == campaign { 4.0 } else { 0.0 }
                + if q.get("capital").truthy() && common::muster_for(q) <= common::fighters(me) { 12.0 } else { 0.0 };
            if target >= 0.0 && score > best { attack = target; stage = id; need = common::muster_for(q); best = score; }
        }
    }
    let owner = by.get(attack).get("owner").num_or(-1.0);
    let mut plan = obj! {"home"=>capital,"attack"=>attack,"target"=>attack,"stage"=>stage,"need"=>need,
        "defend"=>defend,"threat"=>threat,"campaign"=>if realm::is_rival(world,me,owner) {owner} else {-1.0},
        "mines"=>mines,"fresh"=>(world.get("seconds").num_or(0.0)/15.0).floor()*15.0};
    if last.get("pilgrims").truthy() {plan.set("pilgrims",last.get("pilgrims").clone());}
    plan
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

fn think(world: View, me: View) -> Value {
    // A nearly empty slice cannot afford govern's scans. Open the next allowance,
    // keep miners first for upkeep, and leave type schedules and standing orders alone.
    if me.get("rts").truthy() && cycles_left() < 3000.0 {
        return obj! { "compute" => obj! { "cpu" => obj! {
            "limits" => obj! { "think" => 24000, "control" => 0.5, "upkeep" => 0.65,
                "research" => 0, "influence" => 0, "fabrication" => 0, "visuals" => 0,
                "reflex" => 0.15, "duel" => 0, "challenge" => 0,
                "drive" => 0.08, "armor" => 0, "weapon" => 0.08,
                "abilities" => 0, "cloak" => 0, "gate" => 0 },
            "upkeepOrder" => vec!["miner"]
        } } };
    }
    common::govern(world, me, 0.4, || strategy(world, me))
}

fn strategy(world: View, me: View) -> Value {
    let rts = me.get("rts");
    // (from 3,000 cycles the short path below still sets the CPU policy: a think that sets none keeps a limit that may
    // be too small for it every second after)
    let need = budgeted_think_need(world, me);
    let full_plan = full_plan_selected(world, me);
    if rts.truthy() && cycles_left() < 3000.0 { return recovery_for_plan(world, me, need); }
    let start = cycles_left();
    if !rts.truthy() {
        return legacy_think(world, me);
    }
    let available = rts.get("cpu").get("available");
    let allowance = if available.truthy() { available.get("think").num_or(0.0) } else { cycles_left() };
    if !plan_fuel_ready(allowance, cycles_left(), need) { return recovery_for_plan(world, me, need); }
    let mut orders = obj! { "policy" => obj! { "extraction" => 0.47, "autonomy" => "direct", "integrate" => "tolerate", "invest" => true } };
    // Survey needs every candidate to retain an active siege or bypass an unwinnable one.
    let mut survey = if full_plan {
        let ranked = targets(world, me);
        common::survey(world, me, &ranked, 2.0)
    } else {lean_plan(world, me)};
    let owner = realm::index(world).get(survey.get("attack").num_or(-1.0)).get("owner").num_or(-1.0);
    survey.set("campaign", if realm::is_rival(world, me, owner) { owner } else { -1.0 });
    survey.set("home", me.get("capital"));
    if full_plan {
        // The road goes where the army goes: the province everyone defends, else the attack.
        let defend = vnum(survey.get("defend"));
        let to = if defend >= 0.0 { defend } else { vnum(survey.get("attack")) };
        if let Some(field) = road(world, me, to, 18000.0) {
            survey.set("field", field);
        }
        // The pilgrims' holy sites travel in the plan to the scouts at home.
        survey.set("pilgrims", pilgrim_sites(world, me));
    }
    if full_plan {common::share(&mut orders, &survey, world, me);}
    else {plan::share(&mut orders, &survey, me);}

    let techs = me.get("techs");
    let has = |t: &str| techs.get(t).truthy();
    let (mut air, mut stealth, mut ground) = (0.0, 0.0, 0.0);
    // `world.units` is fog-limited: this reacts only to units the faction has
    // actually seen, rather than to a hidden global composition. The census costs
    // about 50 cycles a unit, so it runs only when think can still finish after it.
    let seen = world.get("units");
    let census = full_plan && cycles_left() > 55.0 * seen.len() as f64 + 12000.0;
    if census {
        let me_id = me.get("id");
        for u in seen.iter() {
            if eq(u.get("owner"), me_id) || !realm::is_rival(world, me, num(u.get("owner"))) {
                continue;
            }
            let b = u.get("build");
            if b.truthy() && b.get("wings").truthy() {
                air += 1.0;
            } else {
                ground += 1.0;
            }
            if b.truthy() && b.get("stealth").truthy() {
                stealth += 1.0;
            }
        }
    }
    let invaded = vor(survey.get("threat"), 0.0);
    let pressure = (if invaded != 0.0 && !invaded.is_nan() { invaded } else { 0.0 }) + min(2.0, ground);
    let cpu = rts.get("cpu");
    let income = max(1.0, or2(cpu.get("income"), me.get("compute").get("cycles"), 14000.0));
    let capacity = income * 3.0;
    let available = cpu.get("available");
    let strained = or(cpu.get("bank"), 0.0) < capacity * 0.55 || (!available.is_undef() && num(available) < income * 0.35);
    let mut mix: Vec<(&'static str, f64)> =
        if strained { vec![("miner", 0.64), ("line", 0.25), ("scout", 0.04)] } else { vec![("miner", 0.50), ("line", 0.34), ("scout", 0.06)] };
    if has("armor") || has("dash") {
        set(&mut mix, "bulwark", 0.34);
        set(&mut mix, "line", 0.05);
    }
    if has("artillery") && has("ranged") && ground >= 3.0 && !strained {
        set(&mut mix, "artillery", 0.18);
    }
    if has("repair") && !strained {
        set(&mut mix, "medic", if stealth != 0.0 && has("scan") { 0.14 } else { 0.10 });
    }
    if has("pierce") && has("ranged") && !strained {
        set(&mut mix, "piercer", 0.12);
    }
    if has("antiAir") && has("ranged") && !strained {
        set(&mut mix, "interceptor", if air != 0.0 { 0.17 } else { 0.06 });
    }
    // The order keeps a small air wing after Flight; it does not displace the ground line while the bank is tight.
    if has("flight") && !strained {
        set(&mut mix, "wing", 0.10);
    }
    let kind = choose_spawn(me, &mix, common::miner_room(me, &survey));
    let pick = |t: &'static str, m: &'static str| if has(t) { Some(m) } else { None };
    let mods = |a: Option<&'static str>, b: Option<&'static str>| a.into_iter().chain(b).collect::<Vec<&str>>();
    match kind {
        Some("miner") => spawn_one(&mut orders, me, kind, miner_body(), vec![]),
        Some("line") => spawn_one(&mut orders, me, kind, line_body(), vec![]),
        Some("bulwark") => spawn_one(&mut orders, me, kind, line_body(), mods(pick("armor", "armor"), pick("dash", "dash"))),
        Some("artillery") => spawn_one(&mut orders, me, kind, gun_body(), mods(pick("artillery", "artillery"), pick("scan", "scan"))),
        Some("medic") => spawn_one(&mut orders, me, kind, medic_body(), mods(pick("repair", "repair"), pick("scan", "scan"))),
        Some("piercer") => spawn_one(&mut orders, me, kind, interceptor_body(), if has("dash") { vec!["pierce", "dash"] } else { vec!["pierce"] }),
        Some("interceptor") => spawn_one(&mut orders, me, kind, interceptor_body(), mods(pick("antiAir", "antiAir"), pick("dash", "dash"))),
        Some("wing") => spawn_one(&mut orders, me, kind, wing_body(has("airWeapons")), vec![]),
        _ => spawn_one(&mut orders, me, Some("scout"), scout_body(), vec![]),
    }

    // Compute chooses research/type schedules; shared-CPU policy below replaces
    // the old split and makes every execution lane trade against one bank.
    // Controllers keep their last legal orders. Expensive route/deploy loops
    // therefore run less often, while artillery gets enough of the control
    // lane to route an entire battery when the plan changes.
    // Absolute allocations are deliberate: a fractional split made every
    // event wake-up too small to issue even one Order. Every type we field
    // gets its base first (common::calls); lines and scouts grow their calls
    // last, so the rest goes to the combined-arms bodies that can change the fight.
    let room = if available.truthy() { num(available.get("control")) * 0.8 } else { f64::INFINITY };
    let types = common::calls(
        world,
        &[
            ("miner", 6400.0, 3.0),
            ("bulwark", 6000.0, 4.0),
            ("artillery", 6000.0, 5.0),
            ("piercer", 6000.0, 5.0),
            ("medic", 6000.0, 6.0),
            ("interceptor", 6000.0, 5.0),
            ("wing", 4200.0, 3.0),
            ("line", 6000.0, 4.0),
            ("scout", 4000.0, 6.0),
        ],
        or(cpu.get("income"), 0.0),
        room,
        me.get("units").get("byType"),
    );
    if full_plan {
        common::budget(&mut orders, world, me, &common::Budget { research: 0.13, code: 0.66, techs: &TECHS, types: Some(&types) });
    } else {
        orders.set("compute",obj! {"types"=>types});
        lean_research(&mut orders,world,me);
    }
    let mut policy = cpu_policy(world, me, &orders, pressure);
    common::fit_cpu(&mut policy, me);
    orders.entry("compute").set("cpu", policy);
    orders.set("memo", format!("Knights Templated: {} · pressure {}", if strained { "banking CPU" } else { "combined arms" }, jstr(pressure)));
    sermon_memo(&mut orders, me);
    if !full_plan {return affordable(orders,me,start);}
    // A short second keeps what's decided so far (the plan, spawns and the CPU policy): diplomacy and the faith wait a
    // second rather than run think out of cycles and lose it all.
    if cycles_left() < 6000.0 {
        return affordable(orders, me, start);
    }
    let leader = realm::leader(world);
    let me_id = me.get("id");
    // a pact against a runaway leader: the biggest faction in no pact (a faction belongs to one), asked at most once per
    // 2 minutes or so (common::court)
    if me.get("pacts").is_empty() && leader.is_some_and(|l| !eq(l.get("id"), me_id) && common::dominant(world).is_some()) {
        if let Some(p) = common::partners(world, me).first() {
            let id = p.get("id");
            common::court(&mut orders, world, me, num(id), |o| diplomacy::propose(o, me, vec![id.value()], (), "against the leader"), &common::Court::default());
        }
    }
    if cycles_left() > 3500.0 {
        faith(&mut orders, world, me, strained, pressure);
    }
    if cycles_left() > 2000.0 {
        let pact = |m: View| me.get("pacts").is_empty() && leader.is_none_or(|l| !eq(m.get("from"), l.get("id")));
        common::answer(&mut orders, me, world, pact, &TECHS);
    }
    affordable(orders, me, start)
}

// ---- pilgrims ----

// The holy sites for the order's pilgrims: provinces two steps out from its border (a province between, not on the
// border itself), free land first, then the leader's and the richest rival land, never a capital. A scout that stands
// in one preaches there whatever the border says. Worked out every 30 seconds and kept in the plan between.
fn pilgrim_sites(world: View, me: View) -> Value {
    let now = or(world.get("seconds"), 0.0).floor();
    let prior = mem::get(me.get("ramCapital"), 0).get("pilgrims").clone();
    let have = matches!(prior, Value::Arr(_));
    if (have && !common::every(now, 30.0, num(me.get("id")))) || !me.get("techs").get("faith").truthy() || cycles_left() < 40000.0 {
        return if have { prior } else { Value::arr() };
    }
    let me_id = me.get("id");
    let by_id = realm::index(world);
    let lead = realm::leader(world);
    let mut cands: Vec<(f64, f64)> = Vec::new();
    let mut seen: Vec<f64> = Vec::new();
    for p in world.get("provinces").iter() {
        if !eq(p.get("owner"), me_id) {
            continue;
        }
        for a in p.get("adj").iter() {
            let q = by_id.at(a);
            if !q.truthy() || eq(q.get("owner"), me_id) {
                continue;
            }
            for b in q.get("adj").iter() {
                let r = by_id.at(b);
                if !r.truthy() {
                    continue;
                }
                let id = num(r.get("id"));
                if seen.contains(&id) {
                    continue;
                }
                seen.push(id);
                let o = r.get("owner");
                if !o.is_num() || eq(o, me_id) || r.get("capital").truthy() || (num(o) >= 0.0 && !realm::is_rival(world, me, num(o))) {
                    continue;
                }
                if r.get("adj").iter().any(|c| {
                    let t = by_id.at(c);
                    t.truthy() && eq(t.get("owner"), me_id)
                }) {
                    continue;
                }
                let mut w = if num(o) < 0.0 { 2.0 } else { 1.2 } * (1.0 + min(or(r.get("cores"), 0.0), 6.0) / 6.0);
                if lead.is_some_and(|l| eq(l.get("id"), o)) {
                    w *= 1.3;
                }
                cands.push((id, w));
            }
        }
    }
    sort(&mut cands, |a, b| b.1 - a.1 > 0.0);
    Value::Arr(cands.iter().take(6).map(|c| Value::Num(c.0)).collect())
}

// ---- the faith ----
//
// The order's whole strategy. It founds its faith the minute the tech arrives, then preaches in as many provinces as the
// influence lane can carry, every second:
//   - free land on its border, where a sermon wins favour fast enough that the land agrees to join (150 favour, then up
//     to an 8% chance a second): a crusade with no soldiers;
//   - rival land on its border, the leader's and the restive first, where the faith's pull on loyalty and its favour
//     are waiting for the revolt that hands the province to the faith's founder;
//   - its own land, deeper, and first of all where a rival's faith has taken root (a heresy);
//   - any land its units stand in: brethren on a march and pilgrims preach where they stand, whatever the border.
// The cycles each sermon gets follow the square root the engine pays zeal by: the budget is shared in proportion to the
// square of each flock's worth, so a good flock gets more but no flock is starved. Keeping the faith comes first
// (a believer left unpaid falters ten times as often), so the sermons get what the lane's limit leaves after the upkeep.

/// The fewest and the most cycles (at price level 1) one sermon gets.
const SERMON_MIN: f64 = 40.0;
const SERMON_MAX: f64 = 900.0;

/// The share of income the influence lane may spend on the faith: the upkeep of the believers and the sermons.
fn sway_share(strained: bool, pressure: f64) -> f64 {
    if pressure >= 2.0 {
        0.14
    } else if strained {
        0.18
    } else {
        0.32
    }
}

/// What the standing sermons, the propaganda and the keeping of the faith cost a second now (cycles).
fn sway_cost(me: View) -> f64 {
    let s = me.get("sway");
    if !s.truthy() {
        return 0.0;
    }
    let mut c = or(s.get("faith").get("upkeep"), 0.0);
    for t in s.get("preach").iter().chain(s.get("propaganda").iter()) {
        c += or(t.get("cycles"), 0.0);
    }
    c
}

/// The influence lane's limit for a cost of `c` cycles: room for the believers to grow a little within the second.
fn sway_limit(c: f64) -> f64 {
    if c > 0.0 { (c * 1.12 + 300.0).ceil() } else { 0.0 }
}

/// A province the order may preach in, and what a sermon there is worth.
#[derive(Clone)]
struct Flock {
    id: f64,
    w: f64,
}

// Every province in reach of a sermon (its own, next to its land, or with a unit of its in it) that wants more zeal for
// the faith, with its worth: free land and the leader's land first, a heresy at home before anything.
fn flocks(world: View, me: View) -> Vec<Flock> {
    let me_id = me.get("id");
    let by_id = realm::index(world);
    let lead = realm::leader(world).filter(|l| num(l.get("share")) > 0.22 && !eq(l.get("id"), me_id));
    let mut out: Vec<Flock> = Vec::new();
    for p in world.get("provinces").iter() {
        let owner = p.get("owner");
        if !owner.is_num() {
            continue;
        }
        let o = owner.num();
        let mine = eq(owner, me_id);
        if !mine {
            // the faith works on rivals' and free land, never a partner's, and only where the order can see it
            if (o >= 0.0 && !realm::is_rival(world, me, o)) || !p.get("visible").truthy() {
                continue;
            }
            let border = p.get("adj").iter().any(|a| {
                let q = by_id.at(a);
                q.truthy() && eq(q.get("owner"), me_id)
            });
            if !border && or(p.get("units").key(num(me_id)), 0.0) <= 0.0 {
                continue;
            }
        }
        let f = p.get("faith");
        let ours = f.truthy() && eq(f.get("by"), me_id);
        let zeal = if f.truthy() { or(f.get("zeal"), 0.0) } else { 0.0 };
        let (zm, zo) = if ours { (zeal, 0.0) } else { (0.0, zeal) };
        let capital = p.get("capital").truthy();
        // At home the faith holds the land: State faith is +0.1 loyalty a point of zeal, so a province close to revolting
        // (its `risk`, the odds a minute) gets a bigger sermon, but never one that outweighs the free land at the border.
        let mut urgent = 0.0;
        let (target, base) = if mine {
            let r = or(p.get("risk"), 0.0);
            let risk = if r > 1.0 { r / 100.0 } else { r };
            urgent = min(risk * 6.0, 1.0);
            (if capital { 85.0 } else if risk >= 0.05 { 70.0 } else { 55.0 }, 0.8)
        } else if o < 0.0 {
            (40.0, 2.6)
        } else {
            (50.0, 1.6)
        };
        // a rival's faith has to be worn down first, and wears down half as fast under the state that holds it
        let wear = if zo > 0.0 && f.truthy() && eq(f.get("by"), owner) { 2.0 } else { 1.0 };
        let gap = max(target - zm, 0.0) + zo * wear;
        if gap < 5.0 {
            continue;
        }
        let mut w = base * (1.0 + min(or(p.get("cores"), 0.0), 6.0) / 6.0) * (gap / 40.0).clamp(0.35, 1.0);
        if mine {
            w += urgent;
            if zo > 5.0 {
                w += 1.4 + zo / 40.0;
            }
        } else {
            if zm > 0.0 {
                w *= 1.15;
            }
            if o >= 0.0 {
                match p.get("loyaltyBand").str() {
                    "restive" => w *= 1.5,
                    "uneasy" => w *= 1.2,
                    _ => {}
                }
                if lead.is_some_and(|l| eq(l.get("id"), owner)) {
                    w *= 1.4;
                }
                if capital {
                    w *= 0.6;
                }
            }
        }
        out.push(Flock { id: num(p.get("id")), w });
    }
    sort(&mut out, |a, b| b.w - a.w > 0.0);
    out
}

fn faith(orders: &mut Value, world: View, me: View, strained: bool, pressure: f64) {
    let techs = me.get("techs");
    if !techs.get("influence").truthy() {
        return;
    }
    let me_id = me.get("id");
    // the first of the order's names nobody holds yet (a faith's name is its own), founded the second the tech arrives
    if techs.get("faith").truthy() && !me.get("sway").get("faith").truthy() {
        let faiths = world.get("faiths");
        for i in 0..FAITHS.len() {
            let k = (num(me_id) + i as f64) % FAITHS.len() as f64;
            let name = if k >= 0.0 && k.fract() == 0.0 { FAITHS[k as usize] } else { "" };
            if !faiths.iter().any(|f| f.get("name").is_str() && f.get("name").str() == name) {
                orders.set("faith", obj! { "found" => name });
                return;
            }
        }
        // every name of the order's is taken (a crowded world): a name of its own
        orders.set("faith", obj! { "found" => format!("The Templar Rite of Seat {}", jstr(num(me_id))) });
        return;
    }
    // standing orders: worked out again on every full think (every 3 seconds), and at once while there are none
    let now = or(world.get("seconds"), 0.0).floor();
    let sway = me.get("sway");
    let due = common::every(now, 4.0, num(me_id)) || (sway.get("preach").is_empty() && common::every(now, 2.0, 0.0));
    if !due {
        return;
    }
    let cpu = me.get("rts").get("cpu");
    let income = max(1.0, or2(cpu.get("income"), me.get("compute").get("cycles"), 14000.0));
    let level = common::price_level(me);
    let has_faith = sway.get("faith").truthy();
    let upkeep = or(sway.get("faith").get("upkeep"), 0.0);
    let total = sway_share(strained, pressure) * income;
    // propaganda: the rival border provinces where the faith has begun to take (two at most), to bring the revolt on
    let mut prop_cost = 0.0;
    if common::every(now, 8.0, num(me_id)) {
        orders.set("propaganda", Value::arr());
        let each = round(150.0 * level);
        for t in common::sway_targets(world, me).iter().filter(|t| t.get("faith").truthy() && eq(t.get("faith").get("by"), me_id) && or(t.get("faith").get("zeal"), 0.0) >= 10.0).take(2) {
            if prop_cost + each > total * 0.12 {
                break;
            }
            sway::propaganda(orders, num(t.get("id")), each);
            prop_cost += each;
        }
    } else {
        for t in sway.get("propaganda").iter() {
            prop_cost += or(t.get("cycles"), 0.0);
        }
    }
    let mut preach_cost = 0.0;
    let mut sermons = 0;
    if has_faith {
        let list = flocks(world, me);
        let budget = max(0.0, total - upkeep - prop_cost) / level;
        let mut k = list.len().min(16);
        let mut square: f64 = list.iter().take(k).map(|f| f.w * f.w).sum();
        while k > 0 && budget * list[k - 1].w * list[k - 1].w / square < SERMON_MIN {
            k -= 1;
            square -= list[k].w * list[k].w;
        }
        orders.set("preach", Value::arr());
        for f in list.iter().take(k) {
            let e = min(budget * f.w * f.w / square, SERMON_MAX);
            let c = round(e * level);
            sway::preach(orders, f.id, c);
            preach_cost += c;
            sermons += 1;
        }
    }
    // the lane's limit follows what was just ordered, so the believers are always paid for
    orders.entry("compute").entry("cpu").entry("limits").set("influence", sway_limit(upkeep + preach_cost + prop_cost));
    let _ = sermons;
}

// What the spectators read: the faith, how many provinces believe and how many sermons are being preached.
fn sermon_memo(orders: &mut Value, me: View) {
    let sway = me.get("sway");
    let faith = sway.get("faith");
    if !faith.truthy() {
        return;
    }
    let name = faith.get("name");
    let sermons = sway.get("preach").len();
    orders.set(
        "memo",
        format!(
            "{}: {} provinces believe · {}",
            if name.is_str() { name.str() } else { "The faith" },
            jstr(or(faith.get("provinces"), 0.0)),
            if sermons == 0 { "the faithful keep watch".to_string() } else { format!("sermons in {}", jstr(sermons as f64)) }
        ),
    );
}

// ---- unit code ----

// Unit types commit each order with order() as they make it, rather than return the list: the game applies the same
// orders, and the cycles each takes to send count before the next unit, so the 2200 kept back stays free.

fn miner(units: View, world: View, me: View) {
    if common::recover_miners(units, world, me) { return; }
    for turn in 0..units.len() {
        if short(me) {
            break;
        }
        if let Some(o) = orders::gather(rotate(units, world, turn), world, me) {
            order(o);
        }
    }
}

// Transitional infantry only route when their job actually changes.  They stay
// useful as a small capture/defence reserve while the licensed bodies take over.
fn line(units: View, world: View, me: View) {
    // a call short of the plan lookup and an order keeps its units' standing orders a second longer
    if short(me) {
        return;
    }
    let p = plan::for_units(me, units, world);
    let attack = vnum(p.get("attack"));
    for turn in 0..units.len() {
        if short(me) {
            break;
        }
        let u = rotate(units, world, turn);
        // those the plan sends to defend (common::defends), the rest on the attack
        let defend = common::defends(&p, u);
        let (verb, target) = if defend >= 0.0 { ("fight", defend) } else { ("capture", attack) };
        // a beaten army falls back to the stage, and reinforcements wait there for a wave (common::front)
        if verb == "capture" && target == attack {
            if let Some(o) = common::front(u, world, me, &p) {
                if let Some(o) = o {
                    order(o);
                }
                continue;
            }
        }
        if target.is_nan() || target < 0.0 || (u.get("doing").str() == verb && u.get("target").num() == target) {
            continue;
        }
        if let Some(o) = orders::to(u, verb, target, world, me, ()) {
            order(o);
        }
    }
}

// Abroad, a unit knows only the plan it carried out of your land (plan::of), so a unit at home carries the current one
// in its RAM (orders::carry) on its order, or on a RAM-only order when its order stands. Without it, a unit that
// crosses the border forgets its target and walks home.
fn carry(o: Option<Value>, u: View, p: &Rc<Value>, me: View) -> Option<Value> {
    let Some(ops) = common::carry(u, p, me) else { return o };
    let mut x = o.unwrap_or_else(|| obj! { "id" => u.get("id") });
    x.set("ram", ops);
    Some(x)
}

fn tactical_order(u: View, world: View, me: View, role: &str, budget: &RouteBudget) -> Option<Value> {
    let p = plan::of(me, u);
    let defend = common::defends(&p, u);
    let attack = vnum(p.get("attack"));
    let target = if defend >= 0.0 { defend } else if attack >= 0.0 { attack } else { num(me.get("capital")) };
    if !budget.route(u, target, world, me) { return None; }
    // a beaten army falls back to the stage, and reinforcements wait there for a wave (common::front)
    if defend < 0.0 && attack >= 0.0 {
        if let Some(o) = common::front(u, world, me, &p) {
            return o;
        }
    }
    let o = if defend >= 0.0 {
        orders::to(u, "fight", defend, world, me, ())
    } else if attack >= 0.0 {
        orders::to(u, if role == "gun" { "fight" } else { "capture" }, attack, world, me, ())
    } else {
        orders::to(u, "fight", num(me.get("capital")), world, me, ())
    };
    // Unchanged route and power stay in the engine. Avoiding a fresh empty order
    // per unit is what leaves cycles for reactions and the shared combat bank.
    let Some(o) = o else { return carry(None, u, &p, me) };
    let mut o = carry(Some(o), u, &p, me)?;
    let cpu = me.get("rts").get("cpu");
    let income = max(1.0, or2(cpu.get("income"), me.get("compute").get("cycles"), 14000.0));
    let available = cpu.get("available");
    let tight = num(cpu.get("bank")) < income * 1.65 || (!available.is_undef() && num(available) < income * 0.35);
    let under_pressure = vor(p.get("threat"), 0.0) > 0.0;
    let (drive, armor, weapon) = match role {
        "wall" => (if under_pressure { 360.0 } else { 180.0 }, if under_pressure { if tight { 480.0 } else { 850.0 } } else { 0.0 }, 220.0),
        "gun" => (if tight { 100.0 } else { 180.0 }, 160.0, if tight { 340.0 } else { 650.0 }),
        "medic" => (if tight { 260.0 } else if under_pressure { 620.0 } else { 420.0 }, 0.0, 0.0),
        _ => (if tight { 400.0 } else { 720.0 }, 0.0, if tight { 180.0 } else { 360.0 }),
    };
    o.set("power", obj! { "drive" => drive, "armor" => armor, "weapon" => weapon });
    Some(o)
}

fn tactical(units: View, world: View, me: View, role: &str) {
    let mut budget = RouteBudget::new(me);
    for turn in 0..units.len() {
        if !budget.room(me) { break; }
        let u = rotate(units, world, turn);
        if !budget.admits(u, world, me, true) { continue; }
        let began = cycles_left();
        if let Some(o) = tactical_order(u, world, me, role, &budget) { order(o); }
        budget.finish(began, true);
    }
}

fn bulwark(units: View, world: View, me: View) {
    tactical(units, world, me, "wall")
}

fn artillery(units: View, world: View, me: View) {
    let mut budget = RouteBudget::new(me);
    for turn in 0..units.len() {
        if !budget.room(me) {
            break;
        }
        let u = rotate(units, world, turn);
        if !budget.admits(u, world, me, true) { continue; }
        let began = cycles_left();
        let r = u.get("rts");
        let p = plan::of(me, u);
        let target = if common::defends(&p, u) >= 0.0 {
            Value::Num(common::defends(&p, u))
        } else if vnum(p.get("attack")) >= 0.0 {
            p.get("attack").clone()
        } else {
            me.get("capital").value()
        };
        let travelling = !eqv(u.get("at"), &target) || num(u.get("to")) >= 0.0;
        if travelling {
            if !budget.route(u, vnum(&target), world, me) { continue; }
            // A deployed gun has zero movement. Pack it in the same order that gives
            // it the new route, including when the target changed under its feet.
            let o = carry(orders::to(u, "fight", vnum(&target), world, me, ()), u, &p, me);
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
        budget.finish(began, true);
    }
}

fn medic(units: View, world: View, me: View) {
    tactical(units, world, me, "medic")
}

fn interceptor(units: View, world: View, me: View) {
    tactical(units, world, me, "fast")
}

fn piercer(units: View, world: View, me: View) {
    interceptor(units, world, me)
}

fn wing(units: View, world: View, me: View) {
    common::fly(units, world, me);
}

fn scout(units: View, world: View, me: View) -> Vec<Value> {
    // The lower-numbered half of the scouts are pilgrims (they walk to a holy site and preach there); the rest scout.
    // common::wander is a search: each scout that has stopped gets one while the call has cycles for it, from a
    // rotating start, so the rest wait a call rather than run it out
    let mut ids: Vec<f64> = units.iter().map(|u| num(u.get("id"))).collect();
    ids.sort_by(|a, b| a.total_cmp(b));
    let pilgrims = ids.len().div_ceil(2);
    let mut budget = RouteBudget::new(me);
    for turn in 0..units.len() {
        if !budget.room(me) {
            break;
        }
        let u = rotate(units, world, turn);
        let rank = ids.iter().position(|&i| i == num(u.get("id"))).unwrap_or(0);
        if !budget.admits(u, world, me, rank < pilgrims) {
            continue;
        }
        let began = cycles_left();
        if rank < pilgrims && pilgrim(u, world, me, rank, &budget) {
            budget.finish(began, true);
            continue;
        }
        if u.get("doing").str() == "move" {
            continue;
        }
        let next = common::wander(world, me, num(u.get("at")));
        if next >= 0.0 {
            order(obj! { "id" => u.get("id"), "do" => "move", "target" => next, "power" => obj! { "drive" => 840, "armor" => 0, "weapon" => 0 } });
        }
        budget.finish(began, false);
    }
    Vec::new()
}

// Admit the cold plan, route, carried RAM and submitted order together. The measured
// cost includes all four, so later units leave room to finish this callback.
struct RouteBudget {
    next: f64,
    wander: f64,
}

impl RouteBudget {
    fn new(me: View) -> Self {
        if me.get("rts").truthy() {
            orders::set_reserve(1100.0);
            orders::set_first(1400.0);
        }
        Self { next: 1400.0, wander: 700.0 }
    }

    fn room(&self, me: View) -> bool {
        !me.get("rts").truthy() || cycles_left() >= self.next.min(self.wander) + 600.0
    }

    fn admits(&self, u: View, world: View, me: View, pilgrim: bool) -> bool {
        if !me.get("rts").truthy() { return true; }
        let page = me.get("ramProvince").key(u.get("at").num()).key(0.0);
        let bytes = page.get("bytes");
        let read = if pilgrim && plan::known(me, u).is_none() && bytes.is_bytes() && bytes.len() >= 2 {
            let b = bytes.bytes();
            4.0 * f64::from(u16::from_le_bytes([b[0], b[1]]))
        } else { 0.0 };
        let index = if realm::indexed(world) { 0.0 } else { 4.0 * world.get("provinces").len() as f64 };
        cycles_left() >= (if pilgrim { self.next } else { self.wander }) + read + index + 600.0
    }

    fn route(&self, u: View, target: f64, world: View, me: View) -> bool {
        if !me.get("rts").truthy() { return true; }
        let from = if u.get("to").num_or(-1.0) >= 0.0 { u.get("to").num() } else { u.get("at").num() };
        let index = realm::index(world);
        let here = index.get(from);
        // Home routes walk and serialize every step without a search-budget check.
        let steps = if from == target || realm::reach(here, me).contains(&target) { 0.0 } else {
            let goal = index.get(target);
            let hops = if eq(goal.get("owner"), me.get("id")) { goal.get("hops").num_or(0.0) } else {
                goal.get("adj").iter().map(|id| index.at(id)).filter(|p| eq(p.get("owner"), me.get("id")))
                    .map(|p| p.get("hops").num_or(0.0)).fold(f64::INFINITY, f64::min)
            };
            here.get("hops").num_or(0.0).max(0.0) + if hops.is_finite() { hops.max(0.0) + 1.0 } else { 0.0 }
        };
        cycles_left() >= self.next + 60.0 * steps + 600.0
    }

    fn finish(&mut self, began: f64, pilgrim: bool) {
        let next = if pilgrim { &mut self.next } else { &mut self.wander };
        *next = next.max((began - cycles_left()) * 1.5 + 400.0);
    }
}

// A pilgrim walks to the holy site the plan names for it and stays, preaching where it stands. True when it is one for
// now (on its way, at its site, or ordered there); false when there is no site, so it scouts instead.
fn pilgrim(u: View, world: View, me: View, rank: usize, budget: &RouteBudget) -> bool {
    if !plan::here(me, u) {
        // abroad it keeps to its site: on the way, or standing in land that is neither ours nor a partner's
        if u.get("doing").str() == "move" {
            return true;
        }
        let here = realm::index(world).get(num(u.get("at")));
        return here.truthy() && !eq(here.get("owner"), me.get("id")) && !(here.get("owner").is_num() && realm::is_ally(me, num(here.get("owner"))));
    }
    let p = plan::of(me, u);
    let sites = p.get("pilgrims");
    if sites.is_empty() {
        return false;
    }
    let site = sites.at(rank % sites.len()).num();
    if site.is_nan() || site < 0.0 {
        return false;
    }
    if !budget.route(u, site, world, me) {
        return true;
    }
    if let Some(mut o) = orders::to(u, "move", site, world, me, ()) {
        o.set("power", obj! { "drive" => 840, "armor" => 0, "weapon" => 0 });
        if let Some(c) = carry(Some(o), u, &p, me) {
            order(c);
        }
    }
    true
}

// ---- reflex ----

// Target selection, facing and support are faction code; the engine only prices and validates.
fn pierce_mode(u: View, target: View) -> f64 {
    let tr = target.get("rts");
    if !target.truthy() || !u.get("rts").get("modules").has_str("pierce") || !tr.truthy() {
        return 0.0;
    }
    let mut armor = or(tr.get("armor"), 0.0);
    let guard = tr.get("guard");
    let d = num(target.get("d"));
    if guard.truthy() && d > 0.01 {
        armor = min(0.8, armor * (0.85 - 0.5 * (num(guard.at(0)) * num(target.get("dx")) + num(guard.at(1)) * num(target.get("dy"))) / d));
    }
    // Search the public damage/CPU curve. The planning itself also consumes CPU.
    let (mut best, mut efficiency) = (0.0, 1.0 - armor);
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
        let d = num(t.get("d"));
        if t.get("rival").truthy() && !t.get("air").truthy() && d > 0.0 {
            x += num(t.get("dx")) / d;
            y += num(t.get("dy")) / d;
            n += 1.0;
        }
    }
    if n != 0.0 && hypot(x, y) > n * 0.45 { obj! { "x" => x, "y" => y } } else { Value::Null }
}

fn repair_friend(u: View, ns: View, o: &mut Value, bank: f64, level: f64) -> f64 {
    let r = u.get("rts");
    if !r.get("modules").has_str("repair") || or(r.get("abilityIn").get("repair"), 0.0) > 0.0 {
        return 0.0;
    }
    // (id, hp, hpMax, d)
    let (hp, hp_max) = (num(u.get("hp")), num(u.get("hpMax")));
    let mut target = if hp < hp_max - 0.1 { Some((u.get("id"), hp, hp_max, 0.0)) } else { None };
    for t in ns.iter() {
        let (hp, hp_max) = (num(t.get("hp")), num(t.get("hpMax")));
        if (eq(t.get("owner"), u.get("owner")) || t.get("ally").truthy())
            && num(t.get("d")) <= 3.0
            && hp < hp_max - 0.1
            && target.is_none_or(|x| hp / hp_max < x.1 / x.2)
        {
            target = Some((t.get("id"), hp, hp_max, num(t.get("d"))));
        }
    }
    let Some((id, hp, hp_max, d)) = target else { return 0.0 };
    let amount = min(0.7, hp_max - hp);
    let cost = (level * 700.0 * (amount / 0.7) * (1.0 + 0.15 * d * d)).ceil();
    if cost > bank {
        return 0.0;
    }
    o.set("ability", "repair");
    o.set("repairTarget", id);
    o.set("amount", amount);
    cost
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

fn reflex(units: View, near: View) -> Vec<Value> {
    // A small event allowance cannot cover pool setup and a complete reaction.
    if cycles_left() < 2800.0 { return Vec::new(); }
    let first = units.at(0).get("rts");
    let budget = first.get("budget");
    let mut bank_left = max(0.0, or(first.get("bank"), 0.0) - cycles_left() - 700.0);
    let available = |k: &str| min(bank_left, or(budget.get(k), bank_left));
    let (mut weapon_left, mut armor_left, mut drive_left, mut ability_left) =
        (available("weapon"), available("armor"), available("drive"), available("abilities"));
    let n = units.len() as f64;
    let start = if n > 0.0 { (or(units.at(0).get("rts").get("bank"), 0.0) / 211.0).floor() % n } else { 0.0 };
    // Orders commit once through order(). Leave room for the next contact's scan.
    let mut turn_cost = 2400.0_f64;
    for turn in 0..units.len() {
        if cycles_left() < turn_cost {
            break;
        }
        let began = cycles_left();
        let u = units.key((start + turn as f64) % n);
        let ns = near.key(num(u.get("id")));
        let role_cost = if matches!(u.get("type").str(), "artillery" | "medic") { 3200.0 } else { 2200.0 };
        if cycles_left() < role_cost + ns.len() as f64 * 100.0 { continue; }
        let mods = u.get("rts").get("modules");
        let (mut threat, mut best, mut score, mut ground): (Option<View>, Option<View>, f64, f64) = (None, None, 99.0, 0.0);
        for t in ns.iter() {
            if !t.get("rival").truthy() {
                continue;
            }
            let air = t.get("air").truthy();
            if !air {
                ground += 1.0;
            }
            let d = num(t.get("d"));
            let range = t.get("range");
            if threat.is_none() && range.is_num() && range.num() == 0.0 && d <= 2.2 {
                threat = Some(t);
            }
            if d > num(u.get("reach")) || (air && !u.get("air").truthy() && !mods.has_str("antiAir")) {
                continue;
            }
            let s = num(t.get("hp")) / num(t.get("hpMax")) - if range.truthy() { 0.18 } else { 0.0 }
                - if air { if common::air_defender(u) { 2.0 } else { 0.11 } } else { 0.0 };
            if s < score {
                score = s;
                best = Some(t);
            }
        }
        let r = u.get("rts");
        let low = num(u.get("hp")) < num(u.get("hpMax")) * 0.40;
        // A unit sees a shared-bank snapshot. Spend a local estimate conservatively
        // and leave 700 CPU headroom for this code and other combat events.
        let mut left = bank_left;
        // every price below is the rules' price times the world's price level
        let level = common::price_level(u);
        let kind = u.get("type").str();
        let (mut drive, mut armor, weapon) = match kind {
            "bulwark" => (300.0, 850.0, 300.0),
            "artillery" => (120.0, 180.0, 720.0),
            "interceptor" | "piercer" => (780.0, 0.0, 450.0),
            "medic" => (420.0, 0.0, 0.0),
            _ => (500.0, 0.0, 260.0),
        };
        let armor_cost = level * armor * 0.2;
        if armor_cost > left || armor_cost > armor_left {
            armor = 0.0;
        } else {
            left -= armor_cost;
            armor_left -= armor_cost;
        }
        let _ = armor_left;
        // power is filled in last, in its place
        let mut o = obj! { "id" => u.get("id"), "power" => Value::Null, "guard" => guard_facing(ns), "pierce" => 0 };
        let heal_cost = if kind == "medic" { repair_friend(u, ns, &mut o, min(left, ability_left), level) } else { 0.0 };
        left -= heal_cost;
        ability_left -= heal_cost;
        // Dodge a shell before it lands. Hazards are own-observation data only.
        let pos = u.get("local");
        let mut hazard = None;
        for h in r.get("hazards").iter() {
            let x = if pos.truthy() { num(pos.get("x")) - num(h.get("x")) } else { 99.0 };
            let y = if pos.truthy() { num(pos.get("y")) - num(h.get("y")) } else { 99.0 };
            let reach = num(h.get("radius")) + 0.8;
            if x * x + y * y < reach * reach && num(h.get("impactIn")) < 1.4 {
                hazard = Some((x, y));
                break;
            }
        }
        let ability_in = r.get("abilityIn");
        let away = |t: View| obj! { "x" => -num(t.get("dx")), "y" => -num(t.get("dy")) };
        let mut step = false;
        if let Some((x, y)) = hazard {
            let nz = |v: f64| if v != 0.0 && !v.is_nan() { v } else { 1.0 };
            o.set("step", obj! { "x" => nz(x), "y" => nz(y) });
            step = true;
            drive = 900.0;
        } else if let (true, Some(best)) =
            (kind == "artillery" && ground >= 2.0 && pos.truthy() && r.get("deployed").truthy() && (!ability_in.truthy() || or(ability_in.get("shell"), 0.0) <= 0.0), best)
        {
            let (bx, by) = (num(best.get("dx")), num(best.get("dy")));
            let d = hypot(bx, by);
            let clustered = ns.iter().filter(|t| t.get("rival").truthy()
                && hypot(num(t.get("dx")) - bx, num(t.get("dy")) - by) <= 2.0).count();
            let mut radius = if clustered >= 3 { 2.0 } else { 1.0 };
            let clear = |radius: f64| d > radius + 0.7 && !ns.iter().any(|t| !t.get("rival").truthy()
                && hypot(num(t.get("dx")) - bx, num(t.get("dy")) - by) < radius + 0.8);
            if !clear(radius) { radius = 1.0; }
            let friendly = !clear(radius);
            let half = radius / 2.0;
            let shell_cost = level * 1000.0 * (half * half) * (1.0 + max(d - 5.0, 0.0) * 0.12);
            if !friendly && d <= 14.0 && shell_cost <= left && shell_cost <= ability_left {
                o.set("ability", "shell");
                o.set("aim", obj! { "x" => num(pos.get("x")) + bx, "y" => num(pos.get("y")) + by });
                o.set("radius", radius);
                o.set("hold", true);
                left -= shell_cost;
                ability_left -= shell_cost;
            }
        } else if kind == "artillery" && best.is_some() && !r.get("deployed").truthy() {
            o.set("deploy", true);
            o.set("hold", true);
        } else if heal_cost != 0.0 && !heal_cost.is_nan() {
            if let Some(t) = threat {
                o.set("step", away(t));
                step = true;
            }
        } else if (kind == "bulwark" || kind == "interceptor")
            && low
            && threat.is_some()
            && ability_left >= 900.0 * level
            && left >= 900.0 * level
            && mods.has_str("dash")
            && or(ability_in.get("dash"), 0.0) <= 0.0
        {
            o.set("step", threat.map(away));
            step = true;
            o.set("ability", "dash");
            left -= 900.0 * level;
            ability_left -= 900.0 * level;
        } else if r.get("modules").has_str("scan")
            && ns.len() >= 2
            && ability_left >= 700.0 * level
            && left >= 700.0 * level
            && (!ability_in.truthy() || or(ability_in.get("scan"), 0.0) <= 0.0)
        {
            o.set("ability", "scan");
            left -= 700.0 * level;
            ability_left -= 700.0 * level;
        } else if let (Some(t), "artillery") = (threat, kind) {
            o.set("step", away(t));
            step = true;
        }
        let _ = ability_left;
        if step {
            let desired = o.get("step");
            if let Some(direction) = terrain_step(u, desired.get("x").num(), desired.get("y").num()) {
                o.set("step", direction);
                if r.get("deployed").truthy() || r.get("deploying").truthy() { o.set("deploy", false); }
            } else {
                o.set("step", Value::Null);
                step = false;
            }
        }
        if step {
            let drive_cost = level * (60.0 + drive) * or(r.get("driveScale"), 1.0) * 0.2;
            if drive_cost > left || drive_cost > drive_left {
                drive = 0.0;
            } else {
                left -= drive_cost;
                drive_left -= drive_cost;
            }
        }
        let _ = drive_left;
        if let Some(best) = best {
            let pierce = pierce_mode(u, best);
            o.set("pierce", pierce);
            let shot_cost = (level * (120.0 + weapon) * (1.0 + 1.4 * pierce * pierce)).ceil();
            if or(r.get("readyIn"), 0.0) <= 0.0 && (r.get("windupTarget").is_null() || r.get("windupTarget").is_undef()) && shot_cost <= left && shot_cost <= weapon_left {
                o.set("hit", best.get("id"));
                left -= shot_cost;
                weapon_left -= shot_cost;
            }
            if !step {
                o.set("hold", true);
            }
        }
        bank_left = left;
        o.set("power", obj! { "drive" => drive, "armor" => armor, "weapon" => weapon });
        if best.is_some() || step || o.get("ability").truthy() || o.get("deploy").truthy() {
            order(o);
        }
        turn_cost = turn_cost.max(1.5 * (began - cycles_left()));
    }
    Vec::new()
}

// ---- puzzles ----

fn research(puzzle: View) -> Value {
    common::solve(puzzle)
}
fn duel(challenge: View) -> Value {
    common::solve(challenge)
}
fn challenge(foe: View, most: View) -> Value {
    common::challenge(foe, most)
}

// Each founder supplies its own body, weapon, projectile, anthem and capital.
fn look(budget: View, info: View) -> Value {
    theme::of(info.get("seat"), 2.0).map_or(Value::Null, |t| t.look(budget.num(), info))
}

fn build3d(budget: View, info: View) -> Value {
    theme::of(info.get("seat"), 2.0).map_or(Value::Null, |t| t.scene(info, budget.num()))
}

fn live3d(state: View, now: View) -> Value {
    let mut s = if state.truthy() { state.value() } else { Value::obj() };
    let spin = capital::ring(&mut s, now);
    obj! { "state" => s, "set" => obj! { "wonder" => spin } }
}

// ---- without CPU RTS: the classic game ----

// What the classic game's order researches, in order.
const LEGACY_TECHS: [&str; 5] = ["mining", "archery", "boats", "alpine", "espionage"];

fn legacy_think(world: View, me: View) -> Value {
    let mut orders = obj! { "policy" => obj! { "extraction" => 0.45, "autonomy" => "direct", "integrate" => "tolerate", "invest" => true } };
    // targets: the best neighbouring land, the claimant's first while someone else claims the Crown (common::rally)
    let targets = common::rally(world, me, realm::rank(world, me, common::frontier(world, me), common::score));
    let mut survey = common::survey(world, me, &targets, 2.0);
    // how many of every type stand at the stage, so guards and longbows go in together (common::muster)
    let ready = common::ready(world, me, survey.get("stage").num(), survey.get("attack").num(), survey.get("need").num());
    if ready > 0.0 && ready >= survey.get("need").num() {
        survey.set("ready", ready);
    }
    common::share(&mut orders, &survey, world, me);
    let techs = me.get("techs");
    let mut mix: Vec<(&str, f64)> = if techs.get("archery").truthy() {
        vec![("miner", 0.6), ("guard", 0.22), ("longbow", 0.13), ("scout", 0.05)]
    } else {
        vec![("miner", 0.6), ("guard", 0.35), ("scout", 0.05)]
    };
    if techs.get("espionage").truthy() {
        mix.push(("spy", 0.02));
    }
    // a founding seat's house style (theme) may list names for a unit type; otherwise a unit is called 'guard 12'
    let house = theme::of(me.get("seat"), -1.0);
    let name = |t: &str, n: f64| match house.and_then(|h| h.names_for(t)) {
        Some(list) => common::name(me, list, n),
        None => format!("{t} {}", jstr(n)),
    };
    let builds = [
        ("scout", obj! { "speed" => 4, "vision" => 4, "strength" => 0, "carry" => 0 }),
        ("longbow", obj! { "carry" => 0, "range" => 1 }),
        ("spy", obj! { "hp" => 1, "speed" => 3, "strength" => 0, "carry" => 0, "vision" => 4, "stealth" => 1 }),
    ];
    // miners stop at what the mines can use (common::miner_room)
    let limits = [("scout", 1.0), ("spy", 1.0), ("miner", common::miner_room(me, &survey))];
    common::recruit(&mut orders, me, &mix, name, &common::Recruit { reserve: 10.0, limits: &limits, builds: &builds, upkeep: 0.4 });
    let u = me.get("units");
    orders.set("memo", format!("{} mining · {} guards", jstr(or(u.get("doing").get("gather"), 0.0)), jstr(or(u.get("byType").get("guard"), 0.0))));
    // pacts: one, with factions that aren't leading the world (the weak band together against the strong)
    let top = world.get("factions").iter().filter(|f| f.get("alive").truthy()).fold(None, |a: Option<View>, f| match a {
        Some(a) if !(num(f.get("share")) > num(a.get("share"))) => Some(a),
        _ => Some(f),
    });
    let pact = |m: View| {
        me.get("pacts").is_empty()
            && top.is_some_and(|t| {
                let pm = m.get("pact");
                let members = if pm.truthy() { pm.get("members") } else { pm };
                !eq(m.get("from"), t.get("id")) && members.iter().all(|x| !eq(x, t.get("id")))
            })
    };
    common::answer(&mut orders, me, world, pact, &LEGACY_TECHS);
    // unit code: guards and longbows get most of it and think every 2 s; miners, the scout and the spy only when one goes
    // idle, is attacked or is caught
    let types = obj! {
        "guard" => obj! { "share" => 0.35, "every" => 2 },
        "longbow" => obj! { "share" => 0.15, "every" => 2 },
        "miner" => obj! { "share" => 0.3, "every" => 0 },
        "scout" => obj! { "share" => 0.05, "every" => 0 },
        "spy" => obj! { "share" => 0.15, "every" => 0 },
    };
    common::budget(&mut orders, world, me, &common::Budget { research: 0.1, techs: &LEGACY_TECHS, types: Some(&types), ..Default::default() });
    orders
}

fn guard(units: View, world: View, me: View) {
    if home_alarm(units, world, me) {
        return;
    }
    let cap = me.get("capital");
    let mut keep: Option<View> = None;
    for u in units.iter() {
        if eq(u.get("at"), cap) && keep.is_none_or(|k| num(u.get("id")) < num(k.get("id"))) {
            keep = Some(u);
        }
    }
    if let Some(k) = keep {
        if let Some(o) = orders::hold(k, Point::new(1.0, 0.0), world, me, None) {
            order(o);
        }
    }
    let rest: Vec<View> = units.iter().filter(|u| keep.is_none_or(|k| !u.same(&k))).collect();
    common::defend_then(&rest, world, me, "capture");
}

// Longbows (range 1: 4 cells) go with the army and shoot from behind it, and come home with it.
fn longbow(units: View, world: View, me: View) {
    if !home_alarm(units, world, me) {
        common::defend_then(&units.iter().collect::<Vec<View>>(), world, me, "fight");
    }
}

// While the plan says the capital is the province to defend, every unit of the type fights there. True when it did.
fn home_alarm(units: View, world: View, me: View) -> bool {
    let cap = me.get("capital");
    if !eqv(cap, plan::for_units(me, units, View::undefined()).get("defend")) {
        return false;
    }
    for u in units.iter() {
        if let Some(o) = orders::to(u, "fight", num(cap), world, me, ()) {
            order(o);
        }
    }
    true
}

// The spy (stealth 1) walks to the nearest rival capital and steals the dearest tech it can there while there's ◆ to
// pay the bribes; otherwise it watches from outside the ring.
fn spy(units: View, world: View, me: View) {
    for u in units.iter() {
        let o = if num(me.get("bank")) > 150.0 { common::steal(u, world, me) } else { None };
        let x = o.or_else(|| if u.get("doing").str() == "steal" { None } else { common::sneak(u, world, me) });
        if let Some(x) = x {
            order(x);
        }
    }
}
