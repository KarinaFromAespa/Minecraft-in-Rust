//! Clientbound world packets in 26.2's layout for a 26.2 client (#10404, the mirror of `translate`'s
//! hand-written 26.2 readers): chunk columns and light, particles, explosions, entity moves and hand
//! swings. Everything else in a packet keeps its pinned layout; ids go down as the rest of
//! [`super`] renumbers them, with stand-ins where 26.2 lacks an entry.
use super::{Sent, state_down};
use crate::{
    Result,
    adapter::ids::{self, Ids, Way, down_or_substitute},
    buf::{Codec, Reader, Writer},
    chunk::{self, Paletted, Section, Strategy},
    entity_data::{Particle, ParticleData},
    ids as packet_ids,
    packets::play,
};

/// `SoundEvents.EMPTY`: what a 26.2 client is told to play for a blast 26.3 keeps silent.
const SILENT: &str = "minecraft:intentionally_empty";

/// The pinned play packet `id` in 26.2's layout; `None` when 26.2 lays it out as the pinned version does.
pub(super) fn down(ids: &'static Ids, id: i32, payload: &[u8]) -> Result<Option<Sent>> {
    use packet_ids::play::clientbound as cb;
    Ok(Some(match id {
        cb::LEVEL_CHUNK_WITH_LIGHT => Sent::Rewritten(level_chunk(ids, payload)?),
        cb::LIGHT_UPDATE => Sent::Rewritten(light_update(payload)?),
        cb::LEVEL_PARTICLES => particles(ids, payload)?,
        cb::EXPLODE => explode(ids, payload)?,
        cb::ADD_ENTITY => add_entity(ids, payload)?,
        cb::SET_ENTITY_MOTION => Sent::Rewritten(set_entity_motion(payload)?),
        cb::PLAYER_ROTATION => Sent::Rewritten(player_rotation(payload)?),
        cb::SET_DEFAULT_SPAWN_POSITION => Sent::Rewritten(spawn_position(payload)?),
        cb::MOVE_ENTITY_POS | cb::MOVE_ENTITY_POS_ROT | cb::MOVE_ENTITY_ROT => {
            Sent::Rewritten(move_entity(id, payload)?)
        }
        cb::ENTITY_POSITION_SYNC => Sent::Rewritten(position_sync(payload)?),
        cb::ANIMATE => animate(payload)?,
        _ => return Ok(None),
    }))
}

/// 26.2's add-entity carries the velocity as an LpVec3 before the angles;
/// 1.21.8 sends three shorts after the data. The kind and a falling block's
/// state take 26.2's ids, as [`super::renumber`] would.
fn add_entity(ids: &'static Ids, payload: &[u8]) -> Result<Sent> {
    use crate::{
        adapter::pinned_ids::entity_type::FALLING_BLOCK,
        buf::{LpVec3, Vec3d},
    };
    let mut e = play::AddEntity::read(&mut Reader::new(payload))?;
    if e.kind == FALLING_BLOCK {
        e.data = state_down(ids, e.data);
    }
    let Some(kind) = down_or_substitute(
        ids.entity_type,
        ids.substitutes.entity_type,
        e.kind,
    ) else {
        return Ok(Sent::Dropped);
    };
    e.kind = kind;
    let mut w = Writer::new();
    w.varint(e.id);
    w.uuid(e.uuid);
    w.varint(e.kind);
    Vec3d::write(&e.position, &mut w);
    LpVec3::write(&e.velocity, &mut w);
    w.i8(e.pitch);
    w.i8(e.yaw);
    w.i8(e.head_yaw);
    w.varint(e.data);
    Ok(Sent::Rewritten(w.data))
}

/// 26.2's set-entity-motion carries the velocity as an LpVec3; 1.21.8 sends
/// three shorts.
fn set_entity_motion(payload: &[u8]) -> Result<Vec<u8>> {
    use crate::buf::LpVec3;
    let m = play::SetEntityMotion::read(&mut Reader::new(payload))?;
    let mut w = Writer::new();
    w.varint(m.id);
    LpVec3::write(&m.velocity, &mut w);
    Ok(w.data)
}

