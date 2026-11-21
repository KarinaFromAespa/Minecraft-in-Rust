//! Leads (`Leashable`): who holds a mob's lead and the elastic physics of
//! `Leashable.tickLeash` (the legacy pull once the rope stretches past six
//! blocks, the lead snapping past ten with no sound, `PathfinderMob`'s walk
//! toward the holder while the rope is slack).
use super::{Mob, MobEvent, MobWorld, Species, Who};
use horizon_core::geometry::Vec3;
use horizon_world::nbt::Tag;

/// `Leashable.LEASH_ELASTIC_DIST`.
pub const ELASTIC: f64 = 6.0;
/// `Leashable.LEASH_TOO_FAR_DIST` (1.21.5: ten; 1.21.6 raised it to twelve).
pub const SNAP: f64 = 10.0;

/// What holds the other end of a lead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holder {
    Player,
    Mob(u32),
    /// A boat or raft (`AbstractBoat` is `Leashable`, and any entity can be a holder).
    Boat(u32),
    /// A `LeashFenceKnotEntity` on the fence at this block.
    Knot([i32; 3]),
}
impl Holder {
    pub fn who(self) -> Option<Who> {
        match self {
            Holder::Player => Some(Who::Player),
            Holder::Mob(id) => Some(Who::Mob(id)),
            Holder::Boat(_) | Holder::Knot(_) => None,
        }
    }
}
/// A holder read from a save that is not in the world yet (the entity may
/// load later): the holder's UUID, or the fence knot's block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pending {
    Uuid([i32; 4]),
    Knot([i32; 3]),
}
/// `Leashable.LeashData.CODEC`: `{UUID: [I;..]}` for an entity holder or the
/// fence's block position as an int array.
pub fn pending_from_nbt(tag: Option<&Tag>) -> Option<Pending> {
    match tag? {
        Tag::IntArray(p) if p.len() == 3 => Some(Pending::Knot([p[0], p[1], p[2]])),
        t @ Tag::Compound(_) => crate::save::uuid_from_tag(t.get("UUID")).map(Pending::Uuid),
        _ => None,
    }
}
/// The `leash` tag for a holder: the holder entity's UUID, or the knot's block.
pub fn to_nbt(holder: Holder, uuid: Option<[i32; 4]>) -> Option<Tag> {
    match holder {
        Holder::Knot(p) => Some(Tag::IntArray(p.to_vec())),
        _ => uuid.map(|u| Tag::compound([("UUID", Tag::IntArray(u.to_vec()))])),
    }
}
/// Where the other end of the lead is this tick.
#[derive(Clone, Copy, Debug)]
pub struct HolderState {
    pub position: Vec3,
    pub width: f64,
    pub height: f64,
    pub yaw: f32,
    pub velocity: Vec3,
}
/// `LeashFenceKnotEntity`: 0.375 wide, 0.5 tall, centred a block's 0.375 up.
pub fn knot_state(pos: [i32; 3]) -> HolderState {
    let position = Vec3::new(
        f64::from(pos[0]) + 0.5,
        f64::from(pos[1]) + 0.375,
        f64::from(pos[2]) + 0.5,
    );
    HolderState {
        position,
        width: 0.375,
        height: 0.5,
        yaw: 0.0,
        velocity: Vec3::ZERO,
    }
}
/// The holder's state, or `None` when it is gone.
pub fn holder_state(w: &MobWorld, holder: Holder) -> Option<HolderState> {
    match holder {
        Holder::Player => {
            let p = w.player.as_ref()?;
            let size = p.bounds.max_position().subtract(p.bounds.min_position());
            Some(HolderState {
                position: p.position,
                width: size.x,
                height: size.y,
                yaw: p.yaw,
                velocity: p.velocity,
            })
        }
        Holder::Mob(id) => {
            let s = w.seen(id).filter(|s| s.alive)?;
            let lo = s.bounds.min_position();
            let hi = s.bounds.max_position();
            Some(HolderState {
                position: s.position,
                width: hi.x - lo.x,
                height: hi.y - lo.y,
                yaw: s.yaw,
                velocity: s.velocity,
            })
        }
        Holder::Boat(id) => w
            .leash_boats
            .iter()
            .find(|(boat, _)| *boat == id)
            .map(|(_, state)| *state),
        Holder::Knot(pos) => {
            let fence = w.has_tag("minecraft:fences", w.block_name(pos[0], pos[1], pos[2]));
            fence.then(|| knot_state(pos))
        }
    }
}
/// `Leashable.legacyElasticRangeLeashBehaviour`: the pull toward the
/// holder, `copySign(unit^2 * 0.4, unit)` per axis, where `unit` is the
/// direction to the holder over the (float) distance between the two.
/// 1.21.6 replaced this with the dampened-spring `Wrench` system.
pub fn legacy_pull(from: Vec3, to: Vec3, distance: f32) -> [f64; 3] {
    let d = f64::from(distance);
    let unit = [
        (to.x - from.x) / d,
        (to.y - from.y) / d,
        (to.z - from.z) / d,
    ];
    unit.map(|u| u.signum() * u * u * 0.4)
}

