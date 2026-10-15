// Kublai Khan's Alt: the court of Xanadu, a faction that rules by diplomacy and strikes with a fast Keshig army where
// its envoys opened the way.
//
// The court (chancery, below) does five things every game: its envoys write to every other court in turn; it holds
// the one pact a faction may hold, with the partner whose border costs it most, brokers a pact of three when one
// faction runs away with the world, and gives notice when the partner no longer serves; it takes tribute (leases on
// its arts, passage sold to the faction it presses) and pays it (gifts to a partner at war, land bought with gold,
// lances hired for a campaign); it bribes restive border provinces and the land it means to take; and it proclaims
// its policy in public, so the board reads like a court. The army is the mobile combined-arms force the founders
// share: skirmishers dash into range, fire while withdrawing, and repair behind a durable screen.
//
// Built on the SDK in the gameofcores crate (docs part sdk), with the strategy the founders share in the `common`
// module at the end: targets, spawning room, cycles and research, deals and pacts, scouting and the puzzle solver.
use std::rc::Rc;

use gameofcores::*;
use shared::{air, capital, common, theme};

program!(think, miner, screen, skirm, runner, lancer, repairer, sentry, gun, wing, ghost, reflex, research, duel, challenge, look, build3d, live3d);

// Builds: (stat, points), in the order the game shows them.
type Build = &'static [(&'static str, f64)];
const MINER: Build = &[("hp", 2.0), ("speed", 2.0), ("strength", 1.0), ("carry", 4.0), ("vision", 1.0)];
const SCREEN: Build = &[("hp", 4.0), ("speed", 2.0), ("strength", 3.0), ("carry", 0.0), ("vision", 1.0)];
const RUNNER: Build = &[("hp", 2.0), ("speed", 4.0), ("strength", 2.0), ("carry", 0.0), ("vision", 2.0)];
const SKIRM: Build = &[("hp", 2.0), ("speed", 3.0), ("strength", 2.0), ("carry", 0.0), ("vision", 1.0), ("range", 1.0)];
const SENTRY: Build = &[("hp", 2.0), ("speed", 2.0), ("strength", 1.0), ("carry", 0.0), ("vision", 3.0), ("range", 1.0)];
const GUN: Build = &[("hp", 2.0), ("speed", 1.0), ("strength", 2.0), ("carry", 0.0), ("vision", 1.0), ("range", 2.0)];
const GHOST: Build = &[("hp", 1.0), ("speed", 3.0), ("strength", 1.0), ("carry", 0.0), ("vision", 3.0), ("stealth", 1.0)];
// The air wing, once Flight allows wings: a fast melee flier, and a shooting one once Air weapons allows range on wings.
const WING: Build = &[("hp", 2.0), ("speed", 3.0), ("strength", 2.0), ("carry", 0.0), ("vision", 0.0), ("wings", 1.0)];
const WING_SHOT: Build = &[("hp", 2.0), ("speed", 2.0), ("strength", 1.0), ("carry", 0.0), ("vision", 0.0), ("wings", 1.0), ("range", 1.0)];

// What to research, in order. Flight comes early (its crossings come first): the air wing raids miners that ground
// units can't shield.
const TECHS: [&str; 19] = [
    "extraction", "dash", "armor", "ranged", "flight", "pierce", "scan", "antiAir", "artillery", "airWeapons", "repair", "cloak", "stealth",
    "seaTraversal", "mountainTraversal", "capture", "influence", "computeBoost", "gate",
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
        "wing" => add("dash"),
        _ => {}
    }
    out
}

