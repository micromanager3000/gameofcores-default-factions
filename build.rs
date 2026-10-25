use std::{env, fs, path::PathBuf};
/// Whether program `name` wears founding style `style` (a seat of shared/theme.rs): the seats it founds and its home
/// style. A house program leaves the other styles out (a program pays for its length); the default program and the
/// structure examples wear every style. (tools/source.mjs does the same.)
fn wears(name: &str, style: usize) -> bool {
    let styles: &[usize] = match name {
        "kublai" => &[0],
        "arjuna" => &[0, 1, 8, 10],
        "templar" => &[2],
        "atlantis" => &[0, 3, 9],
        "fleet" => &[0, 4],
        "manifesto" => &[0, 5],
        "nullptr" => &[0, 6],
        "nations" => &[0, 7],
        _ => return true,
    };
    styles.contains(&style)
}
/// Shared code for program `name`: the default program leaves out code fenced `// @house` … `// @end`, but includes
/// the same power planner as every founder; a program that doesn't wear style N (`wears`) leaves out code fenced
/// `// @style N` … `// @end`, and writes the rest of an `// @else` line in it in its place (a comment otherwise, so the
/// file compiles as it stands, wearing every style).
fn fence(text: &str, name: &str) -> String {
    let mut drop = false;
    let mut out: Vec<String> = Vec::new();
    for line in text.split('\n') {
        let t = line.trim_start();
        if t.starts_with("// @house") {
            drop = name == "default";
        } else if let Some(n) = t.strip_prefix("// @style ") {
            drop = !wears(name, n.split_whitespace().next().and_then(|n| n.parse().ok()).expect("a style number"));
        } else if let Some(rest) = t.strip_prefix("// @else ") {
            if drop { out.push(format!("{}{rest}", &line[..line.len() - t.len()])); }
            continue;
        } else if t.starts_with("// @end") {
            if !drop { out.push(line.to_string()); }
            drop = false;
            continue;
        }
        if !drop { out.push(line.to_string()); }
    }
    out.join("\n")
}
fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let mut modules = Vec::new();
    for name in ["air", "capital", "common", "compat", "energy", "structures", "theme", "unit_art"] {
        let path = format!("shared/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        modules.push((name, fs::read_to_string(path).expect("shared source")));
    }
    // Structure examples carry no strategy: no common, structures or energy module.
    let shared = |program: &str, example: bool| {
        let mut s = String::from("\n#[allow(dead_code, unused_imports)]\nmod shared {\n");
        for (name, text) in &modules {
            if example && matches!(*name, "common" | "structures" | "energy") { continue; }
            s.push_str(&format!("pub mod {name} {{\n{}\n}}\n", fence(text, program)));
        }
        s
    };
    // Each house's buildings, a line a house in seat order: a house program carries only its own (a program pays for
    // its length in memory), the default program every house's, since it wears the seat's style. Arjuna's program also
    // carries seat 10's, since it flies for Highway to the Danger Node there.
    println!("cargo:rerun-if-changed=shared/buildings.txt");
    let arts: Vec<String> = fs::read_to_string("shared/buildings.txt").expect("building art").lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#')).map(|l| format!("\"{l}\"")).collect();
    let names = ["kublai", "arjuna", "templar", "atlantis", "fleet", "manifesto", "nullptr", "nations", "default"];
    let mut constants = String::new();
    for (seat, name) in names.into_iter().enumerate() {
        let path = format!("factions/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        let (own, pick) = match name {
            "default" => (arts.join(", "), "style.min(a.len() - 1)"),
            "arjuna" => (format!("{}, {}", arts[seat], arts[10]), "usize::from(style == 10)"),
            _ => (arts[seat].clone(), "style.min(a.len() - 1)"),
        };
        let art = format!("pub fn art(style: usize) -> String {{ let a = [{own}]; a[{pick}].to_string() }}\n}}\n");
        let source = fs::read_to_string(path).expect("faction source") + &shared(name, false) + &art;
        fs::write(out.join(format!("{name}.rs")), source).expect("assembled source");
        constants.push_str(&format!("({name:?}, include_str!(concat!(env!(\"OUT_DIR\"), \"/{name}.rs\"))),\n"));
    }
    fs::write(out.join("programs.rs"), format!("pub const FOUNDERS: [(&str, &str); 9] = [\n{constants}];\npub const DEFAULT_BOT: &str = FOUNDERS[8].1;\n")).expect("program table");
    let mut examples = String::new();
    for name in ["foundry", "lantern", "raiders"] {
        let path = format!("examples/structures/{name}.rs");
        println!("cargo:rerun-if-changed={path}");
        let source = fs::read_to_string(path).expect("example source") + &shared(name, true) + "}\n";
        fs::write(out.join(format!("{name}.rs")), source).expect("assembled example");
        examples.push_str(&format!("({name:?}, include_str!(concat!(env!(\"OUT_DIR\"), \"/{name}.rs\"))),\n"));
    }
    fs::write(out.join("examples.rs"), format!("pub const EXAMPLES: [(&str, &str); 3] = [\n{examples}];\n")).expect("example table");
}
