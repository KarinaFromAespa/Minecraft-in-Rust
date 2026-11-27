//! `Creaking.canBeNameTagged` (1.21.4 only): a creaking bound to its heart
//! cannot be named -- `NameTagItem.interactLivingEntity` checks
//! `canBeNameTagged`, and 1.21.5 dropped that check, so a heart-bound
//! creaking takes a name tag there but not here.
use super::*;
use horizon_core::geometry::Vec3;
use horizon_world::nbt::Tag;

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
    game
}

fn named_tag() -> ItemStack {
    let mut tag = ItemStack::new("minecraft:name_tag", 1);
    tag.data_mut().custom_name = Some(Tag::String("Boo".into()));
    tag
}

fn creaking(game: &Game, id: u32, home: Option<[i32; 3]>) -> Mob {
    let kind = game
        .res
        .mob_types()
        .get("minecraft:creaking")
        .unwrap()
        .clone();
    let mut mob = Mob::new(id, kind, Vec3::new(0.0, 64.0, 0.0), 0.0);
    mob.extra.home = home;
    mob
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn a_heart_bound_creaking_refuses_a_name_tag() {
    let mut game = game();
    game.inventory[0] = Some(named_tag());
    game.mobs.push(creaking(&game, 1, Some([0, 64, 0])));
    assert!(!game.name_mob(0), "bound to its heart, it cannot be named");
    assert_eq!(game.mobs[0].custom_name, None);
    assert_eq!(game.inventory[0].as_ref().map(|s| s.count), Some(1));
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn a_free_creaking_takes_a_name_tag() {
    let mut game = game();
    game.inventory[0] = Some(named_tag());
    game.mobs.push(creaking(&game, 1, None));
    assert!(game.name_mob(0));
    assert_eq!(game.mobs[0].custom_name.as_deref(), Some("Boo"));
}
