//! Vexes (`Vex`): summoned by evokers, flying through walls around their
//! bound origin, charging at their target, and wasting away after their
//! limited life.
use super::{
    Damage, DamageKind, Mob, MobEvent, Species, Who,
    anim::{Poses, State},
    control::Operation,
    goals::{
        Cx, Float, Goal, HurtByTarget, LookAtPlayer, MOVE, NearestTarget, goal_clone, reduced,
    },
};
use horizon_core::geometry::trig::Mth;
use horizon_core::{geometry::Vec3, random::RandomSource};
use std::f32::consts::PI;

// Vex state: `flag` charging, `counter2` the owner's mob id (0 none),
// `home` the bound origin, `egg_time` the life left (-1 unlimited).

pub fn register(m: &mut Mob) {
    m.no_gravity = true;
    m.ai.mov.kind = super::control::MoveKind::Custom;
    m.extra.set_main_hand(Some("minecraft:iron_sword".into()));
    if m.extra.egg_time == 0 {
        m.extra.egg_time = -1;
    }
    let g = &mut m.ai.goals;
    g.add(0, Float);
    g.add(4, Charge);
    g.add(8, RandomMove);
    g.add(9, LookAtPlayer::new(3.0).probability(1.0));
    g.add(10, LookAtPlayer::mobs(8.0));
    // `HurtByTargetGoal(this, Raider.class)`: raiders are ignored, vexes are not.
    let mut hurt = HurtByTarget::new().alert();
    hurt.ignore = vec![
        Species::Pillager,
        Species::Vindicator,
        Species::Evoker,
        Species::Illusioner,
        Species::Witch,
        Species::Ravager,
    ];
    m.ai.targets.add(1, hurt);
    m.ai.targets.add(2, CopyOwnerTarget);
    m.ai.targets.add(3, NearestTarget::player(true));
}
fn has_wanted(m: &Mob) -> bool {
    m.ai.mov.operation == Operation::MoveTo
}
fn set_wanted(m: &mut Mob, at: [f64; 3], speed: f64) {
    m.ai.mov.wanted = at;
    m.ai.mov.speed_modifier = speed;
    m.ai.mov.operation = Operation::MoveTo;
}
/// `VexChargeAttackGoal`.
#[derive(Clone, Debug)]
struct Charge;
impl Goal for Charge {
    goal_clone!();
    fn flags(&self) -> u8 {
        MOVE
    }
    fn every_tick(&self) -> bool {
        true
    }
    fn can_use(&mut self, m: &mut Mob, cx: &mut Cx) -> bool {
        let Some(t) = m.target_info(cx.w).filter(|t| t.alive) else {
            return false;
        };
        !has_wanted(m)
            && m.random.next_int_bound(reduced(7)).unwrap_or(1) == 0
            && m.distance_sqr(t.position) > 4.0
    }
    fn can_continue(&mut self, m: &mut Mob, cx: &mut Cx) -> bool {
        has_wanted(m) && m.extra.flag && m.target_info(cx.w).is_some_and(|t| t.alive)
    }
    fn start(&mut self, m: &mut Mob, cx: &mut Cx) {
        if let Some(t) = m.target_info(cx.w) {
            set_wanted(m, [t.position.x, t.eye_y, t.position.z], 1.0);
        }
        m.extra.flag = true;
        cx.events.push(MobEvent::Sound(
            "entity.vex.charge",
            [m.position.x, m.position.y, m.position.z],
            1.0,
            1.0,
        ));
    }
    fn stop(&mut self, m: &mut Mob, _cx: &mut Cx) {
        m.extra.flag = false;
    }
    fn tick(&mut self, m: &mut Mob, cx: &mut Cx) {
        let (Some(who), Some(t)) = (m.ai.target, m.target_info(cx.w)) else {
            return;
        };
        if m.bounds().intersects(t.bounds) {
            super::ai::do_hurt_target(m, who, cx);
            m.extra.flag = false;
        } else if m.distance_sqr(t.position) < 9.0 {
            set_wanted(m, [t.position.x, t.eye_y, t.position.z], 1.0);
        }
    }
}
/// `VexRandomMoveGoal`: drift to an empty block near the bound origin.
#[derive(Clone, Debug)]
struct RandomMove;
impl Goal for RandomMove {
    goal_clone!();
    fn flags(&self) -> u8 {
        MOVE
    }
    fn can_use(&mut self, m: &mut Mob, _cx: &mut Cx) -> bool {
        !has_wanted(m) && m.random.next_int_bound(reduced(7)).unwrap_or(1) == 0
    }
    fn can_continue(&mut self, _m: &mut Mob, _cx: &mut Cx) -> bool {
        false
    }
    fn start(&mut self, m: &mut Mob, cx: &mut Cx) {
        // Ticked once: the goal never continues.
        let origin = m.extra.home.unwrap_or([
            m.position.x.floor() as i32,
            m.position.y.floor() as i32,
            m.position.z.floor() as i32,
        ]);
        for _ in 0..3 {
            let r = &mut m.random;
            let p = [
                origin[0] + r.next_int_bound(15).unwrap_or(7) - 7,
                origin[1] + r.next_int_bound(11).unwrap_or(5) - 5,
                origin[2] + r.next_int_bound(15).unwrap_or(7) - 7,
            ];
            if cx.w.facts.is_air(cx.w.world.block(p[0], p[1], p[2])) {
                let at = [
                    f64::from(p[0]) + 0.5,
                    f64::from(p[1]) + 0.5,
                    f64::from(p[2]) + 0.5,
                ];
                set_wanted(m, at, 0.25);
                if m.ai.target.is_none() {
                    m.look_at_speed(at, 180.0, 20.0);
                }
                break;
            }
        }
    }
}
/// `Vex.getOwner`: the mob the vex is bound to, `counter2` while the evoker
/// that summoned it is in the level (0 none). 1.21.4 keeps the owner as a
/// plain field, never saved: a vex loaded from a save has no owner. (1.21.6
/// made it an `EntityReference`, resolved through the level by UUID.)
pub fn owner(m: &Mob) -> Option<u32> {
    (m.extra.counter2 != 0).then_some(m.extra.counter2 as u32)
}
/// `VexCopyOwnerTargetGoal`: the evoker's target.
#[derive(Clone, Debug)]
struct CopyOwnerTarget;
impl Goal for CopyOwnerTarget {
    goal_clone!();
    fn flags(&self) -> u8 {
        super::goals::TARGET
    }
    fn can_use(&mut self, m: &mut Mob, cx: &mut Cx) -> bool {
        let seen = owner(m).and_then(|owner| {
            cx.w.others
                .iter()
                .find(|s| s.id == owner)
                .and_then(|s| s.target)
        });
        seen.is_some_and(|t| t != Who::Mob(m.id))
    }
    fn can_continue(&mut self, m: &mut Mob, cx: &mut Cx) -> bool {
        m.target_info(cx.w).is_some_and(|t| t.alive)
    }
    fn start(&mut self, m: &mut Mob, cx: &mut Cx) {
        // 1.21.4's `start` reads the owner's target without a null check;
        // `can_use` gates on a non-null owner with a target, so the lookup
        // here finds one whenever `start` runs.
        m.ai.target = owner(m).and_then(|owner| {
            cx.w.others
                .iter()
                .find(|s| s.id == owner)
                .and_then(|s| s.target)
        });
    }
}
/// `VexMoveControl`: accelerate straight at the wanted point.
pub fn move_control(m: &mut Mob) {
    if m.ai.mov.operation != Operation::MoveTo {
        return;
    }
    let [wx, wy, wz] = m.ai.mov.wanted;
    let d = Vec3::new(wx - m.position.x, wy - m.position.y, wz - m.position.z);
    let len = (d.x * d.x + d.y * d.y + d.z * d.z).sqrt();
    let b = m.bounds();
    let (lo, hi) = (b.min_position(), b.max_position());
    let size = ((hi.x - lo.x) + (hi.y - lo.y) + (hi.z - lo.z)) / 3.0;
    if len < size {
        m.ai.mov.operation = Operation::Wait;
        m.velocity = m.velocity.scale(0.5);
        return;
    }
    m.velocity = m
        .velocity
        .add_vector(d.scale(m.ai.mov.speed_modifier * 0.05 / len));
    let yaw = match m.extra.point.filter(|_| m.ai.target.is_some()) {
        Some(t) => -((t[0] - m.position.x).atan2(t[2] - m.position.z) as f32).to_degrees(),
        None => -(m.velocity.x.atan2(m.velocity.z) as f32).to_degrees(),
    };
    m.yaw = yaw;
    m.body_yaw = yaw;
}
/// `Vex` travel with `noPhysics`: straight through blocks, air friction.
pub fn travel(m: &mut Mob) -> bool {
    m.position = m.position.add_vector(m.velocity);
    m.velocity = Vec3::new(
        m.velocity.x * 0.91,
        m.velocity.y * 0.98,
        m.velocity.z * 0.91,
    );
    m.on_ground = false;
    true
}
/// `Vex.tick`: the limited life costs a heart a second once spent.
pub fn ai_step(m: &mut Mob, w: &super::MobWorld) {
    m.extra.point = m
        .target_info(w)
        .map(|t| [t.position.x, t.position.y, t.position.z]);
    if m.extra.egg_time > 0 {
        m.extra.egg_time -= 1;
        if m.extra.egg_time == 0 {
            m.extra.egg_time = 20;
            m.damage(1.0, Damage::of(DamageKind::Generic));
        }
    }
}
/// `VexModel.setupAnim`.
pub fn anim(m: &Mob, s: &State, p: &mut Poses) {
    p.yrot("head", s.y_rot.to_radians());
    p.xrot("head", s.x_rot.to_radians());
    let bob = (s.age * 5.5 * horizon_core::geometry::trig::DEG_TO_RAD).mcos() * 0.1;
    p.zrot("right_arm", PI / 5.0 + bob);
    p.zrot("left_arm", -(PI / 5.0 + bob));
    if m.extra.flag {
        p.xrot("body", 0.0);
        // setArmsCharging
        let (right, left) = (m.extra.main_hand.is_some(), m.extra.off_hand.is_some());
        if !right && !left {
            p.xrot("right_arm", -1.221_730_5);
            p.yrot("right_arm", PI / 12.0);
            p.zrot("right_arm", -0.471_238_88 - bob);
            p.xrot("left_arm", -1.221_730_5);
            p.yrot("left_arm", -PI / 12.0);
            p.zrot("left_arm", 0.471_238_88 + bob);
        } else {
            let sword = (std::f64::consts::PI * 7.0 / 6.0) as f32;
            if right {
                p.xrot("right_arm", sword);
                p.yrot("right_arm", PI / 12.0);
                p.zrot("right_arm", -0.471_238_88 - bob);
            }
            if left {
                p.xrot("left_arm", sword);
                p.yrot("left_arm", -PI / 12.0);
                p.zrot("left_arm", 0.471_238_88 + bob);
            }
        }
    } else {
        p.xrot("body", PI / 20.0);
    }
    let deg = horizon_core::geometry::trig::DEG_TO_RAD;
    let wing = 1.099_557_4 + (s.age * 45.836_624 * deg).mcos() * deg * 16.2;
    p.yrot("left_wing", wing);
    p.yrot("right_wing", -wing);
    p.xrot("left_wing", 0.471_238_88);
    p.zrot("left_wing", -0.471_238_88);
    p.xrot("right_wing", 0.471_238_88);
    p.zrot("right_wing", 0.471_238_88);
}
/// The vex's texture: brighter while charging.
pub fn texture(m: &Mob) -> &'static str {
    if m.extra.flag {
        "entity/illager/vex_charging"
    } else {
        "entity/illager/vex"
    }
}

