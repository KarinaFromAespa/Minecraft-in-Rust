# Batch 10 — Rendering, sound, UI (1.21.7)

**DRAFT — not committed.** An execution-risk notice blocks repo execution
and git commands without the user's go-ahead (Batch 2 precedent), so no
driver was run, no cargo test was run, no oracle was repaired and nothing
was committed. Everything below is static analysis: `javap` on the named
1.21.7 jars, source comparison against `versions/1.21.7/crates/`, and checks
of the prepared 1.21.7 data plus vanilla's own asset files. Deliverables
live in `~/workspace/mcir-work/sweep-batch-10-deliverables/` as drafts
awaiting the user's go-ahead to commit to `parity-sweep`.

Date: 2026-11-22. Scope: `docs/PARITY-SWEEP.md` §4 Batch 10 — models,
block-entity rendering, screens, sound wiring, options. Checks + triage
only; no production code touched.

## Tally

**17 resolved checks: 6 PASS, 11 GAP** (all static; see §1 for what
"resolved" means here). The 50 in-scope drivers were **not run** — their
status was triaged from Batch 0's post-repair oracle table
(`docs/sweep/batch-0-oracles.json`, regenerated after all Batch 0 repairs)
plus driver reads:

| Driver state | Count | Drivers |
|---|---|---:|
| Runnable by javac (all oracles compile) — NOT RUN | 11 | cloud_render, container_title, cuboid_model, firework_tooltip, first_person, item_model, lore_tooltip, sound_event, stop_sound_wire, tooltip_display, world_options |
| Blocked on ≥1 unrepaired oracle | 33 | ambient_sound, arrow_hit_sound, audio_attributes, block_entity_render, blockstate_model, breaking_model, chat_screen, container_screen, dragon_render, entity_bound_sound, entity_model, entity_sound_packet, environment_render, fire_starter_sound, hud, menu_click, mipmap, model_baking, model_choice, model_resolution, music, options, pack_screen, recipe_book_render, screen_layout, situational_music, sound_engine, subtitle, text_display_render, title_debug, weather_render, world_border_render, zombie_cure_sound |
| No oracle (server-e2e / protocol drivers) — NOT RUN | 6 | dragon_death_roar, goat_voices, hostile_sound_categories, mob_local_sound, ominous_warning_sound, secure_chat |
| **Total in scope** | **50** | |

Of the 11 runnable-by-javac drivers, 2 are statically known to fail if run:
`sound_event` (B10-G02 — no `sounds.json`, the driver cannot even build
fixtures) and `first_person` (B10-G05 — the probe emits 2 columns the
repaired oracle dropped). The other 9 are unmeasured, not passing.

Per-driver oracle detail is in §5. Oracle defects found: 5 items
(B10-O01–B10-O05, §3). Behaviour gaps found: 6 items (B10-G01–B10-G06, §2).

## 1. Method and what "resolved" means

No driver runs and no cargo tests were possible (execution-risk block), so
this batch resolves a check only when static evidence settles it:

- **javap on the named 1.21.7 jars** — client jar
  `mcir-work/stream3-e6-quick/jars/1.21.7-client-named.jar` for all
  rendering/sound/UI classes (the server jar does not carry them), server
  jar where noted. Vanilla 1.21.7 is the spec throughout; nothing was
  assumed from the 26.3 base or the 1.21.11 form.
- **Vanilla asset files from the client jar** — the jar bundles 9,909
  `assets/` entries (blockstates, models, atlases, lang, font, textures),
  so block-model emptiness, item-model forms and special-renderer use are
  verifiable directly. `sounds.json` is NOT in the jar (it ships in the
  asset index); the 1.21.7 copy was fetched from Mojang's asset CDN via
  piston-meta (asset index 26, sha1-verified
  `25fe1e45f2f3f67a59ccf53652f8e4182056f6cb`, 548,751 bytes) into `/tmp`
  for the wiring diff only — it is a Mojang asset file, not committed.
- **Source comparison** against the 1.21.7 tree on `main`
  (`~/workspace/minecraft-in-rust`, tip `a2f95487`).
- **Oracle status** from Batch 0's post-repair table. Caveat: the table
  records one `first_error` per file, apparently from the server
  classpath — several Batch 10 oracles show a server-cp-only first error
  (`package net.minecraft.client does not exist`) while also failing on
  the client cp for a different reason. Where I verified the real
  client-cp break directly (options), I say so; elsewhere the repair
  agent must read the client-cp errors per file, not the table's
  first_error.

