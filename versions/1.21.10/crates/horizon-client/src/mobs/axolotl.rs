//! Axolotl-specific damage, animation and prey rules; activities live in animal_brain.
use super::{Mob, MobEvent, MobWorld, Species, Who, animal_brain, keyframes::AnimationState};
use horizon_core::random::RandomSource;
pub const PLAY_DEAD_TICKS: i32 = 200;
pub const HUNTING_COOLDOWN: i64 = 2400;
/// `Axolotl.getMaxAirSupply`.
pub const MAX_AIR: i32 = 6000;

/// AxolotlAttackablesSensor: always-hostile targets bypass the hunting cooldown.
pub fn prey_eligible(
    distance: f64,
    water: bool,
    hostile: bool,
    hunt: bool,
    cooldown: bool,
    attackable: bool,
) -> bool {
    distance <= 64.0 && water && (hostile || hunt && !cooldown) && attackable
}
/// Axolotl.hurtServer draws its two play-dead rolls from the level random in
/// 1.21.10 (`level().random.nextInt(3)`); 26.1 drew them from the entity
/// random. The Rust still draws from the mob's random: the damage pipeline
/// (`Mob::damage` and its callers) carries no level random, and threading one
/// through it is a cross-cutting refactor, not this item's fix. The ordered
/// draws are preserved either way, even out of water.
fn roll_play_dead(
    random: &mut impl RandomSource,
    no_ai: bool,
    health: f32,
    max: f32,
    damage: f32,
    water: bool,
    entity_source: bool,
    already: bool,
) -> bool {
    !no_ai
        && random.next_int_bound(3).unwrap_or(0) == 0
        && ((random.next_int_bound(3).unwrap_or(0) as f32) < damage || health / max < 0.5)
        && damage < health
        && water
        && entity_source
        && !already
}
pub fn before_damage(m: &mut Mob, damage: f32, source: &super::Damage) {
    if m.species() != Species::Axolotl {
        return;
    }
    let playing = m.extra.axolotl.playing_dead;
    if roll_play_dead(
        &mut m.random,
        m.no_ai,
        m.health,
        m.attrs.max_health,
        damage,
        m.fluids.in_water,
        source.attacker.is_some() || source.is_projectile(),
        playing,
    ) {
        animal_brain::set_play_dead(m);
    }
}
pub fn is_playing_dead(m: &Mob) -> bool {
    m.species() == Species::Axolotl && m.extra.axolotl.playing_dead
}
/// Axolotl.onStopAttacking: help the player whose last damage killed the target.
pub fn on_stop_attacking(m: &Mob, w: &MobWorld, target: Who, events: &mut Vec<MobEvent>) {
    if let Who::Mob(id) = target
        && let Some(s) = w.seen(id)
        && !s.alive
        && s.last_damage
            .is_some_and(|d| d.attacker == Some(Who::Player))
        && w.player.as_ref().is_some_and(|p| {
            m.bounds()
                .inflate(horizon_core::geometry::Vec3::new(20.0, 20.0, 20.0))
                .intersects(p.bounds)
        })
    {
        events.push(MobEvent::AxolotlSupportPlayer { mob: m.id });
    }
}
/// BinaryAnimator(10, IN_OUT_SINE): the four animators `tickAnimations` drives
/// for every age in 1.21.10. 26.1 ticked seven extra baby-only keyframe states
/// (`tickBabyAnimations`); 1.21.10 has no baby/adult split.
#[derive(Clone, Debug, Default)]
pub struct Animation {
    pub playing_dead: bool,
    old: [u8; 4],
    value: [u8; 4],
    /// 26.1's baby keyframe states: nothing starts them in 1.21.10, so they
    /// stay stopped; item 779 removes them with the model poses.
    pub states: [AnimationState; 7],
}
impl Animation {
    pub fn tick(&mut self, water: bool, ground: bool, moving: bool) {
        self.old = self.value;
        let flags = [
            self.playing_dead,
            !self.playing_dead && water,
            !self.playing_dead && !water && ground,
            moving,
        ];
        for (v, on) in self.value.iter_mut().zip(flags) {
            *v = if on {
                (*v + 1).min(10)
            } else {
                v.saturating_sub(1)
            };
        }
    }
    pub fn factor(&self, i: usize, partial: f32) -> f32 {
        let x = (f32::from(self.old[i])
            + (f32::from(self.value[i]) - f32::from(self.old[i])) * partial)
            / 10.0;
        ((1.0 - horizon_core::jmath::cos(f64::from(x) * std::f64::consts::PI)) / 2.0) as f32
    }
}
pub fn tick_animation(m: &mut Mob) {
    if m.species() != Species::Axolotl {
        return;
    }
    let moving =
        m.limb_speed > 1.0e-5 || m.yaw != m.previous_yaw || m.head_pitch != m.previous_head_pitch;
    m.extra.axolotl.tick(m.fluids.in_water, m.on_ground, moving);
}
#[cfg(test)]
mod axolotl_parity_tests {
    use super::*;
    use horizon_core::random::LegacyRandom;
    #[test]
    fn prey_water_range_and_cooldown_match_native() {
        assert!(prey_eligible(64.0, true, false, true, false, true));
        assert!(!prey_eligible(64.01, true, false, true, false, true));
        assert!(!prey_eligible(4.0, false, true, true, false, true));
        assert!(!prey_eligible(4.0, true, false, true, true, true));
        assert!(prey_eligible(4.0, true, true, false, true, true));
        assert!(!prey_eligible(4.0, true, false, false, false, true));
        assert!(!prey_eligible(4.0, true, true, true, false, false));
    }
    #[test]
    fn damage_roll_matches_native_draw_order_and_gates() {
        for seed in 0..100 {
            for (water, entity, already, no_ai) in [
                (true, true, false, false),
                (false, true, false, false),
                (true, false, false, false),
                (true, true, true, false),
                (true, true, false, true),
            ] {
                let mut expected = LegacyRandom::new(seed);
                let mut actual = LegacyRandom::new(seed);
                let result = !no_ai
                    && expected.next_int_bound(3).unwrap() == 0
                    && ((expected.next_int_bound(3).unwrap() as f32) < 2.0 || 4.0f32 / 14.0 < 0.5)
                    && 2.0 < 4.0
                    && water
                    && entity
                    && !already;
                assert_eq!(
                    roll_play_dead(&mut actual, no_ai, 4.0, 14.0, 2.0, water, entity, already),
                    result
                );
                assert_eq!(actual.next_int(), expected.next_int());
            }
        }
    }
    #[test]
    fn playing_dead_eases_and_no_baby_state_starts() {
        let mut x = Animation {
            playing_dead: true,
            ..Animation::default()
        };
        for _ in 0..10 {
            x.tick(true, true, true);
        }
        assert_eq!(x.factor(0, 1.0), 1.0);
        assert_eq!(x.factor(1, 1.0), 0.0);
        // 1.21.10 has no baby keyframe states: `tickAnimations` drives the
        // four animators for every age, and nothing starts a baby state.
        assert!(x.states.iter().all(|s| s.started.is_none()));
        x.playing_dead = false;
        for _ in 0..10 {
            x.tick(true, false, true);
        }
        assert_eq!(x.factor(0, 1.0), 0.0);
        assert_eq!(x.factor(1, 1.0), 1.0);
        assert!(x.states.iter().all(|s| s.started.is_none()));
    }
}
