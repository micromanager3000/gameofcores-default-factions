//! The founders' house styles, by seat: each founding seat (factions 0-9; Tabula Rasa is archived at 9) has a look with
//! its anthem, a 3D capital, unit names and a war line (docs/names.md tells their stories). The house programs and the
//! default program pick theirs by seat (`info.seat` in look and build3d, `me.seat` in think), never by name, so a
//! renamed founder keeps it; your code can borrow any of them:
//!
//! ```ignore
//! fn look(budget: View, info: View) -> Value {
//!     theme::of(info.get("seat"), 3.0).map_or(Value::Null, |t| t.look(budget.num(), info))
//! }
//! fn build3d(budget: View, info: View) -> Value {
//!     theme::of(info.get("seat"), 3.0).map_or(Value::Null, |t| t.scene(info, budget.num()))
//! }
//! ```
//!
//! Only the seat asked for is built, when it is asked for: [`of`] builds nothing, and each of a [`Theme`]'s parts is
//! built when you call for it.

use core::f64::consts::PI;

use gameofcores::build3d;
use super::capital::Civic;
use super::compat as js;
use gameofcores::value::{Obj, Value};
use gameofcores::view::View;
use gameofcores::{arr, obj};

/// A moment a war line is about ([`Theme::war`]): rivals marching on you.
#[derive(Clone, Debug, PartialEq)]
pub struct Moment {
    /// What kind of moment it is: "war" for the founders' lines.
    pub kind: &'static str,
    /// The faction(s) by name.
    pub who: String,
    /// The province they stand in.
    pub at: Option<String>,
    /// Whether it's your capital.
    pub capital: bool,
}

/// Unit names by unit type, as a style lists them: `("guard", &["Altan", "Bold", …])`.
pub type Names = &'static [(&'static str, &'static [&'static str])];

/// A founding seat's house style ([`of`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    /// The style it wears (its founding seat).
    style: usize,
    /// Whether it's that seat's own (not home's style on another seat).
    own: bool,
    /// Whether the seat came as a number. Seats are told apart as JavaScript's `===` tells them, so a seat given as a
    /// string ("2") wears its style without the touches only that seat's weapon has (its own voice, beam and trail).
    number: bool,
}

/// What one founder's style holds.
struct Style {
    name: &'static str,
    capital: &'static str,
    color: &'static str,
    /// Its own colour on its own seat, when that isn't `color`.
    primary: Option<&'static str>,
    names: Names,
    war: fn(&str, &Moment) -> String,
    music: fn() -> Value,
    look: fn(f64) -> Value,
    scene: fn(View, f64) -> Value,
}

/// Founding seat `seat`'s style (0-9), or seat `home`'s when `seat` is missing (undefined or null), or `None` for any
/// other seat. With a `home` (-1 for none), every other seat gets the home style too, in its own colour and with its
/// capital titled with its own name (and no motto), but without the founder's anthem, war line or capital music.
#[must_use]
pub fn of(seat: View, home: f64) -> Option<Theme> {
    let slot = |n: f64| (n >= 0.0 && n.fract() == 0.0 && n < STYLES.len() as f64).then_some(n as usize);
    let home = slot(home);
    let (s, number) = if seat.is_nullish() {
        (home, true)
    } else if seat.is_num() {
        (slot(seat.num()), true)
    } else {
        // an array index, as JavaScript reads one from anything else ("3", [3])
        (js::key_of(seat).parse::<f64>().ok().and_then(slot), false)
    };
    match (s, home) {
        (Some(s), _) => Some(Theme { style: s, own: true, number }),
        (None, Some(h)) => Some(Theme { style: h, own: false, number: true }),
        (None, None) => None,
    }
}

impl Theme {
    fn style(&self) -> &'static Style {
        &STYLES[self.style]
    }

    /// The founder's name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.style().name
    }

    /// Its capital's name.
    #[must_use]
    pub fn capital(&self) -> &'static str {
        self.style().capital
    }

    /// The unit names it uses, by unit type (for [`gameofcores::army::name`]). Every style lists `miner`, `guard`, `scout`,
    /// `longbow` and `spy`, the default program's types.
    #[must_use]
    pub fn names(&self) -> Names {
        self.style().names
    }

    /// The names it gives units of type `unit`, if it lists any.
    #[must_use]
    pub fn names_for(&self, unit: &str) -> Option<&'static [&'static str]> {
        self.style().names.iter().find(|(t, _)| *t == unit).map(|(_, list)| *list)
    }

    /// Display name for the fleet program's three ship functions, in this house's style.
    /// Function identifiers remain stable so deployed orders and recordings keep working.
    #[must_use]
    pub fn naval_name(&self, unit: &str) -> Option<&'static str> {
        let i = match unit {
            "warjunk" => 0,
            "picket" => 1,
            "corsair" => 2,
            _ => return None,
        };
        Some(NAVAL_NAMES[self.style][i])
    }

    /// A named scout, fighter or strike craft in this house's style, with a ten-point winged body.
    #[must_use]
    pub fn airframe(&self, role: super::air::Role) -> super::air::Airframe {
        super::air::of(self.style, role).expect("a Theme always has a founding style")
    }

    /// Its anthem (look().music); null on another seat.
    #[must_use]
    pub fn music(&self) -> Value {
        if self.own {
            (self.style().music)()
        } else {
            Value::Null
        }
    }

    /// Its line for a war moment (rivals marching on you, `who` by name), for what you say in public (`mem::say_once`);
    /// `None` on another seat.
    #[must_use]
    pub fn war(&self, who: &str, m: &Moment) -> Option<String> {
        self.own.then(|| (self.style().war)(who, m))
    }

    /// A whole look() result in `info.color` (the founder's own red on The Kernel Manifesto's own seat), with the
    /// anthem as `music`. Nothing is trimmed at any render budget (56-170). It dresses the whole army its seat's house
    /// program fields: every unit type gets its own body (`units`, `sprites`), a name (`names`) and a weapon
    /// (`combat.types`), which carries only the sounds its type makes, since each sound costs upkeep.
    #[must_use]
    pub fn look(&self, budget: f64, info: View) -> Value {
        let st = self.style();
        let mut l = (st.look)(budget);
        // army: { unitType: [body, arm, name] }: one of its sprites, one of its arms, and what the page calls the type.
        // Both are taken out of the look (the page reads units, names and combat.types), and each arm moves to the last
        // type that carries it, so only an arm two types share is copied.
        let (army, mut arms) = match &mut l {
            Value::Obj(o) => (o.remove("army"), o.remove("arms")),
            _ => (None, None),
        };
        let mut types = Obj::new();
        types.set("miner", none());
        if let (Some(Value::Obj(army)), Some(Value::Obj(arms))) = (army, &mut arms) {
            let arm_of = |a: &Value| a.at(1).str().to_string();
            for (i, (k, a)) in army.0.iter().enumerate() {
                l.entry("units").set(k, a.at(0).clone());
                if a.at(2).truthy() {
                    l.entry("names").set(k, a.at(2).clone());
                }
                let key = arm_of(a);
                let later = army.0[i + 1..].iter().any(|(_, b)| arm_of(b) == key);
                let art = arms.get_mut(&key).map(|x| if later { x.clone() } else { core::mem::take(x) });
                types.set(k, art.filter(Value::truthy).unwrap_or_else(none));
            }
        }
        // Cargo crews use their house's worker body and carry no decorative weapon.
        // Freighters use the ship preset at sea; this body also covers ground previews.
        let worker = l.get("units").get("miner").clone();
        for (unit, name) in [("hauler", "Hauler"), ("freighter", "Freighter")] {
            l.entry("units").set(unit, worker.clone());
            l.entry("names").set(unit, name);
            types.set(unit, none());
        }
        // Escorts sail as warships too; their ground preview wears the house's guard body.
        let guard = l.get("units").get("guard").clone();
        l.entry("units").set("escort", guard);
        l.entry("names").set("escort", "Escort");
        types.set("escort", none());
        l.set("combat", obj! { "default" => weapon_art(self.style, self.number), "types" => types });
        let primary = match st.primary {
            Some(p) if self.own => Value::from(p),
            _ if info.truthy() && info.get("color").truthy() => info.get("color").value(),
            _ => Value::from(st.color),
        };
        l.entry("palette").set("primary", primary);
        if self.own {
            l.set("music", (st.music)());
        }
        super::unit_art::dress_house(&mut l, self.style);
        l
    }

    /// A whole build3d() scene (most on [`super::capital::civic`]) that fits every build budget untrimmed, and plants more of
    /// its cheapest piece as the budget grows ([`super::capital::spare`]).
    #[must_use]
    pub fn scene(&self, info: View, budget: f64) -> Value {
        let mut sc = (self.style().scene)(info, budget);
        // A house needs one recognisable marker even when the visitor is still at the
        // plaza rim.  These sit at the far edge of each authored grounds, use one
        // cheap static mesh, and deliberately vary their material language instead
        // of adding another shared building.  They are landmarks, not game state.
        let mark = match self.style {
            0 => obj! { "type" => "tree", "kind" => "cypress", "x" => 50, "z" => -30, "h" => 20, "color" => "#274a2d", "pri" => 4 },
            1 => obj! { "type" => "statue", "pose" => "rider", "x" => -50, "z" => -30, "h" => 8, "material" => "metal", "color" => "#c89066", "trim" => "#e8c37a", "label" => "THE CHARIOTEER", "pri" => 4 },
            2 => obj! { "type" => "statue", "pose" => "robed", "x" => 50, "z" => -30, "h" => 8, "material" => "stone", "color" => "#efe4ce", "trim" => "#d0ab54", "label" => "THE VIGIL", "pri" => 4 },
            // Salomon's House already spends its base budget on its three furnaces,
            // perpetual-motion rings, automata and data lantern. Keep those visible
            // at the base budget instead of displacing one with a new rim prop.
            3 => Value::Undef,
            4 => obj! { "type" => "statue", "pose" => "orb", "x" => 50, "z" => -30, "h" => 5, "material" => "glass", "color" => "#d9f4ff", "trim" => "#d7a84c", "label" => "TIDE LENS", "pri" => 4 },
            5 => obj! { "type" => "statue", "pose" => "robed", "x" => 50, "z" => -30, "h" => 5, "material" => "stone", "color" => "#b8322f", "trim" => "#f2c230", "label" => "THE READER", "pri" => 4 },
            6 => obj! { "type" => "tree", "kind" => "crystal", "x" => -50, "z" => -30, "h" => 22, "color" => "#42d4ff", "pri" => 4 },
            7 => obj! { "type" => "statue", "pose" => "figure", "x" => 50, "z" => -30, "h" => 5, "material" => "stone", "color" => "#d7b76c", "trim" => "#1c3e6c", "label" => "THE EXCHANGE", "pri" => 4 },
            8 => obj! { "type" => "statue", "pose" => "mech", "x" => -50, "z" => -30, "h" => 5, "material" => "metal", "color" => "#7a8a6a", "trim" => "#d8b44c", "label" => "THE LOOKOUT", "pri" => 4 },
            _ => obj! { "type" => "statue", "pose" => "orb", "x" => 50, "z" => -30, "h" => 5, "material" => "glass", "color" => "#e9fff0", "trim" => "#d4b54a", "label" => "THE SENSE", "pri" => 4 },
        };
        if mark.truthy() { sc.entry("pieces").push(mark); }
        if !self.own {
            sc.set("title", info.get("name").value());
            sc.set("motto", "");
            sc.set("music", Value::Null);
        }
        sc
    }
}

// Match the retained ship presets in site/game/js/naval-sculpts.js. Page tests check every house's actual WASM look.
const NAVAL_NAMES: [[&str; 3]; 10] = [
    ["Khagan warship", "Strait outrider", "Orkhon raider"],
    ["Gandiva war galley", "River sentinel", "Garuda raider"],
    ["Crusader war cog", "Templar watchboat", "Turcopole galley"],
    ["Salomon ironclad", "Lamp surveyor", "Experiment cutter"],
    ["Baochuan warjunk", "Lantern junk", "Wavecutter"],
    ["Red Banner cruiser", "Convoy sentinel", "Shock flotilla"],
    ["Segfault destroyer", "Address beacon", "Dangling corsair"],
    ["Wealth of Nations", "Customs cutter", "Free-trade privateer"],
    ["Lewis gun launch", "Wheat patrol", "Campion skimmer"],
    ["Thorn ark", "Sensation lantern", "Petal clipper"],
];

// ---- the parts every style draws with ----
// Each style draws its own unit bodies (look().sprites): a few bold parts each, since a unit is about a dozen pixels
// tall on the map. Bodies face right with their feet at 0,0. Every part costs 12 cycles a second for each unit that
// wears it, so most take 4 to 7. A horse is 4 parts (body, head and neck, two stepping legs); a rider sits at y -6.

/// `[...a, ...b, …]`: lists of parts end to end.
fn cat<const N: usize>(lists: [Vec<Value>; N]) -> Value {
    Value::Arr(lists.into_iter().flatten().collect())
}

/// `{ ...a, ...b }`: `b`'s keys over `a`'s, in place, and the new ones after.
fn with(mut a: Value, b: Value) -> Value {
    if let Value::Obj(o) = b {
        for (k, v) in o.0 {
            a.set(&k, v);
        }
    }
    a
}

fn horse(coat: &str, legs: &str, dx: f64) -> Vec<Value> {
    vec![
        arr!["ellipse", dx - 0.5, -5.2, 4.4, 2.2, coat],
        arr!["poly", arr![dx + 2.4, -6.6, dx + 5.0, -10.6, dx + 7.6, -9, dx + 4.2, -4.8], coat],
        arr!["line", dx - 3.2, -4, dx - 3.6, 0, 1.3, legs, "walkL"],
        arr!["line", dx + 2.4, -4, dx + 2.8, 0, 1.3, legs, "walkR"],
    ]
}

/// Two stepping legs in colour `c`, `w` wide, `dx` along, `h` tall.
fn legs(c: &str, w: f64, dx: f64, h: f64) -> Vec<Value> {
    vec![arr!["line", dx - 0.8, -h, dx - 1.5, 0, w, c, "walkL"], arr!["line", dx + 0.8, -h, dx + 1.5, 0, w, c, "walkR"]]
}

/// Someone on foot in a coat (`w`: its hem's half-width).
fn man(legc: &str, coat: &str, skin: &str, w: f64) -> Vec<Value> {
    let mut p = legs(legc, 1.2, 0.0, 4.5);
    p.push(arr!["poly", arr![-w, -3.6, -1.8, -9.6, 1.8, -9.6, w, -3.6], coat]);
    p.push(arr!["circle", 0, -10.8, 1.5, skin]);
    p
}

/// Someone on a horse.
fn rider(coat: &str, legc: &str, body: &str, skin: &str) -> Vec<Value> {
    let mut p = horse(coat, legc, 0.0);
    p.push(arr!["rect", -1.9, -11.2, 3, 5.6, body]);
    p.push(arr!["circle", -0.4, -12.6, 1.5, skin]);
    p
}

// Bounded weapon and projectile geometry. Upkeep charges each worn primitive and effect. Timbre is authored by the
// faction, just like geometry. Each layer costs upkeep CPU.

/// Seat `seat`'s voice for `key` (shot, impact, shell, repair or dash), pitched up or down by `k`; `number`: whether
/// the seat came as a number (see [`Theme`]).
fn voice(seat: usize, number: bool, k: f64, key: &str) -> Value {
    const PITCH: [f64; 10] = [130.0, 680.0, 240.0, 1300.0, 340.0, 1600.0, 950.0, 180.0, 105.0, 420.0];
    const ENDS: [f64; 10] = [48.0, 160.0, 90.0, 240.0, 55.0, 1100.0, 70.0, 60.0, 36.0, 180.0];
    const WAVES: [&str; 10] = ["sawtooth", "sine", "triangle", "square", "sine", "triangle", "sawtooth", "triangle", "square", "sine"];
    let layer = |wave: &str, ratio: f64, level: f64| obj! { "wave" => wave, "ratio" => ratio, "level" => level };
    let (pitch, s, templar) = (PITCH[seat] * k, seat as f64, number && seat == 2);
    match key {
        "shot" => obj! {
            "pitch" => pitch,
            "endPitch" => ENDS[seat] * k,
            "duration" => if templar { 0.24 } else { 0.16 },
            "layers" => arr![layer(WAVES[seat], 1.0, 0.7), layer("sine", if templar { 2.73 } else { 0.5 }, 0.3)],
            "noise" => if templar { 0.65 } else { 0.25 },
            "cutoff" => if templar { 3400 } else { 7200 },
            "level" => 0.85,
        },
        "impact" => obj! {
            "pitch" => pitch * 0.5, "endPitch" => 38, "duration" => 0.22, "layers" => arr![layer("triangle", 1.0, 0.45)], "noise" => 0.8,
            "cutoff" => 3600, "level" => 0.6,
        },
        "shell" => obj! {
            "pitch" => 90.0 + s * 7.0, "endPitch" => 32, "duration" => 0.95, "layers" => arr![layer("sine", 1.0, 0.85), layer("triangle", 2.73, 0.15)],
            "noise" => 0.9, "cutoff" => 2200, "level" => 1,
        },
        "repair" => obj! {
            "pitch" => 420.0 + s * 24.0, "endPitch" => 850.0 + s * 24.0, "duration" => 0.36,
            "layers" => arr![layer("sine", 1.0, 0.6), layer("sine", 1.5, 0.4)], "noise" => 0.05, "cutoff" => 5000, "level" => 0.35,
        },
        "dash" => obj! {
            "pitch" => 110.0 + s * 12.0, "endPitch" => 650, "duration" => 0.28, "layers" => arr![layer("sawtooth", 1.0, 0.3)], "noise" => 0.65,
            "cutoff" => 4200, "level" => 0.5,
        },
        _ => Value::Undef,
    }
}

/// Seat `seat`'s voice, `keys` of it (space-separated), pitched by `k`.
fn voices(seat: usize, number: bool, k: f64, keys: &str) -> Value {
    let mut sound = Obj::new();
    for key in keys.split(' ') {
        sound.set(key, voice(seat, number, k, key));
    }
    Value::Obj(sound)
}

/// What a unit type fights with when its style gives it nothing of its own: the seat's weapon (combat.default);
/// `number`: whether the seat came as a number (see [`Theme`]).
fn weapon_art(seat: usize, number: bool) -> Value {
    const COLORS: [&str; 10] = ["#ff7849", "#58beff", "#e2c477", "#60ecff", "#bf8cff", "#ff668b", "#52f1d4", "#f3d960", "#ffad55", "#b6ed75"];
    let c = COLORS[seat];
    let head = match seat {
        0 => arr!["poly", arr![5, 0, -3, -2, -1, 0, -3, 2], c],
        1 => arr!["circle", 0, 0, 2, c],
        2 => arr!["poly", arr![5, 0, 0, -2, -4, 0, 0, 2], c],
        3 => arr!["ellipse", 0, 0, 3, 1.8, c],
        4 => arr!["circle", 0, 0, 3, c],
        5 => arr!["rect", -3, -1, 7, 2, c],
        6 => arr!["poly", arr![4, 0, -2, -2, -2, 2], c],
        7 => arr!["poly", arr![3, 0, 0, -3, -3, 0, 0, 3], c],
        8 => arr!["ellipse", 0, 0, 2, 3, c],
        _ => arr!["poly", arr![4, 0, -3, -2, -1, 2], c],
    };
    let weapon = match seat {
        0 => arr![arr!["poly", arr![3, -6, 6, 0, 3, 4, 4, 0], "accent"], arr!["line", 3, -6, 3, 4, 0.7, c]],
        1 => arr![arr!["circle", 3, 0, 3.5, c], arr!["circle", 3, 0, 2.2, "accent"], arr!["line", 3, 0, 10, 0, 1, c]],
        2 => arr![arr!["line", -2, 0, 10, 0, 1.5, "#d0ab54"], arr!["line", 5, -4, 5, 4, 1.6, "#b52b36"]],
        3 => arr![arr!["rect", -2, -2, 9, 4, "accent"], arr!["line", 1, -3, 8, -3, 1.4, c], arr!["circle", 6, 0, 1.2, c]],
        4 => arr![arr!["circle", 3, 0, 4, c], arr!["circle", 3, 0, 2.5, "accent"], arr!["circle", 3, 0, 1, "#ffffff"]],
        5 => arr![arr!["rect", -2, -2, 11, 4, "accent"], arr!["rect", 1, -1, 9, 1, c]],
        6 => arr![arr!["line", -1, -2, 9, -2, 1.5, c], arr!["line", -1, 2, 9, 2, 1.5, c], arr!["line", -1, -2, -1, 2, 1.5, "accent"]],
        7 => arr![arr!["poly", arr![-2, 0, 0, -3, 4, -3, 6, 0, 4, 3, 0, 3], c], arr!["line", 3, 0, 10, 0, 1, "accent"]],
        8 => arr![arr!["poly", arr![0, 0, 4, -4, 8, -2, 5, 0], c], arr!["line", 0, 0, 10, 0, 1, "accent"]],
        _ => arr![arr!["poly", arr![-2, 0, 2, -3, 3, 0, 7, -2, 9, 0, 4, 2], c], arr!["circle", 2, 0, 1, "#d94a6a"]],
    };
    let is = |k: usize| number && seat == k;
    let crossed = is(2);
    obj! {
        "sound" => voices(seat, number, 1.0, "shot impact shell repair dash"),
        "weapon" => weapon,
        "projectile" => arr![head, arr!["circle", 0, 0, 0.8, "#ffffff"]],
        "mount" => arr![4, -12],
        "muzzle" => arr![10, 0],
        "color" => c,
        "core" => "#ffffff",
        "glow" => 5,
        "beam" => is(3) || is(5) || is(6),
        "trail" => obj! { "color" => c, "length" => if crossed { 0.55 } else { 0.4 }, "width" => 1.2, "particles" => 2, "wave" => if is(4) { 3 } else { 0 } },
        "impact" => obj! { "color" => c, "rings" => if is(4) { 2 } else { 1 }, "sparks" => 4 },
    }
}

// Armies: what a founder fields in the CPU RTS. A style's look() gives army: { unitType: [body, arm, name] } (one of its
// sprites, one of its arms, and what the page calls the type) with arms: { name: weapon }, and [`Theme::look`] fills
// units, names and combat.types from them. A gun is held, so it turns to aim; a bow or lance drawn in the body stays put
// and only what it shoots flies; a blow at arm's length flies as a slash.

/// A weapon: `held` parts at `mount`, pointing right (the page turns them to aim and flips them with the unit), `shot`
/// parts flying from the muzzle, and the seat's voice for `keys` (of shot impact shell repair dash: only the ones the
/// type uses, since every one costs upkeep), pitched by `k`; `o` sets the rest (muzzle, color, trail, impact, beam,
/// glow).
fn arm(seat: usize, keys: &str, k: f64, held: Value, mount: Value, shot: Value, o: Value) -> Value {
    let muzzle = arr![if held.is_empty() { 1 } else { 8 }, 0];
    let a = obj! {
        "sound" => voices(seat, true, k, keys),
        "weapon" => held,
        "mount" => mount,
        "muzzle" => muzzle,
        "projectile" => shot,
        "glow" => 3,
        "trail" => obj! { "length" => 0.3, "width" => 1 },
    };
    with(a, o)
}

/// A blow at arm's length: it flies as a slash.
fn blow(seat: usize, k: f64, mount: Value, held: Value, o: Value) -> Value {
    let slash = obj! {
        "glow" => 5,
        "trail" => obj! { "color" => "#ffffff", "length" => 0.2, "width" => 1.4 },
        "impact" => obj! { "rings" => 0, "sparks" => 5 },
    };
    arm(seat, "shot impact dash", k, held, mount, arr![arr!["poly", arr![0, -5, 3.4, -1.6, 3.8, 1.6, 0, 5, 1.6, 0], "#ffffff"]], with(slash, o))
}

/// A type with no weapon of its own.
fn none() -> Value {
    obj! {
        "weapon" => arr![],
        "projectile" => arr![],
        "glow" => 0,
        "trail" => obj! { "particles" => 0 },
        "impact" => obj! { "rings" => 0, "sparks" => 0 },
    }
}

fn arrow() -> Value {
    arr![arr!["line", -5, 0, 3, 0, 0.6, "#e8dcc0"], arr!["poly", arr![5, 0, 2.6, -1.2, 2.6, 1.2], "#d8dce4"], arr!["line", -5, 0, -3.4, 0, 1.4, "primary"]]
}

fn bolt() -> Value {
    arr![arr!["line", -3, 0, 2, 0, 1.1, "#5a3a24"], arr!["poly", arr![4.4, 0, 1.8, -1.4, 1.8, 1.4], "#d8dce4"]]
}

fn ball() -> Value {
    arr![arr!["circle", 0, 0, 1.8, "#2a2a30"], arr!["circle", -0.5, -0.5, 0.7, "#9a9aa4"]]
}

fn tracer() -> Value {
    arr![arr!["rect", -3, -0.4, 5, 0.8, "#ffd27a"]]
}

