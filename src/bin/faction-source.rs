fn main() {
    let name = std::env::args().nth(1).expect("usage: faction-source <name>");
    let source = gameofcores_default_factions::FOUNDERS.iter().chain(gameofcores_default_factions::EXAMPLES.iter()).find(|(n, _)| *n == name).expect("known faction").1;
    print!("{source}");
}
