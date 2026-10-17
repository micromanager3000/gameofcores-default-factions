# Default buildings

The default factions build cities in CPU RTS worlds with structures enabled.
Both uniform and house-program worlds use these designs and priorities.
Each faction pays construction, maintenance and service costs from its CPU bank.

## What they build

Every province gets a civic hall once the loyalty it adds repays the hall. The capital's hall is the house landmark.
Districts follow:

| Building | Ports | Built when |
| --- | --- | --- |
| Landmark (capital, radius 2.5) | support, logistics, weapon on a border | The capital has no civic hall and the hall repays itself. |
| Civic hall (radius 2) | support, logistics, weapon on a border | The province has no civic hall and the hall repays itself. |
| Supply yard (radius 1.6) | logistics | Nearby troops beyond the yards already there save enough upkeep. |
| Repair bay (radius 1.5) | logistics, repair | As a yard, once the province has a yard, with repair research. |
| Bastion (radius 1.6) | weapon, and repair with research | A border province of a house with batteries, from a comfortable bank. |
| Works (radius 1.5) | extract | New Atlantis, Compute of Nations and Tabula Rasa, at a province with a free natural mine. |

The planner values a hall by the income its civic bonus adds. It assumes each province earns an equal share of income.
Income scales from 35% to 100% with loyalty, and civic support adds up to the published `civicBonus`.
It adds faith savings where the faction holds the province's faith, and supply savings for troops near the town.
A yard counts the six best upkeep savings among troops within nine cells that existing yards don't cover, at 60%.
Each economic building must repay its quote within five minutes after maintenance.
Defense, repairs and extraction are extra benefits, excluded from that estimate.
A hall on a border also mounts the house battery, for houses with batteries and ranged research.

Each faction builds one site at a time, near a living local builder.
It reserves two seconds of income plus three construction quotes before ordering.
The planner chooses the building with the largest net return, and tries the next if no spot fits.

## Where they build

Buildings ring the town. A spot must clear the town square, every mine, water, cliffs, fords and every standing site.
The whole shell must stay on the province's side of each border, assuming neighbouring towns stand twelve cells away.
A road may cross the edge of a footprint but not its middle. The innermost ring with a free spot wins.

## How they look

Each house has a kit: wall material, terrace, roof form and tower form. Halls, yards, works, bastions and bays are
built from the kit. Each landmark is drawn by hand.

| Faction | Landmark | Kit |
| --- | --- | --- |
| Kublai Khan's Alt | Pleasure dome | Marble walls, domes on drums, domed towers, gardens. |
| Arjuna's Halting Problem | Temple of the bow | Sandstone, stepped temple roofs, minarets, gardens. |
| Knights Templated | Sanctum cathedral | Stone, steep gables, spired towers, cloister gardens. |
| New Atlantis | Salomon's House | Marble, domes, towers with glass observatories, gardens. |
| Quiet Fleet | Longjiang pagoda | Timber, tiered pagoda roofs. |
| The Kernel Manifesto | Great Reading Room | Stone, flat roofs in the house colour, spired towers, statues. |
| Null Pointer Society | Address Zero | Glass and dark stone, flat roofs, glass towers with masts. |
| Compute of Nations | Kirkcaldy exchange | Brick, gables, clock towers. |
| Emu High Command | Campion command | Corrugated metal, gables, water towers. |
| Tabula Rasa | Flux commons | White marble, flat roofs, obelisks, gardens. |

Roofs, domes, spires and banners take the owner's colour, so a city reads as its faction's from above.

A program pays for its length in memory, so the designs are written small. Each house's kit, names and landmark are a
line of `shared/buildings.txt`. A house program carries only its own line; the default program carries every line, so
it builds in the style of its seat.

## Operating them

Ready buildings supply nearby grounded units only when the upkeep discount exceeds service costs.
Each province's first civic hall works civic support. Knights Templated and other faith holders give part of it to faith.
Quiet Fleet and Null Pointer Society give part to watching an adjoining water.
Extractors use natural mines that accept the faction's workers and have spare crew capacity.
They supplement workers without retiring them. Saturated mines receive no extractor allocation.
Repair bays prioritize damaged buildings, then damaged units within six cells.
Funded bays offer nearby injured allies bounded repair grants. Nations charges 0.25 ◆ per HP; other houses donate repairs.
Recipients accept explicit grants priced at no more than 0.25 ◆ per HP.
Standing jobs refresh every six seconds and expire after 45 seconds without renewal.
Recovery clears their paid allocations. The planner reserves headroom for returning orders and writing RAM.
Nearby combat units attack hostile buildings when no enemy army needs their weapon cooldown.

These buildings do not change the world's rules or enable structures themselves.
Start a supported world with `hypercycle-sim watch --cpu-rts --structures`.
