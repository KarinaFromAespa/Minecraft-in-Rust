# Batch 0 measurement: oracle compilability against the 26.2 jars

Date: 2026-11-24. Branch: `parity-sweep-26.2`. Measured before any repair.
Companion data: `docs/sweep/26.2/batch-0-oracles.json` (per-oracle table, all
391 files) and `docs/sweep/measure-oracles.py` (the measurement script,
rerunnable).

**Scope decision (coordinator, per the sweep brief): Batch 0 for 26.2 is
measurement + triage ONLY — no oracle repairs.** The brief limits the sweep
to checks + triage and forbids touching `versions/26.2/` code; the oracle
`.java` files live in `versions/26.2/mods/horizon/`. The 1.21.7 Batch 0 did
repair oracles (135/391 pass after repair, up from 94); for 26.2 the repair
list below is a follow-up decision for the parent. Batches use static checks
(javap + source + data) for drivers whose oracles don't compile, as 1.21.7
Batch 2 did.

## Headline

**262 of 391 oracles (67%) compile against the 26.2 jars on at least one
classpath; 129 (33%) fail on all three.** The failure rate is less than half
of 1.21.7's pre-repair 76% — expected, since the oracles were written
against 26.3 and 26.2 is one version back. The remaining breakage is
concentrated in a few systematic 26.2-vs-26.3 API deltas plus a long tail of
1–3-error files: median 2 errors per failing file, 114 of 129 with ≤10
errors, 46 with exactly 1 error.

| Classpath (driver configuration) | Pass | Fail | Failure rate |
|---|---|---|---|
| server — server-game.jar + server libraries + jsr305 (direct drivers) | 217 | 174 | **44.5%** |
| client — client.jar + client libraries + jsr305 (oracle_harness drivers) | 262 | 129 | **33.0%** |
| both — server + client jars + both library sets (superset) | 262 | 129 | 33.0% |
| **fail on all three classpaths (genuinely broken vs 26.2)** | — | **129** | **33.0%** |

Notes:

- The `both` set is identical to the `client` set, and every server-pass file
  also passes on client cp: the client jar contains the shared/server classes
  these oracles use, so client cp is the superset that matters. The 45 files
  that pass on client but fail on server are client-only oracles
  (`net.minecraft.client.*`, blaze3d, renderpearl) — expected, not breakage.
- The client-cp measurement adds jsr305 to the classpath. The harness itself
  does not (see §5, driver defect 1): without jsr305, javac can crash
  (`CompletionFailure: class file for javax.annotation.Nullable not found`,
  rc=4) on some oracles. The measurement isolates oracle breakage from that
  harness bug; the harness bug is recorded separately and NOT fixed in this
  batch (scope decision above).
- Error totals: server cp 3,913 errors in 171 files; client cp 894 errors in
  122 files (both cp identical). For comparison, 1.21.7 pre-repair was 6,199
  server / 3,916 client errors.

## Method

Same fixpoint big-compile as the 1.21.7 measurement (`docs/sweep/measure-
oracles.py`, unchanged): per classpath, compile all 391 files together,
then iterate the clean set (sourcepath=False) until the compile passes.
Fixpoint iterations: server 1 round (3 cascade drops), client 1 round (7
cascade drops), both 1 round (7 cascade drops). Total runtime ~10.7 min
under load (6 batch agents + a port agent sharing the VM).

Jars: the named 26.2 jars from the prepared `.local/minecraft/26.2/`
(see §6); `javac --release 21 -proc:none`, JDK 25.0.5. The 26.2
server-game.jar IS named (Mojang-mapped classes and methods) — the
"no named jar exists for any 26.x version" note in PARITY-SWEEP.md §7
(Nov 22) is outdated for 26.2.

## Failure classes (first error per file)

Server cp (174 failures):

| Class | Files | Meaning |
|---|---|---|
| missing-package | 94 | imports a package that does not exist on the server cp (mostly client-only packages — 45 of these pass on client cp) |
| missing-symbol | 46 | references a class/method/field 26.2 does not have (renamed/moved API) |
| no-override | 11 | `@Override` of a method 26.2's supertype lacks |
| other | 7 | assorted (static-context, access, …) |
| missing-override | 5 | abstract-method gap |
| incompatible-types | 4 | 26.3 vs 26.2 signature change |
| constructor-mismatch | 4 | calls a constructor with a 26.3 signature |
| cascade | 3 | file itself is clean; fails only because a helper it pulls is broken |

Client cp (129 failures): missing-symbol 51, constructor-mismatch 21,
missing-package 19, no-override 13, missing-override 7, other 7,
cascade 7, incompatible-types 4. The client cp sheds the client-package
failures and exposes the real drift: missing symbols and constructor
mismatches dominate.

### The systematic groups (repair list for the follow-up decision)

