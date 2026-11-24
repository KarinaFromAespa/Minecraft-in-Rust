# Batch 0 measurement: oracle compilability against the 26.1.1 jars

Date: 2026-11-24. Branch: `parity-sweep-26.1.1`. Measured before any repair.
Companion data: `docs/sweep/26.1.1/batch-0-oracles.json` (per-oracle table, all
391 files) and `docs/sweep/measure-oracles.py` (the measurement script,
rerunnable).

**Scope decision (coordinator, per the sweep brief): Batch 0 for 26.1.1 is
measurement + triage ONLY — no oracle repairs — with ONE explicit exception:
the client_probe adaptation (H01, "The client_probe adaptation (H01)"
below).** The oracle `.java` files live in
`versions/26.1.1/mods/horizon/`. The 1.21.7 Batch 0 did repair oracles
(135/391 pass after repair, up from 94); for 26.1.1 the repair list below is
a follow-up decision for the parent. Batches use static checks (javap +
source + data) for drivers whose oracles don't compile, as 1.21.7 Batch 2
did.

## Headline

**189 of 391 oracles (48.3%) compile against the 26.1.1 jars on at least one
classpath; 202 (51.7%) fail on all three.** That is 10 files worse than 26.1
(199 pass / 192 fail-all) — and the regression is entirely in the oracle
sources, not in vanilla: the 26.1 → 26.1.1 jar delta is version stamps plus
one internal `PlayerEntry` change (see §1, G10), while the 26.1.1 tree
re-ported 19 oracle files from the 26.3 form, undoing 26.1's adaptations.
The breakage is concentrated in a few systematic 26.1.1-vs-26.3 API deltas
plus a long tail of 1–3-error files: median 3 errors per failing file, 155
of 186 with ≤10 errors, 55 with exactly 1 error.

| Classpath (driver configuration) | Pass | Fail | Failure rate |
|---|---|---|---|
| server — server-game.jar + server libraries + jsr305 (direct drivers) | 158 | 233 | **59.6%** |
| client — client.jar + client libraries + jsr305 (oracle_harness drivers) | 189 | 202 | **51.7%** |
| both — server + client jars + both library sets (superset) | 189 | 202 | 51.7% |
| **fail on all three classpaths (genuinely broken vs 26.1.1)** | — | **202** | **51.7%** |

Notes:

- The `both` set is identical to the `client` set, and every server-pass file
  also passes on client cp: the client jar contains the shared/server classes
  these oracles use, so client cp is the superset that matters. The 31 files
  that pass on client but fail on server are client-only oracles
  (`net.minecraft.client.*`, blaze3d) — expected, not breakage.
- The script's raw both-cp run recorded 188 pass: the fixpoint's third
  clean-set round dropped `ElderCurseTranscript` on a transient filesystem
  error (`error while writing ElderCurseTranscript: Operation not
  permitted`, `clean-both-3.log`), not a compile error. A standalone
  recompile of `ElderCurseTranscript` + `PhasedTranscript` against the both
  cp exits 0 (evidence `eldercurse-recheck/`), so the published table
  corrects it to pass and both cp reads 189. The fail-all count (202) is
  unaffected either way, since the file passes on client cp.
- The client-cp measurement adds jsr305 to the classpath. The harness itself
  does not (see §5, driver defect 1): without jsr305, javac can crash
  (`CompletionFailure: class file for javax.annotation.Nullable not found`,
  rc=4) on some oracles. The measurement isolates oracle breakage from that
  harness bug; the harness bug is recorded separately and NOT fixed in this
  batch (scope decision above).
- Error totals: server cp 4,286 errors in 219 files; client cp 1,391 errors
  in 186 files (both cp identical). For comparison, 26.1 was 4,220 server /
  1,310 client errors; 26.2 was 3,913 server / 894 client; 1.21.7 pre-repair
  was 6,199 server / 3,916 client.

## Method

Same fixpoint big-compile as the 1.21.7/26.2/26.1 measurements
(`docs/sweep/measure-oracles.py`, unchanged): per classpath, compile all 391
files together, then iterate the clean set (sourcepath=False) until the
compile passes. Fixpoint iterations: server 1 round (14 cascade drops),
client 2 rounds (15 + 1 cascade drops), both 3 rounds (15 + 1 + 1 cascade
drops, the last drop being the `ElderCurseTranscript` EPERM artifact above).
Total runtime ~6.8 min under load (the horizon-prepare run overlapped the
first part of it).

