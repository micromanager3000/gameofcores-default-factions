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

Emu High Command uses Arjuna's strategy. Tabula Rasa uses Atlantis through the `steward` assignment.
Their own house presentation is selected by seat. Those assignments match the existing game.

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
Quiet Fleet also recovers visible wreck cargo. The same programs retain their earlier mining policy in saved worlds without logistics.

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
