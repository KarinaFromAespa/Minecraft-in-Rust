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
| 26.1.2 | `versions/26.1.2` | in-progress | harness — · behaviour 0/7 — completion pass + adapter re-base carried forward: protocol rewire closed, 6/6 tables, workspace compiles, protocol tests 437/2; patch delta of 26.1.1 (DataVersion bump, 6 classes incl. handleAttack), see `versions/26.1.2/PORT-NOTES-26.1.2.md` |
| 26.1.1 | `versions/26.1.1` | in-progress | harness — · behaviour 0/6 — completion pass + adapter re-base carried forward: protocol rewire closed, 6/6 tables, workspace compiles, protocol tests 437/2; patch delta of 26.1 (DataVersion bump, 3 classes), see `versions/26.1.1/PORT-NOTES-26.1.1.md` |
| 26.1 | `versions/26.1` | in-progress | harness 3/17 · behaviour 0/795 — completion pass landed: protocol rewire closed, 6/6 tables, prepare ok, workspace compiles; 1.20.2–1.21.11 adapters re-based to the 775 pin, protocol tests 437 pass / 2 fail (sandbox-network pair only); CI port-verify v5 profile 3 pass / 0 oracle / 14 fail (see `versions/26.1/PORT-NOTES-26.1.md`) |

## E6 — Components (14)

| Version | Location | Status | Description |
|---|---|---|---|
| 1.21.11 | `versions/1.21.11` | in-progress | harness 5/23 · behaviour 340/815 — protocol packet-set rewire closed: nine 26.3-only packets rewired to their 1.21.11 mechanisms from the jar, 6/6 tables regenerated (vanilla_command_order +5 −26); cross-version adapter ladder re-based to the 774 pin (all tables regenerated, 19 adapter + 2 legacy-item failures closed), workspace compiles 0 errors / 0 warnings, protocol tests 440 pass on CI (2 sandbox-network failures locally only); oracle bootstrap reworked to 1.21.11's mechanisms, horizon-prepare bundle compiles and runs end-to-end against the named jars; CI port-verify run 44850889695 (branch behaviour/1.21.11 @ d967ed0, the run-44839260120 record) SUCCESS, profile 5 pass / 18 fail of 23 (block_properties, component_patch, food, lan, resource_pack); delta verdict FAILED — D1 fail (23 dead production tags, down from 24), D2 pass (0 warnings, 440 protocol tests), D3 fail (pin work-items lagged at 321/492 at dispatch), D4/D5 blocked (profile-stamp quirk), D6 deferred; see `versions/1.21.11/PORT-NOTES-1.21.11.md` |
| 1.21.10 | `versions/1.21.10` | in-progress | harness 1/17 (4 oracle fail, 8 unmeasured locally) · behaviour 0/719 — first adjacent E6 delta, from template 1.21.11: protocol 774 → 773, one packet change (pure rename, mount_screen_open → horse_screen_open, same id); 6/6 tables regenerated; command-parse oracle ported to the pre-permissions API; adapter ladder re-based to the 773 pin, protocol tests 438 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; exporter mains ported to the 1.21.10 API (Identifier → ResourceLocation, ChunkPos.pack → asLong) and jsr305 on the javac classpath (1.21.10's jar references javax.annotation.Nullable from 1,219 classes), prepare verified ok end-to-end; local harness profile (9 of 17) 1 pass (block_properties) / 4 oracle / 4 fail, see `versions/1.21.10/PORT-NOTES-1.21.10.md` |
| 1.21.9 | `versions/1.21.9` | in-progress | harness — · behaviour 0/27 — second adjacent E6 delta, from 1.21.10: same protocol 773 (world 4556 → 4554), so the pin is a version bump only — packets, registries, blocks, items and commands identical; 2/6 tables regenerated (version_consts, packet_ids header only), 4 confirmed unchanged; no adapter re-base needed, protocol tests 438 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; harness profile not measured locally (tree differs from 1.21.10 in 3 files; CI to measure), see `versions/1.21.9/PORT-NOTES-1.21.9.md` |
| 1.21.8 | `versions/1.21.8` | in-progress | harness 2/17 (2 oracle fail) · behaviour 0/526 — third adjacent E6 delta, from 1.21.9: protocol 773 → 772 (world 4554 → 4440), 108 play packets renumbered, 7 packets removed (code_of_conduct, accept_code_of_conduct, 5 debug packets), debug_subscription_request renamed debug_sample_subscription; 6/6 tables regenerated; adapter ladder re-based to the 772 pin, protocol tests 430 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; parity oracles ported to the 1.21.8 API (Vertex, Keyframe, ProtoChunk, ServerLevel, BlockTags.BARS) and jsr305 on the javac classpath, prepare verified ok end-to-end; local harness profile 2 pass (block_properties, lan) / 2 oracle / 13 fail (true oracle count 9 — six more oracle-compile failures hide in the classifier's 'fail' bucket), see `versions/1.21.8/PORT-NOTES-1.21.8.md` |
| 1.21.7 | `versions/1.21.7` | in-progress | harness 2/17 (2 oracle fail) · behaviour 0/5 — fourth adjacent E6 delta, from 1.21.8: same protocol 772 (world 4440 → 4438), so the pin is a version bump only — packets, registries, blocks, items and commands identical; 2/6 tables regenerated (version_consts, packet_ids header only), 4 confirmed unchanged; no adapter re-base needed, protocol tests 430 pass / 2 fail (sandbox-network pair only), workspace compiles 0 errors / 0 warnings; local harness profile 2 pass (block_properties, lan) / 2 oracle / 13 fail (true oracle count 9 — seven more oracle-compile failures hide in the classifier's 'fail' bucket), see `versions/1.21.7/PORT-NOTES-1.21.7.md` |
| 1.21.6 | `versions/1.21.6` | planned | |
| 1.21.5 | `versions/1.21.5` | planned | |
| 1.21.4 | `versions/1.21.4` | planned | |
| 1.21.3 | `versions/1.21.3` | planned | |
| 1.21.2 | `versions/1.21.2` | planned | |
| 1.21.1 | `versions/1.21.1` | planned | |
| 1.21 | `versions/1.21` | planned | |
| 1.20.6 | `versions/1.20.6` | planned | |
| 1.20.5 | `versions/1.20.5` | planned | |

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
