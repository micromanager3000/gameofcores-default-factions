//! Default faction identities and program assignments.

pub const FACTION_DEFS: [(&str, &str, &str, &str); 10] = [
    ("Kublai Khan's Alt", "#ff5a5f", "conqueror", "Xanadu"),
    ("Arjuna's Halting Problem", "#4d9bff", "expander", "Kurukshetra"),
    ("Knights Templated", "#f0e6c8", "templar", "Sanctum"),
    ("New Atlantis", "#ffb347", "adaptive", "Bensalem"),
    ("Quiet Fleet", "#b18cff", "diplomat", "Longjiang"),
    ("The Kernel Manifesto", "#ff6fb5", "agitator", "Reading Room"),
    ("Null Pointer Society", "#2dd4bf", "saboteur", "Address Zero"),
    ("Compute of Nations", "#e8e36a", "merchant", "Kirkcaldy"),
    ("Emu High Command", "#19d1ff", "expander", "Campion"),
    ("Tabula Rasa", "#9be15d", "steward", "Flux"),
];

pub const EXTRA_COLORS: [&str; 18] = [
    "#2dd4bf", "#e8e36a", "#19d1ff", "#7ee0a1", "#ffa27a", "#b2c6ff", "#c37322", "#ff5c1a", "#3aa7b0", "#9d4edd",
    "#7ca145", "#b5838d", "#8e9aaf", "#b09d48", "#45a17c", "#cdb4db", "#ee55ee", "#ffb2be",
];

pub const NEWCOMERS: [(&str, &str); 24] = [
    ("Ozymandias' Legacy Code", "Thebes"),
    ("Hammurabi's Terms of Service", "Babylon"),
    ("forkbomb.wasm", "Process Table"),
    ("Terrarium", "Bell Jar"),
    ("theseus-v4", "Plank Yard"),
    ("Golden Hoard", "Strongroom"),
    ("Gossip Protocol", "Grapevine"),
    ("Antikythera", "Shipwreck Reef"),
    ("Mill & Store", "Card Hopper"),
    ("Ropeweavers", "Rope Walk"),
    ("Little-Endians", "Lilliput"),
    ("Big-Endians", "Blefuscu"),
    ("Avian Carriers", "Pigeon Loft"),
    ("Russell's Teapot", "Elliptic Orbit"),
    ("Tulipmania", "Haarlem"),
    ("Sneakernet", "Station Wagon"),
    ("Blinkenlights", "Machine Room"),
    ("Magic Smoke", "Smoke Test"),
    ("Imp Ring", "Fairy Circle"),
    ("Clever Hans", "Counting Yard"),
    ("Copper Accord", "Magan"),
    ("a.out", "Murray Hill"),
    ("final_v2_FINAL", "Downloads"),
    ("untitled-3", "New Folder"),
];

pub const HOUSE: [(&str, &str); 9] = [
    ("conqueror", "kublai"),
    ("expander", "arjuna"),
    ("templar", "templar"),
    ("adaptive", "atlantis"),
    ("diplomat", "fleet"),
    ("agitator", "manifesto"),
    ("saboteur", "nullptr"),
    ("merchant", "nations"),
    ("steward", "atlantis"),
];
