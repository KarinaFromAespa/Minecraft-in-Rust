//! Leads on boats and rafts (`AbstractBoat implements Leashable`): putting a
//! lead on one, taking it off, `Leashable.tickLeash` (the legacy pull of the
//! rope, the lead snapping past ten blocks with no sound), saving who holds
//! it, and dropping the lead when the boat breaks. The rope is drawn by
//! `leads.rs` (`EntityRenderer.renderLeash`, one rope). A boat is also a
//! holder (`Holder::Boat`).
//!
//! The elastic pull of a ridden boat: `Leashable.tickLeash` runs on the
//! server only, and the server's boat that its rider's client drives takes
//! that client's position back (`handleMoveVehicle`), so vanilla never moves
//! it by the rope either. Here the pull lands on the server's copy and the
//! rider's client overwrites it the same way (`tick_boats`).
use super::{Game, boats::Boat};
use horizon_client::mobs::leash::{self, Holder, HolderState, Pending};
use horizon_core::geometry::Vec3;
use horizon_world::nbt::Tag;

/// `AbstractBoat`'s `getBbWidth` and `getBbHeight`.
const WIDTH: f32 = 1.375;
const HEIGHT: f32 = 0.5625;

/// Where the other end of a boat's lead is.
enum Held {
    /// The holder is gone: the lead comes off (`canInteractWithLevel`).
    Gone,
    /// In another level or not loaded: nothing happens (`level() ==`).
    Elsewhere,
    /// Its state.
    At(HolderState),
}