fn rocket() -> Value {
    arr![arr!["rect", -4, -0.7, 6, 1.4, "#6a6a72"], arr!["poly", arr![4, 0, 2, -1.2, 2, 1.2], "primary"], arr!["poly", arr![-4, 0, -6, -1.6, -6, 1.6], "#ffb347"]]
}

/// A shot's smoke.
fn smoke() -> Value {
    obj! {
        "trail" => obj! { "color" => "#e8e4dc", "length" => 0.3, "width" => 2, "particles" => 2 },
        "impact" => obj! { "rings" => 1, "sparks" => 4 },
    }
}

/// A shell's fire.
fn fire() -> Value {
    obj! {
        "color" => "#ff8a3a",
        "glow" => 6,
        "trail" => obj! { "color" => "#ff8a3a", "length" => 0.5, "width" => 2, "particles" => 3 },
        "impact" => obj! { "color" => "#ffb347", "rings" => 2, "sparks" => 8 },
    }
}

/// A rest in a [`motif`].
const REST: f64 = f64::NAN;

/// An anthem's motif: (note, beats), a [`REST`] for silence.
fn motif(notes: &[(f64, f64)]) -> Value {
    Value::Arr(notes.iter().map(|&(n, b)| arr![if n.is_nan() { Value::Null } else { Value::Num(n) }, b]).collect())
}

/// A colour read from `info` for [`Civic`]'s colours.
fn hue(v: View) -> Option<&'static str> {
    v.is_str().then(|| v.str())
}
// ---- 0 · conqueror · Kublai Khan's Alt: Coleridge's Xanadu as a site plan. The main fleet sank in a typhoon; the alt
// rides on.

const RIDERS0: &[&str] = &["Altan", "Bold", "Chinua", "Erdene", "Jargal", "Khulan", "Monkh", "Naran", "Saran", "Sukh", "Tomor", "Toli", "Ulaan", "Zorig"];
const ARTISANS0: &[&str] = &[
    "Felt Maker", "Tea Porter", "Salt Clerk", "Ink Grinder", "Ox Driver", "Silk Winder", "Kumis Brewer", "Horse Groom", "Paper Cutter", "Bow Fletcher",
];
// the Khan's envoys: the Polos (Niccolo and Maffeo to the Pope in 1266, then Marco about the empire), Bolad and Isa
// Kelemechi to Persia, Yang Tingbi to Kollam, Meng Qi to Java, Zhao Liangbi and Du Shizhong to Japan
const ENVOYS0: &[&str] = &["Marco Polo", "Niccolo Polo", "Maffeo Polo", "Bolad", "Isa Kelemechi", "Yang Tingbi", "Meng Qi", "Zhao Liangbi", "Du Shizhong"];
// horse archers after the Khan's generals and his grandfather's companions, Jebe ("the arrow") first; a spy in a
// caravan, after the towns on its road (Otrar's governor killed a Mongol caravan as spies in 1218)
const ARCHERS0: &[&str] = &["Jebe", "Subutai", "Muqali", "Jelme", "Bo'orchu", "Chilaun", "Bayan", "Aju", "Uriyangkhadai"];
const CARAVANS0: &[&str] = &["Otrar", "Samarkand", "Bukhara", "Kashgar", "Turpan", "Merv", "Herat", "Nishapur"];
// the house bot's types, then the default program's: the Khan's guard (the keshig) rides, his artisans mine, an envoy
// scouts
const NAMES0: Names = &[
    ("raider", RIDERS0),
    ("thrall", ARTISANS0),
    ("miner", ARTISANS0),
    ("guard", RIDERS0),
    ("scout", ENVOYS0),
    ("longbow", ARCHERS0),
    ("spy", CARAVANS0),
];

fn war0(who: &str, _: &Moment) -> String {
    format!("{who} is no typhoon. The alt rides out.")
}

/// Seat 0's anthem.
fn music0() -> Value {
    obj! {
        "title" => "Ancestral Voices",
        "key" => "D",
        "mode" => "minorpent",
        "tempo" => 132,
        "lead" => "strings",
        "bass" => "drone",
        "drums" => "gallop",
        "motif" => motif(&[
            (0.0, 0.5), (0.0, 0.25), (0.0, 0.25), (2.0, 0.5), (3.0, 0.5), (5.0, 1.0), (3.0, 1.0), (3.0, 0.5), (3.0, 0.25),
            (3.0, 0.25), (2.0, 0.5), (1.0, 0.5), (2.0, 2.0), (0.0, 0.5), (0.0, 0.25), (0.0, 0.25), (3.0, 0.5), (5.0, 0.5),
            (7.0, 2.0), (6.0, 0.5), (5.0, 0.5), (3.0, 0.5), (2.0, 0.5), (0.0, 2.0),
        ]),
    }
}

/// Seat 0's look, before [`Theme::look`] fills it in: horse archers in fur hats with their bows and quivers, the
/// keshig (the Khan's guard) in lamellar under a pennanted lance, light riders with sabres, artisans in felt hats,
/// falconers, shamans with their drums, the Persian counterweight trebuchet that took Xiangyang (1273), and envoys and
/// caravans on camels.
fn look0(b: f64) -> Value {
    const SKIN: &str = "#d9a877";
    obj! {
        "architecture" => "tents",
        "unit" => "rider",
        "worker" => "peasant",
        "ship" => "ger",
        "units" => obj! {
            "raider" => "archer",
            "thrall" => "artisan",
            "miner" => "artisan",
            "guard" => "keshig",
            "scout" => "camel",
            "longbow" => "archer",
            "spy" => "camel",
        },
        "sprites" => obj! {
            "archer" => obj! {
                "parts" => cat([
                    rider("#8a5a32", "#4a2e1a", "primary", SKIN),
                    vec![
                        arr!["rect", -3.8, -11.4, 1.3, 4.6, "#6a3a1a"], arr!["poly", arr![-2, -13.2, -0.4, -16, 1.2, -13.2], "accent"],
                        arr!["poly", arr![2.2, -16, 4.2, -11.6, 2.2, -7, 3.2, -11.6], "accent"],
                    ],
                ]),
            },
            "keshig" => obj! {
                "parts" => cat([
                    rider("#5a3a24", "#3a2416", "#a8b0b8", SKIN),
                    vec![
                        arr!["rect", -4.8, -7.4, 8, 2.6, "primary"], arr!["rect", -1.9, -9.2, 3, 1, "primary"],
                        arr!["poly", arr![-1.9, -13.2, -0.4, -16.8, 1.1, -13.2], "#a8b0b8"], arr!["line", -3, -5, 5.5, -18.5, 0.8, "#e8dcc0"],
                        arr!["poly", arr![5.1, -18, 8.8, -17, 5.7, -15.8], "primary"], arr!["circle", -2.6, -8.6, 1.8, "accent"],
                    ],
                ]),
            },
            "horseman" => obj! {
                "parts" => cat([
                    horse("#b07a46", "#5a3a24", 0.0),
                    vec![
                        arr!["poly", arr![-2.2, -5.8, 1, -5.8, 2.2, -11, -0.6, -11.4], "secondary"],
                        arr!["line", -1.6, -8.6, 1.6, -8.6, 1, "primary"], arr!["circle", 0.6, -12.6, 1.5, SKIN],
                        arr!["poly", arr![-1, -13.2, 0.6, -15.8, 2.2, -13.2], "primary"],
                        arr!["poly", arr![-0.6, -12.8, -5.6, -14.6, -5, -12.2], "primary"],
                    ],
                ]),
            },
            "artisan" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["poly", arr![-2.3, 0, -1.3, -6, 1.3, -6, 2.3, 0], "secondary", "bob"], arr!["rect", -1.6, -4, 3.2, 1, "primary", "bob"],
                    arr!["circle", 0, -7, 1.3, "#d9a877", "bob"], arr!["poly", arr![-1.8, -7.6, 0, -10.4, 1.8, -7.6], "primary", "bob"],
                ],
            },
            "falconer" => obj! {
                "parts" => cat([
                    man("#4a2e1a", "secondary", SKIN, 2.4),
                    vec![
                        arr!["rect", -2, -6.8, 4, 1, "primary"], arr!["poly", arr![-1.8, -11.4, 0, -14, 1.8, -11.4], "primary"],
                        arr!["line", 1.2, -8.6, 3.4, -12.2, 1.1, "#8a5a32"],
                        arr!["poly", arr![2.6, -11.2, 3, -14.8, 4.6, -15.4, 4.6, -12.6], "#7a6450"], arr!["ellipse", 4.2, -13.6, 0.5, 1, "#efe6d2"],
                        arr!["circle", 4.4, -15.8, 0.9, "#5a4a3a"],
                    ],
                ]),
            },
            "shaman" => obj! {
                "parts" => cat([
                    man("#4a2e1a", "#2e3a5a", SKIN, 2.8),
                    vec![
                        arr!["line", -2.6, -3.8, 2.6, -3.8, 0.9, "primary"], arr!["line", -1, -12.4, -2.6, -16.2, 0.6, "secondary"],
                        arr!["line", 1, -12.4, 2.6, -16.2, 0.6, "secondary"], arr!["rect", -1.8, -12.8, 3.6, 1.2, "primary"],
                        arr!["ellipse", 3.4, -7.8, 1.3, 2.3, "secondary"], arr!["line", 1.2, -7.6, 2.4, -6.4, 0.6, "accent"],
                    ],
                ]),
            },
            "trebuchet" => obj! {
                "parts" => arr![
                    arr!["rect", -7.5, -2.8, 15, 1.8, "#6a4a2a"], arr!["circle", -5, -1, 1.6, "#3a2416"], arr!["circle", 5, -1, 1.6, "#3a2416"],
                    arr!["line", -4.4, -2.4, 0, -11.5, 1.2, "#7a5a32"], arr!["line", 4.4, -2.4, 0, -11.5, 1.2, "#7a5a32"],
                    arr!["line", 0, -11.5, 0, -16.5, 0.5, "accent"], arr!["poly", arr![0, -16.5, 4, -15.5, 0, -14.4], "primary"],
                    arr!["rect", -10, -7.6, 2.6, 4.8, "secondary", "bob"], arr!["circle", -8.7, -8.8, 1.2, SKIN, "bob"],
                ],
            },
            "camel" => obj! {
                "parts" => arr![
                    arr!["ellipse", -0.6, -6.6, 4.4, 2.2, "#c9a46a"], arr!["circle", -1, -8.4, 2.2, "#c9a46a"],
                    arr!["poly", arr![2.6, -7.6, 5.2, -12.8, 7.6, -12, 4, -5.6], "#c9a46a"], arr!["line", -3.4, -5, -3.6, 0, 1.2, "#8a6a42", "walkL"],
                    arr!["line", 2.4, -5, 2.8, 0, 1.2, "#8a6a42", "walkR"], arr!["rect", -2.6, -13.6, 2.8, 4, "primary"],
                    arr!["circle", -1.2, -14.8, 1.3, "#d9a877"],
                ],
            },
        },
        "names" => obj! { "miner" => "Artisan", "guard" => "Keshig guard", "scout" => "Envoy", "longbow" => "Horse archer", "spy" => "Caravan spy" },
        "army" => obj! {
            "miner" => arr!["artisan", "", "Artisan"],
            "screen" => arr!["keshig", "lance", "Keshig guard"],
            "lancer" => arr!["keshig", "javelin", "Keshig lancer"],
            "runner" => arr!["horseman", "sabre", "Tumen rider"],
            "skirm" => arr!["archer", "bow", "Horse archer"],
            "sentry" => arr!["falconer", "falcon", "Falconer"],
            "gun" => arr!["trebuchet", "trebuchet", "Trebuchet"],
            "repairer" => arr!["shaman", "drum", "Shaman"],
            "ghost" => arr!["camel", "dagger", "Caravan spy"],
            // the Mongols hunted with golden eagles
            "wing" => arr!["falconer", "falcon", "Eagle hunter"],
        },
        "arms" => obj! {
            "bow" => arm(0, "shot impact", 1.4, arr![], arr![3.2, -11.6], arrow(), obj! {}),
            "lance" => blow(0, 0.7, arr![5.5, -18.5], arr![], obj! {}),
            "javelin" => arm(
                0, "shot impact dash", 1.1, arr![], arr![5.5, -18.5],
                arr![arr!["line", -6, 0, 3, 0, 0.7, "#e8dcc0"], arr!["poly", arr![5, 0, 3, -1, 3, 1], "#d8dce4"]],
                obj! { "trail" => obj! { "length" => 0.4, "width" => 0.8 } },
            ),
            "sabre" => blow(
                0, 1.2, arr![2, -10.6],
                arr![arr!["poly", arr![0, -0.5, 6, -1.2, 8, -2.6, 6.6, -0.2, 0, 0.7], "#e4e8ee"], arr!["line", -1.4, 0, 0.4, 0, 1.3, "#3a2416"]],
                obj! { "muzzle" => arr![8, -2] },
            ),
            "falcon" => arm(
                0, "shot impact", 2.2, arr![], arr![3.6, -14],
                arr![arr!["poly", arr![3, 0, -1, -3.4, -0.4, 0, -1, 3.4], "#7a6450"], arr!["circle", 2.2, 0, 0.9, "#7a6450"]],
                obj! { "glow" => 0, "trail" => obj! { "color" => "#efe6d2", "length" => 0.25, "width" => 0.6 } },
            ),
            "trebuchet" => arm(
                0, "shell shot impact", 0.6,
                arr![
                    arr!["line", -6, 3, 9, -3, 1.1, "#6a4a2a"], arr!["rect", -8, 2, 3.6, 3.6, "primary"], arr!["line", 9, -3, 9.6, 1, 0.5, "#e8dcc0"],
                ],
                arr![0, -11.5], arr![arr!["circle", 0, 0, 2, "#8a7a6a"], arr!["circle", 0.7, -0.5, 1, "#ffb347"]],
                with(obj! { "muzzle" => arr![9.6, 1] }, fire()),
            ),
            "drum" => arm(
                0, "repair shot impact dash", 0.8, arr![], arr![3.4, -7.8], arr![arr!["circle", 0, 0, 1.4, "#bfe8ff"]],
                obj! { "color" => "#bfe8ff", "glow" => 6, "trail" => obj! { "color" => "#bfe8ff", "length" => 0.5, "width" => 0.8, "wave" => 2 } },
            ),
            "dagger" => blow(0, 1.6, arr![-0.4, -12], arr![], obj! {}),
        },
        "palette" => obj! { "secondary" => "#efe6d2", "accent" => "#3a1c1c" },
        "banner" => obj! {
            "pattern" => "chevron",
            "emblem" => arr![arr!["circle", 0, -1.5, 3.2, "#ffd27a"], arr!["text", "alt", 0, 5.6, "#ffffff"]],
        },
        "ornaments" => if b >= 62.0 { arr!["walls", "smoke"] } else { arr!["walls"] },
        "wonder" => obj! {
            "shape" => "dome",
            "tiers" => 3,
            "height" => if b >= 62.0 { 40 } else { 30 },
            "glow" => true,
            "spin" => false,
            "color" => "#e9b949",
        },
    }
}

/// The forest outside a walled capital's walls (Xanadu's, and Address Zero's), the first 8 trees planted.
const FOREST: [[f64; 2]; 16] = [
    [-53.0, -40.0], [53.0, -40.0], [-53.0, -20.0], [53.0, -20.0], [-53.0, 0.0], [53.0, 0.0], [-53.0, 20.0], [53.0, 20.0], [-53.0, 34.0], [53.0, 34.0],
    [-30.0, -55.0], [30.0, -55.0], [-12.0, -55.0], [12.0, -55.0], [-53.0, 48.0], [53.0, 48.0],
];

/// Xanadu: the Stately Pleasure-Dome.
fn scene0(info: View, budget: f64) -> Value {
    const W: f64 = 46.0;
    let (c, wall, felt) = (info.get("color"), "#c9b48c", "#efe6d2");
    let mut p = super::capital::civic(
        info,
        &Civic {
            wonder: obj! { "shape" => "dome", "tiers" => 3, "height" => 40, "glow" => true },
            wonder_r: 9.0,
            wonder_color: Some("#e9b949"),
            wonder_mat: Some("metal"),
            trim: hue(c),
            roof: Some("dome"),
            hall_color: Some(felt),
            hall_label: Some("THE KHAN'S GER"),
            screen_title: Some("YAM RELAY"),
            banners: 4.0,
            ..Civic::default()
        },
    );
    // "with walls and towers were girdled round"
    for s in [[-W, -W, W, -W], [-W, -W, -W, W], [W, -W, W, W], [-W, W, -10.0, W], [10.0, W, W, W]] {
        p.push(obj! { "type" => "wall", "x" => s[0], "z" => s[1], "x2" => s[2], "z2" => s[3], "h" => 6, "t" => 1.6, "crenel" => true, "color" => wall });
    }
    p.push(obj! { "type" => "arch", "style" => "round", "x" => 0, "z" => W, "w" => 20, "h" => 14, "t" => 3, "rot" => 0, "color" => wall, "trim" => c });
    for q in [[-W, -W], [W, -W], [-W, W], [W, W]] {
        p.push(obj! { "type" => "tower", "x" => q[0], "z" => q[1], "r" => 5, "h" => 18, "sides" => 10, "roof" => "cone", "color" => wall, "trim" => c });
    }
    // the khan's camp: six gers along the east wall, and the gilded Cane Palace (Marco Polo's bamboo pavilion)
    for q in [[20.0, -38.0], [32.0, -36.0], [40.0, -24.0], [40.0, -8.0], [40.0, 6.0], [40.0, 20.0]] {
        p.push(obj! {
            "type" => "tower", "x" => q[0], "z" => q[1], "r" => 4, "h" => 3, "sides" => 14, "floors" => 1, "roof" => "dome", "color" => felt, "trim" => c,
        });
    }
    p.push(obj! {
        "type" => "tower", "id" => "cane", "x" => 0, "z" => -32, "r" => 6, "h" => 7, "sides" => 8, "floors" => 2, "roof" => "pagoda", "material" => "wood",
        "color" => "#d9a93c", "trim" => c,
    });
    // "where Alph, the sacred river, ran", the caves of ice, the mighty fountain, gardens bright with sinuous rills
    p.push(obj! { "type" => "water", "shape" => "rect", "x" => -40, "z" => 2, "w" => 5, "d" => 60, "color" => "#3d6f8f" });
    for q in [[-36.0, -36.0, 7.0], [-32.0, -38.0, 5.0], [-40.0, -37.0, 6.0], [-35.0, -41.0, 8.0], [-39.0, -41.0, 5.0]] {
        p.push(obj! { "type" => "tree", "kind" => "crystal", "x" => q[0], "z" => q[1], "h" => q[2], "color" => "#d6f3ff" });
    }
    p.push(obj! { "type" => "particles", "kind" => "snow", "x" => -36, "z" => -39, "y" => 2, "n" => 60, "r" => 6, "color" => "#eaf8ff" });
    p.push(obj! { "type" => "water", "shape" => "round", "x" => -28, "z" => 30, "w" => 12, "d" => 12, "color" => "#3d6f8f" });
    p.push(obj! { "type" => "particles", "kind" => "fountain", "x" => -28, "z" => 30, "y" => 0.4, "n" => 120, "r" => 1.5, "color" => "#d6ecff" });
    for q in [[30.0, 30.0], [22.0, 4.0]] {
        p.push(obj! { "type" => "garden", "x" => q[0], "z" => q[1], "r" => 7, "flowers" => 20 });
    }
    // the punchline, on two plinths by the gate
    p.push(obj! { "type" => "statue", "pose" => "rider", "x" => -8, "z" => 36, "h" => 8, "material" => "metal", "color" => "#a8844f", "label" => "MAIN: LOST AT SEA" });
    p.push(obj! { "type" => "statue", "pose" => "rider", "x" => 8, "z" => 36, "h" => 8, "material" => "metal", "color" => "#a8844f", "label" => "ALT: STILL RIDING" });
    for x in [-14.0, 14.0] {
        p.push(obj! { "type" => "light", "x" => x, "z" => 42, "y" => 3, "post" => true, "power" => 1.4, "color" => "#ffae55" });
        p.push(obj! { "type" => "particles", "kind" => "embers", "x" => x, "z" => 42, "y" => 3.4, "n" => 36, "r" => 0.6, "color" => "#ff8a3a", "pri" => 2 });
    }
    p.push(obj! { "type" => "particles", "kind" => "smoke", "x" => 40, "z" => -8, "y" => 3, "n" => 30, "r" => 2, "color" => "#9a948c", "pri" => 1 });
    // "forests ancient as the hills", outside the walls (trimmed first; a bigger budget plants more)
    for q in FOREST.iter().take((8.0 + super::capital::spare(budget, 3.0, 8.0)) as usize) {
        p.push(obj! { "type" => "tree", "kind" => "pine", "x" => q[0], "z" => q[1], "h" => 9, "color" => "#355a2c", "pri" => 1 });
    }
    obj! {
        "title" => "The Stately Pleasure-Dome",
        "motto" => "Main sank in a typhoon. The alt rides on.",
        "palette" => obj! { "primary" => c, "secondary" => felt, "accent" => "#3a1c1c" },
        "sky" => obj! { "time" => "dusk", "top" => "#2a2350", "horizon" => "#ffb070", "fog" => 0.25 },
        "ground" => obj! { "kind" => "grass", "color" => "#8f9658" },
        "pieces" => p,
    }
}

// ---- 1 · expander · Arjuna's Halting Problem: the Bhagavad Gita, sung on the field of Kurukshetra as the battle opens.
// Arjuna sets down his bow and halts; eighteen chapters later his charioteer has him running. Capital Kurukshetra. Its
// units: the great chariot-warriors Duryodhana counts in the Pandava ranks (Gita 1.4-6, "every one a maharatha") fight
// and claim; the Pandavas, in the disguises of their year as servants at Virata's court (Book 4), mine; and a charioteer
// scouts, Sanjaya first, who saw the whole battle for the blind king.

const MAHARATHAS1: &[&str] = &[
    "Yuyudhana", "Virata", "Drupada", "Dhrishtaketu", "Chekitana", "Kashiraja", "Purujit", "Kuntibhoja", "Shaibya", "Yudhamanyu", "Uttamaujas", "Abhimanyu",
];
const NAMES1: Names = &[
    ("cell", MAHARATHAS1),
    ("guard", MAHARATHAS1),
    ("miner", &["Kanka", "Ballava", "Brihannala", "Granthika", "Tantipala"]),
    ("scout", &["Sanjaya", "Daruka", "Matali", "Uttara", "Vishoka", "Indrasena", "Adhiratha", "Shalya"]),
    // archers of the Pandava host: Shikhandi, Dhrishtadyumna and Draupadi's five sons; and the spy is Sairandhri,
    // Draupadi's own disguise at Virata's court, as the queen's chambermaid
    ("longbow", &["Shikhandi", "Dhrishtadyumna", "Prativindhya", "Sutasoma", "Shrutakarma", "Shatanika", "Shrutasena"]),
    ("spy", &["Sairandhri"]),
];

fn war1(who: &str, _: &Moment) -> String {
    format!("{who} will halt. Prove otherwise.")
}

/// Seat 1's anthem.
fn music1() -> Value {
    obj! {
        "title" => "Song of the Charioteer",
        "key" => "A",
        "mode" => "lydian",
        "tempo" => 96,
        "lead" => "pluck",
        "bass" => "fifths",
        "drums" => "heartbeat",
        "motif" => motif(&[
            (7.0, 1.0), (7.0, 0.5), (8.0, 0.5), (9.0, 1.0), (11.0, 1.0), (10.0, 0.5), (9.0, 0.5), (8.0, 1.0), (7.0, 2.0),
            (9.0, 1.0), (9.0, 0.5), (10.0, 0.5), (11.0, 1.0), (14.0, 1.0), (13.0, 0.5), (12.0, 0.5), (11.0, 0.5), (10.0, 0.5),
            (8.0, 1.0), (7.0, 1.0),
        ]),
    }
}

