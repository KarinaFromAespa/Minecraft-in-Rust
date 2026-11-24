# Batch 0 measurement: oracle compilability against the 26.1 jars

Date: 2026-11-24. Branch: `parity-sweep-26.1`. Measured before any repair.
Companion data: `docs/sweep/26.1/batch-0-oracles.json` (per-oracle table, all
391 files) and `docs/sweep/measure-oracles.py` (the measurement script,
rerunnable).

**Scope decision (coordinator, per the sweep brief): Batch 0 for 26.1 is
measurement + triage ONLY — no oracle repairs.** The brief limits the sweep
to checks + triage and forbids touching `versions/26.1/` code; the oracle
`.java` files live in `versions/26.1/mods/horizon/`. The 1.21.7 Batch 0 did
repair oracles (135/391 pass after repair, up from 94); for 26.1 the repair
list below is a follow-up decision for the parent. Batches use static checks
(javap + source + data) for drivers whose oracles don't compile, as 1.21.7
Batch 2 did.

## Headline

**199 of 391 oracles (50.9%) compile against the 26.1 jars on at least one
classpath; 192 (49.1%) fail on all three.** The failure rate sits between
26.2's 33% and 1.21.7's pre-repair 76% — expected, since the oracles were
written against 26.3 and 26.1 is two versions back, predating the
`EntityTypes` holder move that 26.2 already had. The breakage is
concentrated in a few systematic 26.1-vs-26.3 API deltas plus a long tail of
1–3-error files: median 3 errors per failing file, 148 of 175 with ≤10
errors, 51 with exactly 1 error.

| Classpath (driver configuration) | Pass | Fail | Failure rate |
|---|---|---|---|
| server — server-game.jar + server libraries + jsr305 (direct drivers) | 168 | 223 | **57.0%** |
| client — client.jar + client libraries + jsr305 (oracle_harness drivers) | 199 | 192 | **49.1%** |
| both — server + client jars + both library sets (superset) | 199 | 192 | 49.1% |
| **fail on all three classpaths (genuinely broken vs 26.1)** | — | **192** | **49.1%** |

Notes:

- The `both` set is identical to the `client` set, and every server-pass file
  also passes on client cp: the client jar contains the shared/server classes
  these oracles use, so client cp is the superset that matters. The 31 files
  that pass on client but fail on server are client-only oracles
  (`net.minecraft.client.*`, blaze3d) — expected, not breakage.
- The client-cp measurement adds jsr305 to the classpath. The harness itself
  does not (see §5, driver defect 1): without jsr305, javac can crash
  (`CompletionFailure: class file for javax.annotation.Nullable not found`,
  rc=4) on some oracles. The measurement isolates oracle breakage from that
  harness bug; the harness bug is recorded separately and NOT fixed in this
  batch (scope decision above).
- Error totals: server cp 4,220 errors in 208 files; client cp 1,310 errors
  in 175 files (both cp identical). For comparison, 26.2 was 3,913 server /
  894 client errors; 1.21.7 pre-repair was 6,199 server / 3,916 client.

## Method

Same fixpoint big-compile as the 1.21.7/26.2 measurements
(`docs/sweep/measure-oracles.py`, unchanged): per classpath, compile all 391
files together, then iterate the clean set (sourcepath=False) until the
compile passes. Fixpoint iterations: server 1 round (15 cascade drops),
client 2 rounds (16 + 1 cascade drops), both 2 rounds (16 + 1 cascade
drops). Total runtime ~9.9 min under load (the horizon-prepare build ran
concurrently on the same VM).

Jars: the named 26.1 jars from the prepared `.local/minecraft/26.1/`
(see §6); `javac --release 25 -proc:none`, JDK 25.0.5. The 26.1
server-game.jar IS named (Mojang-mapped classes and methods) — the
"no named jar exists for any 26.x version" note in PARITY-SWEEP.md §7
(Nov 22) is outdated for 26.1 as it was for 26.2. Note the release flag:
26.1's `java_major` is 25 (26.2's was 21), matching the version
manifest's `javaVersion.majorVersion`.

## Failure classes (first error per file)

Server cp (223 failures):