**G1 — `Level.fuelValues()` abstract-method gap (4 oracles, 1 error each).**
26.2's `Level` declares `public abstract FuelValues fuelValues()`
(javap-verified: `net.minecraft.world.level.block.entity.FuelValues`);
26.3's does not (or not abstractly), so the oracles' fake `Level`
subclasses don't implement it: `VanillaActiveEffectTickOracle`,
`VanillaAttributeUpdatesOracle`, `VanillaBlockEntityAnimOracle`,
`VanillaLivingLandingOracle`. Mechanical repair: add a `fuelValues()`
override returning a default to each fake level. (Three more
missing-override files have different abstract gaps:
`VanillaDecorationRegionOracle` / `VanillaFeatureCatalogOracle` —
anonymous classes missing a `lookup…` override; `VanillaSectionMeshOracle`
— `Materials` missing `bake(Material)` in `MaterialBaker`.)

**G2 — `LocalPlayer` constructor (17 oracles, mostly 1 error each).**
26.2's `LocalPlayer` constructor takes different arguments than 26.3's;
every oracle constructing a `LocalPlayer` with the 26.3 form fails:
`VanillaAllayInteractionStateOracle`, `VanillaAmbientOracle`,
`VanillaAutoJumpOracle`, `VanillaBoatRiderOracle`, `VanillaCameraOracle`,
`VanillaFireStarterSoundOracle`, `VanillaGameLoopOracle`,
`VanillaHeldDropOracle`, `VanillaInteractionInputOracle`,
`VanillaLocalInputOracle`, `VanillaMarkerParticlesOracle`,
`VanillaPlayerSyncOracle`, `VanillaRemoteAttackOracle`,
`VanillaSheepDyeOracle`, `VanillaSoundEngineOracle`,
`VanillaSprintInputOracle`, `VanillaTeamRulesOracle`. One signature
translation, verified once against the 26.2 jar, rescues all 17.

**G3 — `densityfunction` package absent from 26.2 (7 oracles by first
error; 3 more by symbol).** `net.minecraft.world.level.levelgen.
densityfunction` has ZERO entries in the 26.2 server-game.jar (verified by
jar listing) — the `SamplerContext`/`DensityVolume` API is a 26.3
addition, not a 26.2 one. First-error files: `VanillaAnchorOracle` (10
errors), `VanillaBiomeSourceCodecOracle`, `VanillaBiomeSourceOracle`,
`VanillaFloatLogOracle`, `VanillaMaterialExtensionOracle`,
`VanillaStructureDensityOracle`, `VanillaWorldgenOracle` (42 errors).
`VanillaDensityOracle` (247 errors, the largest single failure) and
`VanillaMaterialConditionOracle` (76 errors) fail on the same API's
symbols. `VanillaStructureDensityOracle` is unrepairable by design (it
directly exercises the 26.3-only API; density is verified via
bytecode/chunk comparison instead — same ruling as 1.21.7). The others
need per-file translation to 26.2's actual density API, not import fixes.

**G4 — `renderpearl` package absent from 26.2 (7 oracles).**
`com.mojang.renderpearl.api.pipeline` (and `.buffers`) do not exist in
26.2 — the render-pipeline API is 26.3-only. Files: `VanillaHudOracle`,
`VanillaLoomScreenOracle`, `VanillaPackScreenOracle`,
`VanillaRecipeBookScreenOracle`, `VanillaSpriteAnimationOracle`,
`VanillaTranslucencyOracle`, `VanillaWorldBorderRenderOracle` (28
errors). These need rewrites against 26.2's screen/render APIs, not
import fixes.

**G5 — Other constructor mismatches (4 oracles).**
`VanillaFallingBlockOracle` (`ChunkMap` ctor),
`VanillaInventoryUseStartOracle` + `VanillaItemUseStartOracle`
(`ServerChunkCache` ctor), `VanillaDecorationOracle` (`Feature` ctor,
4 errors). Each is a one-signature translation against the 26.2 jar.

**G6 — Other missing packages (5 oracles).**
`net.minecraft.world.level.levelgen.material.rule` does not exist in
26.2 (the 26.3 material-system package): `VanillaBiomeGenerationOracle`
(12 errors), `VanillaMaterialStageOracle` (26), `VanillaTerrainOracle`
(13). `VanillaTabListOracle` (29 errors) imports
`com.mojang.authlib.services`, absent from the 26.2 classpath; and
`VanillaTickScript` (17 errors) imports `org.lwjgl.sdl`, absent likewise.
Each needs per-file investigation against the 26.2 jar/libraries.

