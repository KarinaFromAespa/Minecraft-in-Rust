# Recipe coverage — every vanilla 1.21.7 recipe, per recipe

The owner reported the boat recipe missing from the 1.21.7 playtest
(build-12). The boat is not missing: manual crafting works, the recipe
unlocks on first craft, and the inventory book hides it because it is a
3-wide recipe in a 2x2 grid — all vanilla-correct (see "The boat" below).
The investigation built a permanent check instead: one unit test per
vanilla 1.21.7 recipe, asserting each exists in the port and crafts,
cooks, cuts or smiths to the right result. That suite found a real gap
the boat report was standing next to: **11 special recipes were absent
from the port's crafting tables entirely** — fixed in
`fix/1.21.7-special-recipes` (see "Missing recipes", now fixed).

## The boat — vanilla-correct, no defect

Verified against the 1.21.7 jars (server jar named classes, client jar
bytecode, and the 1.21.7 data), not against the 26.3 base:

- **Grid craft works.** `minecraft:oak_boat` is a shaped recipe
  (`["# #", "###"]`, `#` = `minecraft:oak_planks`, 1 boat). The port
  loads it and the U shape crafts it (coverage test
  `recipe_oak_boat` passes; the owner also built one manually).
- **Unlock on first craft, not on pickup.** `ServerRecipeBook.addRecipes`
  is called only from `ServerPlayer.awardRecipes`, whose callers are
  `RecipeCraftingHolder` (craft), `AbstractFurnaceBlockEntity` (smelt),
  `KnowledgeBookItem`, `AdvancementRewards`, `RecipeCommand`, and
  `ServerPlayer` itself — never on pickup. The data agrees: 1,395 of the
  1,396 recipe advancements need a `recipe_unlocked` criterion, and none
  grants a recipe on `inventory_changed` alone. The boat's own
  advancement (`recipes/transportation/oak_boat`) needs `has_the_recipe`
  (recipe_unlocked) *and* `in_water`, so it can only complete after the
  recipe is already known. The port mirrors this: `award_crafted`
  (`game/inventory_slots.rs`) adds the crafted recipe to the book and
  fires `recipe_unlocked`.
- **The inventory book hides 3-wide recipes.** The 1.21.7 client's
  `RecipeBookComponent.updateCollections` always removes every
  collection with no fitting recipe (`RecipeCollection.hasFitting`), in
  both the craftable-only and show-all modes; fitting is computed
  against the menu's grid. The boat (3x2) has no fitting recipe in a
  2x2 inventory grid, so the inventory book never lists it, while the
  crafting table's book (3x3) lists it once unlocked. The port mirrors
  this (`Entry::fits` + the `selected` filter in `build_view`).
- **"Show all" shows known fitting recipes only.** The craftable-only
  toggle adds a second filter (`hasCraftable`); with it off, the book
  lists every known recipe that fits the grid, craftable or not (the
  red-bordered entries the owner saw). Unknown recipes never appear in
  either mode. The creative search tab is not unlock-gated in the port
  (its items come from the creative tabs, filtered only by operator
  permissions), so the "now shows in the listing" the owner saw after
  crafting was the recipe book's search tab, which lists known recipes.

Pinned by `game/recipe_book_unlock_tests.rs` (three `#[ignore]` game
tests): `picking_up_ingredients_unlocks_no_recipe`,
`crafting_the_boat_once_unlocks_it`, and
`the_inventory_book_lists_only_recipes_that_fit_its_grid`. They compile
(verified: the `horizon-client` lib test binary lists and builds them),
but they do **not** run on the VM: the fixture's `load_resources` needs
a fully prepared 1.21.7 resources dir, and the VM has none — the only
prepared data here is the 125 MB level-test subset (which lacks fonts,
models and the prepare-generated tables). Strace of the failing run
shows the fatal open is `occlusion.nbt` (opened with `?` by
`BlockTableFile::open` in `light.rs:136` / `models.rs:162`), a file
only `horizon-prepare`'s `VanillaFaceVisibilityOracle` export produces;
no `occlusion.nbt` exists anywhere on the VM. Run them on the laptop,
where the live prepared 1.21.7 `.local` exists:
`HORIZON_TEST_RESOURCES=<prepared 1.21.7 resources> cargo test -p horizon-client --lib recipe_book_unlock -- --ignored`.
Do not weaken the fixture to make them pass on the subset.

