//! Sniffers (`Sniffer`, `SnifferAi`): the brain's core, idle, sniff and dig
//! activities, scenting and sniffing for diggable ground, walking there,
//! digging up a seed, the explored positions, laying eggs when bred, and
//! `SnifferModel`'s keyframe animations.
use super::{
    Mob, MobEvent,
    anim::{Poses, State},
    keyframes::Animation,
};
use std::collections::HashMap;

// Sniffer state: `variant` the state (below), `scratch[0]` the state the
// animations last saw, `positions` the explored positions (newest first),
// `ints[3]` the tick the dug seed drops. AnimState states: feeling happy,
// scenting, sniffing, digging, rising.
pub const IDLING: u8 = 0;
pub const FEELING_HAPPY: u8 = 1;
pub const SCENTING: u8 = 2;
pub const SNIFFING: u8 = 3;
pub const SEARCHING: u8 = 4;
pub const DIGGING: u8 = 5;
pub const RISING: u8 = 6;

/// `Sniffer.setBaby`: a baby's age in ticks (`SNIFFER_BABY_AGE_TICKS`;
/// 26.1 exposed it as `getBabyStartAge`).
pub const BABY_START_AGE: i32 = -48000;

pub fn state(m: &Mob) -> u8 {
    m.extra.variant
}
fn now(m: &Mob) -> i32 {
    m.age as i32
}
/// `Sniffer.getDefaultDimensions`: lower while digging.
pub fn digging(m: &Mob) -> bool {
    state(m) == DIGGING
}
/// `Sniffer.canPlayDiggingSound`.
pub fn can_play_digging_sound(m: &Mob) -> bool {
    matches!(state(m), DIGGING | SEARCHING)
}
pub fn register(m: &mut Mob) {
    use super::path::PathType;
    m.ai.nav.can_float = true;
    m.set_malus(PathType::Water, -1.0);
    m.set_malus(PathType::DangerPowderSnow, -1.0);
    m.set_malus(PathType::DamageCautious, -1.0);
}
/// `Sniffer.setState`.
pub fn set_state(m: &mut Mob, s: u8) {
    m.extra.variant = s;
}
/// `Sniffer.getOnPos`: the block under its feet.
pub fn on_pos(m: &Mob) -> [i32; 3] {
    [
        m.position.x.floor() as i32,
        (m.position.y - f64::from(0.2_f32)).floor() as i32,
        m.position.z.floor() as i32,
    ]
}
/// `Sniffer.getHeadBlock`: 2.25 blocks along `getForward()`, at its feet
/// plus 0.2.
pub fn head_block(m: &Mob) -> [i32; 3] {
    use horizon_core::geometry::trig::Mth;
    // Vec3.directionFromRotation(xRot, yRot).
    let to_rad = std::f32::consts::PI / 180.0;
    let y = -m.yaw * to_rad - std::f32::consts::PI;
    let x = -m.head_pitch * to_rad;
    let (y_cos, y_sin) = (y.mcos(), y.msin());
    let x_cos = -x.mcos();
    let forward = [f64::from(y_sin * x_cos), f64::from(y_cos * x_cos)];
    [
        (m.position.x + forward[0] * 2.25).floor() as i32,
        (m.position.y + f64::from(0.2_f32)).floor() as i32,
        (m.position.z + forward[1] * 2.25).floor() as i32,
    ]
}
/// `Sniffer.tick` and `Sniffer.aiStep` additions: the animation states,
/// the dropped seed and the water malus.
pub fn ai_step(m: &mut Mob, events: &mut Vec<MobEvent>) {
    use super::path::PathType;
    let current = state(m);
    animate(m);
    if current == DIGGING && now(m) == m.extra.ints[3] {
        let h = head_block(m);
        events.push(MobEvent::GiftLoot {
            mob: m.id,
            table: "minecraft:gameplay/sniffer_digging",
            at: [f64::from(h[0]), f64::from(h[1]), f64::from(h[2])],
        });
        events.push(MobEvent::Sound(
            "entity.sniffer.drop_seed",
            m.center(),
            1.0,
            1.0,
        ));
    }
    // The searching sound is the client's (`playSearchingSound`).
    // onPathfindingStart / Done: water only blocks the way while it is dry.
    let wet = m.fluids.in_water || m.fire_ticks > 0;
    m.set_malus(PathType::Water, if wet { 0.0 } else { -1.0 });
}
/// `Sniffer.onSyncedDataUpdated(DATA_STATE)`: the animation states of the
/// state it changed to (on a client, each tick for the state it has).
pub fn animate(m: &mut Mob) {
    let age = m.age as f32;
    let current = state(m);
    if current != m.extra.scratch[0] as u8 {
        // resetAnimations, then the state's own.
        for s in &mut m.anim.states[..5] {
            s.stop();
        }
        match current {
            FEELING_HAPPY => m.anim.states[0].start_if_stopped(age),
            SCENTING => m.anim.states[1].start_if_stopped(age),
            SNIFFING => m.anim.states[2].start_if_stopped(age),
            DIGGING => m.anim.states[3].start_if_stopped(age),
            RISING => m.anim.states[4].start_if_stopped(age),
            _ => {}
        }
        m.extra.scratch[0] = i32::from(current);
    }
}
/// The brain memory holding the explored positions
/// (`SNIFFER_EXPLORED_POSITIONS`).
pub fn to_nbt(m: &Mob, map: &mut horizon_world::nbt::Compound) {
    use horizon_world::nbt::Tag;
    let list: Vec<Tag> = m
        .extra
        .positions
        .iter()
        .map(|p| {
            Tag::compound([
                ("dimension", Tag::String("minecraft:overworld".into())),
                ("pos", Tag::IntArray(p.to_vec())),
            ])
        })
        .collect();
    let mut memories = horizon_world::nbt::Compound::new();
    if !list.is_empty() {
        memories.insert(
            "minecraft:sniffer_explored_positions".into(),
            Tag::compound([("value", Tag::List(list))]),
        );
    }
    map.insert(
        "Brain".into(),
        Tag::compound([("memories", Tag::Compound(memories))]),
    );
}
pub fn from_nbt(m: &mut Mob, tag: &horizon_world::nbt::Tag) {
    use horizon_world::nbt::Tag;
    let Some(Tag::List(list)) = tag
        .get("Brain")
        .and_then(|b| b.get("memories"))
        .and_then(|b| b.get("minecraft:sniffer_explored_positions"))
        .and_then(|b| b.get("value"))
    else {
        return;
    };
    m.extra.positions = list
        .iter()
        .filter_map(|e| match e.get("pos") {
            Some(Tag::IntArray(p)) if p.len() == 3 => Some([p[0], p[1], p[2]]),
            _ => None,
        })
        .collect();
}
/// `Sniffer.canMate`'s state test.
pub fn can_mate_state(s: u8) -> bool {
    matches!(s, IDLING | SCENTING | FEELING_HAPPY)
}
/// Ambient sound: none while digging or searching.
pub fn ambient(m: &Mob) -> Option<&'static str> {
    (!matches!(state(m), DIGGING | SEARCHING)).then_some("entity.sniffer.idle")
}
/// `SnifferModel.setupAnim`, babies included: 1.21.10 applies
/// `SnifferAnimation.BABY_TRANSFORM` statically for a baby (26.1's
/// separate SniffletModel never runs it).
pub fn anim(m: &Mob, s: &State, p: &mut Poses, animations: &HashMap<String, Animation>) {
    let get = |n: &str| animations.get(&format!("SnifferAnimation.{n}"));
    p.xrot("head", s.x_rot.to_radians());
    p.yrot("head", s.y_rot.to_radians());
    let walk = if state(m) == SEARCHING {
        "SNIFFER_SNIFF_SEARCH"
    } else {
        "SNIFFER_WALK"
    };
    if let Some(a) = get(walk) {
        a.apply_walk(s.pos, s.speed, 9.0, 100.0, p);
    }
    let age = s.age;
    for (st, name) in [
        (&m.anim.states[3], "SNIFFER_DIG"),
        (&m.anim.states[2], "SNIFFER_LONGSNIFF"),
        (&m.anim.states[4], "SNIFFER_STAND_UP"),
        (&m.anim.states[0], "SNIFFER_HAPPY"),
        (&m.anim.states[1], "SNIFFER_SNIFFSNIFF"),
    ] {
        if let (Some(ms), Some(a)) = (st.millis(age), get(name)) {
            a.apply(ms, 1.0, p);
        }
    }
    // `SnifferModel.setupAnim`: a baby applies `BABY_TRANSFORM` statically
    // (head ×1.2, + (0, 1, 1)) after the other animations. The baby layer
    // is the adult geometry under `BABY_TRANSFORMER` (0.5 scale), not
    // 26.1's separate SniffletModel geometry.
    if m.is_baby() {
        if let Some(a) = get("BABY_TRANSFORM") {
            a.apply(0, 1.0, p);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobs::{
        Anim, Brain, Flags, MobType, Part, Species,
        anim::{self, Poses},
    };
    use horizon_core::geometry::Vec3;
    use serde_json::json;
    use std::sync::Arc;

    fn sniffer() -> Mob {
        let model = Arc::new(Part::parse("root", &json!({"children": {"head": {}}})));
        let kind = Arc::new(MobType {
            id: "minecraft:sniffer".into(),
            species: Species::Sniffer,
            width: 1.9,
            height: 1.75,
            eye_height: 1.7,
            category: "creature".into(),
            max_health: 14.0,
            speed: 0.25,
            flying_speed: 0.4,
            attack_damage: 0.0,
            attack_knockback: 0.0,
            follow_range: 16.0,
            step_height: 0.6,
            armor: 0.0,
            armor_toughness: 0.0,
            knockback_resistance: 0.0,
            jump_strength: 0.42,
            attributes: Arc::default(),
            item_stack_limits: Default::default(),
            gravity: 0.08,
            safe_fall_distance: 3.0,
            fall_damage_multiplier: 1.0,
            water_efficiency: 0.0,
            water_slowdown: 0.8,
            air_speed: 0.02,
            baby_scale: 0.5,
            head_rot_speed: 10.0,
            max_head_x_rot: 40.0,
            max_head_y_rot: 75.0,
            ambient_interval: 120,
            flags: Flags {
                animal: true,
                ..Default::default()
            },
            model,
            layers: Default::default(),
            texture: String::new(),
            brain: Brain::Passive,
            anim: Anim::Sniffer,
            sounds: [""; 3],
            step_sound: None,
            food: Arc::default(),
            tempt: Arc::default(),
        });
        Mob::new(1, kind, Vec3::ZERO, 0.0)
    }

    #[test]
    fn only_calm_states_can_mate() {
        assert!(can_mate_state(IDLING));
        assert!(can_mate_state(SCENTING));
        assert!(can_mate_state(FEELING_HAPPY));
        assert!(!can_mate_state(SNIFFING));
        assert!(!can_mate_state(DIGGING));
    }

    #[test]
    fn a_baby_sniffer_applies_the_baby_transform_and_an_adult_does_not() {
        // `SnifferModel.setupAnim`: BABY_TRANSFORM (head ×1.2, + (0, 1, 1))
        // applies statically for a baby; 26.1's SniffletModel never runs it.
        let baby_transform = Animation::parse(&json!({
            "bones": {"head": [
                {"target": "scale", "keyframes": [[0.0, [1.2, 1.2, 1.2], [1.2, 1.2, 1.2]]]},
                {"target": "position", "keyframes": [[0.0, [0.0, 1.0, 1.0], [0.0, 1.0, 1.0]]]}
            ]}
        }));
        let animations =
            HashMap::from([("SnifferAnimation.BABY_TRANSFORM".to_owned(), baby_transform)]);
        let head_pose = |age: i32| {
            let mut m = sniffer();
            m.extra.age = age;
            let mut poses = Poses::new(&m.kind.model);
            let state = anim::state(&m, 0.5);
            anim(&m, &state, &mut poses, &animations);
            *poses.get("head")
        };
        let adult = head_pose(0);
        let baby = head_pose(BABY_START_AGE);
        for i in 0..3 {
            assert!((baby[i] - adult[i] - [0.0, 1.0, 1.0][i]).abs() < 1e-6);
            assert!((baby[i + 6] - adult[i + 6] - 1.2).abs() < 1e-6);
        }
    }

    #[test]
    fn summoned_age_selects_the_baby_layer_and_the_adult_texture() {
        // 1.21.10: babies use the sniffer_baby layer but the adult texture
        // (26.x gives babies their own snifflet texture).
        let mut template = sniffer();
        let kind = Arc::make_mut(&mut template.kind);
        kind.texture = "entity/sniffer/sniffer".into();
        kind.layers
            .insert("sniffer_baby#main".into(), kind.model.clone());
        let types = HashMap::from([(kind.id.clone(), template.kind.clone())]);
        for (age, baby) in [
            (BABY_START_AGE, true),
            (-1, true),
            (0, false),
            (6000, false),
        ] {
            let (tag, _) = horizon_world::nbt::parse_snbt(&format!(
                "{{id:\"minecraft:sniffer\",Pos:[0.0d,0.0d,0.0d],Age:{age}}}"
            ))
            .unwrap();
            let m = crate::entity_nbt::mob_from_nbt(&tag, &types, 2, None).unwrap();
            assert_eq!(m.extra.age, age);
            assert_eq!(m.is_baby(), baby);
            assert_eq!(
                crate::mobs::render::body(&m),
                (
                    if baby {
                        "sniffer_baby#main".into()
                    } else {
                        "sniffer#main".into()
                    },
                    "entity/sniffer/sniffer".into()
                )
            );
        }
    }
}
