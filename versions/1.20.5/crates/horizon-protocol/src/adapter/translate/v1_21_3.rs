//! 1.21.2/1.21.3 to 1.21.4, from vanilla packet stream codecs.
use super::*;

pub(super) fn clientbound(
    _session: &Session,
    state: State,
    id: i32,
    payload: &[u8],
) -> Result<Option<Vec<u8>>> {
    if state != State::Play {
        return Ok(None);
    }
    use packet_ids::play::clientbound as cb;
    let mut r = Reader::new(payload);
    let mut w = Writer::new();
    match id {
        cb::LEVEL_PARTICLES => {
            w.bool(r.bool()?);
            // The old packet had no alwaysShow flag: the normal particle visibility rule applies.
            w.bool(false);
            w.bytes(r.rest());
        }
        cb::SET_CARRIED_ITEM => w.varint(i32::from(r.i8()?)),
        cb::PLAYER_INFO_UPDATE => {
            let p = play::PlayerInfoUpdate::read(&mut r)?;
            if p.actions & 1 == 0 {
                // No ADD_PLAYER: the 1.21.4 form is identical (no hat bit).
                return Ok(None);
            }
            // Before UPDATE_HAT, tab-list player skins always included their
            // hat layer: 1.21.4's form sets the bit and a true bool per entry.
            // The pin codec has no hat field, so the 1.21.4 form is written
            // by hand.
            write_with_hat(&p, &mut w);
        }
        _ => return Ok(None),
    }
    Ok(Some(w.data))
}

