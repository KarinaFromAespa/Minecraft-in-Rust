//! 26.2 to 26.1: the mirror of `translate`'s `v26_1` (26.1 to 26.2), for a [`super::ClientSession`] whose client
//! is 26.1 (roadmap `older-clients`). What the pinned release sends is in 26.2's layout by then ([`super::world`]
//! and [`super::super::layout_26_2`]); this writes 26.1's.
//!
//! Only the packets `v26_1` changes are here; translate's `v26_1` renumbers no id itself, so none is added or
//! dropped on this step.
//!
//! Serverbound there is nothing to do: 26.1's `spectate_entity` has no counterpart in the pinned 1.21.11
//! (which spectates by `teleport_to_entity`), so the session drops it before this step is reached.
use crate::{
    Error, Result, State,
    adapter::translate,
    buf::{Codec, Opt, Reader, VarInt, Writer},
    ids as packet_ids,
};

/// `ChatFormatting.RESET`, which 26.1 sends for a team without a colour.
const RESET: i32 = 21;

/// The pinned clientbound packet `id` in 26.2's layout, in 26.1's; `None` when 26.1 lays it out alike.
pub(super) fn clientbound(state: State, id: i32, payload: &[u8]) -> Result<Option<Vec<u8>>> {
    use packet_ids::play::clientbound as cb;
    Ok(match (state, id) {
        (State::Login, packet_ids::login::clientbound::LOGIN_FINISHED) => {
            Some(login_finished(payload)?)
        }
        (State::Play, cb::LOGIN) => Some(login(payload)?),
        (State::Play, cb::SET_PLAYER_TEAM) => Some(team(payload)?),
        // ADD_ENTITY and REMOVE_ENTITIES are laid out alike. 26.1's spectate_entity needs the UUID of each
        // entity added, which the session keeps (translate's v26_1 does the same the other way).
        // ADD_ENTITY's (id, UUID) are kept by `ClientSession::track_entities`.
        _ => None,
    })
}

/// 26.2 appends the session's id, 16 bytes, to the profile; 26.1 has none, so they go.
fn login_finished(payload: &[u8]) -> Result<Vec<u8>> {
    let Some(end) = payload.len().checked_sub(16) else {
        return Err(Error::new("login finished shorter than its session id"));
    };
    Ok(payload[..end].to_vec())
}

/// 26.2 tells the client whether the server checks accounts (online_mode), a bool after the spawn info; 26.1
/// has none, so it goes.
fn login(payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    r.i32()?;
    r.bool()?;
    let n = r.count(r.remaining())?;
    for _ in 0..n {
        r.string(32767)?;
    }
    for _ in 0..3 {
        r.varint()?;
    }
    for _ in 0..3 {
        r.bool()?;
    }
    translate::spawn_info(&mut r, crate::adapter::ServerVersion::V26_2)?;
    let at = r.position();
    r.bool()?;
    let mut body = payload[..at].to_vec();
    body.extend_from_slice(&payload[at + 1..]);
    Ok(body)
}

/// 26.2's team parameters (the pinned layout): display name, prefix, suffix, visibility, collision, an
/// optional TeamColor, options. 26.1 sends the display name, options, visibility, collision, a ChatFormatting
/// (RESET for none), prefix, suffix. The players that follow are the same in both.
fn team(payload: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(payload);
    let mut w = Writer::new();
    w.string(&r.string(32767)?);
    let method = r.i8()?;
    w.i8(method);
    if method == 0 || method == 2 {
        let display = r.nbt()?;
        let prefix = r.nbt()?;
        let suffix = r.nbt()?;
        let visibility = r.varint()?;
        let collision = r.varint()?;
        let color = <Opt<VarInt>>::read(&mut r)?;
        let options = r.i8()?;
        w.nbt(&display);
        w.i8(options);
        w.varint(visibility);
        w.varint(collision);
        w.varint(color.unwrap_or(RESET));
        w.nbt(&prefix);
        w.nbt(&suffix);
    }
    w.bytes(r.rest());
    Ok(w.data)
}

#[cfg(test)]
mod tests {
    //! Each packet written here in 26.1's layout is read back with `translate`'s 26.1 reader, which brings it to
    //! 26.2's again. Packets 26.2 has more of than 26.1 (the online_mode bool, the session id) come back as
    //! 26.1's reader reads them: offline, and an all-zero id.
    use super::*;
    use crate::{
        adapter::{ServerVersion, Session},
        packets::play,
    };
    use horizon_world::nbt::Tag;

    use packet_ids::play::clientbound as cb;

