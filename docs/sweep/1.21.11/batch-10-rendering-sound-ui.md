# Batch 10 — Rendering, sound, UI (1.21.11)

**Version:** 1.21.11 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.11` ·
**Ledger slice:** `sweep-1.21.11-batches/b10/slice.json` · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 10 — models, block-entity rendering,
screens, sound wiring, options. Checks + triage only; no production code
touched, no driver run, no cargo test run, no git command run in the
worktree. Everything below is static analysis: `javap` on the named
1.21.11 jars, source comparison against `versions/1.21.11/crates/`,
checks of the prepared 1.21.11 data plus vanilla's own asset files, and
oracle status from a Batch 0-method measurement run by this batch (see
§1 — the Batch 0 table had not landed when this batch ran).

Date: 2026-11-25. Scope: `docs/PARITY-SWEEP.md` §4 Batch 10 — models,
block-entity rendering, screens, sound wiring, options — re-checked
against vanilla **1.21.11** (DataVersion 4671, Java 21, protocol 774,
data pack 94.1, resource pack 75.0). The 1.21.10 Batch 10 report is the
template; every 1.21.10 (PRIMARY) and 1.21.7 (SECONDARY) defect was
re-verified against 1.21.11's own jar/assets + the 1.21.11 Rust source,
not assumed to carry over. **Vanilla 1.21.11 is a different step from
1.21.10 in scope**: it HAS the `minecraft:environment_attribute`
registry (45 entries) and the `minecraft:game_rule` registry (59
entries), the `server.permissions` package (18 classes), the
`world.entity.vehicle.minecart` (20) and
`world.entity.monster.zombie` (15) subpackages, the
`world.entity.animal.{cow,equine}` / `world.entity.npc.villager` /
`world.entity.monster.illager` subpackages, the `SwingAnimation`
component + `SwingAnimationType`, `Player.getItemSwapScale` +
`LivingEntity.itemSwapTicker`, `TextRenderable$Styled`,
`MipmapStrategy`, `client/renderer/state` (18), `client/model/object`
(41) and `client/renderer/rendertype` (12) — and it still has NO
`world.clock` package, no `renderer.block.dispatch` package, no
`model.sprite`/`model.geometry`/`model.cuboid` packages, no
`renderpearl` GPU API, no `Hud` class, no `GuiGraphicsExtractor`, no
`MaterialBaker`, no `SwingState` / `FirstPersonHandsAndItems`, and no
`EntityTypes`/`BlockEntityTypes` holder move. Slot displays number 8.
**The port differs from the 1.21.10 tree just as much**: 1.21.11 IS the
E6 era template — the sign-board fix originated here (ac8dd9dc) — and
its oracle sources were adapted during the template port, so six
oracles that failed for 1.21.10 PASS here (see §3). The headline
finding of this batch is the **loading stick (B10-G16, NEW)**: 1.21.11
carries the identical pause-during-load deadlock the 1.21.7 fix
(775a25c2) closed — all three pause sites lack the `remote_loading()`
gate.

## Tally

**25 resolved behaviour checks: 13 PASS, 12 GAP** (all static; see §1
for what "resolved" means here). Behaviour/non-behaviour split:
**25 behaviour + 0 non-behaviour** — oracle-status checks are NOT
counted in the tally (26.1 convention — oracle triage has its own
counts in §3). The 25 are the 24 template checks (one verdict flip:
G05's check now passes for a different reason — vanilla 1.21.11 HAS
the swap ticker, so probe and oracle agreeing on the columns is a
pass, not a fix — and the Checkbox check is N/A for 1.21.11, see §4
note) plus one new check, the loading-stick check (§4 check 14, a
gap). Of the 1.21.10 batch's 18 items (PRIMARY), **14 reproduce**
(B10-G02, B10-G03, B10-G08–B10-G15, B10-O02–B10-O05 — §2/§3, with
1.21.11-specific evidence), **1 is fixed** (B10-G01), and **3 never
existed** (B10-G04, B10-G05, B10-G07 — §2, all land as closed items).
Of the 1.21.7 batch's 11 items (SECONDARY), **6 reproduce** (B10-G02,
B10-G03, B10-O02–B10-O05), **1 is fixed** (B10-G01), **2 never
existed** (B10-G04, B10-G05), and **2 stay withdrawn** (B10-G06,
B10-O01 — §4, report-only). Withdrawn numbers are NOT reused. **One
new behaviour item** is minted: B10-G16, the loading stick (§2).
Per-section counts reconcile: §2 has 11 gap items covering 12 gap
checks (B10-G03 covers two checks — placed beds and the special-model
decoder), §4 has 13 pass checks, 12 + 13 = 25 = slice checks.total.

The 50 in-scope drivers were **not run** — their status was triaged
from this batch's Batch 0-method measurement
(`sweep-1.21.11-batches/b10/evidence/measure/results.json`, fixpoint
big-compile) and this batch's driver map
(`b10/evidence/measure/drivers.json`; see §1):

| Driver state | Count | Drivers |
|---|---|---:|
| Runnable by javac (all oracles compile) — NOT RUN | 13 | cloud_render, container_title, cuboid_model, firework_tooltip, first_person, item_model, lore_tooltip, options, sound_event, stop_sound_wire, tooltip_display, world_options, zombie_cure_sound |
| Blocked on ≥1 unrepaired oracle | 34 | ambient_sound, arrow_hit_sound, audio_attributes, block_entity_render, blockstate_model, breaking_model, chat_screen, container_screen, dragon_death_roar, dragon_render, entity_bound_sound, entity_model, entity_sound_packet, environment_render, fire_starter_sound, goat_voices, hud, menu_click, mipmap, model_baking, model_choice, model_resolution, music, pack_screen, recipe_book_render, screen_layout, secure_chat, situational_music, sound_engine, subtitle, text_display_render, title_debug, weather_render, world_border_render |
| No oracle (server-e2e / protocol drivers) — NOT RUN | 3 | hostile_sound_categories, mob_local_sound, ominous_warning_sound |
| **Total in scope** | **50** | |

Four drivers blocked for 1.21.10 are runnable for 1.21.11
(first_person, item_model, sound_event, zombie_cure_sound — their
oracles pass against the 1.21.11 jar; §6). Unlike 1.21.10, one
runnable driver IS data-gated by B10-G02: `sound_event` is
oracle-clean but calls `require_sounds()`, so it cannot run until
`sounds.json` exists (§8 item 1). The other 5 `audio_harness`
drivers are oracle-blocked AND data-gated. The remaining runnable
drivers are unmeasured, not passing.

Per-driver oracle detail is in §6. Oracle defects found: 4 items
(B10-O02–B10-O05, §3), covering 39 failing in-scope oracles recorded
explicitly in §3 (1 + 18 + 9 + 11), plus 3 pure-cascade oracles
recorded with their families and 2 shared packet-client helpers
recorded in §6 as Batch 0 residual. Behaviour gaps found: 11 items
(B10-G02, B10-G03, B10-G08–B10-G16, §2). One fixed item (B10-G01)
and three closed items (B10-G04, B10-G05, B10-G07). No B10-H items:
every check was settled statically or is recorded as not-measured in
§6; the in-game probes are specified in §7.

## 1. Method and what "resolved" means

No driver runs and no cargo tests were possible (checks-only batch),
so this batch resolves a check only when static evidence settles it:

- **javap on the named 1.21.11 jars** — client jar
  `e6-deltas/versions/1.21.11/jars/client-named.jar` (10,291 classes,
  sha1 `29bddf144a2f2907528b25d616770bfea67c4792`, Mojang-mapped)
  for all rendering/sound/UI classes, and the named server-game jar
  `e6-deltas/versions/1.21.11/jars/server-game-named.jar` (7,163
  classes, sha1 `7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2`) for
  server-side classes (`LivingEntity`, `ServerGamePacketListenerImpl`,
  `ShearsDispenseItemBehavior`, species `shear` methods,
  `AbstractThrownPotion`). Vanilla 1.21.11 is the spec throughout;
  nothing was assumed from the 26.3 base, the 1.21.10 form, or the
  1.21.7 form. The bundler jar was never javap'd (no game classes).
  JDK 25 (`~/workspace/tools/jdk-25/bin/javap`). Classes needed
  repeatedly were extracted once to the batch evidence dir
  (`evidence/classes/`, `evidence/sclasses/`) and javap'd from there.
- **Vanilla asset files from the client jar** — the jar bundles
  `models/block/` entries and 1,505 `items/` entries, so block-model
  emptiness, item-model forms and special-renderer use are verifiable
  directly. `sounds.json` is NOT in the jar (it ships in the asset
  index); the 1.21.11 copy was fetched from Mojang's asset CDN via
  piston-meta (asset index 29, sha1-verified
  `68e5c7db2acb8fc85d17a95db8ec6a6faf55e5fc`, 530,166 bytes;
  `sounds.json` sha1 `123370fbb15bf1e85cf5038b3e8ca719ad391dfb`,
  581,030 bytes, 1,837 events — `evidence/sounds-1.21.11.json`).
- **The prepared 1.21.11 data** — no `horizon-prepare` run has
  produced client resources in this worktree: `versions/1.21.11/.local`
  holds only `measure-oracles/` (Batch 0, 15M) and
  `minecraft/1.21.11/` (220M: jars, libraries, manifest, prepared.json,
  remapped.json, reports, sources — a mappings/research dir, NOT
  prepared client resources; no `assets/` dir at all). A find over the
  whole `.local` returns no `sounds.json` anywhere. Batch 12 has not
  yet produced a prepare run (its evidence holds only a build log so
  far). The G02 evidence is therefore the prepare flow itself (the
  `sounds` subcommand / `--complete`, verified in-tree) plus the
  client's silent skip — see §2.
- **Datagen registries** —
  `26.1-adapters/versions/1.21.11/datagen/reports/registries.json`
  for registry counts (1,838 sound events, 104 data component types,
  24 decorated-pot patterns, 49 BE types incl. `minecraft:bed`,
  `provides_pottery_pattern` absent, `map_color` present,
  `swing_animation` present, `environment_attribute` 45,
  `game_rule` 59, slot displays 8, items 1,505).
- **Oracle status from this batch's own Batch 0-method measurement.**
  The Batch 0 table (`docs/sweep/1.21.11/batch-0-oracles.json`) had
  NOT landed when this batch ran (`sweep-1.21.11-batches/b00/` holds
  only H01 probe work), and the common.md fallback — provisional
  triage from the 1.21.10 table — would have been badly wrong: the
  oracle sources differ heavily between the trees (the 1.21.11
  template's oracles were adapted during the port — see §3), and the
  measurement proved it (32/67 in-scope oracles deviate from the
  1.21.10 table). So this batch ran a scoped re-measurement using the
  exact Batch 0 method (fixpoint big-compile per classpath,
  `sourcepath=False` on fixpoint rounds, jsr305 on the client
  classpath) against 1.21.11's own named jars + sha1-verified Mojang
  1.21.11 libraries (61 client + 39 server jars, fetched via
  piston-meta; server libs from the bundler's
  `META-INF/libraries.list`). Script:
  `b10/evidence/measure-oracles-b10.py` (adapted from the 1.21.10
  batch's script; the 1.21.11 tree carries no `tools/jsr305-3.0.2.jar`,
  so the measurement uses a copy kept in `b10/evidence/`, taken from
  the 1.21.10 tree). Results:
  `b10/evidence/measure/results.json` (server 102/391 pass, client
  127/391 pass, both 127/391 — for comparison 1.21.10 was 104/391 on
  the client classpath), logs `measure/big-compile-{server,client,
  both}.log`, driver map `measure/drivers.json` (648 test files
  mapped). Full per-oracle error detail (with symbols/locations) was
  extracted to `b10/evidence/oracle-errors-client.txt`; every oracle
  claim in §3/§6 is grounded in that measurement plus fresh javap,
  never in an import check. When the real Batch 0 table lands, the
  coordinator should diff it against this measurement (§8 item 6).
  Nothing was written to the worktree; all measurement outputs live
  in `b10/evidence/`.

"Resolved" means the static evidence above settles the check one way
or the other. Anything needing a driver run or a live game is recorded
as not-measured (§6) or specified as a probe (§7), never claimed as a
pass. Exact geometry fidelity (sign-board placement, beacon-beam
shape, geometry-function output) is unverified statically throughout —
the probes in §7 are the check for it.

## 2. Behaviour gaps

### B10-G01 — Sign boards ARE rendered (FIXED — the fix originated in this template)

The 1.21.7 gap is gone, and this is where it was fixed: the 1.21.11
port draws sign boards. The `Kind::Sign` arm in
`block_entity_render.rs:2808`
(`horizon-client-render/src/block_entity_render.rs`) draws the board
via `draw_part`, with a comment citing vanilla
`AbstractSignRenderer.submitSign`: the board draws under
`translateSign` with the model scale (2/3 for standing and wall signs,
1.0 for hanging ones), "and needs only the block state — it shows even
before the text data has loaded". Model keys `sign_standing` /
`sign_wall` / `hanging_sign_wall` / `hanging_sign_ceiling` /
`hanging_sign_ceiling_middle` (`:2822-2831`, selected by hanging /
wall / attached), registered in the MODELS table at `:1426-1434` to
the oak entity-model layers, with per-wood texture
`signs::board_texture` (`signs.rs:333`,
`entity/signs/{wood}`). Vanilla side confirmed fresh:
`oak_sign.json` is particle-only, and `AbstractSignRenderer` is
abstract over `getSignModel(BlockState, WoodType)` + `submitSign`
(`javap-signs-beds.txt`) — the port's model-key + texture approach
matches that shape. This is the 1.21.11 fix (ac8dd9dc) itself, carried
into 1.21.10 as the template's first child. Two caveats: exact
geometry fidelity is unverified statically (probe P2, §7), and the
`:1020` comment ("Sign boards and beds are block models in the terrain
mesh") is still stale for signs in the template — it describes
`CacheOptions`, but it reads as a claim that boards are not drawn.
Lands as a **fixed** item.

### B10-G02 — No `sounds.json` in the prepared 1.21.11 resources: the game is silent and 6 audio drivers cannot run (MAJOR)

REPRODUCES. No `horizon-prepare` run has produced client resources in
this worktree (see §1): `versions/1.21.11/.local` holds only Batch 0's
`measure-oracles/` and the mappings/research dir
`minecraft/1.21.11/` — no `horizon-client/` resources at all, and a
find over the whole `.local` returns **no `sounds.json` anywhere**.
The flow defect is verified in-tree: `horizon-prepare` has a separate
`sounds` subcommand (`crates/horizon-prepare/src/main.rs:24`, "copies
the pinned version's sounds (about 380 MB) from the official …") and
`--complete` copies sounds (`:55`, `:275-282`), but `--prepare` alone
does not (`:230` — "Preparing is most of a complete run; the sounds
and the check are the rest"). The client then fails silently:
`SoundRegistry::load` (`horizon-client-render/src/sound_events.rs:262`)
reads each namespace's `sounds.json` with `let Ok(text) =
fs::read(...) else { continue; }` (`:280-282`) — a missing file is
skipped without a word, and `is_empty()` (`:287-290`) documents the
state ("the resources hold no `sounds.json` (sounds not imported from
the player's launcher when the game was prepared)"). With an empty
registry the game plays no event sound at all. The 6 `audio_harness`
drivers (`ambient_sound`, `audio_attributes`, `music`,
`situational_music`, `sound_engine`, `sound_event`) all call
`require_sounds()` (`tools/audio_harness.py:35`, requiring
`RESOURCES / 'assets/minecraft/sounds.json'`) and are data-gated by
this item — and unlike 1.21.10, one of them (`sound_event`) is
oracle-clean now, so the sounds step alone would make it runnable
(§8 item 1); the other five are oracle-blocked too (B10-O04). The
wiring itself is fine (§4 check 1): the failure is the missing file,
not a mismatch.

### B10-G03 — Beds are invisible when placed, AND bed items fail to decode (MAJOR)

REPRODUCES, both halves, with the item half proven statically (the
fallback is the missing model).

**Placed beds.** Vanilla 1.21.11 renders beds through
`BedRenderer` (`createHeadLayer` / `createFootLayer` / `submitPiece`,
registered for `BlockEntityType.BED` — `javap-ber.txt`,
`ber-types.txt`); the block model is particle-only
(`models/block/bed.json`, all bed blockstates point at it), so there
is no terrain-mesh fallback. The port has no bed draw path at all: the
`Kind` enum (`block_entity_render.rs:323`) has no `Bed` variant and no
`Kind::Bed` arm exists anywhere; the only bed mention in the whole
block-entity-render file is the lowercase "beds" in the stale `:1020`
comment; `item_render/special.rs` has 0 bed mentions; `game/beds.rs` is
sleep mechanics only. A placed bed renders nothing.

**Bed items.** Vanilla 1.21.11's `items/red_bed.json` (and all 16 bed
colours — census, §4 check 13) is a SINGLE `minecraft:special` with
type `minecraft:bed` + texture — the 1.21.7 form, NOT 26.1.2's
composite-of-two (1.21.11 has no `composite` model type at all). The
port's `SpecialModel::decode` (`horizon-assets/src/item_model/
special.rs:82`) has NO bed arm: the ten arms are banner, conduit,
chest, copper_golem_statue, decorated_pot, head, player_head,
shulker_box, shield, trident (`:89-135`). The file's own header doc
(`:1-8`) states it plainly: "Bed, standing-sign and hanging-sign types
exist in 1.21.11's registry but have no kind here yet: a model naming
one fails to decode and the item falls back to the missing model (a
known gap, not a 1.21.11 behaviour)". The decode error propagates via
`?` (`item_model.rs:489-491`, `"minecraft:special" =>
SpecialModel::decode(...)?` — the `is_foreign` arm at `:482` does not
apply: bed is in 1.21.11's own registry); `load_definitions`
(`horizon-client/src/item_render.rs:691`) logs a warning and SKIPS the
item; `append_layers` renders a missing definition via
`state.missing(out)` (`item_model.rs:1110`). So all 16 bed items render
as the missing model — proven statically. (Standing-sign and
hanging-sign specials are also undecodable, but no bundled 1.21.11
item definition uses them — census §4 check 13 — so bed items are the
only observable gap. Bell, book and end_cube are correctly absent:
1.21.11's special registry holds 13 types and does not include them.)

### B10-G04 — Five vanilla block-entity renderers have no port equivalent (NEVER EXISTED — closed)

The 1.21.7 item (five renderers) and the 26.1.2 item (seven) are both
wrong for 1.21.11, for the same reason the 1.21.10 run found: the
earlier runs searched only the `Kind` enum and missed the `game.rs`
geometry path. All five 1.21.7 members — and the two extra 26.1.2
members — have geometry-function equivalents with live call sites in
the 1.21.11 tree, each citing its vanilla renderer in a doc comment:

| Vanilla renderer | Port equivalent | Call site |
|---|---|---|
| SpawnerRenderer + TrialSpawnerRenderer | `spawner_geometry` (`game/spawner.rs:508`) | `game/creatures.rs:8066` |
| CampfireRenderer | `campfire_geometry` (`game/campfire.rs:131`) | `game.rs:6776` |
| PistonRenderer (moving piston) | `moving_piston_geometry` (`game/moving_pistons.rs:10`) | `game.rs:7260` |
| BrushableBlockRenderer | `brushable_geometry` (`game/brush.rs:313`) | `game.rs:6777` |
| VaultRenderer | `vault_geometry` (`game/vault.rs:223`) | `game.rs:6773` |
| ShelfRenderer | `shelf_geometry` (`game/shelf.rs:200`) | `game.rs:6778` |
| OminousItemSpawnerRenderer | `ominous_item_spawner_geometry` (`game/ominous_item_spawner.rs:205`) | `game.rs:6780` |
| StructureBlockRenderer | `bounding_box_gizmos` (`game/structure_blocks.rs:61`, modelling `BlockEntityWithBoundingBoxRenderer`: game-master gate, 96-block view distance, render modes) | `debug_keys.rs:985` via `debug_gizmos`, called every frame at `game.rs:9495` |
| TestInstanceBlockRenderer | `test_instance_marker_geometry` (`game/test_instances.rs:274`) | `game.rs:6812` |

Vanilla 1.21.11 registers 27 BE renderers (`javap-ber-static.txt`,
`ber-types.txt`: BANNER, BEACON, BED, BELL, BRUSHABLE_BLOCK, CAMPFIRE,
CHEST, CONDUIT, COPPER_GOLEM_STATUE, DECORATED_POT, ENCHANTING_TABLE,
ENDER_CHEST, END_GATEWAY, END_PORTAL, HANGING_SIGN, LECTERN,
MOB_SPAWNER, PISTON, SHELF, SHULKER_BOX, SIGN, SKULL, STRUCTURE_BLOCK,
TEST_INSTANCE_BLOCK, TRAPPED_CHEST, TRIAL_SPAWNER, VAULT — the same 27
as 1.21.10). The 27-slot recount for 1.21.11: **26 fully covered, 1
uncovered (bed, G03)** — Kind arms cover 13 kinds + sign text counts
2 slots (standing + hanging) and chest covers 3 slots (chest, trapped
chest, ender chest — `block_entity_render.rs:2936`, `:2942-2949`),
beacon beams are check 5, and the nine geometry functions above cover
the rest. Exact geometry fidelity is unverified statically (probes
P3/P4/P5, §7). The 1.21.7-record correction from the 1.21.10 run
stands: the 1.21.7 tree ALSO has all five functions with live call
sites, so the 1.21.7 run's B10-G04 was wrong for the 1.21.7 tree too.
Lands as a **closed** item.

### B10-G05 — First-person probe emits swap-ticker columns the repaired oracle drops (NEVER EXISTED for 1.21.11 — closed)

The 1.21.7/1.21.10 item's premise inverts for 1.21.11: vanilla 1.21.11
HAS the swap ticker. `LivingEntity` carries the protected field
`itemSwapTicker` (`javap-livingentity-server.txt:129`), and `Player`
has `getItemSwapScale(float)` (`javap-player-client.txt:244`) —
neither exists in 1.21.10. Both sides emit the columns and agree: the
probe (`crates/horizon-client/examples/probe/first_person_probe.rs`,
472 lines) emits `strength.swap_ticker` and `swap_scale` at `:379-380`,
and the oracle (`VanillaFirstPersonOracle.java:421`) emits
`swapTicker.getInt(player)` + `player.getItemSwapScale(1.0F)` (field
looked up at `:268`, incremented at `:398`). The oracle PASSES against
the 1.21.11 jar (see §3 — the template's oracle was rewritten for the
classic swing model and keeps the swap columns, which are valid here).
There is no format mismatch to fix on either side: the probe, the
oracle and vanilla 1.21.11 all carry the swap ticker. (Whether the
probe's swap-scale VALUES match vanilla's `getItemSwapScale` needs the
driver run — and the driver is runnable now, §6.) Lands as a
**closed** item. Note the three-way split this creates: the item was a
gap for 1.21.7 (probe emitted, oracle dropped), fixed for 1.21.10
(probe dropped, matching ticker-less vanilla), and never existed for
1.21.11 (vanilla has the ticker, both sides emit it) — all three
verdicts are consistent because the vanilla spec differs per version.

### B10-G07 — Baby zombie holding an item lowers its arms instead of keeping them down (NEVER EXISTED — closed)

The 26.1.2 gap's premise does not exist in 1.21.11. Vanilla 1.21.11's
`AnimationUtils.animateZombieArms` is the 4-arg generic form
`(ModelPart, ModelPart, boolean, T extends UndeadRenderState)` — it
reads `swingAnimationType` (STAB check) and `attackTime` from the
render state, with NO baby/item check (0 `isBaby` / `getMainHandItem`
occurrences in the method, `javap-animationutils-c.txt`), and
`AbstractZombieModel.setupAnim` calls it unconditionally, gated only
on `isAggressive` (`javap-zombiemodel.txt`). The baby guard is a 26.1
addition — it postdates 1.21.11. The port matches vanilla 1.21.11
exactly: `mobs/anim.rs:634-636` ("1.21.11 has no baby guard, so a baby
holding an item raises its arms like an adult (26.1 returned early
for a baby with a main-hand item)") plus the unit test
`a_baby_zombie_holding_an_item_raises_its_arms_like_an_adult`
(`:1668`). A baby zombie holding an item raising its arms IS vanilla
1.21.11 behaviour. Lands as a **closed** item.

### B10-G08 — First-person swing uses the 26.3 `SwingState` state machine vanilla 1.21.11 does not have (MINOR)

REPRODUCES, with a new nuance: vanilla 1.21.11 DOES have per-item swing
animation now, but still no state machine. Vanilla 1.21.11's swing is
plain fields: `swinging` / `swingTime` / `swingingArm` / `attackAnim` /
`oAttackAnim` (`javap-livingentity-server.txt:105-129`); there is no
`SwingState` class and no `FirstPersonHandsAndItems` class anywhere in
the jar (both named in the port's own doc comment,
`first_person.rs:1-2`). `getCurrentSwingDuration()` is private and
reads the held item's swing animation —
`getItemInHand(MAIN_HAND).getSwingAnimation().duration()` — adjusted
by haste (`duration-(1+amp)`) / fatigue (`duration+(1+amp)*2`)
(`:4884-4921`); `swing(hand[, boolean])` restarts only when `!swinging
|| swingTime >= duration/2 || swingTime < 0` (`:4930-4959`);
`updateSwingTime()` re-reads the duration each tick and resets at
duration, so `attackAnim` never reaches 1.0 (`:5456-5490`). The port
carries the 26.3 form: `SwingDescription` / `SwingState`
(`horizon-client/src/first_person.rs:490-499`); `start_if_able`
allows a restart at ticks==0 where vanilla blocks it (`:507-522`);
`tick()` holds the animation at 1.0 an extra tick (clears only when
`ticks > duration`, `:528-541`); the duration is cached at swing start
where vanilla re-reads effects each tick. (`swing_duration`, `:564`,
matches vanilla's formula for the per-item animation — the formula is
right, the state machine around it is not. The 1.21.10 difference —
vanilla 1.21.10 had NO per-item swing animation at all — does not
apply here: 1.21.11 gained the `swing_animation` component, registry
104, and the port's formula matches it.)

### B10-G09 — Decorated-pot sherd patterns never resolve: every side renders blank (MINOR)

REPRODUCES. Vanilla 1.21.11 resolves sherd patterns via
`DecoratedPotPatterns.getPatternFromItem(Item)`
(`javap-potpatterns.txt` — a static item→pattern map; no
`provides_pottery_pattern` anywhere in the class). The port reads the
26.3 component instead: `components["minecraft:provides_pottery_pattern"]`
(`block_entity_render.rs:1540`, in the sherd-item scan at `:1528-1552`).
That component is ABSENT from 1.21.11's 104-component registry, and
there is no `data/minecraft/decorated_pot_pattern/` directory in the
datagen output — so `pot_patterns` stays empty and every sherd side
falls back to blank (`:2469`, `:3193`). The registry lists 24 patterns,
unreachable through the component path. (The port's
`DATA_COMPONENT_TYPES` list itself is exact — 104/104, zero drift
either way, verified against the registry — so this is a lookup-path
defect, not a list defect; the 26.1.2 latent-drift note does not apply
here.)

### B10-G10 — `filled_map` fails item decode: the `map_color` tint source is gone (MINOR)

REPRODUCES. Vanilla `items/filled_map.json` tints with
`minecraft:constant` (-1) + `minecraft:map_color` (default 4603950).
The port's `TintSource` (`horizon-assets/src/item_model.rs:551-559`)
has NO Map variant; the decode fallthrough returns
`Err("unknown item tint source")` (`:588`), propagating via `?` so the
definition is skipped at load and the item renders as the missing model
(same load path as bed items, G03). The census (§4 check 13) shows
`map_color` appears in exactly one bundled item definition
(`filled_map.json`), so the blast radius is one item — but it is the
map every player holds. (`minecraft:map_color` IS in the port's
`DATA_COMPONENT_TYPES` list, `item_model.rs:86` — the component is
known; only the tint source is missing.)

### B10-G11 — Item-use swings never reach other players: `swing_hand` no longer sends `ServerboundSwingPacket` (MINOR)

REPRODUCES. Vanilla `LocalPlayer.swing` calls
`AbstractClientPlayer.swing(hand)` then UNCONDITIONALLY sends
`ServerboundSwingPacket(hand)` (`javap-localplayer.txt:672-682`); the
server's `handleAnimate` resets the last-action time + calls
`player.swing(hand)` (`javap-sgpli.txt:4764-4791`). The port's
`swing_hand` (`game/hand.rs:210-232`) sends NOTHING — it only records
an Animate packet if the session is recording. The only three
`S::Swing` sends left in the tree are attack paths
(`game/remote.rs:9936`, `:10056`, `:10063`). Item-use swings (placing
blocks, using items) are invisible to other players; attack swings
still show. (Vanilla 1.21.11 has no `SwingSource` class at all — the
1.21.10 report's three-valued `SwingSource` note does not carry here;
the port's swing-source modelling was not re-verified beyond the
packet gap.)

### B10-G12 — 3003 wax-on sound missing from the level-event path: other players' waxing is silent (MINOR)

REPRODUCES. Vanilla `LevelEventHandler` case 3003 plays the sound
itself: WAX_ON particles (UniformInt 3-5) + `playLocalSound(pos,
HONEYCOMB_WAX_ON, BLOCKS, 1.0, 1.0)`
(`javap-leveleventhandler.txt` offsets 3369-3399). `HoneycombItem.useOn`
plays NO sound directly — 3 `levelEvent(3003)` sites, 0 `playSound`
(`javap-honeycomb.txt`). The port splits the two halves the 26.3 way:
`block_particle_event` (`game/level_particles.rs:35-38`) plays
particles only for 3003, and the sound comes from item code —
`wax_on_sound` (`game.rs:8032`, callers `:8343`/`:8362`) — which only
runs for the local player's own waxing. Another player's (or a
dispenser's, or a copper golem's) waxing arrives as a bare 3003 event
and is silent. (The other wax call sites exist in the 1.21.11 tree and
all play the sound directly from their own code — dispenser wax
`game/dispenser.rs:2206-2235`, copper-golem wax
`game/copper_golems.rs:506`, local honeycomb prediction
`game/remote.rs:8801`/:8809 — so the gap is the event path, not the
call sites: a 3003 arriving from a vanilla server has no sound
attached to it.)

### B10-G13 — 2002/2007 potion-break sound missing from the event path: potion breaks are silent on a 1.21.11-form server (MINOR)

REPRODUCES. Vanilla's server sends ONE event:
`AbstractThrownPotion.onHit` → `levelEvent(instant? 2007 : 2002, pos,
color)`; `ThrownExperienceBottle.onHit` → `levelEvent(2002, pos,
0xFF385DC6)`. The client 2002/2007 case plays `SPLASH_POTION_BREAK`
itself on NEUTRAL, pitch `nextFloat()*0.1+0.9`, after the particles
(`javap-leveleventhandler.txt` offsets 1525-1553).
`LevelEventHandler` is an 82-case lookupswitch with NO 1053/1054/
2019/2020 cases (case list extracted). The port maps the sound to
1053/1054 (`game.rs:4489`, pitch `:4508`) and its own server sends
them (`game/potions.rs:1935` for the experience bottle, `:1967` for
thrown potions, alongside the vanilla 2002/2007 at `:1933`/`:1965`) —
a closed loop that plays against its own server, but a vanilla-server
client (which sends only 2002/2007) hears nothing: the client's
2002/2007 arms are particles-only (`game.rs:4355-4362`,
`potion_splash_particles`).

### B10-G14 — Digging hit sound plays at START, then never again on a 1.21.11-form server (MINOR)

REPRODUCES. Vanilla `MultiPlayerGameMode.startDestroyBlock` plays
NOTHING (0 "Sound" occurrences in the method); `continueDestroyBlock`
plays ONLY the hit sound on `destroyTicks % 4.0 == 0`
(`javap-mpgm.txt` offsets 198-208) as
`SimpleSoundInstance(hitSound, BLOCKS, (volume+1)/8, pitch*0.5,
unseededRandom, pos)`; `addBreakingBlockEffect` has 0 occurrences in
the class. The port inverts this: `remote_mining`'s start branch calls
`breaking_block_effects(hit.pos, hit.face, true)`
(`game/remote.rs:8171`, comment: "startDestroyBlock plays the first
hit and crack itself; the server's level events 2019 and 2020 do the
rest"), and the continue branch plays NO hit sound locally
(`remote.rs:8180-8265` — its only `audio.play` is the block BREAK
sound at completion, `:8254`). The port's own server sends 2019 every
tick / 2020 every 4th (`game/block_breaking.rs:97-101`,
`destroy_progress_event`; client handles 2019/2020 at
`game.rs:4298-4306`) — again a closed loop: against a vanilla 1.21.11
server, which sends neither event (they do not exist in vanilla's
`LevelEventHandler`), the port plays one sound at the first hit and
then goes silent for the rest of the dig.

### B10-G15 — Sound categories come from the event NAME, not the call site: shear and wax-on sounds land on the wrong volume slider (MINOR)

REPRODUCES. Vanilla passes `SoundSource` as a parameter:
`Shearable.shear(level, source, stack)` takes BLOCKS from the
dispenser (`ShearsDispenseItemBehavior`, `javap-shear.txt:77`/`:147`)
and PLAYERS from player shearing (sheep `:294`, mushroom cow `:877`,
snow golem `:1649`, bogged `:1930` — `javap-shear-species.txt`);
species `shear` plays via `ServerLevel.playSound(null, this, SOUND,
source, 1.0, 1.0)` (sheep `SHEEP_SHEAR`, mushroom cow
`MOOSHROOM_SHEAR`, snow golem `SNOW_GOLEM_SHEAR`, bogged
`BOGGED_SHEAR`, copper golem `COPPER_GOLEM_SHEAR` — all five
javap-verified); 3003 plays wax-on on BLOCKS. The port's
`Audio::play` derives the source from the event name via
`settings::sound_source` (`horizon-client-render/src/settings.rs:18-93`,
a prefix heuristic with a HOSTILE list of 34): dispenser shear
`entity.sheep.shear` → NEUTRAL, `entity.bogged.shear` → HOSTILE,
wax-on `item.` → PLAYER — all three should be BLOCKS. The dispenser
shear sites use `audio.play`, not `audio.play_on`
(`game/dispenser.rs:1941/1975/1989/2003/2018`). The plumbing exists —
`Audio::play_on` has many game-code callers in 1.21.11
(`game.rs:4445` portal/AMBIENT, `:6147` bubble pop/PLAYERS,
`game/creatures.rs:5153` projectiles/NEUTRAL,
`ominous_item_spawner.rs:117`, `player_equip.rs:83`, `raids.rs:713`,
`remote.rs:3880/4623/4762`, `game/commands/world_cmds.rs:635`
/playsound) — the shear/wax sites just don't use it. (Correction carried from 26.1.2:
`Audio::play_on` is NOT "used only by /playsound".)

### B10-G16 — NEW: Pause during terrain load deadlocks the game — the 1.21.7 loading stick, unpropagated (MAJOR)

NEW item (mandatory check, §0 brief). 1.21.11 carries the identical
code to the 1.21.7 loading-stick defect (fixed in 1.21.7 by 775a25c2,
merged 8123e438): a pause landing after the Login packet and before
the load tracker finishes deadlocks three ways — a paused server sends
no chunks, the frozen tick never finishes the load tracker, and the
loading screen hides the pause menu, so there is no way back. The
1.21.11 tree HAS `remote_loading()` (`game/remote.rs:10906`,
"Connected or connecting with no player yet", level-load active with
`Reason::Other` or no player yet) and uses it as a gate elsewhere
(mouse input `game.rs:10768`, native hooks, screen backgrounds,
studio) — but NONE of the three pause sites is gated:

| Site | 1.21.11 (ungated) | 1.21.7 fixed (775a25c2) |
|---|---|---|
| Escape → pause | `play_keys.rs:78-88`: `Screen::Playing => { self.pause_focus = None; self.screen = Screen::Paused; ... }` — no gate | `play_keys.rs:91`: `if self.remote_loading() { return Ok(()); }` with the `LevelLoadingScreen.shouldCloseOnEsc is false` comment |
| `pauses_game()` | `pointer_capture.rs:47-52`: no `remote_loading` term | `pointer_capture.rs:55`: `&& !self.remote_loading()` |
| Focus-loss pause | `pause_if_inactive`, `pointer_capture.rs:72-95`: no `remote_loading` term | `pointer_capture.rs:84`: `&& !self.remote_loading()` |

Vanilla side confirmed fresh for 1.21.11:
`LevelLoadingScreen.shouldCloseOnEsc()` returns false
(`javap-levelloading-c.txt`: `iconst_0; ireturn`) — Escape does
nothing while terrain is loading in vanilla, so the 1.21.7 fix's
vanilla basis applies to 1.21.11 too. The deadlock itself was NOT
reproduced headless on the VM (as in 1.21.7, the laptop is where it
reproduces directly — probe P12, §7); the code identity is exact —
the three 1.21.11 sites match the pre-fix 1.21.7 sites line for line
except for the missing gates. Propagation is a separate owner
decision — this batch only CHECKS. Lands as a **gap** item, major:
the game hangs with no way back when it triggers.

## 3. Oracle defects

All counts are from this batch's Batch 0-method measurement
(`b10/evidence/measure/results.json`), client classpath unless noted;
full per-oracle error detail (with symbols/locations) is in
`b10/evidence/oracle-errors-client.txt`. "First error" is the first
error javac reports for the file, not necessarily the root cause —
the cause column names the root causes, verified by javap where cheap.

**The headline of this section is the template's oracle repairs.**
The 1.21.11 tree's oracle sources were adapted during the template
port, and six oracles that failed for 1.21.10 PASS here:
`VanillaFirstPersonOracle` (rewritten for the classic swing model —
no `SwingState`/`FirstPersonHandsAndItems`, `getCurrentSwingDuration`
looked up reflectively, swap-ticker columns kept because vanilla
1.21.11 has them), `VanillaItemModelOracle` (rewritten to the 1.21.11
model packages — `block.model.BakedQuad`/`BlockModel`/
`ItemModelGenerator`, no `model.sprite`/`geometry`/`cuboid`),
`VanillaPackResourcesOracle` (`PackResources.forEach` replaced by a
direct add), `VanillaSoundEventOracle` (1-arg `SoundManager` ctor,
no-arg `getAttenuationDistance()`), `VanillaZombieCureSoundOracle`
(identical source — passes because the jar now has the
`monster.zombie` package), and `VanillaEntityParticlesOracle`
(identical source — passes because the jar now has the animal
subpackages; Batch 3 cross-ref). 32/67 in-scope oracles deviate from
the 1.21.10 table — provisional triage from that table would have
been wrong for nearly half the scope.

### B10-O02 — `ParityFont` needs `VanillaPackResources.asResourceManager()`, absent in 1.21.11 (MINOR)

REPRODUCES, reduced to one error (1.21.10: 2). `ParityFont` fails
with **1 error** on the client classpath:

| Line | Error | Cause |
|---|---|---|
| `:43` | `cannot find symbol: method asResourceManager()` | 1.21.11's `VanillaPackResources` has `asProvider()` but NO `asResourceManager()` (`javap-packresources.txt`) — same as 1.21.10 |

The 1.21.10 second error is GONE: 1.21.11's `TextRenderable` HAS the
nested `Styled` type (`TextRenderable$Styled.class` in the jar). All
other `ParityFont` imports exist in 1.21.11 (`GlyphSource`,
`BakedGlyph`, `GlyphProviderDefinition` + `Conditional`,
`ClientPackSource`; `Font` at `net/minecraft/client/gui/Font`). It
blocks 7 drivers (`block_entity_render`, `chat_screen`, `hud`,
`pack_screen`, `recipe_book_render`, `subtitle`,
`text_display_render` — plus its own font driver, outside the 50),
so it stays the highest-leverage single repair (§8 item 2).

### B10-O03 — Render/model oracle family does not compile for 1.21.11 (MAJOR)

REPRODUCES, better than 1.21.10: **18 failing oracles** (1.21.10: 21).
Three oracles that FAILED in 1.21.10 now PASS (see the section
headline): `VanillaFirstPersonOracle`, `VanillaItemModelOracle`,
`VanillaPackResourcesOracle`. No oracle that passed in 1.21.10 newly
fails here.

| Oracle | Errors | Cause (root, javap-verified where cheap) |
|---|---|---|
| VanillaBlockEntityRenderOracle | 30 (1.21.10: 42) | Missing packages `client.renderer.state.level`, `client.resources.model.sprite` (×2), `world.clock`; `UvMapping`, `SpriteGetter`, `SpriteId`, `ItemStackTemplate`, `ChestRenderState.facing`, `CameraRenderState` absent; `CaseLevel.fuelValues()` abstract gap; `Font.prepare8xTextOutline` signature; `SignText` ctor (ArrayList vs Component[]); `StandingSignRenderState` / `HangingSignRenderState`, `PlainSignBlock` / `HangingSignBlock`, `SignRenderState.transformations`, `HangingSignRenderer.TRANSFORMATIONS` all absent |
| VanillaBreakingModelOracle | 11 (same) | `renderer.block.dispatch` + `model.sprite` packages missing; `MaterialBaker`, `BlockStateModelPart`, `QuadInstance`, `BlockModelFeatureRenderer`, `BlockStateModel` absent |
| VanillaSectionMeshOracle | 45 (1.21.10: 48) | `model.geometry` / `model.cuboid` / `model.sprite` / `renderer.block.dispatch` packages missing; `MaterialBaker`, `BlockStateModel`, `FluidStateModelSet`, `BlockAndTintGetter`, `CardinalLighting`, `BlockModelLighter`, `FluidRenderer`, `BlockQuadOutput` absent; `ModelBaker.sprites()` / `parts()` abstract gaps; `ModelBlockRenderer` ctor arity; `BlockStateBase.emissiveRendering()` arity, `BlockState.isLightPermeable`, `BlockColors.getTintSources` absent |
| VanillaModelBakingOracle | 19 (1.21.10: 20) | `model.sprite` / `model.cuboid` / `model.geometry` / `renderer.block.dispatch` packages missing; `ModelBaker.Interner`, `CuboidRotation`, `FaceBakery`, `UnbakedCuboidGeometry`, `CuboidFace`, `Variant`, `BakedQuad`, `CuboidModel` / `MissingCuboidModel` absent; `Material` ctor arity |
| VanillaDragonRenderOracle | 1 (1.21.10: 2) | One no-override (`:31`) |
| VanillaEntityAnimOracle | 5 (1.21.10: 8) | `ArmedEntityRenderState.swingAnimation` / `currentSwing`, `LivingEntity.SwingDescription` absent; `PoseStack.rotateDegrees` absent; one no-override |
| VanillaTextDisplayRenderOracle | 6 (1.21.10: 8) | `ServerLevel` ctor arity (11 args required, 10 passed); `GameRenderState` absent; `Font.prepareBackground` absent; two no-overrides |
| VanillaEnvironmentOracle | 5 (1.21.10: 18) | The environment-attribute cause is GONE (1.21.11 HAS the registry — `EnvironmentAttributeSystem`, `EnvironmentAttribute`, `Biome.getAttributes`, `BuiltInRegistries.ENVIRONMENT_ATTRIBUTE` all exist now); remaining: `world.clock` package missing; `RegistryLayer.WORLD`, `RegistryDataLoader.WORLD_REGISTRIES`, `ClockManager`, `ClockInstance` absent |
| VanillaTitleDebugOracle | 4 (same) | `Gui.canInterruptScreen()` absent (×4) |
| VanillaBlockEntityAnimOracle | 7 (1.21.10: 21) | `world.clock` package missing; `EntityTypes` absent (×2); `Level.fuelValues()` abstract gap; `State.forAnimation` absent; two no-overrides |
| VanillaTextDisplayOracle | 2 (same) | `ServerLevel` ctor arity (`:19`) + one no-override (`:20`) |
| VanillaBlockStateModelOracle | 13 (same) | `renderer.block.dispatch` + `dispatch.multipart` packages missing; `BlockStateModel`, `SingleVariant`, `WeightedVariants`, `BlockStateModelDispatcher`, `Variant`, `Condition`, `CombinedCondition`, `KeyValueCondition` absent |
| VanillaModelChoiceOracle | 11 (same) | `model.sprite` + `renderer.block.dispatch` packages missing; `BlockStateModel` (×5), `BlockStateModelPart`, `WeightedVariants`, `Material` absent |
| VanillaModelResolutionOracle | 8 (same) | `model.geometry` / `model.sprite` / `model.cuboid` packages missing; `Material.sprite()` / `forceTranslucent()` absent; `CuboidModel` / `MissingCuboidModel`, `UnbakedCuboidGeometry` absent |
| VanillaWeatherRenderOracle | 1 (same) | `RandomSource.createThreadLocalInstance(long)` absent (`:57`) |
| VanillaMipmapOracle | 1 (1.21.10: 5) | `MipmapStrategy` EXISTS in 1.21.11 now; only `NativeImage.computeTransparency()` absent (`:49`) |
| VanillaSpriteAnimationOracle | 2 (1.21.10: 5) | `renderpearl.api.buffers` package missing; `GpuBufferSlice` absent. Driver outside this batch's 50 — recorded here as render-family |
| VanillaTranslucencyOracle | 4 (same) | `renderpearl.api.pipeline` package missing; `MeshData.decodeQuadCentroids` absent; `IndexType` absent; `SortState.writeSortedIndexBuffer` absent. Driver outside this batch's 50 — recorded here as render-family |

Passing in this family: `VanillaFirstPersonOracle`,
`VanillaItemModelOracle`, `VanillaPackResourcesOracle`,
`VanillaCuboidModelOracle`, `VanillaCloudRenderOracle`,
`VanillaEntityModelExporter`, `VanillaStackModelFlagsOracle`, and
`VanillaTextDisplayTransformOracle` (0 own errors — a pure cascade
via `VanillaTextDisplayOracle` in the fixpoint table, same as
1.21.10). Cross-batch: `VanillaBlockEntityAnimOracle` (Batch 4) and
`VanillaTextDisplayOracle` (Batch 3/4) are recorded here, cross-ref'd
in §8 item 5.

### B10-O04 — Sound oracle family does not compile for 1.21.11 (MAJOR)

REPRODUCES, better than 1.21.10: **9 failing oracles** (1.21.10: 11 —
8 with own errors + `VanillaMusicOracle` as a pure cascade).
`VanillaSoundEventOracle` and `VanillaZombieCureSoundOracle` now PASS
(see the section headline).

| Oracle | Errors | Cause |
|---|---|---|
| VanillaSoundRig | 2 (1.21.10: 3) | `DeviceList` absent (`:129`); `SoundManager.getChannelDebugString()` absent (`:202`). (`SoundInstance.getIdentifier()` exists in 1.21.11 now.) |
| VanillaMusicOracle | cascade (1.21.10: 5 own errors) | 0 own errors — pure cascade via `VanillaSoundRig`. (`getSituationalMusic()` and `SoundManager.refreshCategoryVolume` exist in 1.21.11 now.) |
| VanillaSituationalMusicOracle | 6 (1.21.10: 20) | Environment-attribute cause gone (registry exists); remaining: `RegistryLayer.WORLD` / `RegistryDataLoader.WORLD_REGISTRIES` absent; `Hud` absent (×3); one `:11` |
| VanillaAmbientOracle | 1 (1.21.10: 15) | Only the `LocalPlayer` ctor arity remains (`:50`: 7 args required, 9 passed). `AmbientSounds`, `AmbientMoodSettings`, `BiomeAmbientSoundsHandler` ctor, `EnvironmentAttributeSystem` all exist now |
| VanillaSoundEngineOracle | 4 (1.21.10: 12) | `LocalPlayer` ctor arity (`:33`); `UnderLiquidSubSound` (×2) + `UnderLiquidAmbientSoundInstance` absent. (The minecart package exists in 1.21.11 now — that cause is gone.) |
| VanillaFireStarterSoundOracle | 2 (same) | `ServerLevel` ctor arity (`:43`) + `LocalPlayer` ctor arity (`:61`) |
| VanillaAudioAttributesOracle | 5 (1.21.10: 23) | Environment-attribute cause gone; remaining: `world.clock` package missing; `RegistryLayer.WORLD` / `WORLD_REGISTRIES`; `ClockManager` / `ClockInstance` |
| VanillaSubtitleOracle | 10 (1.21.10: 12) | `GuiGraphicsExtractor` absent; six no-overrides; `Util.setTimeSource` absent; `SubtitleOverlay.extractRenderState` absent. (`SoundManager` ctor arity is gone — 1-arg ctor exists.) |
| VanillaEntitySoundPacketOracle | 3 (1.21.10: 11) | `MinecraftServer` ctor arity (`:62`); two no-overrides. (Minecart + `server.permissions` packages exist in 1.21.11 now — those causes are gone.) |

Passing in this family: `VanillaSoundEventOracle`,
`VanillaZombieCureSoundOracle`, `VanillaStopSoundWireOracle`.

### B10-O05 — UI/screen oracle family needs `renderpearl` + `Hud`, 26.3-only APIs (MAJOR)

REPRODUCES: **11 failing oracles** (same count as 1.21.10, fewer
errors almost everywhere).

| Oracle | Errors | Cause |
|---|---|---|
| VanillaHudOracle | 77 (1.21.10: 100) | `renderpearl.api.pipeline` package missing; `Hud` absent (×20+); `GuiGraphicsExtractor` absent (×15+); `RenderPipeline` absent; `LocalPlayer` ctor arity; `ExperienceBar`, `JumpableVehicleBar` absent; `BossHealthOverlay.extractRenderState`, `ChatComponent.addClientSystemMessage` / `addPlayerMessage` / `extractRenderState`, `ChatComponent.DisplayMode`, `GuiMessageTag`, `world.scores.TeamColor` absent; `model.sprite.Material` + `renderer.block.dispatch` packages missing; `BlockStateModelSet` absent |
| VanillaPackScreenOracle | 39 (1.21.10: 62) | `renderpearl.api.pipeline` package missing; `GuiGraphicsExtractor`, `RenderPipeline`, `TextInputManager` absent; `Gui` ctor arity (requires `Minecraft`); `Language.DEFAULT_INSTANCE` absent; `Util.setTimeSource` absent; `Screen.extractRenderState` absent; ~25 no-overrides |
| VanillaRecipeBookScreenOracle | 31 (1.21.10: 35) | `renderpearl.api.pipeline` package missing; `GuiGraphicsExtractor`, `RenderPipeline`, `TextInputManager` absent; `Capture.blit` arity; `RecipeBookPage` / `GhostSlots` / `EditBox` / `RecipeBookTabButton` / `CycleButton` `extractRenderState` absent; `ItemStackTemplate` absent; ~17 no-overrides |
| VanillaLoomScreenOracle | 27 (1.21.10: 29) | `renderpearl.api.pipeline` + `model.sprite` packages missing; `GuiGraphicsExtractor`, `RenderPipeline`, `SpriteId`, `Hud` absent; `LoomScreen` / `CrafterScreen` `extractBackground` absent; ~13 no-overrides |
| VanillaWorldBorderRenderOracle | 42 (1.21.10: 48) | `renderpearl.api.buffers` / `.commands` / `.device` / `.pipeline` packages missing; `DynamicGpuData`, `CommandEncoder`, `GpuDevice`, `GpuBuffer`, `RenderPass`, `WorldBorderRenderState`, `Hud` absent; `renderer.state.level` package missing; `Util.setTimeSource` absent |
| VanillaTabListOracle | 45 (same) | `com.mojang.authlib.services.*` (8 imports, `:8-15` — absent from authlib 7.0.61, which 1.21.11 pins, same as 1.21.10) + `renderpearl.api.pipeline` + `Hud` + `TeamColor` + `PlayerTabOverlay.extractRenderState` |
| VanillaChatScreenOracle | 23 (1.21.10: 41) | The permissions cause is GONE (`server.permissions` exists in 1.21.11); remaining: `ChatComponent.DisplayMode` absent; `CommandSourceStack.getAvailablePostEffects` absent; `RegistryLayer.WORLD`, `RegistryDataLoader.WORLD_REGISTRIES`, `BuiltInRegistries.DATA_COMPONENT_INITIALIZERS`, `DataComponentInitializers` absent; `TimerQueue` no-arg ctor absent; `world.clock.ServerClockManager` package missing; `CharacterEvent` ctor arity; `Hud` absent; `ChatAbilities` absent; `Capture.fill` arity; `Renderable` / `CommandSuggestions` `extractRenderState` absent |
| VanillaMenuOracle | 8 (1.21.10: 10) | `util.Prediction` absent (`:138`); `DedicatedServer` ctor arity; `ServerLevel` ctor arity; `world.entity.EntityTypes` absent; `ContainerInput` absent (×2); one no-override. (The `world.level.gamerules` package exists in 1.21.11 now — that cause is gone.) |
| ServerPackClient | 1 (same) | One no-override (`:264`). Driver outside this batch's 50 — recorded here, cross-ref Batch 11 (§8 item 5) |
| VanillaContainerScreenOracle | 5 (same) | `ContainerInput` absent (×5: Chest, Hopper, Shulker, Beacon + base) |
| VanillaSimpleContainerOracle | 1 (same) | `Reference<Item>.bindComponents` absent (`:25`) |

Passing in this family: `VanillaContainerTitleOracle`,
`VanillaPackSelectionOracle`, `VanillaRecipeBookViewOracle`,
`VanillaStorageMenuOracle`, `VanillaFloatTextOracle`,
`VanillaFireworkTooltipOracle`, `VanillaLoreTooltipOracle`,
`VanillaTooltipDisplayOracle`, `VanillaWorldOptionsOracle` — and,
as in 1.21.10, `VanillaRecipeBookOracle` and
`VanillaPlaceRecipeOracle` have 0 own errors (pure cascades via
`VanillaMenuOracle` in the fixpoint table; their Batch 4/7 drivers'
shared `VanillaMenuOracle` leg is the broken one).

### Withdrawn oracle item

**B10-O01 (options) — WITHDRAWN, stays.** `VanillaOptionsOracle`
PASSES on the client + both classpaths in the measured table. The
1.21.7 break does not apply: `KeyMapping.getCategory()` returns
`KeyMapping$Category` in 1.21.11 (`javap-keymapping.txt`), and
`Category` is a record with `id()` returning `Identifier`
(`javap-keymapping-category.txt`) — so the oracle's
`k.getCategory().id().getPath()` call (`VanillaOptionsOracle.java:55`)
compiles. Report-only, as in 1.21.10.

### Passing in-scope oracles (auditable coverage)

23 in-scope oracles pass on the client classpath:
`VanillaOptionsOracle`, `ParityData`, `GoatVoiceTranscript`,
`VanillaFirstPersonOracle`, `VanillaItemModelOracle`,
`VanillaPackResourcesOracle`, `VanillaCuboidModelOracle`,
`VanillaCloudRenderOracle`, `VanillaEntityModelExporter`,
`VanillaStackModelFlagsOracle`, `VanillaSoundEventOracle`,
`VanillaZombieCureSoundOracle`, `VanillaStopSoundWireOracle`,
`VanillaContainerTitleOracle`, `VanillaPackSelectionOracle`,
`VanillaRecipeBookViewOracle`, `VanillaStorageMenuOracle`,
`VanillaFloatTextOracle`, `VanillaFireworkTooltipOracle`,
`VanillaLoreTooltipOracle`, `VanillaTooltipDisplayOracle`,
`VanillaWorldOptionsOracle`, `VanillaEntityParticlesOracle` (Batch 3
cross-ref — 0 errors; it FAILED with 18 errors in 1.21.10) — plus
`VanillaTextDisplayTransformOracle`, `VanillaRecipeBookOracle` and
`VanillaPlaceRecipeOracle` (0 own errors each, pure cascades) and
`VanillaCommandParseOracle` (passes; used by the `chat_screen`
driver). Shared packet-client helpers `VanillaProtocolClient` (39e)
and `SecureChatClient` (8e) fail and are recorded in §6 as Batch 0
residual, not B10 items.

## 4. Passing checks (13)

1. **Sound-wiring diff (registry ↔ sounds.json) — PASS.** Vanilla 1.21.11
   `sounds.json` (asset index 29, sha1-verified) defines 1,837
   events; `registries.json` lists 1,838 sound events. The only
   registry event without a `sounds.json` entry is
   `minecraft:intentionally_empty` — vanilla's intentional sentinel
   (same as 1.21.7, 1.21.10, 26.2, 26.1, 26.1.1, 26.1.2). There are ZERO
   `sounds.json` orphans (1.21.10 had none either). 1,740 entries carry
   subtitles; 7 entries have empty sound lists (the same 7
   vanilla-intentional empties as every other run: `block.fungus.fall`/
   `hit`, `entity.cod`/`salmon`/`snow_golem`/`tropical_fish` ambient,
   `music.nether.warped_forest`). The wiring spec is fully
   established; the port-side failure is the missing file (B10-G02),
   not a wiring mismatch.
2. **First-person swap ticker — PASS (B10-G05 NEVER EXISTED).** Vanilla
   1.21.11 HAS the swap ticker: `LivingEntity.itemSwapTicker` field
   and `Player.getItemSwapScale(float)`. The Rust probe emits the
   swap-ticker columns (`first_person_probe.rs:379-380`) and the
   oracle emits them too (`VanillaFirstPersonOracle.java:421`) — and
   the oracle PASSES against the 1.21.11 jar. Probe, oracle and
   vanilla agree; the 1.21.7 format mismatch does not exist here at
   all. (Swap-scale VALUES still need the driver run — and the driver
   is runnable now, §6.)
3. **Silent-goat ambient suppression — PASS (B10-G06 WITHDRAWN, stays).**
   Vanilla 1.21.11: `Entity.playSound` checks `isSilent()` first
   (`javap -c`: `isSilent()` then `ifne` skip, both overloads,
   `javap-entity.txt:3517ff`) and `Mob.playAmbientSound` routes
   through it (`makeSound` → `playSound`), so a `Silent:1b` goat
   makes no ambient sound. The 1.21.11 port implements exactly this:
   `Mob::voice` (`horizon-client/src/mobs.rs:4675`) draws the pitch
   first ("Java evaluates the arguments … before the call") then
   returns early `if self.silent` (`:4680-4682`), `play_ambient`
   routes through `voice()` (`mobs/ai.rs:1484`, `m.voice(sound, events)`
   at `:1581`), and a unit test pins
   it (`mobs.rs:6520`, `a_silent_mob_makes_no_ambient_sound`). The
   `test_goat_voices_parity.py:4` docstring ("Silent ambient
   suppression is a separate gap") is STALE for 1.21.11 — the driver
   still gates those two controls out, so the suppression is
   unmeasured by the driver, but the code is there. Driver-hygiene
   follow-up (§8), not a behaviour gap.
4. **Sign items — PASS (no divergence).** Vanilla renders sign items
   as flat generated sprites (`items/oak_sign.json` →
   `minecraft:model` → `item/oak_sign` → `item/generated`; same for
   hanging signs), same as the port. (1.21.11's jar DOES contain
   `StandingSignSpecialRenderer` / `HangingSignSpecialRenderer` — but
   no bundled item definition uses them (census, check 13), so
   nothing diverges. The port's missing `standing_sign` /
   `hanging_sign` decoder arms are unobservable for bundled items —
   see G03.)
5. **Beacon beams — PASS (present).** `beacon_beam`
   (`block_entity_render.rs:3561`, modelling vanilla
   `BeaconRenderer.submitBeaconBeam`) and the `gbuffers_beaconbeam`
   packing (`gfx/pack_entities.rs:501`, `:820`) both exist; the beacon
   base block model has elements, so the block + beam both have draw
   paths. Exact beam geometry/colour is unverified statically (no
   beacon driver in scope).
6. **BE renderer Kind arms present — PASS.** Chest
   (single/left/right, trapped/ender — `block_entity_render.rs:2936`,
   `:2942-2949`), shulker, banner, skull, bell, decorated pot,
   conduit, enchanting table, lectern, copper golem statue, end
   portal, end gateway, sign — all have a `Kind` arm
   (`block_entity_render.rs:323-371`), 13 kinds covering 16 of the 27
   slots counting sign as 2 (standing + hanging) and chest as 3
   (chest, trapped, ender), plus beacon beams (check 5). The slots
   outside the Kind enum are covered by geometry functions (check 11)
   except beds (G03).
7. **Options API surface — PASS (B10-O01 WITHDRAWN, stays).**
   Everything the options oracle touches exists in 1.21.11:
   `Options(Minecraft, File)` 2-arg ctor, `options.keyMappings`,
   `KeyMapping.getName()` / `getDefaultKey()`, `ToggleKeyMapping`,
   and — the 1.21.7 break that does not apply here —
   `KeyMapping.getCategory()` returns `Category` with `id()`
   (javap-verified; in 1.21.7 it returns a plain `String`).
   `VanillaOptionsOracle` PASSES on the client + both classpaths in
   the measured table. (Option VALUES/defaults per key were not diffed
   — that needs the options driver run once it runs.)
8. **Dispenser shear presentation — PASS (verified fresh, NOT a
   defect).** The SOUND / ANIMATION side, which is B10's, matches
   vanilla throughout (all javap `-c`, fresh, 1.21.11 server jar):
   beehive — `block.beehive.shear` at the block centre, 1.0/1.0,
   `minecraft:shear` game event at the target pos, honeycomb drops,
   bees released, honey reset (`dispenser.rs:1792-1824`); entity
   shear — per-species sound at the mob's feet, 1.0/1.0
   (`entity.sheep.shear`, `entity.bogged.shear`,
   `entity.snow_golem.shear`, `entity.copper_golem.shear`,
   `entity.mooshroom.shear` — vanilla plays each via
   `playSound(null, this, SOUND, source, 1.0, 1.0)`; all five
   `*_SHEAR` events javap-verified in the species classes — note the
   mooshroom class is `MushroomCow` in 1.21.11's mappings),
   `minecraft:shear` game event at the target block pos
   (`dispenser.rs:1905-2060`); leash shear — `item.shears.snip` at
   the entity's block centre, 1.0/1.0, plus the SHEAR game event
   (vanilla `Entity.shearOffAllLeashConnections` plays `SHEARS_SNIP`
   at `blockPosition()` and posts SHEAR via
   `dropAllLeashConnections`); mooshroom puff — one EXPLOSION
   particle at mid-height (`dispenser.rs:2022`,
   `mooshroom_shear_puff`; vanilla `lambda$shear$0`:
   `sendParticles(EXPLOSION, getX(), getY(0.5), getZ(), 1, 0,0,0,0)`);
   dispenser click/fail — CLICK on success, FAIL otherwise
   (`dispenser.rs:1776-1781`; vanilla
   `OptionalDispenseItemBehavior.playSound`). Order is beehive-first,
   then entity, as in vanilla (`dispenser.rs:1776`). The one deviation
   in the area is the sound CATEGORY (BLOCKS in vanilla, NEUTRAL/HOSTILE
   by the port's name heuristic) — minted separately as B10-G15, not
   here.
9. **`layer_prefixes` (Batch 3 flag) — PASS (no finding).** The
   function (`mobs/species.rs:1424`) returns model layer families
   (adult, baby, variants) — pig/cow/chicken cold variants, horse
   armor, happy-ghast harness/ropes, and the rest — matching the
   1.21.11 entity-model layers. No B10 defect; the Batch 3 species
   diff is a dimensions-table matter (Batch 3's item), not a render
   change.
10. **Sign boards — PASS (B10-G01 FIXED).** The placed-sign board
    draws via the `Kind::Sign` arm with vanilla scales and model keys
    (full evidence in §2, B10-G01). Exact geometry fidelity is
    unverified statically (probe P2, §7).
11. **BE renderers beyond Kind — PASS (B10-G04 NEVER EXISTED).** All
    seven renderers the 26.1.2 item claimed missing — and the five
    the 1.21.7 item claimed — have port equivalents with live call
    sites (full evidence in §2, B10-G04). The 27-slot recount: 26
    fully covered, 1 bed slot uncovered (G03). Exact geometry
    fidelity is unverified statically (probes P3/P4/P5, §7).
12. **Baby zombie arms — PASS (B10-G07 NEVER EXISTED).** Vanilla
    1.21.11 has no baby guard and the port matches it (full evidence
    in §2, B10-G07).
13. **Item-definition census — PASS (NEW check; spec established).**
    All 1,505 bundled item definitions extracted from the client jar
    (`evidence/jar-assets/assets/minecraft/items/`) and counted:
    model types `minecraft:model` 1,358, `minecraft:special` 67,
    `minecraft:select` 71, `minecraft:condition` 7,
    `minecraft:range_dispatch` 2 — there is NO `composite` type in
    1.21.11 (a 26.x form; this is why 1.21.11's bed items are single
    specials, G03). Special types: shulker_box 17, banner 16, bed 16,
    chest 9, head 6, conduit 1, decorated_pot 1, player_head 1 —
    standing_sign and hanging_sign appear in ZERO bundled definitions.
    Tint sources: dye 50, constant 12, grass 6, potion 4, map_color 1
    (`filled_map.json` only), firework 1. The census pins the exact
    blast radius of the two decode gaps (G03: 16 bed items; G10: 1
    filled map) and confirms no other bundled definition hits an
    undecodable type. The port's `DATA_COMPONENT_TYPES` list is
    exactly 1.21.11's 104 components (verified 104/104 against the
    registry, zero drift either way).

**Not a check — Checkbox row-limit tooltip (26.1.2 §4 check 10) is N/A
for 1.21.11.** Vanilla 1.21.11's `Checkbox` has NO `ROWS` /
`overflowsRowLimit` / `setMaxRows` (grep of `javap-checkbox.txt` +
`javap-checkbox-builder.txt` empty) — the row-limit tooltip was a
26.1.2 change. The 1.21.11 tree carries no
`checkbox_overflows_row_limit` helper (grep of the whole client
source: the only `max_rows` is an unrelated tooltip truncation in
`ui_theme.rs:634`). There is nothing to port and nothing to diverge;
the check is not counted.

## 5. Re-check verdicts

### 1.21.10 re-checks (PRIMARY — all 18 decided)

| 1.21.10 item | Verdict for 1.21.11 | 1.21.11 item |
|---|---|---|
| B10-G01 sign boards (major/fixed) | FIXED (stays) — the fix originated in this template (§2) | B10-G01 (fixed) |
| B10-G02 sounds.json (major/gap) | REPRODUCES | B10-G02 (gap); no prepare has run in this worktree, so the evidence is the flow + silent skip, not a completed prepare |
| B10-G03 beds placed + items (major/gap) | REPRODUCES | B10-G03 (gap) |
| B10-G04 five BE renderers (minor/closed) | NEVER EXISTED (stays closed) | B10-G04 (closed) |
| B10-G05 swap-ticker columns (minor/fixed) | NEVER EXISTED — vanilla 1.21.11 HAS the ticker; probe + oracle both emit the columns (§2) | B10-G05 (closed) |
| B10-G07 baby zombie arms (minor/closed) | NEVER EXISTED (stays closed) | B10-G07 (closed) |
| B10-G08 SwingState (minor/gap) | REPRODUCES | B10-G08 (gap); 1.21.11 differs from 1.21.10 — vanilla HAS per-item swing animation here, and the port's formula matches it; only the state machine is wrong |
| B10-G09 pot patterns (minor/gap) | REPRODUCES | B10-G09 (gap) |
| B10-G10 filled map (minor/gap) | REPRODUCES | B10-G10 (gap); component list is exact here (104/104, no latent drift) |
| B10-G11 swing packet (minor/gap) | REPRODUCES | B10-G11 (gap) |
| B10-G12 wax-on sound (minor/gap) | REPRODUCES | B10-G12 (gap) |
| B10-G13 potion-break sound (minor/gap) | REPRODUCES | B10-G13 (gap) |
| B10-G14 digging sound (minor/gap) | REPRODUCES | B10-G14 (gap) |
| B10-G15 sound categories (minor/gap) | REPRODUCES | B10-G15 (gap) |
| B10-O02 ParityFont (minor/oracle-defect) | REPRODUCES | B10-O02 (oracle-defect) — 1 error now (TextRenderable.Styled exists in 1.21.11) |
| B10-O03 render/model oracles (major/oracle-defect) | REPRODUCES | B10-O03 (oracle-defect) — 18 failing (was 21); FirstPerson/ItemModel/PackResources now pass |
| B10-O04 sound oracles (major/oracle-defect) | REPRODUCES | B10-O04 (oracle-defect) — 9 failing (was 11); SoundEvent/ZombieCure now pass; Music is a pure cascade now |
| B10-O05 UI/screen oracles (major/oracle-defect) | REPRODUCES | B10-O05 (oracle-defect) — 11 failing, same count, fewer errors (permissions + gamerules causes gone) |

14 REPRODUCES, 1 FIXED, 3 NEVER EXISTED, 0 WITHDRAWN, 0 UNVERIFIED.

### 1.21.7 re-checks (SECONDARY — all 11 decided)

| 1.21.7 item | Verdict for 1.21.11 | 1.21.11 item |
|---|---|---|
| B10-G01 sign boards (major/gap) | FIXED — the port draws boards (§2) | B10-G01 (fixed) |
| B10-G02 sounds.json (major/gap) | REPRODUCES | B10-G02 (gap) |
| B10-G03 beds placed + items (major/gap) | REPRODUCES | B10-G03 (gap) |
| B10-G04 five BE renderers (minor/gap) | NEVER EXISTED — all five have port equivalents (§2); also wrong for the 1.21.7 tree itself (1.21.10 correction stands) | B10-G04 (closed) |
| B10-G05 swap-ticker columns (minor/gap) | NEVER EXISTED — vanilla 1.21.11 has the ticker; probe + oracle agree (§2) | B10-G05 (closed) |
| B10-G06 silent-goat suppression (minor/gap) | WITHDRAWN (stays) | Report-only (§4 check 3) |
| B10-O01 options (minor/oracle-defect) | WITHDRAWN (stays) | Report-only (§3, §4 check 7) |
| B10-O02 ParityFont (major in 1.21.7) | REPRODUCES | B10-O02 (oracle-defect) — 1 error now |
| B10-O03 render/model (major) | REPRODUCES | B10-O03 (oracle-defect) — 18 failing oracles |
| B10-O04 sound (major) | REPRODUCES | B10-O04 (oracle-defect) — 9 failing oracles |
| B10-O05 UI/screen (major) | REPRODUCES | B10-O05 (oracle-defect) — 11 failing oracles |

6 REPRODUCES, 1 FIXED, 2 NEVER EXISTED, 2 WITHDRAWN, 0 UNVERIFIED.

## 6. Per-driver status (all 50, measured)

Oracle status from this batch's Batch 0-method measurement
(`b10/evidence/measure/results.json`); "fail" = fails on the
classpath the driver's path uses. `cp` is that classpath
(oracle_harness → both, client jar → client, server jar → server,
no-flag → both superset). Driver→oracle map from
`b10/evidence/measure/drivers.json` (648 test files mapped from the
1.21.11 tree). Full per-driver detail in
`b10/evidence/driver-table.txt`.

### Runnable by javac — NOT RUN (13)

| Driver | Oracle(s) | Note |
|---|---|---|
| cloud_render | VanillaCloudRenderOracle pass | Unmeasured |
| container_title | ParityData + VanillaContainerTitleOracle pass | Unmeasured |
| cuboid_model | VanillaCuboidModelOracle pass | Unmeasured |
| firework_tooltip | VanillaFireworkTooltipOracle pass | Unmeasured |
| first_person | ParityData + VanillaFirstPersonOracle pass | Unmeasured — NEW runnable vs 1.21.10 (oracle rewritten for the classic swing model) |
| item_model | VanillaItemModelOracle pass | Unmeasured — NEW runnable vs 1.21.10 (oracle rewritten to the 1.21.11 model packages) |
| lore_tooltip | VanillaLoreTooltipOracle pass | Unmeasured |
| options | VanillaOptionsOracle pass | Unmeasured |
| sound_event | VanillaSoundEventOracle pass | Unmeasured — NEW runnable vs 1.21.10; DATA-GATED by B10-G02 (`require_sounds()`) |
| stop_sound_wire | VanillaStopSoundWireOracle pass | Unmeasured |
| tooltip_display | VanillaTooltipDisplayOracle pass | Unmeasured |
| world_options | VanillaWorldOptionsOracle pass | Unmeasured |
| zombie_cure_sound | VanillaZombieCureSoundOracle pass | Unmeasured — NEW runnable vs 1.21.10 (jar has the `monster.zombie` package now) |

Four drivers blocked for 1.21.10 are runnable for 1.21.11
(first_person, item_model, sound_event, zombie_cure_sound — above).
Unlike 1.21.10, ONE runnable driver is data-gated by B10-G02:
`sound_event` is oracle-clean but calls `require_sounds()`, so the
sounds step alone would make it runnable (§8 item 1). The other 5
`audio_harness` drivers are oracle-blocked AND data-gated.

### Oracle-blocked (34)

| Driver | Blocking oracle(s) | Item |
|---|---|---|
| ambient_sound | VanillaAmbientOracle 1e (LocalPlayer ctor) + VanillaSoundRig 2e (own errors) | B10-O04 |
| arrow_hit_sound | VanillaEntitySoundPacketOracle 3e (MinecraftServer ctor) | B10-O04 |
| audio_attributes | VanillaAudioAttributesOracle 5e (world.clock + RegistryLayer.WORLD) | B10-O04 |
| block_entity_render | ParityFont 1e + VanillaBlockEntityRenderOracle 30e (model packages + UvMapping + sign states) + VanillaTextDisplayRenderOracle 6e | B10-O02, B10-O03 |
| blockstate_model | VanillaBlockStateModelOracle 13e (dispatch package; CuboidModel passes) | B10-O03 |
| breaking_model | VanillaBreakingModelOracle 11e (BlockModelFeatureRenderer) + VanillaSectionMeshOracle 45e (MaterialBaker + model packages) | B10-O03 |
| chat_screen | ParityFont 1e + VanillaChatScreenOracle 23e (Hud + RegistryLayer.WORLD) + VanillaPackScreenOracle 39e (renderpearl) | B10-O02, B10-O05 |
| container_screen | VanillaContainerScreenOracle 5e (ContainerInput) | B10-O05 |
| dragon_death_roar | VanillaProtocolClient 39e (packet-client helper) | Batch 0 residual |
| dragon_render | VanillaDragonRenderOracle 1e (no-override) | B10-O03 |
| entity_bound_sound | VanillaSoundEngineOracle 4e (LocalPlayer ctor + UnderLiquid sounds) + VanillaSoundRig 2e | B10-O04 |
| entity_model | VanillaEntityAnimOracle 5e (swing fields + rotateDegrees) | B10-O03 |
| entity_sound_packet | VanillaEntitySoundPacketOracle 3e (MinecraftServer ctor) | B10-O04 |
| environment_render | VanillaEnvironmentOracle 5e (world.clock + RegistryLayer.WORLD) | B10-O03 |
| fire_starter_sound | VanillaFireStarterSoundOracle 2e (ServerLevel + LocalPlayer ctors) | B10-O04 |
| goat_voices | VanillaProtocolClient 39e (packet-client helper) | Batch 0 residual |
| hud | ParityFont 1e + VanillaHudOracle 77e (renderpearl + Hud + ExperienceBar) | B10-O02, B10-O05 |
| menu_click | VanillaMenuOracle 8e (Prediction + DedicatedServer ctor + EntityTypes + ContainerInput) | B10-O05 |
| mipmap | VanillaMipmapOracle 1e (NativeImage.computeTransparency) | B10-O03 |
| model_baking | VanillaModelBakingOracle 19e (model packages + FaceBakery; CuboidModel passes) | B10-O03 |
| model_choice | VanillaModelChoiceOracle 11e (model.sprite + BlockStateModel; CuboidModel passes) | B10-O03 |
| model_resolution | VanillaModelResolutionOracle 8e (model packages; CuboidModel passes) | B10-O03 |
| music | VanillaMusicOracle cascade (via VanillaSoundRig) + VanillaSoundRig 2e (own errors) | B10-O04 |
| pack_screen | ParityFont 1e + VanillaPackScreenOracle 39e (renderpearl + Gui ctor + Language) | B10-O02, B10-O05 |
| recipe_book_render | ParityFont 1e + VanillaRecipeBookScreenOracle 31e (renderpearl) | B10-O02, B10-O05 |
| screen_layout | VanillaLoomScreenOracle 27e (renderpearl + Hud) | B10-O05 |
| secure_chat | SecureChatClient 8e (packet-client helper) | Batch 0 residual |
| situational_music | VanillaSituationalMusicOracle 6e (RegistryLayer.WORLD + Hud) | B10-O04 |
| sound_engine | VanillaSoundEngineOracle 4e (LocalPlayer ctor) + VanillaSoundRig 2e | B10-O04 |
| subtitle | ParityFont 1e + VanillaSubtitleOracle 10e (GuiGraphicsExtractor) | B10-O02, B10-O04 |
| text_display_render | ParityFont 1e + VanillaTextDisplayRenderOracle 6e (ServerLevel ctor + GameRenderState) | B10-O02, B10-O03 |
| title_debug | VanillaTitleDebugOracle 4e (Gui.canInterruptScreen) | B10-O03 |
| weather_render | VanillaWeatherRenderOracle 1e (RandomSource.createThreadLocalInstance) | B10-O03 |
| world_border_render | VanillaWorldBorderRenderOracle 42e (renderpearl + Hud + DynamicGpuData) | B10-O05 |

Note: `dragon_death_roar`, `goat_voices` and `secure_chat` are
server-e2e drivers whose Java side is a packet client, not a vanilla
oracle — the 1.21.7 report grouped them as "no oracle", but the
driver map shows they compile a helper (`VanillaProtocolClient` /
`SecureChatClient`) that currently fails, so they are counted blocked
here with that qualification (same treatment as 26.2/26.1/26.1.1/
26.1.2/1.21.10). `VanillaProtocolClient`'s 39 errors are mostly the
`x()`/`z()` → `getX()`/`getZ()` packet-accessor renames (1.21.11 keeps
the `get-` forms) plus `ClientboundUpdateTagsPacket.tags()` →
`getTags()`, `ClientboundSetTimePacket.clockUpdates()` absent,
`MessageSignature`/`Component` not being `Optional`s, the
`ServerboundInteractPacket` / `ServerboundAcceptTeleportationPacket`
ctor shapes, `EntityTypes`, `ContainerInput`, and the advancements
packet accessors; `SecureChatClient`'s 8 are the same packet-shape
drift (`ClientboundLoginPacket.onlineMode()`,
`ServerboundChatPacket` taking a plain `MessageSignature`,
`MessageSignature`/`Component` not being `Optional`s).

### No oracle — server-e2e / protocol (NOT RUN, 3)

| Driver | What it needs |
|---|---|
| hostile_sound_categories | Live server + client (locator-bar net harness). |
| mob_local_sound | Live server + client (imports hostile_sound_categories). |
| ominous_warning_sound | Live server + client (locator-bar net harness). |

## 7. In-game probes specified (method 3, NOT RUN)

For the owner-initiated playtest only — D6 stays deferred unless he
starts one. Each names the exact scenario and the expected vanilla
behaviour; none is a pass until run. P2 is reframed vs 26.1.2: signs
SHOULD render now (confirmation), beds still should not. P8 is
reframed too: the missing-model fallback is proven statically for
1.21.11, so the probe confirms it on screen rather than discovering
it. P12 is NEW: the loading stick.

- **P1 — Sound at all.** Any world, break a block / hurt a mob / open a
  chest: vanilla plays the event sound; the port is expected to play
  nothing if `sounds.json` is absent from the laptop's `.local`
  (B10-G02). Also check the laptop file directly (see §8).
- **P2 — Signs (confirmation) and beds (expected absence).** Place an
  oak sign (standing, wall, hanging) and a red bed: vanilla shows a
  wooden board with text and a bed frame/mattress; the port SHOULD
  show the board too (B10-G01 fixed — confirm the board geometry,
  scale and per-wood texture match vanilla) but is expected to show
  no bed at all (B10-G03). Check the bed ITEM in the inventory too —
  it is expected to show the missing model (proven statically, G03).
- **P3 — Spawner / trial spawner (confirmation).** Find a dungeon
  spawner or trial chamber: vanilla shows the spinning mob inside
  the cage; the port SHOULD show it too (`spawner_geometry`, §2
  B10-G04) — confirm the mob is present, spinning, and scaled like
  vanilla's, and that the trial spawner's ominous item appears.
- **P4 — Moving piston (confirmation).** Build a piston pushing a block
  on a clock: vanilla shows the block sliding; the port SHOULD show
  it too (`moving_piston_geometry`) — confirm no vanish mid-stroke
  and no z-fighting on the finished bridge.
- **P5 — Campfire / vault / brushable / shelf (confirmation).**
  Campfire with food cooking, vault with its display item,
  suspicious sand while brushing, shelf with items: vanilla shows
  the item(s); the port SHOULD show them too (the four geometry
  functions, §2 B10-G04) — confirm placement, rotation and the
  brushable reveal progress match vanilla.
- **P6 — Silent goat (withdrawal sanity check).** Summon a goat with
  `Silent:1b`: vanilla makes no ambient sound; the port is expected
  to match (B10-G06 withdrawn — code + unit test exist, driver
  unmeasured). Worth one explicit look since no driver run exists.
- **P7 — Digging sound (LAN guest view).** Join a LAN world as a
  guest and dig a stone block for 10+ ticks: vanilla plays the hit
  sound every 4th tick from the first continue tick; the port is
  expected to play one sound at the first hit, then sounds only from
  the host's 2020 events (B10-G14). Against a vanilla 1.21.11 host,
  the port is expected to go silent after the first hit.
- **P8 — Undecodable items' on-screen fallback (confirmation).** Hold
  a red bed and a filled map: vanilla renders both; the port's
  `ClientItem` decode fails for both (B10-G03, B10-G10) and the code
  path ends at the missing model (`item_model.rs:1110`) — confirm
  the missing model is what actually shows (not an empty slot or a
  crash).
- **P9 — Wax-on and potion-break sound (LAN guest view).** Have
  another player wax a copper block and throw a splash potion while
  you watch: vanilla plays `item.honeycomb.wax_on` and
  `entity.splash_potion.break` for you; the port is expected to play
  neither from the event (B10-G12, B10-G13).
- **P10 — Decorated pot patterns.** Place a decorated pot with
  sherds: vanilla shows the sherd patterns on the sides; the port is
  expected to show blank sides (B10-G09).
- **P11 — Item-use swing (multiplayer view).** Watch another player
  place blocks / use items: vanilla shows their arm swinging; the
  port is expected to show no swing (B10-G11). Attack swings should
  still show.
- **P12 — Loading stick (NEW).** Join a world (single-player or LAN)
  and press Escape — or alt-tab away — while terrain is still
  loading, after the Login packet lands and before the load tracker
  finishes: vanilla does nothing on Escape
  (`LevelLoadingScreen.shouldCloseOnEsc` is false) and does not
  pause on focus loss; the port is expected to deadlock (B10-G16) —
  the pause menu hidden behind the loading screen, the paused server
  sending no chunks, the load tracker never finishing, no way back
  except killing the game. (The 1.21.7 fix's repro harness in
  `~/workspace/mcir-work/fresh-tester-repro/` is the model for this
  probe; do NOT run it against 1.21.11 unasked — it is a
  destructive-probe candidate, not a routine check.)

## 8. Follow-ups and questions for the coordinator

1. **`horizon-prepare sounds` for 1.21.11 (highest value, one step).**
   Unlike 1.21.10, it unblocks a driver on its own: `sound_event` is
   oracle-clean now and only data-gated by B10-G02. The other five
   audio drivers still need their oracles repaired too (B10-O04).
   Note no prepare has run at all in this worktree yet — Batch 12's
   prepare is still in flight, so the sounds step should ride with
   it (or run `--complete`).
2. **Oracle repair order for Batch 10** (when repair resumes):
   `ParityFont` first (1 error now, 7 drivers), then the `LocalPlayer`
   ctor group (rescues `ambient_sound`, `entity_bound_sound`,
   `sound_engine`, `fire_starter_sound` in this batch, and is one of
   `VanillaHudOracle`'s errors), then `VanillaSoundRig` (2 errors —
   rescues `music` outright, since `VanillaMusicOracle` is a pure
   cascade now, plus the rig legs of `ambient_sound`,
   `entity_bound_sound`, `sound_engine`),
   `VanillaWeatherRenderOracle` (1 error),
   `VanillaMipmapOracle` (1 error),
   `VanillaDragonRenderOracle` (1 error) +
   `VanillaTextDisplayOracle` (2 errors) +
   `VanillaFireStarterSoundOracle` (2 errors),
   `VanillaSimpleContainerOracle` (1 error),
   `VanillaTitleDebugOracle` (4 errors, one missing method),
   `VanillaContainerScreenOracle` (5 errors, one missing class),
   `VanillaMenuOracle` (8 errors, mechanical — rescues
   `VanillaRecipeBookOracle` / `VanillaPlaceRecipeOracle` as
   cascades),
   `VanillaBlockStateModelOracle` / `VanillaModelChoiceOracle` /
   `VanillaModelResolutionOracle` (13/11/8 errors — the 1.21.11 model
   packages need the 1.21.11-form backport, not the 26.x form; note
   `VanillaItemModelOracle` already shows what that backport looks
   like),
   then `VanillaBlockEntityRenderOracle` (30 errors — the
   `block_entity_render` driver is the highest-value single repair:
   it is the check that would verify the G01 sign-board fix and the
   G04 geometry equivalents), `VanillaSectionMeshOracle` (45 errors),
   `VanillaModelBakingOracle` (19 errors),
   `VanillaEntitySoundPacketOracle` / `VanillaProtocolClient` /
   `SecureChatClient` (packet-shape translations), the
   `world.clock` family (`VanillaEnvironmentOracle`,
   `VanillaAudioAttributesOracle` — 1.21.11 HAS environment
   attributes now, so these need only the clock/RegistryLayer legs
   removed, not a rewrite), and last the renderpearl/`Hud` backports
   (deepest — B10-O05).
3. **1.21.7-record correction — B10-G04 never existed (carried from
   the 1.21.10 run).** The 1.21.7 ledger's B10-G04 ("Five more vanilla
   block-entity renderers have no port equivalent") is wrong: all five
   have port equivalents in `game.rs` geometry functions in the 1.21.7
   tree itself, so the item was wrong for 1.21.7 too — not just for
   26.1/26.1.1/26.1.2/1.21.10/1.21.11. The 1.21.7 ledger should carry
   the correction. The lesson for future runs: a "no port equivalent"
   claim needs a search over the whole render path (`game.rs`
   geometry calls), not just the `Kind` enum — the BE renderer file
   is not where this port draws half its block entities.
4. **Goat-voices docstring** (B10-G06 note): the driver's "Silent
   ambient suppression is a separate gap" docstring is stale for
   1.21.11 (suppression is implemented + unit-tested) — driver-hygiene
   fix, ungate the two controls when the driver next runs.
5. **Boundary decisions** (to avoid double-counting): `test_sign_literal`
   (sign BE NBT via commands) counted as Batch 4/9, not Batch 10;
   `test_storage_menu_parity` and the recipe-book logic drivers
   (`recipe_book`, `recipe_book_variants`, `recipe_book_view`) counted
   as Batch 4/7; the dragon `/damage` gap in the dragon-roar driver
   docstring belongs to Batch 3/9; the silent-goat check (B10-G06)
   overlaps Batch 3's entity work. Recorded here with cross-refs
   (coordinator dedups): `VanillaBlockEntityAnimOracle` (Batch 4),
   `VanillaTextDisplayOracle` + `VanillaTextDisplayTransformOracle`
   (Batch 3/4), `VanillaRecipeBookOracle` / `VanillaPlaceRecipeOracle`
   (Batch 4/7 — both 0 own errors in 1.21.11, pure cascades via
   `VanillaMenuOracle`),
   `ServerPackClient` (Batch 11? — needs assignment),
   `VanillaSpriteAnimationOracle` / `VanillaTranslucencyOracle` /
   `VanillaTabListOracle` / `VanillaPackResourcesOracle` (drivers
   outside this batch's 50 but squarely render/UI family — no other
   batch brief names them), `VanillaEntityParticlesOracle` (Batch 3 —
   PASSES in 1.21.11, failed 18e in 1.21.10).
   `VanillaProtocolClient` / `SecureChatClient` are shared helpers
   recorded as Batch 0 residual, not B10 items.
6. **Batch 0 table landing.** This batch's oracle measurement
   (`b10/evidence/measure/results.json`) is a scoped re-measurement
   run because the Batch 0 table had not landed. When the real Batch 0
   table lands, diff it against this measurement: any difference in
   the 67 in-scope oracles changes §3/§6. The measurement used the
   exact Batch 0 method, so the two should agree — a disagreement
   would be a finding, not a rounding error. The provisional-triage
   fallback in common.md should NOT be used for 1.21.11: 32/67
   in-scope oracles deviate from the 1.21.10 table.
7. **26.3 leaks spotted in the 1.21.11 tree (cross-refs, NOT B10
   items).** `entity.sulfur_cube.eject` and sulfur-cube handling in
   `game/dispenser.rs` (`:12`, `:644`, `:703-707`, `:1407-1413` —
   0 "sulfur" occurrences in 1.21.11's `sounds.json`, the sulfur cube
   does not exist in 1.21.11); `block.potent_sulfur.noxious_gas`
   special-cased in `settings.rs` `sound_source`;
   `minecraft:potent_sulfur` in
   `horizon-client-core/src/block_entity_nbt.rs` and
   `horizon-client-render/src/chunk/block_entity.rs`. For
   Batch 3–6 / the leakage audit, not this batch.
8. **Method note for future batches:** the measured table is the only
   oracle authority — this batch verified every oracle claim against
   its own measurement plus fresh javap, and found no case where the
   measurement was wrong. An import-level check would have missed the
   method-level breaks again (notably `VanillaSoundEventOracle`'s
   repair, `VanillaSubtitleOracle`, `VanillaChatScreenOracle`,
   `VanillaWeatherRenderOracle`, `VanillaMipmapOracle`, and the
   `MaterialBaker`-is-an-interface cases). And a "no port equivalent"
   claim needs a whole-render-path search, not one file (§8 item 3).
   Finally: do not carry a previous version's oracle table forward as
   provisional triage for a different version — 1.21.11 vs 1.21.10
   differs on 32 of 67 in-scope oracles, and the oracle SOURCES differ
   between the trees (the template's repairs), so even same-named
   oracles are different code.

## 9. Verdict

Batch 10 for 1.21.11 is complete: 25 behaviour checks resolved
statically (13 pass, 12 gap), 19 items minted (11 behaviour gaps,
1 fixed, 3 closed, 4 oracle defects), all 50 in-scope drivers triaged
from this batch's Batch 0-method measurement (13 runnable-not-run, 34
oracle-blocked, 3 no-oracle), 12 in-game probes specified and not run.
Fourteen of the 1.21.10 run's 18 items reproduce with fresh 1.21.11
evidence; one is fixed (B10-G01 sign boards — the fix originated in
this template); three never existed (B10-G04, B10-G07, and B10-G05,
whose premise inverts because vanilla 1.21.11 HAS the swap ticker).
Six of the 1.21.7 run's 11 items reproduce; one is fixed (B10-G01),
two never existed, two stay withdrawn. The headline defects that
remain are the familiar ones — beds are invisible (and their items
fail to decode too, falling back to the missing model), the game has
no `sounds.json` — with the same sound-side cluster behind them:
three level-event sounds lost to 26.3-form splits (wax-on, potion
break, digging), item-use swings invisible to other players, and
sound categories derived from event names instead of call sites. The
new headline is the loading stick (B10-G16): 1.21.11 carries the
identical pause-during-load deadlock the 1.21.7 fix closed, all three
pause sites ungated — propagation is the owner's decision. The oracle
picture is the best of any 1.21.x run yet and the clearest sign the
template's oracle repairs landed: 39 failing in-scope oracles (down
from 44 in 1.21.10), six oracles passing that failed for 1.21.10,
and 13 of 50 drivers runnable by javac. Nothing in this batch was
fixed, no production code was touched, no driver was run, and no git
command was run in the worktree.

Evidence index (all under
`~/workspace/mcir-work/sweep-1.21.11-batches/b10/evidence/`, plus the
deliverables in `b10/`):

- `client-classes.txt`, `server-classes.txt` — full jar listings
  (10,291 / 7,163 classes).
- `jar-assets/` — sign/bed models + blockstates + all 1,505 item
  definitions extracted from the client jar.
- `measure/` — the Batch 0-method oracle measurement: `results.json`
  (391 files × 3 classpaths), `drivers.json` (648 test files),
  `big-compile-{server,client,both}.log`.
- `measure-oracles-b10.py`, `fetch-libs.py`, `oracle-detail.py`,
  `oracle-status.py`, `driver-table.py`, `census-sounds.py` — the
  measurement and analysis scripts (adapted from the 1.21.10 batch's).
- `oracle-errors-client.txt` — full per-oracle error detail (symbols
  and locations) for all in-scope oracles, extracted from the client
  big-compile log.
- `driver-table.txt` — per-driver status for all 50 in-scope drivers.
- `census-sounds.txt` — item-definition census + sounds wiring diff.
- `ber-types.txt` — the 27 registered BE renderer types.
- `javap-ber.txt`, `javap-ber-static.txt`,
  `javap-signs-beds.txt`, `javap-specialmodelrenderers.txt`,
  `javap-special-bootstrap.txt` — BE renderers, sign/bed renderers,
  special-model bootstrap (G01/G03/G04).
- `javap-animationutils.txt`, `javap-animationutils-c.txt`,
  `javap-zombiemodel.txt`, `javap-livingentity-server.txt`,
  `javap-livingentity-client.txt`, `javap-player-client.txt`,
  `javap-localplayer.txt`, `javap-sgpli.txt`, `javap-mpgm.txt`,
  `javap-iteminhand.txt` — swing state machine (G08), swing packet
  (G11), zombie arms (G07), digging (G14), swap ticker (G05).
- `javap-potpatterns.txt` — `DecoratedPotPatterns` (G09).
- `javap-shear.txt`, `javap-shear-species.txt`, `javap-entity.txt`,
  `javap-mob.txt` — dispenser shear (§4 check 8) and the shear sound
  categories (G15), silent-goat suppression (G06), leash shear.
- `javap-leveleventhandler.txt` — the 82-case lookupswitch, 3003 /
  2002-2007 cases, no 1053/1054, no 2019/2020 (G12/G13/G14).
- `javap-potions.txt`, `javap-honeycomb.txt` — vanilla server sends
  only 2002/2007 and 3003, no 1053/1054, no direct wax sound
  (G12/G13).
- `javap-checkbox.txt`, `javap-checkbox-builder.txt` — Checkbox has
  no row-limit feature in 1.21.11 (§4 note).
- `javap-keymapping.txt`, `javap-keymapping-category.txt`,
  `javap-options.txt`, `javap-packresources.txt`,
  `javap-textrenderable.txt`, `javap-font.txt`, `javap-gui.txt`,
  `javap-soundmanager.txt`, `javap-sound.txt`,
  `javap-simplesound.txt` — options API (O01), ParityFont cause
  (O02), Font/Gui/Sound oracle causes (O03/O04/O05).
- `javap-levelloading.txt`, `javap-levelloading-c.txt` —
  `LevelLoadingScreen.shouldCloseOnEsc` returns false (G16).
- `1.21.11.json`, `asset-index-29.json`, `sounds-1.21.11.json`,
  `version_manifest_v2.json` — Mojang version/asset files for the
  wiring diff only; must NOT be committed.
- `libs/` — sha1-verified Mojang 1.21.11 libraries used for the
  measurement (61 client + 39 server jars); must NOT be committed.
- `jsr305-3.0.2.jar` — copy from the 1.21.10 tree (the 1.21.11 tree
  carries none); must NOT be committed.
- `classes/`, `sclasses/` — extracted jar classes for javap; must
  NOT be committed.
