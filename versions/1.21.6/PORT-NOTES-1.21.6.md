# Port notes: Minecraft 1.21.6

1.21.6 is the fifth adjacent delta port in the **Components era (E6,
1.20.5–1.21.11)**, taken from 1.21.7. It is a cross-protocol delta:
protocol **772 → 771**, world version 4438 → 4435, data pack 81.0 →
80.0 and resource pack 64.0 → 63.0 — but the packet set is
**byte-identical** between the two releases (0 added, 0 removed,
0 renumbered), so there is no packet rewire: the cross-protocol work
is the adapter re-base to 771 alone. The measured delta is **31
ranked work items**, all closed by the behaviour port below
(2026-11-21). The port is the mechanical layer plus the behaviour
port: the pin is set, the six generated tables are regenerated
against 1.21.6's own jar, the adapter ladder is re-based to the new
protocol, the verification gates pass, and the behaviour diff is
ported (see "Behaviour port").

## Provenance

- Base: the programme's completed 1.21.7 tree — the `port/1.21.7`
  branch at 9128f75, which contains the 1.21.7 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle port),
  the jsr305 jar, the both-forms `gen_version_consts.py`, the
  jsr305 classpath fix in `tools/port_regen.py`, and the full
  ported oracle (391 files). No overlay was needed: the branch is
  the completed tree, unlike the bare `port/1.21.8` branch the
  1.21.7 agent had to overlay. Published 1.21.7: main@be65dca7.
- 1.21.6's facts, from its own version JSON and generated tables:
  protocol **771**, world (data) version **4435**, data pack
  format **80.0**, resource pack format **63.0**, Java **21**,
  stable release (2025-06-17).