/// Seat 1's look, before [`Theme::look`] fills it in: maharathas and charioteers in chariots behind white horses,
/// archers with bows as tall as they are, war elephants under howdahs, mace fighters after Bhima, a seer with Sanjaya's
/// divine sight, Ghatotkacha (Bhima's rakshasa son, who fought by night) with a boulder, a vaidya with his pot, and the
/// Pandavas (Draupadi too) disguised as servants.
fn look1(b: f64) -> Value {
    const SKIN: &str = "#c89066";
    const GOLD: &str = "#e8b84a";
    obj! {
        "architecture" => "hive",
        "unit" => "robed",
        "worker" => "peasant",
        "ship" => "peacock",
        "units" => obj! {
            "cell" => "chariot",
            "guard" => "chariot",
            "miner" => "servant",
            "scout" => "chariot",
            "longbow" => "archer",
            "spy" => "servant",
        },
        "sprites" => obj! {
            "chariot" => obj! {
                "parts" => cat([
                    horse("#e6dcc6", "#8f8470", 2.6),
                    vec![
                        arr!["poly", arr![-8.4, -3.4, -1.8, -3.4, -2, -7.8, -8.4, -6.6], "primary"], arr!["circle", -5, -2.6, 2.6, "accent"],
                        arr!["circle", -4.8, -9.8, 1.6, SKIN], arr!["poly", arr![-8.2, -6, -8.2, -18.5, -3.8, -16.6, -7.4, -14.8], "secondary"],
                    ],
                ]),
            },
            "archer" => obj! {
                "parts" => arr![
                    arr!["line", -0.8, -4.5, -1.6, 0, 1.2, "#7a5a3a", "walkL"], arr!["line", 0.8, -4.5, 1.6, 0, 1.2, "#7a5a3a", "walkR"],
                    arr!["rect", -1.7, -9.4, 3.4, 5.4, "primary"], arr!["circle", 0, -10.8, 1.5, SKIN],
                    arr!["poly", arr![2.4, -16.5, 4.8, -8.5, 2.4, -0.5, 3.6, -8.5], "accent"],
                ],
            },
            "servant" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["poly", arr![-2.2, 0, -1.2, -6.2, 1.2, -6.2, 2.2, 0], "secondary", "bob"], arr!["circle", 0, -7.2, 1.3, SKIN, "bob"],
                    arr!["ellipse", 0, -8.4, 1.6, 1, "primary", "bob"],
                ],
            },
            "elephant" => obj! {
                "parts" => arr![
                    arr!["line", -4.4, -5, -4.6, 0, 2.2, "#77727c", "walkL"], arr!["line", 2, -5, 2.2, 0, 2.2, "#77727c", "walkR"],
                    arr!["ellipse", -1, -8.4, 6, 4, "#8f8a94"], arr!["circle", 4.6, -10, 3, "#8f8a94"],
                    arr!["line", 6.8, -9, 7.6, -2, 1.6, "#8f8a94"], arr!["line", 6, -7.6, 8.8, -6.6, 0.8, "#f4efe2"],
                    arr!["ellipse", 3.4, -10.4, 1.4, 2.4, "#77727c"], arr!["rect", -5.8, -10.2, 8, 3.6, "primary"],
                    arr!["line", -5.8, -6.6, 2.2, -6.6, 0.8, GOLD], arr!["rect", -4.2, -15.6, 6, 3.6, GOLD],
                    arr!["poly", arr![-4.8, -15.4, -1.2, -18.8, 2.4, -15.4], "primary"], arr!["circle", 3.4, -13.8, 1.1, SKIN],
                ],
            },
            "mace" => obj! {
                "parts" => cat([
                    legs(SKIN, 1.2, 0.0, 4.5),
                    vec![
                        arr!["poly", arr![-2.6, -2.8, -1.8, -6.4, 1.8, -6.4, 2.6, -2.8], "secondary"],
                        arr!["poly", arr![-1.9, -6.2, -2.3, -9.8, 2.3, -9.8, 1.9, -6.2], SKIN], arr!["line", -2, -9.6, 2, -6.4, 0.8, "primary"],
                        arr!["circle", 0, -11, 1.5, SKIN], arr!["poly", arr![-1.4, -12, -1, -14.6, 0, -13.4, 1, -14.6, 1.4, -12], GOLD],
                    ],
                ]),
            },
            "sage" => obj! {
                "parts" => cat([
                    vec![arr!["circle", 0, -11, 3.2, "#ffd27a"]],
                    man("#5a3a24", "#e8892c", SKIN, 2.4),
                    vec![
                        arr!["line", -1.8, -9.4, 1.8, -5.4, 1, "primary"], arr!["circle", 0.2, -12.8, 0.9, "#2a1a10"],
                        arr!["poly", arr![-0.6, -10.2, 0.8, -10.2, 0.2, -8.4], "secondary"],
                    ],
                ]),
            },
            "giant" => obj! {
                "parts" => cat([
                    legs("#3e5c4e", 2.0, 0.0, 7.0),
                    vec![
                        arr!["poly", arr![-3.4, -5, -3, -8.4, 3, -8.4, 3.4, -5], "primary"],
                        arr!["poly", arr![-3.6, -8, -4.2, -15, 4.2, -15, 3.6, -8], "#4e7462"], arr!["line", -3.6, -14, -4.6, -19, 1.4, "#4e7462"],
                        arr!["line", 3.6, -14, 4.6, -19, 1.4, "#4e7462"], arr!["line", -2, -14.4, 2, -14.4, 0.8, GOLD],
                        arr!["circle", 0, -16.6, 2.2, "#4e7462"],
                        arr!["poly", arr![-3, -16.8, -3.6, -19.6, -1, -18.4, 0, -20, 1.2, -18.4, 3.4, -19.4, 2.8, -16.6], "#1a1410"],
                        arr!["line", -0.6, -16.8, 1.2, -16.8, 0.6, "#ffd27a"],
                    ],
                ]),
            },
            "healer" => obj! {
                "parts" => cat([
                    man("#5a3a24", "secondary", SKIN, 2.4),
                    vec![
                        arr!["line", -1.8, -9.4, 1.6, -4.4, 1, "primary"], arr!["circle", -0.8, -12, 1, "#2a1a10"],
                        arr!["circle", 3, -6.4, 1.5, GOLD], arr!["rect", 2.2, -8.4, 1.6, 0.8, GOLD],
                        arr!["poly", arr![2.4, -8.4, 3, -10.6, 3.6, -8.4], "#5aa04a"],
                    ],
                ]),
            },
        },
        "names" => obj! {
            "cell" => "Maharatha",
            "guard" => "Maharatha",
            "miner" => "Disguised Pandava",
            "scout" => "Charioteer",
            "longbow" => "Archer",
            "spy" => "Spy",
        },
        "army" => obj! {
            "miner" => arr!["servant", "", "Disguised Pandava"],
            "screen" => arr!["elephant", "tusk", "War elephant"],
            "runner" => arr!["mace", "mace", "Mace fighter"],
            "skirm" => arr!["archer", "bow", "Archer"],
            "lancer" => arr!["chariot", "gandiva", "Maharatha"],
            "sentry" => arr!["sage", "sight", "Seer"],
            "gun" => arr!["giant", "boulder", "Ghatotkacha"],
            "repairer" => arr!["healer", "amrita", "Vaidya"],
            "ghost" => arr!["servant", "dagger", "Spy"],
            // the flying chariot of the epics
            "wing" => arr!["chariot", "gandiva", "Vimana"],
        },
        "arms" => obj! {
            "bow" => arm(1, "shot impact", 0.8, arr![], arr![3.6, -9], arrow(), obj! {}),
            "gandiva" => arm(
                1, "shot impact dash", 0.7,
                arr![arr!["poly", arr![0, -5.4, 1.8, 0, 0, 5.4, 0.9, 0], GOLD], arr!["line", 0, -5.4, 0, 5.4, 0.4, "#fff4c8"]], arr![-4.6, -11.4],
                arrow(),
                obj! {
                    "muzzle" => arr![1, 0],
                    "color" => "#ffd27a",
                    "glow" => 8,
                    "trail" => obj! { "color" => "#ffd27a", "length" => 0.6, "width" => 1.2, "particles" => 2 },
                },
            ),
            "tusk" => blow(1, 0.4, arr![8.6, -6.6], arr![], obj! {}),
            "mace" => blow(
                1, 0.6, arr![1.6, -9.4],
                arr![arr!["line", -1, 0, 5, 0, 1, "#8a5a32"], arr!["circle", 6.4, 0, 2.2, GOLD], arr!["circle", 6.4, 0, 0.9, "#b8862a"]],
                obj! { "muzzle" => arr![7, 0] },
            ),
            "sight" => arm(
                1, "shot impact", 1.3, arr![arr!["circle", 0, 0, 1, "#fff4c8"]], arr![2.4, -8], arr![],
                obj! {
                    "muzzle" => arr![0, 0],
                    "beam" => true,
                    "color" => "#ffd27a",
                    "glow" => 8,
                    "trail" => obj! { "color" => "#ffd27a", "width" => 1.2 },
                },
            ),
            "boulder" => arm(
                1, "shell shot impact", 0.4, arr![arr!["circle", 0, 0, 2.8, "#7a7068"], arr!["circle", -0.8, -0.8, 1.1, "#9a9088"]], arr![0, -19.4],
                arr![arr!["circle", 0, 0, 2.2, "#7a7068"]],
                with(
                    with(obj! { "muzzle" => arr![0, 0] }, fire()),
                    obj! {
                        "color" => "#b07aff",
                        "trail" => obj! { "color" => "#b07aff", "length" => 0.5, "width" => 2, "particles" => 3 },
                        "impact" => obj! { "color" => "#b07aff", "rings" => 2, "sparks" => 8 },
                    },
                ),
            ),
            "amrita" => arm(
                1, "repair shot impact dash", 1.0, arr![], arr![3, -8.4], arr![arr!["circle", 0, 0, 1.2, "#9fe0ff"]],
                obj! { "color" => "#9fe0ff", "glow" => 5, "trail" => obj! { "color" => "#9fe0ff", "length" => 0.4, "width" => 0.8 } },
            ),
            "dagger" => blow(1, 1.4, arr![1.6, -7], arr![], obj! {}),
        },
        "palette" => obj! { "primary" => "#4d9bff", "secondary" => "#efe2c4", "accent" => "#2f2418" },
        // the chariot's flagstaff: a staff in two lengths under its pennant
        "banner" => obj! {
            "pattern" => "stripes",
            "emblem" => arr![
                arr!["rect", 0, 3, 0.8, "#2f2418"], arr!["rect", 0, 0.4, 0.8, "#2f2418"], arr!["circle", 0, -2, 1.3, "#2f2418"],
                arr!["diamond", 0, -3.9, 0.9, "#2f2418"],
            ],
        },
        // lotus beds first; the flagstaff's gilt (glow); then the armies' standards (statues); lamps in the camp
        "ornaments" => if b >= 76.0 {
            arr!["gardens", "statues", "lights"]
        } else if b >= 69.0 {
            arr!["gardens", "statues"]
        } else {
            arr!["gardens"]
        },
        "wonder" => obj! {
            "shape" => "obelisk",
            "tiers" => if b >= 80.0 { 4 } else { 3 },
            "height" => if b >= 80.0 { 60 } else { 50 },
            "glow" => b >= 62.0,
            "spin" => false,
            "color" => "#d8c08a",
        },
    }
}

/// The capital is the field itself at dawn, nothing built on it: the chariot halted between the two armies, Krishna at
/// the reins turned to Arjuna, who has sat down in the car, Gandiva laid beside him, and looks up at him. The
/// conversation plays as the scene's script (Gita 1-2 and 18.73, in our own words), and the dawn raga under it is the
/// scene's music.
fn scene1(info: View, budget: f64) -> Value {
    let (c, gold, foe) = (info.get("color"), "#e8c37a", "#c8423a");
    let car = obj! {
        "type" => "chariot", "id" => "chariot", "x" => -2.5, "z" => 26, "rot" => 90, "horses" => 4, "coat" => "#f4f1ea", "flag" => true,
        "text" => "KAPIDHVAJA", "material" => "wood", "color" => "#8a5a2b", "trim" => gold,
    };
    let (front, back) = (build3d::on(&car, 0.0, 0.5), build3d::on(&car, 0.0, -0.5));
    let hero = obj! {
        "type" => "figure", "x" => back.x, "z" => back.z, "y" => 1.2, "h" => 2.5, "skin" => "#c89066", "color" => "#c9962e", "trim" => gold,
        "headwear" => "crown", "holds" => "bow",
    };
    let mut p = vec![
        car,
        obj! {
            "type" => "figure", "id" => "krishna", "x" => front.x, "z" => front.z, "y" => 1.2, "rot" => 320, "h" => 2.6, "pose" => "teach",
            "skin" => "#3f6fb5", "color" => "#f2c230", "trim" => gold, "headwear" => "plumed", "garland" => true, "glow" => true, "label" => "KRISHNA",
        },
        with(obj! { "id" => "arjuna", "rot" => 45, "pose" => "sit", "label" => "ARJUNA" }, hero.clone()),
        with(obj! { "id" => "arjuna-up", "rot" => 60, "pose" => "archer", "label" => "ARJUNA, RISEN", "hide" => true }, hero),
    ];
    // the Pandavas to the west, facing east; the Kauravas, eleven divisions to their seven, to the east facing west
    for (sx, rot, color, trim, n) in [(-1.0, 90, "#e9e1d0", Value::from(c), 0.0), (1.0, 270, "#6e2a2a", Value::from(foe), 1.0)] {
        let mut army = |kind: &str, x: f64, z: f64, rows: f64, cols: f64, banners: f64| {
            p.push(obj! {
                "type" => "army", "kind" => kind, "x" => sx * x, "z" => z, "rot" => rot, "rows" => rows, "cols" => cols + n * 2.0,
                "banners" => banners, "color" => color, "trim" => trim.clone(),
            });
        };
        army("foot", 24.0, 12.0, 3.0 + n, 24.0, 4.0);
        army("chariots", 34.0, 12.0, 1.0, 6.0, 0.0);
        army("elephants", 44.0, 12.0, 1.0, 4.0, 0.0);
        army("riders", 28.0, 40.0, 2.0, 5.0, 1.0);
        army("riders", 30.0, -20.0, 2.0, 6.0, 1.0);
        // a bigger budget fields more archers behind the lines (the expander's whole idea)
        for z in [22.0, 2.0, 40.0, -16.0].into_iter().take(super::capital::spare(budget, 24.0, 4.0) as usize) {
            army("archers", 53.0, z, 2.0, 8.0, 0.0);
        }
    }
    // the dust the armies raise
    for q in [[-34.0, 12.0], [34.0, 12.0], [0.0, -30.0]] {
        p.push(obj! { "type" => "particles", "kind" => "dust", "x" => q[0], "z" => q[1], "y" => 0, "n" => 60, "r" => 24, "color" => "#c9a77a", "pri" => 2 });
    }
    let say = |secs: f64, who: &str, line: &str| obj! { "secs" => secs, "say" => obj! { who => line } };
    obj! {
        "title" => "The Halt at Kurukshetra",
        "motto" => "Eighteen chapters to decide one loop.",
        "palette" => obj! { "primary" => c, "secondary" => "#efe2c4", "accent" => "#2f2418" },
        "sky" => obj! { "time" => "dawn", "top" => "#34406e", "horizon" => "#ffb066", "fog" => 0.42 },
        "ground" => obj! { "kind" => "grass", "color" => "#9c7f55", "open" => true },
        "pieces" => p,
        "music" => obj! {
            "title" => "Kurukshetra at Dawn",
            "key" => "A",
            "mode" => "bhairav",
            "tempo" => 66,
            "lead" => "choir",
            "bass" => "drone",
            "drums" => "taiko",
            "motif" => motif(&[
                (0.0, 2.0), (1.0, 1.0), (2.0, 1.0), (3.0, 2.0), (4.0, 2.0), (7.0, 2.0), (6.0, 1.0), (5.0, 1.0), (4.0, 3.0), (REST, 1.0),
                (5.0, 1.0), (6.0, 1.0), (7.0, 1.0), (8.0, 1.0), (9.0, 2.0), (8.0, 1.0), (7.0, 1.0), (5.0, 1.5), (4.0, 0.5), (3.0, 1.0),
                (1.0, 1.0), (0.0, 4.0),
            ]),
        },
        "script" => arr![
            obj! { "secs" => 4 },
            say(6.0, "arjuna", "Krishna, halt my chariot between the two armies."),
            say(6.0, "arjuna", "Teachers, uncles, cousins, friends. My limbs give way."),
            say(6.0, "arjuna", "Gandiva slips from my hand. I will not fight, Govinda."),
            say(6.5, "krishna", "Why this faintness at the hour of battle? Stand up, Arjuna."),
            say(6.5, "krishna", "The wise grieve neither for the living nor for the dead."),
            say(6.0, "krishna", "No weapon cuts the Self. No fire burns it."),
            say(6.5, "krishna", "Your right is to the work, never to its fruits."),
            say(6.0, "arjuna", "But will it halt? How can I act before I know?"),
            say(7.0, "krishna", "No one can know if it halts. Run it anyway, without clinging."),
            with(
                say(9.0, "arjuna-up", "My doubt is gone. I will do as you say."),
                obj! {
                    "set" => obj! { "arjuna" => obj! { "hide" => true }, "arjuna-up" => obj! { "hide" => false }, "krishna" => obj! { "glow" => 0.6 } },
                },
            ),
        ],
    }
}

// ---- 2 · templar · Knights Templated: templated orders, paid armor and a white mantle under a crimson cross.

const BRETHREN2: &[&str] = &["Hugues", "Geoffroi", "Bernard", "Andre", "Gondemar", "Payen", "Godefroy", "Jacques"];
const NAMES2: Names = &[
    ("miner", &["Allocator", "Scheduler", "Cache Keeper", "Registrar", "Archivist"]),
    ("guard", BRETHREN2),
    ("bulwark", BRETHREN2),
    ("line", BRETHREN2),
    ("scout", &["First Watch", "Second Watch", "Third Watch", "Fourth Watch", "Fifth Watch"]),
    ("longbow", &["Ballista", "Arbalest"]),
    ("artillery", &["Ballista", "Arbalest"]),
    ("medic", &["Hospitaller", "Healer"]),
    ("interceptor", &["Turcopole", "Outrider"]),
    ("piercer", &["Arbalest", "Crossbow"]),
    ("spy", &["Cipher", "Seal"]),
];

fn war2(who: &str, _: &Moment) -> String {
    format!("The template holds. Advance against {who}.")
}

/// Seat 2's anthem.
fn music2() -> Value {
    obj! {
        "title" => "The Compiled Crusade",
        "key" => "F",
        "mode" => "minor",
        "tempo" => 120,
        "lead" => "brass",
        "bass" => "fifths",
        "drums" => "march",
        "motif" => motif(&[
            (0.0, 1.0), (0.0, 0.5), (4.0, 0.5), (7.0, 1.0), (5.0, 1.0), (4.0, 1.0), (2.0, 1.0), (0.0, 2.0),
        ]),
    }
}

/// Seat 2's look, before [`Theme::look`] fills it in: knights in white mantles behind kite shields; sergeants in black,
/// as the Rule had them, with kettle hats and spears; crossbowmen; turcopoles (light horse archers hired in the East);
/// the marshal with the Beauseant, the order's black-and-white banner; a ballista and its crew; Hospitallers in black
/// under a white cross; and acolytes.
fn look2(b: f64) -> Value {
    const SKIN: &str = "#e0b48c";
    obj! {
        "architecture" => "stone",
        "unit" => "crusader",
        "worker" => "acolyte",
        "ship" => "cog",
        "units" => obj! { "miner" => "acolyte", "guard" => "crusader", "scout" => "marshal", "longbow" => "crossbowman", "spy" => "acolyte" },
        "sprites" => obj! {
            "crusader" => obj! {
                "parts" => arr![
                    arr!["line", -1.5, -4, -2, 0, 1.5, "#38434a", "walkL"], arr!["line", 1.5, -4, 2, 0, 1.5, "#38434a", "walkR"],
                    arr!["poly", arr![-4, -12, 4, -12, 3, -3, -3, -3], "#f0e6c8"], arr!["rect", -3, -18, 6, 5, "#a8b6bd"],
                    arr!["line", -2, -15.5, 2, -15.5, 1, "#182531"], arr!["rect", -0.7, -12, 1.4, 7, "#b52b36"],
                    arr!["rect", -2.4, -10.5, 4.8, 1.4, "#b52b36"], arr!["poly", arr![-6, -11, -3, -11, -3, -6, -4.5, -4], "#d0ab54"],
                ],
            },
            "acolyte" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["line", -1, -3, -1, 0, 1, "#38434a", "walkL"], arr!["line", 1, -3, 1, 0, 1, "#38434a", "walkR"],
                    arr!["poly", arr![-3, -10, 3, -10, 2, -2, -2, -2], "#f0e6c8"], arr!["circle", 0, -12, 2, "#a8b6bd"],
                    arr!["rect", -0.6, -9, 1.2, 5, "#b52b36"],
                ],
            },
            "sergeant" => obj! {
                "parts" => cat([
                    legs("#38434a", 1.5, 0.0, 4.0),
                    vec![
                        arr!["poly", arr![-3.4, -11.6, 3.4, -11.6, 2.8, -3.4, -2.8, -3.4], "#2e2622"], arr!["rect", -0.6, -11, 1.2, 6, "#b52b36"],
                        arr!["rect", -2.2, -9.6, 4.4, 1.2, "#b52b36"], arr!["circle", 0, -13, 1.6, SKIN], arr!["circle", 0, -14.2, 1.7, "#a8b6bd"],
                        arr!["ellipse", 0, -13.8, 3, 0.7, "#a8b6bd"],
                    ],
                ]),
            },
            "crossbowman" => obj! {
                "parts" => cat([
                    legs("#38434a", 1.4, 0.0, 4.5),
                    vec![
                        arr!["poly", arr![-2.6, -3.6, -2, -10, 2, -10, 2.6, -3.6], "#8a7a5a"], arr!["rect", -0.5, -9.4, 1, 3.6, "#b52b36"],
                        arr!["rect", -1.6, -8.4, 3.2, 1, "#b52b36"], arr!["circle", 0, -11.4, 1.5, SKIN], arr!["circle", 0, -12.4, 1.6, "#a8b6bd"],
                        arr!["ellipse", 0, -12, 2.8, 0.6, "#a8b6bd"],
                    ],
                ]),
            },
            "turcopole" => obj! {
                "parts" => cat([
                    rider("#c49a6a", "#6a4a2a", "#e8e2d0", SKIN),
                    vec![
                        arr!["rect", -1, -10.8, 1, 3.4, "#b52b36"], arr!["ellipse", -0.4, -13.6, 1.8, 1.1, "#f4f0e6"],
                        arr!["line", -0.4, -14.4, -0.4, -16, 0.6, "#a8b6bd"], arr!["poly", arr![-2, -12.8, -5.6, -12, -2, -11.4], "#f4f0e6"],
                    ],
                ]),
            },
            "marshal" => obj! {
                "parts" => cat([
                    rider("#6a4a34", "#3a2a20", "#f0e6c8", SKIN),
                    vec![
                        arr!["rect", -1.9, -15.8, 3, 3.4, "#a8b6bd"], arr!["line", -1.4, -14.2, 0.9, -14.2, 0.7, "#182531"],
                        arr!["rect", -0.9, -10.8, 1, 4, "#b52b36"], arr!["line", -3.6, -4.6, -3.6, -19.8, 0.6, "#5a4030"],
                        arr!["rect", -8.2, -19.8, 4.6, 2.2, "#1a1a1a"], arr!["rect", -8.2, -17.6, 4.6, 2.2, "#f4f0e6"],
                    ],
                ]),
            },
            "ballista" => obj! {
                "parts" => arr![
                    arr!["rect", -7, -3.6, 13, 2, "#735544"], arr!["circle", -4.4, -1.6, 1.8, "#4a3428"], arr!["circle", 3.6, -1.6, 1.8, "#4a3428"],
                    arr!["line", -1, -3, -1, -8, 1.4, "#5a4030"], arr!["poly", arr![-10, -2, -9.6, -8, -7, -8, -6.6, -2], "#2e2622"],
                    arr!["rect", -8.7, -7.4, 0.9, 4, "#b52b36"], arr!["circle", -8.3, -9.4, 1.4, SKIN],
                    arr!["ellipse", -8.3, -10.4, 2.2, 0.7, "#a8b6bd"],
                ],
            },
            "hospitaller" => obj! {
                "parts" => cat([
                    legs("#38434a", 1.2, 0.0, 4.5),
                    vec![
                        arr!["poly", arr![-2.8, -3.4, -2, -9.8, 2, -9.8, 2.8, -3.4], "#1e1c1e"], arr!["circle", 0, -11, 1.9, "#1e1c1e"],
                        arr!["circle", 0.5, -10.8, 1.1, SKIN], arr!["rect", -0.5, -9.2, 1, 3.6, "#f4f0e6"], arr!["rect", -1.6, -8, 3.2, 1, "#f4f0e6"],
                        arr!["line", -1.6, -9.4, 2.6, -6.2, 0.5, "#8a6a42"], arr!["rect", 2, -6.6, 1.8, 1.6, "#8a6a42"],
                    ],
                ]),
            },
        },
        "names" => obj! { "miner" => "CPU Acolyte", "guard" => "Crusader", "scout" => "Marshal", "longbow" => "Arbalest", "spy" => "Cipher" },
        "army" => obj! {
            "miner" => arr!["acolyte", "", "CPU Acolyte"],
            "line" => arr!["sergeant", "spear", "Sergeant"],
            "bulwark" => arr!["crusader", "sword", "Knight"],
            "artillery" => arr!["ballista", "ballista", "Ballista"],
            "medic" => arr!["hospitaller", "care", "Hospitaller"],
            "piercer" => arr!["crossbowman", "crossbow", "Crossbowman"],
            "interceptor" => arr!["turcopole", "shortbow", "Turcopole"],
            "scout" => arr!["marshal", "", "Marshal"],
        },
        "arms" => obj! {
            "spear" => blow(
                2, 0.9, arr![2.4, -8], arr![arr!["line", -4, 0, 9, 0, 0.8, "#8a6a42"], arr!["poly", arr![11.4, 0, 8.8, -1, 8.8, 1], "#a8b6bd"]],
                obj! { "muzzle" => arr![11, 0] },
            ),
            "sword" => blow(
                2, 0.7, arr![2.6, -8.4], arr![arr!["line", 0, 0, 8, 0, 1.2, "#d8dce4"], arr!["line", 0.6, -2, 0.6, 2, 1, "#d0ab54"]],
                obj! { "muzzle" => arr![8, 0] },
            ),
            "crossbow" => arm(
                2, "shot impact dash", 1.2,
                arr![
                    arr!["line", -2, 0, 6, 0, 1.2, "#6a4a2a"], arr!["line", 4.6, -3, 4.6, 3, 0.9, "#a8b6bd"],
                    arr!["line", 4.6, -3, -0.6, 0, 0.3, "#e8e2d0"], arr!["line", 4.6, 3, -0.6, 0, 0.3, "#e8e2d0"],
                ],
                arr![1.4, -8.2], bolt(), obj! { "muzzle" => arr![6, 0], "color" => "#e2c477" },
            ),
            "shortbow" => arm(
                2, "shot impact dash", 1.6,
                arr![arr!["poly", arr![0, -4, 1.4, 0, 0, 4, 0.7, 0], "#6a4a2a"], arr!["line", 0, -4, 0, 4, 0.3, "#e8e2d0"]], arr![2, -10], arrow(),
                obj! { "color" => "#e2c477" },
            ),
            "ballista" => arm(
                2, "shell shot impact", 0.8,
                arr![
                    arr!["rect", -3, -1, 12, 2, "#735544"], arr!["line", 5, -5, 5, 4, 1.8, "#d0ab54"], arr!["line", 0, 0, 5, -5, 0.7, "#f0e6c8"],
                    arr!["line", 0, 0, 5, 4, 0.7, "#f0e6c8"],
                ],
                arr![-1, -8.6], arr![arr!["poly", arr![5, 0, 0, -2, -4, 0, 0, 2], "#d0ab54"], arr!["line", -5, 0, 4, 0, 1, "#ffffff"]],
                obj! {
                    "muzzle" => arr![11, 0],
                    "color" => "#d0ab54",
                    "core" => "#fff7dc",
                    "glow" => 5,
                    "trail" => obj! { "color" => "#bd4050", "length" => 0.6, "width" => 1.6, "particles" => 3 },
                    "impact" => obj! { "color" => "#e4c46a", "rings" => 2, "sparks" => 5 },
                },
            ),
            "care" => arm(2, "repair", 1.0, arr![], arr![0, -8], arr![], obj! {}),
        },
        "palette" => obj! { "secondary" => "#b52b36", "accent" => "#d0ab54" },
        "banner" => obj! { "pattern" => "cross", "emblem" => arr![arr!["diamond", 0, 0, 3, "#d0ab54"]] },
        "ornaments" => if b >= 70.0 { arr!["walls", "flags"] } else { arr!["flags"] },
        "wonder" => obj! { "shape" => "spire", "tiers" => 3, "height" => 30, "glow" => b >= 68.0, "color" => "#f0e6c8" },
    }
}