/// A player-info update in 1.21.4's layout: the pin form's fields, the
/// UPDATE_HAT bit set, and a true hat bool after each entry's fields.
fn write_with_hat(p: &play::PlayerInfoUpdate, w: &mut Writer) {
    w.u8(p.actions | 128);
    w.varint(p.entries.len() as i32);
    for e in &p.entries {
        w.uuid(e.uuid);
        if p.actions & 1 != 0 {
            w.string(e.name.as_deref().unwrap_or(""));
            w.varint(e.properties.len() as i32);
            e.properties
                .iter()
                .for_each(|prop| crate::packets::login::Property::write(prop, w));
        }
        if p.actions & 2 != 0 {
            w.bool(e.chat_session.is_some());
            if let Some((id, expires, key, sig)) = &e.chat_session {
                w.uuid(*id);
                w.i64(*expires);
                w.byte_array(key);
                w.byte_array(sig);
            }
        }
        if p.actions & 4 != 0 {
            w.varint(e.game_mode.unwrap_or(0));
        }
        if p.actions & 8 != 0 {
            w.bool(e.listed.unwrap_or(false));
        }
        if p.actions & 16 != 0 {
            w.varint(e.latency.unwrap_or(0));
        }
        if p.actions & 32 != 0 {
            let name = e.display_name.clone().flatten();
            w.bool(name.is_some());
            if let Some(t) = &name {
                w.nbt(t);
            }
        }
        if p.actions & 64 != 0 {
            w.varint(e.list_order.unwrap_or(0));
        }
        w.bool(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_particle_flag_survives_the_whole_pipeline_with_registry_ids() {
        let session = Session::new(ServerVersion::V1_21_3);
        let mut old = Writer::new();
        old.bool(true);
        for v in [1.0, 2.0, -3.0] {
            old.f64(v);
        }
        for v in [0.25, 0.5, 0.75, 1.5] {
            old.f32(v);
        }
        old.i32(12);
        // Vanilla 1.21.3 minecraft:block, followed by minecraft:chain[axis=y,waterlogged=false].
        old.varint(1);
        old.varint(7006);
        let (id, body) = super::super::clientbound(
            &session,
            State::Play,
            packet_ids::play::clientbound::LEVEL_PARTICLES,
            &old.data,
        )
        .unwrap()
        .unwrap();
        let play::Clientbound::LevelParticles(p) =
            play::Clientbound::decode(id, &body.unwrap()).unwrap()
        else {
            panic!("particle")
        };
        assert!(p.override_limiter);
        assert_eq!(p.pos, [1.0, 2.0, -3.0]);
        assert_eq!(p.dist, [0.25, 0.5, 0.75]);
        assert_eq!(p.max_speed, [1.5; 3]);
        assert_eq!(p.count, 12);
        assert_eq!(p.particle.id, 1);
        assert_eq!(
            p.particle.data,
            // chain[axis=y,waterlogged=false] is 6776 at this pin, not 7006.
            Some(crate::entity_data::ParticleData::State(6776))
        );
    }

    #[test]
    fn a_particle_the_pin_lacks_drops_the_packet() {
        let session = Session::new(ServerVersion::V1_21_3);
        let mut old = Writer::new();
        old.bool(true);
        for v in [1.0, 2.0, -3.0] {
            old.f64(v);
        }
        for v in [0.25, 0.5, 0.75, 1.5] {
            old.f32(v);
        }
        old.i32(12);
        // Vanilla 1.21.3 minecraft:trail (46), which 1.21.1 has no
        // counterpart for: three doubles, then the colour. 1.21.3's trail
        // carries no duration.
        old.varint(46);
        for at in [1.0, 2.0, -3.0] {
            old.f64(at);
        }
        old.i32(0x7f3f1f);
        assert_eq!(
            super::super::clientbound(
                &session,
                State::Play,
                packet_ids::play::clientbound::LEVEL_PARTICLES,
                &old.data,
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn creaking_content_drops_on_the_way_up_to_the_767_pin() {
        let ids = ServerVersion::V1_21_3.ids().unwrap();
        // 1.21.1 has no creaking at all, so the nine heart states
        // (2917..2925 in 1.21.3, axis-major, creaking
        // disabled/dormant/active) and both creaking entities (29, 30)
        // have no pin counterpart and drop on the way up.
        for state in 2917..2926 {
            assert_eq!(ids.block_state.up(state), None);
        }
        assert_eq!(ids.entity_type.up(29), None);
        assert_eq!(ids.entity_type.up(30), None);
        // Content both versions have still maps: chain[axis=y,
        // waterlogged=false] is 7006 in 1.21.3 and 6776 at the pin.
        assert_eq!(ids.block_state.up(7006), Some(6776));
        assert_eq!(ids.block_state.down(6776), Some(7006));
    }

    #[test]
    fn player_list_updates_keep_their_scope_and_lose_the_hat() {
        let session = Session::new(ServerVersion::V1_21_3);
        let mut old = Writer::new();
        old.u8(0x7f);
        old.varint(2);
        for (uuid, name, latency, order) in [(1, "First", 150, 4), (2, "Second", 20, -1)] {
            old.uuid(uuid);
            old.string(name);
            old.varint(0); // properties
            old.bool(false); // chat session
            old.varint(1); // game mode
            old.bool(true);
            old.varint(latency);
            old.bool(false); // display name
            old.varint(order);
        }
        let id = packet_ids::play::clientbound::PLAYER_INFO_UPDATE;
        let translated = super::super::clientbound(&session, State::Play, id, &old.data)
            .unwrap()
            .unwrap()
            .1
            .unwrap();
        let p = play::PlayerInfoUpdate::read(&mut Reader::new(&translated)).unwrap();
        // The 1.21.3 step adds UPDATE_HAT on the way up to 26.2's layout and
        // pin_layout strips it again: the pin sees the seven actions as sent.
        assert_eq!(p.actions, 0x7f);
        assert_eq!(p.entries[0].name.as_deref(), Some("First"));
        assert_eq!(p.entries[0].latency, Some(150));
        assert_eq!(p.entries[1].list_order, Some(-1));
        let mut partial = Writer::new();
        partial.u8(16);
        partial.varint(1);
        partial.uuid(1);
        partial.varint(201);
        let (_, body) = super::super::clientbound(&session, State::Play, id, &partial.data)
            .unwrap()
            .unwrap();
        // A latency-only update has no ADD_PLAYER, so no hat is added or
        // stripped: the packet passes through unchanged.
        assert_eq!(body, None);
    }

    #[test]
    fn riding_and_swinging_use_the_legacy_server_layout_and_ids() {
        let session = Session::new(ServerVersion::V1_21_3);
        let request = play::Serverbound::MoveVehicle(play::MoveVehicleRequest {
            position: [1.5, -3.0, 4.5],
            yaw: 90.0,
            pitch: -10.0,
            on_ground: true,
        });
        let (id, payload) = super::super::serverbound(
            &session,
            State::Play,
            packet_ids::play::serverbound::MOVE_VEHICLE,
            &strip_id(request.encode()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(id, 32);
        assert_eq!(payload.len(), 32);
        let mut r = Reader::new(&payload);
        assert_eq!(
            [r.f64().unwrap(), r.f64().unwrap(), r.f64().unwrap()],
            [1.5, -3.0, 4.5]
        );
        assert_eq!([r.f32().unwrap(), r.f32().unwrap()], [90.0, -10.0]);
        assert_eq!(r.remaining(), 0);
        let swing = super::super::serverbound(
            &session,
            State::Play,
            packet_ids::play::serverbound::SWING,
            &[0],
        )
        .unwrap()
        .unwrap();
        assert_eq!(swing, (56, vec![0]));
    }
}
