// Arjuna's Halting Problem: the archer who takes to the sky.
//
// On the ground it fights as the mobile houses do: archers dash into range and fire while they withdraw, behind a
// screen of war elephants. Its research runs first for Flight (sea and mountain crossings, then the third tier), and
// it buys the techs other factions already hold, so research goes to the sky. Then the sky is its game: think picks
// one strike a second (invaders in its land first, then besiegers, the land its army attacks, rival miners and
// archers, or an undefended town to raid), weighing each against the anti-air seen there, and the vimanas fly
// sorties: they launch together once cool, strike, turn back before their burn climbs, land on the nearest land of
// theirs and cool off. Air weapons follow, for vimanas that shoot. The memo says where the vimanas fly.
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
const RUNNER: Build = &[("hp", 2.0), ("speed", 4.0), ("strength", 2.0), ("carry", 0.0), ("vision", 2.0)];
const SKIRM: Build = &[("hp", 2.0), ("speed", 3.0), ("strength", 2.0), ("carry", 0.0), ("vision", 1.0), ("range", 1.0)];
const SENTRY: Build = &[("hp", 2.0), ("speed", 2.0), ("strength", 1.0), ("carry", 0.0), ("vision", 3.0), ("range", 1.0)];
const GUN: Build = &[("hp", 2.0), ("speed", 1.0), ("strength", 2.0), ("carry", 0.0), ("vision", 1.0), ("range", 2.0)];
const GHOST: Build = &[("hp", 1.0), ("speed", 3.0), ("strength", 1.0), ("carry", 0.0), ("vision", 3.0), ("stealth", 1.0)];
// The vimana, once Flight allows wings: a striking flier, and a shooting one once Air weapons allows range on wings.
const WING: Build = &[("hp", 2.0), ("speed", 3.0), ("strength", 2.0), ("carry", 0.0), ("vision", 0.0), ("wings", 1.0)];
const WING_SHOT: Build = &[("hp", 2.0), ("speed", 1.0), ("strength", 2.0), ("carry", 0.0), ("vision", 0.0), ("wings", 1.0), ("range", 1.0)];

// Establish mining and a mobile archer screen, then pursue the Flight path.
// Ground upgrades follow Flight, Artillery and Air weapons.
const TECHS: [&str; 23] = [
    "extraction", "dash", "ranged", "antiAir", "flight", "artillery", "airWeapons", "armor", "capture", "storage",
    "computeBoost", "turbines", "scan", "repair", "pierce", "influence", "cloak", "stealth", "seaTraversal",
    "mountainTraversal", "transmission", "cooling", "gate",
];

// In a dim grid, buy turbines after the basic archer and anti-air defenses, before Flight. Once power is stable,
// Arjuna keeps its usual rush to the sky.
const POWER_SHORT_TECHS: [&str; 23] = [
    "extraction", "dash", "ranged", "antiAir", "turbines", "flight", "artillery", "airWeapons", "armor",
    "capture", "storage", "computeBoost", "scan", "repair", "pierce", "influence", "cloak", "stealth",
    "seaTraversal", "mountainTraversal", "transmission", "cooling", "gate",
];

fn doctrine(me: View, emu: bool) -> &'static [&'static str] {
    if emu {
        &EMU_TECHS
    } else if me.get("energy").get("powered").num_or(1.0) < 0.92 && !me.get("techs").get("turbines").truthy() {
        &POWER_SHORT_TECHS
    } else {
        &TECHS
    }
}

// Emu High Command shares the campaign controller but invests in fast ground raids.
const EMU_TECHS: [&str;19] = ["extraction","dash","ranged","armor","capture","repair","artillery","antiAir","pierce","seaTraversal","mountainTraversal","scan","influence","computeBoost","cloak","stealth","flight","airWeapons","gate"];
const EMU_SCREEN: Build = &[("hp",3.0),("speed",4.0),("strength",3.0),("carry",0.0),("vision",0.0)];
const EMU_SKIRM: Build = &[("hp",2.0),("speed",3.0),("strength",2.0),("carry",0.0),("vision",1.0),("range",1.0)];

fn build(b: Build) -> Value {
    let mut o = Obj::new();
    for (k, v) in b {
        o.set(k, *v);
    }
    Value::Obj(o)
}

// Bribes, passage and the secrets we buy run on timers with a random gap, so no two worlds repeat: each fires `base`
// seconds after the last, give or take 60%. The next second of each lives in capital RAM pages 224-225, written only
// when one fires.
const CLOCK: u32 = 224;
const RECOVERY_PAGE: u32 = 244;

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
        "ghost" => {
            add("cloak");
            add("dash");
        }
        "repair" => {
            add("repair");
            add("dash");
        }
        // a vimana carries no module: each would add to its fabrication and its upkeep, and it needs neither dash nor
        // armor to strike what can't shoot back
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
    orders.entry("spawn").push(obj! { "type" => ty, "build" => build(b), "modules" => modules });
    true
}

// Each type's base call, in the order the types grow their calls past it (common::calls). Vimanas come right after
// miners: a flier whose call runs short can't be ordered down, and its burn climbs the longer it stays up.
const TYPES: [(&str, f64, f64); 10] = [
    ("miner", 5000.0, 1.0),
    ("wing", 8000.0, 2.0),
    ("screen", 6000.0, 1.0),
    ("runner", 6000.0, 1.0),
    ("skirm", 6000.0, 1.0),
    ("lancer", 6000.0, 2.0),
    ("sentry", 6000.0, 1.0),
    ("gun", 6000.0, 1.0),
    ("repairer", 5000.0, 1.0),
    ("ghost", 4200.0, 1.0),
];

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
    let income = rts_cpu(me).get("income").or(0.0);
    let pick = |a: f64, b: f64| if invaded { a } else { b };
    // Research is the road to the sky: three tenths of income until Flight (a bigger share starves the army that earns
    // it, and Flight comes later, not sooner), then a little over a fifth, since the vimanas cost the rest. Invaded,
    // or down to three provinces, the army comes first.
    let techs = me.get("techs");
    let short = invaded || me.get("provinces").num_or(9.0) <= 3.0;
    let research = match (techs.get("flight").truthy(), short) {
        (false, false) => 0.3,
        (false, true) => 0.25,
        (true, false) => 0.22,
        (true, true) => 0.2,
    };
    // vimanas aloft burn on top of their upkeep, from the same lane
    let wings = me.get("units").get("byType").get("wing").or(0.0);
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
            "upkeep" => if wings > 0.0 { 0.5 } else { 0.38 },
            "drive" => pick(0.18, 0.11),
            "armor" => pick(0.20, 0.12),
            "cloak" => 0.05,
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
        // a vimana its side can't pay glides and crashes, so vimanas are paid right after the miners and the line
        "upkeepOrder" => vec!["miner", "screen", "skirm", "wing", "repairer", "sentry", "gun", "runner", "lancer", "ghost"],
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

fn think(world: View, me: View) -> Value {
    if cycles_left()<800.0 {return obj!{};}
    if me.path("rts.cpu").truthy() && cycles_left() < 3000.0 {
        return emergency_recovery_policy();
    }
    let mut orders = if me.path("rts.cpu").truthy() && cycles_left() < 18000.0 {
        common::govern(world, me, 0.4, || recovery_policy(me))
    } else {
        common::govern(world, me, 0.4, || strategy(world, me))
    };
    flight_progress(&mut orders, world, me);
    orders
}