/// `Vex.addAdditionalSaveData`: the bound origin and the limited life.
/// 1.21.4 writes the origin as three `BoundX` / `BoundY` / `BoundZ` ints
/// and the life as `LifeTicks` (only while the vex has a limited life);
/// 1.21.5 renamed both to the `bound_pos` int array and `life_ticks`.
/// The owner is a plain field in 1.21.4, never saved (1.21.6's
/// `EntityReference.store` wrote its UUID under `owner`); a stale `owner`
/// key a loaded compound carried is dropped by the re-save.
pub fn write_nbt(m: &Mob, map: &mut horizon_world::nbt::Compound) {
    use horizon_world::nbt::Tag;
    if let Some(h) = m.extra.home {
        map.insert("BoundX".into(), Tag::Int(h[0]));
        map.insert("BoundY".into(), Tag::Int(h[1]));
        map.insert("BoundZ".into(), Tag::Int(h[2]));
    }
    if m.extra.egg_time > 0 {
        map.insert("LifeTicks".into(), Tag::Int(m.extra.egg_time));
    }
    map.remove(&"owner".into());
}

/// `Vex.readAdditionalSaveData`: the bound origin (only when `BoundX`
/// is present) and the life left (-1 when `LifeTicks` is absent, i.e.
/// unlimited). The owner is not read: 1.21.4 never saved it, so a loaded
/// vex has none, whatever the compound carries.
pub fn read_nbt(m: &mut Mob, tag: &horizon_world::nbt::Tag) {
    if tag.get("BoundX").is_some() {
        let x = crate::save::number(tag.get("BoundX")).map_or(0, |v| v as i32);
        let y = crate::save::number(tag.get("BoundY")).map_or(0, |v| v as i32);
        let z = crate::save::number(tag.get("BoundZ")).map_or(0, |v| v as i32);
        m.extra.home = Some([x, y, z]);
    }
    m.extra.egg_time = crate::save::number(tag.get("LifeTicks")).map_or(-1, |v| v as i32);
}

