# Batch 10 — Rendering, sound, UI (26.2)

**Version:** 26.2 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.2` ·
**Ledger slice:** `pins/26.2/sweep/batch-10.json` · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 10 — models, block-entity rendering,
screens, sound wiring, options. Checks + triage only; no production code
touched, no driver run, no cargo test run. Everything below is static
analysis: `javap` on the named 26.2 jars, source comparison against
`versions/26.2/crates/`, checks of the prepared 26.2 data plus vanilla's
own asset files.

**Coordinator note (reconciliation):** the batch agent ran before Batch 0
landed and triaged oracles by an import-level static check only. The
coordinator reconciled every oracle claim against the measured Batch 0
table (`docs/sweep/26.2/batch-0-oracles.json`, fixpoint big-compile) and
corrected two withdrawals (B10-O02, B10-O04 — both reinstated as live
defects, see §3), the driver table (§5, now measured), and the tally.
The agent's report also ended mid-sentence at §4 check 7; the
coordinator completed §4 (checks 8–11), §5, §6 and §7 from the agent's
evidence plus the measured table. No finding below rests on the
import-level check alone.

Date: 2026-11-24. Scope: `docs/PARITY-SWEEP.md` §4 Batch 10 — models,
block-entity rendering, screens, sound wiring, options — re-checked
against vanilla **26.2** (DataVersion 4903, protocol 776). The 1.21.7
Batch 10 report (`docs/sweep/batch-10-rendering-sound-ui.md`) is the
template; every 1.21.7 defect was re-verified against 26.2's own
jar/assets + the 26.2 Rust source, not assumed to carry over. **26.2 is
a 26.x version and its port derives from 26.3, so most 1.21.7 gaps do
not carry** — see the withdrawal record in §4.

## Tally

**17 resolved checks: 11 PASS, 6 GAP** (all static; see §1 for what
"resolved" means here). Of the 1.21.7 batch's 11 defects, **5 are
withdrawn for 26.2** (B10-G01, B10-G03, B10-G05, B10-G06, B10-O01 —
evidence in §4) and **6 carry** (B10-G02, B10-G04, B10-O02, B10-O03,
B10-O04, B10-O05 — §2/§3, with 26.2-specific content; G04 gains a
seventh missing renderer, the shelf, and O02/O03/O04/O05 have different
breaks than in 1.21.7). Withdrawn numbers are NOT reused.

The 50 in-scope drivers were **not run** — their status was triaged from
the measured Batch 0 oracle table
(`docs/sweep/26.2/batch-0-oracles.json`, fixpoint big-compile; see §1):

| Driver state | Count | Drivers |
|---|---|---:|
| Runnable by javac (all oracles compile) — NOT RUN | 25 | arrow_hit_sound, blockstate_model, cloud_render, container_screen, container_title, cuboid_model, entity_sound_packet, environment_render, firework_tooltip, first_person, item_model, lore_tooltip, mipmap, model_choice, model_resolution, music, options, situational_music, sound_event, stop_sound_wire, title_debug, tooltip_display, weather_render, world_options, zombie_cure_sound |
| Blocked on ≥1 unrepaired oracle | 22 | ambient_sound, audio_attributes, block_entity_render, breaking_model, chat_screen, dragon_death_roar, dragon_render, entity_bound_sound, entity_model, fire_starter_sound, goat_voices, hud, menu_click, model_baking, pack_screen, recipe_book_render, screen_layout, secure_chat, sound_engine, subtitle, text_display_render, world_border_render |
| No oracle (server-e2e / protocol drivers) — NOT RUN | 3 | hostile_sound_categories, mob_local_sound, ominous_warning_sound |
| **Total in scope** | **50** | |

Of the 25 runnable-by-javac drivers, 3 are statically known to fail if
run on data grounds (B10-G02 — no `sounds.json`, the driver cannot even
build fixtures): `sound_event`, `music`, `situational_music`. The other
3 `audio_harness` drivers (`ambient_sound`, `sound_engine`,
`audio_attributes`) are oracle-blocked as well as data-gated (see §5).
The remaining runnable drivers are unmeasured, not passing.

Per-driver oracle detail is in §5. Oracle defects found: 4 items
(B10-O02–B10-O05, §3). Behaviour gaps found: 2 items (B10-G02,
B10-G04, §2). No B10-H items: every check was settled statically or is
recorded as not-measured in §5; the in-game probes are specified in §6.

## 1. Method and what "resolved" means

No driver runs and no cargo tests were possible (checks-only batch, and
another agent shares the VM), so this batch resolves a check only when
static evidence settles it:

- **javap on the named 26.2 jars** — client jar
  `leakage-audit/jars/26.2-client.jar` (39,193,383 bytes, Mojang-mapped:
  `net/minecraft/client/renderer/blockentity/SignRenderer` etc. present;
  identical copy at
  `26.1-adapters/versions/26.2/jars/client.jar`) for all
  rendering/sound/UI classes, and the named server-game jar
  `26.1-adapters/versions/26.2/jars/server-game.jar` for server-side
  classes (`LivingEntity`, `Mob`, `Goat`, `Entity`). Vanilla 26.2 is the
  spec throughout; nothing was assumed from the 26.3 base or the 1.21.7
  form. The bundler jar `leakage-audit/jars/26.2-server.jar` was never
  javap'd (no game classes). JDK 25 (`~/workspace/tools/jdk-25/bin/javap`).
- **Vanilla asset files from the client jar** — the jar bundles 2,658
  `models/block/` entries, 1,538 `items/` entries and the blockstates,
  so block-model emptiness, item-model forms and special-renderer use
  are verifiable directly. `sounds.json` is NOT in the jar (it ships in
  the asset index); the 26.2 copy was fetched from Mojang's asset CDN
  via piston-meta (asset index 32, sha1-verified
  `3c3870c9175c45348c8549855958e59e674a80d3`, 586,566 bytes;
  `sounds.json` sha1 `9ac006d5537ed0fa4a7bcd1eccfc505155847686`,
  626,160 bytes) into `/tmp` for the wiring diff only — it is a Mojang
  asset file, not committed.
- **Source comparison** against the 26.2 tree on
  `parity-sweep-26.2` (`~/workspace/mcir-work/sweep-26.2/versions/26.2/crates/`,
  synced to main `732c800f`). Read-only; nothing under `versions/26.2/`
  was modified.
- **Prepared 26.2 data**: the task brief said no prepared 26.2 data
  existed, but a prepare run was in progress in this worktree the
  morning of 2026-11-24 (started 02:49 UTC,
  `.local/horizon-client/26.2/`, jar-extracted assets present) — so
  the `sounds.json` presence check could be done directly after all,
  and it settles B10-G02 for 26.2 (§2): sounds are never imported by
  the `--prepare` flow, so the file's absence does not depend on the
  run's completion. The coordinator still owns the follow-up (run
  `horizon-prepare sounds`).
- **Oracle status** from the measured Batch 0 table
  (`docs/sweep/26.2/batch-0-oracles.json` — fixpoint big-compile of
  all 391 oracles against the named 26.2 jars, per classpath; see
  `docs/sweep/26.2/batch-0-measurement.md`). The batch agent's
  import-level static check (every `import` in each referenced oracle
  against the client jar's 10,952-class list) was used only as a
  first pass and was superseded by the coordinator's reconciliation:
  import-clean catches missing classes, not missing/wrong methods,
  and the measured table overturned it in 12 driver cases (notably
  `ParityFont`, the sound oracles, `VanillaMenuOracle`, and the
  render/model oracles — see §3/§5). Where a method-level question
  mattered (options category chain, first-person swap ticker,
  UvMapping arity) it was verified by javap directly and is reported
  as such.
- **Datagen reports** from `~/workspace/mcir-work/26.2/versions/26.2/datagen/reports/`
  (`registries.json`: 1,968 sound events, 49 block-entity types,
  1,196 blocks) — never the 26.3 dump.

Checks that need a run (the 22 unmeasured runnable drivers, all 3
no-oracle e2e drivers, and every blocked driver's behaviour) are
recorded as **not measured**, not as passes. Render feel / sound
audibility that no driver can reach takes method 3 (in-game probe, §6)
— specified, not run; the owner's laptop playtest is owner-initiated
only and nothing was launched there.

## 2. Behaviour gaps

### B10-G02 — No `sounds.json` in the prepared 26.2 resources: the game is silent and 6 audio drivers cannot run (MAJOR)

Carries from 1.21.7, confirmed against 26.2's own prepared data:

- `audio_harness.require_sounds()` (`tools/audio_harness.py:35-37`)
  exits unless `.local/horizon-client/26.2/assets/minecraft/sounds.json`
  exists ("run cargo run -p horizon-prepare -- sounds first"). The
  file is absent: the prepared `.local/horizon-client/26.2/` (prepare run
  in progress the morning of 2026-11-24, started 02:49 UTC) has a full
  `assets/minecraft/` tree (atlases, blockstates, equipment, font,
  items, lang, models, particles, post_effect, shaders, texts,
  textures, waypoint_style) but no `sounds.json` and no `sounds/`
  directory; a find over the whole prepared tree returns nothing. It
  is also absent from the client jar — it ships only in the asset
  index. (The absence does not depend on the prepare run's completion:
  sounds are never imported by the `--prepare` flow — see below.)
- Root cause is the prepare flow, not the port code:
  `horizon-prepare --prepare` runs `prepare_with_progress` only; sounds
  need the separate `horizon-prepare sounds` subcommand
  (`crates/horizon-prepare/src/main.rs:240`, `add_assets("sounds", …)`)
  or `--complete` (`complete_resources`, same file, which adds sounds
  then runs the completeness check). The morning's prepare ran the
  export flow only (no `sounds` step), so the 26.2 resources are in
  exactly the state 1.21.7's were.
- Runtime consequence, verified in code: the only runtime load path is
  `SoundRegistry::load` (`sound_events.rs:260-287`), which silently
  skips a missing `sounds.json` per namespace
  (`sound_events.rs:280-282`: `let Ok(text) = fs::read(…)` else
  skip). With no file the registry is empty and every event resolves
  to `Sound::empty()`. The port even documents the mode (`is_empty`,
  `sound_events.rs:287`: "the resources hold no sounds.json (sounds
  not imported from the player's launcher when the game was
  prepared)").
- Driver consequence: all 6 `audio_harness` drivers gate-fail before
  any comparison — `sound_event`, `ambient_sound`, `music`,
  `situational_music`, `sound_engine`, `audio_attributes` (the same 6
  as 1.21.7, verified by grep over `tools/test_*_parity.py`). Three of
  the six are oracle-clean for 26.2 (`sound_event`, `music`,
  `situational_music` — measured PASS on their classpath), so for
  those three the block is data-only; the other three are
  oracle-blocked as well as data-gated (`ambient_sound`,
  `sound_engine` on the `LocalPlayer` constructor — Batch 0 G2;
  `audio_attributes` on a missing symbol — B10-O04).
- The wiring spec itself is clean (see §4, check 3): vanilla 26.2
  `sounds.json` defines 1,968 events against 1,968 registry events;
  the only registry event without a `sounds.json` entry is the
  intentional `minecraft:intentionally_empty` sentinel, and the only
  `sounds.json` entry without a registry event is a vanilla orphan
  (`entity.chicken_picky.step`). Once `horizon-prepare sounds` runs,
  the 6 drivers can measure resolution parity.

### B10-G04 — Seven vanilla block-entity renderers have no port equivalent (MINOR)

Carries from 1.21.7, with one more renderer than 1.21.7's five: the
shelf (a 26.x addition). Vanilla 26.2 registers 26 block-entity
renderers (`BlockEntityRenderers`, javap `-c` verified on the static
initializer: SIGN, HANGING_SIGN, MOB_SPAWNER, PISTON, CHEST,
ENDER_CHEST, TRAPPED_CHEST, ENCHANTING_TABLE, LECTERN, END_PORTAL,
END_GATEWAY, BEACON, SKULL, BANNER, STRUCTURE_BLOCK,
TEST_INSTANCE_BLOCK, SHULKER_BOX, CONDUIT, BELL, CAMPFIRE,
BRUSHABLE_BLOCK, DECORATED_POT, TRIAL_SPAWNER, VAULT,
COPPER_GOLEM_STATUE, SHELF — no BED: `minecraft:bed` is not a 26.2
block-entity type, confirmed in `registries.json` and in the register
list). The port's `block_entity_render.rs` `Kind`
(`crates/horizon-client-render/src/block_entity_render.rs:321-371`)
covers 17 of the 26 slots (chest incl. trapped/ender/copper, shulker,
banner, skull, bell, decorated pot, conduit, enchanting table, lectern,
copper golem statue, end portal, end gateway, sign text) plus beacon
beams elsewhere (`beacon_beam`, same file `:3448`,
`gfx/pack_entities.rs:820`). `classify()` falls through to `Kind::None`
for everything else (`:589`). Missing, with the visible effect of each
(block-model state from the jar):

- **Mob spawner / trial spawner** — the cage is a block model
  (`spawner.json` → `cube_all_inner_faces`;
  `trial_spawner.json` → `cube_bottom_top_inner_faces`); the SPINNING
  MOB (and trial spawner's items) are BE-rendered
  (`SpawnerRenderer` / `TrialSpawnerRenderer`). Port has the spawner
  DATA types (`chunk/block_entity.rs:66-69`, `mobs::spawner`,
  `mobs::trial_spawner`) but no spinning-mob rendering: spawner cages
  render empty.
- **Campfire** — logs are a block model (`template_campfire`, 7
  elements); the COOKING ITEMS are BE-rendered (`CampfireRenderer`).
  Port has the campfire BE data (`chunk/block_entity.rs:76-77`) but no
  item rendering — cooking food is invisible.
- **Moving piston** — `moving_piston`'s block model is EMPTY
  (particle texture only); `PistonHeadRenderer` draws the moving
  block. Port: no renderer, no `moving_piston` handling in the mesher
  or BE renderer (only collision shapes: `collisions.rs:579`,
  `world.rs:140-146`) — the block vanishes while a piston
  extends/retracts (the static `piston_head` uses
  `template_piston_head` and renders).
- **Brushable block** (suspicious sand/gravel) — the block itself
  renders via the sand/gravel models (`suspicious_sand_0.json` →
  `cube_all`); the BRUSHED ITEM is BE-rendered
  (`BrushableBlockRenderer`). Port has the data
  (`chunk/block_entity.rs:95`) but no rendering — the emerging item is
  invisible while brushing.
- **Vault** — the vault block renders via `template_vault` (2
  elements); the DISPLAY ITEM is BE-rendered (`VaultRenderer`). Port:
  `vault.rs` is server-tick logic only; no display-item rendering.
- **Shelf** (NEW for 26.2 — no 1.21.7 equivalent) — the shelf body is
  a block model (`template_shelf_body`, 3 elements); the THREE
  DISPLAYED ITEMS are BE-rendered (`ShelfRenderer`, registered for
  `BlockEntityTypes.SHELF`). Port has the shelf BE data
  (`chunk/block_entity.rs:96-97`: "A shelf holds three displayed item
  stacks") but no Shelf kind — displayed items are invisible.
- Unverified, noted not claimed: **structure block** and **test
  instance block** both have cube block models
  (`structure_block.json` / `test_instance_block.json` →
  `cube_all`) and vanilla renderers
  (`BlockEntityWithBoundingBoxRenderer` / `TestInstanceRenderer`)
  whose visible in-play output I did not verify — the port has
  dedicated files (`structure_block.rs`, `test_instance_block.rs`,
  `jigsaw_block.rs`) whose rendering role needs a driver run to judge.
  Not counted as gap checks. `potent_sulfur` (the BE type that
  replaced bed in 26.2) has no vanilla renderer either, so the port
  rendering nothing for it is correct.

## 3. Oracle defects

Measured Batch 0 status (see §1); no oracle was repaired in this
batch. The 26.2 picture is far healthier than 1.21.7's, because the
oracles were written against 26.3 and 26.2 IS 26.x — the 1.21.7 breaks
(font reorg, `renderpearl` absent from 1.21.7, `model.sprite`/
`model.geometry` absent, `EnvironmentAttributeSystem` absent) mostly do
not apply. The remaining breaks are 26.3-only APIs that 26.2 lacks,
plus method-level signature drift the import-level check could not see
(coordinator reconciliation — see the header note).

### B10-O02 — `ParityFont` needs `VanillaPackResources.asResourceManager()`, absent in 26.2 (MINOR)

**REINSTATED on reconciliation** (the batch agent withdrew it on the
import-level check — both font packages exist in 26.2 — but the measured
table shows a 1-symbol failure). `ParityFont.java:43` calls
`pack.asResourceManager()` on a `VanillaPackResources`; the method does
not exist in 26.2 (`error: cannot find symbol, symbol: method
asResourceManager(), location: variable pack of type
VanillaPackResources`). Client cp: 1 error; server cp: 41 errors (the
`blaze3d.font` package is client-only, expected). Repair: find 26.2's
way to get a `ResourceManager` from a `VanillaPackResources` (javap the
26.2 class — the 26.3 method may be named differently or the conversion
may go through the `PackResources` interface). Highest-leverage
single-symbol repair in the batch: `ParityFont` blocks 8 drivers
programme-wide (7 in this batch: block_entity_render, chat_screen, hud,
pack_screen, recipe_book_render, subtitle, text_display_render) and
cascades into two more oracles (`VanillaSubtitleOracle`,
`VanillaChatScreenOracle` fail only via `ParityFont` + `PackScreen`).

### B10-O03 — VanillaBlockEntityRenderOracle needs `UvMapping`, a 26.3-only class (MAJOR)

One import: `net.minecraft.client.renderer.texture.UvMapping`
(`VanillaBlockEntityRenderOracle.java:23`), used at `:155` where the
oracle's `SubmitNodeCollector` proxy casts `args[7]` to `UvMapping`
and pattern-matches it against `TextureAtlasSprite` to resolve the
submitted model's texture. The class does not exist anywhere in the
26.2 client jar (verified against the full 10,952-class list).
Blocks `test_block_entity_render_parity` — the driver that would
catch B10-G04 (and would confirm the sign-text/bed passes) once its
oracle compiles. The repair is small and precisely scoped: in 26.2,
`OrderedSubmitNodeCollector.submitModel`'s 8th parameter IS the
`TextureAtlasSprite` directly (javap:
`submitModel(Model, S, PoseStack, RenderType, int, int, int,
TextureAtlasSprite, int, CrumblingOverlay)`), so the oracle can cast
`args[7]` to `TextureAtlasSprite` and drop the `UvMapping` import —
the light/overlay/tint positions (args 4/5/6) already match 26.2's
signature. Measured: 7 errors on the client cp (first error line 23,
the import). Note the driver is blocked by two more oracles as well:
`ParityFont` (B10-O02, 1 error) and `VanillaTextDisplayRenderOracle`
(missing-symbol — Batch 0 residual, see §5); all three must compile
before the driver runs. `ParityData` compiles clean.

### B10-O04 — Sound oracle family: 4 fail on own errors, 1 cascades (MINOR)

**REINSTATED on reconciliation** (the batch agent withdrew it — all 10
sound oracles are import-clean — but the measured table shows 5
failures, all method-level). Per-oracle measured status (client cp):

| Oracle | Status | Cause |
|---|---|---|
| VanillaAmbientOracle | FAIL, 1 error | `LocalPlayer` ctor — Batch 0 G2 |
| VanillaSoundEngineOracle | FAIL, 4 errors | `LocalPlayer` ctor — Batch 0 G2 |
| VanillaFireStarterSoundOracle | FAIL, 1 error | `LocalPlayer` ctor — Batch 0 G2 |
| VanillaAudioAttributesOracle | FAIL, 1 error | `ClockInstance` class missing (`:72`, `new ClockInstance()` — 26.2 has `world.clock.ClockManager` but no `ClockInstance`) |
| VanillaSubtitleOracle | FAIL, cascade | depends on broken `ParityFont` (B10-O02) — passes for free once it is repaired |
| VanillaEntitySoundPacketOracle, VanillaSoundRig, VanillaMusicOracle, VanillaSituationalMusicOracle, VanillaZombieCureSoundOracle, VanillaSoundEventOracle | PASS | — |

Drivers blocked: `ambient_sound`, `entity_bound_sound` + `sound_engine`
(both via `VanillaSoundEngineOracle`), `fire_starter_sound`,
`audio_attributes`, `subtitle`. The three `LocalPlayer`-ctor failures
are Batch 0's G2 group (one signature translation rescues all 17
oracles programme-wide) — cross-referenced, not double-minted.
`VanillaAudioAttributesOracle`'s `ClockInstance` gap needs a javap
check of 26.2's clock API by the repair agent. Note the 6
`audio_harness` drivers are data-gated by B10-G02 regardless — oracle
repair alone does not make them runnable.

### B10-O05 — UI/screen oracle family needs `renderpearl`, a 26.3-only API (MAJOR)

`com.mojang.renderpearl.*` appears ZERO times in the 26.2 client jar
— the render-pipeline API the 26.3-derived oracles use does not exist
in 26.2 at all. Five oracles import it:

| Oracle | Missing import(s) |
|---|---|
| VanillaHudOracle | `com.mojang.renderpearl.api.pipeline.RenderPipeline` |
| VanillaPackScreenOracle | `com.mojang.renderpearl.api.pipeline.RenderPipeline` |
| VanillaRecipeBookScreenOracle | `com.mojang.renderpearl.api.pipeline.RenderPipeline` |
| VanillaLoomScreenOracle | `com.mojang.renderpearl.api.pipeline.RenderPipeline` |
| VanillaWorldBorderRenderOracle | `renderpearl.api.buffers.GpuBuffer`, `renderpearl.api.commands.CommandEncoder`, `renderpearl.api.commands.RenderPass`, `renderpearl.api.device.GpuDevice`, `renderpearl.api.pipeline.RenderPipeline`, plus `net.minecraft.client.renderer.DynamicGpuData` |

Drivers blocked: hud, pack_screen, recipe_book_render, screen_layout,
world_border_render — and chat_screen, which compiles
`VanillaPackScreenOracle` as a dependency
(`test_chat_screen_parity.py:146`). Unlike 1.21.7, where these four
screen oracles needed rewrites against 1.21.7's screen APIs, for 26.2
the repair is a 26.3→26.2 backport: find what 26.2's screens use where
26.3 uses `RenderPipeline` (26.2's screens render through the
`SubmitNodeCollector`/`RenderType` path — e.g. `GuiGraphics` — not a
GPU pipeline object). `VanillaWorldBorderRenderOracle` is the deepest
(6 missing imports incl. `DynamicGpuData`); the other four each miss
exactly one import. The family's other members, measured:
`VanillaContainerScreenOracle` and `VanillaOptionsOracle` PASS on the
client cp; `VanillaChatScreenOracle` is a cascade failure (depends on
broken `ParityFont` + `VanillaPackScreenOracle`); `VanillaMenuOracle`
fails on its own 1 error — it overrides `Player.drop` with the 26.x
3-arg form including `net.minecraft.util.Prediction`, which does not
exist in 26.2 (same break as 1.21.7's B5-O01/B11-O01; repair: drop the
third parameter). `menu_click` is blocked on `VanillaMenuOracle` alone.

## 4. Passing checks (11)

1. **Sign boards — PASS (B10-G01 WITHDRAWN).** In 26.2 the boards ARE
   block models: `template_sign_rot_0.json`,
   `template_wall_sign.json`, `template_hanging_sign_rot_0.json` and
   the attached-hanging variants all carry `elements` (post + board
   for standing signs, board + chains for hanging), and every sign
   blockstate points at them (`oak_sign.json` → `oak_sign_rot_0..3`).
   Vanilla 26.2's `AbstractSignRenderer` has NO `getSignModel` /
   `createSignModel` / `getSignMaterial` (javap: only `submit` /
   `extractRenderState` / `getDarkColor`) — the BE renderer draws
   text only. The port's assumption is therefore CORRECT for 26.2:
   `block_entity_render.rs:365` ("Sign text …; the boards are block
   models") and `:1009` ("Sign boards and beds are block models in
   the terrain mesh"), and `block_entity_cem.rs:11-12`. `Kind::Sign`
   carries the text layout (hanging/wall/angle) exactly as vanilla
   26.2 splits the work. The 1.21.11 sign-boards fix (ac8dd9dc) is
   irrelevant here — 26.2 never had the gap.
2. **Sound-wiring diff (registry ↔ sounds.json) — PASS.** Vanilla 26.2
   `sounds.json` (asset index 32, sha1-verified) defines 1,968
   events; `registries.json` lists 1,968 sound events. The only
   registry event without a `sounds.json` entry is
   `minecraft:intentionally_empty` — vanilla's intentional sentinel
   (same as 1.21.7). The only `sounds.json` entry without a registry
   event is `minecraft:entity.chicken_picky.step` — a vanilla quirk:
   the picky chicken has registry events for ambient/death/hurt but
   no step event, while `sounds.json` defines the step as an
   event-ref to `entity.chicken.step`; it is unreachable in the port
   (resolution is keyed by registered event id) and is not a port
   defect. 1,864 entries carry subtitles; 7 entries have empty sound
   lists (the same 7 vanilla-intentional empties as 1.21.7:
   `block.fungus.fall`/`hit`, `entity.cod`/`salmon`/`snow_golem`/
   `tropical_fish` ambient, `music.nether.warped_forest`); 8,024
   file refs + 61 event refs. The wiring spec is fully established;
   the port-side failure is the missing file (B10-G02), not a wiring
   mismatch.
3. **Beds, placed and as items — PASS (B10-G03 WITHDRAWN).** Placed
   beds in 26.2 are block models, not BE-rendered: there is NO
   `BedRenderer` class in the client jar, `minecraft:bed` is NOT a
   26.2 block-entity type (`registries.json`: 49 BE types, bed out,
   `potent_sulfur` in), and the per-colour models
   (`red_bed_head.json`/`red_bed_foot.json` → `template_bed_head`/
   `template_bed_foot`, 3 elements each: mattress + 2 legs) are what
   the blockstates point at. The terrain mesh renders them. Bed
   ITEMS use `minecraft:composite` of the two block models
   (`items/red_bed.json` — no `minecraft:bed` special type anywhere
   in 26.2's 1,537 item definitions), and the port's
   `ItemModelNode::decode` handles `minecraft:composite`
   (`horizon-assets/src/item_model.rs:448`). All 16 bed items decode.
4. **First-person swap ticker — PASS (B10-G05 WITHDRAWN).** Vanilla
   26.2 HAS the swap ticker: `LivingEntity.itemSwapTicker`
   (protected int, both jars) and `Player.getItemSwapScale(float)`
   (javap). The 26.2 oracle reads them reflectively and emits the two
   columns (`VanillaFirstPersonOracle.java:278`, `:431`), and the
   Rust probe emits the same two columns
   (`first_person_probe.rs:391-392`: `strength.swap_ticker`,
   `swap_scale(1.0, speed)`) from `attack_strength.rs`
   (`swap_ticker`, `getItemSwapScale`, `:12-13`, `:72-74`). Probe and
   oracle agree on format for 26.2; the 1.21.7 format mismatch does
   not exist here. (Values still need the driver run.)
5. **Silent-goat ambient suppression — PASS (B10-G06 WITHDRAWN).**
   Vanilla 26.2: `Entity.playSound` checks `isSilent()` first
   (`javap -c`: `isSilent()` then `ifne` skip) and `Mob.playAmbientSound`
   routes through it, so a `Silent:1b` goat makes no ambient sound.
   The 26.2 port implements exactly this: `Mob::voice`
   (`horizon-client/src/mobs.rs:4702-4709`) draws the pitch first
   ("Java evaluates the arguments … before the call") then returns
   early `if self.silent`, `play_ambient` routes through `voice()`
   (`mobs/ai.rs:1487-1490`, `:1587-1590`), and a unit test pins it
   (`mobs.rs:6547`, `a_silent_mob_makes_no_ambient_sound`: the loud
   cow plays `entity.cow.ambient`, the silent one plays nothing, RNG
   state identical). The `test_goat_voices_parity.py:4` docstring
   ("Silent ambient suppression is a separate gap") is STALE for
   26.2 — the driver still gates those two controls out (`:144-147`),
   so the suppression is unmeasured by the driver, but the code is
   there. Driver-hygiene follow-up (§7), not a behaviour gap.
6. **Sign items — PASS (no divergence).** Vanilla renders sign items
   as flat generated sprites (`items/oak_sign.json` →
   `minecraft:model` → `item/oak_sign` → `item/generated`), same as
   the port. (26.2's jar has no `StandingSignSpecialRenderer` /
   `HangingSignSpecialRenderer` at all — the 1.21.7 classes are gone;
   nothing to diverge on.)
7. **Special-model decoder coverage — PASS.** Exhaustive scan of all
   1,537 vanilla 26.2 item definitions (top-level + nested): 1,390
   `minecraft:model`, 71 `select`, 51 `special`, 16 `composite` (all
   beds), 7 `condition`, 2 `range_dispatch` — the port's
   `ItemModelNode::decode` handles all six top-level types
   (`item_model.rs:425-540`). Special types used (nested walk):
   banner 16, shulker_box 17, chest 13, copper_golem_statue 32,
   head 6, conduit 1, decorated_pot 1, player_head 1, shield 2,
   trident 2 — every one has a `SpecialModel::decode` arm
   (`special.rs:90-147`); the copper-golem poses used (standing /
   sitting / running / star, 8 each) are exactly the decoder's
   allowed set, the head kinds (skeleton / wither_skeleton / zombie /
   creeper / piglin / dragon) match, and no special item lacks a
   `base`. Select properties used (block_state, display_context,
   trim_material, local_time, context_dimension, charge_type) each
   appear in the decoder's select handling in `item_model.rs`, and
   the 7 `condition` + 2 `range_dispatch` items decode through the
   same `ItemModelNode::decode` arms. (Completion by the coordinator:
   the agent's draft ended mid-sentence here — a pasted truncation
   marker was the file's last line — and the select-property handling
   was verified by grep over `item_model.rs`.)
8. **Beacon beams — PASS (present).** `beacon_beam`
   (`block_entity_render.rs:3448`, modelling vanilla
   `BeaconRenderer.submitBeaconBeam`) and the `gbuffers_beaconbeam`
   packing (`gfx/pack_entities.rs:820`) both exist; the beacon base
   block model has elements, so the block + beam both have draw
   paths. Exact beam geometry/colour is unverified statically (no
   beacon driver in scope).
9. **BE renderer kinds present (17/26 slots) — PASS.** Chest
   (single/left/right, trapped/ender/copper), shulker, banner, skull,
   bell, decorated pot, conduit, enchanting table, lectern, copper
   golem statue, end portal, end gateway, sign text — all have a
   `Kind` arm (`block_entity_render.rs:321-371`), plus beacon beams
   (check 8). The missing 9 slots are B10-G04's 7 plus the 2
   unverified (structure/test-instance).
10. **Options API surface — PASS.** Everything the options oracle
    touches exists in 26.2 with the 26.x form: `Options(Minecraft,
    File)` 2-arg ctor, `options.keyMappings`, `KeyMapping.getName()`
    / `getDefaultKey()`, `ToggleKeyMapping`, and — the 1.21.7 break
    that does not apply here — `KeyMapping.getCategory()` returns
    `Category` with `id()` (javap-verified; in 1.21.7 it returns a
    plain `String`). (Option VALUES/defaults per key were not diffed
    — that needs the options driver run once it runs.)
11. **Options oracle compiles — PASS (B10-O01 WITHDRAWN).**
    `VanillaOptionsOracle` PASSES on the client + both classpaths in
    the measured Batch 0 table (server cp fails only on the
    client-only packages, expected). The 1.21.7 O01 defect (10
    errors, category chain) does not exist for 26.2. The `options`
    driver is runnable by javac (§5).

## 5. Per-driver status (all 50, measured)

Oracle status from the measured Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`); "fail" = fails on the
classpath the driver's path uses. `cp` is that classpath
(oracle_harness → both, client jar → client, server jar → server).
Written by the coordinator from the Batch 0 driver map — the batch
agent's import-level table is superseded.