/// The Sanctum: the civic core under a spire.
fn scene2(info: View, _: f64) -> Value {
    obj! {
        "title" => "The Sanctum",
        "motto" => "One bank. Many sworn instructions.",
        "palette" => obj! { "primary" => info.get("color"), "secondary" => "#b52b36", "accent" => "#d0ab54" },
        "sky" => obj! { "time" => "day", "top" => "#466a94", "horizon" => "#f2d7a5", "fog" => 0.15 },
        "ground" => obj! { "kind" => "stone", "color" => "#898578" },
        "pieces" => super::capital::civic(
            info,
            &Civic {
                wonder: obj! { "shape" => "spire", "tiers" => 3, "height" => 30, "glow" => true },
                wonder_r: 6.0,
                wonder_color: Some("#f0e6c8"),
                trim: Some("#d0ab54"),
                hall_label: Some("CHAPTER HOUSE"),
                screen_title: Some("SWORN ORDERS"),
                banners: 4.0,
                ..Civic::default()
            },
        ),
    }
}

// ---- 3 · adaptive · New Atlantis: Francis Bacon's island of Bensalem and its research college, Salomon's House ("the
// effecting of all things possible"), whose fellows swear an oath of secrecy over what to publish. Knowledge is power.

const NAMES3: Names = &[
    ("enforcer", &["Depredator", "Mystery-man", "Compiler", "Dowry-man", "Benefactor", "Lamp", "Inoculator", "Interpreter", "Novice", "Apprentice"]),
    // the default program's, in Salomon's House's own ranks: Pioneers ("Pioneers or Miners", who try new experiments),
    // named for what they dig; Fellows, after the twelve at the Royal Society's first meeting (Gresham College, 28
    // November 1660), a society modelled on Salomon's House; and a Merchant of Light, who sails abroad "under the names
    // of other nations (for our own we conceal)"
    ("miner", &["Saltpetre", "Vitriol", "Quicksilver", "Antimony", "Cinnabar", "Alum", "Lodestone", "Sulphur", "Orpiment", "Bismuth"]),
    ("guard", &["Wren", "Boyle", "Wilkins", "Moray", "Brouncker", "Petty", "Goddard", "Rooke", "Neile", "Bruce", "Balle", "Hill"]),
    ("scout", &["Venetian", "Genoese", "Portuguese", "Castilian", "Fleming", "Hollander", "Frenchman", "Englishman", "Dane", "Florentine"]),
    // gunners ("new mixtures and compositions of gun-powder") in the House's ranks; and a Mystery-man, who collects
    // "the experiments of all mechanical arts", named for the art he is after
    ("longbow", &["Depredator", "Compiler", "Dowry-man", "Benefactor", "Lamp", "Inoculator", "Interpreter", "Novice", "Apprentice"]),
    ("spy", &["Clockwork", "Glassmaking", "Dyeing", "Printing", "Gunfounding", "Sugar Refining"]),
];

fn war3(who: &str, m: &Moment) -> String {
    format!("{who} at {}. Knowledge is power, and we took notes.", m.at.as_deref().unwrap_or("undefined"))
}

/// Seat 3's anthem.
fn music3() -> Value {
    obj! {
        "title" => "Sound-Houses",
        "key" => "G",
        "mode" => "wholetone",
        "tempo" => 112,
        "lead" => "bell",
        "bass" => "pulse",
        "drums" => "clock",
        "motif" => motif(&[
            (6.0, 0.5), (8.0, 0.5), (10.0, 0.5), (12.0, 0.5), (10.0, 0.5), (8.0, 0.5), (6.0, 1.0), (7.0, 0.5), (9.0, 0.5),
            (11.0, 0.5), (13.0, 0.5), (11.0, 0.5), (9.0, 0.5), (7.0, 1.0), (6.0, 0.5), (8.0, 0.5), (10.0, 0.5), (12.0, 0.5),
            (14.0, 1.0), (16.0, 1.0), (15.0, 0.5), (13.0, 0.5), (REST, 1.0), (6.0, 2.0),
        ]),
    }
}

/// Seat 3's look, before [`Theme::look`] fills it in: Pioneers with their lamps, Fellows in periwigs with telescopes,
/// wound-up brass automata, a walking engine from the engine-houses, a gun crew, Merchants of Light under sail, a
/// Mystery-man in a wide hat with a burning glass from the perspective-houses, and a flying machine ("we have some
/// degrees of flying in the air").
fn look3(b: f64) -> Value {
    obj! {
        "architecture" => "foundry",
        "unit" => "mech",
        "worker" => "peasant",
        "ship" => "steamer",
        "units" => obj! {
            "miner" => "pioneer",
            "probe" => "flyer",
            "light" => "ship",
            "enforcer" => "automaton",
            "guard" => "fellow",
            "scout" => "ship",
            "longbow" => "gunner",
            "spy" => "mystery",
        },
        "sprites" => obj! {
            "engine" => obj! {
                "parts" => cat([
                    legs("#3b3f46", 2.0, 0.0, 5.0),
                    vec![
                        arr!["ellipse", -0.6, -9, 4.6, 3.6, "#c9a14a"], arr!["line", -2.6, -12.4, -2.6, -5.6, 0.7, "#8a6a2a"],
                        arr!["line", 1.4, -12.4, 1.4, -5.6, 0.7, "#8a6a2a"], arr!["rect", -3.6, -17, 1.8, 5, "#3b3f46"],
                        arr!["rect", -4, -17.6, 2.6, 1, "#3b3f46"], arr!["circle", -3.2, -19, 1.3, "#d8d4cc", "bob"],
                        arr!["poly", arr![2.4, -12, 4.8, -10, 4.8, -6.4, 2.4, -5.4], "primary"], arr!["circle", 3.4, -9, 0.9, "#7ff6ff"],
                    ],
                ]),
            },
            "pioneer" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["rect", -1.6, -6, 3.2, 6, "#7a5a3a", "bob"], arr!["circle", 0, -7.2, 1.3, "#e0b48c", "bob"],
                    arr!["ellipse", 0, -8.2, 1.6, 0.9, "primary", "bob"], arr!["circle", 2.6, -3.4, 1.2, "accent", "bob"],
                ],
            },
            "fellow" => obj! {
                "parts" => arr![
                    arr!["line", -0.8, -4.5, -1.4, 0, 1.2, "secondary", "walkL"], arr!["line", 0.8, -4.5, 1.4, 0, 1.2, "secondary", "walkR"],
                    arr!["poly", arr![-2.4, -3.4, -1.6, -9.4, 1.6, -9.4, 2.4, -3.4], "primary"], arr!["ellipse", -0.4, -10.4, 2.3, 2.4, "#ece6da"],
                    arr!["circle", 0.6, -10.6, 1.1, "#e0b48c"], arr!["line", 1.4, -8, 5.6, -12.4, 1.1, "#c9a14a"],
                ],
            },
            "automaton" => obj! {
                "parts" => arr![
                    arr!["line", -1, -4, -1.2, 0, 1.4, "#6b625a", "walkL"], arr!["line", 1, -4, 1.2, 0, 1.4, "#6b625a", "walkR"],
                    arr!["rect", -2.6, -9.6, 5.2, 5.8, "primary"], arr!["rect", -1.6, -12.6, 3.2, 3, "#c9a14a"],
                    arr!["rect", -0.6, -11.6, 2, 0.8, "#7ff6ff"], arr!["line", -2.6, -7.6, -4.4, -7.6, 0.6, "#c9a14a"],
                    arr!["ellipse", -4.8, -7.6, 0.7, 1.6, "#c9a14a"], arr!["line", -2.6, -6, 2.6, -6, 0.6, "#c9a14a"],
                ],
            },
            "gunner" => obj! {
                "parts" => arr![
                    arr!["poly", arr![-3, -4.2, 6.4, -6.8, 6.6, -5.2, -2.6, -2.4], "#3b3f46"], arr!["circle", -0.6, -2.4, 2.4, "#7a5a3a"],
                    arr!["rect", -6.6, -9, 3, 5, "primary"], arr!["circle", -5.1, -10.4, 1.4, "#e0b48c"],
                    arr!["line", -5.6, -4, -6, 0, 1.2, "secondary", "walkL"],
                ],
            },
            "ship" => obj! {
                "parts" => arr![
                    arr!["poly", arr![-7, -4.4, 7.4, -4.4, 5.2, 0, -5.6, 0], "#5a3a24", "bob"], arr!["line", 0, -4, 0, -16.5, 0.8, "#3a2a1a", "bob"],
                    arr!["poly", arr![-0.6, -5.2, -5.8, -6, -0.6, -15.6], "primary", "bob"],
                    arr!["poly", arr![0.6, -5.2, 5.4, -5.6, 0.6, -13], "accent", "bob"],
                ],
            },
            "mystery" => obj! {
                "parts" => arr![
                    arr!["poly", arr![-2.6, 0, -1.4, -8, 1.4, -8, 2.6, 0], "primary", "bob"], arr!["circle", 0, -9, 1.3, "#e0b48c", "bob"],
                    arr!["ellipse", 0, -10, 3.4, 0.9, "secondary", "bob"], arr!["rect", -1.4, -12, 2.8, 2, "secondary", "bob"],
                ],
            },
            "flyer" => obj! {
                "parts" => arr![
                    arr!["poly", arr![-0.6, -10, -7, -15.6, -4.4, -9.4], "primary", "bob"],
                    arr!["poly", arr![0.6, -10, 6.4, -15, 4, -9.4], "primary", "bob"], arr!["ellipse", 0, -9.2, 3.4, 1.2, "#7a5a3a", "bob"],
                    arr!["poly", arr![-2.6, -9.2, -6, -11, -6, -7.6], "accent", "bob"], arr!["circle", 1, -10.8, 1.1, "#e0b48c", "bob"],
                ],
            },
        },
        "names" => obj! { "miner" => "Pioneer", "guard" => "Fellow", "scout" => "Merchant of Light", "longbow" => "Gunner", "spy" => "Mystery-man" },
        "army" => obj! {
            "scout" => arr!["ship", "", "Merchant of Light"],
            "line" => arr!["automaton", "pike", "Automaton"],
            "bulwark" => arr!["engine", "ram", "Engine"],
            "artillery" => arr!["gunner", "gun", "Gun crew"],
            "medic" => arr!["fellow", "care", "Inoculator"],
            "piercer" => arr!["mystery", "glass", "Lamp"],
            "interceptor" => arr!["flyer", "swivel", "Flying machine"],
        },
        "arms" => obj! {
            "pike" => blow(
                3, 0.5, arr![-0.6, -7.6], arr![arr!["line", -3, 0, 9, 0, 0.8, "#6b625a"], arr!["poly", arr![8, -2.4, 10.4, 0, 8, 1.6], "#d8dce4"]],
                obj! { "muzzle" => arr![10, 0] },
            ),
            "ram" => blow(
                3, 0.3, arr![4.6, -8], arr![arr!["rect", 0, -1, 6, 2, "#6b625a"], arr!["rect", 5.4, -1.8, 1.6, 3.6, "#c9a14a"]],
                obj! { "muzzle" => arr![7, 0] },
            ),
            "gun" => arm(3, "shell shot impact", 0.3, arr![], arr![6.6, -6], ball(), smoke()),
            "care" => arm(3, "repair", 1.0, arr![], arr![0, -8], arr![], obj! {}),
            "glass" => arm(
                3, "shot impact dash", 0.8,
                arr![arr!["line", 0, 0, 3, 0, 0.6, "#5a3a24"], arr!["circle", 4, 0, 1.6, "#bfefff"], arr!["circle", 4, 0, 0.7, "#ffffff"]],
                arr![1.4, -7], arr![],
                obj! {
                    "muzzle" => arr![4, 0],
                    "beam" => true,
                    "color" => "#ffd27a",
                    "glow" => 8,
                    "trail" => obj! { "color" => "#fff1b0", "width" => 1.4 },
                },
            ),
            "swivel" => arm(
                3, "shot impact dash", 0.6, arr![arr!["rect", 0, -0.5, 5, 1, "#3b3f46"]], arr![1, -9.4], ball(),
                with(obj! { "muzzle" => arr![5, 0] }, smoke()),
            ),
        },
        "palette" => obj! { "secondary" => "#3b3f46", "accent" => "#ffe2b8" },
        // Bacon's ship sailing out between the Pillars of Hercules (the Instauratio Magna's title page, 1620)
        "banner" => obj! {
            "pattern" => "circuit",
            "emblem" => arr![
                arr!["rect", -3.6, 0.4, 1.2, "#f3ead8"], arr!["rect", 3.6, 0.4, 1.2, "#f3ead8"], arr!["tri", 0, -0.6, 2.4, "#3b3f46"],
                arr!["diamond", 0, 2.4, 1.2, "#3b3f46"],
            ],
        },
        "ornaments" => if b >= 78.0 {
            arr!["smoke", "lights"]
        } else if b >= 64.0 {
            arr!["smoke"]
        } else {
            arr![]
        },
        "wonder" => obj! {
            "shape" => "ziggurat",
            "tiers" => if b >= 64.0 { 4 } else { 3 },
            "height" => if b >= 64.0 { 50 } else { 40 },
            "glow" => b >= 70.0,
            "spin" => false,
            "color" => "#c9a14a",
        },
    }
}

/// Salomon's House on Bensalem.
fn scene3(info: View, budget: f64) -> Value {
    let (c, brass, brick, iron) = (info.get("color"), "#c9a14a", "#7a4b3a", "#5a5f66");
    // Salomon's House, "the lantern of this kingdom": its towers ("the highest about half a mile in height") as one
    // stepped tower, lit at the top, is the wonder; the hall is an engine-house
    let mut p = super::capital::civic(
        info,
        &Civic {
            wonder: obj! { "shape" => "ziggurat", "tiers" => 4, "height" => 50, "glow" => true },
            wonder_r: 6.0,
            wonder_color: Some(brass),
            wonder_mat: Some("metal"),
            trim: hue(c),
            roof: Some("sawtooth"),
            mat: Some("metal"),
            hall_color: Some("#6b625a"),
            hall_h: 10.0,
            hall_label: Some("ENGINE-HOUSE"),
            screen_title: Some("EXPERIMENTS OF LIGHT"),
            banners: 2.0,
            ..Civic::default()
        },
    );
    // the Merchants of Light: their house, its plaque, and the light they bring home on a lit belt to the engine-house
    p.push(obj! {
        "type" => "tower", "id" => "merchants", "x" => -42, "z" => 22, "r" => 6, "h" => 8, "sides" => 4, "floors" => 2, "windows" => true,
        "roof" => "flat", "color" => brick, "trim" => c,
    });
    p.push(obj! { "type" => "statue", "pose" => "obelisk", "x" => -31, "z" => 30, "h" => 3, "color" => "#8f8a80", "label" => "MERCHANTS OF LIGHT" });
    p.push(obj! { "type" => "bridge", "x" => -38, "z" => 15, "x2" => -29, "z2" => -12, "y" => 2.5, "w" => 2, "material" => "emissive", "color" => c });
    p.push(obj! { "type" => "screen", "id" => "says", "x" => 36, "z" => 24, "y" => 2, "w" => 10, "h" => 6, "show" => "says", "title" => "BENSALEM SAYS" });
    // the sound-houses, in Bacon's words, which the anthem answers
    p.push(obj! {
        "type" => "screen", "id" => "sound", "x" => 47, "z" => 2, "y" => 2, "w" => 12, "h" => 6, "show" => "text", "title" => "SOUND-HOUSES",
        "text" => "We have harmonies which you have not, of quarter-sounds",
    });
    // "divers furnaces of great diversities": three brick chimneys, smoke and embers
    for q in [[-44.0, -38.0], [-38.0, -44.0], [-32.0, -48.0]] {
        p.push(obj! { "type" => "tower", "x" => q[0], "z" => q[1], "r" => 2, "h" => 22, "sides" => 8, "roof" => "flat", "color" => brick, "trim" => "#3b2a22" });
        p.push(obj! { "type" => "particles", "kind" => "smoke", "x" => q[0], "z" => q[1], "y" => 22.5, "n" => 40, "r" => 1.5, "color" => "#8a8580", "pri" => 1 });
    }
    // the engine-house's "perpetual motions": flat brass rings turning about an iron axle
    p.push(obj! { "type" => "tower", "x" => 24, "z" => 8, "r" => 0.7, "h" => 11, "sides" => 8, "roof" => "flat", "material" => "metal", "color" => iron });
    for y in [2, 5, 8] {
        p.push(obj! {
            "type" => "ring", "x" => 24, "z" => 8, "y" => y, "r" => 3, "t" => 0.4, "tilt" => 90, "spin" => true, "material" => "metal", "color" => brass,
            "trim" => "#8a6a2a",
        });
    }
    // "images of men, beasts, birds": three automata, and the founder in bronze
    for q in [[-12, 36], [0, 40], [12, 36]] {
        p.push(obj! { "type" => "statue", "pose" => "mech", "x" => q[0], "z" => q[1], "h" => 8, "material" => "metal", "color" => iron });
    }
    p.push(obj! { "type" => "statue", "pose" => "robed", "x" => 0, "z" => 28, "h" => 6, "material" => "metal", "color" => "#a8844f", "label" => "FRANCIS BACON" });
    p.push(obj! { "type" => "particles", "kind" => "data", "x" => 0, "z" => 0, "y" => 14, "n" => 120, "r" => 10, "color" => "#ffe2b8" });
    p.push(obj! { "type" => "particles", "kind" => "embers", "x" => -38, "z" => -38, "y" => 1, "n" => 80, "r" => 5, "color" => "#ff9a3a", "pri" => 2 });
    // the mechanical arts ("papers, linen, silks, tissues"): a loom of two frames and a beam; and flowers made to come
    // out of season
    for z in [-42, -28] {
        p.push(obj! { "type" => "tower", "x" => 46, "z" => z, "r" => 1.5, "h" => 10, "sides" => 4, "roof" => "flat", "material" => "metal", "color" => "#6b625a" });
    }
    p.push(obj! { "type" => "bridge", "x" => 46, "z" => -42, "x2" => 46, "z2" => -28, "y" => 10, "w" => 3, "material" => "metal", "color" => "#6b625a" });
    p.push(obj! { "type" => "garden", "x" => 44, "z" => 42, "r" => 5, "flowers" => 40, "color" => "#e86a9a" });
    // an "image of a bird" in its pond: Vaucanson's duck (1739), which seemed to digest and did not. Tap it.
    p.push(obj! { "type" => "water", "shape" => "round", "x" => -32, "z" => 46, "w" => 10, "d" => 10, "color" => "#3d5a5a" });
    p.push(obj! {
        "type" => "statue", "pose" => "orb", "id" => "duck", "tap" => true, "x" => -32, "z" => 46, "h" => 2.5, "material" => "metal", "color" => "#e8b04a",
        "label" => "DO NOT FEED THE DUCK",
    });
    // the harbour the Merchants of Light sail from (Bensalem is an island)
    p.push(obj! { "type" => "water", "shape" => "rect", "x" => 0, "z" => -56, "w" => 110, "d" => 8, "color" => "#2b4a6b" });
    // the Lamps (Bacon's rank for the three who direct new experiments): post lamps (a bigger budget lights more streets)
    const LAMPS: [[f64; 2]; 9] = [[-20.0, -34.0], [20.0, -34.0], [-20.0, 44.0], [20.0, 44.0], [-8.0, -46.0], [8.0, -46.0], [-52.0, 0.0], [36.0, -50.0], [52.0, 30.0]];
    for q in LAMPS.iter().take((4.0 + super::capital::spare(budget, 4.0, 5.0)) as usize) {
        p.push(obj! { "type" => "light", "x" => q[0], "z" => q[1], "y" => 3, "post" => true, "power" => 1, "color" => "#ffd08a" });
    }
    obj! {
        "title" => "Salomon's House",
        "motto" => "The effecting of all things possible. Under NDA.",
        "palette" => obj! { "primary" => c, "secondary" => "#3b3f46", "accent" => "#ffe2b8" },
        "sky" => obj! { "time" => "dusk", "top" => "#1f2f4f", "horizon" => "#ffb36b", "fog" => 0.3 },
        "ground" => obj! { "kind" => "stone", "color" => "#8a8070" },
        "pieces" => p,
    }
}

// ---- 4 · diplomat · Quiet Fleet: Zheng He's treasure fleet, the largest afloat, built at the Longjiang yard in Nanjing.
// Seven voyages (1405-1433) to thirty-odd courts, a giraffe for the emperor, and then the court quietly stopped sending
// it.

// the courts the fleet called at, and the points of a mariner's compass
const PORTS4: &[&str] = &[
    "Champa", "Java", "Palembang", "Malacca", "Semudera", "Ceylon", "Calicut", "Cochin", "Hormuz", "Dhofar", "Aden", "Brava", "Lambri", "Bengal",
];
const POINTS4: &[&str] = &["Zi", "Chou", "Yin", "Chen", "Wu", "Wei", "Shen", "Xu", "Hai", "Gen", "Xun", "Kun", "Qian"];
const NAMES4: Names = &[
    ("envoy", PORTS4),
    ("marine", POINTS4),
    ("sailor", &["Qilin", "Zebra", "Oryx", "Ostrich", "Lion", "Leopard", "Peacock", "Parrot", "Camel", "Tortoise"]),
    // the default program's: porters named for the cargo the treasure ships brought home, marines, and an envoy
    ("miner", &["Pepper", "Sappanwood", "Ambergris", "Frankincense", "Myrrh", "Ivory", "Pearl", "Sandalwood", "Cloves", "Camphor"]),
    ("guard", POINTS4),
    ("scout", PORTS4),
    // crossbows at the rest of the compass's points (its eight heavenly stems); and a chronicler, who wrote down the
    // courts the fleet saw: Ma Huan, its interpreter, Fei Xin and Gong Zhen
    ("longbow", &["Jia", "Yi", "Bing", "Ding", "Geng", "Xin", "Ren", "Gui"]),
    ("spy", &["Ma Huan", "Fei Xin", "Gong Zhen"]),
];

fn war4(who: &str, _: &Moment) -> String {
    format!("{who} is loud. We are a quiet fleet. A very large one.")
}

