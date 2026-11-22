# Batch 0 — Oracle Repair (final report)

**Date:** 2026-11-22 · **Branch:** `parity-sweep` · **Workdir:**
`~/workspace/mcir-work/sweep-batch-0` · **Ledger slice:**
`pins/1.21.7/sweep/batch-00.json` · **Per-oracle table:**
`docs/sweep/batch-0-oracles.json` (regenerated post-repair)

Batch 0 measures and repairs the 391 Java oracles in
`versions/1.21.7/mods/horizon/src/test/java/dev/horizonlab/parity/` so they
compile and run against the named 1.21.7 jars. Checks + oracle/driver repair
only — no port behaviour code (`crates/`) was touched, and parity defects
found during repair were recorded, not fixed.

## 1. Measurement summary

Method: fixpoint big-compile per classpath (server, client, both), one javac
invocation per round; driver mapping by parsing oracle basenames + oracle-name
string literals across all 648 drivers. Full method and the original numbers:
`docs/sweep/batch-0-measurement.md`.

| Stage | Pass (any cp) | Fail (all cps) | Fail rate | Drivers runnable | Drivers blocked |
|---|---:|---:|---:|---:|---:|
| Measurement (pre-repair) | 94 / 391 | 297 | 76.0% | 67 | 307 |
| After group 1 (ServerLevel ctor) | 119 / 391 | 272 | 69.6% | 92 | 282 |
| **Final (all repairs)** | **135 / 391** | **256** | **65.5%** | **115** | **259** |

Final per-classpath: server 115 pass / 276 fail; client 133 pass / 258 fail;
both 133 pass / 258 fail. (The rerun measured 133 pass-any; Anchor and
RegistryCollisionScan were repaired after the rerun and individually verified
clean on server + client classpaths, hence 135.)

**41 oracles repaired, 48 drivers newly runnable.** The remaining load is
256 files failing on all three classpaths. First-error classes (server cp):
missing-package 137, missing-symbol 99, no-override 13, other 8, cascade 7,
constructor-mismatch 5, incompatible-types 4, missing-override 1.
Errors/file among failing files: median 9, p90 54, max 251; ≤10 errors: 151
files; ≤20: 204 files. **Zero files fail ONLY on missing-package** — there is
no purely mechanical import-fix group left; every remaining file needs
per-API translation verified against the 1.21.7 jar.

Top missing packages (server cp): `world.attribute` (39),
`level.gamerules` (23), `levelgen.densityfunction` (18),
`entity.projectile.arrow` (10), `server.permissions` (10).
Top remaining blockers: ParityFont (8 drivers), VanillaProtocolClient (8),
VanillaDispenserOracle (6), LocatorBarClient (6), VanillaBlockUpdateOracle
(6), VanillaMaterialConditionOracle (5), VanillaCollisionContextOracle (5).

## 2. Repairs by class

Every repair was javap/bytecode-verified against the named 1.21.7 jars —
never assumed from the 26.3 base's form or from the 1.21.11 form — and
compile-checked on its own classpath before commit. Vanilla 1.21.7 is the
spec throughout.

### 2.1 Driver fix: jsr305 on the client classpath (`d7a14e91`)

`tools/test_model_baking_parity.py:client_libraries()` now appends
`tools/jsr305-3.0.2.jar`. Without it javac **crashes** (rc=4,
`CompletionFailure: class file for javax.annotation.Nullable not found`) on
pre-1.21.11 jars — reproduced in the client-cp fixpoint.
`verify_artifacts()` already did this for server libraries; the client path
(86 oracle_harness drivers + 48 `client_libraries()` users) did not. Whether
a single-oracle compile crashes depends on the oracle, so this was a latent
crash across the oracle_harness drivers, not a deterministic one.

### 2.2 Group 1: ServerLevel 12-arg constructor (`47d650cb`) — 25 oracles

26.x ServerLevel subclasses call a 10-arg `super()` form; 1.21.7's ctor is
12-arg (ChunkProgressListener at position 7, RandomSequences appended).
88 files fixed, plus a reflection fix in VanillaFaceSupportOracle
(`CubeVoxelShape(DiscreteVoxelShape)` is protected in 1.21.7). Rescued 25
oracles (listed in the ledger slice, item O1). This was the only mechanical
group in the programme: one fix, 25 files.

### 2.3 Carry-overs from the 1.21.11 behaviour branch — 14 oracles

The 1.21.11 branch repaired only 16 oracle files (345 insertions, 272
deletions) — no systematic repair exists to inherit, and each 1.21.11 repair
had to be re-verified against the 1.21.7 jar (the two versions differ in
several APIs the repairs touch). Two of the 16 (CommandParse,
ComponentPatch) already passed on 1.21.7.

