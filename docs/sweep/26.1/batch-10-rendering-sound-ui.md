# Batch 10 — Rendering, sound, UI (26.1)

**Version:** 26.1 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.1` ·
**Ledger slice:** `sweep-26.1-batches/b10/slice.json` · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 10 — models, block-entity rendering,
screens, sound wiring, options. Checks + triage only; no production code
touched, no driver run, no cargo test run. Everything below is static
analysis: `javap` on the named 26.1 jars, source comparison against
`versions/26.1/crates/`, checks of the prepared 26.1 data plus vanilla's
own asset files, and oracle status from the measured Batch 0 table only
(`docs/sweep/26.1/batch-0-oracles.json`).

Date: 2026-11-24. Scope: `docs/PARITY-SWEEP.md` §4 Batch 10 — models,
block-entity rendering, screens, sound wiring, options — re-checked
against vanilla **26.1** (DataVersion 4786, protocol 775, Java 25). The
26.2 Batch 10 report is the template; every 1.21.7 defect was
re-verified against 26.1's own jar/assets + the 26.1 Rust source, not
assumed to carry over. **26.1 is a 26.x version, but it is OLDER than
26.2 in exactly the ways that matter here**: it still has the bed
block-entity type, still draws sign boards and beds in BE renderers
(1.21.7-style, not 26.2's block-model form), still has the HUD inside
`Gui` (no `net.minecraft.client.gui.Hud` class), and predates the
`EntityTypes` holder move. The 26.2 withdrawals of B10-G01 and B10-G03
do NOT carry to 26.1 — both reproduce (see §2).

## Tally

**12 resolved behaviour checks: 8 PASS, 4 GAP** (all static; see §1 for
what "resolved" means here). Oracle-status checks are NOT counted in
the tally (26.1 convention — differs from the 26.2 run, where Batch 10
counted 4 oracle items as gaps; here oracle triage has its own counts
in §3). Of the 1.21.7 batch's 11 defects, **3 are withdrawn for 26.1**
(B10-G05, B10-G06, B10-O01 — evidence in §4/§3) and **8 carry**
(B10-G01, B10-G02, B10-G03, B10-G04, B10-O02, B10-O03, B10-O04,
B10-O05 — §2/§3, with 26.1-specific content; G03 reproduces only for
placed beds, bed items decode and render in the 26.1 port; O03/O04/O05
have different and mostly larger breaks than in 26.2). Withdrawn
numbers are NOT reused.

The 50 in-scope drivers were **not run** — their status was triaged
from the measured Batch 0 oracle table
(`docs/sweep/26.1/batch-0-oracles.json`, fixpoint big-compile) and the
Batch 0 driver map (`sweep-26.1-batches/b0/drivers.json`; see §1):

| Driver state | Count | Drivers |
|---|---|---:|
| Runnable by javac (all oracles compile) — NOT RUN | 16 | blockstate_model, cloud_render, container_screen, container_title, cuboid_model, firework_tooltip, lore_tooltip, mipmap, model_choice, model_resolution, options, stop_sound_wire, tooltip_display, weather_render, world_options, zombie_cure_sound |
| Blocked on ≥1 unrepaired oracle | 31 | ambient_sound, arrow_hit_sound, audio_attributes, block_entity_render, breaking_model, chat_screen, dragon_death_roar, dragon_render, entity_bound_sound, entity_model, entity_sound_packet, environment_render, fire_starter_sound, first_person, goat_voices, hud, item_model, menu_click, model_baking, music, pack_screen, recipe_book_render, screen_layout, secure_chat, situational_music, sound_engine, sound_event, subtitle, text_display_render, title_debug, world_border_render |
| No oracle (server-e2e / protocol drivers) — NOT RUN | 3 | hostile_sound_categories, mob_local_sound, ominous_warning_sound |
| **Total in scope** | **50** | |

26.1 is much worse than 26.2 here (25 runnable there, 16 here): nine
drivers that were runnable for 26.2 are oracle-blocked for 26.1
(`arrow_hit_sound`, `entity_sound_packet`, `environment_render`,
`first_person`, `item_model`, `music`, `situational_music`,
`sound_event`, `title_debug`). All 6 `audio_harness` drivers are
oracle-blocked AND data-gated by B10-G02 — unlike 26.2, where 3 of the
6 were oracle-clean and blocked on data only. The remaining runnable
drivers are unmeasured, not passing.

Per-driver oracle detail is in §5. Oracle defects found: 4 items
(B10-O02–B10-O05, §3), covering 38 failing in-scope oracles (plus 2
shared packet-client helpers recorded in §5 as Batch 0 residual).
Behaviour gaps found: 4 items (B10-G01, B10-G02, B10-G03, B10-G04,
§2). No B10-H items: every check was settled statically or is
recorded as not-measured in §5; the in-game probes are specified in
§6.

## 1. Method and what "resolved" means

No driver runs and no cargo tests were possible (checks-only batch),
so this batch resolves a check only when static evidence settles it:

- **javap on the named 26.1 jars** — client jar
  `26.1.x/versions/26.1/jars/client.jar` (10,682 classes,
  Mojang-mapped) for all rendering/sound/UI classes, and the named
  server-game jar `26.1.x/versions/26.1/jars/server-game.jar`
  (7,351 classes) for server-side classes (`LivingEntity`, `Mob`,
  `Entity`, `Level`, `MinecraftServer`). Vanilla 26.1 is the spec
  throughout; nothing was assumed from the 26.3 base, the 26.2 form,
  or the 1.21.7 form. The bundler jars were never javap'd (no game
  classes). JDK 25 (`~/workspace/tools/jdk-25/bin/javap`). Classes
  needed repeatedly were extracted once to the batch evidence dir
  (`evidence/classes/`) and javap'd from there.
- **Vanilla asset files from the client jar** — the jar bundles 2,393
  `models/block/` entries and 1,506 `items/` entries, so block-model
  emptiness, item-model forms and special-renderer use are verifiable
  directly. `sounds.json` is NOT in the jar (it ships in the asset
  index); the 26.1 copy was fetched from Mojang's asset CDN via
  piston-meta (asset index 30, sha1-verified
  `49107cf4e5f961b086ac4f06f3c2c56142b023dc`, 548,591 bytes;
  `sounds.json` sha1 `d4bdfba22b31906e9aded146b5e71ac4e16740b1`,
  599,251 bytes) into the batch evidence dir for the wiring diff
  only — it is a Mojang asset file, not committed.
- **Source comparison** against the 26.1 tree on
  `parity-sweep-26.1` (`~/workspace/mcir-work/sweep-26.1/versions/26.1/crates/`,
  synced to main). Read-only; nothing under `versions/26.1/` was
  modified, and no git command was run in the worktree.
- **Prepared 26.1 data**: `versions/26.1/.local/horizon-client/26.1/`
  (Batch 0's prepare run, GATE OK). `sounds.json` is absent (see §2,
  B10-G02); `en_us.json` IS present and md5-identical to the client
  jar's file (`98deafdb5e90e4438db30853494e1aeb`, 502,008 bytes) —
  verified directly, not assumed.
- **Oracle status** from the measured Batch 0 table ONLY
  (`docs/sweep/26.1/batch-0-oracles.json` — fixpoint big-compile of
  all 391 oracles against the named 26.1 jars, per classpath; see
  `docs/sweep/26.1/batch-0-measurement.md`). No import-level check
  was used for any oracle verdict — the 26.2 run's B10 failure (oracle
  triage by import check only, two items wrongly withdrawn) was not
  repeated. Every failing oracle in scope was read in the Batch 0
  big-compile client log (`b0/big-compile-client.log`, full error
  text extracted to `evidence/oracle-errors.txt`), and every missing
  symbol/package/constructor was javap-verified against the 26.1 jars
  by this batch (dumps in `evidence/javap-batch*.txt`,
  `evidence/client-classes.txt`, `evidence/server-classes.txt`).
  Driver→oracle mapping and classpath judgement come from the Batch 0
  driver map (`b0/drivers.json`): oracle_harness → both, client jar
  → client, server jar → server, no-flag → both superset.
- **Datagen reports** from
  `~/workspace/mcir-work/26.1.x/versions/26.1/datagen/reports/`
  (`registries.json`: 1,902 sound events, 49 block-entity types,
  1,168 blocks, 1,506 items, 157 entities) — never the 26.3 dump.

Checks that need a run (the 16 unmeasured runnable drivers, all 3
no-oracle e2e drivers, and every blocked driver's behaviour) are
recorded as **not measured**, not as passes. Render feel / sound
audibility that no driver can reach takes method 3 (in-game probe, §6)
— specified, not run; the owner's laptop playtest is owner-initiated
only and nothing was launched there.

Counting rule: one behaviour check = one vanilla-side/Rust-side
comparison settled by the evidence above. Oracle triage is counted
separately (§3: 38 failing in-scope oracles recorded across 4 items).
Per-section counts reconcile: §2 has 4 gap checks, §4 has 8 pass
checks, 4 + 8 = 12 = slice checks.total.

## 2. Behaviour gaps

### B10-G01 — Sign boards are not rendered: placed signs show floating text only (MAJOR)

REPRODUCES for 26.1 (the 26.2 withdrawal does not carry — 26.1 draws
sign boards in BE renderers, 1.21.7-style):

- All 24 sign block models in the vanilla 26.1 client jar are
  particle-only (no elements, no parent): `models/block/oak_sign.json`
  is `{"textures": {"particle": "minecraft:block/oak_planks"}}`,
  `oak_hanging_sign.json` the same with stripped-oak-log particle;
  every sign blockstate points at its per-wood model
  (`oak_sign.json` / `oak_wall_sign.json` → `minecraft:block/oak_sign`).
  There are NO `template_sign*.json` / `template_wall_sign.json`
  files in the 26.1 jar at all (26.2 has them, with elements).
- Vanilla draws the board in the BE renderer: `AbstractSignRenderer`
  declares abstract `getSignModel(S)` and `getSignSprite(WoodType)`
  plus `submitSign(...)` (javap), and `StandingSignRenderer` /
  `HangingSignRenderer` implement them with `createSignModel` /
  `createSignLayer` / `createHangingSignLayer` and `$Models` inner
  classes. `BlockEntityRenderers` registers SIGN and HANGING_SIGN
  (javap `-c`).
- The port assumes the opposite: `block_entity_render.rs:366`
  ("Sign text …; the boards are block models"), `:1010-1011` ("Sign
  boards and beds are block models in the terrain mesh"), and
  `block_entity_cem.rs:11-12` ("Signs, hanging signs and beds are
  block models here, so their files are not read"). `Kind::Sign`
  (`block_entity_render.rs:322-371`, `:367-371`) carries text layout
  only; the only sign drawing code is `sign_text_calls`
  (`:662`, modelling `AbstractSignRenderer.submitSignText`). With
  particle-only block models, the mesher emits nothing for the board,
  so a placed sign shows floating text only — exactly the 1.21.7
  defect. (The 1.21.11 sign-boards fix ac8dd9dc does not apply to
  26.1 as-is; the 1.21.7 tree's post-sweep fix is the nearer model.)
- Sign ITEMS are unaffected (flat generated sprites in vanilla too —
  §4 check 6).

### B10-G02 — No `sounds.json` in the prepared 26.1 resources: the game is silent and 6 audio drivers cannot run (MAJOR)

Carries from 1.21.7 and 26.2, confirmed against 26.1's own prepared
data:

- `audio_harness.require_sounds()` (`tools/audio_harness.py:35-37`)
  exits unless `.local/horizon-client/26.1/assets/minecraft/sounds.json`
  exists ("run cargo run -p horizon-prepare -- sounds first"). The
  file is absent: the prepared `.local/horizon-client/26.1/` (Batch
  0's full prepare run, exit 0) has a complete `assets/minecraft/`
  tree (atlases, blockstates, equipment, font, items, lang, models,
  particles, post_effect, shaders, texts, textures, waypoint_style)
  but no `sounds.json` and no `sounds/` directory; a find over the
  whole prepared tree returns nothing. It is also absent from the
  client jar — it ships only in the asset index.
- Root cause is the prepare flow, not the port code:
  `horizon-prepare --prepare` runs `prepare_with_progress` only;
  sounds need the separate `horizon-prepare sounds` subcommand
  (`crates/horizon-prepare/src/main.rs:240`, `add_assets("sounds", …)`)
  or `--complete` (`complete_resources`, `:276-282`, which adds
  sounds then runs the completeness check). Batch 0's prepare ran
  the export flow only (no `sounds` step — it needs a launcher asset
  store the VM lacks), so the 26.1 resources are in exactly the
  state 1.21.7's and 26.2's were.
- Runtime consequence, verified in code: the only runtime load path
  is `SoundRegistry::load`
  (`crates/horizon-client-render/src/sound_events.rs:262-287`), which
  silently skips a missing `sounds.json` per namespace
  (`:280-282`: `let Ok(text) = fs::read(…)` else `continue`). With
  no file the registry is empty and every event resolves to
  `Sound::empty()`. The port even documents the mode (`is_empty`,
  `:289-291`: "the resources hold no sounds.json (sounds not
  imported from the player's launcher when the game was prepared)").
- Driver consequence: all 6 `audio_harness` drivers gate-fail before
  any comparison — `sound_event`, `ambient_sound`, `music`,
  `situational_music`, `sound_engine`, `audio_attributes` (the same 6
  as 1.21.7/26.2, verified by grep over `tools/test_*_parity.py`).
  Worse than 26.2: for 26.1 ALL SIX are oracle-blocked as well as
  data-gated (B10-O04 — including `sound_event`, whose oracle
  passed for 26.2), so the sounds step alone unblocks none of them
  until their oracles are repaired too.
- The wiring spec itself is clean (see §4, check 2): vanilla 26.1
  `sounds.json` defines 1,902 events against 1,902 registry events;
  the only registry event without a `sounds.json` entry is the
  intentional `minecraft:intentionally_empty` sentinel, and the only
  `sounds.json` entry without a registry event is a vanilla orphan
  (`entity.chicken_picky.step`). Once `horizon-prepare sounds` runs
  AND the oracles are repaired, the 6 drivers can measure resolution
  parity.

### B10-G03 — Beds are invisible when placed (MAJOR; bed items OK)

REPRODUCES for 26.1, in part (the 26.2 withdrawal does not carry —
26.1 keeps the bed block-entity type and BE renderer; but the 26.1
port fixed the item half during the port):

- Placed: vanilla 26.1 `models/block/bed.json` is particle-only (no
  elements — `{"textures": {"particle":
  "minecraft:block/oak_planks"}}`) and every bed blockstate points
  at it (`red_bed.json` → `minecraft:block/bed`); there are NO
  `template_bed_head`/`template_bed_foot` or per-colour bed models
  in the 26.1 jar (26.2 has them). `BedRenderer` draws the whole
  bed (javap: `createHeadLayer`/`createFootLayer`, `submitPiece`,
  `modelTransform`), registered for `BlockEntityType.BED`
  (`BlockEntityRenderers`, javap `-c`; `minecraft:bed` IS a 26.1
  block-entity type covering all 16 bed blocks —
  `registries.json`, and `BlockEntityType.BED` is a static final on
  the old holder, javap-verified). The port has no Bed kind in
  `block_entity_render.rs` (Kind enum `:322-371` — no Bed arm;
  `classify()` falls through to `Kind::None`, `:593`) and no bed
  drawing code anywhere in the render crates (only sleep mechanics
  in `bed_geometry.rs` / `game/beds.rs`), so the mesher emits
  nothing for a placed bed. The same wrong "beds are block models"
  assumption as G01 (`block_entity_render.rs:1010-1011`,
  `block_entity_cem.rs:11-12`).
- Items: FIXED in the 26.1 port (unlike 1.21.7). Vanilla renders all
  16 bed items via `minecraft:composite` of two `minecraft:special`
  nodes with `minecraft:bed` model type, head + foot
  (`items/red_bed.json`). The port's `SpecialModel::decode` HAS a
  `minecraft:bed` arm (`horizon-assets/src/item_model/special.rs:120`,
  `SpecialKind::Bed { part: head/foot }`, texture from the dye set),
  and the client renders it (`horizon-client/src/item_render/special.rs:357`,
  `:710-713` — "26.1 only: `BedSpecialRenderer` draws the head or
  foot model with the bed texture"). All 16 bed items decode and
  have a draw path. The item half of the 1.21.7 defect is therefore
  closed for 26.1; only the placed-bed half reproduces.

### B10-G04 — Seven vanilla block-entity renderers have no port equivalent (MINOR)

Carries from 1.21.7 and 26.2, same seven as 26.2 (the shelf is a 26.x
addition present in 26.1 too; beds are G03, not counted here).
Vanilla 26.1 registers 27 block-entity renderers
(`BlockEntityRenderers`, javap `-c` verified on the static
initializer: SIGN, HANGING_SIGN, MOB_SPAWNER, PISTON, CHEST,
ENDER_CHEST, TRAPPED_CHEST, ENCHANTING_TABLE, LECTERN, END_PORTAL,
END_GATEWAY, BEACON, SKULL, BANNER, STRUCTURE_BLOCK,
TEST_INSTANCE_BLOCK, SHULKER_BOX, BED, CONDUIT, BELL, CAMPFIRE,
BRUSHABLE_BLOCK, DECORATED_POT, TRIAL_SPAWNER, VAULT,
COPPER_GOLEM_STATUE, SHELF — 26.2's 26 plus BED). The port's
`block_entity_render.rs` `Kind` (`:322-371`) covers 13 kinds
(chest incl. trapped/ender/copper, shulker, banner, skull, bell,
decorated pot, conduit, enchanting table, lectern, copper golem
statue, end portal, end gateway, sign text) = 15 of the 27 slots,
plus beacon beams elsewhere (`beacon_beam`, same file `:3456`,
`gfx/pack_entities.rs:820`) = 16 slots. `classify()` falls through
to `Kind::None` for everything else (`:593`). Missing, with the
visible effect of each (block-model state from the jar):

- **Mob spawner / trial spawner** — the cage is a block model
  (`spawner.json` → `cube_all_inner_faces`;
  `trial_spawner.json` → `cube_bottom_top_inner_faces`); the SPINNING
  MOB (and trial spawner's items) are BE-rendered
  (`SpawnerRenderer` / `TrialSpawnerRenderer`). Port has the spawner
  DATA types (`chunk/block_entity.rs:67-70`, `mobs::spawner`,
  `mobs::trial_spawner`) but no spinning-mob rendering: spawner cages
  render empty.
- **Campfire** — logs are a block model (`template_campfire`, 7
  elements); the COOKING ITEMS are BE-rendered (`CampfireRenderer`).
  Port has the campfire BE data (`chunk/block_entity.rs:77-78`) but
  no item rendering — cooking food is invisible.
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
  (`chunk/block_entity.rs:96`) but no rendering — the emerging item
  is invisible while brushing.
- **Vault** — the vault block renders via `template_vault` (2
  elements); the DISPLAY ITEM is BE-rendered (`VaultRenderer`). Port:
  `vault.rs` is server-tick logic only; no display-item rendering.
- **Shelf** — the shelf body is a block model
  (`template_shelf_body`, 3 elements); the THREE DISPLAYED ITEMS are
  BE-rendered (`ShelfRenderer`, registered for
  `BlockEntityType.SHELF`). Port has the shelf BE data
  (`chunk/block_entity.rs:97-98`: "A shelf holds three displayed item
  stacks") but no Shelf kind — displayed items are invisible.
- Unverified, noted not claimed: **structure block** and **test
  instance block** both have cube block models
  (`structure_block.json` / `test_instance_block.json` →
  `cube_all`) and vanilla renderers
  (`BlockEntityWithBoundingBoxRenderer` / `TestInstanceRenderer`)
  whose visible in-play output I did not verify — the port has
  dedicated files (`structure_block.rs`, `test_instance_block.rs`,
  `jigsaw_block.rs`) whose rendering role needs a driver run to
  judge. Not counted as gap checks. No 26.1 BE type lacks a vanilla
  renderer the way 26.2's `potent_sulfur` did (26.1 has no
  sulfur types at all), so there is no "correctly unrendered" case
  to record here.

## 3. Oracle defects

Measured Batch 0 status throughout (see §1); no oracle was repaired
in this batch. The 26.1 picture is much worse than 26.2's, because
26.1 predates several 26.2 APIs the oracles assume: the `Hud` class
split out of `Gui`, the `EntityTypes` holder move, `FirstPersonHandsAndItems`,
`UvMapping`, `renderpearl`, `shadeDirectionOverride`, the
`getX()`→`x()` packet accessor renames, and more. The remaining
breaks are 26.3-only APIs that 26.1 lacks, plus method-level
signature drift — every one below was verified by javap against the
26.1 jars, not taken from Batch 0's attribution alone.

### B10-O02 — `ParityFont` needs `VanillaPackResources.asResourceManager()`, absent in 26.1 (MINOR)

REPRODUCES with the identical 1-error break as 26.2.
`ParityFont.java:43` calls `pack.asResourceManager()` on a
`VanillaPackResources`; the method does not exist in 26.1 (`error:
cannot find symbol, symbol: method asResourceManager(), location:
variable pack of type VanillaPackResources`). Client cp: 1 error;
server cp: fails on the client-only packages (expected). Javap of
26.1's `VanillaPackResources` shows it HAS `asProvider()` but no
`asResourceManager()` — the repair is to build the `ResourceManager`
from the provider (or find 26.1's equivalent conversion). Same
highest-leverage single-symbol repair as 26.2: `ParityFont` blocks 8
drivers programme-wide (7 in this batch: block_entity_render,
chat_screen, hud, pack_screen, recipe_book_render, subtitle,
text_display_render) and cascades into one more oracle
(`VanillaRecipeBookOracle` is a `VanillaMenuOracle` cascade, not a
ParityFont one, in 26.1; `VanillaSubtitleOracle` has its own error
here — see B10-O04).

### B10-O03 — Render/model oracle family does not compile for 26.1 (MAJOR)

Twelve oracles fail on own errors on the client cp, plus two
cross-batch oracles recorded here with cross-references. (In 26.2
this item was a single oracle — the UvMapping break; for 26.1 the
whole family fails again, 1.21.7-style, with 26.1-specific causes.)

| Oracle | Errors (client cp) | Cause (javap-verified) |
|---|---|---|
| VanillaBlockEntityRenderOracle | 9, missing-symbol | `UvMapping` import (`:23`, 26.3-only class, 0 jar entries) + cast at `:155`; `CaseLevel` lacks abstract `Level.fuelValues()` (Batch 0 G1); `acceptRenderable` no-override (`:304`); `Font.prepare8xTextOutline` absent (`:309` — Font has only `prepareText`); `SignText` ctor takes `Component[]`, not `ArrayList<Component>` (`:332`); `PotDecorations` ctor takes `Item`/`Optional<Item>`, not `Optional<ItemStackTemplate>` (`:368`); `submitModel` call missing the `CrumblingOverlay` arg (`:418`); `BannerRenderer.submitPatterns` needs `DyeColor, BannerPatternLayers, CrumblingOverlay` (`:419`) |
| VanillaFirstPersonOracle | 11, missing-symbol | `FirstPersonHandsAndItems` absent (0 jar entries; 7 errors incl. import `:14`); `LocalPlayer` ctor (Batch 0 G2, `:153`); `addBreakingBlockEffects(BlockPos, Direction, boolean)` no-override (`:125` — 26.1 has the 2-arg singular `addBreakingBlockEffect`); `itemUsed(InteractionHand)` absent from `Player`/`LocalPlayer` (`:183-185`) |
| VanillaItemModelOracle | 10, missing-symbol | `EntityTypes` import (Batch 0 G4, `:37` + `:216`); `ResourceManager.Selector` absent (`:136-137` — 26.1's `listResources` takes a `Predicate<Identifier>`); `MaterialInfo.shadeDirectionOverride()` absent (`:260` ×2 — 26.1 has only `shade()`/`lightEmission()`); `ItemQuads` absent (0 jar entries, `:289`); `VanillaRegistries.createWorldLookup()` absent (`:321` — 26.1 has `createLookup()`); `MaterialBaker` is an interface in 26.1, cannot be instantiated (`:379`); `BundleContents.asMutable()` absent (`:404`) |
| VanillaBreakingModelOracle | 2, missing-symbol | `BlockModelFeatureRenderer` absent (0 jar entries, `:12` + `:68`) |
| VanillaSectionMeshOracle | 7, other | `MaterialBaker` is an interface — `extends` fails (`:84`, "no interface expected here"); `BlockStateBase.emissiveRendering` takes `(BlockGetter, BlockPos)`, not no-arg (`:313`); `BlockState.isLightPermeable()` absent (`:314`); `VanillaRegistries.createWorldLookup()` absent (`:347`) |
| VanillaModelBakingOracle | 3, missing-symbol | `CuboidModelElement.shadeDirectionOverride()` absent (`:53`, `:62` ×2 — 26.1 has only `shade()`/`lightEmission()`) |
| VanillaDragonRenderOracle | 1, no-override | `VertexConsumer.setUv3` absent (`:31` — 26.1 has setUv/setUv1/setUv2 only) |
| VanillaEntityAnimOracle | 5, mixed | `VertexConsumer.setUv3` absent (`:106`); `ArmedEntityRenderState.swingAnimation`/`currentSwing` absent (`:333-334` — 26.1 has `swingAnimationType` only); `LivingEntity.SwingDescription` absent (0 jar entries, `:334`); `PoseStack.rotateDegrees` absent (`:350` — 26.1 has `rotateAround`) |
| VanillaTextDisplayRenderOracle | 3, mixed | `ServerLevel.getNextEntityId()` absent (`:52` — no such method on `ServerLevel`/`Level`); one more no-override (`:225`); `Font.prepareBackground` absent (`:243` — Font has only `prepareText`) |
| VanillaEnvironmentOracle | 1, missing-symbol | `ClockInstance` (`:71` — see B10-O04 note: exists only as the package-private nested `ClientClockManager$ClockInstance` / `ServerClockManager$ClockInstance`; no top-level class) |
| VanillaTitleDebugOracle | 4, missing-symbol | `Gui.canInterruptScreen()` absent (`:29`, `:31`, `:34`, `:36` — the `Gui` class has no such method in 26.1) |
| VanillaPackResourcesOracle | 1, missing-symbol | `PackResources.forEach` absent (`:65` — the interface has `listResources`/`getResource` but no `forEach`); blocks the `resource_pack` driver (9-harness check — the cheapest of the three failing 9-harness oracles) |
| VanillaBlockEntityAnimOracle | 3, missing-override | `Level.fuelValues()` abstract gap (Batch 0 G1, `:102`) + `EntityTypes` (G4, `:184` ×2). Driver `test_block_entity_anim_parity` is outside this batch's 50 — recorded here, cross-ref Batch 4 (block entities). |
| VanillaTextDisplayOracle | 1, no-override | `ServerLevel.getNextEntityId()` absent (`:20`, same as TextDisplayRender). Driver `test_text_display_parity` is outside this batch's 50 — recorded here, cross-ref Batch 3/4 (entity scope). Cascades: `VanillaTextDisplayTransformOracle` (clean, fails only via this one). |

Drivers blocked in this batch: block_entity_render, first_person,
item_model, breaking_model, model_baking, dragon_render, entity_model,
text_display_render, environment_render, title_debug (10). The
`block_entity_render` driver — the one that would catch B10-G01/G03/G04
— needs `ParityFont` (B10-O02) + this oracle + `VanillaTextDisplayRenderOracle`
all repaired first. Repair notes: the `UvMapping` fix is the same
one-line translation as 26.2 (26.1's
`OrderedSubmitNodeCollector.submitModel` 8th parameter IS the
`TextureAtlasSprite` directly, javap-verified); the `submitModel` /
`submitPatterns` calls additionally need their trailing
`CrumblingOverlay` (and pattern) arguments; `MaterialBaker` repairs
need `implements` + a factory instead of `new`.

### B10-O04 — Sound oracle family does not compile for 26.1 (MAJOR)

Worse than 26.2 (where 6 of 10 passed): for 26.1 only 2 of the 12
sound oracles pass. Per-oracle measured status (client cp):

| Oracle | Status | Cause |
|---|---|---|
| VanillaSoundEventOracle | FAIL, 2 errors | `SimpleSoundInstance.getOrResolve(SoundManager)` absent (`:83`); `Sound.getAttenuationDistance(float)` — 26.1's takes no arguments and returns int (`:104`). NEW failure vs 26.2 (passed there). Cascades: `VanillaSoundRig` → `VanillaMusicOracle` (Batch 0 G9). |
| VanillaSoundRig | FAIL, cascade | depends on broken `VanillaSoundEventOracle` — passes for free once it is repaired |
| VanillaMusicOracle | FAIL, cascade | depends on broken `VanillaSoundRig` (chained cascade) |
| VanillaSituationalMusicOracle | FAIL, 4 errors | `net.minecraft.client.gui.Hud` absent (0 jar entries — 26.1 keeps the HUD in `Gui`; `:11`, `:81` ×2, `:85`) |
| VanillaAmbientOracle | FAIL, 1 error | `LocalPlayer` ctor — Batch 0 G2 (`:50`) |
| VanillaSoundEngineOracle | FAIL, 4 errors | `LocalPlayer` ctor (G2, `:33`) + `UnderLiquidSubSound` absent (`:72` ×2) + `UnderLiquidAmbientSoundInstance` absent (`:96`, 0 jar entries for `UnderLiquid*`) |
| VanillaFireStarterSoundOracle | FAIL, 1 error | `LocalPlayer` ctor — Batch 0 G2 (`:61`) |
| VanillaAudioAttributesOracle | FAIL, 1 error | `ClockInstance` (`:72` — exists only as the package-private nested `ClientClockManager$ClockInstance` / `ServerClockManager$ClockInstance`; the oracle's bare `new ClockInstance()` has no such top-level class in scope, and the nested class is not accessible outside its package — the repair needs `ClientClockManager`'s own API, not a bare import) |
| VanillaSubtitleOracle | FAIL, 1 error | `Util.setTimeSource` absent (`:188` — 26.1's `Util` exposes the `timeSource` FIELD, not a setter; repair is a field assignment). NOT a cascade in 26.1 (it was in 26.2 — here it fails on its own error). |
| VanillaEntitySoundPacketOracle | FAIL, 3 errors | `MinecraftServer` ctor (Batch 0 G6, `:62` — 26.1 takes 10 args incl. `Proxy`/`LevelLoadListener`, oracle passes 11 in 26.3 order) + `getCommandSpamThresholdSeconds` / `getChatSpamThresholdSeconds` no-override (`:77-78` — absent from 26.1's `MinecraftServer`) |
| VanillaZombieCureSoundOracle, VanillaStopSoundWireOracle | PASS | — |

Drivers blocked: sound_event, music, situational_music,
ambient_sound, sound_engine, entity_bound_sound, fire_starter_sound,
audio_attributes, subtitle, arrow_hit_sound, entity_sound_packet (11).
The three `LocalPlayer`-ctor failures are Batch 0's G2 group (one
signature translation rescues all 20 oracles programme-wide carrying
the error) — cross-referenced, not double-minted. Note the 6
`audio_harness` drivers are data-gated by B10-G02 regardless — oracle
repair alone does not make them runnable.

### B10-O05 — UI/screen oracle family needs `renderpearl` + `Hud`, 26.3-only APIs (MAJOR)

`com.mojang.renderpearl.*` appears ZERO times in the 26.1 client jar
— the render-pipeline API the 26.3-derived oracles use does not exist
in 26.1 at all. Eight oracles import it (Batch 0 G5), and five more
fail on the missing `Hud` class or neighbouring 26.3-only APIs:

| Oracle | Errors (client cp) | Missing / cause |
|---|---|---|
| VanillaHudOracle | 34, missing-package | `renderpearl.api.pipeline.RenderPipeline` (`:4`); `net.minecraft.client.gui.Hud` (`:16`); `contextualbar.ExperienceBar` (`:17` — 26.1 has only `ExperienceBarRenderer`); `LocalPlayer` ctor (G2, `:133`) |
| VanillaPackScreenOracle | 13, missing-package | `RenderPipeline` (`:4` + 3 use sites); text-input overrides (`:224-226`); `Gui` ctor — 26.1's takes only `Minecraft` (`:246`, G6); `setScreen`/`screen()` no-override (`:247`, `:261`); `Language.DEFAULT_INSTANCE` absent (`:309`, `:323` — 26.1's `Language` has only the `DEFAULT` string) |
| VanillaRecipeBookScreenOracle | 4, missing-package | `RenderPipeline` only (`:4` + 3 use sites) |
| VanillaLoomScreenOracle | 5, missing-package | `RenderPipeline` (`:4` + 3 use sites) + `Hud` (`:68`) |
| VanillaWorldBorderRenderOracle | 34, missing-package | `renderpearl.api.buffers.GpuBuffer`, `api.commands.CommandEncoder`, `api.commands.RenderPass`, `api.device.GpuDevice`, `api.pipeline.RenderPipeline` (`:5-9`), plus `net.minecraft.client.gui.Hud` (`:19`) and `net.minecraft.client.renderer.DynamicGpuData` (`:21`, 0 jar entries) |
| VanillaSpriteAnimationOracle | 2, missing-package | `renderpearl.api.buffers.GpuBufferSlice` (`:6`, `:41`). Driver `test_sprite_animation_parity` is outside this batch's 50 — recorded here as render-family. |
| VanillaTranslucencyOracle | 4, mixed | `renderpearl.api.pipeline.IndexType` (`:5`); `MeshData.decodeQuadCentroids` absent (`:26`); `MeshData.SortState.writeSortedIndexBuffer` absent (`:50`). Driver `test_translucency_parity` is outside this batch's 50 — recorded here as render-family. |
| VanillaTabListOracle | 36, missing-package | `com.mojang.authlib.services.*` (8 imports, `:8-15` — absent from authlib 7.0.63, Batch 0 G7) + `RenderPipeline` (`:21`) + `Hud` (`:32`) + `world.scores.TeamColor` (`:53` — 26.1 has only `client.color.item.TeamColor`). Driver `test_tab_list_parity` is outside this batch's 50 — recorded here as UI-family. |
| VanillaChatScreenOracle | 5, missing-symbol | `Hud` (`:20`, `:278-279`) + `CommandSourceStack.getAvailablePostEffects()` absent (`:120`, no-override + missing-symbol) |
| VanillaMenuOracle | 4, missing-symbol | `net.minecraft.util.Prediction` absent (`:138`, same break as 1.21.7's B5-O01/B11-O01); `DedicatedServer` ctor — 26.1 takes 8 args, oracle passes 10 nulls (`:65`); `EntityTypes.PLAYER` (Batch 0 G4, `:128-129`). Cascades: `VanillaRecipeBookOracle` + `VanillaPlaceRecipeOracle` (Batch 0 G9) — recipe-book drivers, cross-ref Batch 4/7. |
| ServerPackClient | 1, no-override | one no-override (`:264`). Driver `test_server_resource_pack_parity` is outside this batch's 50 — recorded here, cross-ref Batch 11 (network/pack scope?) — coordinator to assign. |

Drivers blocked in this batch: hud, pack_screen, recipe_book_render,
screen_layout, world_border_render, chat_screen (compiles
`VanillaPackScreenOracle` as a dependency), menu_click (7). Unlike
1.21.7, where these screen oracles needed rewrites against 1.21.7's
screen APIs, for 26.1 the repair is a 26.3→26.1 backport: find what
26.1's screens use where 26.3 uses `RenderPipeline` (26.1's screens
render through the `SubmitNodeCollector`/`RenderType` path — e.g.
`GuiGraphics` — not a GPU pipeline object) and where 26.3 uses `Hud`
(26.1's `Gui`). `VanillaWorldBorderRenderOracle` and
`VanillaTabListOracle` are the deepest (34/36 errors each); the other
renderpearl-only oracles miss exactly one import each.
`VanillaMenuOracle`'s repair is mechanical (drop the `Prediction`
parameter, fix the ctor arity, `EntityType.PLAYER`).

The family's other members, measured: `VanillaContainerScreenOracle`
and `VanillaOptionsOracle` PASS on the client cp;
`VanillaPackSelectionOracle` and `VanillaRecipeBookViewOracle` also
PASS on the client cp (their drivers, `pack_selection` /
`recipe_book_view`, are outside this batch's 50 — the recipe-book
logic drivers belong to Batch 4/7 per the boundary note in §7).

### Withdrawn oracle item

- **B10-O01 (WITHDRAWN).** `VanillaOptionsOracle` PASSES on the
  client + both classpaths in the measured Batch 0 table (server cp
  fails only on the client-only packages, expected). The 1.21.7 O01
  defect (10 errors, category chain) does not exist for 26.1 — see
  §4 check 7. The `options` driver is runnable by javac (§5).

### Passing in-scope oracles (auditable coverage)

From the Batch 0 table, client cp PASS: `ParityData`,
`VanillaBlockStateModelOracle`, `VanillaCloudRenderOracle`,
`VanillaContainerScreenOracle`, `VanillaContainerTitleOracle`,
`VanillaCuboidModelOracle`, `VanillaFireworkTooltipOracle`,
`VanillaLoreTooltipOracle`, `VanillaMipmapOracle`,
`VanillaModelChoiceOracle`, `VanillaModelResolutionOracle`,
`VanillaOptionsOracle`, `VanillaStopSoundWireOracle`,
`VanillaTooltipDisplayOracle`, `VanillaWeatherRenderOracle`,
`VanillaWorldOptionsOracle`, `VanillaZombieCureSoundOracle` (17 —
all tied to runnable drivers in §5), plus family members whose
drivers sit outside this batch's 50: `VanillaEntityModelExporter`
(no driver), `VanillaOptionsTableExporter` (no driver),
`VanillaPackSelectionOracle`, `VanillaRecipeBookViewOracle`,
`VanillaStackModelFlagsOracle`, `VanillaStorageMenuOracle`,
`VanillaSimpleContainerOracle`, `VanillaFloatTextOracle`,
`VanillaWorldSpawnPacketOracle`, `GoatVoiceTranscript` (no driver).
Out-of-scope oracles seen in the keyword sweep but NOT recorded here
(other batches' families): `VanillaCollisionContextOracle`
(collision), `VanillaDragonContactsOracle` (entity),
`VanillaVehicleContainerOracle` (vehicles),
`VanillaPacketCodecOracle` (packets), `VanillaWorldBorderOracle`
(cascade via `VanillaFaceSupportOracle`). `VanillaProtocolClient`
(34 errors) and `SecureChatClient` (8 errors) are shared
packet-client helpers recorded in §5 as Batch 0 residual, not minted
as B10 items (same treatment as 26.2).

## 4. Passing checks (8)

1. **Sound-wiring diff (registry ↔ sounds.json) — PASS.** Vanilla 26.1
   `sounds.json` (asset index 30, sha1-verified) defines 1,902
   events; `registries.json` lists 1,902 sound events. The only
   registry event without a `sounds.json` entry is
   `minecraft:intentionally_empty` — vanilla's intentional sentinel
   (same as 1.21.7 and 26.2). The only `sounds.json` entry without a
   registry event is `minecraft:entity.chicken_picky.step` — a
   vanilla quirk: the picky chicken has registry events for
   ambient/death/hurt but no step event, while `sounds.json` defines
   the step as an event-ref to `entity.chicken.step`; it is
   unreachable in the port (resolution is keyed by registered event
   id) and is not a port defect. 1,804 entries carry subtitles; 7
   entries have empty sound lists (the same 7 vanilla-intentional
   empties as 1.21.7/26.2: `block.fungus.fall`/`hit`,
   `entity.cod`/`salmon`/`snow_golem`/`tropical_fish` ambient,
   `music.nether.warped_forest`); 7,615 sound refs (4,927 string +
   2,688 dict) + 61 event refs. The wiring spec is fully
   established; the port-side failure is the missing file (B10-G02),
   not a wiring mismatch.
2. **First-person swap ticker — PASS (B10-G05 WITHDRAWN).** Vanilla
   26.1 HAS the swap ticker: `LivingEntity.itemSwapTicker`
   (protected int, both jars) and `Player.getItemSwapScale(float)`
   (javap). The oracle reads them reflectively and emits the two
   columns (`VanillaFirstPersonOracle.java:251`, `:399`), and the
   Rust probe emits the same two columns
   (`first_person_probe.rs:384-385`: `strength.swap_ticker`,
   `swap_scale(1.0, speed)`) from `attack_strength.rs`
   (`swap_ticker`, `swap_scale`, `:13`, `:73-74`). Probe and oracle
   agree on format for 26.1; the 1.21.7 format mismatch does not
   exist here. (Values still need the driver run — and the driver
   is oracle-blocked on `FirstPersonHandsAndItems` etc., B10-O03 —
   but the swap-ticker columns themselves agree.)
3. **Silent-goat ambient suppression — PASS (B10-G06 WITHDRAWN).**
   Vanilla 26.1: `Entity.playSound` checks `isSilent()` first
   (`javap -c`: `isSilent()` then `ifne` skip, both overloads) and
   `Mob.playAmbientSound` routes through it, so a `Silent:1b` goat
   makes no ambient sound. The 26.1 port implements exactly this:
   `Mob::voice` (`horizon-client/src/mobs.rs:4707-4719`) draws the
   pitch first ("Java evaluates the arguments … before the call")
   then returns early `if self.silent`, `play_ambient` routes
   through `voice()` (`mobs/ai.rs:1481`, `:1587`), and a unit test
   pins it (`mobs.rs:6552`,
   `a_silent_mob_makes_no_ambient_sound`: the loud cow plays
   `entity.cow.ambient`, the silent one plays nothing, RNG state
   identical). The `test_goat_voices_parity.py:4` docstring
   ("Silent ambient suppression is a separate gap") is STALE for
   26.1 — the driver still gates those two controls out (`:144-147`),
   so the suppression is unmeasured by the driver, but the code is
   there. Driver-hygiene follow-up (§7), not a behaviour gap.
4. **Sign items — PASS (no divergence).** Vanilla renders sign items
   as flat generated sprites (`items/oak_sign.json` →
   `minecraft:model` → `item/oak_sign` → `item/generated`; same for
   hanging signs), same as the port. (26.1's jar has no
   `StandingSignSpecialRenderer` / `HangingSignSpecialRenderer` at
   all — the 1.21.7 classes are gone; nothing to diverge on. The
   port's `StandingSign`/`HangingSign` special kinds exist for other
   versions' items and are unused by 26.1's item definitions.)
5. **Special-model decoder coverage — PASS.** Exhaustive scan of all
   1,506 vanilla 26.1 item definitions (top-level + nested): 1,359
   `minecraft:model`, 71 `select`, 51 `special`, 16 `composite` (all
   beds), 7 `condition`, 2 `range_dispatch` — the port's
   `ItemModelNode::decode` handles all six top-level types
   (`item_model.rs:420-540`, incl. `minecraft:empty`). Special types
   used (nested walk): banner 16, bed 32, shulker_box 17, chest 13,
   copper_golem_statue 32, head 6, conduit 1, decorated_pot 1,
   player_head 1, shield 2, trident 2 — every one has a
   `SpecialModel::decode` arm (`item_model/special.rs:111-189`,
   incl. the `minecraft:bed` arm at `:120` that 1.21.7's port
   lacked); the copper-golem poses used (standing / sitting /
   running / star, 8 each) are exactly the decoder's allowed set
   (`special.rs:142`), the head kinds (skeleton / wither_skeleton /
   zombie / creeper / piglin / dragon) match, and no special item
   lacks a `base`. Select properties used (block_state 12,
   display_context 26, trim_material 29, local_time 2,
   context_dimension 1, charge_type 1) each appear in the decoder's
   select handling (`item_model.rs:728-741`), and the 7 `condition`
   (bow, compass, elytra, fishing_rod, goat_horn, shield,
   wolf_armor) + 2 `range_dispatch` (brush, recovery_compass) items
   decode through the same `ItemModelNode::decode` arms. The bed
   specials additionally have a client draw path
   (`item_render/special.rs:357`, `:710-713`).
6. **Beacon beams — PASS (present).** `beacon_beam`
   (`block_entity_render.rs:3456`, modelling vanilla
   `BeaconRenderer.submitBeaconBeam`) and the `gbuffers_beaconbeam`
   packing (`gfx/pack_entities.rs:820`) both exist; the beacon base
   block model has elements, so the block + beam both have draw
   paths. Exact beam geometry/colour is unverified statically (no
   beacon driver in scope).
7. **BE renderer kinds present (16/27 slots) — PASS.** Chest
   (single/left/right, trapped/ender/copper), shulker, banner, skull,
   bell, decorated pot, conduit, enchanting table, lectern, copper
   golem statue, end portal, end gateway, sign text — all have a
   `Kind` arm (`block_entity_render.rs:322-371`), plus beacon beams
   (check 6). The missing 11 slots are B10-G01's sign boards (2
   slots: sign + hanging sign — text is present, boards are not),
   B10-G03's bed (1), B10-G04's 7, plus the 2 unverified
   (structure/test-instance).
8. **Options API surface — PASS (B10-O01 WITHDRAWN).** Everything the
   options oracle touches exists in 26.1 with the 26.x form:
   `Options(Minecraft, File)` 2-arg ctor, `options.keyMappings`,
   `KeyMapping.getName()` / `getDefaultKey()`, `ToggleKeyMapping`,
   and — the 1.21.7 break that does not apply here —
   `KeyMapping.getCategory()` returns `Category` with `id()`
   (javap-verified; in 1.21.7 it returns a plain `String`).
   `VanillaOptionsOracle` PASSES on the client + both classpaths in
   the measured Batch 0 table. (Option VALUES/defaults per key were
   not diffed — that needs the options driver run once it runs.)

## 5. Per-driver status (all 50, measured)

Oracle status from the measured Batch 0 table
(`docs/sweep/26.1/batch-0-oracles.json`); "fail" = fails on the
classpath the driver's path uses. `cp` is that classpath
(oracle_harness → both, client jar → client, server jar → server,
no-flag → both superset). Driver→oracle map from
`sweep-26.1-batches/b0/drivers.json`.

### Runnable by javac — NOT RUN (16)

| Driver | Oracle(s) | Note |
|---|---|---|
| blockstate_model | VanillaBlockStateModelOracle + VanillaCuboidModelOracle pass | Unmeasured |
| cloud_render | VanillaCloudRenderOracle pass | Unmeasured |
| container_screen | VanillaContainerScreenOracle pass | Unmeasured |
| container_title | ParityData + VanillaContainerTitleOracle pass | Unmeasured |
| cuboid_model | VanillaCuboidModelOracle pass | Unmeasured |
| firework_tooltip | VanillaFireworkTooltipOracle pass | Unmeasured |
| lore_tooltip | VanillaLoreTooltipOracle pass | Unmeasured |
| mipmap | VanillaMipmapOracle pass | Unmeasured |
| model_choice | VanillaModelChoiceOracle + VanillaCuboidModelOracle pass | Unmeasured |
| model_resolution | VanillaModelResolutionOracle + VanillaCuboidModelOracle pass | Unmeasured |
| options | VanillaOptionsOracle pass | Unmeasured |
| stop_sound_wire | VanillaStopSoundWireOracle pass | Unmeasured |
| tooltip_display | VanillaTooltipDisplayOracle pass | Unmeasured |
| weather_render | VanillaWeatherRenderOracle pass | Unmeasured |
| world_options | VanillaWorldOptionsOracle pass | Unmeasured |
| zombie_cure_sound | VanillaZombieCureSoundOracle pass | Unmeasured |

Unlike 26.2, NO runnable driver is data-gated by B10-G02: all 6
`audio_harness` drivers are oracle-blocked for 26.1 (§5, blocked
table), so the sounds.json gate never gets reached.

### Oracle-blocked (31)

| Driver | Blocking oracle(s) | Item |
|---|---|---|
| ambient_sound | VanillaAmbientOracle 1e (LocalPlayer ctor) + VanillaSoundRig cascade | B10-O04 / Batch 0 G2+G9 |
| arrow_hit_sound | VanillaEntitySoundPacketOracle 3e (MinecraftServer ctor) | B10-O04 / Batch 0 G6 |
| audio_attributes | VanillaAudioAttributesOracle 1e (ClockInstance) | B10-O04 |
| block_entity_render | ParityFont 1e + VanillaBlockEntityRenderOracle 9e (UvMapping + 7 more) + VanillaTextDisplayRenderOracle 3e | B10-O02, B10-O03 |
| breaking_model | VanillaBreakingModelOracle 2e (BlockModelFeatureRenderer) + VanillaSectionMeshOracle 7e (MaterialBaker interface) | B10-O03 |
| chat_screen | ParityFont 1e + VanillaChatScreenOracle 5e (Hud) + VanillaPackScreenOracle 13e (renderpearl) | B10-O02, B10-O05 |
| dragon_death_roar | VanillaProtocolClient 34e (packet-client helper) | Batch 0 O6 residual |
| dragon_render | VanillaDragonRenderOracle 1e (setUv3) | B10-O03 |
| entity_bound_sound | VanillaSoundEngineOracle 4e (LocalPlayer ctor) + VanillaSoundEventOracle 2e + VanillaSoundRig cascade | B10-O04 / Batch 0 G2+G9 |
| entity_model | VanillaEntityAnimOracle 5e (setUv3, swing fields, rotateDegrees) | B10-O03 |
| entity_sound_packet | VanillaEntitySoundPacketOracle 3e (MinecraftServer ctor) | B10-O04 / Batch 0 G6 |
| environment_render | VanillaEnvironmentOracle 1e (ClockInstance) | B10-O03 |
| fire_starter_sound | VanillaFireStarterSoundOracle 1e (LocalPlayer ctor) | B10-O04 / Batch 0 G2 |
| first_person | VanillaFirstPersonOracle 11e (FirstPersonHandsAndItems + LocalPlayer ctor + 2 no-override) | B10-O03 |
| goat_voices | VanillaProtocolClient 34e (packet-client helper) | Batch 0 O6 residual |
| hud | ParityFont 1e + VanillaHudOracle 34e (renderpearl + Hud + ExperienceBar) | B10-O02, B10-O05 |
| item_model | VanillaItemModelOracle 10e (EntityTypes, Selector, ItemQuads, MaterialBaker, …) | B10-O03 |
| menu_click | VanillaMenuOracle 4e (Prediction + DedicatedServer ctor + EntityTypes) | B10-O05 / Batch 0 G4 |
| model_baking | VanillaModelBakingOracle 3e (shadeDirectionOverride) | B10-O03 |
| music | VanillaMusicOracle cascade + VanillaSoundRig cascade (via VanillaSoundEventOracle 2e) | B10-O04 / Batch 0 G9 |
| pack_screen | ParityFont 1e + VanillaPackScreenOracle 13e (renderpearl + Gui ctor + Language) | B10-O02, B10-O05 |
| recipe_book_render | ParityFont 1e + VanillaRecipeBookScreenOracle 4e (renderpearl) | B10-O02, B10-O05 |
| screen_layout | VanillaLoomScreenOracle 5e (renderpearl + Hud) | B10-O05 |
| secure_chat | SecureChatClient 8e (no-override, packet-client helper) | Batch 0 O6 residual |
| situational_music | VanillaSituationalMusicOracle 4e (Hud) | B10-O04 |
| sound_engine | VanillaSoundEngineOracle 4e (LocalPlayer ctor) + VanillaSoundRig cascade | B10-O04 / Batch 0 G2+G9 |
| sound_event | VanillaSoundEventOracle 2e (getOrResolve + getAttenuationDistance) | B10-O04 |
| subtitle | ParityFont 1e + VanillaSubtitleOracle 1e (Util.setTimeSource) | B10-O02, B10-O04 |
| text_display_render | ParityFont 1e + VanillaTextDisplayRenderOracle 3e | B10-O02, B10-O03 |
| title_debug | VanillaTitleDebugOracle 4e (Gui.canInterruptScreen) | B10-O03 |
| world_border_render | VanillaWorldBorderRenderOracle 34e (renderpearl + Hud + DynamicGpuData) | B10-O05 |

Note: `dragon_death_roar`, `goat_voices` and `secure_chat` are
server-e2e drivers whose Java side is a packet client, not a vanilla
oracle — the 1.21.7 report grouped them as "no oracle", but the Batch 0
driver map shows they compile a helper (`VanillaProtocolClient` /
`SecureChatClient`) that currently fails, so they are counted blocked
here with that qualification (same treatment as 26.2).
`VanillaProtocolClient`'s 34 errors are mostly the `x()`/`z()` →
`getX()`/`getZ()` packet-accessor renames (26.1 keeps the `get-`
forms) plus `ClientboundUpdateTagsPacket.tags()` →
`getTags()` and the `ServerboundAcceptTeleportationPacket(int)`-only
ctor; `SecureChatClient`'s 8 are the same packet-shape drift
(`ClientboundLoginPacket.onlineMode()`, `ServerboundChatPacket`
taking a plain `MessageSignature`, `MessageSignature`/`Component`
not being `Optional`s).

### No oracle — server-e2e / protocol (NOT RUN, 3)

| Driver | What it needs |
|---|---|
| hostile_sound_categories | Live server + client (locator-bar net harness). |
| mob_local_sound | Live server + client (imports hostile_sound_categories). |
| ominous_warning_sound | Live server + client (locator-bar net harness). |

## 6. In-game probes specified (method 3, NOT RUN)

For the owner-initiated playtest only — D6 stays deferred unless he
starts one. Each names the exact scenario and the expected vanilla
behaviour; none is a pass until run.

- **P1 — Sound at all.** Any world, break a block / hurt a mob / open a
  chest: vanilla plays the event sound; the port is expected to play
  nothing if `sounds.json` is absent from the laptop's `.local`
  (B10-G02). Also check the laptop file directly (see §7).
- **P2 — Signs and beds (the 26.1 headline).** Place an oak sign
  (standing, wall, hanging) and a red bed: vanilla shows a wooden
  board with text and a bed frame/mattress; the port is expected to
  show floating text only and no bed at all (B10-G01, B10-G03).
  Check the bed ITEM in the inventory too — it should render
  correctly (head + foot halves), unlike the placed bed.
- **P3 — Spawner / trial spawner.** Find a dungeon spawner or trial
  chamber: vanilla shows the spinning mob inside the cage; the port
  is expected to show an empty cage (B10-G04).
- **P4 — Moving piston.** Build a piston pushing a block on a clock:
  vanilla shows the block sliding; the port is expected to show the
  block vanish mid-stroke (B10-G04).
- **P5 — Campfire / vault / brushable / shelf.** Campfire with food
  cooking, vault with its display item, suspicious sand while
  brushing, shelf with items: vanilla shows the item(s); the port
  is expected to show none (B10-G04).
- **P6 — Silent goat (withdrawal sanity check).** Summon a goat with
  `Silent:1b`: vanilla makes no ambient sound; the port is expected
  to match (B10-G06 withdrawn — code + unit test exist, driver
  unmeasured). Worth one explicit look since no driver run exists.

## 7. Follow-ups and questions for the coordinator

1. **`horizon-prepare sounds` for 26.1 (highest value, one step).**
   Fixes B10-G02 at runtime — but unlike 26.2 it unblocks NO audio
   driver on its own, because all 6 are oracle-blocked too (B10-O04).
   The sounds step and the sound-oracle repairs are both needed
   before any audio driver measures anything.
2. **Oracle repair order for Batch 10** (when repair resumes):
   `ParityFont` first (1 error, 8 drivers), then the `LocalPlayer`
   ctor group (Batch 0 G2 — rescues `ambient_sound`,
   `entity_bound_sound`, `sound_engine`, `fire_starter_sound` in
   this batch, and is one of `VanillaFirstPersonOracle`'s and
   `VanillaHudOracle`'s errors), then `VanillaSoundEventOracle`
   (2 errors — rescues `VanillaSoundRig` → `VanillaMusicOracle` by
   cascade, unblocking `sound_event`, `music`, `ambient_sound`,
   `entity_bound_sound`, `sound_engine`'s rig leg),
   `VanillaPackResourcesOracle` (1 error, cheapest 9-harness fix),
   `VanillaSubtitleOracle` (1 error, field assignment),
   `VanillaEnvironmentOracle` + `VanillaAudioAttributesOracle`
   (1 error each, same `ClockInstance` cause),
   `VanillaTitleDebugOracle` (4 errors, one missing method),
   `VanillaMenuOracle` (4 errors, mechanical),
   `VanillaDragonRenderOracle` (1 error) +
   `VanillaTextDisplayRenderOracle` (3 errors) +
   `VanillaModelBakingOracle` (3 errors, one missing method),
   then `VanillaBlockEntityRenderOracle` (9 errors — the
   `block_entity_render` driver is the highest-value single repair:
   it is the check that would catch B10-G01/G03/G04),
   `VanillaItemModelOracle` (10 errors) and
   `VanillaFirstPersonOracle` (11 errors), then the
   `VanillaEntitySoundPacketOracle` / `VanillaProtocolClient` /
   `SecureChatClient` packet-shape translations, and last the
   renderpearl/`Hud` backports (deepest — B10-O05).
3. **Goat-voices docstring** (B10-G06 note): the driver's "Silent
   ambient suppression is a separate gap" docstring is stale for
   26.1 (suppression is implemented + unit-tested) — driver-hygiene
   fix, ungate the two controls when the driver next runs.
4. **Structure/test-instance renderers** (B10-G04 note): vanilla's
   visible output for these two renderers is unverified — a driver
   run (or a javap read of the two renderer classes) should settle
   whether the port is missing anything visible.
5. **Boundary decisions** (to avoid double-counting): `test_sign_literal`
   (sign BE NBT via commands) counted as Batch 4/9, not Batch 10;
   `test_storage_menu_parity` and the recipe-book logic drivers
   (`recipe_book`, `recipe_book_variants`, `recipe_book_view`) counted
   as Batch 4/7; the dragon `/damage` gap in the dragon-roar driver
   docstring belongs to Batch 3/9; the silent-goat check (B10-G06)
   overlaps Batch 3's entity work. Recorded here with cross-refs
   (coordinator dedups): `VanillaBlockEntityAnimOracle` (Batch 4),
   `VanillaTextDisplayOracle` + cascade (Batch 3/4),
   `VanillaRecipeBookOracle` cascade (Batch 4/7), `ServerPackClient`
   (Batch 11? — needs assignment), `VanillaSpriteAnimationOracle` /
   `VanillaTranslucencyOracle` / `VanillaTabListOracle` /
   `VanillaPackResourcesOracle` (drivers outside this batch's 50 but
   squarely render/UI family — no other batch brief names them).
   `VanillaProtocolClient` / `SecureChatClient` are shared helpers
   recorded as Batch 0 residual, not B10 items.
6. **26.1-vs-26.2 lesson for the next version's run:** the
   sign/bed form FLIPPED between 26.1 and 26.2 (BE-rendered →
   block models), and the `Hud` split + `EntityTypes` move landed
   in the same window. A "26.2 withdrew it" verdict is not evidence
   for an older version — each version's jar decides, and 26.1's
   jar says 1.21.7-style for signs, beds, and the HUD.
7. **Method note for future batches:** the Batch 0 measured table is
   the only oracle authority — this batch verified every oracle
   claim against it plus its own javap, and found no case where the
   table was wrong. An import-level check would have missed the
   method-level breaks again (notably `VanillaSoundEventOracle`,
   `VanillaSubtitleOracle`, `VanillaChatScreenOracle`, and the
   `MaterialBaker`-is-an-interface cases).