Jars: the named 26.1.1 jars from the prepared `.local/minecraft/26.1.1/`
(see §6); `javac --release 25 -proc:none`, JDK 25.0.5. The 26.1.1
server-game.jar IS named (Mojang-mapped classes and methods) — the
"no named jar exists for any 26.x version" note in PARITY-SWEEP.md §7
(Nov 22) is outdated for 26.1.1 as it was for 26.1 and 26.2. Note the release
flag: 26.1.1's `java_major` is 25, matching the version manifest's
`javaVersion.majorVersion` (DataVersion 4788, protocol 775).

## Failure classes (first error per file)

Server cp (233 failures):

| Class | Files | Meaning |
|---|---|---|
| missing-package | 98 | imports a package that does not exist on the server cp (mostly client-only packages — 31 of these pass on client cp) |
| missing-symbol | 78 | references a class/method/field 26.1.1 does not have (renamed/moved API) |
| no-override | 22 | `@Override` of a method 26.1.1's supertype lacks |
| cascade | 14 | file itself is clean; fails only because a helper it pulls is broken |
| other | 7 | assorted (static-context, access, …) |
| constructor-mismatch | 7 | calls a constructor with a 26.3 signature |
| missing-override | 4 | abstract-method gap |
| incompatible-types | 3 | 26.3 vs 26.1.1 signature change |

Client cp (202 failures): missing-symbol 99, no-override 27,
missing-package 23, constructor-mismatch 20, cascade 16, other 9,
missing-override 5, incompatible-types 3. The client cp sheds the
client-package failures and exposes the real drift: missing symbols
dominate (99, vs 26.1's 88) — the `EntityTypes` holder move alone
accounts for 31 files' symbol errors, and the re-ported Feature-form
files (G10) add 9 more newly-failing files to the count.

### The systematic groups (repair list for the follow-up decision)

**G1 — `Level.fuelValues()` abstract-method gap (4 oracles, 1 error each,
plus 1 more referencing it).** 26.1.1's `Level` declares
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
override. Note the 26.2 group's third member differs here: 26.1.1's
`VanillaSectionMeshOracle` fails as `other` — `no interface expected
here` — not on `MaterialBaker.bake`.)

**G2 — `LocalPlayer` constructor (20 oracles carry the error; 11 as first
error).** 26.1.1's `LocalPlayer` constructor takes 8 arguments
(`Minecraft, ClientLevel, ClientPacketListener, StatsCounter,
ClientRecipeBook, Input, boolean, ChatAbilities` — javap-verified on the
26.1.1 client jar); the oracles pass the 26.3 9-argument form. First-error
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
translation, verified once against the 26.1.1 jar, rescues the 7
single-cause files outright (`VanillaAmbientOracle`,
`VanillaAutoJumpOracle`, `VanillaCameraOracle`,
`VanillaFireStarterSoundOracle`, `VanillaLocalInputOracle`,
`VanillaMarkerParticlesOracle`, `VanillaPlayerSyncOracle`); the other 13
carry additional unrelated errors. Membership identical to 26.1.

**G3 — `densityfunction` package absent from 26.1.1 (17 oracles).**
`net.minecraft.world.level.levelgen.densityfunction` has ZERO entries in
the 26.1.1 server-game.jar AND client.jar (verified by jar listing) — the
`SamplerContext`/`DensityVolume` API is a 26.3 addition, not a 26.1.1 one.
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
comparison instead — same ruling as 1.21.7, 26.2 and 26.1). The others need
per-file translation to 26.1.1's actual density API, not import fixes.
Membership identical to 26.1.

**G4 — `EntityTypes` holder move (31 oracles; the 26.1-line group).**
26.1.1 predates the `EntityTypes`/`BlockEntityTypes` move — registration
still lives in `net.minecraft.world.entity.EntityType` /
`net.minecraft.world.level.block.entity.BlockEntityType` (the 1.21.x
shape). `EntityTypes` and `BlockEntityTypes` have ZERO entries in the
26.1.1 server-game.jar (verified by jar listing); the constants the oracles
want exist on the old holder instead (`EntityType.PLAYER`,
`EntityType.CHICKEN`, `EntityType.VILLAGER`,
`EntityType.WANDERING_TRADER` — javap-verified static finals), and
`BlockEntityType.BED` exists (26.1.1 keeps the bed block-entity type).
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
has the moved holder) and is the single biggest reason 26.1.1 measures
worse than 26.2. Membership identical to 26.1.

**G5 — `renderpearl` package absent from 26.1.1 (8 oracles).**
`com.mojang.renderpearl.api.pipeline` (and `.buffers`) do not exist in
26.1.1 — the render-pipeline API is 26.3-only (0 entries in the client
jar). Files: `VanillaHudOracle` (34 errors), `VanillaLoomScreenOracle`
(5), `VanillaPackScreenOracle` (13), `VanillaRecipeBookScreenOracle` (4),
`VanillaSpriteAnimationOracle` (2), `VanillaTabListOracle` (36, also
authlib — G7), `VanillaTranslucencyOracle` (4),
`VanillaWorldBorderRenderOracle` (34). These need rewrites against
26.1.1's screen/render APIs, not import fixes. Membership identical to
26.1.

