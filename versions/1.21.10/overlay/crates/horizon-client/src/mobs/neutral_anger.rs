//! `NeutralMob`'s persistent anger: 1.21.10 keeps a plain remaining-ticks
//! countdown (`Extra::anger`), saved as `AngerTime`, with no absolute end
//! time (1.21.11's `anger_end_time` / `getPersistentAngerEndTime` does not
//! exist here). The countdown ticks down once per tick the mob ticks, and
//! freezes while it does not: ticks the mob missed never count.

use super::{Mob, Species};

/// The `NeutralMob`s whose anger is `Extra::anger`: they write and read
/// `AngerTime` and `AngryAt`. The bee keeps its own anger, and a
/// piglin's is a brain memory, not `NeutralMob`'s.
pub fn persistent_anger(s: Species) -> bool {
    matches!(
        s,
        Species::Wolf
            | Species::IronGolem
            | Species::PolarBear
            | Species::Enderman
            | Species::ZombifiedPiglin
    )
}
/// `NeutralMob.stopBeingAngry`'s anger: no countdown.
pub fn stop(m: &mut Mob) {
    m.extra.anger = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobs::sensor_scan_tests;
    use horizon_core::geometry::Vec3;
    use std::sync::Arc;

    /// A `NeutralMob` of `species`; no resources are needed, only the species
    /// decides how its anger is kept.
    fn neutral(species: Species, id: &str) -> Mob {
        let mut kind = (*sensor_scan_tests::kind()).clone();
        kind.id = id.into();
        kind.species = species;
        Mob::new(1, Arc::new(kind), Vec3::ZERO, 0.0)
    }

    /// The five `NeutralMob`s save their anger; the bee keeps its own, and a
    /// piglin's is a brain memory.
    #[test]
    fn persistent_anger_covers_the_five_neutral_mobs() {
        for s in [
            Species::Wolf,
            Species::IronGolem,
            Species::PolarBear,
            Species::Enderman,
            Species::ZombifiedPiglin,
        ] {
            assert!(persistent_anger(s), "{s:?}");
        }
        for s in [Species::Bee, Species::Piglin, Species::Pig] {
            assert!(!persistent_anger(s), "{s:?}");
        }
    }
    /// `stopBeingAngry` ends the countdown.
    #[test]
    fn stop_clears_the_countdown() {
        let mut m = neutral(Species::Wolf, "minecraft:wolf");
        m.extra.anger = 400;
        stop(&mut m);
        assert_eq!(m.extra.anger, 0);
    }
}
