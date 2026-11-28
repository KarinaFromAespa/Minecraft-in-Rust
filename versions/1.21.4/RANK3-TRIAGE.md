# Rank-3 triage — 1.21.4 behaviour port

Triage of the remaining 190 items (rank 3: 149, rank 4: 39, rank 5: 2),
from `versions/1.21.4/diff-1.21.5-1.21.4/report.json`, verified against the
1.21.4 jars (`~/workspace/mcir-work/1.21.4/jars/`: client-named 8,857 classes,
server-named 6,329) and the 1.21.5 named jar where the direction needed it.
Triage only — no batch porting. Branch `behaviour/1.21.4`, tip `2c63c71d` (batch 83).
Rank 1 10/10, rank 2 348/348 COMPLETE, total 358/548 before triage.

## Verdicts

| Verdict | Count | Meaning |
|---|---|---|
| CLOSE | 173 | Real porting/verification work, assigned a batch below |
| ACCEPT-WITH-CAUSE | 15 | Genuinely absent in 1.21.4 (jar-verified) or outside the ported surface |
| DEFER | 2 | Needs the owner's explicit scope call (Realms) |

Rank 3: 134 CLOSE / 13 ACCEPT / 2 DEFER. Rank 4: 39 CLOSE / 0 ACCEPT / 0 DEFER.
Rank 5: 0 CLOSE / 2 ACCEPT / 0 DEFER.

## Corrections to the prepared brief (2026-11-27)

The brief's numbers all check out (rank 3 = 316 classes / 1,130 methods / 110 named;
90 items zero-named; 74 items <=2 methods; 7 items >=40 methods), but three of its
framings were wrong, each caught by jar evidence:

1. **Rank 4 is NOT confirm-absent.** All 155 rank-4 classes are PRESENT in the 1.21.4
   jars (verified 155/155: client 8,857 + server 6,329 class listings). The report's
   'New in <package>' means 1.21.5 moved/rewrote them: e.g. `BakedModel` is a
   6-method interface in 1.21.4 `client/resources/model`, while 1.21.5 has no
   `BakedModel` class at all (model refactor); `CoreShaders` and `GameTestRegistry`
   likewise exist in 1.21.4 in their old forms. Verdict flips ACCEPT -> CLOSE:
   the batch verifies the Rust uses the 1.21.4 form/location, not the 1.21.5 one.
   The model-refactor items (509, 510, 514, 515, 533-536) go first in the rank-4 order.
2. **The gametest framework EXISTS in 1.21.4** (53 classes per jar); only the test-BLOCK
   additions are absent (`TestBlock`, `TestInstanceBlock`, `TestBlockEntity`,
   `TestEnvironmentDefinition`, `TestData`, `TestInstanceRenderer` — all 0 in both
   1.21.4 jars; test-block packets already removed, commit 65307b5). Item 359 is
   therefore CLOSE (real port), not accept: the Rust's `test_runs.rs` implements the
   1.21.5 /test form. Items 464/465 stay ACCEPT (their classes are 1.21.5-only).
3. **Registry entries 545/546 exist in 1.21.4.** `minecraft:potion` is a 1.21.4-only
   entity type (`EntityType.POTION`, `ThrownPotion`) that 1.21.5 SPLIT into
   `splash_potion`/`lingering_potion`; `entity.wolf.howl` (`SoundEvents.WOLF_HOWL`)
   is in 1.21.4 and ABSENT from 1.21.5 (wolf-variant rework dropped it). Both CLOSE:
   verify the Rust's registries carry the 1.21.4 forms.

Two signals proved reliable and were used throughout: `removed: True` in the report
means 1.21.5-only — all 51 such classes verified ABSENT from both 1.21.4 jars
(present 0 / absent 51). And the 'named in Horizon' signal is mostly live: 101 of
110 named methods found in their module files; the 9 misses are flagged per item
(359 x5, 364/367/377/378 x1 each) and do not change any verdict.

