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
    let mut attacks = Value::arr();
    for &u in units {
        if attacks.len() >= 8 || cycles_left() < 4500.0 { break; }
        if u.get("air").truthy() || u.get("stalled").truthy()
            || matches!(u.get("type").str(), "miner" | "hauler" | "freighter" | "scout")
            || matches!(u.get("doing").str(), "gather" | "study") { continue; }
        let reach = if u.path("build.range").num_or(0.0) > 0.0 { 3.0 + u.path("build.range").num() } else { 1.5 };
        let conduct = u.get("conduct");
        let target = hostile.iter().copied().take_while(|_| cycles_left() > 4500.0).filter(|s| {
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
    let now = world.get("seconds").num_or(0.0);
    // Refresh at most every five seconds; short standing jobs cover the interval.
    if now.floor() % 5.0 != 0.0 && orders.get("memo").str() != "Recovering production" { return; }
    let sites = me.path("rts.structures");
    let mut own: Vec<View> = sites.iter().filter(|s| same(s.get("owner"), id) && s.get("state").str() != "ruin").collect();
    if !own.is_empty() { let start = (now as usize / 5) % own.len(); own.rotate_left(start); }
    let units: Vec<View> = world.get("units").iter().filter(|u| same(u.get("owner"), id) && u.get("hp").num_or(0.0) > 0.0).collect();
    let income = me.path("rts.cpu.income").num_or(0.0);
    let bank = me.path("rts.cpu.bank").num_or(0.0);
    let safe = orders.get("memo").str() != "Recovering production" && me.path("units.stalled").num_or(0.0) == 0.0
        && bank > income.max(1000.0 * price(me)) && me.path("rts.cpu.last.upkeep").num_or(0.0) < income * 0.7;
    let mut jobs = Value::arr();
    let mut services = Value::arr();
    for s in &own {
        if cycles_left() < 4500.0 || jobs.len() >= 24 { break; }
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
        if cycles_left() < 4000.0 { break; }
        // Keep unit fabrication and several seconds of income ahead of each new site.
        if bank < income * 2.0 + w.quote * 3.0 { continue; }
        let Some((x, y)) = spot(w.p, sites, w.r, units) else { continue; };
        let province = w.p.get("id").num();
        let height = match w.role { Role::Landmark => d.height + 2.0, Role::Hall => d.height, _ => (d.height - 1.0).max(2.5) };
        let integrity = if w.role == Role::Bastion { d.integrity * 1.3 } else { d.integrity };
        let key = format!("home-{province:.0}-{now:.0}");
        orders.set("structures", arr![obj! { "key" => key, "p" => province, "x" => x, "y" => y,
            "shell" => obj! { "radius" => w.r, "height" => height, "integrity" => integrity },
            "ports" => w.ports, "appearance" => architecture(style, w.role, w.r), "budget" => w.quote + pl }]);
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

#[derive(Clone, Copy)]
struct Pc { kind: &'static str, x: f64, y: f64, z: f64, w: f64, h: f64, d: f64, rot: f64, tone: &'static str }
impl Pc {
    fn d(&mut self, d: f64) -> &mut Self { self.d = d; self }
    fn rot(&mut self, r: f64) -> &mut Self { self.rot = r; self }
    fn tone(&mut self, t: &'static str) -> &mut Self { self.tone = t; self }
}
struct Art(Vec<Pc>);
impl Art {
    fn add(&mut self, kind: &'static str, x: f64, y: f64, z: f64, w: f64, h: f64) -> &mut Pc {
        self.0.push(Pc { kind, x, y, z, w, h, d: 0.0, rot: 0.0, tone: "" });
        let n = self.0.len() - 1;
        &mut self.0[n]
    }
    /// The engine's `appearance`: at most 40 pieces, inside its bounds, to the hundredth of a cell.
    fn value(&self, name: &str) -> Value {
        let r = |v: f64, lo: f64, hi: f64| (v.clamp(lo, hi) * 100.0).round() / 100.0;
        let mut pieces = Value::arr();
        for q in self.0.iter().take(40) {
            let mut o = obj! { "kind" => q.kind, "x" => r(q.x, -8.0, 8.0), "y" => r(q.y, -8.0, 8.0), "z" => r(q.z, 0.0, 12.0),
                "w" => r(q.w, 0.1, 12.0), "h" => r(q.h, 0.1, 16.0) };
            if q.d > 0.0 { o.set("d", r(q.d, 0.1, 12.0)); }
            if q.rot != 0.0 { o.set("rot", r(q.rot, -360.0, 360.0)); }
            if !q.tone.is_empty() { o.set("tone", q.tone); }
            pieces.push(o);
        }
        obj! { "name" => name, "pieces" => pieces }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Roof { Gable, Dome, Tiers, Stepped, Flat }
#[derive(Clone, Copy, PartialEq)]
enum Spire { Domed, Minaret, Needle, Observatory, Pagoda, Mast, Clock, Tank, Obelisk }
/// A house's building kit: its walls, terraces, roofs and towers.
struct Kit { wall: &'static str, base: &'static str, roof: Roof, spire: Spire, garden: bool, names: [&'static str; 6] }
fn kit(style: usize) -> Kit {
    let k = |wall, base, roof, spire, garden, names| Kit { wall, base, roof, spire, garden, names };
    match style {
        0 => k("marble", "stone", Roof::Dome, Spire::Domed, true,
            ["Pleasure dome", "Khan's court", "Caravan yard", "Ore camp", "Steppe bastion", "Horse lines"]),
        1 => k("sand", "sand", Roof::Stepped, Spire::Minaret, true,
            ["Temple of the bow", "Mandapa", "Chariot yard", "Forge ghat", "Bow relay", "Healing house"]),
        2 => k("stone", "stone", Roof::Gable, Spire::Needle, true,
            ["Sanctum cathedral", "Chapter house", "Commandery yard", "Abbey works", "Chapter keep", "Infirmary"]),
        3 => k("marble", "marble", Roof::Dome, Spire::Observatory, true,
            ["Salomon's House", "Research court", "Merchants' yard", "Furnace house", "Sea bastion", "Physic hall"]),
        4 => k("wood", "stone", Roof::Tiers, Spire::Pagoda, false,
            ["Longjiang pagoda", "Admiralty hall", "Treasure-ship yard", "Rope works", "Harbor fort", "Dry dock"]),
        5 => k("stone", "stone", Roof::Flat, Spire::Needle, false,
            ["Great Reading Room", "Assembly hall", "People's depot", "Red works", "Committee fort", "Workers' clinic"]),
        6 => k("glass", "dark", Roof::Flat, Spire::Mast, false,
            ["Address Zero", "Pointer court", "Cache depot", "Heap works", "Pointer spire", "Patch bay"]),
        7 => k("brick", "sand", Roof::Gable, Spire::Clock, false,
            ["Kirkcaldy exchange", "Exchange hall", "Exchange depot", "Pin factory", "Customs fort", "Mending house"]),
        8 => k("metal", "sand", Roof::Gable, Spire::Tank, false,
            ["Campion command", "Mess hall", "Supply dump", "Bore works", "Emu roost", "Field hospital"]),
        _ => k("marble", "marble", Roof::Flat, Spire::Obelisk, true,
            ["Flux commons", "Open courtyard", "Depot", "Works", "Bastion", "Workshop"]),
    }
}
/// The house roof over a block w × d centred at (x, y) whose walls end at z.
fn roof(a: &mut Art, k: &Kit, x: f64, y: f64, z: f64, w: f64, d: f64) {
    let m = w.min(d);
    match k.roof {
        Roof::Gable => { a.add("roof", x, y, z, w, (m * 0.5).min(1.1)).d(d).tone("faction"); }
        Roof::Dome => {
            a.add("accent", x, y, z, w + 0.06, 0.1).d(d + 0.06).tone(k.base);
            a.add("drum", x, y, z + 0.1, m * 0.6, 0.35).tone(k.wall);
            a.add("dome", x, y, z + 0.45, m * 0.64, m * 0.4).tone("faction");
        }
        Roof::Tiers => { a.add("tiers", x, y, z, w * 0.95, (m * 0.9).clamp(1.2, 2.4)).d(d * 0.95).tone("faction"); }
        Roof::Stepped => { a.add("pyramid", x, y, z, w * 0.92, (m * 0.75).clamp(0.9, 2.2)).d(d * 0.92).tone("faction"); }
        Roof::Flat => { a.add("accent", x, y, z, w * 0.9, 0.16).d(d * 0.9).tone("faction"); }
    }
}
/// A house tower w across and h tall at (x, y), standing on z.
fn spire(a: &mut Art, k: &Kit, x: f64, y: f64, z: f64, w: f64, h: f64) {
    match k.spire {
        Spire::Domed => { a.add("drum", x, y, z, w, h).tone(k.wall); a.add("dome", x, y, z + h, w * 1.08, w * 0.62).tone("faction"); }
        Spire::Minaret => {
            a.add("drum", x, y, z, w * 0.72, h).tone(k.wall);
            a.add("drum", x, y, z + h * 0.72, w * 0.95, 0.12).tone("gold");
            a.add("dome", x, y, z + h, w * 0.8, w * 1.25).tone("faction");
        }
        Spire::Needle => { a.add("tower", x, y, z, w, h).tone(k.wall); a.add("spire", x, y, z + h + 0.22, w * 0.95, h * 0.6).tone("faction"); }
        Spire::Observatory => { a.add("tower", x, y, z, w, h).tone(k.wall); a.add("dome", x, y, z + h + 0.22, w * 0.88, w * 0.55).tone("glass"); }
        Spire::Pagoda => { a.add("tiers", x, y, z, w * 1.3, h * 1.1).tone("faction"); }
        Spire::Mast => {
            a.add("hall", x, y, z, w * 0.85, h).d(w * 0.85).tone("glass");
            a.add("accent", x, y, z + h, w * 0.6, 0.25).d(w * 0.6).tone("faction");
            a.add("light", x, y, z + h + 0.25, 0.2, h * 0.45).tone("metal");
        }
        Spire::Clock => {
            a.add("tower", x, y, z, w, h).tone(k.wall);
            a.add("accent", x, y - w / 2.0 - 0.03, z + h - w * 0.75, w * 0.55, w * 0.55).d(0.06).tone("gold");
            a.add("spire", x, y, z + h + 0.22, w * 0.9, h * 0.4).tone("faction");
        }
        Spire::Tank => {
            a.add("column", x, y, z, 0.16, h * 0.7).tone("metal");
            a.add("drum", x, y, z + h * 0.7, w * 1.2, h * 0.3).tone("metal");
            a.add("spire", x, y, z + h, w * 1.25, w * 0.4).tone("faction");
        }
        Spire::Obelisk => { a.add("statue", x, y, z, w * 0.9, h * 1.1).tone("marble"); }
    }
}

/// Site `style`'s building for `role` on a shell of radius `r`, with its design name.
pub fn architecture(style: usize, role: Role, r: f64) -> Value {
    let k = kit(style);
    let mut a = Art(Vec::new());
    let name = match role {
        Role::Landmark => { landmark(&mut a, &k, style, r / 2.5); k.names[0] }
        Role::Hall => { hall(&mut a, &k, r / 2.0); k.names[1] }
        Role::Yard => { yard(&mut a, &k, r / 2.0); k.names[2] }
        Role::Works => { works(&mut a, &k, r / 2.0); k.names[3] }
        Role::Bastion => { bastion(&mut a, &k, r / 2.0); k.names[4] }
        Role::Bay => { bay(&mut a, &k, r / 2.0); k.names[5] }
    };
    a.value(name)
}

/// A civic hall: a terrace and steps, the hall under the house roof, a portico, wings, two towers, greens and banners.
fn hall(a: &mut Art, k: &Kit, f: f64) {
    a.add("platform", 0.0, 0.1 * f, 0.0, 3.6 * f, 0.35).d(3.1 * f).tone(k.base);
    a.add("platform", 0.0, -1.6 * f, 0.0, 1.4 * f, 0.18).d(0.45).tone(k.base);
    a.add("hall", 0.0, 0.4 * f, 0.35, 2.2 * f, 1.5).d(1.5 * f).tone(k.wall);
    roof(a, k, 0.0, 0.4 * f, 1.85, 2.2 * f, 1.5 * f);
    for x in [-0.9, -0.3, 0.3, 0.9] { a.add("column", x * f, -0.62 * f, 0.35, 0.2, 1.25).tone("marble"); }
    a.add("accent", 0.0, -0.62 * f, 1.6, 2.1 * f, 0.2).d(0.42).tone(k.wall);
    for s in [-1.0, 1.0] {
        a.add("hall", s * 1.35 * f, 0.65 * f, 0.35, 0.75 * f, 1.0).d(1.2 * f).tone(k.wall);
        a.add("accent", s * 1.35 * f, 0.65 * f, 1.35, 0.8 * f, 0.14).d(1.25 * f).tone("faction");
        spire(a, k, s * 1.4 * f, 1.25 * f, 0.35, 0.55, 2.3);
        if k.garden { a.add("garden", s * 1.2 * f, -1.0 * f, 0.35, 0.9 * f, 0.1).d(0.8 * f); }
        else { a.add("statue", s * 1.25 * f, -1.05 * f, 0.35, 0.42, 1.5).tone("dark"); }
        a.add("banner", s * 0.85 * f, -1.35 * f, 0.35, 0.45, 2.0);
        a.add("light", s * 0.42 * f, -1.42 * f, 0.35, 0.16, 0.75).tone("gold");
    }
}
/// A supply yard: warehouses under the house colour, stacked crates, a crane, a fence and a gate.
fn yard(a: &mut Art, k: &Kit, f: f64) {
    let shed = |a: &mut Art, x: f64, y: f64, w: f64, h: f64, d: f64| {
        a.add("hall", x, y, 0.15, w, h).d(d).tone(if k.wall == "glass" { "metal" } else { k.wall });
        if k.roof == Roof::Flat { a.add("accent", x, y, 0.15 + h, w * 0.92, 0.14).d(d * 0.92).tone("faction"); }
        else { a.add("roof", x, y, 0.15 + h, w + 0.1, (w.min(d) * 0.45).min(0.6)).d(d + 0.1).tone("faction"); }
    };
    a.add("platform", 0.0, 0.0, 0.0, 3.5 * f, 0.15).d(3.1 * f).tone(k.base);
    shed(a, -0.3 * f, 0.95 * f, 2.6 * f, 1.15, 1.0 * f);
    shed(a, -1.25 * f, -0.45 * f, 0.9 * f, 0.9, 1.5 * f);
    for (x, y, z) in [(0.15, -0.45, 0.15), (0.5, -0.45, 0.15), (0.32, -0.45, 0.47), (0.15, -0.9, 0.15), (0.85, 0.1, 0.15), (0.85, -0.3, 0.15)] {
        a.add("accent", x * f, y * f, z, 0.32, 0.32).tone("wood");
    }
    a.add("column", 1.3 * f, 0.3 * f, 0.15, 0.14, 2.4).tone("metal");
    a.add("accent", 0.85 * f, 0.1 * f, 2.45, 1.5, 0.1).d(0.12).rot(-25.0).tone("metal");
    a.add("wall", 0.9 * f, -1.5 * f, 0.15, 1.3 * f, 0.45).d(0.12).tone("wood");
    a.add("arch", -0.35 * f, -1.5 * f, 0.15, 0.9 * f, 1.0).d(0.3).tone(k.base);
    a.add("banner", 1.55 * f, -1.3 * f, 0.15, 0.45, 1.8);
    a.add("light", 1.5 * f, 1.0 * f, 0.15, 0.16, 1.3).tone("gold");
}
/// Extraction works: a shed, smoking stacks, tanks, a headframe and heaps of ore.
fn works(a: &mut Art, k: &Kit, f: f64) {
    a.add("platform", 0.0, 0.0, 0.0, 3.5 * f, 0.15).d(3.1 * f).tone("dark");
    a.add("hall", -0.45 * f, 0.55 * f, 0.15, 2.2 * f, 1.3).d(1.4 * f).tone(if k.wall == "glass" { "metal" } else { "brick" });
    a.add("roof", -0.45 * f, 0.55 * f, 1.45, 2.3 * f, 0.5).d(1.5 * f).tone("faction");
    a.add("stack", 0.95 * f, 1.1 * f, 0.15, 0.42, 2.3).tone("brick");
    a.add("stack", 1.4 * f, 0.6 * f, 0.15, 0.36, 1.8).tone("brick");
    for (x, y, w, h) in [(1.15, -0.55, 0.8, 0.9), (0.4, -1.05, 0.6, 0.7)] {
        a.add("drum", x * f, y * f, 0.15, w, h).tone("metal");
        a.add("dome", x * f, y * f, 0.15 + h, w, w * 0.35).tone("metal");
    }
    a.add("tower", -1.35 * f, -0.3 * f, 0.15, 0.45, 2.1).tone("metal");
    a.add("roof", -1.35 * f, -0.3 * f, 2.47, 0.6, 0.3).d(0.6).tone("faction");
    a.add("pyramid", -1.05 * f, -1.15 * f, 0.15, 0.9, 0.5).tone("dark");
    a.add("pyramid", -0.35 * f, -1.25 * f, 0.15, 0.6, 0.35).tone("sand");
    a.add("light", 0.35 * f, -0.2 * f, 0.15, 0.16, 1.2).tone("gold");
    a.add("banner", 1.55 * f, -1.25 * f, 0.15, 0.45, 1.6);
}
/// A bastion: curtain walls round a keep (the battery stands on top), towers at the corners and a gatehouse.
fn bastion(a: &mut Art, k: &Kit, f: f64) {
    let e = 1.4 * f;
    a.add("platform", 0.0, 0.0, 0.0, 3.4 * f, 0.3).d(3.4 * f).tone("stone");
    a.add("wall", 0.0, e, 0.3, 2.8 * f, 1.1).d(0.35).tone("stone");
    a.add("wall", -e, 0.0, 0.3, 2.8 * f, 1.1).d(0.35).rot(90.0).tone("stone");
    a.add("wall", e, 0.0, 0.3, 2.8 * f, 1.1).d(0.35).rot(90.0).tone("stone");
    for s in [-1.0, 1.0] { a.add("wall", s * 0.95 * f, -e, 0.3, 0.95 * f, 1.1).d(0.35).tone("stone"); }
    a.add("arch", 0.0, -e, 0.3, 0.95 * f, 1.5).d(0.5).tone(k.wall);
    a.add("roof", 0.0, -e, 1.8, 1.05 * f, 0.35).d(0.6).tone("faction");
    for (x, y) in [(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] { spire(a, k, x * e, y * e, 0.3, 0.7, 1.8); }
    a.add("hall", -0.55 * f, 0.55 * f, 0.3, 1.0 * f, 0.9).d(0.8 * f).tone(k.wall);
    a.add("tower", 0.25 * f, 0.15 * f, 0.3, 1.2 * f, 2.5).tone("stone");
    a.add("banner", 0.65 * f, 0.55 * f, 3.02, 0.45, 1.1);
    a.add("light", -0.55 * f, -0.75 * f, 0.3, 0.16, 0.8).tone("gold");
}
/// A repair bay: a workshop with a great door, a gantry crane, lamps and spares.
fn bay(a: &mut Art, k: &Kit, f: f64) {
    a.add("platform", 0.0, 0.0, 0.0, 3.4 * f, 0.15).d(3.0 * f).tone(k.base);
    a.add("hall", 0.0, 0.6 * f, 0.15, 2.5 * f, 1.5).d(1.4 * f).tone(if k.wall == "glass" { "metal" } else { k.wall });
    roof(a, k, 0.0, 0.6 * f, 1.65, 2.5 * f, 1.4 * f);
    a.add("arch", 0.0, -0.12 * f, 0.15, 1.5 * f, 1.4).d(0.2).tone(k.base);
    for x in [-1.2, 1.2] { a.add("column", x * f, -0.9 * f, 0.15, 0.14, 2.0).tone("metal"); }
    a.add("accent", 0.0, -0.9 * f, 2.15, 2.6 * f, 0.14).d(0.16).tone("faction");
    for x in [-0.6, 0.6] { a.add("light", x * f, -1.3 * f, 0.15, 0.18, 0.9).tone("gold"); }
    for (x, y) in [(-1.35, -0.25), (-1.0, -0.4), (1.35, -0.3)] { a.add("accent", x * f, y * f, 0.15, 0.3, 0.3).tone("wood"); }
    a.add("banner", 1.5 * f, 1.25 * f, 0.15, 0.45, 2.0);
}

/// Each house's capital landmark (radius 2.5 at f = 1).
fn landmark(a: &mut Art, k: &Kit, style: usize, f: f64) {
    let s = f;
    match style {
        0 => { // Xanadu: a stately pleasure-dome on a round terrace, gers at its corners, gardens and a gate
            a.add("drum", 0.0, 0.0, 0.0, 4.8 * s, 0.35).tone("stone");
            a.add("drum", 0.0, 0.0, 0.35, 3.9 * s, 0.3).tone("marble");
            for (x, y) in [(-1.9, 0.0), (1.9, 0.0), (0.0, 1.9)] { a.add("garden", x * s, y * s, 0.35, 0.75 * s, 0.1).d(0.75 * s); }
            a.add("drum", 0.0, 0.15 * s, 0.65, 2.3 * s, 1.3).tone("marble");
            a.add("drum", 0.0, 0.15 * s, 1.95, 2.1 * s, 0.22).tone("gold");
            a.add("dome", 0.0, 0.15 * s, 2.17, 2.3 * s, 1.2).tone("faction");
            for (x, y) in [(-1.3, -1.3), (1.3, -1.3), (-1.3, 1.3), (1.3, 1.3)] {
                a.add("drum", x * s, y * s, 0.65, 0.8 * s, 0.5).tone("marble");
                a.add("dome", x * s, y * s, 1.15, 0.86 * s, 0.5).tone("faction");
            }
            a.add("arch", 0.0, -2.05 * s, 0.35, 1.3 * s, 1.5).d(0.5).tone("marble");
            a.add("roof", 0.0, -2.05 * s, 1.85, 1.5 * s, 0.45).d(0.7).tone("faction");
            for x in [-0.55, 0.55] { a.add("column", x * s, -1.35 * s, 0.65, 0.18, 1.2).tone("gold"); }
            for i in 0..6 {
                let t = (i as f64 * 45.0 + 67.5).to_radians();
                a.add("wall", 2.3 * s * t.cos(), 2.3 * s * t.sin(), 0.35, 1.5 * s, 0.45).d(0.2).rot(i as f64 * 45.0 + 67.5 + 90.0).tone("marble");
            }
            for (x, y) in [(-2.0, -1.0), (2.0, -1.0), (-1.6, 1.7), (1.6, 1.7)] { a.add("banner", x * s, y * s, 0.35, 0.5, 2.4); }
            for x in [-0.9, 0.9] { a.add("light", x * s, -2.1 * s, 0.35, 0.18, 0.9).tone("gold"); }
        }
        1 => { // Kurukshetra: a temple on a raised plinth, its stepped tower over the sanctum, a pillared hall, shrines, a tank
            a.add("platform", 0.0, 0.1 * s, 0.0, 4.6 * s, 0.35).d(4.2 * s).tone("sand");
            a.add("platform", 0.0, 0.45 * s, 0.35, 3.3 * s, 0.35).d(2.9 * s).tone("sand");
            a.add("hall", 0.0, 0.95 * s, 0.7, 1.4 * s, 1.3).d(1.4 * s).tone("sand");
            a.add("pyramid", 0.0, 0.95 * s, 2.0, 1.55 * s, 2.8).d(1.55 * s).tone("faction");
            a.add("dome", 0.0, 0.95 * s, 4.8, 0.5, 0.32).tone("gold");
            for x in [-0.65, 0.0, 0.65] { for y in [-0.05, -0.65] { a.add("column", x * s, y * s, 0.7, 0.18, 1.1).tone("sand"); } }
            a.add("accent", 0.0, -0.35 * s, 1.8, 1.8 * s, 0.16).d(1.05 * s).tone("sand");
            a.add("pyramid", 0.0, -0.35 * s, 1.96, 1.4 * s, 0.9).d(0.85 * s).tone("faction");
            for (x, y) in [(-1.3, 1.35), (1.3, 1.35), (-1.3, -0.4), (1.3, -0.4)] {
                a.add("hall", x * s, y * s, 0.7, 0.6, 0.6).d(0.6).tone("sand");
                a.add("pyramid", x * s, y * s, 1.3, 0.62, 1.0).d(0.62).tone("faction");
            }
            a.add("platform", 0.0, -1.65 * s, 0.0, 1.8 * s, 0.1).d(0.7 * s).tone("glass");
            for x in [-1.75, 1.75] { a.add("column", x * s, -1.55 * s, 0.35, 0.16, 2.8).tone("gold"); a.add("banner", x * s, -1.55 * s, 2.8, 0.4, 0.6); }
            for x in [-0.6, 0.6] { a.add("light", x * s, -1.15 * s, 0.35, 0.16, 0.7).tone("gold"); }
            for x in [-1.9, 1.9] { a.add("garden", x * s, 0.5 * s, 0.35, 0.6 * s, 0.1).d(1.0 * s); }
        }
        2 => { // Sanctum: a cathedral with a long nave, a transept, twin west towers, a crossing spire, buttresses and a cloister
            a.add("platform", 0.0, 0.0, 0.0, 4.4 * s, 0.3).d(4.6 * s).tone("stone");
            a.add("hall", 0.0, 0.1 * s, 0.3, 1.4 * s, 2.2).d(3.1 * s).tone("stone");
            a.add("roof", 0.0, 0.1 * s, 2.5, 1.5 * s, 1.0).d(3.2 * s).tone("faction");
            a.add("hall", 0.0, 0.55 * s, 0.3, 3.0 * s, 2.0).d(1.0 * s).tone("stone");
            a.add("roof", 0.0, 0.55 * s, 2.3, 3.1 * s, 0.9).d(1.1 * s).tone("faction");
            a.add("drum", 0.0, 1.7 * s, 0.3, 1.35 * s, 1.9).tone("stone");
            a.add("spire", 0.0, 1.7 * s, 2.2, 1.45 * s, 0.8).tone("faction");
            a.add("tower", 0.0, 0.55 * s, 2.3, 0.8 * s, 1.3).tone("stone");
            a.add("spire", 0.0, 0.55 * s, 3.82, 0.8 * s, 2.6).tone("faction");
            for x in [-0.72, 0.72] { a.add("tower", x * s, -1.55 * s, 0.3, 0.68 * s, 3.3).tone("stone"); a.add("spire", x * s, -1.55 * s, 3.82, 0.68 * s, 1.7).tone("faction"); }
            a.add("arch", 0.0, -1.62 * s, 0.3, 0.76 * s, 1.7).d(0.4).tone("marble");
            for x in [-0.82, 0.82] { for y in [-0.85, -0.25, 1.15] { a.add("accent", x * s, y * s, 0.3, 0.24, 1.7).d(0.5).tone("stone"); } }
            a.add("garden", 1.5 * s, -0.75 * s, 0.3, 1.0 * s, 0.1).d(1.2 * s);
            a.add("wall", 2.05 * s, -0.75 * s, 0.3, 1.3 * s, 0.75).d(0.22).rot(90.0).tone("stone");
            a.add("statue", -1.55 * s, -1.0 * s, 0.3, 0.5, 1.6).tone("marble");
            for x in [-1.5, 1.5] { a.add("banner", x * s, -2.0 * s, 0.3, 0.5, 2.2); }
            for x in [-0.32, 0.32] { a.add("light", x * s, -2.05 * s, 0.3, 0.16, 0.7).tone("gold"); }
        }
        3 => { // Bensalem: Salomon's House, a glass-domed hall of experiments between two high towers, observatories, a pool
            a.add("platform", 0.0, 0.0, 0.0, 4.6 * s, 0.3).d(4.4 * s).tone("marble");
            a.add("hall", 0.0, 0.35 * s, 0.3, 2.5 * s, 1.6).d(1.7 * s).tone("marble");
            a.add("accent", 0.0, 0.35 * s, 1.9, 2.6 * s, 0.12).d(1.8 * s).tone("marble");
            a.add("drum", 0.0, 0.35 * s, 2.05, 1.3 * s, 0.5).tone("marble");
            a.add("dome", 0.0, 0.35 * s, 2.55, 1.4 * s, 0.85).tone("glass");
            for (x, h) in [(-1.55, 4.0), (1.55, 3.2)] {
                a.add("tower", x * s, 1.35 * s, 0.3, 0.7 * s, h).tone("marble");
                a.add("dome", x * s, 1.35 * s, 0.52 + h, 0.62 * s, 0.42).tone("faction");
            }
            for x in [-0.95, -0.57, -0.19, 0.19, 0.57, 0.95] { a.add("column", x * s, -0.75 * s, 0.3, 0.17, 1.35).tone("marble"); }
            a.add("roof", 0.0, -0.75 * s, 1.65, 2.3 * s, 0.45).d(0.5 * s).tone("marble");
            for x in [-1.7, 1.7] {
                a.add("drum", x * s, -1.3 * s, 0.3, 0.75 * s, 0.9).tone("marble");
                a.add("dome", x * s, -1.3 * s, 1.2, 0.78 * s, 0.45).tone("glass");
                a.add("garden", x * s, 0.1 * s, 0.3, 0.75 * s, 0.1).d(0.8 * s);
            }
            a.add("platform", 0.0, -1.65 * s, 0.3, 1.6 * s, 0.05).d(0.55 * s).tone("glass");
            a.add("statue", 0.0, -2.05 * s, 0.0, 0.4, 1.5).tone("gold");
            for x in [-1.0, 1.0] { a.add("banner", x * s, -2.0 * s, 0.3, 0.45, 2.0); }
        }
        4 => { // Longjiang: a great pagoda, a lighthouse, shipyard sheds, a basin with a junk, a gate of honour
            a.add("platform", 0.0, 0.0, 0.0, 4.6 * s, 0.3).d(4.2 * s).tone("stone");
            a.add("platform", -0.3 * s, 0.65 * s, 0.3, 1.9 * s, 0.3).d(1.9 * s).tone("marble");
            a.add("tiers", -0.3 * s, 0.65 * s, 0.6, 1.8 * s, 4.6).tone("faction");
            a.add("drum", 1.6 * s, 1.15 * s, 0.3, 0.75 * s, 3.0).tone("marble");
            a.add("drum", 1.6 * s, 1.15 * s, 3.3, 0.95 * s, 0.14).tone("dark");
            a.add("light", 1.6 * s, 1.15 * s, 3.44, 0.42, 0.3).tone("gold");
            a.add("dome", 1.6 * s, 1.15 * s, 3.74, 0.6 * s, 0.4).tone("faction");
            a.add("hall", -1.65 * s, -0.45 * s, 0.3, 0.95 * s, 1.1).d(2.3 * s).tone("wood");
            a.add("roof", -1.65 * s, -0.45 * s, 1.4, 1.05 * s, 0.55).d(2.4 * s).tone("faction");
            a.add("platform", 0.65 * s, -1.4 * s, 0.3, 2.5 * s, 0.05).d(1.05 * s).tone("glass");
            a.add("accent", 0.65 * s, -1.4 * s, 0.35, 1.5 * s, 0.3).d(0.42 * s).tone("wood");
            a.add("accent", 0.15 * s, -1.4 * s, 0.65, 0.4, 0.22).d(0.38 * s).tone("wood");
            for x in [0.5, 1.0] { a.add("banner", x * s, -1.4 * s, 0.65, 0.42, 1.4); }
            for x in [-1.05, -0.35] { a.add("column", x * s, -1.75 * s, 0.3, 0.16, 1.3).tone("faction"); }
            a.add("roof", -0.7 * s, -1.75 * s, 1.6, 1.05 * s, 0.35).d(0.32).tone("faction");
            a.add("tiers", 1.65 * s, -0.15 * s, 0.3, 0.75 * s, 2.3).tone("faction");
            for (x, y) in [(-0.45, -0.75), (0.25, -0.75), (-1.75, 1.6)] { a.add("light", x * s, y * s, 0.3, 0.18, 0.7).tone("faction"); }
            for (x, y) in [(1.85, -1.4), (1.5, -1.6), (1.85, -1.7)] { a.add("accent", x * s, y * s, 0.35, 0.28, 0.28).tone("wood"); }
            a.add("garden", 0.85 * s, 0.35 * s, 0.3, 0.7 * s, 0.1).d(0.7 * s);
        }
        5 => { // The Reading Room: a great domed drum in a court, an eight-column portico, monuments, banners, a lattice tower
            a.add("platform", 0.0, 0.0, 0.0, 4.6 * s, 0.35).d(4.4 * s).tone("stone");
            a.add("drum", 0.0, 0.4 * s, 0.35, 2.6 * s, 1.5).tone("stone");
            a.add("drum", 0.0, 0.4 * s, 1.85, 2.35 * s, 0.22).tone("marble");
            a.add("dome", 0.0, 0.4 * s, 2.07, 2.45 * s, 1.1).tone("faction");
            for x in [-1.75, 1.75] {
                a.add("hall", x * s, 0.35 * s, 0.35, 0.8 * s, 1.6).d(2.8 * s).tone("stone");
                a.add("accent", x * s, 0.35 * s, 1.95, 0.85 * s, 0.14).d(2.85 * s).tone("faction");
                a.add("banner", x * s, -0.95 * s, 1.95, 0.5, 1.3);
            }
            for i in 0..8 { a.add("column", (-1.4 + i as f64 * 0.4) * s, -1.3 * s, 0.35, 0.19, 1.55).tone("marble"); }
            a.add("accent", 0.0, -1.3 * s, 1.9, 3.15 * s, 0.2).d(0.5).tone("marble");
            a.add("roof", 0.0, -1.3 * s, 2.1, 3.15 * s, 0.55).d(0.55).tone("marble");
            for x in [-1.85, 1.85] { a.add("statue", x * s, -1.95 * s, 0.0, 0.5, 2.3).tone("dark"); }
            for (z, w) in [(0.35, 0.8), (1.55, 0.62), (2.55, 0.45)] { a.add("drum", 1.75 * s, 1.95 * s, z, w, if z < 1.0 { 1.2 } else { 1.0 }).tone("metal"); }
            a.add("spire", 1.75 * s, 1.95 * s, 3.55, 0.45, 1.2).tone("faction");
            for x in [-0.5, 0.5] { a.add("light", x * s, -1.85 * s, 0.35, 0.16, 0.7).tone("gold"); }
        }
        6 => { // Address Zero: three glass towers of different heights, a pool, the zero ring, masts and lamps
            a.add("platform", 0.0, 0.0, 0.0, 4.6 * s, 0.25).d(4.4 * s).tone("dark");
            a.add("platform", 0.35 * s, -1.45 * s, 0.25, 2.4 * s, 0.05).d(0.9 * s).tone("glass");
            for (x, y, w, h) in [(-1.15, 0.95, 1.1, 5.2), (0.35, 1.25, 1.2, 3.8), (1.5, 0.25, 0.9, 2.8)] {
                a.add("hall", x * s, y * s, 0.25, w * s, h).d(w * s).tone("glass");
                a.add("accent", x * s, y * s, 0.25 + h * 0.55, w * s + 0.08, 0.12).d(w * s + 0.08).tone("dark");
                a.add("accent", x * s, y * s, 0.25 + h, w * s * 0.7, 0.3).d(w * s * 0.7).tone("faction");
                a.add("light", x * s, y * s, 0.55 + h, 0.16, 0.9).tone("faction");
            }
            a.add("drum", -1.25 * s, -1.2 * s, 0.25, 1.2 * s, 0.22).tone("faction");
            a.add("drum", -1.25 * s, -1.2 * s, 0.47, 0.75 * s, 0.3).tone("dark");
            a.add("light", -1.25 * s, -1.2 * s, 0.77, 0.3, 0.2).tone("faction");
            for (x, y) in [(-0.3, -0.55), (1.3, -0.75), (-1.95, 0.0), (1.95, -1.6)] { a.add("light", x * s, y * s, 0.25, 0.16, 1.1).tone("metal"); }
            a.add("hall", -0.2 * s, -0.25 * s, 0.25, 1.0 * s, 0.7).d(0.6 * s).tone("dark");
            a.add("column", 0.35 * s, 1.25 * s, 4.35, 0.08, 1.4).tone("metal");
        }
        7 => { // Kirkcaldy: an exchange with a portico and gilded dome, a clock tower, brick warehouses, a crane and crates
            a.add("platform", 0.0, 0.0, 0.0, 4.6 * s, 0.3).d(4.2 * s).tone("sand");
            a.add("hall", 0.0, 0.45 * s, 0.3, 2.8 * s, 1.8).d(1.6 * s).tone("sand");
            a.add("roof", 0.0, 0.45 * s, 2.1, 2.9 * s, 0.6).d(1.7 * s).tone("faction");
            for x in [-1.0, -0.6, -0.2, 0.2, 0.6, 1.0] { a.add("column", x * s, -0.62 * s, 0.3, 0.18, 1.5).tone("marble"); }
            a.add("roof", 0.0, -0.62 * s, 1.8, 2.4 * s, 0.45).d(0.5).tone("marble");
            a.add("drum", 0.0, 0.45 * s, 2.45, 0.9 * s, 0.5).tone("marble");
            a.add("dome", 0.0, 0.45 * s, 2.95, 0.95 * s, 0.6).tone("gold");
            a.add("tower", 1.6 * s, 1.35 * s, 0.3, 0.75 * s, 3.9).tone("brick");
            a.add("accent", 1.6 * s, 1.35 * s - 0.42 * s, 3.45, 0.5, 0.5).d(0.06).tone("gold");
            a.add("spire", 1.6 * s, 1.35 * s, 4.42, 0.75 * s, 1.3).tone("faction");
            a.add("hall", -1.7 * s, -0.35 * s, 0.3, 0.9 * s, 1.3).d(2.0 * s).tone("brick");
            a.add("roof", -1.7 * s, -0.35 * s, 1.6, 1.0 * s, 0.5).d(2.1 * s).tone("faction");
            for (x, y, z) in [(1.2, -1.2, 0.3), (1.55, -1.2, 0.3), (1.37, -1.2, 0.62), (1.85, -0.75, 0.3)] { a.add("accent", x * s, y * s, z, 0.32, 0.32).tone("wood"); }
            a.add("column", 1.95 * s, -0.2 * s, 0.3, 0.14, 2.5).tone("metal");
            a.add("accent", 1.5 * s, -0.4 * s, 2.55, 1.4, 0.1).d(0.12).rot(-30.0).tone("metal");
            for x in [-1.0, 1.0] { a.add("banner", x * s, -1.6 * s, 0.3, 0.45, 2.0); }
            for x in [-0.45, 0.45] { a.add("light", x * s, -1.65 * s, 0.3, 0.16, 0.8).tone("gold"); }
            a.add("statue", 0.0, -1.85 * s, 0.0, 0.45, 1.4).tone("dark");
        }
        8 => { // Campion: a corrugated headquarters, barracks, a water tower, a windmill, a lookout and a parade ground
            a.add("platform", 0.0, 0.0, 0.0, 4.6 * s, 0.15).d(4.4 * s).tone("sand");
            a.add("hall", -0.3 * s, 0.65 * s, 0.15, 2.7 * s, 1.2).d(1.3 * s).tone("metal");
            a.add("roof", -0.3 * s, 0.65 * s, 1.35, 2.8 * s, 0.45).d(1.4 * s).tone("faction");
            a.add("hall", -1.7 * s, -0.6 * s, 0.15, 0.85 * s, 0.9).d(1.8 * s).tone("wood");
            a.add("roof", -1.7 * s, -0.6 * s, 1.05, 0.95 * s, 0.4).d(1.9 * s).tone("faction");
            for (dx, dy) in [(-0.35, -0.35), (0.35, -0.35), (-0.35, 0.35), (0.35, 0.35)] { a.add("column", 1.55 * s + dx, 1.35 * s + dy, 0.15, 0.1, 2.2).tone("metal"); }
            a.add("drum", 1.55 * s, 1.35 * s, 2.35, 1.0, 0.8).tone("metal");
            a.add("spire", 1.55 * s, 1.35 * s, 3.15, 1.05, 0.4).tone("faction");
            a.add("column", 1.7 * s, -0.55 * s, 0.15, 0.12, 2.8).tone("metal");
            a.add("accent", 1.7 * s, -0.62 * s, 2.86, 1.3, 0.08).d(0.06).tone("metal");
            a.add("accent", 1.7 * s, -0.62 * s, 2.25, 0.08, 1.3).d(0.06).tone("metal");
            a.add("tower", -1.65 * s, 1.5 * s, 0.15, 0.65, 3.3).tone("wood");
            a.add("roof", -1.65 * s, 1.5 * s, 3.67, 0.85, 0.35).d(0.85).tone("faction");
            a.add("platform", 0.2 * s, -1.0 * s, 0.15, 1.8 * s, 0.04).d(1.2 * s).tone("dark");
            a.add("column", 0.2 * s, -1.0 * s, 0.19, 0.08, 2.9).tone("metal");
            a.add("banner", 0.2 * s, -1.0 * s, 3.09, 0.6, 0.1);
            a.add("statue", 0.95 * s, -1.75 * s, 0.15, 0.45, 1.3).tone("dark");
            for (x, w) in [(-0.6, 1.6), (1.2, 1.4)] { a.add("wall", x * s, -2.1 * s, 0.15, w * s, 0.35).d(0.08).tone("wood"); }
            for (x, y, z) in [(0.85, 0.0, 0.15), (1.15, 0.0, 0.15), (1.0, 0.0, 0.45)] { a.add("accent", x * s, y * s, z, 0.3, 0.3).tone("wood"); }
            a.add("light", -0.85 * s, -0.25 * s, 0.15, 0.16, 1.1).tone("gold");
        }
        _ => { // Flux: white blocks cantilevered over a garden court, a glass gallery, pilotis, a pool and an obelisk
            a.add("platform", 0.0, 0.0, 0.0, 4.6 * s, 0.25).d(4.4 * s).tone("marble");
            a.add("hall", -0.65 * s, 0.75 * s, 0.25, 2.3 * s, 1.3).d(1.3 * s).tone("marble");
            a.add("hall", 0.45 * s, 0.95 * s, 1.55, 2.5 * s, 1.1).d(1.2 * s).tone("glass");
            a.add("accent", 0.45 * s, 0.95 * s, 2.65, 2.55 * s, 0.14).d(1.25 * s).tone("faction");
            for x in [0.9, 1.5] { a.add("column", x * s, 0.55 * s, 0.25, 0.14, 1.3).tone("marble"); }
            a.add("hall", 1.6 * s, -0.4 * s, 0.25, 0.75 * s, 3.4).d(0.75 * s).tone("marble");
            a.add("accent", 1.6 * s, -0.4 * s, 3.65, 0.8 * s, 0.2).d(0.8 * s).tone("faction");
            for (x, y, w, d) in [(-1.25, -0.9, 1.4, 1.2), (0.2, -0.55, 1.0, 0.9)] { a.add("garden", x * s, y * s, 0.25, w * s, 0.1).d(d * s); }
            a.add("platform", -0.2 * s, -1.75 * s, 0.25, 2.4 * s, 0.05).d(0.55 * s).tone("glass");
            a.add("statue", 1.6 * s, -1.6 * s, 0.25, 0.4, 2.6).tone("marble");
            for (x, y) in [(-1.95, 0.0), (0.5, -1.25), (-0.9, -1.35)] { a.add("light", x * s, y * s, 0.25, 0.16, 0.9).tone("faction"); }
            a.add("banner", -1.9 * s, 1.75 * s, 0.25, 0.5, 2.4);
            let _ = k;
        }
    }
}
