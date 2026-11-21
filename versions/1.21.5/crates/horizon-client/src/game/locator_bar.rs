//! The waypoints a server tracks for the player (`handleWaypoint`; on a
//! served world the host's own connection in lan/waypoints.rs). 1.21.5 has
//! no locator bar, so nothing here draws (item 108 removes the bar's
//! rendering and state outright).
use super::Game;
use horizon_client::locator_bar::ClientWaypoints;

fn uuid(ints: [i32; 4]) -> u128 {
    ints.iter()
        .fold(0u128, |acc, &i| acc << 32 | u128::from(i as u32))
}

impl Game<'_> {
    /// A new connection or world starts with no waypoints.
    pub(super) fn clear_waypoints(&mut self) {
        self.waypoints = ClientWaypoints::default();
    }
    /// The local player's UUID, as servers know it.
    pub(super) fn own_uuid(&self) -> u128 {
        match &self.remote {
            Some(r) => r.profile(),
            None => self.level.player_uuid.map_or(0, uuid),
        }
    }
}
