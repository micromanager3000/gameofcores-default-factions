# Default buildings

The actual default factions build in CPU RTS worlds with structures enabled.
Both uniform and house-program worlds use these designs and priorities.
Each faction pays construction, maintenance and service costs from its CPU bank.

| Faction | Design | Priority |
| --- | --- | --- |
| Kublai Khan's Alt | Steppe bastion | Fortify borders with supply and seven-cell batteries. |
| Arjuna's Halting Problem | Bow relay | Supply troops near borders; add six-cell batteries and repairs. |
| Knights Templated | Chapter cloister | Maintain local faith, with remaining capacity assigned to civic work. |
| New Atlantis | Research court | Supply workers and supplement natural mines with spare crew capacity. |
| Quiet Fleet | Harbor lantern | Prefer coasts; assign most support capacity to water observation. |
| The Kernel Manifesto | Assembly hall | Prefer unsettled provinces and deliver full civic coverage. |
| Null Pointer Society | Pointer spire | Prefer coastal observation posts with nine-cell batteries. |
| Compute of Nations | Exchange depot | Supply workers, supplement mines, and repair nearby units. |
| Emu High Command | Emu roost | Supply border troops from elevated roosts with five-cell batteries. |
| Tabula Rasa | Open courtyard | Support busy provinces with supply, civic work, and available equipment. |

Weapon, repair, and extractor ports require their usual completed research.
Each faction builds one site at a time, near a living local builder.
It reserves two seconds of income plus three construction quotes before ordering.
There is no fixed building count limit or one-site-per-province rule.
The planner estimates recurring supply and faith savings before ordering another site.
It discounts current supply demand to 60% and subtracts maintenance and support costs.
The remaining savings must repay construction within five minutes. Nearby existing supply capacity reduces the estimate.
Defense, repairs, extraction and civic stability are additional benefits, excluded from that payback estimate.

Ready buildings supply nearby grounded units only when the upkeep discount exceeds service costs.
Extractors use natural mines that accept the faction's workers and have spare crew capacity.
They supplement workers without retiring them. Saturated mines receive no extractor allocation.
Repair bays prioritize damaged buildings, then damaged units within six cells.
Funded bays offer nearby injured allies bounded repair grants. Nations charges 0.25 ◆ per HP; other houses donate repairs.
Recipients accept explicit grants priced at no more than 0.25 ◆ per HP.
Standing jobs refresh every five seconds and expire after 45 seconds without renewal.
Recovery clears their paid allocations. The planner reserves headroom for returning orders and writing RAM.
Nearby combat units attack hostile buildings when no enemy army needs their weapon cooldown.
In worlds with site maintenance, buildings become eligible for retirement after five minutes.
The planner keeps profitable supply, faith, extraction, civic work in unsettled provinces, active defenses and repair clients.
An idle site without those uses is demolished, ending its maintenance bill without a refund.

These buildings do not change the world's rules or enable structures themselves.
Start a supported world with `hypercycle-sim watch --cpu-rts --structures`.