### Runnable by javac — NOT RUN (25)

| Driver | Oracle(s) | Note |
|---|---|---|
| cloud_render | VanillaCloudRenderOracle pass | Unmeasured |
| container_title | VanillaContainerTitleOracle pass | Unmeasured |
| cuboid_model | VanillaCuboidModelOracle pass | Unmeasured |
| firework_tooltip | VanillaFireworkTooltipOracle pass | Unmeasured |
| first_person | VanillaFirstPersonOracle pass | Unmeasured |
| item_model | VanillaItemModelOracle pass | Unmeasured |
| lore_tooltip | VanillaLoreTooltipOracle pass | Unmeasured |
| sound_event | VanillaSoundEventOracle pass | Would FAIL on run: B10-G02 (no sounds.json) |
| stop_sound_wire | VanillaStopSoundWireOracle pass | Unmeasured |
| tooltip_display | VanillaTooltipDisplayOracle pass | Unmeasured |
| world_options | VanillaWorldOptionsOracle pass | Unmeasured |
| arrow_hit_sound | VanillaEntitySoundPacketOracle pass | Unmeasured |
| blockstate_model | VanillaBlockStateModelOracle + VanillaCuboidModelOracle pass | Unmeasured |
| container_screen | VanillaContainerScreenOracle pass | Unmeasured |
| entity_sound_packet | VanillaEntitySoundPacketOracle pass | Unmeasured |
| environment_render | VanillaEnvironmentOracle pass | Unmeasured |
| mipmap | VanillaMipmapOracle pass | Unmeasured |
| model_choice | VanillaModelChoiceOracle + VanillaCuboidModelOracle pass | Unmeasured |
| model_resolution | VanillaModelResolutionOracle + VanillaCuboidModelOracle pass | Unmeasured |
| music | VanillaMusicOracle + VanillaSoundRig pass | Would FAIL on run: B10-G02 (no sounds.json) |
| options | VanillaOptionsOracle pass | Unmeasured |
| situational_music | VanillaSituationalMusicOracle pass | Would FAIL on run: B10-G02 (no sounds.json) |
| title_debug | VanillaTitleDebugOracle pass | Unmeasured |
| weather_render | VanillaWeatherRenderOracle pass | Unmeasured |
| zombie_cure_sound | VanillaZombieCureSoundOracle pass | Unmeasured |

