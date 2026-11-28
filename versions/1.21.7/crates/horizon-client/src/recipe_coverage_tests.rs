//! One test per vanilla 1.21.7 recipe, generated from the 1.21.7 server
//! jar's `data/minecraft/recipe/*.json` by `tools/recipe_coverage_gen.py`.
//! Each test loads the port's `Recipes` (and `StationRecipes`) from the
//! prepared resources and asserts that one vanilla recipe exists in the port
//! and crafts, cooks, cuts or smiths to the right result. A failure names the
//! exact recipe id; the missing-recipe list is the deliverable, so these tests
//! assert and do not fix. Tests skip (they return without asserting) when
//! `HORIZON_RESOURCES` is unset or the resources are absent, as the level
//! tests do.
//!
//! Generated: 1407 recipes (24 blasting, 9 campfire_cooking, 1 crafting_decorated_pot, 673 crafting_shaped, 295 crafting_shapeless, 1 crafting_special_armordye, 1 crafting_special_bannerduplicate, 1 crafting_special_bookcloning, 1 crafting_special_firework_rocket, 1 crafting_special_firework_star, 1 crafting_special_firework_star_fade, 1 crafting_special_mapcloning, 1 crafting_special_mapextending, 1 crafting_special_repairitem, 1 crafting_special_shielddecoration, 1 crafting_special_tippedarrow, 32 crafting_transmute, 72 smelting, 9 smithing_transform, 18 smithing_trim, 9 smoking, 254 stonecutting).
use super::*;
use crate::enchant::Enchantments;
use crate::stations::StationRecipes;
use std::sync::OnceLock;

struct Fixture {
    root: std::path::PathBuf,
    items: Items,
    recipes: Recipes,
    stations: StationRecipes,
    enchantments: Enchantments,
}

fn fixture() -> Option<&'static Fixture> {
    static FIXTURE: OnceLock<Option<Fixture>> = OnceLock::new();
    FIXTURE
        .get_or_init(|| {
            let root = std::env::var_os("HORIZON_RESOURCES").map(std::path::PathBuf::from)?;
            if !root.join("data/minecraft/recipe").is_dir() {
                eprintln!(
                    "skipping recipe coverage tests: no recipes at {}",
                    root.display()
                );
                return None;
            }
            let items = Items::catalog(&root, &PackOverlay::NONE).ok()?;
            let recipes = Recipes::load(&root, &items, &PackOverlay::NONE).ok()?;
            let stations = StationRecipes::load(&root, &items, &PackOverlay::NONE);
            let enchantments = Enchantments::load(&root);
            Some(Fixture {
                root,
                items,
                recipes,
                stations,
                enchantments,
            })
        })
        .as_ref()
}

/// The first concrete item an ingredient value names: an item id itself, or
/// the first item of a `#tag` in the port's tag order, or the first entry of
/// an array of either.
fn first_item(v: &Value, items: &Items) -> Option<String> {
    match v {
        Value::String(s) => match s.strip_prefix('#') {
            Some(tag) => items.item_tag_order.get(tag)?.first().cloned(),
            None => Some(s.clone()),
        },
        Value::Array(a) => a.iter().find_map(|e| first_item(e, items)),
        Value::Object(o) => {
            if let Some(item) = o.get("item") {
                first_item(item, items)
            } else if let Some(tag) = o.get("tag").and_then(Value::as_str) {
                items.item_tag_order.get(tag)?.first().cloned()
            } else {
                None
            }
        }
        _ => None,
    }
}

/// A recipe JSON's `result` as (item id, count): a bare string is one of the
/// item; a dict carries `id` and an optional `count`.
fn result_id_count(v: &Value) -> Option<(String, u32)> {
    match &v["result"] {
        Value::String(s) => Some((s.clone(), 1)),
        Value::Object(_) => {
            let id = v["result"]["id"].as_str()?.to_owned();
            let count = v["result"]["count"].as_u64().unwrap_or(1) as u32;
            Some((id, count))
        }
        _ => None,
    }
}

/// The stored name is the bare file name (`oak_boat`); accept the qualified
/// form too.
fn assert_same_id(want: &str, found: &str) {
    let bare = want.strip_prefix("minecraft:").unwrap_or(want);
    assert!(
        found == bare || found == want,
        "{want}: matched a different recipe ({found})"
    );
}

/// `assert_same_id`, plus the first-match collision vanilla itself has: the
/// `_simple` twin of a special recipe matches the same grid, and vanilla's
/// `RecipeMap` is fed from a `SortedMap` with `getRecipeFor` returning the
/// first match, so the special sorts first (`decorated_pot` <
/// `decorated_pot_simple`, `firework_rocket` < `firework_rocket_simple`)
/// and the port returning it is vanilla-correct — the two twins assemble the
/// same result, so the test still proves the wanted recipe crafts to the
/// right result. The wanted recipe must itself be loaded, so the test still
/// proves it exists.
fn assert_same_or_twin(id: &str, found_id: &str, recipes: &Recipes) {
    let twin = match id {
        "minecraft:decorated_pot_simple" => "decorated_pot",
        "minecraft:firework_rocket_simple" => "firework_rocket",
        _ => "",
    };
    if found_id == twin {
        assert!(
            recipes.recipe(id).is_some(),
            "{id}: matched its twin but the recipe itself is not loaded"
        );
        return;
    }
    assert_same_id(id, found_id);
}

fn check_recipe(id: &str) {
    let Some(fx) = fixture() else {
        return;
    };
    let bare = id.strip_prefix("minecraft:").unwrap_or(id);
    let path = fx
        .root
        .join("data/minecraft/recipe")
        .join(format!("{bare}.json"));
    let Ok(bytes) = std::fs::read(&path) else {
        panic!(
            "{id}: recipe JSON missing from prepared resources at {}",
            path.display()
        );
    };
    let v: Value = serde_json::from_slice(&bytes).expect("recipe JSON parses");
    let ty = v["type"].as_str().unwrap_or("");
    match ty {
        "minecraft:crafting_shaped" => check_shaped(fx, id, &v),
        "minecraft:crafting_shapeless" => check_shapeless(fx, id, &v),
        "minecraft:smelting"
        | "minecraft:blasting"
        | "minecraft:smoking"
        | "minecraft:campfire_cooking" => check_cooking(fx, id, ty, &v),
        "minecraft:stonecutting" => check_stonecutting(fx, id, &v),
        "minecraft:smithing_transform" | "minecraft:smithing_trim" => {
            check_smithing(fx, id, ty, &v)
        }
        "minecraft:crafting_transmute" => check_transmute(fx, id, &v),
        _ if ty.starts_with("minecraft:crafting_special")
            || ty == "minecraft:crafting_decorated_pot" =>
        {
            check_special(fx, id)
        }
        _ => panic!("{id}: unknown recipe type {ty}"),
    }
}

fn check_shaped(fx: &Fixture, id: &str, v: &Value) {
    let pattern: Vec<String> = v["pattern"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let mut grid: Vec<Option<ItemStack>> = vec![None; 9];
    for (y, row) in pattern.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if c == ' ' {
                continue;
            }
            let ing = &v["key"][c.to_string()];
            let item = first_item(ing, &fx.items)
                .unwrap_or_else(|| panic!("{id}: no item for key '{c}' in {ing}"));
            grid[y * 3 + x] = Some(ItemStack::new(&item, 1));
        }
    }
    let (found_id, result) = fx
        .recipes
        .find(&grid, 3)
        .unwrap_or_else(|| panic!("{id}: did not craft from its pattern"));
    assert_same_or_twin(id, found_id, &fx.recipes);
    let (want_item, want_count) = result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
    assert_eq!(result.item.as_ref(), want_item, "{id}: result item");
    assert_eq!(result.count, want_count, "{id}: result count");
}

fn check_shapeless(fx: &Fixture, id: &str, v: &Value) {
    let mut grid: Vec<Option<ItemStack>> = vec![None; 9];
    for (i, ing) in v["ingredients"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        let item = first_item(ing, &fx.items)
            .unwrap_or_else(|| panic!("{id}: no item for ingredient {i} in {ing}"));
        grid[i] = Some(ItemStack::new(&item, 1));
    }
    let (found_id, result) = fx
        .recipes
        .find(&grid, 3)
        .unwrap_or_else(|| panic!("{id}: did not craft from its ingredients"));
    assert_same_or_twin(id, found_id, &fx.recipes);
    let (want_item, want_count) = result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
    assert_eq!(result.item.as_ref(), want_item, "{id}: result item");
    assert_eq!(result.count, want_count, "{id}: result count");
}

fn check_cooking(fx: &Fixture, id: &str, ty: &str, v: &Value) {
    let item = first_item(&v["ingredient"], &fx.items)
        .unwrap_or_else(|| panic!("{id}: no item for ingredient {}", v["ingredient"]));
    let stack = ItemStack::new(&item, 1);
    let c = fx
        .recipes
        .cook(ty, &stack)
        .unwrap_or_else(|| panic!("{id}: no {ty} recipe for {item}"));
    assert_same_id(id, &c.id);
    let (want_item, want_count) = result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
    assert_eq!(c.result.item.as_ref(), want_item, "{id}: result item");
    assert_eq!(c.result.count, want_count, "{id}: result count");
}

fn check_stonecutting(fx: &Fixture, id: &str, v: &Value) {
    let item = first_item(&v["ingredient"], &fx.items)
        .unwrap_or_else(|| panic!("{id}: no item for ingredient {}", v["ingredient"]));
    let stack = ItemStack::new(&item, 1);
    let found = fx.stations.stonecutter(&stack);
    let (want_item, want_count) = result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
    let bare = id.strip_prefix("minecraft:").unwrap_or(id);
    assert!(
        found.iter().any(|r| (r.id == bare || r.id == id)
            && r.result.item.as_ref() == want_item
            && r.result.count == want_count),
        "{id}: no stonecutting result {want_item} x{want_count} for {item}"
    );
}

fn check_smithing(fx: &Fixture, id: &str, ty: &str, v: &Value) {
    let stack = |k: &str| {
        first_item(&v[k], &fx.items)
            .map(|item| ItemStack::new(&item, 1))
            .unwrap_or_else(|| panic!("{id}: no item for {k} {}", v[k]))
    };
    let template = stack("template");
    let base = stack("base");
    let addition = stack("addition");
    let (found_id, result) = fx
        .stations
        .smithing_recipe(
            &fx.enchantments,
            Some(&template),
            Some(&base),
            Some(&addition),
        )
        .unwrap_or_else(|| panic!("{id}: no smithing result"));
    assert_same_id(id, found_id);
    if ty == "minecraft:smithing_transform" {
        let (want_item, want_count) =
            result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
        assert_eq!(result.item.as_ref(), want_item, "{id}: result item");
        assert_eq!(result.count, want_count, "{id}: result count");
    } else {
        // A trim keeps the base item, trimmed.
        assert_eq!(
            result.item.as_ref(),
            base.item.as_ref(),
            "{id}: a trim keeps the base item"
        );
    }
}

fn check_transmute(fx: &Fixture, id: &str, v: &Value) {
    let input = first_item(&v["input"], &fx.items)
        .unwrap_or_else(|| panic!("{id}: no item for input {}", v["input"]));
    let material = first_item(&v["material"], &fx.items)
        .unwrap_or_else(|| panic!("{id}: no item for material {}", v["material"]));
    let mut grid: Vec<Option<ItemStack>> = vec![None; 9];
    grid[0] = Some(ItemStack::new(&input, 1));
    grid[1] = Some(ItemStack::new(&material, 1));
    let (found_id, result) = fx
        .recipes
        .find(&grid, 3)
        .unwrap_or_else(|| panic!("{id}: did not transmute from input + material"));
    assert_same_id(id, found_id);
    let (want_item, _) = result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
    assert_eq!(result.item.as_ref(), want_item, "{id}: result item");
}

fn check_special(fx: &Fixture, id: &str) {
    let recipe = fx
        .recipes
        .recipe(id)
        .unwrap_or_else(|| panic!("{id}: special recipe not loaded"));
    assert!(recipe.is_special(), "{id}: loaded but not special");
}

#[test]
fn recipe_acacia_boat() {
    check_recipe("minecraft:acacia_boat");
}

#[test]
fn recipe_acacia_button() {
    check_recipe("minecraft:acacia_button");
}

#[test]
fn recipe_acacia_chest_boat() {
    check_recipe("minecraft:acacia_chest_boat");
}

#[test]
fn recipe_acacia_door() {
    check_recipe("minecraft:acacia_door");
}

#[test]
fn recipe_acacia_fence() {
    check_recipe("minecraft:acacia_fence");
}

#[test]
fn recipe_acacia_fence_gate() {
    check_recipe("minecraft:acacia_fence_gate");
}

#[test]
fn recipe_acacia_hanging_sign() {
    check_recipe("minecraft:acacia_hanging_sign");
}

#[test]
fn recipe_acacia_planks() {
    check_recipe("minecraft:acacia_planks");
}

#[test]
fn recipe_acacia_pressure_plate() {
    check_recipe("minecraft:acacia_pressure_plate");
}

#[test]
fn recipe_acacia_sign() {
    check_recipe("minecraft:acacia_sign");
}

#[test]
fn recipe_acacia_slab() {
    check_recipe("minecraft:acacia_slab");
}

#[test]
fn recipe_acacia_stairs() {
    check_recipe("minecraft:acacia_stairs");
}

#[test]
fn recipe_acacia_trapdoor() {
    check_recipe("minecraft:acacia_trapdoor");
}

#[test]
fn recipe_acacia_wood() {
    check_recipe("minecraft:acacia_wood");
}

#[test]
fn recipe_activator_rail() {
    check_recipe("minecraft:activator_rail");
}

#[test]
fn recipe_amethyst_block() {
    check_recipe("minecraft:amethyst_block");
}

#[test]
fn recipe_andesite() {
    check_recipe("minecraft:andesite");
}

#[test]
fn recipe_andesite_slab() {
    check_recipe("minecraft:andesite_slab");
}

#[test]
fn recipe_andesite_slab_from_andesite_stonecutting() {
    check_recipe("minecraft:andesite_slab_from_andesite_stonecutting");
}

#[test]
fn recipe_andesite_stairs() {
    check_recipe("minecraft:andesite_stairs");
}

#[test]
fn recipe_andesite_stairs_from_andesite_stonecutting() {
    check_recipe("minecraft:andesite_stairs_from_andesite_stonecutting");
}

#[test]
fn recipe_andesite_wall() {
    check_recipe("minecraft:andesite_wall");
}

#[test]
fn recipe_andesite_wall_from_andesite_stonecutting() {
    check_recipe("minecraft:andesite_wall_from_andesite_stonecutting");
}

#[test]
fn recipe_anvil() {
    check_recipe("minecraft:anvil");
}

#[test]
fn recipe_armor_dye() {
    check_recipe("minecraft:armor_dye");
}

#[test]
fn recipe_armor_stand() {
    check_recipe("minecraft:armor_stand");
}

#[test]
fn recipe_arrow() {
    check_recipe("minecraft:arrow");
}

#[test]
fn recipe_baked_potato() {
    check_recipe("minecraft:baked_potato");
}

#[test]
fn recipe_baked_potato_from_campfire_cooking() {
    check_recipe("minecraft:baked_potato_from_campfire_cooking");
}

#[test]
fn recipe_baked_potato_from_smoking() {
    check_recipe("minecraft:baked_potato_from_smoking");
}

#[test]
fn recipe_bamboo_block() {
    check_recipe("minecraft:bamboo_block");
}

#[test]
fn recipe_bamboo_button() {
    check_recipe("minecraft:bamboo_button");
}

#[test]
fn recipe_bamboo_chest_raft() {
    check_recipe("minecraft:bamboo_chest_raft");
}

#[test]
fn recipe_bamboo_door() {
    check_recipe("minecraft:bamboo_door");
}

#[test]
fn recipe_bamboo_fence() {
    check_recipe("minecraft:bamboo_fence");
}

#[test]
fn recipe_bamboo_fence_gate() {
    check_recipe("minecraft:bamboo_fence_gate");
}

#[test]
fn recipe_bamboo_hanging_sign() {
    check_recipe("minecraft:bamboo_hanging_sign");
}

#[test]
fn recipe_bamboo_mosaic() {
    check_recipe("minecraft:bamboo_mosaic");
}

#[test]
fn recipe_bamboo_mosaic_slab() {
    check_recipe("minecraft:bamboo_mosaic_slab");
}

#[test]
fn recipe_bamboo_mosaic_stairs() {
    check_recipe("minecraft:bamboo_mosaic_stairs");
}

#[test]
fn recipe_bamboo_planks() {
    check_recipe("minecraft:bamboo_planks");
}

#[test]
fn recipe_bamboo_pressure_plate() {
    check_recipe("minecraft:bamboo_pressure_plate");
}

#[test]
fn recipe_bamboo_raft() {
    check_recipe("minecraft:bamboo_raft");
}

#[test]
fn recipe_bamboo_sign() {
    check_recipe("minecraft:bamboo_sign");
}

#[test]
fn recipe_bamboo_slab() {
    check_recipe("minecraft:bamboo_slab");
}

#[test]
fn recipe_bamboo_stairs() {
    check_recipe("minecraft:bamboo_stairs");
}

#[test]
fn recipe_bamboo_trapdoor() {
    check_recipe("minecraft:bamboo_trapdoor");
}

#[test]
fn recipe_banner_duplicate() {
    check_recipe("minecraft:banner_duplicate");
}

#[test]
fn recipe_barrel() {
    check_recipe("minecraft:barrel");
}

#[test]
fn recipe_beacon() {
    check_recipe("minecraft:beacon");
}

#[test]
fn recipe_beehive() {
    check_recipe("minecraft:beehive");
}

#[test]
fn recipe_beetroot_soup() {
    check_recipe("minecraft:beetroot_soup");
}

#[test]
fn recipe_birch_boat() {
    check_recipe("minecraft:birch_boat");
}

#[test]
fn recipe_birch_button() {
    check_recipe("minecraft:birch_button");
}

#[test]
fn recipe_birch_chest_boat() {
    check_recipe("minecraft:birch_chest_boat");
}

#[test]
fn recipe_birch_door() {
    check_recipe("minecraft:birch_door");
}

#[test]
fn recipe_birch_fence() {
    check_recipe("minecraft:birch_fence");
}

#[test]
fn recipe_birch_fence_gate() {
    check_recipe("minecraft:birch_fence_gate");
}

#[test]
fn recipe_birch_hanging_sign() {
    check_recipe("minecraft:birch_hanging_sign");
}

#[test]
fn recipe_birch_planks() {
    check_recipe("minecraft:birch_planks");
}

#[test]
fn recipe_birch_pressure_plate() {
    check_recipe("minecraft:birch_pressure_plate");
}

#[test]
fn recipe_birch_sign() {
    check_recipe("minecraft:birch_sign");
}

#[test]
fn recipe_birch_slab() {
    check_recipe("minecraft:birch_slab");
}

#[test]
fn recipe_birch_stairs() {
    check_recipe("minecraft:birch_stairs");
}

#[test]
fn recipe_birch_trapdoor() {
    check_recipe("minecraft:birch_trapdoor");
}

#[test]
fn recipe_birch_wood() {
    check_recipe("minecraft:birch_wood");
}

#[test]
fn recipe_black_banner() {
    check_recipe("minecraft:black_banner");
}

#[test]
fn recipe_black_bed() {
    check_recipe("minecraft:black_bed");
}

#[test]
fn recipe_black_bundle() {
    check_recipe("minecraft:black_bundle");
}

#[test]
fn recipe_black_candle() {
    check_recipe("minecraft:black_candle");
}

#[test]
fn recipe_black_carpet() {
    check_recipe("minecraft:black_carpet");
}

#[test]
fn recipe_black_concrete_powder() {
    check_recipe("minecraft:black_concrete_powder");
}

#[test]
fn recipe_black_dye() {
    check_recipe("minecraft:black_dye");
}

#[test]
fn recipe_black_dye_from_wither_rose() {
    check_recipe("minecraft:black_dye_from_wither_rose");
}

#[test]
fn recipe_black_glazed_terracotta() {
    check_recipe("minecraft:black_glazed_terracotta");
}

#[test]
fn recipe_black_harness() {
    check_recipe("minecraft:black_harness");
}

#[test]
fn recipe_black_shulker_box() {
    check_recipe("minecraft:black_shulker_box");
}

#[test]
fn recipe_black_stained_glass() {
    check_recipe("minecraft:black_stained_glass");
}

#[test]
fn recipe_black_stained_glass_pane() {
    check_recipe("minecraft:black_stained_glass_pane");
}

#[test]
fn recipe_black_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:black_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_black_terracotta() {
    check_recipe("minecraft:black_terracotta");
}

#[test]
fn recipe_blackstone_slab() {
    check_recipe("minecraft:blackstone_slab");
}

#[test]
fn recipe_blackstone_slab_from_blackstone_stonecutting() {
    check_recipe("minecraft:blackstone_slab_from_blackstone_stonecutting");
}

#[test]
fn recipe_blackstone_stairs() {
    check_recipe("minecraft:blackstone_stairs");
}

#[test]
fn recipe_blackstone_stairs_from_blackstone_stonecutting() {
    check_recipe("minecraft:blackstone_stairs_from_blackstone_stonecutting");
}

#[test]
fn recipe_blackstone_wall() {
    check_recipe("minecraft:blackstone_wall");
}

#[test]
fn recipe_blackstone_wall_from_blackstone_stonecutting() {
    check_recipe("minecraft:blackstone_wall_from_blackstone_stonecutting");
}

#[test]
fn recipe_blast_furnace() {
    check_recipe("minecraft:blast_furnace");
}

#[test]
fn recipe_blaze_powder() {
    check_recipe("minecraft:blaze_powder");
}

#[test]
fn recipe_blue_banner() {
    check_recipe("minecraft:blue_banner");
}

#[test]
fn recipe_blue_bed() {
    check_recipe("minecraft:blue_bed");
}

#[test]
fn recipe_blue_bundle() {
    check_recipe("minecraft:blue_bundle");
}

#[test]
fn recipe_blue_candle() {
    check_recipe("minecraft:blue_candle");
}

#[test]
fn recipe_blue_carpet() {
    check_recipe("minecraft:blue_carpet");
}

#[test]
fn recipe_blue_concrete_powder() {
    check_recipe("minecraft:blue_concrete_powder");
}

#[test]
fn recipe_blue_dye() {
    check_recipe("minecraft:blue_dye");
}

#[test]
fn recipe_blue_dye_from_cornflower() {
    check_recipe("minecraft:blue_dye_from_cornflower");
}

#[test]
fn recipe_blue_glazed_terracotta() {
    check_recipe("minecraft:blue_glazed_terracotta");
}

#[test]
fn recipe_blue_harness() {
    check_recipe("minecraft:blue_harness");
}

#[test]
fn recipe_blue_ice() {
    check_recipe("minecraft:blue_ice");
}

#[test]
fn recipe_blue_shulker_box() {
    check_recipe("minecraft:blue_shulker_box");
}

#[test]
fn recipe_blue_stained_glass() {
    check_recipe("minecraft:blue_stained_glass");
}

