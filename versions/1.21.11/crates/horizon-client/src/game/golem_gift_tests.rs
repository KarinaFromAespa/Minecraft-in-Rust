//! An iron golem's flower gift (`OfferFlowerGoal.stop`): the copper golem it
//! offered the poppy to wears it on its empty antenna (`setItemSlot`) as a
//! guaranteed drop (`setGuaranteedDrop`: drop chance 2.0 for that slot). The
//! gift goes through the game's own `MobEvent::Gift` handler. Other slots
//! keep their chances, and an antenna that already holds an item keeps it.
//!
//! And the statue's own use (`WeatheringCopperGolemStatueBlock.useItemOn`):
//! a bare hand cycles the pose, honeycomb passes through, and an axe frees
//! an unaffected statue — waxed or not — but not a weathered one.
use super::super::*;
use horizon_client::{
    chunk::Chunk,
    items::ItemStack,
    mobs::{MobEvent, Species},
};
use horizon_core::position::ChunkPos;
use std::sync::Arc;

const EMPTY: &str = "0000609b-0000-0000-0000-000000000001";
const CHANCES: &str = "0000609b-0000-0000-0000-000000000002";
const OCCUPIED: &str = "0000609b-0000-0000-0000-000000000003";

fn game() -> Game<'static> {
    let root = std::path::PathBuf::from(
        std::env::var_os("HORIZON_TEST_RESOURCES")
            .expect("set HORIZON_TEST_RESOURCES to prepared resources"),
    );
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
    options.execution_role = execution_role::ExecutionRole::IntegratedServer;
    let res: &'static _ = Box::leak(Box::new(loaded.resources));
    let mut game = Game::new(
        res,
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
    game.world
        .chunks
        .insert((0, 0), Arc::new(Chunk::empty(0, 0)));
    game.block_ticks
        .add_empty_chunk(ChunkPos::new(0, 0))
        .unwrap();
    for command in [
        "forceload add 0 0",
        "fill 0 199 0 15 199 15 stone",
        "tick freeze",
        // Waxed (`next_weather_age` -2): no statue while the test runs.
        "summon copper_golem 2.5 200 2.5 {NoAI:1b,PersistenceRequired:1b,next_weather_age:-2L,UUID:[I;24731,0,0,1]}",
        "summon copper_golem 6.5 200 2.5 {NoAI:1b,PersistenceRequired:1b,next_weather_age:-2L,UUID:[I;24731,0,0,2],drop_chances:{head:0.5f,mainhand:0.25f}}",
        "summon copper_golem 10.5 200 2.5 {NoAI:1b,PersistenceRequired:1b,next_weather_age:-2L,UUID:[I;24731,0,0,3]}",
    ] {
        assert!(game.run_command(command), "{command}");
    }
    game
}

fn id(game: &Game<'_>, uuid: &str) -> u32 {
    match game.target_with_uuid(horizon_client::save::uuid_from_text(uuid).unwrap()) {
        Some(super::commands::Target::Mob(id)) => id,
        other => panic!("{other:?}"),
    }
}

/// `OfferFlowerGoal.stop` for the golem whose UUID is `uuid`.
fn gift(game: &mut Game<'_>, uuid: &str) {
    gift_item(game, uuid, "minecraft:poppy");
}

fn gift_item(game: &mut Game<'_>, uuid: &str, item: &'static str) {
    let target = id(game, uuid);
    game.handle_mob_events(vec![MobEvent::Gift { target, item }])
        .unwrap();
}

fn holds(game: &mut Game<'_>, uuid: &str, nbt: &str) -> bool {
    game.run_command(&format!(
        "execute if entity @e[nbt={{UUID:{}}},nbt={nbt}]",
        ints(uuid)
    ))
}