    /// A login as 26.2 sends it, with the given online_mode and an enforced secure chat.
    fn login_26_2(online_mode: bool) -> Vec<u8> {
        let mut w = Writer::new();
        w.i32(7);
        w.bool(false);
        w.varint(1);
        w.string("minecraft:overworld");
        w.varint(20);
        w.varint(10);
        w.varint(8);
        w.bool(false);
        w.bool(true);
        w.bool(false);
        // The spawn info, with the game types as bytes (-1 for no previous one).
        w.varint(0);
        w.string("minecraft:overworld");
        w.i64(1234);
        w.i8(0);
        w.i8(-1);
        w.bool(false);
        w.bool(false);
        <Opt<play::GlobalPos>>::write(&None, &mut w);
        w.varint(0);
        w.varint(63);
        w.bool(online_mode);
        w.bool(true);
        w.data
    }

    /// A team update as 26.2 sends it: `color` only for methods 0 and 2 (the parameters), 3 adds a player.
    fn team_26_2(method: i8, color: Option<i32>) -> Vec<u8> {
        let mut w = Writer::new();
        w.string("red");
        w.i8(method);
        if method == 0 || method == 2 {
            w.nbt(&Tag::String("Red".into()));
            w.nbt(&Tag::String("[".into()));
            w.nbt(&Tag::String("]".into()));
            w.varint(1);
            w.varint(2);
            <Opt<VarInt>>::write(&color, &mut w);
            w.i8(3);
        }
        if method == 0 || method == 3 {
            w.varint(1);
            w.string("Steve");
        }
        w.data
    }

    #[test]
    fn an_offline_login_goes_down_and_back_up() {
        let session = Session::new(ServerVersion::V26_1);
        let original = login_26_2(false);
        let down = clientbound(State::Play, cb::LOGIN, &original)
            .unwrap()
            .unwrap();
        // 26.1 has no online_mode byte: one less.
        assert_eq!(down.len() + 1, original.len());
        let back = translate::v26_1::clientbound(&session, State::Play, cb::LOGIN, &down)
            .unwrap()
            .unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn an_online_login_reads_back_as_offline() {
        let session = Session::new(ServerVersion::V26_1);
        let down = clientbound(State::Play, cb::LOGIN, &login_26_2(true))
            .unwrap()
            .unwrap();
        let back = translate::v26_1::clientbound(&session, State::Play, cb::LOGIN, &down)
            .unwrap()
            .unwrap();
        assert_eq!(back, login_26_2(false));
    }

    #[test]
    fn a_team_goes_down_and_back_up() {
        let session = Session::new(ServerVersion::V26_1);
        let id = cb::SET_PLAYER_TEAM;
        for (method, color) in [(0, Some(12)), (2, None), (3, None)] {
            let original = team_26_2(method, color);
            let down = clientbound(State::Play, id, &original).unwrap().unwrap();
            let back = translate::v26_1::clientbound(&session, State::Play, id, &down)
                .unwrap()
                .unwrap();
            assert_eq!(back, original, "method {method}");
        }
    }

    #[test]
    fn a_team_without_a_colour_is_reset_for_26_1() {
        let down = clientbound(State::Play, cb::SET_PLAYER_TEAM, &team_26_2(2, None))
            .unwrap()
            .unwrap();
        let mut r = Reader::new(&down);
        assert_eq!(r.string(32767).unwrap(), "red");
        assert_eq!(r.i8().unwrap(), 2);
        r.nbt().unwrap();
        r.i8().unwrap();
        r.varint().unwrap();
        r.varint().unwrap();
        assert_eq!(r.varint().unwrap(), RESET);
    }

    #[test]
    fn the_session_id_goes_and_the_profile_stays() {
        let session = Session::new(ServerVersion::V26_1);
        let id = packet_ids::login::clientbound::LOGIN_FINISHED;
        let mut w = Writer::new();
        w.uuid(0x1234);
        w.string("Steve");
        w.varint(0);
        let profile = w.data;
        let mut original = profile.clone();
        original.extend_from_slice(&[7; 16]);
        let down = clientbound(State::Login, id, &original).unwrap().unwrap();
        assert_eq!(down, profile);
        // translate gives 26.2 an all-zero session id back.
        let back = translate::v26_1::clientbound(&session, State::Login, id, &down)
            .unwrap()
            .unwrap();
        let mut zeroed = profile;
        zeroed.extend_from_slice(&[0; 16]);
        assert_eq!(back, zeroed);
        assert!(clientbound(State::Login, id, &[0; 15]).is_err());
    }
}
