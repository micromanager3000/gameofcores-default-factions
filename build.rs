use std::{env, fs, path::PathBuf};
fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let mut shared = String::from("\n#[allow(dead_code, unused_imports)]\nmod shared {\n");
    let mut example_shared = shared.clone();
    for name in ["air", "capital", "common", "compat", "theme", "unit_art"] {
        let path = format!("shared/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        let module = format!("pub mod {name} {{\n{}\n}}\n", fs::read_to_string(path).expect("shared source"));
        shared.push_str(&module);
        if name != "common" {
            example_shared.push_str(&module);
        }
    }
    shared.push_str("}\n");
    example_shared.push_str("}\n");
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
    let mut examples = String::new();
    for name in ["foundry", "lantern", "raiders"] {
        let path = format!("examples/structures/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        let source = fs::read_to_string(path).expect("example source") + &example_shared;
        fs::write(out.join(format!("{name}.rs")), source).expect("assembled example");
        examples.push_str(&format!("({name:?}, include_str!(concat!(env!(\"OUT_DIR\"), \"/{name}.rs\"))),\n"));
    }
    fs::write(out.join("examples.rs"), format!("pub const EXAMPLES: [(&str, &str); 3] = [\n{examples}];\n")).expect("example table");
}