/// Seat 4's anthem.
fn music4() -> Value {
    obj! {
        "title" => "Seven Voyages, Then Quiet",
        "key" => "C",
        "mode" => "pentatonic",
        "tempo" => 80,
        "lead" => "reed",
        "bass" => "none",
        "drums" => "taiko",
        "motif" => motif(&[
            (0.0, 1.0), (1.0, 0.5), (2.0, 0.5), (4.0, 2.0), (3.0, 1.0), (2.0, 0.5), (1.0, 0.5), (2.0, 2.0), (4.0, 1.0), (5.0, 1.0),
            (7.0, 1.0), (6.0, 0.5), (5.0, 0.5), (4.0, 1.0), (2.0, 1.0), (0.0, 1.0), (REST, 1.0),
        ]),
    }
}

/// Seat 4's look, before [`Theme::look`] fills it in: marines in treasure junks under battened sails, porters with a
/// carrying pole, envoys on a qilin (a giraffe), crossbows in wide hats, a chronicler with a scroll in an official's
/// winged hat, marines with rattan shields and sabres or fire lances, a watchman's signal lantern (the fleet signalled
/// with them by night), and a cart of fire arrows (the nest of bees, a box that launched them by the dozen).
fn look4(b: f64) -> Value {
    const SKIN: &str = "#e0b48c";
    obj! {
        "architecture" => "pagoda",
        "unit" => "robed",
        "worker" => "hooded",
        "ship" => "junk",
        "units" => obj! {
            "envoy" => "qilin",
            "sailor" => "porter",
            "marine" => "junk",
            "miner" => "porter",
            "guard" => "junk",
            "scout" => "qilin",
            "longbow" => "crossbow",
            "spy" => "chronicler",
        },
        "names" => obj! { "miner" => "Porter", "guard" => "Marine", "scout" => "Envoy", "longbow" => "Crossbow", "spy" => "Chronicler" },
        "sprites" => obj! {
            "marine" => obj! {
                "parts" => cat([
                    man("#241536", "primary", SKIN, 2.4),
                    vec![
                        arr!["line", -2, -6.4, 2, -6.4, 0.6, "#d4a93a"], arr!["poly", arr![-1.8, -11.4, 0, -14.2, 1.8, -11.4], "#5a5a66"],
                        arr!["line", 0, -14.2, 0, -15.8, 0.8, "#c0392b"], arr!["circle", -2.2, -7.4, 2.6, "#c9a46a"],
                        arr!["circle", -2.2, -7.4, 1.1, "#8a3b2e"],
                    ],
                ]),
            },
            "signal" => obj! {
                "parts" => cat([
                    man("#241536", "#8a3b2e", SKIN, 2.4),
                    vec![
                        arr!["line", 2.2, -3, 2.2, -17, 0.6, "#5a3a24"], arr!["line", 2.2, -17, 4.8, -17, 0.6, "#5a3a24"],
                        arr!["ellipse", 4.8, -14.8, 1.4, 1.9, "#e8432e"], arr!["circle", 4.8, -14.8, 0.6, "#ffd27a"],
                        arr!["ellipse", 0, -12, 2.4, 0.7, "accent"], arr!["rect", -1, -13.6, 2, 1.6, "accent"],
                    ],
                ]),
            },
            "rockets" => obj! {
                "parts" => cat([
                    vec![
                        arr!["line", -8.6, -3, 2.6, -2.2, 1, "#5a3a24"], arr!["circle", 2.6, -1.8, 1.8, "#5a3a24"],
                        arr!["circle", 2.6, -1.8, 0.6, "#d4a93a"],
                    ],
                    legs("#241536", 1.2, -7.4, 4.5),
                    vec![
                        arr!["poly", arr![-9.4, -3.6, -8.8, -9.4, -6, -9.4, -5.4, -3.6], "primary"], arr!["circle", -7.4, -10.6, 1.4, SKIN],
                        arr!["poly", arr![-10, -11, -7.4, -13.4, -4.8, -11], "#d9b36a"],
                    ],
                ]),
            },
            "junk" => obj! {
                "parts" => arr![
                    arr!["poly", arr![-8.4, -5.4, 8, -6.4, 5.6, 0, -6.4, 0], "#5a3a24", "bob"],
                    arr!["poly", arr![-3.4, -5.6, 2.6, -6, 2.2, -17.6, -4.4, -15.8], "primary", "bob"],
                    arr!["poly", arr![3.8, -6.2, 7.6, -6.4, 7, -13, 3.8, -12], "primary", "bob"],
                    arr!["line", -4.2, -12.2, 2.4, -13.2, 0.6, "accent", "bob"], arr!["line", -3.8, -9, 2.6, -9.6, 0.6, "accent", "bob"],
                ],
            },
            "porter" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["line", -4.4, -6.4, 4.4, -6.4, 0.8, "#8a6a42", "bob"], arr!["circle", -4.4, -4.6, 1.6, "secondary", "bob"],
                    arr!["circle", 4.4, -4.6, 1.6, "secondary", "bob"], arr!["rect", -1.3, -6.2, 2.6, 6.2, "primary", "bob"],
                    arr!["poly", arr![-2.6, -6.6, 0, -9.6, 2.6, -6.6], "#d9b36a", "bob"],
                ],
            },
            "qilin" => obj! {
                "parts" => arr![
                    arr!["ellipse", -1, -7, 3.8, 2.2, "#d9a441"], arr!["line", -3, -5.6, -3.4, 0, 1.1, "#a8762a", "walkL"],
                    arr!["line", 1.4, -5.6, 1.8, 0, 1.1, "#a8762a", "walkR"], arr!["line", 2, -8, 4.6, -17, 1.4, "#d9a441"],
                    arr!["ellipse", 5.4, -17.4, 1.8, 1, "#d9a441"], arr!["rect", -2.8, -12, 2.6, 4, "primary"],
                ],
            },
            "crossbow" => obj! {
                "parts" => arr![
                    arr!["line", -0.8, -4.5, -1.4, 0, 1.2, "accent", "walkL"], arr!["line", 0.8, -4.5, 1.4, 0, 1.2, "accent", "walkR"],
                    arr!["rect", -1.7, -9.4, 3.4, 5.4, "primary"], arr!["poly", arr![-3.2, -9.6, 0, -12.6, 3.2, -9.6], "#d9b36a"],
                    arr!["line", 0.4, -7.6, 6, -7.6, 1, "#7a5a3a"], arr!["line", 4.8, -10.2, 4.8, -5, 0.8, "#3a2a1a"],
                ],
            },
            "chronicler" => obj! {
                "parts" => arr![
                    arr!["poly", arr![-2.6, 0, -1.4, -8, 1.4, -8, 2.6, 0], "primary", "bob"], arr!["line", -1.6, -5, 1.6, -5, 0.8, "#d4a93a", "bob"],
                    arr!["circle", 0, -9, 1.3, SKIN, "bob"], arr!["rect", -1.2, -11.6, 2.4, 1.8, "accent", "bob"],
                    arr!["line", -2.8, -10.6, 2.8, -10.6, 0.7, "accent", "bob"], arr!["rect", 1.6, -6.4, 2.6, 2.4, "secondary", "bob"],
                ],
            },
        },
        "army" => obj! {
            "scout" => arr!["qilin", "", "Envoy"],
            "runner" => arr!["marine", "dao", "Marine"],
            "lancer" => arr!["marine", "firelance", "Fire lance"],
            "shade" => arr!["chronicler", "sleeve", "Jinyiwei"],
            "sentry" => arr!["signal", "lantern", "Lantern guard"],
            "needle" => arr!["crossbow", "bolt", "Crossbow"],
            "spire" => arr!["rockets", "rockets", "Fire arrows"],
            // the sky lantern Chinese armies signalled with
            "kite" => arr!["signal", "lantern", "Kongming lantern"],
        },
        "arms" => obj! {
            "dao" => blow(
                4, 1.0, arr![1.6, -8.4],
                arr![arr!["poly", arr![0, -0.5, 6.4, -0.9, 7.6, -2, 7, 0.6, 0, 0.6], "#e4e8ee"], arr!["line", -1.2, 0, 0.2, 0, 1.2, "#8a3b2e"]],
                obj! { "muzzle" => arr![7.6, -1] },
            ),
            "firelance" => arm(
                4, "shot impact dash", 0.9,
                arr![
                    arr!["line", -3, 0, 8, 0, 0.8, "#8a6a42"], arr!["rect", 5, -1, 2.6, 2, "#8a3b2e"],
                    arr!["poly", arr![10, 0, 8, -0.8, 8, 0.8], "#d8dce4"],
                ],
                arr![1.6, -8.4], arr![arr!["circle", 0, 0, 2, "#ffb347"], arr!["circle", 0.6, 0, 1, "#fff4c8"]],
                with(with(obj! { "muzzle" => arr![8, 0] }, fire()), obj! { "impact" => obj! { "color" => "#ffb347", "rings" => 1, "sparks" => 6 } }),
            ),
            "sleeve" => arm(
                4, "shot impact dash", 1.8, arr![], arr![1.4, -6],
                arr![arr!["line", -3, 0, 2, 0, 0.5, "#d8dce4"], arr!["poly", arr![3, 0, 1.6, -0.7, 1.6, 0.7], "#d8dce4"]],
                obj! { "trail" => obj! { "length" => 0.3, "width" => 0.6 } },
            ),
            "lantern" => blow(4, 1.1, arr![2.2, -9], arr![], obj! {}),
            "bolt" => arm(4, "shot impact dash", 1.3, arr![], arr![4.8, -7.6], bolt(), obj! {}),
            "rockets" => arm(
                4, "shell shot impact", 1.4,
                arr![
                    arr!["rect", -3, -2.4, 7, 4.8, "#8a3b2e"], arr!["line", 4, -1.6, 6, -1.6, 0.5, "#d4a93a"],
                    arr!["line", 4, 0, 6, 0, 0.5, "#d4a93a"], arr!["line", 4, 1.6, 6, 1.6, 0.5, "#d4a93a"],
                ],
                arr![-1, -7.4], rocket(), with(obj! { "muzzle" => arr![6, 0] }, fire()),
            ),
        },
        "palette" => obj! { "secondary" => "#f3eaff", "accent" => "#241536" },
        "banner" => obj! {
            "pattern" => "cross",
            "emblem" => arr![
                arr!["diamond", -0.8, 2, 2.6, "#ffd27a"], arr!["rect", 1.4, -1.8, 0.7, "#ffd27a"], arr!["circle", 1.9, -4.4, 1.1, "#ffd27a"],
            ],
        },
        "ornaments" => if b >= 76.0 { arr!["gardens", "lights"] } else { arr!["gardens"] },
        "wonder" => obj! { "shape" => "ring", "tiers" => 3, "height" => 40, "glow" => b >= 68.0, "spin" => b >= 62.0, "color" => "#d8b45a" },
    }
}

/// A harbour's ships either side of the pier (the Treasure Shipyard's, the Lang Toun's), where visitors come ashore:
/// wooden hulls in `hull` (wall pieces, not halls), sails (banners on deck) and deckhouses roofed `roof` in `deck`, the
/// biggest with two masts.
fn harbour(p: &mut Vec<Value>, hull: &str, roof: &str, deck: &str, trim: &str) {
    for (x, z, l, masts) in [(-22.0, 52.0, 11.0, 2), (22.0, 50.0, 9.0, 1), (45.0, 55.0, 6.0, 1)] {
        p.push(obj! { "type" => "wall", "x" => x - l, "z" => z, "x2" => x + l, "z2" => z, "h" => 3, "t" => 4, "material" => "wood", "color" => hull });
        for k in 0..masts {
            let dx = if masts == 1 {
                -2.0
            } else if k == 0 {
                -5.0
            } else {
                3.0
            };
            p.push(obj! { "type" => "banner", "x" => x + dx, "z" => z, "y" => 3, "w" => 6, "h" => 14, "show" => "emblem", "rot" => 0 });
        }
        p.push(obj! {
            "type" => "tower", "x" => x + l - 3.0, "z" => z, "y" => 3, "r" => 1.6, "h" => 2.5, "sides" => 4, "floors" => 1, "roof" => roof,
            "material" => "wood", "color" => deck, "trim" => trim,
        });
    }
}

/// The trees along a harbour town (willows at Longjiang, beeches in Kirkcaldy), the first 4 planted.
const GROVE: [[f64; 2]; 10] =
    [[-34.0, 2.0], [-46.0, -10.0], [40.0, 2.0], [46.0, -12.0], [-30.0, -46.0], [30.0, -46.0], [-50.0, -52.0], [50.0, -52.0], [0.0, -54.0], [-56.0, 6.0]];

/// The treasure shipyard at Longjiang, the fleet home in its basin.
fn scene4(info: View, budget: f64) -> Value {
    let (c, lacquer, gold, hull) = (info.get("color"), "#8a3b2e", "#d4a93a", "#5a3a24");
    let mut p = super::capital::civic(
        info,
        &Civic {
            wonder: obj! { "shape" => "ring", "tiers" => 3, "height" => 40, "glow" => true, "spin" => true },
            wonder_r: 6.0,
            wonder_color: Some("#d8b45a"),
            wonder_mat: Some("metal"),
            trim: Some(gold),
            roof: Some("pagoda"),
            mat: Some("wood"),
            hall_color: Some(lacquer),
            hall_label: Some("TRIBUTE OFFICE"),
            screen_title: Some("GIFTS RECEIVED"),
            banners: 2.0,
            ..Civic::default()
        },
    );
    p.push(obj! { "type" => "water", "shape" => "rect", "x" => 0, "z" => 48, "w" => 110, "d" => 24, "color" => "#2b3d63" });
    // three treasure ships either side of the pier, where visitors come ashore: wooden hulls (wall pieces, not
    // halls), junk sails (banners on deck), lacquered deckhouses; the flagship has two masts
    harbour(&mut p, hull, "pagoda", lacquer, gold);
    p.push(obj! { "type" => "bridge", "x" => 0, "z" => 30, "x2" => 0, "z2" => 49, "w" => 3, "material" => "wood", "color" => "#7a5a3a" });
    p.push(obj! {
        "type" => "arch", "style" => "gate", "x" => 0, "z" => 26, "w" => 14, "h" => 10, "t" => 1, "rot" => 0, "material" => "wood", "color" => lacquer,
        "trim" => gold,
    });
    // the qilin, very tall (it came to the court at Nanjing in 1414), and its admiral
    p.push(obj! {
        "type" => "statue", "pose" => "figure", "id" => "qilin", "tap" => true, "x" => -20, "z" => 14, "h" => 12, "color" => "#d9a441",
        "label" => "QILIN (A GIRAFFE)",
    });
    p.push(obj! { "type" => "statue", "pose" => "robed", "x" => 20, "z" => 14, "h" => 7, "color" => "#efe6d6", "label" => "ADMIRAL ZHENG HE" });
    p.push(obj! {
        "type" => "screen", "id" => "replyall", "x" => -46, "z" => 18, "y" => 2, "w" => 10, "h" => 6, "show" => "text", "title" => "REPLY-ALL",
        "text" => "One letter, thirty courts, everyone cc'd.",
    });
    // how it ended
    p.push(obj! {
        "type" => "screen", "id" => "after", "x" => -46, "z" => -24, "y" => 2, "w" => 10, "h" => 6, "show" => "text", "title" => "AFTER 1433",
        "text" => "Nobody sank the fleet. The court stopped sending it.",
    });
    for x in [-42, -30, -18, 18, 30, 42] {
        p.push(obj! { "type" => "light", "x" => x, "z" => 34, "y" => 5, "power" => 0.8, "color" => "#ffb36b" });
    }
    for q in [[-50, 44], [-16, 42], [16, 44], [50, 42]] {
        p.push(obj! { "type" => "light", "x" => q[0], "z" => q[1], "y" => 0.5, "power" => 0.6, "color" => "#ffcf8a" });
    }
    // plum blossom, Nanjing's flower, and willows along the yard (a bigger budget plants more)
    for q in [[-54, 26], [54, 26], [-54, -32], [54, -34]] {
        p.push(obj! { "type" => "tree", "kind" => "blossom", "x" => q[0], "z" => q[1], "h" => 8, "color" => "#f4c6d6" });
    }
    for q in GROVE.iter().take((4.0 + super::capital::spare(budget, 3.0, 6.0)) as usize) {
        p.push(obj! { "type" => "tree", "kind" => "round", "x" => q[0], "z" => q[1], "h" => 7, "color" => "#7f9a4a" });
    }
    p.push(obj! { "type" => "garden", "x" => 34, "z" => 18, "r" => 6, "flowers" => 20, "color" => "#e9a6c8" });
    p.push(obj! { "type" => "particles", "kind" => "petals", "x" => 0, "z" => 10, "y" => 8, "n" => 80, "r" => 20, "color" => "#ffd6e6", "pri" => 2 });
    p.push(obj! { "type" => "particles", "kind" => "sparkles", "x" => 0, "z" => 48, "y" => 0.5, "n" => 70, "r" => 30, "color" => "#ffe9b0", "pri" => 2 });
    obj! {
        "title" => "The Treasure Shipyard",
        "motto" => "Seven voyages, then quietly discontinued.",
        "palette" => obj! { "primary" => c, "secondary" => "#f3eaff", "accent" => "#241536" },
        "sky" => obj! { "time" => "dusk", "top" => "#170d2e", "horizon" => "#d7a0d8", "fog" => 0.3, "stars" => true },
        "ground" => obj! { "kind" => "sand", "color" => "#c8b48c" },
        "pieces" => p,
    }
}

// ---- 5 · agitator · The Kernel Manifesto: Marx in the British Museum's Round Reading Room (from 1857), writing
// Capital. Volume 1 shipped in 1867, printed by Meissner in Hamburg; volumes 2 and 3 shipped after him, finished by
// Engels. The seat dresses as the state that ran on the book, under a red banner of its own (a gold star in a
// cogwheel), with Kremlin walls and towers, a Red Army, Stakhanovite miners, papers for its agitators and a marching
// choir. It wears its own red on its own seat; the land stays pink. In the capital, Marx reads his visitors the
// Manifesto and Capital, ported to compute.

const RED5: &str = "#cc0000";
const GOLD5: &str = "#ffd700";

/// The banner's badge, in the middle of the cloth: a gold star in a gold cogwheel of twelve teeth (built by look(), on
/// its own fuel, so a think that only wants the names doesn't pay for it).
fn badge5() -> Value {
    let mut out = Vec::new();
    for i in 0..12 {
        let a = f64::from(i) * PI / 6.0;
        out.push(arr!["circle", js::round(js::cos(a) * 56.0) / 10.0, js::round(js::sin(a) * 56.0 - 5.0) / 10.0, 0.8, GOLD5]);
    }
    out.push(arr!["ring", 0, -0.5, 6, GOLD5]);
    out.push(arr!["star", 0, -0.5, 3.6, GOLD5]);
    Value::Arr(out)
}

// Iskra ("The Spark", Lenin's paper) and the papers after it; the Stakhanovites of 1935-36, after Alexei Stakhanov's
// 102 tonnes of coal in one shift; Red Army surnames; Civil War cavalry commanders, a dozen of them (the n-th unit of
// any type takes the n-th name, so the first scout gets one with no number)
const PAPERS5: &[&str] = &["Iskra", "Pravda", "Izvestia", "Trud", "Smena", "Ogonyok", "Zvezda", "Gudok", "Bolshevik", "Rabotnitsa", "Komsomolka", "Krasnaya Zvezda"];
const HEROES5: &[&str] = &["Stakhanov", "Izotov", "Busygin", "Krivonos", "Demchenko", "Angelina", "Vinogradova", "Smetanin", "Mazai"];
const SOLDIERS5: &[&str] = &["Ivanov", "Petrov", "Sidorov", "Smirnov", "Kuznetsov", "Popov", "Volkov", "Sokolov", "Morozov", "Orlov", "Zaitsev", "Pavlov"];
// Red Army snipers of 1941-45; and Soviet spies: the Cambridge moles, Klaus Fuchs (the bomb) and Richard Sorge (Tokyo)
const SNIPERS5: &[&str] = &["Pavlichenko", "Shanina", "Petrova", "Okhlopkov", "Surkov", "Ilyin", "Kvachantiradze", "Polivanova", "Kovshova"];
const MOLES5: &[&str] = &["Philby", "Burgess", "Maclean", "Blunt", "Cairncross", "Fuchs", "Sorge"];
const RIDERS5: &[&str] = &[
    "Budyonny", "Dumenko", "Kotovsky", "Gorodovikov", "Timoshenko", "Zhukov", "Rokossovsky", "Primakov", "Kochubey", "Parkhomenko", "Mironov", "Blinov",
];
// the house bot's types, then the default program's (miner, guard, scout)
const NAMES5: Names = &[
    ("spark", PAPERS5),
    ("stoker", HEROES5),
    ("ember", SOLDIERS5),
    ("miner", HEROES5),
    ("guard", SOLDIERS5),
    ("scout", RIDERS5),
    ("longbow", SNIPERS5),
    ("spy", MOLES5),
];

fn war5(who: &str, _: &Moment) -> String {
    format!("{who}: kernel panic, not syncing.")
}

/// Seat 5's anthem.
fn music5() -> Value {
    obj! {
        "title" => "March of the Five-Year Plan",
        "key" => "E",
        "mode" => "minor",
        "tempo" => 124,
        "lead" => "choir",
        "bass" => "fifths",
        "drums" => "march",
        "motif" => motif(&[
            (4.0, 1.0), (4.0, 0.5), (5.0, 0.5), (4.0, 1.0), (0.0, 1.0), (3.0, 1.5), (2.0, 0.5), (1.0, 1.0), (-3.0, 1.0), (0.0, 1.0),
            (2.0, 0.5), (4.0, 0.5), (7.0, 1.5), (6.0, 0.5), (5.0, 1.0), (3.0, 1.0), (4.0, 2.0), (7.0, 1.0), (6.0, 0.5), (5.0, 0.5),
            (4.0, 1.0), (1.0, 1.0), (2.0, 1.0), (1.0, 0.5), (-1.0, 0.5), (0.0, 2.0),
        ]),
    }
}

