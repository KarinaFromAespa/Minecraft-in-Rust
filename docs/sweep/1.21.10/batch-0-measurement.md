# Batch 0 measurement: oracle compilability against the 1.21.10 jars

Date: 2026-11-25. Branch: `parity-sweep-1.21.10`. Measured before any repair.
Companion data: `batch-0-oracles.json` (per-oracle table, all 391 files),
`evidence/` (raw big-compile logs, grouping script + output, jar listings,
26.1.2 diff). The measurement script is `docs/sweep/measure-oracles.py` in
the worktree (rerunnable).

**Scope decision (coordinator, per the sweep brief): Batch 0 for 1.21.10 is
measurement + triage ONLY — no oracle repairs — with ONE explicit exception:
the client_probe check (H01, "The client_probe check (H01)" below).** The
oracle `.java` files live in `versions/1.21.10/mods/horizon/`. The 1.21.7
Batch 0 did repair oracles (135/391 pass after repair, up from 94); for
1.21.10 the repair list below is a follow-up decision for the parent.
Batches use static checks (javap + source + data) for drivers whose oracles
don't compile, as 1.21.7 Batch 2 did.

## §0 Execution status

- Measurement ran in two parts: the server classpath completed in the first
  run; a service restart killed the process during the client fixpoint
  (results.json checkpointing kept the server pass intact — the rerun
  re-measured client + both only, and reproduced the killed run's
  client numbers exactly: 276 own-error files, 11 cascade drops).
  Total wall ~60 min across both runs (javac 25.0.5, VM load average
  ~30 during the run — sibling sweep batches and the 1.21.4 port agent
  were active concurrently).
- The `both` fixpoint converged cleanly in 2 rounds (11 cascade drops);
  no transient-filesystem artifact of the 26.1.1 kind occurred.
- The H01 cargo check ran under `taskset -c 0 -j 1` with a batch-local
  `CARGO_TARGET_DIR` (see H01 section): the first run, on a cold
  target cache, was stopped after ~50 min (10 units — syn alone took
  ~40 min under load ~30) and restarted after seeding the target dir
  from the 1.21.4 port workdir's warm target (stable files only,
  `--ignore-existing`); the seeded run finished EXIT=0 in 35m 37s,
  with the lib and example artifacts rewritten from 1.21.10 sources
  during the run itself.

## Headline