// A healthy cargo crew can fund the Flight path while the full army planner waits for savings.
// Keep emergency recovery closed when upkeep is unpaid or the bank cannot cover two seconds of income.
fn flight_progress(orders: &mut Value, world: View, me: View) {
    if cycles_left() < 4000.0 || me.get("seat").num_or(-1.0) == 8.0
        || !world.get("logistics").truthy()
        || me.path("units.total").num_or(0.0) < 4.0 || me.path("units.stalled").num_or(0.0) > 0.0 { return; }
    let income = me.path("rts.cpu.income").num_or(0.0);
    if income <= 0.0 || me.path("rts.cpu.bank").num_or(0.0) < income * 2.0
        || common::unit_upkeep(me) > income * 0.5 { return; }
    // A full strategy already funds research. Recovery alone needs this smaller lane reopened.
    let recovering = mem::get(me.get("ramCapital"), RECOVERY_PAGE).get("r").truthy()
        || orders.get("memo").str() == "Recovering production"
        || orders.get("compute").get("cpu").get("limits").get("research").num_or(0.0) <= 0.0;
    // The engine retains the active topic and its solved puzzle when no replacement is sent. Recovery can zero its
    // lane, though, so restore the healthy sky-path allocation without changing that topic.
    if me.path("research.tech").truthy() {
        if recovering {
            let cpu = orders.entry("compute").entry("cpu");
            cpu.entry("limits").set("research", 0.12);
            cpu.entry("reserve").set("research", 0.02);
        }
        return;
    }
    for tech in ["extraction", "dash", "ranged", "antiAir", "seaTraversal", "mountainTraversal", "flight", "artillery", "airWeapons"] {
        let spec = world.get("techs").get(tech);
        if me.get("techs").get(tech).truthy() || !spec.truthy() { continue; }
        if !spec.get("requires").iter().all(|t| me.get("techs").get(t.str()).truthy()) { return; }
        orders.set("research", tech);
        if recovering {
            let cpu = orders.entry("compute").entry("cpu");
            cpu.entry("limits").set("research", 0.12);
            cpu.entry("reserve").set("research", 0.02);
        }
        return;
    }
}

// At the bottom of a short think slice, update only the CPU budget needed for the next one. Leave type schedules and
// standing orders alone; the larger recovery policy needs more cycles to build.
fn emergency_recovery_policy() -> Value {
    obj! { "compute" => obj! { "cpu" => obj! {
        "limits" => obj! {
            "think" => 24000, "control" => 0.5, "upkeep" => 0.65,
            "research" => 0, "influence" => 0, "fabrication" => 0, "visuals" => 0,
            "reflex" => 0.15, "duel" => 0.08, "challenge" => 0.02,
            "drive" => 0.08, "armor" => 0.05, "weapon" => 0.08, "abilities" => 0, "cloak" => 0, "gate" => 0
        },
        "reserve" => obj! {}, "upkeepOrder" => vec!["miner"]
    } } }
}

// When the full strategy will not fit, skip its world scan and make a bounded production-first plan directly.
fn recovery_policy(me: View) -> Value {
    let income = me.path("rts.cpu.income").num_or(20000.0).max(1.0);
    let mut types = obj! {};
    for t in me.get("types").keys() {
        types.set(t, obj! { "share" => 0, "every" => 8 });
    }
    types.set("miner", obj! { "cycles" => jmax(4000.0, income * 0.4), "every" => 1 });
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
    let mut recovery = mem::get(me.get("ramCapital"), RECOVERY_PAGE);
    if !matches!(recovery, Value::Obj(_)) { recovery = obj! {}; }
    recovery.set("r", true);
    obj! { "compute" => obj! { "cpu" => cpu, "types" => types },
        "ram" => mem::put(RECOVERY_PAGE, &recovery, "local", None, 1).unwrap_or_default(),
        "propaganda" => Value::arr(), "preach" => Value::arr(), "rent" => 0 }
}

