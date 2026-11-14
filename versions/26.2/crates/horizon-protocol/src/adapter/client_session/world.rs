//! Clientbound world packets in 26.1's layout for a 26.1 client (#10404, the mirror of `translate`'s
//! hand-written 26.1 readers): chunk columns and light, particles, explosions and entity moves.
//! Everything else in a packet keeps its pinned layout; ids go down as the rest of
//! [`super`] renumbers them (26.1 has no stand-in table, so what it lacks shows as air or is dropped).
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

/// `SoundEvents.EMPTY`: what a 26.1 client is told to play for a blast 26.2 keeps silent.
const SILENT: &str = "minecraft:intentionally_empty";

/// The pinned play packet `id` in 26.1's layout; `None` when 26.1 lays it out as the pinned version does.
pub(super) fn down(ids: &'static Ids, id: i32, payload: &[u8]) -> Result<Option<Sent>> {
    use packet_ids::play::clientbound as cb;
    Ok(Some(match id {
        cb::LEVEL_CHUNK_WITH_LIGHT => Sent::Rewritten(level_chunk(ids, payload)?),
        cb::LIGHT_UPDATE => Sent::Rewritten(light_update(payload)?),
        cb::LEVEL_PARTICLES => particles(ids, payload)?,
        cb::EXPLODE => explode(ids, payload)?,
        cb::MOVE_ENTITY_POS | cb::MOVE_ENTITY_POS_ROT | cb::MOVE_ENTITY_ROT => {
            Sent::Rewritten(move_entity(id, payload)?)
        }
        cb::ENTITY_POSITION_SYNC => Sent::Rewritten(position_sync(payload)?),
        _ => return Ok(None),
    }))
}

/// The column with its block states renumbered (air for states 26.1 lacks) and packed for 26.1's
/// narrower global palette, the block entities 26.1 has, and the light masks as 26.1 sends them. Biomes are
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
    // A block entity type 26.1 lacks goes; its block still draws.
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

/// A section's states in 26.1's numbering: a palette's entries renumbered, or the global palette repacked
/// at 26.1's width (`old_bits`).
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

/// 26.1's ClientboundLightUpdatePacketData: each mask as `BitSet.toLongArray` (a VarInt count of longs),
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

/// A particle in 26.1's numbering, its block state too; `None` when 26.1 has nothing for it (a native
/// mod's particle, or one 26.2 added). 26.1 has no stand-in table, so there is no fallback.
fn particle_down(ids: &Ids, p: &Particle) -> Option<Particle> {
    let mut p = p.clone();
    p.id = down_or_substitute(ids.particle_type, ids.substitutes.particle_type, p.id)?;
    if let Some(ParticleData::State(s)) = &mut p.data {
        *s = state_down(ids, *s);
    }
    Some(p)
}

/// A particle written for 26.1: an item in its options takes 26.1's item and component ids.
fn write_particle(ids: &'static Ids, p: &Particle, w: &mut Writer) {
    let _down = ids::renumbering(ids, Way::Down);
    p.write(w);
}

/// 26.1: flags, position, spread, one max speed, an Int count, then the particle. 26.2 sends the particle
/// first, one max speed and a VarInt count. 26.1 always draws a gaussian spread, as 26.2 does.
fn particles(ids: &'static Ids, payload: &[u8]) -> Result<Sent> {
    let p = play::LevelParticles::read(&mut Reader::new(payload))?;
    let Some(particle) = particle_down(ids, &p.particle) else {
        return Ok(Sent::Dropped);
    };
    let speed = p.max_speed;
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

/// 26.1's explode is 26.2's without the trailing "sound plays" flag: 26.1 always plays the sound, so a
/// silent blast names `SoundEvents.EMPTY`. Its particles and sound take 26.1's ids; a scattered particle
/// 26.1 has nothing for is left out.
fn explode(ids: &'static Ids, payload: &[u8]) -> Result<Sent> {
    let mut e = play::Explode::read(&mut Reader::new(payload))?;
    let Some(particle) = particle_down(ids, &e.particle) else {
        return Ok(Sent::Dropped);
    };
    e.particle = particle;
    e.block_particles = std::mem::take(&mut e.block_particles)
        .into_iter()
        .filter_map(|mut b| {
            b.particle = particle_down(ids, &b.particle)?;
            Some(b)
        })
        .collect();
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
        play::Explode::write(&e, &mut w);
    }
    w.data.pop();
    Ok(Sent::Rewritten(w.data))
}

/// 26.1 sends a move as three Short deltas (1/4096 of a block) then on-ground, and a turn as the two angles
/// then on-ground; 26.2 packs on-ground with the delta's step count and sends it before the angles. A
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

/// A delta as one linear step: 26.2's move packets carry a single delta.
fn delta_sum(delta: Option<&play::VecDelta>) -> [i16; 3] {
    delta.map_or([0; 3], |d| d.0)
}