/// 26.2's player rotation carries the relative flags; 1.21.8's is always
/// absolute.
fn player_rotation(payload: &[u8]) -> Result<Vec<u8>> {
    let p = play::PlayerRotation::read(&mut Reader::new(payload))?;
    let mut w = Writer::new();
    w.f32(p.yaw);
    w.bool(false);
    w.f32(p.pitch);
    w.bool(false);
    Ok(w.data)
}

/// 26.2's spawn position carries the dimension and the pitch; 1.21.8 sends
/// the position and the yaw only. The pinned packet carries no dimension:
/// the world spawn is the Overworld's (a mod-set compass target in another
/// dimension cannot survive the pinned packet either way).
fn spawn_position(payload: &[u8]) -> Result<Vec<u8>> {
    let p = play::SetDefaultSpawnPosition::read(&mut Reader::new(payload))?;
    let mut w = Writer::new();
    w.string("minecraft:overworld");
    crate::buf::Pos::write(&p.pos, &mut w);
    w.f32(p.yaw);
    w.f32(0.0);
    Ok(w.data)
}

/// The column with its block states renumbered (stand-ins for states 26.2 lacks) and packed for 26.2's
/// narrower global palette, the block entities 26.2 has, and the light masks as 26.2 sends them. Biomes are
/// the server's own registry and stay as they are.
fn level_chunk(ids: &'static Ids, payload: &[u8]) -> Result<Vec<u8>> {
    let mut p = play::LevelChunkWithLight::read(&mut Reader::new(payload))?;
    let (old_bits, new_bits) = ids.state_bits();
    let mut r = Reader::new(&p.chunk.buffer);
    let mut sections = Vec::new();
    while r.remaining() > 0 {
        let mut s = Section::read(&mut r, new_bits, 0)?;
        s.states = states_down(ids, &s.states, old_bits);
        sections.push(s);
    }
    p.chunk.buffer = chunk::ChunkData::from_sections(&sections);
    // A block entity type 26.2 lacks goes; its block still draws.
    p.chunk.block_entities.retain_mut(|b| {
        ids.block_entity_type
            .down(b.kind)
            .map(|kind| b.kind = kind)
            .is_some()
    });
    let mut w = Writer::new();
    w.i32(p.x);
    w.i32(p.z);
    chunk::ChunkData::write(&p.chunk, &mut w);
    light_down(&p.light, &mut w);
    Ok(w.data)
}

/// A section's states in 26.2's numbering: a palette's entries renumbered, or the global palette repacked
/// at 26.2's width (`old_bits`).
fn states_down(ids: &Ids, states: &Paletted, old_bits: u8) -> Paletted {
    if !states.palette.is_empty() {
        let mut out = states.clone();
        out.palette
            .iter_mut()
            .for_each(|p| *p = state_down(ids, *p));
        return out;
    }
    let values: Vec<i32> = (0..chunk::BLOCKS)
        .map(|i| state_down(ids, states.get(i)))
        .collect();
    Paletted::from_values(&values, Strategy::Blocks, old_bits)
}

/// 26.2's ClientboundLightUpdatePacketData: each mask as `BitSet.toLongArray` (a VarInt count of longs),
/// where 26.3 sends `toByteArray` (the same bits as little-endian bytes); the layers as they are.
fn light_down(light: &chunk::LightData, w: &mut Writer) {
    for mask in [
        &light.sky_mask,
        &light.block_mask,
        &light.empty_sky_mask,
        &light.empty_block_mask,
    ] {
        w.long_array(&mask_longs(mask));
    }
    chunk::Layers::write(&light.sky, w);
    chunk::Layers::write(&light.block, w);
}