- Prepare was salvaged from the 2026-11-13 agent's partial work
  (closed on the owner's redirect, nothing published): the
  mappings front-end run was complete and usable (verified jar
  download + generalized mappings front-end remap + datagen):
  **9,483/9,483 classes mapped (100%), 9,424 renamed,
  0 placeholders; 83,775 methods mapped (62,737 renamed), 42,840
  fields mapped (42,403 renamed)**. The diff log was empty (the
  diff never ran), so the diff and report were re-run from the
  salvaged prepared data. Nothing Mojang-derived is committed in
  this tree.
- Diff 1.21.7 → 1.21.6: **31 ranked work items** (rank 1: 3,
  rank 2: 17, rank 3: 10, rank 5: 1). Classes +0 −1, 0 moved,
  **45 changed (73 methods); 29 concern Horizon**.

## The delta: a cross-protocol pin step

Rank 1 is the version pin (`version.rs`), the save-format item
(DataFixers 4438 → 4435: 0 schemas, 0 fixes added, 0 datafix
classes changed) and the version facts (protocol 772 → 771, world
4438 → 4435). Rank 2 is 17 harness reruns. Rank 3 is 10 unproved
items. Rank 5 is the 17 changed or removed vanilla classes Horizon
does not port, in 12 packages.

The visible content change is the removal of the **Lava Chicken**
music disc, a 1.21.7 addition: the registry loses the item
`minecraft:music_disc_lava_chicken` and the sound event
`minecraft:music_disc.lava_chicken`. Every item id above the
disc's slot shifts down by one, which is why the adapter re-base
and the test re-pins below are mechanical but wide. There is no
other registry, block, item or command change of note.

## Tables: 2 regenerated, 4 reproduce

| Generator | Result vs 1.21.7 |
|---|---|
| `version_consts` | regenerated (+8 −8 — the pin: 1.21.6 / 771 / 4435 / 80.0 / 63.0) |
| `packet_ids` | regenerated (+1 −1 — header comment only; the packet set is identical) |
| `entity_tracking` | unchanged |
| `lan_command_order` | unchanged |
| `command_suggestions` | unchanged |
| `vanilla_command_order` | unchanged |

The committed `regen.json` lists all six generators (D1 completeness,
per the gate change at main@15ff93a8): the final full regen ran with
no `--only` (2026-11-19, commit 7aaa91c49b). A fresh regen reproduces
all six committed tables byte-for-byte — see
`regen-determinism.json`, no overrides. (The brief expected the five
SlotDisplay hand-corrected tables to differ from a fresh regen; they
do not — the generators emit the same numbering the hand corrections
carry, as they did for 1.21.9, 1.21.8 and 1.21.7.)

Two traps from the regen, for the next agent:

- The first regen run was SIGTERM'd at ~37 min during
  `vanilla_command_order`'s client_probe build (a VM restart). The
  retry completed in ~13.6 min. Regen is idempotent; a killed run
  is safe to re-run.
- `version_port.py regen --worktree` takes the path as given: a
  **relative** worktree path breaks the vineflower jar lookup in
  `decompile_classes` (the command runs with cwd=root, so the
  relative jar path resolves inside the worktree). Always pass the
  **absolute** worktree path. The determinism regen hit this and
  failed in seconds; the absolute-path retry ran clean.

## Adapter re-base to 771

The adapter ladder (`crates/horizon-protocol/src/adapter/`) is
re-based from the 772 pin to 771 (commits 881498f, a0b30b3 on
`port/1.21.6`; scripts `rebase771.py` and `fix_slot_display771.py`
in the Stream 1 workdir, modelled on the 1.21.8 `rebase772.py`).
The rebase was recovered from diffing the regenerated tables
against the 772 tree — the committed `rebase772.py` is an
incomplete record of the 1.21.8 re-base, so the 771 maps were
re-derived rather than copied:

- INHERIT map: 1.20.2→1.20.3, 1.20.3→1.20.6, 1.20.6→1.21.1,
  1.21.1→1.21.3, 1.21.3→1.21.4, 1.21.4→1.21.5, 1.21.5→PIN,
  1.21.8→PIN; NAME_ALIASES for 1.20.2/1.20.3 (needed by
  `adapter/legacy.rs`).
- Renames: attributes (1.20.2: 14, 1.20.3: 14, 1.20.6: 22,
  1.21.1: 31 — 1.21.2 dropped the prefixes); items
  (grass→short_grass, scute→turtle_scute); block
  (grass→short_grass); entities (boat→oak_boat,
  chest_boat→oak_chest_boat, potion→splash_potion); particle
  (ambient_entity_effect→entity_effect); component
  (fire_resistant→damage_resistant); sound
  (entity.generic.wind_burst→entity.breeze.wind_burst).
- REGISTRY_REASSIGN (1.21.3 creaking_transient→creaking);
  STATE_ALIASES (1.21.3, nine creaking_heart aliases).
- Two DOWN hand-patches the generator cannot produce:
  v1_20_3 ENTITY_TYPE_DOWN insert (18, 113, 1) (breeze_wind_charge
  → wind_charge); v1_21_3 BLOCK_STATE_DOWN (2054, 2051, 866)→867
  + eight heart runs length 1→2.
- SlotDisplay: 8 entries in both 1.21.6 and 1.21.7, same order —
  the 772 fix values apply unchanged.
- EntityDataSerializers are NOT among the 45 changed classes, so
  the `adapter.rs` serializer remaps are unchanged. `adapter.rs`
  hand edits: "pinned 1.21.8" → "pinned 1.21.6" (5 comment sites),
  the V1_21_8/V1_21_6 static comments rewritten, the
  `from_protocol` test updated (772→V1_21_8, 771→Current).

The Lava Chicken removal shifts every item id above the disc's slot
down by one, so the protocol tests needed re-pins: the sentry trim
template 1354→1353 (×4), context item ids >1256 shifted −1, the
angler sherd 1372→1371 (incl. varint bytes 0xdc→0xdb), the blast
furnace station `SlotDisplay::Item(1291)` → `Item(1290)`, and four
pinned slot fixtures in `item/v1_20_3.rs` (crossbow 1274→1273,
goat_horn 1287→1286, stew 1275→1274, bees 1307→1306). The two
lava-chicken disc tests were rewritten from the 1.21.8 side
(pigstep 1267→1266, levelup sound 1180→1179, `up(1256)==None`,
`down(1256)==Some(1257)`).

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (15 m 21 s at `-j 1`). The first run at `-j 2` was
  OOM-killed at ~43 min on the horizon-client lib rustc — the
  VM's 6 GB, shared with two other streams' concurrent rustc
  processes, cannot hold two heavy codegen jobs at once.
- `cargo test -p horizon-protocol`: **430 pass / 2 fail** (432 total —
  identical to 1.21.7/1.21.8, as expected: the packet set is
  byte-identical across the delta). The 2 failures are the same
  pre-existing sandbox-network pair in untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope.
- `horizon-prepare` completes end-to-end: exit 0. **No oracle
  compile errors** — the 1.21.7 oracles compile cleanly against
  the 1.21.6 jars with zero fixes (unlike 1.21.8, which needed
  7 files / 27 javac errors ported).

## Harness subset: no local profile (blocked, CI to measure)

No local harness profile was measured. The sweep could not
complete on this VM, and the profile is to be measured by the CI
`port-verify` workflow on GitHub runners (16 GB) after merge.

- Sweep attempt 1: prepare ok, but the sounds step failed writing
  a sounds asset object (disk quota; 432M already in
  `versions/1.21.6/launcher`), and parity died after the first
  harness saving `state.json` — "Disk quota exceeded". Fixed by
  deleting the candidate's `target/` (4.8G debug build; the check
  and tests were already recorded).
