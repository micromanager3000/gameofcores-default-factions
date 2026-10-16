// Buildings belong to the default programs. All work uses public observations,
// earned research and the same shared CPU ledger as units.
use gameofcores::*;

struct Design { name: &'static str, limit: usize, range: f64, integrity: f64, height: f64 }
fn design(seat: usize) -> Design {
    let (name, limit, range, integrity, height) = match seat {
        0 => ("Steppe bastion", 4, 7.0, 2.0, 4.5),
        1 => ("Bow relay", 4, 6.0, 1.3, 5.5),
        2 => ("Chapter cloister", 3, 5.0, 1.8, 6.0),
        3 => ("Research court", 3, 0.0, 1.2, 4.0),
        4 => ("Harbor lantern", 4, 8.0, 1.4, 7.0),
        5 => ("Assembly hall", 5, 0.0, 1.5, 3.5),
        6 => ("Pointer spire", 2, 9.0, 1.0, 8.0),
        7 => ("Exchange depot", 5, 0.0, 1.3, 4.5),
        8 => ("Emu roost", 4, 5.0, 1.0, 5.0),
        _ => ("Open courtyard", 3, 4.0, 1.2, 3.0),
    };
    Design { name, limit, range, integrity, height }
}
fn same(a: View, b: View) -> bool { a.is_num() && b.is_num() && a.num() == b.num() }
fn seat(me: View) -> usize { me.get("seat").num_or(9.0).max(0.0) as usize }
fn port(s: View, kind: &str) -> bool { s.get("ports").iter().any(|p| p.str() == kind || p.get("kind").str() == kind) }
fn distance(u: View, p: f64, x: f64, y: f64) -> f64 {
    let pos = u.get("pos");
    if pos.get("p").num_or(-1.0) != p { return f64::INFINITY; }
    (pos.get("x").num_or(1e9) - x).hypot(pos.get("y").num_or(1e9) - y)
}
fn near(u: View, s: View, radius: f64) -> bool {
    distance(u, s.get("p").num(), s.get("x").num(), s.get("y").num()) <= radius
}
fn price(me: View) -> f64 { me.path("rts.costs.priceLevel").num_or(1.0).max(0.01) }
fn supply_worth(me: View, u: View) -> bool {
    let c = me.path("rts.costs.upkeep");
    let b = u.get("build");
    let stats: f64 = ["hp", "speed", "strength", "carry", "vision"].iter().map(|s| (b.get(s).num_or(2.0) - 2.0).max(0.0)).sum();
    let bill = c.get("base").num_or(0.0) + stats * c.get("stat").num_or(0.0)
        + u.path("rts.modules").len() as f64 * c.get("module").num_or(0.0)
        + b.get("range").num_or(0.0) * c.get("range").num_or(0.0)
        + b.get("stealth").num_or(0.0) * c.get("stealth").num_or(0.0);
    !u.get("air").truthy() && (bill * 0.4).min(100.0 * price(me)) > 20.0 * price(me)
}
fn open_slot(p: View, me: View) -> Option<usize> {
    p.get("mines").iter().enumerate().find(|(i, yield_)| {
        yield_.num_or(0.0) == 1.0 && realm::worker(p, *i as f64) == me.get("id").num()
            && p.get("staff").at(*i).num_or(3.0) < 2.0
    }).map(|(i, _)| i)
}

/// Clear visible hostile buildings once nearby soldiers have room to fire.
/// Short orders leave normal combat in control when an enemy army arrives.
fn besiege(orders: &mut Value, world: View, me: View, units: &[View], sites: View, now: f64) {
    let hostile: Vec<View> = sites.iter().filter(|s| s.get("state").str() != "ruin"
        && realm::is_rival(world, me, s.get("owner").num_or(-1.0))).collect();
    if hostile.is_empty() { return; }
    let mut attacks = Value::arr();
    for &u in units {
        if attacks.len() >= 8 || cycles_left() < 4500.0 { break; }
        if u.get("air").truthy() || u.get("stalled").truthy()
            || matches!(u.get("type").str(), "miner" | "hauler" | "freighter" | "scout")
            || matches!(u.get("doing").str(), "gather" | "study") { continue; }
        let reach = if u.path("build.range").num_or(0.0) > 0.0 { 3.0 + u.path("build.range").num() } else { 1.5 };
        let conduct = u.get("conduct");
        let target = hostile.iter().copied().filter(|s| {
            let protected = conduct.get("seconds").num_or(0.0) > 0.0 && same(conduct.get("from"), s.get("owner"));
            !protected && near(u, *s, reach + s.get("radius").num_or(0.0))
        }).min_by(|a, b| {
            // An armed shell stops the advance; finish damaged shells within the same priority.
            let priority = |s: View| s.get("hp").num_or(100.0) - if port(s, "weapon") { 300.0 } else { 0.0 };
            priority(*a).total_cmp(&priority(*b))
        });
        let Some(target) = target else { continue; };
        let enemy_near = world.get("units").iter().any(|foe| cycles_left() < 3500.0 || (foe.get("owner").num_or(-1.0) != me.get("id").num()
            && realm::is_rival(world, me, foe.get("owner").num_or(-1.0))
            && same(foe.path("pos.p"), u.path("pos.p"))
            && (foe.path("pos.x").num() - u.path("pos.x").num()).hypot(foe.path("pos.y").num() - u.path("pos.y").num()) < reach + 1.0));
        if enemy_near { continue; }
        attacks.push(obj! { "unit" => u.get("id"), "target" => target.get("id"), "until" => now + 5.0 });
    }
    if !attacks.is_empty() { orders.set("structureAttacks", attacks); }
}

pub fn apply(orders: &mut Value, world: View, me: View) {
    if !me.path("rts.structureCosts").truthy() { return; }
    // Planning must leave room for the returned RAM writes and building orders.
    // A limit is permission to spend; unused cycles stay in the shared bank.
    let income = me.path("rts.cpu.income").num_or(0.0);
    if me.path("rts.cpu.bank").num_or(0.0) > (income * 1.5).max(32000.0) {
        if !orders.get("compute").get("cpu").truthy() {
            orders.entry("compute").set("cpu", obj! { "limits" => me.path("rts.cpu.limits"),
                "reserve" => me.path("rts.cpu.reserve"), "upkeepOrder" => me.path("rts.cpu.upkeepOrder") });
        }
        let limits = orders.entry("compute").entry("cpu").entry("limits");
        let old = limits.get("think").num_or(0.0);
        let old = if old <= 1.0 { old * income } else { old };
        limits.set("think", old.max(32000.0).max(me.path("compute.thinkPeak").num_or(0.0) * 1.3 + 4000.0));
    }
    if cycles_left() < 6000.0 { return; }
    let id = me.get("id");
    let style = seat(me);
    let d = design(style);
    let now = world.get("seconds").num_or(0.0);
    // Refresh at most every five seconds; short standing jobs cover the interval.
    if now.floor() % 5.0 != 0.0 && orders.get("memo").str() != "Recovering production" { return; }
    let sites = me.path("rts.structures");
    let own: Vec<View> = sites.iter().filter(|s| same(s.get("owner"), id) && s.get("state").str() != "ruin").collect();
    let units: Vec<View> = world.get("units").iter().filter(|u| same(u.get("owner"), id) && u.get("hp").num_or(0.0) > 0.0).collect();
    let income = me.path("rts.cpu.income").num_or(0.0);
    let bank = me.path("rts.cpu.bank").num_or(0.0);
    let safe = orders.get("memo").str() != "Recovering production" && me.path("units.stalled").num_or(0.0) == 0.0
        && bank > income.max(1000.0 * price(me)) && me.path("rts.cpu.last.upkeep").num_or(0.0) < income * 0.7;
    if safe && cycles_left() > 8000.0 { besiege(orders, world, me, &units, sites, now); }
    let mut jobs = Value::arr();
    let mut services = Value::arr();
    for s in &own {
        if cycles_left() < 3500.0 { break; }
        if s.get("state").str() != "ready" { continue; }
        let mut j = obj! { "id" => s.get("id"), "extract" => 0, "extractSlot" => 0, "faith" => 0, "civic" => 0,
            "watch" => 0, "watchWater" => -1, "supply" => 0, "guard" => false, "target" => 0,
            "repair" => 0, "repairSite" => s.get("id"), "budget" => 0, "until" => now + 45.0 };
        if safe {
            j.set("budget", (income * 0.025).min(260.0 * price(me)));
            let p = world.get("provinces").iter().find(|p| same(p.get("id"), s.get("p")));
            if let Some(p) = p {
                // One local support provider; older allied/captured buildings may also exist.
                let first = !own.iter().any(|other| same(other.get("p"), s.get("p")) && port(*other, "support")
                    && other.get("state").str() == "ready" && other.get("id").num() < s.get("id").num());
                if port(*s, "support") && first {
                    let faith = if same(p.path("faith.by"), id) { if style == 2 { 0.8 } else { 0.35 } } else { 0.0 };
                    let water = world.get("waters").iter().find(|w| w.get("coast").has_num(p.get("id").num()));
                    let watch = if matches!(style, 4 | 6) && water.is_some() { if style == 4 { 0.65 } else { 0.35 } } else { 0.0 };
                    j.set("faith", faith);
                    j.set("watch", watch);
                    if let Some(w) = water { j.set("watchWater", w.get("id")); }
                    j.set("civic", (1.0_f64 - faith - watch).max(0.0));
                }
                if port(*s, "extract") {
                    if let Some(slot) = open_slot(p, me) { j.set("extract", 1); j.set("extractSlot", slot); }
                }
            }
            if port(*s, "logistics") && units.iter().any(|u| near(*u, *s, 8.0) && supply_worth(me, *u)) { j.set("supply", 1); }
            j.set("guard", port(*s, "weapon"));
            if port(*s, "repair") {
                // Site repairs and unit repairs share one cooldown. Prefer damaged buildings.
                if let Some(target) = own.iter().find(|t| t.get("state").str() == "ready" && same(t.get("p"), s.get("p"))
                    && t.get("hp").num() < t.get("maxHp").num()
                    && (t.get("x").num() - s.get("x").num()).hypot(t.get("y").num() - s.get("y").num()) <= 6.0) {
                    j.set("repair", 1); j.set("repairSite", target.get("id"));
                } else if let Some(u) = units.iter().find(|u| near(**u, *s, 6.0) && u.get("hp").num() < u.get("hpMax").num_or(0.0)) {
                    services.push(obj! { "site" => s.get("id"), "target" => u.get("id"), "amount" => 1 });
                }
            }
        }
        jobs.push(j);
    }
    if !jobs.is_empty() { orders.set("structureJobs", jobs); }
    if !services.is_empty() { orders.set("structureService", services); }
    if !safe || now < 35.0 || now % 30.0 >= 10.0 || own.len() >= d.limit
        || own.iter().any(|s| s.get("state").str() != "ready") || sites.len() >= 250 || cycles_left() < 6000.0 { return; }
    let mut land: Vec<View> = world.get("provinces").iter().filter(|p| same(p.get("owner"), id)
        && (!p.get("active").is_bool() || p.get("active").truthy()) && !own.iter().any(|s| same(s.get("p"), p.get("id")))).collect();
    let score = |p: View| {
        let capital = same(p.get("id"), me.get("capital"));
        let coast = world.get("waters").iter().any(|w| w.get("coast").has_num(p.get("id").num()));
        let border = p.get("adj").iter().any(|q| world.get("provinces").iter().any(|n| same(n.get("id"), q) && !same(n.get("owner"), id)));
        let crowd = units.iter().filter(|u| same(u.get("at"), p.get("id"))).count() as f64;
        (if capital { 12.0 } else { 0.0 }) + match style {
            0 | 1 | 8 => if border { 14.0 + crowd } else { crowd },
            2 => if same(p.path("faith.by"), id) { 20.0 } else { 0.0 },
            3 | 7 => if open_slot(p, me).is_some() { 16.0 + crowd } else { crowd },
            4 | 6 => if coast { 22.0 } else { crowd },
            5 => p.get("risk").num_or(0.0) * 80.0 + if capital { 0.0 } else { 14.0 },
            _ => crowd,
        }
    };
    land.sort_by(|a, b| score(*b).total_cmp(&score(*a)));
    for p in land.into_iter().take(8) {
        if cycles_left() < 3500.0 { break; }
        let province = p.get("id").num();
        let mut spot = None;
        // Compact town placements keep builders, supplies and the defended ring together.
        for (x, y) in [(2.8, 0.0), (-2.8, 0.0), (0.0, 2.8), (0.0, -2.8), (0.0, 0.0)] {
            if !realm::ground(p, x, y).is_some_and(|g| !matches!(g.kind, "water" | "cliff")) { continue; }
            if !units.iter().any(|u| !u.get("air").truthy() && distance(*u, province, x, y) <= 7.0) { continue; }
            if sites.iter().any(|s| s.get("state").str() != "ruin" && same(s.get("p"), p.get("id"))
                && (s.get("x").num() - x).hypot(s.get("y").num() - y) < s.get("radius").num_or(2.0) + 2.0) { continue; }
            spot = Some((x, y)); break;
        }
        let Some((x, y)) = spot else { continue; };
        let tech = me.get("techs");
        let mut ports = arr!["support", "logistics"];
        let c = me.path("rts.structureCosts");
        let mut quote = c.get("shell").num_or(900.0 * price(me)) * d.integrity + c.get("supportPort").num_or(1200.0 * price(me)) * 2.0;
        if d.range > 0.0 && tech.get("ranged").truthy() { ports.push(obj! { "kind" => "weapon", "range" => d.range, "arc" => 360 }); quote += c.get("weaponPort").num() * (d.range / 5.0).powi(2); }
        if tech.get("repair").truthy() && !matches!(style, 5 | 6) { ports.push("repair"); quote += c.get("repairPort").num(); }
        if matches!(style, 3 | 7 | 9) && tech.get("extraction").truthy() && open_slot(p, me).is_some() { ports.push("extract"); quote += c.get("extractPort").num(); }
        // Keep unit fabrication and several seconds of income ahead of each new site.
        let reserve = income * 2.0 + quote * 3.0;
        if bank < reserve { continue; }
        let key = (0..32).map(|i| format!("home-{province:.0}-{i}")).find(|k| !sites.iter().any(|s| same(s.get("owner"), id) && s.get("key").str() == k));
        let Some(key) = key else { continue; };
        orders.set("structures", arr![obj! { "key" => key, "p" => province, "x" => x, "y" => y,
            "shell" => obj! { "radius" => 2, "height" => d.height, "integrity" => d.integrity },
            "ports" => ports, "appearance" => architecture(style), "budget" => quote + price(me) }]);
        break;
    }
}

// Geometry is authored per house. Colors come from the owning faction in the viewer.
pub fn architecture(style: usize) -> Value {
    let mut pieces = Value::arr();
    let mut add = |kind: &str, x: f64, y: f64, z: f64, w: f64, h: f64, rot: f64| {
        pieces.push(obj! { "kind" => kind, "x" => x, "y" => y, "z" => z, "w" => w, "h" => h, "rot" => rot });
    };
    add("platform", 0., 0., 0., 4.2, 0.3, 0.);
    match style {
        0 => { // Fortified gate with four squat corner towers.
            for x in [-1.5, 1.5] { for y in [-1.5, 1.5] { add("tower", x, y, 0.3, 0.9, 3.2, 0.); } }
            add("wall", 0., 1.5, 0.3, 3., 1.7, 0.); add("arch", 0., -1.5, 0.3, 2.0, 2.3, 0.); add("roof", 0., 0., 2., 2.8, 1.1, 0.);
        }
        1 => { // Twin pylons and an arch like a drawn bow.
            for x in [-1.3, 1.3] { add("column", x, 0., 0.3, 0.65, 4., 0.); add("accent", x, 0., 4.3, 0.8, 0.8, 45.); }
            add("arch", 0., 0., 1., 3.6, 3.4, 0.); add("hall", 0., 1., 0.3, 2., 1.2, 0.);
        }
        2 => { // Cloister arcades surround a chapel and bell tower.
            for x in [-1.5, 1.5] { add("arch", x, 0., 0.3, 2.8, 2.3, 90.); }
            add("hall", 0., 0.7, 0.3, 2.3, 2.3, 0.); add("roof", 0., 0.7, 2.6, 2.6, 1.1, 0.); add("tower", 0., 1.4, 0.3, 0.7, 5., 0.); add("arch", 0., -1.5, 0.3, 2.5, 2.3, 0.);
        }
        3 => { // Stepped laboratory with three experiment columns.
            add("hall", 0., 0., 0.3, 3.1, 1.4, 0.); add("platform", 0., 0., 1.7, 3.4, 0.25, 0.);
            for (x,h) in [(-1.,1.2),(0.,2.0),(1.,1.5)] { add("column", x, 0., 1.95, 0.6, h, 0.); add("accent", x, 0., 1.95+h, 0.75, 0.3, 45.); }
        }
        4 => { // Terraced beacon over a long dock house.
            add("hall", 0., 0., 0.3, 3.2, 1.2, 0.); add("tower", 0., 0.8, 1.5, 1., 4.2, 0.);
            for z in [2., 3.5, 5.] { add("roof", 0., 0.8, z, 2., 0.5, 0.); }
            add("accent", 0., 0.8, 5.7, 0.9, 1., 0.); add("arch", 0., -1.5, 0.3, 3.1, 1.7, 0.);
        }
        5 => { // Broad assembly steps, open colonnade and raised red-star plinth.
            add("platform", 0., 0., 0.3, 3.7, 0.4, 0.); add("hall", 0., 0.6, 0.7, 3., 1.8, 0.);
            for x in [-1.2, 0., 1.2] { add("column", x, -1., 0.7, 0.35, 1.8, 0.); }
            add("roof", 0., 0., 2.5, 3.7, 0.3, 0.); add("accent", 0., 0., 2.8, 1., 0.6, 45.);
        }
        6 => { // Offset needle spires around an empty central address.
            for (x,y,h) in [(-1.2,0.8,6.8),(1.2,0.8,4.6),(0.,-1.2,2.8)] { add("tower", x, y, 0.3, 0.5, h, 0.); add("accent", x, y, 0.3+h, 0.65, 0.5, 45.); }
            add("arch", 0., 0., 0.3, 2., 1.5, 45.);
        }
        7 => { // Paired warehouse roofs frame a trading arcade.
            for x in [-1.25, 1.25] { add("hall", x, 0., 0.3, 1.4, 2., 90.); add("roof", x, 0., 2.3, 1.7, 1., 90.); }
            add("arch", 0., -1., 0.3, 1.8, 2., 0.); add("tower", 0., 1.3, 0.3, 0.8, 3.8, 0.);
        }
        8 => { // Tall legs hold a low roost with two swept roof wings.
            for x in [-1., 1.] { add("column", x, 0., 0.3, 0.4, 2.8, 0.); }
            add("hall", 0., 0., 3.1, 1.8, 1., 0.); add("roof", -1., 0., 4.1, 2., 0.45, -25.); add("roof", 1., 0., 4.1, 2., 0.45, 25.); add("accent", 0., -0.8, 4.1, 0.4, 0.8, 0.);
        }
        _ => { // Open courtyard with asymmetric garden walls.
            add("wall", -1.5, 0., 0.3, 3., 1.4, 90.); add("wall", 0., 1.5, 0.3, 3., 1., 0.);
            add("arch", 1.5, 0., 0.3, 2.6, 1.8, 90.); add("hall", -0.8, 0.8, 0.3, 1.3, 1.7, 0.); add("accent", 0., 0., 0.3, 0.9, 1.1, 45.);
        }
    }
    obj! { "name" => design(style).name, "pieces" => pieces }
}
