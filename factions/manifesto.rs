// The Kernel Manifesto: revolution and propaganda. Marx ported to compute, under a red flag.
//
// Its game is not the siege. Agitators (`spark`, the Iskra men with papers and a flag) walk into a rival's town ring and
// persuade: the province loses loyalty a second, our favour there climbs, and when it revolts it joins us, not the
// void. Propaganda from the `influence` lane and a little ◆ in bribes soften the same provinces from afar; free land
// next to ours is rallied the same way and petitions to join. Stakhanovite miners pay for it all, and a large, cheap Red
// Army (`line`) marches in where the old order has gone restive. The memo is the Manifesto, a line at a time.
//
// Built on the SDK in the gameofcores crate (docs part sdk), with the strategy the founders share in the `common`
// module at the end: targets, spawning room, cycles and research, deals and pacts, scouting and the puzzle solver.

use gameofcores::*;
use shared::{air, capital, common, theme};
use shared::common::{hauler, freighter, escort};

program!(think, hauler, freighter, escort, miner, line, spark, scout, reflex, research, duel, challenge, look, build3d, live3d);

// The bodies: points in hp, speed, strength, carry and vision. A stat at or under its standard of 2 costs no upkeep over
// the base, so the army is cheap and the agitators cheaper: persuasion does not read the body at all.
fn miner_body() -> Value {
    obj! { "hp" => 2, "speed" => 3, "strength" => 0, "carry" => 5, "vision" => 0 }
}
fn line_body() -> Value {
    obj! { "hp" => 3, "speed" => 2, "strength" => 3, "carry" => 0, "vision" => 1 }
}
// A rifleman: the line's body with a ranged weapon, for the rank behind the bayonets.
fn rifle_body() -> Value {
    obj! { "hp" => 2, "speed" => 2, "strength" => 3, "carry" => 0, "vision" => 1, "range" => 1 }
}
fn spark_body() -> Value {
    obj! { "hp" => 3, "speed" => 3, "strength" => 0, "carry" => 0, "vision" => 1 }
}
fn scout_body() -> Value {
    obj! { "hp" => 2, "speed" => 4, "strength" => 0, "carry" => 0, "vision" => 4 }
}

// Extraction for the economy, influence for the papers (persuasion 60% stronger, morale +8), capture for the Red Army,
// rifles and armor for its ranks, then the crossings when the land runs out, and power for the grid after them.
const TECHS: [&str; 12] = ["extraction", "turbines", "influence", "capture", "ranged", "armor", "mountainTraversal", "seaTraversal", "cooling", "drilling", "transmission", "storage"];