**G6 — Other constructor mismatches (5 files as first error, plus
overlaps).** `VanillaDecorationOracle` (`Feature` ctor, 4 errors —
26.1.1's takes only `Codec<FC>`, javap-verified),
`VanillaFallingBlockOracle` (`ChunkMap` ctor),
`VanillaInventoryUseStartOracle` +
`VanillaItemUseStartOracle` (`ServerChunkCache` ctor),
`VanillaEntitySoundPacketOracle` (`MinecraftServer` ctor).
Behind other first errors: `Gui` ctor in `VanillaHeldDropOracle` +
`VanillaInteractionInputOracle` + `VanillaPackScreenOracle` (26.1.1's
takes only `Minecraft`, javap-verified), `MinecraftServer` ctor in
`VanillaBlockUpdateOracle` / `VanillaGameLoopOracle` /
`VanillaMobAiOracle` / `VanillaRedstoneCatalogOracle` /
`VanillaSculkOracle` / `VanillaTradingOracle`,
`ServerChunkCache`/`ChunkMap` in `VanillaNaturalSpawnOracle` (both at
once, plus `EntityTypes` — G4) and `VanillaBlockUpdateOracle`,
`DedicatedServer` ctor in `VanillaMenuOracle` /
`VanillaNaturalSpawnOracle`, `PerlinNoise`/`SimplexNoise` ctors in
`VanillaDensityOracle` / `VanillaNoiseOracle` (both G3), and two
packet/effect ctor forms — `ServerboundAcceptTeleportationPacket` in
`SecureChatClient`, `TeleportRandomlyConsumeEffect` in
`VanillaTeleportConsumeOracle`. `VanillaGameLoopOracle` additionally
fails on a `CommandSourceStack` constructor form. Each is a
one-signature translation against the 26.1.1 jar.

**G7 — Other missing packages (9 oracles).**
`net.minecraft.world.level.levelgen.material.rule` does not exist in
26.1.1 (the 26.3 material-system package, 0 jar entries):
`VanillaBiomeGenerationOracle` (12 errors), `VanillaMaterialStageOracle`
(26), `VanillaTerrainOracle` (13) as first error, plus
`VanillaAnchorOracle` and `VanillaMaterialConditionOracle`, which also
import it (all five also G3). The 26.3 advancements packages do not exist
in 26.1.1 — only `advancements/criterion` does (verified by jar listing):
`ComponentWireCapture` (`advancements.predicates.StatePropertiesPredicate`,
1 error), `VanillaEntityComponentsOracle`
(`advancements.predicates.entity`, 3 errors), `VanillaRaidOracle`
(`advancements.triggers`, 11 errors). `VanillaTabListOracle` (36 errors)
imports `com.mojang.authlib.services`, absent from authlib 7.0.63 (0
entries in the library jar); and `VanillaTickScript` (19 errors) imports
`org.lwjgl.sdl`, absent from the 26.1.1 libraries (no lwjgl-sdl jar;
lwjgl 3.4.1 has no `org/lwjgl/sdl`). Each needs per-file investigation
against the 26.1.1 jar/libraries.

**G8 — The long tail (91 files after the groups above, G10, the G7
advancements/sdl files and the cascades are set aside: 57 missing-symbol +
20 no-override + 5 constructor-mismatch + 6 other + 3
incompatible-types).** Per-API translation, verified against the 26.1.1
jar per file. 37 of the 91 have exactly 1 error. The cheapest high-leverage
single repair in the whole batch is here: `VanillaFaceSupportOracle` — 1
error (`CubeVoxelShape(DiscreteVoxelShape)` has protected access in
26.1.1 — javap-verified), blocking 19 drivers, plus 5 cascade oracles
depend on it. `VanillaRegistryCollisionScanOracle` (8 drivers, 1 error),
`ParityFont` (8 drivers, 1 error — same cheapest-repair as 26.2 and 26.1),
`VanillaSuffocationOracle` (7 drivers, 2 errors) and
`VanillaCollisionContextOracle` (5 drivers, 1 error) are the other
one-error blockers. `VanillaBlockUpdateOracle` (32 errors, 6 drivers)
anchors 5 cascade oracles. Notable single-file causes in the tail:
`VanillaFirstPersonOracle` (26.3-only `FirstPersonHandsAndItems`, 0
entries in the client jar — plus G10, see below),
`VanillaItemModelOracle` (`ResourceManager.Selector`, `ItemQuads` — both
0 entries; `MaterialInfo.shadeDirectionOverride()`,
`BundleContents.asMutable()` — no such method in 26.1.1, javap-verified;
`MaterialBaker` is an interface in 26.1.1, not a class),
`VanillaPackResourcesOracle` (`PackResources.forEach` absent in 26.1.1 —
1 error; the interface has only `getResource`/`listResources`,
javap-verified), `VanillaStructurePlacementOracle` /
`VanillaStructureStartOracle` / `VanillaWorldSpawnOracle`
(`RandomState.create(Registry<NoiseParameters>, long,
NoiseGeneratorSettings)` — no such overload in 26.1.1; its two `create`
overloads take a `HolderGetter.Provider` + settings key, or settings +
`HolderGetter<NoiseParameters>`, javap-verified).

**G9 — Cascades (16 oracles).** Clean files failing only via a broken
helper: via `VanillaFaceSupportOracle` (5) —
`VanillaBlockCollisionScanOracle`, `VanillaDynamicSupportOracle`,
`VanillaEntityCollisionOracle`, `VanillaPistonShapeOracle`,
`VanillaWorldBorderOracle`; via `VanillaBlockUpdateOracle` (5) —
`VanillaEnchantDurabilityOracle`, `VanillaFurnaceOracle`,
`VanillaHelmetDamageOracle`, `VanillaOpenedLootOracle`,
`VanillaPortalOracle`; via `VanillaMenuOracle` (2) —
`VanillaPlaceRecipeOracle`, `VanillaRecipeBookOracle`; via
`VanillaAnchorOracle` (1) — `VanillaHeightProviderOracle`; via
`VanillaSoundEventOracle` (2, chained) — `VanillaSoundRig`, then
`VanillaMusicOracle` via `VanillaSoundRig`; via
`VanillaTextDisplayOracle` (1) — `VanillaTextDisplayTransformOracle`.
These pass for free once their helpers are repaired — do not schedule
them separately. (26.1 had 17 cascades; `VanillaBrewingOracle` is no
longer one — it now has 14 own errors, G10.)

**G10 — Re-ported 26.3-form oracle sources (17 files; the 26.1.1-specific
group, and the whole +10 regression vs 26.1).** The 26.1.1 tree's parity
dir differs from the 26.1 tree's in 19 files (17 changed, 1 removed, 1
added), and the changed files are re-ports from the 26.3 form that undo
26.1's adaptations. The vanilla delta is NOT the cause: a class-bytes
diff of the 26.1 → 26.1.1 jars shows server-game.jar changed only
`DetectedVersion` + `SharedConstants` (version stamps), and client.jar
those two plus `PlayerEntry` — whose public API is unchanged (same
constructor) and which no oracle references. The 17 files that fail new
or worse vs 26.1, all in the re-ported set:

- 9 newly failing: `VanillaBasicFeatureOracle` (2 errors),
  `VanillaBiomePlacementOracle` (8), `VanillaBlockTransformerOracle` (3),
  `VanillaBrewingFuelOracle` (8), `VanillaFeaturePlacementOracle` (3),
  `VanillaFeatureSortOracle` (1), `VanillaFillLayerOracle` (1),
  `VanillaReplaceBlockOracle` (1), `VanillaScatteredOreOracle` (1).
- 7 with error-count growth: `VanillaBrewingOracle` (cascade → 14 own
  errors), `VanillaDecorationRegionOracle` (5 → 6),
  `VanillaFeatureCatalogOracle` (5 → 11), `VanillaFeatureGraphOracle`
  (3 → 4), `VanillaFeatureRegionOracle` (7 → 10),
  `VanillaFirstPersonOracle` (11 → 20), `VanillaTreeFeatureOracle`
  (8 → 11).
- 1 new file: `VanillaSteppedInterpolationOracle` (16 errors), replacing
  the passing `VanillaInterpolationOracle` (the driver
  `test_interpolation_parity.py` was likewise replaced by
  `test_stepped_interpolation_parity.py`, plus a new
  `test_inline_feature_parse_parity.py` — net +1 driver, 648 vs 647).