- Sweep attempt 2: prepare ok; parity recorded **9 FAILs** in
  `.local/parity-runner/state.json` (incl. `active_effect_tick`,
  `active_effects`, `ageable_integer_load` — all expected for
  unported behaviour). The 4th harness (`allay_cooldown`) builds
  the client (`cargo build --profile fast -p horizon-client --bin
  horizon-client`); the build hit parity's 3600s per-harness
  timeout and was killed. The exec then ended exit 120 at ~69 min
  — no code path in `version_port.py`/`parity.py` produces 120;
  the most likely cause is a runtime turn-boundary kill (a
  follow-up message arrived ~6 min before the exit). Cause
  unconfirmed.
- Four attempts to pre-build the client in `.local/parity-target`
  (`--profile fast`, `-j 1`) all failed on the same crate: the
  `horizon_client` lib crate (opt-level=3, codegen-units=16)
  needs ~3–3.2 GB RSS for its rustc, and the VM's 6 GB shared
  with two other streams' concurrent rustc processes cannot
  reliably hold it. Attempt 1: killed by a VM restart at ~45 min
  (at horizon-client-render). Attempt 2: OOM-killed at ~61 min
  (at the horizon_client lib crate). Attempt 3: reached 12:15 CPU
  on the lib crate, then its poll was aborted and its outcome was
  never recorded. Attempt 4: OOM-killed at ~7.5 min. Attempt 5
  (`taskset -c 0`): OOM-killed at ~16.8 min — affinity caps the
  codegen threads but the ~2.5 GB metadata floor dominates.
- A swapfile did not help: a 4G file was created and `mkswap`
  succeeded, but `swapon` failed "Operation not permitted"
  (container restriction); the file was removed. Existing swap is
  1 GB (`/dev/loop0`, ~426 MB used).
- There is no dev-profile escape: parity's client build is fixed
  to the `fast` profile by the recorded artifacts
  (`tools/parity.py` `build_commands` groups recorded artifacts
  by (profile, selection) and emits `cargo build --locked
  --profile <profile>`; the shared dependency records fix the
  client binary to `fast`). No flag selects dev.

The 9 recorded FAILs are 9 of 658 individual harnesses, not gate
harnesses; they are not presented as a profile. `status.json`
records the harness section as 0/0/0 of 17 with this note, and the
CI run will fill it in.

## What remains, by cause

### Behaviour work from the diff (not started)

The 31 ranked items are the port's work list; **0 of 31 are
closed**. The work is the Lava Chicken removal's behaviour
consequences, the 45 changed classes (29 concern Horizon), and
the 17 harness reruns — unattempted. The behaviour code in this
tree is still the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21.7
unchanged — the same oracle failures as 1.21.7 are expected once
the profile is measured (chunk-generation, command-exec,
first-person, food, item-model, datafix, options, resource-pack,
sound-event), but the counts are unmeasured for 1.21.6.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was blocked (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Cost experiment

Cross-protocol delta with a byte-identical packet set, recorded
for contrast with 1.21.8's cross-protocol delta (~10 h 40 min,
108 packets renumbered) and 1.21.7's same-protocol hotfix
(~4 h 44 min). Times are from session timestamps (VM clock, UTC).

- **Wall clock:** work started 2026-11-19 ~01:32 UTC; end
  2026-11-19 ~09:15 UTC. Total: ~7 h 45 min.
- **Work items closed vs total:** 0 / 31 behaviour items closed.
  The port closed the infrastructure layer only: pin, tables,
  adapter re-base, verification. No behaviour item was attempted.
- **Harness profile:** none measured locally (blocked — see
  above); CI to measure.
- **Sweep minutes:** none productive. The two sweep attempts
  (~2 h) and the five client pre-build attempts (~2 h 30 min)
  produced no profile.

### Where the time actually went (approximate)

