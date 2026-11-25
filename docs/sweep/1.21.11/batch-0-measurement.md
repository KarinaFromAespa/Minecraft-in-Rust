# Batch 0 measurement: oracle compilability against the 1.21.11 jars

Date: 2026-11-25. Branch: `parity-sweep-1.21.11`. Measured before any repair.
Companion data: `batch-0-oracles.json` (per-oracle table, all 391 files),
`evidence/` (raw big-compile logs, grouping script + output, jar listings,
1.21.10 diff). The measurement script is `docs/sweep/measure-oracles.py`
(the 1.21.10 sweep worktree's copy, run with `HORIZON_VERSION_DIR`
pointed at the 1.21.11 tree — the script is not present in this worktree's
`docs/sweep/`; a copy with the one 1.21.11 adaptation noted in §0 is kept
at `evidence/measure-oracles-1.21.11.py`, rerunnable).

**Scope decision (coordinator, per the sweep brief): Batch 0 for 1.21.11 is
measurement + triage ONLY — no oracle repairs — with ONE explicit exception:
the client_probe check (H01, "The client_probe check (H01)" below).** The
oracle `.java` files live in `versions/1.21.11/mods/horizon/`. The 1.21.7
Batch 0 did repair oracles (135/391 pass after repair, up from 94); for
1.21.11 the repair list below is a follow-up decision for the parent.
Batches use static checks (javap + source + data) for drivers whose oracles
don't compile, as 1.21.7 Batch 2 did.

## §0 Execution status

- Measurement ran in two parts: the first run died during the server
  big-compile with a transient `Disk quota exceeded` (EDQUOT) write error
  (~01:03 UTC) — `df` showed 46G free at the time and writes worked
  immediately afterwards, so this was a transient quota burst under
  concurrent load (the coordinator's H01 cargo check, sibling sweep
  batches and the 1.21.4 port agent were all writing), not a full disk.
  Nothing had been checkpointed (the server fixpoint had not finished),
  so no partial numbers needed reconciling. The rerun completed all
  three classpaths cleanly in ~36 min (javac 25.0.5, VM load average
  ~30 during the run).
- The `both` fixpoint converged cleanly in 2 rounds (15 cascade drops);
  no transient-filesystem artifact of the 26.1.1 kind occurred.
- One measurement deviation vs prior runs, forced by the tree: the
  1.21.11 tree has NO `tools/jsr305-3.0.2.jar` (unlike 1.21.10, which
  carries it), so the script's `JSR305` path was pointed at the port
  workdir's copy
  (`e6-1.21.11-behaviour/versions/1.21.11/tools/jsr305-3.0.2.jar`,
  sha1 `25ea2e8b…`, identical to 1.21.10's copy). The client cp adds
  jsr305 as in every prior run; the server cp does not (script default).
  The missing jar is itself a finding — see §5, driver defect 1.
- The H01 cargo check is PENDING (coordinator-run; see the H01 section):
  its log stalled at 01:02:53 UTC with no process running, in the same
  window the first measurement run died. This batch did NOT run the
  check itself, per the brief.

## Headline

**127 of 391 oracles (32.5%) compile against the 1.21.11 jars on at least
one classpath; 264 (67.5%) fail on all three.** This is better than
1.21.10 (104 pass, 287 fail-all) — the first oracle measurement to improve
on its predecessor since the 26.x runs — but still well below 26.1.2
(189 pass). Two causes, both measured: the 1.21.11 jar supplies the
1.21.11-era APIs the oracle sources already reference (gamerules,
entity sub-packages, world.attribute, gizmos, permissions — 7 new passes
on byte-identical sources), AND the template port adapted 19 oracle
sources to 1.21.11's APIs in-tree (16 new passes on changed sources,
each carrying an explicit `1.21.11` comment). The remaining failures are
the familiar 26.3-form groups (ServerLevel ctor 85, densityfunction 17,
EntityTypes 30, LocalPlayer 18, renderpearl 8) plus the long tail.

| Classpath (driver configuration) | Pass | Fail | Failure rate |
|---|---|---|---|
| server — server-game.jar + server libraries (direct drivers) | 102 | 289 | **73.9%** |
| client — client.jar + client libraries + jsr305 (oracle_harness drivers) | 127 | 264 | **67.5%** |
| both — server + client jars + both library sets (superset) | 127 | 264 | 67.5% |
| **fail on all three classpaths (genuinely broken vs 1.21.11)** | — | **264** | **67.5%** |

Notes:

- The `both` set is identical to the `client` set, and every server-pass
  file also passes on client cp: the client jar contains the shared/server
  classes these oracles use, so client cp is the superset that matters.
  The 25 files that pass on client but fail on server are client-only
  oracles (`net.minecraft.client.*`, blaze3d) — expected, not breakage.
  No file passes on server only. 102 files pass on all three.
- The client-cp measurement adds jsr305 to the classpath. The harness
  itself does not (see §5, driver defect 1): without jsr305, javac can
  crash (`CompletionFailure: class file for javax.annotation.Nullable
  not found`, rc=4) on some oracles. The measurement isolates oracle
  breakage from that harness bug; the harness bug is recorded separately
  and NOT fixed in this batch (scope decision above). Unlike 1.21.10,
  the jar itself does NOT exist in the 1.21.11 tree at all, and neither
  the client NOR the server driver path appends it — the 1.21.7
  server-path fix (`d7a14e91`, present in 1.21.10's
  `test_rust_parity.py:112-115`) never reached the template.
- Error totals: server cp 4,786 errors in 276 files; client cp 2,162
  errors in 249 files (both cp identical). For comparison, 1.21.10 was
  5,761 server / 3,328 client; 26.1.2 was 4,286 server / 1,391 client;
  1.21.7 pre-repair was 6,199 server / 3,916 client. 1.21.11's error
  volume sits between 26.1.2 and 1.21.10 — but its pass count (127) is
  still far below 26.1.2's (189), because the errors concentrate in
  shared constructors and packages rather than spreading thin: median 4
  errors per failing file on client cp (1.21.10: 6; 26.1.2: 3), p90 19
  (1.21.10: 26; 26.1.2: 14), max 251 (`VanillaDensityOracle`, same file
  as every run), 58 one-error files (1.21.10: 53; 26.1.2: 55), 199 with
  ≤10 errors.

## Method

Same fixpoint big-compile as the 1.21.7/26.2/26.1/26.1.1/26.1.2/1.21.10
measurements (`docs/sweep/measure-oracles.py`, unchanged except the
jsr305 path noted in §0): per classpath, compile all 391 files together,
then iterate the clean set (sourcepath=False) until the compile passes.
Fixpoint iterations: server 2 rounds (13 cascade drops), client 2 rounds
(15 cascade drops), both 2 rounds (15 cascade drops). Total runtime
~36 min wall for the completed run (javac 25.0.5), inflated by VM
contention (load ~30).

Jars: the named 1.21.11 jars from the prepared `.local/minecraft/1.21.11/`
(see §6); `javac --release 21 -proc:none`, JDK 25.0.5. 1.21.11's
`java_major` is 21, matching the version manifest's
`javaVersion.majorVersion` (DataVersion 4671, protocol 774). Jar sha1s
verified against the common-brief pins before measuring: the `.local`
jars are remapped copies whose class lists are byte-identical to the
brief's e6-deltas named jars (client 10,291 classes, server-game 7,163
classes — 0 diff lines on both listings), so the measurement is against
the brief's jars in substance; the bundler server jar
(`64bb6d76…`) was never javap'd. The script was run with
`HORIZON_VERSION_DIR` pointed at `versions/1.21.11` (its committed home
in `docs/sweep/` would otherwise resolve the repo root as the version
tree).

Tally: this batch's 391 checks are oracle-compilability checks — 0
behaviour + 391 non-behaviour, as-tallied per the batch brief (total =
files measured, pass = compiles on at least one classpath, gap =
fail-all). The behaviour-only restatement is the coordinator's job.

## Failure classes (first error per file)

Server cp (289 failures):

| Class | Files | Meaning |
|---|---|---|
| missing-package | 109 | imports a package that does not exist on the server cp (mostly client-only packages — 25 of these pass on client cp — plus the 26.x-only packages) |
| missing-symbol | 76 | references a class/method/field 1.21.11 does not have (renamed/moved API) |
| constructor-mismatch | 70 | calls a constructor with a 26.3 signature (mostly ServerLevel) |
| cascade | 13 | file itself is clean; fails only because a helper it pulls is broken |
| no-override | 10 | `@Override` of a method 1.21.11's supertype lacks |
| other | 6 | assorted (static-context, access, inference, …) |
| incompatible-types | 4 | 26.3 vs 1.21.11 signature change |
| missing-override | 1 | abstract-method gap |

Client cp (264 failures): missing-symbol 103, constructor-mismatch 82,
missing-package 39, cascade 15, no-override 13, other 7,
incompatible-types 4, missing-override 1. The client cp sheds the
client-package failures and exposes the real drift: constructor
mismatches are the largest single class after missing symbols (82 vs
1.21.10's 56 — the ServerLevel group plus a grown MinecraftServer
group), and missing-package first errors collapsed vs 1.21.10 (39 vs
83) because the 1.21.11-era packages now exist.

## The systematic groups (repair list for the follow-up decision)