#[test]
fn recipe_blue_stained_glass_pane() {
    check_recipe("minecraft:blue_stained_glass_pane");
}

#[test]
fn recipe_blue_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:blue_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_blue_terracotta() {
    check_recipe("minecraft:blue_terracotta");
}

#[test]
fn recipe_bolt_armor_trim_smithing_template() {
    check_recipe("minecraft:bolt_armor_trim_smithing_template");
}

#[test]
fn recipe_bolt_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:bolt_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_bone_block() {
    check_recipe("minecraft:bone_block");
}

#[test]
fn recipe_bone_meal() {
    check_recipe("minecraft:bone_meal");
}

#[test]
fn recipe_bone_meal_from_bone_block() {
    check_recipe("minecraft:bone_meal_from_bone_block");
}

#[test]
fn recipe_book() {
    check_recipe("minecraft:book");
}

#[test]
fn recipe_book_cloning() {
    check_recipe("minecraft:book_cloning");
}

#[test]
fn recipe_bookshelf() {
    check_recipe("minecraft:bookshelf");
}

#[test]
fn recipe_bordure_indented_banner_pattern() {
    check_recipe("minecraft:bordure_indented_banner_pattern");
}

#[test]
fn recipe_bow() {
    check_recipe("minecraft:bow");
}

#[test]
fn recipe_bowl() {
    check_recipe("minecraft:bowl");
}

#[test]
fn recipe_bread() {
    check_recipe("minecraft:bread");
}

#[test]
fn recipe_brewing_stand() {
    check_recipe("minecraft:brewing_stand");
}

#[test]
fn recipe_brick() {
    check_recipe("minecraft:brick");
}

#[test]
fn recipe_brick_slab() {
    check_recipe("minecraft:brick_slab");
}

#[test]
fn recipe_brick_slab_from_bricks_stonecutting() {
    check_recipe("minecraft:brick_slab_from_bricks_stonecutting");
}

#[test]
fn recipe_brick_stairs() {
    check_recipe("minecraft:brick_stairs");
}

#[test]
fn recipe_brick_stairs_from_bricks_stonecutting() {
    check_recipe("minecraft:brick_stairs_from_bricks_stonecutting");
}

#[test]
fn recipe_brick_wall() {
    check_recipe("minecraft:brick_wall");
}

#[test]
fn recipe_brick_wall_from_bricks_stonecutting() {
    check_recipe("minecraft:brick_wall_from_bricks_stonecutting");
}

#[test]
fn recipe_bricks() {
    check_recipe("minecraft:bricks");
}

#[test]
fn recipe_brown_banner() {
    check_recipe("minecraft:brown_banner");
}

#[test]
fn recipe_brown_bed() {
    check_recipe("minecraft:brown_bed");
}

#[test]
fn recipe_brown_bundle() {
    check_recipe("minecraft:brown_bundle");
}

#[test]
fn recipe_brown_candle() {
    check_recipe("minecraft:brown_candle");
}

#[test]
fn recipe_brown_carpet() {
    check_recipe("minecraft:brown_carpet");
}

#[test]
fn recipe_brown_concrete_powder() {
    check_recipe("minecraft:brown_concrete_powder");
}

#[test]
fn recipe_brown_dye() {
    check_recipe("minecraft:brown_dye");
}

#[test]
fn recipe_brown_glazed_terracotta() {
    check_recipe("minecraft:brown_glazed_terracotta");
}

#[test]
fn recipe_brown_harness() {
    check_recipe("minecraft:brown_harness");
}

#[test]
fn recipe_brown_shulker_box() {
    check_recipe("minecraft:brown_shulker_box");
}

#[test]
fn recipe_brown_stained_glass() {
    check_recipe("minecraft:brown_stained_glass");
}

#[test]
fn recipe_brown_stained_glass_pane() {
    check_recipe("minecraft:brown_stained_glass_pane");
}

#[test]
fn recipe_brown_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:brown_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_brown_terracotta() {
    check_recipe("minecraft:brown_terracotta");
}

#[test]
fn recipe_brush() {
    check_recipe("minecraft:brush");
}

#[test]
fn recipe_bucket() {
    check_recipe("minecraft:bucket");
}

#[test]
fn recipe_bundle() {
    check_recipe("minecraft:bundle");
}

#[test]
fn recipe_cake() {
    check_recipe("minecraft:cake");
}

#[test]
fn recipe_calibrated_sculk_sensor() {
    check_recipe("minecraft:calibrated_sculk_sensor");
}

#[test]
fn recipe_campfire() {
    check_recipe("minecraft:campfire");
}

#[test]
fn recipe_candle() {
    check_recipe("minecraft:candle");
}

#[test]
fn recipe_carrot_on_a_stick() {
    check_recipe("minecraft:carrot_on_a_stick");
}

#[test]
fn recipe_cartography_table() {
    check_recipe("minecraft:cartography_table");
}

#[test]
fn recipe_cauldron() {
    check_recipe("minecraft:cauldron");
}

#[test]
fn recipe_chain() {
    check_recipe("minecraft:chain");
}

#[test]
fn recipe_charcoal() {
    check_recipe("minecraft:charcoal");
}

#[test]
fn recipe_cherry_boat() {
    check_recipe("minecraft:cherry_boat");
}

#[test]
fn recipe_cherry_button() {
    check_recipe("minecraft:cherry_button");
}

#[test]
fn recipe_cherry_chest_boat() {
    check_recipe("minecraft:cherry_chest_boat");
}

#[test]
fn recipe_cherry_door() {
    check_recipe("minecraft:cherry_door");
}

#[test]
fn recipe_cherry_fence() {
    check_recipe("minecraft:cherry_fence");
}

#[test]
fn recipe_cherry_fence_gate() {
    check_recipe("minecraft:cherry_fence_gate");
}

#[test]
fn recipe_cherry_hanging_sign() {
    check_recipe("minecraft:cherry_hanging_sign");
}

#[test]
fn recipe_cherry_planks() {
    check_recipe("minecraft:cherry_planks");
}

#[test]
fn recipe_cherry_pressure_plate() {
    check_recipe("minecraft:cherry_pressure_plate");
}

#[test]
fn recipe_cherry_sign() {
    check_recipe("minecraft:cherry_sign");
}

#[test]
fn recipe_cherry_slab() {
    check_recipe("minecraft:cherry_slab");
}

#[test]
fn recipe_cherry_stairs() {
    check_recipe("minecraft:cherry_stairs");
}

#[test]
fn recipe_cherry_trapdoor() {
    check_recipe("minecraft:cherry_trapdoor");
}

#[test]
fn recipe_cherry_wood() {
    check_recipe("minecraft:cherry_wood");
}

#[test]
fn recipe_chest() {
    check_recipe("minecraft:chest");
}

#[test]
fn recipe_chest_minecart() {
    check_recipe("minecraft:chest_minecart");
}

#[test]
fn recipe_chiseled_bookshelf() {
    check_recipe("minecraft:chiseled_bookshelf");
}

#[test]
fn recipe_chiseled_copper() {
    check_recipe("minecraft:chiseled_copper");
}

#[test]
fn recipe_chiseled_copper_from_copper_block_stonecutting() {
    check_recipe("minecraft:chiseled_copper_from_copper_block_stonecutting");
}

#[test]
fn recipe_chiseled_copper_from_cut_copper_stonecutting() {
    check_recipe("minecraft:chiseled_copper_from_cut_copper_stonecutting");
}

#[test]
fn recipe_chiseled_deepslate() {
    check_recipe("minecraft:chiseled_deepslate");
}

#[test]
fn recipe_chiseled_deepslate_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:chiseled_deepslate_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_chiseled_nether_bricks() {
    check_recipe("minecraft:chiseled_nether_bricks");
}

#[test]
fn recipe_chiseled_nether_bricks_from_nether_bricks_stonecutting() {
    check_recipe("minecraft:chiseled_nether_bricks_from_nether_bricks_stonecutting");
}

#[test]
fn recipe_chiseled_polished_blackstone() {
    check_recipe("minecraft:chiseled_polished_blackstone");
}

#[test]
fn recipe_chiseled_polished_blackstone_from_blackstone_stonecutting() {
    check_recipe("minecraft:chiseled_polished_blackstone_from_blackstone_stonecutting");
}

#[test]
fn recipe_chiseled_polished_blackstone_from_polished_blackstone_stonecutting() {
    check_recipe("minecraft:chiseled_polished_blackstone_from_polished_blackstone_stonecutting");
}

#[test]
fn recipe_chiseled_quartz_block() {
    check_recipe("minecraft:chiseled_quartz_block");
}

#[test]
fn recipe_chiseled_quartz_block_from_quartz_block_stonecutting() {
    check_recipe("minecraft:chiseled_quartz_block_from_quartz_block_stonecutting");
}

#[test]
fn recipe_chiseled_red_sandstone() {
    check_recipe("minecraft:chiseled_red_sandstone");
}

#[test]
fn recipe_chiseled_red_sandstone_from_red_sandstone_stonecutting() {
    check_recipe("minecraft:chiseled_red_sandstone_from_red_sandstone_stonecutting");
}

#[test]
fn recipe_chiseled_resin_bricks() {
    check_recipe("minecraft:chiseled_resin_bricks");
}

#[test]
fn recipe_chiseled_resin_bricks_from_resin_bricks_stonecutting() {
    check_recipe("minecraft:chiseled_resin_bricks_from_resin_bricks_stonecutting");
}

#[test]
fn recipe_chiseled_sandstone() {
    check_recipe("minecraft:chiseled_sandstone");
}

#[test]
fn recipe_chiseled_sandstone_from_sandstone_stonecutting() {
    check_recipe("minecraft:chiseled_sandstone_from_sandstone_stonecutting");
}

#[test]
fn recipe_chiseled_stone_bricks() {
    check_recipe("minecraft:chiseled_stone_bricks");
}

#[test]
fn recipe_chiseled_stone_bricks_from_stone_bricks_stonecutting() {
    check_recipe("minecraft:chiseled_stone_bricks_from_stone_bricks_stonecutting");
}

#[test]
fn recipe_chiseled_stone_bricks_stone_from_stonecutting() {
    check_recipe("minecraft:chiseled_stone_bricks_stone_from_stonecutting");
}

#[test]
fn recipe_chiseled_tuff() {
    check_recipe("minecraft:chiseled_tuff");
}

#[test]
fn recipe_chiseled_tuff_bricks() {
    check_recipe("minecraft:chiseled_tuff_bricks");
}

#[test]
fn recipe_chiseled_tuff_bricks_from_polished_tuff_stonecutting() {
    check_recipe("minecraft:chiseled_tuff_bricks_from_polished_tuff_stonecutting");
}

#[test]
fn recipe_chiseled_tuff_bricks_from_tuff_bricks_stonecutting() {
    check_recipe("minecraft:chiseled_tuff_bricks_from_tuff_bricks_stonecutting");
}

#[test]
fn recipe_chiseled_tuff_bricks_from_tuff_stonecutting() {
    check_recipe("minecraft:chiseled_tuff_bricks_from_tuff_stonecutting");
}

#[test]
fn recipe_chiseled_tuff_from_tuff_stonecutting() {
    check_recipe("minecraft:chiseled_tuff_from_tuff_stonecutting");
}

#[test]
fn recipe_clay() {
    check_recipe("minecraft:clay");
}

#[test]
fn recipe_clock() {
    check_recipe("minecraft:clock");
}

#[test]
fn recipe_coal() {
    check_recipe("minecraft:coal");
}

#[test]
fn recipe_coal_block() {
    check_recipe("minecraft:coal_block");
}

#[test]
fn recipe_coal_from_blasting_coal_ore() {
    check_recipe("minecraft:coal_from_blasting_coal_ore");
}

#[test]
fn recipe_coal_from_blasting_deepslate_coal_ore() {
    check_recipe("minecraft:coal_from_blasting_deepslate_coal_ore");
}

#[test]
fn recipe_coal_from_smelting_coal_ore() {
    check_recipe("minecraft:coal_from_smelting_coal_ore");
}

#[test]
fn recipe_coal_from_smelting_deepslate_coal_ore() {
    check_recipe("minecraft:coal_from_smelting_deepslate_coal_ore");
}

#[test]
fn recipe_coarse_dirt() {
    check_recipe("minecraft:coarse_dirt");
}

#[test]
fn recipe_coast_armor_trim_smithing_template() {
    check_recipe("minecraft:coast_armor_trim_smithing_template");
}

#[test]
fn recipe_coast_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:coast_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_cobbled_deepslate_slab() {
    check_recipe("minecraft:cobbled_deepslate_slab");
}

#[test]
fn recipe_cobbled_deepslate_slab_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:cobbled_deepslate_slab_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_cobbled_deepslate_stairs() {
    check_recipe("minecraft:cobbled_deepslate_stairs");
}

#[test]
fn recipe_cobbled_deepslate_stairs_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:cobbled_deepslate_stairs_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_cobbled_deepslate_wall() {
    check_recipe("minecraft:cobbled_deepslate_wall");
}

#[test]
fn recipe_cobbled_deepslate_wall_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:cobbled_deepslate_wall_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_cobblestone_slab() {
    check_recipe("minecraft:cobblestone_slab");
}

#[test]
fn recipe_cobblestone_slab_from_cobblestone_stonecutting() {
    check_recipe("minecraft:cobblestone_slab_from_cobblestone_stonecutting");
}

#[test]
fn recipe_cobblestone_stairs() {
    check_recipe("minecraft:cobblestone_stairs");
}

#[test]
fn recipe_cobblestone_stairs_from_cobblestone_stonecutting() {
    check_recipe("minecraft:cobblestone_stairs_from_cobblestone_stonecutting");
}

#[test]
fn recipe_cobblestone_wall() {
    check_recipe("minecraft:cobblestone_wall");
}

#[test]
fn recipe_cobblestone_wall_from_cobblestone_stonecutting() {
    check_recipe("minecraft:cobblestone_wall_from_cobblestone_stonecutting");
}

#[test]
fn recipe_comparator() {
    check_recipe("minecraft:comparator");
}

#[test]
fn recipe_compass() {
    check_recipe("minecraft:compass");
}

#[test]
fn recipe_composter() {
    check_recipe("minecraft:composter");
}

#[test]
fn recipe_conduit() {
    check_recipe("minecraft:conduit");
}

#[test]
fn recipe_cooked_beef() {
    check_recipe("minecraft:cooked_beef");
}

#[test]
fn recipe_cooked_beef_from_campfire_cooking() {
    check_recipe("minecraft:cooked_beef_from_campfire_cooking");
}

#[test]
fn recipe_cooked_beef_from_smoking() {
    check_recipe("minecraft:cooked_beef_from_smoking");
}

#[test]
fn recipe_cooked_chicken() {
    check_recipe("minecraft:cooked_chicken");
}

#[test]
fn recipe_cooked_chicken_from_campfire_cooking() {
    check_recipe("minecraft:cooked_chicken_from_campfire_cooking");
}

#[test]
fn recipe_cooked_chicken_from_smoking() {
    check_recipe("minecraft:cooked_chicken_from_smoking");
}

#[test]
fn recipe_cooked_cod() {
    check_recipe("minecraft:cooked_cod");
}

#[test]
fn recipe_cooked_cod_from_campfire_cooking() {
    check_recipe("minecraft:cooked_cod_from_campfire_cooking");
}

#[test]
fn recipe_cooked_cod_from_smoking() {
    check_recipe("minecraft:cooked_cod_from_smoking");
}

#[test]
fn recipe_cooked_mutton() {
    check_recipe("minecraft:cooked_mutton");
}

#[test]
fn recipe_cooked_mutton_from_campfire_cooking() {
    check_recipe("minecraft:cooked_mutton_from_campfire_cooking");
}

#[test]
fn recipe_cooked_mutton_from_smoking() {
    check_recipe("minecraft:cooked_mutton_from_smoking");
}

#[test]
fn recipe_cooked_porkchop() {
    check_recipe("minecraft:cooked_porkchop");
}

#[test]
fn recipe_cooked_porkchop_from_campfire_cooking() {
    check_recipe("minecraft:cooked_porkchop_from_campfire_cooking");
}

#[test]
fn recipe_cooked_porkchop_from_smoking() {
    check_recipe("minecraft:cooked_porkchop_from_smoking");
}

#[test]
fn recipe_cooked_rabbit() {
    check_recipe("minecraft:cooked_rabbit");
}

#[test]
fn recipe_cooked_rabbit_from_campfire_cooking() {
    check_recipe("minecraft:cooked_rabbit_from_campfire_cooking");
}

#[test]
fn recipe_cooked_rabbit_from_smoking() {
    check_recipe("minecraft:cooked_rabbit_from_smoking");
}

#[test]
fn recipe_cooked_salmon() {
    check_recipe("minecraft:cooked_salmon");
}

#[test]
fn recipe_cooked_salmon_from_campfire_cooking() {
    check_recipe("minecraft:cooked_salmon_from_campfire_cooking");
}

#[test]
fn recipe_cooked_salmon_from_smoking() {
    check_recipe("minecraft:cooked_salmon_from_smoking");
}

#[test]
fn recipe_cookie() {
    check_recipe("minecraft:cookie");
}

#[test]
fn recipe_copper_block() {
    check_recipe("minecraft:copper_block");
}

#[test]
fn recipe_copper_bulb() {
    check_recipe("minecraft:copper_bulb");
}

#[test]
fn recipe_copper_door() {
    check_recipe("minecraft:copper_door");
}

#[test]
fn recipe_copper_grate() {
    check_recipe("minecraft:copper_grate");
}

#[test]
fn recipe_copper_grate_from_copper_block_stonecutting() {
    check_recipe("minecraft:copper_grate_from_copper_block_stonecutting");
}

#[test]
fn recipe_copper_ingot() {
    check_recipe("minecraft:copper_ingot");
}

#[test]
fn recipe_copper_ingot_from_blasting_copper_ore() {
    check_recipe("minecraft:copper_ingot_from_blasting_copper_ore");
}

#[test]
fn recipe_copper_ingot_from_blasting_deepslate_copper_ore() {
    check_recipe("minecraft:copper_ingot_from_blasting_deepslate_copper_ore");
}

#[test]
fn recipe_copper_ingot_from_blasting_raw_copper() {
    check_recipe("minecraft:copper_ingot_from_blasting_raw_copper");
}

#[test]
fn recipe_copper_ingot_from_smelting_copper_ore() {
    check_recipe("minecraft:copper_ingot_from_smelting_copper_ore");
}

#[test]
fn recipe_copper_ingot_from_smelting_deepslate_copper_ore() {
    check_recipe("minecraft:copper_ingot_from_smelting_deepslate_copper_ore");
}

#[test]
fn recipe_copper_ingot_from_smelting_raw_copper() {
    check_recipe("minecraft:copper_ingot_from_smelting_raw_copper");
}

#[test]
fn recipe_copper_ingot_from_waxed_copper_block() {
    check_recipe("minecraft:copper_ingot_from_waxed_copper_block");
}

#[test]
fn recipe_copper_trapdoor() {
    check_recipe("minecraft:copper_trapdoor");
}

#[test]
fn recipe_cracked_deepslate_bricks() {
    check_recipe("minecraft:cracked_deepslate_bricks");
}

#[test]
fn recipe_cracked_deepslate_tiles() {
    check_recipe("minecraft:cracked_deepslate_tiles");
}

#[test]
fn recipe_cracked_nether_bricks() {
    check_recipe("minecraft:cracked_nether_bricks");
}

#[test]
fn recipe_cracked_polished_blackstone_bricks() {
    check_recipe("minecraft:cracked_polished_blackstone_bricks");
}

#[test]
fn recipe_cracked_stone_bricks() {
    check_recipe("minecraft:cracked_stone_bricks");
}

#[test]
fn recipe_crafter() {
    check_recipe("minecraft:crafter");
}

#[test]
fn recipe_crafting_table() {
    check_recipe("minecraft:crafting_table");
}

#[test]
fn recipe_creaking_heart() {
    check_recipe("minecraft:creaking_heart");
}

#[test]
fn recipe_creeper_banner_pattern() {
    check_recipe("minecraft:creeper_banner_pattern");
}

#[test]
fn recipe_crimson_button() {
    check_recipe("minecraft:crimson_button");
}

#[test]
fn recipe_crimson_door() {
    check_recipe("minecraft:crimson_door");
}

#[test]
fn recipe_crimson_fence() {
    check_recipe("minecraft:crimson_fence");
}

#[test]
fn recipe_crimson_fence_gate() {
    check_recipe("minecraft:crimson_fence_gate");
}

#[test]
fn recipe_crimson_hanging_sign() {
    check_recipe("minecraft:crimson_hanging_sign");
}

#[test]
fn recipe_crimson_hyphae() {
    check_recipe("minecraft:crimson_hyphae");
}

#[test]
fn recipe_crimson_planks() {
    check_recipe("minecraft:crimson_planks");
}

#[test]
fn recipe_crimson_pressure_plate() {
    check_recipe("minecraft:crimson_pressure_plate");
}

#[test]
fn recipe_crimson_sign() {
    check_recipe("minecraft:crimson_sign");
}

#[test]
fn recipe_crimson_slab() {
    check_recipe("minecraft:crimson_slab");
}

#[test]
fn recipe_crimson_stairs() {
    check_recipe("minecraft:crimson_stairs");
}

#[test]
fn recipe_crimson_trapdoor() {
    check_recipe("minecraft:crimson_trapdoor");
}

#[test]
fn recipe_crossbow() {
    check_recipe("minecraft:crossbow");
}

#[test]
fn recipe_cut_copper() {
    check_recipe("minecraft:cut_copper");
}

#[test]
fn recipe_cut_copper_from_copper_block_stonecutting() {
    check_recipe("minecraft:cut_copper_from_copper_block_stonecutting");
}

#[test]
fn recipe_cut_copper_slab() {
    check_recipe("minecraft:cut_copper_slab");
}

#[test]
fn recipe_cut_copper_slab_from_copper_block_stonecutting() {
    check_recipe("minecraft:cut_copper_slab_from_copper_block_stonecutting");
}

#[test]
fn recipe_cut_copper_slab_from_cut_copper_stonecutting() {
    check_recipe("minecraft:cut_copper_slab_from_cut_copper_stonecutting");
}

#[test]
fn recipe_cut_copper_stairs() {
    check_recipe("minecraft:cut_copper_stairs");
}

#[test]
fn recipe_cut_copper_stairs_from_copper_block_stonecutting() {
    check_recipe("minecraft:cut_copper_stairs_from_copper_block_stonecutting");
}

#[test]
fn recipe_cut_copper_stairs_from_cut_copper_stonecutting() {
    check_recipe("minecraft:cut_copper_stairs_from_cut_copper_stonecutting");
}

#[test]
fn recipe_cut_red_sandstone() {
    check_recipe("minecraft:cut_red_sandstone");
}

#[test]
fn recipe_cut_red_sandstone_from_red_sandstone_stonecutting() {
    check_recipe("minecraft:cut_red_sandstone_from_red_sandstone_stonecutting");
}

#[test]
fn recipe_cut_red_sandstone_slab() {
    check_recipe("minecraft:cut_red_sandstone_slab");
}