/// A mask's bytes as the longs `BitSet.toLongArray` gives: none past the last set bit.
fn mask_longs(bytes: &[u8]) -> Vec<i64> {
    let mut longs: Vec<i64> = bytes
        .chunks(8)
        .map(|c| {
            let mut b = [0u8; 8];
            b[..c.len()].copy_from_slice(c);
            i64::from_le_bytes(b)
        })
        .collect();
    while longs.last() == Some(&0) {
        longs.pop();
    }
    longs
}

fn light_update(payload: &[u8]) -> Result<Vec<u8>> {
    let p = play::LightUpdate::read(&mut Reader::new(payload))?;
    let mut w = Writer::new();
    w.varint(p.x);
    w.varint(p.z);
    light_down(&p.light, &mut w);
    Ok(w.data)
}

/// A particle in 26.2's numbering, its block state too, or its stand-in; `None` when 26.2 has neither (a
/// native mod's particle). The options stay: the stand-ins in the 26.2 tables (pinned 43..=45 as 41)
/// carry none, as the particles they stand for; a stand-in with other options would need them rebuilt.
fn particle_down(ids: &Ids, p: &Particle) -> Option<Particle> {
    let mut p = p.clone();
    p.id = down_or_substitute(ids.particle_type, ids.substitutes.particle_type, p.id)?;
    if let Some(ParticleData::State(s)) = &mut p.data {
        *s = state_down(ids, *s);
    }
    Some(p)
}

/// A particle written for 26.2: an item in its options takes 26.2's item and component ids.
fn write_particle(ids: &'static Ids, p: &Particle, w: &mut Writer) {
    let _down = ids::renumbering(ids, Way::Down);
    p.write(w);
}

/// 26.2: flags, position, spread, one max speed, an Int count, then the particle. 26.3 sends the particle
/// first, a speed per axis, a VarInt count and how the spread is drawn, which 26.2 can't be told (it always
/// draws a gaussian spread). Of the three speeds the largest goes.
fn particles(ids: &'static Ids, payload: &[u8]) -> Result<Sent> {
    let p = play::LevelParticles::read(&mut Reader::new(payload))?;
    let Some(particle) = particle_down(ids, &p.particle) else {
        return Ok(Sent::Dropped);
    };
    let speed = p
        .max_speed
        .into_iter()
        .fold(0.0f32, |a, v| if v.abs() > a.abs() { v } else { a });
    let mut w = Writer::new();
    w.bool(p.override_limiter);
    w.bool(p.always_show);
    p.pos.iter().for_each(|v| w.f64(*v));
    p.dist.iter().for_each(|v| w.f32(*v));
    w.f32(speed);
    w.i32(p.count);
    write_particle(ids, &particle, &mut w);
    Ok(Sent::Rewritten(w.data))
}

/// 26.2's explode carries the radius, the destroyed-block count and the
/// scattered block particles 1.21.8's packet lacks, and no trailing "sound
/// plays" flag: 26.2 always plays the sound, so a silent blast names
/// `SoundEvents.EMPTY`. The radius and count are unknown to the pinned
/// packet, so 26.2 gets 0 and an empty particle list. Its particle and sound
/// take 26.2's ids.
fn explode(ids: &'static Ids, payload: &[u8]) -> Result<Sent> {
    use crate::buf::{Opt, Vec3d};
    let mut e = play::Explode::read(&mut Reader::new(payload))?;
    let Some(particle) = particle_down(ids, &e.particle) else {
        return Ok(Sent::Dropped);
    };
    e.particle = particle;
    let silent = || (-1, Some((SILENT.to_string(), None)));
    if !e.play_sound {
        e.sound = silent();
    } else if e.sound.1.is_none() {
        // An inline sound is named, not numbered.
        match down_or_substitute(ids.sound_event, ids.substitutes.sound_event, e.sound.0) {
            Some(id) => e.sound.0 = id,
            None => e.sound = silent(),
        }
    }
    let mut w = Writer::new();
    {
        let _down = ids::renumbering(ids, Way::Down);
        Vec3d::write(&e.center, &mut w);
        w.f32(0.0);
        w.i32(0);
        <Opt<Vec3d>>::write(&e.knockback, &mut w);
        e.particle.write(&mut w);
        play::SoundHolder::write(&e.sound, &mut w);
        w.varint(0);
    }
    Ok(Sent::Rewritten(w.data))
}