/// `Leashable.handleLeashAtDistance`, called before the distance branches
/// with the float distance to the holder. `false` stops the tick there.
/// `PathfinderMob` restricts the mob's home to the holder's block, radius
/// 5; `TamableAnimal` keeps a sitting pet put (dropping the lead past ten
/// blocks); horses stop eating and camels stand up past six.
fn handle_leash_at_distance(
    m: &mut Mob,
    w: &MobWorld,
    holder: &HolderState,
    distance: f32,
    events: &mut Vec<MobEvent>,
) -> bool {
    // TamableAnimal.handleLeashAtDistance: a sitting pet is not pulled, and
    // its lead drops past ten blocks (dropLeash: no sound). The override
    // covers every tamable animal, parrots included: a parrot sits when
    // told to (SitWhenOrdered), and a sitting parrot stays put too.
    let sitting = (m.species() == Species::Wolf || m.species() == Species::Parrot)
        && m.extra.sitting
        || m.species() == Species::Cat && super::tame::cat_sitting(m);
    if sitting {
        if distance > 10.0 {
            events.push(MobEvent::LeashBreak {
                mob: m.id,
                drop: true,
            });
        }
        return false;
    }
    // PathfinderMob.handleLeashAtDistance: home is the holder's block.
    m.ai.home = Some((
        [
            holder.position.x.floor() as i32,
            holder.position.y.floor() as i32,
            holder.position.z.floor() as i32,
        ],
        5,
    ));
    super::equine::handle_leash_at_distance(m, distance, w, events);
    true
}

