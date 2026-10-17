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

An army abroad compares the hit points of fighting units in its target province.
Workers, cargo carriers, scouts and ships don't count toward either side.
When rivals there outweigh the army 1.6 to 1, it falls back to its stage province.
At the stage, reinforcements wait until they and the units still in the target outweigh the rivals 1.15 to 1.
They then go back in together as one wave. Defenders, aircraft and ships keep their own orders.

In CPU RTS worlds, a house runs its full strategy every third second.
In the seconds between, it returns no orders, so its plans and CPU policy stay in force.
It still thinks every second when a unit is stalled, the bank holds less than four seconds of income or it is leaving recovery.
It also thinks every second while its costliest recent think passes 40% of income, so a think after a rest still fits in a second.

With cargo, income arrives in loads, so the army lives on the bank between deliveries.
A house recruits only while the bank can pay the larger army's upkeep for 30 seconds.
Research and influence slow down when the bank holds less than 60 seconds of army upkeep.
The upkeep lane may spend at least 80% of income in a second.
Units stalled between deliveries stand until the next load arrives.
A house disbands up to four units at a time for want of cycles after 90 seconds short.
Over its unit cap, it disbands sooner and deeper.

## Export a program

```sh
node tools/source.mjs kublai /tmp/kublai.rs
```

The export contains the selected program and all shared modules in one Rust source file.
Exports fit the game's 512 KiB source limit. Players can copy and change the exported source.
The program compiles against the public game SDK. All faction helpers count toward source RAM.
The game provides 3,072 free RAM pages to every faction, covering the newly included preset source.
Compiled instructions are metered normally.
Both the Rust package and the Node exporter produce the same source; `npm test` checks every program.

Every house supports physical CPU cargo when `world.logistics` is present.
Miners unload locally, ground carriers connect depots, and freighters carry cargo between docks.
Carriers leave exhausted depots with partial loads and prioritize useful delivery trips.
Quiet Fleet recovers visible wreck cargo and returns it through the capital's dock.
It retires an empty corsair when no worthwhile target or nearby wreck remains.
The same programs retain their earlier mining policy in saved worlds without logistics.

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