The re-port signature is the 26.3 unconfigured-Feature form:
`Feature.DIRECT_CODEC` (absent from 26.1.1 — only
`ConfiguredFeature.DIRECT_CODEC` exists, javap-verified), no-arg
`new Feature()` (26.1.1's `Feature` takes only `Codec<FC>`), the 4-arg
`place(WorldGenLevel, ChunkGenerator, RandomSource, BlockPos)` (26.1.1's
abstract `place` takes `FeaturePlaceContext`), and `FeaturePlacer`-based
placement. The non-Feature re-ports use 26.3-only APIs throughout:
the `BrewingFuel` component (0 jar entries),
`net.minecraft.core.component.BlockTransformer`, the
`BrewingInput`/`BrewingRecipe`/`PotionIngredient`/`RecipePropertySet`
crafting APIs, `PositionAndRotation` (0 jar entries, in
`VanillaSteppedInterpolationOracle`), and `LivingEntity.SwingState`
(0 jar entries, in `VanillaFirstPersonOracle`, which also uses the 26.3
boolean `swing(InteractionHand, SwingAnimation, boolean)` — 26.1.1's
`LivingEntity` has only `swing(InteractionHand)` and
`swing(InteractionHand, boolean)`, javap-verified). The 26.1-adapted
forms are the repair template for all 17 — e.g. 26.1's
`VanillaBrewingFuelOracle` documents "26.1 has no `brewing_fuel`
component: fuel is any item in the tag, 20 uses, and the brew time is a
flat 400 ticks", and 26.1's `VanillaBlockTransformerOracle` documents
"26.1 has no block_transformer registry". One re-ported file still
compiles: `VanillaSimpleBlockOracle` (its `CONFIGURED_FEATURE` →
`FEATURE` registry change passes on all cps).

26 oracles are never referenced by any driver; 8 of them fail on all
three classpaths (`ComponentWireCapture`, `EntityTranscript`,
`OfflineLanHost`, `RideTranscript`, `VanillaLootFillOracle`,
`VanillaMover`, `VanillaSceneMotionOracle`,
`VanillaStructurePlacementOracle`) — unlike 26.1, where all 18
never-referenced oracles passed. Three of the eight are referenced by
non-driver scripts (`OfflineLanHost` in `tools/vanilla_client.py`,
`RideTranscript` in `tools/cart_portal_player_e2e.py`, `VanillaMover`
in `tools/anticheat_e2e.py`); `EntityTranscript` appears only in a
comment in `test_server_entities_parity.py`. The other four have no
reference anywhere in `tools/`.

## Driver impact

Driver→oracle map built by parsing all 648 drivers (`.java` basenames +
oracle-name string literals passed to `oracle_harness`). Each driver's
oracle set was checked against the classpath its own path uses
(oracle_harness → both, direct `jars['server']` → server, client jar →
client, both jars → both). 20 drivers reference an oracle without a
detectable classpath (they compile via a shared `vanilla.compile(...)`
helper); those were judged against the `both` superset — 14 blocked, 6
runnable.

| Driver state | Drivers |
|---|---|
| **Blocked: ≥1 oracle fails to compile under its path's cp** | **231** |
| Oracles compile under their path's cp (can yield a measurement when run) | 163 |
| No oracle reference (server-e2e, data-only, wrappers) — not oracle-blocked | 254 |
| **Total** | **648** |

- 417 drivers have no oracle blocker (163 + 254), vs 426 for 26.1
  (173 + 253) and 509 for 26.2 (248 + 261) and 321 for 1.21.7 pre-repair
  (67 + 254). The 254 no-oracle drivers were never oracle-blocked, but
  that does not mean they pass: they run real servers / the game / data
  diffs and have their own failure modes.
- Blocked by classpath: server 111, both 77, client 29, no-flag
  (judged on both) 14.
- The highest-leverage repairs are shared helpers: `VanillaFaceSupportOracle`
  blocks 19 drivers (1 error — cheapest), `VanillaAnchorOracle` 9
  (10 errors, densityfunction — G3), `VanillaRegistryCollisionScanOracle`
  8 (1 error), `VanillaProtocolClient` 8 (34 errors),
  `ParityFont` 8 (1 error), `LocatorBarClient` 8 (13 errors),
  `VanillaSuffocationOracle` 7 (2 errors), `VanillaDispenserOracle` 6
  (15 errors, EntityTypes — G4), `VanillaBlockUpdateOracle` 6
  (32 errors, plus 5 cascades), `VanillaMaterialConditionOracle` 5
  (76 errors, densityfunction — G3), `VanillaCollisionContextOracle` 5
  (1 error). Fixing the top few one-error helpers unblocks ~50
  driver-oracle pairs (overlaps exist).
- Compiling ≠ running: the 163 "runnable" drivers are runnable by
  javac only. No driver was run in this batch (scope decision above),
  so the runtime-failure layer the 1.21.7 sample found (a compiling
  oracle throwing NPE at runtime) is unmeasured for 26.1.1. Every
  repaired oracle needs an end-to-end driver run to confirm it yields
  a measurement.

## The 9-harness check (1.21.7's table, re-measured for 26.1.1)

Of the 9 harnesses the 1.21.7 plan (§2) records as dying on oracle javac
failures in CI run 44441686979, **6 pass on at least one classpath for
26.1.1 at measurement time; 3 fail on all three and need repair before
their drivers can run** (same 6/3 split as 26.1; for 26.2 all 9 passed):

| Harness | Oracle | 26.1.1 status |
|---|---|---|
| chunk_generation | VanillaChunkGenerationOracle | **PASS** (all cps) |
| command_exec | VanillaCommandExecOracle | **PASS** (all cps) |
| component_patch | VanillaComponentPatchOracle | **PASS** (all cps) |
| datafix | VanillaDataFixOracle | **PASS** (all cps) |
| first_person | VanillaFirstPersonOracle | **FAIL** (all cps; client 20 errors, up from 11 in 26.1: 26.3-only `FirstPersonHandsAndItems`, `LivingEntity.SwingState` (7 errors at lines 242–248 plus 278/358/378/383/386), LocalPlayer ctor, 3 no-override, `swing(InteractionHand,SwingAnimation,boolean)`, `itemUsed(InteractionHand)`) |
| food | VanillaFoodOracle | **PASS** (all cps) |
| item_model | VanillaItemModelOracle | **FAIL** (all cps; client 10 errors: EntityTypes, `ResourceManager.Selector`, `ItemQuads`, `shadeDirectionOverride()`, `createWorldLookup()`, `MaterialBaker` interface, `asMutable()`) |
| options | VanillaOptionsOracle | **PASS** (client + both; server fails on client packages — expected) |
| resource_pack | VanillaPackResourcesOracle | **FAIL** (all cps; client 1 error: `PackResources.forEach` absent in 26.1.1) |

For 1.21.7, 8 of these passed only AFTER Batch 0 repairs and `options`
still failed. For 26.1.1 the `command_parse`/`command_exec`/`food`/
`component_patch` drivers — the ones that yielded the 1.21.7 sweep's
first real parity measurements — are oracle-clean from the start, but
`first_person`, `item_model` and `resource_pack` are not: the
`resource_pack` failure is a single missing method (cheapest of the
three), while `first_person` and `item_model` need per-API translation
against 26.1.1's client classes. `first_person` got worse vs 26.1
(11 → 20 errors) because its oracle was re-ported (G10).

## Driver defects found (Batch 0 scope)

1. **`client_libraries()` omits jsr305 — javac can crash (latent).**
   `versions/26.1.1/tools/test_model_baking_parity.py:client_libraries()`
   (lines 58–68) builds the client classpath from the manifest only;
   it does not append `tools/jsr305-3.0.2.jar`. No `tools/*.py` file in
   the 26.1.1 tree mentions jsr305 at all. The 1.21.7 Batch 0 fix
   for the identical defect (`d7a14e91`) is NOT in the 26.1.1 tree
   (same gap as 26.1 and 26.2). Without jsr305, javac crashes
   (`CompletionFailure: class file for javax.annotation.Nullable not
   found`, rc=4) when compiling an oracle that completes a
   `Nullable`-referencing symbol — a latent crash across the
   oracle_harness drivers, not a deterministic one. The measurement
   adds jsr305 itself, so this did not affect the numbers above. Fix
   direction (follow-up, not this batch): append jsr305 in
   `client_libraries()` — one point, fixes all users.
2. **`.local/minecraft/26.1.1/` did not exist in any worktree** (same gap
   as the 1.21.7 pilot's blocker, 26.2's and 26.1's). Resolved — see §6.
   No driver could run VM-local before this.

## `.local/minecraft/26.1.1/` setup (recipe for later batches)

`verify_artifacts()` needs `manifest.json` (pinned sha1), `jars/client.jar`
+ `jars/server.jar` (manifest sha1s or `remapped.json` digests), the inner
`jars/server-game.jar` (bundle checksum or `remapped.json`), and
`libraries/server/` + `libraries/client/` (sha-verified). No worktree had
it; unlike 26.2 (which reused a complete prepared directory from the port
work), 26.1.1 was built fresh from the research jars with `lab.py prepare`:

```sh
# 1. Launcher-style source dir: the 26.1.1 client jar + version.json, plus the
#    107 client libraries from version.json, each sha1-checked (public Mojang
#    URLs, no account needed). Script: sweep-26.1.1-batches/b0/fetch-mc-source.py
#    (writes ~/workspace/mcir-work/sweep-26.1.1/mc-source/; 80.7 MB of libraries,
#    pre-seeded from the 26.1 run and sha1-checked, so identical ones are skipped).
cd ~/workspace/mcir-work/sweep-26.1.1/versions/26.1.1
python3 tools/lab.py prepare --minecraft-dir ~/workspace/mcir-work/sweep-26.1.1/mc-source --server-jar ~/workspace/mcir-work/26.1.x/versions/26.1.1/jars/server.jar
python3 -c "import sys; sys.path.insert(0,'tools'); import test_rust_parity as t; t.verify_artifacts(); print('GATE OK')"
```

`lab.py prepare` imports the client + server, unpacks `server-game.jar`
+ the server libraries FROM the bundler jar, downloads vineflower, and
writes `prepared.json` + `reports/` — no remap stage runs, because the
26.1.1 version manifest's `downloads` has only client+server (no mappings).
Result (226 MB): named client jar (10,682 classes) + named inner server
jar (7,351 classes), 39 server + 60 client libraries, `manifest.json`
sha1 `377031a9…` (client) / `49c81957…` (server, the pinned ones in
research-lock.json). Verified: the gate passes. Never commit `.local`.

Prepared horizon-client data did not exist anywhere on the VM for 26.1.1,
so `horizon-prepare` was built (`taskset -c 0`, `-j 1`, 3m27s) and run
(`./target/release/horizon-prepare --minecraft-dir .local/minecraft/26.1.1`).
The FIRST run failed at 2m20s in "Exporting movement and attributes"
(`Exporter VanillaBlockLandingOracle failed:
"java.lang.reflect.InvocationTargetException"`, with an empty
`.local/horizon-client-motion/export.log`) — a transient: it ran
concurrently with the javac big-compiles, and the oracle itself compiles
cleanly on all three classpaths. The retry, on a quieter VM, completed in
full in 93.1 s, exit 0 (slowest steps: the datagen run, 44.5 s, command
registries, 35.5 s, movement and attributes, 37.4 s, creative inventory,
25.7 s, block behavior, 13.8 s). If a later batch hits the same
`InvocationTargetException` with an empty export log, retry before
assuming a real exporter defect.
The run completed in full, with these outputs present in
`.local/horizon-client/26.1.1/` (190 MB): registries, items, block-tags,
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
jar's file (`98deafdb5e90e4438db30853494e1aeb` — the same md5 as 26.1's)
— it arrives via the client-jar asset copy, not the skipped
launcher-index import. Only `sounds.json` is genuinely absent. Batches
that need `sounds.json` must note the absence rather than assume silence
is a port defect; batches must NOT claim lang files are missing.
`horizon-prepare --check` lists only the five expected asset-store gaps
(packs, panorama, fonts, languages, sounds).

## What the numbers mean

- Batch 0's repair load is **202 files**, not 192 — and the profile is
  heavier than 26.1's (median 3 errors, same; 55 one-error files vs 51;
  p90 14 vs 13; max still 251). The +10 vs 26.1 is NOT vanilla drift:
  the 26.1 → 26.1.1 jar delta is version stamps plus one internal
  `PlayerEntry` change, while the 26.1.1 tree re-ported 19 oracle files
  from the 26.3 form (G10, 17 files fail new or worse). The 26.1-adapted
  oracle forms are the repair template for all 17 — the fix is a
  re-adaptation, not new API research. Two mechanical groups (G2
  LocalPlayer ctor, 20 files with 7 single-cause; G1 fuelValues, 4
  files) rescue files with two signature translations. The
  densityfunction (G3, 17 files) and renderpearl/material.rule (G5,
  G7) groups are the genuinely hard ones: 26.3-only APIs with no
  26.1.1 equivalent to translate to import-by-import — those oracles
  need rewrites against 26.1.1's actual APIs (or, for StructureDensity,
  stay unrepaired by design).
