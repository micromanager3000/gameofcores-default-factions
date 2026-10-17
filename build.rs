use std::{env, fs, path::PathBuf};
/// The default program runs plain, since a program pays for its length in memory: it leaves out shared code fenced
/// `// @house` … `// @end`, and carries shared/energy_plain.rs as its `energy` module. (tools/source.mjs does the same.)
fn fence(text: &str, house: bool) -> String {
    let mut drop = false;
    let mut out: Vec<&str> = Vec::new();
    for line in text.split('\n') {
        let t = line.trim_start();
        if t.starts_with("// @house") {
            drop = !house;
            if house { out.push(line); }
        } else if t.starts_with("// @end") {
            if !drop { out.push(line); }
            drop = false;
        } else if !drop {
            out.push(line);
        }
    }
    out.join("\n")
}
fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let mut shared = String::from("\n#[allow(dead_code, unused_imports)]\nmod shared {\n");
    let mut plain = shared.clone();
    let mut example_shared = shared.clone();
    for name in ["air", "capital", "common", "compat", "energy", "structures", "theme", "unit_art"] {
        let path = format!("shared/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        let text = fs::read_to_string(path).expect("shared source");
        let module = format!("pub mod {name} {{\n{}\n}}\n", fence(&text, true));
        shared.push_str(&module);
        let own = if name == "energy" {
            println!("cargo:rerun-if-changed=shared/energy_plain.rs");
            fs::read_to_string("shared/energy_plain.rs").expect("plain grid")
        } else {
            text
        };
        plain.push_str(&format!("pub mod {name} {{\n{}\n}}\n", fence(&own, false)));
        if name != "common" && name != "structures" && name != "energy" {
            example_shared.push_str(&module);
        }
    }
    example_shared.push_str("}\n");
    // Each house's buildings, a line a house in seat order: a house program carries only its own (a program pays for
    // its length in memory), the default program every house's, since it wears the seat's style.
    println!("cargo:rerun-if-changed=shared/buildings.txt");
    let arts: Vec<String> = fs::read_to_string("shared/buildings.txt").expect("building art").lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#')).map(|l| format!("\"{l}\"")).collect();
    let names = ["kublai", "arjuna", "templar", "atlantis", "fleet", "manifesto", "nullptr", "nations", "default"];
    let mut constants = String::new();
    for (seat, name) in names.into_iter().enumerate() {
        let path = format!("factions/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        let own = if name == "default" { arts.join(", ") } else { arts[seat].clone() };
        let art = format!("pub fn art(style: usize) -> String {{ let a = [{own}]; a[style.min(a.len() - 1)].to_string() }}\n}}\n");
        let shared = if name == "default" { &plain } else { &shared };
        let source = fs::read_to_string(path).expect("faction source") + shared + &art;
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