#[test]
fn recipe_cut_red_sandstone_slab_from_cut_red_sandstone_stonecutting() {
    check_recipe("minecraft:cut_red_sandstone_slab_from_cut_red_sandstone_stonecutting");
}

#[test]
fn recipe_cut_red_sandstone_slab_from_red_sandstone_stonecutting() {
    check_recipe("minecraft:cut_red_sandstone_slab_from_red_sandstone_stonecutting");
}

#[test]
fn recipe_cut_sandstone() {
    check_recipe("minecraft:cut_sandstone");
}

#[test]
fn recipe_cut_sandstone_from_sandstone_stonecutting() {
    check_recipe("minecraft:cut_sandstone_from_sandstone_stonecutting");
}

#[test]
fn recipe_cut_sandstone_slab() {
    check_recipe("minecraft:cut_sandstone_slab");
}

#[test]
fn recipe_cut_sandstone_slab_from_cut_sandstone_stonecutting() {
    check_recipe("minecraft:cut_sandstone_slab_from_cut_sandstone_stonecutting");
}

#[test]
fn recipe_cut_sandstone_slab_from_sandstone_stonecutting() {
    check_recipe("minecraft:cut_sandstone_slab_from_sandstone_stonecutting");
}

#[test]
fn recipe_cyan_banner() {
    check_recipe("minecraft:cyan_banner");
}

#[test]
fn recipe_cyan_bed() {
    check_recipe("minecraft:cyan_bed");
}

#[test]
fn recipe_cyan_bundle() {
    check_recipe("minecraft:cyan_bundle");
}

#[test]
fn recipe_cyan_candle() {
    check_recipe("minecraft:cyan_candle");
}

#[test]
fn recipe_cyan_carpet() {
    check_recipe("minecraft:cyan_carpet");
}

#[test]
fn recipe_cyan_concrete_powder() {
    check_recipe("minecraft:cyan_concrete_powder");
}

#[test]
fn recipe_cyan_dye() {
    check_recipe("minecraft:cyan_dye");
}

#[test]
fn recipe_cyan_dye_from_pitcher_plant() {
    check_recipe("minecraft:cyan_dye_from_pitcher_plant");
}

#[test]
fn recipe_cyan_glazed_terracotta() {
    check_recipe("minecraft:cyan_glazed_terracotta");
}

#[test]
fn recipe_cyan_harness() {
    check_recipe("minecraft:cyan_harness");
}

#[test]
fn recipe_cyan_shulker_box() {
    check_recipe("minecraft:cyan_shulker_box");
}

#[test]
fn recipe_cyan_stained_glass() {
    check_recipe("minecraft:cyan_stained_glass");
}

#[test]
fn recipe_cyan_stained_glass_pane() {
    check_recipe("minecraft:cyan_stained_glass_pane");
}

#[test]
fn recipe_cyan_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:cyan_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_cyan_terracotta() {
    check_recipe("minecraft:cyan_terracotta");
}

#[test]
fn recipe_dark_oak_boat() {
    check_recipe("minecraft:dark_oak_boat");
}

#[test]
fn recipe_dark_oak_button() {
    check_recipe("minecraft:dark_oak_button");
}

#[test]
fn recipe_dark_oak_chest_boat() {
    check_recipe("minecraft:dark_oak_chest_boat");
}

#[test]
fn recipe_dark_oak_door() {
    check_recipe("minecraft:dark_oak_door");
}

#[test]
fn recipe_dark_oak_fence() {
    check_recipe("minecraft:dark_oak_fence");
}

#[test]
fn recipe_dark_oak_fence_gate() {
    check_recipe("minecraft:dark_oak_fence_gate");
}

#[test]
fn recipe_dark_oak_hanging_sign() {
    check_recipe("minecraft:dark_oak_hanging_sign");
}

#[test]
fn recipe_dark_oak_planks() {
    check_recipe("minecraft:dark_oak_planks");
}

#[test]
fn recipe_dark_oak_pressure_plate() {
    check_recipe("minecraft:dark_oak_pressure_plate");
}

#[test]
fn recipe_dark_oak_sign() {
    check_recipe("minecraft:dark_oak_sign");
}

#[test]
fn recipe_dark_oak_slab() {
    check_recipe("minecraft:dark_oak_slab");
}

#[test]
fn recipe_dark_oak_stairs() {
    check_recipe("minecraft:dark_oak_stairs");
}

#[test]
fn recipe_dark_oak_trapdoor() {
    check_recipe("minecraft:dark_oak_trapdoor");
}

#[test]
fn recipe_dark_oak_wood() {
    check_recipe("minecraft:dark_oak_wood");
}

#[test]
fn recipe_dark_prismarine() {
    check_recipe("minecraft:dark_prismarine");
}

#[test]
fn recipe_dark_prismarine_slab() {
    check_recipe("minecraft:dark_prismarine_slab");
}

#[test]
fn recipe_dark_prismarine_slab_from_dark_prismarine_stonecutting() {
    check_recipe("minecraft:dark_prismarine_slab_from_dark_prismarine_stonecutting");
}

#[test]
fn recipe_dark_prismarine_stairs() {
    check_recipe("minecraft:dark_prismarine_stairs");
}

#[test]
fn recipe_dark_prismarine_stairs_from_dark_prismarine_stonecutting() {
    check_recipe("minecraft:dark_prismarine_stairs_from_dark_prismarine_stonecutting");
}

#[test]
fn recipe_daylight_detector() {
    check_recipe("minecraft:daylight_detector");
}

#[test]
fn recipe_decorated_pot() {
    check_recipe("minecraft:decorated_pot");
}

#[test]
fn recipe_decorated_pot_simple() {
    check_recipe("minecraft:decorated_pot_simple");
}

#[test]
fn recipe_deepslate() {
    check_recipe("minecraft:deepslate");
}

#[test]
fn recipe_deepslate_brick_slab() {
    check_recipe("minecraft:deepslate_brick_slab");
}

#[test]
fn recipe_deepslate_brick_slab_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_brick_slab_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_brick_slab_from_deepslate_bricks_stonecutting() {
    check_recipe("minecraft:deepslate_brick_slab_from_deepslate_bricks_stonecutting");
}

#[test]
fn recipe_deepslate_brick_slab_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_brick_slab_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_brick_stairs() {
    check_recipe("minecraft:deepslate_brick_stairs");
}

#[test]
fn recipe_deepslate_brick_stairs_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_brick_stairs_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_brick_stairs_from_deepslate_bricks_stonecutting() {
    check_recipe("minecraft:deepslate_brick_stairs_from_deepslate_bricks_stonecutting");
}

#[test]
fn recipe_deepslate_brick_stairs_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_brick_stairs_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_brick_wall() {
    check_recipe("minecraft:deepslate_brick_wall");
}

#[test]
fn recipe_deepslate_brick_wall_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_brick_wall_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_brick_wall_from_deepslate_bricks_stonecutting() {
    check_recipe("minecraft:deepslate_brick_wall_from_deepslate_bricks_stonecutting");
}

#[test]
fn recipe_deepslate_brick_wall_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_brick_wall_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_bricks() {
    check_recipe("minecraft:deepslate_bricks");
}

#[test]
fn recipe_deepslate_bricks_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_bricks_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_bricks_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_bricks_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_tile_slab() {
    check_recipe("minecraft:deepslate_tile_slab");
}

#[test]
fn recipe_deepslate_tile_slab_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_tile_slab_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_tile_slab_from_deepslate_bricks_stonecutting() {
    check_recipe("minecraft:deepslate_tile_slab_from_deepslate_bricks_stonecutting");
}

#[test]
fn recipe_deepslate_tile_slab_from_deepslate_tiles_stonecutting() {
    check_recipe("minecraft:deepslate_tile_slab_from_deepslate_tiles_stonecutting");
}

#[test]
fn recipe_deepslate_tile_slab_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_tile_slab_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_tile_stairs() {
    check_recipe("minecraft:deepslate_tile_stairs");
}

#[test]
fn recipe_deepslate_tile_stairs_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_tile_stairs_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_tile_stairs_from_deepslate_bricks_stonecutting() {
    check_recipe("minecraft:deepslate_tile_stairs_from_deepslate_bricks_stonecutting");
}

#[test]
fn recipe_deepslate_tile_stairs_from_deepslate_tiles_stonecutting() {
    check_recipe("minecraft:deepslate_tile_stairs_from_deepslate_tiles_stonecutting");
}

#[test]
fn recipe_deepslate_tile_stairs_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_tile_stairs_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_tile_wall() {
    check_recipe("minecraft:deepslate_tile_wall");
}

#[test]
fn recipe_deepslate_tile_wall_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_tile_wall_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_tile_wall_from_deepslate_bricks_stonecutting() {
    check_recipe("minecraft:deepslate_tile_wall_from_deepslate_bricks_stonecutting");
}

#[test]
fn recipe_deepslate_tile_wall_from_deepslate_tiles_stonecutting() {
    check_recipe("minecraft:deepslate_tile_wall_from_deepslate_tiles_stonecutting");
}

#[test]
fn recipe_deepslate_tile_wall_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_tile_wall_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_tiles() {
    check_recipe("minecraft:deepslate_tiles");
}

#[test]
fn recipe_deepslate_tiles_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_tiles_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_deepslate_tiles_from_deepslate_bricks_stonecutting() {
    check_recipe("minecraft:deepslate_tiles_from_deepslate_bricks_stonecutting");
}

#[test]
fn recipe_deepslate_tiles_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:deepslate_tiles_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_detector_rail() {
    check_recipe("minecraft:detector_rail");
}

#[test]
fn recipe_diamond() {
    check_recipe("minecraft:diamond");
}

#[test]
fn recipe_diamond_axe() {
    check_recipe("minecraft:diamond_axe");
}

#[test]
fn recipe_diamond_block() {
    check_recipe("minecraft:diamond_block");
}

#[test]
fn recipe_diamond_boots() {
    check_recipe("minecraft:diamond_boots");
}

#[test]
fn recipe_diamond_chestplate() {
    check_recipe("minecraft:diamond_chestplate");
}

#[test]
fn recipe_diamond_from_blasting_deepslate_diamond_ore() {
    check_recipe("minecraft:diamond_from_blasting_deepslate_diamond_ore");
}

#[test]
fn recipe_diamond_from_blasting_diamond_ore() {
    check_recipe("minecraft:diamond_from_blasting_diamond_ore");
}

#[test]
fn recipe_diamond_from_smelting_deepslate_diamond_ore() {
    check_recipe("minecraft:diamond_from_smelting_deepslate_diamond_ore");
}

#[test]
fn recipe_diamond_from_smelting_diamond_ore() {
    check_recipe("minecraft:diamond_from_smelting_diamond_ore");
}

#[test]
fn recipe_diamond_helmet() {
    check_recipe("minecraft:diamond_helmet");
}

#[test]
fn recipe_diamond_hoe() {
    check_recipe("minecraft:diamond_hoe");
}

#[test]
fn recipe_diamond_leggings() {
    check_recipe("minecraft:diamond_leggings");
}

#[test]
fn recipe_diamond_pickaxe() {
    check_recipe("minecraft:diamond_pickaxe");
}

#[test]
fn recipe_diamond_shovel() {
    check_recipe("minecraft:diamond_shovel");
}

#[test]
fn recipe_diamond_sword() {
    check_recipe("minecraft:diamond_sword");
}

#[test]
fn recipe_diorite() {
    check_recipe("minecraft:diorite");
}

#[test]
fn recipe_diorite_slab() {
    check_recipe("minecraft:diorite_slab");
}

#[test]
fn recipe_diorite_slab_from_diorite_stonecutting() {
    check_recipe("minecraft:diorite_slab_from_diorite_stonecutting");
}

#[test]
fn recipe_diorite_stairs() {
    check_recipe("minecraft:diorite_stairs");
}

#[test]
fn recipe_diorite_stairs_from_diorite_stonecutting() {
    check_recipe("minecraft:diorite_stairs_from_diorite_stonecutting");
}

#[test]
fn recipe_diorite_wall() {
    check_recipe("minecraft:diorite_wall");
}

#[test]
fn recipe_diorite_wall_from_diorite_stonecutting() {
    check_recipe("minecraft:diorite_wall_from_diorite_stonecutting");
}

#[test]
fn recipe_dispenser() {
    check_recipe("minecraft:dispenser");
}

#[test]
fn recipe_dried_ghast() {
    check_recipe("minecraft:dried_ghast");
}

#[test]
fn recipe_dried_kelp() {
    check_recipe("minecraft:dried_kelp");
}

#[test]
fn recipe_dried_kelp_block() {
    check_recipe("minecraft:dried_kelp_block");
}

#[test]
fn recipe_dried_kelp_from_campfire_cooking() {
    check_recipe("minecraft:dried_kelp_from_campfire_cooking");
}

#[test]
fn recipe_dried_kelp_from_smelting() {
    check_recipe("minecraft:dried_kelp_from_smelting");
}

#[test]
fn recipe_dried_kelp_from_smoking() {
    check_recipe("minecraft:dried_kelp_from_smoking");
}

#[test]
fn recipe_dripstone_block() {
    check_recipe("minecraft:dripstone_block");
}

#[test]
fn recipe_dropper() {
    check_recipe("minecraft:dropper");
}

#[test]
fn recipe_dune_armor_trim_smithing_template() {
    check_recipe("minecraft:dune_armor_trim_smithing_template");
}

#[test]
fn recipe_dune_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:dune_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_dye_black_bed() {
    check_recipe("minecraft:dye_black_bed");
}

#[test]
fn recipe_dye_black_carpet() {
    check_recipe("minecraft:dye_black_carpet");
}

#[test]
fn recipe_dye_black_harness() {
    check_recipe("minecraft:dye_black_harness");
}

#[test]
fn recipe_dye_black_wool() {
    check_recipe("minecraft:dye_black_wool");
}

#[test]
fn recipe_dye_blue_bed() {
    check_recipe("minecraft:dye_blue_bed");
}

#[test]
fn recipe_dye_blue_carpet() {
    check_recipe("minecraft:dye_blue_carpet");
}

#[test]
fn recipe_dye_blue_harness() {
    check_recipe("minecraft:dye_blue_harness");
}

#[test]
fn recipe_dye_blue_wool() {
    check_recipe("minecraft:dye_blue_wool");
}

#[test]
fn recipe_dye_brown_bed() {
    check_recipe("minecraft:dye_brown_bed");
}

#[test]
fn recipe_dye_brown_carpet() {
    check_recipe("minecraft:dye_brown_carpet");
}

#[test]
fn recipe_dye_brown_harness() {
    check_recipe("minecraft:dye_brown_harness");
}

#[test]
fn recipe_dye_brown_wool() {
    check_recipe("minecraft:dye_brown_wool");
}

#[test]
fn recipe_dye_cyan_bed() {
    check_recipe("minecraft:dye_cyan_bed");
}

#[test]
fn recipe_dye_cyan_carpet() {
    check_recipe("minecraft:dye_cyan_carpet");
}

#[test]
fn recipe_dye_cyan_harness() {
    check_recipe("minecraft:dye_cyan_harness");
}

#[test]
fn recipe_dye_cyan_wool() {
    check_recipe("minecraft:dye_cyan_wool");
}

#[test]
fn recipe_dye_gray_bed() {
    check_recipe("minecraft:dye_gray_bed");
}

#[test]
fn recipe_dye_gray_carpet() {
    check_recipe("minecraft:dye_gray_carpet");
}

#[test]
fn recipe_dye_gray_harness() {
    check_recipe("minecraft:dye_gray_harness");
}

#[test]
fn recipe_dye_gray_wool() {
    check_recipe("minecraft:dye_gray_wool");
}

#[test]
fn recipe_dye_green_bed() {
    check_recipe("minecraft:dye_green_bed");
}

#[test]
fn recipe_dye_green_carpet() {
    check_recipe("minecraft:dye_green_carpet");
}

#[test]
fn recipe_dye_green_harness() {
    check_recipe("minecraft:dye_green_harness");
}

#[test]
fn recipe_dye_green_wool() {
    check_recipe("minecraft:dye_green_wool");
}

#[test]
fn recipe_dye_light_blue_bed() {
    check_recipe("minecraft:dye_light_blue_bed");
}

#[test]
fn recipe_dye_light_blue_carpet() {
    check_recipe("minecraft:dye_light_blue_carpet");
}

#[test]
fn recipe_dye_light_blue_harness() {
    check_recipe("minecraft:dye_light_blue_harness");
}

#[test]
fn recipe_dye_light_blue_wool() {
    check_recipe("minecraft:dye_light_blue_wool");
}

#[test]
fn recipe_dye_light_gray_bed() {
    check_recipe("minecraft:dye_light_gray_bed");
}

#[test]
fn recipe_dye_light_gray_carpet() {
    check_recipe("minecraft:dye_light_gray_carpet");
}

#[test]
fn recipe_dye_light_gray_harness() {
    check_recipe("minecraft:dye_light_gray_harness");
}

#[test]
fn recipe_dye_light_gray_wool() {
    check_recipe("minecraft:dye_light_gray_wool");
}

#[test]
fn recipe_dye_lime_bed() {
    check_recipe("minecraft:dye_lime_bed");
}

#[test]
fn recipe_dye_lime_carpet() {
    check_recipe("minecraft:dye_lime_carpet");
}

#[test]
fn recipe_dye_lime_harness() {
    check_recipe("minecraft:dye_lime_harness");
}

#[test]
fn recipe_dye_lime_wool() {
    check_recipe("minecraft:dye_lime_wool");
}

#[test]
fn recipe_dye_magenta_bed() {
    check_recipe("minecraft:dye_magenta_bed");
}

#[test]
fn recipe_dye_magenta_carpet() {
    check_recipe("minecraft:dye_magenta_carpet");
}

#[test]
fn recipe_dye_magenta_harness() {
    check_recipe("minecraft:dye_magenta_harness");
}

#[test]
fn recipe_dye_magenta_wool() {
    check_recipe("minecraft:dye_magenta_wool");
}

#[test]
fn recipe_dye_orange_bed() {
    check_recipe("minecraft:dye_orange_bed");
}

#[test]
fn recipe_dye_orange_carpet() {
    check_recipe("minecraft:dye_orange_carpet");
}

#[test]
fn recipe_dye_orange_harness() {
    check_recipe("minecraft:dye_orange_harness");
}

#[test]
fn recipe_dye_orange_wool() {
    check_recipe("minecraft:dye_orange_wool");
}

#[test]
fn recipe_dye_pink_bed() {
    check_recipe("minecraft:dye_pink_bed");
}

#[test]
fn recipe_dye_pink_carpet() {
    check_recipe("minecraft:dye_pink_carpet");
}

#[test]
fn recipe_dye_pink_harness() {
    check_recipe("minecraft:dye_pink_harness");
}

#[test]
fn recipe_dye_pink_wool() {
    check_recipe("minecraft:dye_pink_wool");
}

#[test]
fn recipe_dye_purple_bed() {
    check_recipe("minecraft:dye_purple_bed");
}

#[test]
fn recipe_dye_purple_carpet() {
    check_recipe("minecraft:dye_purple_carpet");
}

#[test]
fn recipe_dye_purple_harness() {
    check_recipe("minecraft:dye_purple_harness");
}

#[test]
fn recipe_dye_purple_wool() {
    check_recipe("minecraft:dye_purple_wool");
}

#[test]
fn recipe_dye_red_bed() {
    check_recipe("minecraft:dye_red_bed");
}

#[test]
fn recipe_dye_red_carpet() {
    check_recipe("minecraft:dye_red_carpet");
}

#[test]
fn recipe_dye_red_harness() {
    check_recipe("minecraft:dye_red_harness");
}

#[test]
fn recipe_dye_red_wool() {
    check_recipe("minecraft:dye_red_wool");
}

#[test]
fn recipe_dye_white_bed() {
    check_recipe("minecraft:dye_white_bed");
}

#[test]
fn recipe_dye_white_carpet() {
    check_recipe("minecraft:dye_white_carpet");
}

#[test]
fn recipe_dye_white_harness() {
    check_recipe("minecraft:dye_white_harness");
}

#[test]
fn recipe_dye_white_wool() {
    check_recipe("minecraft:dye_white_wool");
}

#[test]
fn recipe_dye_yellow_bed() {
    check_recipe("minecraft:dye_yellow_bed");
}

#[test]
fn recipe_dye_yellow_carpet() {
    check_recipe("minecraft:dye_yellow_carpet");
}

#[test]
fn recipe_dye_yellow_harness() {
    check_recipe("minecraft:dye_yellow_harness");
}

#[test]
fn recipe_dye_yellow_wool() {
    check_recipe("minecraft:dye_yellow_wool");
}

#[test]
fn recipe_emerald() {
    check_recipe("minecraft:emerald");
}

#[test]
fn recipe_emerald_block() {
    check_recipe("minecraft:emerald_block");
}

#[test]
fn recipe_emerald_from_blasting_deepslate_emerald_ore() {
    check_recipe("minecraft:emerald_from_blasting_deepslate_emerald_ore");
}

#[test]
fn recipe_emerald_from_blasting_emerald_ore() {
    check_recipe("minecraft:emerald_from_blasting_emerald_ore");
}

#[test]
fn recipe_emerald_from_smelting_deepslate_emerald_ore() {
    check_recipe("minecraft:emerald_from_smelting_deepslate_emerald_ore");
}

#[test]
fn recipe_emerald_from_smelting_emerald_ore() {
    check_recipe("minecraft:emerald_from_smelting_emerald_ore");
}

#[test]
fn recipe_enchanting_table() {
    check_recipe("minecraft:enchanting_table");
}

#[test]
fn recipe_end_crystal() {
    check_recipe("minecraft:end_crystal");
}

#[test]
fn recipe_end_rod() {
    check_recipe("minecraft:end_rod");
}

#[test]
fn recipe_end_stone_brick_slab() {
    check_recipe("minecraft:end_stone_brick_slab");
}

#[test]
fn recipe_end_stone_brick_slab_from_end_stone_brick_stonecutting() {
    check_recipe("minecraft:end_stone_brick_slab_from_end_stone_brick_stonecutting");
}

#[test]
fn recipe_end_stone_brick_slab_from_end_stone_stonecutting() {
    check_recipe("minecraft:end_stone_brick_slab_from_end_stone_stonecutting");
}

#[test]
fn recipe_end_stone_brick_stairs() {
    check_recipe("minecraft:end_stone_brick_stairs");
}

#[test]
fn recipe_end_stone_brick_stairs_from_end_stone_brick_stonecutting() {
    check_recipe("minecraft:end_stone_brick_stairs_from_end_stone_brick_stonecutting");
}

#[test]
fn recipe_end_stone_brick_stairs_from_end_stone_stonecutting() {
    check_recipe("minecraft:end_stone_brick_stairs_from_end_stone_stonecutting");
}

#[test]
fn recipe_end_stone_brick_wall() {
    check_recipe("minecraft:end_stone_brick_wall");
}

#[test]
fn recipe_end_stone_brick_wall_from_end_stone_brick_stonecutting() {
    check_recipe("minecraft:end_stone_brick_wall_from_end_stone_brick_stonecutting");
}

