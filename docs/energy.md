# Default power grids

The default factions plan a power grid in CPU RTS worlds with energy enabled (`world.energy` is present).
The planner is `shared/energy.rs`. Every house program calls it each second through `common::govern`. In a world
without energy it returns at once.

The default program runs on its house generator alone. `tools/source.mjs` gives it `shared/energy_plain.rs`, which plans
nothing, in place of the planner, since a program pays for its length in memory: the planner is about 220 pages of the
4,096 a program's code may fill.
The engine's rules for plants, grids and batteries are in the game's `docs/rts/energy.md`.

## Doctrines

Each house plans for its draw times a margin. It builds the kinds it likes in order of preference, once their research
is done. Kinds it can't build yet are skipped.

| Faction | Plant kinds, best liked first | Margin | Battery store | Reserve kept | Darkens outlying land first | Sells power |
| --- | --- | ---: | ---: | ---: | --- | --- |
| Kublai Khan's Alt | hydro, boiler, wind, geo, solar | 1.15 | none | 10% | yes | no |
| Arjuna's Halting Problem | solar, wind, tidal | 1.3 | 150 s | 45% | yes | no |
| Knights Templated | wind, hydro, solar, tidal | 1.25 | 90 s | 25% | yes | no |
| New Atlantis | fusion, reactor, geo, hydro, solar | 1.2 | 60 s | 20% | yes | no |
| Quiet Fleet | tidal, boiler, wind, solar | 1.2 | 60 s | 20% | yes | no |
| The Kernel Manifesto | hydro, boiler, wind, geo, solar | 1.35 | none | none | no | no |
| Null Pointer Society | solar, wind, geo | 1.05 | 100 s | 30% | yes | no |
| Compute of Nations | wind, hydro, solar, tidal, reactor | 1.4 | 120 s | 20% | yes | yes |
| Other seats | solar, wind, hydro, boiler, geo, reactor | 1.2 | 60 s | 20% | yes | no |

Battery store is seconds of the house's draw. Arjuna keeps its 45% reserve only while the sun is up over its capital,
and 40% of it at night. Every other house keeps 40% of its reserve.

New Atlantis researches drilling and storage before its army technologies in worlds with energy. Fission and fusion
come after its army technologies.

## When a plant is ordered

The planner looks for a plant every three seconds. It skips the second the building planner (`shared/structures.rs`)
runs, every fifth second, and it starts only with 30,000 cycles of think left.

Each grid island in `me.energy.islands` balances alone. A plant lights only the island it stands on, and the house
generator lights only the capital's island, so an overseas town needs a plant of its own. The planner works out each
island's expected supply: each of its plants at its average output over a day, and the generator on the capital's
island. An island is short when that is below its draw × margin, plus 0.5 MW on the capital's island and 0.2 MW on any
other. The planner serves the island furthest short first, then the next one if the first has no site.

It orders a plant when all of these hold:

- The island draws at least 0.2 MW. The capital's island must draw at least 80% of the house generator, so a young
  realm runs on the generator.
- No unit is stalled.
- Fewer than two of its plants are going up, and the second only while an island is more than 1.5 MW short.
- Its bank holds one second of income above a floor. The floor is five seconds of income, or four and a half while the
  grid meets less than 92% of its draw.

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

## Paying for construction

A site's first construction second is charged when it is accepted. The rest is charged over the next 12 seconds
through the fabrication lane, which recruits also use. Every second that the house sends a compute policy, the
planner raises its fabrication limit by the construction rate of each plant going up. The house's own limit stays
for its units.

## Running the grid

Every five seconds the house sends `grid` orders:

- `shed` lists its provinces other than the capital, the farthest and least loyal first. They go dark first in a
  shortfall. The Kernel Manifesto sends no list, so all its provinces dim alike.
- `reserve` is the house's reserve share of its battery storage.

Compute of Nations offers spare power once a minute after `transmission` research. It offers to the bordering faction
whose grid meets the least of its draw, when that grid is below 90%. The offer is up to 6 MW for 180 seconds, at
0.12 ◆ a second for each MW. It keeps a tenth of its own draw spare.

## War

`energy::prize` adds to a rival province's value in each house's targeting. It counts the owner's grid running below
75%, the plants that would change hands with the province and the province's room for hydro, tidal and geothermal
plants.

Upkeep for buildings comes out of the upkeep lane before any unit. `common::unit_upkeep` subtracts it, so recruiting
and recovery see what the army alone costs.
