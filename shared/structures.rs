// Buildings belong to the default programs. All work uses public observations,
// earned research and the same shared CPU ledger as units.
use gameofcores::*;

/// A house's shells: battery range (0: none), integrity and the height of its civic hall.
struct Design { range: f64, integrity: f64, height: f64 }
fn design(seat: usize) -> Design {
    let (range, integrity, height) = match seat {
        0 => (7.0, 2.0, 4.5),
        1 => (6.0, 1.3, 5.5),
        2 => (5.0, 1.8, 6.0),
        3 => (0.0, 1.2, 4.0),
        4 => (8.0, 1.4, 7.0),
        5 => (0.0, 1.5, 3.5),
        6 => (9.0, 1.0, 8.0),
        7 => (0.0, 1.3, 4.5),
        8 => (5.0, 1.0, 5.0),
        _ => (4.0, 1.2, 3.0),
    };
    Design { range, integrity, height }
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
fn supply_return(me: View, u: View) -> f64 {
    if u.get("air").truthy() { return 0.0; }
    let c = me.path("rts.costs.upkeep");
    let b = u.get("build");
    let stats: f64 = ["hp", "speed", "strength", "carry", "vision"].iter().map(|s| (b.get(s).num_or(2.0) - 2.0).max(0.0)).sum();
    let bill = c.get("base").num_or(0.0) + stats * c.get("stat").num_or(0.0)
        + u.path("rts.modules").len() as f64 * c.get("module").num_or(0.0)
        + b.get("range").num_or(0.0) * c.get("range").num_or(0.0)
        + b.get("stealth").num_or(0.0) * c.get("stealth").num_or(0.0);
    ((bill * 0.4).min(100.0 * price(me)) - 20.0 * price(me)).max(0.0)
}
fn supply_worth(me: View, u: View) -> bool { supply_return(me, u) > 0.0 }
fn open_slot(p: View, me: View) -> Option<usize> {
    p.get("mines").iter().enumerate().find(|(i, yield_)| {
        yield_.num_or(0.0) == 1.0 && realm::worker(p, *i as f64) == me.get("id").num()
            && p.get("staff").at(*i).num_or(3.0) < 2.0
    }).map(|(i, _)| i)
}

/// Clear visible hostile buildings once nearby soldiers have room to fire.
/// Short orders leave normal combat in control when an enemy army arrives.
fn besiege(orders: &mut Value, world: View, me: View, units: &[View], sites: View, now: f64) {
    let hostile: Vec<View> = sites.iter().take_while(|_| cycles_left() > 6500.0).filter(|s| s.get("state").str() != "ruin"
        && realm::is_rival(world, me, s.get("owner").num_or(-1.0))).collect();
    if hostile.is_empty() { return; }
    // A live passage contract covers some of our units while they stand in its host's land. The public unit view only
    // exposes former-pact conduct, not that per-unit passage assignment; do not ask those covered units to fire there.
    let passage_hosts: Vec<f64> = me.get("contracts").iter().filter(|c| !c.get("ended").truthy()
        && c.get("get").get("passage").truthy()).map(|c| c.get("with").num_or(-1.0)).collect();
    let provinces = realm::index(world);
    let mut attacks = Value::arr();
    for &u in units {
        if attacks.len() >= 8 || cycles_left() < 4500.0 { break; }
        // Ranged aircraft attack buildings; scouts and interceptors keep their other orders.
        let bomber = u.get("air").truthy() && u.path("build.range").num_or(0.0) > 0.0;
        if (u.get("air").truthy() && !bomber) || u.get("stalled").truthy()
            || matches!(u.get("type").str(), "miner" | "hauler" | "freighter" | "scout")
            || matches!(u.get("doing").str(), "gather" | "study") { continue; }
        let reach = if u.path("build.range").num_or(0.0) > 0.0 { 3.0 + u.path("build.range").num() } else { 1.5 };
        let conduct = u.get("conduct");
        let target = hostile.iter().copied().take_while(|_| cycles_left() > 4500.0).filter(|s| {
            let protected = conduct.get("seconds").num_or(0.0) > 0.0 && same(conduct.get("from"), s.get("owner"));
            let host = s.get("owner").num_or(-1.0);
            let passage = passage_hosts.contains(&host) && same(provinces.get(u.get("at").num_or(-1.0)).get("owner"), s.get("owner"));
            !protected && !passage && near(u, *s, reach + s.get("radius").num_or(0.0))
        }).min_by(|a, b| {
            // An armed shell stops the advance; finish damaged shells within the same priority.
            // Then plants: a rival's dark towns are a weaker rival.
            let priority = |s: View| s.get("hp").num_or(100.0) - if port(s, "weapon") { 300.0 } else { 0.0 }
                - if s.get("power").truthy() { 150.0 } else { 0.0 };
            priority(*a).total_cmp(&priority(*b))
        });
        let Some(target) = target else { continue; };
        let enemy_near = world.get("units").iter().any(|foe| cycles_left() < 3500.0 || (foe.get("owner").num_or(-1.0) != me.get("id").num()
            && realm::is_rival(world, me, foe.get("owner").num_or(-1.0))
            && same(foe.path("pos.p"), u.path("pos.p"))
            && (foe.path("pos.x").num() - u.path("pos.x").num()).hypot(foe.path("pos.y").num() - u.path("pos.y").num()) < reach + 1.0));
        if enemy_near && !bomber { continue; }
        attacks.push(obj! { "unit" => u.get("id"), "target" => target.get("id"), "until" => now + 6.0 });
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
    let now = world.get("seconds").num_or(0.0);
    // Refresh every sixth second, a full-think second for a resting house (common::govern); short standing jobs cover
    // the interval.
    if now.floor() % 6.0 != 0.0 && orders.get("memo").str() != "Recovering production" { return; }
    let sites = me.path("rts.structures");
    let mut own: Vec<View> = sites.iter().filter(|s| same(s.get("owner"), id) && s.get("state").str() != "ruin").collect();
    if !own.is_empty() { let start = (now as usize / 6) % own.len(); own.rotate_left(start); }
    let units: Vec<View> = world.get("units").iter().filter(|u| same(u.get("owner"), id) && u.get("hp").num_or(0.0) > 0.0).collect();
    let income = me.path("rts.cpu.income").num_or(0.0);
    let bank = me.path("rts.cpu.bank").num_or(0.0);
    let funded = orders.get("memo").str() != "Recovering production"
        && bank > income.max(1000.0 * price(me)) && me.path("rts.cpu.last.upkeep").num_or(0.0) < income * 0.7;
    // An isolated stalled unit must not stop funded supply, repairs and defence.
    // New construction still waits until the existing force can operate.
    let safe = funded && me.path("units.stalled").num_or(0.0) == 0.0;
    let mut jobs = Value::arr();
    let mut services = Value::arr();
    for s in &own {
        if cycles_left() < 4500.0 || jobs.len() >= 24 { break; }
        if s.get("state").str() != "ready" { continue; }
        let mut j = obj! { "id" => s.get("id"), "extract" => 0, "extractSlot" => 0, "faith" => 0, "civic" => 0,
            "watch" => 0, "watchWater" => -1, "supply" => 0, "guard" => false, "target" => 0,
            "repair" => 0, "repairSite" => s.get("id"), "budget" => 0, "until" => now + 45.0 };
        if funded {
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
    // Nearby allies can use a funded repair court. Merchant houses charge a
    // small accepted fee; other houses donate a bounded amount to their pact.
    let grants = me.path("rts.structureGrants");
    if safe && cycles_left() > 6500.0 {
        let mut offered = Value::arr();
        for s in &own {
            if cycles_left() < 5000.0 || offered.len() >= 4 { break; }
            if s.get("state").str() != "ready" || !port(*s, "repair") { continue; }
            if let Some(guest) = world.get("units").iter().take_while(|_| cycles_left() > 4500.0).find(|u|
                realm::is_ally(me, u.get("owner").num()) && u.get("owner").num() != id.num()
                && u.get("hp").num() < u.get("hpMax").num_or(0.0) && near(*u, *s, 6.0)) {
                if !grants.iter().any(|g| same(g.get("site"), s.get("id")) && same(g.get("user"), guest.get("owner"))
                    && g.get("until").num_or(0.0) > now) {
                    offered.push(obj! {"site"=>s.get("id"),"user"=>guest.get("owner"),"quota"=>6,"rate"=>0.5,
                        "price"=>if style == 7 {0.25} else {0.0},"until"=>now+60.0,"payer"=>"owner"});
                }
            }
        }
        if !offered.is_empty() { orders.set("structureGrants", offered); }
    }
    if safe && cycles_left() > 4500.0 {
        for grant in grants.iter() {
            if cycles_left() < 3500.0 || services.len() >= 8 { break; }
            if !same(grant.get("user"), id) || grant.get("quota").num_or(0.0) <= 0.0 || grant.get("until").num_or(0.0) <= now
                || grant.get("price").num_or(0.0) > 0.25 { continue; }
            let site = sites.iter().find(|s| same(s.get("id"), grant.get("site")) && s.get("state").str() == "ready");
            if let Some(site) = site {
                if let Some(u) = units.iter().find(|u| near(**u, site, 6.0) && u.get("hp").num() < u.get("hpMax").num_or(0.0)) {
                    services.push(obj! {"site"=>site.get("id"),"target"=>u.get("id"),"amount"=>0.5,"maxPrice"=>0.25});
                }
            }
        }
    }
    if !jobs.is_empty() { orders.set("structureJobs", jobs); }
    if !services.is_empty() { orders.set("structureService", services); }
    if safe && cycles_left() > 8000.0 { besiege(orders, world, me, &units, sites, now); }
    if !safe || now < 35.0 || now % 20.0 >= 10.0
        || own.iter().any(|s| s.get("state").str() != "ready") || cycles_left() < 6000.0 { return; }
    plan(orders, world, me, style, &own, &units, sites, now);
}

/// One building the planner could order next.
struct Want { p: View, role: Role, r: f64, ports: Value, quote: f64, value: f64 }

/// Grow the towns one building at a time. Every province gets a civic hall once the loyalty it buys repays it (the
/// capital's is the house landmark); then districts: supply yards where troops gather, bastions on open borders, works
/// at free mines and repair bays behind busy yards. Economic sites must repay their quote within five minutes.
fn plan(orders: &mut Value, world: View, me: View, style: usize, own: &[View], units: &[View], sites: View, now: f64) {
    let id = me.get("id");
    let d = design(style);
    let c = me.path("rts.structureCosts");
    let pl = price(me);
    let income = me.path("rts.cpu.income").num_or(0.0);
    let bank = me.path("rts.cpu.bank").num_or(0.0);
    let tech = me.get("techs");
    let land: Vec<View> = world.get("provinces").iter().filter(|p| same(p.get("owner"), id)
        && (!p.get("active").is_bool() || p.get("active").truthy())).collect();
    if land.is_empty() { return; }
    // A province's share of income, and what one more point of loyalty adds to it (income runs 35% to 100% with loyalty).
    let share = income / land.len() as f64;
    let support = c.get("supportPort").num_or(1200.0 * pl);
    let shell = |r: f64, integrity: f64| c.get("shell").num_or(900.0 * pl) * integrity * (r / 2.0).sqrt();
    let period = c.get("maintenancePeriod").num_or(600.0).max(1.0);
    let repays = |quote: f64, gain: f64| { let net = gain - quote / period; net > 0.0 && quote / net <= 300.0 };
    let repair = tech.get("repair").truthy() && !matches!(style, 5 | 6);
    let mut wants: Vec<Want> = Vec::new();
    for p in land.iter().copied().take(16) {
        if cycles_left() < 5500.0 { break; }
        let pid = p.get("id").num();
        let here: Vec<View> = own.iter().copied().filter(|s| same(s.get("p"), p.get("id"))).collect();
        let has = |kind: &str| here.iter().any(|s| port(*s, kind));
        // Supply: the best six upkeep savings among nearby troops that this province's yards don't already cover.
        let mut returns: Vec<f64> = units.iter().filter(|u| distance(**u, pid, 0.0, 0.0) <= 9.0).map(|u| supply_return(me, *u)).collect();
        returns.sort_by(|a, b| b.total_cmp(a));
        let covered = here.iter().filter(|s| port(**s, "logistics")).count() * 6;
        let supply = returns.iter().skip(covered).take(6).sum::<f64>() * 0.6;
        let border = p.get("adj").iter().any(|q| !land.iter().any(|o| same(o.get("id"), q)));
        if !has("support") {
            let capital = same(p.get("id"), me.get("capital"));
            let (role, r) = if capital { (Role::Landmark, 2.5) } else { (Role::Hall, 2.0) };
            let l = p.get("loyalty").num_or(50.0);
            let lift = (100.0 - p.get("target").num_or(l)).clamp(0.0, c.get("civicBonus").num_or(18.0));
            let civic = share * 0.65 * lift / 100.0 / (0.35 + 0.65 * l / 100.0).max(0.35) - c.get("civicSecond").num_or(6.0 * pl);
            let faith = if same(p.path("faith.by"), id) { p.path("faith.zeal").num_or(0.0) * if style == 2 { 0.8 } else { 0.35 } * pl } else { 0.0 };
            let mut ports = arr!["support"];
            let mut quote = shell(r, d.integrity) + support;
            let mut gain = civic.max(0.0) + faith;
            if supply > 0.0 { ports.push("logistics"); quote += support; gain += supply; }
            if !repays(quote, gain) { continue; }
            let value = gain - quote / period;
            // A hall on an open border mounts the house battery as well: defence repays nothing measurable, so it rides
            // on a hall that pays for itself.
            if border && d.range > 0.0 && tech.get("ranged").truthy() {
                ports.push(obj! { "kind" => "weapon", "range" => d.range, "arc" => 360 });
                quote += c.get("weaponPort").num_or(1800.0 * pl) * (d.range / 5.0).powi(2);
            }
            wants.push(Want { p, role, r, ports, quote, value });
            continue;
        }
        if supply > 0.0 {
            // A second yard in a crowded province repairs as well.
            let (role, r, mut ports, mut quote) = (Role::Yard, 1.6, arr!["logistics"], shell(1.6, d.integrity) + support);
            let (role, r) = if repair && covered > 0 && !has("repair") {
                ports.push("repair"); quote += c.get("repairPort").num_or(1500.0 * pl); (Role::Bay, 1.5)
            } else { (role, r) };
            if repays(quote, supply) { wants.push(Want { p, role, r, ports, quote, value: supply - quote / period }); continue; }
        }
        if border && d.range > 0.0 && tech.get("ranged").truthy() && !has("weapon") {
            let mut ports = arr![obj! { "kind" => "weapon", "range" => d.range, "arc" => 360 }];
            let mut quote = shell(1.6, d.integrity * 1.3) + c.get("weaponPort").num_or(1800.0 * pl) * (d.range / 5.0).powi(2);
            if repair { ports.push("repair"); quote += c.get("repairPort").num_or(1500.0 * pl); }
            // Defence repays nothing measurable: build it only from a comfortable bank, after the economy.
            if bank > income * 4.0 + quote * 6.0 { wants.push(Want { p, role: Role::Bastion, r: 1.6, ports, quote, value: 0.0 }); }
            continue;
        }
        if matches!(style, 3 | 7 | 9) && tech.get("extraction").truthy() && !has("extract") && open_slot(p, me).is_some() {
            let quote = shell(1.5, d.integrity) + c.get("extractPort").num_or(1800.0 * pl);
            if bank > income * 3.0 + quote * 4.0 { wants.push(Want { p, role: Role::Works, r: 1.5, ports: arr!["extract"], quote, value: -1.0 }); }
        }
    }
    wants.sort_by(|a, b| b.value.total_cmp(&a.value));
    for w in wants.into_iter().take(4) {
        // A site search reads the ground round the town before its own checks: leave it room to finish.
        if cycles_left() < 20000.0 { break; }
        // Keep unit fabrication and several seconds of income ahead of each new site.
        if bank < income * 2.0 + w.quote * 3.0 { continue; }
        let Some((x, y)) = spot(w.p, sites, w.r, units) else { continue; };
        let province = w.p.get("id").num();
        let height = match w.role { Role::Landmark => d.height + 2.0, Role::Hall => d.height, _ => (d.height - 1.0).max(2.5) };
        let integrity = if w.role == Role::Bastion { d.integrity * 1.3 } else { d.integrity };
        let key = format!("home-{province:.0}-{now:.0}");
        orders.set("structures", arr![obj! { "key" => key, "p" => province, "x" => x, "y" => y,
            "shell" => obj! { "radius" => w.r, "height" => height, "integrity" => integrity },
            "ports" => w.ports, "appearance" => architecture(style, w.role), "budget" => w.quote + pl }]);
        break;
    }
}

/// A spot for a shell of radius r round province p's town, nearest the square first: clear of the square itself, its
/// mines, water, cliffs and every standing site, on the province's own side of each border, with a builder near.
pub fn spot(p: View, sites: View, r: f64, units: &[View]) -> Option<(f64, f64)> {
    let pid = p.get("id").num();
    // Neighbouring towns stand about twelve cells away; the border runs halfway.
    let towns: Vec<(f64, f64)> = p.get("adj").iter().map(|q| { let t = realm::toward(p, q.num(), 1.0); (t.x, t.y) })
        .filter(|t| t.0 != 0.0 || t.1 != 0.0).collect();
    let inside = |x: f64, y: f64, margin: f64| towns.iter().all(|(tx, ty)| x * tx + y * ty < 6.0 - margin);
    let mut mines = Vec::new();
    for q in -6i32..=6 { for rr in (-6i32).max(-q - 6)..=6i32.min(-q + 6) {
        let (x, y) = (q as f64 + rr as f64 / 2.0, rr as f64 * 0.866_025_4);
        if realm::ground(p, x, y).is_some_and(|g| g.kind == "mine") { mines.push((x, y)); }
    } }
    let others: Vec<(f64, f64, f64)> = sites.iter().filter(|s| s.get("state").str() != "ruin" && same(s.get("p"), p.get("id")))
        .map(|s| (s.get("x").num(), s.get("y").num(), s.get("radius").num_or(2.0))).collect();
    let open = |x: f64, y: f64| realm::ground(p, x, y).map(|g| g.kind);
    let start = 1.0 + r;
    for ring in 0..5 {
        let rad = start + ring as f64 * 0.5;
        let mut best: Option<(f64, f64, f64)> = None;
        for k in 0..24 {
            if cycles_left() < 3500.0 { return None; }
            let a = (k as f64 * 15.0 + if ring % 2 == 1 { 7.5 } else { 0.0 }).to_radians();
            let (x, y) = (rad * a.cos(), rad * a.sin());
            if !inside(x, y, r * 0.55) || others.iter().any(|(ox, oy, or)| (ox - x).hypot(oy - y) < or + r + 0.15)
                || mines.iter().any(|(mx, my)| (mx - x).hypot(my - y) < r + 0.8) { continue; }
            // The footprint stands on dry ground; a road may cross its edge but not its middle.
            let mut roads = 0.0;
            let mut ok = matches!(open(x, y), Some("open" | "forest" | "rough"));
            for j in 0..8 {
                if !ok { break; }
                let b = (j as f64 * 45.0).to_radians();
                for f in [0.5, 0.9] {
                    match open(x + f * r * b.cos(), y + f * r * b.sin()) {
                        Some("water" | "cliff" | "mine" | "town" | "ford") | None => { ok = false; break; }
                        Some("road") => roads += if f < 0.7 { 2.0 } else { 1.0 },
                        _ => {}
                    }
                }
            }
            if !ok || roads > 3.0 || !units.iter().any(|u| !u.get("air").truthy() && distance(*u, pid, x, y) <= 7.0) { continue; }
            if best.map_or(true, |b| roads < b.2) { best = Some((x, y, roads)); }
        }
        if let Some((x, y, _)) = best { return Some(((x * 10.0).round() / 10.0, (y * 10.0).round() / 10.0)); }
    }
    None
}

// ---------------------------------------------------------------------------------------------------------------------
// Architecture. A building stands at the scale of the ground under it: pieces are in cells from the site's centre
// (x east, y north, z up), so a radius-2 site covers about four cells across. The viewer looks from the south and a
// little east: fronts face south and tall pieces stand at the back. `faction` pieces take the owner's colour, which is
// how a city reads as yours from above; the other tones are materials.

/// What a site is for. Its ports follow from it, and so does its look.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role { Landmark, Hall, Yard, Works, Bastion, Bay }
pub const ROLES: [Role; 6] = [Role::Landmark, Role::Hall, Role::Yard, Role::Works, Role::Bastion, Role::Bay];

/// One piece of a building. Kind and tone are letters from KINDS and TONES; tone 0 is the kind's usual material.
#[derive(Clone, Copy)]
struct Pc { kind: u8, x: f64, y: f64, z: f64, w: f64, h: f64, d: f64, rot: f64, tone: u8 }
const KINDS: &str = "pplatform hhall ttower rroof aarch ccolumn wwall xaccent Ddome sspire Ppyramid ddrum Ttiers kstack ggarden bbanner llight Sstatue";
const TONES: &str = "Ffaction sstone mmarble bbrick wwood Mmetal ggold Gglass kdark jjade nsand";
fn word(table: &str, c: u8) -> &str { table.split(' ').find(|w| w.as_bytes()[0] == c).map_or("", |w| &w[1..]) }
struct Art(Vec<Pc>);
impl Art {
    fn add(&mut self, kind: u8, x: f64, y: f64, z: f64, w: f64, h: f64, d: f64, tone: u8) {
        self.0.push(Pc { kind, x, y, z, w, h, d, rot: 0.0, tone });
    }
    /// The engine's `appearance`: at most 40 pieces, inside its bounds, to the hundredth of a cell.
    fn value(&self, name: &str) -> Value {
        let r = |v: f64, lo: f64, hi: f64| (v.clamp(lo, hi) * 100.0).round() / 100.0;
        let mut pieces = Value::arr();
        for q in self.0.iter().take(40) {
            let mut o = obj! { "kind" => word(KINDS, q.kind), "x" => r(q.x, -8.0, 8.0), "y" => r(q.y, -8.0, 8.0),
                "z" => r(q.z, 0.0, 12.0), "w" => r(q.w, 0.1, 12.0), "h" => r(q.h, 0.1, 16.0) };
            if q.d > 0.0 { o.set("d", r(q.d, 0.1, 12.0)); }
            if q.rot != 0.0 { o.set("rot", r(q.rot, -360.0, 360.0)); }
            if q.tone != 0 { o.set("tone", word(TONES, q.tone)); }
            pieces.push(o);
        }
        obj! { "name" => name, "pieces" => pieces }
    }
}

/// A house's building kit (shared/buildings.txt). The optional sixth glyph controls
/// the small forecourt marker; older lines use their house's chosen fallback tone.
struct Kit { wall: u8, base: u8, roof: u8, spire: u8, garden: bool, accent: u8 }
fn kit(code: &str) -> Kit {
    let b = code.as_bytes();
    Kit { wall: b[0], base: b[1], roof: b[2], spire: b[3], garden: b[4] == b'y', accent: *b.get(5).unwrap_or(&b'g') }
}
/// The house roof over a block w × d centred at (x, y) whose walls end at z.
fn roof(a: &mut Art, k: &Kit, x: f64, y: f64, z: f64, w: f64, d: f64) {
    let m = w.min(d);
    match k.roof {
        b'G' => a.add(b'r', x, y, z, w, (m * 0.5).min(1.1), d, b'F'),
        b'D' => {
            a.add(b'x', x, y, z, w + 0.06, 0.1, d + 0.06, k.base);
            a.add(b'd', x, y, z + 0.1, m * 0.6, 0.35, 0.0, k.wall);
            a.add(b'D', x, y, z + 0.45, m * 0.64, m * 0.4, 0.0, b'F');
        }
        b'T' => a.add(b'T', x, y, z, w * 0.95, (m * 0.9).clamp(1.2, 2.4), d * 0.95, b'F'),
        b'S' => a.add(b'P', x, y, z, w * 0.92, (m * 0.75).clamp(0.9, 2.2), d * 0.92, b'F'),
        _ => a.add(b'x', x, y, z, w * 0.9, 0.16, d * 0.9, b'F'),
    }
}
/// A house tower w across and h tall at (x, y), standing on z.
fn spire(a: &mut Art, k: &Kit, x: f64, y: f64, z: f64, w: f64, h: f64) {
    let t = k.wall;
    match k.spire {
        b'd' => { a.add(b'd', x, y, z, w, h, 0.0, t); a.add(b'D', x, y, z + h, w * 1.08, w * 0.62, 0.0, b'F'); }
        b'm' => {
            a.add(b'd', x, y, z, w * 0.72, h, 0.0, t);
            a.add(b'd', x, y, z + h * 0.72, w * 0.95, 0.12, 0.0, b'g');
            a.add(b'D', x, y, z + h, w * 0.8, w * 1.25, 0.0, b'F');
        }
        b'n' => { a.add(b't', x, y, z, w, h, 0.0, t); a.add(b's', x, y, z + h + 0.22, w * 0.95, h * 0.6, 0.0, b'F'); }
        b'o' => { a.add(b't', x, y, z, w, h, 0.0, t); a.add(b'D', x, y, z + h + 0.22, w * 0.88, w * 0.55, 0.0, b'G'); }
        b'p' => a.add(b'T', x, y, z, w * 1.3, h * 1.1, 0.0, b'F'),
        b'M' => {
            a.add(b'h', x, y, z, w * 0.85, h, w * 0.85, b'G');
            a.add(b'x', x, y, z + h, w * 0.6, 0.25, w * 0.6, b'F');
            a.add(b'l', x, y, z + h + 0.25, 0.2, h * 0.45, 0.0, b'M');
        }
        b'c' => {
            a.add(b't', x, y, z, w, h, 0.0, t);
            a.add(b'x', x, y - w / 2.0 - 0.03, z + h - w * 0.75, w * 0.55, w * 0.55, 0.06, b'g');
            a.add(b's', x, y, z + h + 0.22, w * 0.9, h * 0.4, 0.0, b'F');
        }
        b't' => {
            a.add(b'c', x, y, z, 0.16, h * 0.7, 0.0, b'M');
            a.add(b'd', x, y, z + h * 0.7, w * 1.2, h * 0.3, 0.0, b'M');
            a.add(b's', x, y, z + h, w * 1.25, w * 0.4, 0.0, b'F');
        }
        _ => a.add(b'S', x, y, z, w * 0.9, h * 1.1, 0.0, b'm'),
    }
}

// Designs are written small, since a program pays for its length in memory: pieces `kind x,y,z,w,h,d,rot/tone` split by
// `;`, a blank number is 0 and d and rot may be left off. `+x,y,z` repeats the last piece elsewhere (left off, z stays).
// `<...>` draws its pieces mirrored east to west, then as written. `R` and `Q` are the kit's roof (x,y,z,w,d) and tower
// (x,y,z,w,h). Pieces after `?` or `!` stand only with or without gardens, after `=` or `~` only with or without flat
// roofs. Tones W and B are the kit's walls and terrace; V is the walls and X brick, both metal in a glass house.
// Each design is drawn for its role's radius in `plan`.

/// The hall, yard, works, bastion and bay. 
const DESIGNS: [&str; 5] = [
    // A civic hall: a terrace and steps, the hall under the house roof, a portico, wings, two towers, greens and banners.
    "p,.1,,3.6,.35,3.1/B;p,-1.6,,1.4,.18,.45/B;h,.4,.35,2.2,1.5,1.5/W;R,.4,1.85,2.2,1.5;<c.9,-.62,.35,.2,1.25/m;+.3,-.62>;x,-.62,1.6,2.1,.2,.42/W;<h1.35,.65,.35,.75,1,1.2/W;x1.35,.65,1.35,.8,.14,1.25/F;Q1.4,1.25,.35,.55,2.3;?g1.2,-1,.35,.9,.1,.8;!S1.25,-1.05,.35,.42,1.5/k;b.85,-1.35,.35,.45,2;l.42,-1.42,.35,.16,.75/g>",
    // A supply yard: warehouses under the house colour, stacked crates, a crane, a fence and a gate.
    "p,,,2.8,.15,2.48/B;h-.24,.76,.15,2.08,1.15,.8/V;=x-.24,.76,1.3,1.91,.14,.74/F;~r-.24,.76,1.3,2.18,.36,.9/F;h-1,-.36,.15,.72,.9,1.2/V;=x-1,-.36,1.05,.66,.14,1.1/F;~r-1,-.36,1.05,.82,.32,1.3/F;x.12,-.36,.15,.32,.32/w;+.4,-.36;+.26,-.36,.47;+.12,-.72,.15;+.68,.08;+.68,-.24;c1.04,.24,.15,.14,2.4/M;x.68,.08,2.45,1.5,.1,.12,-25/M;w.72,-1.2,.15,1.04,.45,.12/w;a-.28,-1.2,.15,.72,1,.3/B;b1.24,-1.04,.15,.45,1.8;l1.2,.8,.15,.16,1.3/g",
    // Extraction works: a shed, smoking stacks, tanks, a headframe and heaps of ore.
    "p,,,2.63,.15,2.33/k;h-.34,.41,.15,1.65,1.3,1.05/X;r-.34,.41,1.45,1.73,.5,1.13/F;k.71,.83,.15,.42,2.3/b;k1.05,.45,.15,.36,1.8/b;d.86,-.41,.15,.8,.9/M;D.86,-.41,1.05,.8,.28/M;d.3,-.79,.15,.6,.7/M;D.3,-.79,.85,.6,.21/M;t-1.01,-.22,.15,.45,2.1/M;r-1.01,-.22,2.47,.6,.3,.6/F;P-.79,-.86,.15,.9,.5/k;P-.26,-.94,.15,.6,.35/n;l.26,-.15,.15,.16,1.2/g;b1.16,-.94,.15,.45,1.6",
    // A bastion: curtain walls round a keep (the battery stands on top), towers at the corners and a gatehouse.
    "p,,,2.72,.3,2.72/s;w,1.12,.3,2.24,1.1,.35/s;<w1.12,,.3,2.24,1.1,.35,90/s;w.76,-1.12,.3,.76,1.1,.35/s>;a,-1.12,.3,.76,1.5,.5/W;r,-1.12,1.8,.84,.35,.6/F;<Q1.12,1.12,.3,.7,1.8;Q1.12,-1.12,.3,.7,1.8>;h-.44,.44,.3,.8,.9,.64/W;t.2,.12,.3,.96,2.5/s;b.52,.44,3.02,.45,1.1;l-.44,-.6,.3,.16,.8/g",
    // A repair bay: a workshop with a great door, a gantry crane, lamps and spares.
    "p,,,2.55,.15,2.25/B;h,.45,.15,1.88,1.5,1.05/V;R,.45,1.65,1.875,1.05;a,-.09,.15,1.13,1.4,.2/B;<c.9,-.68,.15,.14,2/M;l.45,-.98,.15,.18,.9/g>;x,-.68,2.15,1.95,.14,.16/F;x-1.01,-.19,.15,.3,.3/w;+-.75,-.3;+1.01,-.22;b1.13,.94,.15,.45,2",
];

/// Draws `src` into `a`; sx is -1 for the mirrored half of a `<...>`.
fn draw(a: &mut Art, k: &Kit, src: &str, sx: f64) {
    let mut rest = src;
    while !rest.is_empty() {
        if let Some(b) = rest.strip_prefix('<') {
            let end = b.find('>').unwrap_or(b.len());
            draw(a, k, &b[..end], -sx);
            draw(a, k, &b[..end], sx);
            rest = b.get(end + 1..).unwrap_or("").trim_start_matches(';');
            continue;
        }
        let end = rest.find([';', '<']).unwrap_or(rest.len());
        piece(a, k, &rest[..end], sx);
        rest = rest[end..].trim_start_matches(';');
    }
}
fn piece(a: &mut Art, k: &Kit, s: &str, sx: f64) {
    let (s, ok) = match s.bytes().next() {
        Some(b'?') => (&s[1..], k.garden),
        Some(b'!') => (&s[1..], !k.garden),
        Some(b'=') => (&s[1..], k.roof == b'F'),
        Some(b'~') => (&s[1..], k.roof != b'F'),
        _ => (s, true),
    };
    if !ok || s.is_empty() { return; }
    let (body, tone) = s[1..].split_once('/').unwrap_or((&s[1..], ""));
    let v: Vec<f64> = body.split(',').map(|n| n.parse().unwrap_or(0.0)).collect();
    let g = |i: usize| v.get(i).copied().unwrap_or(0.0);
    let x = g(0) * sx;
    match s.as_bytes()[0] {
        b'R' => roof(a, k, x, g(1), g(2), g(3), g(4)),
        b'Q' => spire(a, k, x, g(1), g(2), g(3), g(4)),
        b'+' => if let Some(&last) = a.0.last() { a.0.push(Pc { x, y: g(1), z: v.get(2).copied().unwrap_or(last.z), ..last }); },
        kind => {
            let tone = match tone.bytes().next().unwrap_or(0) {
                b'W' => k.wall,
                b'B' => k.base,
                b'V' => if k.wall == b'G' { b'M' } else { k.wall },
                b'X' => if k.wall == b'G' { b'M' } else { b'b' },
                t => t,
            };
            a.0.push(Pc { kind, x, y: g(1), z: g(2), w: g(3), h: g(4), d: g(5), rot: g(6), tone });
        }
    }
}

/// The default houses share useful site roles, but should never read as the same
/// box in ten paint schemes. These are the principal founder masses: each sits
/// over the role's working plan and changes the roofline a viewer sees first.
fn house_massing(a: &mut Art, style: usize, role: Role) {
    // Landmarks have complete authored plans in buildings.txt.  Bastions get a
    // house command core inside their readable shared defensive curtain.
    if role == Role::Landmark { return; }
    let (w, d, h) = match role {
        Role::Landmark => (2.7, 2.3, 1.8), Role::Hall => (2.35, 1.75, 1.5),
        Role::Yard => (1.75, 1.25, 1.05), Role::Works => (1.7, 1.2, 1.12),
        Role::Bastion => (2.0, 1.8, 1.3), Role::Bay => (1.85, 1.25, 1.15),
    };
    let (x, y) = (0.0, 0.35);
    match style {
        // A low radial tent court: several canopies, none large enough to read as a civic dome.
        0 => { a.add(b's', x, y, 0.16, w*0.92, h*0.52, d*0.86, b'm'); a.add(b's', x-w*0.38, y-0.18, 0.16, w*0.38, h*0.35, d*0.42, b'F'); a.add(b's', x+w*0.38, y-0.18, 0.16, w*0.38, h*0.35, d*0.42, b'F'); a.add(b's', x, y+d*0.34, 0.16, 0.14, h*1.45, 0., b'g'); }
        // A tall stepped stone temple.  The receding levels are its silhouette, never a lotus dome.
        1 => { a.add(b'P', x, y, 0.16, w*1.08, h*0.54, d*1.04, b'n'); a.add(b'P', x, y, 0.16+h*0.42, w*0.72, h*0.54, d*0.68, b'n'); a.add(b'P', x, y, 0.16+h*0.84, w*0.42, h*0.62, d*0.38, b'F'); a.add(b's', x, y, 0.16+h*1.42, 0.12, h*0.62, 0., b'g'); }
        // A cathedral is a long roofed nave, two west spires and buttresses rather than one flat arch.
        2 => { a.add(b'h', x, y, 0.16, w*0.62, h*1.05, d*1.22, b's'); a.add(b'r', x, y, 0.16+h*1.05, w*0.72, h*0.58, d*1.32, b'F'); a.add(b's', x-w*0.42, y-d*0.36, 0.16, w*0.16, h*1.82, 0., b'F'); a.add(b's', x+w*0.42, y-d*0.36, 0.16, w*0.16, h*1.82, 0., b'F'); }
        // A raised glass laboratory bridges brass pressure tubes below it.
        3 => { a.add(b'd', x-w*0.34, y, 0.16, w*0.28, h*0.72, 0., b'M'); a.add(b'd', x, y, 0.16, w*0.28, h*0.72, 0., b'M'); a.add(b'd', x+w*0.34, y, 0.16, w*0.28, h*0.72, 0., b'M'); a.add(b'h', x, y+d*0.18, 0.16+h*0.78, w*1.12, h*0.64, d*0.56, b'G'); a.add(b'x', x, y+d*0.18, 0.16+h*0.72, w*1.28, 0.12, d*0.68, b'g'); }
        // Timber pagoda: broad separate eaves and a stilted undercroft, not a stepped pyramid.
        4 => { a.add(b'c', x-w*0.34, y, 0.16, 0.14, h*1.42, 0., b'w'); a.add(b'c', x+w*0.34, y, 0.16, 0.14, h*1.42, 0., b'w'); a.add(b'h', x, y, 0.16, w*0.62, h*0.72, d*0.58, b'w'); a.add(b'r', x, y, 0.16+h*0.68, w*1.20, 0.24, d*1.12, b'F'); a.add(b'r', x, y, 0.16+h*1.10, w*0.88, 0.24, d*0.82, b'F'); }
        // An exposed rectangular works hall with a serrated roofline and chimney bank.
        5 => { a.add(b'h', x, y, 0.16, w*1.16, h*0.72, d*1.04, b'b'); a.add(b'r', x-w*0.30, y, 0.16+h*0.70, w*0.34, h*0.38, d*1.10, b'F'); a.add(b'r', x+w*0.10, y, 0.16+h*0.70, w*0.34, h*0.38, d*1.10, b'F'); a.add(b'r', x+w*0.50, y, 0.16+h*0.70, w*0.28, h*0.38, d*1.10, b'F'); a.add(b'k', x-w*0.48, y+d*0.26, 0.16, 0.18, h*1.62, 0., b'M'); }
        // Address Zero is a discontinuous suspended frame with a void through its centre.
        6 => { a.add(b'x', x-w*0.34, y, 0.42, w*0.32, 0.16, d*0.92, b'G'); a.add(b'x', x+w*0.34, y, 0.72, w*0.32, 0.16, d*0.92, b'G'); a.add(b'a', x, y, 0.96, w*1.12, h*0.72, d*0.22, b'F'); a.add(b's', x-w*0.48, y-d*0.24, 0.16, 0.12, h*1.84, 0., b'F'); a.add(b's', x+w*0.48, y+d*0.24, 0.16, 0.12, h*1.38, 0., b'G'); }
        // A broad columned exchange carries a long slate roof and clocklike tower.
        7 => { a.add(b'h', x, y, 0.16, w*1.28, h*0.66, d*1.10, b'b'); a.add(b'r', x, y, 0.16+h*0.64, w*1.38, h*0.46, d*1.20, b'k'); a.add(b'c', x-w*0.42, y-d*0.44, 0.16, 0.16, h*1.20, 0., b'g'); a.add(b'c', x, y-d*0.44, 0.16, 0.16, h*1.20, 0., b'g'); a.add(b'c', x+w*0.42, y-d*0.44, 0.16, 0.16, h*1.20, 0., b'g'); }
        // A field compound of shallow corrugated sheds, a tank, and a radio mast.
        8 => { a.add(b'h', x-w*0.23, y, 0.16, w*0.64, h*0.48, d*0.78, b'M'); a.add(b'r', x-w*0.23, y, 0.16+h*0.46, w*0.72, 0.26, d*0.88, b'M'); a.add(b'h', x+w*0.34, y+d*0.20, 0.16, w*0.42, h*0.40, d*0.54, b'M'); a.add(b'd', x+w*0.48, y-d*0.30, 0.16, w*0.24, h*1.20, 0., b'M'); a.add(b's', x-w*0.48, y-d*0.30, 0.16, 0.12, h*1.48, 0., b'F'); }
        // Living branches make an irregular canopy over clustered flowers and trunks.
        _ => { a.add(b'g', x, y, 0.16, w*1.18, 0.14, d, b'g'); a.add(b'a', x-w*0.30, y, 0.16, w*0.48, h*1.14, d*0.28, b'm'); a.add(b'a', x+w*0.30, y+d*0.12, 0.16, w*0.48, h*0.88, d*0.28, b'm'); a.add(b'D', x, y-d*0.22, 0.20, w*0.32, h*0.56, 0., b'G'); a.add(b'D', x+w*0.42, y+d*0.22, 0.20, w*0.26, h*0.46, 0., b'F'); }
    }
    if role != Role::Hall { return; }
    // Halls get their actual side volumes here. The old shared classical wings
    // are removed below, so a court reads as a ger camp, temple, laboratory,
    // dock pavilion, works hall, portal court, exchange, field base, or garden.
    match style {
        0 => { a.add(b'd', -1.28, 0.35, 0.16, 0.92, 0.42, 0., b'm'); a.add(b'D', -1.28, 0.35, 0.58, 1.02, 0.62, 0., b'F'); a.add(b'd', 1.28, 0.35, 0.16, 0.82, 0.36, 0., b'm'); a.add(b'D', 1.28, 0.35, 0.52, 0.92, 0.54, 0., b'F'); }
        1 => { a.add(b'T', -1.22, 0.42, 0.16, 0.94, 1.16, 0.84, b'n'); a.add(b'T', 1.22, 0.42, 0.16, 0.94, 1.16, 0.84, b'n'); }
        2 => { a.add(b'a', -1.28, 0.42, 0.16, 0.78, 1.48, 0.34, b's'); a.add(b'a', 1.28, 0.42, 0.16, 0.78, 1.48, 0.34, b's'); }
        3 => { a.add(b'd', -1.18, 0.42, 0.16, 0.92, 0.58, 0., b'M'); a.add(b'D', -1.18, 0.42, 0.74, 1.02, 0.62, 0., b'G'); a.add(b'd', 1.18, 0.42, 0.16, 0.92, 0.58, 0., b'M'); a.add(b'D', 1.18, 0.42, 0.74, 1.02, 0.62, 0., b'G'); }
        4 => { a.add(b'T', -1.25, 0.38, 0.16, 0.98, 1.30, 0.84, b'F'); a.add(b'T', 1.25, 0.38, 0.16, 0.98, 1.30, 0.84, b'F'); }
        5 => { a.add(b'd', -1.25, 0.35, 0.16, 0.92, 1.06, 0., b'b'); a.add(b'd', 1.25, 0.35, 0.16, 0.92, 1.06, 0., b'b'); }
        6 => { a.add(b'h', -1.25, 0.35, 0.16, 0.64, 2.18, 0.64, b'G'); a.add(b'h', 1.25, 0.35, 0.16, 0.64, 1.72, 0.64, b'G'); }
        7 => { a.add(b'h', -1.22, 0.35, 0.16, 0.98, 0.86, 0.78, b'b'); a.add(b'r', -1.22, 0.35, 1.02, 1.08, 0.46, 0.88, b'k'); a.add(b'h', 1.22, 0.35, 0.16, 0.98, 0.86, 0.78, b'b'); a.add(b'r', 1.22, 0.35, 1.02, 1.08, 0.46, 0.88, b'k'); }
        8 => { a.add(b'h', -1.24, 0.35, 0.16, 1.05, 0.58, 0.82, b'M'); a.add(b'd', 1.24, 0.35, 0.16, 0.88, 1.02, 0., b'M'); }
        _ => { a.add(b'g', -1.25, 0.35, 0.16, 0.98, 0.12, 0.84, b'g'); a.add(b'a', -1.25, 0.48, 0.16, 0.94, 1.22, 0.28, b'm'); a.add(b'g', 1.25, 0.35, 0.16, 0.98, 0.12, 0.84, b'g'); a.add(b'a', 1.25, 0.48, 0.16, 0.94, 1.22, 0.28, b'm'); }
    }
}
/// Secondary readable marker for the founder's craft at the forecourt.
fn house_mark(a: &mut Art, style: usize, role: Role) {
    let (x, y) = match role {
        Role::Landmark => (-1.7, 1.45), Role::Hall => (-1.45, 1.05),
        Role::Yard => (-1.0, 0.72), Role::Works => (-1.0, 0.62),
        Role::Bastion => (-1.25, 0.92), Role::Bay => (-1.05, 0.72),
    };
    match style {
        // Xanadu: felt gers, horse standards and low ceremonial rings.
        0 => { a.add(b'd', x, y, 0.16, 0.72, 0.34, 0., b'm'); a.add(b'D', x, y, 0.50, 0.78, 0.46, 0., b'F'); a.add(b's', x + 0.72, y, 0.16, 0.16, 1.18, 0., b'g'); }
        // Kurukshetra: shrine drums, lotus domes and bannered temple gates.
        1 => { a.add(b'd', x, y, 0.16, 0.62, 0.52, 0., b'n'); a.add(b'D', x, y, 0.68, 0.72, 0.48, 0., b'F'); a.add(b'c', x + 0.72, y - 0.18, 0.16, 0.14, 1.12, 0., b'g'); }
        // Sanctum: flying buttress, rose window light and a narrow Gothic finial.
        2 => { a.add(b'a', x, y, 0.16, 0.72, 1.15, 0.32, b's'); a.add(b's', x + 0.55, y + 0.12, 0.16, 0.28, 1.55, 0., b'F'); a.add(b'l', x, y - 0.42, 0.72, 0.22, 0.22, 0., b'g'); }
        // Bensalem: instrument dome, glass lantern and brass observatory arm.
        3 => { a.add(b'd', x, y, 0.16, 0.74, 0.34, 0., b'M'); a.add(b'D', x, y, 0.50, 0.82, 0.54, 0., b'G'); a.add(b'x', x + 0.55, y, 0.95, 0.78, 0.10, 0.12, b'g'); }
        // Longjiang: stacked eaves and a red harbour mast.
        4 => { a.add(b'T', x, y, 0.16, 0.82, 1.05, 0.82, b'F'); a.add(b'x', x, y, 1.20, 0.98, 0.12, 0.98, b'k'); a.add(b's', x + 0.68, y, 0.16, 0.14, 1.55, 0., b'F'); }
        // The Reading Room: brick drum, civic dome and a red banner.
        5 => { a.add(b'd', x, y, 0.16, 0.82, 0.64, 0., b'b'); a.add(b'D', x, y, 0.80, 0.90, 0.54, 0., b'F'); a.add(b'b', x + 0.72, y, 0.16, 0.22, 1.38, 0., b'F'); }
        // Address Zero: a ring portal, paired antennae and a glass signal.
        6 => { a.add(b'd', x, y, 0.16, 0.86, 0.18, 0., b'k'); a.add(b'D', x, y, 0.34, 0.90, 0.22, 0., b'G'); a.add(b's', x + 0.55, y, 0.16, 0.12, 1.46, 0., b'F'); a.add(b's', x - 0.55, y, 0.16, 0.12, 1.16, 0., b'F'); }
        // Kirkcaldy: slate warehouse cupola, copper clock and quay crane.
        7 => { a.add(b'h', x, y, 0.16, 0.82, 0.56, 0.72, b'b'); a.add(b'r', x, y, 0.72, 0.90, 0.42, 0.78, b'k'); a.add(b'd', x + 0.72, y, 0.16, 0.34, 1.22, 0., b'g'); }
        // Campion: corrugated water tank, wind vane and field antenna.
        8 => { a.add(b'd', x, y, 0.16, 0.66, 0.72, 0., b'M'); a.add(b'D', x, y, 0.88, 0.72, 0.22, 0., b'M'); a.add(b's', x + 0.72, y, 0.16, 0.12, 1.38, 0., b'F'); }
        // Flux: planted bowl, translucent canopy and an airy white arch.
        _ => { a.add(b'g', x, y, 0.16, 1.10, 0.12, 0.86, b'g'); a.add(b'a', x, y + 0.30, 0.16, 0.92, 1.06, 0.28, b'm'); a.add(b'D', x, y, 0.28, 0.68, 0.28, 0., b'G'); }
    }
}

/// A faction owns the site plan as well as its skyline.  Two broad pieces keep
/// landmark appearances inside the renderer's forty-piece budget.
fn house_footing(a: &mut Art, style: usize, role: Role, k: &Kit) {
    let (w, d) = match role { Role::Landmark | Role::Hall => (4.9, 4.5), Role::Yard | Role::Works => (3.5, 3.1), Role::Bastion => (3.7, 3.7), Role::Bay => (3.4, 3.0) };
    match style.min(9) {
        0 => { a.add(b'd', 0., 0., 0.02, w*0.82, 0.13, 0., b'm'); a.add(b'x', 0., -d*0.38, 0.15, w*0.34, 0.12, 0.24, b'g'); },
        1 => { a.add(b'p', 0., 0., 0.02, w, 0.16, d, b'n'); a.add(b'P', 0., 0.12, 0.18, w*0.68, 0.18, d*0.66, k.accent); },
        2 => { a.add(b'p', 0., 0.15, 0.02, w*0.70, 0.12, d, b's'); a.add(b'a', 0., -d*0.34, 0.14, w*0.46, 0.46, 0.22, b'F'); },
        3 => { a.add(b'd', 0., 0., 0.02, w*0.84, 0.14, 0., b'M'); a.add(b'x', 0., -d*0.36, 0.16, w*0.42, 0.10, 0.28, b'g'); },
        4 => { a.add(b'p', 0., 0.10, 0.02, w, 0.12, d*0.58, b'w'); a.add(b'w', 0., -d*0.34, 0.04, w*0.34, 0.18, 0.22, b'b'); },
        5 => { a.add(b'p', 0., 0., 0.02, w, 0.12, d, b'M'); a.add(b'x', 0., -d*0.42, 0.14, w*0.62, 0.12, 0.20, b'F'); },
        6 => { a.add(b'p', -w*0.20, 0., 0.10, w*0.54, 0.10, d*0.82, b'k'); a.add(b'x', w*0.23, 0., 0.28, w*0.42, 0.10, d*0.68, b'G'); },
        7 => { a.add(b'p', 0., 0., 0.02, w, 0.12, d, b'b'); a.add(b'x', 0., -d*0.40, 0.14, w*0.38, 0.12, 0.24, b'g'); },
        8 => { a.add(b'p', 0., 0., 0.02, w, 0.10, d, b'n'); a.add(b'x', 0., -d*0.36, 0.12, w*0.70, 0.10, 0.32, b'M'); },
        _ => { a.add(b'g', 0., 0., 0.02, w*0.94, 0.12, d*0.82, b'g'); a.add(b'a', 0., -d*0.28, 0.14, w*0.44, 0.52, 0.24, b'm'); },
    }
}

/// Site `style`'s building for `role`, with its design name. A house program holds only its own house's art (`art`,
/// from shared/buildings.txt), so it builds in its own style in any seat; the default program holds every house's.
pub fn architecture(style: usize, role: Role) -> Value {
    let line = super::art(style);
    let f: Vec<&str> = line.split('|').collect();
    let mut k = kit(f[0]);
    if f[0].len() < 6 {
        k.accent = match style {
            3 => b'g', // Bensalem's instrument brass
            4 => b'b', // Longjiang's red-lacquer timber
            5 => b'g', // Reading Room gilt
            6 => b'M', // Address Zero's illuminated alloy
            7 => b'g', // Kirkcaldy's exchange gold
            8 => b'g', // Campion's brass field fittings
            _ => b'g', // Flux's warm civic trim
        };
    }
    let mut a = Art(Vec::new());
    let i = ROLES.iter().position(|r| *r == role).unwrap_or(1);
    // A faction's footprint is an architectural choice, with no game effect.
    house_footing(&mut a, style, role, &k);
    house_massing(&mut a, style, role);
    // Service roles use the founder mass in place of the generic central shed.
    // Landmarks keep their authored plan; bastions keep their defensive curtain.
    let src = if i == 0 { f[7].to_string() } else {
        let s = DESIGNS[i - 1];
        match role {
            Role::Hall => "p,.1,,3.6,.35,3.1/B;p,-1.6,,1.4,.18,.45/B;b.85,-1.35,.35,.45,2;l.42,-1.42,.35,.16,.75/g".to_string(),
            Role::Yard => s.replace("h-.24,.76,.15,2.08,1.15,.8/V;=x-.24,.76,1.3,1.91,.14,.74/F;~r-.24,.76,1.3,2.18,.36,.9/F;", ""),
            Role::Works => s.replace("h-.34,.41,.15,1.65,1.3,1.05/X;r-.34,.41,1.45,1.73,.5,1.13/F;", ""),
            Role::Bay => s.replace("h,.45,.15,1.88,1.5,1.05/V;R,.45,1.65,1.875,1.05;", ""),
            _ => s.to_string(),
        }
    };
    draw(&mut a, &k, &src, 1.0);
    // Keep a complete landmark or bastion plan ahead of its optional forecourt
    // trim: the appearance cap must never drop its final tower or lamp.
    if matches!(role, Role::Landmark | Role::Bastion) { house_mark(&mut a, style, role); }
    a.value(f[1 + i])
}