#[test]
fn recipe_end_stone_brick_wall_from_end_stone_stonecutting() {
    check_recipe("minecraft:end_stone_brick_wall_from_end_stone_stonecutting");
}

#[test]
fn recipe_end_stone_bricks() {
    check_recipe("minecraft:end_stone_bricks");
}

#[test]
fn recipe_end_stone_bricks_from_end_stone_stonecutting() {
    check_recipe("minecraft:end_stone_bricks_from_end_stone_stonecutting");
}

#[test]
fn recipe_ender_chest() {
    check_recipe("minecraft:ender_chest");
}

#[test]
fn recipe_ender_eye() {
    check_recipe("minecraft:ender_eye");
}

#[test]
fn recipe_exposed_chiseled_copper() {
    check_recipe("minecraft:exposed_chiseled_copper");
}

#[test]
fn recipe_exposed_chiseled_copper_from_exposed_copper_stonecutting() {
    check_recipe("minecraft:exposed_chiseled_copper_from_exposed_copper_stonecutting");
}

#[test]
fn recipe_exposed_chiseled_copper_from_exposed_cut_copper_stonecutting() {
    check_recipe("minecraft:exposed_chiseled_copper_from_exposed_cut_copper_stonecutting");
}

#[test]
fn recipe_exposed_copper_bulb() {
    check_recipe("minecraft:exposed_copper_bulb");
}

#[test]
fn recipe_exposed_copper_grate() {
    check_recipe("minecraft:exposed_copper_grate");
}

#[test]
fn recipe_exposed_copper_grate_from_exposed_copper_stonecutting() {
    check_recipe("minecraft:exposed_copper_grate_from_exposed_copper_stonecutting");
}

#[test]
fn recipe_exposed_cut_copper() {
    check_recipe("minecraft:exposed_cut_copper");
}

#[test]
fn recipe_exposed_cut_copper_from_exposed_copper_stonecutting() {
    check_recipe("minecraft:exposed_cut_copper_from_exposed_copper_stonecutting");
}

#[test]
fn recipe_exposed_cut_copper_slab() {
    check_recipe("minecraft:exposed_cut_copper_slab");
}

#[test]
fn recipe_exposed_cut_copper_slab_from_exposed_copper_stonecutting() {
    check_recipe("minecraft:exposed_cut_copper_slab_from_exposed_copper_stonecutting");
}

#[test]
fn recipe_exposed_cut_copper_slab_from_exposed_cut_copper_stonecutting() {
    check_recipe("minecraft:exposed_cut_copper_slab_from_exposed_cut_copper_stonecutting");
}

#[test]
fn recipe_exposed_cut_copper_stairs() {
    check_recipe("minecraft:exposed_cut_copper_stairs");
}

#[test]
fn recipe_exposed_cut_copper_stairs_from_exposed_copper_stonecutting() {
    check_recipe("minecraft:exposed_cut_copper_stairs_from_exposed_copper_stonecutting");
}

#[test]
fn recipe_exposed_cut_copper_stairs_from_exposed_cut_copper_stonecutting() {
    check_recipe("minecraft:exposed_cut_copper_stairs_from_exposed_cut_copper_stonecutting");
}

#[test]
fn recipe_eye_armor_trim_smithing_template() {
    check_recipe("minecraft:eye_armor_trim_smithing_template");
}

#[test]
fn recipe_eye_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:eye_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_fermented_spider_eye() {
    check_recipe("minecraft:fermented_spider_eye");
}

#[test]
fn recipe_field_masoned_banner_pattern() {
    check_recipe("minecraft:field_masoned_banner_pattern");
}

#[test]
fn recipe_fire_charge() {
    check_recipe("minecraft:fire_charge");
}

#[test]
fn recipe_firework_rocket() {
    check_recipe("minecraft:firework_rocket");
}

#[test]
fn recipe_firework_rocket_simple() {
    check_recipe("minecraft:firework_rocket_simple");
}

#[test]
fn recipe_firework_star() {
    check_recipe("minecraft:firework_star");
}

#[test]
fn recipe_firework_star_fade() {
    check_recipe("minecraft:firework_star_fade");
}

#[test]
fn recipe_fishing_rod() {
    check_recipe("minecraft:fishing_rod");
}

#[test]
fn recipe_fletching_table() {
    check_recipe("minecraft:fletching_table");
}

#[test]
fn recipe_flint_and_steel() {
    check_recipe("minecraft:flint_and_steel");
}

#[test]
fn recipe_flow_armor_trim_smithing_template() {
    check_recipe("minecraft:flow_armor_trim_smithing_template");
}

#[test]
fn recipe_flow_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:flow_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_flower_banner_pattern() {
    check_recipe("minecraft:flower_banner_pattern");
}

#[test]
fn recipe_flower_pot() {
    check_recipe("minecraft:flower_pot");
}

#[test]
fn recipe_furnace() {
    check_recipe("minecraft:furnace");
}

#[test]
fn recipe_furnace_minecart() {
    check_recipe("minecraft:furnace_minecart");
}

#[test]
fn recipe_glass() {
    check_recipe("minecraft:glass");
}

#[test]
fn recipe_glass_bottle() {
    check_recipe("minecraft:glass_bottle");
}

#[test]
fn recipe_glass_pane() {
    check_recipe("minecraft:glass_pane");
}

#[test]
fn recipe_glistering_melon_slice() {
    check_recipe("minecraft:glistering_melon_slice");
}

#[test]
fn recipe_glow_item_frame() {
    check_recipe("minecraft:glow_item_frame");
}

#[test]
fn recipe_glowstone() {
    check_recipe("minecraft:glowstone");
}

#[test]
fn recipe_gold_block() {
    check_recipe("minecraft:gold_block");
}

#[test]
fn recipe_gold_ingot_from_blasting_deepslate_gold_ore() {
    check_recipe("minecraft:gold_ingot_from_blasting_deepslate_gold_ore");
}

#[test]
fn recipe_gold_ingot_from_blasting_gold_ore() {
    check_recipe("minecraft:gold_ingot_from_blasting_gold_ore");
}

#[test]
fn recipe_gold_ingot_from_blasting_nether_gold_ore() {
    check_recipe("minecraft:gold_ingot_from_blasting_nether_gold_ore");
}

#[test]
fn recipe_gold_ingot_from_blasting_raw_gold() {
    check_recipe("minecraft:gold_ingot_from_blasting_raw_gold");
}

#[test]
fn recipe_gold_ingot_from_gold_block() {
    check_recipe("minecraft:gold_ingot_from_gold_block");
}

#[test]
fn recipe_gold_ingot_from_nuggets() {
    check_recipe("minecraft:gold_ingot_from_nuggets");
}

#[test]
fn recipe_gold_ingot_from_smelting_deepslate_gold_ore() {
    check_recipe("minecraft:gold_ingot_from_smelting_deepslate_gold_ore");
}

#[test]
fn recipe_gold_ingot_from_smelting_gold_ore() {
    check_recipe("minecraft:gold_ingot_from_smelting_gold_ore");
}

#[test]
fn recipe_gold_ingot_from_smelting_nether_gold_ore() {
    check_recipe("minecraft:gold_ingot_from_smelting_nether_gold_ore");
}

#[test]
fn recipe_gold_ingot_from_smelting_raw_gold() {
    check_recipe("minecraft:gold_ingot_from_smelting_raw_gold");
}

#[test]
fn recipe_gold_nugget() {
    check_recipe("minecraft:gold_nugget");
}

#[test]
fn recipe_gold_nugget_from_blasting() {
    check_recipe("minecraft:gold_nugget_from_blasting");
}

#[test]
fn recipe_gold_nugget_from_smelting() {
    check_recipe("minecraft:gold_nugget_from_smelting");
}

#[test]
fn recipe_golden_apple() {
    check_recipe("minecraft:golden_apple");
}

#[test]
fn recipe_golden_axe() {
    check_recipe("minecraft:golden_axe");
}

#[test]
fn recipe_golden_boots() {
    check_recipe("minecraft:golden_boots");
}

#[test]
fn recipe_golden_carrot() {
    check_recipe("minecraft:golden_carrot");
}

#[test]
fn recipe_golden_chestplate() {
    check_recipe("minecraft:golden_chestplate");
}

#[test]
fn recipe_golden_helmet() {
    check_recipe("minecraft:golden_helmet");
}

#[test]
fn recipe_golden_hoe() {
    check_recipe("minecraft:golden_hoe");
}

#[test]
fn recipe_golden_leggings() {
    check_recipe("minecraft:golden_leggings");
}

#[test]
fn recipe_golden_pickaxe() {
    check_recipe("minecraft:golden_pickaxe");
}

#[test]
fn recipe_golden_shovel() {
    check_recipe("minecraft:golden_shovel");
}

#[test]
fn recipe_golden_sword() {
    check_recipe("minecraft:golden_sword");
}

#[test]
fn recipe_granite() {
    check_recipe("minecraft:granite");
}

#[test]
fn recipe_granite_slab() {
    check_recipe("minecraft:granite_slab");
}

#[test]
fn recipe_granite_slab_from_granite_stonecutting() {
    check_recipe("minecraft:granite_slab_from_granite_stonecutting");
}

#[test]
fn recipe_granite_stairs() {
    check_recipe("minecraft:granite_stairs");
}

#[test]
fn recipe_granite_stairs_from_granite_stonecutting() {
    check_recipe("minecraft:granite_stairs_from_granite_stonecutting");
}

#[test]
fn recipe_granite_wall() {
    check_recipe("minecraft:granite_wall");
}

#[test]
fn recipe_granite_wall_from_granite_stonecutting() {
    check_recipe("minecraft:granite_wall_from_granite_stonecutting");
}

#[test]
fn recipe_gray_banner() {
    check_recipe("minecraft:gray_banner");
}

#[test]
fn recipe_gray_bed() {
    check_recipe("minecraft:gray_bed");
}

#[test]
fn recipe_gray_bundle() {
    check_recipe("minecraft:gray_bundle");
}

#[test]
fn recipe_gray_candle() {
    check_recipe("minecraft:gray_candle");
}

#[test]
fn recipe_gray_carpet() {
    check_recipe("minecraft:gray_carpet");
}

#[test]
fn recipe_gray_concrete_powder() {
    check_recipe("minecraft:gray_concrete_powder");
}

#[test]
fn recipe_gray_dye() {
    check_recipe("minecraft:gray_dye");
}

#[test]
fn recipe_gray_dye_from_closed_eyeblossom() {
    check_recipe("minecraft:gray_dye_from_closed_eyeblossom");
}

#[test]
fn recipe_gray_glazed_terracotta() {
    check_recipe("minecraft:gray_glazed_terracotta");
}

#[test]
fn recipe_gray_harness() {
    check_recipe("minecraft:gray_harness");
}

#[test]
fn recipe_gray_shulker_box() {
    check_recipe("minecraft:gray_shulker_box");
}

#[test]
fn recipe_gray_stained_glass() {
    check_recipe("minecraft:gray_stained_glass");
}

#[test]
fn recipe_gray_stained_glass_pane() {
    check_recipe("minecraft:gray_stained_glass_pane");
}

#[test]
fn recipe_gray_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:gray_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_gray_terracotta() {
    check_recipe("minecraft:gray_terracotta");
}

#[test]
fn recipe_green_banner() {
    check_recipe("minecraft:green_banner");
}

#[test]
fn recipe_green_bed() {
    check_recipe("minecraft:green_bed");
}

#[test]
fn recipe_green_bundle() {
    check_recipe("minecraft:green_bundle");
}

#[test]
fn recipe_green_candle() {
    check_recipe("minecraft:green_candle");
}

#[test]
fn recipe_green_carpet() {
    check_recipe("minecraft:green_carpet");
}

#[test]
fn recipe_green_concrete_powder() {
    check_recipe("minecraft:green_concrete_powder");
}

#[test]
fn recipe_green_dye() {
    check_recipe("minecraft:green_dye");
}

#[test]
fn recipe_green_glazed_terracotta() {
    check_recipe("minecraft:green_glazed_terracotta");
}

#[test]
fn recipe_green_harness() {
    check_recipe("minecraft:green_harness");
}

#[test]
fn recipe_green_shulker_box() {
    check_recipe("minecraft:green_shulker_box");
}

#[test]
fn recipe_green_stained_glass() {
    check_recipe("minecraft:green_stained_glass");
}

#[test]
fn recipe_green_stained_glass_pane() {
    check_recipe("minecraft:green_stained_glass_pane");
}

#[test]
fn recipe_green_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:green_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_green_terracotta() {
    check_recipe("minecraft:green_terracotta");
}

#[test]
fn recipe_grindstone() {
    check_recipe("minecraft:grindstone");
}

#[test]
fn recipe_hay_block() {
    check_recipe("minecraft:hay_block");
}

#[test]
fn recipe_heavy_weighted_pressure_plate() {
    check_recipe("minecraft:heavy_weighted_pressure_plate");
}

#[test]
fn recipe_honey_block() {
    check_recipe("minecraft:honey_block");
}

#[test]
fn recipe_honey_bottle() {
    check_recipe("minecraft:honey_bottle");
}

#[test]
fn recipe_honeycomb_block() {
    check_recipe("minecraft:honeycomb_block");
}

#[test]
fn recipe_hopper() {
    check_recipe("minecraft:hopper");
}

#[test]
fn recipe_hopper_minecart() {
    check_recipe("minecraft:hopper_minecart");
}

#[test]
fn recipe_host_armor_trim_smithing_template() {
    check_recipe("minecraft:host_armor_trim_smithing_template");
}

#[test]
fn recipe_host_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:host_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_iron_axe() {
    check_recipe("minecraft:iron_axe");
}

#[test]
fn recipe_iron_bars() {
    check_recipe("minecraft:iron_bars");
}

#[test]
fn recipe_iron_block() {
    check_recipe("minecraft:iron_block");
}

#[test]
fn recipe_iron_boots() {
    check_recipe("minecraft:iron_boots");
}

#[test]
fn recipe_iron_chestplate() {
    check_recipe("minecraft:iron_chestplate");
}

#[test]
fn recipe_iron_door() {
    check_recipe("minecraft:iron_door");
}

#[test]
fn recipe_iron_helmet() {
    check_recipe("minecraft:iron_helmet");
}

#[test]
fn recipe_iron_hoe() {
    check_recipe("minecraft:iron_hoe");
}

#[test]
fn recipe_iron_ingot_from_blasting_deepslate_iron_ore() {
    check_recipe("minecraft:iron_ingot_from_blasting_deepslate_iron_ore");
}

#[test]
fn recipe_iron_ingot_from_blasting_iron_ore() {
    check_recipe("minecraft:iron_ingot_from_blasting_iron_ore");
}

#[test]
fn recipe_iron_ingot_from_blasting_raw_iron() {
    check_recipe("minecraft:iron_ingot_from_blasting_raw_iron");
}

#[test]
fn recipe_iron_ingot_from_iron_block() {
    check_recipe("minecraft:iron_ingot_from_iron_block");
}

#[test]
fn recipe_iron_ingot_from_nuggets() {
    check_recipe("minecraft:iron_ingot_from_nuggets");
}

#[test]
fn recipe_iron_ingot_from_smelting_deepslate_iron_ore() {
    check_recipe("minecraft:iron_ingot_from_smelting_deepslate_iron_ore");
}

#[test]
fn recipe_iron_ingot_from_smelting_iron_ore() {
    check_recipe("minecraft:iron_ingot_from_smelting_iron_ore");
}

#[test]
fn recipe_iron_ingot_from_smelting_raw_iron() {
    check_recipe("minecraft:iron_ingot_from_smelting_raw_iron");
}

#[test]
fn recipe_iron_leggings() {
    check_recipe("minecraft:iron_leggings");
}

#[test]
fn recipe_iron_nugget() {
    check_recipe("minecraft:iron_nugget");
}

#[test]
fn recipe_iron_nugget_from_blasting() {
    check_recipe("minecraft:iron_nugget_from_blasting");
}

#[test]
fn recipe_iron_nugget_from_smelting() {
    check_recipe("minecraft:iron_nugget_from_smelting");
}

#[test]
fn recipe_iron_pickaxe() {
    check_recipe("minecraft:iron_pickaxe");
}

#[test]
fn recipe_iron_shovel() {
    check_recipe("minecraft:iron_shovel");
}

#[test]
fn recipe_iron_sword() {
    check_recipe("minecraft:iron_sword");
}

#[test]
fn recipe_iron_trapdoor() {
    check_recipe("minecraft:iron_trapdoor");
}

#[test]
fn recipe_item_frame() {
    check_recipe("minecraft:item_frame");
}

#[test]
fn recipe_jack_o_lantern() {
    check_recipe("minecraft:jack_o_lantern");
}

#[test]
fn recipe_jukebox() {
    check_recipe("minecraft:jukebox");
}

#[test]
fn recipe_jungle_boat() {
    check_recipe("minecraft:jungle_boat");
}

#[test]
fn recipe_jungle_button() {
    check_recipe("minecraft:jungle_button");
}

#[test]
fn recipe_jungle_chest_boat() {
    check_recipe("minecraft:jungle_chest_boat");
}

#[test]
fn recipe_jungle_door() {
    check_recipe("minecraft:jungle_door");
}

#[test]
fn recipe_jungle_fence() {
    check_recipe("minecraft:jungle_fence");
}

#[test]
fn recipe_jungle_fence_gate() {
    check_recipe("minecraft:jungle_fence_gate");
}

#[test]
fn recipe_jungle_hanging_sign() {
    check_recipe("minecraft:jungle_hanging_sign");
}

#[test]
fn recipe_jungle_planks() {
    check_recipe("minecraft:jungle_planks");
}

#[test]
fn recipe_jungle_pressure_plate() {
    check_recipe("minecraft:jungle_pressure_plate");
}

#[test]
fn recipe_jungle_sign() {
    check_recipe("minecraft:jungle_sign");
}

#[test]
fn recipe_jungle_slab() {
    check_recipe("minecraft:jungle_slab");
}

#[test]
fn recipe_jungle_stairs() {
    check_recipe("minecraft:jungle_stairs");
}

#[test]
fn recipe_jungle_trapdoor() {
    check_recipe("minecraft:jungle_trapdoor");
}

#[test]
fn recipe_jungle_wood() {
    check_recipe("minecraft:jungle_wood");
}

#[test]
fn recipe_ladder() {
    check_recipe("minecraft:ladder");
}

#[test]
fn recipe_lantern() {
    check_recipe("minecraft:lantern");
}

#[test]
fn recipe_lapis_block() {
    check_recipe("minecraft:lapis_block");
}

#[test]
fn recipe_lapis_lazuli() {
    check_recipe("minecraft:lapis_lazuli");
}

#[test]
fn recipe_lapis_lazuli_from_blasting_deepslate_lapis_ore() {
    check_recipe("minecraft:lapis_lazuli_from_blasting_deepslate_lapis_ore");
}

#[test]
fn recipe_lapis_lazuli_from_blasting_lapis_ore() {
    check_recipe("minecraft:lapis_lazuli_from_blasting_lapis_ore");
}

#[test]
fn recipe_lapis_lazuli_from_smelting_deepslate_lapis_ore() {
    check_recipe("minecraft:lapis_lazuli_from_smelting_deepslate_lapis_ore");
}

#[test]
fn recipe_lapis_lazuli_from_smelting_lapis_ore() {
    check_recipe("minecraft:lapis_lazuli_from_smelting_lapis_ore");
}

#[test]
fn recipe_lead() {
    check_recipe("minecraft:lead");
}

#[test]
fn recipe_leaf_litter() {
    check_recipe("minecraft:leaf_litter");
}

#[test]
fn recipe_leather() {
    check_recipe("minecraft:leather");
}

#[test]
fn recipe_leather_boots() {
    check_recipe("minecraft:leather_boots");
}

#[test]
fn recipe_leather_chestplate() {
    check_recipe("minecraft:leather_chestplate");
}

#[test]
fn recipe_leather_helmet() {
    check_recipe("minecraft:leather_helmet");
}

#[test]
fn recipe_leather_horse_armor() {
    check_recipe("minecraft:leather_horse_armor");
}

#[test]
fn recipe_leather_leggings() {
    check_recipe("minecraft:leather_leggings");
}

#[test]
fn recipe_lectern() {
    check_recipe("minecraft:lectern");
}

#[test]
fn recipe_lever() {
    check_recipe("minecraft:lever");
}

#[test]
fn recipe_light_blue_banner() {
    check_recipe("minecraft:light_blue_banner");
}

#[test]
fn recipe_light_blue_bed() {
    check_recipe("minecraft:light_blue_bed");
}

#[test]
fn recipe_light_blue_bundle() {
    check_recipe("minecraft:light_blue_bundle");
}

#[test]
fn recipe_light_blue_candle() {
    check_recipe("minecraft:light_blue_candle");
}

#[test]
fn recipe_light_blue_carpet() {
    check_recipe("minecraft:light_blue_carpet");
}

#[test]
fn recipe_light_blue_concrete_powder() {
    check_recipe("minecraft:light_blue_concrete_powder");
}

#[test]
fn recipe_light_blue_dye_from_blue_orchid() {
    check_recipe("minecraft:light_blue_dye_from_blue_orchid");
}

#[test]
fn recipe_light_blue_dye_from_blue_white_dye() {
    check_recipe("minecraft:light_blue_dye_from_blue_white_dye");
}

#[test]
fn recipe_light_blue_glazed_terracotta() {
    check_recipe("minecraft:light_blue_glazed_terracotta");
}

#[test]
fn recipe_light_blue_harness() {
    check_recipe("minecraft:light_blue_harness");
}

#[test]
fn recipe_light_blue_shulker_box() {
    check_recipe("minecraft:light_blue_shulker_box");
}

#[test]
fn recipe_light_blue_stained_glass() {
    check_recipe("minecraft:light_blue_stained_glass");
}

#[test]
fn recipe_light_blue_stained_glass_pane() {
    check_recipe("minecraft:light_blue_stained_glass_pane");
}

#[test]
fn recipe_light_blue_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:light_blue_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_light_blue_terracotta() {
    check_recipe("minecraft:light_blue_terracotta");
}

#[test]
fn recipe_light_gray_banner() {
    check_recipe("minecraft:light_gray_banner");
}

#[test]
fn recipe_light_gray_bed() {
    check_recipe("minecraft:light_gray_bed");
}

#[test]
fn recipe_light_gray_bundle() {
    check_recipe("minecraft:light_gray_bundle");
}

#[test]
fn recipe_light_gray_candle() {
    check_recipe("minecraft:light_gray_candle");
}

#[test]
fn recipe_light_gray_carpet() {
    check_recipe("minecraft:light_gray_carpet");
}

#[test]
fn recipe_light_gray_concrete_powder() {
    check_recipe("minecraft:light_gray_concrete_powder");
}

#[test]
fn recipe_light_gray_dye_from_azure_bluet() {
    check_recipe("minecraft:light_gray_dye_from_azure_bluet");
}

#[test]
fn recipe_light_gray_dye_from_black_white_dye() {
    check_recipe("minecraft:light_gray_dye_from_black_white_dye");
}

