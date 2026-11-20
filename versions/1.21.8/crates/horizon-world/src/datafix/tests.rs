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
fn level_data_of_1_20_keeps_its_1_21_8_fields() {
    let mut level = snbt(
        "{DataVersion: 3465, Difficulty: 3b, DifficultyLocked: 1b, hardcore: 0b, SpawnX: 1, SpawnY: 64, SpawnZ: -3, \
         SpawnAngle: 90.0f, GameRules: {doFireTick: \"false\", keepInventory: \"true\", randomTickSpeed: \"5\"}}",
    );
    update(Ty::Level, &mut level, 3465, CURRENT);
    // (update() applies the fixes; the caller stamps DataVersion.)
    // The difficulty_settings compound (4771), the namespaced game_rules
    // (4658) and the `spawn` compound (4548) are all later than 1.21.8, so
    // a 1.21.8 upgrade leaves the 1.20 fields exactly as they are.
    assert_eq!(level.get("Difficulty"), Some(&Tag::Byte(3)));
    assert_eq!(level.get("DifficultyLocked"), Some(&Tag::Byte(1)));
    assert_eq!(level.get("hardcore"), Some(&Tag::Byte(0)));
    assert!(level.get("difficulty_settings").is_none());
    let rules = level.get("GameRules").expect("game rules");
    assert_eq!(
        dynamic::as_str(rules.get("doFireTick")).as_deref(),
        Some("false")
    );
    assert_eq!(
        dynamic::as_str(rules.get("keepInventory")).as_deref(),
        Some("true")
    );
    assert_eq!(
        dynamic::as_str(rules.get("randomTickSpeed")).as_deref(),
        Some("5")
    );
    assert!(level.get("game_rules").is_none());
    assert_eq!(level.get("SpawnX"), Some(&Tag::Int(1)));
    assert_eq!(level.get("SpawnY"), Some(&Tag::Int(64)));
    assert_eq!(level.get("SpawnZ"), Some(&Tag::Int(-3)));
    assert_eq!(level.get("SpawnAngle"), Some(&Tag::Float(90.0f32.to_bits())));
    assert!(level.get("spawn").is_none());
}

#[test]
fn block_states_take_their_new_field_names() {
    let mut chunk = snbt(
        "{DataVersion: 4903, Status: \"minecraft:full\", sections: [{Y: 0b, block_states: \
         {palette: [{Name: \"minecraft:chain\", Properties: {axis: \"y\"}}]}}]}",
    );
    update_chunk(&mut chunk, "minecraft:the_nether", Some("minecraft:noise"));
    let section = &dynamic::stream(chunk.get("sections"))[0];
    let state = &dynamic::stream(section.get("block_states").and_then(|b| b.get("palette")))[0];
    assert_eq!(
        dynamic::as_str(state.get("id")).as_deref(),
        Some("minecraft:chain")
    );
    assert!(state.get("properties").is_some());
    assert_eq!(chunk.get("DataVersion"), Some(&Tag::Int(CURRENT)));
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