| Activity | Time |
|---|---|
| Setup + salvage / diff / report | ~15 min |
| Table regen (run 1 SIGTERM'd at 37 min; run 2 13.6 min) | ~1 h |
| Adapter re-base + protocol-test re-pins | ~2 h |
| Verification (workspace check 15 min at -j1 after a -j2 OOM; protocol tests) | ~1 h |
| Oracle check (horizon-prepare, zero fixes) | ~15 min |
| Harness attempts (2 sweeps + 5 client pre-builds, all blocked) | ~4 h 30 min |
| Determinism regen (first attempt killed by a runtime restart; retry ~29 min) | ~45 min |
| Behaviour-item porting | 0 min |
| Publish (docs, notes, audit, commit) | ~30 min |

The contrast with 1.21.8 (~10 h 40 min) is not the delta — 1.21.6's
delta is smaller (31 items vs 526) and needed no packet rewire —
it is the VM tax under 3-stream load: the horizon-client lib crate
cannot be built in the `fast` profile on 6 GB shared with two
other streams, so ~4 h 30 min went to harness attempts that
produced no profile. The mechanical work itself (salvage, regen,
adapter re-base, verify, determinism) took ~4 h 45 min, as the
cross-protocol shape suggests.

## Status

Pin set (protocol 771, world 4435); 2 of 6 tables regenerated, 4
reproduce byte-for-byte; adapter ladder re-based to 771, packet
set byte-identical (no rewire); workspace compiles (0 errors,
0 warnings); protocol tests 430 pass / 2 fail (the sandbox-network
pair); prepare verified ok end-to-end, oracles compile with zero
fixes. Behaviour port **complete 2026-11-21: 31/31 items closed**
(see below). Harness profile: **not measured locally** (client build
OOM-blocked under 3-stream load); CI `port-verify` to measure
after merge. D6 deferred (owner).

## Behaviour port (2026-11-21)

Direction 1.21.7 -> 1.21.6, 31 items (ranks 3/17/10/0/1).
Verification method: byte-compare of the named 1.21.7/1.21.6 client
jars (9,484 vs 9,483 classes; 71 changed, 0 added, 1 removed:
`TrackingItemStackRenderState`), then a Vineflower decompile of the
changed classes only (72 + 71 class files) and a diff of the two
source trees (1,157 lines, 42 files). The running jar is the oracle;
decompiled code is a reading aid. The report's 45 changed classes is
the semantic (token-hash) count; the byte set is a superset that
includes constant-pool churn.

### Rank 1: closed 3/3

- **Item 1 — version.rs regenerate:** already 1.21.6-form:
  `VERSION_ID` "1.21.6", `PROTOCOL_VERSION` 771, `DATA_VERSION`
  4435, `RESOURCE_PACK_FORMAT` (63, 0), `DATA_PACK_FORMAT` (80, 0),
  `JAVA_VERSION` 21, `STABLE` true, `BUILD_TIME` "Tue Jun 17
  11:07:28 UTC 2025" — verified against the 1.21.6 version JSON (id
  1.21.6, type release, java-runtime-delta 21). No
  snapshot-protocol const exists in Rust (vanilla's snapshot
  protocol 258 -> 256 has no surface).
- **Item 2 — save format:** 0 schemas, 0 fixes, 0 datafix classes
  changed between 4435 and 4438 — verified: no schema keys or
  version guards for 4436–4438 anywhere in `datafix/`. No-op.
- **Item 3 — version facts:** protocol 772 -> 771, world 4438 ->
  4435, packs 64.0/81.0 -> 63.0/80.0 — all in version.rs (item 1).
  No stale 4438/1.21.7 version references in behaviour code (the
  772 mentions in `adapter.rs` are correct: 772 IS 1.21.7/1.21.8,
  one above the pin).

### Rank 2: closed 17/17

- **Item 4 — raids.rs / Vindicator, zero-diff:**
  `mobs/illager.rs` `vindicator_server_ai_step` (can_navigate_ground
  guard + raided flag) matches both versions' logic; the
  1.21.6/1.21.7 difference is a Java cast
  (`GroundPathNavigation`) vs an instanceof check. Note:
  `register_vindicator` sets no spawn-time can_open_doors (vanilla
  `finalizeSpawn` sets true) — a pre-existing gap, not this delta.
- **Item 5 — save.rs / SimpleRegionStorage, REAL:** 1.21.7 added
  `NbtUtils.addCurrentDataVersion` to `upgradeChunkTag`; 1.21.6
  returns the update unstamped. Verified from the decompiled 1.21.6
  jar: `ChunkStorage.upgradeChunkTag` (region chunks, used by
  `WorldUpgrader`) stamps in BOTH versions, so `update_chunk_to`
  keeps its stamp; `EntityStorage`'s load path and
  `SectionStorage.PackedChunk.parse` (POI, via the Dynamic
  overload, unchanged between versions) call
  `SimpleRegionStorage`'s overload, which does not stamp in 1.21.6.
  `update_region_tag_to` (`datafix/mod.rs`) no longer stamps; the
  stamp happens at save time, where both storages write a fresh
  tag (as `update_player` already does). No test asserted the
  region-tag stamp.
