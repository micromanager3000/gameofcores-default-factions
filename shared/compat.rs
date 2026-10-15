//! Numeric behavior preserved from the original faction presentation code.
use core::fmt::Write as _;
use gameofcores::{Obj, Value, View};

pub(crate) fn num_str(n: f64) -> String {
    if n.is_nan() {
        return "NaN".into();
    }
    if n.is_infinite() {
        return if n > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if n == 0.0 {
        return "0".into();
    }
    let mut s = String::new();
    if n.fract() == 0.0 && n.abs() < 1e21 {
        let _ = write!(s, "{}", n as i128);
    } else {
        let _ = write!(s, "{n}");
    }
    s
}

pub(crate) fn num(v: View) -> f64 {
    if v.is_num() {
        return v.num();
    }
    if v.is_null() {
        return 0.0;
    }
    if v.is_bool() {
        return if v.truthy() { 1.0 } else { 0.0 };
    }
    if v.is_str() {
        return str_num(v.str());
    }
    if v.is_arr() {
        return match v.len() {
            0 => 0.0,
            1 => num(v.at(0)),
            _ => f64::NAN,
        };
    }
    f64::NAN
}

pub(crate) fn vnum(v: &Value) -> f64 {
    match v {
        Value::Undef => f64::NAN,
        Value::Null => 0.0,
        Value::Bool(b) => f64::from(u8::from(*b)),
        Value::Num(n) => *n,
        Value::Str(s) => str_num(s),
        Value::Arr(a) => match a.len() {
            0 => 0.0,
            1 => vnum(&a[0]),
            _ => f64::NAN,
        },
        _ => f64::NAN,
    }
}

fn str_num(s: &str) -> f64 {
    let t = s.trim();
    if t.is_empty() {
        0.0
    } else {
        t.parse().unwrap_or(f64::NAN)
    }
}

pub(crate) fn or(v: View, d: f64) -> f64 {
    if v.truthy() {
        num(v)
    } else {
        d
    }
}

pub(crate) fn vor(v: &Value, d: f64) -> f64 {
    if v.truthy() {
        vnum(v)
    } else {
        d
    }
}

pub(crate) fn round(x: f64) -> f64 {
    if x.is_finite() {
        (x + 0.5).floor()
    } else {
        x
    }
}

pub(crate) fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    f64::NEG_INFINITY.max(a).max(b)
}

pub(crate) fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    f64::INFINITY.min(a).min(b)
}

pub(crate) fn sin(x: f64) -> f64 {
    let ix = (x.to_bits() >> 32) as u32 & 0x7fff_ffff;
    if ix <= 0x3fe9_21fb {
        // |x| ~< π/4
        return if ix < 0x3e50_0000 { x } else { k_sin(x, 0.0, false) };
    }
    if ix >= 0x7ff0_0000 {
        return f64::NAN; // ±inf or NaN
    }
    let Some((n, y0, y1)) = rem_pio2(x, ix) else { return x.sin() };
    match n & 3 {
        0 => k_sin(y0, y1, true),
        1 => k_cos(y0, y1),
        2 => -k_sin(y0, y1, true),
        _ => -k_cos(y0, y1),
    }
}

pub(crate) fn cos(x: f64) -> f64 {
    let ix = (x.to_bits() >> 32) as u32 & 0x7fff_ffff;
    if ix <= 0x3fe9_21fb {
        return if ix < 0x3e46_a09e { 1.0 } else { k_cos(x, 0.0) };
    }
    if ix >= 0x7ff0_0000 {
        return f64::NAN; // ±inf or NaN
    }
    let Some((n, y0, y1)) = rem_pio2(x, ix) else { return x.cos() };
    match n & 3 {
        0 => k_cos(y0, y1),
        1 => -k_sin(y0, y1, true),
        2 => -k_cos(y0, y1),
        _ => k_sin(y0, y1, true),
    }
}

fn k_sin(x: f64, y: f64, tail: bool) -> f64 {
    const S: [u64; 6] = [0xbfc5_5555_5555_5549, 0x3f81_1111_1110_f8a6, 0xbf2a_01a0_19c1_61d5, 0x3ec7_1de3_57b1_fe7d, 0xbe5a_e5e6_8a2b_9ceb, 0x3de5_d93a_5acf_d57c];
    let s = S.map(f64::from_bits);
    let z = x * x;
    let w = z * z;
    let r = s[1] + z * (s[2] + z * s[3]) + z * w * (s[4] + z * s[5]);
    let v = z * x;
    if tail {
        x - ((z * (0.5 * y - v * r) - y) - v * s[0])
    } else {
        x + v * (s[0] + z * r)
    }
}