/// 26.2 sends a move as three Short deltas (1/4096 of a block) then on-ground, and a turn as the two angles
/// then on-ground; 26.3 packs on-ground with the delta's step count and sends it before the angles. A
/// stepped delta goes as its sum (the next position sync corrects a sum past a Short).
fn move_entity(id: i32, payload: &[u8]) -> Result<Vec<u8>> {
    use packet_ids::play::clientbound as cb;
    let mut r = Reader::new(payload);
    let m = match id {
        cb::MOVE_ENTITY_POS => play::MoveEntityPos::read(&mut r)?.0,
        cb::MOVE_ENTITY_POS_ROT => play::MoveEntityPosRot::read(&mut r)?.0,
        _ => play::MoveEntityRot::read(&mut r)?.0,
    };
    let mut w = Writer::new();
    w.varint(m.id);
    if id != cb::MOVE_ENTITY_ROT {
        delta_sum(m.delta.as_ref()).iter().for_each(|v| w.i16(*v));
    }
    if id != cb::MOVE_ENTITY_POS {
        let (yaw, pitch) = m.rotation.unwrap_or((0, 0));
        w.i8(yaw);
        w.i8(pitch);
    }
    w.bool(m.on_ground);
    Ok(w.data)
}

/// A delta as one linear step, clamped to a Short per axis.
fn delta_sum(delta: Option<&play::VecDelta>) -> [i16; 3] {
    match delta {
        None => [0; 3],
        Some(play::VecDelta::Linear(d)) => *d,
        Some(play::VecDelta::Stepped(steps)) => {
            let mut sum = [0i32; 3];
            for (d, _) in steps {
                for (s, v) in sum.iter_mut().zip(d) {
                    *s += i32::from(*v);
                }
            }
            sum.map(|v| v.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16)
        }
    }
}

/// 26.2: the entity, PositionMoveRotation (position, motion, the two angles), on-ground. 26.3 sends where it
/// goes as a path, which 26.2 is told the end of, and no motion (zero).
fn position_sync(payload: &[u8]) -> Result<Vec<u8>> {
    let p = play::EntityPositionSync::read(&mut Reader::new(payload))?;
    let mut w = Writer::new();
    w.varint(p.id);
    p.path.end().iter().for_each(|v| w.f64(*v));
    (0..3).for_each(|_| w.f64(0.0));
    w.f32(p.yaw);
    w.f32(p.pitch);
    w.bool(p.on_ground);
    Ok(w.data)
}

/// 26.3's animate actions as 26.2 numbers them (wake up 0 -> 2, critical hits 1, 2 -> 4, 5); 26.2's own 0
/// and 3 swing hands, which 26.3 sends as `swing_animation` ([`swing`]).
/// The pinned animate actions are already 26.2's numbers (both number the
/// swings 0 and 3, wake up 2, the two critical hits 4 and 5), so the packet
/// crosses unchanged; the read is the validation.
fn animate(payload: &[u8]) -> Result<Sent> {
    let p = play::Animate::read(&mut Reader::new(payload))?;
    if !matches!(p.action, 0 | 2 | 3 | 4 | 5) {
        return Ok(Sent::Dropped);
    }
    let mut w = Writer::new();
    w.varint(p.id);
    w.u8(p.action);
    Ok(Sent::Rewritten(w.data))
}

#[cfg(test)]
mod tests {
    //! Each packet written in 26.2's layout here is read back with `translate`'s 26.2 reader, which brings
    //! it to the pinned layout again.
    use super::super::super::{ServerVersion, translate};
    use super::*;
    use crate::State;
    use horizon_world::nbt::Tag;