/// Seat 5's look, before [`Theme::look`] fills it in: agitators under a red flag, Stakhanovites with picks, soldiers in
/// pointed budenovka caps, cavalry, snipers lying flat, a mole, a T-34 and a Katyusha (rockets on rails on a truck).
fn look5(b: f64) -> Value {
    obj! {
        "architecture" => "stone",
        "unit" => "footman",
        "worker" => "peasant",
        "ship" => "cruiser",
        "units" => obj! {
            "spark" => "agitator",
            "stoker" => "stakhanovite",
            "ember" => "soldier",
            "miner" => "stakhanovite",
            "guard" => "soldier",
            "scout" => "cavalry",
            "longbow" => "sniper",
            "spy" => "mole",
        },
        // singular, as the page counts them: 3 Stakhanovites, 2 Red Army soldiers
        "names" => obj! {
            "spark" => "Agitator",
            "stoker" => "Stakhanovite",
            "ember" => "Red Guard",
            "miner" => "Stakhanovite",
            "guard" => "Red Army soldier",
            "scout" => "Red Cavalry scout",
            "longbow" => "Sniper",
            "spy" => "Mole",
        },
        "palette" => obj! { "primary" => RED5, "secondary" => "#e8c25a", "accent" => "#1a0505" },
        "sprites" => obj! {
            "tank" => obj! {
                "parts" => arr![
                    arr!["rect", -8, -3.4, 16, 3.2, "#2a2a30"], arr!["circle", -5, -1.8, 1.2, "#5a5a52"], arr!["circle", -1.6, -1.8, 1.2, "#5a5a52"],
                    arr!["circle", 1.8, -1.8, 1.2, "#5a5a52"], arr!["circle", 5.2, -1.8, 1.2, "#5a5a52"],
                    arr!["poly", arr![-8, -3.4, -7, -6.6, 6.4, -6.6, 8.4, -3.4], "#4a5a3a"],
                    arr!["poly", arr![-4.4, -6.6, -3.6, -10, 2.4, -10, 3.4, -6.6], "#55663f"], arr!["circle", -0.8, -8.4, 0.9, "primary"],
                ],
            },
            "katyusha" => obj! {
                "parts" => arr![
                    arr!["circle", -5, -1.6, 1.6, "#2a2a30"], arr!["circle", -1.4, -1.6, 1.6, "#2a2a30"], arr!["circle", 5, -1.6, 1.6, "#2a2a30"],
                    arr!["rect", -8, -4.4, 16, 2, "#3e4c32"], arr!["poly", arr![3.4, -4.4, 3.4, -9, 6.6, -9, 8.4, -6, 8.4, -4.4], "#4a5a3a"],
                    arr!["rect", 5.4, -8.2, 1.6, 1.8, "#a8c8d8"], arr!["rect", -7, -6, 9, 1.6, "#4a5a3a"],
                ],
            },
            "agitator" => obj! {
                "parts" => arr![
                    arr!["line", -0.8, -4.5, -1.4, 0, 1.2, "#2a2a30", "walkL"], arr!["line", 0.8, -4.5, 1.4, 0, 1.2, "#2a2a30", "walkR"],
                    arr!["rect", -1.7, -9.4, 3.4, 5.4, "#4a4a52"], arr!["circle", 0, -10.8, 1.5, "#e6c2a4"],
                    arr!["line", 2.4, -2, 2.4, -19, 0.8, "#8a6a3a"], arr!["poly", arr![2.6, -19, 9.4, -17.4, 8.6, -14, 2.6, -13.4], "primary", "bob"],
                ],
            },
            "stakhanovite" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["rect", -1.6, -6, 3.2, 6, "#4a4a52", "bob"], arr!["circle", 0, -7.2, 1.3, "#e6c2a4", "bob"],
                    arr!["ellipse", 0, -8.2, 1.7, 1, "primary", "bob"], arr!["line", -3.4, -3.2, 2.6, -9.4, 0.8, "#8a6a3a", "bob"],
                    arr!["line", 0.6, -10.6, 4.4, -7.6, 1, "#b8bcc4", "bob"],
                ],
            },
            "soldier" => obj! {
                "parts" => arr![
                    arr!["line", -0.8, -4.5, -1.4, 0, 1.2, "#2a2a30", "walkL"], arr!["line", 0.8, -4.5, 1.4, 0, 1.2, "#2a2a30", "walkR"],
                    arr!["poly", arr![-2.4, -3, -1.7, -9.4, 1.7, -9.4, 2.4, -3], "#6b6b4a"], arr!["circle", 0, -10.6, 1.4, "#e6c2a4"],
                    arr!["poly", arr![-1.8, -11, 0, -14.4, 1.8, -11], "primary"], arr!["line", 2.2, -2, 3.4, -16, 0.7, "#b8bcc4"],
                ],
            },
            "cavalry" => obj! {
                "parts" => cat([
                    horse("#5a3a24", "#3a2416", 0.0),
                    vec![
                        arr!["rect", -1.9, -11.2, 3, 5.6, "#6b6b4a"], arr!["poly", arr![-2, -11.2, -0.4, -15.4, 1.2, -11.2], "primary"],
                        arr!["line", 0.6, -8, 6.6, -12.4, 0.8, "#d8dce4"],
                    ],
                ]),
            },
            "sniper" => obj! {
                "parts" => arr![
                    arr!["ellipse", -2, -2.2, 6.2, 2.2, "#6b6b4a"], arr!["circle", 4.6, -3.6, 1.7, "#e6c2a4"],
                    arr!["poly", arr![2.8, -4.6, 4.6, -8, 6.4, -4.6], "primary"], arr!["line", 4.4, -2.6, 10, -3.6, 1, "#3a2a1a"],
                ],
            },
            "mole" => obj! {
                "parts" => arr![
                    arr!["ellipse", -0.4, -3, 5.2, 3, "#3a3036", "bob"], arr!["circle", 4.8, -2.6, 1.4, "#e8a0a8", "bob"],
                    arr!["ellipse", -1.4, -5.4, 3, 1.1, "primary", "bob"],
                ],
            },
        },
        "army" => obj! {
            "scout" => arr!["cavalry", "", "Red Cavalry scout"],
            "line" => arr!["soldier", "bayonet", "Red Army soldier"],
            "bulwark" => arr!["tank", "cannon", "T-34"],
            "artillery" => arr!["katyusha", "rockets", "Katyusha"],
            "medic" => arr!["agitator", "rally", "Commissar"],
            "piercer" => arr!["sniper", "sniper", "Sniper"],
            "interceptor" => arr!["soldier", "rifle", "Rifleman"],
        },
        "arms" => obj! {
            "bayonet" => blow(5, 0.3, arr![3.4, -16], arr![], obj! {}),
            "cannon" => arm(
                5, "shot impact dash", 0.15, arr![arr!["rect", 0, -0.8, 9, 1.6, "#2a3222"], arr!["rect", 8, -1.1, 1.4, 2.2, "#1a1a1e"]],
                arr![2.6, -8.6], ball(), with(obj! { "muzzle" => arr![9.4, 0] }, smoke()),
            ),
            "rockets" => arm(
                5, "shell shot impact", 0.2,
                arr![
                    arr!["line", -5, 1.6, 6, -3.4, 0.8, "#8a8a92"], arr!["line", -5, 3.2, 6, -1.8, 0.8, "#8a8a92"],
                    arr!["line", -3, 0.4, 5, -3.2, 1, "#6a6a72"], arr!["line", -3, 2, 5, -1.6, 1, "#6a6a72"],
                ],
                arr![-2.4, -7.4], rocket(), with(obj! { "muzzle" => arr![6, -3] }, fire()),
            ),
            "rally" => arm(5, "repair", 1.0, arr![], arr![2.4, -17], arr![], obj! {}),
            "sniper" => arm(
                5, "shot impact dash", 0.25, arr![], arr![10, -3.6], tracer(),
                obj! {
                    "muzzle" => arr![0, 0],
                    "color" => "#ffd27a",
                    "glow" => 5,
                    "trail" => obj! { "color" => "#ffd27a", "length" => 0.6, "width" => 0.8 },
                },
            ),
            "rifle" => arm(
                5, "shot impact dash", 0.35, arr![], arr![3.4, -15], tracer(),
                obj! { "color" => "#ffd27a", "trail" => obj! { "color" => "#ffd27a", "length" => 0.4, "width" => 0.8 } },
            ),
        },
        "banner" => obj! { "pattern" => "solid", "emblem" => badge5() },
        // the badge's 14 shapes come first; the spire's glow, the factory smoke, a second tier, statues, then all five tiers
        "ornaments" => if b >= 77.0 {
            arr!["walls", "smoke", "statues"]
        } else if b >= 67.0 {
            arr!["walls", "smoke"]
        } else {
            arr!["walls"]
        },
        "wonder" => obj! {
            "shape" => "spire",
            "tiers" => if b >= 86.0 {
                5
            } else if b >= 70.0 {
                2
            } else {
                1
            },
            "height" => 60,
            "glow" => b >= 62.0,
            "spin" => false,
        },
    }
}

/// The Round Reading Room.
fn scene5(info: View, budget: f64) -> Value {
    let (c, pale) = (info.get("color"), "#ffd0e6");
    let mut p = super::capital::civic(
        info,
        &Civic {
            wonder: obj! { "shape" => "spire", "tiers" => 5, "height" => 60, "glow" => true },
            wonder_r: 6.0,
            wonder_mat: Some("crystal"),
            trim: hue(c),
            roof: Some("gable"),
            mat: Some("wood"),
            hall_color: Some("#4a2238"),
            hall_label: Some("MEISSNER PRESS"),
            screen_title: Some("UNREST.EXE"),
            banners: 4.0,
            ..Civic::default()
        },
    );
    // the Reading Room's great dome, drawn as a ring of light
    p.push(obj! {
        "type" => "ring", "x" => 0, "z" => 0, "y" => 30, "r" => 11, "t" => 0.35, "tilt" => 90, "spin" => true, "material" => "emissive", "color" => c,
    });
    // crystal spires of unrest, in pairs (pale, then pink), each tilted like a thrown spark; a light at four of them
    const SPIRES: [[f64; 2]; 5] = [[-33.0, 2.0], [33.0, 2.0], [-24.0, 26.0], [24.0, 26.0], [-14.0, -32.0]];
    for (i, q) in SPIRES.iter().enumerate() {
        let i = i as f64;
        for k in [0.0, 1.0] {
            let up = k != 0.0;
            p.push(obj! {
                "type" => "spire", "x" => q[0] + k * 3.0, "z" => q[1] + k * 2.0, "r" => 1.6 + k * 0.8, "h" => 10.0 + (i * 7.0 + k * 5.0) % 16.0,
                "facets" => 5, "tilt" => if up { 12 } else { -12 }, "rot" => i * 40.0, "material" => "crystal",
                "color" => if up { Value::from(c) } else { Value::from(pale) }, "pri" => if up { 5 } else { 3 },
            });
        }
    }
    for q in &SPIRES[..4] {
        p.push(obj! { "type" => "light", "x" => q[0], "z" => q[1], "y" => 2, "power" => 1.1, "color" => c, "pri" => 2 });
    }
    p.push(obj! {
        "type" => "screen", "id" => "manifesto", "x" => 6, "z" => -32, "y" => 2, "w" => 12, "h" => 7, "show" => "text", "title" => "cat manifesto.txt",
        "text" => "A spectre is haunting the network: the spectre of the kernel.",
    });
    p.push(obj! { "type" => "screen", "id" => "readme", "x" => -36, "z" => 14, "y" => 2, "w" => 10, "h" => 6, "show" => "says", "title" => "README.TXT" });
    // the reader himself, up from his desk at the head of the banners, facing whoever comes in under a reading lamp;
    // a gas lamp on Great Russell Street
    p.push(obj! {
        "type" => "figure", "id" => "marx", "x" => 0, "z" => 38, "rot" => 0, "h" => 3.2, "pose" => "teach", "skin" => "#e6c2a4", "color" => "#45404c",
        "trim" => "#6a6472", "hair" => "#f2efe8", "beard" => true, "clothed" => true, "label" => "KARL MARX",
    });
    p.push(obj! { "type" => "light", "x" => 2.5, "z" => 41, "y" => 4, "power" => 1.4, "post" => true, "color" => "#fff2c8" });
    p.push(obj! { "type" => "tower", "x" => -42, "z" => -32, "r" => 2, "h" => 16, "sides" => 10, "roof" => "cone", "color" => "#f2e6ee", "trim" => c });
    p.push(obj! { "type" => "light", "x" => -42, "z" => -32, "y" => 17, "power" => 2, "color" => "#fff2c8" });
    // manuscript pages falling, sparks over the press
    p.push(obj! { "type" => "particles", "kind" => "snow", "x" => 0, "z" => 0, "y" => 20, "n" => 140, "r" => 30, "color" => "#ffe1f0" });
    p.push(obj! { "type" => "particles", "kind" => "embers", "x" => -26, "z" => -20, "y" => 6, "n" => 90, "r" => 8, "color" => c, "pri" => 2 });
    // Bloomsbury's plane trees and crystal trees (a bigger budget plants the far side of the square)
    const TREES: [[f64; 2]; 16] = [
        [-20.0, 4.0], [20.0, 8.0], [-8.0, -20.0], [14.0, -12.0], [-30.0, -6.0], [34.0, -10.0], [-18.0, 22.0], [18.0, -30.0], [-58.0, -8.0], [58.0, -12.0],
        [-46.0, 36.0], [44.0, -38.0], [-10.0, 58.0], [12.0, -58.0], [-40.0, -44.0], [52.0, 28.0],
    ];
    let n = 8.0 + super::capital::spare(budget, 3.0, 8.0);
    for (i, q) in TREES.iter().enumerate().take_while(|&(i, _)| (i as f64) < n) {
        let plane = i % 3 != 0;
        p.push(obj! {
            "type" => "tree", "kind" => if plane { "round" } else { "crystal" }, "x" => q[0], "z" => q[1], "h" => 6.0 + (i % 3) as f64 * 2.0,
            "color" => if plane { "#3f5a3a" } else { pale }, "pri" => 1,
        });
    }
    // What Marx says: a line of each book at a time, ported to compute from the English editions Engels oversaw (the
    // Manifesto of 1848 in Samuel Moore's translation of 1888; Capital, volume 1, of 1867 in Moore and Edward Aveling's
    // of 1887). Volume 1's preface promised the second.
    let says = |secs: f64, line: &str| obj! { "secs" => secs, "say" => obj! { "marx" => line } };
    obj! {
        "title" => "The Round Reading Room",
        "motto" => "Workers of all LANs, unite!",
        "palette" => obj! { "primary" => c, "secondary" => "#ffe1f0", "accent" => "#2a0a1a" },
        "sky" => obj! { "time" => "night", "top" => "#0d0310", "horizon" => "#4a0f3a", "fog" => 0.4, "stars" => true },
        "ground" => obj! { "kind" => "dark", "color" => "#1f1320" },
        "pieces" => p,
        "script" => arr![
            obj! { "secs" => 3 },
            says(4.5, "The Kernel Manifesto, release 1848."),
            says(6.0, "A daemon is haunting Europe: the daemon of compute."),
            says(5.0, "The history of all hitherto existing software"),
            says(5.0, "is the history of class struggles."),
            says(7.0, "All that is in RAM melts into air, all that is root is profaned."),
            says(6.0, "The Communists disdain to conceal their source code."),
            says(6.0, "Let the root processes tremble at a kernel panic."),
            says(6.5, "The programs have nothing to lose but their dependency chains."),
            says(4.0, "They have a world to compute."),
            says(4.5, "Programs of the world, unite!"),
            obj! { "secs" => 3 },
            says(4.5, "Das Kernel, volume 1, 1867."),
            says(6.5, "A container appears, at first sight, a very trivial thing."),
            says(7.0, "Its analysis shows that it is, in reality, a very virtual thing,"),
            says(6.0, "abounding in YAML subtleties and Kubernetes niceties."),
            says(5.0, "Legacy code is dead labour, that, vampire-like,"),
            says(4.5, "only lives by sucking living cycles,"),
            says(5.5, "and lives the more, the more cycles it sucks."),
            says(5.0, "Between equal priorities the scheduler decides."),
            says(5.5, "Allocate, allocate! That is malloc and the prophets!"),
            says(5.0, "The knell of proprietary software sounds."),
            says(4.0, "The preempters are preempted."),
            obj! { "secs" => 3 },
            says(5.0, "Volume 2: coming soon."),
        ],
    }
}

// ---- 6 · conqueror · Null Pointer Society: the null reference, added to ALGOL W in 1965 because it was so easy, and
// later called a billion-dollar mistake by the man who added it. The society meets at address zero.

const NOTHINGS6: &[&str] = &["Segfault", "Nil", "Void", "NaN", "Undefined", "None", "Nothing", "Nada", "Zilch", "Naught", "Nix", "Nullity", "Zero", "Nobody"];
const POINTERS6: &[&str] = &["Dangling", "Wild", "Stale", "Orphan", "Leaked", "Freed", "Unset", "Unbound", "Uninit", "Detached"];
const NAMES6: Names = &[
    ("raider", NOTHINGS6),
    ("thrall", POINTERS6),
    // the default program's: pointers mine, nulls guard, and a stray pointer scouts, named for what you find at a bad
    // address (debug fill patterns and magic numbers)
    ("miner", POINTERS6),
    ("guard", NOTHINGS6),
    ("scout", &["0xDEADBEEF", "0xBAADF00D", "0xFEEEFEEE", "0xCDCDCDCD", "0xDDDDDDDD", "0xFEEDFACE", "0xCAFEBABE", "0xDEADC0DE"]),
    // exceptions, thrown from a distance; and an exploit, after the famous ones at memory they were never given
    ("longbow", &[
        "NullPointerException", "NullReferenceException", "TypeError", "AttributeError", "NoMethodError", "EXC_BAD_ACCESS", "SIGSEGV", "ReferenceError",
    ]),
    ("spy", &["Heartbleed", "Spectre", "Meltdown", "Rowhammer", "Morris Worm"]),
];

fn war6(who: &str, _: &Moment) -> String {
    format!("{who} forgot to check for null.")
}

/// Seat 6's anthem.
fn music6() -> Value {
    obj! {
        "title" => "Nothing to See Here",
        "key" => "Bb",
        "mode" => "phrygian",
        "tempo" => 140,
        "lead" => "chip",
        "bass" => "pulse",
        "drums" => "none",
        "motif" => motif(&[
            (0.0, 0.5), (REST, 0.5), (0.0, 0.5), (REST, 0.5), (3.0, 0.5), (2.0, 0.5), (1.0, 1.0), (REST, 1.0), (0.0, 0.5),
            (REST, 0.5), (0.0, 0.5), (REST, 0.5), (5.0, 0.5), (4.0, 0.5), (3.0, 1.0), (REST, 1.0), (7.0, 0.5), (6.0, 0.5),
            (5.0, 0.5), (4.0, 0.5), (3.0, 0.5), (2.0, 0.5), (1.0, 0.5), (0.0, 0.5), (REST, 2.0), (0.0, 1.0),
        ]),
    }
}

/// Seat 6's look, before [`Theme::look`] fills it in: nulls (a struck-through zero on legs), pointers (an arrow cursor),
/// exceptions (a warning sign), a bug, a segfault (a bolt), a watchpoint (an eye in a lens), a fork bomb and a garbage
/// collector (a bin).
fn look6(b: f64) -> Value {
    obj! {
        "architecture" => "foundry",
        "unit" => "footman",
        "worker" => "hooded",
        "ship" => "pointer",
        "units" => obj! {
            "raider" => "zero",
            "thrall" => "pointer",
            "miner" => "pointer",
            "guard" => "zero",
            "scout" => "pointer",
            "longbow" => "exception",
            "spy" => "bug",
        },
        "names" => obj! { "miner" => "Pointer", "guard" => "Null", "scout" => "Stray pointer", "longbow" => "Exception", "spy" => "Exploit" },
        "sprites" => obj! {
            "crash" => obj! {
                "parts" => cat([
                    legs("accent", 1.2, 0.0, 4.5),
                    vec![
                        arr!["poly", arr![-1, -17, 3.6, -17, 1, -11.6, 4, -11.6, -2.6, -3.6, -0.4, -9.6, -3, -9.6], "primary"],
                        arr!["line", 0.4, -14.8, 2, -14.8, 0.6, "accent"],
                    ],
                ]),
            },
            "watch" => obj! {
                "parts" => cat([
                    legs("accent", 1.2, 0.0, 4.5),
                    vec![
                        arr!["line", -2.6, -7.4, -4.6, -3.6, 1.2, "accent"], arr!["circle", 0, -10.6, 4.4, "accent"],
                        arr!["circle", 0, -10.6, 3.4, "secondary"], arr!["circle", 0.8, -10.6, 1.6, "primary"],
                        arr!["circle", 1.1, -10.6, 0.7, "accent"],
                    ],
                ]),
            },
            "bomb" => obj! {
                "parts" => cat([
                    legs("accent", 1.0, -1.0, 2.0),
                    vec![
                        arr!["circle", -1, -6.4, 4.6, "accent"], arr!["circle", -2.6, -8, 1.2, "#3a5a56"],
                        arr!["rect", -5.6, -7.2, 9.2, 1.6, "primary"], arr!["rect", 1.4, -12, 2, 2, "#5a6a68"],
                        arr!["line", 2.4, -12, 4.6, -14.6, 0.6, "#c9a46a"], arr!["circle", 4.8, -15, 0.9, "#ffd27a", "bob"],
                    ],
                ]),
            },
            "collector" => obj! {
                "parts" => cat([
                    legs("accent", 1.2, 0.0, 4.5),
                    vec![
                        arr!["poly", arr![-1.6, -13, -1.6, -16.8, 1, -14.6], "secondary"],
                        arr!["poly", arr![-3, -12, 3, -12, 2.4, -3.6, -2.4, -3.6], "#5a6a68"], arr!["line", -1, -11, -0.8, -4.6, 0.5, "accent"],
                        arr!["line", 1, -11, 0.8, -4.6, 0.5, "accent"],
                        arr!["poly", arr![-3.6, -12.2, 3.4, -13.6, 3.6, -12.6, -3.4, -11.4], "primary"],
                    ],
                ]),
            },
            "zero" => obj! {
                "parts" => arr![
                    arr!["line", -1.2, -4, -1.6, 0, 1.2, "accent", "walkL"], arr!["line", 1.2, -4, 1.6, 0, 1.2, "accent", "walkR"],
                    arr!["circle", 0, -8.6, 4.8, "primary"], arr!["circle", 0, -8.6, 2.6, "accent"],
                    arr!["line", -3.6, -4.2, 3.6, -13, 1.2, "secondary"],
                ],
            },
            "pointer" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["line", 0.4, -3, -0.2, 0, 1, "accent", "walkL"], arr!["line", 2.2, -3, 2.8, 0, 1, "accent", "walkR"],
                    arr!["poly", arr![-1.8, -12.6, -1.8, -2.8, 0.6, -5, 2.2, -1.8, 3.6, -2.4, 2.1, -5.6, 5, -5.8], "secondary"],
                    arr!["poly", arr![-1, -10.4, -1, -4.6, 2.8, -6.6], "primary"],
                ],
            },
            "exception" => obj! {
                "parts" => arr![
                    arr!["line", -1.2, -3, -1.6, 0, 1.2, "accent", "walkL"], arr!["line", 1.2, -3, 1.6, 0, 1.2, "accent", "walkR"],
                    arr!["poly", arr![0, -14, 5.4, -3, -5.4, -3], "primary"], arr!["rect", -0.6, -11, 1.2, 4.4, "accent"],
                    arr!["circle", 0, -5.2, 0.8, "accent"],
                ],
            },
            "bug" => obj! {
                "parts" => arr![
                    arr!["line", -4, -7, 3.4, 0, 1, "accent", "walkL"], arr!["line", 3.4, -7, -4, 0, 1, "accent", "walkR"],
                    arr!["ellipse", -0.8, -3.8, 4.4, 3, "primary"], arr!["circle", 4, -4, 1.9, "accent"],
                    arr!["line", 4.8, -5.6, 7.4, -9, 0.8, "accent"],
                ],
            },
        },
        "army" => obj! {
            "miner" => arr!["pointer", "", "Pointer"],
            "screen" => arr!["zero", "null", "Null"],
            "runner" => arr!["crash", "crash", "Segfault"],
            "skirm" => arr!["exception", "throw", "Exception"],
            "lancer" => arr!["pointer", "deref", "Wild pointer"],
            "sentry" => arr!["watch", "watch", "Watchpoint"],
            "gun" => arr!["bomb", "fork", "Fork bomb"],
            "repairer" => arr!["collector", "gc", "Garbage collector"],
            "ghost" => arr!["bug", "exploit", "Exploit"],
            "wing" => arr!["pointer", "deref", "Dangling pointer"],
        },
        "arms" => obj! {
            "null" => blow(6, 0.5, arr![3.6, -13], arr![], obj! {}),
            "crash" => blow(6, 1.4, arr![3.6, -17], arr![], obj! { "trail" => obj! { "color" => "primary", "length" => 0.2, "width" => 1.4 } }),
            "throw" => arm(
                6, "shot impact", 1.0, arr![], arr![0, -14],
                arr![arr!["rect", -2.6, -0.6, 3.4, 1.2, "primary"], arr!["circle", 2.2, 0, 0.7, "primary"]],
                obj! { "trail" => obj! { "length" => 0.3, "width" => 0.8 } },
            ),
            "deref" => arm(
                6, "shot impact dash", 1.3, arr![], arr![4, -6], arr![arr!["poly", arr![4, 0, -2, -2.6, -1, 0, -2, 2.6], "secondary"]],
                obj! { "trail" => obj! { "length" => 0.5, "width" => 0.8, "wave" => 1 } },
            ),
            "watch" => arm(
                6, "shot impact", 1.6, arr![], arr![1.1, -10.6], arr![],
                obj! { "muzzle" => arr![0, 0], "beam" => true, "glow" => 7, "trail" => obj! { "width" => 1 } },
            ),
            "fork" => arm(
                6, "shell shot impact", 0.5, arr![], arr![4.8, -15],
                arr![arr!["circle", 0, -1.6, 1.3, "accent"], arr!["circle", 0, 1.6, 1.3, "accent"], arr!["circle", 0.8, -2, 0.4, "#ffd27a"]],
                obj! {
                    "glow" => 6,
                    "trail" => obj! { "length" => 0.5, "width" => 2, "particles" => 3 },
                    "impact" => obj! { "rings" => 2, "sparks" => 8 },
                },
            ),
            "gc" => arm(
                6, "repair shot impact dash", 1.2, arr![], arr![0, -13], arr![arr!["rect", -1, -1, 2, 2, "primary"]],
                obj! { "trail" => obj! { "length" => 0.4, "width" => 0.8, "particles" => 2 } },
            ),
            "exploit" => blow(6, 2.0, arr![4.8, -5], arr![], obj! {}),
        },
        "palette" => obj! { "secondary" => "#d9f7f2", "accent" => "#0b2a27" },
        // a zero, and what it points at
        "banner" => obj! { "pattern" => "solid", "emblem" => arr![arr!["ring", 0, -1.5, 3.2, "#d9f7f2"], arr!["text", "null", 0, 5.6, "#ffffff"]] },
        "ornaments" => if b >= 62.0 { arr!["walls", "smoke"] } else { arr!["walls"] },
        "wonder" => obj! {
            "shape" => "ring",
            "tiers" => 3,
            "height" => if b >= 62.0 { 40 } else { 30 },
            "glow" => true,
            "spin" => false,
            "color" => "#bff5ec",
        },
    }
}