#[cfg(test)]
mod owner_save_tests {
    use super::*;
    use crate::entity_nbt::{mob_from_nbt, mob_to_nbt};
    use horizon_world::nbt::Tag;
    use std::sync::Arc;

    /// A vex with the id `EVOKER` in the level: what `Vex.setOwner` records
    /// when an evoker summons it.
    const EVOKER: u32 = 7;

    fn vex() -> Mob {
        let mut kind = crate::mobs::sensor_scan_tests::kind();
        let ty = Arc::make_mut(&mut kind);
        ty.id = "minecraft:vex".into();
        ty.species = Species::Vex;
        Mob::new(1, kind, Vec3::ZERO, 0.0)
    }

    /// `Vex.addAdditionalSaveData` (1.21.4) never writes the owner: it is a
    /// plain field, not an `EntityReference`, so a summoned vex loses its
    /// evoker across a chunk save and copies no target after reloading.
    #[test]
    fn a_vexs_owner_does_not_survive_the_chunk_save() {
        let mut summoned = vex();
        assert_eq!(owner(&summoned), None, "a vex nobody summoned");
        summoned.extra.counter2 = EVOKER as i32;
        assert_eq!(owner(&summoned), Some(EVOKER), "the evoker's id");
        let saved = mob_to_nbt(&summoned, [0; 4], None, None, |_| None);
        assert_eq!(saved.get("owner"), None, "no owner key is written");
        let types = [("minecraft:vex".into(), summoned.kind.clone())].into();
        let back = mob_from_nbt(&saved, &types, 2, None).unwrap();
        assert_eq!(back.extra.counter2, 0);
        assert_eq!(back.extra.owner, None);
        assert_eq!(owner(&back), None, "a reloaded vex has no owner");
    }

