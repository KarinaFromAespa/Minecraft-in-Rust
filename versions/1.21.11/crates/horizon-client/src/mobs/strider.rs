//! Striders (`Strider`): their goals, standing and floating on lava
//! (`floatStrider`, `canStandOnFluid`), going cold away from warm blocks
//! (`setSuffocating`) and heading back to lava (`StriderGoToLavaGoal`).
//! `extra.flag` is the synced `DATA_SUFFOCATING`.
use super::{
    Mob, MobEvent, MobWorld, Species,
    data::LAND,
    goals::{
        Breed, Cx, FollowParent, Goal, JUMP, LookAtPlayer, MOVE, MoveToBlock, Panic,
        RandomLookAround, Stroll, Tempt, goal_clone, reduced,
    },
    path::PathType,
};
use crate::blocks::Fluid;
use horizon_core::random::RandomSource;

/// `SUFFOCATING_MODIFIER`: `-0.34F`, `ADD_MULTIPLIED_BASE`.
const SUFFOCATING: f64 = -0.34_f32 as f64;

pub fn register(m: &mut Mob) {
    m.set_malus(PathType::Water, -1.0);
    m.set_malus(PathType::Lava, 0.0);
    m.set_malus(PathType::DangerFire, 0.0);
    m.set_malus(PathType::DamageFire, 0.0);
    let tempt = m.kind.tempt.clone();
    let g = &mut m.ai.goals;
    g.add(1, Panic::new(1.65));
    g.add(2, Breed::new(1.0));
    g.add(3, Tempt::new(1.4, tempt, false));
    g.add(4, GoToLava(MoveToBlock::new(1.0, 8, 2)));
    g.add(5, FollowParent::new(1.0));
    g.add(7, Stroll::new(1.0).interval(60));
    g.add(8, LookAtPlayer::new(8.0));
    g.add(8, RandomLookAround::default());
    g.add(9, LookAtPlayer::species(Species::Strider, 8.0));
}
pub fn suffocating(m: &Mob) -> bool {
    m.species() == Species::Strider && m.extra.flag
}
/// The movement speed attribute with `SUFFOCATING_MODIFIER` while cold.
pub fn movement_speed(m: &Mob, base: f64) -> f64 {
    if suffocating(m) {
        base + base * SUFFOCATING
    } else {
        base
    }
}
/// `Strider.tick` before `super.tick()`: the tempted and retreating sounds,
/// then cold unless in or on a warm block — or riding a cold strider, whose
/// cold spreads to the rider.
pub fn pre_tick(m: &mut Mob, w: &MobWorld, events: &mut Vec<MobEvent>) {
    if m.ai.goals.is_running("Tempt") && m.random.next_int_bound(140).unwrap_or(1) == 0 {
        m.voice("entity.strider.happy", events);
    } else if m.ai.panicking && m.random.next_int_bound(60).unwrap_or(1) == 0 {
        m.voice("entity.strider.retreat", events);
    }
    if m.no_ai {
        return;
    }
    let warm = |p: [i32; 3]| {
        w.has_tag(
            "minecraft:strider_warm_blocks",
            w.block_name(p[0], p[1], p[2]),
        )
    };
    let inside = [
        m.position.x.floor() as i32,
        m.position.y.floor() as i32,
        m.position.z.floor() as i32,
    ];
    let in_warm = warm(inside) || warm(m.block_on_legacy(w)) || m.fluids.lava > 0.0;
    let on_suffocating_strider = m
        .extra
        .mount
        .and_then(|id| w.seen(id))
        .is_some_and(|v| v.species == Species::Strider && v.cold_strider);
    m.extra.flag = !in_warm || on_suffocating_strider;
}
/// `Strider.floatStrider`, after the living tick: standing on the surface of
/// lava, or bobbing up through it.
pub fn float(m: &mut Mob, w: &MobWorld) {
    if !m.fluids.in_lava {
        return;
    }
    let (x, y, z) = (
        m.position.x.floor() as i32,
        m.position.y.floor() as i32,
        m.position.z.floor() as i32,
    );
    // `CollisionContext.isAbove(LiquidBlock.SHAPE_STABLE, blockPosition())`:
    // the stable shape is half a block tall.
    let above = m.position.y > f64::from(y) + 0.5 - f64::from(1.0e-5_f32);
    if above && w.fluid(x, y + 1, z) != Fluid::Lava {
        m.on_ground = true;
    } else {
        let v = m.velocity;
        m.velocity = horizon_core::geometry::Vec3::new(v.x * 0.5, v.y * 0.5 + 0.05, v.z * 0.5);
    }
}
/// `Strider.getWalkTargetValue`.
pub fn walk_target_value(m: &Mob, w: &MobWorld, p: [i32; 3]) -> f32 {
    if w.fluid(p[0], p[1], p[2]) == Fluid::Lava {
        10.0
    } else if m.fluids.in_lava {
        f32::NEG_INFINITY
    } else {
        0.0
    }
}
/// `StriderGoToLavaGoal.isValidTarget`: lava with room to stand above.
fn lava_target(w: &MobWorld, p: [i32; 3]) -> bool {
    w.block_name(p[0], p[1], p[2]) == "minecraft:lava"
        && w.data.state(w.world.block(p[0], p[1] + 1, p[2])).flags & LAND != 0
}
/// `Strider.StriderGoToLavaGoal`: a `MoveToBlockGoal` for the nearest lava,
/// aiming at the lava block itself rather than the one above.
#[derive(Clone, Debug)]
struct GoToLava(MoveToBlock);
impl Goal for GoToLava {
    goal_clone!();
    fn flags(&self) -> u8 {
        MOVE | JUMP
    }
    fn name(&self) -> &'static str {
        "StriderGoToLavaGoal"
    }
    fn every_tick(&self) -> bool {
        true
    }
    fn can_use(&mut self, m: &mut Mob, cx: &mut Cx) -> bool {
        if m.fluids.in_lava {
            return false;
        }
        if self.0.next_start > 0 {
            self.0.next_start -= 1;
            return false;
        }
        self.0.next_start = reduced(200 + m.random.next_int_bound(200).unwrap_or(0));
        self.0.find_nearest(m, cx.w, lava_target)
    }
    fn can_continue(&mut self, m: &mut Mob, cx: &mut Cx) -> bool {
        !m.fluids.in_lava && lava_target(cx.w, self.0.block)
    }
    fn start(&mut self, m: &mut Mob, cx: &mut Cx) {
        self.0.start(m, cx.w);
    }
    /// `MoveToBlockGoal.tick` towards `getMoveToTarget()` (the lava block),
    /// recomputing the path every 20 ticks (`shouldRecalculatePath`).
    fn tick(&mut self, m: &mut Mob, cx: &mut Cx) {
        let b = self.0.block;
        let center = [
            f64::from(b[0]) + 0.5,
            f64::from(b[1]) + 0.5,
            f64::from(b[2]) + 0.5,
        ];
        let d2 = (m.position.x - center[0]).powi(2)
            + (m.position.y - center[1]).powi(2)
            + (m.position.z - center[2]).powi(2);
        if d2 >= 1.0 {
            self.0.reached = false;
            self.0.try_ticks += 1;
            if self.0.try_ticks % 20 == 0 {
                let speed = self.0.speed;
                m.nav_move_to(cx.w, [center[0], f64::from(b[1]), center[2]], speed);
            }
        } else {
            self.0.reached = true;
            self.0.try_ticks -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::geometry::Vec3;
    use std::sync::Arc;

    /// Real striders from the vanilla data; `None` without it.
    fn real<const N: usize>(at: [[f64; 3]; N]) -> Option<[Mob; N]> {
        let mut out = None;
        crate::mobs::world::with_test_world(&[], |_, types| {
            let kind = Arc::new(types.types["minecraft:strider"].clone());
            let mut id = 0;
            out = Some(at.map(|[x, y, z]| {
                id += 1;
                Mob::new(id, kind.clone(), Vec3::new(x, y, z), 0.0)
            }));
        });
        out
    }

    #[test]
    fn a_ridden_cold_strider_chills_its_rider() {
        // `Strider.tick` in 1.21.11: suffocating = !inWarm || the ridden
        // strider is suffocating (26.1: !inWarm && !riding a warm strider).
        let Some([mut ridden, mut rider]) = real([[0.5, 64.0, 0.5], [0.5, 65.0, 0.5]]) else {
            return;
        };
        let mut events = Vec::new();
        // In lava (warm) on a cold strider: cold in 1.21.11.
        ridden.extra.flag = true;
        rider.extra.mount = Some(ridden.id);
        rider.fluids.lava = 1.0;
        crate::mobs::world::with_test_world(&[ridden.seen(None)], |w, _| {
            pre_tick(&mut rider, w, &mut events);
        });
        assert!(rider.extra.flag, "a cold mount chills a warm rider");
        // Out of the lava on a warm strider: cold in 1.21.11 (26.1: warm).
        ridden.extra.flag = false;
        rider.fluids.lava = 0.0;
        crate::mobs::world::with_test_world(&[ridden.seen(None)], |w, _| {
            pre_tick(&mut rider, w, &mut events);
        });
        assert!(rider.extra.flag, "a warm mount does not warm a cold rider");
        // In lava on a warm strider: warm.
        rider.fluids.lava = 1.0;
        crate::mobs::world::with_test_world(&[ridden.seen(None)], |w, _| {
            pre_tick(&mut rider, w, &mut events);
        });
        assert!(!rider.extra.flag, "warm and warm-mounted stays warm");
    }
}
