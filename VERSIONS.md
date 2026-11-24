# Version index — all 103 major releases

Every major Minecraft Java Edition release, grouped by architecture era (see `docs/ROADMAP.md` §2).
Each version's rewritten Rust code lives on `main` in its own directory, `versions/<id>/`, with its documentation alongside the code.
Status is one of `planned` · `in-progress` · `completed`. Description is
filled for completed and in-progress ports; planned rows carry no description.
Each published version carries a machine-readable `status.json` in its pin
(`pins/<id>/` once the release's metadata has moved there — 26.2 first — else
its directory); the scalars below come from it — `harness n/17` is passing
harnesses of the 17-harness CI subset, `behaviour n/m` is closed items of the
version's ranked work items. Rows migrate to generated `delta-verdict.json` rows
(`tools/delta_verify.py --index`) as releases are gated.

## E7 — Unobfuscated (26.x) (5)

| Version | Location | Status | Description |
|---|---|---|---|
| 26.3 | `versions/26.3` | completed | base — imported (upstream snapshot, unmodified); control baseline measured (CI run 43137475228, 14/17 — the Phase 0 gate baseline) |
| 26.2 | `versions/26.2` (pin: `pins/26.2`) | completed | harness 13/17 · behaviour 653/653 — behaviour port complete (all ranks closed 2026-11-15; item 652 pot_decorations game-path codec closed 2026-11-17, item 653 BE sherds closed 2026-11-18, report amended by hand — see the report's `amendments` list); the E7 eight complete on CI — chunk_generation passes (End cone, inline coral, climate branching 6, template block-entity gate, exporter vault fix); D1/D2/D3 pass, D4/D5 formally blocked (gate design decision, Freebuff's call — patch 0006 cannot run: the judge tree is a single-commit repo with no history), D6 partial (playtest 4/8 boxes, 2026-11-18, `pins/26.2/playtest.md` — the remaining boxes, redstone, worldgen and LAN join, are for community testers per the owner's 2026-11-18 word); playtest-found fixes merged and verified in-game (/swing echo, menu split, loot drops); gated tag audit clean (0 dead production tags); CI port-verify run 44365769852 profile 13/0/4 — item 653 proof run, D3 green on rank-2 412/412 (see `versions/26.2/PORT-NOTES-26.2.md`, verdict `pins/26.2/delta-verdict.json`) |
| 26.1.2 | `versions/26.1.2` | completed | harness — · behaviour 7/7 — patch delta of 26.1.1 (DataVersion bump, 6 classes incl. handleAttack): protocol rewire closed, 6/6 tables, workspace compiles, protocol tests 437/2; behaviour port complete — handleAttack spectator guard + Checkbox row limit ported with regression tests, the other five items verified stamp/plumbing (DataFixers 4788→4790, version facts, SharedConstants, DetectedVersion, unported report screen); pre-existing lib-test compile errors fixed in horizon-client (6) and horizon-client-lib (10) so the suites build; crate-tests run 45150986405 (head 81311fcc): horizon-client 1274/0 pass (both new tests), protocol/items/client-core pass — overall failure solely from 41 pre-existing failures in untouched suites (horizon-world 2 datafix, horizon-client-render 3 pack compat, horizon-client-lib 36 title/save/item-sync/projectiles/realms/server-packs, newly visible because the lib suite compiles for the first time on this line), recorded in the port notes as test-health work; D6 deferred (playtest deferred by owner); see `versions/26.1.2/PORT-NOTES-26.1.2.md` |
| 26.1.1 | `versions/26.1.1` | completed | harness — · behaviour 6/6 — patch delta of 26.1 (DataVersion bump, 3 classes): protocol rewire closed, 6/6 tables, workspace compiles, protocol tests 437/2; behaviour port complete — PlayerEntry chatDisabledOrBlocked fixed (26.1's inverted form disabled chat reporting exactly when messages could arrive; 26.1.1 flips one branch to `!canReceive || isBlocked`), the other five items verified stamp/plumbing (DataFixers 4786→4788, version facts, SharedConstants, DetectedVersion, unported report screen); the port's test-health wave fixed the whole 26.1-line debt in this tree (14 stale-version compile errors, 5 fixture failures, 13 jar-verified failures incl. BlendingDataFix rekey 4997→4295, vibration particle 48, painting variant 102, sign-text wire path, snowball tracking fallback) — crate-tests run 45156246301 (head 2a2419cc) FULLY GREEN, all 7 suites, 6,868 tests, 0 failures, the first green run for this lineage; D6 deferred (playtest deferred by owner); see `versions/26.1.1/PORT-NOTES-26.1.1.md` |
| 26.1 | `versions/26.1` | completed | harness 3/17 · behaviour 795/795 — behaviour port complete Nov 24, 2026 (ranks 7/498/214/74/2): blending f32→f64, synth f64 noise, window.rs fresh port, brewing carry-over (65 mixes), immuneTo literal block sets, dark disc, /unpublish + /compute removed (in neither jar), 26.2-only friends system deleted, minecraft:bed block-entity type restored (48 types); crate-tests run 45287010010 (head cfec1728) FULLY GREEN, all 7 suites, 6,864 tests, 0 fail; D6 deferred (playtest deferred by owner); see `versions/26.1/PORT-NOTES-26.1.md` |

## E6 — Components (14)

| Version | Location | Status | Description |
|---|---|---|---|
| 1.21.11 | `versions/1.21.11` | completed | harness 5/23 · behaviour 815/815 — protocol packet-set rewire closed: nine 26.3-only packets rewired to their 1.21.11 mechanisms from the jar, 6/6 tables regenerated (vanilla_command_order +5 −26); cross-version adapter ladder re-based to the 774 pin (all tables regenerated, 19 adapter + 2 legacy-item failures closed), workspace compiles 0 errors / 0 warnings, protocol tests 440 pass on CI (2 sandbox-network failures locally only); oracle bootstrap reworked to 1.21.11's mechanisms, horizon-prepare bundle compiles and runs end-to-end against the named jars; behaviour port complete — 815 items closed (rank-1 10/10, rank-2 492/492, rank-3 247/247, rank-4 64/64, rank-5 2/2); CI run 44989513836 (success): profile 5 pass / 18 fail of 23; verdict failed — D1 fail (21 dead production tags, inherited), D2 pass (440 protocol tests), D3 pass (502/502), D4/D5 blocked (stamp quirk), D6 deferred (playtest deferred by owner); evidence in `pins/1.21.11/`, see `versions/1.21.11/PORT-NOTES-1.21.11.md` |
| 1.21.10 | `versions/1.21.10` | completed | harness 1/17 (4 oracle fail, 8 unmeasured locally) · behaviour 719/719 — behaviour port complete 2026-11-24 (719/719, ranks 10/454/211/42/2; crate-tests run 45376855229 fully green); first adjacent E6 delta, from template 1.21.11: protocol 774 → 773, one packet change (pure rename, mount_screen_open → horse_screen_open, same id); 6/6 tables regenerated; command-parse oracle ported to the pre-permissions API; adapter ladder re-based to the 773 pin, protocol tests 438 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; exporter mains ported to the 1.21.10 API (Identifier → ResourceLocation, ChunkPos.pack → asLong) and jsr305 on the javac classpath (1.21.10's jar references javax.annotation.Nullable from 1,219 classes), prepare verified ok end-to-end; local harness profile (9 of 17) 1 pass (block_properties) / 4 oracle / 4 fail, see `versions/1.21.10/PORT-NOTES-1.21.10.md` |
| 1.21.9 | `versions/1.21.9` | completed | harness 2/17 (2 oracle fail) · behaviour 27/27 — second adjacent E6 delta, from 1.21.10: same protocol 773 (world 4556 → 4554), so the pin is a version bump only — packets, registries, blocks, items and commands identical; 2/6 tables regenerated (version_consts, packet_ids header only), 4 confirmed unchanged; no adapter re-base needed, workspace compiles 0 errors / 0 warnings, protocol tests 440 pass / 0 fail on CI; 4 code changes (entity_move piston stuck multiplier, bubble-column precise gate removed in motion + projectiles, ItemFrame map-dependent bounds); CI run 44455152997 (success): profile 2 pass (block_properties, lan) / 2 oracle / 13 fail (true oracle count 8); verdict failed — D1 fail (33 dead production tags, inherited from the unported 1.21.11 behaviour; triage belongs at 1.21.11), D2 pass, D3 pass (rank-1 3/3, rank-2 20/20), D4/D5 blocked (stamp quirk; local re-checks: both fail), D6 deferred (playtest deferred by owner); evidence in `pins/1.21.9/`, see `versions/1.21.9/PORT-NOTES-1.21.9.md` |
| 1.21.8 | `versions/1.21.8` | completed | harness 2/17 (2 oracle fail) · behaviour 526/526 — third adjacent E6 delta, from 1.21.9: protocol 773 → 772 (world 4554 → 4440), 108 play packets renumbered, 7 packets removed (code_of_conduct, accept_code_of_conduct, 5 debug packets), debug_subscription_request renamed debug_sample_subscription; 6/6 tables regenerated; adapter ladder re-based to the 772 pin, protocol tests 433 pass / 0 fail on CI, workspace compiles 0 errors / 0 warnings; parity oracles ported to the 1.21.8 API (Vertex, Keyframe, ProtoChunk, ServerLevel, BlockTags.BARS) and jsr305 on the javac classpath, prepare verified ok end-to-end; behaviour port complete — F3 overlay re-cut to fixed columns, debug-entries system + End flash + shelf renderer + JSON-RPC management server removed, /rotate source resolution, zero-length-frame skip, furnace-minecart re-aim; CI run 44777574897 (success): profile 2 pass (block_properties, lan) / 2 oracle / 13 fail (true oracle count 9); verdict failed — D1 fail (36 dead production tags, inherited from the unported 1.21.11 behaviour; triage belongs at 1.21.11), D2 pass, D3 pass (rank-1 9/9, rank-2 317/317), D4/D5 blocked (stamp quirk; local re-checks: D4 fail, D5 fail 7 of 8), D6 deferred (playtest deferred by owner); evidence in `pins/1.21.8/`, see `versions/1.21.8/PORT-NOTES-1.21.8.md` |
| 1.21.7 | `versions/1.21.7` | completed | harness 2/17 (2 oracle fail) · behaviour 5/5 — fourth adjacent E6 delta, from 1.21.8: same protocol 772 (world 4440 → 4438), so the pin is a version bump only — packets, registries, blocks, items and commands identical; 2/6 tables regenerated (version_consts, packet_ids header only), 4 confirmed unchanged; no adapter re-base needed, workspace compiles 0 errors / 0 warnings, protocol tests 432 pass / 0 fail on CI; CI run 44441686979 (success): profile 2 pass (block_properties, lan) / 2 oracle / 13 fail (true oracle count 9); verdict failed — D1 fail (37 dead production tags, inherited from the unported 1.21.11 behaviour; triage belongs at 1.21.11), D2 pass, D3 pass (5/5), D4 blocked (stamp quirk; local re-check: fail), D5 blocked (no subset file at dispatch; local re-check: fail, 7 of 8); D6 in open playtest since 2026-11-21 — playable, tester release 1.21.7-build-5 (build #5, head 1957982c), with a fix wave merged from owner reports (water flow, trees, undead burn, leaves, menu lang, block drops, recipe book, mob textures, furnace/brewing fuel, sheep loot, beds) and one defect still open (crafting-table duplication, root cause not found, live repro with packet logging in build #5); cross-version defect suite 6 pass / 1 fail (bed-models, fixed but not re-proven) / 1 not-covered / 2 skip; parity sweep 10,341 checks (7,730 pass, 208 ledger items) on branch parity-sweep; world comparison vs vanilla Test World, 4,145 paired chunks, essentially vanilla-faithful; crate-tests run 45064730064 green (6,902 pass / 0 fail); evidence in `pins/1.21.7/`, see `versions/1.21.7/PORT-NOTES-1.21.7.md` |
| 1.21.6 | `versions/1.21.6` | completed | harness 2/17 (2 oracle fail) · behaviour 31/31 — fifth adjacent E6 delta, from 1.21.7: protocol 772 → 771 (world 4438 → 4435), packet set byte-identical (no rewire); Lava Chicken music disc removed (item + sound event); 2/6 tables regenerated (version_consts, packet_ids header only), 4 confirmed unchanged; adapter ladder re-based to the 771 pin, protocol tests 432 pass / 0 fail on CI, workspace compiles 0 errors / 0 warnings; oracles compile against the 1.21.6 jars with zero fixes, prepare verified ok end-to-end; behaviour port complete — region-tag DataVersion stamp removed, happy ghast scan AABB, font depths back to 1.21.6 (SHADOW_DEPTH 0.003, EFFECT_DEPTH 0.001), text highlight (every highlight inverts, OR_REVERSE form), fog Water before Blindness/Darkness; CI run 44814971968 (success): profile 2 pass (block_properties, lan) / 2 oracle / 13 fail (true oracle count 9); verdict failed — D1 fail (37 dead production tags, inherited from the unported 1.21.11 behaviour; triage belongs at 1.21.11), D2 pass, D3 pass (rank-1 3/3, rank-2 17/17), D4/D5 blocked (stamp quirk; local re-checks: D4 fail, D5 fail 7 of 8), D6 deferred (playtest deferred by owner); evidence in `pins/1.21.6/`, see `versions/1.21.6/PORT-NOTES-1.21.6.md` |
| 1.21.5 | `versions/1.21.5` | completed | harness 2/17 (2 oracle fail) · behaviour 426/426 — sixth adjacent E6 delta, from 1.21.6: protocol 771 → 770 (world 4435 → 4325), packet rewire (8 packets removed: dialogs, waypoint, change-game-mode, custom-click; 60 play serverbound renumbered); dialog system, waypoint/locator bar and happy ghast removed (item −19, sound −29/+4); 6/6 tables regenerated; adapter ladder re-based to the 770 pin, protocol tests 428 pass / 0 fail on CI, workspace compiles 0 errors / 0 warnings; oracles ported to the 1.21.5 APIs (Player/LocalPlayer ctors, WorldVersion, Gson JsonParser), prepare verified ok end-to-end; behaviour port complete — 426 items closed (rank-1 9/9, rank-2 259/259, rank-3 133/133, rank-4 23/23, rank-5 2/2); CI run 44974398528 (success): profile 2 pass (block_properties, lan) / 2 oracle / 13 fail; verdict failed — D1 fail (37 dead production tags, inherited from the unported 1.21.11 behaviour; triage belongs at 1.21.11), D2 pass, D3 pass (rank-1 9/9, rank-2 259/259), D4/D5 blocked (stamp quirk), D6 deferred (playtest deferred by owner); evidence in `pins/1.21.5/`, see `versions/1.21.5/PORT-NOTES-1.21.5.md` |
| 1.21.4 | `versions/1.21.4` | in-progress | harness unmeasured locally · behaviour 0/548 — seventh adjacent E6 delta, from 1.21.5: protocol 770 → 769 (world 4325 → 4189), packet rewire (3 test-block packets removed, add_experience_orb restored, 117 play clientbound renumbered); test blocks, garden content, blue/brown eggs and 31 data components removed (item −11, sound −52/+1); 6/6 tables regenerated; adapter ladder re-based to the 769 pin, component codec re-pinned to the 1.21.4 registry, protocol tests 424 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; oracles ported to the 1.21.4 APIs (Player gameMode/isCreative, fall-damage float, pre-refactor model API, SNBT datafix register), prepare verified ok end-to-end; no local harness profile (skipped — CI to measure), see `versions/1.21.4/PORT-NOTES-1.21.4.md` |
| 1.21.3 | `versions/1.21.3` | in-progress | harness unmeasured locally · behaviour 0/313 — eighth adjacent E6 delta, from 1.21.4: protocol 769 → 768 (world 4189 → 4082), packet rewire (pick_item restored, pick_item_from_block/entity and player_loaded removed, 25 play serverbound renumbered); resin and eyeblossom content removed (11 blocks, 10 items, pale_oak_leaves particle, 15 sound events), creaking transient entity returns; 6/6 tables regenerated (5 regenerated, vanilla_command_order verified unchanged); adapter ladder re-based to the 768 pin, component codec re-pinned to the 1.21.3 wire (custom_model_data int form), protocol tests 424 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; oracles ported to the 1.21.3 APIs (resetBaseValue replicated, Level dragonParts removed, BlockModel textureMap/parentLocation/hasAmbientOcclusion), prepare verified ok end-to-end; no local harness profile (skipped — CI to measure), see `versions/1.21.3/PORT-NOTES-1.21.3.md` |
| 1.21.2 | `versions/1.21.2` | in-progress | harness unmeasured locally · behaviour 0/7 — ninth adjacent E6 delta, from 1.21.3: same protocol 768 (world 4082 → 4080), so the pin is a version bump only — packets, registries, blocks, items and commands identical; 2/6 tables regenerated (version_consts, packet_ids header only), 4 confirmed unchanged; no packet rewire or adapter re-base needed, no code changes beyond the tables, protocol tests 424 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; oracles compile unchanged against the 1.21.2 jars (zero fixes), prepare verified ok end-to-end; no local harness profile (skipped — CI to measure), see `versions/1.21.2/PORT-NOTES-1.21.2.md` |
| 1.21.1 | `versions/1.21.1` | in-progress | harness unmeasured locally · behaviour 0/633 — tenth adjacent E6 delta, from 1.21.2: cross-protocol 768 → 767 (world 4080 → 3955), 633 ranked items; packet rewire (login_finished → game_profile, recipe-book split packets → single recipe, entity_position_sync/move_minecart_along_track/player_rotation go), component re-pin to the 1.21.1 ids (57 kinds, fire_resistant in, 11 1.21.2+ kinds out), adapter ladder re-based to the 767 pin, parity oracles ported to the 1.21.1 APIs; 6/6 tables regenerated, protocol tests 420 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; prepare verified ok end-to-end (428.4 s); no local harness profile (skipped — CI to measure), see `versions/1.21.1/PORT-NOTES-1.21.1.md` |
| 1.21 | `versions/1.21` | in-progress | harness unmeasured locally · behaviour 0/15 — eleventh adjacent E6 delta, from 1.21.1: same protocol 767 (world 3955 → 3953), so the pin is a version bump only — packets, registries, blocks, items and commands identical; 2/6 tables regenerated (version_consts, packet_ids header only), 4 confirmed unchanged; no packet rewire or adapter re-base needed, no code changes beyond the tables, protocol tests 420 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; oracles compile unchanged against the 1.21 jars (zero fixes), prepare verified ok end-to-end; no local harness profile (skipped — CI to measure), see `versions/1.21/PORT-NOTES-1.21.md` |
| 1.20.6 | `versions/1.20.6` | in-progress | harness unmeasured locally · behaviour 0/431 — twelfth adjacent E6 delta, from 1.21: cross-protocol 767 → 766 (world 3953 → 3839), 431 ranked items; packet rewire (1.21-only server_links + custom_report_details go), component re-pin to the 1.20.6 ids (56 kinds, jukebox_playable out), adapter ladder re-based to the 766 pin, parity oracles ported to the 1.20.6 APIs (13 files); 3/6 tables regenerated, 3 confirmed unchanged, protocol tests 420 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; first pin below 1.21 — the item codec keys attribute/food/container layout off the effective quirks (UUID-form attributes, remainder-less food, count-first containers); no local harness profile (skipped — CI to measure), see `versions/1.20.6/PORT-NOTES-1.20.6.md` |
| 1.20.5 | `versions/1.20.5` | in-progress | harness unmeasured locally · behaviour 0/12 — thirteenth adjacent E6 delta, from 1.20.6: same protocol 766 (world 3839 → 3837), so the pin is a version bump only — packets, registries, blocks, items and commands identical; 2/6 tables regenerated (version_consts, packet_ids header only), 4 confirmed unchanged; no packet rewire or adapter re-base needed, no code changes beyond the tables, protocol tests 420 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; oracles compile unchanged against the 1.20.5 jars (zero fixes); no local harness profile (skipped — CI to measure), see `versions/1.20.5/PORT-NOTES-1.20.5.md` — the E6 floor; E5 needs its own template decision |

## E5 — Height & config (15)

| Version | Location | Status | Description |
|---|---|---|---|
| 1.20.4 | `versions/1.20.4` | planned | |
| 1.20.3 | `versions/1.20.3` | planned | |
| 1.20.2 | `versions/1.20.2` | planned | |
| 1.20.1 | `versions/1.20.1` | planned | |
| 1.20 | `versions/1.20` | planned | |
| 1.19.4 | `versions/1.19.4` | planned | |
| 1.19.3 | `versions/1.19.3` | planned | |
| 1.19.2 | `versions/1.19.2` | planned | |
| 1.19.1 | `versions/1.19.1` | planned | |
| 1.19 | `versions/1.19` | planned | |
| 1.18.2 | `versions/1.18.2` | planned | |
| 1.18.1 | `versions/1.18.1` | planned | |
| 1.18 | `versions/1.18` | planned | |
| 1.17.1 | `versions/1.17.1` | planned | |
| 1.17 | `versions/1.17` | planned | |

## E4 — Mojmap plateau (9)

| Version | Location | Status | Description |
|---|---|---|---|
| 1.16.5 | `versions/1.16.5` | planned | |
| 1.16.4 | `versions/1.16.4` | planned | |
| 1.16.3 | `versions/1.16.3` | planned | |
| 1.16.2 | `versions/1.16.2` | planned | |
| 1.16.1 | `versions/1.16.1` | planned | |
| 1.16 | `versions/1.16` | planned | |
| 1.15.2 | `versions/1.15.2` | planned | |
| 1.15.1 | `versions/1.15.1` | planned | |
| 1.15 | `versions/1.15` | planned | |

## E3 — Flattening (8)

| Version | Location | Status | Description |
|---|---|---|---|
| 1.14.4 | `versions/1.14.4` | planned | |
| 1.14.3 | `versions/1.14.3` | planned | |
| 1.14.2 | `versions/1.14.2` | planned | |
| 1.14.1 | `versions/1.14.1` | planned | |
| 1.14 | `versions/1.14` | planned | |
| 1.13.2 | `versions/1.13.2` | planned | |
| 1.13.1 | `versions/1.13.1` | planned | |
| 1.13 | `versions/1.13` | planned | |

## E2 — Late numeric-ID (14)

| Version | Location | Status | Description |
|---|---|---|---|
| 1.12.2 | `versions/1.12.2` | planned | |
| 1.12.1 | `versions/1.12.1` | planned | |
| 1.12 | `versions/1.12` | planned | |
| 1.11.2 | `versions/1.11.2` | planned | |
| 1.11.1 | `versions/1.11.1` | planned | |
| 1.11 | `versions/1.11` | planned | |
| 1.10.2 | `versions/1.10.2` | planned | |
| 1.10.1 | `versions/1.10.1` | planned | |
| 1.10 | `versions/1.10` | planned | |
| 1.9.4 | `versions/1.9.4` | planned | |
| 1.9.3 | `versions/1.9.3` | planned | |
| 1.9.2 | `versions/1.9.2` | planned | |
| 1.9.1 | `versions/1.9.1` | planned | |
| 1.9 | `versions/1.9` | planned | |

## E1 — Netty protocol (19)

| Version | Location | Status | Description |
|---|---|---|---|
| 1.8.9 | `versions/1.8.9` | planned | |
| 1.8.8 | `versions/1.8.8` | planned | |
| 1.8.7 | `versions/1.8.7` | planned | |
| 1.8.6 | `versions/1.8.6` | planned | |
| 1.8.5 | `versions/1.8.5` | planned | |
| 1.8.4 | `versions/1.8.4` | planned | |
| 1.8.3 | `versions/1.8.3` | planned | |
| 1.8.2 | `versions/1.8.2` | planned | |
| 1.8.1 | `versions/1.8.1` | planned | |
| 1.8 | `versions/1.8` | planned | |
| 1.7.10 | `versions/1.7.10` | planned | |
| 1.7.9 | `versions/1.7.9` | planned | |
| 1.7.8 | `versions/1.7.8` | planned | |
| 1.7.7 | `versions/1.7.7` | planned | |
| 1.7.6 | `versions/1.7.6` | planned | |
| 1.7.5 | `versions/1.7.5` | planned | |
| 1.7.4 | `versions/1.7.4` | planned | |
| 1.7.3 | `versions/1.7.3` | planned | |
| 1.7.2 | `versions/1.7.2` | planned | |

## E0 — Pre-Netty legacy (19)

| Version | Location | Status | Description |
|---|---|---|---|
| 1.6.4 | `versions/1.6.4` | planned | |
| 1.6.2 | `versions/1.6.2` | planned | |
| 1.6.1 | `versions/1.6.1` | planned | |
| 1.5.2 | `versions/1.5.2` | planned | |
| 1.5.1 | `versions/1.5.1` | planned | |
| 1.4.7 | `versions/1.4.7` | planned | |
| 1.4.5 | `versions/1.4.5` | planned | |
| 1.4.6 | `versions/1.4.6` | planned | |
| 1.4.4 | `versions/1.4.4` | planned | |
| 1.4.2 | `versions/1.4.2` | planned | |
| 1.3.2 | `versions/1.3.2` | planned | |
| 1.3.1 | `versions/1.3.1` | planned | |
| 1.2.5 | `versions/1.2.5` | planned | |
| 1.2.4 | `versions/1.2.4` | planned | |
| 1.2.3 | `versions/1.2.3` | planned | |
| 1.2.2 | `versions/1.2.2` | planned | |
| 1.2.1 | `versions/1.2.1` | planned | |
| 1.1 | `versions/1.1` | planned | |
| 1.0 | `versions/1.0` | planned | |