**G7 — The long tail (49 missing-symbol + 13 no-override + 4
incompatible-types + 7 other, after G3's two symbol files are counted
there).** Per-API translation, verified against the 26.2 jar per file.
46 fail-all files have exactly 1 error; 71 have ≤3. The cheapest
high-leverage single repair is `ParityFont` — 1 error on the client cp
(`43: cannot find symbol`), blocking 8 drivers. `OfflineLanHost`
(`publishServer(MultiplayerScope, boolean, int)` — no suitable method)
is in this group.

**G8 — Cascades (7 oracles).** Clean files failing only via a broken
helper: `VanillaBoatAuthorityOracle`, `VanillaBreakingModelOracle`,
`VanillaChatScreenOracle`, `VanillaHeightProviderOracle`,
`VanillaPlaceRecipeOracle`, `VanillaRecipeBookOracle`,
`VanillaSubtitleOracle`. These pass for free once their helpers are
repaired — do not schedule them separately.

25 oracles are never referenced by any driver; 6 of them fail on all
cps — lowest repair priority (dead code unless a batch adopts them).

## Driver impact

Driver→oracle map built by parsing all 656 drivers (`.java` basenames +
oracle-name string literals passed to `oracle_harness`). Each driver's
oracle set was checked against the classpath its own path uses
(oracle_harness → both, direct `jars['server']` → server, client jar →
client, both jars → both).

| Driver state | Drivers |
|---|---|
| **Blocked: ≥1 oracle fails to compile under its path's cp** | **147** |
| Oracles compile under their path's cp (can yield a measurement when run) | 248 |
| No oracle reference (server-e2e, data-only, wrappers) — not oracle-blocked | 261 |
| **Total** | **656** |

- 509 drivers have no oracle blocker (248 + 261), vs 321 for 1.21.7
  pre-repair (67 + 254). The 261 no-oracle drivers were never
  oracle-blocked, but that does not mean they pass: they run real
  servers / the game / data diffs and have their own failure modes.
- The highest-leverage repairs are shared helpers: `ParityFont` blocks
  8 drivers (1 error — cheapest), `VanillaProtocolClient` 8 (32
  errors), `LocatorBarClient` 8 (10 errors), `VanillaAnchorOracle` 7
  (10 errors, densityfunction — G3), `VanillaDispenserOracle` 6 (1
  error, no-override), `VanillaMaterialConditionOracle` 5 (76 errors,
  densityfunction symbols — G3), `KnockbackClient` 4, `VanillaMenuOracle`
  4 (1 error). Fixing the top 8 unblocks ~50 driver-oracle pairs
  (overlaps exist).
- Compiling ≠ running: the 248 "runnable" drivers are runnable by
  javac only. No driver was run in this batch (scope decision above),
  so the runtime-failure layer the 1.21.7 sample found (a compiling
  oracle throwing NPE at runtime) is unmeasured for 26.2. Every
  repaired oracle needs an end-to-end driver run to confirm it yields
  a measurement.

## The 9-harness check (1.21.7's table, re-measured for 26.2)

Of the 9 harnesses the 1.21.7 plan (§2) records as dying on oracle javac
failures in CI run 44441686979, **all 9 pass on at least one classpath
for 26.2 at measurement time — no repair needed**:

| Harness | Oracle | 26.2 status |
|---|---|---|
| chunk_generation | VanillaChunkGenerationOracle | **PASS** (all cps) |
| command_exec | VanillaCommandExecOracle | **PASS** (all cps) |
| component_patch | VanillaComponentPatchOracle | **PASS** (all cps) |
| datafix | VanillaDataFixOracle | **PASS** (all cps) |
| first_person | VanillaFirstPersonOracle | **PASS** (client + both; server fails on client packages — expected) |
| food | VanillaFoodOracle | **PASS** (all cps) |
| item_model | VanillaItemModelOracle | **PASS** (client + both; server fails on client packages — expected) |
| options | VanillaOptionsOracle | **PASS** (client + both; server fails on client packages — expected) |
| resource_pack | VanillaPackResourcesOracle | **PASS** (client + both; server fails on client packages — expected) |

For 1.21.7, 8 of these passed only AFTER Batch 0 repairs and `options`
still failed. For 26.2 the `command_parse`/`command_exec`/`food`/
`component_patch` drivers — the ones that yielded the 1.21.7 sweep's
first real parity measurements — are oracle-clean from the start.

## Driver defects found (Batch 0 scope)

1. **`client_libraries()` omits jsr305 — javac can crash (latent).**
   `versions/26.2/tools/test_model_baking_parity.py:client_libraries()`
   (lines 58–68) builds the client classpath from the manifest only;
   it does not append `tools/jsr305-3.0.2.jar`. The 1.21.7 Batch 0 fix
   for the identical defect (`d7a14e91`) is NOT in the 26.2 tree.
   Without jsr305, javac crashes (`CompletionFailure: class file for
   javax.annotation.Nullable not found`, rc=4) when compiling an oracle
   that completes a `Nullable`-referencing symbol — a latent crash
   across the oracle_harness drivers, not a deterministic one. The
   measurement adds jsr305 itself, so this did not affect the numbers
   above. Fix direction (follow-up, not this batch): append jsr305 in
   `client_libraries()` — one point, fixes all users.