Checks that need a run (the 9 unmeasured runnable drivers, all 6 e2e
drivers, and every blocked driver's behaviour) are recorded as
**not measured**, not as passes. Render feel / sound audibility that no
driver can reach takes method 3 (in-game probe, §6) — specified, not run;
the owner's laptop playtest is owner-initiated only and nothing was
launched there.

## 2. Behaviour gaps

### B10-G01 — Sign boards are not rendered (MAJOR)

Placed signs show text floating on nothing: the board geometry is never
drawn. Verified against 1.21.7's own jar and assets, not assumed from
1.21.11 (ledger item 760 covers the same gap on the template):

- All 24 block sign models in the vanilla 1.21.7 client jar
  (`models/block/*_sign.json`, standing + hanging, all woods) have
  neither `elements` nor `parent` — particle texture only. E.g.
  `oak_sign.json` is `{"textures": {"particle":
  "minecraft:block/oak_planks"}}`. Wall signs reuse the standing model.
  The terrain mesh therefore emits zero sign geometry.
- Vanilla draws the board in the block-entity renderer: javap confirms
  1.21.7 `SignRenderer` / `HangingSignRenderer` (`getSignModel(state,
  woodType)`, `getSignMaterial(woodType)`,
  `createSignModel(EntityModelSet, WoodType, …)`,
  `createSignLayer`/`createHangingSignLayer`).
- The port's `block_entity_render.rs:366` says "Sign text
  (`StandingSignRenderer` / `HangingSignRenderer`); the boards are block
  models" — wrong for 1.21.7. `Kind::Sign` carries only text layout
  (hanging/wall/angle); no board model is ever submitted. The same wrong
  assumption is repeated at `block_entity_render.rs:1010` ("Sign boards
  and beds are block models in the terrain mesh") and in
  `block_entity_cem.rs:11` ("Signs, hanging signs and beds are block
  models here, so their files are not read").
- The 1.21.11 sign-boards fix (branch `fix/1.21.11-sign-boards`, commit
  `ac8dd9dc`) does not apply to 1.21.7 as-is: it adds a `wood` field and
  board drawing to the 1.21.11 tree, but 1.21.7's tree still carries the
  old code. A 1.21.7 fix needs the same shape (board as an entity model
  with the wood's sign material, `attached` handling for ceiling hanging
  signs) verified against 1.21.7's `SignModel` layer definition.
- Sign ITEMS are NOT affected: vanilla 1.21.7 renders sign items as flat
  generated sprites (`items/oak_sign.json` → `minecraft:model` →
  `item/generated`), same as the port. `StandingSignSpecialRenderer` /
  `HangingSignSpecialRenderer` exist in the jar but no vanilla item uses
  them. No divergence for held/inventory signs.

### B10-G02 — No `sounds.json` in the prepared 1.21.7 resources: the game is silent and 6 audio drivers cannot run (MAJOR)

- `audio_harness.require_sounds()` exits unless
  `.local/horizon-client/1.21.7/assets/minecraft/sounds.json` exists
  ("run cargo run -p horizon-prepare -- sounds first"). The file is
  absent: the VM's prepared `.local/horizon-client/1.21.7/` has no
  `assets/` directory at all, `mob-spawn/resources/assets/minecraft/`
  holds only `textures/`, and a VM-wide search found no `sounds.json`
  anywhere (it is also absent from the client jar — it ships only in the
  asset index).
- Root cause is the prepare flow, not the port code: `horizon-prepare
  --prepare` copies the client jar's assets and data
  (`extract_client_resources`) and runs `optional_imports` (panorama,
  fonts, built-in packs, languages) — but sounds are NOT an optional
  import. Sounds need the separate `horizon-prepare sounds` subcommand
  (~380 MB from the launcher asset store) or `--complete`. The laptop's
  `prepare-1.21.7.bat` runs `--prepare` only, so the laptop's `.local`
  almost certainly lacks `sounds.json` too — **the game the owner plays
  is very likely silent**. Verify on the laptop before claiming it
  outright (check
  `.local/horizon-client/1.21.7/assets/minecraft/sounds.json` there).
- Runtime consequence, verified in code: the only runtime load path is
  `SoundRegistry::load(root)` (`audio.rs:1359`, `audio.rs:1813`), which
  silently skips a missing `sounds.json` per namespace
  (`sound_events.rs:280-282`). With no file the registry is empty and
  every event resolves to `Sound::empty()`. The port even documents the
  mode (`is_empty`: "the resources hold no sounds.json (sounds not
  imported from the player's launcher when the game was prepared)").
- Driver consequence: all 6 `audio_harness` drivers gate-fail before any
  comparison — `sound_event` (the recorded CI failure), `ambient_sound`,
  `music`, `situational_music`, `sound_engine`, `audio_attributes`. The
  `sound_event` oracle itself was repaired by Batch 0 (`5933e844`) and
  compiles; the driver is blocked on data, not on the oracle.
- The wiring spec itself is clean (see §4, check 1): vanilla 1.21.7
  `sounds.json` defines 1,727 events; the registry's 1,728th event is
  the intentional `minecraft:intentionally_empty` sentinel. Once
  `horizon-prepare sounds` runs, the sound_event driver can measure
  resolution parity.

### B10-G03 — Beds are invisible when placed, and bed items fail to decode (MAJOR)

Same bug class as G01, one defect deeper — beds fail on BOTH paths:

- Placed beds: vanilla 1.21.7 `models/block/bed.json` is particle-only
  (`{"textures": {"particle": "minecraft:block/oak_planks"}}`, no
  elements); every bed blockstate points at it. `BedRenderer` draws the
  whole bed as an entity model. The port has no Bed kind in
  `block_entity_render.rs` and no bed drawing code anywhere in the
  render crate or `horizon-client` (only sleep mechanics:
  `bed_geometry.rs`, `game/beds.rs`). The mesher emits nothing for the
  empty model, so placed beds are invisible.
- Bed items: vanilla 1.21.7 renders all 16 bed items via the special
  renderer (`items/red_bed.json`: `minecraft:special`, base
  `minecraft:item/red_bed` = `template_bed`, 0 elements,
  `model: {type: minecraft:bed, texture: minecraft:red}`). The port's
  `SpecialModel::decode` (`horizon-assets/src/item_model/special.rs`)
  has no `minecraft:bed` arm; `minecraft:bed` is not a foreign
  namespace, so the `?` in `item_model.rs:550-554` propagates
  `Err("unknown special model type")` and the whole ClientItem decode
  fails — all 16 bed items have no model (exact on-screen fallback
  unconfirmed without a run: missing-texture vs invisible).
- The 1.21.11 sign-boards fix does NOT add bed rendering (no `Bed` in
  its `block_entity_render.rs`), so beds are unaddressed on the template
  too. New finding, not a carry-over.
- Note for the coordinator: beds are ubiquitous (villages, player bases)
  and the owner sleeps in-game, so this should be visible in play —
  worth one explicit in-game confirmation (§6, probe P2) alongside the
  fix, since the code+jar evidence is conclusive but no run exists.

### B10-G04 — Five more vanilla block-entity renderers have no port equivalent (MINOR)

Vanilla 1.21.7 registers 25 block-entity renderers
(`BlockEntityRenderers`, javap-verified). The port's
`block_entity_render.rs` `Kind` covers 13 (chest incl. trapped/ender/
copper, shulker, banner, skull, bell, decorated pot, conduit,
enchanting table incl. book, lectern incl. book, end portal, end
gateway, sign text) plus beacon beams elsewhere
(`gfx/pack_entities.rs`, `gbuffers_beaconbeam`). Missing, with the
visible effect of each (block-model state from the jar):

- **Mob spawner / trial spawner** — the cage is a block model
  (`cube_all_inner_faces` / `cube_bottom_top_inner_faces`); the
  SPINNING MOB (and trial spawner's items) are BE-rendered. Port has
  the spawner DATA types (`chunk/block_entity.rs`, `mobs::spawner`)
  but no spinning-mob rendering: spawner cages render empty.
- **Campfire** — logs are a block model (`template_campfire`); the
  COOKING ITEMS are BE-rendered (`CampfireRenderer`). Port: none —
  cooking food is invisible.
- **Moving piston** — `moving_piston`'s block model is EMPTY
  (particle texture only); `PistonHeadRenderer` draws the moving
  block. Port: no renderer, no `moving_piston` handling in the mesher
  or BE renderer — the block vanishes while a piston extends/retracts
  (the static `piston_head` uses `template_piston_head` and renders).
- **Brushable block** (suspicious sand/gravel) — the block itself
  renders via the sand model; the BRUSHED ITEM is BE-rendered. Port:
  none — the emerging item is invisible while brushing.
- **Vault** — the vault block renders via `template_vault`; the
  DISPLAY ITEM is BE-rendered (`VaultRenderer`). Port: `vault.rs` is
  server-tick logic only; no display-item rendering.
- Unverified, noted not claimed: **structure block** and
  **test instance block** both have cube block models and vanilla
  renderers whose visible in-play output I did not verify — the port
  has dedicated files (`structure_block.rs`,
  `test_instance_block.rs`, `jigsaw_block.rs`) whose rendering role
  needs a driver run to judge. Not counted as gap checks.
- Also noted: the port carries a `CopperGolem` BE kind and a
  `copper_golem_statue` special kind, but 1.21.7 has no copper golem
  (Batch 0 §2.5) — dead code for this version, not a defect.

### B10-G05 — The first-person probe emits swap-ticker columns the repaired oracle dropped: the driver cannot pass (MINOR)

- Batch 0's FirstPerson repair (`9dc5dc8d`) dropped the swap-ticker
  output columns: 1.21.7 has no `itemSwapTicker` /
  `getItemSwapScale` / `swapAnimationScale` — the swap animation lives
  entirely in `ItemInHandRenderer`'s hand heights
  (`VanillaFirstPersonOracle.java:271-273`). The repaired oracle's tick
  line has 20 fields after `tick N <current>`.
- The Rust probe (`first_person_probe.rs:379-380`) still emits
  `strength.swap_ticker` and `swap_scale(1.0, speed)` — 22 fields, 2
  extra, inserted between the attack ticker and the main hand height.
  The driver compares byte-identical output, so it fails on format
  alone even if every value matched. Fix is probe-side (drop the two
  columns); the game-side `first_person.rs` also models a swap scale
  (`heights.tick(…, swap_scale, …)`, `equip_displacement(height,
  swap_animation_scale)`) whose equivalence to vanilla's hand-height
  ticks needs the driver run to judge — recorded as a follow-up, not a
  gap.

### B10-G06 — Silent-goat ambient suppression is a known gap, gated out of the driver (MINOR, driver-documented)

- `test_goat_voices_parity.py`'s docstring: "Silent ambient suppression
  is a separate gap: record those two controls, but gate only their
  hurt/death phases." I did not independently verify this against the
  jar (goat ambient logic is Batch 3-adjacent entity behaviour); it is
  recorded here because the driver documents it in Batch 10's sound
  scope and the driver run is still owed. Needs the driver run (or a
  Batch 3 javap check of goat ambient suppression) to confirm.

## 3. Oracle defects

All from Batch 0's post-repair table (`docs/sweep/batch-0-oracles.json`)
plus driver reads. Counts are the table's; the table's `first_error` is
the server-cp error (see §1 caveat) — the client-cp break needs per-file
inspection except where noted. No oracle was repaired in this batch
(execution-risk block); in-family repair is the follow-up.

### B10-O01 — VanillaOptionsOracle does not compile for 1.21.7 (MINOR)

10 errors. Blocks `test_options_parity` — the only measured UI harness,
still failing after Batch 0 (the 1 of 9 that stayed broken). The
client-cp break is verified directly: the oracle calls
`k.getCategory().id().getPath()`, but 1.21.7's
`KeyMapping.getCategory()` returns `String` (javap) — there is no
`KeyMapping$Category` class and no `id()` in 1.21.7. The rest of the
oracle's API surface exists in 1.21.7 (verified): `Options(Minecraft,
File)` 2-arg ctor, `options.keyMappings`, `KeyMapping.getName()` /
`getDefaultKey()`, `ToggleKeyMapping`, `Minecraft.instance`,
`Minecraft.fixerUpper` (private final DataFixer — the reflective set
works), `DataFixers.getDataFixer()`. Small repair: print the category
string directly.

### B10-O02 — ParityFont does not compile for 1.21.7 (MAJOR)

43 errors, `missing-package`, first error
`com.mojang.blaze3d.font` (server cp). Blocks 7 drivers:
block_entity_render, chat_screen, hud, pack_screen,
recipe_book_render, subtitle, text_display_render. The font package
reorganised between 1.21.7 (`net.minecraft.client.gui.font`) and 26.x
— Batch 0 §6 already flags it as the top-leverage repair (8 drivers
programme-wide). Repairing it unblocks more Batch 10 drivers than any
other single file, but each of the 7 also needs its own oracle repaired.

### B10-O03 — Render/model oracle family (MAJOR)

14 drivers blocked. Every file fails on all three classpaths:

| Oracle | Errors | First error (server cp) / note |
|---|---:|---|
| VanillaBlockEntityRenderOracle | 99 | `com.mojang.blaze3d.vertex` |
| VanillaBlockStateModelOracle | 13 | missing-symbol (cannot find symbol) |
| VanillaBreakingModelOracle | 37 | `com.mojang.blaze3d.platform` |
| VanillaSectionMeshOracle | 99 | `com.mojang.blaze3d.platform` |
| VanillaDragonRenderOracle | 26 | `com.mojang.blaze3d.vertex` |
| VanillaEntityAnimOracle | 61 | `com.mojang.blaze3d.vertex` |
| VanillaEnvironmentOracle | 18 | `EnvironmentAttributeSystem` — 26.x-only, no 1.21.7 equivalent (same break as B10-O04's audio-attributes oracle) |
| VanillaMipmapOracle | 12 | `com.mojang.blaze3d.platform` |
| VanillaModelBakingOracle | 35 | `com.mojang.blaze3d.platform` |
| VanillaModelChoiceOracle | 11 | `net.minecraft.client.resources.model.sprite` — does not exist in 1.21.7 (Batch 0 §1: sprite/model API reorganised) |
| VanillaModelResolutionOracle | 11 | `net.minecraft.client.resources.model.geometry` — does not exist in 1.21.7 |
| VanillaTextDisplayRenderOracle | 78 | `com.mojang.blaze3d.font` |
| VanillaTitleDebugOracle | 12 | `net.minecraft.client.gui` (server cp) |
| VanillaWeatherRenderOracle | 14 | `com.mojang.blaze3d.vertex` |
| VanillaWorldBorderRenderOracle | 74 | `com.mojang.blaze3d.systems` |

Drivers blocked: block_entity_render, blockstate_model,
breaking_model, dragon_render, entity_model, environment_render,
mipmap, model_baking, model_choice, model_resolution,
text_display_render, title_debug, weather_render,
world_border_render. The `block_entity_render` driver is the one that
would catch B10-G01/G03/G04 once its oracle + ParityFont compile.

### B10-O04 — Sound oracle family (MAJOR)

11 drivers blocked:

| Oracle | Errors | First error (server cp) / note |
|---|---:|---|
| VanillaAmbientOracle | 58 | `net.minecraft.client` (server cp) |
| VanillaEntitySoundPacketOracle | 13 | `net.minecraft.world.entity.vehicle.minecart` — 1.21.7 keeps minecarts in `world.entity.vehicle` |
| VanillaSoundEngineOracle | 28 | `net.minecraft.client.multiplayer` (server cp) |
| VanillaSoundRig | 72 | `net.minecraft.client` (server cp) |
| VanillaMusicOracle | 38 | `net.minecraft.client` (server cp) |
| VanillaSituationalMusicOracle | 54 | `net.minecraft.client` (server cp) |
| VanillaFireStarterSoundOracle | 5 | `net.minecraft.client.player` (server cp) |
| VanillaZombieCureSoundOracle | 8 | `net.minecraft.client.multiplayer` (server cp) |
| VanillaSubtitleOracle | 55 | `com.mojang.blaze3d.audio` (server cp) |
| VanillaAudioAttributesOracle | 23 | `EnvironmentAttributeSystem` — 26.x-only, no 1.21.7 equivalent |

Drivers blocked: ambient_sound, arrow_hit_sound,
audio_attributes, entity_bound_sound, entity_sound_packet,
fire_starter_sound, music, situational_music, sound_engine, subtitle,
zombie_cure_sound. Note the sound drivers ALSO need `sounds.json`
(B10-G02) — oracle repair alone does not make the 6 `audio_harness`
drivers runnable. `VanillaSoundEventOracle` itself compiles (Batch 0
`5933e844`); `sound_event` is blocked on data only.

### B10-O05 — UI/screen oracle family (MAJOR)

7 drivers blocked:

| Oracle | Errors | First error (server cp) / note |
|---|---:|---|
| VanillaChatScreenOracle | 128 | `net.minecraft.client` (server cp) |
| VanillaHudOracle | 228 | `com.mojang.renderpearl.api.pipeline` — absent in 1.21.7 (26.x render-pipeline API); the largest single oracle failure in the batch |
| VanillaPackScreenOracle | 124 | `com.mojang.renderpearl.api.pipeline` |
| VanillaRecipeBookScreenOracle | 112 | `com.mojang.renderpearl.api.pipeline` |
| VanillaLoomScreenOracle | 76 | `com.mojang.renderpearl.api.pipeline` |
| VanillaContainerScreenOracle | 54 | `net.minecraft.client` (server cp) |
| VanillaMenuOracle | 9 | missing-symbol (cannot find symbol) |

Drivers blocked: chat_screen, container_screen, hud, menu_click,
pack_screen, recipe_book_render, screen_layout. The four
`renderpearl` oracles need rewrites against 1.21.7's screen APIs, not
import fixes.

## 4. Passing checks (6)

1. **Sound-wiring diff (registry ↔ sounds.json) — PASS.** Vanilla
   1.21.7 `sounds.json` (asset index 26, sha1-verified) defines 1,727
   events; `registries.json` lists 1,728 sound events. The only
   registry event without a `sounds.json` entry is
   `minecraft:intentionally_empty` — vanilla's intentional sentinel.
   Zero `sounds.json` entries outside the registry. 1,631 entries
   carry subtitles; 7 entries have empty sound lists (6 ambient/fall/
   hit + `music.nether.warped_forest` — vanilla-intentional empties);
   6,993 file refs + 46 event refs. The wiring spec is fully
   established; the port-side failure is the missing file (B10-G02),
   not a wiring mismatch.
2. **Sign items — PASS (no divergence).** Vanilla renders sign items
   as flat generated sprites (`items/*.json` →
   `minecraft:model` → `item/generated`), same as the port. The
   jar's `StandingSignSpecialRenderer` /
   `HangingSignSpecialRenderer` are unused by any vanilla item.
3. **Special-model decoder coverage — PASS except bed.** The port's
   `SpecialKind` covers every special type vanilla 1.21.7 items use
   except `minecraft:bed`: chest (incl. the Christmas select),
   shulker_box, banner, head, player_head, conduit, decorated_pot,
   shield, trident. (Bed is B10-G03; the port also carries 26.x-only
   `copper_golem_statue` and `book`/`end_cube` kinds with no 1.21.7
   vanilla users — dead, not defects.)
4. **Beacon beams — PASS (present).** `gfx/pack_entities.rs` runs
   beacon beams (`gbuffers_beaconbeam`); the beacon base block model
   has elements, so the block + beam both have draw paths. Exact beam
   geometry/colour is driver territory (blocked oracle).
5. **BE renderer kinds present (13/25) — PASS.** Chest (single/left/
   right, trapped/ender/copper), shulker, banner, skull (7 types),
   bell, decorated pot, conduit (empty block model — renderer-only,
   port has it), enchanting table + book, lectern + book, end portal,
   end gateway, sign text. The missing 12 are B10-G01/G03/G04 plus
   the 2 unverified (structure/test-instance).
6. **Options API surface — PASS.** Everything the options oracle
   touches except the category chain exists in 1.21.7 (see B10-O01):
   2-arg `Options` ctor, `keyMappings`, `getName`/`getDefaultKey`,
   `ToggleKeyMapping`, `fixerUpper`, `DataFixers.getDataFixer()`,
   `Options.load/save`. (Option VALUES/defaults per key were not
   diffed — that needs the options driver run once B10-O01 lands.)

Also structure-verified but not counted as a pass: **music plumbing**
— the port reads biome music from data (`res.music().at(dimension,
biome, …)`, matching vanilla's `Music` record with min/max delay +
`replaceCurrentMusic`) and mirrors vanilla's situation table
(credits/end/boss/underwater/creative/menu). Exact per-situation
track/delay equality needs the music drivers (blocked: B10-O04 +
B10-G02).

## 5. Per-driver status (all 50)

Oracle status from Batch 0's post-repair table; "fail" = fails on all
three classpaths unless noted. `cp` is the classpath the driver's path
uses (oracle_harness → client).

### Runnable by javac — NOT RUN (11)

| Driver | Oracle(s) | Note |
|---|---|---|
| cloud_render | VanillaCloudRenderOracle pass | Unmeasured |
| container_title | VanillaContainerTitleOracle pass | Unmeasured |
| cuboid_model | VanillaCuboidModelOracle pass | Unmeasured |
| firework_tooltip | VanillaFireworkTooltipOracle pass | Unmeasured |
| first_person | VanillaFirstPersonOracle pass (Batch 0 `9dc5dc8d`) | Would FAIL on run: B10-G05 (probe format) |
| item_model | VanillaItemModelOracle pass (Batch 0 `5e6564e5`) | Unmeasured |
| lore_tooltip | VanillaLoreTooltipOracle pass | Unmeasured |
| sound_event | VanillaSoundEventOracle pass (Batch 0 `5933e844`) | Would FAIL on run: B10-G02 (no sounds.json) |
| stop_sound_wire | VanillaStopSoundWireOracle pass | Unmeasured |
| tooltip_display | VanillaTooltipDisplayOracle pass | Unmeasured |
| world_options | VanillaWorldOptionsOracle pass | Unmeasured |

### Oracle-blocked (33)

| Driver | Blocking oracle(s) | Item |
|---|---|---|
| ambient_sound | VanillaAmbientOracle 58e | B10-O04 |
| arrow_hit_sound | VanillaEntitySoundPacketOracle 13e | B10-O04 |
| audio_attributes | VanillaAudioAttributesOracle 23e | B10-O04 |
| block_entity_render | VanillaBlockEntityRenderOracle 99e + ParityFont 43e (+ TextDisplayRender 78e) | B10-O02, B10-O03 |
| blockstate_model | VanillaBlockStateModelOracle 13e | B10-O03 |
| breaking_model | VanillaBreakingModelOracle 37e + VanillaSectionMeshOracle 99e | B10-O03 |
| chat_screen | VanillaChatScreenOracle 128e + ParityFont 43e (+ PackScreen 124e) | B10-O02, B10-O05 |
| container_screen | VanillaContainerScreenOracle 54e | B10-O05 |
| dragon_render | VanillaDragonRenderOracle 26e | B10-O03 |
| entity_bound_sound | VanillaSoundEngineOracle 28e + VanillaSoundRig 72e | B10-O04 |
| entity_model | VanillaEntityAnimOracle 61e | B10-O03 |
| entity_sound_packet | VanillaEntitySoundPacketOracle 13e | B10-O04 |
| environment_render | VanillaEnvironmentOracle 18e | B10-O03 |
| fire_starter_sound | VanillaFireStarterSoundOracle 5e | B10-O04 |
| hud | VanillaHudOracle 228e + ParityFont 43e | B10-O02, B10-O05 |
| menu_click | VanillaMenuOracle 9e | B10-O05 |
| mipmap | VanillaMipmapOracle 12e | B10-O03 |
| model_baking | VanillaModelBakingOracle 35e | B10-O03 |
| model_choice | VanillaModelChoiceOracle 11e | B10-O03 |
| model_resolution | VanillaModelResolutionOracle 11e | B10-O03 |
| music | VanillaMusicOracle 38e | B10-O04 |
| options | VanillaOptionsOracle 10e | B10-O01 |
| pack_screen | VanillaPackScreenOracle 124e + ParityFont 43e | B10-O02, B10-O05 |
| recipe_book_render | VanillaRecipeBookScreenOracle 112e + ParityFont 43e | B10-O02, B10-O05 |
| screen_layout | VanillaLoomScreenOracle 76e | B10-O05 |
| situational_music | VanillaSituationalMusicOracle 54e | B10-O04 |
| sound_engine | VanillaSoundEngineOracle 28e | B10-O04 |
| subtitle | VanillaSubtitleOracle 55e + ParityFont 43e | B10-O02, B10-O04 |
| text_display_render | VanillaTextDisplayRenderOracle 78e + ParityFont 43e | B10-O02, B10-O03 |
| title_debug | VanillaTitleDebugOracle 12e | B10-O03 |
| weather_render | VanillaWeatherRenderOracle 14e | B10-O03 |
| world_border_render | VanillaWorldBorderRenderOracle 74e | B10-O03 |
| zombie_cure_sound | VanillaZombieCureSoundOracle 8e | B10-O04 |

### No oracle — server-e2e / protocol (NOT RUN, 6)

| Driver | What it needs |
|---|---|
| dragon_death_roar | Two dedicated servers + protocol client (LevelEventTranscript, passes). Also documents a non-Batch-10 gap in its docstring: "/damage on a dragon is a gap of Horizon's" → Batch 3/9. |
| goat_voices | Two dedicated servers. Documents B10-G06 (silent ambient suppression). |
| hostile_sound_categories | Live server + client (locator-bar net harness). |
| mob_local_sound | Live server + client (imports hostile_sound_categories). |
| ominous_warning_sound | Live server + client (locator-bar net harness). |
| secure_chat | Two dedicated servers + mock Mojang services. |

## 6. In-game probes specified (method 3, NOT RUN)

For the owner-initiated playtest only — D6 stays deferred unless he
starts one. Each names the exact scenario and the expected vanilla
behaviour; none is a pass until run.

- **P1 — Sign boards.** Fresh world, place an oak sign (standing, wall,
  hanging), write text, step back: vanilla shows a wooden board with
  text; the port is expected to show floating text only (B10-G01).
- **P2 — Beds.** Place a red bed, sleep in it: vanilla shows the bed
  frame/mattress/pillow; the port is expected to show nothing at the
  bed's position, and the bed item in the hotbar/inventory is expected
  to be broken (B10-G03).
- **P3 — Sound at all.** Any world, break a block / hurt a mob / open a
  chest: vanilla plays the event sound; the port is expected to play
  nothing if `sounds.json` is absent from the laptop's `.local`
  (B10-G02). Also check the laptop file directly (see §7).
- **P4 — Spawner / trial spawner.** Find a dungeon spawner or trial
  chamber: vanilla shows the spinning mob inside the cage; the port is
  expected to show an empty cage (B10-G04).
- **P5 — Moving piston.** Build a piston pushing a block on a clock:
  vanilla shows the block sliding; the port is expected to show the
  block vanish mid-stroke (B10-G04).
- **P6 — Campfire / vault / brushable.** Campfire with food cooking,
  vault with its display item, suspicious sand while brushing: vanilla
  shows the item; the port is expected to show none (B10-G04).

## 7. Follow-ups and questions for the coordinator

1. **Laptop `sounds.json` check (highest value, 1 minute).** Does
   `.local/horizon-client/1.21.7/assets/minecraft/sounds.json` exist on
   the owner's laptop? `prepare-1.21.7.bat` runs `--prepare` only (no
   `sounds` step, no `--complete`), so it is very likely absent and the
   owner's game is silent. If absent, run `horizon-prepare sounds` on
   the laptop (needs the launcher asset store; ~380 MB) — that one step
   fixes B10-G02 at runtime AND unblocks the 6 audio drivers' fixture
   gate on the VM side once the file is also imported here.
2. **First-person probe fix** (B10-G05): drop `swap_ticker` and
   `swap_scale` from `first_person_probe.rs` output to match the
   repaired oracle, then the driver can run. Probe-side, not a game
   fix; needs the execution-risk go-ahead to touch the repo.
3. **Oracle repair order for Batch 10** (when repair resumes):
   ParityFont first (7 drivers), then VanillaOptionsOracle (smallest
   repair, 10 errors, unblocks the only measured UI harness), then the
   sound family (11 drivers, but gated behind B10-G02 anyway), then
   render/model (14), then UI screens (7, incl. the 4 `renderpearl`
   rewrites). The `block_entity_render` driver is the highest-value
   single repair: it is the check that would catch B10-G01/G03/G04.
4. **The VM's prepared `.local` is incomplete**: it has no `assets/`
   directory at all (no jar-extracted models/textures/lang/fonts) —
   only the JSON exporters' output. A complete prepare (or a copy from
   the laptop's `.local`) is needed before any render/model driver can
   run VM-local, independent of oracle repair.
5. **Boundary decisions** (to avoid double-counting): `test_sign_literal`
   (sign BE NBT via commands) counted as Batch 4/9, not Batch 10;
   `test_storage_menu_parity` and the recipe-book logic drivers
   (`recipe_book`, `recipe_book_variants`, `recipe_book_view`) counted
   as Batch 4/7; the dragon `/damage` gap in the dragon-roar driver
   docstring belongs to Batch 3/9; the silent-goat check (B10-G06)
   overlaps Batch 3's entity work.
6. **Structure/test-instance renderers** (B10-G04 note): vanilla's
   visible output for these two renderers is unverified — a driver run
   (or a javap read of the two renderer classes) should settle whether
   the port is missing anything visible.
7. **Bed/sign fix sequencing**: B10-G01 and B10-G03 share one fix shape
   (BE-renderer entity models for boards/beds, plus the `minecraft:bed`
   special kind for items). The 1.21.11 sign-boards fix is a shape
   reference only — 1.21.7's fix must be verified against 1.21.7's own
   `SignModel`/`BedModel` layer definitions, and beds are NOT covered
   by the 1.21.11 fix at all.

## 8. Method notes / limitations

- The named 1.21.7 CLIENT jar is the spec for this batch; the server
  jar does not carry render/sound/UI classes. It was found at
  `mcir-work/stream3-e6-quick/jars/1.21.7-client-named.jar`.
- `sounds.json` is not in any jar; the wiring diff used the
  sha1-verified asset-index copy fetched from Mojang's CDN into `/tmp`
  (not committed, not part of the deliverables — only counts/diffs are
  recorded here).
- No unit tests were run (execution-risk block); the render crate's
  2,139 `#[test]`s and the sound tests are unmeasured for this batch.
- The `first_error` values in Batch 0's oracle table are server-cp
  errors; several Batch 10 oracles' client-cp breaks are unrecorded
  and need per-file inspection by the repair agent.
- Option VALUES/defaults per key, HUD/screen pixel layouts, model
  baking numerics, and music situation→track mapping were NOT
  verified — all need their drivers run once the oracles compile.
