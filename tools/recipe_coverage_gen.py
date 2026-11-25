#!/usr/bin/env python3
"""Generate the per-recipe coverage test module for one Minecraft version.

Reads every recipe in a version's server jar (`data/minecraft/recipe/*.json`)
and emits a Rust test module with one #[test] per recipe. Each test loads the
port's Recipes (and StationRecipes) from the prepared resources and asserts
that one vanilla recipe exists in the port and crafts, cooks, cuts or smiths
to the right result, so a failure names the exact recipe id. The missing list
is the deliverable: the tests assert, they do not fix.

Usage:
    python3 tools/recipe_coverage_gen.py \
        --jar ~/workspace/mcir-work/leakage-audit/jars/1.21.7-server-named.jar \
        --version 1.21.7 \
        --out versions/1.21.7/crates/horizon-client/src/recipe_coverage_tests.rs
"""

import argparse
import json
import re
import zipfile

HEADER = """//! One test per vanilla {version} recipe, generated from the {version} server
//! jar's `data/minecraft/recipe/*.json` by `tools/recipe_coverage_gen.py`.
//! Each test loads the port's `Recipes` (and `StationRecipes`) from the
//! prepared resources and asserts that one vanilla recipe exists in the port
//! and crafts, cooks, cuts or smiths to the right result. A failure names the
//! exact recipe id; the missing-recipe list is the deliverable, so these tests
//! assert and do not fix. Tests skip (they return without asserting) when
//! `HORIZON_RESOURCES` is unset or the resources are absent, as the level
//! tests do.
//!
//! Generated: {count} recipes ({breakdown}).
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
        panic!("{id}: recipe JSON missing from prepared resources at {}", path.display());
    };
    let v: Value = serde_json::from_slice(&bytes).expect("recipe JSON parses");
    let ty = v["type"].as_str().unwrap_or("");
    match ty {
        "minecraft:crafting_shaped" => check_shaped(fx, id, &v),
        "minecraft:crafting_shapeless" => check_shapeless(fx, id, &v),
        "minecraft:smelting" | "minecraft:blasting" | "minecraft:smoking"
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
    assert_same_id(id, found_id);
    let (want_item, want_count) =
        result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
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
    assert_same_id(id, found_id);
    let (want_item, want_count) =
        result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
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
    let (want_item, want_count) =
        result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
    assert_eq!(c.result.item.as_ref(), want_item, "{id}: result item");
    assert_eq!(c.result.count, want_count, "{id}: result count");
}

fn check_stonecutting(fx: &Fixture, id: &str, v: &Value) {
    let item = first_item(&v["ingredient"], &fx.items)
        .unwrap_or_else(|| panic!("{id}: no item for ingredient {}", v["ingredient"]));
    let stack = ItemStack::new(&item, 1);
    let found = fx.stations.stonecutter(&stack);
    let (want_item, want_count) =
        result_id_count(v).unwrap_or_else(|| panic!("{id}: no result"));
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

"""

TEST_FN = """#[test]
fn {fn_name}() {{
    check_recipe("{id}");
}}

"""


def fn_name(bare: str) -> str:
    return "recipe_" + re.sub(r"[^a-z0-9]+", "_", bare)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--jar", required=True)
    ap.add_argument("--version", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    with zipfile.ZipFile(args.jar) as z:
        names = sorted(
            n
            for n in z.namelist()
            if n.startswith("data/minecraft/recipe/") and n.endswith(".json")
        )
        recipes = []
        for n in names:
            d = json.loads(z.read(n))
            bare = n[len("data/minecraft/recipe/") : -len(".json")]
            recipes.append((f"minecraft:{bare}", d.get("type", "?")))

    seen: dict[str, str] = {}
    for rid, _ in recipes:
        bare = rid[len("minecraft:") :]
        name = fn_name(bare)
        if name in seen:
            raise SystemExit(f"fn name collision: {name} for {rid} and {seen[name]}")
        seen[name] = rid

    from collections import Counter

    breakdown = ", ".join(
        f"{c} {t.removeprefix('minecraft:')}"
        for t, c in sorted(Counter(t for _, t in recipes).items())
    )
    header = (
        HEADER.replace("{version}", args.version)
        .replace("{count}", str(len(recipes)))
        .replace("{breakdown}", breakdown)
    )
    with open(args.out, "w") as f:
        f.write(header)
        for rid, _ in recipes:
            bare = rid[len("minecraft:") :]
            f.write(TEST_FN.format(fn_name=fn_name(bare), id=rid))
    print(f"wrote {args.out}: {len(recipes)} recipes")


if __name__ == "__main__":
    main()
