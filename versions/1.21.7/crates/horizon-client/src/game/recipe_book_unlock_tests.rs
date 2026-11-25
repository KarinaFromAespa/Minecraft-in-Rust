//! Vanilla 1.21.7 recipe-book behaviour, pinned: no recipe unlocks on
//! ingredient pickup, a recipe unlocks on first craft, and the inventory's
//! book (2x2) lists only recipes that fit its grid while the crafting
//! table's book (3x3) lists all unlocked recipes — so the oak boat, a 3x2
//! shaped recipe, never appears in the inventory book. Verified against the
//! 1.21.7 jars: `ServerRecipeBook.addRecipes` is called only from
//! `ServerPlayer.awardRecipes` (craft, smelt, knowledge book, advancement
//! rewards, /recipe) — never on pickup — and the client's
//! `RecipeBookComponent.updateCollections` drops every collection with no
//! fitting recipe (`RecipeCollection.hasFitting`) in both the craftable-only
//! and show-all modes. The 1.21.7 data agrees: 1,395 of the 1,396 recipe
//! advancements need a `recipe_unlocked` criterion, and none grants a recipe
//! on `inventory_changed` alone.
use super::super::*;
use super::*;
use horizon_client::{chunk::Chunk, motion::PlayerMotion};
use std::sync::Arc;

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
    game.world = World::new(game.res.facts().air, 1);
    game.world
        .chunks
        .insert((0, 0), Arc::new(Chunk::empty(0, 0)));
    game.player = Some(PlayerMotion::new(game.motion, Vec3::new(0.5, 64.0, 0.5)).unwrap());
    game
}

fn known(game: &Game<'_>, id: &str) -> bool {
    game.level.recipe_book.recipes.contains(id)
}

/// The recipe ids the open book lists, for the given grid size, with the
/// craftable-only filter off (show all).
fn book_recipe_ids(game: &mut Game<'_>, craft_size: usize) -> Vec<String> {
    game.craft_size = craft_size;
    game.level.recipe_book.filtering[BookType::Crafting.index()] = false;
    game.refresh_book_view(BookType::Crafting);
    let ui = &game.recipe_ui;
    let book = ui.book.as_ref().expect("book built");
    ui.view
        .buttons
        .iter()
        .flat_map(|b| b.entries.iter())
        .map(|(i, _)| book.entries[*i].id.clone())
        .collect()
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn picking_up_ingredients_unlocks_no_recipe() {
    let mut game = game();
    // Oak planks in hand, as the owner had them: the boat must stay locked.
    game.inventory[0] = Some(ItemStack::new("minecraft:oak_planks", 5));
    game.check_inventory_trigger();
    assert!(
        !known(&game, "minecraft:oak_boat"),
        "holding planks must not unlock the boat"
    );
    assert!(
        !known(&game, "minecraft:oak_planks"),
        "holding planks must not unlock the planks recipe either"
    );
    // An oak log too: the planks advancement's `inventory_changed` criterion
    // is now met, but the recipe still needs its `recipe_unlocked` trigger,
    // which only a craft (or smelt, knowledge book, /recipe) fires.
    game.inventory[1] = Some(ItemStack::new("minecraft:oak_log", 1));
    game.check_inventory_trigger();
    assert!(
        !known(&game, "minecraft:oak_planks"),
        "holding a log must not unlock the planks recipe"
    );
    assert!(
        !known(&game, "minecraft:oak_boat"),
        "holding a log must not unlock the boat"
    );
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn crafting_the_boat_once_unlocks_it() {
    let mut game = game();
    assert!(!known(&game, "minecraft:oak_boat"));
    // The five planks of the boat's U shape, as the grid holds them.
    let plank = || Some(ItemStack::new("minecraft:oak_planks", 1));
    game.award_crafted(&[(
        "minecraft:oak_boat".to_owned(),
        vec![
            plank(),
            None,
            plank(),
            plank(),
            plank(),
            plank(),
            None,
            None,
            None,
        ],
    )]);
    assert!(
        known(&game, "minecraft:oak_boat"),
        "crafting the boat once must unlock it"
    );
    // Crafting the boat does not unlock anything else.
    assert!(!known(&game, "minecraft:oak_planks"));
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn the_inventory_book_lists_only_recipes_that_fit_its_grid() {
    let mut game = game();
    // Unlock the boat and the planks by crafting each once.
    let plank = || Some(ItemStack::new("minecraft:oak_planks", 1));
    let log = || Some(ItemStack::new("minecraft:oak_log", 1));
    game.award_crafted(&[(
        "minecraft:oak_boat".to_owned(),
        vec![
            plank(),
            None,
            plank(),
            plank(),
            plank(),
            plank(),
            None,
            None,
            None,
        ],
    )]);
    game.award_crafted(&[("minecraft:oak_planks".to_owned(), vec![log()])]);
    assert!(known(&game, "minecraft:oak_boat"));
    assert!(known(&game, "minecraft:oak_planks"));
    // Inventory (2x2): the 3-wide boat does not fit and is not listed; the
    // 1x1 planks fit and are.
    let inventory = book_recipe_ids(&mut game, 2);
    assert!(
        !inventory.contains(&"minecraft:oak_boat".to_owned()),
        "the inventory book must not list the 3-wide boat: {inventory:?}"
    );
    assert!(
        inventory.contains(&"minecraft:oak_planks".to_owned()),
        "the inventory book must list the planks recipe"
    );
    // Crafting table (3x3): the boat fits and is listed.
    let table = book_recipe_ids(&mut game, 3);
    assert!(
        table.contains(&"minecraft:oak_boat".to_owned()),
        "the crafting table book must list the boat"
    );
    assert!(
        table.contains(&"minecraft:oak_planks".to_owned()),
        "the crafting table book must list the planks recipe"
    );
}