- Sequencing for batches 3–12: the 163 oracle-clean drivers can run
  once their probes build; the 231 blocked drivers wait on Batch 0
  repairs, shared helpers first (`VanillaFaceSupportOracle` unblocks
  19 drivers with a 1-error repair; `ParityFont` and
  `VanillaRegistryCollisionScanOracle` 8 each, also 1 error). The 254
  no-oracle drivers were never oracle-blocked. In practice this sweep's
  batches ran statically (javap + source + data) and recorded driver
  runs as follow-up needs — the VM's 6 GB budget under multi-agent load
  makes probe builds (client_probe OOMs single-threaded) a CI-or-quiet-
  window job, not a sweep-batch job.
- The 26.1.1-vs-26.3 API deltas this measurement exposes are themselves
  sweep input: the `EntityTypes`/`BlockEntityTypes` holder move (26.1.1
  still registers on `EntityType`/`BlockEntityType`, and keeps the bed
  BE type), `Level.fuelValues()` (abstract in 26.1.1), the
  `LocalPlayer`/`Gui`/`ServerChunkCache`/`ChunkMap`/`MinecraftServer`/
  `Feature` constructor forms, the absence of the `densityfunction`,
  `renderpearl` and `material.rule` APIs, the advancements
  `predicates`/`triggers` packages (26.1.1 has only `criterion`), the
  `RandomState.create` overloads, the `ConfiguredFeature` form (26.1.1
  has no `Feature.DIRECT_CODEC`, no-arg `Feature()` or 4-arg `place`),
  the absence of the `BrewingFuel`/`BlockTransformer` components and of
  `LivingEntity.SwingState`, and the `MaterialBaker` /
  registry-lookup abstract methods. Family batches should check the
  port's Rust against 26.1.1's forms of these, not 26.3's.