#[test]
fn recipe_light_gray_dye_from_gray_white_dye() {
    check_recipe("minecraft:light_gray_dye_from_gray_white_dye");
}

#[test]
fn recipe_light_gray_dye_from_oxeye_daisy() {
    check_recipe("minecraft:light_gray_dye_from_oxeye_daisy");
}

#[test]
fn recipe_light_gray_dye_from_white_tulip() {
    check_recipe("minecraft:light_gray_dye_from_white_tulip");
}

#[test]
fn recipe_light_gray_glazed_terracotta() {
    check_recipe("minecraft:light_gray_glazed_terracotta");
}

#[test]
fn recipe_light_gray_harness() {
    check_recipe("minecraft:light_gray_harness");
}

#[test]
fn recipe_light_gray_shulker_box() {
    check_recipe("minecraft:light_gray_shulker_box");
}

#[test]
fn recipe_light_gray_stained_glass() {
    check_recipe("minecraft:light_gray_stained_glass");
}

#[test]
fn recipe_light_gray_stained_glass_pane() {
    check_recipe("minecraft:light_gray_stained_glass_pane");
}

#[test]
fn recipe_light_gray_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:light_gray_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_light_gray_terracotta() {
    check_recipe("minecraft:light_gray_terracotta");
}

#[test]
fn recipe_light_weighted_pressure_plate() {
    check_recipe("minecraft:light_weighted_pressure_plate");
}

#[test]
fn recipe_lightning_rod() {
    check_recipe("minecraft:lightning_rod");
}

#[test]
fn recipe_lime_banner() {
    check_recipe("minecraft:lime_banner");
}

#[test]
fn recipe_lime_bed() {
    check_recipe("minecraft:lime_bed");
}

#[test]
fn recipe_lime_bundle() {
    check_recipe("minecraft:lime_bundle");
}

#[test]
fn recipe_lime_candle() {
    check_recipe("minecraft:lime_candle");
}

#[test]
fn recipe_lime_carpet() {
    check_recipe("minecraft:lime_carpet");
}

#[test]
fn recipe_lime_concrete_powder() {
    check_recipe("minecraft:lime_concrete_powder");
}

#[test]
fn recipe_lime_dye() {
    check_recipe("minecraft:lime_dye");
}

#[test]
fn recipe_lime_dye_from_smelting() {
    check_recipe("minecraft:lime_dye_from_smelting");
}

#[test]
fn recipe_lime_glazed_terracotta() {
    check_recipe("minecraft:lime_glazed_terracotta");
}

#[test]
fn recipe_lime_harness() {
    check_recipe("minecraft:lime_harness");
}

#[test]
fn recipe_lime_shulker_box() {
    check_recipe("minecraft:lime_shulker_box");
}

#[test]
fn recipe_lime_stained_glass() {
    check_recipe("minecraft:lime_stained_glass");
}

#[test]
fn recipe_lime_stained_glass_pane() {
    check_recipe("minecraft:lime_stained_glass_pane");
}

#[test]
fn recipe_lime_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:lime_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_lime_terracotta() {
    check_recipe("minecraft:lime_terracotta");
}

#[test]
fn recipe_lodestone() {
    check_recipe("minecraft:lodestone");
}

#[test]
fn recipe_loom() {
    check_recipe("minecraft:loom");
}

#[test]
fn recipe_mace() {
    check_recipe("minecraft:mace");
}

#[test]
fn recipe_magenta_banner() {
    check_recipe("minecraft:magenta_banner");
}

#[test]
fn recipe_magenta_bed() {
    check_recipe("minecraft:magenta_bed");
}

#[test]
fn recipe_magenta_bundle() {
    check_recipe("minecraft:magenta_bundle");
}

#[test]
fn recipe_magenta_candle() {
    check_recipe("minecraft:magenta_candle");
}

#[test]
fn recipe_magenta_carpet() {
    check_recipe("minecraft:magenta_carpet");
}

#[test]
fn recipe_magenta_concrete_powder() {
    check_recipe("minecraft:magenta_concrete_powder");
}

#[test]
fn recipe_magenta_dye_from_allium() {
    check_recipe("minecraft:magenta_dye_from_allium");
}

#[test]
fn recipe_magenta_dye_from_blue_red_pink() {
    check_recipe("minecraft:magenta_dye_from_blue_red_pink");
}

#[test]
fn recipe_magenta_dye_from_blue_red_white_dye() {
    check_recipe("minecraft:magenta_dye_from_blue_red_white_dye");
}

#[test]
fn recipe_magenta_dye_from_lilac() {
    check_recipe("minecraft:magenta_dye_from_lilac");
}

#[test]
fn recipe_magenta_dye_from_purple_and_pink() {
    check_recipe("minecraft:magenta_dye_from_purple_and_pink");
}

#[test]
fn recipe_magenta_glazed_terracotta() {
    check_recipe("minecraft:magenta_glazed_terracotta");
}

#[test]
fn recipe_magenta_harness() {
    check_recipe("minecraft:magenta_harness");
}

#[test]
fn recipe_magenta_shulker_box() {
    check_recipe("minecraft:magenta_shulker_box");
}

#[test]
fn recipe_magenta_stained_glass() {
    check_recipe("minecraft:magenta_stained_glass");
}

#[test]
fn recipe_magenta_stained_glass_pane() {
    check_recipe("minecraft:magenta_stained_glass_pane");
}

#[test]
fn recipe_magenta_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:magenta_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_magenta_terracotta() {
    check_recipe("minecraft:magenta_terracotta");
}

#[test]
fn recipe_magma_block() {
    check_recipe("minecraft:magma_block");
}

#[test]
fn recipe_magma_cream() {
    check_recipe("minecraft:magma_cream");
}

#[test]
fn recipe_mangrove_boat() {
    check_recipe("minecraft:mangrove_boat");
}

#[test]
fn recipe_mangrove_button() {
    check_recipe("minecraft:mangrove_button");
}

#[test]
fn recipe_mangrove_chest_boat() {
    check_recipe("minecraft:mangrove_chest_boat");
}

#[test]
fn recipe_mangrove_door() {
    check_recipe("minecraft:mangrove_door");
}

#[test]
fn recipe_mangrove_fence() {
    check_recipe("minecraft:mangrove_fence");
}

#[test]
fn recipe_mangrove_fence_gate() {
    check_recipe("minecraft:mangrove_fence_gate");
}

#[test]
fn recipe_mangrove_hanging_sign() {
    check_recipe("minecraft:mangrove_hanging_sign");
}

#[test]
fn recipe_mangrove_planks() {
    check_recipe("minecraft:mangrove_planks");
}

#[test]
fn recipe_mangrove_pressure_plate() {
    check_recipe("minecraft:mangrove_pressure_plate");
}

#[test]
fn recipe_mangrove_sign() {
    check_recipe("minecraft:mangrove_sign");
}

#[test]
fn recipe_mangrove_slab() {
    check_recipe("minecraft:mangrove_slab");
}

#[test]
fn recipe_mangrove_stairs() {
    check_recipe("minecraft:mangrove_stairs");
}

#[test]
fn recipe_mangrove_trapdoor() {
    check_recipe("minecraft:mangrove_trapdoor");
}

#[test]
fn recipe_mangrove_wood() {
    check_recipe("minecraft:mangrove_wood");
}

#[test]
fn recipe_map() {
    check_recipe("minecraft:map");
}

#[test]
fn recipe_map_cloning() {
    check_recipe("minecraft:map_cloning");
}

#[test]
fn recipe_map_extending() {
    check_recipe("minecraft:map_extending");
}

#[test]
fn recipe_melon() {
    check_recipe("minecraft:melon");
}

#[test]
fn recipe_melon_seeds() {
    check_recipe("minecraft:melon_seeds");
}

#[test]
fn recipe_minecart() {
    check_recipe("minecraft:minecart");
}

#[test]
fn recipe_mojang_banner_pattern() {
    check_recipe("minecraft:mojang_banner_pattern");
}

#[test]
fn recipe_moss_carpet() {
    check_recipe("minecraft:moss_carpet");
}

#[test]
fn recipe_mossy_cobblestone_from_moss_block() {
    check_recipe("minecraft:mossy_cobblestone_from_moss_block");
}

#[test]
fn recipe_mossy_cobblestone_from_vine() {
    check_recipe("minecraft:mossy_cobblestone_from_vine");
}

#[test]
fn recipe_mossy_cobblestone_slab() {
    check_recipe("minecraft:mossy_cobblestone_slab");
}

#[test]
fn recipe_mossy_cobblestone_slab_from_mossy_cobblestone_stonecutting() {
    check_recipe("minecraft:mossy_cobblestone_slab_from_mossy_cobblestone_stonecutting");
}

#[test]
fn recipe_mossy_cobblestone_stairs() {
    check_recipe("minecraft:mossy_cobblestone_stairs");
}

#[test]
fn recipe_mossy_cobblestone_stairs_from_mossy_cobblestone_stonecutting() {
    check_recipe("minecraft:mossy_cobblestone_stairs_from_mossy_cobblestone_stonecutting");
}

#[test]
fn recipe_mossy_cobblestone_wall() {
    check_recipe("minecraft:mossy_cobblestone_wall");
}

#[test]
fn recipe_mossy_cobblestone_wall_from_mossy_cobblestone_stonecutting() {
    check_recipe("minecraft:mossy_cobblestone_wall_from_mossy_cobblestone_stonecutting");
}

#[test]
fn recipe_mossy_stone_brick_slab() {
    check_recipe("minecraft:mossy_stone_brick_slab");
}

#[test]
fn recipe_mossy_stone_brick_slab_from_mossy_stone_brick_stonecutting() {
    check_recipe("minecraft:mossy_stone_brick_slab_from_mossy_stone_brick_stonecutting");
}

#[test]
fn recipe_mossy_stone_brick_stairs() {
    check_recipe("minecraft:mossy_stone_brick_stairs");
}

#[test]
fn recipe_mossy_stone_brick_stairs_from_mossy_stone_brick_stonecutting() {
    check_recipe("minecraft:mossy_stone_brick_stairs_from_mossy_stone_brick_stonecutting");
}

#[test]
fn recipe_mossy_stone_brick_wall() {
    check_recipe("minecraft:mossy_stone_brick_wall");
}

#[test]
fn recipe_mossy_stone_brick_wall_from_mossy_stone_brick_stonecutting() {
    check_recipe("minecraft:mossy_stone_brick_wall_from_mossy_stone_brick_stonecutting");
}

#[test]
fn recipe_mossy_stone_bricks_from_moss_block() {
    check_recipe("minecraft:mossy_stone_bricks_from_moss_block");
}

#[test]
fn recipe_mossy_stone_bricks_from_vine() {
    check_recipe("minecraft:mossy_stone_bricks_from_vine");
}

#[test]
fn recipe_mud_brick_slab() {
    check_recipe("minecraft:mud_brick_slab");
}

#[test]
fn recipe_mud_brick_slab_from_mud_bricks_stonecutting() {
    check_recipe("minecraft:mud_brick_slab_from_mud_bricks_stonecutting");
}

#[test]
fn recipe_mud_brick_stairs() {
    check_recipe("minecraft:mud_brick_stairs");
}

#[test]
fn recipe_mud_brick_stairs_from_mud_bricks_stonecutting() {
    check_recipe("minecraft:mud_brick_stairs_from_mud_bricks_stonecutting");
}

#[test]
fn recipe_mud_brick_wall() {
    check_recipe("minecraft:mud_brick_wall");
}

#[test]
fn recipe_mud_brick_wall_from_mud_bricks_stonecutting() {
    check_recipe("minecraft:mud_brick_wall_from_mud_bricks_stonecutting");
}

#[test]
fn recipe_mud_bricks() {
    check_recipe("minecraft:mud_bricks");
}

#[test]
fn recipe_muddy_mangrove_roots() {
    check_recipe("minecraft:muddy_mangrove_roots");
}

#[test]
fn recipe_mushroom_stew() {
    check_recipe("minecraft:mushroom_stew");
}

#[test]
fn recipe_music_disc_5() {
    check_recipe("minecraft:music_disc_5");
}

#[test]
fn recipe_nether_brick() {
    check_recipe("minecraft:nether_brick");
}

#[test]
fn recipe_nether_brick_fence() {
    check_recipe("minecraft:nether_brick_fence");
}

#[test]
fn recipe_nether_brick_slab() {
    check_recipe("minecraft:nether_brick_slab");
}

#[test]
fn recipe_nether_brick_slab_from_nether_bricks_stonecutting() {
    check_recipe("minecraft:nether_brick_slab_from_nether_bricks_stonecutting");
}

#[test]
fn recipe_nether_brick_stairs() {
    check_recipe("minecraft:nether_brick_stairs");
}

#[test]
fn recipe_nether_brick_stairs_from_nether_bricks_stonecutting() {
    check_recipe("minecraft:nether_brick_stairs_from_nether_bricks_stonecutting");
}

#[test]
fn recipe_nether_brick_wall() {
    check_recipe("minecraft:nether_brick_wall");
}

#[test]
fn recipe_nether_brick_wall_from_nether_bricks_stonecutting() {
    check_recipe("minecraft:nether_brick_wall_from_nether_bricks_stonecutting");
}

#[test]
fn recipe_nether_bricks() {
    check_recipe("minecraft:nether_bricks");
}

#[test]
fn recipe_nether_wart_block() {
    check_recipe("minecraft:nether_wart_block");
}

#[test]
fn recipe_netherite_axe_smithing() {
    check_recipe("minecraft:netherite_axe_smithing");
}

#[test]
fn recipe_netherite_block() {
    check_recipe("minecraft:netherite_block");
}

#[test]
fn recipe_netherite_boots_smithing() {
    check_recipe("minecraft:netherite_boots_smithing");
}

#[test]
fn recipe_netherite_chestplate_smithing() {
    check_recipe("minecraft:netherite_chestplate_smithing");
}

#[test]
fn recipe_netherite_helmet_smithing() {
    check_recipe("minecraft:netherite_helmet_smithing");
}

#[test]
fn recipe_netherite_hoe_smithing() {
    check_recipe("minecraft:netherite_hoe_smithing");
}

#[test]
fn recipe_netherite_ingot() {
    check_recipe("minecraft:netherite_ingot");
}

#[test]
fn recipe_netherite_ingot_from_netherite_block() {
    check_recipe("minecraft:netherite_ingot_from_netherite_block");
}

#[test]
fn recipe_netherite_leggings_smithing() {
    check_recipe("minecraft:netherite_leggings_smithing");
}

#[test]
fn recipe_netherite_pickaxe_smithing() {
    check_recipe("minecraft:netherite_pickaxe_smithing");
}

#[test]
fn recipe_netherite_scrap() {
    check_recipe("minecraft:netherite_scrap");
}

#[test]
fn recipe_netherite_scrap_from_blasting() {
    check_recipe("minecraft:netherite_scrap_from_blasting");
}

#[test]
fn recipe_netherite_shovel_smithing() {
    check_recipe("minecraft:netherite_shovel_smithing");
}

#[test]
fn recipe_netherite_sword_smithing() {
    check_recipe("minecraft:netherite_sword_smithing");
}

#[test]
fn recipe_netherite_upgrade_smithing_template() {
    check_recipe("minecraft:netherite_upgrade_smithing_template");
}

#[test]
fn recipe_note_block() {
    check_recipe("minecraft:note_block");
}

#[test]
fn recipe_oak_boat() {
    check_recipe("minecraft:oak_boat");
}

#[test]
fn recipe_oak_button() {
    check_recipe("minecraft:oak_button");
}

#[test]
fn recipe_oak_chest_boat() {
    check_recipe("minecraft:oak_chest_boat");
}

#[test]
fn recipe_oak_door() {
    check_recipe("minecraft:oak_door");
}

#[test]
fn recipe_oak_fence() {
    check_recipe("minecraft:oak_fence");
}

#[test]
fn recipe_oak_fence_gate() {
    check_recipe("minecraft:oak_fence_gate");
}

#[test]
fn recipe_oak_hanging_sign() {
    check_recipe("minecraft:oak_hanging_sign");
}

#[test]
fn recipe_oak_planks() {
    check_recipe("minecraft:oak_planks");
}

#[test]
fn recipe_oak_pressure_plate() {
    check_recipe("minecraft:oak_pressure_plate");
}

#[test]
fn recipe_oak_sign() {
    check_recipe("minecraft:oak_sign");
}

#[test]
fn recipe_oak_slab() {
    check_recipe("minecraft:oak_slab");
}

#[test]
fn recipe_oak_stairs() {
    check_recipe("minecraft:oak_stairs");
}

#[test]
fn recipe_oak_trapdoor() {
    check_recipe("minecraft:oak_trapdoor");
}

#[test]
fn recipe_oak_wood() {
    check_recipe("minecraft:oak_wood");
}

#[test]
fn recipe_observer() {
    check_recipe("minecraft:observer");
}

#[test]
fn recipe_orange_banner() {
    check_recipe("minecraft:orange_banner");
}

#[test]
fn recipe_orange_bed() {
    check_recipe("minecraft:orange_bed");
}

#[test]
fn recipe_orange_bundle() {
    check_recipe("minecraft:orange_bundle");
}

#[test]
fn recipe_orange_candle() {
    check_recipe("minecraft:orange_candle");
}

#[test]
fn recipe_orange_carpet() {
    check_recipe("minecraft:orange_carpet");
}

#[test]
fn recipe_orange_concrete_powder() {
    check_recipe("minecraft:orange_concrete_powder");
}

#[test]
fn recipe_orange_dye_from_open_eyeblossom() {
    check_recipe("minecraft:orange_dye_from_open_eyeblossom");
}

#[test]
fn recipe_orange_dye_from_orange_tulip() {
    check_recipe("minecraft:orange_dye_from_orange_tulip");
}

#[test]
fn recipe_orange_dye_from_red_yellow() {
    check_recipe("minecraft:orange_dye_from_red_yellow");
}

#[test]
fn recipe_orange_dye_from_torchflower() {
    check_recipe("minecraft:orange_dye_from_torchflower");
}

#[test]
fn recipe_orange_glazed_terracotta() {
    check_recipe("minecraft:orange_glazed_terracotta");
}

#[test]
fn recipe_orange_harness() {
    check_recipe("minecraft:orange_harness");
}

#[test]
fn recipe_orange_shulker_box() {
    check_recipe("minecraft:orange_shulker_box");
}

#[test]
fn recipe_orange_stained_glass() {
    check_recipe("minecraft:orange_stained_glass");
}

#[test]
fn recipe_orange_stained_glass_pane() {
    check_recipe("minecraft:orange_stained_glass_pane");
}

#[test]
fn recipe_orange_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:orange_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_orange_terracotta() {
    check_recipe("minecraft:orange_terracotta");
}

#[test]
fn recipe_oxidized_chiseled_copper() {
    check_recipe("minecraft:oxidized_chiseled_copper");
}

#[test]
fn recipe_oxidized_chiseled_copper_from_oxidized_copper_stonecutting() {
    check_recipe("minecraft:oxidized_chiseled_copper_from_oxidized_copper_stonecutting");
}

#[test]
fn recipe_oxidized_chiseled_copper_from_oxidized_cut_copper_stonecutting() {
    check_recipe("minecraft:oxidized_chiseled_copper_from_oxidized_cut_copper_stonecutting");
}

#[test]
fn recipe_oxidized_copper_bulb() {
    check_recipe("minecraft:oxidized_copper_bulb");
}

#[test]
fn recipe_oxidized_copper_grate() {
    check_recipe("minecraft:oxidized_copper_grate");
}

#[test]
fn recipe_oxidized_copper_grate_from_oxidized_copper_stonecutting() {
    check_recipe("minecraft:oxidized_copper_grate_from_oxidized_copper_stonecutting");
}

#[test]
fn recipe_oxidized_cut_copper() {
    check_recipe("minecraft:oxidized_cut_copper");
}

#[test]
fn recipe_oxidized_cut_copper_from_oxidized_copper_stonecutting() {
    check_recipe("minecraft:oxidized_cut_copper_from_oxidized_copper_stonecutting");
}

#[test]
fn recipe_oxidized_cut_copper_slab() {
    check_recipe("minecraft:oxidized_cut_copper_slab");
}

#[test]
fn recipe_oxidized_cut_copper_slab_from_oxidized_copper_stonecutting() {
    check_recipe("minecraft:oxidized_cut_copper_slab_from_oxidized_copper_stonecutting");
}

#[test]
fn recipe_oxidized_cut_copper_slab_from_oxidized_cut_copper_stonecutting() {
    check_recipe("minecraft:oxidized_cut_copper_slab_from_oxidized_cut_copper_stonecutting");
}

#[test]
fn recipe_oxidized_cut_copper_stairs() {
    check_recipe("minecraft:oxidized_cut_copper_stairs");
}

#[test]
fn recipe_oxidized_cut_copper_stairs_from_oxidized_copper_stonecutting() {
    check_recipe("minecraft:oxidized_cut_copper_stairs_from_oxidized_copper_stonecutting");
}

#[test]
fn recipe_oxidized_cut_copper_stairs_from_oxidized_cut_copper_stonecutting() {
    check_recipe("minecraft:oxidized_cut_copper_stairs_from_oxidized_cut_copper_stonecutting");
}

#[test]
fn recipe_packed_ice() {
    check_recipe("minecraft:packed_ice");
}

#[test]
fn recipe_packed_mud() {
    check_recipe("minecraft:packed_mud");
}

#[test]
fn recipe_painting() {
    check_recipe("minecraft:painting");
}

#[test]
fn recipe_pale_moss_carpet() {
    check_recipe("minecraft:pale_moss_carpet");
}

#[test]
fn recipe_pale_oak_boat() {
    check_recipe("minecraft:pale_oak_boat");
}

#[test]
fn recipe_pale_oak_button() {
    check_recipe("minecraft:pale_oak_button");
}

#[test]
fn recipe_pale_oak_chest_boat() {
    check_recipe("minecraft:pale_oak_chest_boat");
}

#[test]
fn recipe_pale_oak_door() {
    check_recipe("minecraft:pale_oak_door");
}

#[test]
fn recipe_pale_oak_fence() {
    check_recipe("minecraft:pale_oak_fence");
}

#[test]
fn recipe_pale_oak_fence_gate() {
    check_recipe("minecraft:pale_oak_fence_gate");
}

#[test]
fn recipe_pale_oak_hanging_sign() {
    check_recipe("minecraft:pale_oak_hanging_sign");
}

#[test]
fn recipe_pale_oak_planks() {
    check_recipe("minecraft:pale_oak_planks");
}

#[test]
fn recipe_pale_oak_pressure_plate() {
    check_recipe("minecraft:pale_oak_pressure_plate");
}

#[test]
fn recipe_pale_oak_sign() {
    check_recipe("minecraft:pale_oak_sign");
}

#[test]
fn recipe_pale_oak_slab() {
    check_recipe("minecraft:pale_oak_slab");
}

#[test]
fn recipe_pale_oak_stairs() {
    check_recipe("minecraft:pale_oak_stairs");
}

#[test]
fn recipe_pale_oak_trapdoor() {
    check_recipe("minecraft:pale_oak_trapdoor");
}

