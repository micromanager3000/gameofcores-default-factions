//! Shared strategy used by the default factions.
use super::{air, theme};

    use std::cell::{Cell, OnceCell, RefCell};
    use std::rc::Rc;

    use gameofcores::*;

    /// Keep the target of a paid windup while it is still visible and in reach.
    /// A new priority target can wait until this shot has resolved.
    pub fn windup_target(u: View, near: View, preferred: Option<View>) -> Option<View> {
        let pending=u.path("rts.windupTarget");
        if pending.is_num() {
            if let Some(t)=near.iter().find(|t| t.get("id").num()==pending.num() && reflex::can_hit(u,*t)) { return Some(t); }
        }
        preferred
    }

    /// A moving ground unit's next 0.2-second drive charge, before the price level.
    pub fn drive_tick(u: View, power: f64) -> f64 {
        (60.0+power)*u.path("rts.driveScale").num_or(1.0)/5.0
    }

    /// Close with a visible ground rival and buy a dash only near striking distance.
    pub fn close_melee(u: View, near: View, out: &mut Value) {
        let r=u.get("rts");
        if !r.truthy() || u.get("range").num_or(0.0)>0.0 || u.get("air").truthy()
            || r.path("sea.ship").truthy() || !reflex::engaged(u) || out.get("step").truthy() { return; }
        let Some(t)=near.iter().find(|t|t.get("rival").truthy()&&!t.get("air").truthy()) else {return};
        let d=t.get("d").num();
        if d>0.7 {
            out.set("hold",false);out.set("step",obj!{"x"=>t.get("dx"),"y"=>t.get("dy")});
        }
        let cost=900.0*price_level(u);
        if d>1.2 && d<3.5 && r.get("modules").has_str("dash") && r.path("abilityIn.dash").num_or(0.0)<=0.0
            && r.get("bank").num_or(0.0)>=cost && r.path("budget.abilities").num_or(cost)>=cost {
            out.set("ability","dash");
        }
    }

    /// Separate moving allies and turn away from blocked terrain. Direction samples share `u.local`'s frame.
    pub fn steer_ground(u: View, near: View, out: &mut Value) {
        if u.get("air").truthy() || u.path("rts.sea.ship").truthy() || u.path("rts.deployed").truthy() {return;}
        let step=out.get("step");if !step.truthy(){return;}
        let (mut x,mut y)=(step.get("x").num_or(0.0),step.get("y").num_or(0.0));
        let length=x.hypot(y);if length<0.01{return;}
        x/=length;y/=length;
        for a in near.iter().filter(|a| !a.get("rival").truthy() && a.get("owner").num()==u.get("owner").num()) {
            let d=a.get("d").num_or(10.0);
            if d>0.01 && d<1.1 {let push=((1.1-d)/1.1).powi(2)*3.0;x-=a.get("dx").num()/d*push;y-=a.get("dy").num()/d*push;}
            else if d<=0.01 {y+=if u.get("id").num()<a.get("id").num(){-0.6}else{0.6};}
        }
        // Keep combat micro within the current province's neighbourhood; turning beats an endless retreat.
        let local=u.get("local");let (px,py)=(local.get("x").num_or(0.0),local.get("y").num_or(0.0));
        let radius=px.hypot(py);
        if radius>10.0 && px*x+py*y>0.0 {
            let turn=if -py*x+px*y>=0.0 {1.0}else{-1.0};
            x=(-py*turn-px*0.35)/radius;y=(px*turn-py*0.35)/radius;
        }
        let amount=x.hypot(y).min(1.0)*length.min(1.0);
        let terrain=u.get("terrain");
        if !terrain.is_empty() {
            let len=x.hypot(y).max(0.01);x/=len;y/=len;
            let best=terrain.iter().filter(|t|t.get("walk").num_or(0.0)>0.0).max_by(|a,b| {
                let score=|t:View|x*t.get("x").num()+y*t.get("y").num()+0.2*t.get("walk").num();
                score(*a).partial_cmp(&score(*b)).unwrap_or(std::cmp::Ordering::Equal)
            });
            if let Some(t)=best {x=t.get("x").num();y=t.get("y").num();}else{x=0.0;y=0.0;}
        }
        out.set("hold",false);out.set("step",obj!{"x"=>x*amount,"y"=>y*amount});
    }

    /// Range control for mobile ground shooters. Explicit hazard dodges take priority.
    /// Step back during reload, plant for the next shot, and keep a gap from allies.
    /// This uses public enemy reach and the unit's own weapon clock; it predicts no damage.
    pub fn space_ranged(u: View, near: View, out: &mut Value) {
        let r=u.get("rts");
        if !r.truthy() || u.get("range").num_or(0.0)<=0.0 || u.get("air").truthy()
            || r.path("sea.ship").truthy() || r.get("deployed").truthy() || out.get("step").truthy() { return; }
        let reach=u.get("reach").num_or(0.0);
        let threat=near.iter().find(|t| t.get("rival").truthy() && !t.get("air").truthy());
        let Some(t)=threat else { return };
        let enemy_reach=t.get("reach").num_or(match t.get("range").num_or(0.0) as u32 {0=>1.0,1=>4.0,_=>7.0});
        // Equal-range armies trade fire. Longer-range or faster enemies can still punish a retreat.
        if reach<=enemy_reach+0.5 { return; }
        let (d,dx,dy)=(t.get("d").num_or(0.0),t.get("dx").num_or(0.0),t.get("dy").num_or(0.0));
        let outer=reach-0.4;
        let inner=(reach*0.72).max(enemy_reach+0.9).min(outer-0.25);
        let reloading=r.get("readyIn").num_or(0.0)>0.0 && r.get("windupTarget").is_nullish();
        let desired=if reloading {outer-0.15} else {inner};
        if d<desired && d>0.01 {
            let step=(desired-d+0.12).clamp(0.12,1.0);
            let mut x=-dx/d*step;let mut y=-dy/d*step;
            for ally in near.iter().filter(|a| !a.get("rival").truthy() && a.get("owner").num_or(-1.0)==u.get("owner").num_or(-2.0)) {
                let gap=ally.get("d").num_or(10.0);
                if gap>0.01 && gap<0.8 {x-=ally.get("dx").num()/gap*0.16;y-=ally.get("dy").num()/gap*0.16;}
            }
            out.set("hold",false);out.set("step",obj!{"x"=>x,"y"=>y});
        } else if matches!(out.get("hit"),Value::Num(_)) && reflex::engaged(u) {
            out.set("hold",true);
        }
    }

    /// Rotate a small wing through eyes, interception and (once licensed) strike craft.
    pub fn airframe(me: View, slot: f64) -> air::Airframe {
        let role = match (slot.max(0.0) as usize) % 3 {
            0 => air::Role::Scout,
            2 if me.path("techs.airWeapons").truthy() => air::Role::Striker,
            _ => air::Role::Fighter,
        };
        theme::of(me.get("seat"), 3.0).map_or_else(|| air::of(3, role).expect("Atlantis"), |t| t.airframe(role))
    }

    /// Aerial orders for a house that opts into an aircraft function. Land before the expensive heat climb,
    /// preserve a cooling stop, and use the carried plan again after cooling. Wings remain part of the body.
    pub fn fly(units: View, world: View, me: View) {
        for u in units.iter() {
            if cycles_left() < 200.0 { return; }
            let a = u.path("rts.air");
            if a.get("aloft").truthy() && a.get("heat").num_or(0.0) >= 45.0 {
                let coming_home=u.get("doing").str()=="move"&&world.get("provinces").iter().any(|p| {
                    let owner=p.get("owner").num_or(-1.0);
                    p.get("id").num_or(-2.0)==u.get("target").num_or(-1.0)&&(owner==me.get("id").num()||realm::is_ally(me,owner))
                });
                if !coming_home { order(obj! {"id"=>u.get("id"),"land"=>true}); }
            }
        }
        if cycles_left() < 2400.0 { return; }
        let plan = plan::for_units(me, units, world);
        let goal = ["defend", "target", "attack"].iter().map(|k|plan.get(k)).find(|v|v.num().is_finite()&&v.num()>=0.0);
        let Some(goal) = goal else { return };
        for u in units.iter() {
            if cycles_left() < 1800.0 { break; }
            let a=u.path("rts.air");
            let heat=a.get("heat").num_or(0.0);
            if heat>=45.0 || (heat>0.0&&!a.get("aloft").truthy()) { continue; }
            let verb=if u.path("build.vision").num_or(0.0)>=2.0 {"move"} else {"fight"};
            if u.get("doing").str()==verb&&u.get("target").num_or(-1.0)==goal.num() { continue; }
            if let Some(mut o)=orders::to(u,verb,goal.num(),world,me,()) {
                o.set("alert",6);o.set("power",obj!{"drive"=>600,"weapon"=>420});order(o);
            }
        }
    }

    /// `Number(x)` for a value from the game: null is 0, anything that isn't a number NaN.
    fn n(v: View) -> f64 {
        if v.is_null() {
            0.0
        } else {
            v.num()
        }
    }

    /// `x || d` for a number you built or read from RAM.
    fn or(v: &Value, d: f64) -> f64 {
        let x = v.num();
        if x != 0.0 && !x.is_nan() {
            x
        } else {
            d
        }
    }

    /// Adds RAM ops to `orders.ram`.
    fn ram(orders: &mut Value, ops: Vec<Value>) {
        for op in ops {
            orders.entry("ram").push(op);
        }
    }

    // ---- targets ----

    /// What land you've never seen scores in [`score`]: its owner and cores are unknown, and an army sent there walks
    /// into the fog, so it ranks below any land in sight but a well-held capital.
    pub const UNSEEN: f64 = -4.0;

    /// A target's worth: cores first; free land is cheap to take; a garrison and a capital cost; restive land, rebels
    /// and a runaway leader's land invite attack; land deep in another faction's faith costs its zeal ÷ 15, since it
    /// keeps winning its founder favour and revolts back after you take it. Land you've never seen scores [`UNSEEN`],
    /// and a figure you can't read counts as none, so the score is always a number.
    pub fn score(world: View, me: View, q: View) -> f64 {
        if !q.get("owner").is_num() {
            return UNSEEN;
        }
        let mut s = q.get("cores").num_or(0.0) * 2.0;
        if q.get("owner").num() < 0.0 {
            s += 2.5;
        } else {
            s -= q.get("garrison").or(0.0) * 1.1 + if q.get("capital").truthy() { 6.0 } else { 0.0 };
        }
        if q.get("loyaltyBand").str() == "restive" {
            s += 2.0;
        }
        if q.get("rebel").truthy() {
            s += 1.0;
        }
        let me_id = me.get("id").num();
        if dominant(world).is_some_and(|d| d.get("id").num() == q.get("owner").num() && d.get("id").num() != me_id) {
            s += 2.5;
        }
        let faith = q.get("faith");
        if faith.truthy() && faith.get("by").num() != me_id {
            s -= faith.get("zeal").num_or(0.0) / 15.0;
        }
        s
    }

    thread_local! {
        /// Whether the leader is dominant, for the call's world it was worked out for.
        static DOMINANT: Cell<Option<(View, bool)>> = const { Cell::new(None) };
    }

    /// The leader (`realm::leader`) when it's pulling away from the field: more than 18% of the world's cores, and more
    /// than 1.6 times a fair share of the factions standing. Worked out once a second.
    pub fn dominant(world: View) -> Option<View> {
        let lead = realm::leader(world)?;
        if let Some((_, yes)) = DOMINANT.with(Cell::get).filter(|(w, _)| w.same(&world)) {
            return yes.then_some(lead);
        }
        let alive = world.get("factions").iter().filter(|f| f.get("alive").truthy()).count().max(1) as f64;
        let yes = n(lead.get("share")) > (1.6 / alive).max(0.18);
        DOMINANT.with(|d| d.set(Some((world, yes))));
        yes.then_some(lead)
    }

    /// Turning on a Crown claim: while someone else claims the Crown, only the claimants' provinces in `list` (in its
    /// order) when you border any; otherwise `list` itself. A claim has `RULES.crown.hold` seconds to survive everyone.
    pub fn rally(world: View, me: View, list: Vec<View>) -> Vec<View> {
        let Some(foes) = realm::claimant(world, me) else { return list };
        let hit: Vec<View> = list.iter().copied().filter(|q| q.get("owner").is_num() && foes.has_num(q.get("owner").num())).collect();
        if hit.is_empty() {
            list
        } else {
            hit
        }
    }

    /// Whether your army can hold on to province `q` once it takes it: it's on your land border or over a mountain
    /// pass, or over the sea or through the air where a ship of yours keeps station on the crossing (`sea::station`).
    /// RAM, and with it the plan, crosses to another landmass only near a ship of yours, so an army that takes land over
    /// the water with no ship there stands idle on it. `ships`: whether you field ships at all (the default program and
    /// the house programs don't); without, it looks for none.
    pub fn can_hold(world: View, me: View, ships: bool) -> impl Fn(View) -> bool {
        let me_id = me.get("id").num();
        let by_id = realm::index(world);
        let ours = move |id: View| by_id.at(id).get("owner").num() == me_id;
        // the waters our ships are on, looked up the first time land over the water needs them
        let fleet = ships;
        let ships: OnceCell<Vec<f64>> = OnceCell::new();
        move |q: View| {
            let routes = q.get("routes");
            if q.get("adj").iter().any(&ours) || routes.get("pass").iter().any(&ours) {
                return true;
            }
            if !fleet {
                return false;
            }
            let ships = ships.get_or_init(|| {
                let mine = world.get("units").iter().filter(|u| u.get("owner").num() == me_id);
                mine.map(|u| u.get("rts").get("sea")).filter(|s| s.get("ship").truthy()).map(|s| s.get("water").num()).collect()
            });
            if ships.is_empty() {
                return false;
            }
            let waters = sea::index(world);
            routes.get("sea").iter().chain(routes.get("air").iter()).filter(|p| ours(*p)).any(|p| {
                let w = sea::station(world, p.num(), q.get("id").num());
                w >= 0.0 && ships.iter().any(|&s| s == w || waters.get(w).get("adj").has_num(s))
            })
        }
    }

    /// What a target your army couldn't hold on to ([`can_hold`]) is marked down by, in [`score`]'s points: more than any
    /// target is worth, so the army crosses the water to land it would stand idle on only when nothing on this side of
    /// it is left to take.
    pub const OVER_WATER: f64 = 100.0;

    thread_local! {
        /// Partner land you can't see ([`partner_land`]), with the call's world.
        static PARTNER: RefCell<Option<(View, Rc<Vec<f64>>)>> = const { RefCell::new(None) };
    }

    /// Land a pact partner holds though you can't see who holds it: the provinces a "your pact partner's" refusal
    /// named (`me.refused` holds the last second's), and those your last plan kept in its `partner` list, while they
    /// stay out of sight (out of sight, the owner you have is the one you saw last). An order to capture or raid there is refused every second you send it, so leave
    /// them off your targets ([`frontier`]); [`survey`] keeps the list in the plan so it outlasts the refusal.
    pub fn partner_land(world: View, me: View) -> Rc<Vec<f64>> {
        if let Some(x) = PARTNER.with(|c| c.borrow().as_ref().filter(|(w, _)| w.same(&world)).map(|(_, v)| v.clone())) {
            return x;
        }
        let by_id = realm::index(world);
        // out of sight, a province's owner is the one you saw last, if any
        let unknown = |id: f64| by_id.get(id).truthy() && !by_id.get(id).get("visible").truthy();
        let mut out: Vec<f64> = match mem::get(me.get("ramCapital"), 0).get("partner") {
            Value::Arr(a) => a.iter().map(|x| x.num_or(-1.0)).filter(|&id| unknown(id)).collect(),
            _ => Vec::new(),
        };
        for r in me.get("refused").iter() {
            // "<unit>: <province> is <faction>'s, your pact partner's: capture and raid rivals' land"
            let why = r.get("why").str();
            if !why.contains("your pact partner's") {
                continue;
            }
            let Some((name, _)) = why.split_once(": ").and_then(|(_, rest)| rest.split_once(" is ")) else { continue };
            if let Some(q) = world.get("provinces").iter().find(|q| q.get("name").str() == name) {
                let id = q.get("id").num();
                if unknown(id) && !out.contains(&id) {
                    out.push(id);
                }
            }
        }
        let out = Rc::new(out);
        PARTNER.with(|c| *c.borrow_mut() = Some((world, out.clone())));
        out
    }

    /// `realm::frontier` without the partner land you can't see ([`partner_land`]), nor the land of a faction whose
    /// passage you hold: a capture there ends the deal at once, in public ("broke its deal by attacking"). Free land and
    /// rivals' next to yours.
    pub fn frontier(world: View, me: View) -> Vec<View> {
        let skip = partner_land(world, me);
        let hosts: Vec<f64> =
            me.get("contracts").iter().filter(|c| !c.get("ended").truthy() && c.get("get").get("passage").truthy()).map(|c| c.get("with").num()).collect();
        realm::frontier(world, me)
            .into_iter()
            .filter(|q| !skip.contains(&q.get("id").num()) && !(q.get("owner").is_num() && hosts.contains(&q.get("owner").num())))
            .collect()
    }

    /// Rival provinces in sight on your land border, the likeliest to turn first: restive, then uneasy, then the most
    /// cores. Where to bribe, broadcast propaganda or preach: a revolt joins only a faction whose land borders it, so
    /// land across a sea route, a pass or a flight is no target, even where a unit of yours stands.
    pub fn sway_targets(world: View, me: View) -> Vec<View> {
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        let band = |q: &View| -> f64 {
            match q.get("loyaltyBand").str() {
                "restive" => 0.0,
                "uneasy" => 1.0,
                _ => 2.0,
            }
        };
        let mut out: Vec<View> = realm::frontier(world, me)
            .into_iter()
            .filter(|q| {
                let owner = q.get("owner").num();
                owner >= 0.0 && realm::is_rival(world, me, owner) && q.get("visible").truthy() && q.get("adj").iter().any(|a| by_id.at(a).get("owner").num() == me_id)
            })
            .collect();
        out.sort_by(|a, b| band(a).total_cmp(&band(b)).then(b.get("cores").or(0.0).total_cmp(&a.get("cores").or(0.0))));
        out
    }

    // ---- the plan ----

    /// A siege moves only while its capturers in the town's ring number this many for each defender there
    /// (`RULES.siegeOdds`).
    pub const SIEGE_ODDS: f64 = 3.0;

    /// How many units to gather before attacking `q`: 2 for free land, [`SIEGE_ODDS`] for each of its garrison + 2 (at
    /// least 3) for a rival's, and 3 more for a capital.
    pub fn muster_for(q: View) -> f64 {
        if !q.truthy() {
            return 0.0;
        }
        if n(q.get("owner")) < 0.0 {
            return 2.0;
        }
        (q.get("garrison").or(0.0) * SIEGE_ODDS + 2.0).max(3.0) + if q.get("capital").truthy() { 3.0 } else { 0.0 }
    }

    /// How many miners your mines can use now: those at work or on their way (`me.units.doing.gather`) and one for each
    /// open place in the plan's `mines` ([`survey`]: `realm::open_mines`, up to 24). Past that a new miner stands idle.
    pub fn miner_room(me: View, plan: &Value) -> f64 {
        me.get("units").get("doing").get("gather").or(0.0) + plan.get("mines").len() as f64
    }

    /// Your units free to fight: all of them but those at work in a mine or studying.
    pub fn fighters(me: View) -> f64 {
        let u = me.get("units");
        n(u.get("total")) - u.get("doing").get("gather").or(0.0) - u.get("doing").get("study").or(0.0)
    }

    /// Whether a siege of `q` is hopeless: its garrison needs more capturers ([`SIEGE_ODDS`] each) than you have fighters
    /// ([`fighters`]), so your units there would die without taking it. A siege that came to this, its defenders
    /// reinforced or your army worn down, is given up: the army goes elsewhere until it's big enough.
    pub fn hopeless(me: View, q: View) -> bool {
        q.get("garrison").or(0.0) * SIEGE_ODDS > fighters(me)
    }

    /// The next target from a ranked list, sticky: the last plan's (`last`, its id) while it's still on the list, so
    /// units don't march back and forth.
    pub fn pick(targets: &[View], last: f64) -> Option<View> {
        targets.iter().copied().find(|q| last >= 0.0 && q.get("id").num() == last).or_else(|| targets.first().copied().filter(View::truthy))
    }

    /// How many of your units stand at province `stage` or in `attack`, of every type but those at work in your mines or
    /// studying, so units of two types that muster together count together. 0 while fewer than `need` of your units of
    /// any type stand there (no pass over the units then). think only: put it in the plan as `ready`, and [`muster`]
    /// sends the plan's units in once it reaches `need`.
    pub fn ready(world: View, me: View, stage: f64, attack: f64, need: f64) -> f64 {
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        let here = |id: f64| by_id.get(id).get("units").key(me_id).or(0.0);
        if here(stage) + here(attack) < need || !world.get("units").is_arr() {
            return 0.0;
        }
        let (mut work, mut by): (Vec<&str>, Vec<(&str, f64)>) = (Vec::new(), Vec::new());
        for u in world.get("units").iter().filter(|u| u.get("owner").num() == me_id) {
            let (kind, doing) = (u.get("type").str(), u.get("doing").str());
            if doing == "gather" || doing == "study" {
                if !work.contains(&kind) {
                    work.push(kind);
                }
            } else if (u.get("at").num() == stage || u.get("at").num() == attack) && {
                // standing there, or holding there (a unit holding the muster point keeps it as its `to`)
                let to = n(u.get("to"));
                to < 0.0 || to == stage || to == attack
            } {
                match by.iter_mut().find(|x| x.0 == kind) {
                    Some(x) => x.1 += 1.0,
                    None => by.push((kind, 1.0)),
                }
            }
        }
        by.iter().filter(|(k, _)| !work.contains(k)).map(|(_, c)| c).sum()
    }

    /// The plan most strategies start from: `{ mines, attack, stage, need, joint, defend, threat, home, fresh }`.
    /// mines: open slots (`realm::open_mines`); attack: the target (an ally's siege on your list comes first: you split
    /// its CPU); stage: where to muster (`realm::from`); need: how many to muster ([`muster_for`]); joint: the ally you join;
    /// defend/threat (and picket/guard): the invaded province, how many invaders, and who goes ([`defense`]); home: your
    /// capital; fresh: the second, rounded down to 15, so the plan changes and is written again (`plan::share` writes
    /// only a change) and your units can tell a plan that stands from land cut off from it (`plan::check`); partner:
    /// unseen land the game said a pact partner holds ([`partner_land`]), left off the targets. The last plan (in capital
    /// RAM) keeps the target sticky. Targets that need more than about 60% of your fighters wait while others are on
    /// the list, and a hopeless siege ([`hopeless`]) is no target at all. `per_slot` is how deep to stack miners (2 is
    /// the usual).
    pub fn survey(world: View, me: View, targets: &[View], per_slot: f64) -> Value {
        let last = mem::get(me.get("ramCapital"), 0).get("attack").num_or(-1.0);
        let me_id = me.get("id").num();
        let partner = partner_land(world, me);
        let kept: Vec<View> = targets.iter().copied().filter(|q| !partner.contains(&q.get("id").num()) && !hopeless(me, *q)).collect();
        let targets = &kept[..];
        let joint = targets.iter().copied().find(|q| {
            let by = q.get("siege").get("by").num();
            q.get("siege").truthy() && by != me_id && realm::is_ally(me, by)
        });
        let can = (fighters(me) * 0.6).floor().max(4.0);
        let fit: Vec<View> = targets.iter().copied().filter(|q| muster_for(*q) <= can).collect();
        let t = joint.or_else(|| pick(if fit.is_empty() { targets } else { &fit }, last));
        let need = if joint.is_some() { 1.0 } else { t.map_or(0.0, muster_for) };
        let mines: Vec<Value> = realm::open_mines(world, me, per_slot, 24.0).into_iter().map(|[p, k]| arr![p, k]).collect();
        let mut plan = obj! {
            "mines" => mines,
            "attack" => t.map_or(Value::Num(-1.0), |t| t.get("id").value()),
            "stage" => t.map_or_else(|| me.get("capital").value(), |t| realm::from(t).map_or(Value::Undef, Value::Num)),
            "need" => need,
            "joint" => joint.map_or(Value::Undef, |j| j.get("siege").get("by").value()),
            "home" => me.get("capital"),
            "fresh" => (n(world.get("seconds")) / 15.0).floor() * 15.0,
        };
        defense(world, me, &mut plan);
        if !partner.is_empty() {
            plan.set("partner", Value::Arr(partner.iter().map(|&id| Value::Num(id)).collect()));
        }
        plan
    }

    /// The most units a plan names to guard a province ([`defense`]): a bigger threat calls everyone.
    pub const GUARD_MAX: f64 = 24.0;

    /// Sets the plan's defence against the invaders in your land (`realm::invaded`, which leaves out units there under a
    /// deal and puts your capital first): `threat`, how many there are, and who goes. Everyone defends (`defend`, the
    /// province) when it's your capital, or when two for each invader and one more would take most of your fighters (or
    /// more than [`GUARD_MAX`]). Otherwise those two for each and one more go, nearest first (those fighting there,
    /// those standing there, those next to it, then by distance): `picket` is the province and `guard` their ids (chosen
    /// again every 15 s: `guardAt`), and the rest of the army keeps to the plan. Units abroad carry the plan without its
    /// picket, so they keep on. [`defends`] reads it for a unit.
    pub fn defense(world: View, me: View, plan: &mut Value) {
        let inv = realm::invaded(world, me);
        plan.set("threat", inv.n);
        plan.set("defend", -1.0);
        if inv.at < 0.0 {
            return;
        }
        let want = inv.n * 2.0 + 1.0;
        // (the count of units free to fight comes from me.units, so a big threat costs no pass over world.units)
        if inv.at == me.get("capital").num() || want > GUARD_MAX || want >= fighters(me) * 0.6 {
            plan.set("defend", inv.at);
            return;
        }
        // the last plan's guard stands while its picket does, for 15 s: choosing one is a pass over world.units, about
        // 40 cycles a unit in sight
        let last = mem::get(me.get("ramCapital"), 0);
        let now = n(world.get("seconds"));
        if last.get("picket").num_or(-1.0) == inv.at && now - last.get("guardAt").num_or(-1e9) < 15.0 && matches!(last.get("guard"), Value::Arr(_)) {
            plan.set("picket", inv.at);
            plan.set("guard", last.get("guard").clone());
            plan.set("guardAt", last.get("guardAt").clone());
            return;
        }
        let me_id = me.get("id").num();
        let by_id = realm::index(world);
        let here = by_id.get(inv.at);
        let far = |p: View| (n(p.get("x")) - n(here.get("x"))).powi(2) + (n(p.get("y")) - n(here.get("y"))).powi(2);
        let mut near: Vec<(f64, f64)> = Vec::new();
        for u in world.get("units").iter().filter(|u| u.get("owner").num() == me_id) {
            let doing = u.get("doing").str();
            if doing == "gather" || doing == "study" || u.get("type").str() == "miner" {
                continue;
            }
            let at = u.get("at").num();
            let d = if doing == "fight" && u.get("target").num() == inv.at {
                -2.0
            } else if at == inv.at {
                -1.0
            } else if here.get("adj").has_num(at) {
                0.0
            } else {
                far(by_id.get(at))
            };
            near.push((d, u.get("id").num()));
        }
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        plan.set("picket", inv.at);
        plan.set("guard", Value::Arr(near.iter().take(want as usize).map(|x| Value::Num(x.1)).collect()));
        plan.set("guardAt", now);
    }

    /// The province unit `u` defends under plan `pl` ([`defense`]): the plan's `defend`, or its `picket` when `u` is on
    /// the `guard`; -1 when it keeps to the plan.
    pub fn defends(pl: &Value, u: View) -> f64 {
        let d = pl.get("defend").num_or(-1.0);
        if d >= 0.0 {
            return d;
        }
        let picket = pl.get("picket").num_or(-1.0);
        let id = u.get("id").num();
        match pl.get("guard") {
            Value::Arr(a) if picket >= 0.0 && a.iter().any(|x| x.num_or(-1.0) == id) => picket,
            _ => -1.0,
        }
    }

    // ---- unit orders on the plan ----

    /// Adds `u`'s order and RAM ops (either may be missing) to `out`, committing it with `order()`.
    fn add(out: &mut Vec<Value>, u: View, o: Option<Value>, ops: Option<Vec<Value>>) {
        if o.is_none() && ops.is_none() {
            return;
        }
        let mut x = o.unwrap_or_else(|| obj! { "id" => u.get("id") });
        if let Some(ops) = ops {
            x.set("ram", ops);
        }
        order(x.clone());
        out.push(x);
    }

    /// Each unit gathers at its plan's `stage` until `need` units are there (or the plan's `ready` says enough of every
    /// type stand there), then all go in on its `attack` with `verb`; once some are committed, the rest follow. The
    /// attack routes along the plan's field when it leads there (`path::along`). A target that has become a pact
    /// partner's land sends its units back to the stage. Units in your land carry the plan for when they leave it
    /// (`orders::carry`). `pls`: each unit's plan.
    pub fn muster(units: &[View], world: View, me: View, verb: &str, pls: &[Rc<Value>]) -> Vec<Value> {
        let attack_of = |pl: &Value| pl.get("attack").num_or(-1.0);
        let (mut there, mut committed): (Vec<(f64, f64)>, Vec<f64>) = (Vec::new(), Vec::new());
        for (u, pl) in units.iter().zip(pls) {
            let a = attack_of(pl);
            if a < 0.0 {
                continue;
            }
            let at = u.get("at").num();
            if at == pl.get("stage").num() || at == a {
                match there.iter_mut().find(|x| x.0 == a) {
                    Some(x) => x.1 += 1.0,
                    None => there.push((a, 1.0)),
                }
            }
            if u.get("doing").str() == verb && u.get("target").num() == a && !committed.contains(&a) {
                committed.push(a);
            }
        }
        let mut out = Vec::new();
        for (&u, pl) in units.iter().zip(pls) {
            if !orders::room(world) {
                continue;
            }
            let ops = orders::carry(u, pl, me);
            let a = attack_of(pl);
            if a < 0.0 {
                add(&mut out, u, None, ops);
                continue;
            }
            let need = pl.get("need").num();
            let go = committed.contains(&a) || there.iter().find(|x| x.0 == a).map_or(0.0, |x| x.1) >= need || pl.get("ready").num() >= need;
            let stage = pl.get("stage").num();
            // a target that became a partner's land after the plan was made (units abroad keep the plan they carry):
            // back to the stage, where a newer plan reaches them
            let owner = realm::index(world).get(a).get("owner").num_or(-1.0);
            let conduct = u.get("conduct");
            let protected = realm::is_ally(me, owner)
                || (conduct.get("seconds").num_or(0.0) > 0.0 && conduct.get("from").num_or(-2.0) == owner);
            // A transferred flier keeps its wings even if this function is called guard(). It raids instead of
            // trying to capture; the choice lives here in faction code, as does returning to the stage.
            let verb = if verb == "capture" && u.path("build.wings").num_or(0.0) > 0.0 { "raid" } else { verb };
            let o = if protected {
                if u.get("at").num() == stage || n(u.get("to")) == stage { None } else { orders::to(u, "move", stage, world, me, ()) }
            } else if go {
                let along = if pl.get("field").get("to").num() == a { path::along(u, verb, pl, world) } else { None };
                along.or_else(|| orders::to(u, verb, a, world, me, ()))
            } else if u.get("at").num() == stage {
                None
            } else {
                orders::to(u, "move", stage, world, me, ())
            };
            add(&mut out, u, o, ops);
        }
        out
    }

    /// If the plan says you're invaded (`defend`), two units for each invader and one more fight there, those already
    /// fighting there first; units the plan's guard names fight at its picket ([`defense`]); the rest [`muster`] with
    /// `verb`.
    pub fn defend_then(units: &[View], world: View, me: View, verb: &str) -> Vec<Value> {
        let mut out = Vec::new();
        let (mut rest, mut rest_pl): (Vec<View>, Vec<Rc<Value>>) = (Vec::new(), Vec::new());
        // (province, units it wants, its units and their plans)
        type Group = (f64, f64, Vec<(View, Rc<Value>)>);
        let mut by: Vec<Group> = Vec::new();
        for &u in units {
            let pl = plan::check(plan::of(me, u), world);
            let d = pl.get("defend").num_or(-1.0);
            if d.is_nan() || d < 0.0 {
                let picket = defends(&pl, u);
                if picket >= 0.0 && orders::room(world) {
                    let o = orders::to(u, "fight", picket, world, me, ());
                    let ops = orders::carry(u, &pl, me);
                    add(&mut out, u, o, ops);
                } else if picket < 0.0 {
                    rest.push(u);
                    rest_pl.push(pl);
                }
                continue;
            }
            match by.iter_mut().find(|g| g.0 == d) {
                Some(g) => g.2.push((u, pl)),
                None => {
                    let want = or(pl.get("threat"), 1.0) * 2.0 + 1.0;
                    by.push((d, want, vec![(u, pl)]));
                }
            }
        }
        for (d, want, us) in by {
            let fighting = |u: View| u.get("doing").str() == "fight" && u.get("target").num() == d;
            let mut picked: Vec<f64> = us.iter().filter(|(u, _)| fighting(*u)).map(|(u, _)| u.get("id").num()).collect();
            picked.truncate(want as usize);
            for (u, _) in &us {
                if (picked.len() as f64) < want && !picked.contains(&u.get("id").num()) {
                    picked.push(u.get("id").num());
                }
            }
            for (u, pl) in us {
                if !picked.contains(&u.get("id").num()) {
                    rest.push(u);
                    rest_pl.push(pl);
                } else if orders::room(world) {
                    let o = orders::to(u, "fight", d, world, me, ());
                    let ops = orders::carry(u, &pl, me);
                    add(&mut out, u, o, ops);
                }
            }
        }
        out.extend(muster(&rest, world, me, verb, &rest_pl));
        out
    }

    /// A cheap scout's next hop from province `at`: the neighbour whose own neighbours you know least about (never seen
    /// counts as 10 minutes). With everything near in sight, the neighbour farthest from your capital; -1 at the far
    /// edge. Short of cycles (`orders::reserve()` left), the best neighbour so far.
    pub fn wander(world: View, me: View, at: f64) -> f64 {
        let by_id = realm::index(world);
        let here = by_id.get(at);
        let cap = by_id.at(me.get("capital"));
        let home = if cap.truthy() { cap } else { here };
        let far = |p: View| (n(p.get("x")) - n(home.get("x"))).powi(2) + (n(p.get("y")) - n(home.get("y"))).powi(2);
        let (mut best, mut bs, mut bd) = (-1.0, 0.0, far(here));
        for id in realm::reach(here, me) {
            if best >= 0.0 && cycles_left() < orders::reserve() {
                break;
            }
            let q = by_id.get(id);
            let s: f64 = q
                .get("adj")
                .iter()
                .map(|a| {
                    let r = by_id.at(a);
                    if r.get("visible").truthy() {
                        0.0
                    } else {
                        r.get("seenAgo").num_or(600.0)
                    }
                })
                .sum();
            let d = far(q);
            if s > bs || (s == bs && d > bd) {
                (best, bs, bd) = (id, s, d);
            }
        }
        best
    }

    /// The tech to steal from faction `o`: the one with the most puzzles that it has and you don't, among those whose
    /// requirements you have.
    pub fn pick_tech(world: View, me: View, o: f64) -> Option<&'static str> {
        let (all, mine) = (world.get("techs"), me.get("techs"));
        let open = |t: &str| {
            let d = all.get(t);
            !mine.get(t).truthy() && d.truthy() && d.get("requires").iter().all(|q| mine.get(q.str()).truthy())
        };
        let need = |t: &str| n(all.get(t).get("need"));
        let techs = world.get("factions").key(o).get("techs");
        techs.iter().map(|t| t.str()).filter(|t| open(t)).fold(None::<&'static str>, |b, t| if b.is_none_or(|b| need(t) > need(b)) { Some(t) } else { b })
    }

    /// A spy's errand at the nearest rival capital (`spy::capital`): steal the dearest tech it can there ([`pick_tech`]).
    /// `None` when there's nothing to steal or it's on it already.
    pub fn steal(u: View, world: View, me: View) -> Option<Value> {
        let cap = spy::capital(world, me, u);
        let tech = pick_tech(world, me, realm::index(world).get(cap).get("owner").num_or(-1.0))?;
        spy::steal(u, world, me, Some(cap), tech)
    }

    /// A spy's watch on the nearest rival capital, from 6 cells east of its town, outside its ring where fewer eyes are.
    pub fn sneak(u: View, world: View, me: View) -> Option<Value> {
        spy::go(u, "move", spy::capital(world, me, u), world, me, Point::new(6.0, 0.0))
    }

    // ---- prices ----

    /// The world's price level, from `me` (`me.rts.costs.priceLevel`) or one of your units (`u.rts.priceLevel`, which
    /// reflex reads): every price in cycles is the rules' price times it. 1 outside the CPU RTS.
    pub fn price_level(v: View) -> f64 {
        let r = v.get("rts");
        let x = if r.get("costs").truthy() { r.get("costs").get("priceLevel") } else { r.get("priceLevel") };
        if x.is_num() && x.num() > 0.0 {
            x.num()
        } else {
            1.0
        }
    }

    /// What fabricating a body with `modules` modules costs, in cycles (`me.rts.costs.fabrication`).
    pub fn fabrication(me: View, modules: f64) -> f64 {
        let f = me.path("rts.costs.fabrication");
        if f.truthy() {
            n(f.get("base")) + modules * n(f.get("module"))
        } else {
            1200.0 + 600.0 * modules
        }
    }

    /// What the fabrication lane may still pay this second once think has run (it's charged right after think): what it
    /// may spend now less half again think's recent peak, so a spawn is rarely refused. `None` outside the CPU RTS, and
    /// 0 before think's first second, which has no measure of it.
    pub fn fabrication_room(me: View) -> Option<f64> {
        let fab = me.path("rts.cpu.available.fabrication");
        if !fab.is_num() {
            return None;
        }
        let peak = n(me.get("compute").get("thinkPeak"));
        if peak <= 0.0 { return Some(0.0); }
        let cpu = me.path("rts.cpu");
        let income = cpu.get("income").num_or(0.0);
        let think = 1.5 * peak;
        let held: f64 = cpu.get("reserve").entries().filter(|(lane, _)| *lane != "fabrication").map(|(lane, v)| {
            let reserve = v.num_or(0.0);
            let amount = if reserve <= 1.0 { reserve * income } else { reserve };
            (amount - cpu.get("spent").get(lane).num_or(0.0) - if lane == "think" { think } else { 0.0 }).max(0.0)
        }).sum();
        // Think reduces the shared bank, not the fabrication lane's limit. Subtracting it from `fab` made every
        // recruit wait forever once a successful planner's peak grew larger than that independent lane.
        Some(n(fab).min((cpu.get("bank").num_or(0.0) - think - held).max(0.0)))
    }

    // ---- spawning ----

    /// How many more units your cycles can keep with upkeep within `share` of your income (0.4 is the usual), under your
    /// unit limit (units you hired out count toward it while they're away). Past your cap (CPU RTS) each unit's upkeep
    /// is multiplied by units ÷ cap, so from there the count grows with the square root of income. A unit's upkeep is
    /// last second's average, or `me.costs.upkeep`.
    pub fn room(me: View, share: f64) -> f64 {
        let u = me.get("units");
        let total = n(u.get("total"));
        let upkeep = n(me.get("costs").get("upkeep"));
        let last = me.path("rts.cpu.last.upkeep");
        let each = if last.truthy() && total != 0.0 { upkeep.max(n(last) / total / u.get("upkeepRate").or(1.0)) } else { upkeep };
        let budget = n(me.get("compute").get("cycles")) * share;
        let flat = (budget / each).floor();
        let cap = n(u.get("cap"));
        let max = u.get("max");
        let keep = if !max.truthy() || flat <= cap { flat } else { (budget * cap / each).sqrt().floor() };
        (if max.truthy() { n(max) } else { cap }).min(keep) - total - n(u.get("queued")) - u.get("hiredOut").or(0.0)
    }

    /// How [`recruit`] spends: `reserve` ◆ to keep (0), `limits` the most of a type, `builds` the body each type is
    /// spawned with, `upkeep` the share of income upkeep may take (0.4).
    pub struct Recruit<'a> {
        pub reserve: f64,
        pub limits: &'a [(&'a str, f64)],
        pub builds: &'a [(&'a str, Value)],
        pub upkeep: f64,
    }

    impl Default for Recruit<'_> {
        fn default() -> Self {
            Recruit { reserve: 0.0, limits: &[], builds: &[], upkeep: 0.4 }
        }
    }

    /// Sets `orders.spawn`: up to 2 units a second toward the target `mix` of your types (e.g. `&[("miner", 0.6),
    /// ("guard", 0.4)]`), as many as [`room`] allows, your ◆ and (CPU RTS) the fabrication lane. `namer(type, n)` names the
    /// n-th unit you ever spawned.
    pub fn recruit<N: Into<Value>>(orders: &mut Value, me: View, mix: &[(&str, f64)], mut namer: impl FnMut(&str, f64) -> N, opts: &Recruit) {
        let u = me.get("units");
        let coin = ((n(me.get("bank")) - opts.reserve) / (n(me.get("costs").get("unit")) + 2.0)).floor();
        let mut count = room(me, opts.upkeep).min(2.0).min(coin);
        if let Some(room) = fabrication_room(me) {
            count = count.min((room / fabrication(me, 0.0)).floor());
        }
        if count.is_nan() || count <= 0.0 {
            return;
        }
        let by_type = u.get("byType");
        // what you'd have of each type: the game's count, plus what this call spawns
        let mut more: Vec<(&str, f64)> = Vec::new();
        let tot = n(u.get("total")) + n(u.get("queued"));
        let mut k = u.get("spawned").or(0.0);
        orders.set("spawn", Value::arr());
        let mut i = 0.0;
        while i < count {
            let have = |more: &[(&str, f64)], t: &str| more.iter().find(|m| m.0 == t).map_or_else(|| by_type.get(t).or(0.0), |m| m.1);
            let (mut best, mut gap) = (None, -1e9);
            for &(t, w) in mix {
                let had = have(&more, t);
                if opts.limits.iter().rev().find(|l| l.0 == t).is_some_and(|l| had >= l.1) {
                    continue;
                }
                let g = w * (tot + i + 1.0) - had;
                if g > gap {
                    (best, gap) = (Some(t), g);
                }
            }
            let Some(t) = best else { break };
            let had = have(&more, t) + 1.0;
            match more.iter_mut().find(|m| m.0 == t) {
                Some(m) => m.1 = had,
                None => more.push((t, had)),
            }
            k += 1.0;
            let mut sp = obj! { "type" => t, "name" => namer(t, k) };
            if let Some((_, b)) = opts.builds.iter().rev().find(|b| b.0 == t).filter(|b| b.1.truthy()) {
                sp.set("build", b.clone());
            }
            orders.entry("spawn").push(sp);
            i += 1.0;
        }
    }

    /// The n-th name from `list` for a unit of yours (`army::name`): a seat added after the epoch began starts elsewhere
    /// in the list, so two factions on the same program don't field twins.
    pub fn name(me: View, list: &[&str], k: f64) -> String {
        let id = n(me.get("id"));
        let seat = me.get("seat").num();
        let later = id > 5.0 && (seat.is_nan() || seat < 0.0);
        army::name(list, k + if later { id * 5.0 } else { 0.0 })
    }

    // ---- recovering after an economic loss ----

    // Local capital RAM; 224–239 belong to house ledgers, 249–250 to plan::share.
    const RECOVERY_PAGE: u32 = 244;
    const RECOVERY_WAIT: f64 = 12.0;

    struct RunningCost {
        unit: View,
        body: f64,
        art: f64,
    }
    struct Economy {
        units: Vec<RunningCost>,
        count: f64,
        cap: f64,
        body: f64,
        art: f64,
    }
    impl Economy {
        fn demand(&self, count: f64, body: f64, art: f64) -> f64 {
            body * (count / self.cap).max(1.0) + art
        }
        fn total(&self) -> f64 {
            self.demand(self.count, self.body, self.art)
        }
    }

    fn body_upkeep(me: View, build: View, modules: f64) -> f64 {
        let costs = me.path("rts.costs.upkeep");
        let stat: f64 = ["hp", "speed", "strength", "carry", "vision"].iter()
            .map(|s| (build.get(s).num_or(2.0) - 2.0).max(0.0)).sum();
        costs.get("base").num_or(me.path("costs.upkeep").num_or(100.0))
            + stat * costs.get("stat").num_or(30.0 * price_level(me))
            + modules * costs.get("module").num_or(40.0 * price_level(me))
            + build.get("range").num_or(0.0) * costs.get("range").num_or(80.0 * price_level(me))
            + if build.get("wings").truthy() { costs.get("flight").num_or(120.0 * price_level(me)) } else { 0.0 }
            + build.get("stealth").num_or(0.0) * costs.get("stealth").num_or(180.0 * price_level(me))
    }

    fn economy(world: View, me: View) -> Economy {
        // The view prices each body, but does not map types to artwork. Use the dearest declared body as a
        // conservative artwork allowance; observed upkeep also includes weapon art and flight burn. Divide by
        // PAID units: dividing by the whole army during a brownout makes its idle units appear nearly free.
        let art = me.path("costs.draw").entries().map(|(_, v)| v.num_or(0.0)).fold(0.0, f64::max);
        let cap = me.path("units.cap").num_or(16.0).max(1.0);
        let mut e = Economy { units: Vec::new(), count: 0.0, cap, body: 0.0, art: 0.0 };
        for u in world.get("units").iter().filter(|u| u.get("owner").num_or(-1.0) == me.get("id").num()) {
            let body = body_upkeep(me, u.get("build"), u.path("rts.modules").len() as f64);
            e.body += body;
            e.art += art;
            e.units.push(RunningCost { unit: u, body, art });
        }
        e.count = e.units.len() as f64;
        let paid = (me.path("units.total").num_or(0.0) - me.path("units.stalled").num_or(0.0)).max(1.0);
        let observed = me.path("rts.cpu.last.upkeep").num_or(0.0) / paid;
        let rate = (e.units.len() as f64 / cap).max(1.0);
        let mean = if e.units.is_empty() { 0.0 } else { e.body * rate / e.units.len() as f64 + art };
        let extra = (observed - mean).max(0.0);
        for u in &mut e.units { u.art += extra; }
        e.art += extra * e.units.len() as f64;
        e
    }

    fn recovery_economy(world: View, me: View) -> Economy {
        let cap = me.path("units.cap").num_or(16.0).max(1.0);
        let count = me.path("units.total").num_or(0.0);
        let rate = (count / cap).max(1.0);
        let paid = (count - me.path("units.stalled").num_or(0.0)).max(1.0);
        let art = me.path("costs.draw").entries().map(|(_, v)| v.num_or(0.0)).fold(0.0, f64::max);
        // Last second's paid upkeep includes the actual bodies, modules and artwork. Its average is a starting
        // estimate, not a claim that unpaid units cost zero. Re-evaluate after every bounded retirement batch.
        let body = (me.path("rts.cpu.last.upkeep").num_or(0.0) / paid / rate)
            .max(me.path("costs.upkeep").num_or(100.0));
        // Rotate a bounded window even at the faction limit; an emergency never has to rank 512 bodies at once.
        let start = ((world.get("seconds").num_or(0.0) as usize / 2) * 96) % (count as usize).max(1);
        let units: Vec<RunningCost> = world.get("units").iter()
            .filter(|u| u.get("owner").num_or(-1.0) == me.get("id").num())
            .enumerate().filter(|(i, _)| (*i + count as usize - start) % (count as usize).max(1) < 96)
            .map(|(_, unit)| RunningCost { unit, body, art }).collect();
        Economy { cap, count, body: body * count, art: art * count, units }
    }

    /// Runs a house's own strategy, with its own economic safety policy around every return path. Classic worlds
    /// keep the house's strategy. No engine allocation, exemption or rule is involved.
    pub fn govern(world: View, me: View, share: f64, strategy: impl FnOnce() -> Value) -> Value {
        if !me.path("rts.cpu").truthy() { return strategy(); }
        if cycles_left() < 6000.0 { return recovery(world, me); }
        if me.path("units.stalled").num_or(0.0) > 0.0 { return recovery(world, me); }
        let income = me.path("rts.cpu.income").num_or(20000.0).max(1.0);
        let bank = me.path("rts.cpu.bank").num_or(0.0);
        // With no stalled units, last second's paid bill is the whole army's actual upkeep. Keep this
        // entry check cheap; a detailed body scan belongs after the planner, only when it proposes a recruit.
        let shortage = me.path("rts.cpu.last.upkeep").num_or(0.0) > income * 0.7 && bank < income * 8.0;
        let was = mem::get(me.get("ramCapital"), RECOVERY_PAGE);
        // Stay in recovery until upkeep fits and a little headroom has returned, rather than switching the old
        // spending back on the first second in which upkeep happened to fit.
        if shortage || (was.get("r").truthy() && bank < income * 2.0) {
            let e = recovery_economy(world, me);
            return recover_with(world, me, &e, shortage);
        }
        let mut orders = strategy();
        // A short strategy can itself have chosen recovery; its positive miner allocation must survive this wrapper.
        if orders.get("memo").str() == "Recovering production" { return orders; }
        if was.get("r").truthy() {
            ram(&mut orders, mem::free(RECOVERY_PAGE, 1, None));
        }
        if orders.get("spawn").is_empty() { return orders; }
        if cycles_left() < 1500.0 + me.path("units.total").num_or(0.0) * 100.0 {
            orders.set("spawn", Value::arr());
            return orders;
        }
        let e = economy(world, me);
        let mut count = e.units.len() as f64 + me.path("units.queued").num_or(0.0) + me.path("units.hiredOut").num_or(0.0);
        // Queued and hired-out bodies still need room when they arrive. Charge the current mean, never zero.
        let mean = (e.body / (e.units.len() as f64).max(1.0)).max(body_upkeep(me, View::undefined(), 0.0));
        let mut body = e.body + (count - e.units.len() as f64) * mean;
        let mut art = e.art + (count - e.count) * (e.art / e.count.max(1.0));
        let mut keep = Vec::new();
        for sp in orders.get("spawn").items() {
            let b = body_upkeep_value(me, sp.get("build"), sp.get("modules").len() as f64);
            let a = (e.art / (e.units.len() as f64).max(1.0)).max(me.path("costs.draw").entries().map(|(_, v)| v.num_or(0.0)).fold(0.0, f64::max));
            if e.demand(count + 1.0, body + b, art + a) <= income * share {
                count += 1.0; body += b; art += a;
                let mut recruit = sp.clone();
                if !sp.get("name").truthy() {
                    let ty = sp.get("type").str();
                    let role = match ty {
                        "miner" => "miner", "scout" => "scout", "ghost" | "shade" | "spy" => "spy",
                        "skirm" | "lancer" | "gun" | "wing" | "sentry" | "needle" => "longbow", _ => "guard",
                    };
                    let serial = me.path("units.spawned").num_or(0.0) + keep.len() as f64 + 1.0;
                    let list = theme::of(me.get("seat"), -1.0).and_then(|house| house.names_for(ty).or_else(|| house.names_for(role)));
                    recruit.set("name", list.map_or_else(|| format!("{ty} {serial:.0}"), |list| name(me, list, serial)));
                }
                keep.push(recruit);
            }
        }
        orders.set("spawn", keep);
        orders
    }

    fn body_upkeep_value(me: View, build: &Value, modules: f64) -> f64 {
        // Spawn bodies are owned Values, fielded bodies are Views. Price both through the same public costs.
        let costs = me.path("rts.costs.upkeep");
        let stat: f64 = ["hp", "speed", "strength", "carry", "vision"].iter()
            .map(|s| (build.get(s).num_or(2.0) - 2.0).max(0.0)).sum();
        costs.get("base").num_or(me.path("costs.upkeep").num_or(100.0))
            + stat * costs.get("stat").num_or(30.0 * price_level(me))
            + modules * costs.get("module").num_or(40.0 * price_level(me))
            + build.get("range").num_or(0.0) * costs.get("range").num_or(80.0 * price_level(me))
            + if build.get("wings").truthy() { costs.get("flight").num_or(120.0 * price_level(me)) } else { 0.0 }
            + build.get("stealth").num_or(0.0) * costs.get("stealth").num_or(180.0 * price_level(me))
    }

    /// A cheap fallback used by every house's low-budget branch. Even when a map scan cannot fit, clear optional
    /// standing spending and old reserves, and leave real controller cycles for miners on the following second.
    pub fn recovery(world: View, me: View) -> Value {
        if cycles_left() < 6000.0 {
            let mut orders = recovery_policy(me);
            ram(&mut orders, mem::put(RECOVERY_PAGE, &obj! { "r" => true }, "local", None, 1).unwrap_or_default());
            return orders;
        }
        let e = recovery_economy(world, me);
        recover_with(world, me, &e, me.path("units.stalled").num_or(0.0) > 0.0)
    }

    fn recovery_policy(me: View) -> Value {
        let income = me.path("rts.cpu.income").num_or(20000.0).max(1.0);
        let bank = me.path("rts.cpu.bank").num_or(0.0);
        let reserve = |v: f64| if v.floor() > 1.0 { v.floor() } else { 0.0 };
        let mut types = obj! {};
        for t in me.get("types").keys() { types.set(t, obj! { "share" => 0, "every" => 8 }); }
        types.set("miner", obj! { "cycles" => (income * 0.4).max(4000.0), "every" => 1 });
        obj! {
            "compute" => obj! { "cpu" => obj! {
                "limits" => obj! { "think" => if bank > income * 2.0 { income * 1.2 } else { income * 0.7 }, "control" => 0.5, "upkeep" => 0.65,
                    "research" => 0, "influence" => 0, "fabrication" => 0, "visuals" => 0,
                    "reflex" => 0.15, "duel" => 0.08, "challenge" => 0.02,
                    "drive" => 0.08, "armor" => 0.05, "weapon" => 0.08, "abilities" => 0, "cloak" => 0, "gate" => 0 },
                "reserve" => obj! { "upkeep" => reserve((bank * 0.2).min(income * 0.3)),
                    "control" => reserve((bank * 0.1).min(income * 0.2)) },
                "upkeepOrder" => vec!["miner"]
            }, "types" => types },
            "propaganda" => Value::arr(), "preach" => Value::arr(), "rent" => 0,
            "memo" => "Recovering production"
        }
    }

    fn recover_with(world: View, me: View, e: &Economy, shortage: bool) -> Value {
        let mut orders = recovery_policy(me);
        let now = world.get("seconds").num_or(0.0);
        let old = mem::get(me.get("ramCapital"), RECOVERY_PAGE);
        let since = if shortage { old.get("s").num_or(now).min(now) } else { now };
        ram(&mut orders, mem::put(RECOVERY_PAGE, &obj! { "r" => true, "s" => since }, "local", None, 1).unwrap_or_default());
        let ours = me.get("id").num();
        // Price mining rights once per province, not once per miner. Stacked armies often have dozens in the
        // same town, and the emergency call must still fit the floor after the capital's RAM read.
        let mut sites = Vec::<(f64, usize)>::new();
        let mut mines = Vec::new();
        for p in world.get("provinces").iter() {
            let id = p.get("id").num();
            let connected = p.get("owner").num_or(-1.0) == ours && p.get("hops").num_or(-1.0) >= 0.0;
            // A miner already working a leased mine is productive even outside the capital's RAM network.
            let working_lease = !connected && p.get("staff").truthy() && e.units.iter().any(|c| {
                let u = c.unit;
                u.get("type").str() == "miner" && u.get("at").num() == id && u.get("doing").str() == "gather"
            });
            if !connected && !working_lease { continue; }
            let mut slots = 0;
            for (k, value) in p.get("mines").iter().enumerate() {
                if value.num_or(0.0) <= 0.0 || realm::worker(p, k as f64) != ours { continue; }
                slots += 2;
                let staff = p.get("staff").at(k).num_or(0.0).max(0.0) as usize;
                for _ in staff..2 { if connected && mines.len() < 6 { mines.push(arr![id, k]); } }
            }
            if slots > 0 { sites.push((id, slots)); }
        }
        let have_mines = !sites.is_empty();
        plan::share(&mut orders, &obj! { "home" => me.get("capital"), "stage" => me.get("capital"), "attack" => -1, "mines" => mines }, me);
        let mut ranked: Vec<(usize, f64)> = e.units.iter().enumerate().map(|(i, c)| {
            let u = c.unit;
            let work = sites.iter().any(|(id, _)| *id == u.get("at").num());
            let miner = u.get("type").str() == "miner";
            let score = if miner && work { if u.get("doing").str() == "gather" { 400.0 } else { 300.0 } }
                else if !miner && u.get("at").num() == me.get("capital").num() && !u.get("air").truthy() && !u.path("rts.sea.ship").truthy() { 200.0 }
                else if u.path("rts.sea.ship").truthy() { 100.0 }
                else if miner { 50.0 } else { 0.0 };
            (i, score - (c.body + c.art) / 100000.0)
        }).collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        let mut protected = Vec::new(); let mut crew = Vec::<(f64, usize)>::new(); let mut defense = false; let mut bridge = false;
        for &(i, score) in &ranked {
            let u = e.units[i].unit;
            if score > 250.0 {
                let at = u.get("at").num();
                let slots = sites.iter().find(|(id, _)| *id == at).map_or(0, |(_, n)| *n);
                let entry = match crew.iter().position(|(p, _)| *p == at) { Some(j) => j, None => { crew.push((at, 0)); crew.len()-1 } };
                if crew[entry].1 < slots && protected.len() < 6 { crew[entry].1 += 1; protected.push(i); }
            } else if score > 150.0 && !defense { protected.push(i); defense = true; }
            else if score > 75.0 && !bridge { protected.push(i); bridge = true; }
        }
        if let Some(&i) = protected.iter().find(|&&i| e.units[i].unit.get("type").str() != "miner" && !e.units[i].unit.path("rts.sea.ship").truthy()) {
            let ty = e.units[i].unit.get("type").str();
            orders.entry("compute").entry("types").set(ty, obj! { "cycles" => 3000, "every" => 4 });
            orders.entry("compute").entry("cpu").set("upkeepOrder", vec!["miner", ty]);
        }
        // Do not leave a faction without a miner merely because all its miners stand one hop from a useful mine.
        if !protected.iter().any(|&i| e.units[i].unit.get("type").str() == "miner") {
            if let Some(&(i, _)) = ranked.iter().find(|(i, _)| e.units[*i].unit.get("type").str() == "miner") { protected.push(i); }
        }
        let income = me.path("rts.cpu.income").num_or(20000.0).max(1.0);
        if shortage && now - since >= RECOVERY_WAIT {
            let (mut count, mut body, mut art) = (e.count, e.body, e.art);
            let mut disband = Vec::new();
            for &(i, _) in ranked.iter().rev() {
                if e.demand(count, body, art) <= income * 0.55 || disband.len() >= 16 { break; }
                if protected.contains(&i) || e.units[i].unit.get("hired").truthy() { continue; }
                disband.push(e.units[i].unit.get("id").value());
                count -= 1.0; body -= e.units[i].body; art -= e.units[i].art;
            }
            orders.set("disband", disband);
        }
        // A completely wiped mining crew can bootstrap again without waiting for the full strategy's planner.
        if !e.units.iter().any(|c| c.unit.get("type").str() == "miner") && have_mines
            && me.path("units.queued").num_or(0.0) == 0.0 && me.get("bank").num_or(0.0) >= me.path("costs.unit").num_or(1e9)
            && me.path("rts.cpu.bank").num_or(0.0) >= fabrication(me, 0.0) + income {
            orders.entry("compute").entry("cpu").entry("limits").set("fabrication", fabrication(me, 0.0) + 100.0);
            orders.set("spawn", vec![obj! { "type" => "miner" }]);
        }
        orders
    }

    /// The emergency miner call uses current, local mining rights, even when its carried plan is old or its previous
    /// control allowance was zero. Local orders need no route search or carried-plan write.
    pub fn recover_miners(units: View, world: View, me: View) -> bool {
        if !me.path("rts.cpu").truthy() || !mem::get(me.get("ramCapital"), RECOVERY_PAGE).get("r").truthy() { return false; }
        let index = realm::index(world); let ours = me.get("id").num();
        let mut assigned = Vec::<(f64, usize, usize)>::new();
        for u in units.iter() {
            if cycles_left() < 1600.0 { break; }
            let p = index.at(u.get("at"));
            if p.get("owner").num_or(-1.0) != ours || p.get("hops").num_or(-1.0) < 0.0 { continue; }
            let old_slot = u.get("slot").num_or(-1.0);
            if u.get("doing").str() == "gather" && u.get("target").num() == u.get("at").num()
                && old_slot >= 0.0 && old_slot < p.get("mines").len() as f64
                && realm::worker(p, old_slot) == ours { continue; }
            let slot = p.get("mines").iter().enumerate().find(|(k, v)| {
                let added = assigned.iter().find(|(at, slot, _)| *at == p.get("id").num() && *slot == *k).map_or(0, |(_, _, n)| *n);
                v.num_or(0.0) > 0.0 && realm::worker(p, *k as f64) == ours && p.get("staff").at(*k).num_or(0.0) + (added as f64) < 2.0
            });
            if let Some((k, _)) = slot {
                order(obj! { "id" => u.get("id"), "do" => "gather", "target" => u.get("at"), "slot" => k });
                match assigned.iter_mut().find(|(at, slot, _)| *at == p.get("id").num() && *slot == k) {
                    Some((_, _, n)) => *n += 1, None => assigned.push((p.get("id").num(), k, 1)),
                }
            } else if let Some(o) = orders::gather(u, world, me) { order(o); }
        }
        true
    }

    // ---- cycles and research ----

    /// Each unit type's call for `compute.types`, from `types` ([type, base, every], in the order they come first). A
    /// type's call is its base (enough for its orders and one route search), 6% of `income`, or 2,400 and 300 a unit of
    /// the type you field, whichever is most, so a call grows with the faction and with its army; a base of 1 or less
    /// is a share of the lane, passed on as it is. Of what the control lane holds (`room`), every type you field gets
    /// its base first, then the rest of its call in the listed order while the lane lasts. When the lane can't pay
    /// every base, the types take turns, starting one further along the list each second (`world.seconds`): a type
    /// left out keeps its standing orders that second, and none sits out for good. The game would otherwise cut every
    /// call to a sliver of what it asked.
    pub fn calls(world: View, types: &[(&str, f64, f64)], income: f64, room: f64, fielded: View) -> Value {
        // what each type asks; the types you field with a call in cycles share the lane, the rest get theirs as asked
        let mut want = Vec::with_capacity(types.len());
        let mut share = Vec::with_capacity(types.len());
        let mut field = Vec::new();
        for (i, &(t, base, _)) in types.iter().enumerate() {
            let k = fielded.get(t).or(0.0);
            let w = if base > 1.0 { base.max(income * 0.06).max(2400.0 + 300.0 * k).round() } else { base };
            want.push(w);
            if k != 0.0 && w > 1.0 {
                field.push(i);
                share.push(0.0);
            } else {
                share.push(w);
            }
        }
        let mut room = room;
        let bases: f64 = field.iter().map(|&i| types[i].1).sum();
        let turn = if bases > room && !field.is_empty() { world.get("seconds").num_or(0.0).max(0.0) as usize % field.len() } else { 0 };
        for j in 0..field.len() {
            let i = field[(turn + j) % field.len()];
            if room >= types[i].1 {
                share[i] = types[i].1;
                room -= types[i].1;
            }
        }
        for &i in &field {
            if share[i] > 0.0 {
                let more = (want[i] - share[i]).min(room.floor()).max(0.0);
                share[i] += more;
                room -= more;
            }
        }
        let mut out = Value::obj();
        for (&(t, _, every), &s) in types.iter().zip(&share) {
            out.set(t, obj! { "share" => s, "every" => every });
        }
        out
    }

    /// How [`budget`] splits: `research` the most research gets (0.1), `techs` what to research, in order, `code` your
    /// units' code's share of your cycles (0.25), `types` how that's handed out (`compute.types`).
    pub struct Budget<'a> {
        pub research: f64,
        pub techs: &'a [&'a str],
        pub code: f64,
        pub types: Option<&'a Value>,
    }

    impl Default for Budget<'_> {
        fn default() -> Self {
            Budget { research: 0.1, techs: &[], code: 0.25, types: None }
        }
    }

    /// Sets `orders.compute`, `orders.rent` and `orders.research`:
    /// - think keeps about 1.6× the most it used in the last 5 seconds, up to 85% (60% while you have units, so they
    ///   always have cycles to run on);
    /// - units get their upkeep and what drawing your own sprites cost last second, plus `opts.code` of your cycles for
    ///   their code, which `opts.types` hands out; what code doesn't spend banks for duels;
    /// - research gets what's left, up to `opts.research`;
    /// - spare ◆ (beyond two units' worth + 100) rents cloud compute when you're at the unit cap or hoarding;
    /// - research: the first tech in `opts.techs` you can start, in the list's order (a route tech goes first only while
    ///   your land has sea routes or passes and no land of a rival's or free land borders it by land: nothing left to
    ///   take without the crossing). In the CPU RTS it passes over a tech whose offer is more than your CPU bank holds, unless every one is:
    ///   then the cheapest of them. It keeps the tech under way, though: each tech keeps its own steps solved, so a pick
    ///   that flipped as the bank crossed an offer would spread them over several. A listed tech whose prerequisites you
    ///   lack leads to them first (and theirs), so a list naming airWeapons gets flight. Ids the world lacks are skipped.
    pub fn budget(orders: &mut Value, world: View, me: View, opts: &Budget) {
        let c = me.get("compute");
        let u = me.get("units");
        let cyc = n(c.get("cycles")).max(1.0);
        let (total, queued) = (n(u.get("total")), n(u.get("queued")));
        let each = n(me.get("costs").get("upkeep"));
        let peak = if c.get("thinkPeak").is_nullish() { c.get("used").get("think") } else { c.get("thinkPeak") };
        let think = (1.6 * n(peak) / cyc).max(0.15).min(if total > 0.0 { 0.6 } else { 0.85 });
        let draw = (c.get("used").get("units").or(0.0) - total * each).max(0.0) / cyc;
        let units = ((total + queued) * each / cyc + draw + opts.code).min(1.0 - think);
        let mut compute = obj! { "units" => units, "research" => opts.research.min(1.0 - think - units).max(0.0) };
        if let Some(t) = opts.types.filter(|t| t.truthy()) {
            compute.set("types", t.clone());
        }
        orders.set("compute", compute);
        let spare = n(me.get("bank")) - 2.0 * n(me.get("costs").get("unit")) - 100.0;
        let rent = if spare > 0.0 && (total + queued >= n(u.get("cap")) || spare > 600.0) { (n(c.get("rentMax")) / n(c.get("rentRate"))).min(spare / 20.0) } else { 0.0 };
        orders.set("rent", rent);
        let me_id = me.get("id").num();
        let (mut sea, mut pass) = (false, false);
        for p in world.get("provinces").iter().filter(|p| p.get("owner").num() == me_id) {
            sea |= !p.path("routes.sea").is_empty();
            pass |= !p.path("routes.pass").is_empty();
        }
        let crossing = |t: &str| (t == "seaTraversal" && sea) || (t == "mountainTraversal" && pass);
        // whether land of a rival's or free land borders yours by land (asked only when a crossing could jump the queue)
        let land = !opts.techs.iter().any(|t| crossing(t) && !me.get("techs").get(t).truthy()) || {
            let by_id = realm::index(world);
            world.get("provinces").iter().filter(|p| p.get("owner").num() == me_id).any(|p| {
                p.get("adj").iter().any(|a| {
                    let o = by_id.at(a).get("owner").num_or(-1.0);
                    o != me_id && (o < 0.0 || realm::is_rival(world, me, o))
                })
            })
        };
        let open = |t: &str| !land && crossing(t);
        let (all, mine) = (world.get("techs"), me.get("techs"));
        let ready = |t: &str| {
            let x = all.get(t);
            !mine.get(t).truthy() && x.truthy() && x.get("requires").iter().all(|q| mine.get(q.str()).truthy())
        };
        let capacity = me.path("rts.cpu.capacity");
        let now = me.path("research.tech").str();
        let (mut pick, mut big): (Option<String>, Option<String>) = (None, None);
        for want in opts.techs.iter().filter(|t| open(t)).chain(opts.techs.iter().filter(|t| !open(t))) {
            let t = first(all, mine, want);
            if !ready(&t) {
                continue;
            }
            let offer = n(all.get(&t).get("offer"));
            if capacity.truthy() && offer > n(capacity) && now != t {
                if big.as_ref().is_none_or(|b| offer < n(all.get(b).get("offer"))) {
                    big = Some(t);
                }
                continue;
            }
            pick = Some(t);
            break;
        }
        if let Some(p) = pick.or(big).filter(|p| *p != now) {
            orders.set("research", p);
        }
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

    /// The duel lane's share of income ([`fit_cpu`]): the most it may spend a second, and what it keeps reserved while
    /// your units are out fighting or capturing, never less than `DUEL_FLOOR` cycles (a duel() call costs a few
    /// hundred).
    pub const DUEL_LIMIT: f64 = 0.25;
    pub const DUEL_RESERVE: f64 = 0.08;
    pub const DUEL_FLOOR: f64 = 3000.0;

    /// Fits a house program's CPU policy to what its bank can pay. Think's limit becomes a share of income (a fraction
    /// up to 1, cycles past that), so it grows with the faction even in a second whose think ran out and set no new
    /// policy: the policy's own limit, or half again think's recent peak when that's more. Duels get at least
    /// [`DUEL_LIMIT`] of income a second, and while units fight or capture [`DUEL_RESERVE`] of it ([`DUEL_FLOOR`]
    /// cycles at least) kept for them: each province where your units are attacked answers its own duel a second, and
    /// prices (upkeep, power, abilities) would otherwise spend the bank down to the other lanes' reserves first. A duel
    /// lost unanswered lands at full strength. The reserves together keep at most a third of the bank now, the duel's
    /// first: a reserve the bank can't cover leaves every other lane nothing, and unit calls run dry. With fewer than 4
    /// units research waits: on the floor's income a second can't pay think, research and a body.
    pub fn fit_cpu(policy: &mut Value, me: View) {
        let c = me.path("rts.cpu");
        if !c.truthy() {
            return;
        }
        if n(me.path("units.total")) + n(me.path("units.queued")) < 4.0 {
            policy.entry("limits").set("research", 0);
            policy.entry("reserve").set("research", 0);
        }
        let income = n(c.get("income")).max(1.0);
        let amount = |v: f64| if v <= 1.0 { v * income } else { v };
        let want = amount(policy.get("limits").get("think").num_or(0.0)).max(1.5 * n(me.get("compute").get("thinkPeak")));
        policy.entry("limits").set("think", if want <= income { (want / income * 1000.0).ceil() / 1000.0 } else { want.ceil() });
        if amount(policy.get("limits").get("duel").num_or(0.0)) < DUEL_LIMIT * income {
            policy.entry("limits").set("duel", DUEL_LIMIT);
        }
        let doing = me.get("units").get("doing");
        let fighting = doing.get("fight").or(0.0) + doing.get("capture").or(0.0) > 0.0;
        let mut duel = amount(policy.get("reserve").get("duel").num_or(0.0));
        if fighting {
            duel = duel.max(DUEL_RESERVE * income).max(DUEL_FLOOR).ceil();
            policy.entry("reserve").set("duel", duel);
        }
        let held: f64 = match policy.get("reserve") {
            Value::Obj(r) => r.0.iter().filter(|(lane, _)| lane.as_str() != "duel").map(|(_, v)| amount(v.num_or(0.0))).sum(),
            _ => 0.0,
        };
        // the duel's reserve comes first in the room; the others share what's left of it
        let room = n(c.get("bank")).min(c.get("capacity").num_or(f64::INFINITY)) / 3.0;
        let left = (room - duel).max(0.0);
        if duel > room || held > left {
            let k = if held > 0.0 { (left / held).min(1.0) } else { 1.0 };
            if let Value::Obj(r) = policy.entry("reserve") {
                for (lane, v) in &mut r.0 {
                    // cycles now (1 or less would read as a share of income)
                    let x = if lane.as_str() == "duel" { duel.min(room).floor() } else { (amount(v.num_or(0.0)) * k).floor() };
                    *v = Value::Num(if x > 1.0 { x } else { 0.0 });
                }
            }
        }
    }

    // ---- deals and pacts ----

    /// The default rule's price of land, ◆ a core, and the fewest provinces it keeps ([`price`]).
    pub const LAND_PRICE: f64 = 700.0;
    pub const MIN_LAND: f64 = 4.0;

    /// What a bundle is worth to the default rule: its ◆, 120 a province, 40 a unit, plus [`worth`].
    pub fn value(b: &Value) -> f64 {
        if !b.truthy() {
            return 0.0;
        }
        b.get("coin").num_or(0.0) + b.get("provinces").len() as f64 * 120.0 + b.get("units").len() as f64 * 40.0 + worth(b)
    }

    /// What a tech sold is worth: 150 ◆, but 600 for the third tier and 900 for the fourth, which take tens of millions of
    /// cycles to research.
    pub fn tech_worth(id: &str) -> f64 {
        match id {
            "computeBoost" | "flight" | "stealth" => 600.0,
            "gate" | "airWeapons" => 900.0,
            _ => 150.0,
        }
    }

    /// What the goods that run over time in bundle `b` are worth: ◆ a second × seconds, a tech sold at [`tech_worth`], a
    /// quarter ◆ a second for a tech lease, a sixth for each leased mine slot, a twelfth for relay rights, a thirtieth for sight, a
    /// sixtieth a unit (up to 24) for passage, and a tenth a unit for a hire.
    pub fn worth(b: &Value) -> f64 {
        let secs = |v: &Value| v.get("seconds").num_or(0.0);
        let mut v = b.get("coinPerSecond").num_or(0.0) * secs(b) + if let Value::Arr(a) = b.get("techs") { a.iter().map(|t| if let Value::Str(s) = t { tech_worth(s) } else { 150.0 }).sum::<f64>() } else { 0.0 };
        let lease = b.get("techLease");
        v += lease.get("techs").len() as f64 * secs(lease) / 4.0;
        let mines = b.get("mines");
        let list = if let Value::Arr(a) = mines { a.as_slice() } else { std::slice::from_ref(mines) };
        for m in list.iter().filter(|m| m.truthy()) {
            v += or(m.get("slots"), 1.0) * secs(m) / 6.0;
        }
        let p = b.get("passage");
        v += p.get("units").num_or(0.0).min(24.0) * secs(p) / 60.0 + secs(b.get("sight")) / 30.0 + secs(b.get("relay")) / 12.0;
        let h = b.get("hire");
        let k = if h.get("units").truthy() { h.get("units").len() as f64 } else { or(h.get("count"), 0.0) };
        v + k * secs(h) / 10.0
    }

    /// What the default rule asks for your provinces `ids` from faction `buyer`, or infinity when it won't sell them:
    /// [`LAND_PRICE`] ◆ a core plus half that a mine slot, times scarcity (1 + 4 ÷ the provinces you'd keep) and the
    /// buyer's size (1 + 4 × its share of the world). Never your capital, never to the leader, and never below
    /// [`MIN_LAND`] provinces. A buyer prices a bid with the seller's entry in `world.factions` as `me`.
    pub fn price(world: View, me: View, ids: &[f64], buyer: f64) -> f64 {
        if ids.is_empty() {
            return 0.0;
        }
        let keep = me.get("provinces").or(0.0) - ids.len() as f64;
        let f = world.get("factions").key(buyer);
        let by_id = realm::index(world);
        let me_id = me.get("id").num();
        if ids.contains(&me.get("capital").num()) || keep < MIN_LAND || !f.truthy() || realm::leader(world).is_some_and(|l| l.get("id").num() == buyer) {
            return f64::INFINITY;
        }
        let mut worth = 0.0;
        for &id in ids {
            let p = by_id.get(id);
            if p.get("owner").num() != me_id {
                return f64::INFINITY;
            }
            worth += LAND_PRICE * (p.get("cores").or(0.0) + 0.5 * p.get("mines").len() as f64).max(1.0);
        }
        (worth * (1.0 + 4.0 / keep) * (1.0 + 4.0 * f.get("share").or(0.0)) + 0.5).floor()
    }

    /// The default rule for deal `m` in your inbox: yes when what you'd get is worth at least what you'd give
    /// ([`value`], with your land at [`price`]), and only when your bank holds every ◆ you'd give (all of a stream) and
    /// the units you'd get fit under your unit limit. A tech you hold, or that isn't in `techs` (what you research), is
    /// worth nothing to you, and so are passage, sight and relay rights you hold from the same faction for more than
    /// another minute: it buys none of them twice. It sells no passage to a faction whose units stand in your land, unless it renews passage
    /// that faction has from you (passage ends the moment its holder attacks). It takes no mine lease in a province
    /// you're besieging, and no passage from a faction whose land you're besieging: either would stop your own siege.
    /// With `world` undefined it sells no land and no passage.
    pub fn fair(m: View, me: View, world: View, techs: &[&str]) -> bool {
        let (give, ask) = (m.get("give"), m.get("ask"));
        let from = m.get("from").num();
        let me_id = me.get("id").num();
        if ask.get("coin").or(0.0) + ask.get("coinPerSecond").or(0.0) * ask.get("seconds").or(1.0) > n(me.get("bank")) {
            return false;
        }
        // rights you hold from them already, for more than another minute
        let held = |right: &str| {
            give.get(right).truthy()
                && me.get("contracts").iter().any(|c| c.get("with").num() == from && !c.get("ended").truthy() && c.get("get").get(right).truthy() && c.get("endsIn").or(0.0) > 60.0)
        };
        if held("passage") || held("sight") || held("relay") {
            return false;
        }
        let hire = give.get("hire");
        let units = give.get("units").len() as f64 + if hire.get("units").truthy() { hire.get("units").len() as f64 } else { hire.get("count").or(0.0) };
        let mu = me.get("units");
        let limit = if mu.get("max").truthy() { n(mu.get("max")) } else { n(mu.get("cap")) };
        if units > 0.0 && mu.get("total").or(0.0) + mu.get("queued").or(0.0) + mu.get("hiredOut").or(0.0) + units > limit {
            return false;
        }
        if ask.get("passage").truthy() {
            let renew =
                me.get("contracts").iter().any(|c| c.get("with").num() == from && !c.get("ended").truthy() && c.get("give").get("passage").truthy());
            if !world.truthy() || (!renew && world.get("provinces").iter().any(|p| p.get("owner").num() == me_id && p.get("units").key(from).or(0.0) > 0.0)) {
                return false;
            }
        }
        if world.truthy() {
            let besieged = |p: View| p.get("siege").get("by").num() == me_id;
            let by_id = realm::index(world);
            let mines = give.get("mines");
            let leased: Vec<View> = if mines.is_arr() { mines.iter().collect() } else { vec![mines] };
            if leased.iter().any(|l| l.truthy() && besieged(by_id.get(l.get("province").num())))
                || (give.get("passage").truthy() && world.get("provinces").iter().any(|p| p.get("owner").num() == from && besieged(p)))
            {
                return false;
            }
        }
        let lands: Vec<f64> = ask.get("provinces").iter().map(|p| p.num()).collect();
        let land = if lands.is_empty() {
            0.0
        } else if world.truthy() {
            price(world, me, &lands, from)
        } else {
            f64::INFINITY
        };
        // techs you hold, or don't research, are worth nothing to you
        let want = |t: &View| !me.get("techs").get(t.str()).truthy() && techs.contains(&t.str());
        let keep = |list: View| Value::Arr(list.iter().filter(want).map(|t| t.value()).collect());
        let mut get = give.value();
        if give.get("techs").is_arr() {
            get.set("techs", keep(give.get("techs")));
        }
        let lease = give.get("techLease");
        if lease.get("techs").is_arr() {
            get.set("techLease", obj! { "techs" => keep(lease.get("techs")), "seconds" => lease.get("seconds") });
        }
        land < f64::INFINITY && value(&get) >= value(&ask.value()) - lands.len() as f64 * 120.0 + land
    }

    /// Answers the inbox: a deal when [`fair`] says yes (`techs`: what you research) and the deals it took before it this
    /// second leave the ◆ it asks for and room for the units it brings (each is checked against the whole bank, and
    /// two that both fit alone may not fit together), a pact when `pact(m)` says yes and nobody in it, you included, is
    /// in a pact already (a faction belongs to one pact at a time, so the game would refuse it), at most one a second;
    /// plain messages are marked read.
    pub fn answer(orders: &mut Value, me: View, world: View, pact: impl Fn(View) -> bool, techs: &[&str]) {
        let me_id = me.get("id").num();
        let mut joined = !me.get("pacts").is_empty();
        let free = |f: View| world.get("factions").key(n(f)).get("pacts").is_empty();
        let mu = me.get("units");
        let (mut coin, mut room) = (n(me.get("bank")), if mu.get("max").truthy() { n(mu.get("max")) } else { n(mu.get("cap")) } - mu.get("total").or(0.0) - mu.get("queued").or(0.0) - mu.get("hiredOut").or(0.0));
        for m in me.get("inbox").iter() {
            if m.get("accepted").has_num(me_id) {
                continue;
            }
            let yes = if m.get("pact").truthy() {
                let ok = !joined && m.get("pact").get("members").iter().all(free) && pact(m);
                joined |= ok;
                ok
            } else if m.get("give").truthy() || m.get("ask").truthy() {
                let (give, ask) = (m.get("give"), m.get("ask"));
                let pay = ask.get("coin").or(0.0) + ask.get("coinPerSecond").or(0.0) * ask.get("seconds").or(1.0);
                let hire = give.get("hire");
                let units = give.get("units").len() as f64 + if hire.get("units").truthy() { hire.get("units").len() as f64 } else { hire.get("count").or(0.0) };
                let ok = pay <= coin && units <= room.max(0.0) && fair(m, me, world, techs);
                if ok {
                    coin -= pay;
                    room -= units;
                }
                ok
            } else {
                true
            };
            orders.entry(if yes { "accept" } else { "reject" }).push(m.get("id").value());
        }
    }

    /// Factions you could ask for a pact: standing, not you, not the leader, and in no pact yet (a faction belongs to
    /// one), biggest first.
    pub fn partners(world: View, me: View) -> Vec<View> {
        let lead = realm::leader(world).map(|l| l.get("id").num());
        let me_id = me.get("id").num();
        let mut out: Vec<View> = world
            .get("factions")
            .iter()
            .filter(|f| f.get("alive").truthy() && f.get("id").num() != me_id && lead != Some(f.get("id").num()) && f.get("pacts").is_empty())
            .collect();
        out.sort_by(|a, b| n(b.get("share")).total_cmp(&n(a.get("share"))));
        out
    }

    /// How [`court`] backs off: at most one ask per `every` seconds (120); after `patience` (2) asks in a row went
    /// unanswered, `quiet` (300) seconds more of silence.
    pub struct Court {
        pub every: f64,
        pub quiet: f64,
        pub patience: f64,
    }

    impl Default for Court {
        fn default() -> Court {
            Court { every: 120.0, quiet: 300.0, patience: 2.0 }
        }
    }

    thread_local! {
        /// The asks record ([`court`]) as this call's think has it, with the call's world.
        static ASKED: RefCell<Option<(View, Value)>> = const { RefCell::new(None) };
    }

    /// Back-off for asking one faction again and again (pact proposals, gifts): calls `ask(orders)` and returns true
    /// when `to` may be asked now, else false. An ask counts as answered once you share a pact with `to`. Each wait is
    /// stretched by a random 1 to 1.6, drawn at the ask, so asks don't fall on a clock and none comes sooner than
    /// `every` (the game takes a pact proposal to the same faction once every `RULES.pact.askGap` seconds). The record
    /// lives in capital RAM from page 232 (up to 8 pages: `{ factionId: [lastAsk, unanswered, stretch] }`). think only.
    pub fn court(orders: &mut Value, world: View, me: View, to: f64, ask: impl FnOnce(&mut Value), opts: &Court) -> bool {
        let now = n(world.get("seconds"));
        let mut asked = ASKED.with(|a| a.borrow().as_ref().filter(|(w, _)| w.same(&world)).map(|(_, v)| v.clone()));
        let asked = asked.get_or_insert_with(|| {
            let v = mem::get(me.get("ramCapital"), 232);
            if v.truthy() {
                v
            } else {
                Value::obj()
            }
        });
        let key = format!("{to}");
        let e = asked.get(&key);
        let (last, r) = (e.at(0).num_or(-1e9), e.at(2).num_or(1.0));
        let mut missed = e.at(1).num_or(0.0);
        if me.get("allies").has_num(to) || (missed >= opts.patience && now - last >= (opts.every + opts.quiet) * r) {
            missed = 0.0;
        }
        if now - last < if missed >= opts.patience { opts.every + opts.quiet } else { opts.every } * r {
            return false;
        }
        ask(orders);
        let mut keep = Value::obj();
        if let Value::Obj(o) = &*asked {
            for (k, v) in &o.0 {
                if now - v.at(0).num_or(-1e9) < (opts.every + opts.quiet) * 1.6 {
                    keep.set(k, v.clone());
                }
            }
        }
        keep.set(&key, arr![now, missed + 1.0, (100.0 + 60.0 * rand()).round() / 100.0]);
        ram(orders, mem::put(232, &keep, "local", None, 8).unwrap_or_default());
        ASKED.with(|a| *a.borrow_mut() = Some((world, keep)));
        true
    }

    // ---- puzzles ----

    /// The answer to any duel or research puzzle: Pollard's rho for factor, meeting in the middle for subset, BFS for
    /// path, backtracking (the node most tied to those already coloured next) for color, and the plain search for toll.
    pub fn solve(c: View) -> Value {
        let nums = |v: View| v.iter().map(|x| x.num()).collect::<Vec<f64>>();
        match c.get("kind").str() {
            "factor" => Value::Num(factor(c.get("n").num())),
            "subset" => subset(&nums(c.get("xs")), c.get("target").num()).into(),
            "path" => Value::Num(grid_path(&c.get("grid").iter().map(|r| r.str()).collect::<Vec<_>>())),
            "color" => color(c.get("n").num(), c.get("k").num(), &c.get("edges").iter().map(|e| [e.key(0.0).num(), e.key(1.0).num()]).collect::<Vec<_>>()).into(),
            "toll" => Value::Num(toll(c.get("seed").num(), c.get("bits").num())),
            _ => Value::Num(0.0),
        }
    }

    /// (a × b) mod n for a, b < n < 2^53: the quotient from doubles is off by a few at most, and the remainder is fixed
    /// up exactly, in a handful of instructions.
    fn mulmod_u(a: u64, b: u64, n: u64) -> u64 {
        let q = ((a as f64) * (b as f64) / (n as f64)) as u64;
        let n = n as i64;
        let mut r = a.wrapping_mul(b).wrapping_sub(q.wrapping_mul(n as u64)) as i64;
        while r < 0 {
            r += n;
        }
        while r >= n {
            r -= n;
        }
        r as u64
    }

    fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    }

    /// A factor of `n` (a whole number below 2^53): small primes, then Pollard's rho with gcds batched 32 steps at a
    /// time, about n^(1/4) steps where trial division takes n^(1/2). 0 when it finds none.
    pub fn factor(n: f64) -> f64 {
        if !(n.fract() == 0.0 && (2.0..9_007_199_254_740_992.0).contains(&n)) {
            return 0.0;
        }
        let n = n as u64;
        if n.is_multiple_of(2) {
            return 2.0;
        }
        let mut d = 3;
        while d < 64 && d * d <= n {
            if n.is_multiple_of(d) {
                return d as f64;
            }
            d += 2;
        }
        let step = |x: u64, c: u64| (mulmod_u(x, x, n) + c) % n;
        for c in 1..40 {
            let (mut x, mut y) = (2u64, 2u64);
            for _ in 0..1000 {
                let (sx, sy) = (x, y);
                let mut q = 1;
                for _ in 0..32 {
                    x = step(x, c);
                    y = step(step(y, c), c);
                    q = mulmod_u(q, x.abs_diff(y), n);
                }
                let d = gcd(q, n);
                if d == 1 {
                    continue;
                }
                if d != n {
                    return d as f64;
                }
                // the batch went past the factor: walk it again one step at a time
                (x, y) = (sx, sy);
                for _ in 0..32 {
                    x = step(x, c);
                    y = step(step(y, c), c);
                    let e = gcd(x.abs_diff(y), n);
                    if e != 1 {
                        if e != n {
                            return e as f64;
                        }
                        break;
                    }
                }
                break;
            }
        }
        0.0
    }

    /// Indices of `xs` summing to `target` (every sum of each half, matched through an open-addressed table), or an
    /// empty list.
    pub fn subset(xs: &[f64], target: f64) -> Vec<f64> {
        let h = xs.len() / 2;
        let sums = |part: &[f64]| {
            let mut out = vec![0.0];
            for &x in part {
                for j in 0..out.len() {
                    out.push(out[j] + x);
                }
            }
            out
        };
        let (a, b) = (sums(&xs[..h]), sums(&xs[h..]));
        // b's sums by value (as whole numbers), each with the last index that made it
        let size = (b.len() * 2).next_power_of_two();
        let mut slots: Vec<(i64, u32)> = vec![(0, u32::MAX); size];
        let at = |k: i64| ((k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40) as usize & (size - 1);
        for (j, &s) in b.iter().enumerate() {
            let mut i = at(s as i64);
            while slots[i].1 != u32::MAX && slots[i].0 != s as i64 {
                i = (i + 1) & (size - 1);
            }
            slots[i] = (s as i64, j as u32);
        }
        for (i, &s) in a.iter().enumerate() {
            let k = (target - s) as i64;
            let mut x = at(k);
            while slots[x].1 != u32::MAX && slots[x].0 != k {
                x = (x + 1) & (size - 1);
            }
            let j = slots[x].1;
            if j == u32::MAX || (i == 0 && j == 0) || b[j as usize] != target - s {
                continue;
            }
            let bits = |m: usize, base: usize| (0..usize::BITS as usize).filter(move |b| m >> b & 1 == 1).map(move |b| (base + b) as f64);
            return bits(i, 0).chain(bits(j as usize, h)).collect();
        }
        Vec::new()
    }

    /// The shortest way's length from the top-left to the bottom-right cell of `grid` through anything but '#' (BFS),
    /// or -1.
    pub fn grid_path(grid: &[&str]) -> f64 {
        let (h, w) = (grid.len(), grid.first().map_or(0, |r| r.len()));
        if w == 0 {
            return -1.0;
        }
        let open = |c: usize| grid[c / w].as_bytes().get(c % w) != Some(&b'#');
        if !open(0) {
            return -1.0;
        }
        let mut d = vec![-1i32; h * w];
        d[0] = 0;
        let mut q = vec![0usize];
        let mut i = 0;
        while i < q.len() {
            let c = q[i];
            i += 1;
            let (y, x) = (c / w, c % w);
            for (ok, nc) in [(y + 1 < h, c + w), (y > 0, c.wrapping_sub(w)), (x + 1 < w, c + 1), (x > 0, c.wrapping_sub(1))] {
                if ok && d[nc] < 0 && open(nc) {
                    d[nc] = d[c] + 1;
                    q.push(nc);
                }
            }
        }
        f64::from(d[h * w - 1])
    }

    /// A colour per node (`n` nodes, `k` colours) with no edge joining two the same, by backtracking, each next node
    /// the one most tied to those already coloured; an empty list when there's none.
    pub fn color(n: f64, k: f64, edges: &[[f64; 2]]) -> Vec<f64> {
        let count = if n > 0.0 { n.ceil() as usize } else { 0 };
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); count];
        let node = |x: f64| (x >= 0.0 && x.fract() == 0.0 && x < count as f64).then_some(x as usize);
        for e in edges {
            if let (Some(a), Some(b)) = (node(e[0]), node(e[1])) {
                adj[a].push(b);
                adj[b].push(a);
            }
        }
        let (mut order, mut used, mut ties) = (Vec::with_capacity(count), vec![false; count], vec![0u32; count]);
        for _ in 0..count {
            let Some(b) = (0..count).filter(|&v| !used[v]).max_by(|&a, &b| (ties[a], adj[a].len()).cmp(&(ties[b], adj[b].len())).then(b.cmp(&a))) else { break };
            used[b] = true;
            order.push(b);
            for &w in &adj[b] {
                ties[w] += 1;
            }
        }
        let (mut col, mut pos) = (vec![-1.0; count], vec![0.0; count]);
        let mut i: isize = 0;
        while i >= 0 && (i as usize) < count {
            let (iu, v) = (i as usize, order[i as usize]);
            let mut placed = false;
            while pos[iu] < k && !placed {
                let c = pos[iu];
                pos[iu] += 1.0;
                if adj[v].iter().all(|&w| col[w] != c) {
                    col[v] = c;
                    placed = true;
                }
            }
            if placed {
                i += 1;
                if (i as usize) < count {
                    pos[i as usize] = 0.0;
                }
            } else {
                col[v] = -1.0;
                i -= 1;
                if i >= 0 {
                    col[order[i as usize]] = -1.0;
                }
            }
        }
        if i as f64 == n {
            col
        } else {
            Vec::new()
        }
    }

    /// A nonce whose `hash(seed, nonce)` starts with `bits` zero bits: about 2^bits tries, whatever the code.
    pub fn toll(seed: f64, bits: f64) -> f64 {
        let lim = 2f64.powf(32.0 - bits);
        let mut x = 0.0;
        while f64::from(hash(seed, x)) >= lim {
            x += 1.0;
        }
        x
    }

    /// Three primes of each size from 4 to 26 bits (`PRIMES[bits - 4]`): a factor challenge built from them costs no
    /// search at throw time.
    pub const PRIMES: [[f64; 3]; 23] = [
        [13.0, 11.0, 13.0],
        [31.0, 29.0, 23.0],
        [61.0, 59.0, 53.0],
        [127.0, 113.0, 109.0],
        [251.0, 241.0, 239.0],
        [509.0, 503.0, 499.0],
        [1021.0, 1019.0, 1013.0],
        [2039.0, 2029.0, 2027.0],
        [4093.0, 4091.0, 4079.0],
        [8191.0, 8179.0, 8171.0],
        [16381.0, 16369.0, 16363.0],
        [32749.0, 32719.0, 32717.0],
        [65521.0, 65519.0, 65497.0],
        [131071.0, 131063.0, 131059.0],
        [262139.0, 262133.0, 262127.0],
        [524287.0, 524269.0, 524261.0],
        [1048573.0, 1048571.0, 1048559.0],
        [2097143.0, 2097133.0, 2097131.0],
        [4194301.0, 4194287.0, 4194277.0],
        [8388593.0, 8388587.0, 8388581.0],
        [16777213.0, 16777199.0, 16777183.0],
        [33554393.0, 33554383.0, 33554371.0],
        [67108859.0, 67108837.0, 67108819.0],
    ];

    /// Four-cell pieces of wall ('#') and floor ('.'), about a third wall: [`challenge`] strings them into a grid's noise.
    pub const NOISE: [&str; 8] = ["....", "#...", ".#..", "..#.", "...#", "#..#", ".##.", "#.#."];

    /// For `challenge(foe, max)`: a random category at the biggest size `max` allows (`{ factor: bits, subset: numbers,
    /// path: grid side }`: what your duel bank buys, and what's fair to the defender), as the secret the game builds the
    /// problem from: `{ kind: "factor", p, q }`, `{ kind: "subset", xs, pick }` or `{ kind: "path", grid }` (a staircase
    /// way through, walls from [`NOISE`] everywhere else, no bigger than the call's cycles allow). Null lets the game pick.
    pub fn challenge(_foe: View, max: View) -> Value {
        let kinds: Vec<&str> = ["factor", "subset", "path"].into_iter().filter(|k| n(max.get(k)) >= if *k == "factor" { 8.0 } else { 4.0 }).collect();
        if kinds.is_empty() {
            return Value::Null;
        }
        let some = |len: usize| rand_int(len as f64) as usize;
        match kinds[some(kinds.len())] {
            "factor" => {
                let b = n(max.get("factor")).min(52.0);
                let prime = |bits: f64| PRIMES.get((bits - 4.0) as usize).map_or(Value::Undef, |p| Value::Num(p[some(3)]));
                obj! { "kind" => "factor", "p" => prime((b / 2.0).floor()), "q" => prime(b - (b / 2.0).floor()) }
            }
            "subset" => {
                let (mut xs, mut pick) = (Vec::new(), Vec::new());
                for i in 0..n(max.get("subset")) as usize {
                    xs.push(Value::Num(1.0 + rand_int(1_000_000.0)));
                    if rand() < 0.5 {
                        pick.push(Value::Num(i as f64));
                    }
                }
                if pick.is_empty() {
                    pick.push(Value::Num(0.0));
                }
                obj! { "kind" => "subset", "xs" => xs, "pick" => pick }
            }
            _ => {
                let side = n(max.get("path")).min(((cycles_left() - 300.0) / 100.0).floor());
                if side < 4.0 {
                    return Value::Null;
                }
                let w = side as usize;
                let noise: String = (0..w.div_ceil(2)).map(|_| NOISE[some(NOISE.len())]).collect();
                let cut = |from: usize, len: usize| &noise[from.min(noise.len())..(from + len).min(noise.len())];
                let (mut grid, mut a) = (Vec::with_capacity(w), 0);
                for y in 0..w {
                    if cycles_left() < 120.0 {
                        return Value::Null;
                    }
                    let b = if y == w - 1 { w - 1 } else { (a + some(3)).min(w - 1) };
                    let row = format!("{}{}{}", cut(some(w), a), ".".repeat(b - a + 1), cut(some(w), w - 1 - b));
                    grid.push(Value::from(row));
                    a = b;
                }
                obj! { "kind" => "path", "grid" => grid }
            }
        }
    }
