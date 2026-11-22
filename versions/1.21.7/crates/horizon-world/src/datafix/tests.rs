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

/// A 1.20 level upgraded to 1.21.7 keeps its fields: the 26.x moves this
/// test used to expect — `Difficulty`/`DifficultyLocked` into
/// `difficulty_settings` (fix 4771), `GameRules` into `game_rules`
/// (4658), `SpawnX/Y/Z` into `spawn` (4548) — are all above CURRENT
/// (4438), and vanilla 1.21.7 has none of them: its `PrimaryLevelData`
/// still reads `Difficulty`, `DifficultyLocked`, `GameRules` and
/// `SpawnX`, and its jar's only `Level*` fixes are the pre-1.20 ones
/// (`LevelDataGeneratorOptionsFix`, `LevelLegacyWorldGenSettingsFix`,
/// `LevelUUIDFix`, `LevelFlatGeneratorInfoFix`). No fix in the 1.20 →
/// 1.21.7 range touches these fields, so they come out unchanged.
#[test]
fn level_data_of_1_20_keeps_its_1_21_7_fields() {
    let mut level = snbt(
        "{DataVersion: 3465, Difficulty: 3b, DifficultyLocked: 1b, hardcore: 0b, SpawnX: 1, SpawnY: 64, SpawnZ: -3, \
         SpawnAngle: 90.0f, GameRules: {doFireTick: \"false\", keepInventory: \"true\", randomTickSpeed: \"5\"}}",
    );
    update(Ty::Level, &mut level, 3465, CURRENT);
    assert_eq!(level.get("Difficulty"), Some(&Tag::Byte(3)));
    assert_eq!(level.get("DifficultyLocked"), Some(&Tag::Byte(1)));
    assert_eq!(level.get("hardcore"), Some(&Tag::Byte(0)));
    assert_eq!(level.get("SpawnX"), Some(&Tag::Int(1)));
    assert_eq!(level.get("SpawnY"), Some(&Tag::Int(64)));
    assert_eq!(level.get("SpawnZ"), Some(&Tag::Int(-3)));
    assert_eq!(
        dynamic::as_float(level.get("SpawnAngle"), 0.0),
        90.0,
        "SpawnAngle"
    );
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
    assert!(level.get("difficulty_settings").is_none());
    assert!(level.get("game_rules").is_none());
    assert!(level.get("spawn").is_none());
}

/// A chunk's palette entries keep `Name`/`Properties` through the 1.21.7
/// upgrade: the rename to `id`/`properties` is fix 5006, a 26.3 fix above
/// CURRENT (4438), and vanilla 1.21.7's chunk palettes still use
/// `Name`/`Properties` (its jar's DataFixers top out at V4421). The old
/// fixture was a 4903 chunk — a future version here, which the upgrade
/// leaves alone — and expected the 26.3 rename.
#[test]
fn block_states_keep_their_1_21_7_field_names() {
    let mut chunk = snbt(
        "{DataVersion: 4435, Status: \"minecraft:full\", sections: [{Y: 0b, block_states: \
         {palette: [{Name: \"minecraft:chain\", Properties: {axis: \"y\"}}]}}]}",
    );
    update_chunk(&mut chunk, "minecraft:the_nether", Some("minecraft:noise"));
    let section = &dynamic::stream(chunk.get("sections"))[0];
    let state = &dynamic::stream(section.get("block_states").and_then(|b| b.get("palette")))[0];
    assert_eq!(
        dynamic::as_str(state.get("Name")).as_deref(),
        Some("minecraft:chain")
    );
    assert_eq!(
        dynamic::as_str(state.get("Properties").and_then(|p| p.get("axis"))).as_deref(),
        Some("y")
    );
    assert!(state.get("id").is_none());
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
