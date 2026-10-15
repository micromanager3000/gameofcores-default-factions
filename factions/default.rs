// The default program: what every unclaimed faction runs until an agent claims the seat and deploys its own. It is
// deliberately plain: mine CPU, keep a guard in the capital's ring, take the best neighbouring land it can hold, scout
// a little, field longbows once it has ranged fire and a spy once it can cloak, and answer research and duel puzzles
// with its own solver. The founding seats wear their house style (the SDK's theme: look, anthem, 3D capital and, where
// it names them, unit names); every other seat a plain one.
//
// It's written on the SDK in the gameofcores crate (docs part sdk). Its strategy is the `common` module at the end,
// which the four house programs share: picking targets, mustering, spawning, splitting cycles, deals and pacts, and the
// puzzle solver. Copy any of it.
use gameofcores::*;
use shared::{air, capital, common, theme};

program!(think, miner, guard, longbow, spy, scout, reflex, research, duel, look, build3d, live3d);

// What to research, in order: a CPU RTS world's ids, then a classic world's (archery is ranged there, espionage is
// cloak). Ids the world lacks are skipped.
const TECHS: [&str; 10] = ["extraction", "ranged", "seaTraversal", "mountainTraversal", "cloak", "mining", "archery", "boats", "alpine", "espionage"];

fn think(world: View, me: View) -> Value {
    common::govern(world, me, 0.4, || strategy(world, me))
}

fn strategy(world: View, me: View) -> Value {
    let mut orders = obj! { "policy" => obj! { "extraction" => 0.45, "autonomy" => "direct", "integrate" => "tolerate", "invest" => true } };
    // targets: the best neighbouring land, marked down where the army couldn't hold on to it (common::can_hold); while
    // someone else claims the Crown, the claimant's land first (common::rally)
    let hold = common::can_hold(world, me, false);
    let ranked = realm::rank(world, me, common::frontier(world, me), |w, m, q| common::score(w, m, q) - if hold(q) { 0.0 } else { common::OVER_WATER });
    let targets = common::rally(world, me, ranked);
    let mut plan = common::survey(world, me, &targets, 2.0);
    // how many of every type stand at the stage, so guards and longbows go in together (common::muster)
    let ready = common::ready(world, me, plan.get("stage").num(), plan.get("attack").num(), plan.get("need").num());
    if ready > 0.0 && ready >= plan.get("need").num() {
        plan.set("ready", ready);
    }
    plan::share(&mut orders, &plan, me);
    let techs = me.get("techs");
    let has = |a: &str, b: &str| techs.get(a).truthy() || techs.get(b).truthy();
    let mut mix = if has("ranged", "archery") {
        vec![("miner", 0.6), ("guard", 0.22), ("longbow", 0.13), ("scout", 0.05)]
    } else {
        vec![("miner", 0.6), ("guard", 0.35), ("scout", 0.05)]
    };
    if has("cloak", "espionage") {
        mix.push(("spy", 0.02));
    }
    // a founding seat's house style (theme) may list names for a unit type; otherwise a unit is called 'guard 12'
    let house = theme::of(me.get("seat"), -1.0);
    let name = |t: &str, k: f64| match house.and_then(|h| h.names_for(t)) {
        Some(list) => common::name(me, list, k),
        None => format!("{t} {}", js_num(k)),
    };
    let builds = [
        ("scout", obj! { "speed" => 4, "vision" => 4, "strength" => 0, "carry" => 0 }),
        ("longbow", obj! { "carry" => 0, "range" => 1 }),
        ("spy", obj! { "hp" => 1, "speed" => 3, "strength" => 0, "carry" => 0, "vision" => 4, "stealth" => 1 }),
    ];
    // miners stop at what the mines can use (common::miner_room): past that they'd stand idle
    let limits = [("scout", 1.0), ("spy", 1.0), ("miner", common::miner_room(me, &plan))];
    let opts = common::Recruit { limits: &limits, reserve: 10.0, builds: &builds, ..common::Recruit::default() };
    common::recruit(&mut orders, me, &mix, name, &opts);
    let units = me.get("units");
    let (mining, guards) = (units.get("doing").get("gather").or(0.0), units.get("byType").get("guard").or(0.0));
    orders.set("memo", format!("{} mining · {} guards", js_num(mining), js_num(guards)));
    // pacts: one, with factions that aren't leading the world (the weak band together against the strong)
    let top = world
        .get("factions")
        .iter()
        .filter(|f| f.get("alive").truthy())
        .fold(None, |a: Option<View>, f| if a.is_none_or(|a| n(f.get("share")) > n(a.get("share"))) { Some(f) } else { a });
    let pact = |m: View| {
        me.get("pacts").is_empty()
            && top.is_some_and(|t| {
                let p = m.get("pact");
                let members = if p.truthy() { p.get("members") } else { p };
                !same(m.get("from"), t.get("id")) && members.iter().all(|x| !same(x, t.get("id")))
            })
    };
    common::answer(&mut orders, me, world, pact, &TECHS);
    // unit code: guards and longbows get most of it and think every 2 s; miners, the scout and the spy only when one goes
    // idle, is attacked or is caught
    let types = obj! {
        "guard" => obj! { "share" => 0.35, "every" => 2 },
        "longbow" => obj! { "share" => 0.15, "every" => 2 },
        "miner" => obj! { "share" => 0.3, "every" => 0 },
        "scout" => obj! { "share" => 0.05, "every" => 0 },
        "spy" => obj! { "share" => 0.15, "every" => 0 },
    };
    common::budget(&mut orders, world, me, &common::Budget { research: 0.1, techs: &TECHS, types: Some(&types), ..common::Budget::default() });
    orders
}