**104 of 391 oracles (26.6%) compile against the 1.21.10 jars on at least
one classpath; 287 (73.4%) fail on all three.** This is the worst oracle
measurement of any sweep so far — worse than 26.1.2 (189/202), worse than
1.21.7 post-repair (135 pass), and close to 1.21.7 pre-repair (94 pass).
The reason is provenance, not a measurement failure: 1.21.10's oracle
sources are the E6-chain set (381/391 byte-identical to the worktree's
1.21.7 tree, 389/391 to 1.21.9's, only 2 files from 1.21.9), i.e. the
UNREPAIRED 26.x-form sources — the 1.21.7 sweep's Batch 0 repairs live on
the sweep branch, not in this tree — plus 10 files adapted to 1.21.10
during the port (7 of the 10 now pass on at least one classpath). The
1.21.10 jar is also further from 26.3 than any 26.x jar: the ServerLevel
constructor, the GameRules package, the entity sub-packages, the
permissions package, gizmos and the environment-attribute API all moved
or arrived in 1.21.11, and densityfunction/renderpearl/material.rule/
clock/filefix never existed before 26.x.

| Classpath (driver configuration) | Pass | Fail | Failure rate |
|---|---|---|---|
| server — server-game.jar + server libraries + jsr305 (direct drivers) | 89 | 302 | **77.2%** |
| client — client.jar + client libraries + jsr305 (oracle_harness drivers) | 104 | 287 | **73.4%** |
| both — server + client jars + both library sets (superset) | 104 | 287 | 73.4% |
| **fail on all three classpaths (genuinely broken vs 1.21.10)** | — | **287** | **73.4%** |

Notes:

- The `both` set is identical to the `client` set, and every server-pass
  file also passes on client cp: the client jar contains the shared/server
  classes these oracles use, so client cp is the superset that matters.
  The 15 files that pass on client but fail on server are client-only
  oracles (`net.minecraft.client.*`, blaze3d) — expected, not breakage.
  No file passes on server only. 89 files pass on all three.
- The client-cp measurement adds jsr305 to the classpath. The harness
  itself does not (see §5, driver defect 1): without jsr305, javac can
  crash (`CompletionFailure: class file for javax.annotation.Nullable
  not found`, rc=4) on some oracles. The measurement isolates oracle
  breakage from that harness bug; the harness bug is recorded separately
  and NOT fixed in this batch (scope decision above). Unlike 26.1.2,
  the jar itself EXISTS in the 1.21.10 tree
  (`versions/1.21.10/tools/jsr305-3.0.2.jar`, sha1 `25ea2e8b…`, matching
  the 1.21.7 copy) — only the `client_libraries()` omission reproduces.
- Error totals: server cp 5,761 errors in 292 files; client cp 3,328
  errors in 276 files (both cp identical). For comparison, 26.1.2 was
  4,286 server / 1,391 client; 26.1 was 4,220 / 1,310; 26.2 was 3,913 /
  894; 1.21.7 pre-repair was 6,199 server / 3,916 client. 1.21.10's
  error volume sits between 1.21.7 pre-repair and the 26.x runs — but
  its pass count (104) is barely above 1.21.7 pre-repair (94), because
  the errors concentrate in shared constructors and packages rather
  than spreading thin: median 6 errors per failing file on client cp
  (26.1.2: 3), p90 26 (26.1.2: 14), max 251 (`VanillaDensityOracle`,
  same file as every run), 53 one-error files (26.1.2: 55), 189 with
  ≤10 errors.

## Method

Same fixpoint big-compile as the 1.21.7/26.2/26.1/26.1.1/26.1.2
measurements (`docs/sweep/measure-oracles.py`, unchanged): per classpath,
compile all 391 files together, then iterate the clean set
(sourcepath=False) until the compile passes. Fixpoint iterations: server
1 round (10 cascade drops), client 2 rounds (11 cascade drops), both
2 rounds (11 cascade drops). Total runtime ~60 min wall across the two
runs (javac 25.0.5), inflated by VM contention (load ~30).

Jars: the named 1.21.10 jars from the prepared `.local/minecraft/1.21.10/`
(see §6); `javac --release 21 -proc:none`, JDK 25.0.5. 1.21.10's
`java_major` is 21, matching the version manifest's
`javaVersion.majorVersion` (DataVersion 4556, protocol 773). Jar sha1s
verified against the common-brief pins before measuring: client-named
`715f5816…`, server-game-named `ba1fc89e…`, client `d3bdf582…`, bundler
server `95495a7f…` (the bundler jar was never javap'd). The script was
run with `HORIZON_VERSION_DIR` pointed at `versions/1.21.10` (its
committed home in `docs/sweep/` would otherwise resolve the repo root as
the version tree).

Tally: this batch's 391 checks are oracle-compilability checks — 0
behaviour + 391 non-behaviour, as-tallied per the batch brief (total =
files measured, pass = compiles on at least one classpath, gap =
fail-all). The behaviour-only restatement is the coordinator's job.

## Failure classes (first error per file)

Server cp (302 failures):

| Class | Files | Meaning |
|---|---|---|
| missing-package | 141 | imports a package that does not exist on the server cp (mostly client-only packages — 15 of these pass on client cp — plus the 1.21.11/26.x-only packages) |
| missing-symbol | 80 | references a class/method/field 1.21.10 does not have (renamed/moved API) |
| constructor-mismatch | 50 | calls a constructor with a 26.3/1.21.11 signature (mostly ServerLevel) |
| cascade | 10 | file itself is clean; fails only because a helper it pulls is broken |
| no-override | 8 | `@Override` of a method 1.21.10's supertype lacks |
| other | 8 | assorted (static-context, access, inference, …) |
| incompatible-types | 4 | 26.3 vs 1.21.10 signature change |
| missing-override | 1 | abstract-method gap |

Client cp (287 failures): missing-symbol 113, missing-package 83,
constructor-mismatch 56, cascade 11, no-override 10, other 9,
incompatible-types 4, missing-override 1. The client cp sheds the
client-package failures and exposes the real drift: constructor
mismatches more than double vs 26.1.2 (56 vs 20 — the ServerLevel group),
and missing symbols dominate (113) — the 1.21.11 package moves alone
account for most of them.

## The systematic groups (repair list for the follow-up decision)

All groups below were derived FRESH from the 1.21.10 measurement
(`evidence/group.py` + `evidence/groups.txt`: error-text and
source-pattern matching over the 1.21.10 big-compile logs, same
derivation as 26.1.2) and verified against 1.21.10's own jars with
javap / jar listings — none was copied from the 26.1.2 report. Where a
group's membership matches 26.1.2's, that is a measured fact, stated
as such.

**G0 — `ServerLevel` constructor (86 oracles; the headline group, NEW
at this scale).** 1.21.10's `ServerLevel` constructor takes 11 arguments
(`MinecraftServer, Executor, LevelStorageAccess, ServerLevelData,
ResourceKey, LevelStem, boolean, long, List<CustomSpawner>, boolean,
RandomSequences` — javap-verified on the 1.21.10 server-game jar); the
oracles call the 26.x 10-argument form (no `RandomSequences`), e.g.
`VanillaActiveEffectsOracle:36`
`super(null,null,null,null,null,null,false,0L,List.of(),false)`. 45 of
the 56 constructor-mismatch first errors on client cp are ServerLevel.
23 files are single-cause — one mechanical fix (append the
`RandomSequences` argument) rescues them outright:
`VanillaActiveEffectsOracle`, `VanillaAttributeMapQueriesOracle`,
`VanillaBlockEntityTickerOracle`, `VanillaBlockEventReadinessOracle`,
`VanillaCombatRulesOracle`, `VanillaConsumeEffectOracle`,
`VanillaDamageReductionOracle`, `VanillaEdgeBackoffOracle`,
`VanillaFallDamageOracle`, `VanillaFoodPropertiesOracle`,
`VanillaGroundStateOracle`, `VanillaHelmetDamageOracle`,
`VanillaLiveAttributeUpdatesOracle`, `VanillaLivingDamageOracle`,
`VanillaMovementEmissionOracle`, `VanillaMovementSupportOracle`,
`VanillaPistonMovementOracle`, `VanillaRedstonePersistenceOracle`,
`VanillaServerPlayerDamageOracle`, `VanillaStoredConsumeOracle`,
`VanillaStoredEffectNotificationsOracle`, `VanillaSwimmingOracle`,
`VanillaWaterSplashOracle`. The other 63 carry additional unrelated
errors. NOTE for the repair: the 1.21.7 fix (12-arg, with
`ChunkProgressListener`) does NOT carry — 1.21.10's ctor has no
progress-listener parameter; the translation is 10→11 args, appending
`RandomSequences` (the 10 files the 1.21.10 port adapted already use
this form — e.g. `VanillaBlockLandingOracle`, which passes on all cps).
This group does not exist in 26.1.2 (26.1.2's ServerLevel ctor matches
the 26.3 10-arg form) and is the single biggest reason 1.21.10
measures worse than any 26.x version. It is the 1.21.7 B0-O1 group at
3.4× the size (25 oracles there, 86 here).

**G1 — `Level.fuelValues()` abstract-method gap (5 oracles) + 1 other
missing override.** 1.21.10's `Level` declares
`public abstract FuelValues fuelValues()` (javap-verified:
`net.minecraft.world.level.block.entity.FuelValues` — the class exists
in 1.21.10, 2 jar entries); the oracles' fake `Level` subclasses don't
implement it: `VanillaActiveEffectTickOracle`,
`VanillaAttributeUpdatesOracle`, `VanillaBlockEntityAnimOracle`,
`VanillaLivingLandingOracle` (same 4 as 26.1.2), plus
`VanillaBlockEntityRenderOracle` which references `fuelValues()` among
its 42 errors. Mechanical repair: add a `fuelValues()` override
returning a default to each fake level. One more missing-override file
has a different abstract gap: `VanillaDecorationRegionOracle` (first
error at line 49 — anonymous class missing a `lookup…` override on
`RegistryInfoLookup`; 16 errors here vs 6 in 26.1.2). The
missing-override first-error count collapsed vs 26.1.2 (1 vs 5) only
because the other files fail earlier on ServerLevel/package errors —
the gaps are still there underneath.

**G2 — `LocalPlayer` constructor (20 oracles carry the error; 3 as
first error).** 1.21.10's `LocalPlayer` constructor takes 7 arguments
(`Minecraft, ClientLevel, ClientPacketListener, StatsCounter,
ClientRecipeBook, Input, boolean` — javap-verified on the 1.21.10
client jar); the oracles pass the 26.3 9-argument form. Membership is
IDENTICAL to 26.1.2's 20 files
(`VanillaAllayInteractionStateOracle`, `VanillaAmbientOracle`,
`VanillaAutoJumpOracle`, `VanillaBoatRiderOracle`,
`VanillaCameraOracle`, `VanillaFireStarterSoundOracle`,
`VanillaFirstPersonOracle`, `VanillaGameLoopOracle`,
`VanillaHeldDropOracle`, `VanillaHeldItemStackOracle`,
`VanillaHudOracle`, `VanillaInteractionInputOracle`,
`VanillaLocalInputOracle`, `VanillaMarkerParticlesOracle`,
`VanillaPlayerSyncOracle`, `VanillaRemoteAttackOracle`,
`VanillaSheepDyeOracle`, `VanillaSoundEngineOracle`,
`VanillaSprintInputOracle`, `VanillaTeamRulesOracle`) — but only 3 are
first-error here (vs 11 in 26.1.2) and only 1 is single-cause
(`VanillaPlayerSyncOracle`, vs 7 in 26.1.2), because the 1.21.10-only
package/ctor errors fire first in the rest. One signature translation,
verified once against the 1.21.10 jar, rescues the 1 single-cause file
outright; the other 19 carry additional unrelated errors.

**G3 — `densityfunction` package absent from 1.21.10 (17 oracles).**
`net.minecraft.world.level.levelgen.densityfunction` has ZERO entries
in the 1.21.10 server-game.jar AND client.jar (verified by jar listing)
— the `SamplerContext`/`DensityVolume` API is a 26.3 addition. Membership
is IDENTICAL to 26.1.2's 17 files: `VanillaAnchorOracle` (10 errors,
blocks 9 drivers), `VanillaBiomeGenerationOracle` (12),
`VanillaBiomeSourceCodecOracle` (9), `VanillaBiomeSourceOracle` (10),
`VanillaDensityOracle` (251 errors, the largest single failure),
`VanillaFloatLogOracle` (9), `VanillaMaterialConditionOracle` (76 errors,
blocks 5 drivers), `VanillaMaterialExtensionOracle` (12),
`VanillaMaterialStageOracle` (26), `VanillaMaterialSurfaceOracle` (15),
`VanillaNoiseOracle` (32), `VanillaStructureDensityOracle` (4),
`VanillaStructurePlacementOracle` (16 — vs 7 in 26.1.2),
`VanillaStructureStartOracle` (2), `VanillaTerrainOracle` (13),
`VanillaWorldSpawnOracle` (5), `VanillaWorldgenOracle` (42). Several
files reference the package fully-qualified rather than by import, so
the package error hides behind a symbol first error in some files.
`VanillaStructureDensityOracle` is unrepairable by design (it directly
exercises the 26.3-only API; density is verified via bytecode/chunk
comparison instead — same ruling as 1.21.7, 26.2, 26.1, 26.1.1 and
26.1.2). The others need per-file translation to 1.21.10's actual
density API, not import fixes.

**G4 — `EntityTypes` holder move (32 oracles; the 1.21.x-shape group).**
1.21.10 predates the `EntityTypes`/`BlockEntityTypes` move — registration
still lives in `net.minecraft.world.entity.EntityType` /
`net.minecraft.world.level.block.entity.BlockEntityType` (the 1.21.x
shape). `EntityTypes` and `BlockEntityTypes` have ZERO entries in the
1.21.10 server-game.jar (verified by jar listing); the constants the
oracles want exist on the old holder instead (`EntityType.PLAYER`,
`EntityType.CHICKEN` — javap-verified static finals), and
`BlockEntityType.BED` exists (1.21.10 keeps the bed block-entity type).
Every oracle referencing `EntityTypes.X` fails — 32 files, the 26.1.2
group of 31 plus `VanillaCommandExecOracle` (new member: its
`EntityTypes` reference is one of its 45 errors):
`EntityTranscript`, `KnockbackClient`, `LocatorBarClient` (13 errors,
blocks 8 drivers), `RideTranscript`,
`VanillaAllayInteractionStateOracle`, `VanillaBlockEntityAnimOracle`,
`VanillaBoatAuthorityOracle`, `VanillaBoatRiderOracle`,
`VanillaBreezeHookOracle`, `VanillaCommandExecOracle`,
`VanillaDispenserOracle` (15 errors, blocks 6 drivers),
`VanillaEntityDeathLootOracle`, `VanillaFinalizeSpawnOracle`,
`VanillaItemModelOracle`, `VanillaLateEntityPairOracle`,
`VanillaMenuOracle` (10 errors, blocks 4 drivers plus 2 cascades),
`VanillaMobAiOracle`, `VanillaMobBreathingOracle`,
`VanillaMobFallResetOracle`, `VanillaNaturalSpawnOracle`,
`VanillaProtocolClient` (34 errors, blocks 8 drivers),
`VanillaRegistryFallRayOracle`, `VanillaRemoteAttackOracle`,
`VanillaSceneMotionOracle`, `VanillaShulkerBulletOracle`,
`VanillaShulkerSpaceOracle`, `VanillaSourceSilenceOracle`,
`VanillaSprintInputOracle`, `VanillaSquidClientOracle`,
`VanillaTradingOracle`, `VanillaWetFireOracle`, `VanillaWorldRayOracle`.
Repair is a mechanical `EntityTypes.X` → `EntityType.X` holder
translation per reference, but most files carry additional unrelated
errors, so the translation alone rescues few outright. (The brief's
"O5 inverts" note: the failure direction is the SAME as 26.1.2's —
both versions predate the move — and the opposite of 26.2/26.3, where
the holder exists. No inversion vs 26.1.2 was measured; the group
grew by one file instead.)

**G5 — `renderpearl` package absent from 1.21.10 (8 oracles).**
`com.mojang.renderpearl.api.pipeline` (and `.buffers`) do not exist in
1.21.10 — the render-pipeline API is 26.3-only (0 entries in the client
jar). Membership IDENTICAL to 26.1.2's 8 files: `VanillaHudOracle` (100
errors — vs 34 in 26.1.2), `VanillaLoomScreenOracle` (5),
`VanillaPackScreenOracle` (62 — vs 13), `VanillaRecipeBookScreenOracle`
(35 — vs 4), `VanillaSpriteAnimationOracle` (2),
`VanillaTabListOracle` (36, also authlib — G7),
`VanillaTranslucencyOracle` (4), `VanillaWorldBorderRenderOracle` (34).
These need rewrites against 1.21.10's screen/render APIs, not import
fixes.