- **Item 6 — version.rs / SharedConstants, zero-diff:**
  `getProtocolVersion` is 771 in version.rs (item 1).
- **Item 7 — particles.rs / ItemStackRenderState, zero-diff:** the
  only reference is a comment citing `pickParticleMaterial`; the
  breaking-item sprite code reads `o.item` layers directly. The
  1.21.6/1.21.7 change is the model-identity tracking, which has
  no Rust surface.
- **Item 8 — EditBox / MultiLineEditBox, REAL (highlight):** see
  the highlight note below.
- **Item 9 — hud.rs / TextureAtlas + ReloadableTexture,
  zero-diff:** vanilla dropped a `close()` before texture creation;
  the Rust creates wgpu textures once at startup (`gfx.rs`) and
  replaces the pack atlas by assignment (the old texture drops
  with its handle) — there is no close-then-create sequence to
  port, and the Rust form already matches 1.21.6 (no close before
  create).
- **Item 10 — happy_ghast.rs, REAL:** `scanPlayerAboveGhast`'s
  AABB minY is `maxY` in 1.21.6 (`maxY - 1.0E-5F` in 1.21.7);
  `player_above` now uses `hi.y` directly. The
  `setServerStillTimeout` position-sync broadcast has no Rust
  surface (`still_tick` just sets the counter).
- **Item 11 — BakedGlyph, REAL (font depths):** see the font note
  below.
- **Item 12 — creative.rs, zero-diff:** no MUSIC_DISC references;
  the Lava Chicken tab entry was data, regenerated without the
  disc.
- **Item 13 — first_person.rs / ItemRenderer, zero-diff:** no model
  identity / `clearModelIdentity` in Rust.
- **Item 14 — villager.rs, zero-diff:** `can_open_doors = true`
  is set; the cast is Java-only.
- **Item 15 — RenderPipelines, citations:** `Blend::Invert`'s doc
  no longer cites `GUI_INVERT` (removed in 1.21.6);
  `Blend::Additive`'s doc no longer cites `GUI_TEXT_HIGHLIGHT`
  (not additive in 1.21.6). Both variants stay: the crosshair
  keeps INVERT's factors inline, and the highlight's second pass
  is additive.
- **Item 16 — Util, zero-diff:** `Util.isAarch64` removed; the
  aarch64 references in `crash.rs`/`gfx.rs` are arch labels and
  backend ordering, unrelated.
- **Item 17 — SoundEvents, zero-diff:**
  `MUSIC_DISC_LAVA_CHICKEN` removed from the registry tables
  (regenerated); no Rust code references it.
- **Item 18 — AbstractPiglin, zero-diff:** `mobs/piglin.rs` sets
  can_open_doors; the cast is Java-only.
- **Item 19 — Items, zero-diff:** `MUSIC_DISC_LAVA_CHICKEN`
  removed from the registry tables (regenerated); the only
  `lava_chicken` reference in Rust is a 1.21.8-pin adapter test
  name.
- **Item 20 — font.rs / Font, REAL (font depths):** see the font
  note below.

### Rank 3: closed 10/10

- **Item 21 — inventory_slots.rs / GuiGraphics, REAL (highlight):**
  `renderItem`'s Tracking→plain rename is zero-diff (no model
  identity in Rust); `textHighlight`'s removal is the highlight
  port (see below).
- **Item 22 — undead.rs / MoveThroughVillageGoal, zero-diff:** the
  Rust (set doors, create path, reset true) matches both versions'
  logic; the cast is Java-only.
- **Item 23 — pack_text.rs / GuiRenderer, zero-diff:** MIN_GUI_Z
  visibility and the scissor nullability are annotation-only; the
  Tracking→plain rename has no Rust surface; the GUI element
  z-step (`index` vs `index * 0.01`) has no Rust surface either —
  the Rust GUI has no depth test and orders by draw, all quads at
  depth 0.0.
- **Item 24 — block_entity_render.rs / PreparedTextBuilder, REAL
  (font depths):** see the font note below.
- **Item 25 — goals.rs / RestrictSunGoal, zero-diff:** a plain
  `avoid_sun` bool set/clear; the cast/instanceof is Java-only,
  and all RestrictSun users are ground navigators.