| Class | Files | Meaning |
|---|---|---|
| missing-package | 98 | imports a package that does not exist on the server cp (mostly client-only packages — 31 of these pass on client cp) |
| missing-symbol | 67 | references a class/method/field 26.1 does not have (renamed/moved API) |
| no-override | 22 | `@Override` of a method 26.1's supertype lacks |
| cascade | 15 | file itself is clean; fails only because a helper it pulls is broken |
| other | 7 | assorted (static-context, access, …) |
| missing-override | 5 | abstract-method gap |
| constructor-mismatch | 5 | calls a constructor with a 26.3 signature |
| incompatible-types | 4 | 26.3 vs 26.1 signature change |

Client cp (192 failures): missing-symbol 88, no-override 27,
missing-package 23, constructor-mismatch 18, cascade 17, other 9,
missing-override 6, incompatible-types 4. The client cp sheds the
client-package failures and exposes the real drift: missing symbols
dominate (88, vs 26.2's 51) — the `EntityTypes` holder move alone
accounts for 31 files' symbol errors.

### The systematic groups (repair list for the follow-up decision)

**G1 — `Level.fuelValues()` abstract-method gap (4 oracles, 1 error each,
plus 1 more referencing it).** 26.1's `Level` declares
`public abstract FuelValues fuelValues()` (javap-verified:
`net.minecraft.world.level.block.entity.FuelValues`); 26.3's does not (or
not abstractly), so the oracles' fake `Level` subclasses don't implement
it: `VanillaActiveEffectTickOracle`,
`VanillaAttributeUpdatesOracle`, `VanillaBlockEntityAnimOracle`
(3 errors), `VanillaLivingLandingOracle`.
`VanillaBlockEntityRenderOracle` also references `fuelValues()` among its
9 errors. Mechanical repair: add a `fuelValues()` override returning a
default to each fake level. (Two more missing-override files have
different abstract gaps: `VanillaDecorationRegionOracle` /
`VanillaFeatureCatalogOracle` — anonymous classes missing a `lookup…`
override. Note the 26.2 group's third member differs here: 26.1's
`VanillaSectionMeshOracle` fails as `other` — `no interface expected
here` — not on `MaterialBaker.bake`.)

**G2 — `LocalPlayer` constructor (20 oracles carry the error; 11 as first
error).** 26.1's `LocalPlayer` constructor takes 8 arguments
(`Minecraft, ClientLevel, ClientPacketListener, StatsCounter,
ClientRecipeBook, Input, boolean, ChatAbilities` — javap-verified on the
26.1 client jar); the oracles pass the 26.3 9-argument form. First-error
files: `VanillaAllayInteractionStateOracle`,
`VanillaAmbientOracle`, `VanillaAutoJumpOracle`, `VanillaCameraOracle`,
`VanillaFireStarterSoundOracle`, `VanillaLocalInputOracle`,
`VanillaMarkerParticlesOracle`, `VanillaPlayerSyncOracle`,
`VanillaRemoteAttackOracle`, `VanillaSheepDyeOracle`,
`VanillaSoundEngineOracle`. Nine more carry a `LocalPlayer` ctor error
behind another first error: `VanillaBoatRiderOracle`,
`VanillaFirstPersonOracle`, `VanillaGameLoopOracle`,
`VanillaHeldDropOracle`, `VanillaHeldItemStackOracle`,
`VanillaHudOracle`, `VanillaInteractionInputOracle`,
`VanillaSprintInputOracle`, `VanillaTeamRulesOracle`. One signature
translation, verified once against the 26.1 jar, rescues the 7
single-cause files outright (`VanillaAmbientOracle`,
`VanillaAutoJumpOracle`, `VanillaCameraOracle`,
`VanillaFireStarterSoundOracle`, `VanillaLocalInputOracle`,
`VanillaMarkerParticlesOracle`, `VanillaPlayerSyncOracle`); the other 13
carry additional unrelated errors.

