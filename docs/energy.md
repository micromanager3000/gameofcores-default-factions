# Default power grids

The default factions plan a power grid in CPU RTS worlds with energy enabled (`world.energy` is present).
The planner is `shared/energy.rs`. Every house program calls it each second through `common::govern`. In a world
without energy it returns at once.

The default program uses the same planner. `tools/source.mjs` includes `shared/energy.rs` in every default export, so an
unclaimed seat can build local generation, set a battery reserve and trade power after it unlocks transmission.
The engine's rules for plants, grids and batteries are in the game's `docs/rts/energy.md`.

## Doctrines

Each house plans for its draw times a margin. It builds the kinds it likes in order of preference, once their research
is done. Kinds it can't build yet are skipped.

| Faction | Plant kinds, best liked first | Margin | Battery store | Reserve kept | Darkens outlying land first | Sells power |
| --- | --- | ---: | ---: | ---: | --- | --- |
| Kublai Khan's Alt | hydro, boiler, wind, geo, solar | 1.1 | none | 10% | yes | no |
| Arjuna's Halting Problem | solar, wind, tidal | 1.2 | 150 s | 45% | yes | no |
| Knights Templated | wind, hydro, solar, tidal | 1.15 | 90 s | 25% | yes | no |
| New Atlantis | fusion, reactor, geo, hydro, solar | 1.1 | 60 s | 20% | yes | no |
| Quiet Fleet | tidal, boiler, wind, solar | 1.1 | 60 s | 20% | yes | no |
| The Kernel Manifesto | hydro, boiler, wind, geo, solar | 1.2 | none | none | no | no |
| Null Pointer Society | solar, wind, geo | 1.0 | 100 s | 30% | yes | no |
| Compute of Nations | wind, hydro, solar, tidal, reactor | 1.25 | 120 s | 20% | yes | yes |
| Other seats | solar, wind, hydro, boiler, geo, reactor | 1.1 | 60 s | 20% | yes | yes |

Battery store is seconds of the house's draw. Arjuna keeps its 45% reserve only while the sun is up over its capital,
and 40% of it at night. Every other house keeps 40% of its reserve.

New Atlantis researches drilling and storage before its army technologies in worlds with energy. Fission and fusion
come after its army technologies.

## When a plant is ordered

The planner looks for a plant every six seconds. A resting house thinks in full only every third second; the building
planner (`shared/structures.rs`) takes every other one of those seconds, and the grid planner the rest. It starts only
with 30,000 cycles of think left.

Each grid island in `me.energy.islands` balances alone. A plant lights only the island it stands on, and the house
generator lights only the capital's island, so an overseas town needs a plant of its own. The planner works out each
island's expected supply: each of its plants at its average output over a day, and the generator on the capital's
island. An island is short when that is below its draw × margin, plus 0.3 MW on the capital's island and 0.2 MW on any
other. The planner serves the island furthest short first, then the next one if the first has no site.

It orders a plant when all of these hold:

- The island draws at least 0.2 MW. The capital's island must draw at least 80% of the house generator, so a young
  realm runs on the generator.
- No unit is stalled.
- Fewer than two of its plants are going up, and the second only while an island is more than 1.5 MW short.
- Its bank holds one second of income above a floor. The floor is five seconds of income, or four and a half while the
  grid meets less than 92% of its draw. In worlds with cargo the floor is at least 30 seconds of the whole upkeep bill,
  since the army lives on the bank between loads.

The plant may cost 70% of the bank above that floor. `common::govern` drops to its recovery policy when the bank falls
below four seconds of income, so a plant never pushes a healthy house into recovery.

## Which plant

Each kind the house can build on the island is sized for its shortfall, up to the room the province has for it, the
kind's limit and 4 MW (10 MW for reactors and fusion). The planner prefers several small plants over one large one.
Each goes up sooner and a raid takes less.

It ranks every candidate by quote ÷ average output, so the cheapest steady megawatt wins. Each step down the house's
list of kinds adds 40%. A border province adds 30%. A boiler's cost counts three times, because it burns cargo that
would otherwise be income. A boiler needs cargo in its town's depot.

When no plant is ordered and the sky makes more than 35% of the capital island's power, the house adds batteries at
the capital until they hold its store of that island's draw. Each battery holds up to a minute of draw and at most
2,400 MW·s.

The best four candidates are tried in order. The first with room for its shell beside a living builder is ordered.
The search stops when fewer than 25,000 cycles of think are left, so think never runs out and loses its orders.

## Paying for plants

A site's first construction second is charged when it is accepted. The rest is charged over the next 12 seconds
through the fabrication lane, which recruits also use. A standing plant's upkeep comes out of the upkeep lane before
any unit's. Every second that the house sends a compute policy, the planner raises both limits by what its plants
cost:

- fabrication, by the construction rate of each plant going up;
- upkeep, by each plant's `energyCost` ÷ `plantMaintenancePeriod` (1,800 s) and the rest of its quote ÷ 600 a second,
  counting a plant going up by the share of it that is built.

A limit given as a share of income becomes that share of last second's income plus the plants' cost. The house's own
limits stay for its units, so a grid never stalls the army that guards it.

## Running the grid

Every twelve seconds the house sends `grid` orders:

- `shed` lists its provinces other than the capital, the farthest and least loyal first. They go dark first in a
  shortfall. The Kernel Manifesto sends no list, so all its provinces dim alike.
- `reserve` is the house's reserve share of its battery storage.

Compute of Nations and unclaimed default seats offer spare power once a minute after `transmission` research. They offer
to the bordering faction whose grid meets the least of its draw, when that grid is below 90%. A deal uses the first grid
link the engine finds, reported in `me.energy.exportLinks`. Each entry is `{ "with": factionId, "at": provinceId }`:
the first live link to that faction and its own province. The planner makes no power offer until that observation is
available, then only offers that island's own generation after keeping 15% of its local draw.
Imported power, batteries and boiler fuel are never offered onward. The offer is up to 6 MW for 180 seconds, at 0.125 ◆
a second for each MW.

## War

`energy::prize` adds to a rival province's value in each house's targeting. It counts the owner's grid running below
75%, the plants that would change hands with the province and the province's room for hydro, tidal and geothermal
plants.

Upkeep for buildings comes out of the upkeep lane before any unit, so a recruit has to fit the army's share of income
after what buildings and plants take (`common::building_upkeep`). `common::unit_upkeep` subtracts it, so recovery sees
what the army alone costs.
