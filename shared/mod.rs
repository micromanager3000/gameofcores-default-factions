pub mod air;
pub mod capital;
pub mod common;
pub mod compat;
pub mod theme;
pub mod unit_art;
pub mod structures;

/// Every house's building art, as the default program carries it (a house program carries only its own line).
pub fn art(style: usize) -> String {
    let a: Vec<&str> = include_str!("buildings.txt").lines().filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
    a[style.min(a.len() - 1)].to_string()
}