fn ints(uuid: &str) -> String {
    let u = horizon_client::save::uuid_from_text(uuid).unwrap();
    format!("[I;{},{},{},{}]", u[0], u[1], u[2], u[3])
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn a_gifted_poppy_is_a_guaranteed_drop() {
    let mut game = game();
    assert!(
        !game.run_command(&format!(
            "execute if items entity {EMPTY} saddle minecraft:poppy"
        )),
        "the antenna starts empty"
    );
    assert!(!holds(&mut game, EMPTY, "{drop_chances:{saddle:2.0f}}"));
    gift(&mut game, EMPTY);
    assert!(
        game.run_command(&format!(
            "execute if items entity {EMPTY} saddle minecraft:poppy"
        )),
        "the poppy is on the antenna"
    );
    // The same check as the vanilla capture (tools: probes/golem-gift).
    assert!(
        holds(
            &mut game,
            EMPTY,
            "{equipment:{saddle:{id:\"minecraft:poppy\"}},drop_chances:{saddle:2.0f}}"
        ),
        "Mob.setGuaranteedDrop(antenna)"
    );
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn the_gift_keeps_the_other_slots_chances() {
    let mut game = game();
    assert!(holds(
        &mut game,
        CHANCES,
        "{drop_chances:{head:0.5f,mainhand:0.25f}}"
    ));
    gift(&mut game, CHANCES);
    assert!(holds(
        &mut game,
        CHANCES,
        "{equipment:{saddle:{id:\"minecraft:poppy\"}},drop_chances:{head:0.5f,mainhand:0.25f,saddle:2.0f}}"
    ));
    let i = game
        .mobs
        .iter()
        .position(|m| m.id == id(&game, CHANCES))
        .unwrap();
    let chances = *game.mobs[i].extra.drop_chances.clone().unwrap();
    // EquipmentSlot.VALUES: mainhand, offhand, feet, legs, chest, head, body, saddle.
    let default = horizon_client::mobs::drops::DEFAULT_DROP_CHANCE;
    assert_eq!(
        chances,
        [0.25, default, default, default, default, 0.5, default, 2.0]
    );
}

/// The antenna is only filled by a gift here (a loaded or commanded antenna
/// item is not modelled yet), so a first gift of another item occupies it.
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn an_antenna_that_holds_an_item_keeps_it_and_its_chance() {
    let mut game = game();
    gift_item(&mut game, OCCUPIED, "minecraft:dandelion");
    assert!(game.run_command(&format!(
        "execute if items entity {OCCUPIED} saddle minecraft:dandelion"
    )));
    let i = game
        .mobs
        .iter()
        .position(|m| m.id == id(&game, OCCUPIED))
        .unwrap();
    let chances = game.mobs[i].extra.drop_chances.clone();
    gift(&mut game, OCCUPIED);
    assert!(game.run_command(&format!(
        "execute if items entity {OCCUPIED} saddle minecraft:dandelion"
    )));
    assert!(!game.run_command(&format!(
        "execute if items entity {OCCUPIED} saddle minecraft:poppy"
    )));
    let i = game
        .mobs
        .iter()
        .position(|m| m.id == id(&game, OCCUPIED))
        .unwrap();
    assert_eq!(game.mobs[i].extra.drop_chances, chances, "the chances stay");
}

/// The effective chance of each slot (`Mob.dropChances.byEquipment`), as a
/// death drop reads it, in `EquipmentSlot.VALUES` order.
fn effective(game: &Game<'_>, uuid: &str) -> Vec<f32> {
    let m = game.mobs.iter().find(|m| m.id == id(game, uuid)).unwrap();
    horizon_client::mobs::drops::SLOTS
        .iter()
        .map(|slot| horizon_client::mobs::drops::drop_chance(m, slot))
        .collect()
}

/// A golem with no chance array gets one with the defaults, and one with an
/// array keeps it: every slot but the antenna reads the chance it read
/// before the gift.
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn every_other_slot_keeps_its_effective_chance() {
    let mut game = game();
    for uuid in [EMPTY, CHANCES] {
        let before = effective(&game, uuid);
        gift(&mut game, uuid);
        let after = effective(&game, uuid);
        eprintln!("{uuid}: before {before:?} after {after:?}");
        assert_eq!(after[..7], before[..7], "{uuid}: the other slots");
        assert_eq!(after[7], 2.0, "{uuid}: the antenna is a guaranteed drop");
    }
    let i = game
        .mobs
        .iter()
        .position(|m| m.id == id(&game, EMPTY))
        .unwrap();
    assert!(
        game.mobs[i].extra.drop_chances.is_some(),
        "the gift made the array"
    );
}

/// The `copper_golem_pose` property of the statue at `p`, or "".
fn pose(game: &Game<'_>, p: [i32; 3]) -> String {
    let id = game.world.block(p[0], p[1], p[2]);
    game.pack
        .assets
        .registry
        .state(id)
        .and_then(|s| s.properties.get("copper_golem_pose").cloned())
        .unwrap_or_default()
}

fn block_name(game: &Game<'_>, p: [i32; 3]) -> String {
    let id = game.world.block(p[0], p[1], p[2]);
    game.pack
        .assets
        .registry
        .state(id)
        .map_or(String::new(), |s| s.name.clone())
}

/// `CopperGolemStatueBlock.updatePose`: a bare hand cycles the pose
/// (standing -> sitting -> running -> star -> standing); honeycomb passes
/// through to wax instead.
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn a_statue_cycles_its_pose_on_a_bare_hand_and_ignores_honeycomb() {
    let mut game = game();
    let p = [13, 200, 13];
    assert!(game.run_command(
        "setblock 13 200 13 minecraft:copper_golem_statue[facing=south,copper_golem_pose=standing] strict"
    ));
    assert_eq!(pose(&game, p), "standing");
    for next in ["sitting", "running", "star", "standing"] {
        assert!(game.use_golem_statue(p, None), "a bare hand cycles");
        assert_eq!(pose(&game, p), next);
    }
    let honeycomb = ItemStack::new("minecraft:honeycomb", 1);
    assert!(!game.use_golem_statue(p, Some(&honeycomb)));
    assert_eq!(pose(&game, p), "standing");
}

/// `WeatheringCopperGolemStatueBlock.useItemOn`: an axe frees an unaffected
/// statue — waxed or not — and leaves a weathered one standing (the axe's
/// own transform scrapes it instead).
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn an_axe_frees_a_waxed_statue_but_not_a_weathered_one() {
    let mut game = game();
    let axe = ItemStack::new("minecraft:diamond_axe", 1);
    let golems = |game: &Game<'_>| {
        game.mobs
            .iter()
            .filter(|m| m.species() == Species::CopperGolem && !m.removed)
            .count()
    };
    let waxed = [14, 200, 13];
    assert!(game.run_command(
        "setblock 14 200 13 minecraft:waxed_copper_golem_statue[facing=south,copper_golem_pose=star] strict"
    ));
    let before = golems(&game);
    assert!(game.use_golem_statue(waxed, Some(&axe)));
    assert_eq!(golems(&game), before + 1, "the golem walks out");
    assert_eq!(block_name(&game, waxed), "minecraft:air");
    let weathered = [13, 200, 14];
    assert!(game.run_command(
        "setblock 13 200 14 minecraft:weathered_copper_golem_statue[facing=south,copper_golem_pose=standing] strict"
    ));
    assert!(!game.use_golem_statue(weathered, Some(&axe)));
    assert_eq!(
        block_name(&game, weathered),
        "minecraft:weathered_copper_golem_statue"
    );
    assert_eq!(golems(&game), before + 1);
}