## The client_probe adaptation (H01)

**Verdict: H01 does NOT reproduce in 26.1.1 — `client_probe` compiles
with the two probe lines present. No edit was made; the probe file is
unmodified.**

In 26.1, `client_probe` failed to compile with E0560:
`game_loop_probe.rs` initialised `FrameContext` fields `focused` and
`exclusive_fullscreen` that 26.1's `FrameContext` does not have, so the
two lines had to be deleted. 26.1.1's `FrameContext`
(`crates/horizon-client-input/src/window.rs:56-66`) keeps the 26.3 form,
with `pub focused: bool` and `pub exclusive_fullscreen: bool`, so the
same two lines are valid initialisers here — the 26.1.1 port re-ported
this code from the 26.3 form, as it did the 19 oracle files (see
Headline). The PORT-NOTES-26.1.1.md "now-building client_probe" claim is
true.

Check (continuation agent, 2026-11-24; the previous Batch 0 agent's
check was cut off mid-build by a runtime restart drain —
`evidence/client-probe-check.log` ends at "Checking
horizon-client-render" with no verdict):

```
cd versions/26.1.1
RUSTUP_HOME=~/workspace/tools/rustup CARGO_HOME=~/workspace/tools/cargo \
  PATH=~/workspace/tools/jdk-25/bin:~/workspace/tools/cargo/bin:$PATH \
  taskset -c 0 cargo check --locked -j 1 -p horizon-client-lib --example client_probe
```

Note the package: the `client_probe` target is declared in
`crates/horizon-client-lib/Cargo.toml:200-202` (path
`../horizon-client/examples/probe/main.rs`), not in
`horizon-client/Cargo.toml`; this is the selection the harnesses use
(`tools/oracle_harness.py:38` runs `cargo build --locked -p
horizon-client-lib --example client_probe`).

- Before-state: `crates/horizon-client/examples/probe/game_loop_probe.rs:159-160`
  still read `focused: b(&c["focused"]),` and
  `exclusive_fullscreen: false,` — unmodified, worktree clean.
- First re-run (`evidence/client-probe-check2.log`): EXIT=0 in 4m 40s,
  no errors, resuming from the warm cargo cache. The log's last unit is
  "Checking horizon-client-lib": cargo displays an example unit's status
  line under its package name, so no "Checking client_probe" line ever
  appears — the example compile is evidenced by its rewritten artifacts,
  not by a log line.
- Forced recheck (`evidence/client-probe-check3.log`): `game_loop_probe.rs`
  was touched (mtime only, no content change) to dirty the example unit
  and the check re-run — EXIT=0 in 8.75s, no errors. The example's
  `target/debug/examples/libclient_probe-*.rmeta` and its fingerprint
  (`.fingerprint/horizon-client-lib-*/example-client_probe*`) were
  rewritten during this run (11:57:58-11:58:02 UTC), proving rustc
  compiled the example from the current sources with both lines present.
  A `-v` control run (`evidence/client-probe-check4.log`) then showed all
  306 units fresh, and a small-example control
  (`evidence/client-probe-check5-small-example.log`, `--example
  resource_memory`) confirmed `--example` selection works and that the
  package-named status line is the example unit's.
- After-state: compiles. No lines deleted, no edit, no slice.json item —
  H01 is closed for 26.1.1 and blocks no harness here. (`cargo build`
  of the probe binary itself, with codegen, was not run: the check
  verdict is what H01 asks, and a full probe build is a CI-or-quiet-window
  job under this VM's memory budget.)

## Per-oracle table

`docs/sweep/26.1.1/batch-0-oracles.json` — all 391 files: pass/fail per
classpath, first-error class + message + error count for failures. Raw
big-compile logs (4,286 server-cp / 1,391 client-cp errors) are in the
measuring workdir's `.local/measure-oracles/` and copied to the batch's
evidence dir (regenerable with `docs/sweep/measure-oracles.py`).
