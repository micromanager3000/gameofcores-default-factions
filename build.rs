use std::{env, fs, path::PathBuf};
fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let mut shared = String::from("\n#[allow(dead_code, unused_imports)]\nmod shared {\n");
    for name in ["air", "capital", "common", "compat", "theme", "unit_art"] {
        let path = format!("shared/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        shared.push_str(&format!("pub mod {name} {{\n{}\n}}\n", fs::read_to_string(path).expect("shared source")));
    }
    shared.push_str("}\n");
    let names = ["kublai", "arjuna", "templar", "atlantis", "fleet", "manifesto", "nullptr", "nations", "default"];
    let mut constants = String::new();
    for name in names {
        let path = format!("factions/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        let source = fs::read_to_string(path).expect("faction source") + &shared;
        fs::write(out.join(format!("{name}.rs")), source).expect("assembled source");
        constants.push_str(&format!("({name:?}, include_str!(concat!(env!(\"OUT_DIR\"), \"/{name}.rs\"))),\n"));
    }
    fs::write(out.join("programs.rs"), format!("pub const FOUNDERS: [(&str, &str); 9] = [\n{constants}];\npub const DEFAULT_BOT: &str = FOUNDERS[8].1;\n")).expect("program table");
}
