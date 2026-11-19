//! 1.21.10's packets in 1.21.8's layout for a 1.21.8 client (and 1.21.7, protocol 772): the mirror of `translate`'s
//! `mod v1_21_8` (1.21.8 to 1.21.10). Five packets change layout: add_entity and set_entity_motion (the LpVec3
//! velocity becomes three shorts), set_default_spawn_position (the dimension name and the pitch go),
//! player_rotation (the relative flags go) and explode (the radius, the block count and the block particles go).
//! Entity data needs no step here: the pinned layout is already 1.21.8's (a newer client's goes up in
//! [`super::ClientSession::pin_entity_data_up`], through `track_entities`).
//!
//! The debug packets 1.21.10 added and the serverbound debug subscription have no 1.21.8 counterpart the id tables
//! pair them with, so they are dropped there, and nothing here touches serverbound packets.
use crate::{
    Result, State,
    adapter::translate::v1_21_8::VELOCITY_UNIT,
    buf::{Codec, LpVec3, Opt, Reader, Vec3d, Writer},
    ids as packet_ids,
    packets::play::SoundHolder,
};

/// The pinned clientbound packet `id` (`payload` in 1.21.10's layout) in 1.21.8's layout; `None` when laid out
/// alike.
pub(super) fn clientbound(state: State, id: i32, payload: &[u8]) -> Result<Option<Vec<u8>>> {
    use packet_ids::play::clientbound as cb;
    if state != State::Play {
        return Ok(None);
    }
    Ok(match id {
        cb::ADD_ENTITY => Some(add_entity(payload)?),
        cb::SET_ENTITY_MOTION => Some(set_entity_motion(payload)?),
        cb::SET_DEFAULT_SPAWN_POSITION => Some(spawn_position(payload)?),
        cb::PLAYER_ROTATION => Some(player_rotation(payload)?),
        cb::EXPLODE => Some(explode(payload)?),
        _ => None,
    })
}

/// A velocity component as 1.21.8's short: 1/8000 of a block per tick, clamped to the short's range.
fn to_short(v: f64) -> i16 {
    (v * VELOCITY_UNIT)
        .round()
        .clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i16
}

/// add_entity: the LpVec3 after the position becomes three shorts after the data.
fn add_entity(payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    let mut w = Writer::new();
    w.varint(r.varint()?);
    w.uuid(r.uuid()?);
    w.varint(r.varint()?);
    for _ in 0..3 {
        w.f64(r.f64()?);
    }
    let velocity = LpVec3::read(&mut r)?;
    let angles = [r.u8()?, r.u8()?, r.u8()?];
    let data = r.varint()?;
    w.bytes(&angles);
    w.varint(data);
    for v in velocity {
        w.i16(to_short(v));
    }
    Ok(w.data)
}

/// set_entity_motion: the LpVec3 becomes three shorts.
fn set_entity_motion(payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    let mut w = Writer::new();
    w.varint(r.varint()?);
    for v in LpVec3::read(&mut r)? {
        w.i16(to_short(v));
    }
    Ok(w.data)
}

/// set_default_spawn_position: the dimension name goes, and the pitch; the yaw is the one angle.
fn spawn_position(payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    let _dimension = r.string(32767)?;
    let position = r.i64()?;
    let yaw = r.f32()?;
    let _pitch = r.f32()?;
    let mut w = Writer::new();
    w.i64(position);
    w.f32(yaw);
    Ok(w.data)
}

/// player_rotation: the relative flags go. 1.21.8 has no relative rotation, so a relative one is sent as the
/// absolute angle it names, which is wrong.
// TODO(step1.21.8): a relative rotation (flag set) is read as absolute; 1.21.8 has no way to send one.
fn player_rotation(payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    let yaw = r.f32()?;
    r.bool()?;
    let pitch = r.f32()?;
    r.bool()?;
    let mut w = Writer::new();
    w.f32(yaw);
    w.f32(pitch);
    Ok(w.data)
}

/// explode: the centre, the knockback, the particle and the sound stay; the radius, the block count and the block
/// particles go. The 1.21.10 layout is read by hand: the pinned codec is 1.21.8's now.
fn explode(payload: &[u8]) -> Result<Vec<u8>> {
    use crate::entity_data::Particle;
    let (center, knockback, particle, sound) = super::read_in_client_ids(|| {
        let mut r = Reader::new(payload);
        let center = Vec3d::read(&mut r)?;
        let _radius = r.f32()?;
        let _block_count = r.i32()?;
        let knockback = <Opt<Vec3d>>::read(&mut r)?;
        let particle = Particle::read(&mut r)?;
        let sound = SoundHolder::read(&mut r)?;
        let n = r.varint()?;
        for _ in 0..n {
            Particle::read(&mut r)?;
            r.f32()?;
            r.f32()?;
            r.varint()?;
        }
        Ok((center, knockback, particle, sound))
    })?;
    Ok(super::write_in_client_ids(|w| {
        Vec3d::write(&center, w);
        <Opt<Vec3d>>::write(&knockback, w);
        particle.write(w);
        SoundHolder::write(&sound, w);
    }))
}