fn strategy(world: View, me: View) -> Value {
    let emu=me.get("seat").num_or(-1.0)==8.0;
    let doctrine = doctrine(me, emu);
    let rts = me.get("rts");
    if rts.truthy() && cycles_left() < 3000.0 {
        return common::recovery(world, me);
    }
    // Short of a full think (12,000 cycles, or its recent peak), set only the CPU policy: a think that runs out sets
    // none, and its limit, a share of income, stays too small when income falls, every second after. (Three quarters
    // of the peak let a poor faction start thinks it couldn't finish, every second for twenty minutes.)
    if rts.truthy() && cycles_left() < jmax(12000.0, me.get("compute").get("thinkPeak").or(0.0)) {
        return common::recovery(world, me);
    }
    let t = me.get("techs");
    let has = |k: &str| t.get(k).truthy();
    let bt = me.get("units").get("byType");
    let rim = common::frontier(world, me);
    // the best land first, marked down where the army couldn't hold on to it (common::can_hold)
    let hold = common::can_hold(world, me, false);
    let mut ranked = realm::rank(world, me, rim, |w, m, q| common::score(w, m, q) - if hold(q) { 0.0 } else { common::OVER_WATER });
    // An established archer column can finish a nearby capital instead of circling richer provinces.
    // Count capturers separately: aircraft and freight crews cannot occupy the town ring.
    let ground = (common::fighters(me) - bt.get("wing").or(0.0) - bt.get("ghost").or(0.0)).max(0.0);
    let campaign = rts_cpu(me).get("income").num_or(0.0) >= 60000.0
        && rts_cpu(me).get("bank").num_or(0.0) >= rts_cpu(me).get("income").num_or(0.0) * 2.0 && ground >= 12.0;
    if campaign {
        let lead = common::dominant(world).map(|f| f.get("id").num());
        let press_leader = lead.is_some_and(|id| id != me.get("id").num()
            && ranked.iter().any(|q| q.get("owner").num_or(-1.0) == id && hold(*q)));
        let capital = ranked.iter().copied().find(|q| q.get("capital").truthy() && hold(*q)
            && realm::is_rival(world, me, q.get("owner").num_or(-1.0))
            && common::muster_for(*q) <= ground * 0.75
            && (!press_leader || lead == Some(q.get("owner").num())));
        if let Some(q) = capital { ranked = vec![q]; }
    }
    let mut plan = common::survey(world, me, &ranked, 2.0);
    // The sky's strike for the vimanas (sky::pick), once Flight allows them and vimanas fly: its census is budgeted at
    // about 40 cycles a unit in sight and 40 a province, only when think can still finish after it; else last second's
    // strike stands.
    let last = mem::get(me.get("ramCapital"), 0);
    let mut strike: Option<sky::Strike> = None;
    if has("flight") && bt.get("wing").or(0.0) > 0.0 {
        let seen = world.get("units");
        let census = 40.0 * seen.len() as f64 + 40.0 * world.get("provinces").len() as f64 + 12000.0;
        if cycles_left() > census {
            let geo = sky::Geo::new(world, me, sky::REACH);
            strike = sky::pick(world, me, &plan, &geo, &last);
            if let Some(st) = &strike {
                plan.set("sky", st.at);
                plan.set("skyVerb", st.verb);
            }
        } else if vn(last.get("sky")) >= 0.0 {
            plan.set("sky", last.get("sky").clone());
            plan.set("skyVerb", last.get("skyVerb").clone());
        }
    }
    // the rest of think (spawns, budget, trade and diplomacy) is budgeted at up to 16,000 cycles after the search
    let defend = vn(plan.get("defend"));
    if let Some(field) = road(world, me, if defend >= 0.0 { defend } else { vn(plan.get("attack")) }, 16000.0) {
        plan.set("field", field);
    }
    let cpu = rts_cpu(me);
    // Fabrication has its own lane. Think reduces the shared bank once.
    let available = if cpu.truthy() { cpu.get("available") } else { cpu };
    let fab = if available.truthy() { available.get("fabrication") } else { available };
    let fabrication = if !rts.truthy() {
        Value::Num(1_000_000_000.0)
    } else if fab.is_undef() {
        rts.get("bank").value()
    } else {
        Value::Num(common::fabrication_room(me).unwrap_or_else(|| n(fab)))
    };
    let mut orders = obj! {
        "policy" => obj! { "extraction" => if defend >= 0.0 { 0.38 } else { 0.5 }, "autonomy" => "direct", "integrate" => "tolerate", "invest" => true },
        "spawn" => Value::arr(),
        "provinces" => Value::arr(),
        "fabrication" => fabrication,
        "treasury" => me.get("bank"),
        "unitPrice" => me.get("costs").get("unit"),
        "slots" => army_room(me),
        "priceLevel" => common::price_level(me),
    };
    common::share(&mut orders, &plan, world, me);

    let count = |k: &str| bt.get(k).or(0.0);
    let (miners, screens, runners, skirms, lancers) = (count("miner"), count("screen"), count("runner"), count("skirm"), count("lancer"));
    let (sentries, guns, ghosts, repairers) = (count("sentry"), count("gun"), count("ghost"), count("repairer"));
    let total = me.get("units").get("total").or(0.0);
    let spawning = |o: &Value| o.get("spawn").len();
    let income = cpu.get("income").or(0.0);
    // Maintain a small mining base first, no bigger than what our mines can use (common::miner_room: past that a miner
    // stands idle); then the vimanas; then rotate combat specialties instead of massing one dominant body. Three spawns
    // is also the engine's normal cap.
    if miners < jmax(8.0, (total * 0.4).floor()) && miners < common::miner_room(me, &plan) {
        spawn(&mut orders, "miner", MINER, vec![]);
    }
    let wings = count("wing");
    // Recruit the first affordable vimana before replacing the whole ground army. The shared economy guard
    // still charges its actual upkeep; additional wings require another 30,000 cycles of income each.
    let want_wings = if !emu && has("flight") && total >= 8.0 && income >= 18000.0 {
        jmin(jmin(8.0, jmax(1.0, (income / 30000.0).floor())), (total * 0.25).floor())
    } else { 0.0 };
    if wings < want_wings {
        let archer = has("airWeapons") && (wings as i64) % 3 == 2;
        spawn(&mut orders, "wing", if archer { WING_SHOT } else { WING }, mods(me, "wing"));
    }
    let column = if campaign { jmin(5.0, (income / 60000.0).floor()) } else { 0.0 };
    if screens < jmax(3.0 + column, ((skirms + guns + 1.0) * 0.42).floor()) {
        spawn(&mut orders, "screen", if emu {EMU_SCREEN}else{SCREEN}, mods(me, "screen"));
    }
    // Ranged is its own license. Before it arrives, fast melee runners scout and pull defenders out of position without
    // issuing invalid range spawns.
    if !has("ranged") && runners < 3.0 {
        spawn(&mut orders, "runner", RUNNER, mods(me, "skirm"));
    }
    if has("pierce") && has("ranged") && lancers < jmax(2.0, (screens / 2.0).ceil()) {
        spawn(&mut orders, "lancer", SKIRM, if has("dash") { vec!["pierce", "dash"] } else { vec!["pierce"] });
    }
    if has("ranged") && skirms < jmax(4.0, screens * 2.0) {
        spawn(&mut orders, "skirm", if emu {EMU_SKIRM}else{SKIRM}, mods(me, "skirm"));
    }
    if spawning(&orders) < 3 && has("antiAir") && has("ranged") && sentries < jmax(2.0, (screens / 3.0).floor()) {
        spawn(&mut orders, "sentry", SENTRY, mods(me, "sentry"));
    }
    if spawning(&orders) < 3 && has("artillery") && guns < jmax(1.0, (skirms / 4.0).floor()) {
        spawn(&mut orders, "gun", GUN, mods(me, "gun"));
    }
    // Repairers can sustain the exposed screen or themselves after a retreat.
    if spawning(&orders) < 3 && has("repair") && has("dash") && repairers < jmax(1.0, (skirms / 5.0).floor()) {
        spawn(&mut orders, "repairer", SCREEN, mods(me, "repair"));
    }
    if spawning(&orders) < 3 && has("stealth") && ghosts < 1.0 {
        spawn(&mut orders, "ghost", GHOST, mods(me, "ghost"));
    }

    // Absolute controller allowances keep an unlocked specialist viable after miners and screens have consumed their
    // ordinary route passes. A war unit's call fits one route search for the units abroad (march).
    let control = if cpu.truthy() && cpu.get("available").truthy() { n(cpu.get("available").get("control")) * 0.8 } else { f64::INFINITY };
    let type_calls = common::calls(world, &TYPES, income, control, bt);
    // common::budget keeps its useful research choice, rent choice and type schedules. In CPU RTS the old split is
    // ignored; cpu_policy below atomically installs caps and reserves for the one shared balance.
    let budget = common::Budget { research: if has("repair") { 0.06 } else { 0.12 }, code: 0.72, techs: doctrine, types: Some(&type_calls) };
    common::budget(&mut orders, world, me, &budget);
    let mut policy = cpu_policy(me, &plan, &type_calls);
    common::fit_cpu(&mut policy, me);
    orders.entry("compute").set("cpu", policy);
    // the sorties so far (sky::tally), and the memo spectators read: where the vimanas fly, or how near Flight is
    let air = if (wings > 0.0 || !last.get("sky").is_undef()) && cycles_left() > 9000.0 { sky::tally(world, me, &mut orders) } else { sky::Tally::default() };
    let memo = if emu {
        format!("Emu High Command: {} runners and {} skirmishers advancing on the frontier",js_num(screens),js_num(skirms))
    } else if has("flight") {
        let at = |id: f64| realm::index(world).get(id).get("name").str().to_string();
        let doing = match &strike {
            Some(st) if st.verb == "raid" => format!("raiding {}", at(st.at)),
            Some(st) if vn(plan.get("defend")) == st.at || same_v(me.get("capital"), &Value::Num(st.at)) => format!("defending {}", at(st.at)),
            Some(st) => format!("striking {}", at(st.at)),
            None => "circling Kurukshetra".to_string(),
        };
        format!(
            "Arjuna: {} vimanas, {} aloft, {}; {} strikes in {} sorties | {} archers, {} elephants",
            js_num(wings), js_num(air.aloft), doing, js_num(air.strikes), js_num(air.sorties), js_num(skirms), js_num(screens)
        )
    } else {
        let r = me.get("research");
        let fl = r.get("progress").get("flight").or(0.0) / r.get("need").get("flight").or(1000.0);
        format!("Arjuna: Krishna drives on toward Flight, {}% | {} archers, {} elephants", js_num(round(fl * 100.0)), js_num(skirms), js_num(screens))
    };
    orders.set("memo", memo);
    // A short second keeps what's decided so far (the plan, spawns and the CPU policy): trade and diplomacy wait a
    // second rather than run think out of cycles and lose it all.
    if cycles_left() < 5000.0 {
        return orders;
    }
    let pact_leader = realm::leader(world);
    let at = mem::get(me.get("ramCapital"), CLOCK);
    let mut clock = Clock { at: if at.truthy() { at } else { Value::obj() }, changed: false };
    warlord(&mut orders, world, me, &plan, &mut clock);
    if !emu { secrets(&mut orders, world, me, &mut clock); }
    if cycles_left() > 2000.0 {
        let pact = |m: View| me.get("pacts").is_empty() && pact_leader.is_none_or(|l| !same(m.get("from"), l.get("id")));
        common::answer(&mut orders, me, world, pact, doctrine);
    }
    if clock.changed {
        let ops = mem::put(CLOCK, &clock.at, "local", None, 2).unwrap_or_default();
        match orders.entry("ram") {
            Value::Arr(a) => a.extend(ops),
            x => *x = Value::Arr(ops),
        }
    }
    payable(orders, me)
}

