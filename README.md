# Game of Cores default factions

This repository owns the default faction programs, their shared strategy, and their presentation assets.
The programs use the public `gameofcores` SDK. Aircraft builds, house themes, names, anthems and capital layouts are faction code.

| Directory | Contents |
| --- | --- |
| `factions/` | Eight house strategies and the default program for unclaimed seats. |
| `shared/` | Strategy, aircraft builds, themes, capital layouts and art selection reused by those programs. |
| `art/` | Ground, naval and aircraft sculptures, labels and weapon artwork for the viewer. |
| `src/catalog.rs` | Founding seats, colours, newcomer names and house program assignments. |
| `examples/structures/` | Foundry, Lantern and Raiders programs for worlds with programmable structures enabled. |

Emu shares Arjuna's program with a separate ground doctrine selected by seat.
Tabula shares Atlantis through `steward`, using a broader research order.
Their presentation also follows their own seat.

| Faction | Strategy |
| --- | --- |
| Kublai Khan's Alt | Expand with mobile ground columns, passage deals and border support. |
| Arjuna's Halting Problem | Build a ground screen, buy useful technologies, then launch aircraft sorties. |
| Knights Templated | Combine faith, armored troops, artillery and local support. |
| New Atlantis | Research early technologies, trade access and field specialist units. |
| Quiet Fleet | Hold crossings, ship cargo, and pursue exposed cargo when recovery can pay. |
| The Kernel Manifesto | Expand production, spread influence and field line infantry with rifle and armored ranks. |
| Null Pointer Society | Scout with cloaked units and attack with a mobile mixed force. |
| Compute of Nations | Combine leases, hired troops, bribes, technology trade and military expansion. |
| Emu High Command | Prioritize quick ground troops and practical ground technologies. |
| Tabula Rasa | Learn a broad set of practical technologies and adapt its specialist force. |

## Armies

Every house fights in waves, with the shared code in `shared/common.rs` (`front`, `govern` and the recovery below them).

An army abroad weighs the hit points of fighting units in its target province: its own and its allies' against everyone else's.
Workers, cargo carriers, scouts and ships don't count.
When the rivals there outweigh it 1.6 to 1, its hurt units fall back to the stage province, and the rest fight on.
While that fight is lost, units at the stage hold there instead of walking in one at a time.
They go in together once they and the units still in the target outweigh the rivals 1.15 to 1.
A first attack goes in on the plan. Defenders, aircraft and ships keep their own orders.

In CPU RTS worlds, a house runs its full strategy every third second.
In the seconds between, it returns no orders, so its unit orders, plan and CPU policy stand.
It thinks every second while a unit is stalled, while the bank holds under four seconds of income, and while it recovers.
Jobs on a timer, such as propaganda, preaching and letters, run on those full-think seconds (`common::every`).
A job keyed to any other second would run only when the two line up.
In the simulations, thinking fell from about a third of the houses' spend to a fifth, and the army got the difference.

With cargo, income arrives in loads, and the army lives on the bank between deliveries.
Units stalled between loads stand at no cost until the next one.
They send a house into recovery only once its bank falls under two seconds of income.
Once the army's upkeep passes half of income, a recruit waits until the bank can pay the larger army for 30 seconds.
Buildings and plants count against the army's share of income, since the upkeep lane pays them first.
Research and influence slow down while the bank holds under 60 seconds of the army's upkeep.
The upkeep lane may always spend at least 80% of income a second.
A house disbands units for want of cycles only after 90 seconds short, a few at a time, until upkeep fits in 80% of income.
Over its unit cap, or in a world without cargo, it cuts sooner and deeper, to 55% of income.

## Export a program

```sh
node tools/source.mjs kublai /tmp/kublai.rs
```

