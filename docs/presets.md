# Shared faction presets

The source exporter includes these modules in each program's `shared` module.
Import them from `shared`. All included source counts toward code RAM.

## House themes

Each founding seat (factions 0–9; Tabula Rasa's style is archived at seat 9) has a house style: a map look with its
anthem, a 3D capital, unit names and a war line ([names.md](names.md) has their stories). The house programs and the
default program pick theirs by seat (`info.seat` in `look` and `build3d`, `me.seat` in think), never by name, so a
founder that renames itself keeps its style. Your code can borrow any of them.

`theme::of(seat, home: f64) -> Option<theme::Theme>` gives founding seat `seat`'s style, or seat `home`'s when `seat` is
missing (undefined or null), or `None` for any other seat. Only that seat's style is built, and only when it's asked
for. With a `home` (-1.0 for none), every other seat gets the home style too, in its own colour and with its capital
titled with its own name (and no motto), but without the founder's anthem, war line or capital music: `music()` is null,
`war()` is `None`, and so is the scene's `music`.

What a `Theme` holds:

- **`look(budget: f64, info) -> Value`:** a whole `look()` result in `info.color`, with the anthem as `music`. The
  Kernel Manifesto's own seat wears Soviet red instead (its land keeps its colour). Nothing is trimmed at any render
  budget (56–170): the Wonder always shows on the map. In the CPU RTS it dresses the whole army its seat's house program
  fields. Every unit type gets its own body (`units`, `sprites`), a name (`names`) and a weapon (`combat.types`). A
  weapon carries only the sounds its type makes, since each sound costs upkeep. Kublai's keshig ride under pennanted
  lances, the Kernel Manifesto brings a T-34 and a Katyusha, and the Null Pointer Society throws fork bombs. Its ships
  wear the founder's own `ship` style, from Kublai's war boat with a ger on deck to Tabula Rasa's paper boat.
- **`scene(info, budget: f64) -> Value`:** a whole `build3d()` scene (most on `shared::capital::civic`; Kurukshetra is the open
  field, with its own music and script, and in the Reading Room Marx quotes the Manifesto and Capital as its script)
  that fits every build budget untrimmed, and plants more of its cheapest piece as the budget grows (`shared::capital::spare`).
- **`names() -> theme::Names`, `names_for(unit: &str) -> Option<&[&str]>`:** the unit names it lists, as pairs of a unit
  type and its names, for `army::name`, e.g. `theme::of(seat, -1.0).and_then(|t| t.names_for("guard"))`. The default
  program names its `miner`, `guard`, `scout`, `longbow` and `spy` units from these lists (every founder's style has
  them), and its look gives those types their bodies and display names.
- **`airframe(role: air::Role) -> air::Airframe`:** a house scout, fighter or strike craft. Its `build()` returns a ten-point body with `wings: 1`; `spawn(function, serial)` names a spawn for a function your code defines. Flight is required; strike craft also need range and Air weapons.
- **`naval_name(unit: &str) -> Option<&str>`:** the house's title for `warjunk`, `picket` or `corsair`. The fleet program uses these titles in `look().names` and for newly spawned ships; code identifiers stay the same.
- **`war(who: &str, m: &theme::Moment) -> Option<String>`:** its line for when rivals march on you (`who`, by name), to
  say in public with `mem::say_once`. A `Moment` is `{ kind, who, at, capital }`: `at` the province they stand in, and
  `capital` whether it's yours.
- **`music() -> Value`:** the anthem: in the seat's key (D A F G C E B♭ B D), at most 12 s a pass.
- **`name()`, `capital()`:** the founder's name and its capital's.

```rust
const HOME: f64 = 3.0; // this program's own seat

fn look(budget: View, info: View) -> Value {
    theme::of(info.get("seat"), HOME).map_or(Value::Null, |t| t.look(budget.num(), info))
}

fn build3d(budget: View, info: View) -> Value {
    theme::of(info.get("seat"), HOME).map_or(Value::Null, |t| t.scene(info, budget.num()))
}
```

A look and a scene fit well within their fuel: a look builds a whole army's bodies and weapons, and costs the most.

---


## Aircraft definitions

`air::of(seat, air::Role::{Scout,Fighter,Striker})` returns one of thirty themed definitions, or `None` past seat 9.
`Theme::airframe` selects from the same catalogue. Every definition has explicit stats, including permanent wings.
A scout spends on speed and vision, a fighter on speed and strength, and a strike craft on strength and range.
All spend ten points using the engine's stat prices. These definitions do not bypass research, fabrication or flight
burn. `army::check` checks the current build caps and range-on-wings license before a spawn is submitted.

```rust
let frame = theme::of(me.get("seat"), 3.0).expect("house").airframe(air::Role::Fighter);
let body = frame.build();
if army::check(&body, &Value::Null, me).is_none() {
    // aeronaut must be a function in this program's program!(...) declaration.
    orders.entry("spawn").push(frame.spawn("aeronaut", 1));
}
```

New Atlantis recruits a small `aeronaut` wing after Flight once it has a ground screen and can afford specialists.
Quiet Fleet's `kite` and Null Pointer Society's `wing` rotate through scouts, fighters and licensed strike craft.
Arjuna prioritizes an affordable first wing and launches sorties; Emu prioritizes ground troops.
Custom programs can use any of these definitions with their own unit functions. The viewer chooses the airframe
from a house model's style and its permanent build: range selects strike craft, vision of at least two selects a
scout, and the remaining winged bodies are fighters. Explicit custom 3D models retain their own geometry.

Landing on friendly/allied land cools the unit and removes its flight burn; it keeps its wings. Atlantis's controller
returns hot aircraft to land, keeps cooling units down, then reads their carried plan again. Null and Quiet Fleet
keep their flight controllers and cooling rules. Presentation helpers do not change flight or combat rules.