**G6 — Other constructor mismatches (15 files carry one; 8 as first
error after ServerLevel/LocalPlayer).** `VanillaChunkGenerationOracle`
(`ServerChunkCache` ctor — 1.21.10's takes 11 args, javap-verified),
`VanillaInventoryUseStartOracle` + `VanillaItemUseStartOracle`
(`ServerChunkCache` ctor), `VanillaFallingBlockOracle` (`ChunkMap`
ctor), `VanillaBiomePlacementOracle` + `VanillaDecorationOracle` +
`VanillaFeatureCatalogOracle` + `VanillaFeaturePlacementOracle`
(`Feature` ctor — 1.21.10's takes only `Codec<FC>`, javap-verified),
`VanillaSoundEventOracle` (`SoundManager` ctor — 3 errors; 1.21.7's
B0-O3 repair used a 2-arg form, not present here),
`VanillaHeldDropOracle` + `VanillaInteractionInputOracle` +
`VanillaPackScreenOracle` (`Gui` ctor), `VanillaBlockUpdateOracle` /
`VanillaEntitySoundPacketOracle` / `VanillaGameLoopOracle`
(`MinecraftServer` ctor), `VanillaMenuOracle` (`DedicatedServer` ctor).
Each is a one-signature translation against the 1.21.10 jar. Note the
shrink vs 26.1.2's G6: several 26.1.2 members (e.g.
`VanillaNaturalSpawnOracle`'s `ServerChunkCache`/`ChunkMap` errors)
are hidden here behind earlier ServerLevel/package first errors.

**G7 — Other missing packages (93 files reference one; 83
missing-package first errors on client cp).** Two strata, dated by
jar listing against 1.21.10 AND 1.21.11 (the 1.21.11 server-game jar
was listed for this — `evidence/server-jar-1.21.11-listing.txt`):

*1.21.11-era moves (present in 1.21.11, absent in 1.21.10) — NEW vs
the 26.x runs, and the largest new failure cause after ServerLevel:*
- `net.minecraft.world.level.gamerules` (GameRules moved there in
  1.21.11; 1.21.10 keeps `net.minecraft.world.level.GameRules`): 24
  source refs, 14 first-errors — `VanillaAttributeHealthOracle`,
  `VanillaBlockUpdateOracle`, `VanillaDamageImmunityOracle`,
  `VanillaDispenserOracle`, `VanillaEntityDeathLootOracle`,
  `VanillaExplosionItemsOracle`, `VanillaFinalizeSpawnOracle`,
  `VanillaFoodOracle`, `VanillaLavaCauldronItemOracle`,
  `VanillaMenuOracle`, `VanillaMobAiOracle`, `VanillaRaidOracle`,
  `VanillaRedstoneCatalogOracle`, `VanillaRegisteredPlayerCombatOracle`,
  `VanillaRegistryFallRayOracle`, `VanillaSceneMotionOracle`,
  `VanillaSculkOracle`, `VanillaServerPlayerCombatOracle`,
  `VanillaSourceSilenceOracle`, `VanillaTradingOracle`,
  `VanillaWorldRayOracle`, `VanillaArrowOracle`,
  `VanillaCommandExecOracle`, `VanillaEntityComponentsOracle`.
- Entity sub-packages (the 1.21.11 entity-package split; 1.21.10 keeps
  the classes in the parent packages — `Boat` in `vehicle/`,
  `Zombie` in `monster/`, `Arrow` in `projectile/`, all
  jar-verified): 32 source refs — `…vehicle.boat` (2 first-errors),
  `…vehicle.minecart` (2), `…monster.zombie` (4),
  `…projectile.arrow` (5), `…animal.cow` (3), `…animal.squid` (1),
  `…animal.fox` (1), `…decoration.painting` (1),
  `…projectile.throwableitemprojectile` (1).
- `net.minecraft.world.attribute` (environment attributes — arrived in
  1.21.11, 55 jar entries there, 0 in 1.21.10): 28 source refs, 6
  first-errors.
- `net.minecraft.gizmos` (arrived in 1.21.11, 21 jar entries there, 0
  in 1.21.10): 4 first-errors (`VanillaMobAiOracle` — 103 errors,
  the second-largest failure — `VanillaRedstoneCatalogOracle`,
  `VanillaSculkOracle`, `VanillaTradingOracle`).
- `net.minecraft.server.permissions` (arrived in 1.21.11, 19 jar
  entries there, 0 in 1.21.10): 9 source refs, 2 first-errors
  (`VanillaGameLoopOracle`, `VanillaServerFilesOracle`).

*26.x-only packages (absent from both 1.21.10 and 1.21.11):*
- `net.minecraft.world.level.levelgen.material.rule`: 5 source refs
  (same 5 as 26.1.2), 3 first-errors — all also G3.
- 26.3 advancements packages (1.21.10 has only `critereon`):
  `ComponentWireCapture` (`advancements.predicates.StatePropertiesPredicate`,
  1 error), `VanillaEntityComponentsOracle`
  (`advancements.predicates.entity`, 12 errors), `VanillaRaidOracle`
  (`advancements.triggers`, 26 errors).
- `com.mojang.authlib.services` (absent from authlib 7.0.63):
  `VanillaTabListOracle` (36 errors) AND `VanillaCommandExecOracle`
  (45 errors — new member vs 26.1.2, and the first error that blocks
  Batch 9's command_parse run).
- `org.lwjgl.sdl`: `VanillaTickScript` (19 errors).
- `net.minecraft.world.clock`: 13 source refs, 3 first-errors
  (`VanillaActiveEffectTickOracle`, `VanillaAttributeUpdatesOracle`,
  `VanillaLivingLandingOracle` — all also G1).
- `net.minecraft.util.filefix`: `VanillaDataFixOracle` (25 errors).
- `net.minecraft.client.resources.model.geometry` /
  `.sprite`: 12 source refs, 4 first-errors (`VanillaItemModelOracle`
  45 errors, `VanillaModelResolutionOracle`, `VanillaModelBakingOracle`,
  `VanillaModelChoiceOracle`).
- `net.minecraft.client.model.object.book` / `.banner`:
  `VanillaBlockEntityAnimOracle`, `VanillaBlockEntityRenderOracle`.
- `net.minecraft.client.renderer.block.dispatch`:
  `VanillaGallerySelectionOracle` (25 errors).
- `java.lang.classfile` (+ `.instruction`): `SeedAgent` (5 errors),
  `VanillaFrameLimiterOracle` (7 errors) — the Class-File API is a
  JDK 22+ API; the measurement compiles at `--release 21` (1.21.10's
  `java_major`), and `oracle_harness.py:30` uses the same release, so
  this is a real 1.21.10 failure, not a measurement artifact. A third
  file references it behind other errors.

**G8 — The long tail (82 files after the groups above and the cascades
are set aside: 61 missing-symbol + 9 other + 6 no-override + 4
incompatible-types + 1 constructor-mismatch + 1 missing-package).**
Per-API translation, verified against the 1.21.10 jar per file. 24 of
the 82 have exactly 1 error. The cheapest high-leverage single repair
in the whole batch is here: `VanillaFaceSupportOracle` — 1 error
(`CubeVoxelShape(DiscreteVoxelShape)` has protected access in 1.21.10
— same error as 26.1.2, 26.1.1, 26.1 and 26.2), blocking 19 drivers,
plus 5 cascade oracles depend on it. `VanillaRegistryCollisionScanOracle`
(8 drivers, 2 errors — ServerLevel ctor first, then the
PalettedContainerFactory gap), `ParityFont` (8 drivers, 2 errors — vs
1 in 26.1.2), `VanillaSuffocationOracle` (7 drivers, 2 errors),
`VanillaCollisionContextOracle` (5 drivers, 4 errors — no-override,
vs 1 in 26.1.2) and `VanillaSoundRig` (4 drivers, own errors here —
in 26.1.2 it was a cascade) are the other one-or-two-error blockers.
`VanillaBlockUpdateOracle` (63 errors, 6 drivers) anchors 5 cascade
oracles. Notable single-file causes in the tail:
`VanillaFirstPersonOracle` (26.3-only `FirstPersonHandsAndItems`, 0
entries in the client jar — plus G10, see below),
`VanillaItemModelOracle` (`ResourceManager.Selector`, `ItemQuads` —
both 0 entries; `MaterialBaker` is an interface in 1.21.10, not a
class), `VanillaPackResourcesOracle` (`PackResources.forEach` absent
in 1.21.10 — the interface has only `getResource`/`listResources`,
javap-verified; the oracle's `.open().forEach` call at line 65 is the
same root cause as 26.1.2's, at a different call site — and note
`PackResources` itself moved packages: `net.minecraft.server.packs`
in 1.21.10 vs `net.minecraft.server.packs.resources` in 26.x),
`VanillaStructurePlacementOracle` / `VanillaStructureStartOracle` /
`VanillaWorldSpawnOracle` (`RandomState.create` overloads),
`VanillaNaturalSpawnOracle` (`ChunkPos.pack()` — no such method in
1.21.10, line 274 — plus 21 more errors; the 1.21.10 port's adaptation
of this file did not rescue it), `VanillaRecipeOracle`
(`Recipe.assemble` signature), `VanillaItemUseOracle`
(`LivingEntity.getTicksUsingItem` signature), `VanillaConversionTagsOracle`
(`Brain<>` type inference).

**G9 — Cascades (11 oracles on client cp, 10 on server).** Clean files
failing only via a broken helper: via `VanillaFaceSupportOracle` (5) —
`VanillaBlockCollisionScanOracle`, `VanillaDynamicSupportOracle`,
`VanillaEntityCollisionOracle`, `VanillaPistonShapeOracle`,
`VanillaWorldBorderOracle`; via `VanillaBlockUpdateOracle` (3) —
`VanillaEnchantDurabilityOracle`, `VanillaOpenedLootOracle`,
`VanillaPortalOracle` (26.1.2 also had `VanillaFurnaceOracle` and
`VanillaHelmetDamageOracle` here — both now have own errors:
`VanillaFurnaceOracle` is in the tail, `VanillaHelmetDamageOracle`
is a single-cause ServerLevel file); via `VanillaMenuOracle` (2) —
`VanillaPlaceRecipeOracle`, `VanillaRecipeBookOracle`; via
`VanillaAnchorOracle` (1) — `VanillaHeightProviderOracle`; via
`VanillaTextDisplayOracle` (1) — `VanillaTextDisplayTransformOracle`;
via `VanillaContainerScreenOracle` (1, client only) —
`VanillaSlotHoverOracle` (new cascade vs 26.1.2). The 26.1.2
`VanillaSoundRig` → `VanillaMusicOracle` chain is gone: both now have
own errors. These pass for free once their helpers are repaired — do
not schedule them separately.

**G10 — Re-ported 26.3-form oracle sources (17 files; E6-chain
inheritance, NOT a 1.21.11 re-port).** The 26.1.2 run's 17 re-ported
files all still fail in 1.21.10, most with identical error counts —
but the provenance story differs from 26.1.2's: 1.21.10's oracle
sources come from the E6 chain (2 files differ from 1.21.9, 10 from
1.21.7, 15 from 1.21.8), NOT from the 1.21.11 template (168 files
differ). 8 of the 17 are byte-identical to the 26.1.2 files
(`VanillaBasicFeatureOracle`, `VanillaFeaturePlacementOracle`,
`VanillaFeatureSortOracle`, `VanillaFillLayerOracle`,
`VanillaReplaceBlockOracle`, `VanillaScatteredOreOracle`,
`VanillaFeatureGraphOracle`, `VanillaSteppedInterpolationOracle`);
the other 9 differ in source but carry the same 26.3-form signatures
and fail same-or-worse:

- Same error count as 26.1.2 (12 files): `VanillaBasicFeatureOracle`
  (2), `VanillaBiomePlacementOracle` (8), `VanillaBlockTransformerOracle`
  (3), `VanillaFeaturePlacementOracle` (3), `VanillaFeatureSortOracle`
  (1), `VanillaFillLayerOracle` (1), `VanillaReplaceBlockOracle` (1),
  `VanillaScatteredOreOracle` (1), `VanillaBrewingOracle` (14),
  `VanillaFeatureCatalogOracle` (11), `VanillaFeatureGraphOracle` (4),
  `VanillaSteppedInterpolationOracle` (16).
- Worse than 26.1.2 (5 files): `VanillaBrewingFuelOracle` (8 → 9),
  `VanillaDecorationRegionOracle` (6 → 16),
  `VanillaFeatureRegionOracle` (10 → 15),
  `VanillaFirstPersonOracle` (20 → 25), `VanillaTreeFeatureOracle`
  (11 → 16).
- `VanillaSimpleBlockOracle` still compiles on all cps (its 1.21.10
  adaptation — `PalettedContainerFactory` form — passes, unlike the
  1.21.7 tree's no-factory form).

The re-port signature is the 26.3 unconfigured-Feature form:
`Feature.DIRECT_CODEC` (absent from 1.21.10 — only
`ConfiguredFeature.DIRECT_CODEC` exists), no-arg `new Feature()`
(1.21.10's `Feature` takes only `Codec<FC>`), the 4-arg
`place(WorldGenLevel, ChunkGenerator, RandomSource, BlockPos)`
(1.21.10's abstract `place` takes `FeaturePlaceContext`), and
`FeaturePlacer`-based placement. The non-Feature re-ports use 26.3-only
APIs throughout: the `BrewingFuel` component (0 jar entries),
`net.minecraft.core.component.BlockTransformer`, the
`BrewingInput`/`BrewingRecipe`/`PotionIngredient`/`RecipePropertySet`
crafting APIs, `PositionAndRotation` (0 jar entries, in
`VanillaSteppedInterpolationOracle`), and `LivingEntity.SwingState`
(0 jar entries, in `VanillaFirstPersonOracle`). The 26.1-adapted
forms remain the repair template for the Feature files; the
1.21.7-adapted forms (B0-O3) are the template for the rest — e.g.
1.21.7's `VanillaBrewingFuelOracle` documents "1.21.7 has no
`brewing_fuel` component: fuel is any item in the tag, 20 uses, and
the brew time is a flat 400 ticks".

26 oracles are never referenced by any driver; 12 of them fail on all
three classpaths (`ComponentWireCapture`, `EntityTranscript`,
`OfflineLanHost`, `RideTranscript`, `VanillaGallerySelectionOracle`,
`VanillaLootFillOracle`, `VanillaMover`, `VanillaOptionsTableExporter`,
`VanillaSceneMotionOracle`, `VanillaSlotHoverOracle`,
`VanillaStructurePlacementOracle`, `SeedAgent`) — vs 8 in 26.1.2 (the
4 new ones are `SeedAgent`, `VanillaGallerySelectionOracle`,
`VanillaOptionsTableExporter`, `VanillaSlotHoverOracle`). Three of the
twelve are referenced by non-driver scripts (`OfflineLanHost` in
`tools/vanilla_client.py`, `RideTranscript` in
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
helper); those were judged against the `both` superset — 15 blocked, 5
runnable (26.1.2: 14 blocked, 6 runnable).

| Driver state | Drivers |
|---|---|
| **Blocked: ≥1 oracle fails to compile under its path's cp** | **314** |
| Oracles compile under their path's cp (can yield a measurement when run) | 80 |
| No oracle reference (server-e2e, data-only, wrappers) — not oracle-blocked | 254 |
| **Total** | **648** |

- 334 drivers have no oracle blocker (80 + 254), vs 417 for 26.1.2
  (163 + 254), 417 for 26.1.1, 426 for 26.1 and 509 for 26.2 and 321
  for 1.21.7 pre-repair (67 + 254). The 254 no-oracle drivers were
  never oracle-blocked, but that does not mean they pass: they run
  real servers / the game / data diffs and have their own failure modes.
- Blocked by classpath: server 168, both 113 (98 explicit + 15
  no-flag), client 33 (26.1.2: 111 / 77 / 29, plus 14 no-flag on both).
- The highest-leverage repairs are shared helpers: `VanillaFaceSupportOracle`
  blocks 19 drivers (1 error — cheapest), `VanillaAnchorOracle` 9
  (10 errors, densityfunction — G3), `VanillaRegistryCollisionScanOracle`
  8 (2 errors), `VanillaProtocolClient` 8 (34 errors),
  `ParityFont` 8 (2 errors), `LocatorBarClient` 8 (13 errors),
  `VanillaSuffocationOracle` 7 (2 errors), `VanillaCommandExecOracle`
  6 (45 errors — NEW blocker vs 26.1.2, where it passed),
  `VanillaDispenserOracle` 6 (15 errors, EntityTypes — G4),
  `VanillaBlockUpdateOracle` 6 (63 errors, plus 3 cascades),
  `VanillaMaterialConditionOracle` 5 (76 errors, densityfunction —
  G3), `VanillaCollisionContextOracle` 5 (4 errors). Fixing the top
  few one-error helpers unblocks ~50 driver-oracle pairs (overlaps
  exist) — and the G0 ServerLevel fix alone rescues 23 files outright.
- Compiling ≠ running: the 80 "runnable" drivers are runnable by
  javac only. No driver was run in this batch (scope decision above),
  so the runtime-failure layer the 1.21.7 sample found (a compiling
  oracle throwing at runtime) is unmeasured for 1.21.10 — with ONE
  concrete exception: `VanillaComponentPatchOracle` compiles clean but
  will throw `ClassNotFoundException` at runtime (see §5, B0-O10).
  Every repaired oracle needs an end-to-end driver run to confirm it
  yields a measurement.

## The 9-harness check (1.21.7's table, re-measured for 1.21.10)

Of the 9 harnesses the 1.21.7 plan (§2) records as dying on oracle javac
failures in CI run 44441686979, **only 2 pass on at least one classpath
for 1.21.10 at measurement time; 7 fail on all three and need repair
before their drivers can run** (vs 6/3 for 26.1.2, 26.1.1 and 26.1; for
26.2 all 9 passed):

| Harness | Oracle | 1.21.10 status |
|---|---|---|
| chunk_generation | VanillaChunkGenerationOracle | **FAIL** (all cps; client 24 errors: ServerChunkCache ctor first, then gamerules + ServerLevel) |
| command_exec | VanillaCommandExecOracle | **FAIL** (all cps; client 45 errors: authlib.services first, then EntityTypes, gamerules, GameRules) |
| component_patch | VanillaComponentPatchOracle | **PASS** (all cps) — but see the runtime crash below (B0-O10) |
| datafix | VanillaDataFixOracle | **FAIL** (all cps; client 25 errors: `net.minecraft.util.filefix` first) |
| first_person | VanillaFirstPersonOracle | **FAIL** (all cps; client 25 errors, vs 20 in 26.1.2: 26.3-only `FirstPersonHandsAndItems`, `LivingEntity.SwingState`, LocalPlayer ctor, no-override) |
| food | VanillaFoodOracle | **FAIL** (all cps; client 8 errors: gamerules package first) |
| item_model | VanillaItemModelOracle | **FAIL** (all cps; client 45 errors, vs 10 in 26.1.2: model.geometry package first, then EntityTypes, `ResourceManager.Selector`, `ItemQuads`, `MaterialBaker` interface) |
| options | VanillaOptionsOracle | **PASS** (client + both; server fails on client packages — expected) |
| resource_pack | VanillaPackResourcesOracle | **FAIL** (all cps; client 1 error: `PackResources.forEach` absent in 1.21.10 — same root cause as 26.1.2, at the `.open().forEach` call site, line 65) |

For 1.21.7, 8 of these passed only AFTER Batch 0 repairs. For 1.21.10
the `command_parse`/`command_exec`/`food`/`component_patch` drivers —
the ones that yielded the 1.21.7 sweep's first real parity measurements
— are oracle-blocked from the start except `component_patch` (which
compiles but crashes at runtime — B0-O10). Batch 9's command_parse run
cannot proceed until `VanillaCommandExecOracle` is repaired (45
errors, authlib.services + EntityTypes + gamerules + GameRules).

## Comparison against the 26.1.2 table

The oracle sources DIFFER between the 26.1.2 and 1.21.10 trees (172 of
391 files differ by `diff -rq`; the 1.21.10 set is the E6-chain set,
the 26.1.2 set the 26.x re-port set), and the jar delta is large (a
full era step, not a patch). The measured table was diffed against
`26.1.2/batch-0-oracles.json` file by file on every field (server /
client / both status, first-error class, first-error message, error
count) — `evidence/diff-26.1.2.json`:

**Deviations: 220 of 391 files.** Identical: 171. New pass: ZERO — no
file passes new vs 26.1.2. New fail: 85 files that passed on at least
one classpath in 26.1.2 fail on all three in 1.21.10. Changed error
(same fail-all status, different first error or count): 135 files.
Among the 202 files that fail-all in BOTH runs, error counts grew in
127, shrank in ZERO, and stayed the same in 59 (16 cascade files have
no count). 1.21.10 is strictly worse-or-equal on every single file.

New-fail breakdown by 1.21.10 first-error class: constructor-mismatch
32 (30 of them ServerLevel — G0), missing-symbol 27, missing-package
20 (gamerules, entity sub-packages, world.attribute, classfile),
other 3 (`VanillaConversionTagsOracle` Brain inference,
`VanillaItemUseOracle` getTicksUsingItem, `VanillaRecipeOracle`
assemble), no-override 2, cascade 1 (`VanillaSlotHoverOracle`).
The 85 new-fail files include the four 9-harness oracles that passed
in 26.1.2 but fail here (`VanillaChunkGenerationOracle`,
`VanillaCommandExecOracle`, `VanillaDataFixOracle`,
`VanillaFoodOracle`) and both `java.lang.classfile` files
(`SeedAgent`, `VanillaFrameLimiterOracle` — release-21 failures).

Biggest error-count growth among fail-all-in-both files:
`VanillaMobAiOracle` 12 → 103 (+91, gizmos + gamerules + entity
packages), `VanillaTradingOracle` 6 → 86 (+80),
`VanillaHudOracle` 34 → 100 (+66), `VanillaPackScreenOracle` 13 → 62
(+49), `VanillaSculkOracle` 7 → 51 (+44),
`VanillaSectionMeshOracle` 7 → 48 (+41),
`VanillaFinalizeSpawnOracle` 7 → 48 (+41),
`VanillaChatScreenOracle` 5 → 41 (+36),
`VanillaItemModelOracle` 10 → 45 (+35),
`VanillaBlockEntityRenderOracle` 9 → 42 (+33),
`VanillaRedstoneCatalogOracle` 4 → 36 (+32),
`VanillaRecipeBookScreenOracle` 4 → 35 (+31).

Cause attribution for the deviations: the overwhelming majority are
jar-delta effects (the 1.21.10 jar lacks APIs the 26.1.2 jar has:
11-arg ServerLevel ctor, gamerules package, entity sub-packages,
permissions package, gizmos, world.attribute) plus source-delta
effects where the 1.21.10 oracle sources differ from 26.1.2's (172
files — e.g. `VanillaCommandExecOracle`'s authlib.services import and
EntityTypes reference, which the 26.1.2 source lacks). No deviation
is a measurement difference: the method, script, JDK and fixpoint
rules are identical to the 26.1.2 run, and the one rerun reproduced
the killed run's client numbers exactly.

Consequence: there IS an O9-style "fails new or worse" group for this
run — 85 new-fail files plus 127 error-count growths — but it is NOT
a re-port story: it is the jar-delta story (1.21.10 is two API steps
behind the oracle sources' 26.3 forms, where 26.1.2 was one). O9
itself (the 17 re-ported 26.3-form files) reproduces in substance —
same 17 files failing, 5 worse — as G10 above.

## Driver defects found (Batch 0 scope)

1. **`client_libraries()` omits jsr305 — javac can crash (latent).**
   `versions/1.21.10/tools/test_model_baking_parity.py:client_libraries()`
   builds the client classpath from the manifest only; it does not
   append `tools/jsr305-3.0.2.jar`. No `tools/*.py` file in the 1.21.10
   tree mentions jsr305 in the client path (the server path,
   `test_rust_parity.py:112-115`, appends it — the 1.21.7 Batch 0 fix
   `d7a14e91` for the server path IS in this tree, but the client
   half is not). Without jsr305, javac crashes (`CompletionFailure:
   class file for javax.annotation.Nullable not found`, rc=4) when
   compiling an oracle that completes a `Nullable`-referencing symbol
   — a latent crash across the oracle_harness drivers, not a
   deterministic one. The measurement adds jsr305 itself, so this did
   not affect the numbers above. Unlike 26.1.2, the jar itself exists
   in the tree (`tools/jsr305-3.0.2.jar`, sha1 `25ea2e8b…`) — only
   the one-line `client_libraries()` fix is missing. Fix direction
   (follow-up, not this batch): append jsr305 in `client_libraries()`
   — one point, fixes all users.
2. **MobAi driver gates nothing 1.21.10 cannot run.** The 1.21.7 Batch
   0 fix gated 27 scenarios 1.21.7 cannot run (copper golem, spears,
   age lock); that gating is NOT in the 1.21.10 tree
   (`test_mob_ai_parity.py` runs all 156 scenarios ungated). Against
   1.21.10's own jar: copper golem EXISTS (11 jar entries — those 7
   scenarios are runnable, the 1.21.7 concern is withdrawn for them),
   but spears are ABSENT (0 jar entries — 4 scenarios:
   `zombie_spear_player`, `zombie_spear_close`,
   `vindicator_spear_player`, `vindicator_spear_close`, lines 60-74)
   and the AgeableMob age lock is ABSENT (no `setAgeLocked`/`ageLocked`
   on `AgeableMob` — 16 scenarios). 20 scenarios will fail at runtime
   for want of the gating the 1.21.7 tree carries. Recorded as slice
   item O11 (minor).

## `.local/minecraft/1.21.10/` setup (recipe for later batches)

`verify_artifacts()` needs `manifest.json` (pinned sha1), `jars/client.jar`
+ `jars/server.jar` (manifest sha1s or `remapped.json` digests), the inner
`jars/server-game.jar` (bundle checksum or `remapped.json`), and
`libraries/server/` + `libraries/client/` (sha-verified). No worktree had
it in the sweep worktree; unlike 26.1.2 (built fresh with `lab.py
prepare`), 1.21.10 was copied from the port workdir's already-prepared
directory (227 MB, sha-verified by the gate itself):

```sh
cp -r ~/workspace/mcir-work/e6-deltas/ports/1.21.10/.local/minecraft/1.21.10 \
  ~/workspace/mcir-work/sweep-1.21.10/versions/1.21.10/.local/minecraft/1.21.10
cd ~/workspace/mcir-work/sweep-1.21.10/versions/1.21.10
python3 -c "import sys; sys.path.insert(0,'tools'); import test_rust_parity as t; t.verify_artifacts(); print('GATE OK')"
```

Result: named client jar (9,969 classes) + named inner server jar
(6,964 classes), 45 server + 70 client libraries, `manifest.json` sha1
`1dcba28d…` (the pinned one in research-lock.json). Verified: the gate
passes. Never commit `.local`. (If a fresh build is ever needed instead
of the copy, the launcher-style source dir is
`~/workspace/mcir-work/e6-deltas/versions/1.21.10/launcher/` and the
server bundler jar is `…/jars/server.jar` — same `lab.py prepare`
invocation as the 26.1.2 run. Note 1.21.10's prepare includes a remap
stage — the version manifest's `downloads` has mappings — unlike the
26.1.x runs.)

## Prepared-data setup (for Batch 9 and the B3-G22 re-check)

Prepared 1.21.10 **minecraft** data now exists in the sweep worktree
(see §6 above, copied by this batch from the port workdir). Prepared
1.21.10 **horizon-client** data EXISTS on this VM — unlike the 26.1.2
run, where none existed anywhere:

- `~/workspace/mcir-work/e6-deltas/ports/1.21.10/.local/horizon-client/1.21.10/`
  (523 MB, 44 entries): the full prepare output — `registries.json`,
  `items.json`, `commands.json`, `command-registries.json`,
  `landing-definitions.json`, `movement-properties.nbt`,
  `block-behaviours.json`, `entity-models.json`, `mob-data.json`,
  `model-states.json`, `attributes.json`, `facts.json`, `datafix-tables.json`,
  `creative-inventory.json`, `damage-catalog.json`, plus
  `assets/minecraft/lang/en_us.json` and `assets/minecraft/sounds.json`.
  No expected gaps were found: sounds and lang are both present (under
  `assets/minecraft/`, not at the top level).
- `~/workspace/mcir-work/candidate-test-1.21.10/.local/horizon-client/1.21.10/`
  (143 MB): a smaller copy — partial, do not prefer it over the port
  workdir's.
- The sweep worktree's own `.local/horizon-client/1.21.10/` (185 MB at
  time of writing) is being built by a sibling batch (Batch 9's work —
  a `.horizon-prepare.lock` file is present and
  `.local/command-parse-parity/` already holds compile logs). This
  batch did NOT touch it and did NOT run horizon-prepare itself.

What Batch 9's `command_parse` run needs, and what is present:

| Need | Present? |
|---|---|
| Prepared minecraft data (`.local/minecraft/1.21.10/`, gate OK) | YES (built by this batch) |
| `VanillaCommandExecOracle` / command_parse oracles compile | **NO — 45 errors** (authlib.services, EntityTypes, gamerules) — the run is oracle-blocked until the CommandExec repair lands |
| Prepared horizon-client data (`.local/horizon-client/1.21.10/`) | YES in the port workdir (523 MB); in-progress copy in the sweep worktree (sibling's) |
| Built `client_probe` binary (`cargo build --locked -p horizon-client-lib --example client_probe`) | NO — only `cargo check` was run (H01); a full probe build is Batch 9's job (CI-or-quiet-window under this VM's memory budget) |

What the B3-G22 re-check (client loading its core tables with prepared
1.21.10 data) needs: the same `.local/horizon-client/1.21.10/`
directory (registries, items, movement-properties, landing-definitions,
…) plus a built client. The data exists in the port workdir (copy it
into the sweep worktree's `.local/`, or point the check at it — do not
re-run the prepare unless the copy proves stale). The B3-G22 re-check
itself is Batch 3's, not this batch's; this batch records only that
its data prerequisite is met.

Exact commands to produce the missing data if the copies ever go stale
(mirroring the 26.1.1 run):

```sh
cd ~/workspace/mcir-work/sweep-1.21.10/versions/1.21.10
# Build horizon-prepare (taskset -c 0, -j 1):
RUSTUP_HOME=~/workspace/tools/rustup CARGO_HOME=~/workspace/tools/cargo \
  PATH=~/workspace/tools/jdk-25/bin:~/workspace/tools/cargo/bin:$PATH \
  taskset -c 0 cargo build --locked -j 1 -p horizon-prepare --release
# Run it against the prepared minecraft data:
./target/release/horizon-prepare --minecraft-dir .local/minecraft/1.21.10
# Then check for the expected asset-store gaps only:
./target/release/horizon-prepare --check
```

Caution (from the 26.1.1 run): a first horizon-prepare run can fail
transiently at "Exporting movement and attributes" (`Exporter
VanillaBlockLandingOracle failed:
"java.lang.reflect.InvocationTargetException"`, with an empty
`.local/horizon-client-motion/export.log`) when run concurrently with
heavy javac work — retry on a quieter VM before assuming a real
exporter defect. (For 1.21.10 the oracle itself compiles cleanly on
all three classpaths, so the same advice applies.)

## What the numbers mean

- Batch 0's repair load is **287 files** — the largest of any sweep
  (26.1.2: 202, 26.1.1: 202, 26.1: 199, 26.2: 133, 1.21.7 pre-repair:
  297). But the profile is front-loaded, not diffuse: two mechanical
  groups (G0 ServerLevel ctor, 86 files with 23 single-cause; G2
  LocalPlayer ctor, 20 files with 1 single-cause) plus the package-move
  translations (gamerules 24, entity sub-packages 32, world.attribute
  28 — mostly mechanical import + holder translations) rescue a large
  share with signature-level work. The densityfunction (G3, 17 files),
  renderpearl/material.rule (G5, G7) and model-API groups are the
  genuinely hard ones: 26.3-only APIs with no 1.21.10 equivalent to
  translate to import-by-import — those oracles need rewrites against
  1.21.10's actual APIs (or, for StructureDensity, stay unrepaired by
  design). The 1.21.7 Batch 0 repairs (B0-O1–O4) are the repair
  template for much of this — but they must be RE-DERIVED for 1.21.10,
  not copied: the ServerLevel ctor is 11-arg here (not 1.21.7's
  12-arg), the GameRules package is `world.level` (as in 1.21.7, but
  the 1.21.7 repair assumed 1.21.7's other APIs), and 1.21.10 has
  APIs 1.21.7 lacks (copper golem, `PalettedContainerFactory`,
  `RandomSequences` on the level).
- Sequencing for batches 3–12: only 80 oracle-clean drivers can run
  once their probes build (and once Batch 9's prepared horizon-client
  data is in place in the sweep worktree — see above); the 314 blocked
  drivers wait on Batch 0 repairs, shared helpers first
  (`VanillaFaceSupportOracle` unblocks 19 drivers with a 1-error
  repair; `ParityFont` and `VanillaRegistryCollisionScanOracle` 8
  each). The 254 no-oracle drivers were never oracle-blocked. In
  practice this sweep's batches run statically (javap + source +
  data) and record driver runs as follow-up needs — the VM's 6 GB
  budget under multi-agent load makes probe builds (client_probe OOMs
  single-threaded) a CI-or-quiet-window job, not a sweep-batch job.
- The 1.21.10-vs-26.3 API deltas this measurement exposes are
  themselves sweep input: the 11-arg `ServerLevel` ctor (with
  `RandomSequences` on the level), the 7-arg `LocalPlayer` ctor, the
  `GameRules`-in-`world.level` shape, the flat entity packages (no
  `vehicle.boat`/`monster.zombie`/`projectile.arrow` sub-packages),
  the absence of the `permissions`/`gizmos`/`world.attribute` APIs
  (all 1.21.11 additions), the `EntityType`/`BlockEntityType`
  holder shape (no `EntityTypes` move), `Level.fuelValues()`
  (abstract in 1.21.10), the `ServerChunkCache`/`ChunkMap`/`Gui`/
  `Feature` constructor forms, the absence of the `densityfunction`,
  `renderpearl`, `material.rule`, `clock` and `filefix` APIs, the
  advancements `predicates`/`triggers` packages (1.21.10 has only
  `critereon`), the `ConfiguredFeature` form (1.21.10 has no
  `Feature.DIRECT_CODEC`, no-arg `Feature()` or 4-arg `place`), the
  absence of the `BrewingFuel`/`BlockTransformer` components and of
  `LivingEntity.SwingState`, the `PackResources`-in-`server.packs`
  shape without `forEach`, and the `MaterialBaker` / registry-lookup
  abstract methods. Family batches should check the port's Rust
  against 1.21.10's forms of these, not 26.3's — and not 1.21.11's
  either, where the two differ (gamerules, entity packages,
  permissions, gizmos, world.attribute are all 1.21.11, not 1.21.10).

## Re-check verdicts

### 1.21.7 re-checks (PRIMARY — 10 items, all decided)

| Item | 1.21.7 status | 1.21.10 verdict | 1.21.10 item |
|---|---|---|---|
| B0-O1 ServerLevel 12-arg ctor group (25 oracles) | fixed | **REPRODUCES** — the 1.21.7 repair (12-arg) is NOT in this tree; oracles call the 26.x 10-arg form; 1.21.10's ctor is 11-arg; 86 files carry the error, 23 single-cause | O10 |
| B0-O2 jsr305 client-classpath crash | fixed | **REPRODUCES** — `client_libraries()` still omits jsr305; the jar itself IS in the tree (half-fixed vs 26.1.2) | O2 |
| B0-O3 1.21.11 carry-over repairs (14 oracles) | fixed | **REPRODUCES** — none of the 14 repairs is in this tree; all 14 oracles still fail-all (Suffocation 2, Food 8, EntityComponents 12, SheepDye 7, SoundEvent 3, ChunkGeneration 24, SquidClient 22, DataFix 25, FinalizeSpawn 48, PackResources 1, CommandExec 45, FirstPerson 25, MobAi 103, ItemModel 45 errors) | O1–O7 (by cause) |
| B0-O4 Anchor + RegistryCollisionScan helpers | fixed | **REPRODUCES** — Anchor still fails (10 errors, densityfunction); RegistryCollisionScan still fails (2 errors, ServerLevel ctor first) | O4 / O10 |
| B0-O5 MobAi driver gates 27 scenarios | fixed | **REPRODUCES (partial)** — no gating in this tree; copper golem EXISTS in 1.21.10 (7 scenarios runnable — withdrawn for that part) but spears (4 scenarios) and the age lock (16) are absent and ungated | O11 |
| B0-O6 256 fail-all (65.5%); 259 drivers blocked | gap | **REPRODUCES, worse** — 287 fail-all (73.4%); 314 drivers blocked | O7 |
| B0-O7 StructureDensity unrepairable | gap | **REPRODUCES** — densityfunction absent from both 1.21.10 jars; StructureDensity fails with 4 errors, same ruling (verify via bytecode/chunk comparison) | O4 |
| B0-O8 runtime-failure layer | gap | **REPRODUCES** — 80 "runnable" drivers are runnable by javac only; no driver was run; one concrete runtime crash found (B0-O10) | O8 |
| B0-O9 format-changing repairs expose probe assumptions | gap | **WITHDRAWN** — the 1.21.10 tree carries no format-changing oracle repairs (FirstPerson, ItemModel, Anchor, Suffocation all still fail javac), so no probe-side exposure exists to record; if the repairs land, the exposure question re-opens | O13 (closed) |
| B0-O10 ComponentPatch Removed runtime crash | fixed | **REPRODUCES** — the 1.21.7 repair (Optional.empty() form) is NOT in this tree; the oracle still does `Class.forName("net.minecraft.core.component.Removed")` via reflection (line 14) and `Removed` has 0 entries in the 1.21.10 jars; compiles clean, will throw ClassNotFoundException at runtime | O12 |

### 26.1.2 re-checks (SECONDARY — 9 items, all decided)

| Item | 26.1.2 status | 1.21.10 verdict | 1.21.10 item |
|---|---|---|---|
| O1 LocalPlayer ctor group (20 oracles) | gap | **REPRODUCES** — identical 20-file membership; target form is 7-arg here (not 8-arg); only 1 single-cause (vs 7) | O1 |
| O2 jsr305 client-classpath crash (latent) | gap | **REPRODUCES** — same `client_libraries()` omission; jar present in tree (26.1.2 had to copy it in) | O2 |
| O3 fuelValues() + missing overrides | gap | **REPRODUCES** — same 4 + 1 files; missing-override first-error count 1 (vs 5) only because other errors fire first | O3 |
| O4 densityfunction absent (17 oracles) | gap | **REPRODUCES** — identical 17-file membership; StructurePlacement worse (7 → 16 errors) | O4 |
| O5 EntityTypes holder move (31 oracles) | gap | **REPRODUCES** — 32 files (31 + `VanillaCommandExecOracle`); same direction as 26.1.2 (both predate the move) | O5 |
| O6 other 26.3-only packages | gap | **REPRODUCES, expanded** — renderpearl 8, material.rule 5, advancements 3, authlib.services 2 (was 1), lwjgl.sdl 1, PLUS 1.21.11-era moves (gamerules 24, entity sub-packages 32, world.attribute 28, gizmos 4, server.permissions 9) and 26.x-only clock 13, filefix 1, model geometry/sprite 12, classfile 3 | O6 |
| O7 202 fail-all (51.7%); 231 drivers blocked | gap | **REPRODUCES, worse** — 287 fail-all (73.4%); 314 drivers blocked; 0 files pass new vs 26.1.2 | O7 |
| O8 runtime-failure layer unmeasured | gap | **REPRODUCES** — plus one concrete runtime crash (ComponentPatch Removed) | O8 |
| O9 re-ported 26.3-form sources (17 files) | gap | **REPRODUCES** — same 17 files failing, 5 worse; provenance differs (E6-chain inheritance, not a 1.21.11 re-port; 8 byte-identical to 26.1.2's, 9 differ in source) | O9 |

## The client_probe check (H01)

**Verdict: H01 does NOT reproduce in 1.21.10 — `client_probe` compiles
with no edits. No edit was made; the probe file is unmodified.**

In 26.1, `client_probe` failed to compile with E0560:
`game_loop_probe.rs` initialised `FrameContext` fields `focused` and
`exclusive_fullscreen` that 26.1's `FrameContext` does not have, so the
two lines had to be deleted. 1.21.10's `FrameContext`
(`crates/horizon-client-input/src/window.rs:56-66`) has exactly 7
fields — `inactivity`, `iconified`, `now_ms`, `latest_input_ms`,
`has_level`, `has_screen`, `has_overlay` — with NO `focused` and NO
`exclusive_fullscreen` (they are 26.x additions), and the 1.21.10
probe's `FrameContext` literal (`game_loop_probe.rs:152-164`)
initialises exactly those 7 fields. The probe source differs from
26.1.2's by exactly the two lines the brief predicted (26.1.2 adds
`focused: b(&c["focused"]),` and `exclusive_fullscreen: false,` at
lines 159-160 — verified by `diff`), so the probe matches its own
tree and the H01 error cannot occur. The probe file is unmodified
(mtime 21:04 UTC, before this batch started).

Check (finished 23:17 UTC Nov 24, i.e. 05:02 NPT Nov 25):

```
cd versions/1.21.10
RUSTUP_HOME=~/workspace/tools/rustup CARGO_HOME=~/workspace/tools/cargo \
  CARGO_TARGET_DIR=~/workspace/mcir-work/sweep-1.21.10-batches/b0/target \
  PATH=~/workspace/tools/jdk-25/bin:~/workspace/tools/cargo/bin:$PATH \
  taskset -c 0 cargo check --locked -j 1 -p horizon-client-lib --example client_probe
```

Note the package: the `client_probe` target is declared in
`crates/horizon-client-lib/Cargo.toml:201-202` (path
`../horizon-client/examples/probe/main.rs`), not in
`horizon-client/Cargo.toml`; this is the selection the harnesses use
(`tools/oracle_harness.py:38` runs `cargo build --locked -p
horizon-client-lib --example client_probe`). A batch-local
`CARGO_TARGET_DIR` was used so this check would not contend with a
sibling batch's cargo work in the worktree's own `target/` dir.

- First run (`evidence/client-probe-check.log`): started on a cold
  `target/` (no warm 1.21.10 target exists anywhere on the VM) and
  stopped after ~50 min with only 10 units done (syn v3.0.6 alone took
  ~40 min at load ~30) to seed the cache: the 1.21.4 port workdir's
  warm `target/` (1.4 GB, 421 rmeta files) was rsynced in
  (`--ignore-existing`, files unmodified for >5 min only, excluding
  `incremental/` and the lock file, so no live write from the
  concurrently-running 1.21.4 check could be copied mid-write —
  file list in `evidence/rsync-filelist.txt`). Cargo fingerprinting
  governs reuse — stale absolute paths in the rsynced dep-info force
  rebuild where needed — and the verdict below rests on artifacts
  rewritten during the 1.21.10 run itself, not on reused ones.
- Second run (`evidence/client-probe-check2.log`): EXIT=0 in 35m 37s,
  no errors, no warnings (the 4 "error" matches in the log are the
  `thiserror` crate names). The log's last unit is "Checking
  horizon-client-lib": cargo displays an example unit's status line
  under its package name, so no "Checking client_probe" line ever
  appears — the example compile is evidenced by its rewritten
  artifacts, not by a log line. The example's
  `target/debug/examples/libclient_probe-7f5c9e82df826d1b.rmeta`
  (23:17:39 UTC), its dep-info (23:17:42 UTC) and its fingerprint
  (`.fingerprint/horizon-client-lib-7f5c9e82df826d1b/example-client_probe*`,
  23:18:05 UTC) were all rewritten during this run's final minute, and
  the dep-info references the 1.21.10 probe sources
  (`crates/horizon-client-lib/../horizon-client/examples/probe/…`),
  proving rustc compiled the example from the current 1.21.10 sources.
  The lib itself was also rebuilt from 1.21.10 sources (fresh
  `libhorizon_client_lib-c18f716cd608a685.rmeta`, 23:14:42 UTC,
  17,753,878 bytes — a different metadata hash and size from the two
  seeded 1.21.4 rmetas).
- After-state: compiles. No lines deleted, no edit, no slice.json item —
  H01 is closed for 1.21.10 and blocks no harness here. (`cargo build`
  of the probe binary itself, with codegen, was not run: the check
  verdict is what H01 asks, and a full probe build is Batch 9's job —
  a CI-or-quiet-window job under this VM's memory budget.)

## Per-oracle table

`batch-0-oracles.json` — all 391 files: pass/fail per classpath,
first-error class + message + error count for failures (same shape as
26.1.2's table; first-error attribution prefers the client cp, falling
back to server — the 26.1.2 convention, verified against that table).
Raw big-compile logs (5,761 server-cp / 3,328 client-cp errors) are in
`evidence/` (copies of the measuring workdir's
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
  (first run killed by a service restart during the client fixpoint;
  second run re-measured client + both).
- `evidence/group.py`, `evidence/groups.txt` — group derivation script
  + output (error-text and source-pattern matching over the logs).
- `evidence/diff-26.1.2.json` — file-by-file deviation lists vs the
  26.1.2 table (newpass / newfail / changed_err).
- `evidence/server-jar-listing.txt`,
  `evidence/client-jar-listing.txt` — 1.21.10 jar class listings
  (used for the package-absence verifications).
- `evidence/server-jar-1.21.11-listing.txt` — 1.21.11 server-game jar
  listing (used to date the 1.21.11-era package moves).
- `evidence/client-probe-check.log` (first run, cold target, stopped
  after ~50 min) and `evidence/client-probe-check2.log` (seeded run,
  EXIT=0 in 35m 37s) — H01 cargo check output.
- `evidence/rsync-filelist.txt` — file list used for the target-dir
  seed (stable files only, from the 1.21.4 port workdir's warm
  target).
- Jar facts verified by javap on the 1.21.10 named jars:
  ServerLevel 11-arg ctor; LocalPlayer 7-arg ctor; Level.fuelValues()
  abstract; EntityType.PLAYER/CHICKEN + BlockEntityType.BED statics;
  Feature ctor `Codec<FC>` only; ServerChunkCache 11-arg ctor;
  PackResources (in `net.minecraft.server.packs`) without forEach;
  AgeableMob without age-lock methods; Items without spears.

## Verdict

**104/391 oracles compile (26.6%); 287 fail on all three classpaths
(73.4%); 314 of 648 drivers are oracle-blocked.** The measurement is
complete and reproducible (fixpoint big-compile, same script and
method as every prior run; the one rerun reproduced the killed run's
numbers exactly). The headline cause is the ServerLevel constructor
(86 files, 23 single-cause) plus the 1.21.11-era package moves
(gamerules, entity sub-packages, world.attribute, gizmos, permissions)
— neither of which the 26.x runs had at this scale — layered on the
familiar 26.3-form groups (densityfunction 17, EntityTypes 32,
LocalPlayer 20, renderpearl 8, fuelValues 5). The 1.21.7 sweep's
repairs are NOT in this tree and must be re-derived for 1.21.10
(11-arg ServerLevel, not 12-arg). Only 2 of the 9 harness oracles
pass; Batch 9's command_parse run is oracle-blocked on
`VanillaCommandExecOracle` (45 errors). Prepared minecraft data was
built by this batch (gate OK); prepared horizon-client data already
exists in the port workdir (523 MB, complete). H01 does NOT reproduce:
`client_probe` compiles (cargo check EXIT=0 in 35m 37s, no edits) —
the probe matches its own 7-field `FrameContext`.
