//! Select the default factions' retained sculptures.
use gameofcores::{unit3d::dress, Value};

/// Dress a founding army in the viewer's retained 3D sculptures. The source body remains the
/// canonical render price and the portable fallback; sculpting adds no gameplay upkeep.
pub fn dress_house(look: &mut Value, seat: usize) {
    dress(look);
    if let Value::Obj(models) = look.entry("models3d") {
        for (name, model) in &mut models.0 {
            model.set("sculpt", format!("{seat}:{name}"));
        }
    }
}