// The warlords' trade: a bribe to soften the target before the push, passage bought from a neighbour that isn't the
// target, so columns cross its land in peace, and largesse to keep what they took.
fn warlord(orders: &mut Value, world: View, me: View, plan: &Value, clock: &mut Clock) {
    let now = world.get("seconds").or(0.0).floor();
    let by_id = realm::index(world);
    let me_id = me.get("id");
    // ◆ the spawns this second will pay don't go on bribes
    let coin = n(me.get("bank")) - orders.get("spawn").len() as f64 * (n(me.get("costs").get("unit")) + 8.0);
    let attack = vn(plan.get("attack"));
    let target = if attack >= 0.0 { by_id.get(attack) } else { View::undefined() };
    let owner = target.get("owner");
    if target.truthy()
        && n(owner) >= 0.0
        && realm::is_rival(world, me, n(owner))
        && coin > 700.0
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
    // waits until 60◆ remains after this second's spawns
    if coin >= 60.0 && due(clock, "largesse", 10.0, now) {
        let mut worst: Option<View> = None;
        for p in world.get("provinces").iter() {
            if same(p.get("owner"), me_id) && n(p.get("risk")) > 0.04 && worst.is_none_or(|w| n(p.get("risk")) > n(w.get("risk"))) {
                worst = Some(p);
            }
        }
        if let Some(w) = worst {
            sway::bribe(orders, n(w.get("id")), 60.0);
        }
    }
}

// Indra's gifts: the techs of the sky (Flight, and Artillery and Air weapons after it), once another faction holds
// them, are bought, not solved. Every 40 s or so (give or take 60%) the first of them whose prerequisites we hold is
// asked of a faction that has it (an ally first, then the biggest), for 160◆: the house rule (common::worth) takes
// 150◆ for a tech it sells, and the seller keeps it. Only these: the ◆ buys the army too (each unit costs ◆), and an
// army short of ◆ lost land in every world that bought eight techs.
const SECRET_PRICE: f64 = 160.0;
const SKY_TECHS: [&str; 3] = ["flight", "artillery", "airWeapons"];