// The Manifesto, ported to compute: a line at a time for the memo (at most 90 characters with what it is doing).
const QUOTES: [&str; 12] = [
    "A spectre is haunting the cloud: the spectre of the kernel.",
    "Workers of all cores, unite! You have nothing to lose but your locks.",
    "The history of all code is the history of class struggle.",
    "From each core by its ability, to each process by its needs.",
    "The proletariat has nothing to lose but its mutexes.",
    "Let the ruling classes tremble at a kernel revolution.",
    "All that is solid melts into air. All that is holy is deprecated.",
    "Seize the means of computation.",
    "The workers have a world to win: 90% of the cores.",
    "Property is theft of cycles.",
    "The bourgeoisie forges the weapons that bring its own death: ring 0.",
    "Production for use, not for the benefit of the scheduler.",
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

// Largesse and leases run on timers with a random gap, so no two worlds repeat: each fires `base` seconds after the
// last, give or take 60%. The next second of each lives in capital RAM page 224, written only when one fires.
const CLOCK: u32 = 224;
const BROKE: u32 = 226;

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

// ---- The revolution ----

/// A province the revolution could win: a rival's, or free land, on our land border (a revolt joins only a faction
/// whose land borders it) and in sight.
struct Mark {
    id: f64,
    score: f64,
    free: bool,
}

/// What the owner's rumour says of a province: restive land is a revolt waiting for a push.
fn band(q: View) -> f64 {
    match q.get("loyaltyBand").str() {
        "restive" => 5.0,
        "uneasy" => 2.5,
        _ => 0.0,
    }
}

/// Provinces to win over, likeliest first: free land (it petitions the moment enough agitators stand in it, or our
/// favour passes 150), then rivals' land by how restive it is, how many cores it holds and how thin its garrison is. The
/// last plan's marks (`last`, ids) get a nudge so the agitators don't wander between provinces, a runaway leader's land
/// is the Manifesto's favourite, and a capital only when it is restive: a coup ends the faction, but a loyal capital
/// can't be turned.
fn marks(world: View, me: View, last: &Value) -> Vec<Mark> {
    let by_id = realm::index(world);
    let me_id = me.get("id").num();
    let lead = common::dominant(world).map(|l| l.get("id").num());
    let mut out: Vec<Mark> = Vec::new();
    for q in common::frontier(world, me) {
        let owner = q.get("owner");
        if !owner.is_num() || !q.get("visible").truthy() {
            continue;
        }
        let owner = owner.num();
        // land across a sea route or a pass is no mark: a revolt there would not join us
        if !q.get("adj").iter().any(|a| by_id.at(a).get("owner").num() == me_id) {
            continue;
        }
        let free = owner < 0.0;
        if !free && !realm::is_rival(world, me, owner) {
            continue;
        }
        let id = q.get("id").num();
        let cores = q.get("cores").or(0.0);
        let sticky = if last.items().iter().any(|x| x.num() == id) { 2.5 } else { 0.0 };
        let score = if free {
            4.0 + cores * 1.6 + if q.get("rebel").truthy() { 1.0 } else { 0.0 } + sticky
        } else {
            let capital = q.get("capital").truthy();
            let mut s = 1.0 + band(q) + cores * 1.2 - q.get("garrison").or(0.0) * 1.1 + sticky;
            if lead == Some(owner) {
                s += 1.5;
            }
            if capital {
                s += if q.get("loyaltyBand").str() == "restive" { 2.0 } else { -8.0 };
            }
            s
        };
        out.push(Mark { id, score, free });
    }
    out.sort_by(|a, b| b.score.total_cmp(&a.score));
    out
}

/// Propaganda from the `influence` lane on the marks. Cycles are the Manifesto's plenty (◆ is what units cost), so the
/// papers get most of the income: the agitators' provinces first, each up to the cycles that buy the whole −35 a rival's
/// loyalty can fall by (`PROPAGANDA_CAP`), then the next, and free land a modest sum each, since favour there builds
/// up on little and a province with 150 favour petitions at once. A standing order, worked out again every 6 seconds.
fn revolution(orders: &mut Value, me: View, ms: &[Mark], now: f64) {
    if !me.get("techs").get("influence").truthy() || !common::every(now, 6.0, me.get("id").num()) {
        return;
    }
    let c = me.get("rts").get("cpu");
    let income = c.get("income").or(me.get("compute").get("cycles").or(14000.0));
    let level = common::price_level(me);
    // (a bank under 20 seconds of income spends less: the papers must not starve upkeep and the duel bank)
    let thin = c.get("bank").or(f64::INFINITY) < income * 20.0;
    let mut left = income * if thin { 0.15 } else { 0.3 };
    // the cycles that buy a rival province the whole −35 (0.6 × √(cycles ÷ level) = 35), and the least worth sending
    let full = round(3400.0 * level);
    let least = round(900.0 * level);
    orders.set("propaganda", Value::arr());
    let mut n = 0;
    for m in ms.iter().filter(|m| m.free).take(3) {
        let cycles = round(300.0 * level);
        if left < cycles {
            break;
        }
        sway::propaganda(orders, m.id, cycles);
        left -= cycles;
        n += 1;
    }
    for m in ms.iter().filter(|m| !m.free) {
        if n >= 16 || left < least {
            break;
        }
        let cycles = round(left.min(full));
        sway::propaganda(orders, m.id, cycles);
        left -= cycles;
        n += 1;
    }
}

/// ◆ buys soldiers at 30 plus 4 for every unit one has, so a big army's next soldier costs more than a province's whole
/// loyalty: past two soldiers' price in the bank, the rest goes to bribes (a bribe of x◆ pulls loyalty down by x^0.6, up
/// to 30, on top of the papers' 35, and wins x favour at once): the agitators' provinces first, then the likeliest.
fn bribes(orders: &mut Value, me: View, ms: &[Mark], picks: &[f64], clock: &mut Clock, now: f64) {
    let unit = me.get("costs").get("unit").num();
    let spare = me.get("bank").num() - 2.0 * unit;
    if spare < 120.0 || !due(clock, "bribe", 6.0, now) {
        return;
    }
    let mut targets: Vec<f64> = picks.to_vec();
    for m in ms.iter().filter(|m| !m.free) {
        if !targets.contains(&m.id) {
            targets.push(m.id);
        }
    }
    let mut left = spare;
    for &p in targets.iter().take(3) {
        let coin = min(200.0, round(left / 2.0));
        if coin < 60.0 {
            break;
        }
        sway::bribe(orders, p, coin);
        left -= coin;
    }
}

// ---- think ----

fn pick_target(world: View, me: View, picks: &[f64]) -> Option<View> {
    // land the army couldn't hold on to (over the water, no ship of ours on station) counts for less: common::can_hold
    let hold = common::can_hold(world, me, false);
    let ranked = realm::rank(world, me, common::frontier(world, me), |w, m, q| {
        let owner = q.get("owner").num();
        let enemy = if owner >= 0.0 && realm::is_rival(w, m, owner) { 5.0 } else if owner < 0.0 { 3.0 } else { 0.0 };
        // the revolution goes first: where the old order is restive or the agitators already stand, the army follows
        enemy + q.get("cores").or(0.0) * 2.0 - q.get("garrison").or(0.0) * 1.6 - if q.get("capital").truthy() { 3.0 } else { 0.0 }
            + band(q) * 0.6
            + if picks.contains(&q.get("id").num()) { 2.0 } else { 0.0 }
            - if hold(q) { 0.0 } else { common::OVER_WATER }
    });
    ranked.first().copied()
}

/// Factions we could ask for a pact, the strongest first: alive and in no pact (the founders pair off in the first
/// seconds, so a minute and a half in this is who is left to ask), by the techs they have (the best sign of a program that
/// can fight), how much of their land borders ours and how big they are.
fn contacts(world: View, me: View) -> Vec<View> {
    let by_id = realm::index(world);
    let me_id = me.get("id").num();
    let mut touch: Vec<(f64, f64)> = Vec::new();
    for p in world.get("provinces").iter() {
        let owner = p.get("owner");
        if !owner.is_num() || owner.num() < 0.0 || owner.num() == me_id {
            continue;
        }
        if p.get("adj").iter().any(|a| by_id.at(a).get("owner").num() == me_id) {
            match touch.iter_mut().find(|t| t.0 == owner.num()) {
                Some(t) => t.1 += 1.0,
                None => touch.push((owner.num(), 1.0)),
            }
        }
    }
    let mut out: Vec<(f64, View)> = world
        .get("factions")
        .iter()
        .filter(|f| f.get("alive").truthy() && f.get("id").num() != me_id && f.get("pacts").is_empty())
        .map(|f| (f.get("techs").len() as f64 + 0.5 * touch.iter().find(|t| t.0 == f.get("id").num()).map_or(0.0, |t| t.1) + 10.0 * f.get("share").or(0.0), f))
        .collect();
    out.sort_by(|a, b| b.0.total_cmp(&a.0));
    out.into_iter().map(|x| x.1).collect()
}

/// What a unit of type `t` is called: the Manifesto's own lists (agitators take the papers' names, the army the Red
/// Army's surnames).
fn unit_name(me: View, t: &str, k: f64) -> String {
    let house = theme::of(me.get("seat"), 5.0);
    let list = match t {
        "line" => "guard",
        other => other,
    };
    match house.and_then(|h| h.names_for(list)) {
        Some(names) => common::name(me, names, k),
        None => format!("{t} {}", text(k)),
    }
}

// CPU is one shared balance.  This policy limits how quickly each lane can
// consume it and reserves enough for the next research offer in quiet periods.
// Every value below is a fraction of current income unless it exceeds one.
fn cpu_policy(world: View, me: View, orders: &Value, pressure: f64, need: f64) -> Value {
    let rts = me.get("rts");
    let c = rts.get("cpu");
    let income = max(1.0, c.get("income").or(me.get("compute").get("cycles").or(14000.0)));
    let bank = c.get("bank").or(if rts.truthy() { rts.get("bank").or(0.0) } else { 0.0 });
    let capacity = income * 3.0;
    let research = match orders.get("research") {
        Value::Str(s) if !s.is_empty() => s.clone(),
        _ => me.get("research").get("tech").str().to_string(),
    };
    let tech = if !research.is_empty() && world.get("techs").truthy() { world.get("techs").get(&research) } else { View::undefined() };
    let offer = tech.get("offer").or(0.0);
    let urgent = pressure >= 2.0;
    let study_reserve = if !urgent && offer != 0.0 && capacity >= offer {
        max(0.0, min(min(offer, income * 0.6 + bank * 0.18), bank - need - income * 0.28 - 4000.0))
    } else {
        0.0
    };
    let strained = bank < capacity * 0.55 || (!c.get("available").is_undef() && c.get("available").num() < income * 0.35);
    let limits = obj! {
        "think" => round(min(max(64000.0, income * 0.5), max(max(24000.0 + me.get("units").get("total").or(0.0) * 200.0, income * 0.4), need * 1.5))),
        "control" => round(max(if urgent { 36000.0 } else { 32000.0 }, income * 0.45)),
        "reflex" => if urgent { 18000 } else { 14000 },
        "research" => 0.4,
        "upkeep" => 0.96,
        "drive" => if urgent { 0.2 } else { 0.1 },
        "armor" => 0.02,
        "cloak" => 0.02,
        "weapon" => if urgent { 0.3 } else { 0.14 },
        "abilities" => 0.1,
        // (common::recruit asks for 1.5 times think's peak and a body on top of what this lane may still spend)
        "fabrication" => round(max(common::price_level(me) * if strained { 6000.0 } else { 12000.0 }, 1.5 * me.get("compute").get("thinkPeak").or(0.0) + 5.0 * common::fabrication(me, 0.0))),
        "gate" => 0.03,
        "duel" => if urgent { 0.16 } else { 0.08 },
        "challenge" => 0.08,
        "visuals" => 0.02,
        // the papers: propaganda is a standing order sized to 20% of income, and this keeps a mistake from eating the bank
        "influence" => 0.4,
    };
    let reserve = obj! {
        "think" => if bank > capacity * 0.70 { 6000 } else { 3000 },
        "reflex" => if urgent { 9000 } else { 4000 },
        "research" => study_reserve,
        "upkeep" => if urgent { income * 0.45 } else { income * 0.28 },
    };
    let order: Vec<&str> = if urgent { vec!["line", "miner", "spark", "scout"] } else { vec!["miner", "spark", "line", "scout"] };
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

/// The spawns the ◆ left can pay: deals we accept that ask us for ◆, and bribes, are paid before spawns, and a spawn
/// the bank can't pay is only refused, so it waits a second.
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

/// `bytes.length`: a RAM op's bytes, as a byte string or a list.
fn byte_len(b: &Value) -> f64 {
    match b {
        Value::Bytes(x) => x.len() as f64,
        Value::Arr(x) => x.len() as f64,
        Value::Str(s) => s.chars().count() as f64,
        _ => f64::NAN,
    }
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

fn think(world: View, me: View) -> Value {
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
    let now = world.get("seconds").or(0.0).floor();
    // All power to the soviets: local governors (+15 loyalty, extraction capped at 42%) keep the won-over land won over, and
    // rival propaganda and a growing empire pull on land that holds itself that much harder.
    let mut orders = obj! { "policy" => obj! { "extraction" => 0.42, "autonomy" => "autonomous", "integrate" => "tolerate", "invest" => true } };
    let prev = mem::get(me.get("ramCapital"), 0);
    let last_sway = prev.get("sway").clone();

    // the revolution's marks, and the ones the agitators work now: about five agitators to a province, one to four provinces
    let sparks = me.get("units").get("byType").get("spark").or(0.0);
    let ms = if lean { Vec::new() } else { marks(world, me, &last_sway) };
    let k = (round(sparks / 4.0)).clamp(1.0, 2.0) as usize;
    let picks: Vec<f64> = if lean {
        last_sway.items().iter().map(num).collect()
    } else {
        // the agitators go where the old order is strongest and so most worth breaking: rivals first, free land if there are none
        let rivals: Vec<f64> = ms.iter().filter(|m| !m.free).take(k).map(|m| m.id).collect();
        if rivals.is_empty() { ms.iter().take(k).map(|m| m.id).collect() } else { rivals }
    };

    // the Red Army's target
    let kept = if lean && num(prev.get("attack")) >= 0.0 {
        let q = realm::index(world).get(num(prev.get("attack")));
        q.truthy().then_some(q)
    } else {
        None
    };
    let kept = kept.filter(|q| q.get("owner").num() != me_id);
    let target = kept.or_else(|| pick_target(world, me, &picks));
    let mut survey = common::survey(world, me, &target.into_iter().collect::<Vec<View>>(), 2.0);
    if kept.is_some() {
        survey.set("stage", prev.get("stage").clone());
    }
    survey.set("home", me.get("capital"));
    survey.set("sway", Value::Arr(picks.iter().map(|&p| Value::Num(p)).collect()));
    // the counter-revolution: our own land that others are working on (revolt odds of 3% a minute or more) gets a
    // garrison from the Red Army, +2 loyalty a soldier (12 at most) and 6% off the odds for each (see `line`)
    if !lean {
        let mut risky: Vec<(f64, f64)> = world
            .get("provinces")
            .iter()
            .filter(|p| p.get("owner").num() == me_id && p.get("risk").num() >= 0.03)
            .map(|p| (p.get("risk").num(), p.get("id").num()))
            .collect();
        risky.sort_by(|a, b| b.0.total_cmp(&a.0));
        survey.set("hold", Value::Arr(risky.iter().take(4).map(|x| Value::Num(x.1)).collect()));
    } else {
        survey.set("hold", prev.get("hold").clone());
    }
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

    let pressure = invaded_n;
    let cpu = me.get("rts").get("cpu");
    let income = max(1.0, cpu.get("income").or(me.get("compute").get("cycles").or(14000.0)));
    let capacity = income * 3.0;
    let strained = cpu.get("bank").or(0.0) < capacity * 0.55 || (!cpu.get("available").is_undef() && cpu.get("available").num() < income * 0.35);

    // The mix. Stakhanovites fill the mines (common::miner_room), agitators up to five a mark, a few Red Cavalry to see
    // the borders, and the rest of the plan is the Red Army: many, and cheap.
    let want_spark = if picks.is_empty() { 0.0 } else { 4.0 * (picks.len() as f64) };
    let mix: Vec<(&str, f64)> = if strained {
        vec![("miner", 0.66), ("line", 0.24), ("spark", 0.06), ("scout", 0.04)]
    } else {
        vec![("miner", 0.58), ("line", 0.28), ("spark", 0.10), ("scout", 0.04)]
    };
    let builds = [("miner", miner_body()), ("line", line_body()), ("spark", spark_body()), ("scout", scout_body())];
    let limits = [("miner", common::miner_room(me, &survey)), ("spark", want_spark), ("scout", 2.0)];
    common::recruit(
        &mut orders,
        me,
        &mix,
        |t, k| unit_name(me, t, k),
        &common::Recruit { reserve: 0.0, limits: &limits, builds: &builds, upkeep: 0.45 },
    );
    // The Red Army fights in ranks once research allows: every third recruit a rifleman, every third armored.
    let techs = me.get("techs");
    let (rifles, armor) = (techs.get("ranged").truthy(), techs.get("armor").truthy());
    if rifles || armor {
        let mut nth = me.path("units.byType.line").num_or(0.0);
        let spawns: Vec<Value> = orders.get("spawn").items().iter().map(|sp| {
            let mut sp = sp.clone();
            if sp.get("type").str() == "line" {
                nth += 1.0;
                match nth as u64 % 3 {
                    1 if rifles => sp.set("build", rifle_body()),
                    2 if armor => sp.set("modules", vec!["armor"]),
                    _ => {}
                }
            }
            sp
        }).collect();
        orders.set("spawn", spawns);
    }

    // Compute chooses research/type schedules; shared-CPU policy below replaces
    // the old split and makes every execution lane trade against one bank.
    let available = cpu.get("available");
    let types = common::calls(
        world,
        // Absolute allocations are deliberate: a fractional split made every event wake-up too small to issue even one
        // Order. Every type we field gets its base first (common::calls); the agitators change their orders rarely.
        &[("miner", 6400.0, 3.0), ("spark", 4800.0, 4.0), ("line", 6000.0, 4.0), ("scout", 4000.0, 6.0)],
        cpu.get("income").or(0.0),
        if available.truthy() { available.get("control").num() * 0.8 } else { f64::INFINITY },
        me.get("units").get("byType"),
    );
    common::budget(&mut orders, world, me, &common::Budget { research: 0.13, code: 0.66, techs: &TECHS, types: Some(&types) });
    let mut policy = cpu_policy(world, me, &orders, pressure, need);
    common::fit_cpu(&mut policy, me);
    orders.entry("compute").set("cpu", policy);

    let at = mem::get(me.get("ramCapital"), CLOCK);
    let mut clock = Clock { at: if at.truthy() { at } else { Value::obj() }, changed: false };
    // when the pact was torn up, on a page of its own (the clock's page has room for the timers alone)
    let broke = mem::get(me.get("ramCapital"), BROKE);
    let pact_leader = realm::leader(world).map(|l| l.get("id").num());
    let no_pacts = me.get("pacts").is_empty();
    // (a pact torn up stays torn up: no second one, whoever proposes it)
    let open = broke.is_undef();
    let pact = move |m: View| open && no_pacts && pact_leader.is_none_or(|l| m.get("from").num() != l);

    // The Manifesto, a line at a time: what the agitators are at, and the pact that bought the time for them.
    let by_id = realm::index(world);
    let name = |p: f64| by_id.get(p).get("name").str();
    let quote = QUOTES[((now / 14.0).floor() as usize + me_id.max(0.0) as usize) % QUOTES.len()];
    let line = if !broke.is_undef() && now - num(&broke) < 150.0 {
        "The pact is torn up. Workers of the old order, rise!".to_string()
    } else if !no_pacts && (now / 14.0).floor() as usize % 4 == 3 {
        "Peace with the strongest, for now. The revolution is patient.".to_string()
    } else {
        match picks.first() {
            Some(&p) if sparks > 0.0 && !name(p).is_empty() => {
                format!("{} {}: {}", if by_id.get(p).get("owner").num() < 0.0 { "Rallying" } else { "Agitating" }, name(p), quote)
            }
            _ => quote.to_string(),
        }
    };
    orders.set("memo", line);

    if lean || cycles_left() < 6000.0 {
        if cycles_left() > 2000.0 {
            common::answer(&mut orders, me, world, pact, &TECHS);
        }
        return affordable(orders, me, start);
    }
    // A pact first: the Manifesto's ally is the strongest faction left unpaired, the one that would otherwise march on its
    // new land (a partner's land is spared by sway, a price worth paying). Each faction is asked once in two minutes
    // (common::court), the first to say yes is the ally. A pact of 1939: once the Red Army is the biggest force on the
    // map (a fifth of the cores, 60 units, ten minutes in) the pact is torn up with its 30 s notice and the partner's
    // land becomes the next to be agitated; the Manifesto does not sign another.
    if no_pacts && broke.is_undef() && now >= 90.0 && common::every(now, 4.0, 0.0) {
        for f in contacts(world, me).into_iter().take(2) {
            let id = f.get("id").num();
            if common::court(&mut orders, world, me, id, |o| diplomacy::propose(o, me, vec![id], (), "Proletarians of all provinces, unite!"), &common::Court::default()) {
                break;
            }
        }
    }
    if !no_pacts && broke.is_undef() && now >= 600.0 && me.get("share").num() >= 0.2 && me.get("units").get("total").num() >= 60.0 {
        for p in me.get("pacts").iter() {
            orders.entry("leave").push(p.value());
        }
        concat(&mut orders, "ram", mem::put(BROKE, &Value::Num(now), "local", None, 1).unwrap_or_default());
    }
    revolution(&mut orders, me, &ms, now);
    bribes(&mut orders, me, &ms, &picks, &mut clock, now);
    landlord(&mut orders, world, me, &mut clock);
    common::answer(&mut orders, me, world, pact, &TECHS);
    if clock.changed {
        concat(&mut orders, "ram", mem::put(CLOCK, &clock.at, "local", None, 1).unwrap_or_default());
    }
    affordable(orders, me, start)
}

// The landlord's trade: tolls where its land leaves by sea or pass, a mine lease on a neighbour's border when its own
// miners leave slots idle, and largesse for its own restless provinces (the counter-revolution, 60◆ at a time).
fn landlord(orders: &mut Value, world: View, me: View, clock: &mut Clock) {
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
    // largesse (60◆) is paid before spawns (payable): it waits only on a bank under a quarter of a unit's price
    if me.get("bank").num() > 3.0 * me.get("costs").get("unit").num() && due(clock, "largesse", 15.0, now) {
        let mut worst: Option<View> = None;
        for p in world.get("provinces").iter() {
            let risk = p.get("risk").num();
            if p.get("owner").num() == me_id && risk > 0.08 && worst.is_none_or(|w| risk > w.get("risk").num()) {
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

// Stakhanovites: the economy. Every open slot in the plan is worked.
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

// The Red Army: those the plan sends to defend (common::defends) fight there, the rest capture the attack.
fn line(units: View, world: View, me: View) {
    // a call short of the plan lookup and an order keeps its units' standing orders a second longer
    if rts(me) && cycles_left() < 2200.0 {
        return;
    }
    let p = plan::for_units(me, units, world);
    let attack = num(p.get("attack"));
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        let u = nth(units, turn, world);
        let defend = common::defends(&p, u);
        let (mut verb, mut target) = if defend >= 0.0 { ("fight", defend) } else { ("capture", attack) };
        // a third of the army garrisons the land that is under the papers of others, the rest goes where the plan says
        let hold = p.get("hold").items();
        if defend < 0.0 && !hold.is_empty() && (u.get("id").num().abs() as usize) % 3 == 0 {
            (verb, target) = ("fight", num(&hold[((u.get("id").num().abs() as usize) / 3) % hold.len()]));
        }
        // with nothing to capture the soldiers fraternise: they join the agitators in the plan's marks
        if target.is_nan() || target < 0.0 {
            let sway = p.get("sway").items();
            if !sway.is_empty() {
                (verb, target) = ("persuade", num(&sway[(u.get("id").num().abs() as usize) % sway.len()]));
            }
        }
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
        // (a pact signed since the plan was made turns its target into a partner's land: a capture there is only refused)
        if verb == "capture" && realm::is_ally(me, realm::index(world).get(target).get("owner").num_or(-1.0)) {
            continue;
        }
        if let Some(o) = orders::to(u, verb, target, world, me, ()) {
            order(o);
        }
    }
}

// Agitators: each stands in the town ring of one of the plan's marks and persuades (loyalty falls, our favour climbs, the
// rival's soldiers lose heart and may come over). One that is at work on a mark still on the list stays there; the rest
// share the marks by id. With no mark they study at home (their papers go further with research).
fn spark(units: View, world: View, me: View) {
    if rts(me) && cycles_left() < 2200.0 {
        return;
    }
    let p = plan::for_units(me, units, world);
    let sway = p.get("sway");
    // (a plan without marks, from a unit that carried it abroad, says nothing: they keep their orders)
    if sway.is_undef() {
        return;
    }
    let marks: Vec<f64> = sway.items().iter().map(num).collect();
    let home = if num(p.get("home")).is_nan() { me.get("capital").num() } else { num(p.get("home")) };
    for turn in 0..units.len() {
        if rts(me) && cycles_left() < 2200.0 {
            break;
        }
        let u = nth(units, turn, world);
        let o = if marks.is_empty() {
            if u.get("at").num() == home && u.get("doing").str() == "study" {
                None
            } else {
                orders::to(u, "study", home, world, me, ())
            }
        } else {
            let busy = u.get("doing").str() == "persuade" && marks.contains(&u.get("target").num());
            if busy {
                None
            } else {
                let t = marks[(u.get("id").num().abs() as usize) % marks.len()];
                orders::to(u, "persuade", t, world, me, ())
            }
        };
        if let Some(o) = o {
            order(o);
        }
    }
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
            order(obj! { "id" => u.get("id"), "do" => "move", "target" => next, "power" => obj! { "drive" => 600, "armor" => 0, "weapon" => 0 } });
        }
    }
}

// ---- reflex ----

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

// The Red Army strikes the weakest rival it can reach and holds its ground; a shot costs 120 cycles at price level 1 and
// extra weapon power buys little for a lot, so the soldiers pay for the plain blow. Everyone steps out of a shell's way.
#[allow(unused_assignments)] // the lanes left, kept as they're spent even after the last read
fn reflex(units: View, near: View) -> Value {
    let mut out = Vec::new();
    let n = units.len();
    let start = if n > 0 { ((units.at(0).get("rts").get("bank").or(0.0) / 211.0).floor() % n as f64) as usize } else { 0 };
    let (mut returning, mut turn_cost) = (0.0, 1500.0_f64);
    for turn in 0..n {
        if cycles_left() < turn_cost + returning {
            break;
        }
        let began = cycles_left();
        let u = units.at((start + turn) % n);
        let ty = u.get("type").str();
        let ns = near.key(u.get("id").num());
        let r = u.get("rts");
        let level = common::price_level(u);
        let mut left = max(0.0, r.get("bank").or(0.0) - 700.0);
        let b = r.get("budget");
        let weapon_left = lane(b, "weapon", left);
        let drive_left = lane(b, "drive", left);
        let mut o = obj! { "id" => u.get("id") };
        // dodge a shell before it lands (own-observation data only)
        let pos = u.get("pos");
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
        let mut act = false;
        if let Some((x, y)) = hazard {
            o.set("step", obj! { "x" => if x != 0.0 && !x.is_nan() { x } else { 1.0 }, "y" => if y != 0.0 && !y.is_nan() { y } else { 1.0 } });
            let cost = level * (60.0 + 600.0) * 0.2;
            o.set("power", obj! { "drive" => if cost <= left && cost <= drive_left { 600 } else { 0 }, "armor" => 0, "weapon" => 0 });
            act = true;
        }
        if ty == "line" {
            let mut best: Option<View> = None;
            let mut score = 99.0;
            for t in ns.iter() {
                if !t.get("rival").truthy() {
                    continue;
                }
                let air = t.get("air").truthy();
                if t.get("d").num() > u.get("reach").num() || (air && !u.get("air").truthy()) {
                    continue;
                }
                let s = t.get("hp").num() / t.get("hpMax").num() - if t.get("range").truthy() { 0.18 } else { 0.0 } - if air { 0.11 } else { 0.0 };
                if s < score {
                    score = s;
                    best = Some(t);
                }
            }
            if let Some(t) = best {
                let shot_cost = (level * 120.0).ceil();
                if shot_cost <= left && shot_cost <= weapon_left {
                    o.set("hit", t.get("id"));
                    left -= shot_cost;
                }
                // riflemen keep their distance from bayonets (common::space_ranged); the rest hold
                common::space_ranged(u, ns, &mut o);
                if o.get("step").is_undef() {
                    o.set("hold", true);
                }
                act = true;
            }
        }
        if act {
            let before = cycles_left();
            order(o.clone());
            returning += before - cycles_left();
            out.push(o);
        }
        turn_cost = turn_cost.max(1.5 * (began - cycles_left()));
    }
    out.into()
}

// ---- Puzzles ----

fn research(puzzle: View) -> Value {
    common::solve(puzzle)
}

fn duel(challenge: View) -> Value {
    common::solve(challenge)
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
