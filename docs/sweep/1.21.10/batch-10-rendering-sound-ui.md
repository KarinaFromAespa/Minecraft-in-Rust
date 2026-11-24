# Batch 10 — Rendering, sound, UI (1.21.10)

**Version:** 1.21.10 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.10` ·
**Ledger slice:** `sweep-1.21.10-batches/b10/slice.json` · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 10 — models, block-entity rendering,
screens, sound wiring, options. Checks + triage only; no production code
touched, no driver run, no cargo test run, no git command run in the
worktree. Everything below is static analysis: `javap` on the named
1.21.10 jars, source comparison against `versions/1.21.10/crates/`,
checks of the prepared 1.21.10 data plus vanilla's own asset files, and
oracle status from a Batch 0-method measurement run by this batch (see
§1 — the Batch 0 table had not landed when this batch ran).

Date: 2026-11-25. Scope: `docs/PARITY-SWEEP.md` §4 Batch 10 — models,
block-entity rendering, screens, sound wiring, options — re-checked
against vanilla **1.21.10** (DataVersion 4556, Java 21, protocol 773,
data pack 88.0, resource pack 69.0). The 26.1.2 Batch 10 report is the
template; every 1.21.7 (PRIMARY) and 26.1.2 (SECONDARY) defect was
re-verified against 1.21.10's own jar/assets + the 1.21.10 Rust source,
not assumed to carry over. **Vanilla 1.21.10 is a different era from
26.1.2 in scope**: it predates the `EntityTypes`/`BlockEntityTypes`
holder move, keeps the `minecraft:bed` BE type, has no
`environment_attribute`/`game_rule` registries, no `SwingState` /
`SwingAnimation` / `FirstPersonHandsAndItems`, no `Hud` class, no
`renderpearl` GPU API, no `model.sprite`/`model.geometry`/
`model.cuboid` packages, no `renderer.block.dispatch` package, and
slot displays number 8. **The port differs from the 26.1.2 tree just as
much**: the 1.21.10 tree is template-derived from the completed 1.21.11
port, not a delta of 26.1.2, and it carries the 1.21.11 sign-board fix
— so the headline finding of this batch is that **B10-G01 (sign
boards) is FIXED for 1.21.10**, the first version in the sweep where
placed signs render their boards. The oracle picture is far worse than
26.1.2's: only 104 of 391 oracles pass on the client classpath (26.1.2:
189), and 42 of the 67 in-scope oracles deviate from the 26.1.2 table —
provisional triage from the 26.1.2 table would have been wrong for most
of the scope, which is why this batch measured the table itself (§1).

## Tally

**24 resolved behaviour checks: 13 PASS, 11 GAP** (all static; see §1
for what "resolved" means here). Behaviour/non-behaviour split:
**24 behaviour + 0 non-behaviour** — oracle-status checks are NOT
counted in the tally (26.1 convention — oracle triage has its own
counts in §3). The 24 are the 23 template checks (four verdict flips:
G01's check now passes, G04's check now passes, G05's check now
passes, G07's check now passes — the Checkbox check is N/A for 1.21.10,
see §4 note) plus one new check, the item-definition census (§4 check
13). Of the 1.21.7 batch's 11 items (PRIMARY), **6 reproduce**
(B10-G02, B10-G03, B10-O02–B10-O05 — §2/§3, with 1.21.10-specific
evidence), **2 are fixed** (B10-G01, B10-G05), **1 never existed**
(B10-G04 — §2, lands as a closed item), and **2 stay withdrawn**
(B10-G06, B10-O01 — §4, report-only). Of the 26.1.2 batch's 17 items
(SECONDARY), **13 reproduce** (B10-G02, B10-G03, B10-G08–B10-G15,
B10-O02–B10-O05), **1 is fixed** (B10-G01), and **2 never existed**
(B10-G04, B10-G07 — §2, both land as closed items). The 26.1.2
withdrawals stay withdrawn (B10-G05, B10-G06, B10-O01 — §4). Withdrawn
numbers are NOT reused. **No new behaviour items** are minted: the one
new thing in scope (the item-definition census) verifies as a pass
(§4 check 13), and the G04/G07 corrections remove items rather than
adding them. Per-section counts reconcile: §2 has 10 gap items covering
11 gap checks (B10-G03 covers two checks — placed beds and the
special-model decoder), §4 has 13 pass checks, 11 + 13 = 24 = slice
checks.total.

The 50 in-scope drivers were **not run** — their status was triaged
from this batch's Batch 0-method measurement
(`sweep-1.21.10-batches/b10/evidence/measure/results.json`, fixpoint
big-compile) and this batch's driver map
(`b10/evidence/measure/drivers.json`; see §1):

| Driver state | Count | Drivers |
|---|---|---:|
| Runnable by javac (all oracles compile) — NOT RUN | 9 | cloud_render, container_title, cuboid_model, firework_tooltip, lore_tooltip, options, stop_sound_wire, tooltip_display, world_options |
| Blocked on ≥1 unrepaired oracle | 38 | ambient_sound, arrow_hit_sound, audio_attributes, block_entity_render, blockstate_model, breaking_model, chat_screen, container_screen, dragon_death_roar, dragon_render, entity_bound_sound, entity_model, entity_sound_packet, environment_render, fire_starter_sound, first_person, goat_voices, hud, item_model, menu_click, mipmap, model_baking, model_choice, model_resolution, music, pack_screen, recipe_book_render, screen_layout, secure_chat, situational_music, sound_engine, sound_event, subtitle, text_display_render, title_debug, weather_render, world_border_render, zombie_cure_sound |
| No oracle (server-e2e / protocol drivers) — NOT RUN | 3 | hostile_sound_categories, mob_local_sound, ominous_warning_sound |
| **Total in scope** | **50** | |

Seven drivers runnable in 26.1.2 are blocked for 1.21.10
(blockstate_model, container_screen, mipmap, model_choice,
model_resolution, weather_render, zombie_cure_sound — their oracles
fail against the 1.21.10 jar; §6). All 6 `audio_harness` drivers are
oracle-blocked AND data-gated by B10-G02, as for 26.1.2. The remaining
runnable drivers are unmeasured, not passing.

Per-driver oracle detail is in §6. Oracle defects found: 4 items
(B10-O02–B10-O05, §3), covering 44 failing in-scope oracles recorded
explicitly in §3 (1 + 21 + 11 + 11), plus 2 shared packet-client
helpers recorded in §6 as Batch 0 residual. Behaviour gaps found: 10
items (B10-G02, B10-G03, B10-G08–B10-G15, §2). Two fixed items (B10-G01,
B10-G05) and two closed items (B10-G04, B10-G07). No B10-H items: every
check was settled statically or is recorded as not-measured in §6; the
in-game probes are specified in §7.

## 1. Method and what "resolved" means

No driver runs and no cargo tests were possible (checks-only batch),
so this batch resolves a check only when static evidence settles it:

- **javap on the named 1.21.10 jars** — client jar
  `e6-deltas/versions/1.21.10/jars/client-named.jar` (9,969 classes,
  sha1 `715f58165acfaf8b61ab921064f08c202b41bd0c`, Mojang-mapped)
  for all rendering/sound/UI classes, and the named server-game jar
  `e6-deltas/versions/1.21.10/jars/server-game-named.jar` (6,964
  classes, sha1 `ba1fc89ead95b8ebdb8727f84e2bd487533c8b7f`) for
  server-side classes (`LivingEntity`, `ServerGamePacketListenerImpl`,
  `ShearsDispenseItemBehavior`, species `shear` methods,
  `AbstractThrownPotion`). Vanilla 1.21.10 is the spec throughout;
  nothing was assumed from the 26.3 base, the 26.1.2 form, or the
  1.21.7 form. The bundler jar was never javap'd (no game classes).
  JDK 25 (`~/workspace/tools/jdk-25/bin/javap`). Classes needed
  repeatedly were extracted once to the batch evidence dir
  (`evidence/classes/`, `evidence/sclasses/`) and javap'd from there.
- **Vanilla asset files from the client jar** — the jar bundles
  `models/block/` entries and 1,488 `items/` entries, so block-model
  emptiness, item-model forms and special-renderer use are verifiable
  directly. `sounds.json` is NOT in the jar (it ships in the asset
  index); the 1.21.10 copy was fetched from Mojang's asset CDN via
  piston-meta (asset index 27, sha1-verified
  `fbd947a289640ecc84bcfd7e7ff1dbeb095512a3`, 508,156 bytes;
  `sounds.json` sha1 `65e6d5482d97966fc8c870ce3c0c6483158e63b8`,
  565,050 bytes, 1,770 events — `evidence/sounds-1.21.10.json`).
- **The prepared 1.21.10 data** — Batch 12's `horizon-prepare` run
  completed EXIT 0 into
  `versions/1.21.10/.local/horizon-client/1.21.10` (185M; log at
  `sweep-1.21.10-batches/b12/evidence/horizon-prepare.log`, export
  flow only, no sounds step). The tree was inspected directly for
  B10-G02 (no `sounds.json` anywhere in it).
- **Datagen registries** —
  `26.1-adapters/versions/1.21.10/datagen/reports/registries.json`
  for registry counts (1,771 sound events, 96 data component types,
  24 decorated-pot patterns, 49 BE types incl. `minecraft:bed`,
  `provides_pottery_pattern` absent, `map_color` present,
  `swing_animation` absent).
- **Oracle status from this batch's own Batch 0-method measurement.**
  The Batch 0 table (`docs/sweep/1.21.10/batch-0-oracles.json`) had
  NOT landed when this batch ran (`sweep-1.21.10-batches/b0/` is
  empty), and the common.md fallback — provisional triage from the
  26.1.2 table — would have been badly wrong: the oracle sources
  differ heavily (172/391 files differ 1.21.10 vs 26.1.2), and the
  measurement proved it (42/67 in-scope oracles deviate from the
  26.1.2 table). So this batch ran a scoped re-measurement using the
  exact Batch 0 method (fixpoint big-compile per classpath,
  `sourcepath=False` on fixpoint rounds, jsr305 on the client
  classpath) against 1.21.10's own named jars + sha1-verified Mojang
  1.21.10 libraries (71 client + 44 server jars, fetched via
  piston-meta; server libs from the bundler's
  `META-INF/libraries.list`). Script:
  `b10/evidence/measure-oracles-b10.py` (adapted from the worktree's
  `docs/sweep/measure-oracles.py`; the stock script is unusable here
  because `lab.config()` needs a `.local` the worktree tree does not
  carry in the form it expects). Results:
  `b10/evidence/measure/results.json` (server 89/391 pass, client
  104/391 pass, both 104/391 — for comparison 26.1.2 was 189/391 on
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

### B10-G01 — Sign boards ARE rendered (FIXED — the 1.21.10 headline)

The 1.21.7 and 26.1.2 gap is gone: the 1.21.10 port draws sign boards.
The `Kind::Sign` arm in `block_entity_render.rs:2808`
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
`entity/signs/{wood}`). Vanilla side confirmed fresh: `oak_sign.json`
is particle-only, and `AbstractSignRenderer` is abstract over
`getSignModel(BlockState, WoodType)` + `submitSign`
(`javap-signs-beds.txt`) — the port's model-key + texture approach
matches that shape. This is the 1.21.11 fix (ac8dd9dc) carried into the
template-derived port. Two caveats: exact geometry fidelity is
unverified statically (probe P2, §7), and the `:1020` comment ("Sign
boards and beds are block models in the terrain mesh") is now stale
for signs — it describes `CacheOptions`, but it reads as a claim that
boards are not drawn. Lands as a **fixed** item.

### B10-G02 — No `sounds.json` in the prepared 1.21.10 resources: the game is silent and 6 audio drivers cannot run (MAJOR)

REPRODUCES, with the strongest evidence of any run: a prepare actually
ran for 1.21.10 and still produced no `sounds.json`. Batch 12's
`horizon-prepare` completed EXIT 0 into
`versions/1.21.10/.local/horizon-client/1.21.10` (185M; log
`b12/evidence/horizon-prepare.log` — "Copying the client's assets and
data", export flow only, no sounds step, no `--complete`), and a find
over the whole prepared tree returns **no `sounds.json` anywhere**:
`assets/minecraft/` holds atlases, blockstates, equipment, font,
items, lang, models, particles, shaders, texts, textures — but no
`sounds.json` and no `sounds/` directory. The prepare flow has a
`sounds` subcommand and `--complete` copies sounds; `--prepare` alone
does not. The client then fails silently: `SoundRegistry::load`
(`horizon-client-render/src/sound_events.rs:262`) reads each
namespace's `sounds.json` with `let Ok(text) = fs::read(...) else {
continue; }` (`:280-282`) — a missing file is skipped without a word,
and `is_empty()` (`:287-290`) documents the state ("the resources hold
no `sounds.json` (sounds not imported from the player's launcher when
the game was prepared)"). With an empty registry the game plays no
event sound at all. The 6 `audio_harness` drivers
(`ambient_sound`, `audio_attributes`, `music`, `situational_music`,
`sound_engine`, `sound_event`) all call `require_sounds()`
(`tools/audio_harness.py:35`, requiring
`RESOURCES / 'assets/minecraft/sounds.json'`) and are data-gated by
this item — and oracle-blocked too (B10-O04), so the sounds step and
the sound-oracle repairs are both needed before any audio driver
measures anything (§8 item 1). The wiring itself is fine (§4 check 1):
the failure is the missing file, not a mismatch.

### B10-G03 — Beds are invisible when placed, AND bed items fail to decode (MAJOR)

REPRODUCES, both halves, with the item half now proven statically
(26.1.2's "unproven fallback" is settled — the fallback is the missing
model).

**Placed beds.** Vanilla 1.21.10 renders beds through
`BedRenderer` (`createHeadLayer` / `createFootLayer` / `submitPiece`,
registered for `BlockEntityType.BED` — `javap-ber.txt`); the block
model is particle-only (`models/block/bed.json`, all bed blockstates
point at it), so there is no terrain-mesh fallback. The port has no
bed draw path at all: the `Kind` enum
(`block_entity_render.rs:323`) has no `Bed` variant and no `Kind::Bed`
arm exists anywhere; the only "bed" mention in the whole
block-entity-render file is the stale `:1020` comment (see G01);
`item_render/special.rs` has 0 bed mentions; `game/beds.rs` is sleep
mechanics only. A placed bed renders nothing.

**Bed items.** Vanilla 1.21.10's `items/red_bed.json` (and all 16 bed
colours — census, §4 check 13) is a SINGLE `minecraft:special` with
type `minecraft:bed` + texture — the 1.21.7 form, NOT 26.1.2's
composite-of-two (1.21.10 has no `composite` model type at all). The
port's `SpecialModel::decode` (`horizon-assets/src/item_model/
special.rs:82`) has NO bed arm: the ten arms are banner, conduit,
chest, copper_golem_statue, decorated_pot, head, player_head,
shulker_box, shield, trident (`:89-135`). The file's own header doc
(`:1-8`) states it plainly: "Bed, standing-sign and hanging-sign types
exist in 1.21.10's registry but have no kind here yet: a model naming
one fails to decode and the item falls back to the missing model (a
known gap, not a 1.21.10 behaviour)". The decode error propagates via
`?` (`item_model.rs:483`, `"minecraft:special" =>
SpecialModel::decode(...)?`); `load_definitions`
(`horizon-client/src/item_render.rs:691`) logs a warning and SKIPS the
item; `append_layers` renders a missing definition via
`state.missing(out)` (`item_model.rs:1102`). So all 16 bed items render
as the missing model — proven statically, unlike 26.1.2. (Standing-sign
and hanging-sign specials are also undecodable, but no bundled 1.21.10
item definition uses them — census §4 check 13 — so bed items are the
only observable gap. Bell, book and end_cube are correctly absent:
1.21.10's special registry holds 13 types and does not include them.)

### B10-G04 — Five vanilla block-entity renderers have no port equivalent (NEVER EXISTED — closed)

The 1.21.7 item (five renderers) and the 26.1.2 item (seven) are both
wrong for 1.21.10, for the same reason the 26.1.2 run found: the
26.1.1/1.21.7 runs searched only the `Kind` enum and missed the
`game.rs` geometry path. All five 1.21.7 members — and the two extra
26.1.2 members — have geometry-function equivalents with live call
sites in the 1.21.10 tree, each citing its vanilla renderer in a doc
comment:

| Vanilla renderer | Port equivalent | Call site |
|---|---|---|
| SpawnerRenderer + TrialSpawnerRenderer | `spawner_geometry` (`game/spawner.rs:508`) | `creatures.rs:8067` |
| CampfireRenderer | `campfire_geometry` (`game/campfire.rs:131`) | `game.rs:6787` |
| PistonRenderer (moving piston) | `moving_piston_geometry` (`game/moving_pistons.rs:10`) | `game.rs:7271` |
| BrushableBlockRenderer | `brushable_geometry` (`game/brush.rs:313`) | `game.rs:6788` |
| VaultRenderer | `vault_geometry` (`game/vault.rs:223`) | `game.rs:6784` |
| ShelfRenderer | `shelf_geometry` (`game/shelf.rs:200`) | `game.rs:6789` |
| OminousItemSpawnerRenderer | `ominous_item_spawner_geometry` (`game/ominous_item_spawner.rs:205`) | `game.rs:6791` |
| StructureBlockRenderer | `bounding_box_gizmos` (`game/structure_blocks.rs:61`, modelling `BlockEntityWithBoundingBoxRenderer`: game-master gate, 96-block view distance, render modes) | `debug_keys.rs:985` via `debug_gizmos`, called every frame at `game.rs:9509` |
| TestInstanceBlockRenderer | `test_instance_marker_geometry` (`game/test_instances.rs:275`) | `game.rs:6823` |

Vanilla 1.21.10 registers 27 BE renderers (`javap-ber.txt`: BANNER,
BEACON, BED, BELL, BRUSHABLE_BLOCK, CAMPFIRE, CHEST, CONDUIT,
COPPER_GOLEM_STATUE, DECORATED_POT, ENCHANTING_TABLE, ENDER_CHEST,
END_GATEWAY, END_PORTAL, HANGING_SIGN, LECTERN, MOB_SPAWNER, PISTON,
SHELF, SHULKER_BOX, SIGN, SKULL, STRUCTURE_BLOCK,
TEST_INSTANCE_BLOCK, TRAPPED_CHEST, TRIAL_SPAWNER, VAULT — the same 27
as 26.1.2). The 27-slot recount for 1.21.10: **26 fully covered, 1
uncovered (bed, G03)** — Kind arms cover 13 kinds + sign text counts
2 slots (standing + hanging) and chest covers 3 slots (chest, trapped
chest, ender chest — `block_entity_render.rs:2936`, `:2942-2949`),
beacon beams are check 5, and the nine geometry functions above cover
the rest. Exact geometry fidelity is unverified statically (probes
P3/P4/P5, §7). **1.21.7-record correction:** the 1.21.7 tree ALSO has
all five functions with live call sites (verified 2026-11-25:
`versions/1.21.7/.../game.rs:6778/6781/6782/7265`,
`creatures.rs:8137`) — so the 1.21.7 run's B10-G04 was wrong for the
1.21.7 tree too (the same Kind-enum-only search error). Flagged to the
coordinator (§8 item 3). Lands as a **closed** item.

### B10-G05 — First-person probe emits swap-ticker columns the repaired oracle drops (FIXED — probe side)

The 1.21.7 gap is fixed on the probe side. Vanilla 1.21.10 has NO
`itemSwapTicker` / `getItemSwapScale` (grep of
`javap-livingentity-server.txt` + `javap-player-server.txt` finds only
`swapHandItems`), and the 1.21.10 probe
(`crates/horizon-client/examples/probe/first_person_probe.rs`, 471
lines) emits NO swap-ticker columns — its tick line carries
strength ticker/scale + hand heights only. The probe now matches
vanilla 1.21.10. The ORACLE half is not fixed: `VanillaFirstPersonOracle`
still emits the columns (`VanillaFirstPersonOracle.java:387`:
`swapTicker.getInt(player)` + `player.getItemSwapScale(1.0F)`, plus the
reflective field at `:234`) and cannot compile against 1.21.10 — that
residue is part of B10-O03 (the oracle repair must drop the columns,
as the 1.21.7 repair did). Lands as a **fixed** item, with the oracle
residue recorded under B10-O03.

### B10-G07 — Baby zombie holding an item lowers its arms instead of keeping them down (NEVER EXISTED — closed)

The 26.1.2 gap's premise does not exist in 1.21.10. Vanilla 1.21.10 has
NO baby guard: `AnimationUtils.animateZombieArms` is the 5-arg form
`(ModelPart, ModelPart, boolean, float, float)` with no baby/item
check, and `AbstractZombieModel.setupAnim` calls it unconditionally,
gated only on `isAggressive` (`javap-zombiemodel.txt:18-27`; no
`isBaby` / `getMainHandItem` anywhere in the file). The baby guard is
a 26.1 addition — it postdates 1.21.10. The port matches vanilla
1.21.10 exactly: `mobs/anim.rs:635-657` ("1.21.10 has no baby guard, so
a baby holding an item raises its arms like an adult (26.1 returned
early for a baby with a main-hand item)") plus the unit test
`a_baby_zombie_holding_an_item_raises_its_arms_like_an_adult`
(`:1668`). A baby zombie holding an item raising its arms IS vanilla
1.21.10 behaviour. Lands as a **closed** item.

### B10-G08 — First-person swing uses the 26.3 `SwingState` state machine vanilla 1.21.10 does not have (MINOR)

REPRODUCES. Vanilla 1.21.10's swing is plain fields:
`swinging` / `swingTime` / `swingingArm` / `attackAnim` /
`oAttackAnim` (`javap-livingentity-server.txt:101-121`);
`getCurrentSwingDuration()` is private and returns a flat 6 adjusted
by haste (6-(1+amp)) / fatigue (6+(1+amp)*2) — unlike 26.1.2 it does
NOT read the held item's swing animation (there is no
`swing_animation` component in 1.21.10's 96-component registry);
`swing(hand[, boolean])` restarts only when `!swinging || swingTime >=
duration/2 || swingTime < 0` (`:4852-4872`); `updateSwingTime()`
resets at duration, so `attackAnim` never reaches 1.0 (`:5317-5360`).
The port carries the 26.3 form: `SwingDescription` / `SwingState`
(`horizon-client/src/first_person.rs:490-499`); `start_if_able`
allows a restart at ticks==0 where vanilla blocks it (`:507-522`);
`tick()` holds the animation at 1.0 an extra tick (clears only when
`ticks > duration`, `:528-541`); the duration is cached at swing start
where vanilla re-reads effects each tick. (`swing_duration`, `:564`,
matches vanilla's formula for the default 6-tick animation — the
formula is right, the state machine around it is not.)

### B10-G09 — Decorated-pot sherd patterns never resolve: every side renders blank (MINOR)

REPRODUCES. Vanilla 1.21.10 resolves sherd patterns via
`DecoratedPotPatterns.getPatternFromItem(Item)`
(`javap-potpatterns.txt:29` — a static item→pattern map; no
`provides_pottery_pattern` anywhere in the class). The port reads the
26.3 component instead: `components["minecraft:provides_pottery_pattern"]`
(`block_entity_render.rs:1540`, in the sherd-item scan at `:1528-1552`).
That component is ABSENT from 1.21.10's 96-component registry, and
there is no `data/minecraft/decorated_pot_pattern/` directory in the
datagen output — so `pot_patterns` stays empty and every sherd side
falls back to blank (`:2469`, `:3193`). The registry lists 24 patterns,
unreachable through the component path. (The port's
`DATA_COMPONENT_TYPES` list itself is exact — 96/96, zero drift either
way, verified against the registry — so this is a lookup-path defect,
not a list defect; the 26.1.2 latent-drift note does not apply here.)

### B10-G10 — `filled_map` fails item decode: the `map_color` tint source is gone (MINOR)

REPRODUCES. Vanilla `items/filled_map.json` tints with
`minecraft:constant` (-1) + `minecraft:map_color` (default 4603950).
The port's `TintSource` (`horizon-assets/src/item_model.rs:543-551`)
has NO Map variant; the decode fallthrough returns
`Err("unknown item tint source")` (`:580`), propagating via `?` so the
definition is skipped at load and the item renders as the missing model
(same load path as bed items, G03). The census (§4 check 13) shows
`map_color` appears in exactly one bundled item definition
(`filled_map.json`), so the blast radius is one item — but it is the
map every player holds. (`minecraft:map_color` IS in the port's
`DATA_COMPONENT_TYPES` list, `item_model.rs:83` — the component is
known; only the tint source is missing.)

### B10-G11 — Item-use swings never reach other players: `swing_hand` no longer sends `ServerboundSwingPacket` (MINOR)

REPRODUCES. Vanilla `LocalPlayer.swing` calls
`AbstractClientPlayer.swing(hand)` then UNCONDITIONALLY sends
`ServerboundSwingPacket(hand)` (`javap-localplayer.txt:628-644`); the
server's `handleAnimate` resets the last-action time + calls
`player.swing(hand)` (`javap-sgpli.txt:4698-4727`). The port's
`swing_hand` (`game/hand.rs:195-216`) sends NOTHING — it only records
an Animate packet if the session is recording. The only three
`S::Swing` sends left in the tree are attack paths
(`game/remote.rs:9937`, `:10057`, `:10064`). Item-use swings (placing
blocks, using items) are invisible to other players; attack swings
still show. (Vanilla 1.21.10's `SwingSource` is three-valued —
`javap-swingsource.txt` — the port's swing-source modelling was not
re-verified beyond the packet gap.)

### B10-G12 — 3003 wax-on sound missing from the level-event path: other players' waxing is silent (MINOR)

REPRODUCES. Vanilla `LevelEventHandler` case 3003 plays the sound
itself: WAX_ON particles (UniformInt 3-5) + `playLocalSound(pos,
HONEYCOMB_WAX_ON, BLOCKS, 1.0, 1.0)`
(`javap-leveleventhandler.txt` offsets 3369-3399). `HoneycombItem.useOn`
plays NO sound directly — 3 `levelEvent(3003)` sites, 0 `playSound`
(`javap-honeycomb.txt`). The port splits the two halves the 26.3 way:
`block_particle_event` (`game/level_particles.rs:35-38`) plays
particles only for 3003, and the sound comes from item code —
`wax_on_sound` (`game.rs:8043`, callers `:8354`/`:8373`) — which only
runs for the local player's own waxing. Another player's (or a
dispenser's, or a copper golem's) waxing arrives as a bare 3003 event
and is silent. (The other wax call sites exist in the 1.21.10 tree and
all play the sound directly from their own code — dispenser wax
`game/dispenser.rs:2206-2235`, copper-golem wax
`game/copper_golems.rs:508`, local honeycomb prediction
`game/remote.rs:8802`/:8810 — so the gap is the event path, not the
call sites: a 3003 arriving from a vanilla server has no sound
attached to it.)

### B10-G13 — 2002/2007 potion-break sound missing from the event path: potion breaks are silent on a 1.21.10-form server (MINOR)

REPRODUCES. Vanilla's server sends ONE event:
`AbstractThrownPotion.onHit` → `levelEvent(instant? 2007 : 2002, pos,
color)`; `ThrownExperienceBottle.onHit` → `levelEvent(2002, pos,
0xFF385DC6)`. The client 2002/2007 case plays `SPLASH_POTION_BREAK`
itself on NEUTRAL, pitch `nextFloat()*0.1+0.9`, after the particles
(`javap-leveleventhandler.txt` offsets 1525-1553).
`LevelEventHandler` is a 79-case lookupswitch with NO 1053/1054/2019/
2020 cases (case list extracted). The port maps the sound to 1053/1054
(`game.rs:4500`, pitch `:4519-4548`) and its own server sends them
(`game/potions.rs:1935` for the experience bottle, `:1967` for thrown
potions: `level_event_at(if instant { 1054 } else { 1053 }, ...)`) —
a closed loop that plays against its own server, but a vanilla-server
client (which sends only 2002/2007) hears nothing.

### B10-G14 — Digging hit sound plays at START, then never again on a 1.21.10-form server (MINOR)

REPRODUCES. Vanilla `MultiPlayerGameMode.startDestroyBlock` plays
NOTHING (0 "Sound" occurrences in the method); `continueDestroyBlock`
plays ONLY the hit sound on `destroyTicks % 4.0 == 0`
(`javap-mpgm.txt` offsets 198-208) as
`SimpleSoundInstance(hitSound, BLOCKS, (volume+1)/8, pitch*0.5,
unseededRandom, pos)`; `addBreakingBlockEffect` has 0 occurrences in
the class. The port inverts this: `remote_mining`'s start branch calls
`breaking_block_effects(hit.pos, hit.face, true)`
(`game/remote.rs:8172`, comment: "startDestroyBlock plays the first
hit and crack itself; the server's level events 2019 and 2020 do the
rest"), and the continue branch plays NOTHING locally
(`remote.rs:8187-8261`). The port's own server sends 2019 every tick /
2020 every 4th (`game/block_breaking.rs:97-101`,
`destroy_progress_event`; client handles 2019/2020 at
`game.rs:4309-4317`) — again a closed loop: against a vanilla 1.21.10
server, which sends neither event (they do not exist in vanilla's
`LevelEventHandler`), the port plays one sound at the first hit and
then goes silent for the rest of the dig.

### B10-G15 — Sound categories come from the event NAME, not the call site: shear and wax-on sounds land on the wrong volume slider (MINOR)

REPRODUCES. Vanilla passes `SoundSource` as a parameter:
`Shearable.shear(level, source, stack)` takes BLOCKS from the
dispenser (`ShearsDispenseItemBehavior`, `javap-shear.txt:147-149`)
and PLAYERS from player shearing (bogged `:111`, mooshroom `:180`);
species `shear` plays via `ServerLevel.playSound(null, this, SOUND,
source, 1.0, 1.0)`; 3003 plays wax-on on BLOCKS. The port's
`Audio::play` derives the source from the event name via
`settings::sound_source` (`horizon-client-render/src/settings.rs:18-90`,
a prefix heuristic with a HOSTILE list of 34): dispenser shear
`entity.sheep.shear` → NEUTRAL, `entity.bogged.shear` → HOSTILE,
wax-on `item.` → PLAYER — all three should be BLOCKS. The dispenser
shear sites use `audio.play`, not `audio.play_on`
(`game/dispenser.rs:1941/1975/1989/2003/2018`). The plumbing exists —
`Audio::play_on` has many game-code callers in 1.21.10
(`game.rs:4456` portal/AMBIENT, `:6158` bubble pop/PLAYERS,
`creatures.rs:5154` projectiles/NEUTRAL,
`ominous_item_spawner.rs:117`, `player_equip.rs:83`, `raids.rs:689`,
`remote.rs:3880/4623/4762`, `world_cmds.rs:635` /playsound) — the
shear/wax sites just don't use it. (Correction carried from 26.1.2:
`Audio::play_on` is NOT "used only by /playsound".)

## 3. Oracle defects

All counts are from this batch's Batch 0-method measurement
(`b10/evidence/measure/results.json`), client classpath unless noted;
full per-oracle error detail (with symbols/locations) is in
`b10/evidence/oracle-errors-client.txt`. "First error" is the first
error javac reports for the file, not necessarily the root cause —
the cause column names the root causes, verified by javap where cheap.

### B10-O02 — `ParityFont` needs `VanillaPackResources.asResourceManager()` plus `TextRenderable.Styled`, both absent in 1.21.10 (MINOR)

REPRODUCES, with a second error 26.1.2 did not have. `ParityFont`
fails with **2 errors** on the client classpath (26.1.2: 1):

| Line | Error | Cause |
|---|---|---|
| `:43` | `cannot find symbol: method asResourceManager()` | 1.21.10's `VanillaPackResources` has `asProvider()` but NO `asResourceManager()` (`javap-packresources.txt`) — same as 26.1.2 |
| `:127` | `cannot find symbol: class Styled` (in `TextRenderable`) | 1.21.10's `TextRenderable` has no nested `Styled` type (the `BakedGlyph` glyph path changed shape) — new for 1.21.10 |

All other `ParityFont` imports exist in 1.21.10 (`GlyphSource`,
`BakedGlyph`, `GlyphProviderDefinition` + `Conditional`,
`ClientPackSource`; `Font` at `net/minecraft/client/gui/Font`). It
blocks 8 drivers (`block_entity_render`, `chat_screen`, `hud`,
`pack_screen`, `recipe_book_render`, `subtitle`,
`text_display_render` — plus its own font driver, outside the 50),
so it stays the highest-leverage single repair (§8 item 2).

### B10-O03 — Render/model oracle family does not compile for 1.21.10 (MAJOR)

REPRODUCES, much worse than 26.1.2: **21 failing oracles** (26.1.2:
15 in this family by the same count). Five oracles that PASSED in
26.1.2 now FAIL (marked ★): the 1.21.10 jar lacks whole model
packages the 26.1.2 jar had.

| Oracle | Errors | Cause (root, javap-verified where cheap) |
|---|---|---|
| VanillaBlockEntityRenderOracle | 42 (26.1.2: 9) | Missing packages `client.model.object.banner`, `client.renderer.rendertype`, `client.renderer.state.level`, `client.resources.model.sprite` (×2), `world.clock`, `world.attribute`; `UvMapping`, `RenderType`, `SpriteGetter`, `BannerFlagModel` absent; `CaseLevel.fuelValues()` abstract gap; `GlyphVisitor.acceptEffect`; `Font.prepare8xTextOutline` / `prepareText` signature; `SignText` ctor (ArrayList vs Component[]); `StandingSignRenderState` / `HangingSignRenderState`, `PlainSignBlock` / `HangingSignBlock`, `SignRenderState.transformations`, `HangingSignRenderer.TRANSFORMATIONS` all absent; `CameraRenderState`, `ChestRenderState.facing` absent |
| VanillaFirstPersonOracle | 25 (26.1.2: 20) | `FirstPersonHandsAndItems` (×7), `SwingState` (×6), `LocalPlayer` ctor (7 args required, 9 passed), `itemUsed` absent, `getCurrentSwing` / `getSwingAnimation` / `getItemSwapScale` absent (incl. the G05 swap-ticker columns at `:234`/`:363`/`:387`), plus no-overrides |
| VanillaItemModelOracle | 45 (26.1.2: 10) | Missing packages `model.geometry` / `model.sprite` / `model.cuboid`; `EntityTypes`, `ResourceManager.Selector`, `BakedQuad`, `TextureAtlas.LOCATION_ITEMS`, `MaterialBaker`, `ItemModelGenerator`, `CuboidModel` / `MissingCuboidModel` absent; `TextureAtlasSprite` / `SpriteContents` ctor arity; `VanillaRegistries.createWorldLookup`, `Reference.bindComponents`, `BundleContents.asMutable`, `Properties.swapAnimationScale` absent; `SpecialModelRenderer.getExtents` shape; `Unbaked.transformation()` absent |
| VanillaBreakingModelOracle | 11 (26.1.2: 2) | `renderer.block.dispatch` + `model.sprite` packages missing; `MaterialBaker`, `BlockStateModelPart`, `QuadInstance`, `BlockModelFeatureRenderer`, `BlockStateModel` absent |
| VanillaSectionMeshOracle | 48 (26.1.2: 7) | `model.geometry` / `model.cuboid` / `model.sprite` / `renderer.block.dispatch` packages missing; `MaterialBaker`, `BlockStateModel`, `FluidStateModelSet`, `BlockAndTintGetter`, `CardinalLighting`, `BlockModelLighter`, `FluidRenderer`, `BlockQuadOutput` absent; `ModelBaker.sprites()` abstract gap; `ModelBlockRenderer` ctor arity; `LeavesBlock.setCutoutLeaves`, `BlockStateBase.emissiveRendering()` arity, `BlockState.isLightPermeable`, `BlockColors.getTintSources` absent |
| VanillaModelBakingOracle | 20 (26.1.2: 3) | `model.sprite` / `model.cuboid` / `model.geometry` / `renderer.block.dispatch` packages missing; `ModelBaker.Interner`, `CuboidRotation`, `FaceBakery`, `UnbakedCuboidGeometry`, `CuboidFace`, `Variant`, `BakedQuad`, `CuboidModel` / `MissingCuboidModel` absent; `Material` ctor arity; `TextureAtlasSprite` ctor arity |
| VanillaDragonRenderOracle | 2 (26.1.2: 1) | Two no-overrides (`:31`, `:33`) |
| VanillaEntityAnimOracle | 8 (26.1.2: 5) | `client.model.monster.illager` package missing; `ArmedEntityRenderState.swingAnimation` / `currentSwing`, `LivingEntity.SwingDescription`, `item.component.SwingAnimation` absent; `PoseStack.rotateDegrees` absent |
| VanillaTextDisplayRenderOracle | 8 (26.1.2: 3) | `ServerLevel` ctor arity (10 args required, 9 passed); `GameRenderState` absent; `GlyphVisitor.acceptEffect`; `Font.prepareText` signature; `Font.prepareBackground` absent |
| VanillaEnvironmentOracle | 18 (26.1.2: 1) | No `environment_attribute` registry in 1.21.10: `world.attribute` / `world.clock` packages missing; `EnvironmentAttributeSystem`, `EnvironmentAttribute`, `ClockManager`, `ClockInstance`, `WeatherAttributes` absent; `Biome.getAttributes`, `DimensionType.attributes` / `timelines`, `RegistryLayer.WORLD`, `RegistryDataLoader.WORLD_REGISTRIES`, `BuiltInRegistries.ENVIRONMENT_ATTRIBUTE` absent |
| VanillaTitleDebugOracle | 4 (26.1.2: 4) | `Gui.canInterruptScreen()` absent (×4) |
| VanillaPackResourcesOracle | 1 (26.1.2: 1) | `PackResources.forEach` absent (`:65`) |
| VanillaBlockEntityAnimOracle | 21 (26.1.2: 3) | `client.model.object.book` / `client.model.object.chest` packages missing; `world.entity.monster.illager`, `world.clock`, `world.attribute` packages missing; `ChestModel`, `BookModel`, `EntityTypes` absent; `Level.fuelValues()` abstract gap; `Mob.position()` absent |
| VanillaTextDisplayOracle | 2 (26.1.2: 1) | `ServerLevel` ctor arity (`:19`) + one no-override (`:20`) |
| VanillaBlockStateModelOracle ★ | 13 (26.1.2: PASS) | `renderer.block.dispatch` + `dispatch.multipart` packages missing; `BlockStateModel`, `SingleVariant`, `WeightedVariants`, `BlockStateModelDispatcher`, `Variant`, `Condition`, `CombinedCondition`, `KeyValueCondition` absent |
| VanillaModelChoiceOracle ★ | 11 (26.1.2: PASS) | `model.sprite` + `renderer.block.dispatch` packages missing; `BlockStateModel` (×5), `BlockStateModelPart`, `WeightedVariants`, `Material` absent |
| VanillaModelResolutionOracle ★ | 8 (26.1.2: PASS) | `model.geometry` / `model.sprite` / `model.cuboid` packages missing; `Material.sprite()` / `forceTranslucent()` absent; `CuboidModel` / `MissingCuboidModel`, `UnbakedCuboidGeometry` absent |
| VanillaWeatherRenderOracle ★ | 1 (26.1.2: PASS) | `RandomSource.createThreadLocalInstance(long)` absent (`:57`) |
| VanillaMipmapOracle ★ | 5 (26.1.2: PASS) | `MipmapStrategy` absent (×3); `NativeImage.computeTransparency()` absent (`:49`) |
| VanillaSpriteAnimationOracle | 5 (26.1.2: 2) | `renderpearl.api.buffers` package missing; `SpriteContents.AnimationState` absent; `SpriteContents` ctor arity; `GpuBufferSlice` absent. Driver outside this batch's 50 — recorded here as render-family |
| VanillaTranslucencyOracle | 4 (26.1.2: 4) | `renderpearl.api.pipeline` package missing; `MeshData.decodeQuadCentroids` absent; `IndexType` absent; `SortState.writeSortedIndexBuffer` absent. Driver outside this batch's 50 — recorded here as render-family |

Passing in this family: `VanillaCuboidModelOracle`,
`VanillaCloudRenderOracle`, `VanillaEntityModelExporter`,
`VanillaStackModelFlagsOracle`, and — a correction to this batch's
own early note — `VanillaTextDisplayTransformOracle` (0 errors; it is
NOT a cascade via `VanillaTextDisplayOracle` in 1.21.10). Cross-batch:
`VanillaBlockEntityAnimOracle` (Batch 4) and
`VanillaTextDisplayOracle` (Batch 3/4) are recorded here, cross-ref'd
in §8 item 5.

### B10-O04 — Sound oracle family does not compile for 1.21.10 (MAJOR)

REPRODUCES, much worse than 26.1.2: **11 failing oracles**. Two
oracles that were pure cascades in 26.1.2 now have OWN errors
(marked †): the cascade chain broke differently because the
`VanillaSoundEventOracle` errors changed shape.

| Oracle | Errors | Cause |
|---|---|---|
| VanillaSoundEventOracle | 3 (26.1.2: 2) | `SoundManager` ctor arity (`:55`: requires `Options, MusicManager`; passed `Options`); `SimpleSoundInstance.getOrResolve` absent (`:83`); `Sound.getAttenuationDistance(float)` — 1.21.10's takes no args (`:104`) |
| VanillaSoundRig † | 3 (26.1.2: cascade) | Own errors: `DeviceList` absent (`:129`); `SoundInstance.getIdentifier()` absent (`:144`); `SoundManager.getChannelDebugString()` absent (`:202`) |
| VanillaMusicOracle † | 5 (26.1.2: cascade) | Own errors: `getSituationalMusic()` override errors (`:38-39`, `:43`); `SoundManager.refreshCategoryVolume` absent (`:126`); `SoundInstance.getIdentifier()` absent (`:132`) |
| VanillaSituationalMusicOracle | 20 (26.1.2: 4) | `world.attribute` package + `EnvironmentAttributeSystem` missing (same environment-attribute cause as O03); `Music.sound()` absent; `Hud` absent (×3); `EnvironmentAttributeProbe` absent; `MusicInfo` vs `Music` type mismatch; `Minecraft.getMusicVolume()` absent |
| VanillaAmbientOracle | 15 (26.1.2: 1) | `AmbientSounds`, `AmbientMoodSettings`, `AmbientAdditionsSettings` absent; `LocalPlayer` ctor arity; `BiomeAmbientSoundsHandler` ctor arity (requires `LocalPlayer, SoundManager, BiomeManager`); `EnvironmentAttributeSystem` / `EnvironmentAttributes` absent; `SoundManager.refreshCategoryVolume`, `SoundInstance.getIdentifier()` absent |
| VanillaSoundEngineOracle | 12 (26.1.2: 4) | `world.entity.vehicle.minecart` package missing (1.21.10 keeps minecarts flat in `world.entity.vehicle`); `Minecart`, `UnderLiquidSubSound`, `UnderLiquidAmbientSoundInstance` absent; `LocalPlayer` ctor arity; `RidingMinecartSoundInstance` ctor arity; `SoundManager.updateCategoryVolume` / `refreshCategoryVolume` absent |
| VanillaFireStarterSoundOracle | 2 (26.1.2: 1) | `ServerLevel` ctor arity (`:43`) + `LocalPlayer` ctor arity (`:61`) |
| VanillaAudioAttributesOracle | 23 (26.1.2: 1) | No environment attributes in 1.21.10 (same cause as `VanillaEnvironmentOracle`); plus `Music.sound()` absent, `BackgroundMusic` / `AmbientSounds` / `AmbientAdditionsSettings` absent |
| VanillaSubtitleOracle | 12 (26.1.2: 1) | `GuiGraphicsExtractor` absent; `SoundManager` ctor arity (`:76`); six no-overrides; `Util` absent; `SubtitleOverlay.extractRenderState` absent |
| VanillaEntitySoundPacketOracle | 11 (26.1.2: 3) | `world.entity.vehicle.minecart` + `server.permissions` packages missing; `MinecraftServer` ctor arity; `isEpollEnabled()` abstract gap; four no-overrides; `Minecart` absent |
| VanillaZombieCureSoundOracle ★ | 5 (26.1.2: PASS) | `world.entity.monster.zombie` package missing (1.21.10 keeps zombies flat in `world.entity.monster`); `ZombieVillager` absent; two no-overrides; `Actor.handleEntityEvent(byte)` absent |

Only `VanillaStopSoundWireOracle` PASSES in this family.

### B10-O05 — UI/screen oracle family needs `renderpearl` + `Hud`, 26.3-only APIs (MAJOR)

REPRODUCES, much worse than 26.1.2: **11 failing oracles**.

| Oracle | Errors | Cause |
|---|---|---|
| VanillaHudOracle | 100 (26.1.2: 34) | `renderpearl.api.pipeline` package missing; `Hud` absent (×20+); `GuiGraphicsExtractor` absent (×15+); `RenderPipeline` absent; `world.entity.animal.equine` package missing; `LocalPlayer` ctor arity; `ActiveTextCollector`, `TextAlignment`, `Parameters` absent; `ExperienceBar`, `JumpableVehicleBar` absent; `BossHealthOverlay.extractRenderState`, `ChatComponent.addClientSystemMessage` / `addPlayerMessage` / `extractRenderState`, `ChatComponent.DisplayMode`, `GuiMessageTag`, `world.scores.TeamColor` absent; `model.sprite.Material` + `renderer.block.dispatch` packages missing; `BlockStateModelSet` absent |
| VanillaPackScreenOracle | 62 (26.1.2: 13) | `renderpearl.api.pipeline` package missing; `GuiGraphicsExtractor`, `RenderPipeline`, `ActiveTextCollector`, `TextInputManager`, `Parameters`, `TextAlignment` absent; `SoundManager` ctor arity; `Gui` ctor arity (requires `Minecraft`); `Language.DEFAULT_INSTANCE` absent; `Util` absent; `Screen.extractRenderState` absent; ~30 no-overrides |
| VanillaRecipeBookScreenOracle | 35 (26.1.2: 4) | `renderpearl.api.pipeline` package missing; `GuiGraphicsExtractor`, `RenderPipeline`, `TextInputManager` absent; `Capture.blit` arity; `RecipeBookTabButton` ctor arity; `CycleButton.DisplayState` absent; `CycleButton.booleanBuilder` arity; `ItemStackTemplate` absent; `RecipeBookPage` / `GhostSlots` / `EditBox` `extractRenderState` absent; `Util` absent; ~20 no-overrides |
| VanillaLoomScreenOracle | 29 (26.1.2: 5) | `renderpearl.api.pipeline` + `client.model.object.banner` + `model.sprite` packages missing; `GuiGraphicsExtractor`, `RenderPipeline`, `SpriteId`, `BannerFlagModel`, `Hud` absent; `LoomScreen` / `CrafterScreen` `extractBackground` absent; ~13 no-overrides |
| VanillaWorldBorderRenderOracle | 48 (26.1.2: 34) | `renderpearl.api.buffers` / `.commands` / `.device` / `.pipeline` packages missing; `DynamicGpuData`, `CommandEncoder`, `GpuDevice`, `GpuBuffer`, `RenderPass`, `WorldBorderRenderState`, `Hud` absent; `renderer.state.level` package missing; `WorldBorder.lerpSizeBetween` arity; `getMinX`/`getMinZ`/`getMaxX`/`getMaxZ` take no args in 1.21.10 |
| VanillaTabListOracle | 45 (26.1.2: 36) | `com.mojang.authlib.services.*` (8 imports, `:8-15` — absent from authlib 7.0.61, which 1.21.10 pins; 26.1.2 pins 7.0.63) + `renderpearl.api.pipeline` + `Hud` + `TeamColor` + `PlayerTabOverlay.extractRenderState` |
| VanillaChatScreenOracle | 41 (26.1.2: 5) | `server.permissions` package missing (×3); `PermissionSet`, `LevelBasedPermissionSet`, `PermissionLevel` absent; `ActiveTextCollector`, `ChatComponent.DisplayMode` absent; `CommandSourceStack.getAvailablePostEffects` absent; `RegistryLayer.WORLD`, `RegistryDataLoader.WORLD_REGISTRIES`, `BuiltInRegistries.DATA_COMPONENT_INITIALIZERS`, `DataComponentInitializers` absent; `world.Stopwatches` absent; `RandomSequences` / `TimerQueue` no-arg ctors absent; `world.clock.ServerClockManager` package missing; `CharacterEvent` ctor arity; `Hud` absent; `ChatAbilities` absent; `Capture.fill` arity; `Renderable` / `CommandSuggestions` `extractRenderState` absent |
| VanillaMenuOracle | 10 (26.1.2: 4) | `util.Prediction` absent (`:138`); `world.level.gamerules` package missing (×2); `DedicatedServer` ctor arity; `ServerLevel` ctor arity; `world.entity.EntityTypes` absent; `ContainerInput` absent (×2) |
| ServerPackClient | 1 (26.1.2: 1) | One no-override (`:264`). Driver outside this batch's 50 — recorded here, cross-ref Batch 11 (§8 item 5) |
| VanillaContainerScreenOracle ★ | 5 (26.1.2: PASS) | `ContainerInput` absent (×5: Chest, Hopper, Shulker, Beacon + base) |
| VanillaSimpleContainerOracle ★ | 1 (26.1.2: PASS) | `Reference<Item>.bindComponents` absent (`:25`) |

Passing in this family: `VanillaContainerTitleOracle`,
`VanillaPackSelectionOracle`, `VanillaRecipeBookViewOracle`,
`VanillaStorageMenuOracle`, `VanillaFloatTextOracle`,
`VanillaFireworkTooltipOracle`, `VanillaLoreTooltipOracle`,
`VanillaTooltipDisplayOracle`, `VanillaWorldOptionsOracle` — and,
unlike 26.1.2, `VanillaRecipeBookOracle` and
`VanillaPlaceRecipeOracle` PASS standalone (0 errors; they were
cascades via `VanillaMenuOracle` in 26.1.2 — in 1.21.10 they compile
clean and only their Batch 4/7 drivers' shared `VanillaMenuOracle`
leg is broken).

### Withdrawn oracle item

**B10-O01 (options) — WITHDRAWN, stays.** `VanillaOptionsOracle`
PASSES on the client + both classpaths in the measured table. The
1.21.7 break does not apply: `KeyMapping.getCategory()` returns
`KeyMapping$Category` in 1.21.10 (`javap-keymapping.txt`), and
`Category` is a record with `id()` returning `ResourceLocation`
(`javap-keymapping-category.txt:19`) — so the oracle's
`k.getCategory().id().getPath()` call (`VanillaOptionsOracle.java:55`)
compiles. Report-only, as in 26.1.2.

### Passing in-scope oracles (auditable coverage)

19 in-scope oracles pass on the client classpath:
`VanillaOptionsOracle`, `ParityData`, `GoatVoiceTranscript`,
`VanillaCuboidModelOracle`, `VanillaCloudRenderOracle`,
`VanillaEntityModelExporter`, `VanillaStackModelFlagsOracle`,
`VanillaStopSoundWireOracle`, `VanillaContainerTitleOracle`,
`VanillaPackSelectionOracle`, `VanillaRecipeBookViewOracle`,
`VanillaStorageMenuOracle`, `VanillaFloatTextOracle`,
`VanillaFireworkTooltipOracle`, `VanillaLoreTooltipOracle`,
`VanillaTooltipDisplayOracle`, `VanillaWorldOptionsOracle`,
`VanillaRecipeBookOracle`, `VanillaPlaceRecipeOracle` — plus
`VanillaTextDisplayTransformOracle` (0 errors) and
`VanillaCommandParseOracle` (passes; used by the `chat_screen`
driver). Shared packet-client helpers `VanillaProtocolClient` (39e)
and `SecureChatClient` (8e) fail and are recorded in §6 as Batch 0
residual, not B10 items. Cross-batch flag (not a B10 item):
`VanillaEntityParticlesOracle` FAILS with 18 errors in 1.21.10 (the
`world.entity.animal.{cow,dolphin,equine,feline,nautilus,parrot}` and
`world.entity.npc.villager` subpackages do not exist — 1.21.10 keeps
those classes flat under `world.entity.animal` / `world.entity.npc`);
it PASSED in 26.1.2. Belongs to Batch 3 — flagged, not minted (§8
item 5).

## 4. Passing checks (13)

1. **Sound-wiring diff (registry ↔ sounds.json) — PASS.** Vanilla 1.21.10
   `sounds.json` (asset index 27, sha1-verified) defines 1,770
   events; `registries.json` lists 1,771 sound events. The only
   registry event without a `sounds.json` entry is
   `minecraft:intentionally_empty` — vanilla's intentional sentinel
   (same as 1.21.7, 26.2, 26.1, 26.1.1, 26.1.2). There are ZERO
   `sounds.json` orphans (26.1.2 had one, the picky-chicken step
   event-ref — 1.21.10 has no such quirk). 1,674 entries carry
   subtitles; 7 entries have empty sound lists (the same 7
   vanilla-intentional empties as every other run: `block.fungus.fall`/
   `hit`, `entity.cod`/`salmon`/`snow_golem`/`tropical_fish` ambient,
   `music.nether.warped_forest`). The wiring spec is fully
   established; the port-side failure is the missing file (B10-G02),
   not a wiring mismatch.
2. **First-person swap ticker — PASS (B10-G05 FIXED).** Vanilla 1.21.10
   has NO swap ticker: no `itemSwapTicker` field, no
   `getItemSwapScale` (only `swapHandItems`). The Rust probe emits no
   swap-ticker columns (`first_person_probe.rs`, 471 lines — tick line
   has strength ticker/scale + hand heights only). Probe and vanilla
   agree; the 1.21.7 format mismatch does not exist on the probe side
   any more. (The oracle still emits the columns — B10-O03. Values
   still need the driver run — and the driver is oracle-blocked.)
3. **Silent-goat ambient suppression — PASS (B10-G06 WITHDRAWN, stays).**
   Vanilla 1.21.10: `Entity.playSound` checks `isSilent()` first
   (`javap -c`: `isSilent()` then `ifne` skip, both overloads,
   `javap-entity.txt:3465ff`) and `Mob.playAmbientSound` routes
   through it (`makeSound` → `playSound`), so a `Silent:1b` goat
   makes no ambient sound. The 1.21.10 port implements exactly this:
   `Mob::voice` (`horizon-client/src/mobs.rs:4668`) draws the pitch
   first ("Java evaluates the arguments … before the call") then
   returns early `if self.silent` (`:4673`), `play_ambient` routes
   through `voice()` (`mobs/ai.rs:1480`), and a unit test pins it
   (`mobs.rs:6513`, `a_silent_mob_makes_no_ambient_sound`). The
   `test_goat_voices_parity.py:4` docstring ("Silent ambient
   suppression is a separate gap") is STALE for 1.21.10 — the driver
   still gates those two controls out (`:144-147`), so the suppression
   is unmeasured by the driver, but the code is there. Driver-hygiene
   follow-up (§8), not a behaviour gap.
4. **Sign items — PASS (no divergence).** Vanilla renders sign items
   as flat generated sprites (`items/oak_sign.json` →
   `minecraft:model` → `item/oak_sign` → `item/generated`; same for
   hanging signs), same as the port. (1.21.10's jar DOES contain
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
   Everything the options oracle touches exists in 1.21.10:
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
   vanilla throughout (all javap `-c`, fresh, 1.21.10 server jar):
   beehive — `block.beehive.shear` at the block centre, 1.0/1.0,
   `minecraft:shear` game event at the target pos, honeycomb drops,
   bees released, honey reset (`dispenser.rs:1792-1824`); entity
   shear — per-species sound at the mob's feet, 1.0/1.0
   (`entity.sheep.shear`, `entity.bogged.shear`,
   `entity.snow_golem.shear`, `entity.copper_golem.shear`,
   `entity.mooshroom.shear` — vanilla plays each via
   `playSound(null, this, SOUND, source, 1.0, 1.0)`; all five
   `*_SHEAR` events javap-verified in the species classes),
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
   1.21.10 entity-model layers. No B10 defect; the Batch 3 species
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
    1.21.10 has no baby guard and the port matches it (full evidence
    in §2, B10-G07).
13. **Item-definition census — PASS (NEW check; spec established).**
    All 1,488 bundled item definitions extracted from the client jar
    (`evidence/jar-assets/assets/minecraft/items/`) and counted:
    model types `minecraft:model` 1,348, `minecraft:special` 67,
    `minecraft:select` 64, `minecraft:condition` 7,
    `minecraft:range_dispatch` 2 — there is NO `composite` type in
    1.21.10 (a 26.x form; this is why 1.21.10's bed items are single
    specials, G03). Special types: shulker_box 17, banner 16, bed 16,
    chest 9, head 6, conduit 1, decorated_pot 1, player_head 1 —
    standing_sign and hanging_sign appear in ZERO bundled definitions.
    Tint sources: dye 50, constant 12, grass 6, potion 4, map_color 1
    (`filled_map.json` only), firework 1. The census pins the exact
    blast radius of the two decode gaps (G03: 16 bed items; G10: 1
    filled map) and confirms no other bundled definition hits an
    undecodable type. The port's `DATA_COMPONENT_TYPES` list is
    exactly 1.21.10's 96 components (verified 96/96 against the
    registry, zero drift either way).

**Not a check — Checkbox row-limit tooltip (26.1.2 §4 check 10) is N/A
for 1.21.10.** Vanilla 1.21.10's `Checkbox` has NO `ROWS` /
`overflowsRowLimit` / `setMaxRows` (grep of `javap-checkbox.txt` +
`javap-checkbox-builder.txt` empty) — the row-limit tooltip was a
26.1.2 change. The 1.21.10 tree carries no
`checkbox_overflows_row_limit` helper (grep of the whole client
source: the only `max_rows` is an unrelated tooltip truncation in
`ui_theme.rs:634`). There is nothing to port and nothing to diverge;
the check is not counted.

## 5. Re-check verdicts

### 1.21.7 re-checks (PRIMARY — all 11 decided)

| 1.21.7 item | Verdict for 1.21.10 | 1.21.10 item |
|---|---|---|
| B10-G01 sign boards (major/gap) | FIXED — the port draws boards (§2) | B10-G01 (fixed) |
| B10-G02 sounds.json (major/gap) | REPRODUCES | B10-G02 (gap) |
| B10-G03 beds placed + items (major/gap) | REPRODUCES | B10-G03 (gap) |
| B10-G04 five BE renderers (minor/gap) | NEVER EXISTED — all five have port equivalents (§2); also wrong for the 1.21.7 tree itself (§8 item 3) | B10-G04 (closed) |
| B10-G05 swap-ticker columns (minor/gap) | FIXED (probe side) — the probe emits no swap columns; oracle residue is B10-O03 (§2) | B10-G05 (fixed) |
| B10-G06 silent-goat suppression (minor/gap) | WITHDRAWN (stays) | Report-only (§4 check 3) |
| B10-O01 options (minor/oracle-defect) | WITHDRAWN (stays) | Report-only (§3, §4 check 7) |
| B10-O02 ParityFont (major in 1.21.7) | REPRODUCES | B10-O02 (oracle-defect) — 2 errors now (added `TextRenderable.Styled`) |
| B10-O03 render/model (major) | REPRODUCES | B10-O03 (oracle-defect) — 21 failing oracles, far worse than 1.21.7 |
| B10-O04 sound (major) | REPRODUCES | B10-O04 (oracle-defect) — 11 failing oracles |
| B10-O05 UI/screen (major) | REPRODUCES | B10-O05 (oracle-defect) — 11 failing oracles |

6 REPRODUCES, 2 FIXED, 1 NEVER EXISTED, 2 WITHDRAWN, 0 UNVERIFIED.

### 26.1.2 re-checks (SECONDARY — all 17 decided)

| 26.1.2 item | Verdict for 1.21.10 | 1.21.10 item |
|---|---|---|
| B10-G01 sign boards (major/gap) | FIXED — the 1.21.11 fix carried into the template-derived port (§2) | B10-G01 (fixed) |
| B10-G02 sounds.json (major/gap) | REPRODUCES | B10-G02 (gap) |
| B10-G03 beds placed + items (major/gap) | REPRODUCES | B10-G03 (gap); item form differs (single special, not composite) |
| B10-G04 seven BE renderers (minor/closed) | NEVER EXISTED (stays closed) | B10-G04 (closed) |
| B10-G07 baby zombie arms (minor/gap) | NEVER EXISTED — vanilla 1.21.10 has no baby guard; the port matches (§2) | B10-G07 (closed) |
| B10-G08 SwingState (minor/gap) | REPRODUCES | B10-G08 (gap); 1.21.10 differs further (no held-item swing animation at all) |
| B10-G09 pot patterns (minor/gap) | REPRODUCES | B10-G09 (gap) |
| B10-G10 filled map (minor/gap) | REPRODUCES | B10-G10 (gap); component list is exact here (no latent drift) |
| B10-G11 swing packet (minor/gap) | REPRODUCES | B10-G11 (gap) |
| B10-G12 wax-on sound (minor/gap) | REPRODUCES | B10-G12 (gap) |
| B10-G13 potion-break sound (minor/gap) | REPRODUCES | B10-G13 (gap) |
| B10-G14 digging sound (minor/gap) | REPRODUCES | B10-G14 (gap) |
| B10-G15 sound categories (minor/gap) | REPRODUCES | B10-G15 (gap) |
| B10-O02 ParityFont (minor/oracle-defect) | REPRODUCES | B10-O02 (oracle-defect) — 2 errors, not 1 |
| B10-O03 render/model oracles (major/oracle-defect) | REPRODUCES | B10-O03 (oracle-defect) — 21 failing, 5 newly failing vs 26.1.2 |
| B10-O04 sound oracles (major/oracle-defect) | REPRODUCES | B10-O04 (oracle-defect) — 11 failing; SoundRig/Music now own-error, not cascade |
| B10-O05 UI/screen oracles (major/oracle-defect) | REPRODUCES | B10-O05 (oracle-defect) — 11 failing; ContainerScreen/SimpleContainer newly failing |

13 REPRODUCES, 1 FIXED, 2 NEVER EXISTED, 0 WITHDRAWN, 0 UNVERIFIED.
The 26.1.2 withdrawals stay withdrawn (B10-G05 swap ticker, B10-G06
silent goat, B10-O01 options — §4 checks 2, 3, 7); note B10-G05 is
withdrawn for 26.1.2 (vanilla 26.1.2 HAS the ticker) but FIXED for
1.21.10 (vanilla 1.21.10 lacks it and the probe no longer emits it) —
the two verdicts are consistent because the vanilla spec differs.

## 6. Per-driver status (all 50, measured)

Oracle status from this batch's Batch 0-method measurement
(`b10/evidence/measure/results.json`); "fail" = fails on the
classpath the driver's path uses. `cp` is that classpath
(oracle_harness → both, client jar → client, server jar → server,
no-flag → both superset). Driver→oracle map from
`b10/evidence/measure/drivers.json` (648 test files mapped from the
1.21.10 tree). Full per-driver detail in
`b10/evidence/driver-table.txt`.

### Runnable by javac — NOT RUN (9)

| Driver | Oracle(s) | Note |
|---|---|---|
| cloud_render | VanillaCloudRenderOracle pass | Unmeasured |
| container_title | ParityData + VanillaContainerTitleOracle pass | Unmeasured |
| cuboid_model | VanillaCuboidModelOracle pass | Unmeasured |
| firework_tooltip | VanillaFireworkTooltipOracle pass | Unmeasured |
| lore_tooltip | VanillaLoreTooltipOracle pass | Unmeasured |
| options | VanillaOptionsOracle pass | Unmeasured |
| stop_sound_wire | VanillaStopSoundWireOracle pass | Unmeasured |
| tooltip_display | VanillaTooltipDisplayOracle pass | Unmeasured |
| world_options | VanillaWorldOptionsOracle pass | Unmeasured |

Seven drivers runnable in 26.1.2 are blocked for 1.21.10
(blockstate_model, container_screen, mipmap, model_choice,
model_resolution, weather_render, zombie_cure_sound — below). Unlike
26.2, NO runnable driver is data-gated by B10-G02: all 6
`audio_harness` drivers are oracle-blocked for 1.21.10 (below), so
the sounds.json gate never gets reached.

### Oracle-blocked (38)

| Driver | Blocking oracle(s) | Item |
|---|---|---|
| ambient_sound | VanillaAmbientOracle 15e (AmbientSounds + LocalPlayer ctor) + VanillaSoundRig 3e (own errors) | B10-O04 |
| arrow_hit_sound | VanillaEntitySoundPacketOracle 11e (minecart package + MinecraftServer ctor) | B10-O04 |
| audio_attributes | VanillaAudioAttributesOracle 23e (EnvironmentAttributeSystem) | B10-O04 |
| block_entity_render | ParityFont 2e + VanillaBlockEntityRenderOracle 42e (model packages + UvMapping + sign states) + VanillaTextDisplayRenderOracle 8e | B10-O02, B10-O03 |
| blockstate_model | VanillaBlockStateModelOracle 13e (dispatch package; CuboidModel passes) | B10-O03 |
| breaking_model | VanillaBreakingModelOracle 11e (BlockModelFeatureRenderer) + VanillaSectionMeshOracle 48e (MaterialBaker + model packages) | B10-O03 |
| chat_screen | ParityFont 2e + VanillaChatScreenOracle 41e (permissions + Hud) + VanillaPackScreenOracle 62e (renderpearl) | B10-O02, B10-O05 |
| container_screen | VanillaContainerScreenOracle 5e (ContainerInput) | B10-O05 |
| dragon_death_roar | VanillaProtocolClient 39e (packet-client helper) | Batch 0 residual |
| dragon_render | VanillaDragonRenderOracle 2e (no-override) | B10-O03 |
| entity_bound_sound | VanillaSoundEngineOracle 12e (minecart package) + VanillaSoundEventOracle 3e + VanillaSoundRig 3e | B10-O04 |
| entity_model | VanillaEntityAnimOracle 8e (illager package + swing fields + rotateDegrees) | B10-O03 |
| entity_sound_packet | VanillaEntitySoundPacketOracle 11e (minecart package + MinecraftServer ctor) | B10-O04 |
| environment_render | VanillaEnvironmentOracle 18e (EnvironmentAttributeSystem) | B10-O03 |
| fire_starter_sound | VanillaFireStarterSoundOracle 2e (ServerLevel + LocalPlayer ctors) | B10-O04 |
| first_person | VanillaFirstPersonOracle 25e (FirstPersonHandsAndItems + SwingState + LocalPlayer ctor + swap-ticker columns) | B10-O03 |
| goat_voices | VanillaProtocolClient 39e (packet-client helper) | Batch 0 residual |
| hud | ParityFont 2e + VanillaHudOracle 100e (renderpearl + Hud + ExperienceBar) | B10-O02, B10-O05 |
| item_model | VanillaItemModelOracle 45e (model packages + EntityTypes + Selector + MaterialBaker, …) | B10-O03 |
| menu_click | VanillaMenuOracle 10e (Prediction + gamerules + DedicatedServer ctor + EntityTypes + ContainerInput) | B10-O05 |
| mipmap | VanillaMipmapOracle 5e (MipmapStrategy + computeTransparency) | B10-O03 |
| model_baking | VanillaModelBakingOracle 20e (model packages + FaceBakery; CuboidModel passes) | B10-O03 |
| model_choice | VanillaModelChoiceOracle 11e (model.sprite + BlockStateModel; CuboidModel passes) | B10-O03 |
| model_resolution | VanillaModelResolutionOracle 8e (model packages; CuboidModel passes) | B10-O03 |
| music | VanillaMusicOracle 5e (own errors) + VanillaSoundRig 3e (own errors) | B10-O04 |
| pack_screen | ParityFont 2e + VanillaPackScreenOracle 62e (renderpearl + Gui ctor + Language) | B10-O02, B10-O05 |
| recipe_book_render | ParityFont 2e + VanillaRecipeBookScreenOracle 35e (renderpearl) | B10-O02, B10-O05 |
| screen_layout | VanillaLoomScreenOracle 29e (renderpearl + Hud) | B10-O05 |
| secure_chat | SecureChatClient 8e (packet-client helper) | Batch 0 residual |
| situational_music | VanillaSituationalMusicOracle 20e (environment attributes + Hud) | B10-O04 |
| sound_engine | VanillaSoundEngineOracle 12e (minecart package) + VanillaSoundRig 3e | B10-O04 |
| sound_event | VanillaSoundEventOracle 3e (SoundManager ctor + getOrResolve + getAttenuationDistance) | B10-O04 |
| subtitle | ParityFont 2e + VanillaSubtitleOracle 12e (SoundManager ctor + GuiGraphicsExtractor) | B10-O02, B10-O04 |
| text_display_render | ParityFont 2e + VanillaTextDisplayRenderOracle 8e (ServerLevel ctor + GameRenderState) | B10-O02, B10-O03 |
| title_debug | VanillaTitleDebugOracle 4e (Gui.canInterruptScreen) | B10-O03 |
| weather_render | VanillaWeatherRenderOracle 1e (RandomSource.createThreadLocalInstance) | B10-O03 |
| world_border_render | VanillaWorldBorderRenderOracle 48e (renderpearl + Hud + DynamicGpuData) | B10-O05 |
| zombie_cure_sound | VanillaZombieCureSoundOracle 5e (monster.zombie package) | B10-O04 |

Note: `dragon_death_roar`, `goat_voices` and `secure_chat` are
server-e2e drivers whose Java side is a packet client, not a vanilla
oracle — the 1.21.7 report grouped them as "no oracle", but the
driver map shows they compile a helper (`VanillaProtocolClient` /
`SecureChatClient`) that currently fails, so they are counted blocked
here with that qualification (same treatment as 26.2/26.1/26.1.1/
26.1.2). `VanillaProtocolClient`'s 39 errors are mostly the `x()`/`z()`
→ `getX()`/`getZ()` packet-accessor renames (1.21.10 keeps the `get-`
forms) plus `ClientboundUpdateTagsPacket.tags()` → `getTags()`,
`ClientboundSetTimePacket.clockUpdates()` absent,
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
1.21.10, so the probe confirms it on screen rather than discovering
it.

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
  the host's 2020 events (B10-G14). Against a vanilla 1.21.10 host,
  the port is expected to go silent after the first hit.
- **P8 — Undecodable items' on-screen fallback (confirmation).** Hold
  a red bed and a filled map: vanilla renders both; the port's
  `ClientItem` decode fails for both (B10-G03, B10-G10) and the code
  path ends at the missing model (`item_model.rs:1102`) — confirm
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

## 8. Follow-ups and questions for the coordinator

1. **`horizon-prepare sounds` for 1.21.10 (highest value, one step).**
   Fixes B10-G02 at runtime — but unlike 26.2 it unblocks NO audio
   driver on its own, because all 6 are oracle-blocked too (B10-O04).
   The sounds step and the sound-oracle repairs are both needed
   before any audio driver measures anything.
2. **Oracle repair order for Batch 10** (when repair resumes):
   `ParityFont` first (2 errors, 8 drivers), then the `LocalPlayer`
   ctor group (rescues `ambient_sound`, `entity_bound_sound`,
   `sound_engine`, `fire_starter_sound` in this batch, and is one of
   `VanillaFirstPersonOracle`'s and `VanillaHudOracle`'s errors),
   then `VanillaSoundEventOracle` (3 errors — rescues `sound_event`
   and the rig/music legs), `VanillaPackResourcesOracle` (1 error),
   `VanillaWeatherRenderOracle` (1 error),
   `VanillaSimpleContainerOracle` (1 error),
   `VanillaTitleDebugOracle` (4 errors, one missing method),
   `VanillaDragonRenderOracle` (2 errors) +
   `VanillaTextDisplayOracle` (2 errors) +
   `VanillaFireStarterSoundOracle` (2 errors),
   `VanillaContainerScreenOracle` (5 errors, one missing class),
   `VanillaMipmapOracle` (5 errors),
   `VanillaZombieCureSoundOracle` (5 errors, one missing package),
   `VanillaMenuOracle` (10 errors, mechanical),
   `VanillaBlockStateModelOracle` / `VanillaModelChoiceOracle` /
   `VanillaModelResolutionOracle` (13/11/8 errors — the 1.21.10 model
   packages need the 1.21.10-form backport, not the 26.x form),
   then `VanillaBlockEntityRenderOracle` (42 errors — the
   `block_entity_render` driver is the highest-value single repair:
   it is the check that would verify the G01 sign-board fix and the
   G04 geometry equivalents), `VanillaItemModelOracle` (45 errors),
   `VanillaSectionMeshOracle` (48 errors) and
   `VanillaFirstPersonOracle` (25 errors — its SwingState half needs
   the 1.21.10 plain-field form, see B10-G08, and its swap-ticker
   columns must be dropped as the 1.21.7 repair did), then the
   `VanillaEntitySoundPacketOracle` / `VanillaProtocolClient` /
   `SecureChatClient` packet-shape translations, the environment-
   attribute family (`VanillaEnvironmentOracle`,
   `VanillaAudioAttributesOracle`, `VanillaSituationalMusicOracle` —
   1.21.10 has no environment attributes at all, so these need a
   1.21.10-form rewrite, not a patch), and last the renderpearl/`Hud`
   backports (deepest — B10-O05).
3. **1.21.7-record correction — B10-G04 never existed.** The 1.21.7
   ledger's B10-G04 ("Five more vanilla block-entity renderers have
   no port equivalent") is wrong: all five have port equivalents in
   `game.rs` geometry functions in the 1.21.7 tree itself
   (`versions/1.21.7/.../game.rs:6778/6781/6782/7265`,
   `creatures.rs:8137`, verified 2026-11-25), so the item was wrong
   for 1.21.7 too — not just for 26.1/26.1.1/26.1.2 (where the 26.1.2
   run already corrected it). The 1.21.7 ledger should carry the
   correction. The lesson for future runs: a "no port equivalent"
   claim needs a search over the whole render path (`game.rs`
   geometry calls), not just the `Kind` enum — the BE renderer file
   is not where this port draws half its block entities.
4. **Goat-voices docstring** (B10-G06 note): the driver's "Silent
   ambient suppression is a separate gap" docstring is stale for
   1.21.10 (suppression is implemented + unit-tested) — driver-hygiene
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
   (Batch 4/7 — both PASS standalone in 1.21.10),
   `ServerPackClient` (Batch 11? — needs assignment),
   `VanillaSpriteAnimationOracle` / `VanillaTranslucencyOracle` /
   `VanillaTabListOracle` / `VanillaPackResourcesOracle` (drivers
   outside this batch's 50 but squarely render/UI family — no other
   batch brief names them), `VanillaEntityParticlesOracle` (Batch 3 —
   FAILS 18e in 1.21.10, passed in 26.1.2).
   `VanillaProtocolClient` / `SecureChatClient` are shared helpers
   recorded as Batch 0 residual, not B10 items.
6. **Batch 0 table landing.** This batch's oracle measurement
   (`b10/evidence/measure/results.json`) is a scoped re-measurement
   run because the Batch 0 table had not landed. When the real Batch 0
   table lands, diff it against this measurement: any difference in
   the 67 in-scope oracles changes §3/§6. The measurement used the
   exact Batch 0 method, so the two should agree — a disagreement
   would be a finding, not a rounding error. The provisional-triage
   fallback in common.md should NOT be used for 1.21.10: 42/67
   in-scope oracles deviate from the 26.1.2 table.
7. **26.3 leaks spotted in the 1.21.10 tree (cross-refs, NOT B10
   items).** `entity.sulfur_cube.eject` in `game/dispenser.rs:1935`
   (0 "sulfur" occurrences in 1.21.10's `sounds.json` — the sulfur
   cube does not exist in 1.21.10); `block.potent_sulfur.noxious_gas`
   special-cased in `settings.rs` `sound_source`;
   `minecraft:potent_sulfur` in
   `horizon-client-core/src/block_entity_nbt.rs:364` / `:737` and
   `horizon-client-render/src/chunk/block_entity.rs:104`. For
   Batch 3–6 / the leakage audit, not this batch.
8. **Method note for future batches:** the measured table is the only
   oracle authority — this batch verified every oracle claim against
   its own measurement plus fresh javap, and found no case where the
   measurement was wrong. An import-level check would have missed the
   method-level breaks again (notably `VanillaSoundEventOracle`,
   `VanillaSubtitleOracle`, `VanillaChatScreenOracle`,
   `VanillaWeatherRenderOracle`, `VanillaMipmapOracle`, and the
   `MaterialBaker`-is-an-interface cases). And a "no port equivalent"
   claim needs a whole-render-path search, not one file (§8 item 3).
   Finally: do not carry a previous version's oracle table forward as
   provisional triage for a different era — 1.21.10 vs 26.1.2 differs
   on 42 of 67 in-scope oracles.

## 9. Verdict

Batch 10 for 1.21.10 is complete: 24 behaviour checks resolved
statically (13 pass, 11 gap), 18 items minted (10 behaviour gaps,
2 fixed, 2 closed, 4 oracle defects), all 50 in-scope drivers triaged
from this batch's Batch 0-method measurement (9 runnable-not-run, 38
oracle-blocked, 3 no-oracle), 11 in-game probes specified and not run.
Six of the 1.21.7 run's 11 items reproduce with fresh 1.21.10 evidence;
two are fixed (B10-G01 sign boards — the headline, the first version
in the sweep where placed signs render their boards — and B10-G05, the
probe half of the swap-ticker columns); one, B10-G04, never existed
(all five "missing" block-entity renderers have port equivalents in
`game.rs` geometry functions, a correction that also applies to the
1.21.7 ledger itself); two stay withdrawn. Thirteen of the 26.1.2
run's 17 items reproduce; G01 is fixed there too, and G07 never
existed for 1.21.10 (vanilla 1.21.10 has no baby guard). The headline
defects that remain are the familiar ones — beds are invisible (and
their items fail to decode too, now proven to fall back to the missing
model), the game has no `sounds.json` — with the same sound-side
cluster behind them: three level-event sounds lost to 26.3-form splits
(wax-on, potion break, digging), item-use swings invisible to other
players, and sound categories derived from event names instead of call
sites. The oracle picture is the worst of any run yet: 44 in-scope
oracles fail, including five that passed in 26.1.2, and only 9 of 50
drivers are runnable by javac. Nothing in this batch was fixed, no
production code was touched, no driver was run, and no git command
was run in the worktree.

Evidence index (all under
`~/workspace/mcir-work/sweep-1.21.10-batches/b10/evidence/`, plus the
deliverables in `b10/`):

- `client-classes.txt`, `server-classes.txt` — full jar listings
  (9,969 / 6,964 classes).
- `jar-assets/` — sign/bed/spawner/campfire/vault/shelf/
  piston/brushable models + blockstates + all 1,488 item definitions
  extracted from the client jar.
- `measure/` — the Batch 0-method oracle measurement: `results.json`
  (391 files × 3 classpaths), `drivers.json` (648 test files),
  `big-compile-{server,client,both}.log`.
- `measure-oracles-b10.py`, `fetch-libs.py`, `oracle-detail.py`,
  `oracle-status.py`, `driver-table.py`, `census-sounds.py` — the
  measurement and analysis scripts.
- `oracle-errors-client.txt` — full per-oracle error detail (symbols
  and locations) for all in-scope oracles, extracted from the client
  big-compile log.
- `driver-table.txt` — per-driver status for all 50 in-scope drivers.
- `census-sounds.txt` — item-definition census + sounds wiring diff.
- `javap-ber.txt`, `javap-signs-beds.txt`,
  `javap-specialmodelrenderers.txt`, `javap-be-type.txt` — BE
  renderers, sign/bed renderers, special-model bootstrap (G01/G03/G04).
- `javap-animationutils.txt`, `javap-zombiemodel.txt`,
  `javap-livingentity-server.txt`, `javap-livingentity-client.txt`,
  `javap-player-server.txt`, `javap-player-client.txt`,
  `javap-localplayer.txt`, `javap-sgpli.txt`, `javap-mpgm.txt`,
  `javap-swingsource.txt`, `javap-interactionresult.txt` — swing
  state machine (G08), swing packet (G11), zombie arms (G07),
  digging (G14), swap ticker (G05).
- `javap-potpatterns.txt`, `javap-potdecorations.txt` —
  `DecoratedPotPatterns` (G09).
- `javap-shear.txt`, `javap-shear-*.txt`, `javap-entity.txt`,
  `javap-mob.txt` — dispenser shear (§4 check 8) and the shear sound
  categories (G15), silent-goat suppression (G06).
- `javap-leveleventhandler.txt` — the 79-case lookupswitch, 3003 /
  2002-2007 cases, no 1053/1054, no 2019/2020 (G12/G13/G14).
- `javap-potions.txt`, `javap-honeycomb.txt` — vanilla server sends
  only 2002/2007 and 3003, no 1053/1054, no direct wax sound
  (G12/G13).
- `javap-checkbox.txt`, `javap-checkbox-builder.txt` — Checkbox has
  no row-limit feature in 1.21.10 (§4 note).
- `javap-keymapping.txt`, `javap-keymapping-category.txt`,
  `javap-options.txt`, `javap-packresources.txt`,
  `javap-packresources-iface.txt`, `javap-font2.txt`, `javap-gui.txt`,
  `javap-resourcemanager.txt` — options API (O01), ParityFont cause
  (O02), Font/Gui oracle causes (O03/O05).
- `javap-report-screen.txt`, `javap-signtext.txt`,
  `javap-dialog-input.txt`, `javap-dialog-conn.txt`,
  `javap-banner.txt`, `javap-modelfeature.txt`, `javap-cuboid.txt`,
  `javap-submitnode.txt`, `javap-materialinfo.txt`,
  `javap-posestack.txt`, `javap-language.txt`, `javap-util.txt`,
  `javap-simplesound.txt`, `javap-sound.txt`,
  `javap-berprovider.txt`, `javap-berdispatcher.txt`,
  `javap-iteminhand.txt`, `javap-blockstatebase.txt`,
  `javap-blockstate.txt`, `javap-level.txt`,
  `javap-serverlevel.txt`, `javap-mcserver.txt`,
  `javap-dedicated.txt`, `javap-bundle.txt`,
  `javap-resourcemanager-server.txt` — supporting javap for oracle
  causes and vanilla API shapes.
- `1.21.10.json`, `asset-index-27.json`, `sounds-1.21.10.json`,
  `version_manifest_v2.json` — Mojang version/asset files for the
  wiring diff only; must NOT be committed.
- `libs/` — sha1-verified Mojang 1.21.10 libraries used for the
  measurement (71 client + 44 server jars); must NOT be committed.
- `classes/`, `sclasses/` — extracted jar classes for javap; must
  NOT be committed.