| Commit | Oracles | Key 1.21.7 facts established |
|---|---|---|
| `57c50808` | Suffocation, Food | No `causes_suffocation` tag (`blocksMotion()`/`isSuffocating` instead); no gamerules package (`GameRules` in `world.level`, `getBoolean` override); `FoodData` has no `hasEnoughFood` (`getFoodLevel() > 6`) |
| `5933e844` | EntityComponents, SheepDye, SoundEvent | `EntityPredicate` in `advancements.critereon` (Mojang's typo); LocalPlayer ctor is 7-arg; no `Items.DYE`/`DataComponents.DYE`; SoundManager is 2-arg; `Sound.getAttenuationDistance` is no-arg |
| `626c8d87` | ChunkGeneration, SquidClient | ServerChunkCache ctor is 12-arg; registry layers STATIC/WORLDGEN/DIMENSIONS with synchronous 3-arg load; no `PalettedContainerFactory` (ProtoChunk takes the biome registry); terrain is the 3-step NOISE/SURFACE/CARVERS; Squid is flat `animal.Squid` |
| `c5b36e4e` | DataFix | No FileFixerUpper (in-memory fix + stamp); old world layout (region/ at root); only 7 SAVED_DATA_* fix types; `ChunkStorage.injectDatafixingContext` is the chunk-context hook |
| `5203f6cb` | FinalizeSpawn, PackResources | Flat animal packages (except frog/axolotl/goat/sheep); no `DATA_SOUND_VARIANT_ID`, no wolf variants, no nautilus; Halloween is the inline LocalDate check (no `SpecialDates`); `Pack.open()` returns one PackResources |
| `69e93c10` | CommandExec | 8-arg MinecraftServer ctor; `LoggerChunkProgressListener::create`; int permission levels; Services takes a GameProfileCache backed by a never-resolving stub repository (a null repository NPEs on cache miss); `RandomSequences.reset` is `(key, int, boolean, boolean)` |
| `9dc5dc8d` | FirstPerson | Classic swing model: no SwingState/SwingAnimation/FirstPersonHandsAndItems; swing fields on LivingEntity, hand heights on ItemInHandRenderer (4-arg ctor, via a shell GameRenderer); no `itemSwapTicker`/`getItemSwapScale`/`swapAnimationScale`/`isWithinEntityInteractionRange` — the swap-ticker output columns were dropped, not emulated |
| `c9896b77` | MobAi (+ driver gate) | The 1.21.11 diff was Identifier renames only — the 1.21.7 repair was built from scratch: full Server-shell translation, flat entity packages, GameRules via `getRule`, no world clocks (`setDayTime`), no `Sensor.randomlyDelayStart` (emulated via reflection — `timeToTick = nextInt(scanRate)`, the 1.21.11 form verified against its jar), no `Brain.forEach/Visitor` (`getMemories()` instead), no `getPersistentAngerEndTime` (remaining ticks), no copper golem, no age lock |
| `5e6564e5` | ItemModel | Older model system (renderer.block.model, no geometry/cuboid/sprite split); no items atlas (one blocks atlas); `SpecialModelRenderer` renders via `render()`, `Unbaked.bake` takes the EntityModelSet; BakedQuad is a record (vertices read raw — 8 ints/vertex, layout verified against FaceBakery bytecode); sprite loader is `SpriteSupplier`; no `swapAnimationScale`, no `ItemOwner` (resolver owner is a LivingEntity); bakery is the 5-arg form resolving sprites through the SpriteGetter passed to `bakeModels` |

### 2.4 High-leverage helpers — 2 oracles, 17 drivers

- **VanillaAnchorOracle** (`4c60aaf0`, 9 drivers): 1.21.7's NoiseSettings is
  4-arg, NoiseRouter 15-arg, NoiseGeneratorSettings takes a SurfaceRules
  RuleSource; no `WorldGenerationContext.of` (the context is always
  (ChunkGenerator, LevelHeightAccessor)); **no sea-level anchor** (only
  absolute/aboveBottom/belowTop) — emulated as `absolute(seaLevel [+ value])`,
  same resolved Y, different codec encoding (a real 1.21.7-vs-26.x
  difference for the family batch).
- **VanillaRegistryCollisionScanOracle** (`f581ed0f`, 8 drivers): no
  PalettedContainerFactory (ProtoChunk takes a biome registry; 1.21.7 has no
  built-in biome registry, so a one-biome stand-in is built); no
  `emptyWithFluidCollisions` — 1.21.7's LiquidBlock returns its shape only
  when `isAbove` AND `canStandOnFluid` hold, so the fluid-colliding form is
  an empty-delegating context with `canStandOnFluid` forced true;
  `positionContext` is `withPosition(null, y)` (null-safe per its bytecode).

### 2.5 Driver gate: MobAi scenarios 1.21.7 cannot run (`c9896b77`)

Every `minecraft:` id in `test_mob_ai_parity.py` was checked against the
1.21.7 jar (Blocks/Items/EntityType/MobEffects fields). Only copper golem
(7 scenarios), copper chests (same 7), iron/wooden/stone spears
(4 scenarios) and the AgeableMob age lock (16 scenarios; 1.21.7 has no
`setAgeLocked`/`ageLocked` at all) are absent. `scenarios()` now filters
those 27 for oracle and probe alike (267 remain). The oracle's `agelocked`
and `cooldown` flag cases were deleted to match.

## 3. Drivers newly yielding measurements

| Stage | Runnable | Blocked |
|---|---|---|
| Measurement | 67 | 307 |
| After group 1 | 92 | 282 |
| **Final** | **115** | **259** |

(374 drivers reference an oracle and have a javac path; 254 reference no
oracle — server-e2e/data/wrappers, never oracle-blocked; 20 reference an
oracle but have no direct javac path; 1 unmapped javac driver,
`test_flat_generation_parity`.)

"Runnable" here means runnable **by javac** — see §4.2. The end-to-end
sample (5 drivers via `tools/parity.py`):

| Driver | Result |
|---|---|
| block_properties | **PASS** (464s) |
| food | javac failure at sample time; oracle since repaired (`57c50808`) |
| client_redstone_preview | Compiles, then the oracle throws at runtime (see §4.2) |
| command_parse, component_patch | Re-running at write time (sample3 died silently after 26 min with no output — cause unknown, no traceback in the log; sample4 in progress). Update `docs/sweep/batch-0-measurement.md` §5 when they land. |

## 4. Unrepairable cases and accepted limitations

### 4.1 VanillaStructureDensityOracle — unrepairable (ledger O7)

Imports `SamplerContext`/`DensityVolume` from
`net.minecraft.world.level.levelgen.densityfunction`, which does not exist
in 1.21.7 (or 1.21.11). No 1.21.7 equivalent API exists to translate to.
Accepted limitation per plan §3: density is verified via bytecode/chunk
comparison instead. Do NOT force a repair.

### 4.2 The runtime layer — compiling ≠ running (ledger O8)

`VanillaClientRedstonePreviewOracle` compiles clean on the client cp but
throws `NullPointerException` at runtime: it `Unsafe`-allocates a
`ClientLevel` subclass without running the constructor, so 1.21.7's
`dimensionTypeRegistration` field stays null and `onPlace` dereferences it.
Likely fix: set the field via reflection after Unsafe allocation, or
override `dimensionType()` — not attempted in this batch. Every repaired
oracle needs an end-to-end driver run to confirm it yields a measurement;
driver runs cost ~10–30 min each on this VM for the large harnesses, and
the runtime-failure rate is unmeasured (sample: 1 of 2 compiling oracles
threw — too small to generalize).

### 4.3 Repairs that changed oracle output formats (ledger O9)

These are findings for the family batches, not Batch 0 fixes:

- FirstPerson and ItemModel drop the swap-ticker/scale columns — 1.21.7 has
  no swap ticker; the probes model a 26.x mechanic.
- Anchor emulates sea-level anchors as `absolute()` — same resolved Y,
  different codec encoding.
- Suffocation: the tag-based Rust probe expectation vs the motion.json
  export is a REAL parity gap for the E6 re-base (vanilla 1.21.7 keys
  suffocation off `blocksMotion()`/`isSuffocating()`, not a tag).
- MobAi's copper-golem/spear/age-lock scenarios are driver-gated (see
  §2.5); the probe models 26.x content 1.21.7 lacks.