2. **`.local/minecraft/26.2/` did not exist in any worktree** (same gap
   as the 1.21.7 pilot's blocker). Resolved — see §6. No driver could
   run VM-local before this.

## `.local/minecraft/26.2/` setup (recipe for later batches)

`verify_artifacts()` needs `manifest.json` (pinned sha1), `jars/client.jar`
+ `jars/server.jar` (manifest sha1s or `remapped.json` digests), the inner
`jars/server-game.jar` (bundle checksum or `remapped.json`), and
`libraries/server/` + `libraries/client/` (sha-verified). No worktree had
it; instead, reuse the complete prepared directory the 26.2 port work
already produced:

```sh
cd versions/26.2
mkdir -p .local/minecraft
cp --reflink=auto -r ~/workspace/mcir-work/26.2/versions/26.2/game .local/minecraft/26.2
python3 -c "import sys; sys.path.insert(0,'tools'); import test_rust_parity as t; t.verify_artifacts(); print('GATE OK')"
```

Source layout (242 MB, reflinked so ~0 new disk): `game/` is a prepared
`.local/minecraft/26.2/` — manifest sha1 `f28ad96e…` (the pinned one in
research-lock.json), named client + named inner server jars, both library
sets (39 server + 67 client jars), `prepared.json`, `reports/`. Verified:
the gate passes. Never commit `.local`.

Prepared horizon-client data did not exist anywhere on the VM for 26.2,
so `horizon-prepare` was built (`-j 1`, single core, 9m58s) and run
(`./target/release/horizon-prepare --minecraft-dir .local/minecraft/26.2`;
the two slow steps under load were the datagen run, 237 s, and the
creative-inventory export, 231 s, then command registries, 294 s).
At Batch 0 commit time the run was still in its final Java exporter
phase, with these outputs already present in
`.local/horizon-client/26.2/`: registries, items, block-tags, facts,
mob-data, entity-models, model-states, creative-inventory,
command-registries, commands, data/, worldgen/, biome-parameters/,
assets/. (Update when the run completes: block-behaviours,
gameplay-facts, entity-types, attributes, horizon-resources are the
remaining exports.) Asset-dependent exports skipped as expected — panorama,
fonts, packs, languages need the official launcher's asset index
(`assets/indexes/32.json`), which the VM lacks; sounds need the separate
`horizon-prepare sounds` step against a launcher asset store (~380 MB)
and were not run. Batches that need `sounds.json` or lang files must
note the absence rather than assume silence/emptiness is a port defect.

## What the numbers mean

- Batch 0's repair load is **129 files**, not 297 — and the profile is
  far cheaper than 1.21.7's (median 2 errors vs 9; 46 one-error files).
  Two mechanical groups (G1 fuelValues, 4 files; G2 LocalPlayer ctor,
  17 files) rescue 21 files with two signature translations. The
  densityfunction (G3) and renderpearl (G4) groups are the genuinely
  hard ones: 26.3-only APIs with no 26.2 equivalent to translate to
  import-by-import — those oracles need rewrites against 26.2's actual
  APIs (or, for StructureDensity, stay unrepaired by design).
- Sequencing for batches 3–12: the 248 oracle-clean drivers can run
  once their probes build; the 147 blocked drivers wait on Batch 0
  repairs, shared helpers first (`ParityFont` unblocks 8 drivers with
  a 1-error repair). The 261 no-oracle drivers were never
  oracle-blocked. In practice this sweep's batches ran statically
  (javap + source + data) and recorded driver runs as follow-up needs
  — the VM's 6 GB budget under multi-agent load makes probe builds
  (client_probe OOMs single-threaded) a CI-or-quiet-window job, not a
  sweep-batch job.
- The 26.2-vs-26.3 API deltas this measurement exposes are themselves
  sweep input: `Level.fuelValues()` (abstract in 26.2), the
  `LocalPlayer`/`ServerChunkCache`/`ChunkMap` constructor forms, the
  absence of the `densityfunction` and `renderpearl` APIs, and the
  `MaterialBaker.bake` / registry-lookup abstract methods. Family
  batches should check the port's Rust against 26.2's forms of these,
  not 26.3's.

## Per-oracle table

`docs/sweep/26.2/batch-0-oracles.json` — all 391 files: pass/fail per
classpath, first-error class + message + error count for failures. Raw
big-compile logs (3,913 server-cp / 894 client-cp errors) are in the
measuring workdir's `.local/measure-oracles/` (not committed —
regenerable with `docs/sweep/measure-oracles.py`).
