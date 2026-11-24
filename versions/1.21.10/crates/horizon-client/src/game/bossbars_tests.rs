//! `/bossbar add` in 1.21.10: `CustomBossEvents.create` takes no random —
//! `ServerBossEvent` draws the bar's UUID from `Mth.RANDOM` — so creating
//! a bar must not advance the level random (26.1 drew it from
//! `level.random`), while the bar still gets a fresh UUID.
use super::*;
use horizon_core::random::RandomSource;

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
    game
}

#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn adding_a_boss_bar_leaves_the_level_random_untouched() {
    let mut game = game();
    let mut before = game.world.random.clone();
    assert!(game.run_command("bossbar add minecraft:test {\"text\":\"Test\"}"));
    let bar = game
        .bosses
        .bars
        .get("minecraft:test")
        .expect("the bar was created");
    assert_ne!(
        bar.uuid,
        horizon_client::bossbar::stable_uuid("minecraft:test"),
        "the bar gets a fresh UUID, not the load-time stable one"
    );
    let mut after = game.world.random.clone();
    assert_eq!(
        before.next_long(),
        after.next_long(),
        "creating a boss bar must not consume the level random"
    );
}