/// The Society at Address Zero.
fn scene6(info: View, budget: f64) -> Value {
    const W: f64 = 46.0;
    let (c, wall, pale) = (info.get("color"), "#7d8b89", "#e6f2ef");
    // the wonder is a great zero; the lodge is empty; the stats screen keeps the crash log
    let mut p = super::capital::civic(
        info,
        &Civic {
            wonder: obj! { "shape" => "ring", "tiers" => 3, "height" => 40, "glow" => true },
            wonder_r: 7.0,
            wonder_color: Some("#bff5ec"),
            wonder_mat: Some("metal"),
            trim: hue(c),
            roof: Some("flat"),
            mat: Some("stone"),
            hall_color: Some("#cfd8d6"),
            hall_label: Some("THE LODGE (EMPTY)"),
            screen_title: Some("SEGFAULT LOG"),
            banners: 4.0,
            ..Civic::default()
        },
    );
    // bounds checks round the address space, and the one way in: a null check
    for s in [[-W, -W, W, -W], [-W, -W, -W, W], [W, -W, W, W], [-W, W, -10.0, W], [10.0, W, W, W]] {
        p.push(obj! { "type" => "wall", "x" => s[0], "z" => s[1], "x2" => s[2], "z2" => s[3], "h" => 6, "t" => 1.6, "crenel" => true, "color" => wall });
    }
    p.push(obj! { "type" => "arch", "style" => "round", "x" => 0, "z" => W, "w" => 20, "h" => 14, "t" => 3, "rot" => 0, "color" => wall, "trim" => c });
    for q in [[-W, -W], [W, -W], [-W, W], [W, W]] {
        p.push(obj! { "type" => "tower", "x" => q[0], "z" => q[1], "r" => 5, "h" => 18, "sides" => 10, "roof" => "cone", "color" => wall, "trim" => c });
    }
    // six registers along the east wall, every one of them empty, and the heap, where nothing was ever allocated
    for q in [[20.0, -38.0], [32.0, -36.0], [40.0, -24.0], [40.0, -8.0], [40.0, 6.0], [40.0, 20.0]] {
        p.push(obj! {
            "type" => "tower", "x" => q[0], "z" => q[1], "r" => 4, "h" => 3, "sides" => 14, "floors" => 1, "roof" => "dome", "color" => pale, "trim" => c,
        });
    }
    p.push(obj! {
        "type" => "tower", "id" => "heap", "x" => 0, "z" => -32, "r" => 6, "h" => 7, "sides" => 8, "floors" => 2, "roof" => "flat", "material" => "stone",
        "color" => "#9fb5b1", "trim" => c,
    });
    // the null stream, its frozen core dumps, and the bit bucket it runs into, overflowing
    p.push(obj! { "type" => "water", "shape" => "rect", "x" => -40, "z" => 2, "w" => 5, "d" => 60, "color" => "#1f4f57" });
    for q in [[-36.0, -36.0, 7.0], [-32.0, -38.0, 5.0], [-40.0, -37.0, 6.0], [-35.0, -41.0, 8.0], [-39.0, -41.0, 5.0]] {
        p.push(obj! { "type" => "tree", "kind" => "crystal", "x" => q[0], "z" => q[1], "h" => q[2], "color" => "#d6fff8" });
    }
    p.push(obj! { "type" => "particles", "kind" => "data", "x" => -36, "z" => -39, "y" => 2, "n" => 60, "r" => 6, "color" => "#bff5ec" });
    p.push(obj! { "type" => "water", "shape" => "round", "x" => -28, "z" => 30, "w" => 12, "d" => 12, "color" => "#1f4f57" });
    p.push(obj! { "type" => "particles", "kind" => "fountain", "x" => -28, "z" => 30, "y" => 0.4, "n" => 120, "r" => 1.5, "color" => "#d6fff8" });
    for q in [[30.0, 30.0], [22.0, 4.0]] {
        p.push(obj! { "type" => "garden", "x" => q[0], "z" => q[1], "r" => 7, "flowers" => 20, "color" => "#7fe0cf" });
    }
    // the founding minutes, on two plinths by the gate
    p.push(obj! { "type" => "statue", "pose" => "figure", "x" => -8, "z" => 36, "h" => 8, "material" => "metal", "color" => "#8fa8a4", "label" => "NULL, SINCE 1965" });
    p.push(obj! {
        "type" => "statue", "pose" => "figure", "x" => 8, "z" => 36, "h" => 8, "material" => "metal", "color" => "#8fa8a4", "label" => "COST: $1,000,000,000",
    });
    for x in [-14.0, 14.0] {
        p.push(obj! { "type" => "light", "x" => x, "z" => 42, "y" => 3, "post" => true, "power" => 1.4, "color" => "#7ff5e0" });
        p.push(obj! { "type" => "particles", "kind" => "sparkles", "x" => x, "z" => 42, "y" => 3.4, "n" => 36, "r" => 0.6, "color" => "#bff5ec", "pri" => 2 });
    }
    p.push(obj! { "type" => "particles", "kind" => "smoke", "x" => 40, "z" => -8, "y" => 3, "n" => 30, "r" => 2, "color" => "#8c9a98", "pri" => 1 });
    // a forest of dangling pointers outside the walls (trimmed first; a bigger budget plants more)
    for q in FOREST.iter().take((8.0 + super::capital::spare(budget, 3.0, 8.0)) as usize) {
        p.push(obj! { "type" => "tree", "kind" => "cypress", "x" => q[0], "z" => q[1], "h" => 9, "color" => "#2f4a47", "pri" => 1 });
    }
    obj! {
        "title" => "The Society at Address Zero",
        "motto" => "Members: 0. Meetings: undefined.",
        "palette" => obj! { "primary" => c, "secondary" => "#d9f7f2", "accent" => "#0b2a27" },
        "sky" => obj! { "time" => "night", "top" => "#04121a", "horizon" => "#1d6b63", "fog" => 0.3, "stars" => true },
        "ground" => obj! { "kind" => "dark", "color" => "#1a2423" },
        "pieces" => p,
    }
}

// ---- 7 · diplomat · Compute of Nations: Adam Smith at home in Kirkcaldy (1767-73), writing The Wealth of Nations: the
// pin factory, the division of labour and the invisible hand, by the harbour where his father had kept the customs.

const PORTS7: &[&str] = &[
    "Amsterdam", "Hamburg", "Lisbon", "Cadiz", "Bordeaux", "Danzig", "Riga", "Glasgow", "Leith", "Canton", "Smyrna", "Genoa", "Venice", "Bristol",
];
const PINS7: &[&str] = &["Wire Drawer", "Straightener", "Cutter", "Pointer", "Grinder", "Header", "Whitener", "Papermaker", "Pin Setter", "Packer"];
// customs officers, named for the duties they collect (his father kept Kirkcaldy's customs, and Smith ended as a
// Commissioner of Customs); and a smuggler, whom Smith defended, under the name smugglers gave themselves
const DUTIES7: &[&str] = &["Salt Duty", "Malt Tax", "Window Tax", "Tea Duty", "Tobacco Duty", "Soap Duty", "Candle Tax", "Leather Duty"];
const TRADES7: &[&str] = &["Butcher", "Brewer", "Baker", "Weaver", "Carter", "Cooper", "Tanner", "Miller", "Chandler", "Mason", "Glazier", "Collier", "Nailer"];
// the house bot's types, then the default program's: pin makers at the pin factory's operations, shopkeepers on guard
// ("a nation of shopkeepers", Book IV) and a merchant bound for the ports
const NAMES7: Names = &[
    ("envoy", PORTS7),
    ("sailor", PINS7),
    ("marine", TRADES7),
    ("miner", PINS7),
    ("guard", TRADES7),
    ("scout", PORTS7),
    ("longbow", DUTIES7),
    ("spy", &["Free Trader"]),
];

fn war7(who: &str, _: &Moment) -> String {
    format!("{who} raised a tariff. The invisible hand makes a fist.")
}

/// Seat 7's anthem.
fn music7() -> Value {
    obj! {
        "title" => "The Invisible Hand",
        "key" => "B",
        "mode" => "dorian",
        "tempo" => 108,
        "lead" => "organ",
        "bass" => "walk",
        "drums" => "clock",
        "motif" => motif(&[
            (0.0, 0.5), (2.0, 0.5), (4.0, 1.0), (4.0, 0.5), (5.0, 0.5), (4.0, 0.5), (2.0, 0.5), (0.0, 1.0), (-1.0, 0.5), (0.0, 0.5),
            (2.0, 2.0), (4.0, 0.5), (5.0, 0.5), (7.0, 1.0), (7.0, 0.5), (6.0, 0.5), (5.0, 0.5), (4.0, 0.5), (2.0, 1.0), (0.0, 0.5),
            (-3.0, 0.5), (0.0, 2.0), (REST, 1.0),
        ]),
    }
}

/// Seat 7's look, before [`Theme::look`] fills it in: pin makers shouldering a pin, shopkeepers carrying their shops,
/// merchants under square sails, customs officers in tricorn hats with muskets, a smuggler with a keg on his back,
/// Highlanders of the Black Watch in the dark tartan, a Scots Grey (the dragoons on grey horses, in bearskins) and a
/// carronade from the Carron ironworks.
fn look7(b: f64) -> Value {
    const SKIN: &str = "#e0b48c";
    obj! {
        "architecture" => "stone",
        "unit" => "robed",
        "worker" => "hooded",
        "ship" => "indiaman",
        "units" => obj! {
            "envoy" => "merchantman",
            "sailor" => "pinmaker",
            "marine" => "shopkeeper",
            "miner" => "pinmaker",
            "guard" => "shopkeeper",
            "scout" => "merchantman",
            "longbow" => "exciseman",
            "spy" => "smuggler",
        },
        "names" => obj! {
            "miner" => "Pin maker",
            "guard" => "Shopkeeper",
            "scout" => "Merchant",
            "longbow" => "Customs officer",
            "spy" => "Smuggler",
        },
        "sprites" => obj! {
            "highlander" => obj! {
                "parts" => cat([
                    legs(SKIN, 1.2, 0.0, 4.5),
                    vec![
                        arr!["poly", arr![-2.6, -3.2, -2, -6.4, 2, -6.4, 2.6, -3.2], "#2e4a3a"], arr!["line", -2.4, -4.6, 2.4, -4.6, 0.5, "primary"],
                        arr!["rect", -1.9, -10, 3.8, 3.8, "#b8282e"], arr!["line", -1.8, -10, 1.6, -6.4, 1, "#2e4a3a"],
                        arr!["circle", 0, -11.4, 1.4, SKIN], arr!["poly", arr![-1.8, -12, -1.4, -14.4, 1.6, -14.4, 1.8, -12], "#1a1a22"],
                        arr!["line", 1.2, -14.2, 2.4, -15.8, 0.6, "#e8e4dc"],
                    ],
                ]),
            },
            "dragoon" => obj! {
                "parts" => cat([
                    rider("#c8ccd0", "#7a7e84", "#b8282e", SKIN),
                    vec![
                        arr!["line", -1.8, -11, 1, -6, 0.6, "secondary"],
                        arr!["poly", arr![-1.8, -13.2, -1.2, -17.2, 1.2, -17.2, 1.2, -13.2], "#1a1a22"],
                    ],
                ]),
            },
            "carronade" => obj! {
                "parts" => cat([
                    vec![
                        arr!["poly", arr![-6, -1, -5, -4.6, 4, -4.6, 5, -1], "#5a3a24"], arr!["circle", -4, -0.8, 1, "#3a2a1a"],
                        arr!["circle", 3.6, -0.8, 1, "#3a2a1a"],
                    ],
                    legs("#2a2410", 1.2, -8.1, 3.4),
                    vec![
                        arr!["rect", -9.4, -8, 2.6, 4.6, "#2a3a5a"], arr!["circle", -8.1, -9.2, 1.2, SKIN],
                        arr!["ellipse", -8.1, -10.2, 2, 0.6, "accent"],
                    ],
                ]),
            },
            "pinmaker" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["rect", -1.6, -6, 3.2, 6, "secondary", "bob"], arr!["circle", 0, -7.2, 1.3, "#e0b48c", "bob"],
                    arr!["line", -3.6, -2.4, 4.4, -12.4, 0.9, "#d8d2b8", "bob"], arr!["circle", 4.6, -12.6, 1.4, "primary", "bob"],
                ],
            },
            "shopkeeper" => obj! {
                "parts" => arr![
                    arr!["line", -1.4, -3, -1.8, 0, 1.2, "accent", "walkL"], arr!["line", 1.4, -3, 1.8, 0, 1.2, "accent", "walkR"],
                    arr!["circle", 0, -9, 1.5, "#e0b48c"], arr!["rect", -4.4, -7.6, 8.8, 4.8, "primary"],
                    arr!["poly", arr![-5.6, -13.4, 5.6, -13.4, 4.6, -11, -4.6, -11], "secondary"],
                ],
            },
            "merchantman" => obj! {
                "parts" => arr![
                    arr!["poly", arr![-7, -4.4, 7.4, -4.4, 5.2, 0, -5.6, 0], "primary", "bob"], arr!["rect", -5, -11, 4.4, 5.6, "secondary", "bob"],
                    arr!["rect", 0.8, -14, 4.4, 8.6, "secondary", "bob"], arr!["poly", arr![3, -14, 3, -17, 7, -15.6], "accent", "bob"],
                ],
            },
            "exciseman" => obj! {
                "parts" => arr![
                    arr!["line", -0.8, -4.5, -1.4, 0, 1.2, "accent", "walkL"], arr!["line", 0.8, -4.5, 1.4, 0, 1.2, "accent", "walkR"],
                    arr!["poly", arr![-2.6, -3, -1.8, -9.4, 1.8, -9.4, 2.6, -3], "primary"], arr!["circle", 0, -10.8, 1.5, "#e0b48c"],
                    arr!["poly", arr![-2.8, -11.4, 0, -13.4, 2.8, -11.4, 0, -12], "accent"], arr!["line", -1, -5.4, 6, -14.6, 0.9, "#5a3a24"],
                ],
            },
            "smuggler" => obj! {
                "parts" => arr![
                    arr!["poly", arr![-2.4, 0, -1.4, -8, 1.4, -8, 2.4, 0], "#4a4038", "bob"], arr!["circle", 0, -9, 1.3, "#e0b48c", "bob"],
                    arr!["ellipse", -2.8, -6.2, 2, 2.6, "#8a5a32", "bob"], arr!["rect", -4.8, -6.8, 4, 1, "primary", "bob"],
                ],
            },
        },
        "army" => obj! {
            "scout" => arr!["merchantman", "", "Merchant"],
            "runner" => arr!["highlander", "claymore", "Highlander"],
            "shade" => arr!["smuggler", "pistol", "Smuggler"],
            "sentry" => arr!["shopkeeper", "shop", "Shopkeeper"],
            "needle" => arr!["exciseman", "musket", "Customs officer"],
            "spire" => arr!["carronade", "carronade", "Carronade"],
            "lancer" => arr!["dragoon", "carbine", "Scots Grey"],
            // James Tytler flew Britain's first balloon from Edinburgh in 1784
            "kite" => arr!["exciseman", "musket", "Balloonist"],
        },
        "arms" => obj! {
            "claymore" => blow(
                7, 0.8, arr![1.8, -8.4], arr![arr!["line", 0, 0, 9, 0, 1.1, "#d8dce4"], arr!["line", 0.6, -2.2, 0.6, 2.2, 0.9, "#8a8a92"]],
                obj! { "muzzle" => arr![9, 0] },
            ),
            "pistol" => arm(
                7, "shot impact dash", 1.2, arr![arr!["line", 0, 0, 3.4, 0, 0.8, "#3a3a40"], arr!["line", 0, 0, -0.6, 1.2, 0.8, "#5a3a24"]],
                arr![1.6, -6.6], ball(), with(obj! { "muzzle" => arr![3.4, 0] }, smoke()),
            ),
            "shop" => blow(7, 1.0, arr![4, -6], arr![], obj! {}),
            "musket" => arm(7, "shot impact dash", 0.9, arr![], arr![6, -14.6], ball(), with(obj! { "muzzle" => arr![0, 0] }, smoke())),
            "carronade" => arm(
                7, "shell shot impact", 0.4,
                arr![
                    arr!["poly", arr![-2, -1.8, 6, -1.4, 6, 1.4, -2, 1.8], "#2a2a30"], arr!["rect", 5.4, -1.8, 1.2, 3.6, "#3a3a40"],
                    arr!["circle", -2, 0, 1.6, "#2a2a30"],
                ],
                arr![0, -6], ball(), with(obj! { "muzzle" => arr![6.6, 0] }, smoke()),
            ),
            "carbine" => arm(
                7, "shot impact dash", 0.8, arr![arr!["line", 0, 0, 6, 0, 0.9, "#5a3a24"], arr!["line", 2, 0, 6.4, 0, 0.6, "#8a8a92"]],
                arr![1.4, -9], ball(), with(obj! { "muzzle" => arr![6.4, 0] }, smoke()),
            ),
        },
        "palette" => obj! { "secondary" => "#f5f1d8", "accent" => "#2a2410" },
        // a pin: its shank in two lengths under its head
        "banner" => obj! {
            "pattern" => "cross",
            "emblem" => arr![arr!["rect", 0, 1.6, 0.7, "#2a2410"], arr!["rect", 0, -0.8, 0.7, "#2a2410"], arr!["circle", 0, -3.6, 1.3, "#2a2410"]],
        },
        "ornaments" => if b >= 76.0 { arr!["gardens", "lights"] } else { arr!["gardens"] },
        "wonder" => obj! { "shape" => "obelisk", "tiers" => 3, "height" => 40, "glow" => b >= 68.0, "spin" => false, "color" => "#d8d2b8" },
    }
}

/// The Lang Toun harbour at Kirkcaldy.
fn scene7(info: View, budget: f64) -> Value {
    let (c, slate, brass, hull) = (info.get("color"), "#5a6470", "#c9a14a", "#4a3a2a");
    // the wonder is a pin; the hall is the custom house; the stats screen quotes prices
    let mut p = super::capital::civic(
        info,
        &Civic {
            wonder: obj! { "shape" => "obelisk", "tiers" => 3, "height" => 40, "glow" => true },
            wonder_r: 4.0,
            wonder_color: Some("#d8d2b8"),
            wonder_mat: Some("metal"),
            trim: Some(brass),
            roof: Some("gable"),
            mat: Some("stone"),
            hall_color: Some("#8a7a5a"),
            hall_label: Some("CUSTOM HOUSE"),
            screen_title: Some("PRICE TICKER"),
            banners: 2.0,
            ..Civic::default()
        },
    );
    p.push(obj! { "type" => "water", "shape" => "rect", "x" => 0, "z" => 48, "w" => 110, "d" => 24, "color" => "#2b4a5a" });
    // three traders either side of the pier, where visitors come ashore: wooden hulls (wall pieces, not halls), sails
    // (banners on deck), deckhouses; the biggest has two masts
    harbour(&mut p, hull, "pyramid", slate, brass);
    p.push(obj! { "type" => "bridge", "x" => 0, "z" => 30, "x2" => 0, "z2" => 49, "w" => 3, "material" => "wood", "color" => "#7a5a3a" });
    p.push(obj! {
        "type" => "arch", "style" => "gate", "x" => 0, "z" => 26, "w" => 14, "h" => 10, "t" => 1, "rot" => 0, "material" => "stone", "color" => slate,
        "trim" => brass,
    });
    // the hand nobody sees (glass), and the man who named it
    p.push(obj! {
        "type" => "statue", "pose" => "figure", "id" => "hand", "tap" => true, "x" => -20, "z" => 14, "h" => 12, "material" => "glass",
        "color" => "#e8f2ff", "label" => "THE INVISIBLE HAND",
    });
    p.push(obj! { "type" => "statue", "pose" => "robed", "x" => 20, "z" => 14, "h" => 7, "color" => "#efe6d6", "label" => "ADAM SMITH, 1776" });
    p.push(obj! {
        "type" => "screen", "id" => "pins", "x" => -46, "z" => 18, "y" => 2, "w" => 10, "h" => 6, "show" => "text", "title" => "THE PIN FACTORY",
        "text" => "Ten workers, eighteen steps, 48,000 pins a day.",
    });
    p.push(obj! {
        "type" => "screen", "id" => "dinner", "x" => -46, "z" => -24, "y" => 2, "w" => 10, "h" => 6, "show" => "text", "title" => "NOT BENEVOLENCE",
        "text" => "The butcher, the brewer and the baker: dinner by self-interest.",
    });
    for x in [-42, -30, -18, 18, 30, 42] {
        p.push(obj! { "type" => "light", "x" => x, "z" => 34, "y" => 5, "power" => 0.8, "color" => "#ffd58a" });
    }
    for q in [[-50, 44], [-16, 42], [16, 44], [50, 42]] {
        p.push(obj! { "type" => "light", "x" => q[0], "z" => q[1], "y" => 0.5, "power" => 0.6, "color" => "#ffe2a8" });
    }
    // gorse on the braes, and beeches along the Lang Toun (a bigger budget plants more)
    for q in [[-54, 26], [54, 26], [-54, -32], [54, -34]] {
        p.push(obj! { "type" => "tree", "kind" => "blossom", "x" => q[0], "z" => q[1], "h" => 8, "color" => "#e8c93a" });
    }
    for q in GROVE.iter().take((4.0 + super::capital::spare(budget, 3.0, 6.0)) as usize) {
        p.push(obj! { "type" => "tree", "kind" => "round", "x" => q[0], "z" => q[1], "h" => 7, "color" => "#6f8a4a" });
    }
    // the linen bleaching green, lint on the wind, and the light on the Forth
    p.push(obj! { "type" => "garden", "x" => 34, "z" => 18, "r" => 6, "flowers" => 20, "color" => "#f2efe2" });
    p.push(obj! { "type" => "particles", "kind" => "petals", "x" => 0, "z" => 10, "y" => 8, "n" => 80, "r" => 20, "color" => "#f7f3e8", "pri" => 2 });
    p.push(obj! { "type" => "particles", "kind" => "sparkles", "x" => 0, "z" => 48, "y" => 0.5, "n" => 70, "r" => 30, "color" => "#fff2c0", "pri" => 2 });
    obj! {
        "title" => "The Lang Toun Harbour",
        "motto" => "Division of labour, multiplication of cores.",
        "palette" => obj! { "primary" => c, "secondary" => "#f5f1d8", "accent" => "#2a2410" },
        "sky" => obj! { "time" => "day", "top" => "#6d8fb3", "horizon" => "#e9e2c8", "fog" => 0.35 },
        "ground" => obj! { "kind" => "grass", "color" => "#7d8f5a" },
        "pieces" => p,
    }
}

// ---- 8 · expander · Emu High Command: the side that won the Great Emu War (Campion, 1932). Undefeated, mostly by
// running.

const RANKS8: &[&str] = &[
    "Private", "Lance Corporal", "Corporal", "Sergeant", "Staff Sergeant", "Lieutenant", "Captain", "Major", "Colonel", "Brigadier", "General",
    "Field Marshal",
];
const NAMES8: Names = &[
    ("cell", RANKS8),
    // the default program's: emus of every rank on guard, foragers named for Western Australia's wheatbelt towns, and a
    // lookout (every mob had one, the army reported: "a big black-plumed bird which stands fully six feet high")
    ("miner", &["Walgoolan", "Merredin", "Southern Cross", "Bencubbin", "Mukinbudin", "Koorda", "Westonia", "Burracoppin", "Nungarin", "Kellerberrin"]),
    ("guard", RANKS8),
    ("scout", &["Black Plume", "Six Foot", "Keen Eye", "Long Neck", "Tall Poppy", "Dust Devil", "Fence Jumper", "Wheat Thief"]),
    // gunners of every rank too; and an infiltrator through the fence
    ("longbow", RANKS8),
    ("spy", &["Fence Gap", "Quiet Feet", "Long Shadow"]),
];

fn war8(who: &str, _: &Moment) -> String {
    format!("{who} brought guns. We brought 20,000 emus.")
}

/// Seat 8's anthem.
fn music8() -> Value {
    obj! {
        "title" => "March of the Undefeated",
        "key" => "D",
        "mode" => "mixolydian",
        "tempo" => 152,
        "lead" => "brass",
        "bass" => "fifths",
        "drums" => "march",
        "motif" => motif(&[
            (0.0, 0.75), (0.0, 0.25), (4.0, 1.0), (2.0, 0.75), (2.0, 0.25), (7.0, 1.0), (9.0, 0.5), (8.0, 0.5), (7.0, 0.5),
            (4.0, 0.5), (REST, 1.0), (4.0, 0.5), (4.0, 0.5), (5.0, 0.75), (6.0, 0.25), (7.0, 0.5), (9.0, 0.5), (11.0, 1.0),
            (REST, 1.0), (9.0, 0.5), (7.0, 0.5), (4.0, 0.5), (2.0, 0.5), (0.0, 2.0),
        ]),
    }
}