## The suite

- Generator: `tools/recipe_coverage_gen.py` — reads the 1.21.7 server
  jar's `data/minecraft/recipe/*.json` and emits the test module.
  Regenerate with:
  `python3 tools/recipe_coverage_gen.py --jar <1.21.7-server-named.jar> --version 1.21.7 --out versions/1.21.7/crates/horizon-client/src/recipe_coverage_tests.rs`
- Tests: `versions/1.21.7/crates/horizon-client/src/recipe_coverage_tests.rs`
  — one `#[test]` per recipe, generated from the jar list, so a failure
  names the exact recipe id. Each test loads the port's `Recipes` (and
  `StationRecipes`) from the prepared resources and checks one recipe:
  shaped/shapeless and transmute by crafting from the JSON's ingredients
  (a `#tag` resolves to its first item in the port's tag order),
  smelting/blasting/smoking/campfire by `cook`, stonecutting by the
  station recipes, smithing by `smithing_recipe`, specials by presence
  in the tables. Tests skip (do not fail) when `HORIZON_RESOURCES` is
  unset or the resources are absent, as the level tests do.
- Run (needs prepared 1.21.7 resources):
  `HORIZON_RESOURCES=<resources> cargo test -p horizon-client-lib --lib recipe_coverage`
  The tests live in `crates/horizon-client/src/`, so they run as
  `horizon-client-lib`, not `horizon-client` (see
  `tools/defect_suite_rust.json`'s note). In CI (`crate-tests.yml`)
  they compile and skip, as the level tests do; the real run is local.

## Counts (1.21.7, fix `fix/1.21.7-special-recipes`)

**Total 1,407 recipes: 1,407 pass, 0 missing.**

| type | count | pass | missing |
|---|---|---|---|
| crafting_shaped | 673 | 673 | 0 |
| crafting_shapeless | 295 | 295 | 0 |
| stonecutting | 254 | 254 | 0 |
| smelting | 72 | 72 | 0 |
| blasting | 24 | 24 | 0 |
| smoking | 9 | 9 | 0 |
| campfire_cooking | 9 | 9 | 0 |
| smithing_transform | 9 | 9 | 0 |
| smithing_trim | 18 | 18 | 0 |
| crafting_transmute | 32 | 32 | 0 |
| crafting_special_* / decorated_pot | 12 | 12 | 0 |

All 10 boat/raft recipes and all 10 chest boat/raft recipes pass.

## Missing recipes — fixed in `fix/1.21.7-special-recipes`

All 11 were special recipes whose JSONs in 1.21.7 carry only `type` and
`category` — no ingredients, no result. Vanilla hardcodes these
recipes' inputs in their classes (`SpecialRecipe` subclasses); the
port's loader instead read the ingredients from the JSON, got
nothing, and dropped the recipe from `Recipes.crafting`, so the grid
could not craft them and the book could not list them. The `Special`
logic itself exists and is unit-tested (e.g. banner duplication, shield
decoration in `crafting.rs` tests); only the loading was missing.

The fix is in the loader (`crafting.rs`, `from_sources`): where a
special's JSON names a field, the data-driven value stands; where it
does not, the loader falls back to vanilla 1.21.7's hardcoded input.
Every fallback was verified against the 1.21.7 server jar
(`javap` on the named `*Recipe` classes) and cross-checked against
the data-driven 26.2 JSONs for the same recipes, which carry the
fields the loader arms read:

- `minecraft:armor_dye` — new arm for `crafting_special_armordye`
  (the loader's `crafting_dye` arm is a different type id):
  target `#minecraft:dyeable`, dye the sixteen `*_dye` items
  (1.21.7 has no `#minecraft:dyes` tag), result the dyed target
  itself (no fixed item).
- `minecraft:banner_duplicate` — banner `#minecraft:banners`, result
  `minecraft:white_banner` (first of the tag).
- `minecraft:book_cloning` — source `minecraft:written_book`,
  material `#minecraft:book_cloning_target` (`minecraft:writable_book`),
  result `minecraft:written_book`; generations 0..1 (the arm's
  default — vanilla's `tryCraftCopy` refuses generation 2+).
- `minecraft:decorated_pot` — all four sides
  `#minecraft:decorated_pot_ingredients` (brick + sherds), result
  `minecraft:decorated_pot`.
- `minecraft:firework_rocket` — shell `minecraft:paper`, fuel
  `minecraft:gunpowder`, star `minecraft:firework_star`, result
  `minecraft:firework_rocket` ×3 (vanilla assembles 3, not 1).
- `minecraft:firework_star` — dye the sixteen dyes, fuel
  `minecraft:gunpowder`, trail `minecraft:diamond`, twinkle
  `minecraft:glowstone_dust`, shapes from vanilla's `SHAPE_BY_ITEM`
  (large_ball `minecraft:fire_charge`, burst `minecraft:feather`,
  star `minecraft:gold_nugget`, creeper `#minecraft:skulls`), result
  `minecraft:firework_star`.
- `minecraft:firework_star_fade` — dye the sixteen dyes, target
  `minecraft:firework_star`, result `minecraft:firework_star`.
- `minecraft:map_cloning` — new arm for `crafting_special_mapcloning`:
  constructs the `Transmute` 26.2 made of it — input
  `minecraft:filled_map`, material `minecraft:map`, 1..8 materials,
  count added to the result (vanilla: one filled map, 1..8 empty maps,
  copies = empties + 1).
- `minecraft:map_extending` — map `minecraft:filled_map`, material
  `minecraft:paper`, result `minecraft:filled_map`.
- `minecraft:shield_decoration` — banner `#minecraft:banners`, target
  `minecraft:shield`, result `minecraft:shield`.
- `minecraft:tipped_arrow` — new arm for `crafting_special_tippedarrow`:
  constructs the `Imbue` 26.2 made of it — source
  `minecraft:lingering_potion`, material `minecraft:arrow`, result
  `minecraft:tipped_arrow` ×8 (vanilla: 8 arrows around one lingering
  potion, potion contents copied).

The one special that already passed is `minecraft:repair_item`
(`crafting_special_repairitem`), whose arm needs no JSON fields
(`Some(Special::RepairItem)`).

Known limits of this fix (loading only; the `Special` assemble
logic is untouched):

- `BannerDuplicate`'s assemble stamps the loaded `result` item id
  onto the copy, so a non-white patterned banner crafts into a white
  banner carrying the patterns — vanilla copies the source banner
  itself. The loader can only name one item; making assemble keep
  the source's item is a separate change to `Special::assemble`.
- Only `Transmute`, `Dye` and `Imbue` have recipe-book displays
  (`Special::book_displays`); the other eight specials load and craft
  but the book lists no entry for them. Book displays for those eight
  are a separate change.

## Full-suite check (2026-11-25, same tree)

- `cargo test -p horizon-client-lib --lib`: 3,870 pass, 13 fail, 92
  ignored. The 13 are the 11 specials above, plus
  `random_tick_sections::tick_parity::actual_random_ticks_match_a_block_scan`
  (pre-existing: fails identically on base `74109ffa` with my changes
  stashed, a worldgen/random-tick assertion at
  `random_tick_sections.rs:619`, nothing to do with recipes) and
  `save::tests::points_of_interest_save_as_poi_sections` (passes in
  isolation; flaky under parallel load).
- `cargo test -p horizon-client --lib`: 1,272 pass, 1 fail, 480
  ignored. The 1 is
  `game::maps::tests::map_on_banner_in_adventure_mode_honours_can_place_on`,
  whose fixture panics on the same missing `occlusion.nbt` as the
  unlock tests (file untouched by this branch).
- The 3 unlock tests are `#[ignore]`d, so neither suite runs them;
  they need the laptop run above.

Done in `fix/1.21.7-special-recipes` as described above (fallback
inputs in the loader; the loader code is shared across version
trees, so the same gap likely exists wherever a version's special
JSONs are minimal — check before assuming any version crafts them).
The pre-fix counts were 1,396 pass / 11 missing (main `74109ffa`,
run 2026-11-25).