- **Item 26 — panorama.rs / CubeMapTexture, zero-diff:** no
  `close()` anywhere in `gfx/panorama.rs` (same GL-lifecycle
  reasoning as item 9).
- **Item 27 — submit.rs / CommandEncoder, zero-diff:** no
  `copy_to_buffer` in Rust; `copy_buffer_to_buffer` is wgpu API,
  not vanilla's method.
- **Item 28 — sign_edit.rs / AbstractSignEditScreen, REAL
  (highlight):** the sign highlight now goes through the shared
  `text_highlight` (see below); the screen's direct-fill form in
  1.21.6 is the same OR_REVERSE fill.
- **Item 29 — fog.rs / FogRenderer, REAL:** 1.21.6's
  FOG_ENVIRONMENTS lists Water before Blindness and Darkness
  (1.21.7 moved it after). `setup_fog`'s distance match now takes
  Water before the blindness/darkness arms; the colour path is
  unchanged (Blindness/Darkness never provide colour, only
  darkness — verified from `MobEffectFogEnvironment`). New test
  `water_fog_wins_over_blindness_and_darkness`.
- **Item 30 — BlendFunction, citations:** `BlendFunction.INVERT`
  removed in 1.21.6; CROSSHAIR inlines its factors. The
  `hud_layout.rs` `Blend::Crosshair` doc now says so (the `hud.rs`
  `Blend::Invert` doc likewise, item 15). No behaviour change:
  the factors are identical.

### Rank 5: closed 1/1

- **Item 31 — 17 unported classes, verified no Rust surface:**
  `DirectStateAccess` (+`Core`/`Emulated`), `GlCommandEncoder`,
  `GlDevice` (GL backend; the Rust renderer is wgpu);
  `RenderPipeline` (only a `@Deprecated` removal —
  annotation-only); `VertexFormat` (GL staging-buffer workaround;
  the two `DefaultVertexFormat` mentions in Rust are comments on
  the format constants, not the upload path); `DetectedVersion`
  (version facts, item 3); `ItemModelGenerators`,
  `VanillaEntityLoot`, `PaintingVariantTagsProvider` (datagen
  only; data regenerated); `OversizedItemRenderer`,
  `GuiItemRenderState` (Tracking→plain renames);
  `GuiRenderState` (decompiles identically — byte-only diff);
  `ReportEnvironment` (version string only);
  `TrackingItemStackRenderState` (removed class);
  `PaintingVariants` (DENNIS removed; painting variants are
  data-driven, no `dennis` in Rust); `JukeboxSongs` (LAVA_CHICKEN
  removed; songs resolve data-driven via the items registry).
  (The report counts 17; the outer-class count is 16 — the opengl
  package count of 5 includes the `DirectStateAccess$Core` /
  `$Emulated` nested classes.)

### The text-highlight port (items 8, 15, 21, 28, 30)

1.21.6's `GUI_TEXT_HIGHLIGHT` is `withColorLogic(LogicOp.OR_REVERSE)`
over a blue fill (-16776961); 1.21.7's was ADDITIVE, and 1.21.7's
`GuiGraphics.textHighlight` did two fills (white with GUI_INVERT,
then blue with GUI_TEXT_HIGHLIGHT). Both forms compute the same
pixels: OR_REVERSE is GL 5387, `src | ~dst`, so blue over `(r, g,
b)` gives `(1-r, 1-g, 1)`; the two-pass form gives `(1-r, 1-g,
1-b)` then `(1-r, 1-g, 1)` after the clamped blue add. The Rust
keeps the two-pass form (wgpu has no logic ops, and no single wgpu
blend expresses OR_REVERSE-with-blue — the blue channel needs
`One` while red/green need `OneMinusDst`).
`HudBatch::text_highlight` drops its `invert_text` parameter —
every 1.21.6 highlight inverts (there is no
`setInvertHighlightedTextColor` in 1.21.6; neither EditBox has an
invert flag) — and `append_field` drops it too; all 8 call sites
updated, and the ui.rs test's anvil case now expects
`[Invert, Additive]`.

### The font-depth port (items 11, 20, 24)

