//! Default capital layout and Wonder animation.
use gameofcores::{Value, View};
use super::compat as js;

/// What [`civic`] builds: `wonder` (a look's wonder, e.g. `info.get("wonder").value()`: `{ shape, tiers, height, glow,
/// spin, color }`; undefined for none) and its radius `wonder_r` (6), colour (`wonder_color`, else the wonder's own,
/// else yours) and material `wonder_mat` ('stone'); `trim`, the colour of every piece's trim; the hall's height `hall_h`
/// (9), `roof` ('gable'), material `mat` ('stone'), `hall_color` and `hall_label`; the stats screen's `screen_title`
/// ('LIVE STATS'); and `banners` (6) on poles `pole_h` high (15). A number left 0 or a name left `None` is its default.
#[derive(Clone, Debug)]
pub struct Civic<'a> {
    pub wonder: Value,
    pub wonder_r: f64,
    /// Multiplier for the Wonder's authored height.  Houses use this to keep their
    /// landmark legible without letting it dominate the visitor's court.
    pub wonder_h_scale: f64,
    pub wonder_color: Option<&'a str>,
    pub wonder_mat: Option<&'a str>,
    pub trim: Option<&'a str>,
    pub hall_h: f64,
    pub roof: Option<&'a str>,
    pub mat: Option<&'a str>,
    pub hall_color: Option<&'a str>,
    pub hall_label: Option<&'a str>,
    pub screen_title: Option<&'a str>,
    pub banners: f64,
    pub pole_h: f64,
    /// The bounded arrival-court language used by the hall renderer.  This is scene
    /// data, rather than an inferred seat or display name, so borrowed house styles
    /// keep their authored architecture.
    pub court: Option<&'a str>,
    /// Whether the hall forecourt has its two decorative lamps.
    pub forecourt_lights: bool,
}

impl Default for Civic<'_> {
    fn default() -> Self {
        Civic {
            wonder: Value::Undef,
            wonder_r: 6.0,
            wonder_h_scale: 1.0,
            wonder_color: None,
            wonder_mat: None,
            trim: None,
            hall_h: 9.0,
            roof: None,
            mat: None,
            hall_color: None,
            hall_label: None,
            screen_title: None,
            banners: 6.0,
            pole_h: 15.0,
            court: None,
            forecourt_lights: true,
        }
    }
}

/// `x || d` for a number given in Rust: `d` when it's 0 or NaN.
fn or(x: f64, d: f64) -> f64 {
    if x != 0.0 && !x.is_nan() {
        x
    } else {
        d
    }
}

/// `s || d` for a name given in Rust.
fn or_str<'a>(s: Option<&'a str>, d: &'a str) -> &'a str {
    s.filter(|s| !s.is_empty()).unwrap_or(d)
}

/// The civic core every capital can start from: the Wonder (from look()) in the middle (id 'wonder', visitors can tap
/// it), a town hall showing what you say in public (`mem::say`; every faction's messages when you say nothing), a live
/// stats screen and an avenue of banners.
#[must_use]
pub fn civic(info: View, o: &Civic) -> Vec<Value> {
    let mut out = Vec::new();
    let w = &o.wonder;
    if w.truthy() {
        let color = match o.wonder_color.filter(|c| !c.is_empty()) {
            Some(c) => Value::from(c),
            None if w.get("color").truthy() => w.get("color").clone(),
            None => info.get("color").value(),
        };
        out.push(js::object(vec![
            ("type", Value::from("wonder")),
            ("id", Value::from("wonder")),
            ("tap", Value::Bool(true)),
            ("shape", w.get("shape").clone()),
            ("tiers", w.get("tiers").clone()),
            ("h", Value::Num(w.get("height").num() * or(o.wonder_h_scale, 1.0))),
            ("r", Value::Num(or(o.wonder_r, 6.0))),
            ("glow", w.get("glow").clone()),
            ("spin", w.get("spin").clone()),
            ("color", color),
            ("material", Value::from(or_str(o.wonder_mat, "stone"))),
            ("trim", Value::from(o.trim)),
            ("court", Value::from(o.court)),
            ("pri", Value::Num(9.0)),
        ]));
    }
    out.push(js::object(vec![
        ("type", Value::from("hall")),
        ("id", Value::from("hall")),
        ("x", Value::Num(-26.0)),
        ("z", Value::Num(-20.0)),
        ("w", Value::Num(18.0)),
        ("d", Value::Num(11.0)),
        ("h", Value::Num(or(o.hall_h, 9.0))),
        ("roof", Value::from(or_str(o.roof, "gable"))),
        ("material", Value::from(or_str(o.mat, "stone"))),
        ("color", Value::from(o.hall_color)),
        ("trim", Value::from(o.trim)),
        ("label", Value::from(o.hall_label)),
        ("court", Value::from(o.court)),
        ("pri", Value::Num(8.0)),
    ]));
    // Decorative lamps yield first when a house has spent its base scene budget on
    // authored machinery. That preserves its existing landmark composition at 224.
    if o.forecourt_lights {
        for x in [-34.0, -18.0] {
            out.push(js::object(vec![
                ("type", Value::from("light")),
                ("x", Value::Num(x)),
                ("z", Value::Num(-11.5)),
                ("y", Value::Num(4.0)),
                ("post", Value::Bool(true)),
                ("power", Value::Num(1.2)),
                ("color", Value::from(o.trim)),
                ("pri", Value::Num(6.0)),
            ]));
        }
    }
    out.push(js::object(vec![
        ("type", Value::from("screen")),
        ("id", Value::from("stats")),
        ("x", Value::Num(26.0)),
        ("z", Value::Num(-20.0)),
        ("y", Value::Num(3.0)),
        ("w", Value::Num(15.0)),
        ("h", Value::Num(8.5)),
        ("show", Value::from("stats")),
        ("title", Value::from(or_str(o.screen_title, "LIVE STATS"))),
        ("pri", Value::Num(8.0)),
    ]));
    let n = or(o.banners, 6.0);
    let mut i = 0.0;
    while i < n {
        let row = (i / 2.0).floor();
        out.push(js::object(vec![
            ("type", Value::from("banner")),
            ("x", Value::Num(if i % 2.0 != 0.0 { 12.0 } else { -12.0 })),
            ("z", Value::Num(20.0 + row * 11.0)),
            ("h", Value::Num(or(o.pole_h, 15.0))),
            ("w", Value::Num(5.0)),
            ("rot", Value::Num(0.0)),
            ("pri", Value::Num(7.0 - row)),
        ]));
        i += 1.0;
    }
    out
}

/// For `live3d(state, now)`: taps on the Wonder spin it up, and it winds down again. `s` is what live3d keeps (made an
/// object if it isn't one); `s.heat` holds the spin. Returns the change for the 'wonder' piece (`{ spin }`).
pub fn ring(s: &mut Value, now: View) -> Value {
    let heat = js::round(js::min(240.0, js::vor(s.get("heat"), 0.0) * 0.75 + js::or(now.get("taps").get("wonder"), 0.0) * 45.0));
    let heat = if heat < 3.0 { 0.0 } else { heat };
    s.set("heat", heat);
    js::object(vec![("spin", if heat != 0.0 && !heat.is_nan() { Value::Num(heat) } else { Value::Null })])
}

/// How many extra pieces a bigger budget affords: one per `per` build points past 224, at most `cap` (cosmetic credits
/// raise the budget).
#[must_use]
pub fn spare(budget: f64, per: f64, cap: f64) -> f64 {
    js::max(0.0, js::min(cap, ((budget - 224.0) / per).floor()))
}