/// 26.2: the entity, PositionMoveRotation (position, motion, the two angles), on-ground — the same form
/// 26.1 takes, so the packet goes down as it stands.
fn position_sync(payload: &[u8]) -> Result<Vec<u8>> {
    let p = play::EntityPositionSync::read(&mut Reader::new(payload))?;
    let mut w = Writer::new();
    w.varint(p.id);
    p.values.position.iter().for_each(|v| w.f64(*v));
    p.values.delta_movement.iter().for_each(|v| w.f64(*v));
    w.f32(p.values.yaw);
    w.f32(p.values.pitch);
    w.bool(p.on_ground);
    Ok(w.data)
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
        ServerVersion::V26_1.ids().unwrap()
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
        // A section of one state (calcite: 27160 pinned, 24687 in 26.1), and one on the global palette:
        // 4096 states that 26.1 has too (its 13000.., oak_slab's 13330 among them).
        let one = Paletted::from_values(&[27160; chunk::BLOCKS], Strategy::Blocks, 15);
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
        // A block entity of the first type 26.1 has.
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
        // In 26.1's own numbering: the calcite is 24687, the global palette 15 bits wide, 26.1's ids.
        {
            let mut r = Reader::new(&old);
            r.i32().unwrap();
            r.i32().unwrap();
            let c = chunk::ChunkData::read(&mut r).unwrap();
            let s = c.sections(2, ids.state_bits().0, 0).unwrap();
            assert_eq!(s[0].states.get(0), 24687);
            assert_eq!(s[1].states.bits, 15);
            assert_eq!(s[1].states.get(330), 13330);
            assert_eq!(c.block_entities[0].kind, 0);
        }
        let back = translate::chunk(
            &crate::adapter::session::Session::new(crate::adapter::ServerVersion::V26_1),
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
    fn a_state_crosses_down_in_26_1_s_numbering() {
        let ids = ids();
        // calcite's state: 27160 in 26.2, 24687 in 26.1.
        let p = Paletted::from_values(&[27160; chunk::BLOCKS], Strategy::Blocks, 15);
        assert_eq!(states_down(ids, &p, 15).get(0), 24687);
        // sulfur (24687, new in 26.2) has no stand-in for 26.1: it shows as air.
        let p = Paletted::from_values(&[24687; chunk::BLOCKS], Strategy::Blocks, 15);
        assert_eq!(states_down(ids, &p, 15).get(0), 0);
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
                max_speed: 0.1,
                count: 12,
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
        // entity_effect (28 in 26.2, 21 in 26.1) carries a colour, alike in both numberings.
        let colour = Particle {
            id: 28,
            data: Some(ParticleData::Color(0x00ff00ff)),
        };
        assert_eq!(send(colour.clone()), colour);
        // block (1) carries a state: chest's first state is 3987 in both
        // numberings, so it crosses unchanged and comes back.
        let block = Particle {
            id: 1,
            data: Some(ParticleData::State(3987)),
        };
        assert_eq!(send(block.clone()), block);
        // ash is 91 in 26.2 and 84 in 26.1: renumbered both ways, it comes
        // back as itself.
        assert_eq!(
            send(Particle { id: 91, data: None }),
            Particle { id: 91, data: None }
        );
    }

    #[test]
    fn a_single_speed_goes_down_unchanged() {
        let ids = ids();
        let p = play::LevelParticles {
            particle: Particle {
                id: 28,
                data: Some(ParticleData::Color(0)),
            },
            override_limiter: false,
            always_show: false,
            pos: [0.0; 3],
            dist: [0.0; 3],
            max_speed: -0.4,
            count: 0,
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
            radius: 4.0,
            block_count: 17,
            knockback: Some([0.1, 0.2, 0.3]),
            particle: Particle {
                id: 28,
                data: Some(ParticleData::Color(-1)),
            },
            sound: (sound, None),
            block_particles: vec![
                play::ExplosionParticle {
                    particle: Particle {
                        id: 28,
                        data: Some(ParticleData::Color(7)),
                    },
                    scaling: 1.0,
                    speed: 0.5,
                    weight: 3,
                },
                // A native mod's particle: 26.1 has nothing for it.
                play::ExplosionParticle {
                    particle: Particle {
                        id: crate::entity_data::MOD_PARTICLE_ID,
                        data: Some(ParticleData::Mod("mymod:spark".into())),
                    },
                    scaling: 1.0,
                    speed: 0.5,
                    weight: 1,
                },
            ],
            play_sound,
        }
    }

    /// 26.1's explode read back as `translate` reads it: the flag 26.1 lacks appended, ids renumbered up.
    fn explode_up(old: &[u8]) -> play::Explode {
        let mut body = old.to_vec();
        body.push(1);
        let _up = ids::renumbering(ids(), Way::Up);
        play::Explode::read(&mut Reader::new(&body)).unwrap()
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
        assert_eq!(
            old.len() + 1,
            encode(&{
                let mut kept = e.clone();
                kept.block_particles.truncate(1);
                kept
            })
            .len()
        );
        let back = explode_up(&old);
        let mut expected = e;
        expected.block_particles.truncate(1);
        assert_eq!(back, expected);
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
        let pos = play::MoveEntityPos(m(Some(play::VecDelta([4096, 0, -2048])), None));
        let old = rewritten(down(ids, cb::MOVE_ENTITY_POS, &encode(&pos)).unwrap());
        assert_eq!(old.len(), 1 + 6 + 1);
        let back = translate::move_entity(&old, true, false).unwrap();
        assert_eq!(
            play::MoveEntityPos::read(&mut Reader::new(&back)).unwrap(),
            pos
        );

        let pos_rot =
            play::MoveEntityPosRot(m(Some(play::VecDelta([1, 2, 3])), Some((10, -20))));
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
    fn a_position_sync_goes_down_and_back_up() {
        let ids = ids();
        let p = play::EntityPositionSync {
            id: 42,
            values: play::PositionMoveRotation {
                position: [100.5, 64.0, -20.25],
                delta_movement: [0.25, -0.5, 0.0],
                yaw: 90.0,
                pitch: -10.0,
            },
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