## Scope calls remaining (owner)

- **Realms (362 + 497) — DEFER.** Realms is real, wired Horizon code: `realms.rs` is
  12,946 lines, 8/8 named methods live in it, and the Realms classes exist in the
  1.21.4 jars (229 entries). This is genuine porting, not a stub. But it only runs
  with a signed-in Microsoft account; offline play never reaches it. Recommendation:
  DEFER the port, ACCEPT-with-cause only if the owner rules Realms out of scope for
  1.21.4 — the same call as for the other versions, where Realms was ported.
  Item 497 (`realms/upload.rs`, 1 method, 0 named) rides with 362.
- **Gametest (359) is NOT a scope call anymore.** The framework exists in 1.21.4, so
  the /test command port is ordinary batch work (G6, batch 113). Only the test-BLOCK
  halves (464/465) are accepted as absent, with the Rust-side dead support recorded
  for a separate removal item (item 283 pattern).
- **Nothing else needs the owner.** All 15 ACCEPT items carry jar evidence below;
  all 173 CLOSE items are assigned batches.

## Recommended batch order

Batches continue the rank-2 cadence (4 items per batch; 8 per long-tail sweep batch;
1 per big-system batch). Do NOT run in id order: ids 359-362 are the four most
expensive items and two of them need the scope call above.

| Group | Batches | Items | Shape |
|---|---|---|---|
| G1 small wired gameplay | 84-92 (9) | 33 | 4/batch, named gameplay/mobs, player-visible first |
| G2 commands, protocol & launcher | 93-96 (4) | 14 | 4/batch |
| G3 structure worldgen | 97-100 (4) | 13 | 4/batch, one pass |
| G4 long-tail verification sweep | 101-107 (7) | 52 | 8/batch, jar-verify each (carry-over or port) |
| G5 render internals | 108-112 (5) | 20 | 4/batch |
| G6 big system | 113-114 (2) | 2 | 1/batch: 359 gametest, 360 packrat |
| G7 rank-4 pre-refactor verification | 115-124 (10) | 39 | 4/batch; registry entries, then model, then rest |
| ACCEPT | — | 15 | No batch; jar evidence in the table |
| DEFER | — | 2 | Realms, awaiting the owner |

Total: 41 batches (84-124) for the 173 CLOSE items.

## Item-by-item triage

Columns: id, rank, module (shortened), methods/named, verdict, batch, evidence.
`live` = named methods found in the module file (camelCase or snake_case).