    fn ids() -> &'static Ids {
        ServerVersion::V26_2.ids().unwrap()
    }

    fn in_play() -> super::super::ClientSession {
        use std::sync::atomic::Ordering;
        let s = super::super::ClientSession::new(ServerVersion::V26_2);
        let play = super::super::to_u8(State::Play);
        s.reading.store(play, Ordering::Release);
        s.writing.store(play, Ordering::Release);
        s
    }

    fn rewritten(sent: Option<Sent>) -> Vec<u8> {
        match sent {
            Some(Sent::Rewritten(body)) => body,
            Some(Sent::AsIs) => panic!("sent as is"),
            Some(Sent::Dropped) => panic!("dropped"),
            None => panic!("not a world packet"),
        }
    }

    fn encode<T: Codec<T>>(p: &T) -> Vec<u8> {
        let mut w = Writer::new();
        T::write(p, &mut w);
        w.data
    }

    fn light() -> chunk::LightData {
        chunk::LightData {
            // Sections 0 and 2; section 9 (past the first byte); section 64 (past the first long).
            sky_mask: vec![0b101],
            block_mask: vec![0, 0b10],
            empty_sky_mask: vec![0, 0, 0, 0, 0, 0, 0, 0, 1],
            empty_block_mask: Vec::new(),
            sky: vec![vec![0x11; 2048], vec![0xf0; 2048]],
            block: vec![vec![0x07; 2048]],
        }
    }

    #[test]
    fn a_mask_becomes_the_longs_of_bitset_to_long_array() {
        assert_eq!(mask_longs(&[0b101]), vec![0b101]);
        assert_eq!(mask_longs(&[0, 0b10]), vec![1 << 9]);
        assert_eq!(mask_longs(&[0, 0, 0, 0, 0, 0, 0, 0, 1]), vec![0, 1]);
        assert!(mask_longs(&[]).is_empty());
    }

    #[test]
    fn a_chunk_goes_down_and_back_up() {
        let ids = ids();
        // A section of one state (calcite: 23376 pinned, 27160 in 26.2), and one on the global palette:
        // 4096 states that 26.2 has too (its 13000.., oak_slab's 13330 among them).
        let one = Paletted::from_values(&[23376; chunk::BLOCKS], Strategy::Blocks, 15);
        let many: Vec<i32> = (0..4096)
            .map(|i| ids.block_state.up(13000 + i).unwrap_or(1))
            .collect();
        let global = Paletted::from_values(&many, Strategy::Blocks, 15);
        assert_eq!(global.bits, 15);
        let plains = Paletted::from_values(&[1; chunk::BIOMES], Strategy::Biomes, 0);
        let section = |states: Paletted| Section {
            non_empty: 4096,
            fluids: 0,
            states,
            biomes: plains.clone(),
        };
        let sections = vec![section(one), section(global)];
        // A block entity of the first type 26.2 has.
        let kind = ids.block_entity_type.up(0).unwrap();
        let p = play::LevelChunkWithLight {
            x: 3,
            z: -7,
            chunk: chunk::ChunkData {
                heightmaps: vec![(1, vec![0x0101; 37])],
                buffer: chunk::ChunkData::from_sections(&sections),
                block_entities: vec![chunk::BlockEntityInfo {
                    packed_xz: 0x12,
                    y: 64,
                    kind,
                    tag: Tag::Compound(Default::default()),
                }],
            },
            light: light(),
        };
        let old = rewritten(
            down(
                ids,
                packet_ids::play::clientbound::LEVEL_CHUNK_WITH_LIGHT,
                &encode(&p),
            )
            .unwrap(),
        );
        // In 26.2's own numbering: the calcite is 27160, the global palette 15 bits wide, 26.2's ids.
        {
            let mut r = Reader::new(&old);
            r.i32().unwrap();
            r.i32().unwrap();
            let c = chunk::ChunkData::read(&mut r).unwrap();
            let s = c.sections(2, ids.state_bits().0, 0).unwrap();
            assert_eq!(s[0].states.get(0), 27160);
            assert_eq!(s[1].states.bits, 15);
            assert_eq!(s[1].states.get(330), 13330);
            assert_eq!(c.block_entities[0].kind, 0);
        }
        let back = translate::chunk(
            &crate::adapter::session::Session::new(crate::adapter::ServerVersion::V26_2),
            ids,
            &old,
        )
        .unwrap();
        let q = play::LevelChunkWithLight::read(&mut Reader::new(&back)).unwrap();
        assert_eq!((q.x, q.z), (3, -7));
        assert_eq!(q.chunk.heightmaps, p.chunk.heightmaps);
        assert_eq!(q.chunk.block_entities, p.chunk.block_entities);
        assert_eq!(q.light, p.light);
        let s = q.chunk.sections(2, ids.state_bits().1, 0).unwrap();
        for (a, b) in s.iter().zip(&sections) {
            for i in 0..chunk::BLOCKS {
                assert_eq!(a.states.get(i), b.states.get(i));
            }
            assert_eq!(a.biomes, b.biomes);
        }
    }

    #[test]
    fn a_light_update_goes_down_and_back_up() {
        let ids = ids();
        let p = play::LightUpdate {
            x: -1,
            z: 9,
            light: light(),
        };
        let old = rewritten(
            down(
                ids,
                packet_ids::play::clientbound::LIGHT_UPDATE,
                &encode(&p),
            )
            .unwrap(),
        );
        let back = translate::light_update(&old).unwrap();
        assert_eq!(play::LightUpdate::read(&mut Reader::new(&back)).unwrap(), p);
    }

    #[test]
    fn particles_go_down_and_back_up() {
        let ids = ids();
        let send = |particle: Particle| {
            let p = play::LevelParticles {
                particle,
                override_limiter: true,
                always_show: false,
                pos: [1.5, 64.0, -3.25],
                dist: [0.5, 0.25, 0.5],
                max_speed: [0.1; 3],
                count: 12,
                randomization: 0,
            };
            let old = rewritten(
                down(
                    ids,
                    packet_ids::play::clientbound::LEVEL_PARTICLES,
                    &encode(&p),
                )
                .unwrap(),
            );
            let back = translate::particles(ids, &old).unwrap();
            let q = play::LevelParticles::read(&mut Reader::new(&back)).unwrap();
            assert_eq!(
                (
                    q.override_limiter,
                    q.always_show,
                    q.pos,
                    q.dist,
                    q.max_speed,
                    q.count
                ),
                (
                    p.override_limiter,
                    p.always_show,
                    p.pos,
                    p.dist,
                    p.max_speed,
                    p.count
                )
            );
            q.particle
        };
        // entity_effect (20 in 1.21.8, 28 in 26.2) carries a colour, alike
        // in both numberings.
        let colour = Particle {
            id: 20,
            data: Some(ParticleData::Color(0x00ff00ff)),
        };
        assert_eq!(send(colour.clone()), colour);
        // block (1) carries a state: chest's 3018 goes down to 3987 and comes back.
        let block = Particle {
            id: 1,
            data: Some(ParticleData::State(3018)),
        };
        assert_eq!(send(block.clone()), block);
        // ash is 81 in 1.21.8 and 91 in 26.2: renumbered both ways, it comes
        // back as itself.
        assert_eq!(
            send(Particle { id: 81, data: None }),
            Particle { id: 81, data: None }
        );
    }

    #[test]
    fn of_three_speeds_the_largest_goes() {
        let ids = ids();
        let p = play::LevelParticles {
            particle: Particle {
                id: 20,
                data: Some(ParticleData::Color(0)),
            },
            override_limiter: false,
            always_show: false,
            pos: [0.0; 3],
            dist: [0.0; 3],
            max_speed: [0.1, -0.4, 0.2],
            count: 0,
            randomization: 1,
        };
        let old = rewritten(
            down(
                ids,
                packet_ids::play::clientbound::LEVEL_PARTICLES,
                &encode(&p),
            )
            .unwrap(),
        );
        // Two flags, three doubles, three floats, then the speed.
        let mut r = Reader::new(&old[2 + 24 + 12..]);
        assert_eq!(r.f32().unwrap(), -0.4);
        assert_eq!(r.i32().unwrap(), 0);
    }

    fn blast(play_sound: bool, sound: i32) -> play::Explode {
        play::Explode {
            center: [10.0, 70.0, -4.0],
            knockback: Some([0.1, 0.2, 0.3]),
            particle: Particle {
                id: 20,
                data: Some(ParticleData::Color(-1)),
            },
            sound: (sound, None),
            play_sound,
        }
    }

    /// 26.2's explode read back as a pinned one: the radius, count and block
    /// particles 26.2 carries are asserted 0/empty on the way (the pinned
    /// packet has none), ids renumbered up. 26.2 has no sound flag: its sound
    /// always plays.
    fn explode_up(old: &[u8]) -> play::Explode {
        use crate::buf::{Opt, Vec3d};
        let _up = ids::renumbering(ids(), Way::Up);
        let mut r = Reader::new(old);
        let center = Vec3d::read(&mut r).unwrap();
        assert_eq!(r.f32().unwrap(), 0.0);
        assert_eq!(r.i32().unwrap(), 0);
        let knockback = <Opt<Vec3d>>::read(&mut r).unwrap();
        let particle = Particle::read(&mut r).unwrap();
        let sound = play::SoundHolder::read(&mut r).unwrap();
        assert_eq!(r.varint().unwrap(), 0);
        play::Explode {
            center,
            knockback,
            particle,
            sound,
            play_sound: true,
        }
    }

    #[test]
    fn an_explosion_goes_down_without_the_sound_flag() {
        let ids = ids();
        // A sound event both versions number alike.
        let sound = (0..400)
            .find(|&s| ids.sound_event.down(s) == Some(s))
            .unwrap();
        let e = blast(true, sound);
        let old =
            rewritten(down(ids, packet_ids::play::clientbound::EXPLODE, &encode(&e)).unwrap());
        // 26.2's layout adds the radius (4), the block count (4) and the
        // block-particle count (1), and drops the flag (1).
        assert_eq!(old.len(), encode(&e).len() + 8);
        assert_eq!(explode_up(&old), e);
    }

    #[test]
    fn a_silent_explosion_names_the_empty_sound() {
        let ids = ids();
        let old = rewritten(
            down(
                ids,
                packet_ids::play::clientbound::EXPLODE,
                &encode(&blast(false, 0)),
            )
            .unwrap(),
        );
        assert_eq!(
            explode_up(&old).sound,
            (-1, Some((SILENT.to_string(), None)))
        );
    }

    #[test]
    fn moves_go_down_and_back_up() {
        use packet_ids::play::clientbound as cb;
        let ids = ids();
        let m = |delta: Option<play::VecDelta>, rotation: Option<(i8, i8)>| play::MoveEntity {
            id: 7,
            delta,
            rotation,
            on_ground: true,
        };
        let pos = play::MoveEntityPos(m(Some(play::VecDelta::Linear([4096, 0, -2048])), None));
        let old = rewritten(down(ids, cb::MOVE_ENTITY_POS, &encode(&pos)).unwrap());
        assert_eq!(old.len(), 1 + 6 + 1);
        let back = translate::move_entity(&old, true, false).unwrap();
        assert_eq!(
            play::MoveEntityPos::read(&mut Reader::new(&back)).unwrap(),
            pos
        );

        let pos_rot =
            play::MoveEntityPosRot(m(Some(play::VecDelta::Linear([1, 2, 3])), Some((10, -20))));
        let old = rewritten(down(ids, cb::MOVE_ENTITY_POS_ROT, &encode(&pos_rot)).unwrap());
        let back = translate::move_entity(&old, true, true).unwrap();
        assert_eq!(
            play::MoveEntityPosRot::read(&mut Reader::new(&back)).unwrap(),
            pos_rot
        );

        let rot = play::MoveEntityRot(m(None, Some((-5, 90))));
        let old = rewritten(down(ids, cb::MOVE_ENTITY_ROT, &encode(&rot)).unwrap());
        assert_eq!(old, vec![7, -5i8 as u8, 90, 1]);
        let back = translate::move_entity(&old, false, true).unwrap();
        assert_eq!(
            play::MoveEntityRot::read(&mut Reader::new(&back)).unwrap(),
            rot
        );
    }

    #[test]
    fn a_stepped_move_goes_down_as_its_sum() {
        let stepped = play::VecDelta::Stepped(vec![
            ([100, 0, -5], 1),
            ([200, 30000, -5], 2),
            ([0, 30000, 0], 3),
        ]);
        assert_eq!(delta_sum(Some(&stepped)), [300, i16::MAX, -10]);
    }

    #[test]
    fn a_position_sync_goes_down_and_back_up() {
        let ids = ids();
        let p = play::EntityPositionSync {
            id: 42,
            path: play::PositionPath::Linear([100.5, 64.0, -20.25]),
            yaw: 90.0,
            pitch: -10.0,
            on_ground: false,
        };
        let old = rewritten(
            down(
                ids,
                packet_ids::play::clientbound::ENTITY_POSITION_SYNC,
                &encode(&p),
            )
            .unwrap(),
        );
        assert_eq!(old.len(), 1 + 48 + 8 + 1);
        let back = translate::position_sync(&old).unwrap();
        assert_eq!(
            play::EntityPositionSync::read(&mut Reader::new(&back)).unwrap(),
            p
        );
        // A stepped path goes as where it ends.
        let stepped = play::EntityPositionSync {
            path: play::PositionPath::Stepped(vec![([1.0, 2.0, 3.0], 1), ([4.0, 5.0, 6.0], 2)]),
            ..p
        };
        let old = rewritten(
            down(
                ids,
                packet_ids::play::clientbound::ENTITY_POSITION_SYNC,
                &encode(&stepped),
            )
            .unwrap(),
        );
        let back = translate::position_sync(&old).unwrap();
        let q = play::EntityPositionSync::read(&mut Reader::new(&back)).unwrap();
        assert_eq!(q.path, play::PositionPath::Linear([4.0, 5.0, 6.0]));
    }

    #[test]
    fn animate_actions_are_already_26_2_s_numbers() {
        let ids = ids();
        let send =
            |action: u8| down(ids, packet_ids::play::clientbound::ANIMATE, &[5, action]).unwrap();
        // The swings, wake up and the two critical hits cross unchanged.
        for action in [0u8, 3, 2, 4, 5] {
            assert_eq!(rewritten(send(action)), vec![5, action]);
        }
        // An action neither version has is dropped.
        assert!(matches!(
            down(ids, packet_ids::play::clientbound::ANIMATE, &[5, 1]).unwrap(),
            Some(Sent::Dropped)
        ));
    }

    #[test]
    fn a_swing_reaches_a_26_2_client_as_animate() {
        let s = in_play();
        let swing = play::Clientbound::Animate(play::Animate { id: 5, action: 3 });
        let sent = s.clientbound(&swing.encode()).unwrap().unwrap();
        let mut r = Reader::new(&sent);
        let animate = ServerVersion::V26_2
            .clientbound_id_down(State::Play, packet_ids::play::clientbound::ANIMATE)
            .unwrap();
        assert_eq!(r.varint().unwrap(), animate);
        assert_eq!(play::Clientbound::decode(animate, r.rest()).unwrap(), swing);
    }

    #[test]
    fn a_light_update_through_the_session_is_in_26_2_s_layout() {
        let s = in_play();
        let p = play::Clientbound::LightUpdate(play::LightUpdate {
            x: 0,
            z: 0,
            light: light(),
        });
        let sent = s.clientbound(&p.encode()).unwrap().unwrap();
        let mut r = Reader::new(&sent);
        r.varint().unwrap();
        let back = translate::light_update(r.rest()).unwrap();
        assert_eq!(
            play::Clientbound::decode(packet_ids::play::clientbound::LIGHT_UPDATE, &back).unwrap(),
            p
        );
    }
}