#[test]
fn recipe_pale_oak_wood() {
    check_recipe("minecraft:pale_oak_wood");
}

#[test]
fn recipe_paper() {
    check_recipe("minecraft:paper");
}

#[test]
fn recipe_pink_banner() {
    check_recipe("minecraft:pink_banner");
}

#[test]
fn recipe_pink_bed() {
    check_recipe("minecraft:pink_bed");
}

#[test]
fn recipe_pink_bundle() {
    check_recipe("minecraft:pink_bundle");
}

#[test]
fn recipe_pink_candle() {
    check_recipe("minecraft:pink_candle");
}

#[test]
fn recipe_pink_carpet() {
    check_recipe("minecraft:pink_carpet");
}

#[test]
fn recipe_pink_concrete_powder() {
    check_recipe("minecraft:pink_concrete_powder");
}

#[test]
fn recipe_pink_dye_from_cactus_flower() {
    check_recipe("minecraft:pink_dye_from_cactus_flower");
}

#[test]
fn recipe_pink_dye_from_peony() {
    check_recipe("minecraft:pink_dye_from_peony");
}

#[test]
fn recipe_pink_dye_from_pink_petals() {
    check_recipe("minecraft:pink_dye_from_pink_petals");
}

#[test]
fn recipe_pink_dye_from_pink_tulip() {
    check_recipe("minecraft:pink_dye_from_pink_tulip");
}

#[test]
fn recipe_pink_dye_from_red_white_dye() {
    check_recipe("minecraft:pink_dye_from_red_white_dye");
}

#[test]
fn recipe_pink_glazed_terracotta() {
    check_recipe("minecraft:pink_glazed_terracotta");
}

#[test]
fn recipe_pink_harness() {
    check_recipe("minecraft:pink_harness");
}

#[test]
fn recipe_pink_shulker_box() {
    check_recipe("minecraft:pink_shulker_box");
}

#[test]
fn recipe_pink_stained_glass() {
    check_recipe("minecraft:pink_stained_glass");
}

#[test]
fn recipe_pink_stained_glass_pane() {
    check_recipe("minecraft:pink_stained_glass_pane");
}

#[test]
fn recipe_pink_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:pink_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_pink_terracotta() {
    check_recipe("minecraft:pink_terracotta");
}

#[test]
fn recipe_piston() {
    check_recipe("minecraft:piston");
}

#[test]
fn recipe_polished_andesite() {
    check_recipe("minecraft:polished_andesite");
}

#[test]
fn recipe_polished_andesite_from_andesite_stonecutting() {
    check_recipe("minecraft:polished_andesite_from_andesite_stonecutting");
}

#[test]
fn recipe_polished_andesite_slab() {
    check_recipe("minecraft:polished_andesite_slab");
}

#[test]
fn recipe_polished_andesite_slab_from_andesite_stonecutting() {
    check_recipe("minecraft:polished_andesite_slab_from_andesite_stonecutting");
}

#[test]
fn recipe_polished_andesite_slab_from_polished_andesite_stonecutting() {
    check_recipe("minecraft:polished_andesite_slab_from_polished_andesite_stonecutting");
}

#[test]
fn recipe_polished_andesite_stairs() {
    check_recipe("minecraft:polished_andesite_stairs");
}

#[test]
fn recipe_polished_andesite_stairs_from_andesite_stonecutting() {
    check_recipe("minecraft:polished_andesite_stairs_from_andesite_stonecutting");
}

#[test]
fn recipe_polished_andesite_stairs_from_polished_andesite_stonecutting() {
    check_recipe("minecraft:polished_andesite_stairs_from_polished_andesite_stonecutting");
}

#[test]
fn recipe_polished_basalt() {
    check_recipe("minecraft:polished_basalt");
}

#[test]
fn recipe_polished_basalt_from_basalt_stonecutting() {
    check_recipe("minecraft:polished_basalt_from_basalt_stonecutting");
}

#[test]
fn recipe_polished_blackstone() {
    check_recipe("minecraft:polished_blackstone");
}

#[test]
fn recipe_polished_blackstone_brick_slab() {
    check_recipe("minecraft:polished_blackstone_brick_slab");
}

#[test]
fn recipe_polished_blackstone_brick_slab_from_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_brick_slab_from_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_brick_slab_from_polished_blackstone_bricks_stonecutting() {
    check_recipe(
        "minecraft:polished_blackstone_brick_slab_from_polished_blackstone_bricks_stonecutting",
    );
}

#[test]
fn recipe_polished_blackstone_brick_slab_from_polished_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_brick_slab_from_polished_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_brick_stairs() {
    check_recipe("minecraft:polished_blackstone_brick_stairs");
}

#[test]
fn recipe_polished_blackstone_brick_stairs_from_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_brick_stairs_from_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_brick_stairs_from_polished_blackstone_bricks_stonecutting() {
    check_recipe(
        "minecraft:polished_blackstone_brick_stairs_from_polished_blackstone_bricks_stonecutting",
    );
}

#[test]
fn recipe_polished_blackstone_brick_stairs_from_polished_blackstone_stonecutting() {
    check_recipe(
        "minecraft:polished_blackstone_brick_stairs_from_polished_blackstone_stonecutting",
    );
}

#[test]
fn recipe_polished_blackstone_brick_wall() {
    check_recipe("minecraft:polished_blackstone_brick_wall");
}

#[test]
fn recipe_polished_blackstone_brick_wall_from_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_brick_wall_from_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_brick_wall_from_polished_blackstone_bricks_stonecutting() {
    check_recipe(
        "minecraft:polished_blackstone_brick_wall_from_polished_blackstone_bricks_stonecutting",
    );
}

#[test]
fn recipe_polished_blackstone_brick_wall_from_polished_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_brick_wall_from_polished_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_bricks() {
    check_recipe("minecraft:polished_blackstone_bricks");
}

#[test]
fn recipe_polished_blackstone_bricks_from_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_bricks_from_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_bricks_from_polished_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_bricks_from_polished_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_button() {
    check_recipe("minecraft:polished_blackstone_button");
}

#[test]
fn recipe_polished_blackstone_from_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_from_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_pressure_plate() {
    check_recipe("minecraft:polished_blackstone_pressure_plate");
}

#[test]
fn recipe_polished_blackstone_slab() {
    check_recipe("minecraft:polished_blackstone_slab");
}

#[test]
fn recipe_polished_blackstone_slab_from_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_slab_from_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_slab_from_polished_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_slab_from_polished_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_stairs() {
    check_recipe("minecraft:polished_blackstone_stairs");
}

#[test]
fn recipe_polished_blackstone_stairs_from_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_stairs_from_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_stairs_from_polished_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_stairs_from_polished_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_wall() {
    check_recipe("minecraft:polished_blackstone_wall");
}

#[test]
fn recipe_polished_blackstone_wall_from_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_wall_from_blackstone_stonecutting");
}

#[test]
fn recipe_polished_blackstone_wall_from_polished_blackstone_stonecutting() {
    check_recipe("minecraft:polished_blackstone_wall_from_polished_blackstone_stonecutting");
}

#[test]
fn recipe_polished_deepslate() {
    check_recipe("minecraft:polished_deepslate");
}

#[test]
fn recipe_polished_deepslate_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:polished_deepslate_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_polished_deepslate_slab() {
    check_recipe("minecraft:polished_deepslate_slab");
}

#[test]
fn recipe_polished_deepslate_slab_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:polished_deepslate_slab_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_polished_deepslate_slab_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:polished_deepslate_slab_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_polished_deepslate_stairs() {
    check_recipe("minecraft:polished_deepslate_stairs");
}

#[test]
fn recipe_polished_deepslate_stairs_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:polished_deepslate_stairs_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_polished_deepslate_stairs_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:polished_deepslate_stairs_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_polished_deepslate_wall() {
    check_recipe("minecraft:polished_deepslate_wall");
}

#[test]
fn recipe_polished_deepslate_wall_from_cobbled_deepslate_stonecutting() {
    check_recipe("minecraft:polished_deepslate_wall_from_cobbled_deepslate_stonecutting");
}

#[test]
fn recipe_polished_deepslate_wall_from_polished_deepslate_stonecutting() {
    check_recipe("minecraft:polished_deepslate_wall_from_polished_deepslate_stonecutting");
}

#[test]
fn recipe_polished_diorite() {
    check_recipe("minecraft:polished_diorite");
}

#[test]
fn recipe_polished_diorite_from_diorite_stonecutting() {
    check_recipe("minecraft:polished_diorite_from_diorite_stonecutting");
}

#[test]
fn recipe_polished_diorite_slab() {
    check_recipe("minecraft:polished_diorite_slab");
}

#[test]
fn recipe_polished_diorite_slab_from_diorite_stonecutting() {
    check_recipe("minecraft:polished_diorite_slab_from_diorite_stonecutting");
}

#[test]
fn recipe_polished_diorite_slab_from_polished_diorite_stonecutting() {
    check_recipe("minecraft:polished_diorite_slab_from_polished_diorite_stonecutting");
}

#[test]
fn recipe_polished_diorite_stairs() {
    check_recipe("minecraft:polished_diorite_stairs");
}

#[test]
fn recipe_polished_diorite_stairs_from_diorite_stonecutting() {
    check_recipe("minecraft:polished_diorite_stairs_from_diorite_stonecutting");
}

#[test]
fn recipe_polished_diorite_stairs_from_polished_diorite_stonecutting() {
    check_recipe("minecraft:polished_diorite_stairs_from_polished_diorite_stonecutting");
}

#[test]
fn recipe_polished_granite() {
    check_recipe("minecraft:polished_granite");
}

#[test]
fn recipe_polished_granite_from_granite_stonecutting() {
    check_recipe("minecraft:polished_granite_from_granite_stonecutting");
}

#[test]
fn recipe_polished_granite_slab() {
    check_recipe("minecraft:polished_granite_slab");
}

#[test]
fn recipe_polished_granite_slab_from_granite_stonecutting() {
    check_recipe("minecraft:polished_granite_slab_from_granite_stonecutting");
}

#[test]
fn recipe_polished_granite_slab_from_polished_granite_stonecutting() {
    check_recipe("minecraft:polished_granite_slab_from_polished_granite_stonecutting");
}

#[test]
fn recipe_polished_granite_stairs() {
    check_recipe("minecraft:polished_granite_stairs");
}

#[test]
fn recipe_polished_granite_stairs_from_granite_stonecutting() {
    check_recipe("minecraft:polished_granite_stairs_from_granite_stonecutting");
}

#[test]
fn recipe_polished_granite_stairs_from_polished_granite_stonecutting() {
    check_recipe("minecraft:polished_granite_stairs_from_polished_granite_stonecutting");
}

#[test]
fn recipe_polished_tuff() {
    check_recipe("minecraft:polished_tuff");
}

#[test]
fn recipe_polished_tuff_from_tuff_stonecutting() {
    check_recipe("minecraft:polished_tuff_from_tuff_stonecutting");
}

#[test]
fn recipe_polished_tuff_slab() {
    check_recipe("minecraft:polished_tuff_slab");
}

#[test]
fn recipe_polished_tuff_slab_from_polished_tuff_stonecutting() {
    check_recipe("minecraft:polished_tuff_slab_from_polished_tuff_stonecutting");
}

#[test]
fn recipe_polished_tuff_slab_from_tuff_stonecutting() {
    check_recipe("minecraft:polished_tuff_slab_from_tuff_stonecutting");
}

#[test]
fn recipe_polished_tuff_stairs() {
    check_recipe("minecraft:polished_tuff_stairs");
}

#[test]
fn recipe_polished_tuff_stairs_from_polished_tuff_stonecutting() {
    check_recipe("minecraft:polished_tuff_stairs_from_polished_tuff_stonecutting");
}

#[test]
fn recipe_polished_tuff_stairs_from_tuff_stonecutting() {
    check_recipe("minecraft:polished_tuff_stairs_from_tuff_stonecutting");
}

#[test]
fn recipe_polished_tuff_wall() {
    check_recipe("minecraft:polished_tuff_wall");
}

#[test]
fn recipe_polished_tuff_wall_from_polished_tuff_stonecutting() {
    check_recipe("minecraft:polished_tuff_wall_from_polished_tuff_stonecutting");
}

#[test]
fn recipe_polished_tuff_wall_from_tuff_stonecutting() {
    check_recipe("minecraft:polished_tuff_wall_from_tuff_stonecutting");
}

#[test]
fn recipe_popped_chorus_fruit() {
    check_recipe("minecraft:popped_chorus_fruit");
}

#[test]
fn recipe_powered_rail() {
    check_recipe("minecraft:powered_rail");
}

#[test]
fn recipe_prismarine() {
    check_recipe("minecraft:prismarine");
}

#[test]
fn recipe_prismarine_brick_slab() {
    check_recipe("minecraft:prismarine_brick_slab");
}

#[test]
fn recipe_prismarine_brick_slab_from_prismarine_stonecutting() {
    check_recipe("minecraft:prismarine_brick_slab_from_prismarine_stonecutting");
}

#[test]
fn recipe_prismarine_brick_stairs() {
    check_recipe("minecraft:prismarine_brick_stairs");
}

#[test]
fn recipe_prismarine_brick_stairs_from_prismarine_stonecutting() {
    check_recipe("minecraft:prismarine_brick_stairs_from_prismarine_stonecutting");
}

#[test]
fn recipe_prismarine_bricks() {
    check_recipe("minecraft:prismarine_bricks");
}

#[test]
fn recipe_prismarine_slab() {
    check_recipe("minecraft:prismarine_slab");
}

#[test]
fn recipe_prismarine_slab_from_prismarine_stonecutting() {
    check_recipe("minecraft:prismarine_slab_from_prismarine_stonecutting");
}

#[test]
fn recipe_prismarine_stairs() {
    check_recipe("minecraft:prismarine_stairs");
}

#[test]
fn recipe_prismarine_stairs_from_prismarine_stonecutting() {
    check_recipe("minecraft:prismarine_stairs_from_prismarine_stonecutting");
}

#[test]
fn recipe_prismarine_wall() {
    check_recipe("minecraft:prismarine_wall");
}

#[test]
fn recipe_prismarine_wall_from_prismarine_stonecutting() {
    check_recipe("minecraft:prismarine_wall_from_prismarine_stonecutting");
}

#[test]
fn recipe_pumpkin_pie() {
    check_recipe("minecraft:pumpkin_pie");
}

#[test]
fn recipe_pumpkin_seeds() {
    check_recipe("minecraft:pumpkin_seeds");
}

#[test]
fn recipe_purple_banner() {
    check_recipe("minecraft:purple_banner");
}

#[test]
fn recipe_purple_bed() {
    check_recipe("minecraft:purple_bed");
}

#[test]
fn recipe_purple_bundle() {
    check_recipe("minecraft:purple_bundle");
}

#[test]
fn recipe_purple_candle() {
    check_recipe("minecraft:purple_candle");
}

#[test]
fn recipe_purple_carpet() {
    check_recipe("minecraft:purple_carpet");
}

#[test]
fn recipe_purple_concrete_powder() {
    check_recipe("minecraft:purple_concrete_powder");
}

#[test]
fn recipe_purple_dye() {
    check_recipe("minecraft:purple_dye");
}

#[test]
fn recipe_purple_glazed_terracotta() {
    check_recipe("minecraft:purple_glazed_terracotta");
}

#[test]
fn recipe_purple_harness() {
    check_recipe("minecraft:purple_harness");
}

#[test]
fn recipe_purple_shulker_box() {
    check_recipe("minecraft:purple_shulker_box");
}

#[test]
fn recipe_purple_stained_glass() {
    check_recipe("minecraft:purple_stained_glass");
}

#[test]
fn recipe_purple_stained_glass_pane() {
    check_recipe("minecraft:purple_stained_glass_pane");
}

#[test]
fn recipe_purple_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:purple_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_purple_terracotta() {
    check_recipe("minecraft:purple_terracotta");
}

#[test]
fn recipe_purpur_block() {
    check_recipe("minecraft:purpur_block");
}

#[test]
fn recipe_purpur_pillar() {
    check_recipe("minecraft:purpur_pillar");
}

#[test]
fn recipe_purpur_pillar_from_purpur_block_stonecutting() {
    check_recipe("minecraft:purpur_pillar_from_purpur_block_stonecutting");
}

#[test]
fn recipe_purpur_slab() {
    check_recipe("minecraft:purpur_slab");
}

#[test]
fn recipe_purpur_slab_from_purpur_block_stonecutting() {
    check_recipe("minecraft:purpur_slab_from_purpur_block_stonecutting");
}

#[test]
fn recipe_purpur_stairs() {
    check_recipe("minecraft:purpur_stairs");
}

#[test]
fn recipe_purpur_stairs_from_purpur_block_stonecutting() {
    check_recipe("minecraft:purpur_stairs_from_purpur_block_stonecutting");
}

#[test]
fn recipe_quartz() {
    check_recipe("minecraft:quartz");
}

#[test]
fn recipe_quartz_block() {
    check_recipe("minecraft:quartz_block");
}

#[test]
fn recipe_quartz_bricks() {
    check_recipe("minecraft:quartz_bricks");
}

#[test]
fn recipe_quartz_bricks_from_quartz_block_stonecutting() {
    check_recipe("minecraft:quartz_bricks_from_quartz_block_stonecutting");
}

#[test]
fn recipe_quartz_from_blasting() {
    check_recipe("minecraft:quartz_from_blasting");
}

#[test]
fn recipe_quartz_pillar() {
    check_recipe("minecraft:quartz_pillar");
}

#[test]
fn recipe_quartz_pillar_from_quartz_block_stonecutting() {
    check_recipe("minecraft:quartz_pillar_from_quartz_block_stonecutting");
}

#[test]
fn recipe_quartz_slab() {
    check_recipe("minecraft:quartz_slab");
}

#[test]
fn recipe_quartz_slab_from_stonecutting() {
    check_recipe("minecraft:quartz_slab_from_stonecutting");
}

#[test]
fn recipe_quartz_stairs() {
    check_recipe("minecraft:quartz_stairs");
}

#[test]
fn recipe_quartz_stairs_from_quartz_block_stonecutting() {
    check_recipe("minecraft:quartz_stairs_from_quartz_block_stonecutting");
}

#[test]
fn recipe_rabbit_stew_from_brown_mushroom() {
    check_recipe("minecraft:rabbit_stew_from_brown_mushroom");
}

#[test]
fn recipe_rabbit_stew_from_red_mushroom() {
    check_recipe("minecraft:rabbit_stew_from_red_mushroom");
}

#[test]
fn recipe_rail() {
    check_recipe("minecraft:rail");
}

#[test]
fn recipe_raiser_armor_trim_smithing_template() {
    check_recipe("minecraft:raiser_armor_trim_smithing_template");
}

#[test]
fn recipe_raiser_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:raiser_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_raw_copper() {
    check_recipe("minecraft:raw_copper");
}

#[test]
fn recipe_raw_copper_block() {
    check_recipe("minecraft:raw_copper_block");
}

#[test]
fn recipe_raw_gold() {
    check_recipe("minecraft:raw_gold");
}

#[test]
fn recipe_raw_gold_block() {
    check_recipe("minecraft:raw_gold_block");
}

#[test]
fn recipe_raw_iron() {
    check_recipe("minecraft:raw_iron");
}

#[test]
fn recipe_raw_iron_block() {
    check_recipe("minecraft:raw_iron_block");
}

#[test]
fn recipe_recovery_compass() {
    check_recipe("minecraft:recovery_compass");
}

#[test]
fn recipe_red_banner() {
    check_recipe("minecraft:red_banner");
}

#[test]
fn recipe_red_bed() {
    check_recipe("minecraft:red_bed");
}

#[test]
fn recipe_red_bundle() {
    check_recipe("minecraft:red_bundle");
}

#[test]
fn recipe_red_candle() {
    check_recipe("minecraft:red_candle");
}

#[test]
fn recipe_red_carpet() {
    check_recipe("minecraft:red_carpet");
}

#[test]
fn recipe_red_concrete_powder() {
    check_recipe("minecraft:red_concrete_powder");
}

#[test]
fn recipe_red_dye_from_beetroot() {
    check_recipe("minecraft:red_dye_from_beetroot");
}

#[test]
fn recipe_red_dye_from_poppy() {
    check_recipe("minecraft:red_dye_from_poppy");
}

#[test]
fn recipe_red_dye_from_rose_bush() {
    check_recipe("minecraft:red_dye_from_rose_bush");
}

#[test]
fn recipe_red_dye_from_tulip() {
    check_recipe("minecraft:red_dye_from_tulip");
}

#[test]
fn recipe_red_glazed_terracotta() {
    check_recipe("minecraft:red_glazed_terracotta");
}

#[test]
fn recipe_red_harness() {
    check_recipe("minecraft:red_harness");
}

#[test]
fn recipe_red_nether_brick_slab() {
    check_recipe("minecraft:red_nether_brick_slab");
}

#[test]
fn recipe_red_nether_brick_slab_from_red_nether_bricks_stonecutting() {
    check_recipe("minecraft:red_nether_brick_slab_from_red_nether_bricks_stonecutting");
}

#[test]
fn recipe_red_nether_brick_stairs() {
    check_recipe("minecraft:red_nether_brick_stairs");
}

#[test]
fn recipe_red_nether_brick_stairs_from_red_nether_bricks_stonecutting() {
    check_recipe("minecraft:red_nether_brick_stairs_from_red_nether_bricks_stonecutting");
}

#[test]
fn recipe_red_nether_brick_wall() {
    check_recipe("minecraft:red_nether_brick_wall");
}

#[test]
fn recipe_red_nether_brick_wall_from_red_nether_bricks_stonecutting() {
    check_recipe("minecraft:red_nether_brick_wall_from_red_nether_bricks_stonecutting");
}

#[test]
fn recipe_red_nether_bricks() {
    check_recipe("minecraft:red_nether_bricks");
}

#[test]
fn recipe_red_sandstone() {
    check_recipe("minecraft:red_sandstone");
}

#[test]
fn recipe_red_sandstone_slab() {
    check_recipe("minecraft:red_sandstone_slab");
}

#[test]
fn recipe_red_sandstone_slab_from_red_sandstone_stonecutting() {
    check_recipe("minecraft:red_sandstone_slab_from_red_sandstone_stonecutting");
}

#[test]
fn recipe_red_sandstone_stairs() {
    check_recipe("minecraft:red_sandstone_stairs");
}

#[test]
fn recipe_red_sandstone_stairs_from_red_sandstone_stonecutting() {
    check_recipe("minecraft:red_sandstone_stairs_from_red_sandstone_stonecutting");
}

#[test]
fn recipe_red_sandstone_wall() {
    check_recipe("minecraft:red_sandstone_wall");
}

#[test]
fn recipe_red_sandstone_wall_from_red_sandstone_stonecutting() {
    check_recipe("minecraft:red_sandstone_wall_from_red_sandstone_stonecutting");
}

#[test]
fn recipe_red_shulker_box() {
    check_recipe("minecraft:red_shulker_box");
}

#[test]
fn recipe_red_stained_glass() {
    check_recipe("minecraft:red_stained_glass");
}

#[test]
fn recipe_red_stained_glass_pane() {
    check_recipe("minecraft:red_stained_glass_pane");
}

