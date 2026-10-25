//! Themed aircraft definitions. Flight licenses permanent wings; ranged wings also need Air weapons.
//! A definition supplies a body and a name. Its unit function still chooses orders and lands to cool down.
use gameofcores::{obj, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Scout,
    Fighter,
    Striker,
}
impl Role {
    #[must_use]
    pub fn of(build: &Value) -> Self {
        if build.get("range").num() > 0.0 {
            Self::Striker
        } else if build.get("vision").num() >= 2.0 {
            Self::Scout
        } else {
            Self::Fighter
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Airframe {
    pub name: &'static str,
    pub role: Role,
}
impl Airframe {
    /// Ten points including wings' three-point price. All zero stats are explicit.
    #[must_use]
    pub fn build(self) -> Value {
        let [hp, speed, strength, vision, range] = match self.role {
            Role::Scout => [1, 3, 0, 3, 0],
            Role::Fighter => [2, 3, 2, 0, 0],
            Role::Striker => [2, 1, 2, 0, 1],
        };
        obj! {"hp"=>hp,"speed"=>speed,"strength"=>strength,"carry"=>0,"vision"=>vision,"range"=>range,"wings"=>1,"stealth"=>0}
    }

    /// A spawn for a unit function your program defines. Check it with `army::check` before submitting.
    #[must_use]
    pub fn spawn(self, function: &str, serial: u32) -> Value {
        obj! {"type"=>function,"name"=>format!("{} {serial}",self.name),"build"=>self.build()}
    }
}

pub(crate) const AIR_NAMES: [[&str; 3]; 11] = [
    ["Steppe falcon", "Sky keshig", "Thunderdrum airship"],
    ["Garuda eye", "Gandiva vimana", "Indra thunderchariot"],
    ["Pilgrim gryphon", "Crusader gryphon", "Sanctum sky chapel"],
    [
        "Lens glider",
        "Salomon ornithopter",
        "Bensalem sky laboratory",
    ],
    ["Kongming eye", "Swallow fire kite", "Heaven-sailing junk"],
    [
        "Red Star spotter",
        "Red Banner interceptor",
        "Hammer bomber",
    ],
    ["Watchpoint mote", "Null interceptor", "Fork bomber"],
    [
        "Survey balloon",
        "Invisible Hand glider",
        "Company sky freighter",
    ],
    ["Airborne emu", "Lewis gyro-emu", "Wheat-front cropduster"],
    ["Dandelion eye", "Petal glider", "Rose garden ark"],
    ["Hawkeye spotter", "Tomcat interceptor", "Hornet striker"],
];

#[must_use]
pub fn of(seat: usize, role: Role) -> Option<Airframe> {
    let i = match role {
        Role::Scout => 0,
        Role::Fighter => 1,
        Role::Striker => 2,
    };
    AIR_NAMES.get(seat).map(|names| Airframe {
        name: names[i],
        role,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_airframe_has_permanent_wings_and_a_legal_ten_point_body() {
        for seat in 0..11 {
            for role in [Role::Scout, Role::Fighter, Role::Striker] {
                let frame = of(seat, role).expect("house");
                let b = frame.build();
                assert_eq!(b.get("wings").num(), 1.0);
                assert_eq!(Role::of(&b), role);
                let points = [
                    ("hp", 1.0),
                    ("speed", 1.0),
                    ("strength", 1.0),
                    ("carry", 1.0),
                    ("vision", 1.0),
                    ("range", 2.0),
                    ("wings", 3.0),
                    ("stealth", 2.0),
                ]
                .iter()
                .map(|(k, p)| b.get(k).num() * p)
                .sum::<f64>();
                assert_eq!(points, 10.0);
                assert!(frame.name.len() <= 28);
            }
        }
        assert!(of(11, Role::Scout).is_none());
    }
}
