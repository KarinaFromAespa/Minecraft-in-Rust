//! #13076: `NeutralMob`'s saved anger countdown through the real summon, save
//! and tick paths.

use super::h262_enderman_anger_tests::with_game;
use super::*;

/// #13076: `NeutralMob.addPersistentAngerSaveData` writes the remaining
/// ticks as `readPersistentAngerSaveData` read them, and a mob that missed
/// ticks while the clock went on keeps them: the countdown freezes while
/// the mob does not tick.
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn h13076_loaded_anger_countdown_is_kept() {
    with_game(|game| {
        game.world.game_time = 100;
        assert!(game.run_command("gamerule spawn_mobs false"));
        let mut summon = |n: i32, kind: &str, anger: i32, no_ai: bool| {
            let cmd = format!(
                "summon {kind} {}.5 200 4.5 {{UUID:[I;262,2,0,{n}],NoAI:{}b,NoGravity:1b,PersistenceRequired:1b,AngerTime:{anger}}}",
                4 + n,
                u8::from(no_ai)
            );
            assert!(game.run_command(&cmd), "{cmd}");
            game.persist
                .uuids
                .iter()
                .find_map(|(id, u)| (*u == [262, 2, 0, n]).then_some(*id))
                .unwrap()
        };
        let calm = summon(1, "wolf", 0, true);
        let past = summon(2, "iron_golem", -5, true);
        let future = summon(3, "iron_golem", 999_999, false);
        let saved = |game: &mut Game<'_>, id: u32| {
            let i = game.mobs.iter().position(|m| m.id == id).unwrap();
            game.mob_tree_nbt(i).get("AngerTime").cloned()
        };
        for (id, anger) in [(calm, 0), (past, -5), (future, 999_999)] {
            assert_eq!(saved(game, id), Some(Tag::Int(anger)), "as loaded");
        }
        // Five ticks of the clock in which the mobs do not tick.
        game.world.game_time += 5;
        for (id, anger) in [(calm, 0), (past, -5), (future, 999_999)] {
            assert_eq!(saved(game, id), Some(Tag::Int(anger)), "unticked");
        }
        // Then real ticks.
        for _ in 0..2 {
            game.world.game_time += 1;
            game.update_chunk_tickets();
            assert!(game.entity_ticking_area()(6, 4));
            game.tick_creature_entities().unwrap();
        }
        for (id, anger) in [(calm, 0), (past, -5)] {
            assert_eq!(saved(game, id), Some(Tag::Int(anger)), "ticked");
        }
        let i = game.mobs.iter().position(|m| m.id == future).unwrap();
        // The countdown counted only its own two ticks, not the five it
        // missed: 999999 - 2, not 999999 - 7.
        assert_eq!(game.mobs[i].extra.anger, 999_997);
        assert_eq!(saved(game, future), Some(Tag::Int(999_997)));
    });
}