| id | rank | module | m/named | verdict | batch | evidence |
|---|---|---|---|---|---|---|
| 359 | 3 | game/commands/test_runs.rs | 135/12 | CLOSE | 113 | Gametest framework EXISTS in 1.21.4 (53 classes/jar); TestCommand/StructureUtils/TestFinder present. Rust test_runs.rs (489 lines) implements the 1.21.5 /test form: it references clear/reset/run/verify, which javap shows ABSENT from 1.21.4 TestCommand, and misses trackAndStartRunner, which 1.21.4 HAS. Batch reworks /test registration to the 1.21.4 register tree (structure-block-based GameTestInfo). |
| 360 | 3 | packrat.rs | 78/6 | CLOSE | 114 | Packrat package EXISTS in 1.21.4 (36 entries/jar); 6/6 named live in packrat.rs (643 lines). 11 of 17 classes are 1.21.5-only (CachedParseState*, DelayedException, Dictionary$Entry/Reference, ErrorCollector$*, StringReaderTerms$TerminalCharacters — all verified absent from both 1.21.4 jars); batch ports the diff in the 6 present classes only. |
| 361 | 3 | post_chain.rs | 85/4 | CLOSE | 108 | PostChain/ShaderManager exist in 1.21.4; 4/4 named live in post_chain.rs (2,493 lines). Big render item, late in the order. |
| 362 | 3 | realms.rs | 44/8 | DEFER | — | DEFER — Realms scope call (see Scope calls). |
| 363 | 3 | gfx/entity_batch.rs | 66/0 | CLOSE | 108 | RenderType family (66m), 0 named. Rust entity_batch.rs reimplements render types; batch verifies the Rust's RenderType model against 1.21.4's. |
| 364 | 3 | bake.rs | 44/2 | CLOSE | 108 | FaceBakery 26m (bakeQuad live; calculateFacing NOT in bake.rs — flag for batch); ItemModelGenerator 18m/0 named. Grouped with render: adjacent to the rank-4 model items. |
| 365 | 3 | game/mace.rs | 9/5 | CLOSE | 84 | 9m/5 named, 5/5 live in module; batch ports the diff against the 1.21.4 jar. |
| 366 | 3 | game/brush.rs | 18/4 | CLOSE | 84 | 18m/4 named, 4/4 live in module; batch ports the diff against the 1.21.4 jar. |
| 367 | 3 | level/creaking.rs | 15/4 | CLOSE | 84 | CreakingHeartBlock 15m; animateTick NOT in creaking.rs — flag for batch. Batch 82 already ported the heart to the 1.21.4 active model; this is the remaining block-method diff. |
| 368 | 3 | game/end.rs | 9/4 | CLOSE | 84 | 9m/4 named, 4/4 live in module; batch ports the diff against the 1.21.4 jar. |
| 369 | 3 | clouds.rs | 48/0 | CLOSE | 108 | 48m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 370 | 3 | mobs/frog.rs | 14/3 | CLOSE | 85 | 14m/3 named, 3/3 live in module; batch ports the diff against the 1.21.4 jar. |
| 371 | 3 | game/item_use.rs | 17/2 | CLOSE | 85 | 17m/2 named, 2/2 live in module; batch ports the diff against the 1.21.4 jar. |
| 372 | 3 | item.rs | 19/1 | CLOSE | 93 | Protocol item.rs, 1/1 named live. Batch also re-confirms the hash.rs hashed-stack path is inert under protocol 769 (see 437). |
| 373 | 3 | mobs/goals.rs | 8/2 | CLOSE | 85 | 8m/2 named, 2/2 live in module; batch ports the diff against the 1.21.4 jar. |
| 374 | 3 | args.rs | 18/1 | CLOSE | 93 | 4 of 9 classes 1.21.5-only (ResourceSelectorArgument x3, ParserBasedArgument — verified absent); batch ports the 5 present classes. |
| 375 | 3 | level/redstone.rs | 8/2 | CLOSE | 85 | 8m/2 named, 2/2 live in module; batch ports the diff against the 1.21.4 jar. |
| 376 | 3 | mobs/turtle.rs | 17/1 | CLOSE | 86 | 17m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 377 | 3 | settings.rs | 6/2 | CLOSE | 109 | InstrumentItem; getInstrument NOT in settings.rs — flag for batch. |
| 378 | 3 | game/frames.rs | 6/2 | CLOSE | 86 | HangingEntityItem; appendHoverText NOT in frames.rs — flag for batch. |
| 379 | 3 | snbt.rs | 25/0 | CLOSE | 101 | 25m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 380 | 3 | text.rs | 25/0 | CLOSE | 101 | HoverEvent/ClickEvent; 3 ClickEvent subclasses 1.21.5-only (ChangePage/OpenUrl/RunCommand — verified absent); HoverEvent + LocateCommand present. |
| 381 | 3 | font.rs | 15/1 | CLOSE | 109 | 15m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 382 | 3 | game/interactions.rs | 5/2 | CLOSE | 86 | 5m/2 named, 2/2 live in module; batch ports the diff against the 1.21.4 jar. |
| 383 | 3 | game/golems.rs | 2/2 | CLOSE | 86 | 2m/2 named, 2/2 live in module; batch ports the diff against the 1.21.4 jar. |
| 384 | 3 | game/respawn_anchor.rs | 2/2 | CLOSE | 87 | 2m/2 named, 2/2 live in module; batch ports the diff against the 1.21.4 jar. |
| 385 | 3 | mobs/goat.rs | 2/2 | CLOSE | 87 | 2m/2 named, 2/2 live in module; batch ports the diff against the 1.21.4 jar. |
| 386 | 3 | launcher/screenshots.rs | 12/1 | CLOSE | 93 | 12m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 387 | 3 | scoreboard_cmd.rs | 11/1 | CLOSE | 93 | 11m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 388 | 3 | packs.rs | 8/1 | CLOSE | 109 | 8m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 389 | 3 | worldgen/structure/ruined_portal.rs | 8/1 | CLOSE | 97 | 8m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 390 | 3 | game/commands/execute.rs | 6/1 | CLOSE | 94 | 6m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 391 | 3 | codec.rs | 6/1 | CLOSE | 94 | 6m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 392 | 3 | game/commands/text_commands.rs | 5/1 | CLOSE | 94 | 5m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 393 | 3 | game/jigsaw_blocks.rs | 5/1 | CLOSE | 87 | 5m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 394 | 3 | game/structure_blocks.rs | 5/1 | CLOSE | 87 | BlockEntityWithBoundingBoxRenderer 1.21.5-only (verified absent); other class present. |
| 395 | 3 | mobs/camel.rs | 5/1 | CLOSE | 88 | 5m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 396 | 3 | gfx/panorama.rs | 4/1 | CLOSE | 109 | 4m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 397 | 3 | entity_nbt/trident.rs | 4/1 | CLOSE | 88 | 4m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 398 | 3 | game/books.rs | 4/1 | CLOSE | 88 | ClickEvent$SuggestCommand 1.21.5-only (verified absent). |
| 399 | 3 | game/growth.rs | 4/1 | CLOSE | 88 | 4m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 400 | 3 | title.rs | 4/1 | CLOSE | 94 | 4m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 401 | 3 | mobs/phantom.rs | 3/1 | CLOSE | 89 | 3m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 402 | 3 | game/block_edits.rs | 2/1 | CLOSE | 89 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 403 | 3 | game/commands/extra.rs | 2/1 | CLOSE | 95 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 404 | 3 | game/commands/profiles.rs | 2/1 | CLOSE | 95 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 405 | 3 | game/commands/raid.rs | 2/1 | CLOSE | 95 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 406 | 3 | game/enchant_effects.rs | 2/1 | CLOSE | 89 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 407 | 3 | game/inventory_slots.rs | 2/1 | CLOSE | 89 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 408 | 3 | game/remote_chunks.rs | 2/1 | CLOSE | 90 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 409 | 3 | game/sniffers.rs | 2/1 | CLOSE | 90 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 410 | 3 | game/wandering_trader.rs | 2/1 | CLOSE | 90 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 411 | 3 | multiplayer.rs | 2/1 | CLOSE | 95 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 412 | 3 | toasts.rs | 2/1 | CLOSE | 96 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 413 | 3 | level/buckets.rs | 2/1 | CLOSE | 90 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 414 | 3 | worldgen/tree/decorator.rs | 2/1 | CLOSE | 91 | 2m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 415 | 3 | datapack.rs | 1/1 | CLOSE | 110 | 1m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 416 | 3 | block_pattern.rs | 1/1 | CLOSE | 91 | 1m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 417 | 3 | game/fishing.rs | 1/1 | CLOSE | 91 | 1m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 418 | 3 | game/golem_defense_tests.rs | 1/1 | CLOSE | 91 | 1m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 419 | 3 | game/launch_options.rs | 11/0 | CLOSE | 101 | 11m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 420 | 3 | game/pumpkin.rs | 1/1 | CLOSE | 92 | 1m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 421 | 3 | packets/play/commands.rs | 1/1 | CLOSE | 96 | 1m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 422 | 3 | worldgen/structure/monument.rs | 1/1 | CLOSE | 97 | 1m/1 named, 1/1 live in module; batch ports the diff against the 1.21.4 jar. |
| 423 | 3 | worldgen/structure/stronghold.rs | 11/0 | CLOSE | 97 | 11m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 424 | 3 | game/block_editor.rs | 10/0 | CLOSE | 101 | StructureBlockEditScreen present (7m); 3 test-block edit screens 1.21.5-only (verified absent). |
| 425 | 3 | level/shapes.rs | 10/0 | CLOSE | 101 | 10m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 426 | 3 | widely named: client/gui/components | 10/0 | CLOSE | 101 | 10m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 427 | 3 | cit.rs | 9/0 | CLOSE | 110 | 9m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 428 | 3 | pack_palette.rs | 8/0 | CLOSE | 110 | 8m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 429 | 3 | game/chat.rs | 8/0 | CLOSE | 101 | 8m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 430 | 3 | worldgen/structure/mineshaft.rs | 8/0 | CLOSE | 97 | 8m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 431 | 3 | game/debug_lines.rs | 7/0 | CLOSE | 101 | SectionBuffers 1.21.5-only (verified absent). |
| 432 | 3 | mobs/anim.rs | 7/0 | CLOSE | 102 | EquineSaddleModel 1.21.5-only (verified absent). |
| 433 | 3 | game/native_client_entities.rs | 6/0 | CLOSE | 102 | 6m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 434 | 3 | worldgen/structure.rs | 6/0 | CLOSE | 98 | 6m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 435 | 3 | screen_effects.rs | 5/0 | CLOSE | 110 | 5m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 436 | 3 | game/moving_pistons.rs | 5/0 | CLOSE | 102 | 5m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 437 | 3 | hash_ops.rs | 5/0 | ACCEPT | — | All 5 classes absent from both 1.21.4 jars (HashedPatchMap/HashOps = 1.21.5 hashed-component protocol; 1.21.4 has no HashedStack class). Rust hash_ops.rs/item/hash.rs has no live callers outside its own tests — dead cross-version tolerance, leave alone; item 372's batch re-confirms inertness. |
| 438 | 3 | containers.rs | 4/0 | CLOSE | 111 | 4m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 439 | 3 | gfx/animate.rs | 4/0 | CLOSE | 111 | 4m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 440 | 3 | trims.rs | 4/0 | CLOSE | 111 | TrimPredicate 1.21.5-only (verified absent). |
| 441 | 3 | game/chiseled_bookshelf.rs | 4/0 | CLOSE | 102 | 4m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 442 | 3 | game/commands/test_locate.rs | 4/0 | CLOSE | 102 | 4m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 443 | 3 | worldgen/structure/fortress.rs | 4/0 | CLOSE | 98 | 4m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 444 | 3 | mesher/lighting.rs | 3/0 | CLOSE | 111 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 445 | 3 | game/cauldrons.rs | 3/0 | CLOSE | 102 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 446 | 3 | game/locator_bar.rs | 3/0 | CLOSE | 102 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 447 | 3 | legacy_combat.rs | 3/0 | CLOSE | 102 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 448 | 3 | mobs/farm.rs | 3/0 | ACCEPT | — | SheepColorSpawnRules x3 absent (1.21.5 sheep-variant spawn rules). |
| 449 | 3 | shadows.rs | 3/0 | CLOSE | 112 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 450 | 3 | flowers.rs | 3/0 | CLOSE | 103 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 451 | 3 | datafix/world_upgrader.rs | 3/0 | CLOSE | 103 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 452 | 3 | worldgen/structure/mansion.rs | 3/0 | CLOSE | 98 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 453 | 3 | worldgen/structure/ocean_ruin.rs | 3/0 | CLOSE | 98 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 454 | 3 | worldgen/structure/saved_templates.rs | 3/0 | CLOSE | 99 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 455 | 3 | worldgen/structure/simple.rs | 3/0 | CLOSE | 99 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 456 | 3 | widely named: server/commands/data | 3/0 | CLOSE | 103 | 3m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 457 | 3 | particle_type.rs | 2/0 | ACCEPT | — | FireflyParticle x2 absent (1.21.5 garden content — firefly particle, delta section). |
| 458 | 3 | pattern.rs | 2/0 | CLOSE | 103 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 459 | 3 | streaming.rs | 2/0 | CLOSE | 112 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 460 | 3 | game/banners.rs | 2/0 | CLOSE | 103 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 461 | 3 | game/code_of_conduct.rs | 2/0 | CLOSE | 103 | ClientConfigurationPacketListenerImpl$1 1.21.5-only (verified absent). |
| 462 | 3 | game/fireworks.rs | 2/0 | CLOSE | 103 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 463 | 3 | game/leads.rs | 2/0 | CLOSE | 103 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 464 | 3 | game/test_environments.rs | 2/0 | ACCEPT | — | TestEnvironmentDefinition$Functions + SavedDataType absent (1.21.5 test-block framework). Rust test_environments.rs is dead support; removal is a separate focused item (item 283 pattern). |
| 465 | 3 | game/test_instances.rs | 2/0 | ACCEPT | — | TestInstanceRenderer + TestData absent. Rust test_instances.rs dead support, as 464. |
| 466 | 3 | item_sync.rs | 2/0 | ACCEPT | — | DataComponentMatchers x2 absent (1.21.5 advancement criterion). |
| 467 | 3 | menu/equippable.rs | 2/0 | CLOSE | 104 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 468 | 3 | mobs/armadillo.rs | 2/0 | CLOSE | 104 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 469 | 3 | mobs/body_anim.rs | 2/0 | CLOSE | 104 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 470 | 3 | mobs/model_layers.rs | 2/0 | CLOSE | 104 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 471 | 3 | mobs/nether.rs | 2/0 | CLOSE | 104 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 472 | 3 | lithostitched.rs | 2/0 | CLOSE | 104 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 473 | 3 | item/hash/gameplay.rs | 2/0 | ACCEPT | — | WrittenBookPredicate x2 absent (1.21.5 component predicate). |
| 474 | 3 | level/ticks.rs | 2/0 | CLOSE | 104 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 475 | 3 | worldgen/structure/end_city.rs | 2/0 | CLOSE | 99 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 476 | 3 | worldgen/structure/nether_fossil.rs | 2/0 | CLOSE | 99 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 477 | 3 | worldgen/structure/temple.rs | 2/0 | CLOSE | 100 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 478 | 3 | widely named: client/gui/components/events | 2/0 | CLOSE | 104 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 479 | 3 | widely named: client/gui/screens/inventory | 2/0 | CLOSE | 105 | 2m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 480 | 3 | component.rs | 1/0 | ACCEPT | — | HoverEvent$ShowText absent (1.21.5 hover-event form); see 380 for the live HoverEvent diff. |
| 481 | 3 | gamepad.rs | 1/0 | CLOSE | 105 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 482 | 3 | gfx/submit.rs | 1/0 | ACCEPT | — | CommandEncoder absent (1.21.5 blaze3d render-system class). |
| 483 | 3 | gfx/translucency.rs | 1/0 | CLOSE | 112 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 484 | 3 | game/advancements_screen.rs | 1/0 | CLOSE | 105 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 485 | 3 | game/bossbars.rs | 1/0 | CLOSE | 105 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 486 | 3 | game/commands/misc.rs | 1/0 | CLOSE | 105 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 487 | 3 | game/commands/world_cmds.rs | 1/0 | CLOSE | 105 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 488 | 3 | game/item_particles.rs | 1/0 | CLOSE | 105 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 489 | 3 | game/lan_screen.rs | 1/0 | CLOSE | 105 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 490 | 3 | game/native_profiles.rs | 1/0 | CLOSE | 106 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 491 | 3 | game/ominous_item_spawner.rs | 1/0 | CLOSE | 106 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 492 | 3 | game/paintings.rs | 1/0 | CLOSE | 106 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 493 | 3 | game/resource_load.rs | 1/0 | ACCEPT | — | Scope$1 absent (1.21.5-only synthetic class). |
| 494 | 3 | game/text_displays.rs | 1/0 | CLOSE | 106 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 495 | 3 | lan/guest.rs | 1/0 | ACCEPT | — | ProvidesTrimMaterial absent (1.21.5 trim component). |
| 496 | 3 | mobs/enderman.rs | 1/0 | CLOSE | 106 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 497 | 3 | realms/upload.rs | 1/0 | DEFER | — | DEFER — Realms scope call (see Scope calls). |
| 498 | 3 | tree_execute.rs | 1/0 | CLOSE | 106 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 499 | 3 | oracle.rs | 1/0 | CLOSE | 106 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 500 | 3 | upgrade/settings.rs | 1/0 | CLOSE | 106 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 501 | 3 | connection.rs | 1/0 | CLOSE | 107 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 502 | 3 | biomes.rs | 1/0 | CLOSE | 112 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 503 | 3 | level/place.rs | 1/0 | CLOSE | 107 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 504 | 3 | widely named: client/gui/screens/options | 1/0 | CLOSE | 107 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 505 | 3 | widely named: com/mojang/blaze3d/pipeline | 1/0 | ACCEPT | — | BlendFunction absent (1.21.5 pipeline class). |
| 506 | 3 | widely named: com/mojang/blaze3d/systems | 1/0 | ACCEPT | — | RenderPass$Draw absent (1.21.5 systems class). |
| 507 | 3 | widely named: recipebook | 1/0 | CLOSE | 107 | 1m/0 named; batch verifies against the 1.21.4 jar (carry-over or port). |
| 508 | 4 |  | 26 cls | CLOSE | 117 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (26/26 present in 1.21.4 jars.) |
| 509 | 4 |  | 13 cls | CLOSE | 115 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (13/13 present in 1.21.4 jars.) |
| 510 | 4 |  | 12 cls | CLOSE | 115 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (12/12 present in 1.21.4 jars.) |
| 511 | 4 |  | 11 cls | CLOSE | 117 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (11/11 present in 1.21.4 jars.) |
| 512 | 4 |  | 9 cls | CLOSE | 118 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (9/9 present in 1.21.4 jars.) |
| 513 | 4 |  | 9 cls | CLOSE | 118 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (9/9 present in 1.21.4 jars.) |
| 514 | 4 |  | 7 cls | CLOSE | 116 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (7/7 present in 1.21.4 jars.) |
| 515 | 4 |  | 7 cls | CLOSE | 116 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (7/7 present in 1.21.4 jars.) |
| 516 | 4 |  | 7 cls | CLOSE | 118 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (7/7 present in 1.21.4 jars.) |
| 517 | 4 |  | 3 cls | CLOSE | 118 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 518 | 4 |  | 3 cls | CLOSE | 119 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 519 | 4 |  | 3 cls | CLOSE | 119 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 520 | 4 |  | 3 cls | CLOSE | 119 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 521 | 4 |  | 3 cls | CLOSE | 119 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 522 | 4 |  | 3 cls | CLOSE | 120 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 523 | 4 |  | 3 cls | CLOSE | 120 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 524 | 4 |  | 3 cls | CLOSE | 120 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 525 | 4 |  | 3 cls | CLOSE | 120 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 526 | 4 |  | 3 cls | CLOSE | 121 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (3/3 present in 1.21.4 jars.) |
| 527 | 4 |  | 2 cls | CLOSE | 121 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (2/2 present in 1.21.4 jars.) |
| 528 | 4 |  | 2 cls | CLOSE | 121 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (2/2 present in 1.21.4 jars.) |
| 529 | 4 |  | 2 cls | CLOSE | 121 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (2/2 present in 1.21.4 jars.) |
| 530 | 4 |  | 2 cls | CLOSE | 122 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (2/2 present in 1.21.4 jars.) |
| 531 | 4 |  | 2 cls | CLOSE | 122 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (2/2 present in 1.21.4 jars.) |
| 532 | 4 |  | 2 cls | CLOSE | 122 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (2/2 present in 1.21.4 jars.) |
| 533 | 4 |  | 1 cls | CLOSE | 116 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 534 | 4 |  | 1 cls | CLOSE | 116 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 535 | 4 |  | 1 cls | CLOSE | 117 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 536 | 4 |  | 1 cls | CLOSE | 117 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 537 | 4 |  | 1 cls | CLOSE | 122 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 538 | 4 |  | 1 cls | CLOSE | 123 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 539 | 4 |  | 1 cls | CLOSE | 123 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 540 | 4 |  | 1 cls | CLOSE | 123 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 541 | 4 |  | 1 cls | CLOSE | 123 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 542 | 4 |  | 1 cls | CLOSE | 124 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 543 | 4 |  | 1 cls | CLOSE | 124 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 544 | 4 |  | 1 cls | CLOSE | 124 | Class(es) present in the 1.21.4 jars (verified n/n for this item); 1.21.5 moved/rewrote them — batch verifies the Rust uses the 1.21.4 form/location. E.g. BakedModel: 1.21.4 client/resources/model interface (6 methods); 1.21.5 has no BakedModel class at all (model refactor). (1/1 present in 1.21.4 jars.) |
| 545 | 4 |  | 1 entry | CLOSE | 115 | minecraft:potion EXISTS in 1.21.4 (EntityType.POTION, ThrownPotion) but was SPLIT into splash_potion/lingering_potion in 1.21.5 (verified in 1.21.5-client-named.jar). Verify the Rust's entity-type table carries the single 1.21.4 form. |
| 546 | 4 |  | 1 entry | CLOSE | 115 | entity.wolf.howl EXISTS in 1.21.4 (SoundEvents.WOLF_HOWL) but is ABSENT from 1.21.5 (wolf-variant rework dropped it). Verify the Rust's sound registry includes it. |
| 547 | 5 |  | 17 cls | ACCEPT | — | 17 new classes in client/data/models/blockstates (15) + data/tags (2) — datagen packages the report classes as unported; no Horizon references. |
| 548 | 5 |  | 603 cls | ACCEPT | — | 603 changed/removed classes in 133 packages the report classes as unported (largest: world/level/block 56, blaze3d/opengl 27, blaze3d/platform 18). Caveat: classes not individually enumerated in the report; acceptance rests on the report's accounting + the delta section's removal list. The world/level/block 56 are the 1.21.5-only blocks (test blocks, garden content) already covered individually by rank-3 removed=True checks. |

## Method

- Counts recomputed from `report.json` (not taken from the brief) and cross-checked:
  rank 3 = 1,130 methods / 110 named / 316 classes; 90 zero-named; 74 items <=2
  methods (113 total); 7 items >=40 methods (359/360/361/362/363/364/369 = 550).
- Jar evidence: `unzip -l` class listings of both 1.21.4 named jars (8,857 + 6,329
  classes) plus `javap` on `TestCommand`, `StructureUtils`, `EntityType`,
  `SoundEvents`, `BakedModel`, `CoreShaders`, `GameTestRegistry`, `MultiPart`, and
  the 1.21.5 named jar (`~/workspace/mcir-work/1.21.5/jars/1.21.5-client-named.jar`)
  for the direction of 545/546 and the BakedModel move.
- Liveness: each named method grepped (camelCase or snake_case) in its module file
  under `versions/1.21.4/`; 101/110 found. 'Widely named' modules have no single
  file and were triaged on their classes alone.
- The 1.21.5-side checks used the 1.21.5 named jar, never the 26.3 form.