All groups below were derived FRESH from the 1.21.11 measurement
(`evidence/group.py` + `evidence/groups.txt` + `evidence/analyze.py`:
error-text and source-pattern matching over the 1.21.11 big-compile
logs, same derivation as 1.21.10) and verified against 1.21.11's own
jars with javap / jar listings — none was copied from the 1.21.10
report. Where a group's membership matches 1.21.10's, that is a
measured fact, stated as such.

**G0 — `ServerLevel` constructor (85 oracles; the headline group).**
1.21.11's `ServerLevel` constructor takes 11 arguments
(`MinecraftServer, Executor, LevelStorageAccess, ServerLevelData,
ResourceKey, LevelStem, boolean, long, List<CustomSpawner>, boolean,
RandomSequences` — javap-verified on the 1.21.11 server-game jar,
IDENTICAL to 1.21.10's); the oracles call the 26.x 10-argument form
(no `RandomSequences`), e.g. `VanillaActiveEffectsOracle:36`
`super(null,null,null,null,null,null,false,0L,List.of(),false)`. 66 of
the 82 constructor-mismatch first errors on client cp are ServerLevel.
28 files are single-cause — one mechanical fix (append the
`RandomSequences` argument) rescues them outright:
`VanillaActiveEffectsOracle`, `VanillaAttributeHealthOracle`,
`VanillaAttributeMapQueriesOracle`, `VanillaBlockEventReadinessOracle`,
`VanillaClimbingOracle`, `VanillaCombatRulesOracle`,
`VanillaConsumeEffectOracle`, `VanillaCrackFeedbackOracle`,
`VanillaDamageReductionOracle`, `VanillaEdgeBackoffOracle`,
`VanillaEffectSyncOracle`, `VanillaFallDamageOracle`,
`VanillaFoodPropertiesOracle`, `VanillaGroundStateOracle`,
`VanillaHelmetDamageOracle`, `VanillaItemUseLifecycleOracle`,
`VanillaLiveAttributeUpdatesOracle`, `VanillaLivingDamageOracle`,
`VanillaMovementEmissionOracle`, `VanillaMovementSupportOracle`,
`VanillaPistonMovementOracle`, `VanillaPlayerFoodOracle`,
`VanillaRedstonePersistenceOracle`, `VanillaServerPlayerDamageOracle`,
`VanillaStoredConsumeOracle`, `VanillaStoredEffectNotificationsOracle`,
`VanillaSwimmingOracle`, `VanillaWaterSplashOracle`. The other 57
carry additional unrelated errors. NOTE for the repair: the 1.21.7 fix
(12-arg, with `ChunkProgressListener`) does NOT carry — 1.21.11's ctor
has no progress-listener parameter; the translation is 10→11 args,
appending `RandomSequences` (the 1.21.10 port's adaptation of
`VanillaNaturalSpawnOracle` already uses this form — and the 1.21.11
tree's own `VanillaNaturalSpawnOracle` does NOT, see G10). This group
is one file smaller than 1.21.10's 86 (the dropped file is
`VanillaBlockLandingOracle`, which passes here) but has 5 more
single-cause files (28 vs 23), because the 1.21.11-era package errors
that used to fire first in those files are gone.

**G1 — `Level.fuelValues()` abstract-method gap (5 oracles) + 1 other
missing override.** 1.21.11's `Level` declares
`public abstract FuelValues fuelValues()` (javap-verified:
`net.minecraft.world.level.block.entity.FuelValues`); the oracles'
fake `Level` subclasses don't implement it:
`VanillaActiveEffectTickOracle`, `VanillaAttributeUpdatesOracle`,
`VanillaBlockEntityAnimOracle`, `VanillaLivingLandingOracle` (same 4
as 26.1.2 and 1.21.10), plus `VanillaBlockEntityRenderOracle` which
references `fuelValues()` among its errors. Mechanical repair: add a
`fuelValues()` override returning a default to each fake level. One
more missing-override file has a different abstract gap:
`VanillaDecorationRegionOracle` (first error at line 49 — anonymous
class missing a `lookup…` override on `RegistryInfoLookup`; 16 errors,
same as 1.21.10). The missing-override first-error count is 1 (same
as 1.21.10) only because the other files fail earlier on
ServerLevel/package errors — the gaps are still there underneath.

**G2 — `LocalPlayer` constructor (18 oracles carry the error; 7 as
first error).** 1.21.11's `LocalPlayer` constructor takes 7 arguments
(`Minecraft, ClientLevel, ClientPacketListener, StatsCounter,
ClientRecipeBook, Input, boolean` — javap-verified on the 1.21.11
client jar, identical to 1.21.10's); the oracles pass the 26.3
9-argument form. Membership is 1.21.10's 20 minus 2:
`VanillaFirstPersonOracle` (adapted — see G10) and
`VanillaSheepDyeOracle` (restructured — no `new LocalPlayer` call at
all in the 1.21.11 source) now pass. The remaining 18:
`VanillaAllayInteractionStateOracle`, `VanillaAmbientOracle`,
`VanillaAutoJumpOracle`, `VanillaBoatRiderOracle`,
`VanillaCameraOracle`, `VanillaFireStarterSoundOracle`,
`VanillaGameLoopOracle`, `VanillaHeldDropOracle`,
`VanillaHeldItemStackOracle`, `VanillaHudOracle`,
`VanillaInteractionInputOracle`, `VanillaLocalInputOracle`,
`VanillaMarkerParticlesOracle`, `VanillaPlayerSyncOracle`,
`VanillaRemoteAttackOracle`, `VanillaSoundEngineOracle`,
`VanillaSprintInputOracle`, `VanillaTeamRulesOracle`. 3 are
single-cause (`VanillaAmbientOracle`, `VanillaLocalInputOracle`,
`VanillaPlayerSyncOracle` — vs 1 in 1.21.10), because the package
errors that used to fire first are gone. One signature translation,
verified once against the 1.21.11 jar, rescues the 3 single-cause
files outright; the other 15 carry additional unrelated errors.

**G3 — `densityfunction` package absent from 1.21.11 (17 oracles).**
`net.minecraft.world.level.levelgen.densityfunction` has ZERO entries
in the 1.21.11 server-game.jar AND client.jar (verified by jar listing)
— the `SamplerContext`/`DensityVolume` API is a 26.3 addition.
Membership is IDENTICAL to 1.21.10's and 26.1.2's 17 files:
`VanillaAnchorOracle` (10 errors, blocks 9 drivers),
`VanillaBiomeGenerationOracle` (12),
`VanillaBiomeSourceCodecOracle` (9), `VanillaBiomeSourceOracle` (10),
`VanillaDensityOracle` (251 errors, the largest single failure),
`VanillaFloatLogOracle` (9), `VanillaMaterialConditionOracle` (76 errors,
blocks 5 drivers), `VanillaMaterialExtensionOracle` (12),
`VanillaMaterialStageOracle` (28 errors — vs 26 in 1.21.10),
`VanillaMaterialSurfaceOracle` (15), `VanillaNoiseOracle` (32),
`VanillaStructureDensityOracle` (4), `VanillaStructurePlacementOracle`
(17 — vs 16 in 1.21.10), `VanillaStructureStartOracle` (2),
`VanillaTerrainOracle` (13), `VanillaWorldSpawnOracle` (5),
`VanillaWorldgenOracle` (42). Several files reference the package
fully-qualified rather than by import, so the package error hides
behind a symbol first error in some files.
`VanillaStructureDensityOracle` is unrepairable by design (it directly
exercises the 26.3-only API; density is verified via bytecode/chunk
comparison instead — same ruling as 1.21.7, 26.2, 26.1, 26.1.1, 26.1.2
and 1.21.10). The others need per-file translation to 1.21.11's actual
density API, not import fixes.

**G4 — `EntityTypes` holder move (30 oracles; the 1.21.x-shape group).**
1.21.11 predates the `EntityTypes`/`BlockEntityTypes` move — registration
still lives in `net.minecraft.world.entity.EntityType` /
`net.minecraft.world.level.block.entity.BlockEntityType` (the 1.21.x
shape). `EntityTypes` and `BlockEntityTypes` have ZERO entries in the
1.21.11 server-game.jar (verified by jar listing); the constants the
oracles want exist on the old holder instead (`EntityType.PLAYER`,
`EntityType.CHICKEN` — javap-verified static finals), and
`BlockEntityType.BED` exists (1.21.11 keeps the bed block-entity type).
Every oracle referencing `EntityTypes.X` fails — 30 files, 1.21.10's
32 minus `VanillaCommandExecOracle` and `VanillaItemModelOracle`
(both adapted out of the holder reference in the 1.21.11 tree and now
passing):
`EntityTranscript`, `KnockbackClient`, `LocatorBarClient` (17 errors,
blocks 8 drivers), `RideTranscript`,
`VanillaAllayInteractionStateOracle`, `VanillaBlockEntityAnimOracle`,
`VanillaBoatAuthorityOracle`, `VanillaBoatRiderOracle`,
`VanillaBreezeHookOracle`, `VanillaDispenserOracle` (19 errors, blocks
6 drivers), `VanillaEntityDeathLootOracle`,
`VanillaFinalizeSpawnOracle`, `VanillaLateEntityPairOracle`,
`VanillaMenuOracle` (8 errors, blocks 4 drivers plus 2 cascades),
`VanillaMobAiOracle`, `VanillaMobBreathingOracle`,
`VanillaMobFallResetOracle`, `VanillaNaturalSpawnOracle`,
`VanillaProtocolClient` (39 errors, blocks 8 drivers),
`VanillaRegistryFallRayOracle`, `VanillaRemoteAttackOracle`,
`VanillaSceneMotionOracle`, `VanillaShulkerBulletOracle`,
`VanillaShulkerSpaceOracle`, `VanillaSourceSilenceOracle`,
`VanillaSprintInputOracle`, `VanillaSquidClientOracle`,
`VanillaTradingOracle`, `VanillaWetFireOracle`, `VanillaWorldRayOracle`.
Repair is a mechanical `EntityTypes.X` → `EntityType.X` holder
translation per reference, but most files carry additional unrelated
errors, so the translation alone rescues few outright. (The brief's
"O5 inverts" note: the failure direction is the SAME as 1.21.10's and
26.1.2's — all three versions predate the move — and the opposite of
26.2/26.3, where the holder exists. No inversion vs 1.21.10 was
measured; the group shrank by two files instead, both by adaptation.)

**G5 — `renderpearl` package absent from 1.21.11 (8 oracles).**
`com.mojang.renderpearl.api.pipeline` (and `.buffers`) do not exist in
1.21.11 — the render-pipeline API is 26.3-only (0 entries in the client
jar). Membership IDENTICAL to 1.21.10's and 26.1.2's 8 files:
`VanillaHudOracle` (77 errors — vs 100 in 1.21.10),
`VanillaLoomScreenOracle` (5), `VanillaPackScreenOracle` (62 — same),
`VanillaRecipeBookScreenOracle` (35 — same),
`VanillaSpriteAnimationOracle` (2), `VanillaTabListOracle` (36, also
authlib — G7), `VanillaTranslucencyOracle` (4),
`VanillaWorldBorderRenderOracle` (34). These need rewrites against
1.21.11's screen/render APIs, not import fixes.

**G6 — Other constructor mismatches.** `VanillaChunkGenerationOracle`
is GONE from this group (it passes now — the 1.21.11 source no longer
calls the `ServerChunkCache` ctor directly); the remaining members:
`VanillaInventoryUseStartOracle` + `VanillaItemUseStartOracle`
(`ServerChunkCache` ctor — 1.21.11's takes 11 args, javap-verified),
`VanillaFallingBlockOracle` (`ChunkMap` ctor),
`VanillaBiomePlacementOracle` + `VanillaDecorationOracle` +
`VanillaFeatureCatalogOracle` + `VanillaFeaturePlacementOracle`
(`Feature` ctor — 1.21.11's takes only `Codec<FC>`, javap-verified),
`VanillaHeldDropOracle` + `VanillaInteractionInputOracle` +
`VanillaPackScreenOracle` (`Gui` ctor), `VanillaBlockUpdateOracle` /
`VanillaEntitySoundPacketOracle` / `VanillaGameLoopOracle` /
`VanillaMobAiOracle` / `VanillaRedstoneCatalogOracle` /
`VanillaSculkOracle` / `VanillaTradingOracle` (`MinecraftServer` ctor
— 7 files, vs 3 in 1.21.10; the four new members surface now that
their 1.21.11-era package errors are gone), `VanillaMenuOracle` /
`VanillaNaturalSpawnOracle` (`DedicatedServer` ctor).
`VanillaSoundEventOracle` is GONE from this group: 1.21.10's
`SoundManager` ctor is 2-arg (`Options, MusicManager`) while 1.21.11's
is 1-arg (`Options`, javap-verified) — the oracle's 1-arg call passes
here. Each remaining mismatch is a one-signature translation against
the 1.21.11 jar.

**G7 — Other missing packages (39 missing-package first errors on
client cp, vs 83 in 1.21.10).** The 1.21.11-era packages are GONE from
this group — all present in the 1.21.11 jar, zero errors from them:

*Present in 1.21.11 (jar-verified), NOT failures — the 1.21.10 O6
members that do NOT reproduce:*
- `net.minecraft.world.level.gamerules` (GameRules moved there in
  1.21.11; 9 jar entries): 26 source refs, 0 errors. `VanillaFoodOracle`
  (8 errors in 1.21.10) passes outright on the jar delta.
- Entity sub-packages (the 1.21.11 entity-package split): 33 source
  refs, 0 errors — `…vehicle.boat` (8), `…vehicle.minecart` (20),
  `…monster.zombie` (15), `…projectile.arrow` (6),
  `…animal.cow` (8), `…animal.squid` (5), `…animal.fox` (24),
  `…decoration.painting` (4), `…projectile.throwableitemprojectile`
  (9). `VanillaPistonPushOracle`, `VanillaZombieCureSoundOracle`,
  `VanillaEntityParticlesOracle` pass outright on byte-identical
  sources.
- `net.minecraft.world.attribute` (environment attributes, 53 jar
  entries): 31 source refs, 0 errors. `VanillaEnvironmentSystemOracle`
  (36 errors in 1.21.10), `VanillaEnvironmentAttributeModifierOracle`
  (9) and `VanillaLevelShapeOracle` (2) pass.
- `net.minecraft.gizmos` (20 jar entries): 4 source refs, 0 errors.
  `VanillaMobAiOracle` drops 103 → 38 errors on this alone (plus
  gamerules and the entity packages).
- `net.minecraft.server.permissions` (18 jar entries): 12 source
  refs, 0 errors. `VanillaGameLoopOracle`'s and
  `VanillaServerFilesOracle`'s permissions first-errors are gone.

*26.x-only packages (absent from 1.21.11 — the remaining failures):*
- `net.minecraft.world.clock`: 13 source refs, 10 first-errors
  (`VanillaActiveEffectTickOracle`, `VanillaAttributeUpdatesOracle`,
  `VanillaAudioAttributesOracle`, `VanillaBlockEntityAnimOracle`,
  `VanillaBlockUpdateOracle`, `VanillaEnvironmentOracle`,
  `VanillaLivingLandingOracle`, `VanillaMobAiOracle`,
  `VanillaSculkOracle`, `VanillaTradingOracle` — all also G1/G3/G6).
- `net.minecraft.world.level.levelgen.material.rule`: 5 source refs
  (same 5 as 1.21.10), 3 first-errors — all also G3.
- 26.3 advancements packages (1.21.11 has only `critereon`;
  `predicates.entity` and `triggers` both 0 entries):
  `ComponentWireCapture` (`advancements.predicates.StatePropertiesPredicate`,
  1 error), `VanillaRaidOracle` (`advancements.triggers`, 22 errors).
  `VanillaEntityComponentsOracle` (12 errors in 1.21.10) is adapted
  out and passes.
- `com.mojang.authlib.services` (absent from authlib 7.0.61):
  `VanillaTabListOracle` (36 errors) only — `VanillaCommandExecOracle`
  (45 errors in 1.21.10) is adapted out and passes.
- `org.lwjgl.sdl`: `VanillaTickScript` (23 errors).
- `net.minecraft.util.filefix`: 0 source refs — GONE. The 1.21.11
  `VanillaDataFixOracle` was rewritten without the filefix import and
  passes (25 errors in 1.21.10 → 0).
- `net.minecraft.client.resources.model.geometry` /
  `.sprite`: 11 source refs, 3 first-errors (`VanillaModelBakingOracle`,
  `VanillaModelChoiceOracle`, `VanillaModelResolutionOracle`).
  `VanillaItemModelOracle` (45 errors in 1.21.10) is adapted out and
  passes on client.
- `net.minecraft.client.model.object.book` / `.banner`: 3 source refs
  (`VanillaBlockEntityAnimOracle`, `VanillaBlockEntityRenderOracle`).
- `net.minecraft.client.renderer.block.dispatch`: 7 source refs, 1
  first-error (`VanillaGallerySelectionOracle`, 25 errors).
- `net.minecraft.client.renderer.state.level`: 3 source refs, 1
  first-error (`VanillaBlockEntityRenderOracle`).
- `java.lang.classfile` (+ `.instruction`): `SeedAgent` (5 errors),
  `VanillaFrameLimiterOracle` (7 errors) — the Class-File API is a
  JDK 22+ API; the measurement compiles at `--release 21` (1.21.11's
  `java_major`), and `oracle_harness.py:30` uses the same release, so
  this is a real 1.21.11 failure, not a measurement artifact. A third
  file references it behind other errors.
- `FloatProviders`: `VanillaValueProviderOracle` (1 error).

**G8 — The long tail (99 files after the groups above and the cascades
are set aside: 75 missing-symbol + 9 no-override + 6 other + 5
constructor-mismatch + 4 incompatible-types).** Per-API translation,
verified against the 1.21.11 jar per file. 26 of the 99 have exactly 1
error. The cheapest high-leverage single repair in the whole batch is
here: `VanillaFaceSupportOracle` — 1 error
(`CubeVoxelShape(DiscreteVoxelShape)` has protected access in 1.21.11
— same error as 1.21.10, 26.1.2, 26.1.1, 26.1 and 26.2), blocking 19
drivers, plus 5 cascade oracles depend on it. `ParityFont` (8 drivers,
1 error — vs 2 in 1.21.10), `VanillaRegistryCollisionScanOracle`
(8 drivers, 2 errors — ServerLevel ctor first, then the
PalettedContainerFactory gap), `VanillaSuffocationOracle` is GONE
from this list (it passes now — the template adapted it: "1.21.11 has
no causes_suffocation tag: the default rule is motion-based"),
`VanillaCollisionContextOracle` (5 drivers, 4 errors — no-override,
same as 1.21.10) and `VanillaSoundRig` (4 drivers, 2 errors — own
errors here, and the anchor of the restored `VanillaMusicOracle`
cascade) are the other one-or-two-error blockers.
`VanillaBlockUpdateOracle` (38 errors, 6 drivers) anchors 4 cascade
oracles. Notable single-file causes in the tail:
`VanillaItemUseOracle` is GONE (it passes now — 1.21.11's
`LivingEntity` gained the `getTicksUsingItem(float)` overload that
1.21.10 lacks, javap-verified on both jars),
`VanillaPackResourcesOracle` is GONE (the 1.21.11 source no longer
calls the absent `PackResources.forEach`),
`VanillaStructurePlacementOracle` / `VanillaStructureStartOracle` /
`VanillaWorldSpawnOracle` (`RandomState.create` overloads),
`VanillaNaturalSpawnOracle` (`ChunkPos.pack()` — no such method in
1.21.11, plus 29 more errors; the 1.21.11 source is the UNADAPTED
26.3-form — see G10), `VanillaRecipeOracle`
(`Recipe.assemble` signature), `VanillaConversionTagsOracle`
(`Brain<>` type inference), `VanillaSectionMeshOracle` (45 errors, the
largest tail file), `VanillaTickScript` (23 errors, lwjgl.sdl),
`VanillaChatScreenOracle` (23 errors), `VanillaRedstoneCatalogOracle`
(10 errors, down from 36 — the gamerules/gizmos/entity-package errors
are gone).

**G9 — Cascades (15 oracles on client cp, 13 on server).** Clean files
failing only via a broken helper: via `VanillaFaceSupportOracle` (5) —
`VanillaBlockCollisionScanOracle`, `VanillaDynamicSupportOracle`,
`VanillaEntityCollisionOracle`, `VanillaPistonShapeOracle`,
`VanillaWorldBorderOracle`; via `VanillaBlockUpdateOracle` (4) —
`VanillaEnchantDurabilityOracle`, `VanillaFurnaceOracle`,
`VanillaOpenedLootOracle`, `VanillaPortalOracle` (`VanillaFurnaceOracle`
is a cascade again here — in 1.21.10 it had own errors; in 1.21.11 its
own errors are gone and it fails only via BlockUpdate);
via `VanillaMenuOracle` (2) — `VanillaPlaceRecipeOracle`,
`VanillaRecipeBookOracle`; via `VanillaAnchorOracle` (1) —
`VanillaHeightProviderOracle`; via `VanillaTextDisplayOracle` (1) —
`VanillaTextDisplayTransformOracle`; via `VanillaContainerScreenOracle`
(1, client only) — `VanillaSlotHoverOracle`; via `VanillaSoundRig`
(1, client only) — `VanillaMusicOracle` (the 26.1.2 chain restored:
in 1.21.10 both had own errors). The 1.21.10
`VanillaHelmetDamageOracle` cascade is gone (it is a single-cause
ServerLevel file here). These pass for free once their helpers are
repaired — do not schedule them separately.

**G10 — Re-ported 26.3-form oracle sources (17 files; E6-chain
inheritance, NOT a 1.21.11 re-port).** 16 of the 17 1.21.10 G10 files
still fail in 1.21.11; the exception is `VanillaFirstPersonOracle`,
which PASSES on client + both here — the 1.21.11 source is adapted
("1.21.11 has the classic swing model: there is no SwingState and no
FirstPersonHandsAndItems", lines 57-58). Of the 16 still failing, 8 are
byte-identical to the 1.21.10 files (`VanillaBasicFeatureOracle`,
`VanillaFeatureGraphOracle`, `VanillaFeaturePlacementOracle`,
`VanillaFeatureSortOracle`, `VanillaFillLayerOracle`,
`VanillaReplaceBlockOracle`, `VanillaScatteredOreOracle`,
`VanillaSteppedInterpolationOracle`) and 8 differ in source but carry
the same 26.3-form signatures and fail same-or-better:

- Same error count as 1.21.10 (14 files): `VanillaBasicFeatureOracle`
  (2), `VanillaBiomePlacementOracle` (8), `VanillaBlockTransformerOracle`
  (3), `VanillaFeaturePlacementOracle` (3), `VanillaFeatureSortOracle`
  (1), `VanillaFillLayerOracle` (1), `VanillaReplaceBlockOracle` (1),
  `VanillaScatteredOreOracle` (1), `VanillaBrewingOracle` (14),
  `VanillaBrewingFuelOracle` (9), `VanillaDecorationRegionOracle` (16),
  `VanillaFeatureCatalogOracle` (11), `VanillaFeatureGraphOracle` (4),
  `VanillaFeatureRegionOracle` (15), `VanillaSteppedInterpolationOracle`
  (16).
- Better than 1.21.10 (1 file): `VanillaTreeFeatureOracle` (16 → 15).
- Passes new (1 file): `VanillaFirstPersonOracle` (25 errors →
  client+both PASS).
- `VanillaSimpleBlockOracle` still compiles on all cps (its 1.21.11
  adaptation — `PalettedContainerFactory` form — passes, as in
  1.21.10).

The re-port signature is the 26.3 unconfigured-Feature form:
`Feature.DIRECT_CODEC` (absent from 1.21.11 — only
`ConfiguredFeature.DIRECT_CODEC` exists), no-arg `new Feature()`
(1.21.11's `Feature` takes only `Codec<FC>`), the 4-arg
`place(WorldGenLevel, ChunkGenerator, RandomSource, BlockPos)`
(1.21.11's abstract `place` takes `FeaturePlaceContext`), and
`FeaturePlacer`-based placement. The non-Feature re-ports use 26.3-only
APIs throughout: the `BrewingFuel` component (0 jar entries),
`net.minecraft.core.component.BlockTransformer`, the
`BrewingInput`/`BrewingRecipe`/`PotionIngredient`/`RecipePropertySet`
crafting APIs, `PositionAndRotation` (0 jar entries, in
`VanillaSteppedInterpolationOracle`), and `LivingEntity.SwingState`
(0 jar entries — the 1.21.10 `VanillaFirstPersonOracle`'s failure;
the 1.21.11 source is adapted past it). The 26.1-adapted
forms remain the repair template for the Feature files; the
1.21.7-adapted forms (B0-O3) are the template for the rest — e.g.
1.21.7's `VanillaBrewingFuelOracle` documents "1.21.7 has no
`brewing_fuel` component: fuel is any item in the tag, 20 uses, and
the brew time is a flat 400 ticks".

The INVERSE case — the child fixed what the template did not — is
`VanillaNaturalSpawnOracle` (22 → 30 errors, worse): 1.21.10's port
adapted it (11-arg ServerLevel super call with `null` RandomSequences,
`GameRules` from `world.level`, `ResourceLocation`), while 1.21.11's
source is the unadapted 26.3-form (10-arg super, `gamerules` import,
`Identifier`). The same pattern, smaller, in the other six error-count
growths (`VanillaBlockEntityTickerOracle` 1 → 4,
`VanillaGenerationTickOracle` 1 → 2, `VanillaMaterialStageOracle`
26 → 28, `VanillaSpawnRulesOracle` 3 → 6,
`VanillaStructurePlacementOracle` 16 → 17, `VanillaTickOracle`
12 → 14) — all seven growths are source-delta effects, none jar-delta.

26 oracles are never referenced by any driver; 11 of them fail on all
three classpaths (`ComponentWireCapture`, `EntityTranscript`,
`OfflineLanHost`, `RideTranscript`, `VanillaGallerySelectionOracle`,
`VanillaLootFillOracle`, `VanillaMover`, `VanillaSceneMotionOracle`,
`VanillaSlotHoverOracle`, `VanillaStructurePlacementOracle`,
`SeedAgent`) — vs 12 in 1.21.10 (`VanillaOptionsTableExporter` now
passes). Three of the eleven are referenced by non-driver scripts
(`OfflineLanHost` in `tools/vanilla_client.py`, `RideTranscript` in
`tools/cart_portal_player_e2e.py`, `VanillaMover` in
`tools/anticheat_e2e.py`); `EntityTranscript` appears only in a comment
in `test_server_entities_parity.py`. The other seven have no reference
anywhere in `tools/`.

## Driver impact

Driver→oracle map built by parsing all 648 drivers (`.java` basenames +
oracle-name string literals passed to `oracle_harness`). Each driver's
oracle set was checked against the classpath its own path uses
(oracle_harness → both, direct `jars['server']` → server, client jar →
client, both jars → both). 20 drivers reference an oracle without a
detectable classpath (they compile via a shared `vanilla.compile(...)`
helper); those were judged against the `both` superset — 14 blocked, 6
runnable (1.21.10: 15 blocked, 5 runnable).

| Driver state | Drivers |
|---|---|
| **Blocked: ≥1 oracle fails to compile under its path's cp** | **289** |
| Oracles compile under their path's cp (can yield a measurement when run) | 105 |
| No oracle reference (server-e2e, data-only, wrappers) — not oracle-blocked | 254 |
| **Total** | **648** |

- 359 drivers have no oracle blocker (105 + 254), vs 334 for 1.21.10
  (80 + 254), 417 for 26.1.2 (163 + 254), 417 for 26.1.1, 426 for 26.1
  and 509 for 26.2 and 321 for 1.21.7 pre-repair (67 + 254). The 254
  no-oracle drivers were never oracle-blocked, but that does not mean
  they pass: they run real servers / the game / data diffs and have
  their own failure modes.
- Blocked by classpath: server 158, both 85 (plus 14 no-flag), client
  32 (1.21.10: 168 / 98 / 33, plus 15 no-flag on both).
- The highest-leverage repairs are shared helpers: `VanillaFaceSupportOracle`
  blocks 19 drivers (1 error — cheapest), `VanillaAnchorOracle` 9
  (10 errors, densityfunction — G3), `VanillaRegistryCollisionScanOracle`
  8 (2 errors), `VanillaProtocolClient` 8 (39 errors),
  `ParityFont` 8 (1 error — vs 2 in 1.21.10), `LocatorBarClient` 8
  (17 errors), `VanillaDispenserOracle` 6 (19 errors, EntityTypes —
  G4), `VanillaBlockUpdateOracle` 6 (38 errors, plus 4 cascades),
  `VanillaMaterialConditionOracle` 5 (76 errors, densityfunction —
  G3), `VanillaCollisionContextOracle` 5 (4 errors),
  `VanillaSoundRig` 4 (2 errors), `KnockbackClient` 4 (7 errors),
  `VanillaVehicleContainerOracle` 4 (5 errors), `VanillaMenuOracle` 4
  (8 errors, plus 2 cascades), `VanillaNaturalSpawnOracle` 4 (30
  errors). Fixing the top few one-error helpers unblocks ~50
  driver-oracle pairs (overlaps exist) — and the G0 ServerLevel fix
  alone rescues 28 files outright.
- Compiling ≠ running: the 105 "runnable" drivers are runnable by
  javac only. No driver was run in this batch (scope decision above),
  so the runtime-failure layer the 1.21.7 sample found (a compiling
  oracle throwing at runtime) is unmeasured for 1.21.11 — with ONE
  concrete exception in the other direction: `VanillaComponentPatchOracle`
  compiles clean AND no longer carries the runtime crash (see §5,
  B0-O10 — the 1.21.7 repair form IS in this tree). Every repaired
  oracle needs an end-to-end driver run to confirm it yields a
  measurement.

## The 9-harness check (1.21.7's table, re-measured for 1.21.11)

Of the 9 harnesses the 1.21.7 plan (§2) records as dying on oracle javac
failures in CI run 44441686979, **all 9 pass on at least one classpath
for 1.21.11 at measurement time** (vs 2/9 for 1.21.10, 6/9 for 26.1.2,
26.1.1 and 26.1; for 26.2 all 9 passed):

| Harness | Oracle | 1.21.11 status |
|---|---|---|
| chunk_generation | VanillaChunkGenerationOracle | **PASS** (all cps) — the 1.21.11 source no longer calls the `ServerChunkCache` ctor directly |
| command_exec | VanillaCommandExecOracle | **PASS** (all cps) — the 1.21.11 source has no authlib.services/EntityTypes refs and uses the gamerules/permissions packages that exist here |
| component_patch | VanillaComponentPatchOracle | **PASS** (all cps) — and no runtime crash: the `Optional.empty()` form is in this tree (see §5, B0-O10) |
| datafix | VanillaDataFixOracle | **PASS** (all cps) — rewritten without the filefix import |
| first_person | VanillaFirstPersonOracle | **PASS** (client + both; server fails on client packages — expected) — adapted to the classic swing model |
| food | VanillaFoodOracle | **PASS** (all cps) — the gamerules package exists in 1.21.11 |
| item_model | VanillaItemModelOracle | **PASS** (client + both; server fails on client packages — expected) — adapted past the model.geometry/sprite APIs |
| options | VanillaOptionsOracle | **PASS** (client + both; server fails on client packages — expected) |
| resource_pack | VanillaPackResourcesOracle | **PASS** (client + both; server fails on client packages — expected) — the 1.21.11 source no longer calls the absent `PackResources.forEach` |

For 1.21.7, 8 of these passed only AFTER Batch 0 repairs. For 1.21.11
all 9 pass with no repair — Batch 9's command_parse run is NOT
oracle-blocked (its oracle, `VanillaCommandExecOracle`, compiles on
all three classpaths), and the `command_parse`/`command_exec`/`food`/
`component_patch` drivers — the ones that yielded the 1.21.7 sweep's
first real parity measurements — are oracle-clean from the start here.
What Batch 9 still needs is prepared horizon-client data and a built
probe (see the prepared-data section below).

## Comparison against the 1.21.10 table

The oracle sources DIFFER between the 1.21.10 and 1.21.11 trees (168 of
391 files differ by `diff -rq`; the 1.21.11 set is the template's own,
the 1.21.10 set the E6-chain set), and the jar delta is one API step
(1.21.10 → 1.21.11: gamerules, entity sub-packages, permissions,
gizmos, world.attribute all arrived). The measured table was diffed
against `1.21.10/batch-0-oracles.json` file by file on every field
(server / client / both status, first-error class, first-error message,
error count) — `evidence/diff-1.21.10.json`:

**Deviations: 132 of 391 files.** Identical: 259. New pass: 23 files
that failed on all three classpaths in 1.21.10 pass on at least one
here. New fail: ZERO — no file passes new-fail vs 1.21.10. Changed
error (same fail-all status, different first error or count): 109
files. Among the 264 files that fail-all in BOTH runs, error counts
shrank in 98, grew in 7, and stayed the same in 155 (4 changed-error
files are cascades with no error count in one or both runs). 1.21.11
is better-or-equal on every single file except the 7 source-delta
growths.

New-pass breakdown (23 files):
- Jar-delta (byte-identical source; the 1.21.11 jar supplies what
  1.21.10's lacked): 7 files — `VanillaEntityParticlesOracle`
  (entity sub-packages), `VanillaPistonPushOracle` (monster.zombie),
  `VanillaZombieCureSoundOracle` (monster.zombie),
  `VanillaItemUseOracle` (the `getTicksUsingItem(float)` overload,
  javap-verified present in 1.21.11, absent in 1.21.10),
  `VanillaInlineObjectOracle`, `VanillaOptionsTableExporter`,
  `VanillaServerPropertyNumbersOracle`.
- Source-delta (the 1.21.11 oracle source was adapted during the
  template port): 16 files — `VanillaChunkGenerationOracle`,
  `VanillaCommandExecOracle`, `VanillaDataFixOracle`,
  `VanillaEmitterParticlesOracle`, `VanillaEntityComponentsOracle`,
  `VanillaEnvironmentAttributeModifierOracle`,
  `VanillaEnvironmentSystemOracle`, `VanillaEnvironmentTimelineOracle`,
  `VanillaFirstPersonOracle`, `VanillaFoodOracle`,
  `VanillaItemModelOracle`, `VanillaLevelShapeOracle`,
  `VanillaPackResourcesOracle`, `VanillaSheepDyeOracle`,
  `VanillaSoundEventOracle`, `VanillaSuffocationOracle`. Several are
  combined effects (e.g. `VanillaFoodOracle` still imports
  `world.level.gamerules` — a package that exists only in 1.21.11 —
  and `VanillaSoundEventOracle`'s 1-arg `SoundManager` call passes
  because 1.21.11's ctor is 1-arg where 1.21.10's is 2-arg,
  javap-verified on both jars).
- The 23 new-pass files include all four 9-harness oracles that failed
  in 1.21.10 but pass here (`VanillaChunkGenerationOracle`,
  `VanillaCommandExecOracle`, `VanillaDataFixOracle`,
  `VanillaFoodOracle`) plus `VanillaFirstPersonOracle`,
  `VanillaItemModelOracle` and `VanillaPackResourcesOracle`.

Biggest error-count shrinkage among fail-all-in-both files:
`VanillaMobAiOracle` 103 → 38 (−65, gizmos + gamerules + entity
packages now exist), `VanillaFinalizeSpawnOracle` 48 → 13 (−35),
`VanillaTradingOracle` 86 → 51 (−35), `VanillaSculkOracle` 51 → 23
(−28), `VanillaRedstoneCatalogOracle` 36 → 10 (−26),
`VanillaHudOracle` 100 → 77 (−23), `VanillaSquidClientOracle` 22 → 2
(−20), `VanillaSectionMeshOracle` 48 → 45 (−3).
Error-count growths (7, all source-delta — the template's sources are
less adapted than 1.21.10's port-adapted ones in these files):
`VanillaNaturalSpawnOracle` 22 → 30 (+8), `VanillaSpawnRulesOracle`
3 → 6 (+3), `VanillaBlockEntityTickerOracle` 1 → 4 (+3),
`VanillaMaterialStageOracle` 26 → 28 (+2),
`VanillaTickOracle` 12 → 14 (+2),
`VanillaGenerationTickOracle` 1 → 2 (+1),
`VanillaStructurePlacementOracle` 16 → 17 (+1).

Cause attribution for the deviations: the overwhelming majority are
jar-delta effects (the 1.21.11 jar has APIs the 1.21.10 jar lacks:
gamerules package, entity sub-packages, permissions package, gizmos,
world.attribute, the `getTicksUsingItem(float)` overload, the 1-arg
`SoundManager` ctor) plus source-delta effects where the 1.21.11
oracle sources differ from 1.21.10's (168 files — e.g.
`VanillaCommandExecOracle` without the authlib.services import and
EntityTypes reference, `VanillaDataFixOracle` without the filefix
import, `VanillaFirstPersonOracle` adapted to the classic swing
model). Of the 98 shrinkages, 44 are on byte-identical sources
(pure jar-delta) and 54 on changed sources. No deviation is a
measurement difference: the method, script, JDK and fixpoint rules
are identical to the 1.21.10 run, except the jsr305 path noted in §0
(which cannot change any file's verdict — it only prevents the
latent javac crash).

Consequence: there is NO O9-style "fails new or worse" group for this
run in the 1.21.10 sense — new-fail is zero and the only worsenings
are 7 small source-delta growths. The O9 group itself (the 17
re-ported 26.3-form files) reproduces in substance — 16 still failing,
1 passing new — as G10 above. The new story this run is the inverse:
the template's oracle sources are PARTIALLY 1.21.11-form (19 files
carry explicit `1.21.11` adaptation comments, and 16 new passes are
source-delta), while 1.21.10's port-adapted a few files the template
never did (`VanillaNaturalSpawnOracle` most visibly).

## Driver defects found (Batch 0 scope)

1. **`client_libraries()` omits jsr305 — and so does the server path,
   and the jar is missing from the tree (latent, worse than 1.21.10).**
   `versions/1.21.11/tools/test_model_baking_parity.py:client_libraries()`
   builds the client classpath from the manifest only; it does not
   append `tools/jsr305-3.0.2.jar`. No `tools/*.py` file in the 1.21.11
   tree mentions jsr305 anywhere — `test_rust_parity.py` does NOT carry
   the 1.21.7 server-path fix (`d7a14e91`) that 1.21.10's tree has at
   lines 112-115, and `oracle_harness.py` / `lab.py` have no jsr305
   handling either. The jar itself is absent from the tree
   (`versions/1.21.11/tools/jsr305-3.0.2.jar` does not exist; the port
   workdir's copy, same sha1 `25ea2e8b…`, is what the measurement
   used). No manifest library contains `javax.annotation.Nullable`
   (verified by listing all 99 library jars). Without jsr305, javac
   crashes (`CompletionFailure: class file for
   javax.annotation.Nullable not found`, rc=4) when compiling an oracle
   that completes a `Nullable`-referencing symbol — a latent crash
   across BOTH the oracle_harness drivers and the direct server-path
   drivers here, not a deterministic one. The measurement adds jsr305
   to the client cp itself, so this did not affect the numbers above;
   the server cp ran without it and did not crash (the crash is
   latent, as in prior runs). Fix direction (follow-up, not this
   batch): restore the jar to `tools/` and append it in both
   `client_libraries()` and the server-path library list — two points,
   fixes all users.
2. **MobAi driver gates nothing 1.21.11 cannot run — except the age
   lock.** The 1.21.7 Batch 0 fix gated 27 scenarios 1.21.7 cannot run
   (copper golem, spears, age lock); that gating is NOT in the 1.21.11
   tree (`test_mob_ai_parity.py` runs all 278 scenarios ungated).
   Against 1.21.11's own jar: copper golem EXISTS (11 jar entries —
   the golem scenarios are runnable, the 1.21.7 concern is withdrawn
   for them), spears EXIST (`WOODEN_SPEAR` … `GOLDEN_SPEAR` on `Items`,
   javap-verified — the 4 spear scenarios at lines 60-74 are runnable,
   the 1.21.10 concern is withdrawn for them), but the AgeableMob age
   lock is ABSENT (no `setAgeLocked`/`ageLocked` on `AgeableMob` —
   only `setAge`, javap-verified). The 16 age scenarios
   (`{pig,cow,horse,turtle}_age_{-2,2}_noai_{False,True}`, lines
   90-94) set an age the version cannot lock, so the age ticks
   mid-scenario and the comparison premise fails. 16 scenarios will
   fail at runtime for want of the gating the 1.21.7 tree carries.
   Recorded as slice item O11 (minor).

## `.local/minecraft/1.21.11/` setup (recipe for later batches)

`verify_artifacts()` needs `manifest.json` (pinned sha1), `jars/client.jar`
+ `jars/server.jar` (manifest sha1s or `remapped.json` digests), the inner
`jars/server-game.jar` (bundle checksum or `remapped.json`), and
`libraries/server/` + `libraries/client/` (sha-verified). No worktree had
it in the sweep worktree; unlike 26.1.2 (built fresh with `lab.py
prepare`), 1.21.11 was copied from the port workdir's already-prepared
directory (220 MB, sha-verified by the gate itself):

```sh
cp -r ~/workspace/mcir-work/e6-1.21.11-behaviour/versions/1.21.11/.local/minecraft/1.21.11 \
  ~/workspace/mcir-work/sweep-1.21.11/versions/1.21.11/.local/minecraft/1.21.11
cd ~/workspace/mcir-work/sweep-1.21.11/versions/1.21.11
python3 -c "import sys; sys.path.insert(0,'tools'); import test_rust_parity as t; t.verify_artifacts(); print('GATE OK')"
```

Result: named client jar (10,291 classes) + named inner server jar
(7,163 classes), 39 server + 60 client libraries, `manifest.json` sha1
`812ee7fdfdf648bb872f7a1dca33718de4e786b4` (the pinned one in
research-lock.json). Verified: the gate passes. Never commit `.local`.
(The `.local` jars are remapped copies — sha1s `1b09cd18…` /
`99f0600a…` — whose class lists are identical to the brief's e6-deltas
named jars `29bddf14…` / `7f0d85c1…`; same mapping, different remapper
bytes. If a fresh build is ever needed instead of the copy, the
launcher-style source dir is
`~/workspace/mcir-work/e6-deltas/versions/1.21.11/launcher/` and the
server bundler jar is `…/jars/server.jar` — same `lab.py prepare`
invocation as the 26.1.2 run. Note 1.21.11's prepare includes a remap
stage — the version manifest's `downloads` has mappings — unlike the
26.1.x runs.)

## Prepared-data setup (for Batch 9 and the B3-G22 re-check)

Prepared 1.21.11 **minecraft** data now exists in the sweep worktree
(see §6 above, copied by this batch from the port workdir). Prepared
1.21.11 **horizon-client** data does NOT exist anywhere on this VM —
unlike the 1.21.10 run, where the port workdir held a complete 523 MB
copy. Checked and absent:

- `~/workspace/mcir-work/e6-1.21.11-behaviour/versions/1.21.11/.local/`
  (260 MB): has `minecraft/`, `datagen-1.21.11/`, `parity-runner/`,
  `command-parse-parity/` and others — NO `horizon-client/` directory.
- `~/workspace/mcir-work/1.21.11/`, `1.21.11-rewire/`,
  `1.21.11-adapters/`, `1.21.11-fix/`, `e6-deltas/`, `26.1.x/`:
  no `horizon-client/1.21.11` directory at any depth searched.
- `~/workspace/minecraft-in-rust/.local/`: only `mapping-frontend/`
  (the jar-remap output); the repo checkout's `versions/1.21.11/` has
  no `.local` at all.
- The sweep worktree's own `.local/` holds only `minecraft/` (this
  batch) and `measure-oracles/` — no sibling batch is building
  horizon-client data (no `.horizon-prepare.lock` present).

What Batch 9's `command_parse` run needs, and what is present:

| Need | Present? |
|---|---|
| Prepared minecraft data (`.local/minecraft/1.21.11/`, gate OK) | YES (built by this batch) |
| `VanillaCommandExecOracle` / command_parse oracles compile | **YES — all 9 harness oracles pass** (see the 9-harness check above); the run is NOT oracle-blocked |
| Prepared horizon-client data (`.local/horizon-client/1.21.11/`) | **NO — nowhere on the VM**; must be built with horizon-prepare (commands below) |
| Built `client_probe` binary (`cargo build --locked -p horizon-client-lib --example client_probe`) | NO — only the `cargo check` was attempted (H01, PENDING); a full probe build is Batch 9's job (CI-or-quiet-window under this VM's memory budget) |

What the B3-G22 re-check (client loading its core tables with prepared
1.21.11 data) needs: the same `.local/horizon-client/1.21.11/`
directory (registries, items, movement-properties, landing-definitions,
…) plus a built client. The data does NOT exist — it must be built
first (commands below). The B3-G22 re-check itself is Batch 3's, not
this batch's; this batch records only that its data prerequisite is
NOT met.

Exact commands to produce the missing data (mirroring the 1.21.10 run):

```sh
cd ~/workspace/mcir-work/sweep-1.21.11/versions/1.21.11
# Build horizon-prepare (taskset -c 0, -j 1):
RUSTUP_HOME=~/workspace/tools/rustup CARGO_HOME=~/workspace/tools/cargo \
  PATH=~/workspace/tools/jdk-25/bin:~/workspace/tools/cargo/bin:$PATH \
  taskset -c 0 cargo build --locked -j 1 -p horizon-prepare --release
# Run it against the prepared minecraft data:
./target/release/horizon-prepare --minecraft-dir .local/minecraft/1.21.11
# Then check for the expected asset-store gaps only:
./target/release/horizon-prepare --check
```

Caution (from the 26.1.1 run): a first horizon-prepare run can fail
transiently at "Exporting movement and attributes" (`Exporter
VanillaBlockLandingOracle failed:
"java.lang.reflect.InvocationTargetException"`, with an empty
`.local/horizon-client-motion/export.log`) when run concurrently with
heavy javac work — retry on a quieter VM before assuming a real
exporter defect. (For 1.21.11 the oracle itself compiles cleanly on
all three classpaths, so the same advice applies.)

## What the numbers mean

- Batch 0's repair load is **264 files** — smaller than 1.21.10's 287
  but still the second-largest of any sweep (26.1.2: 202, 26.1.1: 202,
  26.1: 199, 26.2: 133, 1.21.7 pre-repair: 297). But the profile is
  front-loaded, not diffuse: two mechanical groups (G0 ServerLevel
  ctor, 85 files with 28 single-cause; G2 LocalPlayer ctor, 18 files
  with 3 single-cause) plus the holder translation (G4 EntityTypes,
  30 files — mostly mechanical) rescue a large share with
  signature-level work. The densityfunction (G3, 17 files),
  renderpearl/material.rule (G5, G7) and model-API groups are the
  genuinely hard ones: 26.3-only APIs with no 1.21.11 equivalent to
  translate to import-by-import — those oracles need rewrites against
  1.21.11's actual APIs (or, for StructureDensity, stay unrepaired by
  design). The 1.21.7 Batch 0 repairs (B0-O1–O4) are the repair
  template for much of this — but they must be RE-DERIVED for 1.21.11,
  not copied: the ServerLevel ctor is 11-arg here (not 1.21.7's
  12-arg), the GameRules package is `world.level.gamerules` (not
  1.21.7's `world.level`), and 1.21.11 has APIs 1.21.7 lacks (copper
  golem, spears, `PalettedContainerFactory`, `RandomSequences` on the
  level, the permissions package, environment attributes).
- The template's oracle sources are PARTIALLY 1.21.11-form — 19 files
  carry explicit `1.21.11` adaptation comments and 16 new passes are
  source-delta — which is why this run improves on 1.21.10 instead of
  degrading further. But the adaptation is incomplete and uneven: the
  17 re-ported 26.3-form files are still 26.3-form, and in 7 files the
  1.21.10 port adapted further than the template did
  (`VanillaNaturalSpawnOracle` most visibly). The repair follow-up
  should take the template's own adaptations as the pattern and extend
  them, and should port 1.21.10's `VanillaNaturalSpawnOracle`
  adaptation BACK to the template.
- Sequencing for batches 3–12: 105 oracle-clean drivers can run once
  their probes build and horizon-prepare data exists in the sweep
  worktree (it does not yet — see above); the 289 blocked drivers wait
  on Batch 0 repairs, shared helpers first
  (`VanillaFaceSupportOracle` unblocks 19 drivers with a 1-error
  repair; `ParityFont` and `VanillaRegistryCollisionScanOracle` 8
  each). The 254 no-oracle drivers were never oracle-blocked. In
  practice this sweep's batches run statically (javap + source +
  data) and record driver runs as follow-up needs — the VM's 6 GB
  budget under multi-agent load makes probe builds (client_probe OOMs
  single-threaded) a CI-or-quiet-window job, not a sweep-batch job.
- The 1.21.11-vs-26.3 API deltas this measurement exposes are
  themselves sweep input: the 11-arg `ServerLevel` ctor (with
  `RandomSequences` on the level), the 7-arg `LocalPlayer` ctor, the
  `GameRules`-in-`world.level.gamerules` shape, the SPLIT entity
  packages (`vehicle.boat`/`monster.zombie`/`projectile.arrow` — the
  opposite of 1.21.10's flat packages), the `permissions`/`gizmos`/
  `world.attribute` APIs (all present in 1.21.11, absent in 1.21.10),
  the `EntityType`/`BlockEntityType` holder shape (no `EntityTypes`
  move), `Level.fuelValues()` (abstract in 1.21.11), the
  `ServerChunkCache`/`ChunkMap`/`Gui`/`Feature` constructor forms, the
  absence of the `densityfunction`, `renderpearl`, `material.rule`,
  `clock` and `filefix` APIs, the advancements `predicates`/`triggers`
  packages (1.21.11 has only `critereon`), the `ConfiguredFeature`
  form (1.21.11 has no `Feature.DIRECT_CODEC`, no-arg `Feature()` or
  4-arg `place`), the absence of the `BrewingFuel`/`BlockTransformer`
  components and of `LivingEntity.SwingState`, the `PackResources`-in-
  `server.packs` shape without `forEach`, the 1-arg `SoundManager`
  ctor, the `getTicksUsingItem(float)` overload, and the
  `MaterialBaker` / registry-lookup abstract methods. Family batches
  should check the port's Rust against 1.21.11's forms of these, not
  26.3's — and not 1.21.10's either, where the two differ (gamerules,
  entity packages, permissions, gizmos, world.attribute are all
  1.21.11, not 1.21.10; `SoundManager` and `getTicksUsingItem` also
  differ).

## Re-check verdicts

### 1.21.10 re-checks (PRIMARY — 13 items, all decided)

| Item | 1.21.10 status | 1.21.11 verdict | 1.21.11 item |
|---|---|---|---|
| O1 LocalPlayer ctor group (20 oracles) | gap | **REPRODUCES** — 18 files (FirstPerson + SheepDye adapted out); target form is 7-arg here (same as 1.21.10); 3 single-cause (vs 1) | O1 |
| O2 jsr305 client-classpath crash (latent) | gap | **REPRODUCES, worse** — `client_libraries()` still omits jsr305 AND the server path omits it too (the 1.21.7 fix is not in this tree) AND the jar itself is absent from the tree | O2 |
| O3 fuelValues() + missing overrides | gap | **REPRODUCES** — same 5 + 1 files; missing-override first-error count 1 (vs 1) | O3 |
| O4 densityfunction absent (17 oracles) | gap | **REPRODUCES** — identical 17-file membership; MaterialStage worse (26 → 28 errors), StructurePlacement worse (16 → 17) | O4 |
| O5 EntityTypes holder move (32 oracles) | gap | **REPRODUCES** — 30 files (32 minus CommandExec + ItemModel, both adapted out); same direction as 1.21.10 (both predate the move) | O5 |
| O6 other 26.3-only packages | gap | **REPRODUCES, narrowed** — renderpearl 8, material.rule 5, advancements 2 (was 3), authlib.services 1 (was 2), lwjgl.sdl 1, clock 13, model geometry/sprite 11, classfile 3; the 1.21.11-era packages (gamerules 26 refs, entity sub-packages 33, world.attribute 31, gizmos 4, permissions 12) all exist — ZERO errors from them; filefix GONE (DataFix adapted) | O6 |
| O7 287 fail-all (73.4%); 314 drivers blocked | gap | **REPRODUCES, better** — 264 fail-all (67.5%); 289 drivers blocked; 23 files pass new vs 1.21.10, 0 fail new | O7 |
| O8 runtime-failure layer unmeasured | gap | **REPRODUCES** — 105 "runnable" drivers are runnable by javac only; no driver was run; the one concrete runtime crash (ComponentPatch Removed) is FIXED in this tree | O8 |
| O9 re-ported 26.3-form sources (17 files) | gap | **REPRODUCES, changed** — 16 still failing (FirstPerson passes new on client); 8 byte-identical to 1.21.10's, 9 differ in source; the template's sources are partially 1.21.11-form elsewhere (19 files carry explicit adaptations) | O9 |
| O10 ServerLevel ctor group (86 oracles) | gap | **REPRODUCES** — 85 files, 28 single-cause (vs 23); the 11-arg ctor is javap-identical to 1.21.10's | O10 |
| O11 MobAi driver gates nothing 1.21.10 cannot run (20 scenarios) | gap | **REPRODUCES (partial)** — no gating in this tree; copper golem EXISTS (runnable — withdrawn for that part) and spears EXIST (4 scenarios runnable — withdrawn) but the age lock (16 scenarios) is absent and ungated | O11 |
| O12 ComponentPatch Removed runtime crash | gap | **FIXED** — the 1.21.11 oracle uses the `Optional.empty()` form (the 1.21.7 repair form); no `Class.forName("…Removed")` anywhere in the file | O12 (fixed) |
| O13 format-changing oracle repairs (closed) | closed | **PREMISE CHANGED** — 1.21.10's was closed because no format-changing repairs exist in that tree; 1.21.11's tree DOES carry format-relevant adaptations (Suffocation rule names, SoundEvent attenuation formula) whose probe-side matching is unverified — re-minted as a gap item | O13 |

### 1.21.7 re-checks (SECONDARY — 10 items, all decided)

| Item | 1.21.7 status | 1.21.11 verdict | 1.21.11 item |
|---|---|---|---|
| B0-O1 ServerLevel 12-arg ctor group (25 oracles) | fixed | **REPRODUCES** (as the 11-arg form) — the 1.21.7 repair (12-arg) is NOT in this tree and would be wrong here; oracles call the 26.x 10-arg form; 1.21.11's ctor is 11-arg; 85 files carry the error, 28 single-cause | O10 |
| B0-O2 jsr305 client-classpath crash | fixed | **REPRODUCES, worse** — neither the client nor the server path appends jsr305; the jar itself is absent from the tree | O2 |
| B0-O3 1.21.11 carry-over repairs (14 oracles) | fixed | **REPRODUCES (partial)** — the 1.21.7 repairs are NOT in this tree (they live on the 1.21.7 sweep branch); 11 of the 14 pass anyway via the template's own 1.21.11 adaptations + jar deltas (Suffocation, Food, EntityComponents, SheepDye, SoundEvent, ChunkGeneration, DataFix, PackResources, CommandExec, FirstPerson, ItemModel); 3 still fail (SquidClient 2, FinalizeSpawn 13, MobAi 38 errors — all much improved vs 1.21.10's 22/48/103) | O1–O7 (by cause) |
| B0-O4 Anchor + RegistryCollisionScan helpers | fixed | **REPRODUCES** — Anchor still fails (10 errors, densityfunction); RegistryCollisionScan still fails (2 errors, ServerLevel ctor first) | O4 / O10 |
| B0-O5 MobAi driver gates 27 scenarios | fixed | **REPRODUCES (partial)** — no gating in this tree; copper golem EXISTS in 1.21.11 (runnable — withdrawn for that part) and spears EXIST (4 scenarios runnable — withdrawn) but the age lock (16) is absent and ungated | O11 |
| B0-O6 256 fail-all (65.5%); 259 drivers blocked | gap | **REPRODUCES** — 264 fail-all (67.5%); 289 drivers blocked | O7 |
| B0-O7 StructureDensity unrepairable | gap | **REPRODUCES** — densityfunction absent from both 1.21.11 jars; StructureDensity fails with 4 errors, same ruling (verify via bytecode/chunk comparison) | O4 |
| B0-O8 runtime-failure layer | gap | **REPRODUCES** — 105 "runnable" drivers are runnable by javac only; no driver was run | O8 |
| B0-O9 format-changing repairs expose probe assumptions | gap | **REPRODUCES in substance** — the template carries format-changing adaptations (Suffocation "motion_full" rule names, SoundEvent attenuation formula); whether the probes match is unverified (no driver run) — family-batch triage, not Batch 0 fixes | O13 |
| B0-O10 ComponentPatch Removed runtime crash | fixed | **FIXED** — the 1.21.7 repair form (`Optional.empty()`, no Removed reflection) IS in this tree; the oracle compiles clean and carries no runtime crash | O12 (fixed) |

## The client_probe check (H01)

**Verdict: PASS — H01 does NOT reproduce for 1.21.11.** The
coordinator's cargo check finished clean on rerun (see the closure
note below). This batch did NOT run the check itself.

Static evidence (verified by this batch, same as the coordinator's
Step 0): H01 is expected NOT to reproduce for 1.21.11. In 26.1,
`client_probe` failed to compile with E0560: `game_loop_probe.rs`
initialised `FrameContext` fields `focused` and
`exclusive_fullscreen` that 26.1's `FrameContext` does not have, so
the two lines had to be deleted. 1.21.11's `FrameContext`
(`crates/horizon-client-input/src/window.rs:56-64`) has exactly 7
fields — `inactivity`, `iconified`, `now_ms`, `latest_input_ms`,
`has_level`, `has_screen`, `has_overlay` — with NO `focused` and NO
`exclusive_fullscreen` (they are 26.x additions), and the 1.21.11
probe's `FrameContext` literal (`game_loop_probe.rs:152-164`)
initialises exactly those 7 fields. Neither file mentions `focused`
or `exclusive_fullscreen` anywhere. The probe source matches its own
tree and the H01 error cannot occur on the static evidence. The probe
file is unmodified. The `client_probe` target is declared in
`crates/horizon-client-lib/Cargo.toml:201`, not in
`horizon-client/Cargo.toml`.

Check (coordinator-run, `b00/h01-run.sh` — seed from the 1.21.4 port
workdir's warm target, then `taskset -c 0 cargo check --locked -j 1
-p horizon-client-lib --example client_probe` with batch-local
`CARGO_TARGET_DIR=b00/target`):

```
=== seed start 2026-11-25T00:52:54Z ===
seed file list: 3773 files
=== seed done 2026-11-25T00:54:16Z; target size: 4.8G ===
=== cargo check start 2026-11-25T00:54:16Z ===
   Compiling syn v2.0.119
   ...
   Checking horizon-cosmetics v0.1.0 (...)
```

- The log (`b00/h01-cargo-check.log`) stalls at 01:02:53 UTC with last
  line `Checking horizon-cosmetics v0.1.0 (...)` — no EXIT line, and
  no cargo process for it is running (verified 01:22 UTC and again at
  handoff). The stall is in the same window (~01:02–01:03 UTC) in
  which this batch's first measurement run died on the transient
  disk-quota error (see §0) — the coordinator's run was most likely
  killed by the same event. The seed (4.8G target dir) is intact in
  `b00/target/`, so a rerun resumes warm.
- **Coordinator closure (02:13 UTC):** the rerun of `b00/h01-run.sh`
  (seed 01:51:46–01:52:43 UTC, target 5.3G; check 01:52:44–02:13:00
  UTC) finished `Finished 'dev' profile [unoptimized + debuginfo]
  target(s) in 20m 13s` with `cargo check EXIT=0` and 0 errors. The
  probe artifact `target/debug/deps/libclient_probe-7f5c9e82df826d1b.rmeta`
  exists, proving the example was actually checked. H01 is CLOSED as
  PASS — no slice.json item is minted for H01 (it is a check, not a
  defect).

## Per-oracle table

`batch-0-oracles.json` — all 391 files: pass/fail per classpath,
first-error class + message + error count for failures (same shape as
1.21.10's table; first-error attribution prefers the client cp,
falling back to server — the 1.21.10 convention, verified against that
table). Raw big-compile logs (4,786 server-cp / 2,162 client-cp
errors) are in `evidence/` (copies of the measuring workdir's
`.local/measure-oracles/` logs, regenerable with
`docs/sweep/measure-oracles.py`).

## Evidence index

- `batch-0-oracles.json` — per-oracle table (391 files).
- `evidence/big-compile-{server,client,both}.log` — all-files
  big-compile logs (raw javac output).
- `evidence/all-{server,client,both}.log`,
  `evidence/clean-{server,client,both}-{1,2}.log` — fixpoint round logs.
- `evidence/measure-oracles-run.log`,
  `evidence/measure-oracles-run2.log` — measurement driver output
  (first run killed by a transient disk-quota error during the server
  big-compile; second run completed all three classpaths).
- `evidence/measure-oracles-1.21.11.py` — the measurement script copy
  used (1.21.10's `docs/sweep/measure-oracles.py` with the jsr305 path
  repointed at the port workdir's copy — the only change).
- `evidence/group.py`, `evidence/groups.txt` — group derivation script
  + output (error-text and source-pattern matching over the logs).
- `evidence/analyze.py`, `evidence/analyze.txt` — follow-up analysis
  (error stats, single-cause lists, new-pass attribution, re-ported
  17, cascades, long tail).
- `evidence/diff-1.21.10.json` — file-by-file deviation lists vs the
  1.21.10 table (newpass / newfail / changed_err).
- `evidence/server-jar-listing.txt`,
  `evidence/client-jar-listing.txt` — 1.21.11 jar class listings
  (used for the package-absence verifications).
- `h01-cargo-check.log` (coordinator-run; first run stalled 01:02:53
  UTC, rerun PASSED 02:13:00 UTC, EXIT=0, 20m13s, 0 errors) and
  `h01-run.sh` — H01 cargo check output and runner.
- `evidence/rsync-filelist.txt` — file list used for the H01 target-dir
  seed (stable files only, from the 1.21.4 port workdir's warm
  target).
- Jar facts verified by javap on the 1.21.11 named jars:
  ServerLevel 11-arg ctor (identical to 1.21.10's); LocalPlayer 7-arg
  ctor; Level.fuelValues() abstract; EntityType.PLAYER/CHICKEN +
  BlockEntityType.BED statics; Feature ctor `Codec<FC>` only;
  ServerChunkCache 11-arg ctor; SoundManager 1-arg ctor (vs 1.21.10's
  2-arg); `LivingEntity.getTicksUsingItem(float)` overload present
  (absent in 1.21.10); PackResources (in `net.minecraft.server.packs`)
  without forEach; AgeableMob without age-lock methods; Items with
  spears; copper golem present.

## Verdict

**127/391 oracles compile (32.5%); 264 fail on all three classpaths
(67.5%); 289 of 648 drivers are oracle-blocked.** The measurement is
complete and reproducible (fixpoint big-compile, same script and
method as every prior run; the rerun after the transient quota error
completed all three classpaths cleanly). The headline causes are the
ServerLevel constructor (85 files, 28 single-cause) plus the familiar
26.3-form groups (densityfunction 17, EntityTypes 30, LocalPlayer 18,
renderpearl 8) — but this run IMPROVES on 1.21.10 for the first time
in the sweep series (23 new passes, 0 new fails): the 1.21.11 jar
supplies the 1.21.11-era APIs (gamerules, entity sub-packages,
world.attribute, gizmos, permissions) and the template port adapted
19 oracle sources to 1.21.11 in-tree. All 9 harness oracles pass, so
Batch 9's command_parse run is NOT oracle-blocked — but prepared
horizon-client data does not exist anywhere on the VM and must be
built before any driver runs. The 1.21.7 sweep's repairs are NOT in
this tree and must be re-derived for 1.21.11 (11-arg ServerLevel, not
12-arg; gamerules package, not `world.level`) — with two exceptions
already in-tree: the ComponentPatch `Optional.empty()` form (O12
FIXED) and the template's own 1.21.11 adaptations. H01 PASSED on the
coordinator's rerun (cargo check EXIT=0, 20m13s, 0 errors, probe
rmeta present) — the client_probe compiles for 1.21.11, as the
static evidence predicted.
