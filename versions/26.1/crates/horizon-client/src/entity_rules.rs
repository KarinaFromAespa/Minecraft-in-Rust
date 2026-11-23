//! PURPOSE: `Entity`'s gravity and fluid-push rules, decided once for every kind.
//! WHY: each kind used to check its own `NoGravity` flag and pick its own
//! number, so a kind that forgot the check ignored the flag (#6481).
//! REFS: vanilla `Entity.isNoGravity`, `getDefaultGravity`, `getGravity`,
//! `applyGravity`, `isPushedByFluid`, `getAirDrag`; issue #6509, slice #6544.
//! LOGS: none. READS-BY: the kinds' ticks in this crate and in `game/`.
//! ENDS-WHEN: kinds share one stored `Entity` state and this folds into it.
use horizon_core::geometry::Vec3;

/// The air drag `PrimedTnt.tick` and `FallingBlockEntity.tick` scale motion
/// by: the double literal 0.98 in 26.1 (26.2 calls `getAirDrag()`, 0.98F
/// widened to 0.9800000190734863).
pub fn scale_air_drag(velocity: Vec3) -> Vec3 {
    let drag = 0.98;
    Vec3::new(velocity.x * drag, velocity.y * drag, velocity.z * drag)
}

/// The rules vanilla's `Entity` answers for every kind. A kind overrides one
/// only where its vanilla class does; the defaults are `Entity`'s.
pub trait EntityRules {
    /// `Entity.isNoGravity` (`NoGravity` in the save).
    fn is_no_gravity(&self) -> bool {
        false
    }
    /// `Entity.getDefaultGravity`.
    fn get_default_gravity(&self) -> f64 {
        0.0
    }
    /// `Entity.getGravity`: none with `NoGravity`, else the kind's default.
    /// `applyGravity` subtracts this from the vertical velocity.
    fn get_gravity(&self) -> f64 {
        if self.is_no_gravity() {
            0.0
        } else {
            self.get_default_gravity()
        }
    }
    /// `Entity.isPushedByFluid`.
    fn is_pushed_by_fluid(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_ticks_of_air_drag_match_vanilla_0_98() {
        // 26.1's ticks use the double literal 0.98: 0.1 × 0.98⁴ is
        // 0.092236816, not 0.1 × 0.98F⁴ (0.09223682318…, the 26.2 form).
        let mut v = Vec3::new(0.1, 0.0, 0.0);
        for _ in 0..4 {
            v = scale_air_drag(v);
        }
        assert_eq!(v.x.to_bits(), 0x3fb7_9cd4_fc35_0910);
        assert_eq!(v.y.to_bits(), 0);
        assert_eq!(v.z.to_bits(), 0);
    }
}