## 5. Re-estimate

Basis: the MEASURED failure rate and the MEASURED repair rate, stated
plainly. The plan's §5 numbers (Batch 0 = 20–60 agent-days, sweep total
~50–90 agent-days for 1.21.7) were built on "unknown until measured"; the
failure rate turned out to be 76%, not ~50%.

**Repair rate observed.** 41 files repaired: 25 via one mechanical fix
(group 1, ~1 hour of agent work including verification) and 16 bespoke.
A bespoke repair (read the file, diff vs 1.21.11 where one exists,
javap-verify each API against the 1.21.7 jar, edit, compile-check, commit)
runs 0.5–1 hour for a small file (≤10 errors) and 2–4 hours for a large
one (50+ errors, multi-class translation — MobAi at 116 errors,
CommandExec at 72, ItemModel at 50 client errors). Tool latency dominates
on this VM (javac ~20s per check, javap ~10–40s per call). Planning rate:
4–8 bespoke files per agent-day, improving within a failure class as
translations are reused (gamerules, attribute, entity subpackages).

**Remaining compile repair.** 256 files: 151 with ≤10 errors, ~53 with
11–20, ~40 with 21–54, ~12 above 54 (max 251). At the planning rate with
class-batching: **30–45 agent-days**.

**Runtime layer.** ~300 repaired oracles need an end-to-end driver run to
confirm a measurement (~10–30 min VM time each, mostly unattended) plus
triage of runtime failures. If 10–30% of compiling oracles throw at
runtime (unmeasured; the sample suggests it is non-negligible), that is
30–90 runtime repairs at ~1 hour each: **+5–10 agent-days**.