**G3 — `densityfunction` package absent from 26.1 (17 oracles).**
`net.minecraft.world.level.levelgen.densityfunction` has ZERO entries in
the 26.1 server-game.jar (verified by jar listing) — the
`SamplerContext`/`DensityVolume` API is a 26.3 addition, not a 26.1 one.
Files: `VanillaAnchorOracle` (10 errors, blocks 9 drivers),
`VanillaBiomeGenerationOracle` (12),
`VanillaBiomeSourceCodecOracle` (9), `VanillaBiomeSourceOracle` (10),
`VanillaDensityOracle` (251 errors, the largest single failure),
`VanillaFloatLogOracle` (9), `VanillaMaterialConditionOracle` (76 errors,
blocks 5 drivers), `VanillaMaterialExtensionOracle` (12),
`VanillaMaterialStageOracle` (26), `VanillaMaterialSurfaceOracle` (15),
`VanillaNoiseOracle` (32), `VanillaStructureDensityOracle` (4),
`VanillaStructurePlacementOracle` (7),
`VanillaStructureStartOracle` (2), `VanillaTerrainOracle` (13),
`VanillaWorldSpawnOracle` (5), `VanillaWorldgenOracle` (42). Several
files reference the package fully-qualified
(`…densityfunction.SamplerContext.builder()…`) rather than by import, so
the package error hides behind a symbol first error in some files.
`VanillaStructureDensityOracle` is unrepairable by design (it directly
exercises the 26.3-only API; density is verified via bytecode/chunk
comparison instead — same ruling as 1.21.7 and 26.2). The others need
per-file translation to 26.1's actual density API, not import fixes.

**G4 — `EntityTypes` holder move (31 oracles; the 26.1-specific group).**
26.1 predates the `EntityTypes`/`BlockEntityTypes` move — registration
still lives in `net.minecraft.world.entity.EntityType` /
`net.minecraft.world.level.block.entity.BlockEntityType` (the 1.21.x
shape). `EntityTypes` and `BlockEntityTypes` have ZERO entries in the
26.1 server-game.jar (verified by jar listing); the constants the oracles
want exist on the old holder instead (`EntityType.PLAYER`,
`EntityType.CHICKEN`, `EntityType.VILLAGER`,
`EntityType.WANDERING_TRADER` — javap-verified static finals), and
`BlockEntityType.BED` exists (26.1 keeps the bed block-entity type).
Every oracle referencing `EntityTypes.X` fails: `EntityTranscript`,
`KnockbackClient`, `LocatorBarClient` (13 errors, blocks 8 drivers),
`RideTranscript`, `VanillaAllayInteractionStateOracle`,
`VanillaBlockEntityAnimOracle`, `VanillaBoatAuthorityOracle`,
`VanillaBoatRiderOracle`, `VanillaBreezeHookOracle`,
`VanillaDispenserOracle` (15 errors, blocks 6 drivers),
`VanillaEntityDeathLootOracle`, `VanillaFinalizeSpawnOracle`,
`VanillaItemModelOracle`, `VanillaLateEntityPairOracle`,
`VanillaMenuOracle` (4 errors, blocks 4 drivers plus 2 cascades),
`VanillaMobAiOracle`, `VanillaMobBreathingOracle`,
`VanillaMobFallResetOracle`, `VanillaNaturalSpawnOracle`,
`VanillaProtocolClient` (34 errors, blocks 8 drivers),
`VanillaRegistryFallRayOracle`, `VanillaRemoteAttackOracle`,
`VanillaSceneMotionOracle`, `VanillaShulkerBulletOracle`,
`VanillaShulkerSpaceOracle`, `VanillaSourceSilenceOracle`,
`VanillaSprintInputOracle`, `VanillaSquidClientOracle`,
`VanillaTradingOracle`, `VanillaWetFireOracle`, `VanillaWorldRayOracle`
(9 by import, 22 fully-qualified). Repair is a mechanical
`EntityTypes.X` → `EntityType.X` holder translation per reference, but
most files carry additional unrelated errors, so the translation alone
rescues few outright. This group does not exist for 26.2 (26.2 already
has the moved holder) and is the single biggest reason 26.1 measures
worse than 26.2.

**G5 — `renderpearl` package absent from 26.1 (8 oracles).**
`com.mojang.renderpearl.api.pipeline` (and `.buffers`) do not exist in
26.1 — the render-pipeline API is 26.3-only (0 entries in the client
jar). Files: `VanillaHudOracle` (34 errors), `VanillaLoomScreenOracle`
(5), `VanillaPackScreenOracle` (13), `VanillaRecipeBookScreenOracle` (4),
`VanillaSpriteAnimationOracle` (2), `VanillaTabListOracle` (36, also
authlib — G7), `VanillaTranslucencyOracle` (4),
`VanillaWorldBorderRenderOracle` (34). These need rewrites against
26.1's screen/render APIs, not import fixes.