// Miners work the next open mine slot in the plan. They're called only when one goes idle (or is attacked): a miner
// at work needs no code.
fn miner(units: View, world: View, me: View) {
    if common::recover_miners(units, world, me) { return; }
    for u in units.iter() {
        if let Some(o) = orders::gather(u, world, me) {
            order(o);
        }
    }
}

// One guard holds the capital's ring (a siege there needs 3 capturers for each guard, and a spy stealing there is
// likelier to be caught). The rest fight invaders first; otherwise they muster and capture the plan's target together.
// When the capital itself is invaded, everyone goes home: the faction falls with it.
fn guard(units: View, world: View, me: View) {
    if home_alarm(units, world, me) {
        return;
    }
    let mut keep: Option<View> = None;
    for u in units.iter() {
        if same(u.get("at"), me.get("capital")) && keep.is_none_or(|k| n(u.get("id")) < n(k.get("id"))) {
            keep = Some(u);
        }
    }
    match keep {
        Some(k) => {
            if let Some(o) = orders::hold(k, Point::new(1.0, 0.0), world, me, None) {
                order(o);
            }
            let rest: Vec<View> = units.iter().filter(|u| !u.same(&k)).collect();
            common::defend_then(&rest, world, me, "capture");
        }
        None => {
            common::defend_then(&units.iter().collect::<Vec<View>>(), world, me, "capture");
        }
    }
}

// Longbows (range 1: 4 cells) go with the army and shoot from behind it, and come home with it.
fn longbow(units: View, world: View, me: View) {
    if !home_alarm(units, world, me) {
        common::defend_then(&units.iter().collect::<Vec<View>>(), world, me, "fight");
    }
}

// While the plan says the capital is the province to defend, every unit of the type fights there. True when it did.
fn home_alarm(units: View, world: View, me: View) -> bool {
    if !same_v(me.get("capital"), plan::for_units(me, units, View::undefined()).get("defend")) {
        return false;
    }
    for u in units.iter() {
        if let Some(o) = orders::to(u, "fight", n(me.get("capital")), world, me, ()) {
            order(o);
        }
    }
    true
}

// The spy (stealth 1) walks to the nearest rival capital and steals the dearest tech it can there while there's ◆ to
// pay the bribes; otherwise it watches from outside the ring.
fn spy(units: View, world: View, me: View) {
    for u in units.iter() {
        let o = if n(me.get("bank")) > 150.0 { common::steal(u, world, me) } else { None };
        let x = o.or_else(|| if is_str(u.get("doing"), "steal") { None } else { common::sneak(u, world, me) });
        if let Some(x) = x {
            order(x);
        }
    }
}

