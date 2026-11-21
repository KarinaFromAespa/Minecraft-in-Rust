//! `EntityGetter.getEntityCollisions` on the game side: the boxes that stop
//! the player, a dropped item, an armor stand and other movers (living
//! shulkers and boats).
use super::Game;
use horizon_client::{entities::ObstacleGrid, mobs::Mob};
use horizon_core::geometry::Aabb;

/// `canBeCollidedWith` toward a mover that is not a player: a living shulker.
fn stops_movers(m: &Mob) -> bool {
    m.species() == horizon_client::mobs::Species::Shulker && m.alive()
}

impl Game<'_> {
    /// Client `EntityGetter.getEntityCollisions` for the player: the
    /// entities that `canBeCollidedWith` it, living shulkers
    /// (`Shulker.canBeCollidedWith`) and boats
    /// (`AbstractBoat.canBeCollidedWith`), except the player's own vehicle.
    pub(super) fn player_entity_collisions(&self) -> Vec<Aabb> {
        if self.player.is_none() {
            return Vec::new();
        }
        self.mobs
            .iter()
            .filter(|m| {
                m.species() == horizon_client::mobs::Species::Shulker && m.alive()
            })
            .map(|m| m.bounds())
            .chain(
                self.boats
                    .iter()
                    .filter(|b| Some(b.id) != self.riding)
                    .map(super::boats::Boat::bounds),
            )
            .collect()
    }
    /// `EntityGetter.getEntityCollisions` for a mover that is not a player
    /// (a dropped item, an armor stand, primed TNT, a falling block): the
    /// entities whose `canBeCollidedWith(mover)` is true, the same for all of them. A
    /// boat always, a living shulker. A minecart keeps `Entity`'s false
    /// (`AbstractMinecart` overrides only `canCollideWith`), so an item falls
    /// through it.
    pub(super) fn item_entity_collisions(&self) -> Vec<Aabb> {
        self.mobs
            .iter()
            .filter(|m| stops_movers(m))
            .map(|m| m.bounds())
            .chain(self.boats.iter().map(super::boats::Boat::bounds))
            .collect()
    }
    /// [`Self::item_entity_collisions`] for an armor stand, which mobs can ride
    /// (`/ride`): each mob's box comes with the vehicle at the bottom of what it rides,
    /// so the stand leaves its own riders out (`isPassengerOfSameVehicle`, through the
    /// grid's `near_except`).
    pub(super) fn stand_entity_collisions(&self) -> ObstacleGrid {
        let mut boxes = Vec::new();
        let mut roots = Vec::new();
        for (j, m) in self.mobs.iter().enumerate() {
            if stops_movers(m) {
                boxes.push(m.bounds());
                roots.push(m.extra.mount.and_then(|_| self.mob_root_vehicle(j)));
            }
        }
        for b in &self.boats {
            boxes.push(b.bounds());
            roots.push(None);
        }
        ObstacleGrid::with_roots(boxes, roots)
    }
}

#[cfg(test)]
mod item_collision_tests {
    use crate::game::{Audio, Game, Options, Progress, Settings, load_resources};
    use horizon_client::chunk::Chunk;
    use horizon_core::position::ChunkPos;
    use std::sync::Arc;

    /// An item dropped onto a minecart falls through it to the floor, and one
    /// dropped onto a boat rests on the boat (`canBeCollidedWith`).
    #[test]
    #[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
    fn a_dropped_item_rests_on_a_boat_but_falls_through_a_minecart() {
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
        options.game_dir = Some(
            std::env::temp_dir().join(format!("horizon-item-cart-unit-{}", std::process::id())),
        );
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
        game.set_allow_commands(true);
        let stone = game
            .pack
            .assets
            .registry
            .default_state("minecraft:stone")
            .unwrap()
            .0;
        let mut chunk = Chunk::empty(0, 0);
        for x in 0..16 {
            for z in 0..16 {
                chunk.set(x, 199, z, stone);
            }
        }
        game.world.chunks.insert((0, 0), Arc::new(chunk));
        game.block_ticks
            .add_empty_chunk(ChunkPos::new(0, 0))
            .unwrap();
        assert!(game.run_command("forceload add 0 0"));
        assert!(game.run_command("summon minecart 4.5 200 4.5"));
        assert!(game.run_command("summon oak_boat 10.5 200 10.5"));
        for (x, z) in [(4.5, 4.5), (10.5, 10.5)] {
            assert!(game.run_command(&format!(
                "summon item {x} 201.5 {z} {{Item:{{id:\"minecraft:stick\",count:1}},Motion:[0.0d,0.0d,0.0d]}}"
            )));
        }
        for _ in 0..60 {
            game.tick_world(true).unwrap();
        }
        let rest = |x: f64| {
            game.dropped
                .iter()
                .find(|d| (d.position.x - x).abs() < 0.5)
                .map(|d| d.position.y)
                .unwrap()
        };
        assert_eq!(rest(4.5), 200.0, "through the minecart to the stone");
        let boat_top = game.boats[0].bounds().max_position().y;
        assert_eq!(rest(10.5), boat_top, "on the boat");
    }
}