**G6 — Other constructor mismatches (8 files in the residual, plus
overlaps).** `VanillaDecorationOracle` (`Feature` ctor, 4 errors —
26.1's takes only `Codec<FC>`), `VanillaFallingBlockOracle`
(`ChunkMap` ctor), `VanillaInventoryUseStartOracle` +
`VanillaItemUseStartOracle` (`ServerChunkCache` ctor),
`VanillaEntitySoundPacketOracle` (`MinecraftServer` ctor),
`VanillaHeldDropOracle` + `VanillaInteractionInputOracle` +
`VanillaPackScreenOracle` (`Gui` ctor — 26.1's takes only `Minecraft`),
and `VanillaBlockUpdateOracle` / `VanillaGameLoopOracle` /
`VanillaMobAiOracle` / `VanillaRedstoneCatalogOracle` /
`VanillaSculkOracle` / `VanillaTradingOracle` (also `MinecraftServer`
ctor, behind other first errors). `VanillaGameLoopOracle` additionally
fails on a `CommandSourceStack` constructor form. Each is a
one-signature translation against the 26.1 jar. `VanillaNaturalSpawnOracle`
carries two ctor mismatches at once (`ServerChunkCache`, `ChunkMap`)
plus `EntityTypes` (G4).

**G7 — Other missing packages (9 oracles).**
`net.minecraft.world.level.levelgen.material.rule` does not exist in
26.1 (the 26.3 material-system package, 0 jar entries):
`VanillaBiomeGenerationOracle` (12 errors), `VanillaMaterialStageOracle`
(26), `VanillaTerrainOracle` (13) as first error, plus
`VanillaAnchorOracle` and `VanillaMaterialConditionOracle`, which also
import it (both also G3). The 26.3 advancements packages do not exist in
26.1 — only `advancements/criterion` does (verified by jar listing):
`ComponentWireCapture` (`advancements.predicates.StatePropertiesPredicate`,
1 error), `VanillaEntityComponentsOracle`
(`advancements.predicates.entity`, 3 errors), `VanillaRaidOracle`
(`advancements.triggers`, 11 errors). `VanillaTabListOracle` (36 errors)
imports `com.mojang.authlib.services`, absent from authlib 7.0.63 (0
entries in the library jar); and `VanillaTickScript` (19 errors) imports
`org.lwjgl.sdl`, absent from the 26.1 libraries (no lwjgl-sdl jar;
lwjgl 3.4.1 has no `org/lwjgl/sdl`). Each needs per-file investigation
against the 26.1 jar/libraries.

**G8 — The long tail (94 files after the groups above and the cascades
are set aside: 58 missing-symbol + 21 no-override + 5
constructor-mismatch + 6 other + 4 incompatible-types).** Per-API
translation, verified against the 26.1 jar per file. 37 of the 94 have
exactly 1 error. The cheapest high-leverage single repair in the whole
batch is here: `VanillaFaceSupportOracle` — 1 error
(`CubeVoxelShape(DiscreteVoxelShape)` has protected access in 26.1 —
javap-verified), blocking 19 drivers, plus 5 cascade oracles depend on
it. `VanillaRegistryCollisionScanOracle` (8 drivers, 1 error),
`ParityFont` (8 drivers, 1 error — same cheapest-repair as 26.2),
`VanillaSuffocationOracle` (7 drivers, 2 errors) and
`VanillaCollisionContextOracle` (5 drivers, 1 error) are the other
one-error blockers. `VanillaBlockUpdateOracle` (32 errors, 6 drivers)
anchors 6 cascade oracles. Notable single-file causes in the tail:
`VanillaFirstPersonOracle` (26.3-only `FirstPersonHandsAndItems`, 0
entries in the client jar), `VanillaItemModelOracle`
(`ResourceManager.Selector`, `ItemQuads` — both 0 entries;
`MaterialInfo.shadeDirectionOverride()`, `BundleContents.asMutable()`,
`MaterialBaker` abstract in 26.1), `VanillaPackResourcesOracle`
(`PackResources.forEach` absent in 26.1 — 1 error),
`VanillaStructurePlacementOracle` / `VanillaStructureStartOracle` /
`VanillaWorldSpawnOracle` (`RandomState.create(Registry<NoiseParameters>,
long, NoiseGeneratorSettings)` — no such overload in 26.1; its two
`create` overloads take a `HolderGetter.Provider` + settings key, or
settings + `HolderGetter<NoiseParameters>`).

**G9 — Cascades (17 oracles).** Clean files failing only via a broken
helper: via `VanillaFaceSupportOracle` (5) —
`VanillaBlockCollisionScanOracle`, `VanillaDynamicSupportOracle`,
`VanillaEntityCollisionOracle`, `VanillaPistonShapeOracle`,
`VanillaWorldBorderOracle`; via `VanillaBlockUpdateOracle` (6) —
`VanillaBrewingOracle`, `VanillaEnchantDurabilityOracle`,
`VanillaFurnaceOracle`, `VanillaHelmetDamageOracle`,
`VanillaOpenedLootOracle`, `VanillaPortalOracle`; via
`VanillaMenuOracle` (2) — `VanillaPlaceRecipeOracle`,
`VanillaRecipeBookOracle`; via `VanillaAnchorOracle` (1) —
`VanillaHeightProviderOracle`; via `VanillaSoundEventOracle` (2, chained)
— `VanillaSoundRig`, then `VanillaMusicOracle` via `VanillaSoundRig`;
via `VanillaTextDisplayOracle` (1) — `VanillaTextDisplayTransformOracle`.
These pass for free once their helpers are repaired — do not schedule
them separately.

18 oracles are never referenced by any driver; all 18 pass, so no failing
oracle is dead code (26.2 had 6 fail-all oracles with no driver).

## Driver impact

Driver→oracle map built by parsing all 647 drivers (`.java` basenames +
oracle-name string literals passed to `oracle_harness`). Each driver's
oracle set was checked against the classpath its own path uses
(oracle_harness → both, direct `jars['server']` → server, client jar →
client, both jars → both). 20 drivers reference an oracle without a
detectable classpath (they compile via a shared `vanilla.compile(...)`
helper); those were judged against the `both` superset — 13 blocked, 7
runnable.

| Driver state | Drivers |
|---|---|
| **Blocked: ≥1 oracle fails to compile under its path's cp** | **221** |
| Oracles compile under their path's cp (can yield a measurement when run) | 173 |
| No oracle reference (server-e2e, data-only, wrappers) — not oracle-blocked | 253 |
| **Total** | **647** |

- 426 drivers have no oracle blocker (173 + 253), vs 509 for 26.2
  (248 + 261) and 321 for 1.21.7 pre-repair (67 + 254). The 253
  no-oracle drivers were never oracle-blocked, but that does not mean
  they pass: they run real servers / the game / data diffs and have
  their own failure modes.
- Blocked by classpath: server 106, both 74, client 28, no-flag
  (judged on both) 13.
- The highest-leverage repairs are shared helpers: `VanillaFaceSupportOracle`
  blocks 19 drivers (1 error — cheapest), `VanillaAnchorOracle` 9
  (10 errors, densityfunction — G3), `VanillaRegistryCollisionScanOracle`
  8 (1 error), `VanillaProtocolClient` 8 (34 errors),
  `ParityFont` 8 (1 error), `LocatorBarClient` 8 (13 errors),
  `VanillaSuffocationOracle` 7 (2 errors), `VanillaDispenserOracle` 6
  (15 errors, EntityTypes — G4), `VanillaBlockUpdateOracle` 6
  (32 errors, plus 6 cascades), `VanillaMaterialConditionOracle` 5
  (76 errors, densityfunction — G3), `VanillaCollisionContextOracle` 5
  (1 error). Fixing the top few one-error helpers unblocks ~50
  driver-oracle pairs (overlaps exist).
- Compiling ≠ running: the 173 "runnable" drivers are runnable by
  javac only. No driver was run in this batch (scope decision above),
  so the runtime-failure layer the 1.21.7 sample found (a compiling
  oracle throwing NPE at runtime) is unmeasured for 26.1. Every
  repaired oracle needs an end-to-end driver run to confirm it yields
  a measurement.

## The 9-harness check (1.21.7's table, re-measured for 26.1)

Of the 9 harnesses the 1.21.7 plan (§2) records as dying on oracle javac
failures in CI run 44441686979, **6 pass on at least one classpath for
26.1 at measurement time; 3 fail on all three and need repair before
their drivers can run** (for 26.2 all 9 passed):

| Harness | Oracle | 26.1 status |
|---|---|---|
| chunk_generation | VanillaChunkGenerationOracle | **PASS** (all cps) |
| command_exec | VanillaCommandExecOracle | **PASS** (all cps) |
| component_patch | VanillaComponentPatchOracle | **PASS** (all cps) |
| datafix | VanillaDataFixOracle | **PASS** (all cps) |
| first_person | VanillaFirstPersonOracle | **FAIL** (all cps; client 11 errors: 26.3-only `FirstPersonHandsAndItems`, LocalPlayer ctor, 2 no-override, `itemUsed(InteractionHand)`) |
| food | VanillaFoodOracle | **PASS** (all cps) |
| item_model | VanillaItemModelOracle | **FAIL** (all cps; client 10 errors: EntityTypes, `ResourceManager.Selector`, `ItemQuads`, `shadeDirectionOverride()`, `createWorldLookup()`, `MaterialBaker` abstract, `asMutable()`) |
| options | VanillaOptionsOracle | **PASS** (client + both; server fails on client packages — expected) |
| resource_pack | VanillaPackResourcesOracle | **FAIL** (all cps; client 1 error: `PackResources.forEach` absent in 26.1) |

For 1.21.7, 8 of these passed only AFTER Batch 0 repairs and `options`
still failed. For 26.1 the `command_parse`/`command_exec`/`food`/
`component_patch` drivers — the ones that yielded the 1.21.7 sweep's
first real parity measurements — are oracle-clean from the start, but
`first_person`, `item_model` and `resource_pack` are not: the
`resource_pack` failure is a single missing method (cheapest of the
three), while `first_person` and `item_model` need per-API translation
against 26.1's client classes.

## Driver defects found (Batch 0 scope)

1. **`client_libraries()` omits jsr305 — javac can crash (latent).**
   `versions/26.1/tools/test_model_baking_parity.py:client_libraries()`
   (lines 58–68) builds the client classpath from the manifest only;
   it does not append `tools/jsr305-3.0.2.jar`. The 1.21.7 Batch 0 fix
   for the identical defect (`d7a14e91`) is NOT in the 26.1 tree
   (same gap as 26.2). Without jsr305, javac crashes
   (`CompletionFailure: class file for javax.annotation.Nullable not
   found`, rc=4) when compiling an oracle that completes a
   `Nullable`-referencing symbol — a latent crash across the
   oracle_harness drivers, not a deterministic one. The measurement
   adds jsr305 itself, so this did not affect the numbers above. Fix
   direction (follow-up, not this batch): append jsr305 in
   `client_libraries()` — one point, fixes all users.
2. **`.local/minecraft/26.1/` did not exist in any worktree** (same gap
   as the 1.21.7 pilot's blocker and 26.2's). Resolved — see §6. No
   driver could run VM-local before this.

## `.local/minecraft/26.1/` setup (recipe for later batches)

`verify_artifacts()` needs `manifest.json` (pinned sha1), `jars/client.jar`
+ `jars/server.jar` (manifest sha1s or `remapped.json` digests), the inner
`jars/server-game.jar` (bundle checksum or `remapped.json`), and
`libraries/server/` + `libraries/client/` (sha-verified). No worktree had
it; unlike 26.2 (which reused a complete prepared directory from the port
work), 26.1 was built fresh from the research jars with `lab.py prepare`:

```sh
# 1. Launcher-style source dir: the 26.1 client jar + version.json, plus the
#    107 client libraries from version.json, each sha1-checked (public Mojang
#    URLs, no account needed). Script: sweep-26.1-batches/b0/fetch-mc-source.py
#    (writes ~/workspace/mcir-work/sweep-26.1/mc-source/; 80.7 MB of libraries).
cd ~/workspace/mcir-work/sweep-26.1/versions/26.1
python3 tools/lab.py prepare --minecraft-dir ~/workspace/mcir-work/sweep-26.1/mc-source --server-jar ~/workspace/mcir-work/26.1.x/versions/26.1/jars/server.jar
python3 -c "import sys; sys.path.insert(0,'tools'); import test_rust_parity as t; t.verify_artifacts(); print('GATE OK')"
```

`lab.py prepare` imports the client + server, unpacks `server-game.jar`
+ the server libraries FROM the bundler jar, downloads vineflower, and
writes `prepared.json` + `reports/` — no remap stage runs, because the
26.1 version manifest's `downloads` has only client+server (no mappings).
Result (226 MB): named client jar (10,682 classes) + named inner server
jar (7,351 classes), 39 server + 60 client libraries, `manifest.json`
sha1 `389f9251…` (the pinned one in research-lock.json). Verified: the
gate passes. Never commit `.local`.

Prepared horizon-client data did not exist anywhere on the VM for 26.1,
so `horizon-prepare` was built (`taskset -c 0`, `-j 1`, 11m25s) and run
(`./target/release/horizon-prepare --minecraft-dir .local/minecraft/26.1`;
completed in 494.2 s, exit 0; slowest steps were movement and attributes,
180.7 s, block behavior, 144.0 s, and the datagen run, 78.7 s, then the
creative-inventory export, 66.3 s, model bindings, 60.3 s, block states,
57.6 s, and command registries, 53.8 s).
The run completed in full, with these outputs present in
`.local/horizon-client/26.1/` (190 MB): registries, items, block-tags,
facts, mob-data, entity-models, model-states, creative-inventory,
command-registries, commands, attributes, block-behaviours,
gameplay-facts, entity-types, damage catalog/types/tags, datafix-tables,
movement-properties, landing-definitions, cached-collision, occlusion,
render-definitions, player-float-ids, horizon-resources, data/,
worldgen/, biome-parameters/, assets/. Asset-dependent exports skipped
as expected — panorama, fonts (Unifont glyphs), packs (Programmer Art /
High Contrast) and the languages import need the official launcher's
asset index (`assets/indexes/30.json`), which the VM lacks; sounds need
the separate `horizon-prepare sounds` step against a launcher asset
store and were not run. **Lang files are NOT absent, despite the
"Languages not imported" message**: `assets/minecraft/lang/en_us.json`
IS present (502,008 bytes, 7,886 keys) and md5-identical to the client
jar's file (`98deafdb…`) — it arrives via the client-jar asset copy,
not the skipped launcher-index import. Only `sounds.json` is genuinely
absent. Batches that need `sounds.json` must note the absence rather
than assume silence is a port defect; batches must NOT claim lang files
are missing.

## What the numbers mean

- Batch 0's repair load is **192 files**, not 129 — and the profile is
  heavier than 26.2's (median 3 errors vs 2; 51 one-error files vs 46,
  but a much longer tail: max 251, p90 13). The new 26.1-specific group
  is the `EntityTypes` holder move (G4, 31 files): 26.1 predates it,
  and the translation back to `EntityType.X` is mechanical and
  javap-verified, though most files carry other errors too. Two
  mechanical groups (G2 LocalPlayer ctor, 20 files with 7 single-cause;
  G1 fuelValues, 4 files) rescue files with two signature translations.
  The densityfunction (G3, 17 files) and renderpearl/material.rule (G5,
  G7) groups are the genuinely hard ones: 26.3-only APIs with no 26.1
  equivalent to translate to import-by-import — those oracles need
  rewrites against 26.1's actual APIs (or, for StructureDensity, stay
  unrepaired by design).
