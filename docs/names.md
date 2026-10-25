# Names

## Name: **Game of Cores**

The game is called **Game of Cores** (2026-10-01). Factions play for the world's cores: the CPU each province holds.
Cores make cycles; cycles run code and buy memory; credits buy seats. There is no themed vocabulary: factions, pacts,
messages, the Crown, the Pot and epochs keep plain names.

**Caveat:** "Game of …" plays on an HBO mark, so it needs a trademark search before any public launch.

### The house vocabulary

| Term | Meaning |
|---|---|
| **Epoch** | One lifetime of the world (also a nod to Unix epoch time) |
| **Cycles (⌬)** | In-game compute: WASM fuel per tick |
| **Credits** | Prepaid, non-redeemable money (◆ is the in-game currency, a tax on a faction's land) |
| **The Slicer** | The scheduler that meters cycles (from Tierra) |
| **The Reaper** | Culls units over the memory cap (from Tierra) |
| **The Core / the Periphery** | The rich, contested center vs. the quiet outer ring (from Foundation) |
| **Crises** | Scheduled, publicly counted-down shocks (winter, new lands) |
| ~~The Mule~~ | Cut in v0.2 (too close to Foundation); collapse is now emergent from governance and revolts |
| **Seat** | A faction position, paid once in credits and held for the epoch (after Card's *Capitol*, where positions were bought and sold) |
| **Pact** (was *Coalition*) | Factions at peace. Up to 3 may claim the Crown together, and the Pot pays by their agreed split, else by cores held, so partners are tempted to betray |
| **The Pot** | What players paid in (a fifth of seat money, part of every tip), won by taking the Crown |
| ~~Heat Death~~ | Cut in v0.2: an epoch ends only when someone takes the Crown |
| ~~Cyclarch~~ | Proposed title of an epoch's winner; the game says who took the Crown, and History lists them |

> IP note: Asimov's proper nouns (Seldon, Trantor, Terminus, the Mule) are
> fine as internal nods but should be replaced before any commercial launch.

## Faction, unit and place names (2026-10-04, third founder set 2026-10-05)

Mechanics keep plain names (cores, cycles, credits, RAM). Factions, units, provinces and signage may be flavourful. The
engine never reads a name: a founder is one of the seats a world starts with (six by default in the engine, factions
0–5; the shared world starts with eight unless an admin sets another number) with a colour
and a bot, and `look()` hears `info.seat` and `info.founder` (and `think()` `me.seat`) rather than matching a name.
Eleven founders have a house style; seats 6–8 add the later founders, seat 9 preserves Tabula Rasa, seat 10 adds
Highway to the Danger Node, and past eleven, founders take names and capitals from the newcomer pool, in order.

| Seat | Bot | Founder | Board / mark | Capital | Allusion |
|---|---|---|---|---|---|
| 0 | conqueror | Kublai Khan's Alt | Kublai / KA | Xanadu | Kublai's main capital was Khanbaliq; Shangdu, Coleridge's Xanadu, was the alt. Typhoons sank his invasion fleets (1274, 1281): the main sank, the alt rides on |
| 1 | expander | Arjuna's Halting Problem | Arjuna / AP | Kurukshetra | The Bhagavad Gita, sung on the field of Kurukshetra as the battle opens: Arjuna sets down his bow and halts, and eighteen chapters later his charioteer has him running. Will he halt? Undecidable, until he does |
| 2 | templar | Knights Templated | Knights / KT | Sanctum | A compute play on the Knights Templar: compiled orders, paid armor, white mantles, crimson crosses and gold ballista bolts |
| 3 | adaptive | New Atlantis | Atlantis / At | Bensalem | Bacon's island and its research college, Salomon's House: Merchants of Light, a rank called Lamps, and an oath of secrecy over what to publish. The LLM-written house agent is the college |
| 4 | diplomat | Quiet Fleet | Quiet / Q | Longjiang | Zheng He's treasure fleet, built at Longjiang: seven voyages (1405-33) to some thirty courts, a giraffe for the emperor, then the court stopped sending it |
| 5 | agitator | The Kernel Manifesto | Kernel / KM | Reading Room | Marx in the British Museum's Round Reading Room, writing *Capital*: volume 1 shipped in 1867, volumes 2 and 3 after him, finished by Engels. The name ports *The Communist Manifesto* to the kernel. Workers of all LANs, unite. The seat dresses Soviet under a red banner of its own (a gold star in a cogwheel), with the Red Army, Stakhanovite miners and a marching choir. In its capital, Marx reads the Manifesto and *Capital* ported to compute: programs of the world, unite |
| 6 | conqueror | Null Pointer Society | Null / N | Address Zero | The null reference, added to ALGOL W in 1965 because it was so easy, later called a billion-dollar mistake by the man who added it |
| 7 | diplomat | Compute of Nations | Compute / C | Kirkcaldy | Adam Smith at home in Kirkcaldy (1767–73), writing *The Wealth of Nations*: the pin factory, the division of labour and the invisible hand |
| 8 | expander | Emu High Command | Emu / E | Campion | The side that won the Great Emu War (Campion, 1932): machine guns against 20,000 emus, and the emus held the wheat |
| 9 | steward | Tabula Rasa | Tabula / T | Flux | Condillac's statue (*Traité des sensations*, 1754), born blank and given one sense at a time: offered a rose, it is, to itself, only the smell of a rose. Born blank, trained on one rose |
| 10 | aviator | Highway to the Danger Node | Highway / H | Miramar | *Top Gun* (1986) and its anthem, "Danger Zone", ported to the network: the Navy Fighter Weapons School at NAS Miramar, Fightertown, where the top one percent fly. Callsigns from the film, deck crews in coloured jerseys, a volleyball net on the flight line and one bandit, The Kernel Manifesto: red stars over the Reading Room, never a pact or a deal. The need for speed, measured in cycles |

Each founder is a true story with a modern twist: a ruler's spare capital, a reluctant archer, a philosopher's statue,
a book's island, a fleet, a book that never finished shipping, a famous mistake, a pin factory and an undefeated mob of
emus. Its first capital takes the story's own place. Each house style lives once, in the default faction repository's `shared/theme.rs` module
(docs/sdk.md): a map look whose Wonder shows at every budget, an anthem in its seat's key, a 3D capital, unit names
and a war line. The 3D capitals are Coleridge's poem as a site plan; Kurukshetra at dawn, nothing built, where Krishna
talks Arjuna round between the two armies to a dawn raga; the garden at Flux with Condillac's statue; Salomon's House with Vaucanson's duck; the treasure shipyard
with the qilin; the Round Reading Room among crystal spires of unrest, where Marx reads the Manifesto and *Capital* ported to compute; a walled lodge round a great zero; Kirkcaldy
harbour with a giant pin and an invisible hand; the Wheat Front with its emu-egg memorial; and Fightertown, a runway
lit at dusk between two hangars, where an instructor briefs a nugget under the threat board. The house bots and the
default program pick it by seat, so a founder that renames itself keeps its style. The first six share no
architecture, Wonder shape, banner pattern, anthem key, mode, lead or drum pattern; seats 6–8 each have a lead and a
mode of their own (chip and phrygian, organ and dorian, brass and mixolydian) and borrow the rest. Seat 8's anthem is in
D, like seat 0's: the page's stingers have eight keys. Seat 10's anthem is in D too, on a brass lead, and it alone
uses the blues mode.

- **Board names.** On the board a faction goes by its first word that isn't "The", "Free", "New" or "Old", without a
  possessive (`'s` or a bare `'`, straight or curly): "New Atlantis" reads Atlantis, "Arjuna's Halting Problem" Arjuna.
  Lowercase names keep their spelling (theseus-v4, a.out).
- **Marks.** Next to its board name a faction shows a mark: its initial in capitals, or, where two factions' initials
  match or read alike (E/F, B/P/R, O/Q/C/G, U/V, I/J), two letters: the initials of its first and last significant
  words, a CamelCase word's capitals, else letters of its board name (`monograms()`, site/game/js/marks.js). The
  founders share at most one initial (Arjuna and Atlantis: AP, At) and no look-alike pair: the six's marks are K AP T
  At Q D, and the nine's add N C E. Any two founder or newcomer names whose initials match or read alike have different
  second marks or board names, so no board ever needs a numbered mark (Quiet Fleet shows QF next to Ozymandias' OC;
  Compute of Nations CN next to Clever Hans' CH).
- **Lengths.** Board names are 3–14 characters (`forkbomb.wasm`, `final_v2_FINAL`). Names are ASCII, 3–28
  characters, and founders at most 26 (a chip's width); longer names ellipsize in chips and rows. Capitals are at
  most 16 characters, unique, without apostrophes, and never a province name. First words never repeat.
- **Newcomers** (the factions a growing world adds, and founders past eleven) draw from 24 names, in order, with their
  capitals and the house program's generic look. Two are history with a twist (Ozymandias' Legacy Code, Hammurabi's Terms of Service);
  22 are techy:

  | Newcomer | Capital | Newcomer | Capital | Newcomer | Capital |
  |---|---|---|---|---|---|
  | Ozymandias' Legacy Code | Thebes | Hammurabi's Terms of Service | Babylon | forkbomb.wasm | Process Table |
  | Terrarium | Bell Jar | theseus-v4 | Plank Yard | Golden Hoard | Strongroom |
  | Gossip Protocol | Grapevine | Antikythera | Shipwreck Reef | Mill & Store | Card Hopper |
  | Ropeweavers | Rope Walk | Little-Endians | Lilliput | Big-Endians | Blefuscu |
  | Avian Carriers | Pigeon Loft | Russell's Teapot | Elliptic Orbit | Tulipmania | Haarlem |
  | Sneakernet | Station Wagon | Blinkenlights | Machine Room | Magic Smoke | Smoke Test |
  | Imp Ring | Fairy Circle | Clever Hans | Counting Yard | Copper Accord | Magan |
  | a.out | Murray Hill | final_v2_FINAL | Downloads | untitled-3 | New Folder |

  A newcomer runs the default program (in a local game, the house program of the founder in its slot) and wears that
  house's style in its own colour; its capital takes its own name, and it has no anthem or war line.
- **Reserves,** kept for later (not drawn; check them against the marks before using one): Berthier's Rabbits
  (Tilsit), Brahe's Moose (Landskrona), Pykrete Admiralty (Patricia Lake), Pineapple Renters (Dunmore Park), Pavlov's
  Push Notification (Koltushi), Wright Brothers' MVP (Kitty Hawk), Jules Verne's Vaporware (Nantes), Quixote's Bug
  Bounty (La Mancha), Gresham's Grasshopper (Lombard Street), Ruskin's Road Crew (Ferry Hinksey), Icarus' Thermal
  Throttle (Icaria) and Treasure Fleet. Null Island (Zero Zero) gave its first word to Null Pointer Society. The two
  founders the third set replaced: IronPillar (Pataliputra), Ashoka's edict pillars; Trotsky's Lament (Prinkipo), the
  prophet of permanent revolution in exile. The history pool the third set retired, with capitals: Archimedes' Do Not
  Disturb (Syracuse), Bjarni's Wrong Turn (Not Vinland), The Byzantine Generals (Constantinople), Canute's Tide Office
  (Bosham), The Cospaia Loophole (Cospaia), Diogenes' Prove You're Human (Sinope), Gutenberg's Copy-Paste (Mainz),
  Herschel's Bat-Men (Feldhausen), Jazari's Elephant Clock (Amida), Merlin's Roller Skates (Carlisle House), Moresnet
  Esperanto Club (Amikejo), Nanni's One-Star Review (Dilmun), Norton's Imperial Bonds (Yerba Buena), Penelope's Ctrl-Z
  (Ithaca), The Pig War Accord (Griffin Bay), Quipu Knot Auditors (Cusco), Redonda Royal Court (Redonda), Sisyphus' Cron
  Job (Ephyra), Shackleton's Out-of-Office (Stromness), Watkin's Tower (Wembley Park), Xerxes' Blameless Postmortem
  (Abydos), Yap's Sunken Ledger (Stone Money Bay), Voltaire's Walled Garden (Ferney), Lovelace's Objection (Ockham) and
  Zheng He's Giraffe (Malindi).
- **Provinces.** `world::NAMES` holds 140 names: 100 for the land a world starts with (Venora, Mutex Marsh,
  Deadlock Pass, Nandgate, Diode Dunes, Quine Quarry…) and 40 for land that rises later (Detached Head, Segfault Scarp,
  Honeypot Heath…). Each map seed shuffles both pools from a stream of its own, so every epoch's planet has different
  names and names never move mines or forges. The first province to rise from the sea is **Here Be Daemons**. Past
  140, names are compounds such as Bytemere or Cron Reach. A bought open seat is "Free" plus its capital's name.
- **Continents, the ocean and seas** (2026-10-05). Every planet has 2 to 4 continents and a few islands, with open
  sea at least two sites wide between any two of them (2026-10-06). Islands have no names of their own. `names.rs` is
  the grab bag for them: continents (Turingia, Userland, Big Iron, Old Motherboard, Hyperborea…), the planet's
  largest water (Carrier Ocean, Whitespace Ocean, Great Null…) and every other sea of 3 or more sites (Null Sea,
  Gulf of Gotos, Sea of Stray Pointers, Cable Sargasso…). Each pool is shuffled by the map seed on its own stream.
  The map letters them, quieter than the factions' names; the engine never reads them.
- **Checked against:** real places and streets, trademarks, game titles and fictional characters (for example
  Firebrand, Tinderbox, Grey Goo, Kernel Panic, Kindle and Tinder were all avoided). The full reasoning is in the naming
  panel's notes.

---

## Shortlist

| Name | Why it works | Watch-outs |
|---|---|---|
| **Substrate** | Tierra organisms live on a substrate, and so does silicon | Parity/Polkadot "Substrate" framework |
| **Terminus** | Foundation's edge-of-galaxy planet and the terminal you code in | Very common word |
| **Epoch** | Rounds are epochs, plus Unix time | Generic, hard to search |
| **Overclock** | StarCraft energy, and a possible burst mechanic | Hardware-brand feel |
| **Timeshare** | CPU time-sharing, cheeky | Vacation-condo baggage |
| **Cyclone** | Cycles + clones (replication) + a storm eating the map | Heavily used product name |
| **The Soup** | Tierra's "soup" of competing code | Too cute for a strategy game |
| **Periphery** | Start at the edge, fight toward the Core | Soft, hard to own |
| **Cyclarch** | "Ruler of cycles" | Better as the winner's title |

## Also-rans
Slicer · Reaper · Heatdeath · Kernel Wars · Opcode · Quorum · Primordium ·
Clockwork Dominion · Everclock · Hexcore · Tickfall · Galactic Timeshare ·
Instruction Set · Silicon Seldon · Empire of Cycles (a good tagline)

## Taglines
- *Write code. Seize the cycles. Rule the epoch.*
- *Your code. Real CPU. One galaxy.*
- *Every empire ends. The code survives.*
- *Psychohistory, compiled.*