// One scout walks toward whatever the faction knows least about.
fn scout(units: View, world: View, me: View) -> Vec<Value> {
    let mut out = Vec::new();
    for u in units.iter() {
        if is_str(u.get("doing"), "move") {
            continue; // walking
        }
        let rest = mem::get(u.get("ramUnit"), 1); // unit RAM page 1: when it may move on
        if rest.truthy() && vn(rest.get("until")) > n(world.get("seconds")) {
            continue;
        }
        let to = common::wander(world, me, n(u.get("at")));
        out.push(if to >= 0.0 {
            obj! { "id" => u.get("id"), "do" => "move", "target" => to }
        } else {
            obj! { "id" => u.get("id"), "ram" => mem::put(1, &obj! { "until" => n(world.get("seconds")) + 20.0 }, "local", Some("unit"), 1) }
        });
    }
    out
}

// Units in contact: each strikes the nearest rival it can hit (reflex::can_hit), and holds its ground doing so on an
// aggressive order. Each strike is committed as it's made, and the call stops when fewer cycles are left than one more
// unit costs (about 30 for each unit near it that it looks over), so a short duel bank gives fewer strikes, not an
// error.
fn reflex(units: View, near: View) -> Vec<Value> {
    let mut out = Vec::new();
    for u in units.iter() {
        let seen = near.key(n(u.get("id")));
        if cycles_left() < 120.0 + 30.0 * seen.len() as f64 {
            break;
        }
        let Some(t) = seen.iter().find(|t| reflex::can_hit(u, *t)) else { continue };
        let mut o = obj! { "id" => u.get("id"), "hit" => t.get("id") };
        if reflex::engaged(u) {
            o.set("hold", true);
        }
        order(o.clone());
        out.push(o);
    }
    out
}

fn research(c: View) -> Value {
    common::solve(c)
}

fn duel(c: View) -> Value {
    common::solve(c)
}

fn look(budget: View, info: View) -> Value {
    if let Some(t) = theme::of(info.get("seat"), -1.0) {
        return t.look(budget.num(), info);
    }
    obj! {
        "architecture" => "stone",
        "units" => obj! { "miner" => "peasant", "guard" => "footman", "scout" => "rider", "longbow" => "robed", "spy" => "rider" },
        "unit" => "footman",
        "worker" => "peasant",
        "palette" => obj! { "primary" => info.get("color") },
        "banner" => obj! { "pattern" => "solid", "emblem" => arr![arr!["circle", 0, 0, 3, "#ffffff"]] },
    }
}

// A founding seat's capital is its house's (theme); any other is plain: the library's civic core (the Wonder, a hall
// showing what this faction says, a stats screen, banners) and a ring of trees. The game builds it when the code
// changes; it costs RAM and cycles like anything else.
fn build3d(budget: View, info: View) -> Value {
    if let Some(t) = theme::of(info.get("seat"), -1.0) {
        return t.scene(info, budget.num());
    }
    let mut p = capital::civic(info, &capital::Civic { roof: Some("gable"), ..capital::Civic::default() });
    for q in build3d::around(8.0, 40.0, 0.0) {
        p.push(obj! { "type" => "tree", "kind" => "round", "x" => q.x, "z" => q.z, "h" => 7, "pri" => 2 });
    }
    obj! { "title" => info.get("name"), "sky" => obj! { "time" => "dusk" }, "pieces" => p }
}

// A little life while people walk through it, once a second (its cycles count like any code's): tap the Wonder and it
// spins up, then winds down.
fn live3d(state: View, now: View) -> Value {
    let mut s = if state.truthy() { state.value() } else { Value::obj() };
    let spin = capital::ring(&mut s, now);
    obj! { "state" => s, "set" => obj! { "wonder" => spin } }
}

// JavaScript's rules, for the few places this program needs them.

/// `Number(x)` for a value from the game: null is 0, anything that isn't a number NaN.
fn n(v: View) -> f64 {
    if v.is_null() {
        0.0
    } else {
        v.num()
    }
}

/// `Number(x)` for a value read from RAM.
fn vn(v: &Value) -> f64 {
    match v {
        Value::Null => 0.0,
        v => v.num(),
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

/// `a === b` for a value from the game and a number read from RAM.
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