1.21.7 multiplied the GUI z scale by 10; 1.21.6 has
`SHADOW_DEPTH` 0.003, `EFFECT_DEPTH`/`OVER_EFFECT_DEPTH` 0.001,
`UNDER_EFFECT_DEPTH` -0.001 (verified from the decompiled
`Font`/`BakedGlyph`; `Z_FIGHTER` is 0.001 in both versions, so the
bold copy's `base + 0.001` is unchanged). `hud.rs` `styled_text`:
inline-object front 0.03 -> 0.003, `base` 0.03 -> 0.003, effect
lines `base + 0.01` -> `base + 0.001`; `block_entity_render.rs`
`text_vertices`: effect lines `z + 0.01` -> `z + 0.001`, with its
three test asserts updated; the `component_shadow_color` test's
0.03 assert -> 0.003. (In 1.21.6 the bold copy and the effect
lines share a plane at `base + 0.001` — vanilla does the same.)
The text background (`UNDER_EFFECT_DEPTH`) has no Rust surface:
the Rust draws backgrounds by draw order, not z.

### Pre-existing datafix unit-test failures (not this delta)

Three `horizon-world` datafix tests fail in this tree, before and
after the behaviour port (verified by re-running with the item-5
change temporarily reverted: 30 passed / 3 failed either way).
All three were written for the 26.x tree and assume versions or
fixes above 4435:

- `failed_upgrade_keeps_old_data_version`: hardcodes old = 4899,
  newer than 1.21.6's CURRENT (4435), so the upgrade
  short-circuits and level.dat is never stamped.
- `block_states_take_their_new_field_names`: the test chunk is at
  DataVersion 4903, newer than CURRENT, so `update_chunk` returns
  early and the palette keeps `Name` instead of `id`.
- `level_data_of_1_20_moves_to_the_new_fields`: expects a
  `difficulty_settings` field, but the `level_dat_difficulty` fix
  is registered at schema 4771, above CURRENT, so it never runs
  for a 1.21.6 upgrade (vanilla 1.21.6 level.dat has no
  `difficulty_settings` field).

They do not gate the delta-bar (D2 is cargo check + protocol
tests; D4 is the datafix harness, not unit tests). The three
entity/POI stamp tests that the item-5 change did break
(`old_chunks_are_converted_and_current_ones_skipped`,
`poi_chunks_without_a_version_start_at_1945`,
`the_worker_thread_finishes_and_a_cancel_joins_it`) were updated
to the 1.21.6 form (no stamp) and pass.

## CI proof (run 44800043813, 2026-11-21)

port-verify on behaviour/1.21.6@13e76f9 (all 31 items closed),
success in 1h33m23s. Profile **2 pass / 2 oracle / 13 fail** of 17
— identical to the mechanical proof (run 44496248958): no
regression from the behaviour port. Passing: block_properties,
lan. Classified oracle: chunk_generation, command_exec. The
classifier undercounts as before: 6 more oracle-compile failures
hide in the fail column (datafix ×6 worlds, first_person, food,
item_model, options, resource_pack — javac CalledProcessError in
the harness logs), and component_patch's oracle compiles but its
run exits 1, so the true oracle count is **9** and the honest
reading is 2 pass / 9 oracle / 6 real fail (command_parse,
entity_components, hotbar, light, save_roundtrip, sound_event).
Published profile: 23 entries (17 + datafix#1..#6), 2 pass / 21
fail.

Verdict: **failed**. D1 fail — the tag audit finds 37 dead
production tags, inherited (triage belongs at 1.21.11). D2 pass —
cargo check 0 errors / 0 warnings, protocol tests 432 / 0. D3
blocked — the pin had no work-items.json at dispatch (it was
committed to the branch after this run started, e30e9f3); the
final proof re-checks with the pin complete. D4/D5 blocked — the
profile stamp quirk (profile stamped 13e76f9, judge's staged
candidate HEAD 090a9fc7; patch 0006 is dead code in the CI judge,
as on 26.2, 1.21.7, 1.21.8 and 1.21.9). D6 blocked — no
playtest.md; the playtest is deferred by the owner, never claimed.

## Final CI proof (run 44814971968, 2026-11-21)

port-verify on behaviour/1.21.6@2077e11 (all 31 items closed, pin
work-items.json in place, stamp tests updated), success in
1h03m25s. Profile **2 pass / 2 oracle / 13 fail** of 17 — the same
as run 44800043813 and the mechanical proof. Passing:
block_properties, lan. Classified oracle: chunk_generation,
command_exec; true oracle 9 as above.

Verdict: **failed**. D1 fail — the tag audit finds 37 dead
production tags, inherited (triage belongs at 1.21.11). D2 pass —
cargo check 0 errors / 0 warnings, protocol tests 432 / 0. D3 pass
— rank-1 3/3, rank-2 17/17, rank-3+ inheritance statement read.
D4/D5 blocked — the profile stamp quirk (profile stamped 2077e11,
judge's staged candidate HEAD b9d9448c; patch 0006 is dead code in
the CI judge). On substance D4 fails (datafix and save_roundtrip
both fail in the profile — datafix on oracle-compile failures,
save_roundtrip on a server-closed connection) and D5 fails 7/8
(only block_properties passes of the E6 subset). D6 blocked — no
playtest.md; the playtest is deferred by the owner, never claimed.

End main island (worldgen-verify): vanilla 1.21.6's
`minecraft:end_islands` includes the central island cone in the
function itself, but the tree carried the 26.3 outer-only sampler
and did not decode the `end_islands` name at all, so the End
pipeline failed to build. Carried 26.2's fix:
`GraphNode::EndIslands { central }`, with `end_islands`
decoding to central: true and `end_outer_islands` to central:
false; the cone formula was verified against the 1.21.11 named
jar's EndIslandDensityFunction bytecode. The end-to-end worldgen
test (`prepared_world_builds_and_generates_chunks`, run by the
worldgen-verify CI job) now generates two chunks per dimension
and dumps them as JSON; it caught this gap.

No mobs spawn at all (2026-11-21, carried from 1.21.7). The
owner's 1.21.7 playtest found no mobs in any fresh world; the same
two 26.x-only assumptions were in this tree, verified by grep
before the fix was carried over (1.21.7's fix: 5738c7e6, merged
dd966dea).

First, every biome's spawn list was empty. `BiomeInfo::from_json`
(`crates/horizon-client-render/src/biome.rs`) read spawns only
from the `minecraft:gameplay/natural_mob_spawns` environment
attribute, but 1.21.6 biomes have no `attributes` key at all (the
attribute system arrived in 1.21.11) — their spawns live in the
legacy top-level `spawners` field (plains: sheep 12, pig 10,
zombie 95, all `minCount`/`maxCount` 4/4, verified against the
vanilla client jar). The 26.2 legacy fallback is ported here (the
attribute wins when present): `spawners` when the attribute list
is empty, `spawn_costs` likewise, `creature_spawn_probability`
before the attribute, default 0.1. `parse_entries` also gains the
legacy `minCount`/`maxCount` count form (a draw even when min ==
max, so never constant), which the 26.3 `count` forms alone didn't
cover — the same fix covers structure `spawn_overrides`, which use
the same entry shape.

Second, mob data failed to load at all. `StructureMobFinalizer::load`
(`crates/horizon-client/src/vanillagen/structure_mobs.rs`) required
`minecraft:camel_husk` in `entity-types.json`, a 26.x-only entity
(1.21.6's registry has 151 entries, no camel_husk — verified with
`javap` against the vanilla named jar, which also confirms the
camel itself is present), so `load()` returned None —
chunk-generation mobs (`spawn_generated`) and structure mobs
(`finalize`) both dead. The husk reuses the camel's model
(`camel#main`), so the camel's own size now stands in when the
husk type is absent; the jockey can never materialize here anyway,
since creating a `minecraft:camel_husk` finds no such type in the
registry. One wart remains: a husk that rolls the 26.x jockey
attempt still gets an iron spear in hand (a 26.x item 1.21.6
lacks) before the camel create fails — a cosmetic oddity for
desert husks, not a spawn blocker.
Regression tests: `legacy_spawners_fill_spawns_without_attributes`
and `attribute_spawns_win_over_legacy_spawners` (hermetic biome
fixtures), `mob_data_loads_without_camel_husk` (minimal fixture).
Biome pack_tests 5/5; `mob_data_loads_without_camel_husk` passes.
The three resource-dependent structure_mobs tests could not be
run for this version: no prepared data exists for 1.21.6 on the
VM, and the two chunk-based tests would additionally be blocked
by the pre-existing "missing motion tag" gap (this tree still
requires the `blocks_motion` tags that 1.21.6's data lacks — the
gap the 1.21.7 launch fix, 789af860, closed there; carrying it
over is separate work). The ocean ruin test's expectation is
adapted as in 1.21.7 (no nautilus here, so the trident drowned
stays unmounted and the later draws shift); its values are
1.21.7's measured ones, not verified by a run in this tree. (The
`arrows_client_tests` compile fix from 1.21.7 does NOT apply here:
1.21.6's `FluidResources::from_registry` still takes
`&BTreeSet<String>`, so the original `&BTreeSet::new()` call is
correct — only 1.21.7 changed the signature to `&[bool]`.)