    /// A stale `owner` key in a loaded compound is ignored on load and
    /// dropped on re-save: 1.21.4 neither reads nor writes it.
    #[test]
    fn a_stale_owner_key_is_ignored_and_dropped() {
        let mut loose = vex();
        let stale = Tag::compound([
            ("id", Tag::String("minecraft:vex".into())),
            (
                "Pos",
                Tag::List(vec![
                    Tag::Double(0.5_f64.to_bits()),
                    Tag::Double(64.0_f64.to_bits()),
                    Tag::Double(0.5_f64.to_bits()),
                ]),
            ),
            ("owner", Tag::IntArray(vec![9, 9, 9, 9])),
        ]);
        let types = [("minecraft:vex".into(), loose.kind.clone())].into();
        let loaded = mob_from_nbt(&stale, &types, 3, None).unwrap();
        assert_eq!(loaded.extra.owner, None);
        assert_eq!(owner(&loaded), None);
        loose.extra.owner = None;
        loose.extra.counter2 = 0;
        let saved = mob_to_nbt(&loose, [0; 4], Some(&stale), None, |_| None);
        assert_eq!(saved.get("owner"), None, "no owner, no key");
    }

    /// `Vex.addAdditionalSaveData` (1.21.4) writes the bound origin as
    /// `BoundX` / `BoundY` / `BoundZ` and the life as `LifeTicks` — not
    /// the 1.21.5 `bound_pos` array and `life_ticks`.
    #[test]
    fn a_bound_vex_saves_bound_xyz_and_life_ticks() {
        let mut bound = vex();
        bound.extra.home = Some([10, 64, -5]);
        bound.extra.egg_time = 600;
        let saved = mob_to_nbt(&bound, [0; 4], None, None, |_| None);
        assert_eq!(saved.get("BoundX"), Some(&Tag::Int(10)));
        assert_eq!(saved.get("BoundY"), Some(&Tag::Int(64)));
        assert_eq!(saved.get("BoundZ"), Some(&Tag::Int(-5)));
        assert_eq!(saved.get("LifeTicks"), Some(&Tag::Int(600)));
        assert_eq!(saved.get("bound_pos"), None, "no 1.21.5 key");
        assert_eq!(saved.get("life_ticks"), None, "no 1.21.5 key");
        let types = [("minecraft:vex".into(), bound.kind.clone())].into();
        let back = mob_from_nbt(&saved, &types, 2, None).unwrap();
        assert_eq!(back.extra.home, Some([10, 64, -5]));
        assert_eq!(back.extra.egg_time, 600);

        let mut free = vex();
        free.extra.home = None;
        free.extra.egg_time = -1;
        let saved = mob_to_nbt(&free, [0; 4], None, None, |_| None);
        assert_eq!(saved.get("BoundX"), None, "no origin, no key");
        assert_eq!(saved.get("LifeTicks"), None, "unlimited, no key");
    }
}