### Oracle-blocked (22)

| Driver | Blocking oracle(s) | Item |
|---|---|---|
| ambient_sound | VanillaAmbientOracle 1e (LocalPlayer ctor) | B10-O04 / Batch 0 G2 |
| audio_attributes | VanillaAudioAttributesOracle 1e (ClockInstance) | B10-O04 |
| block_entity_render | ParityFont 1e + VanillaBlockEntityRenderOracle 7e (UvMapping) + VanillaTextDisplayRenderOracle 1e | B10-O02, B10-O03, Batch 0 O6 |
| breaking_model | VanillaBreakingModelOracle cascade + VanillaSectionMeshOracle 4e (bake) | Batch 0 O6 |
| chat_screen | ParityFont 1e + VanillaChatScreenOracle cascade + VanillaPackScreenOracle 7e (renderpearl) | B10-O02, B10-O05 |
| dragon_death_roar | VanillaProtocolClient 32e (packet-client helper) | Batch 0 O6 |
| dragon_render | VanillaDragonRenderOracle 1e (no-override) | Batch 0 O6 |
| entity_bound_sound | VanillaSoundEngineOracle 4e (LocalPlayer ctor) | B10-O04 / Batch 0 G2 |
| entity_model | VanillaEntityAnimOracle 5e (no-override) | Batch 0 O6 |
| fire_starter_sound | VanillaFireStarterSoundOracle 1e (LocalPlayer ctor) | B10-O04 / Batch 0 G2 |
| goat_voices | VanillaProtocolClient 32e (packet-client helper) | Batch 0 O6 |
| hud | ParityFont 1e + VanillaHudOracle 9e (renderpearl) | B10-O02, B10-O05 |
| menu_click | VanillaMenuOracle 1e (Prediction) | Batch 0 O6 |
| model_baking | VanillaModelBakingOracle 3e (missing-symbol) | Batch 0 O6 |
| pack_screen | ParityFont 1e + VanillaPackScreenOracle 7e (renderpearl) | B10-O02, B10-O05 |
| recipe_book_render | ParityFont 1e + VanillaRecipeBookScreenOracle 4e (renderpearl) | B10-O02, B10-O05 |
| screen_layout | VanillaLoomScreenOracle 4e (renderpearl) | B10-O05 |
| secure_chat | SecureChatClient 7e (no-override, packet-client helper) | Batch 0 O6 |
| sound_engine | VanillaSoundEngineOracle 4e (LocalPlayer ctor) | B10-O04 / Batch 0 G2 |
| subtitle | ParityFont 1e + VanillaSubtitleOracle cascade | B10-O02 |
| text_display_render | ParityFont 1e + VanillaTextDisplayRenderOracle 1e | B10-O02, Batch 0 O6 |
| world_border_render | VanillaWorldBorderRenderOracle 28e (renderpearl) | B10-O05 |

