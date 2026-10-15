# Game of Cores default factions

This repository owns the default faction programs, their shared strategy, and their presentation assets.
The programs use the public `gameofcores` SDK. Aircraft builds, house themes, names, anthems and capital layouts are faction code.

| Directory | Contents |
| --- | --- |
| `factions/` | Eight house strategies and the default program for unclaimed seats. |
| `shared/` | Strategy, aircraft builds, themes, capital layouts and art selection reused by those programs. |
| `art/` | Ground, naval and aircraft sculptures, labels and weapon artwork for the viewer. |
| `src/catalog.rs` | Founding seats, colours, newcomer names and house program assignments. |

Emu High Command uses Arjuna's strategy. Tabula Rasa uses Atlantis through the `steward` assignment.
Their own house presentation is selected by seat. Those assignments match the existing game.

## Export a program

```sh
node tools/source.mjs kublai /tmp/kublai.rs
```

The export contains the selected program and shared strategy in one Rust source file.
Exports fit the game's 256 KiB source limit. Players can copy and change the exported source.
The program links the public game SDK and this repository's `default_factions` library.
The library owns aircraft definitions, themes, capitals and art selection. It is built from `library.rs`.

Build the library beside an SDK built by the same Rust toolchain:

```sh
node tools/build-library.mjs ../gameofcores/target/wasm32-unknown-unknown/release
```

The compiler links it as `--extern default_factions=<directory>/libdefault_factions.rlib`.
Shared strategy remains billed source. Preset library source retains its previous exemption from code RAM.
Compiled instructions from both libraries are metered normally.
Both the Rust package and the Node exporter produce the same source; `npm test` checks every program.

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
