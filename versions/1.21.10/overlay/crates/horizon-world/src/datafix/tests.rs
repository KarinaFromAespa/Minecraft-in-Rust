use super::*;
use crate::nbt::parse_snbt;

fn snbt(s: &str) -> Tag {
    parse_snbt(s).unwrap().0
}

#[test]
fn current_data_is_left_alone() {
    let mut tag = snbt("{DataVersion: 5023, Status: \"minecraft:full\"}");
    let before = tag.clone();
    update_chunk(&mut tag, "minecraft:overworld", Some("minecraft:noise"));
    assert_eq!(tag, before);
}

#[test]
fn item_stacks_of_1_20_take_components() {
    let mut player = snbt(
        "{DataVersion: 3465, Inventory: [{Slot: 0b, id: \"minecraft:diamond_sword\", Count: 1b, \
         tag: {Damage: 5, display: {Name: '{\"text\":\"Blade\"}'}}}]}",
    );
    update_player(&mut player);
    let stack = &dynamic::stream(player.get("Inventory"))[0];
    assert_eq!(
        dynamic::as_str(stack.get("id")).as_deref(),
        Some("minecraft:diamond_sword")
    );
    assert_eq!(stack.get("count"), Some(&Tag::Int(1)));
    let components = stack.get("components").expect("components");
    assert_eq!(components.get("minecraft:damage"), Some(&Tag::Int(5)));
    assert_eq!(
        components.get("minecraft:custom_name"),
        Some(&Tag::compound([("text", dynamic::string("Blade"))]))
    );
}

#[test]
fn level_data_of_1_20_moves_to_the_new_fields() {
    // 1.21.10's own form (verified against the real 1.21.10 world's
    // level.dat): the spawn compound exists (fix 4548), but the
    // difficulty-settings move (4771) and the game-rules registry move
    // (4658) are 1.21.11 fixes — 1.21.10 keeps `Difficulty`,
    // `DifficultyLocked`, `hardcore` and `GameRules` as written.
    let mut level = snbt(
        "{DataVersion: 3465, Difficulty: 3b, DifficultyLocked: 1b, hardcore: 0b, SpawnX: 1, SpawnY: 64, SpawnZ: -3, \
         SpawnAngle: 90.0f, GameRules: {doFireTick: \"false\", keepInventory: \"true\", randomTickSpeed: \"5\"}}",
    );
    update(Ty::Level, &mut level, 3465, CURRENT);
    assert!(level.get("difficulty_settings").is_none());
    assert_eq!(level.get("Difficulty"), Some(&Tag::Byte(3)));
    assert_eq!(level.get("DifficultyLocked"), Some(&Tag::Byte(1)));
    assert_eq!(level.get("hardcore"), Some(&Tag::Byte(0)));
    let rules = level.get("GameRules").expect("game rules");
    assert_eq!(rules.get("doFireTick"), Some(&dynamic::string("false")));
    assert_eq!(rules.get("keepInventory"), Some(&dynamic::string("true")));
    assert_eq!(rules.get("randomTickSpeed"), Some(&dynamic::string("5")));
    assert_eq!(
        level.get("spawn").and_then(|s| s.get("pos")),
        Some(&Tag::IntArray(vec![1, 64, -3]))
    );
}

#[test]
fn block_states_take_their_new_field_names() {
    // 1.21.10's own form (verified against the real 1.21.10 world's
    // region files): palette entries keep `Name`/`Properties` — the
    // `id`/`properties` rename is a 1.21.11 fix. A chunk newer than
    // CURRENT is left untouched. (`chain` becomes `iron_chain` on the
    // way up: the 1.21.9 copper-chain rename, which 1.21.10 carries.)
    let mut chunk = snbt(
        "{DataVersion: 3465, Status: \"minecraft:full\", sections: [{Y: 0b, block_states: \
         {palette: [{Name: \"minecraft:chain\", Properties: {axis: \"y\"}}]}}]}",
    );
    update_chunk(&mut chunk, "minecraft:the_nether", Some("minecraft:noise"));
    let section = &dynamic::stream(chunk.get("sections"))[0];
    let state = &dynamic::stream(section.get("block_states").and_then(|b| b.get("palette")))[0];
    assert_eq!(
        dynamic::as_str(state.get("Name")).as_deref(),
        Some("minecraft:iron_chain")
    );
    assert!(state.get("Properties").is_some());
    assert!(state.get("id").is_none());
    assert_eq!(chunk.get("DataVersion"), Some(&Tag::Int(CURRENT)));

    let mut future = snbt(
        "{DataVersion: 4903, Status: \"minecraft:full\", sections: [{Y: 0b, block_states: \
         {palette: [{Name: \"minecraft:chain\", Properties: {axis: \"y\"}}]}}]}",
    );
    update_chunk(&mut future, "minecraft:the_nether", Some("minecraft:noise"));
    assert_eq!(future.get("DataVersion"), Some(&Tag::Int(4903)));
}

#[test]
fn block_states_from_the_flattening_on_are_not_renamed() {
    if !tables::available() {
        eprintln!("skipping: no prepared datafix tables");
        return;
    }
    let props = [("facing".to_owned(), "north".to_owned())];
    assert_eq!(
        native_upgrade_block_state("minecraft:oak_stairs", &props, FLATTENING),
        ("minecraft:oak_stairs".to_owned(), props.to_vec())
    );
    assert_eq!(
        native_upgrade_block_state("minecraft:no_such_block", &[], 1400),
        ("minecraft:no_such_block".to_owned(), Vec::new())
    );
}

#[test]
fn block_states_from_before_the_flattening_take_their_new_names() {
    if !tables::available() {
        eprintln!("skipping: no prepared datafix tables");
        return;
    }
    let granite = [("variant".to_owned(), "granite".to_owned())];
    assert_eq!(
        native_upgrade_block_state("minecraft:stone", &granite, 1400),
        ("minecraft:granite".to_owned(), Vec::new())
    );
    // The same state written after the flattening is not upgraded.
    assert_eq!(
        native_upgrade_block_state("minecraft:stone", &granite, FLATTENING),
        ("minecraft:stone".to_owned(), granite.to_vec())
    );
}

#[test]
fn saved_data_types_cover_both_layouts() {
    // 1.21.10's flat ids, and the 26.x namespaced forms kept as fallback.
    assert_eq!(saved_data_type("scoreboard.dat"), Some(Ty::SavedScoreboard));
    assert_eq!(
        saved_data_type("minecraft/scoreboard.dat"),
        Some(Ty::SavedScoreboard)
    );
    assert_eq!(saved_data_type("chunks.dat"), Some(Ty::SavedTickets));
    assert_eq!(
        saved_data_type("minecraft/chunk_tickets.dat"),
        Some(Ty::SavedTickets)
    );
    assert_eq!(saved_data_type("minecraft/raids.dat"), Some(Ty::SavedRaids));
    assert_eq!(saved_data_type("minecraft/ender_dragon_fight.dat"), None);
}