The export contains the selected program and all shared modules in one Rust source file.
Exports fit the game's 512 KiB source limit. Players can copy and change the exported source.
The program compiles against the public game SDK. All faction helpers count toward source RAM.
The game provides 3,360 free RAM pages to every faction, covering the included preset source and the grid planner.
Compiled instructions are metered normally.
Shared code fenced `// @house` … `// @end` goes into the house programs only. The default program leaves out the sea
escorts this way, but carries the same power grid planner as the founders.
Both the Rust package and the Node exporter produce the same source; `npm test` checks every program.

Every house supports physical CPU cargo when `world.logistics` is present.
Miners unload locally, ground carriers connect depots, and freighters carry cargo between docks.
Carriers leave exhausted depots with partial loads and prioritize useful delivery trips.
Quiet Fleet recovers visible wreck cargo and returns it through the capital's dock.
It retires an empty corsair when no worthwhile target or nearby wreck remains.
Every other house that ships cargo by sea keeps one armed escort with its freighters, and a second once it runs two
freighters or a raider shows at sea. The second sails as a privateer off the nearest rival dock.
The same programs retain their earlier mining policy in saved worlds without logistics.

Every house plans a power grid when `world.energy` is present. It orders plants and batteries for its own doctrine,
sets which provinces go dark first in a shortfall, and keeps a battery reserve. After `transmission`, every house can
offer native surplus from a linked grid island to a short neighbour. Compute of Nations and unclaimed default seats offer from 1.5 MW
spare. Other houses wait for 3 MW. Incoming buyer requests must fit the seller's route-specific spare power.
When a province with plant capacity is invaded, its house assigns a larger local guard, up to four extra units.
[The grid planner](docs/energy.md) gives each doctrine and rule.

Export a structure example with `node tools/source.mjs foundry /tmp/foundry.rs`.
The names `lantern` and `raiders` select the other two programs.
These examples retain their measured strategy helpers and use this repository's shared presentation helpers.
See [the structure examples](examples/structures/README.md) for engine requirements and recorded results.

## Update the game

Commit changes here, then run this command in a `gameofcores` checkout:

```sh
node tools/sync-default-factions.mjs ../gameofcores-default-factions HEAD
npm run build:founders
npm run build:data
npm run build:wasm
npm run build:lessons
npm run build:reference
```

The sync command installs a commit into `crates/default-factions` and its art into `site/game/default-factions`.
The lock file records the repository, commit and file hashes. Edit this repository and sync again to update that snapshot.
Ordinary game builds use the installed snapshot without fetching another repository.

Run the game's Rust, viewer and copy checks before shipping a new snapshot.

The default programs also build when a CPU RTS world enables structures.
Their shared policy reads `me.rts.structureCosts` before issuing building orders.
Both the uniform default program and the separate house programs use each seat's design.
See [default buildings](docs/buildings.md) for their priorities and spending limits.

## Cargo and Flight checks

Run `npm run test:behavior -- ../gameofcores` against a game checkout with its public SDK.
The tests exercise route selection, load reservations, idle retirement and Arjuna's research policy.
They run the exported faction code. Full game simulations and metered callback checks still run in the game repository.

Freighters reserve reachable loads and use a receiving dock connected to the capital.
An empty freighter retires after two minutes without movement. Loaded, hired and passenger ships are preserved.
Recruitment requires a reachable outbound load and waits after an idle retirement.
Arjuna pursues Flight after mining, Dash and Ranged. A funded recovery crew can continue that research while the army planner waits.

Aircraft controllers use visible `me.rts.air.contacts` entries to intercept rival wings. They use `me.rts.air.decks` to board their own afloat decks while cooling. During recovery, each fielded aircraft type gets 2,400 controller cycles every third second. Recovery gives fabrication zero, so the controllers land aircraft while production waits.

Think shares convoy locations and combat estimates through plan RAM. Ground units carry combat estimates on unit page 93 and discard them after 120 seconds. Convoy guards use locations observed within the last 12 seconds.

Run the authored house fixtures with `node tools/test-strategies.mjs ../gameofcores`. They exercise exported programs against that checkout's SDK.