Note: `dragon_death_roar`, `goat_voices` and `secure_chat` are
server-e2e drivers whose Java side is a packet client, not a vanilla
oracle — the 1.21.7 report grouped them as "no oracle", but the Batch 0
driver map shows they compile a helper (`VanillaProtocolClient` /
`SecureChatClient`) that currently fails, so they are counted blocked
here with that qualification.

### No oracle — server-e2e / protocol (NOT RUN, 3)

| Driver | What it needs |
|---|---|
| hostile_sound_categories | Live server + client (locator-bar net harness). |
| mob_local_sound | Live server + client (imports hostile_sound_categories). |
| ominous_warning_sound | Live server + client (locator-bar net harness). |

## 6. In-game probes specified (method 3, NOT RUN)

For the owner-initiated playtest only — D6 stays deferred unless he
starts one. Each names the exact scenario and the expected vanilla
behaviour; none is a pass until run. (Written by the coordinator; the
agent's draft never reached this section.)

- **P1 — Sound at all.** Any world, break a block / hurt a mob / open a
  chest: vanilla plays the event sound; the port is expected to play
  nothing if `sounds.json` is absent from the laptop's `.local`
  (B10-G02). Also check the laptop file directly (see §7).
- **P2 — Spawner / trial spawner.** Find a dungeon spawner or trial
  chamber: vanilla shows the spinning mob inside the cage; the port is
  expected to show an empty cage (B10-G04).
- **P3 — Moving piston.** Build a piston pushing a block on a clock:
  vanilla shows the block sliding; the port is expected to show the
  block vanish mid-stroke (B10-G04).
- **P4 — Campfire / vault / brushable / shelf.** Campfire with food
  cooking, vault with its display item, suspicious sand while brushing,
  shelf with items: vanilla shows the item(s); the port is expected to
  show none (B10-G04).
- **P5 — Signs and beds (withdrawal sanity check).** Place an oak sign
  (standing, wall, hanging) and a red bed: vanilla shows a wooden board
  with text and a bed frame/mattress; the port is expected to show both
  (B10-G01/G03 withdrawn — boards and beds are block models in 26.2).
  Worth one explicit look since the code+jar evidence is conclusive
  but no run exists.

## 7. Follow-ups and questions for the coordinator

1. **`horizon-prepare sounds` for 26.2 (highest value, one step).**
   Fixes B10-G02 at runtime AND unblocks the 3 runnable audio drivers'
   data gate (`sound_event`, `music`, `situational_music`) — their
   oracles already compile. (The other 3 audio drivers also need their
   oracles repaired first — B10-O04.)
2. **Oracle repair order for Batch 10** (when repair resumes):
   `ParityFont` first (1 error, 8 drivers + 2 cascades), then the
   `LocalPlayer` ctor group (Batch 0 G2 — rescues `ambient_sound`,
   `entity_bound_sound`, `sound_engine`, `fire_starter_sound` in this
   batch), then `VanillaBlockEntityRenderOracle` (1-import UvMapping
   fix) + `VanillaTextDisplayRenderOracle` (1 error) + `VanillaMenuOracle`
   (1 error, Prediction) + `VanillaAudioAttributesOracle` (1 error,
   ClockInstance), then the no-override oracles (`VanillaDragonRenderOracle`
   1e, `VanillaEntityAnimOracle` 5e, `SecureChatClient` 7e) and
   `VanillaModelBakingOracle` (3e), then the renderpearl backports
   (deepest — B10-O05). The `block_entity_render` driver is the
   highest-value single repair: it is the check that would catch
   B10-G04.
3. **Goat-voices docstring** (B10-G06 note): the driver's "Silent
   ambient suppression is a separate gap" docstring is stale for 26.2
   (suppression is implemented + unit-tested) — driver-hygiene fix,
   ungate the two controls when the driver next runs.
4. **Structure/test-instance renderers** (B10-G04 note): vanilla's
   visible output for these two renderers is unverified — a driver run
   (or a javap read of the two renderer classes) should settle whether
   the port is missing anything visible.
5. **Boundary decisions** (to avoid double-counting): `test_sign_literal`
   (sign BE NBT via commands) counted as Batch 4/9, not Batch 10;
   `test_storage_menu_parity` and the recipe-book logic drivers
   (`recipe_book`, `recipe_book_variants`, `recipe_book_view`) counted
   as Batch 4/7; the dragon `/damage` gap in the dragon-roar driver
   docstring belongs to Batch 3/9; the silent-goat check (B10-G06)
   overlaps Batch 3's entity work.
6. **Method note for future batches:** an import-level oracle check is
   NOT a substitute for the Batch 0 measurement — it missed 12
   driver-blocking failures here (all method-level: missing symbols,
   constructor mismatches, no-override, cascades). Batches that ran
   before Batch 0 landed must have their oracle claims reconciled
   against `docs/sweep/26.2/batch-0-oracles.json` before commit.