#[test]
fn recipe_red_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:red_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_red_terracotta() {
    check_recipe("minecraft:red_terracotta");
}

#[test]
fn recipe_redstone() {
    check_recipe("minecraft:redstone");
}

#[test]
fn recipe_redstone_block() {
    check_recipe("minecraft:redstone_block");
}

#[test]
fn recipe_redstone_from_blasting_deepslate_redstone_ore() {
    check_recipe("minecraft:redstone_from_blasting_deepslate_redstone_ore");
}

#[test]
fn recipe_redstone_from_blasting_redstone_ore() {
    check_recipe("minecraft:redstone_from_blasting_redstone_ore");
}

#[test]
fn recipe_redstone_from_smelting_deepslate_redstone_ore() {
    check_recipe("minecraft:redstone_from_smelting_deepslate_redstone_ore");
}

#[test]
fn recipe_redstone_from_smelting_redstone_ore() {
    check_recipe("minecraft:redstone_from_smelting_redstone_ore");
}

#[test]
fn recipe_redstone_lamp() {
    check_recipe("minecraft:redstone_lamp");
}

#[test]
fn recipe_redstone_torch() {
    check_recipe("minecraft:redstone_torch");
}

#[test]
fn recipe_repair_item() {
    check_recipe("minecraft:repair_item");
}

#[test]
fn recipe_repeater() {
    check_recipe("minecraft:repeater");
}

#[test]
fn recipe_resin_block() {
    check_recipe("minecraft:resin_block");
}

#[test]
fn recipe_resin_brick() {
    check_recipe("minecraft:resin_brick");
}

#[test]
fn recipe_resin_brick_slab() {
    check_recipe("minecraft:resin_brick_slab");
}

#[test]
fn recipe_resin_brick_slab_from_resin_bricks_stonecutting() {
    check_recipe("minecraft:resin_brick_slab_from_resin_bricks_stonecutting");
}

#[test]
fn recipe_resin_brick_stairs() {
    check_recipe("minecraft:resin_brick_stairs");
}

#[test]
fn recipe_resin_brick_stairs_from_resin_bricks_stonecutting() {
    check_recipe("minecraft:resin_brick_stairs_from_resin_bricks_stonecutting");
}

#[test]
fn recipe_resin_brick_wall() {
    check_recipe("minecraft:resin_brick_wall");
}

#[test]
fn recipe_resin_brick_wall_from_resin_bricks_stonecutting() {
    check_recipe("minecraft:resin_brick_wall_from_resin_bricks_stonecutting");
}

#[test]
fn recipe_resin_bricks() {
    check_recipe("minecraft:resin_bricks");
}

#[test]
fn recipe_resin_clump() {
    check_recipe("minecraft:resin_clump");
}

#[test]
fn recipe_respawn_anchor() {
    check_recipe("minecraft:respawn_anchor");
}

#[test]
fn recipe_rib_armor_trim_smithing_template() {
    check_recipe("minecraft:rib_armor_trim_smithing_template");
}

#[test]
fn recipe_rib_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:rib_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_saddle() {
    check_recipe("minecraft:saddle");
}

#[test]
fn recipe_sandstone() {
    check_recipe("minecraft:sandstone");
}

#[test]
fn recipe_sandstone_slab() {
    check_recipe("minecraft:sandstone_slab");
}

#[test]
fn recipe_sandstone_slab_from_sandstone_stonecutting() {
    check_recipe("minecraft:sandstone_slab_from_sandstone_stonecutting");
}

#[test]
fn recipe_sandstone_stairs() {
    check_recipe("minecraft:sandstone_stairs");
}

#[test]
fn recipe_sandstone_stairs_from_sandstone_stonecutting() {
    check_recipe("minecraft:sandstone_stairs_from_sandstone_stonecutting");
}

#[test]
fn recipe_sandstone_wall() {
    check_recipe("minecraft:sandstone_wall");
}

#[test]
fn recipe_sandstone_wall_from_sandstone_stonecutting() {
    check_recipe("minecraft:sandstone_wall_from_sandstone_stonecutting");
}

#[test]
fn recipe_scaffolding() {
    check_recipe("minecraft:scaffolding");
}

#[test]
fn recipe_sea_lantern() {
    check_recipe("minecraft:sea_lantern");
}

#[test]
fn recipe_sentry_armor_trim_smithing_template() {
    check_recipe("minecraft:sentry_armor_trim_smithing_template");
}

#[test]
fn recipe_sentry_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:sentry_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_shaper_armor_trim_smithing_template() {
    check_recipe("minecraft:shaper_armor_trim_smithing_template");
}

#[test]
fn recipe_shaper_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:shaper_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_shears() {
    check_recipe("minecraft:shears");
}

#[test]
fn recipe_shield() {
    check_recipe("minecraft:shield");
}

#[test]
fn recipe_shield_decoration() {
    check_recipe("minecraft:shield_decoration");
}

#[test]
fn recipe_shulker_box() {
    check_recipe("minecraft:shulker_box");
}

#[test]
fn recipe_silence_armor_trim_smithing_template() {
    check_recipe("minecraft:silence_armor_trim_smithing_template");
}

#[test]
fn recipe_silence_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:silence_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_skull_banner_pattern() {
    check_recipe("minecraft:skull_banner_pattern");
}

#[test]
fn recipe_slime_ball() {
    check_recipe("minecraft:slime_ball");
}

#[test]
fn recipe_slime_block() {
    check_recipe("minecraft:slime_block");
}

#[test]
fn recipe_smithing_table() {
    check_recipe("minecraft:smithing_table");
}

#[test]
fn recipe_smoker() {
    check_recipe("minecraft:smoker");
}

#[test]
fn recipe_smooth_basalt() {
    check_recipe("minecraft:smooth_basalt");
}

#[test]
fn recipe_smooth_quartz() {
    check_recipe("minecraft:smooth_quartz");
}

#[test]
fn recipe_smooth_quartz_slab() {
    check_recipe("minecraft:smooth_quartz_slab");
}

#[test]
fn recipe_smooth_quartz_slab_from_smooth_quartz_stonecutting() {
    check_recipe("minecraft:smooth_quartz_slab_from_smooth_quartz_stonecutting");
}

#[test]
fn recipe_smooth_quartz_stairs() {
    check_recipe("minecraft:smooth_quartz_stairs");
}

#[test]
fn recipe_smooth_quartz_stairs_from_smooth_quartz_stonecutting() {
    check_recipe("minecraft:smooth_quartz_stairs_from_smooth_quartz_stonecutting");
}

#[test]
fn recipe_smooth_red_sandstone() {
    check_recipe("minecraft:smooth_red_sandstone");
}

#[test]
fn recipe_smooth_red_sandstone_slab() {
    check_recipe("minecraft:smooth_red_sandstone_slab");
}

#[test]
fn recipe_smooth_red_sandstone_slab_from_smooth_red_sandstone_stonecutting() {
    check_recipe("minecraft:smooth_red_sandstone_slab_from_smooth_red_sandstone_stonecutting");
}

#[test]
fn recipe_smooth_red_sandstone_stairs() {
    check_recipe("minecraft:smooth_red_sandstone_stairs");
}

#[test]
fn recipe_smooth_red_sandstone_stairs_from_smooth_red_sandstone_stonecutting() {
    check_recipe("minecraft:smooth_red_sandstone_stairs_from_smooth_red_sandstone_stonecutting");
}

#[test]
fn recipe_smooth_sandstone() {
    check_recipe("minecraft:smooth_sandstone");
}

#[test]
fn recipe_smooth_sandstone_slab() {
    check_recipe("minecraft:smooth_sandstone_slab");
}

#[test]
fn recipe_smooth_sandstone_slab_from_smooth_sandstone_stonecutting() {
    check_recipe("minecraft:smooth_sandstone_slab_from_smooth_sandstone_stonecutting");
}

#[test]
fn recipe_smooth_sandstone_stairs() {
    check_recipe("minecraft:smooth_sandstone_stairs");
}

#[test]
fn recipe_smooth_sandstone_stairs_from_smooth_sandstone_stonecutting() {
    check_recipe("minecraft:smooth_sandstone_stairs_from_smooth_sandstone_stonecutting");
}

#[test]
fn recipe_smooth_stone() {
    check_recipe("minecraft:smooth_stone");
}

#[test]
fn recipe_smooth_stone_slab() {
    check_recipe("minecraft:smooth_stone_slab");
}

#[test]
fn recipe_smooth_stone_slab_from_smooth_stone_stonecutting() {
    check_recipe("minecraft:smooth_stone_slab_from_smooth_stone_stonecutting");
}

#[test]
fn recipe_snout_armor_trim_smithing_template() {
    check_recipe("minecraft:snout_armor_trim_smithing_template");
}

#[test]
fn recipe_snout_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:snout_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_snow() {
    check_recipe("minecraft:snow");
}

#[test]
fn recipe_snow_block() {
    check_recipe("minecraft:snow_block");
}

#[test]
fn recipe_soul_campfire() {
    check_recipe("minecraft:soul_campfire");
}

#[test]
fn recipe_soul_lantern() {
    check_recipe("minecraft:soul_lantern");
}

#[test]
fn recipe_soul_torch() {
    check_recipe("minecraft:soul_torch");
}

#[test]
fn recipe_spectral_arrow() {
    check_recipe("minecraft:spectral_arrow");
}

#[test]
fn recipe_spire_armor_trim_smithing_template() {
    check_recipe("minecraft:spire_armor_trim_smithing_template");
}

#[test]
fn recipe_spire_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:spire_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_sponge() {
    check_recipe("minecraft:sponge");
}

#[test]
fn recipe_spruce_boat() {
    check_recipe("minecraft:spruce_boat");
}

#[test]
fn recipe_spruce_button() {
    check_recipe("minecraft:spruce_button");
}

#[test]
fn recipe_spruce_chest_boat() {
    check_recipe("minecraft:spruce_chest_boat");
}

#[test]
fn recipe_spruce_door() {
    check_recipe("minecraft:spruce_door");
}

#[test]
fn recipe_spruce_fence() {
    check_recipe("minecraft:spruce_fence");
}

#[test]
fn recipe_spruce_fence_gate() {
    check_recipe("minecraft:spruce_fence_gate");
}

#[test]
fn recipe_spruce_hanging_sign() {
    check_recipe("minecraft:spruce_hanging_sign");
}

#[test]
fn recipe_spruce_planks() {
    check_recipe("minecraft:spruce_planks");
}

#[test]
fn recipe_spruce_pressure_plate() {
    check_recipe("minecraft:spruce_pressure_plate");
}

#[test]
fn recipe_spruce_sign() {
    check_recipe("minecraft:spruce_sign");
}

#[test]
fn recipe_spruce_slab() {
    check_recipe("minecraft:spruce_slab");
}

#[test]
fn recipe_spruce_stairs() {
    check_recipe("minecraft:spruce_stairs");
}

#[test]
fn recipe_spruce_trapdoor() {
    check_recipe("minecraft:spruce_trapdoor");
}

#[test]
fn recipe_spruce_wood() {
    check_recipe("minecraft:spruce_wood");
}

#[test]
fn recipe_spyglass() {
    check_recipe("minecraft:spyglass");
}

#[test]
fn recipe_stick() {
    check_recipe("minecraft:stick");
}

#[test]
fn recipe_stick_from_bamboo_item() {
    check_recipe("minecraft:stick_from_bamboo_item");
}

#[test]
fn recipe_sticky_piston() {
    check_recipe("minecraft:sticky_piston");
}

#[test]
fn recipe_stone() {
    check_recipe("minecraft:stone");
}

#[test]
fn recipe_stone_axe() {
    check_recipe("minecraft:stone_axe");
}

#[test]
fn recipe_stone_brick_slab() {
    check_recipe("minecraft:stone_brick_slab");
}

#[test]
fn recipe_stone_brick_slab_from_stone_bricks_stonecutting() {
    check_recipe("minecraft:stone_brick_slab_from_stone_bricks_stonecutting");
}

#[test]
fn recipe_stone_brick_slab_from_stone_stonecutting() {
    check_recipe("minecraft:stone_brick_slab_from_stone_stonecutting");
}

#[test]
fn recipe_stone_brick_stairs() {
    check_recipe("minecraft:stone_brick_stairs");
}

#[test]
fn recipe_stone_brick_stairs_from_stone_bricks_stonecutting() {
    check_recipe("minecraft:stone_brick_stairs_from_stone_bricks_stonecutting");
}

#[test]
fn recipe_stone_brick_stairs_from_stone_stonecutting() {
    check_recipe("minecraft:stone_brick_stairs_from_stone_stonecutting");
}

#[test]
fn recipe_stone_brick_wall() {
    check_recipe("minecraft:stone_brick_wall");
}

#[test]
fn recipe_stone_brick_wall_from_stone_bricks_stonecutting() {
    check_recipe("minecraft:stone_brick_wall_from_stone_bricks_stonecutting");
}

#[test]
fn recipe_stone_brick_walls_from_stone_stonecutting() {
    check_recipe("minecraft:stone_brick_walls_from_stone_stonecutting");
}

#[test]
fn recipe_stone_bricks() {
    check_recipe("minecraft:stone_bricks");
}

#[test]
fn recipe_stone_bricks_from_stone_stonecutting() {
    check_recipe("minecraft:stone_bricks_from_stone_stonecutting");
}

#[test]
fn recipe_stone_button() {
    check_recipe("minecraft:stone_button");
}

#[test]
fn recipe_stone_hoe() {
    check_recipe("minecraft:stone_hoe");
}

#[test]
fn recipe_stone_pickaxe() {
    check_recipe("minecraft:stone_pickaxe");
}

#[test]
fn recipe_stone_pressure_plate() {
    check_recipe("minecraft:stone_pressure_plate");
}

#[test]
fn recipe_stone_shovel() {
    check_recipe("minecraft:stone_shovel");
}

#[test]
fn recipe_stone_slab() {
    check_recipe("minecraft:stone_slab");
}

#[test]
fn recipe_stone_slab_from_stone_stonecutting() {
    check_recipe("minecraft:stone_slab_from_stone_stonecutting");
}

#[test]
fn recipe_stone_stairs() {
    check_recipe("minecraft:stone_stairs");
}

#[test]
fn recipe_stone_stairs_from_stone_stonecutting() {
    check_recipe("minecraft:stone_stairs_from_stone_stonecutting");
}

#[test]
fn recipe_stone_sword() {
    check_recipe("minecraft:stone_sword");
}

#[test]
fn recipe_stonecutter() {
    check_recipe("minecraft:stonecutter");
}

#[test]
fn recipe_stripped_acacia_wood() {
    check_recipe("minecraft:stripped_acacia_wood");
}

#[test]
fn recipe_stripped_birch_wood() {
    check_recipe("minecraft:stripped_birch_wood");
}

#[test]
fn recipe_stripped_cherry_wood() {
    check_recipe("minecraft:stripped_cherry_wood");
}

#[test]
fn recipe_stripped_crimson_hyphae() {
    check_recipe("minecraft:stripped_crimson_hyphae");
}

#[test]
fn recipe_stripped_dark_oak_wood() {
    check_recipe("minecraft:stripped_dark_oak_wood");
}

#[test]
fn recipe_stripped_jungle_wood() {
    check_recipe("minecraft:stripped_jungle_wood");
}

#[test]
fn recipe_stripped_mangrove_wood() {
    check_recipe("minecraft:stripped_mangrove_wood");
}

#[test]
fn recipe_stripped_oak_wood() {
    check_recipe("minecraft:stripped_oak_wood");
}

#[test]
fn recipe_stripped_pale_oak_wood() {
    check_recipe("minecraft:stripped_pale_oak_wood");
}

#[test]
fn recipe_stripped_spruce_wood() {
    check_recipe("minecraft:stripped_spruce_wood");
}

#[test]
fn recipe_stripped_warped_hyphae() {
    check_recipe("minecraft:stripped_warped_hyphae");
}

#[test]
fn recipe_sugar_from_honey_bottle() {
    check_recipe("minecraft:sugar_from_honey_bottle");
}

#[test]
fn recipe_sugar_from_sugar_cane() {
    check_recipe("minecraft:sugar_from_sugar_cane");
}

#[test]
fn recipe_suspicious_stew_from_allium() {
    check_recipe("minecraft:suspicious_stew_from_allium");
}

#[test]
fn recipe_suspicious_stew_from_azure_bluet() {
    check_recipe("minecraft:suspicious_stew_from_azure_bluet");
}

#[test]
fn recipe_suspicious_stew_from_blue_orchid() {
    check_recipe("minecraft:suspicious_stew_from_blue_orchid");
}

#[test]
fn recipe_suspicious_stew_from_closed_eyeblossom() {
    check_recipe("minecraft:suspicious_stew_from_closed_eyeblossom");
}

#[test]
fn recipe_suspicious_stew_from_cornflower() {
    check_recipe("minecraft:suspicious_stew_from_cornflower");
}

#[test]
fn recipe_suspicious_stew_from_dandelion() {
    check_recipe("minecraft:suspicious_stew_from_dandelion");
}

#[test]
fn recipe_suspicious_stew_from_lily_of_the_valley() {
    check_recipe("minecraft:suspicious_stew_from_lily_of_the_valley");
}

#[test]
fn recipe_suspicious_stew_from_open_eyeblossom() {
    check_recipe("minecraft:suspicious_stew_from_open_eyeblossom");
}

#[test]
fn recipe_suspicious_stew_from_orange_tulip() {
    check_recipe("minecraft:suspicious_stew_from_orange_tulip");
}

#[test]
fn recipe_suspicious_stew_from_oxeye_daisy() {
    check_recipe("minecraft:suspicious_stew_from_oxeye_daisy");
}

#[test]
fn recipe_suspicious_stew_from_pink_tulip() {
    check_recipe("minecraft:suspicious_stew_from_pink_tulip");
}

#[test]
fn recipe_suspicious_stew_from_poppy() {
    check_recipe("minecraft:suspicious_stew_from_poppy");
}

#[test]
fn recipe_suspicious_stew_from_red_tulip() {
    check_recipe("minecraft:suspicious_stew_from_red_tulip");
}

#[test]
fn recipe_suspicious_stew_from_torchflower() {
    check_recipe("minecraft:suspicious_stew_from_torchflower");
}

#[test]
fn recipe_suspicious_stew_from_white_tulip() {
    check_recipe("minecraft:suspicious_stew_from_white_tulip");
}

#[test]
fn recipe_suspicious_stew_from_wither_rose() {
    check_recipe("minecraft:suspicious_stew_from_wither_rose");
}

#[test]
fn recipe_target() {
    check_recipe("minecraft:target");
}

#[test]
fn recipe_terracotta() {
    check_recipe("minecraft:terracotta");
}

#[test]
fn recipe_tide_armor_trim_smithing_template() {
    check_recipe("minecraft:tide_armor_trim_smithing_template");
}

#[test]
fn recipe_tide_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:tide_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_tinted_glass() {
    check_recipe("minecraft:tinted_glass");
}

#[test]
fn recipe_tipped_arrow() {
    check_recipe("minecraft:tipped_arrow");
}

#[test]
fn recipe_tnt() {
    check_recipe("minecraft:tnt");
}

#[test]
fn recipe_tnt_minecart() {
    check_recipe("minecraft:tnt_minecart");
}

#[test]
fn recipe_torch() {
    check_recipe("minecraft:torch");
}

#[test]
fn recipe_trapped_chest() {
    check_recipe("minecraft:trapped_chest");
}

#[test]
fn recipe_tripwire_hook() {
    check_recipe("minecraft:tripwire_hook");
}

#[test]
fn recipe_tuff_brick_slab() {
    check_recipe("minecraft:tuff_brick_slab");
}

#[test]
fn recipe_tuff_brick_slab_from_polished_tuff_stonecutting() {
    check_recipe("minecraft:tuff_brick_slab_from_polished_tuff_stonecutting");
}

#[test]
fn recipe_tuff_brick_slab_from_tuff_bricks_stonecutting() {
    check_recipe("minecraft:tuff_brick_slab_from_tuff_bricks_stonecutting");
}

#[test]
fn recipe_tuff_brick_slab_from_tuff_stonecutting() {
    check_recipe("minecraft:tuff_brick_slab_from_tuff_stonecutting");
}

#[test]
fn recipe_tuff_brick_stairs() {
    check_recipe("minecraft:tuff_brick_stairs");
}

#[test]
fn recipe_tuff_brick_stairs_from_polished_tuff_stonecutting() {
    check_recipe("minecraft:tuff_brick_stairs_from_polished_tuff_stonecutting");
}

#[test]
fn recipe_tuff_brick_stairs_from_tuff_bricks_stonecutting() {
    check_recipe("minecraft:tuff_brick_stairs_from_tuff_bricks_stonecutting");
}

#[test]
fn recipe_tuff_brick_stairs_from_tuff_stonecutting() {
    check_recipe("minecraft:tuff_brick_stairs_from_tuff_stonecutting");
}

#[test]
fn recipe_tuff_brick_wall() {
    check_recipe("minecraft:tuff_brick_wall");
}

#[test]
fn recipe_tuff_brick_wall_from_polished_tuff_stonecutting() {
    check_recipe("minecraft:tuff_brick_wall_from_polished_tuff_stonecutting");
}

#[test]
fn recipe_tuff_brick_wall_from_tuff_bricks_stonecutting() {
    check_recipe("minecraft:tuff_brick_wall_from_tuff_bricks_stonecutting");
}

#[test]
fn recipe_tuff_brick_wall_from_tuff_stonecutting() {
    check_recipe("minecraft:tuff_brick_wall_from_tuff_stonecutting");
}

#[test]
fn recipe_tuff_bricks() {
    check_recipe("minecraft:tuff_bricks");
}

#[test]
fn recipe_tuff_bricks_from_polished_tuff_stonecutting() {
    check_recipe("minecraft:tuff_bricks_from_polished_tuff_stonecutting");
}

#[test]
fn recipe_tuff_bricks_from_tuff_stonecutting() {
    check_recipe("minecraft:tuff_bricks_from_tuff_stonecutting");
}

#[test]
fn recipe_tuff_slab() {
    check_recipe("minecraft:tuff_slab");
}

#[test]
fn recipe_tuff_slab_from_tuff_stonecutting() {
    check_recipe("minecraft:tuff_slab_from_tuff_stonecutting");
}

#[test]
fn recipe_tuff_stairs() {
    check_recipe("minecraft:tuff_stairs");
}

#[test]
fn recipe_tuff_stairs_from_tuff_stonecutting() {
    check_recipe("minecraft:tuff_stairs_from_tuff_stonecutting");
}

#[test]
fn recipe_tuff_wall() {
    check_recipe("minecraft:tuff_wall");
}

#[test]
fn recipe_tuff_wall_from_tuff_stonecutting() {
    check_recipe("minecraft:tuff_wall_from_tuff_stonecutting");
}

#[test]
fn recipe_turtle_helmet() {
    check_recipe("minecraft:turtle_helmet");
}

#[test]
fn recipe_vex_armor_trim_smithing_template() {
    check_recipe("minecraft:vex_armor_trim_smithing_template");
}