fn center(b: &Boat) -> Vec3 {
    Vec3::new(
        b.position.x,
        b.position.y + f64::from(HEIGHT) / 2.0,
        b.position.z,
    )
}
/// A boat as a lead's holder (`HolderState`), with its id.
pub(super) fn boat_state(b: &Boat) -> (u32, HolderState) {
    (
        b.id,
        HolderState {
            position: b.position,
            width: f64::from(WIDTH),
            height: f64::from(HEIGHT),
            yaw: b.yaw,
            velocity: b.velocity,
        },
    )
}
impl Game<'_> {
    /// The player whose lead this is: the lead's `leash_player`, else the
    /// local player.
    fn boat_leash_player(&self, b: &Boat) -> Option<[i32; 4]> {
        b.leash_player.or(self.level.player_uuid)
    }
    fn boat_holder(&self, holder: Holder, player: Option<[i32; 4]>) -> Held {
        match holder {
            Holder::Player => {
                let want = player.or(self.level.player_uuid);
                // A server's guest (single player's window included) first.
                let guest = self.lan.as_ref().and_then(|host| {
                    let g = host.hub.guests.iter().find(|g| {
                        Some(horizon_client::lan::playerdata::uuid_ints(g.uuid)) == want
                    })?;
                    Some((host, g))
                });
                if let Some((host, g)) = guest {
                    if !host.hub.here(g) || !g.loaded {
                        return Held::Elsewhere;
                    }
                    if g.dead || g.spectator() {
                        return Held::Gone;
                    }
                    let bounds = g.bounds();
                    let (lo, hi) = (bounds.min_position(), bounds.max_position());
                    let moved = g.known_movement;
                    return Held::At(HolderState {
                        position: Vec3::new(g.position[0], g.position[1], g.position[2]),
                        width: hi.x - lo.x,
                        height: hi.y - lo.y,
                        yaw: g.yaw,
                        velocity: Vec3::new(moved[0], moved[1], moved[2]),
                    });
                }
                if want != self.level.player_uuid {
                    // A player who left: Entity.remove drops what they led.
                    return Held::Gone;
                }
                let Some(v) = self.player_view() else {
                    return Held::Elsewhere;
                };
                if !v.alive || v.spectator {
                    return Held::Gone;
                }
                let lo = v.bounds.min_position();
                let size = v.bounds.max_position().subtract(lo);
                Held::At(HolderState {
                    position: v.position,
                    width: size.x,
                    height: size.y,
                    yaw: v.yaw,
                    velocity: v.velocity,
                })
            }
            Holder::Mob(id) => {
                let Some(m) = self.mobs.iter().find(|m| m.id == id && m.alive()) else {
                    return Held::Gone;
                };
                let (lo, hi) = (m.bounds().min_position(), m.bounds().max_position());
                Held::At(HolderState {
                    position: m.position,
                    width: hi.x - lo.x,
                    height: hi.y - lo.y,
                    yaw: m.yaw,
                    velocity: m.velocity,
                })
            }
            Holder::Boat(id) => match self.boats.iter().find(|o| o.id == id) {
                Some(o) => Held::At(boat_state(o).1),
                None => Held::Gone,
            },
            Holder::Knot(pos) => {
                let name = self
                    .res
                    .facts()
                    .get(self.world.block(pos[0], pos[1], pos[2]));
                let fence = self
                    .res
                    .items()
                    .block_tags
                    .get("minecraft:fences")
                    .is_some_and(|s| s.contains(&*name.name));
                if fence {
                    Held::At(leash::knot_state(pos))
                } else {
                    Held::Gone
                }
            }
        }
    }

    /// `Leashable.dropLeash` / `removeLeash` on a boat: the lead falls as an
    /// item (`drop`). The change reaches clients through the entity tracker.
    fn release_boat_leash(&mut self, b: &mut Boat, drop: bool) {
        if b.leash.take().is_none() {
            return;
        }
        b.leash_player = None;
        if drop {
            self.drop_lead_item([b.position.x, b.position.y, b.position.z]);
        }
    }

    /// `AbstractBoat.remove`: a boat that is destroyed drops its lead.
    pub(super) fn boat_removed(&mut self, b: &Boat) {
        let mut b = b.clone();
        let pending = b.leash_pending.take().is_some();
        if b.leash.is_some() || pending {
            let at = [b.position.x, b.position.y, b.position.z];
            b.leash = None;
            self.drop_lead_item(at);
        }
    }

    /// `Leashable.tickLeash` for a boat, at the end of `Entity.baseTick`.
    pub(super) fn tick_boat_leash(&mut self, b: &mut Boat) {
        b.age += 1;
        if let Some(pending) = b.leash_pending {
            self.restore_boat_leash(b, pending);
        }
        let Some(holder) = b.leash else {
            return;
        };
        let state = match self.boat_holder(holder, b.leash_player) {
            Held::Elsewhere => return,
            // !leashHolder.canInteractWithLevel(): dropLeash with entity_drops
            // on, else removeLeash.
            Held::Gone => {
                let drops = self.ui.rules.bool("entity_drops");
                self.release_boat_leash(b, drops);
                return;
            }
            Held::At(state) => state,
        };
        // Leashable.tickLeash: `distanceTo` is feet to feet, narrowed to a float.
        let away = b.position.distance(state.position) as f32;
        if f64::from(away) > leash::SNAP {
            // leashTooFarBehaviour is dropLeash: the lead drops, and 1.21.5
            // plays no sound (1.21.6 added LEAD_BREAK here).
            self.release_boat_leash(b, true);
            return;
        }
        if f64::from(away) > leash::ELASTIC {
            let pull = leash::legacy_pull(b.position, state.position, away);
            b.velocity = Vec3::new(
                b.velocity.x + pull[0],
                b.velocity.y + pull[1],
                b.velocity.z + pull[2],
            );
        }
    }

    /// `Leashable.restoreLeashFromSave`: a holder read from a save becomes
    /// real once its entity is found; one that never shows up in 100 ticks
    /// drops the lead.
    fn restore_boat_leash(&mut self, b: &mut Boat, pending: Pending) {
        let found = match pending {
            Pending::Knot(p) => Some((Holder::Knot(p), None)),
            Pending::Uuid(u) => {
                let guest = self.lan.as_ref().is_some_and(|h| {
                    h.hub
                        .guests
                        .iter()
                        .any(|g| horizon_client::lan::playerdata::uuid_ints(g.uuid) == u)
                });
                if guest || (self.level.player_uuid == Some(u) && self.player.is_some()) {
                    Some((Holder::Player, Some(u)))
                } else {
                    self.mob_by_uuid(u)
                        .map(|id| (Holder::Mob(id), None))
                        .or_else(|| self.boat_by_uuid(u).map(|id| (Holder::Boat(id), None)))
                        .or_else(|| self.knot_by_uuid(u).map(|p| (Holder::Knot(p), None)))
                }
            }
        };
        if let Some((holder, player)) = found {
            // The boat is out of the list while it ticks: its own knot is
            // made here.
            if let Holder::Knot(fence) = holder {
                self.tie_knot(fence);
            }
            b.leash = Some(holder);
            b.leash_player = player;
            b.leash_pending = None;
        } else if b.age > 100 {
            b.leash_pending = None;
            self.drop_lead_item([b.position.x, b.position.y, b.position.z]);
        }
    }

    /// The `leash` tag of a boat about to be saved.
    pub(super) fn write_boat_leash(&mut self, tag: &mut Tag, index: usize) {
        let Tag::Compound(map) = tag else { return };
        map.remove(&"leash".into());
        let b = &self.boats[index];
        let (holder, pending, player) = (b.leash, b.leash_pending, self.boat_leash_player(b));
        let value = match (holder, pending) {
            (Some(Holder::Mob(id)), _) => {
                let uuid = self.mob_uuid(id);
                leash::to_nbt(Holder::Mob(id), Some(uuid))
            }
            (Some(Holder::Boat(id)), _) => {
                let uuid = self.uuid_for(id);
                leash::to_nbt(Holder::Boat(id), Some(uuid))
            }
            (Some(h), _) => leash::to_nbt(h, player),
            (None, Some(Pending::Knot(p))) => leash::to_nbt(Holder::Knot(p), None),
            (None, Some(Pending::Uuid(u))) => leash::to_nbt(Holder::Player, Some(u)),
            _ => None,
        };
        if let Some(v) = value {
            map.insert("leash".into(), v);
        }
    }

    /// Whether the player acting holds this boat's lead.
    fn boat_led_by_player(&mut self, i: usize) -> bool {
        let me = self.acting_player_uuid();
        let b = &self.boats[i];
        b.leash == Some(Holder::Player) && self.boat_leash_player(b) == Some(me)
    }

    /// The boat with this UUID (`ServerLevel.getEntity`).
    pub(super) fn boat_by_uuid(&self, uuid: [i32; 4]) -> Option<u32> {
        self.persist
            .uuids
            .iter()
            .find(|(id, u)| **u == uuid && self.boats.iter().any(|b| b.id == **id))
            .map(|(id, _)| *id)
    }

    /// The fence post of the leash knot with this UUID (`ServerLevel.getEntity`):
    /// a summoned or loaded `minecraft:leash_knot`, which sits on its block.
    pub(super) fn knot_by_uuid(&self, uuid: [i32; 4]) -> Option<[i32; 3]> {
        self.others
            .iter()
            .find(|o| o.kind == "minecraft:leash_knot" && self.persist.uuid_of(o.id) == Some(uuid))
            .map(|o| {
                let block = horizon_core::position::BlockPos::containing(o.position);
                [block.x, block.y, block.z]
            })
    }

    /// The boats on the acting player's lead whose box meets the seven-block
    /// cube around the fence at `fence` (`LeadItem.leashableInArea`, called
    /// from `LeadItem.bindPlayerMobs`).
    pub(super) fn player_led_boats(&mut self, fence: [i32; 3]) -> Vec<usize> {
        let (fx, fy, fz) = (f64::from(fence[0]), f64::from(fence[1]), f64::from(fence[2]));
        (0..self.boats.len())
            .filter(|&i| {
                if !self.boat_led_by_player(i) {
                    return false;
                }
                let b = &self.boats[i];
                let hw = f64::from(WIDTH) / 2.0;
                b.position.x + hw >= fx - 7.0
                    && b.position.x - hw <= fx + 7.0
                    && b.position.y + f64::from(HEIGHT) >= fy - 7.0
                    && b.position.y <= fy + 7.0
                    && b.position.z + hw >= fz - 7.0
                    && b.position.z - hw <= fz + 7.0
            })
            .collect()
    }

    /// The boats whose lead hangs on `holder`.
    pub(super) fn boats_on(&self, holder: Holder) -> Vec<usize> {
        (0..self.boats.len())
            .filter(|&i| self.boats[i].leash == Some(holder))
            .collect()
    }

    /// The acting player's boats on a lead go to the knot at `fence`
    /// (`LeadItem.bindPlayerMobs`): every one in range, with no attach
    /// check. Whether any did.
    pub(super) fn tie_player_boats(&mut self, fence: [i32; 3]) -> bool {
        let mut any = false;
        for i in self.player_led_boats(fence) {
            self.boats[i].leash = Some(Holder::Knot(fence));
            self.boats[i].leash_player = None;
            any = true;
        }
        any
    }

    /// `Entity.interact`'s lead part for the boat at `i`; whether it took
    /// the click. 1.21.5 has no sneak-tie and no shears branch, and neither
    /// the untie nor the tie plays a sound.
    pub(super) fn interact_boat_leash(&mut self, i: usize) -> bool {
        let item = self.held().map(|s| s.item.to_string());
        let c = center(&self.boats[i]);
        let at = [c.x, c.y, c.z];
        if self.boat_led_by_player(i) {
            // hasInfiniteMaterials: removeLeash, else dropLeash.
            let drop = !self.creative;
            let mut b = self.boats[i].clone();
            self.release_boat_leash(&mut b, drop);
            self.boats[i] = b;
            self.lead_game_event("minecraft:entity_interact", at);
            return true;
        }
        // `canHaveALeashAttachedToIt`: leashable and not already leashed —
        // a boat tied to someone else takes no lead.
        if item.as_deref() == Some("minecraft:lead") && self.boats[i].leash.is_none() {
            let mut b = self.boats[i].clone();
            // LeashData.setLeashHolder forgets a holder still waiting in the save.
            b.leash = Some(Holder::Player);
            b.leash_pending = None;
            b.leash_player = Some(self.acting_player_uuid());
            self.boats[i] = b;
            // The lead is always used up.
            if let Some(stack) = self.held_mut().as_mut() {
                stack.count -= 1;
                if stack.count == 0 {
                    *self.held_mut() = None;
                }
            }
            return true;
        }
        false
    }

    /// The leads on boats that hang on a player or a mob, for the entity
    /// tracker: the holder's entity id (`ClientboundSetEntityLinkPacket`).
    pub(super) fn boat_leash_link(&self, b: &Boat, guests: &[(u128, i32)]) -> Option<i32> {
        match b.leash? {
            Holder::Player => {
                let uuid = self.boat_leash_player(b);
                guests
                    .iter()
                    .find(|(u, _)| Some(horizon_client::lan::playerdata::uuid_ints(*u)) == uuid)
                    .map(|&(_, id)| id)
            }
            Holder::Mob(id) | Holder::Boat(id) => Some(id as i32),
            Holder::Knot(p) => self.knot_ids.get(&p).map(|&id| id as i32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
    fn command_view_uses_live_and_pending_boat_leashes_not_raw_tags() {
        use crate::game::{Audio, Options, Progress, Settings, commands::Target, load_resources};
        let root = std::path::PathBuf::from(std::env::var_os("HORIZON_TEST_RESOURCES").unwrap());
        let settings = Settings::default();
        let loaded = load_resources(
            root.clone(),
            None,
            &root,
            &settings,
            std::time::Instant::now(),
            false,
            true,
            &Progress::new(),
            &mut |_, _| unreachable!(),
        )
        .unwrap();
        let mut options = Options::parse(std::iter::empty()).unwrap().unwrap();
        options.resources = root.clone();
        options.execution_role = crate::game::execution_role::ExecutionRole::IntegratedServer;
        let mut game = Game::new(
            &loaded.resources,
            loaded.pack,
            loaded.gui.unwrap(),
            options,
            settings,
            Audio::silent(&root),
            None,
        )
        .unwrap();
        game.build_registries();
        let (id, uuid, actual_holder, unknown) =
            (1, [326, 1, 0, 1], [326, 2, 0, 1], [111, 222, 333, 444]);
        game.boats
            .push(Boat::new(id, "oak".into(), Vec3::new(6.5, 100.0, 4.5), 0.0));
        game.persist.uuids.insert(id, uuid);
        game.level.player_uuid = Some([326, 3, 0, 1]);
        let read = |game: &mut Game<'_>| game.entity_nbt(Target::Boat(id)).unwrap();
        assert!(!game.persist.raw.contains_key(&id));
        assert_eq!(
            read(&mut game).get("UUID"),
            Some(&Tag::IntArray(uuid.to_vec()))
        );
        assert!(read(&mut game).get("leash").is_none());
        game.boats[0].leash = Some(Holder::Player);
        game.boats[0].leash_player = Some(actual_holder);
        let expected = Tag::compound([("UUID", Tag::IntArray(actual_holder.to_vec()))]);
        assert_eq!(read(&mut game).get("leash"), Some(&expected));
        assert_eq!(read(&mut game).get("leash"), Some(&expected));
        let delayed = Tag::compound([("UUID", Tag::IntArray(unknown.to_vec()))]);
        game.persist
            .raw
            .insert(id, Tag::compound([("leash", delayed.clone())]));
        assert_eq!(read(&mut game).get("leash"), Some(&expected));
        game.boats[0].leash = None;
        game.boats[0].leash_player = None;
        game.boats[0].leash_pending = Some(Pending::Uuid(unknown));
        assert_eq!(read(&mut game).get("leash"), Some(&delayed));
        game.boats[0].leash_pending = None;
        assert!(read(&mut game).get("leash").is_none());
    }

    #[test]
    fn a_boat_is_centred_half_its_height_up() {
        let b = Boat::new(1, "oak".into(), Vec3::new(1.0, 64.0, 2.0), 0.0);
        let c = center(&b);
        assert_eq!((c.x, c.z), (1.0, 2.0));
        assert!((c.y - 64.28125).abs() < 1e-9);
    }
    #[test]
    fn a_boat_holds_a_lead_from_its_position() {
        let mut b = Boat::new(7, "oak".into(), Vec3::new(1.0, 64.0, 2.0), 90.0);
        b.velocity = Vec3::new(0.1, 0.0, 0.0);
        let (id, s) = boat_state(&b);
        assert_eq!(id, 7);
        assert_eq!((s.position.x, s.position.y, s.position.z), (1.0, 64.0, 2.0));
        assert!((s.width - 1.375).abs() < 1e-9 && (s.height - 0.5625).abs() < 1e-9);
        assert_eq!((s.yaw, s.velocity.x), (90.0, 0.1));
    }
    #[test]
    fn a_boat_holder_is_an_entity_not_a_player() {
        assert_eq!(Holder::Boat(7).who(), None);
        assert_ne!(Holder::Boat(7), Holder::Mob(7));
    }
    #[test]
    #[ignore = "needs prepared resources and isolated homes/cwd"]
    fn non_guest_holder_branches_keep_their_existing_velocity_sources() {
        let mut game = super::super::native_machines::tests::server_game();
        game.build_registries();
        game.set_allow_commands(true);
        assert!(game.run_command("summon oak_boat 6.5 100 4.5 {UUID:[I;327,1,0,1]}"));
        let boat = &mut game.boats[0];
        boat.velocity = Vec3::new(0.25, -0.125, 0.5);
        let (id, expected) = (boat.id, boat.velocity);
        let Held::At(state) = game.boat_holder(Holder::Boat(id), None) else {
            panic!("actual boat holder must resolve");
        };
        assert_eq!(state.velocity, expected);
        assert!(game.run_command("summon pig 4.5 100 4.5 {UUID:[I;327,2,0,1],NoAI:1b}"));
        let mob = &mut game.mobs[0];
        mob.velocity = Vec3::new(-0.5, 0.25, 0.125);
        let (id, expected) = (mob.id, mob.velocity);
        let Held::At(state) = game.boat_holder(Holder::Mob(id), None) else {
            panic!("actual mob holder must resolve");
        };
        assert_eq!(state.velocity, expected);
        assert!(matches!(
            game.boat_holder(Holder::Player, Some([327, 3, 0, 1])),
            Held::Gone
        ));
        assert!(matches!(
            game.boat_holder(Holder::Boat(u32::MAX), None),
            Held::Gone
        ));
        assert!(matches!(
            game.boat_holder(Holder::Knot([0, 100, 0]), None),
            Held::Gone
        ));
    }
}
