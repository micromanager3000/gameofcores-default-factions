// Null Pointer Society: the shadow faction of CPU RTS matches. Its exploits (spies built with stealth) walk into
// rivals' capitals unseen, copy their research and sabotage their walls while the exceptions storm them. At home a
// watchpoint stands by the town of Address Zero and shoots the spies that come for ours.
//
// What spectators see: exploits abroad (SPY events when one is caught, and when one finishes stealing a tech), stolen
// techs the Society never researched, cloaked exceptions appearing inside a rival's province, and a memo that says
// which capital is being read. The army is the raiders' combined arms underneath: miners, a screen of nulls, ranged
// exceptions, watchpoints, fork bombs and garbage collectors once research or theft allows them.
//
// Built on the SDK in the gameofcores crate (docs part sdk), with the strategy the founders share in the `common`
// module at the end: targets, spawning room, cycles and research, deals and pacts, scouting and the puzzle solver.
use std::rc::Rc;

use gameofcores::*;
use shared::{air, capital, common, theme};
use shared::common::{hauler, freighter, escort};

program!(think, hauler, freighter, escort, miner, screen, skirm, runner, lancer, repairer, sentry, gun, wing, ghost, reflex, research, duel, challenge, look, build3d, live3d);

// Builds: (stat, points), in the order the game shows them.
type Build = &'static [(&'static str, f64)];
const MINER: Build = &[("hp", 2.0), ("speed", 2.0), ("strength", 1.0), ("carry", 4.0), ("vision", 1.0)];
const SCREEN: Build = &[("hp", 4.0), ("speed", 2.0), ("strength", 3.0), ("carry", 0.0), ("vision", 1.0)];
const RUNNER: Build = &[("hp", 3.0), ("speed", 3.0), ("strength", 2.0), ("carry", 0.0), ("vision", 2.0)];
const SKIRM: Build = &[("hp", 3.0), ("speed", 2.0), ("strength", 2.0), ("carry", 0.0), ("vision", 1.0), ("range", 1.0)];
// The watchpoint: eyes first. It sees a stealth-1 spy 5 cells out (6 with Scan), where a standard body sees it at 2.
const WATCH: Build = &[("hp", 2.0), ("speed", 1.0), ("strength", 0.0), ("carry", 0.0), ("vision", 5.0), ("range", 1.0)];
const WATCH_SCAN: Build = &[("hp", 1.0), ("speed", 1.0), ("strength", 0.0), ("carry", 0.0), ("vision", 6.0), ("range", 1.0)];
const SENTRY: Build = &[("hp", 2.0), ("speed", 2.0), ("strength", 1.0), ("carry", 0.0), ("vision", 3.0), ("range", 1.0)];
const GUN: Build = &[("hp", 2.0), ("speed", 1.0), ("strength", 2.0), ("carry", 0.0), ("vision", 1.0), ("range", 2.0)];
// The exploit: a spy. Stealth 1 once Cloak allows it, stealth 2 with Enhanced stealth (seen only a cell away, and a
// theft roll of 20% a second instead of 15%).
const GHOST: Build = &[("hp", 2.0), ("speed", 3.0), ("strength", 0.0), ("carry", 0.0), ("vision", 3.0), ("stealth", 1.0)];
const GHOST_DEEP: Build = &[("hp", 1.0), ("speed", 3.0), ("strength", 0.0), ("carry", 0.0), ("vision", 2.0), ("stealth", 2.0)];

// What to research, in order. Cloak comes fourth: it licenses stealth bodies, and from then on the exploits copy the
// dear techs from whoever has them, so the list past it is what's left to do ourselves. A tech a spy is copying now is
// left out of the list that second (think filters it).
const TECHS: [&str; 23] = [
    "extraction", "dash", "ranged", "cloak", "armor", "scan", "storage", "pierce", "repair", "capture", "antiAir",
    "artillery", "influence", "turbines", "seaTraversal", "mountainTraversal", "stealth", "computeBoost", "flight",
    "faith", "airWeapons", "cooling", "gate",
];

fn build(b: Build) -> Value {
    let mut o = Obj::new();
    for (k, v) in b {
        o.set(k, *v);
    }
    Value::Obj(o)
}

// Bribes and passage run on timers with a random gap, so no two worlds repeat: each fires `base` seconds after the
// last, give or take 60%. The next second of each lives in capital RAM page 224, written only when one fires.
const CLOCK: u32 = 224;
// Capital RAM page 225: the espionage ledger, {stolen, was}: techs stolen so far, and the techs spies were copying
// last second (so a tech that arrives while a spy copies it counts as stolen, not researched).
const LEDGER: u32 = 225;

struct Clock {
    at: Value,
    changed: bool,
}

fn due(clock: &mut Clock, name: &str, base: f64, now: f64) -> bool {
    let next = clock.at.get(name).clone();
    if !next.is_undef() && now < vn(&next) {
        return false;
    }
    let gap = if next.is_undef() { rand() } else { 0.4 + 1.2 * rand() };
    clock.at.set(name, now + round(base * gap));
    clock.changed = true;
    !next.is_undef()
}

fn mods(me: View, kind: &str) -> Vec<&'static str> {
    let t = me.get("techs");
    let has = |k: &str| t.get(k).truthy();
    let mut out = Vec::new();
    let mut add = |tech: &'static str| {
        if has(tech) {
            out.push(tech);
        }
    };
    match kind {
        "skirm" | "screen" => {
            add("dash");
            add("armor");
        }
        "sentry" => {
            add("scan");
            add("antiAir");
        }
        "gun" => {
            add("artillery");
            add("armor");
        }
        // a spy carries only its cloak: a module costs upkeep every second, and a spy never dashes
        "ghost" => add("cloak"),
        "repair" => {
            add("repair");
            add("dash");
        }
        "wing" => add("dash"),
        _ => {}
    }
    out
}

// Room for more units: up to the allowance, then past it while their upkeep stays under 40% of income (common::room).
fn army_room(me: View) -> f64 {
    let u = me.get("units");
    jmax(n(u.get("cap")) - n(u.get("total")) - n(u.get("queued")) - u.get("hiredOut").or(0.0), common::room(me, 0.4))
}

// Reserve fabrication CPU locally while constructing the spawn list. The engine processes spawns in order, so checking
// the same public bank three times would otherwise create predictable brownout/error spam. The running totals live in
// the orders (fabrication, treasury, unitPrice, slots), beside the world's price level.
fn spawn(orders: &mut Value, ty: &str, b: Build, modules: Vec<&str>) -> bool {
    spawn_build(orders,ty,||build(b),modules,None)
}

fn spawn_build(orders: &mut Value, ty: &str, body: impl FnOnce()->Value, modules: Vec<&str>, name: Option<String>) -> bool {
    let get = |o: &Value, k: &str| vn(o.get(k));
    let cost = (1200.0 + modules.len() as f64 * 600.0) * get(orders, "priceLevel");
    if orders.get("spawn").len() >= 3 || get(orders, "fabrication") < cost || get(orders, "treasury") < get(orders, "unitPrice") || get(orders, "slots") <= 0.0 {
        return false;
    }
    let (fabrication, treasury, price, slots) = (get(orders, "fabrication"), get(orders, "treasury"), get(orders, "unitPrice"), get(orders, "slots"));
    orders.set("fabrication", fabrication - cost);
    orders.set("treasury", treasury - price);
    orders.set("unitPrice", price + 4.0);
    orders.set("slots", slots - 1.0);
    let mut s=obj! { "type" => ty, "build" => body(), "modules" => modules };
    if let Some(name)=name{s.set("name",name);}
    orders.entry("spawn").push(s);
    true
}

// Each type's base call, in the order the types grow their calls past it (common::calls). The exploits come right
// after the miners: a spy's route search through rivals' land needs a real call, and a spy that can't think stands
// still inside a rival's walls.
const TYPES: [(&str, f64, f64); 10] = [
    ("miner", 5000.0, 1.0),
    ("ghost", 5000.0, 1.0),
    ("wing", 4200.0, 2.0),
    ("screen", 6000.0, 1.0),
    ("runner", 6000.0, 1.0),
    ("skirm", 6000.0, 1.0),
    ("lancer", 6000.0, 2.0),
    ("sentry", 6000.0, 1.0),
    ("gun", 6000.0, 1.0),
    ("repairer", 5000.0, 1.0),
];

