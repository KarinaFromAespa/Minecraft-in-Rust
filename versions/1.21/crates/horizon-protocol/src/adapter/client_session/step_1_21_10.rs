//! 1.21.11's packets in 1.21.10's layout for a 1.21.10 client (and 1.21.9): the mirror of `translate`'s
//! `mod v1_21_10` (1.21.10 to 1.21.11). No packet changes layout: 1.21.10's packets are 1.21.11's but for
//! `horse_screen_open`, which the id tables pair with `horse_screen_open`. Entity data changed (a bee's and a
//! wolf's anger end went from an INT to a LONG, a player's main arm from a BYTE to a HUMANOID_ARM), but the
//! pinned 1.21.8 layout is already 1.21.10's for those types, so the conversion lives in
//! [`super::ClientSession::pin_entity_data_up`] (for 1.21.11+ clients) instead of a step here.
use crate::{Result, State};

/// Nothing: no packet of 1.21.11 is laid out differently for 1.21.10 (see the module doc).
pub(super) fn clientbound(_state: State, _id: i32, _payload: &[u8]) -> Result<Option<Vec<u8>>> {
    Ok(None)
}