#[cfg(test)]
mod tests {
    //! Each change written here is read back with `translate`'s 1.21.8 reader: the full chain, 1.21.8's layout
    //! to 1.21.10's and back again, byte for byte where the layouts carry the same information.
    use super::*;
    use crate::adapter::{ServerVersion, session::Session, translate};

    /// 1.21.8's packet, read by translate's 1.21.8 reader, then written back by this step.
    fn round_trip(session: &Session, id: i32, body: &[u8]) -> Vec<u8> {
        let up = translate::client_layout_v1_21_8::clientbound(session, State::Play, id, body)
            .unwrap()
            .expect("1.21.8 packet has a 1.21.10 layout");
        clientbound(State::Play, id, &up)
            .unwrap()
            .expect("1.21.10 packet has a 1.21.8 layout")
    }

    fn short_velocity_body(entity: i32) -> Vec<u8> {
        let mut w = Writer::new();
        w.varint(entity);
        w.uuid(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);
        w.varint(5);
        w.f64(1.5);
        w.f64(-64.25);
        w.f64(3.0);
        w.u8(10);
        w.u8(200);
        w.u8(7);
        w.varint(9);
        w.i16(1343);
        w.i16(-2000);
        w.i16(0);
        w.data
    }

    #[test]
    fn add_entity_round_trips_through_an_lpvec3() {
        let session = Session::new(ServerVersion::V1_21_8);
        let body = short_velocity_body(0);
        assert_eq!(
            round_trip(&session, packet_ids::play::clientbound::ADD_ENTITY, &body),
            body
        );
    }

    #[test]
    fn set_entity_motion_round_trips_through_an_lpvec3() {
        let session = Session::new(ServerVersion::V1_21_8);
        let mut w = Writer::new();
        w.varint(42);
        w.i16(-1343);
        w.i16(8000);
        w.i16(12);
        let body = w.data;
        let id = packet_ids::play::clientbound::SET_ENTITY_MOTION;
        assert_eq!(round_trip(&session, id, &body), body);
    }

    #[test]
    fn spawn_position_and_rotation_round_trip() {
        let session = Session::new(ServerVersion::V1_21_8);
        let mut w = Writer::new();
        w.i64(0x1234_5678);
        w.f32(90.0);
        let spawn = w.data;
        let id = packet_ids::play::clientbound::SET_DEFAULT_SPAWN_POSITION;
        assert_eq!(round_trip(&session, id, &spawn), spawn);

        let mut w = Writer::new();
        w.f32(-45.5);
        w.f32(12.25);
        let rotation = w.data;
        let id = packet_ids::play::clientbound::PLAYER_ROTATION;
        assert_eq!(round_trip(&session, id, &rotation), rotation);
    }

    #[test]
    fn explode_drops_what_1_21_8_never_sent() {
        let session = Session::new(ServerVersion::V1_21_8);
        let mut w = Writer::new();
        w.f64(10.0);
        w.f64(64.0);
        w.f64(-3.0);
        w.bool(true); // a knockback
        w.f64(0.5);
        w.f64(0.0);
        w.f64(-0.25);
        w.varint(0); // the particle (no data)
        w.varint(5); // the sound, by id + 1
        let body = w.data;
        let id = packet_ids::play::clientbound::EXPLODE;
        assert_eq!(round_trip(&session, id, &body), body);

        // And the 1.21.10 side: the radius and count it was given, no block particles.
        // (Read by hand: the pinned codec is 1.21.8's now.)
        let up = translate::client_layout_v1_21_8::clientbound(&session, State::Play, id, &body)
            .unwrap()
            .unwrap();
        let mut r = Reader::new(&up);
        Vec3d::read(&mut r).unwrap();
        assert_eq!(r.f32().unwrap(), 3.0);
        assert_eq!(r.i32().unwrap(), 0);
        assert_eq!(
            <Opt<Vec3d>>::read(&mut r).unwrap(),
            Some([0.5, 0.0, -0.25])
        );
        crate::entity_data::Particle::read(&mut r).unwrap();
        SoundHolder::read(&mut r).unwrap();
        assert_eq!(r.varint().unwrap(), 0);
    }
}