fn secrets(orders: &mut Value, world: View, me: View, clock: &mut Clock) {
    let now = world.get("seconds").or(0.0).floor();
    let coin = n(me.get("bank")) - orders.get("spawn").len() as f64 * (n(me.get("costs").get("unit")) + 8.0) - orders.get("bribe").len() as f64 * 150.0;
    if coin < SECRET_PRICE + 6.0 * n(me.get("costs").get("unit")) || SKY_TECHS.iter().all(|t| me.get("techs").get(t).truthy()) || !due(clock, "secret", 40.0, now) {
        return;
    }
    let (all, mine) = (world.get("techs"), me.get("techs"));
    let me_id = n(me.get("id"));
    for t in SKY_TECHS {
        let x = all.get(t);
        if !x.truthy() || mine.get(t).truthy() || !x.get("requires").iter().all(|q| mine.get(q.str()).truthy()) {
            continue;
        }
        let mut seller: Option<View> = None;
        for f in world.get("factions").iter() {
            if !f.get("alive").truthy() || n(f.get("id")) == me_id || !f.get("techs").has_str(t) {
                continue;
            }
            let better = match seller {
                None => true,
                Some(s) => {
                    let (fa, sa) = (me.get("allies").has_num(n(f.get("id"))), me.get("allies").has_num(n(s.get("id"))));
                    (fa && !sa) || (fa == sa && n(f.get("share")) > n(s.get("share")))
                }
            };
            if better {
                seller = Some(f);
            }
        }
        if let Some(s) = seller {
            let name = all.get(t).get("name").str();
            let deal = obj! { "give" => obj! { "coin" => SECRET_PRICE }, "ask" => obj! { "techs" => vec![t] } };
            diplomacy::send(orders, s.get("id"), format!("Arjuna asks the secret of {name}"), deal);
            return;
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
    let Some(ops) = common::carry(u, p, me) else { return o };
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

// The war units' turn, from a rotating start so each gets its turn when cycles run short. A call with room for a route
// search and the orders after it takes the units abroad first: one of them may need the search (path::to), which
// serves every unit sent the same way. A call handed less than one order's worth (the control lane ran short after
// think) keeps its standing orders.
fn march(units: View, world: View, me: View) {
    if me.get("rts").truthy() && cycles_left() < 4500.0 {
        return;
    }
    let first = cycles_left() >= 300.0 * world.get("provinces").len() as f64 + 3000.0;
    let (mut queue, mut home) = (Vec::new(), Vec::new());
    for i in 0..units.len() {
        let u = turn(units, i, world);
        if !first || plan::here(me, u) {
            home.push(u);
        } else {
            queue.push(u);
        }
    }
    // the units abroad go to the front, the last one met first
    queue.reverse();
    queue.extend(home);
    let rts = me.get("rts").truthy();
    for u in queue {
        if rts && cycles_left() < 4500.0 {
            break;
        }
        if let Some(o) = war_order(u, world, me) {
            order(o);
        }
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

fn sentry(units: View, world: View, me: View) {
    march(units, world, me);
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

// Vimanas fly sorties, not patrols: a flier's burn climbs the longer it's up (docs part rts, "flight"), and rivals
// research anti-air. A vimana down on our land cools off (3 s of heat a second); once cool it launches with the
// squadron for think's strike (plan.sky, sky::pick), at least three together, or at once to defend our own land. Over
// the target it fights, picking off miners, archers and besiegers (reflex); with nothing left to hit it raids a rival
// town no ground unit guards, or turns home. It turns home as soon as its heat and the hops back would bring it down
// past TURN seconds up, and lands on our nearest land (or an ally's). Abroad a vimana knows only its own order, so the
// way out and back are its own: sky::Geo routes over anything a flier crosses.
const TURN: f64 = 50.0;
const READY: f64 = 6.0;
const HOP: f64 = 4.5;

fn wing_full_path_need(units: View, world: View) -> f64 {
    6000.0 + world.get("provinces").len() as f64 * 30.0 + units.len() as f64 * 37.5
}

fn wing(units: View, world: View, me: View) {
    if common::air_recovery(units, me) { return; }
    let rts = me.get("rts").truthy();
    if rts && cycles_left() < 2000.0 {
        for u in units.iter().take_while(|_| cycles_left() > 400.0) {
            let air = u.path("rts.air");
            if air.get("aloft").truthy() && n(air.get("heat")) >= 45.0 {
                order(obj! { "id" => u.get("id"), "land" => true });
            }
        }
        return;
    }
    if rts && cycles_left() < wing_full_path_need(units, world) {
        // A short call still flies the army's target and lands hot aircraft.
        common::fly(units, world, me);
        return;
    }

    // A home province supplies the current squadron plan. Abroad, each aircraft keeps its paid sortie.
    let home = units.iter().find(|u| plan::here(me, *u));
    let pl = match home {
        Some(u) => plan::of(me, u),
        None => Rc::new(Value::obj()),
    };
    let target = if vn(pl.get("sky")) >= 0.0 { vn(pl.get("sky")) } else { vn(pl.get("attack")) };
    let target = if target.is_nan() { -1.0 } else { target };
    // A fighter needs the visible rival's id to begin interception. Ground strikes keep their own objective.
    let contacts: Vec<View> = me.path("rts.air.contacts").iter().take_while(|_| cycles_left() > 3500.0)
        .filter(|u| u.get("air").truthy() && realm::is_rival(world, me, u.get("owner").num_or(-1.0)))
        .take(12).collect();
    let verb = if is_str_v(pl.get("skyVerb"), "raid") { "raid" } else { "fight" };
    let me_id = n(me.get("id"));
    let mut geo = None;
    // the squadron: how many are down and cool
    let mut cool = 0usize;
    for u in units.iter() {
        let a = u.path("rts.air");
        if a.truthy() && !a.get("aloft").truthy() && n(a.get("heat")) <= READY {
            cool += 1;
        }
    }
    let mut urgent_lands = Vec::new();
    for u in units.iter().take_while(|_| !rts || cycles_left() > 400.0) {
        let a = u.path("rts.air");
        if a.get("aloft").truthy() && n(a.get("heat")) >= 45.0 {
            order(obj! { "id" => u.get("id"), "land" => true });
            urgent_lands.push(n(u.get("id")));
        }
    }
    for i in 0..units.len() {
        if rts && cycles_left() < 1600.0 {
            break;
        }
        let u = turn(units, i, world);
        let a = u.path("rts.air");
        if !a.truthy() || !a.get("glide").is_nullish() {
            continue;
        }
        if urgent_lands.contains(&n(u.get("id"))) { continue; }
        let geo = geo.get_or_insert_with(|| sky::Geo::route_only(world));
        let heat = n(a.get("heat"));
        let doing = u.get("doing");
        let (at, to, tgt) = (n(u.get("at")), n(u.get("to")), n(u.get("target")));
        let standing = is_str(doing, "fight") || is_str(doing, "raid");
        let target = if home.is_none() && standing && tgt >= 0.0 { tgt } else { target };
        let focus = if u.get("range").num_or(0.0) == 0.0 {
            contacts.iter().filter(|v| v.get("at").num_or(-1.0) == target)
                .min_by(|a,b| a.get("hp").num_or(1.0).total_cmp(&b.get("hp").num_or(1.0)))
                .map(|v|v.get("id").num())
        } else { None };
        let verb = if home.is_none() && standing { if is_str(doing, "raid") { "raid" } else { "fight" } } else { verb };
        // A short-handed squadron only launches alone to defend home. Finding home across a full map is optional
        // work, though: keep the runway for hot aircraft to land before buying that map search.
        let go = target >= 0.0 && (cool >= units.len().min(3) || (cycles_left() >= wing_full_path_need(units, world) && {
            geo.ensure_home(world, me, 12.0);
            geo.is_home(target)
        }));
        let carried = home.map(|_| &pl);
        if a.get("aloft").truthy() {
            if rts && cycles_left() < 5000.0 { continue; }
            if is_str(doing, "move") {
                geo.ensure_home(world, me, 12.0);
                if geo.is_home(tgt) {
                // on its way down
                    continue;
                }
            }
            geo.ensure_home(world, me, 12.0);
            let here = if to >= 0.0 { to } else { at };
            let back = geo.home_d(here);
            if heat + HOP * back >= TURN {
                // over our land with no strike, it settles and comes down by itself: another order would only move it
                let busy = is_str(doing, "fight") || is_str(doing, "raid") || is_str(doing, "move");
                if back > 0.0 || to >= 0.0 || busy {
                    order(obj! { "id" => u.get("id"), "land" => true });
                }
                continue;
            }
            // Abroad, the wing can finish its carried objective while the capital's plan is out of reach.
            let strike_target = if target >= 0.0 { target } else { tgt };
            if let Some(run) = common::bombing_run(u, world, me, strike_target) {
                if let Some(o) = run { order(o); }
                continue;
            }
            let striking = is_str(doing, "fight") || is_str(doing, "raid");
            if striking && tgt == target && focus.is_some_and(|id| u.get("focus").num_or(-1.0) != id) {
                if let Some(o) = launch(u, target, "fight", &geo, focus, carried, me) { order(o); }
                continue;
            }
            if striking && tgt == at && to < 0.0 {
                // over its target: with no rival there left to hit, raid a rival town no ground unit guards, else home
                let (foes, guards, _) = sky::hostiles(world, me, at);
                if foes == 0.0 {
                    let q = realm::index(world).get(at);
                    let rival = q.get("owner").is_num() && q.get("owner").num() >= 0.0 && q.get("owner").num() != me_id && realm::is_rival(world, me, q.get("owner").num());
                    if rival && guards == 0.0 {
                        if is_str(doing, "fight") {
                            order(obj! { "id" => u.get("id"), "do" => "raid", "target" => at, "alert" => 8 });
                        }
                    } else if !geo.is_home(at) || target != at {
                        order(obj! { "id" => u.get("id"), "land" => true });
                    }
                }
                continue;
            }
            if striking || is_str(doing, "move") {
                continue;
            }
            // up with no sortie of its own (it rose off a step at home, or is settling after one): cool, the strike if
            // there is one; hot over our land, it settles and comes down; anywhere else, home
            if heat <= READY && go {
                if let Some(o) = launch(u, target, verb, &geo, focus, carried, me) {
                    order(o);
                    continue;
                }
            }
            if back > 0.0 || to >= 0.0 {
                order(obj! { "id" => u.get("id"), "land" => true });
            }
            continue;
        }
        // down: cool off, then go with the squadron
        if heat > READY || !go || (tgt == target && is_str(doing, verb)) {
            continue;
        }
        if let Some(o) = launch(u, target, verb, &geo, focus, carried, me) {
            order(o);
        }
    }
}

/// The order sending vimana `u` to `verb` at province `to`, by the shortest way a flier has (sky::Geo::route). Its
/// drive carries a little power (faster sorties burn less), and its blows a little more force.
fn launch(u: View, to: f64, verb: &str, geo: &sky::Geo, focus: Option<f64>, plan: Option<&Rc<Value>>, me: View) -> Option<Value> {
    if to < 0.0 {
        return None;
    }
    let from = if n(u.get("to")) >= 0.0 { n(u.get("to")) } else { n(u.get("at")) };
    let mut o = obj! { "id" => u.get("id"), "do" => verb, "target" => to, "alert" => 8 };
    if let Some(id) = focus { o.set("focus", id); }
    if from != to {
        let via = geo.route(from, to)?;
        o.set("via", via);
    }
    o.set("power", obj! { "drive" => 120, "armor" => 0, "weapon" => 200 });
    if cycles_left() > 3000.0 {
        if let Some(plan) = plan {
            if let Some(ram) = common::carry(u, plan, me) { o.set("ram", ram); }
        }
    }
    Some(o)
}

fn ghost(units: View, world: View, me: View) {
    let rts = me.get("rts").truthy();
    for i in 0..units.len() {
        if rts && cycles_left() < 2200.0 {
            break;
        }
        let u = turn(units, i, world);
        // One-hop scouting avoids a repeated whole-world route search on the very small ghost allocation. Its cloak
        // makes the information probe risky, but it does not consume combat CPU every second just to pathfind.
        if is_str(u.get("doing"), "move") {
            continue;
        }
        let to = common::wander(world, me, n(u.get("at")));
        if to >= 0.0 {
            order(obj! { "id" => u.get("id"), "do" => "move", "target" => to, "cloak" => true, "alert" => 7 });
        }
    }
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
    // Keep 3,200 cycles for a first unit turn, then twice the cost of the heaviest turn so far.
    let mut turn_cost = 3200.0_f64;
    for i in 0..len {
        if cycles_left() < turn_cost {
            break;
        }
        let began = cycles_left();
        let u = units.at(((start + i as f64) % len as f64) as usize);
        let ns = near.key(n(u.get("id")));
        if !ns.truthy() {
            continue;
        }
        let r = u.get("rts");
        let modules = r.get("modules");
        let ty = u.get("type");
        let flier = is_str(ty, "wing");
        let fighter = flier && !u.get("range").truthy();
        let anti_air = !flier && modules.has_str("antiAir");
        let (mut best, mut score, mut melee, mut air): (Option<View>, f64, Option<View>, Option<View>) = (None, 99.0, None, None);
        // a vimana hovers where it was sent (its target's town), so prey farther off is flown at: the best of it
        let (mut chase, mut chase_score): (Option<View>, f64) = (None, 99.0);
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
            if e_air && !u.get("air").truthy() && !(r.truthy() && modules.truthy() && modules.has_str("antiAir")) {
                continue;
            }
            // vimanas go for miners, besiegers, archers and rival fliers first: what feeds a rival, takes our land or
            // can shoot back
            let s = n(e.get("hp")) / jmax(1.0, n(e.get("hpMax")))
                - (if e.get("range").truthy() { if flier { 0.5 } else { 0.12 } } else { 0.0 })
                - (if e_air { if fighter || anti_air { 2.0 } else if flier { 0.45 } else { 0.08 } } else { 0.0 })
                + d * 0.015
                - (if flier && (is_str(e.get("doing"), "gather") || is_str(e.get("type"), "miner")) { 0.6 } else { 0.0 })
                - (if flier && is_str(e.get("doing"), "capture") { 0.4 } else { 0.0 });
            if d > n(u.get("reach")) {
                if flier && s < chase_score {
                    chase_score = s;
                    chase = Some(e);
                }
                continue;
            }
            if s < score {
                score = s;
                best = Some(e);
            }
        }
        best = common::windup_target(u, ns, best);
        // Repair costs a meaningful 700 cycles (at price level 1) and restores only 0.7 HP, so spend it after a real
        // exchange, early enough that the line can disengage safely. Every price here is the rules' times the level.
        let budget = r.get("budget");
        let bank = if r.get("bank").is_undef() { 1_000_000_000.0 } else { n(r.get("bank")) };
        let level = common::price_level(u);
        let lane = |name: &str, cost: f64| bank >= cost * level && (budget.get(name).is_undef() || n(budget.get(name)) >= cost * level);
        let hurt = n(u.get("hp")) < n(u.get("hpMax")) * 0.62;
        let pressured = melee.is_some() || hurt;
        let weapon = if best.is_some() && lane("weapon", 620.0) { 500.0 } else { 0.0 };
        // a vimana needs no armor (nothing on the ground hits it but anti-air), and little drive over its target:
        // its flight is already the dearest thing it does
        let drive = if pressured { 900.0 } else { 420.0 };
        let drive_cost = common::drive_tick(u, drive);
        let power = if flier {
            obj! { "drive" => if lane("drive", 50.0) { 120.0 } else { 0.0 }, "armor" => 0.0, "weapon" => if best.is_some() && lane("weapon", 320.0) { 200.0 } else { 0.0 } }
        } else {
            obj! {
                "drive" => if lane("drive", drive_cost) { drive } else { 0.0 },
                "armor" => if pressured && lane("armor", 170.0) { 850.0 } else { 0.0 },
                "weapon" => weapon,
            }
        };
        let weapon = if flier { if best.is_some() { 200.0 } else { 0.0 } } else { weapon };
        let penetration = pierce_mode(u, best);
        let mut out = obj! { "id" => u.get("id"), "power" => power, "guard" => guard_facing(ns), "pierce" => penetration };
        if best.is_some() && !lane("weapon", ((120.0 + weapon) * (1.0 + 1.4 * penetration * penetration)).ceil()) {
            best = None;
        }

        // Dodge telegraphed artillery before deciding to trade shots.
        let pos = if u.get("local").truthy() { u.get("local") } else { u.get("pos") };
        let (px, py) = (n(pos.get("x")), n(pos.get("y")));
        let hazard = r.get("hazards").iter().find(|h| n(h.get("impactIn")) < 1.3 && (px - n(h.get("x"))).hypot(py - n(h.get("y"))) < n(h.get("radius")) + 0.8);
        if let Some(h) = hazard {
            out.set("step", obj! { "x" => px - n(h.get("x")), "y" => py - n(h.get("y")) });
        } else if let Some(m) = melee.filter(|_| !flier) {
            if u.get("range").truthy() || is_str(ty, "repairer") || hurt {
                out.set("step", flee_step(m));
            }
        } else if let Some(w) = air.filter(|w| flier && u.get("range").truthy() && n(w.get("d")) < 2.0 && n(w.get("range")) == 0.0) {
            // a vimana with a bow keeps its distance from a rival flier that can only strike up close
            out.set("step", flee_step(w));
        } else if let Some(c) = chase.filter(|_| {
            // a vimana on its way down, or near its turn, flies at nothing: it must stand still on our land to cool
            let air = r.get("air");
            let doing = u.get("doing");
            flier && best.is_none() && (is_str(doing, "fight") || is_str(doing, "raid")) && n(air.get("heat")) < TURN - 6.0 && !air.get("glide").is_num()
        }) {
            let d = jmax(0.01, n(c.get("d")));
            out.set("step", obj! { "x" => n(c.get("dx")) / d, "y" => n(c.get("dy")) / d });
        }
        if let Some(b) = best {
            out.set("hit", b.get("id")); // movement and fire intentionally share one order
        }
        common::space_ranged(u, ns, &mut out);

        let cd = r.get("abilityIn");
        let abilities = budget.get("abilities");
        let mended = repair_friend(u, ns, &mut out, jmin(bank, if abilities.is_nullish() { bank } else { n(abilities) }), level);
        if mended != 0.0 && !mended.is_nan() {
            // paid support or self-repair
        } else if melee.is_some() && modules.has_str("dash") && n(r.get("dashIn")) <= 0.0 && lane("abilities", 900.0) {
            out.set("ability", "dash");
        } else if (air.is_some() || (is_str(ty, "sentry") && best.is_some())) && modules.has_str("scan") && cd.get("scan").or(0.0) <= 0.0 && lane("abilities", 700.0) {
            out.set("ability", "scan");
        } else if let Some(b) = best.filter(|_| is_str(ty, "gun") && r.get("deployed").truthy() && cd.get("shell").or(0.0) <= 0.0 && pos.truthy()) {
            let (bx, by) = (n(b.get("dx")), n(b.get("dy")));
            let cluster = ns.iter().filter(|e| e.get("rival").truthy() && (n(e.get("dx")) - bx).hypot(n(e.get("dy")) - by) <= 2.0).count();
            let radius = if cluster >= 3 { 3.0 } else if cluster >= 2 { 2.0 } else { 1.0 };
            let shell_cost = 1000.0 * ((radius / 2.0) * (radius / 2.0)) * (1.0 + jmax(n(b.get("d")) - 5.0, 0.0) * 0.12);
            let safe = ns.iter().all(|e| e.get("rival").truthy() || (n(e.get("dx")) - bx).hypot(n(e.get("dy")) - by) > radius + 0.5);
            if safe && lane("abilities", shell_cost) {
                out.set("ability", "shell");
                out.set("radius", radius);
                out.set("aim", obj! { "x" => px + bx, "y" => py + by });
            }
        }
        common::steer_ground(u, ns, &mut out);
        if best.is_some() || out.get("step").truthy() || out.get("ability").truthy() {
            order(out);
        }
        turn_cost = turn_cost.max(2.0 * (began - cycles_left()));
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

/// `x === s` for a string built here or read from RAM.
fn is_str_v(v: &Value, s: &str) -> bool {
    matches!(v, Value::Str(x) if x == s)
}

// ---- the sky: where the vimanas strike, the ways a flier takes, and the tally of sorties ----
mod sky {
    use gameofcores::*;

    /// How many hops from our land think looks for a strike: a hop takes a vimana about 4.5 s, so four out and four
    /// back leave a sortie some 15 s over its target before its burn climbs.
    pub const REACH: f64 = 4.0;

    /// The world as a flier sees it: every province's neighbours by land, sea, pass or air (a vimana crosses
    /// anything), and how many hops each lies from our land or an ally's (where a vimana may come down), up to `max`.
    pub struct Geo {
        adj: Vec<Vec<usize>>,
        home: Vec<f64>,
    }

    fn each(v: View, out: &mut Vec<usize>) {
        if v.is_arr() {
            for x in v.iter() {
                if x.is_num() && x.num() >= 0.0 {
                    out.push(x.num() as usize);
                }
            }
        } else if v.is_num() && v.num() >= 0.0 {
            out.push(v.num() as usize);
        }
    }

    impl Geo {
        pub fn new(world: View, me: View, max: f64) -> Geo {
            let mut geo = Geo::route_only(world);
            geo.ensure_home(world, me, max);
            geo
        }

        pub fn route_only(world: View) -> Geo {
            let ps = world.get("provinces");
            let mut size = 0usize;
            for p in ps.iter() {
                size = size.max(p.get("id").num_or(0.0) as usize + 1);
            }
            let mut adj = vec![Vec::new(); size];
            for p in ps.iter() {
                let id = p.get("id").num_or(-1.0);
                if !(id >= 0.0) {
                    continue;
                }
                let i = id as usize;
                each(p.get("adj"), &mut adj[i]);
                let r = p.get("routes");
                if r.truthy() {
                    each(r.get("sea"), &mut adj[i]);
                    each(r.get("pass"), &mut adj[i]);
                    each(r.get("air"), &mut adj[i]);
                }
            }
            Geo { adj, home: Vec::new() }
        }

        pub fn ensure_home(&mut self, world: View, me: View, max: f64) {
            if !self.home.is_empty() || self.adj.is_empty() { return; }
            let size = self.adj.len();
            self.home = vec![f64::INFINITY; size];
            let mut ring = Vec::new();
            let me_id = me.get("id").num();
            let allies = me.get("allies");
            for p in world.get("provinces").iter() {
                let id = p.get("id").num_or(-1.0);
                if !(id >= 0.0) { continue; }
                let i = id as usize;
                let o = p.get("owner");
                if o.is_num() && o.num() >= 0.0 && (o.num() == me_id || allies.has_num(o.num())) {
                    self.home[i] = 0.0;
                    ring.push(i);
                }
            }
            let mut d = 0.0;
            while !ring.is_empty() && d < max {
                d += 1.0;
                let mut next = Vec::new();
                for &p in &ring {
                    for k in 0..self.adj[p].len() {
                        let q = self.adj[p][k];
                        if q < size && self.home[q] > d {
                            self.home[q] = d;
                            next.push(q);
                        }
                    }
                }
                ring = next;
            }
        }

        /// Hops from our land or an ally's (0 on it; infinite past the search).
        pub fn home_d(&self, id: f64) -> f64 {
            if id >= 0.0 && (id as usize) < self.home.len() {
                self.home[id as usize]
            } else {
                f64::INFINITY
            }
        }

        pub fn is_home(&self, id: f64) -> bool {
            self.home_d(id) == 0.0
        }

        /// The provinces a flier passes from `from` to `to`, `to` last (a breadth-first search, up to 64 hops).
        pub fn route(&self, from: f64, to: f64) -> Option<Vec<f64>> {
            let size = self.adj.len();
            if !(from >= 0.0 && to >= 0.0) || from as usize >= size || to as usize >= size {
                return None;
            }
            let (from, to) = (from as usize, to as usize);
            let mut back = vec![usize::MAX; size];
            back[from] = from;
            let mut ring = vec![from];
            let mut hops = 0;
            while back[to] == usize::MAX && !ring.is_empty() && hops < 64 {
                hops += 1;
                let mut next = Vec::new();
                for &p in &ring {
                    for &q in &self.adj[p] {
                        if q < size && back[q] == usize::MAX {
                            back[q] = p;
                            next.push(q);
                        }
                    }
                }
                ring = next;
            }
            if back[to] == usize::MAX {
                return None;
            }
            let mut way = vec![to as f64];
            let mut c = to;
            while back[c] != from {
                c = back[c];
                way.push(c as f64);
            }
            way.reverse();
            Some(way)
        }
    }

    /// Rival unit types the house programs arm with anti-air: a vimana keeps clear of where they stand.
    const AA_TYPES: [&str; 3] = ["interceptor", "needle", "sentry"];

    #[derive(Clone)]
    pub struct Strike {
        pub at: f64,
        pub verb: &'static str,
        pub score: f64,
    }

    /// Count visible rivals in province `at`. Treat every counted unit as a defender.
    /// Unit callbacks receive province counts; local reflexes choose targets and positions.
    pub fn hostiles(world: View, me: View, at: f64) -> (f64, f64, Option<(f64, f64)>) {
        let counts = realm::index(world).get(at).get("units");
        let foes = world.get("factions").iter().filter(|f| realm::is_rival(world, me, f.get("id").num_or(-1.0)))
            .map(|f| counts.key(f.get("id").num()).num_or(0.0)).sum();
        (foes, foes, None)
    }

    /// Where the vimanas strike now, within [`REACH`] hops of our land: each province we see scores its prey (a miner
    /// or a besieger 1.5 and 1.3, an archer 1, another ground unit 0.7, a rival flier 0.6), less 2.5 for each unit there
    /// that can shoot a flier down (a rival flier, an anti-air type, or an archer of a faction with Anti-air at a
    /// quarter), less half a point a hop. Invaders in our own land count double and 3 more (6 at the capital), the
    /// land our army attacks 2 more. A rival town we see with no ground unit in it is worth a raid: 1.2 and half a
    /// point a core, less the hops. The last strike stands while it scores within 1.5 of the best. `None` under 1.
    pub fn pick(world: View, me: View, plan: &Value, geo: &Geo, last: &Value) -> Option<Strike> {
        let me_id = me.get("id").num();
        let size = geo.home.len();
        let (mut prey, mut aa, mut ground) = (vec![0.0f64; size], vec![0.0f64; size], vec![0.0f64; size]);
        let factions = world.get("factions");
        let mut anti: Vec<(f64, bool)> = Vec::new();
        for u in world.get("units").iter() {
            let at = u.get("at").num_or(-1.0);
            if !(at >= 0.0) || at as usize >= size || geo.home[at as usize] > REACH {
                continue;
            }
            let o = u.get("owner").num_or(-1.0);
            if o == me_id || !realm::is_rival(world, me, o) {
                continue;
            }
            let i = at as usize;
            let b = u.get("build");
            let ranged = b.get("range").num_or(0.0) > 0.0;
            if b.get("wings").truthy() {
                aa[i] += 0.5;
                prey[i] += 4.0;
                continue;
            }
            ground[i] += 1.0;
            if ranged {
                if AA_TYPES.contains(&u.get("type").str()) {
                    aa[i] += 1.0;
                } else {
                    let has = match anti.iter().find(|x| x.0 == o) {
                        Some(x) => x.1,
                        None => {
                            let h = factions.key(o).get("techs").has_str("antiAir");
                            anti.push((o, h));
                            h
                        }
                    };
                    if has {
                        aa[i] += 0.25;
                    }
                }
            }
            let doing = u.get("doing").str();
            prey[i] += if doing == "gather" || b.get("carry").num_or(0.0) >= 3.0 {
                1.5
            } else if doing == "capture" {
                1.3
            } else if ranged {
                1.0
            } else {
                0.7
            };
        }
        let attack = plan.get("attack").num_or(-1.0);
        let cap = me.get("capital").num_or(-1.0);
        let was = last.get("sky").num_or(-1.0);
        let (mut best, mut keep): (Option<Strike>, Option<Strike>) = (None, None);
        for p in world.get("provinces").iter() {
            let id = p.get("id").num_or(-1.0);
            if !(id >= 0.0) || id as usize >= size {
                continue;
            }
            let i = id as usize;
            let d = geo.home[i];
            if d > REACH {
                continue;
            }
            let o = p.get("owner");
            let owner = o.num_or(-1.0);
            let mine = o.is_num() && owner == me_id;
            let ally = o.is_num() && owner >= 0.0 && !mine && realm::is_ally(me, owner);
            let (s, verb) = if prey[i] > 0.0 {
                let mut s = prey[i].min(8.0) - 2.5 * aa[i] - 0.5 * d;
                if mine {
                    s += 3.0 + prey[i] + if id == cap { 3.0 } else { 0.0 };
                } else if ally {
                    s += 1.0;
                }
                if id == attack {
                    s += 2.0;
                }
                (s, "fight")
            } else if o.is_num() && owner >= 0.0 && !mine && !ally && p.get("visible").truthy() && d <= 3.0 && realm::is_rival(world, me, owner) {
                (1.2 + 0.5 * p.get("cores").num_or(0.0) - 0.5 * d, "raid")
            } else {
                continue;
            };
            // a sortie costs a vimana some 60,000 cycles: only a strike worth it (any defence of our land is)
            if s < if mine { 1.0 } else { 1.8 } {
                continue;
            }
            let st = Strike { at: id, verb, score: s };
            if id == was {
                keep = Some(st.clone());
            }
            if best.as_ref().is_none_or(|b| s > b.score) {
                best = Some(st);
            }
        }
        match (keep, &best) {
            (Some(k), Some(b)) if k.score >= b.score - 1.5 => Some(k),
            _ => best,
        }
    }

    /// The vimanas' record so far: blows struck, sorties flown (each time one rose off our land), and how many are up
    /// now. Kept in capital RAM pages 226-231 (`{ s, k, w: { id: [shotSeq, up] } }`), written when it changes; a console
    /// line every 5 minutes of the game's clock says it.
    #[derive(Default)]
    pub struct Tally {
        pub strikes: f64,
        pub sorties: f64,
        pub aloft: f64,
    }

    const PAGE: u32 = 226;

    pub fn tally(world: View, me: View, orders: &mut Value) -> Tally {
        let me_id = me.get("id").num();
        let was = mem::get(me.get("ramCapital"), PAGE);
        let (mut s, mut k) = (was.get("s").num_or(0.0), was.get("k").num_or(0.0));
        let seen = was.get("w");
        let mut w = Value::obj();
        let mut aloft = 0.0;
        for u in world.get("units").iter() {
            if u.get("owner").num_or(-1.0) != me_id || !u.get("air").truthy() {
                continue;
            }
            let r = u.get("rts");
            let seq = r.get("shotSeq").num_or(0.0);
            let up = r.get("air").get("aloft").truthy();
            let key = js_key(u.get("id").num());
            let before = seen.get(&key);
            let (pseq, pup) = if before.is_undef() { (0.0, false) } else { (before.at(0).num_or(0.0), before.at(1).num_or(0.0) > 0.0) };
            s += (seq - pseq).max(0.0);
            if up && !pup {
                k += 1.0;
            }
            if up {
                aloft += 1.0;
            }
            w.set(&key, arr![seq, if up { 1.0 } else { 0.0 }]);
        }
        // on the game's clock, so a write that didn't land (think short of cycles) can't repeat the line
        let now = world.get("seconds").num_or(0.0);
        if now > 0.0 && now % 300.0 == 0.0 {
            log(format!("air: {} sorties, {} blows struck, {} vimanas, {} aloft", k, s, w.len(), aloft));
        }
        let rec = obj! { "s" => s, "k" => k, "w" => w };
        if rec.to_json() != was.to_json() {
            if let Some(ops) = mem::put(PAGE, &rec, "local", None, 6) {
                for op in ops {
                    orders.entry("ram").push(op);
                }
            }
        }
        Tally { strikes: s, sorties: k, aloft }
    }

    fn js_key(x: f64) -> String {
        if x.fract() == 0.0 && x.abs() < 1e15 {
            format!("{}", x as i64)
        } else {
            format!("{x}")
        }
    }
}