/// `Entity.checkSlowFallDistance` (the body `checkFallDistanceAccumulation`
/// shares in 1.21.6).
fn check_fall_distance(m: &mut Mob) {
    if m.velocity.y > -0.5 && m.fall_distance > 1.0 {
        m.fall_distance = 1.0;
    }
}
/// Whether a mob can be put on a lead (`Mob.canBeLeashed` and overrides).
pub fn can_be_leashed(m: &Mob) -> bool {
    use Species as S;
    match m.species() {
        S::Hoglin | S::Zoglin => true,
        S::Villager
        | S::WanderingTrader
        | S::Turtle
        | S::Panda
        | S::Bat
        | S::Cod
        | S::Salmon
        | S::TropicalFish
        | S::Pufferfish
        | S::Tadpole => false,
        S::Wolf => !super::tame::angry(m),
        // Nautilus.canBeLeashed and ZombieNautilus.canBeLeashed: never while
        // it is aggravated. The zombie also refuses while a mob is riding it
        // (`isMobControlled`), which a mob alone cannot see.
        // shortcut: isMobControlled, when a mob can ride a zombie nautilus.
        S::Nautilus | S::ZombieNautilus => !super::nautilus::aggravated(m),
        // SulfurCube.canBeLeashed: `hasBodyItem()`, so only a cube with a
        // block in its body takes a lead. Its category is a monster's, which
        // `Mob.canBeLeashed`'s `!(this instanceof Enemy)` rule would refuse.
        S::SulfurCube => super::sulfur_cube::has_body(m),
        _ => !m.hostile() && !m.removed,
    }
}
/// `Leashable.tickLeash` at the end of `Entity.baseTick`.
pub fn tick(m: &mut Mob, w: &MobWorld, events: &mut Vec<MobEvent>) {
    let Some(holder) = m.extra.leash else {
        return;
    };
    // Leashable.tickLeash: a dying entity cannot interact with the level.
    if !m.alive() {
        events.push(MobEvent::LeashBreak {
            mob: m.id,
            drop: true,
        });
        return;
    }
    // Leashable.tickLeash: a holder that cannot interact with the level (dead
    // or spectating, `Entity.canInteractWithLevel`) drops the lead.
    if holder == Holder::Player
        && let Some(p) = w.player.as_ref()
        && m.extra.leash_player.is_none_or(|u| p.uuid == u)
        && (p.spectator || !p.alive)
    {
        events.push(MobEvent::LeashBreak {
            mob: m.id,
            drop: true,
        });
        return;
    }
    let state = holder_state(w, holder).filter(|_| {
        holder != Holder::Player
            || m.extra
                .leash_player
                .is_none_or(|u| w.player.as_ref().is_some_and(|p| p.uuid == u))
    });
    let Some(state) = state else {
        // The holder is gone: the lead drops.
        if !matches!(holder, Holder::Player) {
            events.push(MobEvent::LeashBreak {
                mob: m.id,
                drop: true,
            });
        }
        return;
    };
    // Leashable.tickLeash: `distanceTo` is feet to feet, narrowed to a float.
    let distance = m.position.distance(state.position) as f32;
    if !handle_leash_at_distance(m, w, &state, distance, events) {
        return;
    }
    if f64::from(distance) > SNAP {
        // leashTooFarBehaviour is dropLeash: the lead drops, and 1.21.5 plays
        // no sound (1.21.6 added LEAD_BREAK here).
        events.push(MobEvent::LeashBreak {
            mob: m.id,
            drop: true,
        });
        return;
    }
    if f64::from(distance) > ELASTIC {
        let pull = legacy_pull(m.position, state.position, distance);
        m.velocity = Vec3::new(
            m.velocity.x + pull[0],
            m.velocity.y + pull[1],
            m.velocity.z + pull[2],
        );
        check_fall_distance(m);
    } else {
        close_range(m, w, &state);
    }
}
/// The lead comes off (`Mob.onLeashRemoved`): no more home.
pub fn remove(m: &mut Mob) -> Option<Holder> {
    let was = m.extra.leash.take();
    if was.is_some() {
        m.ai.clear_home();
    }
    was
}
/// `PathfinderMob.closeRangeLeashBehaviour`: walk to within two blocks of
/// the holder unless panicking.
fn close_range(m: &mut Mob, w: &MobWorld, holder: &HolderState) {
    if m.ai.panicking {
        return;
    }
    let d = Vec3::new(
        holder.position.x - m.position.x,
        holder.position.y - m.position.y,
        holder.position.z - m.position.z,
    );
    let dist = d.length();
    let len = d.length();
    let scale = if len > 0.0 {
        (dist - 2.0).max(0.0) / len
    } else {
        0.0
    };
    let target = [
        m.position.x + d.x * scale,
        m.position.y + d.y * scale,
        m.position.z + d.z * scale,
    ];
    let speed = follow_speed(m);
    m.nav_move_to(w, target, speed);
}
/// `followLeashSpeed`: 1.0, 2.0 for llamas.
pub fn follow_speed(m: &Mob) -> f64 {
    if super::llama::is_llama(m.species()) {
        2.0
    } else {
        1.0
    }
}
/// `Mob.leashable` position of the rope on a mob (`getLeashOffset`).
pub fn leash_offset(m: &Mob) -> [f64; 3] {
    if super::llama::is_llama(m.species()) {
        // Llama.getLeashOffset.
        [0.0, 0.75 * m.eye_height(), f64::from(m.width()) * 0.5]
    } else if m.species() == super::Species::Wolf {
        // Wolf.getLeashOffset: 0.6 of the eye height.
        [
            0.0,
            f64::from(0.6_f32 * m.eye_height() as f32),
            f64::from(m.width() * 0.4),
        ]
    } else if m.species() == super::Species::Allay {
        // Allay.getLeashOffset.
        [0.0, m.eye_height() * 0.6, f64::from(m.width()) * 0.1]
    } else if matches!(
        m.species(),
        super::Species::Camel | super::Species::CamelHusk
    ) {
        super::camel::leash_offset(m)
    } else if m.species() == super::Species::Bee {
        // Bee.getLeashOffset: half the eye height, a fifth of the width ahead.
        [0.0, 0.5 * m.eye_height(), f64::from(m.width() * 0.2)]
    } else if m.species() == super::Species::IronGolem {
        // IronGolem.getLeashOffset: 0.875 of the eye height.
        [
            0.0,
            f64::from(0.875_f32 * m.eye_height() as f32),
            f64::from(m.width() * 0.4),
        ]
    } else if m.species() == super::Species::SnowGolem {
        // SnowGolem.getLeashOffset: 0.75 of the eye height.
        [
            0.0,
            f64::from(0.75_f32 * m.eye_height() as f32),
            f64::from(m.width() * 0.4),
        ]
    } else if m.species() == super::Species::CopperGolem {
        // CopperGolem.getLeashOffset: 0.75 of the eye height, on the centre
        // rather than the default `bbWidth / 2` in front.
        [0.0, f64::from(0.75_f32 * m.eye_height() as f32), 0.0]
    } else if m.species() == super::Species::SulfurCube {
        // SulfurCube.getLeashOffset: half of `getBbHeight`, on the centre
        // rather than the default `bbWidth / 2` in front.
        [0.0, f64::from(m.height() / 2.0), 0.0]
    } else if m.species() == super::Species::Fox {
        // Fox.getLeashOffset: 0.55 of the eye height.
        [
            0.0,
            f64::from(0.55_f32 * m.eye_height() as f32),
            f64::from(m.width() * 0.4),
        ]
    } else if let Some(factor) = match m.species() {
        // Pig.getLeashOffset, Rabbit.getLeashOffset and
        // Strider.getLeashOffset: 0.6 of the eye height; Parrot.getLeashOffset
        // and Ocelot.getLeashOffset: half of it.
        super::Species::Pig | super::Species::Rabbit | super::Species::Strider => Some(0.6_f32),
        super::Species::Parrot | super::Species::Ocelot => Some(0.5_f32),
        _ => None,
    } {
        [
            0.0,
            f64::from(factor * m.eye_height() as f32),
            f64::from(m.width() * 0.4),
        ]
    } else {
        [0.0, m.eye_height(), f64::from(m.width() * 0.4)]
    }
}
/// `AbstractBoat.getLeashOffset` for a boat or raft (1.375 by 0.5625).
pub fn boat_leash_offset() -> [f64; 3] {
    let (width, height) = (1.375_f32, 0.5625_f32);
    [
        0.0,
        f64::from(0.88_f32 * height),
        f64::from(0.64_f32 * width),
    ]
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_players_leash_does_not_pull_toward_a_different_player() {
        crate::mobs::world::with_test_world(&[], |world, types| {
            let mut w = world.clone();
            let mut cow = Mob::new(
                1,
                std::sync::Arc::new(types.types["minecraft:cow"].clone()),
                Vec3::new(0.5, 64.0, 0.5),
                0.0,
            );
            let owner = [1, 2, 3, 4];
            cow.extra.leash = Some(Holder::Player);
            cow.extra.leash_player = Some(owner);
            let mut nearby = crate::mobs::world::test_player(Vec3::new(8.5, 64.0, 0.5));
            nearby.uuid = [5, 6, 7, 8];
            w.player = Some(nearby);
            let mut events = Vec::new();
            tick(&mut cow, &w, &mut events);
            assert_eq!(
                cow.velocity,
                Vec3::ZERO,
                "absent holder must not use another player"
            );
            assert!(events.is_empty());
            // The same distance does pull once the real holder is the player view.
            w.player.as_mut().unwrap().uuid = owner;
            tick(&mut cow, &w, &mut events);
            assert!(
                cow.velocity.x > 0.0,
                "the actual holder must still pull the cow"
            );
        });
    }

    /// `legacyElasticRangeLeashBehaviour`: the pull is `copySign(unit^2 * 0.4,
    /// unit)` per axis — a straight-ahead holder eight blocks off pulls 0.4,
    /// a diagonal one less on each axis.
    #[test]
    fn the_legacy_pull_is_the_signed_square_of_the_unit_direction() {
        let from = Vec3::new(0.0, 64.0, 0.0);
        let straight = legacy_pull(from, Vec3::new(8.0, 64.0, 0.0), 8.0);
        assert!((straight[0] - 0.4).abs() < 1e-12);
        assert_eq!((straight[1], straight[2]), (0.0, 0.0));
        let diagonal = legacy_pull(from, Vec3::new(6.0, 64.0, 6.0), 8.485_281_374_238_57);
        // The distance passes through f32 (vanilla's float), so the
        // expected unit must come from the truncated value, not the
        // full-precision literal.
        let unit = 6.0 / f64::from(8.485_281_374_238_57f32);
        assert!((diagonal[0] - unit * unit * 0.4).abs() < 1e-12);
        assert!((diagonal[2] - unit * unit * 0.4).abs() < 1e-12);
        let behind = legacy_pull(from, Vec3::new(-8.0, 64.0, 0.0), 8.0);
        assert!((behind[0] + 0.4).abs() < 1e-12);
    }

    /// `tickLeash`: past ten blocks the lead snaps (the item drops, no
    /// sound); between six and ten the legacy pull lands; within six the
    /// mob walks to its holder instead.
    #[test]
    fn the_lead_snaps_past_ten_and_pulls_past_six() {
        crate::mobs::world::with_test_world(&[], |world, types| {
            let new = |x: f64| {
                let mut cow = Mob::new(
                    1,
                    std::sync::Arc::new(types.types["minecraft:cow"].clone()),
                    Vec3::new(x, 64.0, 0.5),
                    0.0,
                );
                cow.extra.leash = Some(Holder::Player);
                cow
            };
            let mut w = world.clone();
            w.player = Some(crate::mobs::world::test_player(Vec3::new(0.5, 64.0, 0.5)));
            // Eleven blocks off: snapped, silently.
            let mut far = new(11.5);
            let mut events = Vec::new();
            tick(&mut far, &w, &mut events);
            assert!(
                events.iter().any(|e| matches!(
                    e,
                    MobEvent::LeashBreak {
                        drop: true,
                                ..
                    }
                )),
                "a snapped lead drops silently: {events:?}"
            );
            // Eight blocks off: pulled toward the holder, not snapped.
            let mut mid = new(8.5);
            let mut events = Vec::new();
            tick(&mut mid, &w, &mut events);
            assert!(events.is_empty());
            assert!(
                mid.velocity.x < 0.0,
                "the pull goes toward the holder: {:?}",
                mid.velocity
            );
            assert!((mid.velocity.x + 0.4).abs() < 1e-9);
            // Four blocks off: no pull, no snap.
            let mut near = new(4.5);
            let mut events = Vec::new();
            tick(&mut near, &w, &mut events);
            assert!(events.is_empty());
            assert_eq!(near.velocity, Vec3::ZERO);
        });
    }

    /// `TamableAnimal.handleLeashAtDistance`: a sitting wolf is not pulled,
    /// and its lead drops past ten blocks.
    #[test]
    fn a_sitting_wolf_is_not_pulled_but_loses_its_lead_past_ten() {
        crate::mobs::world::with_test_world(&[], |world, types| {
            let mut w = world.clone();
            w.player = Some(crate::mobs::world::test_player(Vec3::new(0.5, 64.0, 0.5)));
            let new = |x: f64| {
                let mut wolf = Mob::new(
                    1,
                    std::sync::Arc::new(types.types["minecraft:wolf"].clone()),
                    Vec3::new(x, 64.0, 0.5),
                    0.0,
                );
                wolf.extra.leash = Some(Holder::Player);
                wolf.extra.sitting = true;
                wolf
            };
            let mut close = new(8.5);
            let mut events = Vec::new();
            tick(&mut close, &w, &mut events);
            assert!(events.is_empty());
            assert_eq!(close.velocity, Vec3::ZERO, "a sitting wolf stays put");
            let mut far = new(11.5);
            let mut events = Vec::new();
            tick(&mut far, &w, &mut events);
            assert!(
                events.iter().any(|e| matches!(
                    e,
                    MobEvent::LeashBreak {
                        drop: true,
                        ..
                    }
                )),
                "a sitting wolf past ten blocks loses its lead: {events:?}"
            );
        });
    }

    /// `TamableAnimal.handleLeashAtDistance`: a sitting parrot is not
    /// pulled either, and its lead drops past ten blocks.
    #[test]
    fn a_sitting_parrot_is_not_pulled_but_loses_its_lead_past_ten() {
        crate::mobs::world::with_test_world(&[], |world, types| {
            let mut w = world.clone();
            w.player = Some(crate::mobs::world::test_player(Vec3::new(0.5, 64.0, 0.5)));
            let new = |x: f64| {
                let mut parrot = Mob::new(
                    1,
                    std::sync::Arc::new(types.types["minecraft:parrot"].clone()),
                    Vec3::new(x, 64.0, 0.5),
                    0.0,
                );
                parrot.extra.leash = Some(Holder::Player);
                parrot.extra.sitting = true;
                parrot
            };
            let mut close = new(8.5);
            let mut events = Vec::new();
            tick(&mut close, &w, &mut events);
            assert!(events.is_empty());
            assert_eq!(close.velocity, Vec3::ZERO, "a sitting parrot stays put");
            let mut far = new(11.5);
            let mut events = Vec::new();
            tick(&mut far, &w, &mut events);
            assert!(
                events.iter().any(|e| matches!(
                    e,
                    MobEvent::LeashBreak {
                        drop: true,
                        ..
                    }
                )),
                "a sitting parrot past ten blocks loses its lead: {events:?}"
            );
        });
    }
    /// `IronGolem`, `SnowGolem`, `CopperGolem` and `Fox.getLeashOffset`: the
    /// rope hangs at a share of the eye height, and the copper golem's on the
    /// centre instead of the default `bbWidth / 2` in front.
    #[test]
    fn a_golem_or_a_fox_hangs_its_lead_where_vanilla_does() {
        let _ = crate::mobs::world::with_test_world(&[], |_, types| {
            for (kind, factor, forwards) in [
                ("minecraft:iron_golem", 0.875, 0.4),
                ("minecraft:snow_golem", 0.75, 0.4),
                ("minecraft:copper_golem", 0.75, 0.0),
                ("minecraft:fox", 0.55, 0.4),
            ] {
                let m = Mob::new(
                    1,
                    std::sync::Arc::new(types.types[kind].clone()),
                    Vec3::ZERO,
                    0.0,
                );
                let o = leash_offset(&m);
                let h = f64::from(factor * m.eye_height() as f32);
                let z = f64::from(m.width() * forwards as f32);
                assert!((o[1] - h).abs() < 1e-9, "{kind}: height {} vs {h}", o[1]);
                assert!((o[2] - z).abs() < 1e-9, "{kind}: forward {} vs {z}", o[2]);
                assert_eq!(o[0], 0.0, "{kind}: centred");
            }
        });
    }
    /// `Pig`, `Rabbit`, `Strider`, `Parrot` and `Ocelot.getLeashOffset`:
    /// 0.6 or half of the eye height, against the default eye-height
    /// height the fallthrough gave them.
    #[test]
    fn a_pig_a_rabbit_a_parrot_and_an_ocelot_hang_their_lead_where_vanilla_does() {
        let _ = crate::mobs::world::with_test_world(&[], |_, types| {
            for (kind, factor) in [
                ("minecraft:pig", 0.6),
                ("minecraft:rabbit", 0.6),
                ("minecraft:strider", 0.6),
                ("minecraft:parrot", 0.5),
                ("minecraft:ocelot", 0.5),
            ] {
                let m = Mob::new(
                    1,
                    std::sync::Arc::new(types.types[kind].clone()),
                    Vec3::ZERO,
                    0.0,
                );
                let o = leash_offset(&m);
                let h = f64::from(factor * m.eye_height() as f32);
                let z = f64::from(m.width() * 0.4);
                assert!((o[1] - h).abs() < 1e-9, "{kind}: height {} vs {h}", o[1]);
                assert!((o[2] - z).abs() < 1e-9, "{kind}: forward {} vs {z}", o[2]);
                assert_eq!(o[0], 0.0, "{kind}: centred");
            }
        });
    }
    /// `Nautilus.canBeLeashed` and `ZombieNautilus.canBeLeashed`: an
    /// aggravated nautilus, one holding an `ANGRY_AT` or an `ATTACK_TARGET`
    /// memory, cannot be put on a lead.
    #[test]
    fn an_aggravated_nautilus_cannot_be_leashed() {
        let ran = crate::mobs::world::with_test_world(&[], |_, types| {
            let new = |kind: &str| {
                Mob::new(
                    1,
                    std::sync::Arc::new(types.types[kind].clone()),
                    horizon_core::geometry::Vec3::ZERO,
                    0.0,
                )
            };
            let calm = new("minecraft:nautilus");
            assert!(can_be_leashed(&calm), "a calm nautilus");
            let mut angry = new("minecraft:nautilus");
            angry.extra.anger = 400;
            assert!(!can_be_leashed(&angry), "ANGRY_AT");
            let mut charging = new("minecraft:zombie_nautilus");
            charging.ai.target = Some(crate::mobs::Who::Player);
            assert!(!can_be_leashed(&charging), "ATTACK_TARGET");
            assert!(
                can_be_leashed(&new("minecraft:zombie_nautilus")),
                "a calm zombie nautilus"
            );
        });
        let _ = ran;
    }

    #[test]
    fn a_boats_lead_hangs_at_its_front() {
        let p = boat_leash_offset();
        assert!((p[1] - 0.88 * 0.5625).abs() < 1e-6 && (p[2] - 0.64 * 1.375).abs() < 1e-6);
    }
    #[test]
    fn leash_tags_round_trip() {
        let knot = to_nbt(Holder::Knot([1, 2, 3]), None).expect("knot tag");
        assert_eq!(
            pending_from_nbt(Some(&knot)),
            Some(Pending::Knot([1, 2, 3]))
        );
        let held = to_nbt(Holder::Player, Some([4, 5, 6, 7])).expect("uuid tag");
        assert_eq!(
            pending_from_nbt(Some(&held)),
            Some(Pending::Uuid([4, 5, 6, 7]))
        );
        assert_eq!(to_nbt(Holder::Mob(1), None), None);
    }
    #[test]
    fn a_knot_sits_on_the_fence_post() {
        let k = knot_state([10, 64, -3]);
        assert_eq!(k.position.x, 10.5);
        assert_eq!(k.position.y, 64.375);
    }
    /// `SulfurCube.canBeLeashed` is `hasBodyItem()`: only a cube with a block
    /// in its body takes a lead. `SulfurCube.getLeashOffset` hangs the rope at
    /// half of `getBbHeight`, on the cube's centre rather than the default
    /// `bbWidth / 2` in front.
    #[test]
    fn a_sulfur_cube_is_leashed_by_its_block_and_hangs_at_its_middle() {
        // `with_test_world` answers false when the prepared data is absent, and
        // the edits are inside the closure, so a skip needs no early return.
        let _ = crate::mobs::world::with_test_world(&[], |_, types| {
            let new = || {
                Mob::new(
                    1,
                    std::sync::Arc::new(types.types["minecraft:sulfur_cube"].clone()),
                    horizon_core::geometry::Vec3::ZERO,
                    0.0,
                )
            };
            let empty = new();
            let mut full = new();
            full.extra.body = Some("minecraft:stone".into());
            for m in [&empty, &full] {
                let o = leash_offset(m);
                let h = f64::from(m.height() / 2.0);
                assert!(
                    (o[1] - h).abs() < 1e-9,
                    "half the box height: {} vs {h}",
                    o[1]
                );
                assert_eq!((o[0], o[2]), (0.0, 0.0), "hangs on the centre");
            }
            assert!(!can_be_leashed(&empty), "a cube with no block in it");
            assert!(can_be_leashed(&full), "a cube holding a block");
        });
    }
}