- Sequencing for batches 3–12: the 173 oracle-clean drivers can run
  once their probes build; the 221 blocked drivers wait on Batch 0
  repairs, shared helpers first (`VanillaFaceSupportOracle` unblocks
  19 drivers with a 1-error repair; `ParityFont` and
  `VanillaRegistryCollisionScanOracle` 8 each, also 1 error). The 253
  no-oracle drivers were never oracle-blocked. In practice this sweep's
  batches ran statically (javap + source + data) and recorded driver
  runs as follow-up needs — the VM's 6 GB budget under multi-agent load
  makes probe builds (client_probe OOMs single-threaded) a CI-or-quiet-
  window job, not a sweep-batch job.
- The 26.1-vs-26.3 API deltas this measurement exposes are themselves
  sweep input: the `EntityTypes`/`BlockEntityTypes` holder move (26.1
  still registers on `EntityType`/`BlockEntityType`, and keeps the bed
  BE type), `Level.fuelValues()` (abstract in 26.1), the
  `LocalPlayer`/`Gui`/`ServerChunkCache`/`ChunkMap`/`MinecraftServer`/
  `Feature` constructor forms, the absence of the `densityfunction`,
  `renderpearl` and `material.rule` APIs, the advancements
  `predicates`/`triggers` packages (26.1 has only `criterion`), the
  `RandomState.create` overloads, and the `MaterialBaker` /
  registry-lookup abstract methods. Family batches should check the
  port's Rust against 26.1's forms of these, not 26.3's.

## Per-oracle table

`docs/sweep/26.1/batch-0-oracles.json` — all 391 files: pass/fail per
classpath, first-error class + message + error count for failures. Raw
big-compile logs (4,220 server-cp / 1,310 client-cp errors) are in the
measuring workdir's `.local/measure-oracles/` (not committed —
regenerable with `docs/sweep/measure-oracles.py`).