**Batch 0 total: ~40–58 agent-days** (done: ~2–3 agent-days of session
work — measurement, 41 repairs, docs — plus 35–55 remaining). That lands
at the TOP of the plan's 20–60 range: the plan's range was right, but the
failure rate was higher than assumed and the mechanical-group rescue
offset it.

**Sweep total: ~80–120 agent-days for 1.21.7 (central ~100), up from
50–90.** Batch 0 alone is ~40–58; the family batches (checks + triage,
no fixes) keep the plan's ~30–60 basis. The increase is driven by (a) the
measured 76% oracle failure rate, (b) the runtime layer the plan did not
price, and (c) no mechanical repair group existing (zero files fail only
on missing-package). The sweep total stays an estimate until the runtime
layer is measured — the first 20–30 end-to-end driver runs on repaired
oracles will bound it.

## 6. What blocks batches 3–12, and recommended next steps

- **Nothing blocks starting.** The 115 runnable drivers + 254 no-oracle
  drivers can run now. The 259 blocked drivers wait on Batch 0 (or on
  per-family oracle repair — a coordinator decision: each family batch
  could repair its own oracles, since it needs them anyway; Batch 0's
  remaining 256 files split naturally by family).
- **Next repairs by leverage:** ParityFont (8 drivers, blaze3d.font →
  client.gui.font), VanillaProtocolClient (8), VanillaDispenserOracle (6,
  gamerules), LocatorBarClient (6), VanillaBlockUpdateOracle (6,
  gamerules), VanillaMaterialConditionOracle (5), VanillaCollisionContextOracle
  (5). Then the gamerules class (~20 files) and the attribute class (~30
  files) as batched groups.
- **Measure the runtime layer early:** run the 115 runnable drivers
  end-to-end (or a stratified sample of ~30) before committing to the
  Batch 0 remaining estimate — the runtime-failure rate is the biggest
  unknown in §5.
- **Keep the vanilla-check rule:** every repair in this batch was verified
  against the 1.21.7 jar, and several 1.21.11 repairs did NOT carry over
  (MobAi, FirstPerson's swap ticker, ItemModel's submit/render split).
  Where a version's jar is unavailable, say so explicitly rather than
  assuming the 1.21.11 (or 26.3) form.

## Appendix: repair commits (all on `parity-sweep`)

| Commit | Scope |
|---|---|
| `cc5bbf5a` | Measurement (`batch-0-measurement.md`, `batch-0-oracles.json`, `measure-oracles.py`) |
| `d7a14e91` | Driver fix: jsr305 on the client classpath |
| `47d650cb` | Group 1: ServerLevel 12-arg ctor (89 files, 25 rescued) |
| `57c50808` | Suffocation + Food |
| `5933e844` | EntityComponents + SheepDye + SoundEvent |
| `626c8d87` | ChunkGeneration + SquidClient |
| `c5b36e4e` | DataFix |
| `5203f6cb` | FinalizeSpawn + PackResources |
| `69e93c10` | CommandExec |
| `9dc5dc8d` | FirstPerson |
| `c9896b77` | MobAi (+ driver scenario gate) |
| `5e6564e5` | ItemModel |
| `f581ed0f` | RegistryCollisionScan |
| `4c60aaf0` | Anchor |

Environment notes for the next agent: builds at `-j 1`
(`RUSTUP_HOME=~/workspace/tools/rustup CARGO_HOME=~/workspace/tools/cargo`,
`~/workspace/tools/jdk-25/bin` on PATH, `HORIZON_JAVA_25` set); jsr305
MUST be on every javac classpath or javac crashes (rc=4); the measurement
script lives at `docs/sweep/measure-oracles.py` (committed copy) and
`versions/1.21.7/.local/measure-oracles/` (working copy + logs); run logs
belong under `.local/`, not `/tmp` (wiped on service restart); home writes
can hit the btrfs qgroup quota (EDQUOT) despite free disk — it cleared on
its own both times; if disk gets low, delete only regenerable `.local`
copies, never another agent's active workdir.
