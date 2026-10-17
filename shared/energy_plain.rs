// The default program's grid: it runs on its house generator alone. tools/source.mjs puts this module in the default
// program as `energy`, in place of the house programs' planner (shared/energy.rs), since a program pays for its length
// in memory. Copy shared/energy.rs to plan a grid of your own.
use gameofcores::*;

/// Plans nothing: the house generator lights the capital's island.
pub fn apply(_orders: &mut Value, _world: View, _me: View) {}

/// Adds nothing to a rival province's value.
pub fn prize(_world: View, _me: View, _q: View) -> f64 { 0.0 }