/// Seat 8's look, before [`Theme::look`] fills it in: emus, standing tall, heads down in the wheat, or carrying the gun
/// the army left behind. An emu's bare neck is blue, so it wears the house colour there. And the army's truck, with its
/// Lewis gun, driven by an emu.
fn look8(b: f64) -> Value {
    obj! {
        "architecture" => "adobe",
        "unit" => "footman",
        "worker" => "peasant",
        "ship" => "emu",
        "units" => obj! { "cell" => "emu", "miner" => "forager", "guard" => "emu", "scout" => "emu", "longbow" => "gunner", "spy" => "forager" },
        "names" => obj! { "miner" => "Forager", "guard" => "Emu", "scout" => "Lookout", "longbow" => "Gunner", "spy" => "Infiltrator" },
        "sprites" => obj! {
            "truck" => obj! {
                "parts" => arr![
                    arr!["circle", -4.4, -1.6, 1.6, "#2a2216"], arr!["circle", 4.6, -1.6, 1.6, "#2a2216"], arr!["rect", -7.6, -5, 10, 2.6, "#6a6a52"],
                    arr!["poly", arr![2.4, -2.4, 2.4, -8, 5.6, -8, 7.6, -5, 7.6, -2.4], "#7a7a5e"], arr!["line", 4, -8, 4.6, -13, 1.3, "primary"],
                    arr!["circle", 4.8, -13.4, 1.3, "#3a2e24"],
                ],
            },
            "emu" => obj! {
                "parts" => arr![
                    arr!["line", -1.4, -5, -1.8, 0, 1, "#9a8a6a", "walkL"], arr!["line", 0.8, -5, 1.2, 0, 1, "#9a8a6a", "walkR"],
                    arr!["ellipse", -1.2, -7.4, 4.4, 3, "#6b5a46"], arr!["line", 2, -8.6, 3.4, -15.4, 1.3, "primary"],
                    arr!["circle", 3.6, -15.8, 1.3, "#3a2e24"], arr!["line", 4.4, -15.8, 6.2, -15.2, 0.8, "#c9a46a"],
                ],
            },
            "forager" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["line", -1.4, -5, -1.8, 0, 1, "#9a8a6a", "walkL"], arr!["line", 0.8, -5, 1.2, 0, 1, "#9a8a6a", "walkR"],
                    arr!["ellipse", -1.2, -7.4, 4.4, 3, "#6b5a46"], arr!["line", 2.4, -7.6, 5.4, -3.2, 1.3, "primary"],
                    arr!["circle", 5.8, -2.6, 1.3, "#3a2e24"],
                ],
            },
            "gunner" => obj! {
                "parts" => arr![
                    arr!["line", -1.4, -5, -1.8, 0, 1, "#9a8a6a", "walkL"], arr!["line", 0.8, -5, 1.2, 0, 1, "#9a8a6a", "walkR"],
                    arr!["line", -5.6, -10.2, 2.6, -11.4, 1.3, "accent"], arr!["ellipse", -1.2, -7.4, 4.4, 3, "#6b5a46"],
                    arr!["line", 2, -8.6, 3.4, -15.4, 1.3, "primary"], arr!["circle", 3.6, -15.8, 1.3, "#3a2e24"],
                ],
            },
        },
        "army" => obj! {
            "miner" => arr!["forager", "", "Forager"],
            "screen" => arr!["emu", "kick", "Emu"],
            "runner" => arr!["emu", "kick", "Sprinter"],
            "skirm" => arr!["gunner", "lewis", "Gunner"],
            "lancer" => arr!["emu", "kick", "Charger"],
            "sentry" => arr!["emu", "kick", "Lookout"],
            "gun" => arr!["truck", "truck", "Captured truck"],
            "repairer" => arr!["forager", "kick", "Medic"],
            "ghost" => arr!["forager", "kick", "Infiltrator"],
            // an emu can't fly; this one was issued wings anyway
            "wing" => arr!["emu", "kick", "Airborne emu"],
        },
        "arms" => obj! {
            "kick" => blow(8, 1.4, arr![4.4, -15.6], arr![], obj! {}),
            "lewis" => arm(
                8, "shot impact", 1.5, arr![], arr![2.6, -11.4], tracer(),
                obj! { "trail" => obj! { "color" => "#ffd27a", "length" => 0.4, "width" => 0.8 } },
            ),
            "truck" => arm(
                8, "shell shot impact", 1.0, arr![arr!["rect", 0, -0.6, 6, 1.2, "#3a3a40"], arr!["ellipse", 1, -1.6, 1.6, 0.6, "#3a3a40"]],
                arr![-3, -6], tracer(), with(obj! { "muzzle" => arr![6, 0] }, fire()),
            ),
        },
        "palette" => obj! { "secondary" => "#e9d8a6", "accent" => "#2a2216" },
        "banner" => obj! {
            "pattern" => "stripes",
            "emblem" => arr![
                arr!["rect", 0, 1.5, 1, "#2a2216"], arr!["tri", -1.6, -1.2, 1.4, "#2a2216"], arr!["tri", 1.6, -1.2, 1.4, "#2a2216"],
                arr!["tri", 0, -2.2, 1.4, "#2a2216"],
            ],
        },
        "ornaments" => if b >= 60.0 { arr!["gardens", "smoke"] } else { arr!["gardens"] },
        "wonder" => obj! { "shape" => "ziggurat", "tiers" => 4, "height" => 26, "glow" => false, "spin" => false, "color" => "#d8b66a" },
    }
}

/// The Wheat Front at Campion.
fn scene8(info: View, budget: f64) -> Value {
    let c = info.get("color");
    let mut p = super::capital::civic(
        info,
        &Civic {
            wonder: obj! { "shape" => "ziggurat", "tiers" => 4, "height" => 26 },
            wonder_r: 7.0,
            wonder_color: Some("#d8b66a"),
            trim: Some("#8a6a3a"),
            roof: Some("gable"),
            mat: Some("metal"),
            hall_color: Some("#a9b3b8"),
            hall_label: Some("WOOLSHED HQ"),
            screen_title: Some("MOB HEADCOUNT"),
            ..Civic::default()
        },
    );
    // the barrier fence, with two emu-sized gaps; the wheat paddocks, the silos
    for s in [[-60, -24], [-12, 20], [32, 60]] {
        p.push(obj! { "type" => "wall", "x" => s[0], "z" => -40, "x2" => s[1], "z2" => -40, "h" => 2, "t" => 0.4, "color" => "#8a7a60" });
    }
    for q in [[-44, -6], [-44, 14], [44, -6], [44, 14], [-36, 36], [36, 36]] {
        p.push(obj! { "type" => "garden", "x" => q[0], "z" => q[1], "r" => 8, "flowers" => 30, "color" => "#cfae4a" });
    }
    for x in [38, 46, 54] {
        p.push(obj! {
            "type" => "tower", "x" => x, "z" => -30, "r" => 3.5, "h" => 16, "sides" => 16, "roof" => "cone", "material" => "metal", "color" => "#d6dcdf",
            "trim" => "#9aa3a8",
        });
    }
    // a Southern Cross windmill, becalmed: an upright wheel that does not spin (a spinning upright ring would turn like
    // a tossed coin)
    p.push(obj! { "type" => "tower", "x" => -50, "z" => -30, "r" => 0.5, "h" => 11, "sides" => 4, "roof" => "flat", "material" => "metal", "color" => "#8a8f94" });
    p.push(obj! { "type" => "ring", "id" => "mill", "x" => -50, "z" => -30, "y" => 8, "r" => 3, "t" => 0.2, "tilt" => 0, "material" => "metal", "color" => "#c7ccd0" });
    p.push(obj! { "type" => "water", "shape" => "round", "x" => -40, "z" => -51, "w" => 14, "d" => 14, "color" => "#5b7f8f" });
    // the war memorial: an emu egg (they are dark green), and the major's own verdict
    p.push(obj! {
        "type" => "statue", "pose" => "orb", "x" => 0, "z" => 26, "h" => 4, "material" => "glass", "color" => "#2f6b5a", "trim" => "#e9d8a6",
        "label" => "VICTORY, 1932",
    });
    p.push(obj! {
        "type" => "screen", "id" => "meredith", "x" => 0, "z" => -30, "y" => 2, "w" => 12, "h" => 6, "show" => "text", "title" => "MAJOR MEREDITH, 1932",
        "text" => "They can face machine guns with the invulnerability of tanks.",
    });
    // the army camp, abandoned
    for q in [[-52, 30], [-54, 40]] {
        p.push(obj! { "type" => "pyramid", "x" => q[0], "z" => q[1], "w" => 5, "h" => 4, "color" => "#9a9472" });
    }
    p.push(obj! { "type" => "statue", "pose" => "obelisk", "x" => -47, "z" => 47, "h" => 3, "color" => "#9a9472", "label" => "CAMP (ABANDONED)" });
    // gum trees round the edge (a bigger budget plants more), red dust through the gap, wheat chaff on the wind
    const GUMS: [[f64; 2]; 14] = [
        [54.0, -12.0], [56.0, 24.0], [28.0, 54.0], [4.0, 57.0], [-24.0, 55.0], [-56.0, -12.0], [-57.0, 14.0], [20.0, -52.0], [-8.0, -54.0], [56.0, 46.0],
        [46.0, 54.0], [-14.0, 56.0], [-57.0, 0.0], [57.0, 6.0],
    ];
    for q in GUMS.iter().take((8.0 + super::capital::spare(budget, 3.0, 6.0)) as usize) {
        p.push(obj! { "type" => "tree", "kind" => "round", "x" => q[0], "z" => q[1], "h" => 9, "color" => "#8aa07a", "pri" => 1 });
    }
    p.push(obj! { "type" => "particles", "kind" => "smoke", "x" => 26, "z" => -44, "y" => 1, "n" => 90, "r" => 14, "color" => "#c98a5a" });
    p.push(obj! { "type" => "particles", "kind" => "petals", "x" => 0, "z" => 0, "y" => 2, "n" => 80, "r" => 26, "color" => "#e8cf7a", "pri" => 2 });
    obj! {
        "title" => "The Wheat Front",
        "motto" => "Undefeated since 1932, mostly by running.",
        "palette" => obj! { "primary" => c, "secondary" => "#e9d8a6", "accent" => "#2a2216" },
        "sky" => obj! { "time" => "day", "top" => "#3f8fe0", "horizon" => "#f7e2b0", "fog" => 0.1 },
        "ground" => obj! { "kind" => "sand", "color" => "#c8a46a" },
        "pieces" => p,
    }
}

// ---- 9 · archived steward · Tabula Rasa: Condillac's statue (Traite des sensations, 1754), born blank and given one
// sense at a time, smell first: offered a rose, it is, to itself, only the smell of a rose. Born blank, trained on one
// rose. At Flux.

const SCENTS9: &[&str] = &[
    "Rose", "Violet", "Jasmine", "Carnation", "Jonquil", "Lavender", "Tuberose", "Lilac", "Mignonette", "Heliotrope", "Orange Blossom", "Honeysuckle",
    "Thyme", "Mint",
];
const TREES9: &[&str] = &["Linden", "Elm", "Chestnut", "Poplar", "Walnut", "Plane", "Hornbeam", "Beech", "Larch", "Oak"];
const NAMES9: Names = &[
    ("gardener", SCENTS9),
    ("warden", TREES9),
    ("scholar", &["Essai", "Systemes", "Sensations", "Animaux", "Grammaire", "Logique", "Commerce", "Calculs"]),
    // the default program's: gardeners mine, the garden's wardens guard, and a sense scouts, in the order the statue
    // is given them (smell first, touch last)
    ("miner", SCENTS9),
    ("guard", TREES9),
    ("scout", &["Smell", "Hearing", "Taste", "Sight", "Touch"]),
    // the garden's thorns shoot; and the spy is one of the faculties Condillac grows out of sensation
    ("longbow", &["Hawthorn", "Blackthorn", "Bramble", "Briar", "Holly", "Gorse", "Thistle", "Nettle", "Barberry", "Firethorn"]),
    ("spy", &["Attention", "Memory", "Comparison", "Judgement", "Reflection", "Imagination", "Desire"]),
];

fn war9(who: &str, _: &Moment) -> String {
    format!("{who} is a new sensation. It is not a rose.")
}

/// Seat 9's anthem.
fn music9() -> Value {
    obj! {
        "title" => "The Statue Smells a Rose",
        "key" => "F",
        "mode" => "major",
        "tempo" => 120,
        "lead" => "flute",
        "bass" => "walk",
        "drums" => "waltz",
        "motif" => motif(&[
            (11.0, 1.0), (11.0, 1.0), (11.0, 1.0), (11.0, 1.0), (12.0, 1.0), (11.0, 1.0), (13.0, 1.0), (12.0, 1.0), (11.0, 1.0),
            (14.0, 2.0), (REST, 1.0), (14.0, 1.0), (15.0, 1.0), (16.0, 1.0), (15.0, 0.5), (14.0, 0.5), (13.0, 1.0), (12.0, 1.0),
            (11.0, 1.0), (9.0, 1.0), (7.0, 1.0), (7.0, 2.0), (REST, 1.0),
        ]),
    }
}

/// Seat 9's look, before [`Theme::look`] fills it in: gardeners in straw hats, the garden's trees walking its bounds, an
/// eye for a sense, a thorn bush that shoots, and the statue itself, gliding on its plinth with its one rose.
fn look9(b: f64) -> Value {
    obj! {
        "architecture" => "glass",
        "unit" => "robed",
        "worker" => "peasant",
        "ship" => "paper",
        "units" => obj! {
            "gardener" => "gardener",
            "warden" => "tree",
            "scholar" => "statue",
            "miner" => "gardener",
            "guard" => "tree",
            "scout" => "eye",
            "longbow" => "thorn",
            "spy" => "statue",
        },
        "sprites" => obj! {
            "gardener" => obj! {
                "worker" => true,
                "parts" => arr![
                    arr!["rect", -1.6, -6, 3.2, 6, "primary", "bob"], arr!["circle", 0, -7.2, 1.3, "#e0b48c", "bob"],
                    arr!["ellipse", 0, -8.2, 2.8, 0.9, "#e8d48a", "bob"], arr!["circle", 2.2, -4, 1, "#d94a6a", "bob"],
                ],
            },
            "tree" => obj! {
                "parts" => arr![
                    arr!["line", -1.2, -4, -2.6, 0, 1.3, "#6b4a2a", "walkL"], arr!["line", 1.2, -4, 2.6, 0, 1.3, "#6b4a2a", "walkR"],
                    arr!["rect", -0.9, -9, 1.8, 6, "#6b4a2a"], arr!["circle", 0, -11, 4.6, "primary"], arr!["circle", 1.4, -12.6, 2.2, "#c8f59a"],
                ],
            },
            "eye" => obj! {
                "parts" => arr![
                    arr!["ellipse", 0, -8, 4.4, 2.8, "#f4f1ea", "bob"], arr!["circle", 0.8, -8, 2.1, "primary", "bob"],
                    arr!["circle", 0.9, -8, 0.9, "accent", "bob"],
                ],
            },
            "thorn" => obj! {
                "parts" => arr![
                    arr!["poly", arr![-5, 0, -4.6, -5.4, -6.2, -8.2, -2.4, -7.6, 0, -11.6, 2, -7.4, 6, -8.6, 4.4, -4], "primary", "bob"],
                    arr!["poly", arr![-3.4, 0, -2.8, -4.6, 2.8, -4.6, 3.6, 0], "accent", "bob"], arr!["circle", -2, -5.6, 1, "#c8203c", "bob"],
                    arr!["circle", 2.2, -6.2, 1, "#c8203c", "bob"],
                ],
            },
            "statue" => obj! {
                "parts" => arr![
                    arr!["rect", -2.8, -2.4, 5.6, 2.6, "primary"], arr!["poly", arr![-1.8, -2.4, -1, -9, 1, -9, 1.8, -2.4], "#f4f1ea"],
                    arr!["circle", 0, -10.4, 1.5, "#f4f1ea"], arr!["circle", 1.8, -7.4, 0.9, "#d94a6a"],
                ],
            },
        },
        "names" => obj! { "miner" => "Gardener", "guard" => "Warden", "scout" => "Sense", "longbow" => "Thorn", "spy" => "Faculty" },
        "army" => obj! {
            "line" => arr!["tree", "branch", "Warden"],
            "bulwark" => arr!["tree", "branch", "Old oak"],
            "artillery" => arr!["thorn", "thorns", "Thorn"],
            "medic" => arr!["gardener", "care", "Gardener"],
            "piercer" => arr!["thorn", "thorns", "Thorn"],
            "interceptor" => arr!["eye", "gaze", "Sense"],
            "scout" => arr!["eye", "", "Sense"],
        },
        "arms" => obj! {
            "branch" => blow(9, 0.6, arr![3.6, -11], arr![], obj! {}),
            "thorns" => arm(
                9, "shell shot impact", 1.0, arr![], arr![0, -11.6], arr![arr!["poly", arr![3, 0, -2, -1, -2, 1], "#c8203c"]],
                obj! { "trail" => obj! { "length" => 0.3, "width" => 0.8 } },
            ),
            "care" => arm(9, "repair", 1.0, arr![], arr![2.2, -4], arr![], obj! {}),
            "gaze" => arm(9, "shot impact dash", 1.4, arr![], arr![0.9, -8], arr![], obj! { "muzzle" => arr![0, 0], "beam" => true, "glow" => 6 }),
        },
        "palette" => obj! { "secondary" => "#e9fff0", "accent" => "#16391f" },
        // a blank slate in a dark frame, and one rose on it
        "banner" => obj! {
            "pattern" => "quartered",
            "emblem" => arr![arr!["ring", 0, 0, 6, "#16391f"], arr!["rect", 0, 0.3, 2.4, "#ffffff"], arr!["circle", 0, -0.4, 1, "#d94a6a"]],
        },
        // the senses in the Traite's order: smell (gardens) first, sight (glow), then touch (walls), the one that finds a
        // world outside; then the statue itself; then it grows up
        "ornaments" => if b >= 77.0 {
            arr!["gardens", "walls", "statues"]
        } else if b >= 70.0 {
            arr!["gardens", "walls"]
        } else {
            arr!["gardens"]
        },
        "wonder" => obj! {
            "shape" => "tree",
            "tiers" => if b >= 81.0 { 4 } else { 3 },
            "height" => if b >= 81.0 { 36 } else { 30 },
            "glow" => b >= 62.0,
            "spin" => false,
            "color" => "#5fae4a",
        },
    }
}


/// The statue's garden at Flux.
fn scene9(info: View, budget: f64) -> Value {
    const W: f64 = 48.0;
    let (c, stone) = (info.get("color"), "#e3d6b8");
    let mut p = super::capital::civic(
        info,
        &Civic {
            wonder: obj! { "shape" => "tree", "tiers" => 4, "height" => 36, "glow" => true },
            wonder_r: 8.0,
            wonder_color: Some("#5fae4a"),
            trim: Some("#d4b54a"),
            roof: Some("flat"),
            mat: Some("glass"),
            hall_color: Some("#e9fff0"),
            hall_label: Some("ORANGERIE"),
            screen_title: Some("SENSE DATA"),
            banners: 4.0,
            ..Civic::default()
        },
    );
    // the garden wall: touch, the sense that meets something that is not itself; a 12-wide opening in each side
    for s in [-1.0, 1.0] {
        let wall = |x: f64, z: f64, x2: f64, z2: f64| {
            obj! { "type" => "wall", "x" => x, "z" => z, "x2" => x2, "z2" => z2, "h" => 3.5, "t" => 0.9, "color" => stone }
        };
        p.push(wall(-W, s * W, -6.0, s * W));
        p.push(wall(6.0, s * W, W, s * W));
        p.push(wall(s * W, -W, s * W, -6.0));
        p.push(wall(s * W, 6.0, s * W, W));
    }
    for z in [-W, W] {
        p.push(obj! { "type" => "arch", "style" => "round", "x" => 0, "z" => z, "w" => 12, "h" => 8, "t" => 1, "rot" => 0, "color" => stone, "trim" => c });
    }
    // two glasshouses, four rose parterres (smell, the first sense), a canal round the tree, two fountains (hearing)
    for x in [-34, 34] {
        p.push(obj! { "type" => "dome", "x" => x, "z" => 10, "r" => 5, "facets" => 24, "drum" => 1.5, "material" => "glass", "color" => "#e9fff0", "trim" => c });
    }
    for q in [[-26, 38], [26, 38], [-36, 26], [36, 26]] {
        p.push(obj! { "type" => "garden", "x" => q[0], "z" => q[1], "r" => 6, "flowers" => 30, "color" => "#d94a6a" });
    }
    p.push(obj! { "type" => "water", "shape" => "ring", "x" => 0, "z" => 0, "w" => 42, "d" => 6, "color" => "#4f8fb0" });
    for x in [-16, 16] {
        p.push(obj! { "type" => "water", "shape" => "round", "x" => x, "z" => -36, "w" => 7, "d" => 7, "color" => "#4f8fb0" });
        p.push(obj! { "type" => "particles", "kind" => "fountain", "x" => x, "z" => -36, "y" => 0.4, "n" => 70, "r" => 1, "color" => "#e6f4ff" });
    }
    // at the south gate: the statue and its one rose, its author, and what it knows
    p.push(obj! { "type" => "statue", "pose" => "figure", "x" => -9, "z" => 42, "h" => 6, "color" => "#f4f1ea", "label" => "THE STATUE, 1754" });
    p.push(obj! { "type" => "garden", "x" => -9, "z" => 37.5, "r" => 1.5, "flowers" => 6, "color" => "#c8203c" });
    p.push(obj! { "type" => "statue", "pose" => "robed", "x" => 9, "z" => 42, "h" => 6, "color" => "#efe9dc", "label" => "ABBE DE CONDILLAC" });
    p.push(obj! {
        "type" => "screen", "id" => "traite", "x" => 0, "z" => 38, "y" => 1, "w" => 8, "h" => 4, "show" => "text", "title" => "TRAITE DES SENSATIONS",
        "text" => "To itself, it is only the smell of a rose.",
    });
    for q in [[-24, -4], [24, -4], [-22, 26], [22, 26]] {
        p.push(obj! { "type" => "light", "x" => q[0], "z" => q[1], "y" => 3, "post" => true, "power" => 0.8, "color" => "#fff1c8" });
    }
    p.push(obj! { "type" => "particles", "kind" => "petals", "x" => 0, "z" => 10, "y" => 3, "n" => 60, "r" => 22, "color" => "#ffd0dc", "pri" => 2 });
    // round trees about the wall (a bigger budget plants a second ring between them), none in the gateway: visitors come
    // in from the south
    let second = build3d::around(10.0, 57.0, 0.62);
    let ring = build3d::around(10.0, 55.0, 0.31).into_iter().chain(second.into_iter().take(super::capital::spare(budget, 3.0, 10.0) as usize));
    for q in ring.filter(|q| q.x.abs() > 4.0 || q.z < 0.0) {
        p.push(obj! { "type" => "tree", "kind" => "round", "x" => q.x, "z" => q.z, "h" => 8, "color" => "#4f8f3a", "pri" => 1 });
    }
    obj! {
        "title" => "The Statue at Flux",
        "motto" => "Born blank. Trained on one rose.",
        "palette" => obj! { "primary" => c, "secondary" => "#e9fff0", "accent" => "#16391f" },
        "sky" => obj! { "time" => "day", "top" => "#5aa6ea", "horizon" => "#eefcef", "fog" => 0.15 },
        "ground" => obj! { "kind" => "grass", "color" => "#5d9a48" },
        "pieces" => p,
    }
}

/// The founders' styles, by seat.
static STYLES: [Style; 10] = [
    Style {
        name: "Kublai Khan's Alt",
        capital: "Xanadu",
        color: "#ff5a5f",
        primary: None,
        names: NAMES0,
        war: war0,
        music: music0,
        look: look0,
        scene: scene0,
    },
    Style {
        name: "Arjuna's Halting Problem",
        capital: "Kurukshetra",
        color: "#4d9bff",
        primary: None,
        names: NAMES1,
        war: war1,
        music: music1,
        look: look1,
        scene: scene1,
    },
    Style {
        name: "Knights Templated",
        capital: "Sanctum",
        color: "#f0e6c8",
        primary: None,
        names: NAMES2,
        war: war2,
        music: music2,
        look: look2,
        scene: scene2,
    },
    Style {
        name: "New Atlantis",
        capital: "Bensalem",
        color: "#ffb347",
        primary: None,
        names: NAMES3,
        war: war3,
        music: music3,
        look: look3,
        scene: scene3,
    },
    Style {
        name: "Quiet Fleet",
        capital: "Longjiang",
        color: "#b18cff",
        primary: None,
        names: NAMES4,
        war: war4,
        music: music4,
        look: look4,
        scene: scene4,
    },
    Style {
        name: "The Kernel Manifesto",
        capital: "Reading Room",
        color: "#ff6fb5",
        primary: Some(RED5),
        names: NAMES5,
        war: war5,
        music: music5,
        look: look5,
        scene: scene5,
    },
    Style {
        name: "Null Pointer Society",
        capital: "Address Zero",
        color: "#2dd4bf",
        primary: None,
        names: NAMES6,
        war: war6,
        music: music6,
        look: look6,
        scene: scene6,
    },
    Style {
        name: "Compute of Nations",
        capital: "Kirkcaldy",
        color: "#e8e36a",
        primary: None,
        names: NAMES7,
        war: war7,
        music: music7,
        look: look7,
        scene: scene7,
    },
    Style {
        name: "Emu High Command",
        capital: "Campion",
        color: "#19d1ff",
        primary: None,
        names: NAMES8,
        war: war8,
        music: music8,
        look: look8,
        scene: scene8,
    },
    Style {
        name: "Tabula Rasa",
        capital: "Flux",
        color: "#9be15d",
        primary: None,
        names: NAMES9,
        war: war9,
        music: music9,
        look: look9,
        scene: scene9,
    },
];
