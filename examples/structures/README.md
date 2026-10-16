# Programmable structure examples

These programs build and operate structures in CPU RTS worlds with programmable structures enabled.
They are opt-in exports. The founding seat assignments remain in `src/catalog.rs`.

| Export | Faction | Behavior |
| --- | --- | --- |
| `foundry` | Foundry Compact | Replaces mine crews with paid extractors and deploys the freed capacity. |
| `lantern` | Lantern Covenant | Places courts near armies and assigns services to eligible local targets. |
| `raiders` | Raedline Raiders | Builds armed bastions and raids rival territory. |

```sh
node tools/source.mjs foundry /tmp/foundry.rs
node tools/source.mjs lantern /tmp/lantern.rs
node tools/source.mjs raiders /tmp/raiders.rs
```

Use the game's `codex/programmable-structures` engine branch with structures enabled.
The exported sources require the 512 KiB source allowance and 3,072 free RAM pages from `codex/extract-default-factions`.
They support the SDK after the faction helpers moved into this repository.
The programs keep their own strategy helpers. Their presentation uses `shared/air`, `shared/capital` and `shared/theme`.

## Recorded comparison

The [game report](https://github.com/micromanager3000/gameofcores/blob/c31a88c28ec5d0abf77daef74a3093f4a8990bc9/docs/reports/structures/index.html) preserves the experiments and their limits.
The final comparison used six fresh maps, three seat rotations and five construction conditions: 90 worlds.
Foundry gained 4.4 percentage points of core share against its own construction-disabled counterpart.
Lantern gained 2.5 points. Both gains were positive on four of the six map averages.
These results are specific to the frozen engine, compiler and source recorded in that report.
Placement refusals and adverse map results remain in the evidence.

`provenance.json` identifies the exact measured source for each program.
Only helper imports and capital-helper paths changed when these programs moved here.
The exporter appends the shared presentation source. It omits the shared strategy module because each example carries its own.
This changes source size and may change execution charges.
The new exports have not repeated the 90-world comparison; use the report's frozen sources to reproduce it.

`validation.json` records six successful WebAssembly compilations with Rust 1.97.0: all three exports against both SDK layouts.
The export checks confirm matching Rust and JavaScript output, source-size limits and preserved measured strategies.
All nine default program exports remain byte-identical to the repository's preceding main revision.
