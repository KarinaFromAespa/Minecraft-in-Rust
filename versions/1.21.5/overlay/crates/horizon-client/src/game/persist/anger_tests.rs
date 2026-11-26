//! #13076: `NeutralMob`'s saved anger time through the real summon, save
//! and tick paths (1.21.5 saves the time left, `AngerTime`, from the
//! absolute end time the model keeps).

use super::h262_enderman_absolute_anger_tests::with_game;
use super::*;

/// #13076: `NeutralMob.addPersistentAngerSaveData` writes the time left as
/// `readPersistentAngerSaveData` read it, past ones included, and a mob
/// that missed ticks while the clock went on does not move its end time
/// forward — the saved time shrinks instead.
#[test]
#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]
fn h13076_loaded_anger_time_is_kept() {
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
        let past = summon(1, "wolf", -5, true);
        let zero = summon(2, "iron_golem", 0, true);
        let future = summon(3, "iron_golem", 999899, false);
        let saved = |game: &mut Game<'_>, id: u32| {
            let i = game.mobs.iter().position(|m| m.id == id).unwrap();
            game.mob_tree_nbt(i).get("AngerTime").cloned()
        };
        let end = |game: &Game<'_>, id: u32| {
            let i = game.mobs.iter().position(|m| m.id == id).unwrap();
            game.mobs[i].extra.anger_end
        };
        for (id, anger) in [(past, -5), (zero, 0), (future, 999899)] {
            assert_eq!(saved(game, id), Some(Tag::Int(anger)), "as loaded");
        }
        // The ends behind the saved times: none for the calm two, absolute
        // for the angry one.
        assert_eq!(end(game, past), -1);
        assert_eq!(end(game, zero), 0);
        assert_eq!(end(game, future), 999999);
        // Five ticks of the clock in which the mobs do not tick: the saved
        // time of the angry one shrinks, its end does not move.
        game.world.game_time += 5;
        for (id, anger) in [(past, -5), (zero, 0), (future, 999894)] {
            assert_eq!(saved(game, id), Some(Tag::Int(anger)), "unticked");
        }
        assert_eq!(end(game, future), 999999);
        // Then real ticks.
        for _ in 0..2 {
            game.world.game_time += 1;
            game.update_chunk_tickets();
            assert!(game.entity_ticking_area()(6, 4));
            game.tick_creature_entities().unwrap();
        }
        for (id, anger) in [(past, -5), (zero, 0), (future, 999892)] {
            assert_eq!(saved(game, id), Some(Tag::Int(anger)), "ticked");
        }
        assert_eq!(end(game, future), 999999);
        let i = game.mobs.iter().position(|m| m.id == future).unwrap();
        // The countdown counted the missed ticks too.
        assert_eq!(
            i64::from(game.mobs[i].extra.anger),
            999_999 - game.world.game_time
        );
    });
}