// Spawns the ◆ left can pay. Deals we accept that ask us for ◆, bribes, and the ◆ the spies copying notes this second
// will spend (25 each) are paid before spawns, and a spawn the bank can't pay is only refused, so it waits a second.
fn payable(mut orders: Value, me: View, held: f64) -> Value {
    let spawn = orders.get("spawn");
    if !spawn.truthy() || spawn.is_empty() {
        return orders;
    }
    let yes = orders.get("accept").items();
    let mut left = me.get("bank").num() - held;
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

// CPU RTS uses one balance, rather than old split purses. These lanes are caps, not entitlements: an idle lane leaves
// cycles available to a real tactical event. Research puts a share of income into its savings each second, with no
// sticky reserve: a faction must spend on expansion first.
fn cpu_policy(me: View, plan: &Value, type_calls: &Value) -> Value {
    // the calls of the types we field, so the control lane never scales them below what each asked for
    let bt = me.get("units").get("byType");
    let mut fielded = 0.0;
    if let Value::Obj(calls) = type_calls {
        for (t, c) in &calls.0 {
            if bt.get(t).truthy() {
                fielded += vn(c.get("share"));
            }
        }
    }
    let invaded = vn(plan.get("defend")) >= 0.0 || or0(plan.get("threat")) > 0.0;
    let cpu = rts_cpu(me);
    let income = cpu.get("income").or(0.0);
    // When land loss makes combat spend exceed income, research waits before the bank empties.
    let running: f64 = cpu.get("last").entries().filter(|(lane, _)| *lane != "research")
        .map(|(_, cost)| cost.num_or(0.0)).sum();
    let research = if invaded && cpu.get("bank").num_or(0.0) < income * 90.0 {
        (0.85 - running / income.max(1.0)).clamp(0.0, 0.3)
    } else { 0.3 };
    let pick = |a: f64, b: f64| if invaded { a } else { b };
    obj! {
        "limits" => obj! {
            // think grows with the army, income and its own recent peak (with room to spare: a think that runs out sets
            // no new policy, so the limit has to be there before it's needed)
            "think" => round(jmin(jmax(56000.0, income * 0.5), jmax(jmax(24000.0 + me.get("units").get("total").or(0.0) * 200.0, income * 0.4), me.get("compute").get("thinkPeak").or(0.0) * 1.5))),
            // Three active combat controllers can each need several thousand cycles to route a squad. Keep a real shared
            // ceiling instead of splitting a sub-4k purse across them, and grow it with income as the calls grow.
            "control" => round(jmax(jmax(pick(50000.0, 42000.0), income * 0.6), fielded)),
            "reflex" => pick(0.30, 0.20),
            "research" => research,
            // the exploits are dear to keep (stealth costs 180 cycles a second a level): room for them before anyone stalls
            "upkeep" => 0.45,
            "drive" => pick(0.18, 0.11),
            "armor" => pick(0.20, 0.12),
            // the cloak lane pays for the exploits in rivals' land
            "cloak" => 0.07,
            "weapon" => pick(0.24, 0.16),
            "abilities" => pick(0.18, 0.10),
            // A basic body (1,200 cycles at price level 1) must remain buyable while fighting; surplus is still shared
            // with movement and fire when there is no build order.
            "fabrication" => pick(4800.0, 5200.0) * common::price_level(me),
            "gate" => 0.03,
            "duel" => 0.10,
            "challenge" => 0.05,
            "visuals" => 0.02,
        },
        "reserve" => obj! { "think" => 3000, "reflex" => pick(4500.0, 2600.0), "research" => 0 },
        // the exploits are paid right after the miners: a spy whose upkeep lapses stalls, and a stalled spy hides from
        // no one
        "upkeepOrder" => vec!["miner", "ghost", "screen", "sentry", "repairer", "skirm", "gun", "runner", "lancer", "wing"],
    }
}

// One route search in think instead of one per unit type each second: the plan carries a field toward where the army
// goes (the province to defend, else the attack), cut to the steps from our land and the free or allied land next to
// it (where a unit stepping out stands), and path::to routes along it. None when path::home already leads there from
// all our land; it can't from an exclave or land over the sea. The last plan's field to the same place serves for 10 s.
// A search is budgeted at up to about 300 cycles a province, so it runs only when think can pay for that and still
// have `after` cycles to finish; until then the last field to the same place serves, however old.
fn road(world: View, me: View, to: f64, after: f64) -> Option<Value> {
    if to.is_nan() || to < 0.0 {
        return None;
    }
    let by_id = realm::index(world);
    let ok = path::friendly(world, me);
    let me_id = me.get("id");
    let mut from = Vec::new();
    let mut apart = false;
    for p in world.get("provinces").iter() {
        if !same(p.get("owner"), me_id) {
            continue;
        }
        from.push(n(p.get("id")));
        let hops = n(p.get("hops"));
        if hops > 0.0
            && !p.get("adj").iter().any(|a| {
                let q = by_id.at(a);
                q.truthy() && same(q.get("owner"), me_id) && q.get("hops").is_num() && q.get("hops").num() == hops - 1.0
            })
        {
            apart = true;
        }
        for id in realm::reach(p, me) {
            let q = by_id.get(id);
            if q.truthy() && !same(q.get("owner"), me_id) && ok(q) {
                from.push(id);
            }
        }
    }
    if !apart && path::home(world, me, n(me.get("capital")), to).is_some() {
        return None;
    }
    let now = world.get("seconds").or(0.0).floor();
    let last = mem::get(me.get("ramCapital"), 0);
    let kept = if last.truthy() && last.get("field").truthy() && matches!(last.get("field").get("to"), Value::Num(x) if *x == to) {
        Some(last.get("field").clone())
    } else {
        None
    };
    if let Some(k) = &kept {
        if now - vor0(k.get("at")) < 10.0 {
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

fn think_emergency() -> Value {
    obj! { "compute" => obj! { "cpu" => obj! {
            "limits" => obj! { "think" => 24000, "control" => 0.5, "upkeep" => 0.65,
                "research" => 0, "influence" => 0, "fabrication" => 0, "visuals" => 0,
                "reflex" => 0.15, "duel" => 0.08, "challenge" => 0.02,
                "drive" => 0.08, "armor" => 0, "weapon" => 0.08,
                "abilities" => 0, "cloak" => 0, "gate" => 0 },
            "reserve" => obj! {}, "upkeepOrder" => vec!["miner"]
        } } }
}

fn think_recovery(me: View) -> Value {
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
    return obj! { "compute" => obj! { "cpu" => cpu, "types" => types },
        "ram" => mem::put(244, &recovery, "local", None, 1).unwrap_or_default(),
        "propaganda" => Value::arr(), "preach" => Value::arr(), "rent" => 0,
        "memo" => "Recovering production" };
}

// Output serialization runs after think returns; RAM writes are paid from what it leaves.
fn think_return_cost(orders: &Value) -> f64 {
    let began = cycles_left();
    let encoded = orders.to_json();
    let serialization = began - cycles_left();
    let ram: f64 = orders.get("ram").items().iter().map(|op| {
        let bytes = op.get("bytes");
        if bytes.is_undef() { 1.0 } else { (bytes.len() as f64 / 2.0).ceil().max(1.0) }
    }).sum();
    let messages: f64 = orders.get("send").items().iter().take(8).map(|message| {
        let bytes = match message.get("body") {
            Value::Str(body) => body.len(),
            Value::Bytes(body) => body.len(),
            Value::Arr(body) => body.len(),
            Value::Null | Value::Undef => 0,
            // A literal {"$b": ...} is also accepted; its JSON is longer than its decoded bytes.
            body => body.to_json().len(),
        };
        50.0 + bytes as f64 / 2.0
    }).sum();
    serialization * 1.5 + encoded.len() as f64 / 32.0 + ram + messages + 600.0
}

fn think_room(orders: &Value, work: f64) -> bool {
    if cycles_left() < 2500.0 + work { return false; }
    let tail = think_return_cost(orders);
    cycles_left() >= tail + work
}

fn think(world: View, me: View) -> Value {
    if !me.path("rts.cpu").truthy() {
        return common::govern(world, me, 0.4, || strategy(world, me));
    }
    if cycles_left() < 3000.0 { return think_emergency(); }
    if world.get("logistics").truthy() && me.path("units.total").num_or(0.0) < 4.0
        && me.path("rts.cpu.income").num_or(0.0) < 30000.0 {
        return common::govern(world, me, 0.4, || strategy(world, me));
    }
    // The recent peak includes execution and ABI serialization, but excludes RAM application.
    let peak = me.path("compute.thinkPeak").num_or(0.0);
    let writes = me.get("ramCapital").len() as f64 * 32.0 + 32.0;
    let orders = common::govern(world, me, 0.4, || {
        // Cargo support and recovery must survive a planner admission failure.
        if cycles_left() < 18000.0 || cycles_left() < peak * 1.1 + writes + 600.0 {
            return think_recovery(me);
        }
        let mut orders = strategy(world, me);
        // Governance scans visible bodies after strategy when it checks proposed recruits.
        let review = 1500.0 + world.get("units").len() as f64 * 15.0
            + me.path("units.total").num_or(0.0) * 100.0;
        if !orders.get("spawn").is_empty() && !think_room(&orders, review) {
            orders.set("spawn", Value::arr());
        }
        orders
    });
    if !think_room(&orders, 0.0) { return think_emergency(); }
    orders
}

fn strategy(world: View, me: View) -> Value {
    let rts = me.get("rts");
    if rts.truthy() && cycles_left() < 3000.0 {
        return common::recovery(world, me);
    }
    // Short of a full think (16,000 cycles, or 125% of its recent peak), set only the CPU policy: a think that
    // runs out sets none, and its limit, a share of income, stays too small when income falls, every second after.
    if rts.truthy() && cycles_left() < jmax(16000.0, 1.25 * me.get("compute").get("thinkPeak").or(0.0)) {
        return common::recovery(world, me);
    }
    let t = me.get("techs");
    let has = |k: &str| t.get(k).truthy();
    let bt = me.get("units").get("byType");
    let rim = common::frontier(world, me);
    // the best land first, marked down where the army couldn't hold on to it (common::can_hold)
    let hold = common::can_hold(world, me, false);
    let ranked = realm::rank(world, me, rim, |w, m, q| common::score(w, m, q) - if hold(q) { 0.0 } else { common::OVER_WATER });
    // Survey scans provinces for mines and defense, and may scan visible fighters for a picket.
    // Admit that work before it starts, retaining the CPU policy and finalization allowance.
    // At five fighters or fewer, even one invader mobilizes everyone without a picket census.
    let survey = 60.0 * world.get("provinces").len() as f64
        + if common::fighters(me) > 5.0 { 40.0 * world.get("units").len() as f64 } else { 0.0 };
    if rts.truthy() && cycles_left() < survey + 7000.0 { return think_recovery(me); }
    let mut plan = common::survey(world, me, &ranked, 2.0);
    // the hunt's census is budgeted at about 50 cycles a unit in sight: only when think can still finish after it
    if has("flight") {
        let seen = world.get("units");
        let census = 55.0 * if seen.truthy() { seen.len() as f64 } else { 0.0 } + 26000.0;
        let hunt = if cycles_left() > census { hunt_for(world, me, &plan) } else { plan.get("attack").clone() };
        plan.set("hunt", hunt);
    }
    // the rest of think (spawns, budget, trade and diplomacy) is budgeted at up to 16,000 cycles after the search
    let defend = vn(plan.get("defend"));
    if let Some(field) = road(world, me, if defend >= 0.0 { defend } else { vn(plan.get("attack")) }, 16000.0) {
        plan.set("field", field);
    }
    // the exploits abroad: who's copying what where (think sees its own units in full)
    let spies = nullptr_ext::census(world, me);
    // ◆ the spies hold: 25 a second for each one at work this second
    let held = 25.0 * (spies.stealing.len() + spies.sabotaging) as f64;
    let cpu = rts_cpu(me);
    // Think spends the shared bank. The fabrication lane keeps its own remaining allowance.
    let fabrication = if !rts.truthy() { 1_000_000_000.0 } else {
        common::fabrication_room(me).unwrap_or_else(|| rts.get("bank").num_or(0.0))
    };
    let mut orders = obj! {
        "policy" => obj! { "extraction" => if defend >= 0.0 { 0.38 } else { 0.5 }, "autonomy" => "direct", "integrate" => "tolerate", "invest" => true },
        "spawn" => Value::arr(),
        "provinces" => Value::arr(),
        "fabrication" => fabrication,
        "treasury" => n(me.get("bank")) - held,
        "unitPrice" => me.get("costs").get("unit"),
        "slots" => army_room(me),
        "priceLevel" => common::price_level(me),
    };
    plan::share(&mut orders, &plan, me);
    if rts.truthy() && cycles_left() < 7000.0 {
        let mut recovery = if cycles_left() < 3000.0 { think_emergency() } else { think_recovery(me) };
        recovery.set("memo", "Recovering production");
        return recovery;
    }

    let count = |k: &str| bt.get(k).or(0.0);
    let (miners, screens, runners, skirms, lancers) = (count("miner"), count("screen"), count("runner"), count("skirm"), count("lancer"));
    let (sentries, guns, ghosts, repairers) = (count("sentry"), count("gun"), count("ghost"), count("repairer"));
    let total = me.get("units").get("total").or(0.0);
    let spawning = |o: &Value| o.get("spawn").len();
    // Maintain a small mining base first, no bigger than what our mines can use (common::miner_room: past that a miner
    // stands idle); then rotate combat specialties instead of massing one dominant body. Three spawns is also the
    // engine's normal cap.
    if miners < jmax(6.0, (total * 0.3).floor()) && miners < common::miner_room(me, &plan) {
        spawn(&mut orders, "miner", MINER, vec![]);
    }
    // A small air wing only if Flight ever arrives (stolen, most likely): a tenth of the army, between 2 and 4 fliers.
    let wings = count("wing");
    if spawning(&orders) < 3 && has("flight") && wings < jmin(4.0, jmax(2.0, (total / 10.0).floor())) {
        let frame=common::airframe(me,wings);
        let serial=me.path("units.spawned").num_or(0.0)+spawning(&orders) as f64+1.0;
        spawn_build(&mut orders,"wing",||frame.build(),mods(me,"wing"),Some(format!("{} {serial:.0}",frame.name)));
    }

    // One exploit after eight units, a second after sixteen, while the treasury can pay for espionage.
    // Nine seconds between recruits gives each spy time to leave fabrication.
    let marks = world.get("factions").iter().filter(|f| realm::is_rival(world, me, n(f.get("id"))) && n(f.get("capital")) >= 0.0).count();
    let now = world.get("seconds").or(0.0).floor();
    let bank = n(me.get("bank"));
    let want_spies = if total >= 16.0 && bank >= 350.0 { 2.0 } else if total >= 8.0 && bank>=150.0 { 1.0 } else { 0.0 };
    if has("cloak") && marks > 0 && common::every(now, 8.0, 0.0) && jmax(ghosts, spies.count as f64) < want_spies {
        spawn(&mut orders, "ghost", if has("stealth") { GHOST_DEEP } else { GHOST }, mods(me, "ghost"));
    }
    if screens < jmax(3.0, ((skirms + guns + 1.0) * 0.4).floor()) {
        spawn(&mut orders, "screen", SCREEN, mods(me, "screen"));
    }
    // Ranged is its own license. Before it arrives, fast melee runners scout and pull defenders out of position without
    // issuing invalid range spawns.
    if !has("ranged") && runners < 3.0 {
        spawn(&mut orders, "runner", RUNNER, mods(me, "screen"));
    }
    if has("pierce") && has("ranged") && lancers < jmax(2.0, (screens / 2.0).ceil()) {
        spawn(&mut orders, "lancer", SKIRM, if has("dash") { vec!["pierce", "dash"] } else { vec!["pierce"] });
    }
    if has("ranged") && skirms < jmax(4.0, screens * 2.0) {
        spawn(&mut orders, "skirm", SKIRM, mods(me, "skirm"));
    }
    // One watchpoint for the gate of Address Zero as soon as range allows it (eyes for six cells once Scan is in), then
    // the raiders' sentry with the army once Scan is in.
    if spawning(&orders) < 3 && has("ranged") && sentries < 1.0 {
        spawn(&mut orders, "sentry", if has("scan") { WATCH_SCAN } else { WATCH }, mods(me, "sentry"));
    }
    if spawning(&orders) < 3 && has("scan") && has("ranged") && sentries >= 1.0 && sentries < 2.0 {
        spawn(&mut orders, "sentry", SENTRY, mods(me, "sentry"));
    }
    if spawning(&orders) < 3 && has("artillery") && guns < jmax(1.0, (skirms / 4.0).floor()) {
        spawn(&mut orders, "gun", GUN, mods(me, "gun"));
    }
    // Repairers can sustain the exposed screen or themselves after a retreat.
    if spawning(&orders) < 3 && has("repair") && has("dash") && repairers < jmax(1.0, (skirms / 5.0).floor()) {
        spawn(&mut orders, "repairer", SCREEN, mods(me, "repair"));
    }
    // Absolute controller allowances keep an unlocked specialist viable after miners and screens have consumed their
    // ordinary route passes. A war unit's call fits one route search for the units abroad (march).
    let income = cpu.get("income").or(0.0);
    let control = if cpu.truthy() && cpu.get("available").truthy() { n(cpu.get("available").get("control")) * 0.8 } else { f64::INFINITY };
    let type_calls = common::calls(world, &TYPES, income, control, bt);
    // common::budget keeps its useful research choice, rent choice and type schedules. In CPU RTS the old split is
    // ignored; cpu_policy below atomically installs caps and reserves for the one shared balance. A tech a spy is
    // copying now is left to the spy.
    let techs: Vec<&str> = TECHS.iter().copied().filter(|t| !spies.stealing.iter().any(|s| s == t)).collect();
    let budget = common::Budget { research: if has("repair") { 0.06 } else { 0.12 }, code: 0.72, techs: &techs, types: Some(&type_calls) };
    common::budget(&mut orders, world, me, &budget);
    if spies.count > 0 && n(me.get("bank")) < 1200.0 {
        orders.set("rent", 0);
    }
    let mut policy = cpu_policy(me, &plan, &type_calls);
    common::fit_cpu(&mut policy, me);
    orders.entry("compute").set("cpu", policy);
    if rts.truthy() && !think_room(&orders, 3000.0) { return orders; }
    // the ledger: techs stolen so far, counted when a tech a spy was copying arrives (page 225)
    let ledger = mem::get(me.get("ramCapital"), LEDGER);
    let mut stolen = vor0(&ledger.get("stolen"));
    let researching = me.path("research.tech");
    for w in ledger.get("was").items() {
        if let Value::Str(w) = w {
            if has(w) && !is_str(researching, w) {
                stolen += 1.0;
            }
        }
    }
    let was: Vec<Value> = spies.stealing.iter().map(|s| Value::from(*s)).collect();
    let sab = vor0(&ledger.get("sab")) + spies.sabotaging as f64;
    let ledger_now = obj! { "stolen" => stolen, "was" => was, "sab" => sab };
    if nullptr_ext::DEBUG && world.get("seconds").or(0.0).floor() % 120.0 == 0.0 {
        log(&format!(
            "ledger: stolen {} sabotage-seconds {} spies {} stealing {} sabotaging {} bank {} stalled {} upkeep {}/{}",
            stolen, sab, spies.count, spies.stealing.len(), spies.sabotaging, n(me.get("bank")), me.path("units.stalled").or(0.0),
            me.path("rts.cpu.spent.upkeep").or(0.0).round(), me.path("rts.cpu.income").or(0.0).round()
        ));
    }
    if !same_list(&ledger_now, &ledger) {
        let ops = mem::put(LEDGER, &ledger_now, "local", None, 1).unwrap_or_default();
        match orders.entry("ram") {
            Value::Arr(a) => a.extend(ops),
            x => *x = Value::Arr(ops),
        }
    }
    orders.set("memo", nullptr_ext::memo(world, me, &spies, stolen, sab, skirms, screens));
    // A short second keeps what's decided so far (the plan, spawns and the CPU policy): trade and diplomacy wait a
    // second rather than run think out of cycles and lose it all.
    let diplomacy = 3000.0 + 500.0 * world.get("factions").len() as f64;
    if (rts.truthy() && !think_room(&orders, diplomacy)) || cycles_left() < 5000.0 {
        return orders;
    }
    let pact_leader = realm::leader(world);
    let at = mem::get(me.get("ramCapital"), CLOCK);
    let mut clock = Clock { at: if at.truthy() { at } else { Value::obj() }, changed: false };
    warlord(&mut orders, world, me, &plan, &mut clock, held);
    if cycles_left() > 2000.0 && (!rts.truthy() || think_room(&orders, 2500.0)) {
        let pact = |m: View| me.get("pacts").is_empty() && pact_leader.is_none_or(|l| !same(m.get("from"), l.get("id")));
        common::answer(&mut orders, me, world, pact, &techs);
        nullptr_ext::keep_fund(&mut orders, me, held);
    }
    if clock.changed {
        let ops = mem::put(CLOCK, &clock.at, "local", None, 1).unwrap_or_default();
        match orders.entry("ram") {
            Value::Arr(a) => a.extend(ops),
            x => *x = Value::Arr(ops),
        }
    }
    payable(orders, me, held)
}

/// Whether the ledger to write says what the one in RAM says.
fn same_list(a: &Value, b: &Value) -> bool {
    if vor0(a.get("stolen")) != vor0(b.get("stolen")) || vor0(a.get("sab")) != vor0(b.get("sab")) {
        return false;
    }
    let (x, y) = (a.get("was").items(), b.get("was").items());
    x.len() == y.len() && x.iter().zip(y).all(|(p, q)| matches!((p, q), (Value::Str(p), Value::Str(q)) if p == q))
}

// Where the air wing hunts: the province outside our land where we see the most rival miners and ranged units (miners
// count double: they are the economy), or else the plan's target. Flying, the wing crosses anything.
fn hunt_for(world: View, me: View, plan: &Value) -> Value {
    let by_id = realm::index(world);
    let me_id = me.get("id");
    // weight by province, in the order first seen
    let mut prey: Vec<(f64, f64)> = Vec::new();
    for u in world.get("units").iter() {
        let b = u.get("build");
        if same(u.get("owner"), me_id) || !realm::is_rival(world, me, n(u.get("owner"))) || (b.truthy() && b.get("wings").truthy()) {
            continue;
        }
        let w = if is_str(u.get("doing"), "gather") || (b.truthy() && n(b.get("carry")) >= 3.0) {
            2.0
        } else if b.truthy() && b.get("range").truthy() {
            1.0
        } else {
            0.0
        };
        if w != 0.0 {
            let at = n(u.get("at"));
            match prey.iter_mut().find(|(p, _)| *p == at) {
                Some(slot) => slot.1 += w,
                None => prey.push((at, w)),
            }
        }
    }
    let (mut best, mut most) = (-1.0, 0.0);
    for &(k, w) in &prey {
        let q = by_id.get(k);
        if !q.truthy() || same(q.get("owner"), me_id) || realm::is_ally(me, n(q.get("owner"))) {
            continue;
        }
        if w > most {
            most = w;
            best = k;
        }
    }
    if best >= 0.0 {
        Value::Num(best)
    } else {
        plan.get("attack").clone()
    }
}

// The warlords' trade: a bribe to soften the target before the push, passage bought from a neighbour that isn't the
// target, so columns cross its land in peace, and largesse to keep what they took.
fn warlord(orders: &mut Value, world: View, me: View, plan: &Value, clock: &mut Clock, held: f64) {
    let now = world.get("seconds").or(0.0).floor();
    let by_id = realm::index(world);
    let me_id = me.get("id");
    // ◆ the spawns and the spies' thefts this second will pay don't go on bribes
    let coin = n(me.get("bank")) - held - orders.get("spawn").len() as f64 * (n(me.get("costs").get("unit")) + 8.0);
    let attack = vn(plan.get("attack"));
    let target = if attack >= 0.0 { by_id.get(attack) } else { View::undefined() };
    let owner = target.get("owner");
    if target.truthy()
        && n(owner) >= 0.0
        && realm::is_rival(world, me, n(owner))
        && coin > 900.0
        && sway::reach(world, me, n(target.get("id")))
        && due(clock, "bribe", 15.0, now)
    {
        sway::bribe(orders, n(target.get("id")), 150.0);
    }
    // propaganda on the target too once it has influence, so it may turn to us before the push lands (a standing order,
    // worked out again every 9 seconds)
    if me.get("techs").get("influence").truthy() && common::every(now, 8.0, n(me_id)) {
        orders.set("propaganda", Value::arr());
        if target.truthy() && n(owner) >= 0.0 && realm::is_rival(world, me, n(owner)) && sway::reach(world, me, n(target.get("id"))) {
            let income = me.get("rts").get("cpu").get("income").or(me.get("compute").get("cycles").or(14000.0));
            sway::propaganda(orders, n(target.get("id")), round(income * 0.05));
        }
    }
    // passage through a neighbour whose land touches the target, so the column can walk across it to the fight
    if target.truthy()
        && coin > 250.0
        && !me.get("contracts").iter().any(|c| !c.get("ended").truthy() && c.get("get").get("passage").truthy())
        && due(clock, "passage", 120.0, now)
    {
        let owned_next_to = |p: View, q: View| {
            p.get("adj").iter().any(|a| {
                let r = by_id.at(a);
                r.truthy() && same(r.get("owner"), q)
            })
        };
        let borders = |f: View, q: View| world.get("provinces").iter().any(|p| same(p.get("owner"), f) && owned_next_to(p, q));
        let host = world
            .get("factions")
            .iter()
            .filter(|f| f.get("alive").truthy() && !same(f.get("id"), me_id) && !same(f.get("id"), owner) && !me.get("allies").has_num(n(f.get("id"))))
            .find(|f| borders(f.get("id"), me_id) && owned_next_to(target, f.get("id")));
        if let Some(host) = host {
            let deal = obj! { "give" => obj! { "coin" => 60 }, "ask" => obj! { "passage" => obj! { "units" => 12, "seconds" => 300 } } };
            diplomacy::send(orders, host.get("id"), "passage for our columns", deal);
        }
    }
    // largesse in the most restless province we hold, so land we took doesn't revolt straight back to its neighbour: it
    // waits only on a bank under a quarter of a unit's price once this second's spawns are paid
    if coin >= 60.0 && due(clock, "largesse", 25.0, now) {
        let mut worst: Option<View> = None;
        for p in world.get("provinces").iter() {
            if same(p.get("owner"), me_id) && n(p.get("risk")) > 0.08 && worst.is_none_or(|w| n(p.get("risk")) > n(w.get("risk"))) {
                worst = Some(p);
            }
        }
        if let Some(w) = worst {
            sway::bribe(orders, n(w.get("id")), 60.0);
        }
    }
}

/// The `i`-th unit of a turn that starts from a different unit each second, so each gets its turn when cycles run short.
fn turn(units: View, i: usize, world: View) -> View {
    let k = world.get("seconds").or(0.0).floor();
    units.at(((i as f64 + k) % units.len() as f64) as usize)
}

fn miner(units: View, world: View, me: View) {
    if common::recover_miners(units, world, me) { return; }
    let rts = me.get("rts").truthy();
    for i in 0..units.len() {
        if rts && cycles_left() < 2200.0 {
            break;
        }
        if let Some(o) = orders::gather(turn(units, i, world), world, me) {
            order(o);
        }
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

// Defend with a fight (those the plan sends: common::defends); attack with a capture, since free land and rivals' towns
// fall only to units capturing them.
fn war_order(u: View, world: View, me: View) -> Option<Value> {
    let p = plan::of(me, u);
    let defend = common::defends(&p, u);
    let defending = defend >= 0.0;
    let target = if defending { defend } else { vn(p.get("attack")) };
    if target.is_nan() || target < 0.0 {
        return orders::to(u, "fight", n(me.get("capital")), world, me, ());
    }
    // A carried plan can outlive a pact or change of ownership. Return for new orders before attacking an ally.
    if !defending {
        let owner=realm::index(world).get(target).get("owner").num_or(-1.0);
        let conduct=u.get("conduct");
        if realm::is_ally(me,owner) || (conduct.get("seconds").num_or(0.0)>0.0 && conduct.get("from").num_or(-2.0)==owner) {
            return orders::to(u,"fight",n(me.get("capital")),world,me,());
        }
    }
    // a beaten army falls back to the stage, and reinforcements wait there for a wave (common::front)
    if !defending {
        if let Some(o) = common::front(u, world, me, &p) {
            return o;
        }
    }
    let mut o = carry(orders::to(u, if defending { "fight" } else { "capture" }, target, world, me, ()), u, &p, me);
    if let Some(o) = &mut o {
        o.set("alert", if is_str(u.get("type"), "sentry") { 8 } else { 7 });
    }
    o
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

// The war units' turn, from a rotating start so each gets its turn when cycles run short. A call with room for a route
// search and the orders after it takes the units abroad first: one of them may need the search (path::to), which
// serves every unit sent the same way. A call handed less than one order's worth (the control lane ran short after
// think) keeps its standing orders.
fn march(units: View, world: View, me: View) {
    let mut list = Vec::new();
    let start = world.get("seconds").num_or(0.0) as usize;
    for i in 0..units.len() {
        if me.get("rts").truthy() && cycles_left() < 3200.0 { break; }
        list.push(units.at((start + i) % units.len()));
    }
    march_all(&list, world, me, false);
}

fn march_all(units: &[View], world: View, me: View, rotate: bool) {
    if me.get("rts").truthy() && cycles_left() < 2200.0 {
        return;
    }
    let first = cycles_left() >= 300.0 * world.get("provinces").len() as f64 + 3000.0;
    let (mut queue, mut home) = (Vec::new(), Vec::new());
    let k = if rotate { world.get("seconds").or(0.0).floor() as usize } else { 0 };
    for i in 0..units.len() {
        if me.get("rts").truthy() && cycles_left() < 3000.0 { break; }
        let u = units[(i + k) % units.len()];
        if !first || plan::here(me, u) {
            home.push(u);
        } else {
            queue.push(u);
        }
    }
    // the units abroad go to the front, the last one met first
    queue.reverse();
    queue.extend(home);
    let mut budget = ControlBudget::new(me);
    for u in queue {
        if !budget.has_room(me) { break; }
        let began = cycles_left();
        if !budget.admits(u, world, me) { continue; }
        if let Some(o) = war_order(u, world, me) {
            if !ControlBudget::submit(o) { break; }
        }
        budget.finish(began);
    }
}

fn screen(units: View, world: View, me: View) {
    march(units, world, me);
}

fn skirm(units: View, world: View, me: View) {
    march(units, world, me);
}

fn runner(units: View, world: View, me: View) {
    march(units, world, me);
}

fn repairer(units: View, world: View, me: View) {
    march(units, world, me);
}

// The watchpoints: the first stands by the town of the capital with its eyes open eight cells out, so a spy that comes
// for our research is seen five or six cells away and shot. The rest march with the army.
fn sentry(units: View, world: View, me: View) {
    if me.get("rts").truthy() && cycles_left() < 2200.0 {
        return;
    }
    let cap = n(me.get("capital"));
    let mut rest = Vec::new();
    for (i, u) in units.iter().enumerate() {
        if i == 0 && cap >= 0.0 {
            if let Some(mut o) = orders::hold(u, Point::new(0.0, 1.0), world, me, Some(cap)) {
                o.set("alert", 8);
                order(o);
            }
            continue;
        }
        rest.push(u);
    }
    if !rest.is_empty() {
        march_all(&rest, world, me, true);
    }
}

fn lancer(units: View, world: View, me: View) {
    skirm(units, world, me);
}

fn gun(units: View, world: View, me: View) {
    let rts = me.get("rts").truthy();
    for i in 0..units.len() {
        if rts && cycles_left() < 2200.0 {
            break;
        }
        let u = turn(units, i, world);
        let p = plan::of(me, u);
        let defend = Value::Num(common::defends(&p, u));
        let target = if vn(&defend) >= 0.0 { &defend } else { p.get("attack") };
        let r = u.get("rts");
        let set_up = r.get("deployed").truthy() || r.get("deploying").truthy();
        let pack_up = || obj! { "id" => u.get("id"), "deploy" => false, "alert" => 8 };
        // A deployed gun cannot move. If its strategic target moved, first pack it into the same order that begins the
        // new route; never set up simply because an order exists while it is still travelling from its spawn.
        if target.is_undef() || vn(target) < 0.0 {
            if set_up {
                order(pack_up());
            }
            continue;
        }
        let stage = p.get("stage");
        let under_pressure = is_str(u.get("why"), "attacked") && set_up;
        if under_pressure && vn(stage) >= 0.0 && !same_v(u.get("at"), stage) {
            match orders::to(u, "move", vn(stage), world, me, ()) {
                Some(mut retreat) => {
                    retreat.set("deploy", false);
                    retreat.set("alert", 8);
                    order(retreat);
                }
                None => order(pack_up()),
            }
            continue;
        }
        let o = carry(orders::to(u, "fight", vn(target), world, me, ()), u, &p, me);
        if set_up {
            if !same_v(u.get("at"), target) || n(u.get("to")) >= 0.0 {
                match o {
                    Some(mut o) => {
                        o.set("deploy", false);
                        o.set("alert", 8);
                        order(o);
                    }
                    None => order(pack_up()),
                }
            }
            continue;
        }
        // Only a gun already standing in the target province may set up. `to < 0` tells arrival from a route whose
        // first leg has not started.
        if same_v(u.get("at"), target) && n(u.get("to")) < 0.0 {
            order(obj! { "id" => u.get("id"), "deploy" => true, "alert" => 8 });
        } else if let Some(mut o) = o {
            o.set("alert", 8);
            order(o);
        }
    }
}

// The air wing hunts where think's plan says (plan.hunt): it fights there, picking off miners and archers (reflex),
// and raids the province when nothing it can hit is left. A flier can't capture, mine, persuade or study. Abroad, with
// only the core of the plan to go on, a flier keeps the hunt it's on.
fn wing(units: View, world: View, me: View) {
    // a flier a minute up comes down on your nearest land before anything else, even short of cycles: its burn climbs
    // the longer it flies, and an unpaid one crashes
    for u in units.iter() {
        let a = u.path("rts.air");
        if a.truthy() && a.get("aloft").truthy() && n(a.get("heat")) >= 60.0 && !is_str(u.get("doing"), "move") {
            order(obj! { "id" => u.get("id"), "land" => true });
        }
    }
    if me.get("rts").truthy() && cycles_left() < 2200.0 {
        return;
    }
    let pl = plan::for_units(me, units, world);
    let by_id = realm::index(world);
    let hunt = pl.get("hunt");
    let abroad = hunt.is_undef();
    let goal = if !abroad && vn(hunt) >= 0.0 {
        hunt
    } else if vn(pl.get("defend")) >= 0.0 {
        pl.get("defend")
    } else {
        pl.get("attack")
    };
    if goal.is_undef() || vn(goal) < 0.0 {
        return;
    }
    let g = vn(goal);
    let me_id = n(me.get("id"));
    let allies = me.get("allies");
    let q = by_id.get(g);
    let foes = if q.truthy() && q.get("units").truthy() {
        q.get("units").keys().filter(|f| {
            let f = js_key_num(f);
            f != me_id && !allies.has_num(f)
        }).count()
    } else {
        0
    };
    let owner = q.get("owner");
    let verb = if q.truthy() && n(owner) >= 0.0 && !same(owner, me.get("id")) && !allies.has_num(n(owner)) && foes == 0 { "raid" } else { "fight" };
    let rts = me.get("rts").truthy();
    for i in 0..units.len() {
        if rts && cycles_left() < 2200.0 {
            break;
        }
        let u = turn(units, i, world);
        // a wing on its way down (above) stays on it, and one down stays until it has cooled off
        let air = u.path("rts.air");
        if air.truthy() && (n(air.get("heat")) >= 60.0 || (n(air.get("heat")) > 0.0 && !air.get("aloft").truthy())) {
            continue;
        }
        let doing = u.get("doing");
        if is_str(doing, verb) && same_v(u.get("target"), goal) {
            continue;
        }
        if abroad && (is_str(doing, "fight") || is_str(doing, "raid") || is_str(doing, "move")) && n(u.get("target")) >= 0.0 {
            continue;
        }
        if let Some(mut o) = orders::to(u, verb, g, world, me, ()) {
            o.set("alert", 6);
            o.set("power", obj! { "drive" => 600, "armor" => 0, "weapon" => 420 });
            order(o);
        }
    }
}

// The exploits: the Society's spies. Each walks to a rival capital with a dear tech to copy (nullptr_ext::marks), stands
// hidden inside its ring and steals it while the bank has ◆ to spare; when our army besieges the capital it stands in,
// it sabotages instead, draining the cycles the capital parries with. Caught, it runs for the quietest neighbouring
// province and lies low for 40 s before it goes back to work. Inside a rival's walls a stealth-1 spy cloaks when the
// cloak lane can pay: cloaked, a standard body sees it only at arm's length.
fn ghost(units: View, world: View, me: View) {
    nullptr_ext::spies(units, world, me);
}

fn flee_step(foe: View) -> Value {
    obj! { "x" => -n(foe.get("dx")), "y" => -n(foe.get("dy")) }
}

// Kept intentionally linear: at most eight hostile contacts and no map search. It fires while stepping away, puts
// armor on only under pressure, and spends dash/repair/scan/shell only when a visible situation justifies the cost.
// Target selection, facing and support are faction code; the engine only prices and validates.
fn pierce_mode(u: View, target: Option<View>) -> f64 {
    let Some(t) = target else { return 0.0 };
    let tr = t.get("rts");
    if !u.get("rts").get("modules").has_str("pierce") || !tr.truthy() {
        return 0.0;
    }
    let mut armor = tr.get("armor").or(0.0);
    let g = tr.get("guard");
    if g.truthy() && n(t.get("d")) > 0.01 {
        armor = jmin(0.8, armor * (0.85 - 0.5 * (n(g.at(0)) * n(t.get("dx")) + n(g.at(1)) * n(t.get("dy"))) / n(t.get("d"))));
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
    let (mut x, mut y, mut k) = (0.0, 0.0, 0.0);
    for t in ns.iter() {
        if t.get("rival").truthy() && !t.get("air").truthy() && n(t.get("d")) > 0.0 {
            x += n(t.get("dx")) / n(t.get("d"));
            y += n(t.get("dy")) / n(t.get("d"));
            k += 1.0;
        }
    }
    if k != 0.0 && x.hypot(y) > k * 0.45 {
        obj! { "x" => x, "y" => y }
    } else {
        Value::Null
    }
}

fn repair_friend(u: View, ns: View, o: &mut Value, bank: f64, level: f64) -> f64 {
    let r = u.get("rts");
    if !r.get("modules").has_str("repair") || r.get("abilityIn").get("repair").or(0.0) > 0.0 {
        return 0.0;
    }
    // (id, hp, hpMax, d) of the one to mend: this unit, or the most hurt friend in reach
    let mut target = if n(u.get("hp")) < n(u.get("hpMax")) - 0.1 { Some((u.get("id"), n(u.get("hp")), n(u.get("hpMax")), 0.0)) } else { None };
    for t in ns.iter() {
        let (hp, hp_max) = (n(t.get("hp")), n(t.get("hpMax")));
        if (same(t.get("owner"), u.get("owner")) || t.get("ally").truthy())
            && n(t.get("d")) <= 3.0
            && hp < hp_max - 0.1
            && target.is_none_or(|(_, th, tm, _)| hp / hp_max < th / tm)
        {
            target = Some((t.get("id"), hp, hp_max, n(t.get("d"))));
        }
    }
    let Some((id, hp, hp_max, d)) = target else { return 0.0 };
    let amount = jmin(0.7, hp_max - hp);
    let cost = (level * 700.0 * (amount / 0.7) * (1.0 + 0.15 * d * d)).ceil();
    if cost > bank {
        return 0.0;
    }
    o.set("ability", "repair");
    o.set("repairTarget", id);
    o.set("amount", amount);
    cost
}

fn reflex(units: View, near: View) -> Vec<Value> {
    let len = units.len();
    let start = if len > 0 {
        let r0 = units.at(0).get("rts");
        let bank = if r0.truthy() { r0.get("bank").or(0.0) } else { 0.0 };
        (bank / 211.0).floor() % len as f64
    } else {
        0.0
    };
    // A short callback can still submit paid shots without planning movement or support.
    if cycles_left() < 4400.0 {
        for i in 0..len {
            if cycles_left() < 1250.0 { break; }
            let u = units.at(((start + i as f64) % len as f64) as usize);
            if is_str(u.get("type"), "ghost") { continue; }
            let ns = near.key(n(u.get("id")));
            let preferred = ns.iter().find(|t| reflex::can_hit(u, *t));
            let target = common::windup_target(u, ns, preferred);
            let r = u.get("rts");
            let cost = (120.0 + r.path("power.weapon").num_or(0.0)) * common::price_level(u);
            if let Some(t) = target.filter(|_| r.get("bank").num_or(0.0) >= cost
                && r.path("budget.weapon").num_or(cost) >= cost) {
                if cycles_left() < 600.0 { break; }
                order(obj! { "id" => u.get("id"), "hit" => t.get("id"), "pierce" => 0 });
            }
        }
        return Vec::new();
    }
    let mut turn_cost = 4000.0_f64;
    for i in 0..len {
        if cycles_left() < turn_cost + 400.0 {
            break;
        }
        let began = cycles_left();
        let u = units.at(((start + i as f64) % len as f64) as usize);
        let ns = near.key(n(u.get("id")));
        if cycles_left() < 3400.0 + 160.0 * ns.len() as f64 { break; }
        if !ns.truthy() {
            continue;
        }
        let ty = u.get("type");
        // an exploit has no weapon and nothing to gain from a fight: it steps away from the nearest rival instead,
        // out of the walk of whoever is about to see it
        if is_str(ty, "ghost") {
            let mut nearest: Option<View> = None;
            for e in ns.iter() {
                if e.get("rival").truthy() && !e.get("air").truthy() && nearest.is_none_or(|m| n(e.get("d")) < n(m.get("d"))) {
                    nearest = Some(e);
                }
            }
            if let Some(m) = nearest.filter(|m| n(m.get("d")) <= 2.5 && !is_str(u.get("doing"), "steal") && !is_str(u.get("doing"), "sabotage")) {
                let mut out = obj! { "id" => u.get("id"), "step" => flee_step(m) };
                common::steer_ground(u, ns, &mut out);
                order(out);
            }
            turn_cost = turn_cost.max(1.5 * (began - cycles_left()));
            continue;
        }
        let r = u.get("rts");
        let modules = r.get("modules");
        let flier = is_str(ty, "wing");
        let (mut best, mut score, mut melee, mut air): (Option<View>, f64, Option<View>, Option<View>) = (None, 99.0, None, None);
        for e in ns.iter() {
            if !e.get("rival").truthy() {
                continue;
            }
            let e_air = e.get("air").truthy();
            if e_air {
                air = Some(e);
            }
            let d = n(e.get("d"));
            if melee.is_none() && !e_air && e.get("range").is_num() && e.get("range").num() == 0.0 && d <= 2.2 {
                melee = Some(e);
            }
            if d > n(u.get("reach")) || (e_air && !u.get("air").truthy() && !(r.truthy() && modules.truthy() && modules.has_str("antiAir"))) {
                continue;
            }
            // the air wing goes for miners and archers first; a watchpoint shoots the spy it has just seen before
            // anything else (a spy's body is a point or two of hp)
            let s = n(e.get("hp")) / jmax(1.0, n(e.get("hpMax")))
                - (if e.get("range").truthy() { if flier { 0.5 } else { 0.12 } } else { 0.0 })
                - (if e_air { 0.08 } else { 0.0 })
                + d * 0.015
                - (if flier && (is_str(e.get("doing"), "gather") || is_str(e.get("type"), "miner")) { 0.6 } else { 0.0 })
                - (if is_str(ty, "sentry") && n(e.get("hpMax")) <= 2.0 { 0.5 } else { 0.0 });
            if s < score {
                score = s;
                best = Some(e);
            }
        }
        best=common::windup_target(u,ns,best);
        // Repair costs a meaningful 700 cycles (at price level 1) and restores only 0.7 HP, so spend it after a real
        // exchange, early enough that the line can disengage safely. Every price here is the rules' times the level.
        let budget = r.get("budget");
        let bank = if r.get("bank").is_undef() { 1_000_000_000.0 } else { (n(r.get("bank")) - 700.0).max(0.0) };
        let level = common::price_level(u);
        let lane = |name: &str, cost: f64| bank >= cost * level && (budget.get(name).is_undef() || n(budget.get(name)) >= cost * level);
        let hurt = n(u.get("hp")) < n(u.get("hpMax")) * 0.62;
        let pressured = melee.is_some() || hurt;
        let weapon = if best.is_some() && lane("weapon", 620.0) { 500.0 } else { 0.0 };
        let power = obj! {
            "drive" => if lane("drive", common::drive_tick(u, if pressured { 900.0 } else { 420.0 })) { if pressured { 900.0 } else { 420.0 } } else { 0.0 },
            "armor" => if pressured && modules.has_str("armor") && lane("armor", 170.0) { 850.0 } else { 0.0 },
            "weapon" => weapon,
        };
        let penetration = pierce_mode(u, best);
        let mut out = obj! { "id" => u.get("id"), "power" => power, "guard" => guard_facing(ns), "pierce" => penetration };
        if best.is_some() && !lane("weapon", ((120.0 + weapon) * (1.0 + 1.4 * penetration * penetration)).ceil()) {
            best = None;
        }
        // a cloaked exception in contact drops its cloak the second it has something to shoot: a shot shows it anyway,
        // and the cloak's cycles are better spent on the shot
        if best.is_some() && r.get("cloak").truthy() {
            out.set("cloak", false);
        }

        // Dodge telegraphed artillery before deciding to trade shots.
        let pos = if u.get("local").truthy() {u.get("local")}else{u.get("pos")};
        let (px, py) = (n(pos.get("x")), n(pos.get("y")));
        let hazard = r.get("hazards").iter().find(|h| n(h.get("impactIn")) < 1.3 && (px - n(h.get("x"))).hypot(py - n(h.get("y"))) < n(h.get("radius")) + 0.8);
        if let Some(h) = hazard {
            out.set("step", obj! { "x" => px - n(h.get("x")), "y" => py - n(h.get("y")) });
        } else if let Some(m) = melee {
            if u.get("range").truthy() || is_str(ty, "repairer") || hurt {
                out.set("step", flee_step(m));
            }
        }
        if let Some(b) = best {
            out.set("hit", b.get("id")); // movement and fire intentionally share one order
        }
        common::close_melee(u,ns,&mut out);
        common::space_ranged(u,ns,&mut out);
        let closing_dash = out.get("ability").str() == "dash";
        out.set("ability", Value::Undef);

        let pending_shot = if best.is_some() { level * ((120.0 + weapon) * (1.0 + 1.4 * penetration * penetration)).ceil() } else { 0.0 };
        let drive_cost = if out.get("step").truthy() { level * common::drive_tick(u, vn(out.get("power").get("drive"))) } else { 0.0 };
        let armor_cost = level * vn(out.get("power").get("armor")) * 0.2;
        let support_bank = (bank - pending_shot - drive_cost - armor_cost).max(0.0);
        let can_support = |cost: f64| lane("abilities", cost) && support_bank >= cost * level;
        let cd = r.get("abilityIn");
        let abilities = budget.get("abilities");
        let mended = repair_friend(u, ns, &mut out, jmin(support_bank, if abilities.is_nullish() { support_bank } else { n(abilities) }), level);
        if mended != 0.0 && !mended.is_nan() {
            // paid support or self-repair
        } else if (melee.is_some() || closing_dash) && modules.has_str("dash") && r.path("abilityIn.dash").num_or(0.0) <= 0.0 && out.get("step").truthy() && can_support(900.0) {
            out.set("ability", "dash");
        } else if (air.is_some() || (is_str(ty, "sentry") && best.is_some())) && modules.has_str("scan") && cd.get("scan").or(0.0) <= 0.0 && can_support(700.0) {
            out.set("ability", "scan");
        } else if let Some(b) = best.filter(|_| is_str(ty, "gun") && r.get("deployed").truthy() && cd.get("shell").or(0.0) <= 0.0 && pos.truthy()) {
            let (bx, by) = (n(b.get("dx")), n(b.get("dy")));
            let cluster = ns.iter().filter(|e| e.get("rival").truthy() && (n(e.get("dx")) - bx).hypot(n(e.get("dy")) - by) <= 2.0).count();
            let radius = if cluster >= 3 { 3.0 } else if cluster >= 2 { 2.0 } else { 1.0 };
            let shell_cost = 1000.0 * ((radius / 2.0) * (radius / 2.0)) * (1.0 + jmax(n(b.get("d")) - 5.0, 0.0) * 0.12);
            let safe = ns.iter().all(|e| e.get("rival").truthy() || (n(e.get("dx")) - bx).hypot(n(e.get("dy")) - by) > radius + 0.5);
            if safe && n(b.get("d")) > radius + 0.7 && can_support(shell_cost) {
                out.set("ability", "shell");
                out.set("radius", radius);
                out.set("aim", obj! { "x" => px + bx, "y" => py + by });
            }
        }
        common::steer_ground(u,ns,&mut out);
        if best.is_some() || out.get("step").truthy() || out.get("ability").truthy() || out.get("cloak").truthy() {
            if cycles_left() < 700.0 { break; }
            order(out);
        }
        turn_cost = turn_cost.max(1.5 * (began - cycles_left()));
    }
    Vec::new()
}

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

// The Society's own: spies, marks, the cloak lane and the memo.
mod nullptr_ext {
    use super::*;

    /// Console lines about each spy every 20 s, while the program is being tuned.
    pub const DEBUG: bool = false;
    /// Unit RAM page 1 of a spy: `{ fled }`, the second it last ran from being seen.
    const NOTES: u32 = 1;
    /// How long a spy lies low after it was seen: the sighting lasts 30 s, and a little longer costs nothing.
    const LOW: f64 = 40.0;
    /// The cheapest tech worth copying: the second tier's 60 solves and up. A first-tier tech's small puzzles are cheaper
    /// to solve than to steal at 25 ◆ a second; a third-tier one (1,000 solves) is worth a long walk and a lower floor.
    const DEAR: f64 = 40.0;

    /// What think knows of its exploits this second.
    pub struct Census {
        /// how many exploits stand
        pub count: usize,
        /// the techs hidden spies are copying now
        pub stealing: Vec<&'static str>,
        /// how many are sabotaging now
        pub sabotaging: usize,
        /// how many stand hidden in a rival capital's province with no errand under way (waiting for ◆)
        pub waiting: usize,
        /// (spy's name, tech, province id) of the first one copying, for the memo
        pub first: Option<(&'static str, &'static str, f64)>,
        /// (spy's name, province id) of the first one sabotaging
        pub saboteur: Option<(&'static str, f64)>,
        /// how many are seen by a rival now
        pub seen: usize,
    }

    pub fn census(world: View, me: View) -> Census {
        let me_id = me.get("id");
        let mut c = Census { count: 0, stealing: Vec::new(), sabotaging: 0, waiting: 0, first: None, saboteur: None, seen: 0 };
        // the pass over the units in sight costs about 60 cycles each: short of that (and 24,000 for the rest of think),
        // the counts come from me.units alone, with no names for the memo
        let units = me.get("units");
        c.count = units.get("byType").get("ghost").or(0.0) as usize;
        if cycles_left() < 60.0 * world.get("units").len() as f64 + 24000.0 {
            c.sabotaging = units.get("doing").get("sabotage").or(0.0) as usize;
            let stealing = units.get("doing").get("steal").or(0.0) as usize;
            if stealing > 0 {
                c.stealing.push("?");
            }
            return c;
        }
        c.count = 0;
        let capitals: Vec<f64> = world.get("factions").iter().filter(|f| realm::is_rival(world, me, n(f.get("id")))).map(|f| n(f.get("capital"))).collect();
        for u in world.get("units").iter() {
            if !same(u.get("owner"), me_id) || !is_str(u.get("type"), "ghost") {
                continue;
            }
            c.count += 1;
            if u.get("seenBy").len() > 0 {
                c.seen += 1;
            }
            let steal = u.get("steal");
            if is_str(u.get("doing"), "steal") && steal.is_str() && u.get("hidden").truthy() {
                c.stealing.push(steal.str());
                if c.first.is_none() {
                    c.first = Some((u.get("name").str(), steal.str(), n(u.get("at"))));
                }
            } else if is_str(u.get("doing"), "sabotage") {
                c.sabotaging += 1;
                if c.saboteur.is_none() {
                    c.saboteur = Some((u.get("name").str(), n(u.get("at"))));
                }
            } else if u.get("hidden").truthy() && capitals.contains(&n(u.get("at"))) {
                c.waiting += 1;
            }
        }
        c
    }

    /// The memo spectators read: what the Society is up to, in one line.
    pub fn memo(world: View, me: View, c: &Census, stolen: f64, sab: f64, skirms: f64, screens: f64) -> String {
        let by_id = realm::index(world);
        let place = |id: f64| {
            let p = by_id.get(id);
            if p.truthy() { p.get("name").str() } else { "an unknown address" }
        };
        let army = format!("{} exceptions, {} nulls", js_num(skirms), js_num(screens));
        if c.count == 0 {
            return format!("Address Zero: nothing to see here. {army}.");
        }
        let stolen_s = match (stolen > 0.0, sab > 0.0) {
            (true, true) => format!(" {} techs stolen, {} s of sabotage so far.", js_num(stolen), js_num(sab)),
            (true, false) => format!(" {} techs stolen so far.", js_num(stolen)),
            (false, true) => format!(" {} s of sabotage so far.", js_num(sab)),
            (false, false) => String::new(),
        };
        if let Some((name, tech, at)) = c.first {
            let t = world.get("techs").get(tech);
            let tech_name = if t.truthy() && t.get("name").is_str() { t.get("name").str() } else { tech };
            let need = t.get("need").or(1.0);
            let got = me.path("research.progress").get(tech).or(0.0);
            let pct = round(jmin(100.0, got / jmax(1.0, need) * 100.0));
            return format!("Address Zero: {name} is inside {} copying {tech_name} ({}% read).{stolen_s} {army}.", place(at), js_num(pct));
        }
        if let Some((name, at)) = c.saboteur {
            return format!("Address Zero: {name} is cutting the power at {}.{stolen_s} {army}.", place(at));
        }
        format!("Address Zero: {} exploits abroad, {} seen.{stolen_s} {army}.", c.count, c.seen)
    }

    /// Deals that ask ◆ wait while the spies' fund (`held`) and a unit's price would be left short: their ids move
    /// from `accept` to `reject`.
    pub fn keep_fund(orders: &mut Value, me: View, held: f64) {
        if held <= 0.0 {
            return;
        }
        let spare = n(me.get("bank")) - held - n(me.get("costs").get("unit"));
        let mut costly: Vec<f64> = Vec::new();
        for m in me.get("inbox").iter() {
            let coin = m.get("ask").get("coin").or(0.0) + m.get("ask").get("coinPerSecond").or(0.0) * m.get("ask").get("seconds").or(1.0);
            if coin > 0.0 && coin > spare {
                costly.push(n(m.get("id")));
            }
        }
        if costly.is_empty() {
            return;
        }
        let accepted: Vec<Value> = orders.get("accept").items().to_vec();
        let (keep, drop): (Vec<Value>, Vec<Value>) = accepted.into_iter().partition(|a| !costly.contains(&vn(a)));
        orders.set("accept", keep);
        for d in drop {
            orders.entry("reject").push(d);
        }
    }

    /// How many units the cloak lane can pay for this second (450 cycles a second each, times the price level).
    pub fn cloaks(me: View) -> f64 {
        let avail = me.path("rts.cpu.available.cloak");
        if !avail.is_num() {
            return 9.0;
        }
        (avail.num() / (450.0 * common::price_level(me))).floor()
    }

    /// The tech to copy from faction `o`: the dearest it has that we don't, whose requirements we have, worth at least
    /// [`DEAR`] solves. `(tech, need)`.
    pub fn loot(world: View, me: View, o: f64) -> Option<(&'static str, f64)> {
        let (all, mine) = (world.get("techs"), me.get("techs"));
        let mut best: Option<(&'static str, f64)> = None;
        for t in world.get("factions").key(o).get("techs").iter() {
            let id = t.str();
            let d = all.get(id);
            if mine.get(id).truthy() || !d.truthy() || !d.get("requires").iter().all(|q| mine.get(q.str()).truthy()) {
                continue;
            }
            let need = d.get("need").or(0.0);
            if need >= DEAR && best.is_none_or(|(_, b)| need > b) {
                best = Some((id, need));
            }
        }
        best
    }

    /// The capitals worth a visit, best first: `(capital, owner, tech, need)` for every living rival with a tech to
    /// copy. The third tier (1,000 solves) is worth a long walk; the second a shorter one (a province spacing a point).
    pub fn marks_of(world: View, me: View) -> Vec<(f64, f64, &'static str, f64)> {
        let by_id = realm::index(world);
        let mut out: Vec<(f64, f64, &'static str, f64, f64)> = Vec::new();
        for f in world.get("factions").iter() {
            let (id, cap) = (n(f.get("id")), n(f.get("capital")));
            if !realm::is_rival(world, me, id) || cap < 0.0 || !by_id.get(cap).truthy() {
                continue;
            }
            let Some((tech, need)) = loot(world, me, id) else { continue };
            let worth = if need >= 1000.0 { 30.0 } else { 10.0 };
            out.push((cap, id, tech, need, worth - far(world, me, by_id.get(cap))));
        }
        out.sort_by(|a, b| b.4.total_cmp(&a.4));
        out.into_iter().map(|(c, o, t, nd, _)| (c, o, t, nd)).collect()
    }

    /// How far province `q` lies from our capital, in province spacings on the map (rival provinces carry no `hops`).
    fn far(world: View, me: View, q: View) -> f64 {
        let cap = realm::index(world).at(me.get("capital"));
        if !cap.truthy() || !q.truthy() {
            return 12.0;
        }
        (n(q.get("x")) - n(cap.get("x"))).hypot(n(q.get("y")) - n(cap.get("y")))
    }

    /// Every living rival's capital, the nearest first: where the spies watch when there's nothing to copy.
    pub fn posts_of(world: View, me: View) -> Vec<f64> {
        let by_id = realm::index(world);
        let mut out: Vec<(f64, f64)> = Vec::new();
        for f in world.get("factions").iter() {
            let (id, cap) = (n(f.get("id")), n(f.get("capital")));
            if realm::is_rival(world, me, id) && cap >= 0.0 && by_id.get(cap).truthy() {
                out.push((cap, far(world, me, by_id.get(cap))));
            }
        }
        out.sort_by(|a, b| a.1.total_cmp(&b.1));
        out.into_iter().map(|(c, _)| c).collect()
    }

    /// A spot inside a capital's ring to stand hidden: a forest cell when there is one near the town (a cell of cover),
    /// else a cell and a half out.
    fn hide_spot(p: View) -> Point {
        const SPOTS: [(f64, f64); 8] = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (1.5, 0.0), (-1.5, 0.0), (0.0, 1.5), (0.0, -1.5)];
        for (x, y) in SPOTS {
            if realm::ground(p, x, y).is_some_and(|g| g.kind == "forest") {
                return Point::new(x, y);
            }
        }
        Point::new(1.2, 0.8)
    }

    /// Hostile units standing in province `q`, as its counts by faction say.
    fn hostile(q: View, me: View) -> f64 {
        let me_id = n(me.get("id"));
        let units = q.get("units");
        if !units.truthy() {
            return 0.0;
        }
        units.keys().filter(|k| {
            let f = js_key_num(k);
            f != me_id && !realm::is_ally(me, f)
        }).map(|k| units.get(k).or(0.0)).sum()
    }

    /// An order for `u` to `verb` at point `at` of province `target`, routed through anyone's land (the search stops
    /// short of `keep` cycles and gives nothing rather than run the call out). `None` when it's on it already.
    fn go(u: View, verb: &str, target: f64, world: View, me: View, at: Point, keep: f64) -> Option<Value> {
        let spot = u.get("spot");
        let there = spot.truthy() && (n(spot.get("x")) - at.x).hypot(n(spot.get("y")) - at.y) < 0.5;
        if is_str(u.get("doing"), verb) && same_v(u.get("target"), &Value::Num(target)) && there {
            return None;
        }
        let from = if n(u.get("to")) >= 0.0 { n(u.get("to")) } else { n(u.get("at")) };
        let mut o = obj! { "id" => u.get("id"), "do" => verb, "target" => target, "at" => Value::from(at), "alert" => 0 };
        if from != target && !realm::reach(realm::index(world).get(from), me).contains(&target) {
            let route = path::route(world, me, from, target, &path::Opts { keep: Some(keep), ..path::Opts::default() })?;
            let via: Vec<Value> = route.iter().skip(1).map(|&p| Value::Num(p)).collect();
            if via.is_empty() {
                return None;
            }
            o.set("via", via);
        }
        Some(o)
    }

    /// The exploits' turn (see `ghost`).
    pub fn spies(units: View, world: View, me: View) {
        let rts = me.get("rts").truthy();
        // Leave room for a spy's terrain checks, RAM notes and final order.
        if rts && cycles_left() < 3400.0 {
            return;
        }
        let by_id = realm::index(world);
        let me_id = n(me.get("id"));
        let now = world.get("seconds").or(0.0).floor();
        let bank = n(me.get("bank"));
        // a unit's price in ◆: what each errand keeps in the bank for the army depends on it
        let unit = n(me.get("costs").get("unit"));
        let mut cloaks = cloaks(me);
        // the marks and posts cost a pass over every faction's techs: worked out once a call, and only with the cycles
        // for it (short of them, a spy keeps the mark in its notes and its standing orders)
        let (mut marks, mut posts): (Option<Vec<(f64, f64, &'static str, f64)>>, Option<Vec<f64>>) = (None, None);
        let lead = common::dominant(world).map_or(-1.0, |l| n(l.get("id")));
        for turn in 0..units.len() {
            let i = (turn + now as usize) % units.len();
            if rts && cycles_left() < 3000.0 {
                break;
            }
            if marks.is_none() && (!rts || cycles_left() > 6000.0) {
                marks = Some(marks_of(world, me));
                posts = Some(posts_of(world, me));
            }
            if rts && cycles_left() < 3000.0 { break; }
            let marks: &[(f64, f64, &'static str, f64)] = marks.as_deref().unwrap_or(&[]);
            let posts: &[f64] = posts.as_deref().unwrap_or(&[]);
            // the spies are few: each keeps its place in the list, so the i-th spy keeps the i-th mark
            let u = units.at(i);
            let here = by_id.at(u.get("at"));
            let owner = n(here.get("owner"));
            let abroad = here.truthy() && owner >= 0.0 && owner != me_id && !realm::is_ally(me, owner);
            let seen = u.get("seenBy").len() > 0;
            let notes = mem::get(u.get("ramUnit"), NOTES);
            let fled = vor0(&notes.get("fled"));
            let kept = notes.get("mark").num_or(-1.0);
            let note = |fled: f64, mark: f64| mem::put(NOTES, &obj! { "fled" => fled, "mark" => mark }, "local", Some("unit"), 1).unwrap_or_default();
            let cloaked = u.path("rts.cloak").truthy();
            let can_cloak = u.path("rts.modules").has_str("cloak") && n(u.get("build").get("stealth")) < 2.0;
            if DEBUG && now % 20.0 == 0.0 {
                log(&format!(
                    "spy {} at {} ({}) doing {} target {} steal {} hidden {} seenBy {} to {} fled {} bank {} marks {} cloak {}/{} lane {}",
                    u.get("name").str(), n(u.get("at")), here.get("name").str(), u.get("doing").str(), n(u.get("target")), u.get("steal").str(),
                    u.get("hidden").truthy(), u.get("seenBy").len(), n(u.get("to")), fled, bank, marks.len(), cloaked, u.path("rts.cloaked").truthy(), cloaks
                ));
            }
            // seen on a rival's land: run for home, by the neighbouring province nearest our land (our own first, free or
            // allied land next, the quietest on a tie), and lie low
            if seen && abroad {
                let (mut best, mut bs) = (-1.0, f64::INFINITY);
                for id in realm::reach(here, me) {
                    if rts && cycles_left() < 1200.0 { break; }
                    let q = by_id.get(id);
                    if !q.truthy() {
                        continue;
                    }
                    let qo = n(q.get("owner"));
                    let s = far(world, me, q) * 10.0 + hostile(q, me) + if qo == me_id { -100.0 } else if qo < 0.0 || realm::is_ally(me, qo) { -50.0 } else { 0.0 };
                    if s < bs {
                        (best, bs) = (id, s);
                    }
                }
                let moving = is_str(u.get("doing"), "move") && same_v(u.get("target"), &Value::Num(best));
                if best >= 0.0 && !moving {
                    order(obj! { "id" => u.get("id"), "do" => "move", "target" => best, "cloak" => false, "alert" => 0, "ram" => note(now, kept) });
                } else if cloaked {
                    order(obj! { "id" => u.get("id"), "cloak" => false });
                }
                continue;
            }
            // lying low: finish the move, then stand where it is
            if now - fled < LOW {
                if cloaked {
                    order(obj! { "id" => u.get("id"), "cloak" => false });
                }
                continue;
            }
            // inside a rival capital: sabotage under our siege or while its army marches on our land (its plan says so:
            // a hidden spy reads the RAM where it stands), else copy its dearest tech, else drain the leader
            let capital_of = abroad && realm::is_rival(world, me, owner) && same_v(world.get("factions").key(owner).get("capital"), &Value::Num(n(here.get("id"))));
            let mut o: Option<Value> = None;
            let mut stay = false;
            if capital_of {
                let cap = n(here.get("id"));
                let ours_in = here.get("units").key(me_id).or(0.0) > 0.0 || same_v(here.get("siege").get("by"), &Value::Num(me_id));
                let theirs = spy::read(me, u);
                let marching = theirs.get("attack").num_or(-1.0) >= 0.0 && theirs.get("attack").num() == n(me.get("capital"));
                let spot = hide_spot(here);
                let loot = loot(world, me, owner);
                // what each errand may spend the bank down to: the decisive moments first
                let floor = if ours_in || marching {
                    unit + 40.0
                } else if loot.is_some_and(|(_, need)| need >= 300.0) {
                    unit + 85.0
                } else if loot.is_some() {
                    unit + 160.0
                } else {
                    2.0 * unit + 400.0
                };
                stay = true;
                if bank < floor {
                    // ◆ short: wait hidden where it stands
                    if !is_str(u.get("doing"), "idle") && n(u.get("to")) < 0.0 {
                        o = Some(obj! { "id" => u.get("id"), "do" => "idle" });
                    }
                } else if ours_in || marching {
                    o = go(u, "sabotage", cap, world, me, spot, 2200.0);
                } else if let Some((tech, _)) = loot {
                    o = go(u, "steal", cap, world, me, spot, 2200.0);
                    if let Some(x) = &mut o {
                        x.set("tech", tech);
                    }
                } else if owner == lead {
                    o = go(u, "sabotage", cap, world, me, spot, 2200.0);
                } else {
                    stay = false;
                }
            }
            if !stay {
                // its mark: the one it keeps in its notes while that capital still has a tech to copy, else the i-th of
                // the list, best first; with nothing to copy anywhere, a post: a rival capital watched from six cells
                // out, where fewer eyes are, reading its plan there (the i-th spy the i-th nearest)
                let mut mark = -1.0;
                if !marks.is_empty() {
                    let m = marks.iter().find(|m| m.0 == kept).or_else(|| marks.get(i % marks.len()));
                    if let Some(&(cap, _, tech, _)) = m {
                        mark = cap;
                        o = go(u, "steal", cap, world, me, hide_spot(by_id.get(cap)), 2200.0);
                        if let Some(x) = &mut o {
                            x.set("tech", tech);
                        }
                    }
                } else if !posts.is_empty() {
                    let cap = if posts.contains(&kept) { kept } else { posts[i % posts.len()] };
                    mark = cap;
                    o = go(u, "move", cap, world, me, Point::new(6.0, 0.0), 2200.0);
                }
                if mark >= 0.0 && mark != kept {
                    let x = o.get_or_insert_with(|| obj! { "id" => u.get("id") });
                    x.set("ram", note(fled, mark));
                }
            }
            // the cloak: on in a rival's land while the lane pays for it (cloaked, a standard body sees it only at arm's
            // length), off at home and in free land
            let want = abroad && can_cloak && cloaks >= 1.0;
            if want {
                cloaks -= 1.0;
            }
            if want != cloaked {
                match &mut o {
                    Some(x) => x.set("cloak", want),
                    None => o = Some(obj! { "id" => u.get("id"), "cloak" => want }),
                }
            }
            if let Some(o) = o {
                if rts && cycles_left() < 700.0 { break; }
                order(o);
            }
        }
    }
}

// JavaScript's rules, for the places this program needs them.

/// `Number(x)` for a value from the game: null is 0, anything that isn't a number NaN.
fn n(v: View) -> f64 {
    if v.is_null() {
        0.0
    } else {
        v.num()
    }
}

/// `Number(x)` for a value built here or read from RAM.
fn vn(v: &Value) -> f64 {
    match v {
        Value::Null => 0.0,
        Value::Bool(b) => f64::from(u8::from(*b)),
        v => v.num(),
    }
}

/// `x || 0` for a number built here or read from RAM.
fn vor0(v: &Value) -> f64 {
    if v.truthy() {
        vn(v)
    } else {
        0.0
    }
}

/// `x || 0` for a number in a plan.
fn or0(v: &Value) -> f64 {
    vor0(v)
}

/// `(me.rts && me.rts.cpu)`: what the CPU RTS says of your CPU, or nothing.
fn rts_cpu(me: View) -> View {
    let rts = me.get("rts");
    if rts.truthy() {
        rts.get("cpu")
    } else {
        rts
    }
}

/// `a === b` for two values from the game (numbers, strings, or nothing).
fn same(a: View, b: View) -> bool {
    if a.is_num() || b.is_num() {
        a.is_num() && b.is_num() && a.num() == b.num()
    } else if a.is_str() || b.is_str() {
        a.is_str() && b.is_str() && a.str() == b.str()
    } else {
        (a.is_undef() && b.is_undef()) || (a.is_null() && b.is_null()) || a.same(&b)
    }
}

/// `a === b` for a value from the game and a number from a plan.
fn same_v(a: View, b: &Value) -> bool {
    match b {
        Value::Num(x) => a.is_num() && a.num() == *x,
        Value::Undef => a.is_undef(),
        Value::Null => a.is_null(),
        _ => false,
    }
}

fn is_str(v: View, s: &str) -> bool {
    v.is_str() && v.str() == s
}

/// `+k` for an object key.
fn js_key_num(k: &str) -> f64 {
    let t = k.trim();
    if t.is_empty() {
        0.0
    } else {
        t.parse().unwrap_or(f64::NAN)
    }
}

/// `Math.round`: halves go up.
fn round(x: f64) -> f64 {
    if x.is_finite() {
        (x + 0.5).floor()
    } else {
        x
    }
}

/// `Math.max(a, b)`: NaN when either is.
fn jmax(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}

/// `Math.min(a, b)`: NaN when either is.
fn jmin(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.min(b)
    }
}

/// A number as JavaScript writes it in text.
fn js_num(x: f64) -> String {
    if x.is_nan() {
        "NaN".into()
    } else if x.is_infinite() {
        (if x > 0.0 { "Infinity" } else { "-Infinity" }).into()
    } else if x.fract() == 0.0 && x.abs() < 1e21 {
        format!("{}", x as i128)
    } else {
        format!("{x}")
    }
}