fn k_cos(x: f64, y: f64) -> f64 {
    const C: [u64; 6] = [0x3fa5_5555_5555_554c, 0xbf56_c16c_16c1_5177, 0x3efa_01a0_19cb_1590, 0xbe92_7e4f_809c_52ad, 0x3e21_ee9e_bdb4_b1c4, 0xbda8_fae9_be88_38d4];
    let c = C.map(f64::from_bits);
    let z = x * x;
    let w = z * z;
    let r = z * (c[0] + z * (c[1] + z * c[2])) + w * w * (c[3] + z * (c[4] + z * c[5]));
    let hz = 0.5 * z;
    let w = 1.0 - hz;
    w + (((1.0 - w) - hz) + (z * r - x * y))
}

fn rem_pio2(x: f64, ix: u32) -> Option<(i32, f64, f64)> {
    let to_int = 1.5 / f64::EPSILON;
    let inv_pio2 = f64::from_bits(0x3fe4_5f30_6dc9_c883);
    let pio2 = [0x3ff9_21fb_5440_0000, 0x3dd0_b461_1a62_6331, 0x3dd0_b461_1a60_0000, 0x3ba3_198a_2e03_7073, 0x3ba3_198a_2e00_0000, 0x397b_839a_2520_49c1]
        .map(f64::from_bits);
    let medium = || {
        let f_n = (x * inv_pio2 + to_int) - to_int;
        let mut r = x - f_n * pio2[0];
        let mut w = f_n * pio2[1];
        let mut y0 = r - w;
        let ex = (ix >> 20) as i32;
        if ex - ((y0.to_bits() >> 52) as i32 & 0x7ff) > 16 {
            // a second round, good to 118 bits, and a third (151 bits) where that cancels too
            for k in [2, 4] {
                let t = r;
                w = f_n * pio2[k];
                r = t - w;
                w = f_n * pio2[k + 1] - ((t - r) - w);
                y0 = r - w;
                if ex - ((y0.to_bits() >> 52) as i32 & 0x7ff) <= 49 {
                    break;
                }
            }
        }
        (f_n as i32, y0, (r - y0) - w)
    };
    // within 9π/4: n·π/2 off in one round, but near a multiple of π/2 (where that cancels)
    let near = |n: f64| {
        if x.to_bits() >> 63 == 0 {
            let z = x - n * pio2[0];
            let y0 = z - n * pio2[1];
            (n as i32, y0, (z - y0) - n * pio2[1])
        } else {
            let z = x + n * pio2[0];
            let y0 = z + n * pio2[1];
            (-n as i32, y0, (z - y0) + n * pio2[1])
        }
    };
    Some(if ix <= 0x400f_6a7a {
        if ix & 0xf_ffff == 0x9_21fb {
            medium()
        } else {
            near(if ix <= 0x4002_d97c { 1.0 } else { 2.0 })
        }
    } else if ix <= 0x401c_463b {
        if ix == 0x4012_d97c || ix == 0x4019_21fb {
            medium()
        } else {
            near(if ix <= 0x4015_fdbc { 3.0 } else { 4.0 })
        }
    } else if ix < 0x4139_21fb {
        medium()
    } else {
        return None;
    })
}

pub(crate) fn key_of(v: View) -> String {
    if v.is_num() {
        return num_str(v.num());
    }
    if v.is_str() {
        return v.str().to_string();
    }
    key_of_value(&v.value())
}

pub(crate) fn key_of_value(v: &Value) -> String {
    match v {
        Value::Undef => "undefined".into(),
        Value::Null => "null".into(),
        Value::Bool(b) => if *b { "true" } else { "false" }.into(),
        Value::Num(n) => num_str(*n),
        Value::Str(s) => s.clone(),
        Value::Arr(a) => a.iter().map(|x| if x.is_nullish() { String::new() } else { key_of_value(x) }).collect::<Vec<_>>().join(","),
        Value::Obj(_) => "[object Object]".into(),
        Value::Bytes(b) => format!("[bytes {}]", b.len()),
    }
}

pub(crate) fn object(pairs: Vec<(&str, Value)>) -> Value {
    Value::Obj(Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()))
}