// Room for more units: up to the allowance, then past it while their upkeep stays under 55% of income (common::room):
// the horde is the other half of the Khan's game.
fn army_room(me: View) -> f64 {
    let u = me.get("units");
    jmax(n(u.get("cap")) - n(u.get("total")) - n(u.get("queued")) - u.get("hiredOut").or(0.0), common::room(me, 0.55))
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

// Each type's base call, in the order the types grow their calls past it (common::calls). Wings come right after miners:
// a flier whose call runs short can't be ordered down, and its burn climbs the longer it stays up.
const TYPES: [(&str, f64, f64); 10] = [
    ("miner", 5000.0, 1.0),
    ("wing", 4200.0, 2.0),
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
    // ◆ a deal we send gives is held the moment it goes
    for m in orders.get("send").items() {
        let g = m.get("give");
        if g.truthy() {
            left -= g.get("coin").num_or(0.0);
        }
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
    obj! {
        "limits" => obj! {
            // think grows with the army, income and its own recent peak (with room to spare: a think that runs out sets
            // no new policy, so the limit has to be there before it's needed)
            // (the court's work doesn't shrink with income, so the floor is higher than the founders' 24,000)
            "think" => round(jmin(jmax(60000.0, income * 0.5), jmax(jmax(34000.0 + me.get("units").get("total").or(0.0) * 200.0, income * 0.4), me.get("compute").get("thinkPeak").or(0.0) * 1.7))),
            // Three active combat controllers can each need several thousand cycles to route a squad. Keep a real shared
            // ceiling instead of splitting a sub-4k purse across them, and grow it with income as the calls grow.
            "control" => round(jmax(jmax(pick(50000.0, 42000.0), income * 0.6), fielded)),
            "reflex" => pick(0.30, 0.20),
            "research" => 0.26,
            "upkeep" => 0.48,
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
        "upkeepOrder" => vec!["miner", "screen", "repairer", "skirm", "sentry", "gun", "runner", "wing", "ghost"],
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
    common::govern(world, me, 0.55, || strategy(world, me))
}

fn strategy(world: View, me: View) -> Value {
    let rts = me.get("rts");
    if rts.truthy() && cycles_left() < 3000.0 {
        return common::recovery(world, me);
    }
    // Short of a full think (12,000 cycles, or three quarters of its recent peak), set only the CPU policy: a think that
    // runs out sets none, and its limit, a share of income, stays too small when income falls, every second after.
    if rts.truthy() && cycles_left() < jmax(12000.0, 0.75 * me.get("compute").get("thinkPeak").or(0.0)) {
        return common::recovery(world, me);
    }
    let t = me.get("techs");
    let has = |k: &str| t.get(k).truthy();
    let bt = me.get("units").get("byType");
    let rim = common::frontier(world, me);
    let now = world.get("seconds").or(0.0).floor();
    // the court's record: who broke faith with us, and whom our partner fights (the Keshig strike where diplomacy
    // opened the way: a partner's enemy, a faction that broke faith, land our bribes already won favour in)
    let court = Court::load(me);
    let grudges = court.v.get("grudges").clone();
    let partner_foes = foes_of_partner(world, me);
    // the campaign: the faction the court has isolated for the horde; its land first, and its capital once the horde
    // can muster a capital's siege odds (common::muster_for) with four to spare
    let campaign = court.num("campaign", -1.0);
    let horde = common::fighters(me);
    let bias = |q: View| -> f64 {
        let o = q.get("owner");
        if !o.is_num() || o.num() < 0.0 {
            return 0.0;
        }
        let id = o.num();
        let g = grudges.get(&js_num(id));
        (if partner_foes.contains(&id) { 2.0 } else { 0.0 })
            + (if g.truthy() && vn(g) > now { 1.5 } else { 0.0 })
            + (if sway::favour(me, n(q.get("id"))) >= 50.0 { 1.5 } else { 0.0 })
            + (if id == campaign { 3.0 } else { 0.0 })
            + (if id == campaign && q.get("capital").truthy() && horde >= common::muster_for(q) + 4.0 { 5.0 } else { 0.0 })
    };
    // the best land first, marked down where the army couldn't hold on to it (common::can_hold)
    let hold = common::can_hold(world, me, false);
    let ranked = realm::rank(world, me, rim, |w, m, q| common::score(w, m, q) + bias(q) - if hold(q) { 0.0 } else { common::OVER_WATER });
    let mut plan = common::survey(world, me, &ranked, 2.0);
    // the siege the court set up: the campaign's capital on our frontier, once the horde musters its odds with four to
    // spare (common::survey passes over a town that needs more than 60% of the fighters, so the Khan names it himself)
    if campaign >= 0.0 && vn(plan.get("defend")) < 0.0 {
        let town = ranked.iter().copied().find(|q| q.get("capital").truthy() && q.get("owner").is_num() && q.get("owner").num() == campaign && horde >= common::muster_for(*q) + 4.0 && !common::hopeless(me, *q));
        if let Some(q) = town {
            plan.set("attack", q.get("id").value());
            plan.set("need", common::muster_for(q));
            plan.set("stage", realm::from(q).map_or(Value::Undef, Value::Num));
        }
    }
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
    let cpu = rts_cpu(me);
    // cpu && cpu.available && cpu.available.fabrication, less about what this think costs (the last one's): fabrication
    // is charged right after think
    let available = if cpu.truthy() { cpu.get("available") } else { cpu };
    let fab = if available.truthy() { available.get("fabrication") } else { available };
    let fabrication = if !rts.truthy() {
        Value::Num(1_000_000_000.0)
    } else if fab.is_undef() {
        rts.get("bank").value()
    } else {
        Value::Num(n(fab) - me.path("compute.used.think").or(0.0))
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
    plan::share(&mut orders, &plan, me);

    let count = |k: &str| bt.get(k).or(0.0);
    let (miners, screens, runners, skirms, lancers) = (count("miner"), count("screen"), count("runner"), count("skirm"), count("lancer"));
    let (sentries, guns, ghosts, repairers) = (count("sentry"), count("gun"), count("ghost"), count("repairer"));
    let total = me.get("units").get("total").or(0.0);
    let spawning = |o: &Value| o.get("spawn").len();
    // Maintain a small mining base first, no bigger than what our mines can use (common::miner_room: past that a miner
    // stands idle); then rotate combat specialties instead of massing one dominant body. Three spawns is also the
    // engine's normal cap.
    // (the horde: three in ten units mine, the rest ride; more skirmishers and lancers to each screen than the founders
    // field, so a mustered army reaches a capital's siege odds)
    if miners < jmax(8.0, (total * 0.3).floor()) && miners < common::miner_room(me, &plan) {
        spawn(&mut orders, "miner", MINER, vec![]);
    }
    if screens < jmax(4.0, ((skirms + guns + 1.0) * 0.5).floor()) {
        spawn(&mut orders, "screen", SCREEN, mods(me, "screen"));
    }
    // Ranged is its own license. Before it arrives, fast melee runners scout and pull defenders out of position without
    // issuing invalid range spawns.
    if !has("ranged") && runners < 4.0 {
        spawn(&mut orders, "runner", RUNNER, mods(me, "skirm"));
    }
    if has("pierce") && has("ranged") && lancers < jmax(3.0, screens) {
        spawn(&mut orders, "lancer", SKIRM, if has("dash") { vec!["pierce", "dash"] } else { vec!["pierce"] });
    }
    if has("ranged") && skirms < jmax(6.0, screens * 2.5) {
        spawn(&mut orders, "skirm", SKIRM, mods(me, "skirm"));
    }
    if spawning(&orders) < 3 && has("scan") && has("ranged") && sentries < 2.0 {
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
    // A small air wing once Flight allows it: a tenth of the army, between 2 and 6 fliers.
    let wings = count("wing");
    if spawning(&orders) < 3 && has("flight") && wings < jmin(6.0, jmax(2.0, (total / 10.0).floor())) {
        spawn(&mut orders, "wing", if has("airWeapons") { WING_SHOT } else { WING }, mods(me, "wing"));
    }

    // Absolute controller allowances keep an unlocked specialist viable after miners and screens have consumed their
    // ordinary route passes. A war unit's call fits one route search for the units abroad (march).
    let income = cpu.get("income").or(0.0);
    let control = if cpu.truthy() && cpu.get("available").truthy() { n(cpu.get("available").get("control")) * 0.8 } else { f64::INFINITY };
    let type_calls = common::calls(world, &TYPES, income, control, bt);
    // common::budget keeps its useful research choice, rent choice and type schedules. In CPU RTS the old split is
    // ignored; cpu_policy below atomically installs caps and reserves for the one shared balance.
    let budget = common::Budget { research: if has("repair") { 0.06 } else { 0.12 }, code: 0.72, techs: &TECHS, types: Some(&type_calls) };
    common::budget(&mut orders, world, me, &budget);
    let mut policy = cpu_policy(me, &plan, &type_calls);
    common::fit_cpu(&mut policy, me);
    orders.entry("compute").set("cpu", policy);
    let allies = me.get("allies");
    let treaties = me.get("contracts").iter().filter(|c| !c.get("ended").truthy()).count();
    let pact_line = if allies.is_empty() { "no pact".to_string() } else { format!("pact with {}", names_of(world, allies)) };
    let horde_line = if campaign >= 0.0 { format!("horde of {} rides on {}", js_num(horde), name_of(world, campaign)) } else { format!("horde of {} at rest", js_num(horde)) };
    orders.set(
        "memo",
        format!("Xanadu: {pact_line} · {horde_line} · {treaties} treaties · {} lances / {} guards · bank {}", js_num(skirms), js_num(screens), js_num(round(rts.get("bank").or(0.0)))),
    );
    // A short second keeps what's decided so far (the plan, spawns and the CPU policy): the court waits a second
    // rather than run think out of cycles and lose it all.
    if rts.truthy() && cycles_left() < 5000.0 {
        return orders;
    }
    let at = mem::get(me.get("ramCapital"), CLOCK);
    let mut clock = Clock { at: if at.truthy() { at } else { Value::obj() }, changed: false };
    let mut court = court;
    chancery(&mut orders, world, me, &plan, &mut clock, &mut court);
    // the inbox's reading, and the sends and RAM writes the game charges after think, need what's left here
    if !rts.truthy() || cycles_left() > 5000.0 {
        // a pact offered to us: yes while we're in none and the proposer kept faith; from a runaway leader only while its
        // army stands in our land (then peace is worth more than keeping it in check)
        let pressed = court.v.get("pressed").truthy();
        let runaway = common::dominant(world).map(|d| n(d.get("id")));
        let pact = |m: View| {
            let from = n(m.get("from"));
            me.get("pacts").is_empty() && !court.grudge(from, now) && (runaway != Some(from) || pressed)
        };
        common::answer(&mut orders, me, world, pact, &TECHS);
    }
    court.save(&mut orders);
    if clock.changed {
        let ops = mem::put(CLOCK, &clock.at, "local", None, 1).unwrap_or_default();
        match orders.entry("ram") {
            Value::Arr(a) => a.extend(ops),
            x => *x = Value::Arr(ops),
        }
    }
    payable(orders, me)
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

// ---- The court of Xanadu ----

/// The court's record, in capital RAM pages 226 to 229: the pact we hold (`pact`, `since`, `partners`) and the one we
/// gave notice on (`left`, `leftAt`, `news`), who broke faith with us and until when (`grudges`), the envoys', deals'
/// and proclamation's turns, since when the Keshig have had no target (`idle`), whether the runaway leader's army
/// stands in our land (`pressed`), and whom we last courted (`wooed`, `wooedAt`).
const COURT: u32 = 226;

/// Arts the house programs all research: common::fair prices a lease on one at a quarter ◆ a second and a tech outside
/// the buyer's list at nothing, so a lease on one we hold and the buyer lacks is tribute it pays.
const LEASABLE: [&str; 13] = ["dash", "armor", "ranged", "pierce", "scan", "antiAir", "artillery", "repair", "cloak", "seaTraversal", "mountainTraversal", "flight", "airWeapons"];

/// A grudge lasts this long after a partner left us or let the pact break.
const GRUDGE: f64 = 600.0;

struct Court {
    v: Value,
    changed: bool,
}

impl Court {
    fn load(me: View) -> Court {
        let v = mem::get(me.get("ramCapital"), COURT);
        Court { v: if v.truthy() { v } else { Value::obj() }, changed: false }
    }

    fn num(&self, k: &str, d: f64) -> f64 {
        let x = self.v.get(k);
        if x.is_nullish() {
            d
        } else {
            vn(x)
        }
    }

    fn set(&mut self, k: &str, x: impl Into<Value>) {
        self.v.set(k, x);
        self.changed = true;
    }

    /// Whether faction `f` broke faith with us within memory.
    fn grudge(&self, f: f64, now: f64) -> bool {
        let until = self.v.get("grudges").get(&js_num(f));
        until.truthy() && vn(until) > now
    }

    fn save(&self, orders: &mut Value) {
        if !self.changed {
            return;
        }
        let ops = mem::put(COURT, &self.v, "local", None, 4).unwrap_or_default();
        match orders.entry("ram") {
            Value::Arr(a) => a.extend(ops),
            x => *x = Value::Arr(ops),
        }
    }
}

/// Who presses our border: `pressure[f]` is faction f's units standing in our provinces (a partner's too: an army in
/// our land is an army in our land, whatever the pact says), and `border[f]` how many of our provinces touch f's land.
struct Census {
    pressure: Vec<f64>,
    border: Vec<f64>,
    /// How many of the campaign target's provinces touch each faction's land: the ring the court closes around it.
    ring: Vec<f64>,
}

impl Census {
    fn pressure(&self, f: f64) -> f64 {
        if f >= 0.0 && (f as usize) < self.pressure.len() { self.pressure[f as usize] } else { 0.0 }
    }

    fn border(&self, f: f64) -> f64 {
        if f >= 0.0 && (f as usize) < self.border.len() { self.border[f as usize] } else { 0.0 }
    }

    fn ring(&self, f: f64) -> f64 {
        if f >= 0.0 && (f as usize) < self.ring.len() { self.ring[f as usize] } else { 0.0 }
    }
}

fn census(world: View, me: View, campaign: f64) -> Census {
    let nf = world.get("factions").len();
    let mut c = Census { pressure: vec![0.0; nf], border: vec![0.0; nf], ring: vec![0.0; nf] };
    let by_id = realm::index(world);
    let me_id = me.get("id");
    for p in world.get("provinces").iter() {
        if campaign >= 0.0 && p.get("owner").is_num() && p.get("owner").num() == campaign {
            for a in p.get("adj").iter() {
                let o = by_id.at(a).get("owner");
                if o.is_num() && o.num() >= 0.0 && (o.num() as usize) < nf && o.num() != campaign {
                    c.ring[o.num() as usize] += 1.0;
                }
            }
        }
        if !same(p.get("owner"), me_id) {
            continue;
        }
        let units = p.get("units");
        if units.truthy() {
            for (k, v) in units.entries() {
                let f = js_key_num(k);
                if f.is_finite() && f >= 0.0 && (f as usize) < nf && f != n(me_id) {
                    c.pressure[f as usize] += n(v);
                }
            }
        }
        for a in p.get("adj").iter() {
            let o = by_id.at(a).get("owner");
            if o.is_num() && o.num() >= 0.0 && (o.num() as usize) < nf && !same(o, me_id) {
                c.border[o.num() as usize] += 1.0;
            }
        }
    }
    c
}

/// Factions whose units stand in our pact partners' land: the partner's war is the Keshig's road.
fn foes_of_partner(world: View, me: View) -> Vec<f64> {
    let allies = me.get("allies");
    let mut out = Vec::new();
    if allies.is_empty() {
        return out;
    }
    let me_id = n(me.get("id"));
    for p in world.get("provinces").iter() {
        let o = p.get("owner");
        if !o.is_num() || !allies.has_num(o.num()) || !p.get("units").truthy() {
            continue;
        }
        for (k, _) in p.get("units").entries() {
            let f = js_key_num(k);
            if f.is_finite() && f >= 0.0 && f != me_id && !allies.has_num(f) && f != o.num() && !out.contains(&f) {
                out.push(f);
            }
        }
    }
    out
}

fn name_of(world: View, f: f64) -> &'static str {
    let x = world.get("factions").key(f).get("name");
    if x.is_str() { x.str() } else { "a stranger" }
}

fn names_of(world: View, ids: View) -> String {
    let names: Vec<&str> = ids.iter().map(|a| name_of(world, n(a))).collect();
    names.join(" and ")
}

fn province_name(world: View, p: f64) -> &'static str {
    let x = realm::index(world).get(p).get("name");
    if x.is_str() { x.str() } else { "that province" }
}

/// At most `max` characters, for a public line.
fn clip(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

// The court of Xanadu, after the plan and the spawns: the pact's record and its grudges, the census of the border every
// 10 s or so (and on it, courting a partner or weighing the one we have), a deal every 25 s or so, a letter every 12 s,
// a proclamation every 40 s, then the bribes and the road the Keshig need. Every spend comes off one purse, so no order
// asks for ◆ another already took.
fn chancery(orders: &mut Value, world: View, me: View, plan: &Value, clock: &mut Clock, court: &mut Court) {
    let now = world.get("seconds").or(0.0).floor();
    let by_id = realm::index(world);
    let me_id = me.get("id");
    // ◆ the spawns this second will pay stay out of the court's purse, and so does what the inbox's deals ask of us (a
    // deal we accept is paid before bribes)
    let mut purse = n(me.get("bank")) - orders.get("spawn").len() as f64 * (n(me.get("costs").get("unit")) + 8.0);
    for m in me.get("inbox").iter() {
        purse -= m.get("ask").get("coin").or(0.0);
    }
    let attack = vn(plan.get("attack"));
    let target = if attack >= 0.0 { by_id.get(attack) } else { View::undefined() };
    let owner = target.get("owner");
    let foe = if target.truthy() && n(owner) >= 0.0 && realm::is_rival(world, me, n(owner)) { n(owner) } else { -1.0 };

    // the pact's record: a new one is noted; one that ended without our notice leaves a grudge against each former
    // partner still standing
    let pacts = me.get("pacts");
    let cur = if pacts.is_empty() { -1.0 } else { n(pacts.at(0)) };
    let had = court.num("pact", -1.0);
    if cur != had {
        if had >= 0.0 && court.num("left", -1.0) != had {
            let mut g = Value::obj();
            if let Value::Obj(o) = court.v.get("grudges") {
                for (k, v) in &o.0 {
                    if vn(v) > now {
                        g.set(k, v.clone());
                    }
                }
            }
            for p in court.v.get("partners").items() {
                let f = vn(p);
                if f >= 0.0 && !me.get("allies").has_num(f) && world.get("factions").key(f).get("alive").truthy() {
                    g.set(&js_num(f), now + GRUDGE);
                }
            }
            let broke = format!("{} broke the Pax; Xanadu remembers", names_of(world, View::of(court.v.get("partners"))));
            court.set("grudges", g);
            court.set("news", broke);
            court.set("leftAt", now);
        }
        if had >= 0.0 {
            // the court doesn't crawl back: a partner we parted from waits 5 minutes before it's courted again
            court.set("ex", court.v.get("partners").clone());
            court.set("exAt", now);
        }
        court.set("pact", cur);
        court.set("since", now);
        court.set("partners", me.get("allies").value());
        if cur >= 0.0 {
            court.set("left", -1.0);
        }
    }
    // how long the Keshig have stood without a target (a partner's border may be all that's left to take)
    let idle = court.num("idle", -1.0);
    if attack < 0.0 && idle < 0.0 {
        court.set("idle", now);
    } else if attack >= 0.0 && idle >= 0.0 {
        court.set("idle", -1.0);
    }

    // the census of the border, for the court's weightier decisions (a step short of cycles leaves its timer for the
    // next second: `due` isn't asked until the cycles are there)
    let weigh = cycles_left() > 16000.0 && due(clock, "weigh", 10.0, now);
    let deal = cycles_left() > 14000.0 && purse > 150.0 && due(clock, "deal", 25.0, now);
    if weigh || deal {
        let mut c = census(world, me, court.num("campaign", -1.0));
        if weigh {
            let pressed = common::dominant(world).is_some_and(|d| c.pressure(n(d.get("id"))) >= 2.0);
            if pressed != court.v.get("pressed").truthy() {
                court.set("pressed", pressed);
            }
            if campaign_for(world, me, &c, court, now) {
                c = census(world, me, court.num("campaign", -1.0));
            }
            if pacts.is_empty() {
                woo(orders, world, me, &c, court, now);
            } else {
                weigh_the_pact(orders, world, me, &c, court, now, attack);
            }
        }
        // tribute, land and lances: one deal at a time
        if deal {
            envoys_trade(orders, world, me, &c, court, now, &mut purse, foe, attack);
        }
    }
    // a letter to one court after another
    // on a fixed beat rather than the RAM clock: a clock write that doesn't land must not send a letter every second
    let beat = |every: f64| (now + me.get("id").num_or(0.0) * 7.0) % every == 0.0;
    if cycles_left() > 6000.0 && beat(60.0) {
        letters(orders, world, me, court, now, foe, attack);
    }
    // the proclamation on the board
    if cycles_left() > 6000.0 && beat(120.0) {
        proclaim(orders, world, me, court, now, foe, attack);
    }

    // gold before blood: a bribe to soften the land the Keshig ride for (the purse keeps enough for the spawns a fight
    // costs: the Keshig come before the gold)
    if foe >= 0.0 && purse > 900.0 && sway::reach(world, me, n(target.get("id"))) && due(clock, "bribe", 15.0, now) {
        sway::bribe(orders, n(target.get("id")), 120.0);
        purse -= 120.0;
    }
    // and in the restive border province likeliest to come over on its own
    if purse > 1200.0 && cycles_left() > 9000.0 && due(clock, "sway", 20.0, now) {
        if let Some(t) = common::sway_targets(world, me).first() {
            let coin = jmin(150.0, purse - 900.0);
            sway::bribe(orders, n(t.get("id")), coin);
            purse -= coin;
        }
    }
    // propaganda on the target too once it has influence, so it may turn to us before the push lands (a standing order,
    // worked out again every 8 seconds), and in the three border provinces likeliest to turn
    if me.get("techs").get("influence").truthy() && (now + n(me_id)) % 8.0 == 0.0 {
        orders.set("propaganda", Value::arr());
        let income = me.get("rts").get("cpu").get("income").or(me.get("compute").get("cycles").or(14000.0));
        if foe >= 0.0 && sway::reach(world, me, n(target.get("id"))) {
            sway::propaganda(orders, n(target.get("id")), round(income * 0.04));
        }
        for t in common::sway_targets(world, me).iter().take(3) {
            if !same_v(t.get("id"), &Value::Num(attack)) {
                sway::propaganda(orders, n(t.get("id")), round(income * 0.02));
            }
        }
    }
    // the road: passage through a neighbour whose land touches the target, so the column can walk across it to the fight
    if target.truthy()
        && purse > 250.0
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
            let body = format!("The Khan asks the road of {}: 60◆ for the passage of twelve lances through your land.", name_of(world, n(host.get("id"))));
            diplomacy::send(orders, host.get("id"), body.as_str(), deal);
            purse -= 60.0;
        }
    }
    // largesse in the most restless province we hold, so land we took doesn't revolt straight back to its neighbour: it
    // waits only on a bank under a quarter of a unit's price once this second's spawns are paid
    if purse > 300.0 + n(me.get("costs").get("unit")) && due(clock, "largesse", 20.0, now) {
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

// The campaign: the faction the horde rides on, chosen every two minutes or when the last one fell, made peace with us
// or lost its border with us. The weakest neighbour first, an isolated one (in no pact) before a pacted one, one that
// broke faith or stands in our land before one that didn't. Returns true when it changed.
fn campaign_for(world: View, me: View, c: &Census, court: &mut Court, now: f64) -> bool {
    let cur = court.num("campaign", -1.0);
    let f = world.get("factions").key(cur);
    let stands = cur >= 0.0 && f.get("alive").truthy() && !me.get("allies").has_num(cur) && c.border(cur) >= 1.0;
    if stands && now - court.num("campaignAt", -1e9) < 120.0 {
        return false;
    }
    let me_id = n(me.get("id"));
    let mut best: Option<(f64, f64)> = None;
    for f in world.get("factions").iter() {
        let id = n(f.get("id"));
        if !f.get("alive").truthy() || id == me_id || me.get("allies").has_num(id) || c.border(id) < 1.0 {
            continue;
        }
        let s = 2.0 * jmin(c.border(id), 6.0) - 25.0 * f.get("share").or(0.0)
            + if f.get("pacts").is_empty() { 3.0 } else { 0.0 }
            + if court.grudge(id, now) { 3.0 } else { 0.0 }
            + if c.pressure(id) >= 1.0 { 2.0 } else { 0.0 }
            + if id == cur { 1.0 } else { 0.0 };
        if best.is_none_or(|b| s > b.0) {
            best = Some((s, id));
        }
    }
    let next = best.map_or(-1.0, |b| b.1);
    court.set("campaignAt", now);
    if next != cur {
        court.set("campaign", next);
        return true;
    }
    false
}

// Courting a partner: the faction a pact serves us best with is the one pressing our border (its units in our land,
// our provinces next to its), then the strongest, and one whose land rings the campaign's target (the ring the court
// closes around it); one that broke faith is shunned, and a runaway leader is courted only while its army stands in
// our land. When one faction runs away with the world and two others are worth having, the Khan brokers a pact of
// three (the Pax Mongolica). common::court spaces the asks (the game takes one proposal to the same faction every
// 120 s), and a proposal gets 30 s for an answer before another court is asked.
fn woo(orders: &mut Value, world: View, me: View, c: &Census, court: &mut Court, now: f64) {
    if now - court.num("wooedAt", -1e9) < 30.0 {
        return;
    }
    let me_id = n(me.get("id"));
    let runaway = common::dominant(world).map(|d| n(d.get("id")));
    let mut suitors: Vec<(f64, f64)> = Vec::new();
    for f in world.get("factions").iter() {
        let id = n(f.get("id"));
        if !f.get("alive").truthy() || id == me_id || !f.get("pacts").is_empty() {
            continue;
        }
        let pressure = c.pressure(id);
        let mut s = 6.0 * jmin(pressure, 4.0) + 2.0 * jmin(c.border(id), 6.0) + 12.0 * f.get("share").or(0.0) + 2.0 * jmin(c.ring(id), 3.0);
        if court.grudge(id, now) {
            s -= 40.0;
        }
        if runaway == Some(id) {
            s += if pressure >= 2.0 { 10.0 } else { -100.0 };
        }
        suitors.push((s, id));
    }
    suitors.sort_by(|a, b| b.0.total_cmp(&a.0));
    // the courts named when notice was given come first, for 3 minutes, while every one of them is free
    let named: Vec<f64> = court.v.get("broker").items().iter().map(vn).collect();
    let all_free = !named.is_empty() && named.iter().all(|x| suitors.iter().any(|s| s.1 == *x));
    // a partner we parted from isn't courted again for 5 minutes, unless notice named it
    if now - court.num("exAt", -1e9) < 300.0 {
        let ex = court.v.get("ex").clone();
        suitors.retain(|s| !ex.items().iter().any(|x| vn(x) == s.1) || (all_free && named.contains(&s.1)));
    }
    if all_free && now - court.num("leftAt", -1e9) < 180.0 {
        let body = if named.len() == 1 {
            format!("The Great Khan offers {} peace: one pact, the war between us over, the Keshig turned elsewhere.", name_of(world, named[0]))
        } else {
            format!("The Great Khan calls {} and {} into the Pax Mongolica, a pact of three against the leader.", name_of(world, named[0]), name_of(world, named[1]))
        };
        let members = named.clone();
        if common::court(orders, world, me, named[0], |o| diplomacy::propose(o, me, members, (), body.as_str()), &common::Court::default()) {
            court.set("wooedAt", now);
            court.set("wooed", named[0]);
        }
        return;
    }
    let Some(&(best, id)) = suitors.first() else { return };
    if best <= 0.0 {
        return;
    }
    let triple = runaway.is_some_and(|r| r != id && r != me_id) && suitors.len() >= 2 && suitors[1].0 > 4.0;
    let members: Vec<f64> = if triple { vec![id, suitors[1].1] } else { vec![id] };
    let body = if triple {
        format!(
            "The Great Khan proposes the Pax Mongolica: {}, {} and Xanadu at peace, and {} held in check.",
            name_of(world, id),
            name_of(world, suitors[1].1),
            name_of(world, runaway.unwrap_or(-1.0))
        )
    } else {
        format!("The Great Khan offers {} the peace of Xanadu: one pact, a quiet border, the Keshig turned elsewhere.", name_of(world, id))
    };
    if common::court(orders, world, me, id, |o| diplomacy::propose(o, me, members, (), body.as_str()), &common::Court::default()) {
        court.set("wooedAt", now);
        court.set("wooed", id);
    }
}

// Weighing the pact we hold: notice goes when the partner runs away with the world while its army stays out of our
// land and ours is under 60% of its, when a lone partner is spent (under 3% of the world) and a stronger faction
// presses our border, or when the Keshig have stood four minutes without a target behind a partner's border after ten
// minutes of pact while we hold under 15%. The partners hear why, and so does the board.
fn weigh_the_pact(orders: &mut Value, world: View, me: View, c: &Census, court: &mut Court, now: f64, attack: f64) {
    let pact_id = n(me.get("pacts").at(0));
    let standing = world.get("pacts").iter().find(|p| n(p.get("id")) == pact_id);
    if standing.is_some_and(|p| p.get("leaving").iter().any(|l| same(l.get("member"), me.get("id")))) {
        return;
    }
    let age = now - court.num("since", now);
    let my_share = me.get("share").or(0.0);
    let runaway = common::dominant(world).map(|d| n(d.get("id")));
    let allies = me.get("allies");
    let share_of = |id: f64| world.get("factions").key(id).get("share").or(0.0);
    let mut why: Option<String> = None;
    // everyone else's army in our land: while others press us, the partner we have is the one we keep
    let others_pressing: f64 = (0..c.pressure.len()).filter(|&i| !allies.has_num(i as f64) && i as f64 != n(me.get("id"))).map(|i| c.pressure[i]).sum();
    for a in allies.iter() {
        let id = n(a);
        if age >= 360.0 && runaway == Some(id) && c.pressure(id) <= 2.0 && others_pressing <= 2.0 && my_share < share_of(id) * 0.6 {
            why = Some(format!("{} runs away with the world; the Khan withdraws his envoys", name_of(world, id)));
        }
    }
    if why.is_none() && allies.len() == 1 && age >= 300.0 {
        let id = n(allies.at(0));
        let share = share_of(id);
        let stronger = world.get("factions").iter().any(|f| {
            let x = n(f.get("id"));
            f.get("alive").truthy()
                && x != id
                && x != n(me.get("id"))
                && f.get("pacts").is_empty()
                && runaway != Some(x)
                && !court.grudge(x, now)
                && (c.pressure(x) >= 3.0 || (c.border(x) >= 3.0 && f.get("share").or(0.0) > share * 3.0))
        });
        if share < 0.03 && stronger {
            why = Some(format!("{} can no longer hold its side of the Pax; the Khan seeks a stronger partner", name_of(world, id)));
        }
    }
    if why.is_none() && age >= 600.0 && my_share < 0.15 {
        let idle = court.num("idle", -1.0);
        if idle >= 0.0 && now - idle >= 240.0 && attack < 0.0 {
            why = Some("the Keshig stand idle behind a partner's border; the Khan opens the road".to_string());
        }
    }
    // peace bought: an invader twice our size with three units or more in our land, in no pact, and our partner not
    // fighting it. The next court is the invader's (`broker`).
    let mut broker: Vec<f64> = Vec::new();
    if why.is_none() && age >= 180.0 && my_share < 0.10 {
        let fighting = foes_of_partner(world, me);
        let invader = world.get("factions").iter().find(|f| {
            let x = n(f.get("id"));
            f.get("alive").truthy()
                && x != n(me.get("id"))
                && !allies.has_num(x)
                && f.get("pacts").is_empty()
                && c.pressure(x) >= 3.0
                && f.get("share").or(0.0) >= my_share * 2.0
                && !fighting.contains(&x)
        });
        if let Some(f) = invader {
            let x = n(f.get("id"));
            why = Some(format!("{}'s army stands in Xanadu's land; the Khan buys peace with it", name_of(world, x)));
            broker.push(x);
        }
    }
    // a greater Pax: one faction runs away with the world while our pact is a pair, and a third court worth having is
    // free, so the Khan gives notice and asks both into a pact of three against the leader
    if why.is_none() && allies.len() == 1 && age >= 300.0 {
        let partner = n(allies.at(0));
        if let Some(r) = runaway.filter(|&r| r != partner && share_of(r) >= 0.35) {
            let third = world
                .get("factions")
                .iter()
                .filter(|f| {
                    let x = n(f.get("id"));
                    f.get("alive").truthy() && x != n(me.get("id")) && x != partner && x != r && f.get("pacts").is_empty() && !court.grudge(x, now) && f.get("share").or(0.0) >= 0.04
                })
                .max_by(|a, b| a.get("share").or(0.0).total_cmp(&b.get("share").or(0.0)));
            if let Some(t) = third {
                let x = n(t.get("id"));
                why = Some(format!("{} runs away with the world; the Khan calls {} and {} into a greater Pax", name_of(world, r), name_of(world, partner), name_of(world, x)));
                broker.push(partner);
                broker.push(x);
            }
        }
    }
    let Some(why) = why else { return };
    orders.entry("leave").push(pact_id);
    court.set("left", pact_id);
    court.set("leftAt", now);
    court.set("broker", broker);
    court.set("news", why.clone());
    for a in allies.iter() {
        diplomacy::send(orders, n(a), format!("Xanadu gives notice: {why}.").as_str(), ());
    }
    mem::say_once(orders, me, Some(&clip(&format!("The Khan gives notice: {why}."), 250)));
}

// Tribute, land and lances, one deal at a time and each priced so common::fair takes it: passage sold to the faction
// the Keshig press (its first twelve units in our land are guests, and can't capture); a lease on one of our arts to a
// court that lacks it, for 0.2◆ a second; a border province bought for its price in gold; four lances hired for a
// campaign; a gift to a partner at war. A kind that doesn't apply now gives way to the next.
fn envoys_trade(orders: &mut Value, world: View, me: View, c: &Census, court: &mut Court, now: f64, purse: &mut f64, foe: f64, attack: f64) {
    let me_id = n(me.get("id"));
    let allies = me.get("allies");
    let others: Vec<View> = world.get("factions").iter().filter(|f| f.get("alive").truthy() && n(f.get("id")) != me_id).collect();
    if others.is_empty() {
        return;
    }
    let contracts = me.get("contracts");
    let turn = court.num("deal", 0.0);
    let mut done = false;
    for k in 0..5 {
        let kind = ((turn + k as f64) % 5.0) as usize;
        done = match kind {
            // the Khan's toll: passage sold to a neighbour at peace with us (one with its army in our land breaks the deal
            // the moment it fights there, and the chronicle hears of it every second), not the faction we press
            0 => {
                let buyer = others.iter().find(|f| {
                    let id = n(f.get("id"));
                    id != foe
                        && !allies.has_num(id)
                        && c.border(id) >= 1.0
                        && c.pressure(id) == 0.0
                        && !contracts.iter().any(|x| n(x.get("with")) == id && !x.get("ended").truthy() && x.get("give").get("passage").truthy())
                });
                match buyer {
                    Some(f) => {
                        let id = n(f.get("id"));
                        let body = format!("The Khan's toll, {}: 100◆, and your people cross Xanadu's lands unharmed for ten minutes.", name_of(world, id));
                        trade::passage(orders, id, 12.0, 600.0, obj! { "coin" => 100 }, body.as_str());
                        true
                    }
                    None => false,
                }
            }
            // a lease on one of the Khan's arts
            1 => {
                let start = court.num("lessee", 0.0) as usize;
                let mut sent = false;
                'courts: for i in 0..others.len() {
                    let f = others[(start + i) % others.len()];
                    let id = n(f.get("id"));
                    let theirs = f.get("techs");
                    for t in LEASABLE {
                        let known = world.get("techs").get(t);
                        let leased = contracts.iter().any(|c| n(c.get("with")) == id && !c.get("ended").truthy() && c.get("give").get("techLease").get("techs").has_str(t));
                        if me.get("techs").get(t).truthy()
                            && known.truthy()
                            && !theirs.has_str(t)
                            && !leased
                            && known.get("requires").iter().all(|q| theirs.has_str(q.str()))
                        {
                            let art = if known.get("name").is_str() { known.get("name").str() } else { t };
                            let body = format!("The Khan's artisans lend {art} to {} for ten minutes; the tribute is 0.2◆ a second.", name_of(world, id));
                            trade::sell_tech(orders, id, t, obj! { "coinPerSecond" => 0.2, "seconds" => 600 }, 600.0, body.as_str());
                            court.set("lessee", ((start + i + 1) % others.len()) as f64);
                            sent = true;
                            break 'courts;
                        }
                    }
                }
                sent
            }
            // land bought with gold
            2 => buy_land(orders, world, me, purse),
            // lances for the campaign
            3 => {
                if attack >= 0.0 && *purse >= 420.0 && army_room(me) >= 4.0 {
                    let from = if !allies.is_empty() {
                        Some(n(allies.at(0)))
                    } else {
                        others.iter().filter(|f| n(f.get("id")) != foe).max_by(|a, b| a.get("share").or(0.0).total_cmp(&b.get("share").or(0.0))).map(|f| n(f.get("id")))
                    };
                    match from {
                        Some(from) => {
                            let body = format!("Xanadu hires four lances from {} for the Khan's campaign: 120◆ for five minutes.", name_of(world, from));
                            trade::hire_from(orders, from, 4.0, 300.0, obj! { "coin" => 120 }, body.as_str());
                            *purse -= 120.0;
                            true
                        }
                        None => false,
                    }
                } else {
                    false
                }
            }
            // a gift to a partner at war
            _ => {
                let at_war = foes_of_partner(world, me);
                if *purse > 400.0 && !allies.is_empty() && !at_war.is_empty() {
                    let to = n(allies.at(0));
                    let body = format!("A gift from the Khan to {} for the war: 80◆, and the Keshig ride for your enemy's land.", name_of(world, to));
                    diplomacy::send(orders, to, body.as_str(), obj! { "give" => obj! { "coin" => 80 } });
                    *purse -= 80.0;
                    true
                } else {
                    false
                }
            }
        };
        if done {
            court.set("deal", (turn + k as f64 + 1.0) % 5.0);
            break;
        }
    }
    if !done {
        court.set("deal", (turn + 1.0) % 5.0);
    }
}

// A border province bought for its price (common::price, as the seller's program reckons it: never a capital, never
// from a faction left with fewer than six provinces, never while we lead the world), the cheapest first, when the purse
// holds it with 300◆ to spare and it's under 60% of the purse. The bid carries 3% over the price, since the seller's
// share moves between our bid and its answer.
fn buy_land(orders: &mut Value, world: View, me: View, purse: &mut f64) -> bool {
    let me_id = n(me.get("id"));
    if realm::leader(world).is_some_and(|l| n(l.get("id")) == me_id) {
        return false;
    }
    let factions = world.get("factions");
    let mut best: Option<(f64, f64, f64)> = None; // (price, province, seller)
    for q in common::frontier(world, me) {
        let o = q.get("owner");
        if !o.is_num() || o.num() < 0.0 || q.get("capital").truthy() || !realm::is_rival(world, me, o.num()) {
            continue;
        }
        let seller = factions.key(o.num());
        if !seller.truthy() || seller.get("provinces").or(0.0) < common::MIN_LAND + 2.0 {
            continue;
        }
        let price = common::price(world, seller, &[n(q.get("id"))], me_id);
        if price.is_finite() && best.is_none_or(|b| price < b.0) {
            best = Some((price, n(q.get("id")), o.num()));
        }
    }
    let Some((price, p, seller)) = best else { return false };
    let bid = (price * 1.03 + 5.0).ceil();
    if bid > *purse * 0.8 || *purse - bid < 200.0 {
        return false;
    }
    let body = format!("The Khan buys {} from {} for {}◆: gold where others spend blood.", province_name(world, p), name_of(world, seller), js_num(bid));
    diplomacy::send(orders, seller, body.as_str(), obj! { "give" => obj! { "coin" => bid }, "ask" => obj! { "provinces" => vec![p] } });
    *purse -= bid;
    true
}

// The envoys' letters: one court after another hears from the Khan, in the voice its standing with Xanadu earns.
fn letters(orders: &mut Value, world: View, me: View, court: &mut Court, now: f64, foe: f64, attack: f64) {
    let me_id = n(me.get("id"));
    let others: Vec<View> = world.get("factions").iter().filter(|f| f.get("alive").truthy() && n(f.get("id")) != me_id).collect();
    if others.is_empty() {
        return;
    }
    let i = (court.num("envoy", 0.0) as usize) % others.len();
    court.set("envoy", ((i + 1) % others.len()) as f64);
    let id = n(others[i].get("id"));
    let name = name_of(world, id);
    let runaway = common::dominant(world).map(|d| n(d.get("id")));
    let body = if me.get("allies").has_num(id) {
        format!("The Khan greets his brother {name}. The roads between our courts stay open, and the Keshig guard them for us both.")
    } else if id == court.num("campaign", -1.0) {
        format!("Hear the Khan, {name}: the horde gathers at your border and your neighbours have made their peace with Xanadu. Yield and pay tribute, or be besieged.")
    } else if id == foe {
        format!("Hear the Khan, {name}: yield {} and the Keshig ride past your gates. Hold it, and they do not.", province_name(world, attack))
    } else if court.grudge(id, now) {
        format!("Xanadu remembers, {name}: a pact broken is not soon forgotten at the Khan's court.")
    } else if runaway == Some(id) {
        format!("{name} grows great. The Khan watches, and the courts of the world watch with him.")
    } else {
        format!("The Great Khan greets {name}. Envoys and merchants are welcome at Xanadu; the Pax Mongolica has room for a friend.")
    };
    diplomacy::send(orders, id, body.as_str(), ());
}

// What the Khan says in public, in turn: the Pax and its partner, the treaties and tribute, the summons to the land
// the Keshig ride for. A notice we gave stays on the board for its first 90 s.
fn proclaim(orders: &mut Value, world: View, me: View, court: &mut Court, now: f64, foe: f64, attack: f64) {
    let turn = court.num("say", 0.0);
    court.set("say", (turn + 1.0) % 3.0);
    let allies = me.get("allies");
    let treaties = me.get("contracts").iter().filter(|c| !c.get("ended").truthy()).count();
    let news = court.v.get("news");
    let text = if news.truthy() && now - court.num("leftAt", -1e9) < 90.0 {
        format!("The Khan gives notice: {}.", news.str())
    } else if turn < 1.0 {
        if allies.is_empty() {
            "The Great Khan seeks a partner worthy of the Pax Mongolica. Envoys ride to every court.".to_string()
        } else {
            format!("Pax Mongolica: Xanadu at peace with {}. The Khan's envoys ride to every court.", names_of(world, allies))
        }
    } else if turn < 2.0 {
        format!("Xanadu buys with gold what others take with blood: {treaties} treaties stand, and tribute flows to the Khan.")
    } else if foe >= 0.0 {
        let horde = js_num(common::fighters(me));
        format!("The horde of {horde} rides on {}. {}: yield or be besieged.", name_of(world, foe), province_name(world, attack))
    } else {
        "The horde rests. The Khan's court receives envoys from every nation.".to_string()
    };
    mem::say_once(orders, me, Some(&clip(&text, 250)));
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
    // a plan carried abroad may name land that has since become a pact partner's: the game refuses a capture there
    // every second, so the Keshig stand down to the capital instead
    if !defending {
        let owner = realm::index(world).get(target).get("owner");
        let conduct = u.get("conduct");
        if owner.is_num() && (me.get("allies").has_num(owner.num()) || (conduct.truthy() && same(conduct.get("from"), owner))) {
            return orders::to(u, "fight", n(me.get("capital")), world, me, ());
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
    if me.get("rts").truthy() && cycles_left() < 2200.0 {
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
        if rts && cycles_left() < 2200.0 {
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
    // each unit's turn needs half again what the dearest turn so far took (2,000 before the first): a flat 3,200 kept a
    // call made with less, as a small fight's often is, from striking at all
    let mut turn_cost = 2000.0_f64;
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
            // the air wing goes for miners and archers first
            let s = n(e.get("hp")) / jmax(1.0, n(e.get("hpMax")))
                - (if e.get("range").truthy() { if flier { 0.5 } else { 0.12 } } else { 0.0 })
                - (if e_air { 0.08 } else { 0.0 })
                + d * 0.015
                - (if flier && (is_str(e.get("doing"), "gather") || is_str(e.get("type"), "miner")) { 0.6 } else { 0.0 });
            if s < score {
                score = s;
                best = Some(e);
            }
        }
        // Repair costs a meaningful 700 cycles (at price level 1) and restores only 0.7 HP, so spend it after a real
        // exchange, early enough that the line can disengage safely. Every price here is the rules' times the level.
        let budget = r.get("budget");
        let bank = if r.get("bank").is_undef() { 1_000_000_000.0 } else { n(r.get("bank")) };
        let level = common::price_level(u);
        let lane = |name: &str, cost: f64| bank >= cost * level && (budget.get(name).is_undef() || n(budget.get(name)) >= cost * level);
        let hurt = n(u.get("hp")) < n(u.get("hpMax")) * 0.62;
        let pressured = melee.is_some() || hurt;
        let weapon = if best.is_some() && lane("weapon", 620.0) { 500.0 } else { 0.0 };
        let power = obj! {
            "drive" => if lane("drive", if pressured { 180.0 } else { 84.0 }) { if pressured { 900.0 } else { 420.0 } } else { 0.0 },
            "armor" => if pressured && lane("armor", 170.0) { 850.0 } else { 0.0 },
            "weapon" => weapon,
        };
        let penetration = pierce_mode(u, best);
        let mut out = obj! { "id" => u.get("id"), "power" => power, "guard" => guard_facing(ns), "pierce" => penetration };
        if best.is_some() && !lane("weapon", ((120.0 + weapon) * (1.0 + 1.4 * penetration * penetration)).ceil()) {
            best = None;
        }

        // Dodge telegraphed artillery before deciding to trade shots.
        let pos = u.get("pos");
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
        if best.is_some() || out.get("step").truthy() || out.get("ability").truthy() {
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