#[test]
fn recipe_vex_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:vex_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_ward_armor_trim_smithing_template() {
    check_recipe("minecraft:ward_armor_trim_smithing_template");
}

#[test]
fn recipe_ward_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:ward_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_warped_button() {
    check_recipe("minecraft:warped_button");
}

#[test]
fn recipe_warped_door() {
    check_recipe("minecraft:warped_door");
}

#[test]
fn recipe_warped_fence() {
    check_recipe("minecraft:warped_fence");
}

#[test]
fn recipe_warped_fence_gate() {
    check_recipe("minecraft:warped_fence_gate");
}

#[test]
fn recipe_warped_fungus_on_a_stick() {
    check_recipe("minecraft:warped_fungus_on_a_stick");
}

#[test]
fn recipe_warped_hanging_sign() {
    check_recipe("minecraft:warped_hanging_sign");
}

#[test]
fn recipe_warped_hyphae() {
    check_recipe("minecraft:warped_hyphae");
}

#[test]
fn recipe_warped_planks() {
    check_recipe("minecraft:warped_planks");
}

#[test]
fn recipe_warped_pressure_plate() {
    check_recipe("minecraft:warped_pressure_plate");
}

#[test]
fn recipe_warped_sign() {
    check_recipe("minecraft:warped_sign");
}

#[test]
fn recipe_warped_slab() {
    check_recipe("minecraft:warped_slab");
}

#[test]
fn recipe_warped_stairs() {
    check_recipe("minecraft:warped_stairs");
}

#[test]
fn recipe_warped_trapdoor() {
    check_recipe("minecraft:warped_trapdoor");
}

#[test]
fn recipe_waxed_chiseled_copper() {
    check_recipe("minecraft:waxed_chiseled_copper");
}

#[test]
fn recipe_waxed_chiseled_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_chiseled_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_chiseled_copper_from_waxed_copper_block_stonecutting() {
    check_recipe("minecraft:waxed_chiseled_copper_from_waxed_copper_block_stonecutting");
}

#[test]
fn recipe_waxed_chiseled_copper_from_waxed_cut_copper_stonecutting() {
    check_recipe("minecraft:waxed_chiseled_copper_from_waxed_cut_copper_stonecutting");
}

#[test]
fn recipe_waxed_copper_block_from_honeycomb() {
    check_recipe("minecraft:waxed_copper_block_from_honeycomb");
}

#[test]
fn recipe_waxed_copper_bulb() {
    check_recipe("minecraft:waxed_copper_bulb");
}

#[test]
fn recipe_waxed_copper_bulb_from_honeycomb() {
    check_recipe("minecraft:waxed_copper_bulb_from_honeycomb");
}

#[test]
fn recipe_waxed_copper_door_from_honeycomb() {
    check_recipe("minecraft:waxed_copper_door_from_honeycomb");
}

#[test]
fn recipe_waxed_copper_grate() {
    check_recipe("minecraft:waxed_copper_grate");
}

#[test]
fn recipe_waxed_copper_grate_from_honeycomb() {
    check_recipe("minecraft:waxed_copper_grate_from_honeycomb");
}

#[test]
fn recipe_waxed_copper_grate_from_waxed_copper_block_stonecutting() {
    check_recipe("minecraft:waxed_copper_grate_from_waxed_copper_block_stonecutting");
}

#[test]
fn recipe_waxed_copper_trapdoor_from_honeycomb() {
    check_recipe("minecraft:waxed_copper_trapdoor_from_honeycomb");
}

#[test]
fn recipe_waxed_cut_copper() {
    check_recipe("minecraft:waxed_cut_copper");
}

#[test]
fn recipe_waxed_cut_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_cut_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_cut_copper_from_waxed_copper_block_stonecutting() {
    check_recipe("minecraft:waxed_cut_copper_from_waxed_copper_block_stonecutting");
}

#[test]
fn recipe_waxed_cut_copper_slab() {
    check_recipe("minecraft:waxed_cut_copper_slab");
}

#[test]
fn recipe_waxed_cut_copper_slab_from_honeycomb() {
    check_recipe("minecraft:waxed_cut_copper_slab_from_honeycomb");
}

#[test]
fn recipe_waxed_cut_copper_slab_from_waxed_copper_block_stonecutting() {
    check_recipe("minecraft:waxed_cut_copper_slab_from_waxed_copper_block_stonecutting");
}

#[test]
fn recipe_waxed_cut_copper_slab_from_waxed_cut_copper_stonecutting() {
    check_recipe("minecraft:waxed_cut_copper_slab_from_waxed_cut_copper_stonecutting");
}

#[test]
fn recipe_waxed_cut_copper_stairs() {
    check_recipe("minecraft:waxed_cut_copper_stairs");
}

#[test]
fn recipe_waxed_cut_copper_stairs_from_honeycomb() {
    check_recipe("minecraft:waxed_cut_copper_stairs_from_honeycomb");
}

#[test]
fn recipe_waxed_cut_copper_stairs_from_waxed_copper_block_stonecutting() {
    check_recipe("minecraft:waxed_cut_copper_stairs_from_waxed_copper_block_stonecutting");
}

#[test]
fn recipe_waxed_cut_copper_stairs_from_waxed_cut_copper_stonecutting() {
    check_recipe("minecraft:waxed_cut_copper_stairs_from_waxed_cut_copper_stonecutting");
}

#[test]
fn recipe_waxed_exposed_chiseled_copper() {
    check_recipe("minecraft:waxed_exposed_chiseled_copper");
}

#[test]
fn recipe_waxed_exposed_chiseled_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_exposed_chiseled_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_exposed_chiseled_copper_from_waxed_exposed_copper_stonecutting() {
    check_recipe("minecraft:waxed_exposed_chiseled_copper_from_waxed_exposed_copper_stonecutting");
}

#[test]
fn recipe_waxed_exposed_chiseled_copper_from_waxed_exposed_cut_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_exposed_chiseled_copper_from_waxed_exposed_cut_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_exposed_copper_bulb() {
    check_recipe("minecraft:waxed_exposed_copper_bulb");
}

#[test]
fn recipe_waxed_exposed_copper_bulb_from_honeycomb() {
    check_recipe("minecraft:waxed_exposed_copper_bulb_from_honeycomb");
}

#[test]
fn recipe_waxed_exposed_copper_door_from_honeycomb() {
    check_recipe("minecraft:waxed_exposed_copper_door_from_honeycomb");
}

#[test]
fn recipe_waxed_exposed_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_exposed_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_exposed_copper_grate() {
    check_recipe("minecraft:waxed_exposed_copper_grate");
}

#[test]
fn recipe_waxed_exposed_copper_grate_from_honeycomb() {
    check_recipe("minecraft:waxed_exposed_copper_grate_from_honeycomb");
}

#[test]
fn recipe_waxed_exposed_copper_grate_from_waxed_exposed_copper_stonecutting() {
    check_recipe("minecraft:waxed_exposed_copper_grate_from_waxed_exposed_copper_stonecutting");
}

#[test]
fn recipe_waxed_exposed_copper_trapdoor_from_honeycomb() {
    check_recipe("minecraft:waxed_exposed_copper_trapdoor_from_honeycomb");
}

#[test]
fn recipe_waxed_exposed_cut_copper() {
    check_recipe("minecraft:waxed_exposed_cut_copper");
}

#[test]
fn recipe_waxed_exposed_cut_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_exposed_cut_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_exposed_cut_copper_from_waxed_exposed_copper_stonecutting() {
    check_recipe("minecraft:waxed_exposed_cut_copper_from_waxed_exposed_copper_stonecutting");
}

#[test]
fn recipe_waxed_exposed_cut_copper_slab() {
    check_recipe("minecraft:waxed_exposed_cut_copper_slab");
}

#[test]
fn recipe_waxed_exposed_cut_copper_slab_from_honeycomb() {
    check_recipe("minecraft:waxed_exposed_cut_copper_slab_from_honeycomb");
}

#[test]
fn recipe_waxed_exposed_cut_copper_slab_from_waxed_exposed_copper_stonecutting() {
    check_recipe("minecraft:waxed_exposed_cut_copper_slab_from_waxed_exposed_copper_stonecutting");
}

#[test]
fn recipe_waxed_exposed_cut_copper_slab_from_waxed_exposed_cut_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_exposed_cut_copper_slab_from_waxed_exposed_cut_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_exposed_cut_copper_stairs() {
    check_recipe("minecraft:waxed_exposed_cut_copper_stairs");
}

#[test]
fn recipe_waxed_exposed_cut_copper_stairs_from_honeycomb() {
    check_recipe("minecraft:waxed_exposed_cut_copper_stairs_from_honeycomb");
}

#[test]
fn recipe_waxed_exposed_cut_copper_stairs_from_waxed_exposed_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_exposed_cut_copper_stairs_from_waxed_exposed_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_exposed_cut_copper_stairs_from_waxed_exposed_cut_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_exposed_cut_copper_stairs_from_waxed_exposed_cut_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_oxidized_chiseled_copper() {
    check_recipe("minecraft:waxed_oxidized_chiseled_copper");
}

#[test]
fn recipe_waxed_oxidized_chiseled_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_oxidized_chiseled_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_oxidized_chiseled_copper_from_waxed_oxidized_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_oxidized_chiseled_copper_from_waxed_oxidized_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_oxidized_chiseled_copper_from_waxed_oxidized_cut_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_oxidized_chiseled_copper_from_waxed_oxidized_cut_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_oxidized_copper_bulb() {
    check_recipe("minecraft:waxed_oxidized_copper_bulb");
}

#[test]
fn recipe_waxed_oxidized_copper_bulb_from_honeycomb() {
    check_recipe("minecraft:waxed_oxidized_copper_bulb_from_honeycomb");
}

#[test]
fn recipe_waxed_oxidized_copper_door_from_honeycomb() {
    check_recipe("minecraft:waxed_oxidized_copper_door_from_honeycomb");
}

#[test]
fn recipe_waxed_oxidized_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_oxidized_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_oxidized_copper_grate() {
    check_recipe("minecraft:waxed_oxidized_copper_grate");
}

#[test]
fn recipe_waxed_oxidized_copper_grate_from_honeycomb() {
    check_recipe("minecraft:waxed_oxidized_copper_grate_from_honeycomb");
}

#[test]
fn recipe_waxed_oxidized_copper_grate_from_waxed_oxidized_copper_stonecutting() {
    check_recipe("minecraft:waxed_oxidized_copper_grate_from_waxed_oxidized_copper_stonecutting");
}

#[test]
fn recipe_waxed_oxidized_copper_trapdoor_from_honeycomb() {
    check_recipe("minecraft:waxed_oxidized_copper_trapdoor_from_honeycomb");
}

#[test]
fn recipe_waxed_oxidized_cut_copper() {
    check_recipe("minecraft:waxed_oxidized_cut_copper");
}

#[test]
fn recipe_waxed_oxidized_cut_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_oxidized_cut_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_oxidized_cut_copper_from_waxed_oxidized_copper_stonecutting() {
    check_recipe("minecraft:waxed_oxidized_cut_copper_from_waxed_oxidized_copper_stonecutting");
}

#[test]
fn recipe_waxed_oxidized_cut_copper_slab() {
    check_recipe("minecraft:waxed_oxidized_cut_copper_slab");
}

#[test]
fn recipe_waxed_oxidized_cut_copper_slab_from_honeycomb() {
    check_recipe("minecraft:waxed_oxidized_cut_copper_slab_from_honeycomb");
}

#[test]
fn recipe_waxed_oxidized_cut_copper_slab_from_waxed_oxidized_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_oxidized_cut_copper_slab_from_waxed_oxidized_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_oxidized_cut_copper_slab_from_waxed_oxidized_cut_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_oxidized_cut_copper_slab_from_waxed_oxidized_cut_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_oxidized_cut_copper_stairs() {
    check_recipe("minecraft:waxed_oxidized_cut_copper_stairs");
}

#[test]
fn recipe_waxed_oxidized_cut_copper_stairs_from_honeycomb() {
    check_recipe("minecraft:waxed_oxidized_cut_copper_stairs_from_honeycomb");
}

#[test]
fn recipe_waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_cut_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_oxidized_cut_copper_stairs_from_waxed_oxidized_cut_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_weathered_chiseled_copper() {
    check_recipe("minecraft:waxed_weathered_chiseled_copper");
}

#[test]
fn recipe_waxed_weathered_chiseled_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_weathered_chiseled_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_weathered_chiseled_copper_from_waxed_weathered_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_weathered_chiseled_copper_from_waxed_weathered_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_weathered_chiseled_copper_from_waxed_weathered_cut_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_weathered_chiseled_copper_from_waxed_weathered_cut_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_weathered_copper_bulb() {
    check_recipe("minecraft:waxed_weathered_copper_bulb");
}

#[test]
fn recipe_waxed_weathered_copper_bulb_from_honeycomb() {
    check_recipe("minecraft:waxed_weathered_copper_bulb_from_honeycomb");
}

#[test]
fn recipe_waxed_weathered_copper_door_from_honeycomb() {
    check_recipe("minecraft:waxed_weathered_copper_door_from_honeycomb");
}

#[test]
fn recipe_waxed_weathered_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_weathered_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_weathered_copper_grate() {
    check_recipe("minecraft:waxed_weathered_copper_grate");
}

#[test]
fn recipe_waxed_weathered_copper_grate_from_honeycomb() {
    check_recipe("minecraft:waxed_weathered_copper_grate_from_honeycomb");
}

#[test]
fn recipe_waxed_weathered_copper_grate_from_waxed_weathered_copper_stonecutting() {
    check_recipe("minecraft:waxed_weathered_copper_grate_from_waxed_weathered_copper_stonecutting");
}

#[test]
fn recipe_waxed_weathered_copper_trapdoor_from_honeycomb() {
    check_recipe("minecraft:waxed_weathered_copper_trapdoor_from_honeycomb");
}

#[test]
fn recipe_waxed_weathered_cut_copper() {
    check_recipe("minecraft:waxed_weathered_cut_copper");
}

#[test]
fn recipe_waxed_weathered_cut_copper_from_honeycomb() {
    check_recipe("minecraft:waxed_weathered_cut_copper_from_honeycomb");
}

#[test]
fn recipe_waxed_weathered_cut_copper_from_waxed_weathered_copper_stonecutting() {
    check_recipe("minecraft:waxed_weathered_cut_copper_from_waxed_weathered_copper_stonecutting");
}

#[test]
fn recipe_waxed_weathered_cut_copper_slab() {
    check_recipe("minecraft:waxed_weathered_cut_copper_slab");
}

#[test]
fn recipe_waxed_weathered_cut_copper_slab_from_honeycomb() {
    check_recipe("minecraft:waxed_weathered_cut_copper_slab_from_honeycomb");
}

#[test]
fn recipe_waxed_weathered_cut_copper_slab_from_waxed_weathered_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_weathered_cut_copper_slab_from_waxed_weathered_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_weathered_cut_copper_slab_from_waxed_weathered_cut_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_weathered_cut_copper_slab_from_waxed_weathered_cut_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_weathered_cut_copper_stairs() {
    check_recipe("minecraft:waxed_weathered_cut_copper_stairs");
}

#[test]
fn recipe_waxed_weathered_cut_copper_stairs_from_honeycomb() {
    check_recipe("minecraft:waxed_weathered_cut_copper_stairs_from_honeycomb");
}

#[test]
fn recipe_waxed_weathered_cut_copper_stairs_from_waxed_weathered_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_weathered_cut_copper_stairs_from_waxed_weathered_copper_stonecutting",
    );
}

#[test]
fn recipe_waxed_weathered_cut_copper_stairs_from_waxed_weathered_cut_copper_stonecutting() {
    check_recipe(
        "minecraft:waxed_weathered_cut_copper_stairs_from_waxed_weathered_cut_copper_stonecutting",
    );
}

#[test]
fn recipe_wayfinder_armor_trim_smithing_template() {
    check_recipe("minecraft:wayfinder_armor_trim_smithing_template");
}

#[test]
fn recipe_wayfinder_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:wayfinder_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_weathered_chiseled_copper() {
    check_recipe("minecraft:weathered_chiseled_copper");
}

#[test]
fn recipe_weathered_chiseled_copper_from_weathered_copper_stonecutting() {
    check_recipe("minecraft:weathered_chiseled_copper_from_weathered_copper_stonecutting");
}

#[test]
fn recipe_weathered_chiseled_copper_from_weathered_cut_copper_stonecutting() {
    check_recipe("minecraft:weathered_chiseled_copper_from_weathered_cut_copper_stonecutting");
}

#[test]
fn recipe_weathered_copper_bulb() {
    check_recipe("minecraft:weathered_copper_bulb");
}

#[test]
fn recipe_weathered_copper_grate() {
    check_recipe("minecraft:weathered_copper_grate");
}

#[test]
fn recipe_weathered_copper_grate_from_weathered_copper_stonecutting() {
    check_recipe("minecraft:weathered_copper_grate_from_weathered_copper_stonecutting");
}

#[test]
fn recipe_weathered_cut_copper() {
    check_recipe("minecraft:weathered_cut_copper");
}

#[test]
fn recipe_weathered_cut_copper_from_weathered_copper_stonecutting() {
    check_recipe("minecraft:weathered_cut_copper_from_weathered_copper_stonecutting");
}

#[test]
fn recipe_weathered_cut_copper_slab() {
    check_recipe("minecraft:weathered_cut_copper_slab");
}

#[test]
fn recipe_weathered_cut_copper_slab_from_weathered_copper_stonecutting() {
    check_recipe("minecraft:weathered_cut_copper_slab_from_weathered_copper_stonecutting");
}

#[test]
fn recipe_weathered_cut_copper_slab_from_weathered_cut_copper_stonecutting() {
    check_recipe("minecraft:weathered_cut_copper_slab_from_weathered_cut_copper_stonecutting");
}

#[test]
fn recipe_weathered_cut_copper_stairs() {
    check_recipe("minecraft:weathered_cut_copper_stairs");
}

#[test]
fn recipe_weathered_cut_copper_stairs_from_weathered_copper_stonecutting() {
    check_recipe("minecraft:weathered_cut_copper_stairs_from_weathered_copper_stonecutting");
}

#[test]
fn recipe_weathered_cut_copper_stairs_from_weathered_cut_copper_stonecutting() {
    check_recipe("minecraft:weathered_cut_copper_stairs_from_weathered_cut_copper_stonecutting");
}

#[test]
fn recipe_wheat() {
    check_recipe("minecraft:wheat");
}

#[test]
fn recipe_white_banner() {
    check_recipe("minecraft:white_banner");
}

#[test]
fn recipe_white_bed() {
    check_recipe("minecraft:white_bed");
}

#[test]
fn recipe_white_bundle() {
    check_recipe("minecraft:white_bundle");
}

#[test]
fn recipe_white_candle() {
    check_recipe("minecraft:white_candle");
}

#[test]
fn recipe_white_carpet() {
    check_recipe("minecraft:white_carpet");
}

#[test]
fn recipe_white_concrete_powder() {
    check_recipe("minecraft:white_concrete_powder");
}

#[test]
fn recipe_white_dye() {
    check_recipe("minecraft:white_dye");
}

#[test]
fn recipe_white_dye_from_lily_of_the_valley() {
    check_recipe("minecraft:white_dye_from_lily_of_the_valley");
}

#[test]
fn recipe_white_glazed_terracotta() {
    check_recipe("minecraft:white_glazed_terracotta");
}

#[test]
fn recipe_white_harness() {
    check_recipe("minecraft:white_harness");
}

#[test]
fn recipe_white_shulker_box() {
    check_recipe("minecraft:white_shulker_box");
}

#[test]
fn recipe_white_stained_glass() {
    check_recipe("minecraft:white_stained_glass");
}

#[test]
fn recipe_white_stained_glass_pane() {
    check_recipe("minecraft:white_stained_glass_pane");
}

#[test]
fn recipe_white_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:white_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_white_terracotta() {
    check_recipe("minecraft:white_terracotta");
}

#[test]
fn recipe_white_wool_from_string() {
    check_recipe("minecraft:white_wool_from_string");
}

#[test]
fn recipe_wild_armor_trim_smithing_template() {
    check_recipe("minecraft:wild_armor_trim_smithing_template");
}

#[test]
fn recipe_wild_armor_trim_smithing_template_smithing_trim() {
    check_recipe("minecraft:wild_armor_trim_smithing_template_smithing_trim");
}

#[test]
fn recipe_wind_charge() {
    check_recipe("minecraft:wind_charge");
}

#[test]
fn recipe_wolf_armor() {
    check_recipe("minecraft:wolf_armor");
}

#[test]
fn recipe_wooden_axe() {
    check_recipe("minecraft:wooden_axe");
}

#[test]
fn recipe_wooden_hoe() {
    check_recipe("minecraft:wooden_hoe");
}

#[test]
fn recipe_wooden_pickaxe() {
    check_recipe("minecraft:wooden_pickaxe");
}

#[test]
fn recipe_wooden_shovel() {
    check_recipe("minecraft:wooden_shovel");
}

#[test]
fn recipe_wooden_sword() {
    check_recipe("minecraft:wooden_sword");
}

#[test]
fn recipe_writable_book() {
    check_recipe("minecraft:writable_book");
}

#[test]
fn recipe_yellow_banner() {
    check_recipe("minecraft:yellow_banner");
}

#[test]
fn recipe_yellow_bed() {
    check_recipe("minecraft:yellow_bed");
}

#[test]
fn recipe_yellow_bundle() {
    check_recipe("minecraft:yellow_bundle");
}

#[test]
fn recipe_yellow_candle() {
    check_recipe("minecraft:yellow_candle");
}

#[test]
fn recipe_yellow_carpet() {
    check_recipe("minecraft:yellow_carpet");
}

#[test]
fn recipe_yellow_concrete_powder() {
    check_recipe("minecraft:yellow_concrete_powder");
}

#[test]
fn recipe_yellow_dye_from_dandelion() {
    check_recipe("minecraft:yellow_dye_from_dandelion");
}

#[test]
fn recipe_yellow_dye_from_sunflower() {
    check_recipe("minecraft:yellow_dye_from_sunflower");
}

#[test]
fn recipe_yellow_dye_from_wildflowers() {
    check_recipe("minecraft:yellow_dye_from_wildflowers");
}

#[test]
fn recipe_yellow_glazed_terracotta() {
    check_recipe("minecraft:yellow_glazed_terracotta");
}

#[test]
fn recipe_yellow_harness() {
    check_recipe("minecraft:yellow_harness");
}

#[test]
fn recipe_yellow_shulker_box() {
    check_recipe("minecraft:yellow_shulker_box");
}

#[test]
fn recipe_yellow_stained_glass() {
    check_recipe("minecraft:yellow_stained_glass");
}

#[test]
fn recipe_yellow_stained_glass_pane() {
    check_recipe("minecraft:yellow_stained_glass_pane");
}

#[test]
fn recipe_yellow_stained_glass_pane_from_glass_pane() {
    check_recipe("minecraft:yellow_stained_glass_pane_from_glass_pane");
}

#[test]
fn recipe_yellow_terracotta() {
    check_recipe("minecraft:yellow_terracotta");
}
