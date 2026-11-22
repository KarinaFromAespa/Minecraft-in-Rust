# Batch 0 measurement: oracle compilability against the 1.21.7 jars

Date: 2026-11-22. Branch: `parity-sweep`. Measured before any repair.
Companion data: `docs/sweep/batch-0-oracles.json` (per-oracle table, all 391
files) and `docs/sweep/measure-oracles.py` (the measurement script, rerunnable).

## Headline

**The true javac-failure rate is 76%, not the ~50% the 17-sample suggested.**
Of 391 Java files in `mods/horizon/src/test/java/dev/horizonlab/parity/`
(357 `Vanilla*Oracle` + 34 helpers/exporters/transcripts/clients), **297 fail
to compile against the 1.21.7 jars on every classpath drivers use**. Only 94
(24%) compile on at least one driver classpath. The 17-harness sample
(9/17 javac-broken, 53%) understated the breakage: the oracles were written
against 26.3, and 1.21.7's API differs across the whole surface — packages,
constructors, method names — not just in a few families.

| Classpath (driver configuration) | Pass | Fail | Failure rate |
|---|---|---|---|
| server — server-game.jar + server libraries + jsr305 (direct drivers) | 82 | 309 | **79.0%** |
| client — client.jar + client libraries + jsr305 (oracle_harness drivers) | 94 | 297 | **76.0%** |
| both — server + client jars + both library sets (superset) | 94 | 297 | 76.0% |
| **fail on all three classpaths (genuinely broken vs 1.21.7)** | — | **297** | **76.0%** |

Notes:

- The `both` set is identical to the `client` set, and every server-pass file
  also passes on client cp: the client jar contains the shared/server classes
  these oracles use, so client cp is the superset that matters. The 12 files
  that pass on client but fail on server are client-only oracles
  (`net.minecraft.client.*`).
- The client-cp measurement adds jsr305 to the classpath. The harness itself
  does not (see §6, driver defect 1): without jsr305, javac *crashes*
  (`CompletionFailure: class file for javax.annotation.Nullable not found`,
  rc=4) on some oracles. The measurement isolates oracle breakage from that
  harness bug; the harness bug is recorded separately and fixed in Batch 0.
- Error totals: server cp 6,199 errors in 299 files (+10 cascade failures);
  client cp 3,916 errors in 286 files (+11 cascade).

## Method

One javac invocation costs ~20 s wall on this VM (throttled), so 391
individual compiles per classpath would take ~2 h each. The measurement uses
a fixpoint over big compiles instead, per classpath:

1. Compile ALL 391 files together → files with own errors E, files without C.
2. Compile C alone (no `-sourcepath`, so a missing helper is a
   `cannot find symbol` in the referring file, not a pull of the broken
   file); files that fail drop out of C; repeat until the compile passes.
3. The fixpoint set is exactly the set of files that compile standalone with
   `-sourcepath`: a file fails standalone iff it has own errors or
   transitively pulls a file with errors, and the fixpoint drops exactly
   those. (Two implementation traps, both hit and fixed during the run:
   `-sourcepath` in the fixpoint iterations pulls broken files back in and
   the fixpoint never converges; and the client cp without jsr305 crashes
   javac — see §6.)

Validation: 5/5 direct per-file `javac` spot checks (driver-style invocation,
`-sourcepath`, single file) agree with the fixpoint — a server pass
(`ParityData`), a client-only pass (`VanillaClientRedstonePreviewOracle`,
`VanillaBlockStateBindingOracle`), a fail-everywhere file (`VanillaFoodOracle`),
and a cascade failure (`VanillaBlockCollisionScanOracle`, failing on its
helper `VanillaFaceSupportOracle`'s error).

Jars: the named 1.21.7 jars from the prepared `.local/minecraft/1.21.7/`
(see §7); `javac --release 21 -proc:none` (lock's `java_major`), JDK 25.0.5.

## Failure classes (first error per file)

Server cp (309 failures):

| Class | Files | Meaning |
|---|---|---|
| missing-package | 144 | imports a package that does not exist in 1.21.7 (26.x-only package, or client-only on server cp) |
| missing-symbol | 88 | references a class/method/field 1.21.7 does not have (renamed/moved API) |
| constructor-mismatch | 47 | calls a constructor with a 26.x signature (`ServerLevel`, `ServerPlayer`, …) |
| cascade | 10 | file itself is clean; fails only because a helper it pulls is broken |
| other | 9 | assorted (static-context, access, …) |
| no-override | 6 | `@Override` of a method 1.21.7's supertype lacks |
| incompatible-types | 4 | 26.x vs 1.21.7 signature change (e.g. `boolean` where 1.21.7 takes `ChunkProgressListener`) |
| missing-override | 1 | abstract-method gap |

Client cp (297 failures): missing-package 89, missing-symbol 120,
constructor-mismatch 53, cascade 11, other 10, no-override 8,
incompatible-types 5, missing-override 1. The client cp sheds the
client-package failures and exposes more missing symbols — the same
underlying 26.x-vs-1.21.7 drift.

### The 26.x-only packages (missing on BOTH classpaths)

These packages do not exist in the 1.21.7 jars at all — every oracle
importing them needs a rewrite against the 1.21.7 form, not a classpath fix
(error counts from the big compiles):

| Package | Errors (server / client) | 1.21.7 form (to verify per oracle) |
|---|---|---|
| `net.minecraft.world.attribute` | 70 / 70 | `net.minecraft.world.entity.ai.attributes` |
| `net.minecraft.world.level.levelgen.densityfunction` | 40 / 40 | density functions live under `levelgen.synth` / noise; no `SamplerContext`/`DensityVolume` API |
| `net.minecraft.world.level.gamerules` | 36 / 36 | `net.minecraft.world.level.GameRules` (+ inner `Key`/`Type`) |
| `net.minecraft.server.permissions` | 19 / 19 | no equivalent; permission checks differ |
| `net.minecraft.world.clock` | 14 / 14 | no equivalent |
| `net.minecraft.server.notifications` | 13 / 13 | no equivalent |
| `net.minecraft.world.entity.npc.villager` | 13 / 13 | classes live in `world.entity.npc` / `world.entity` |
| `net.minecraft.world.entity.projectile.arrow` | 12 / 12 | classes live in `world.entity.projectile` |
| `net.minecraft.world.entity.animal.equine` | 12 / 12 | classes live in `world.entity.animal` |
| `net.minecraft.world.entity.monster.zombie` | 11 / — | classes live in `world.entity.monster` |
| `net.minecraft.world.entity.animal.cow` | 11 / — | classes live in `world.entity.animal` |
| `net.minecraft.util.debug` | 12 / 12 | no equivalent |
| `net.minecraft.client.resources.model.sprite` | — / 14 | sprite/model API reorganised in 1.21.7 |

Plus the signature drift: `ServerLevel`'s constructor (88 errors in the
server big compile — 26.x takes a `boolean` where 1.21.7 takes a
`ChunkProgressListener`, 86 errors), `ServerPlayer`'s constructor, and 782
`method does not override` errors (server cp) where oracles subclass vanilla
classes and override 26.x methods.

## Driver impact

Driver→oracle map built by parsing all 648 drivers (`.java` basenames +
oracle-name string literals passed to `oracle_harness`). Each driver's
oracle set was checked against the classpath its own path uses
(oracle_harness → client, direct `jars['server']` → server,
both jars → both).

| Driver state | Drivers |
|---|---|
| **Blocked: ≥1 oracle fails to compile under its path's cp** | **307** (171 server + 103 client + 33 both) |
| Oracles compile under their path's cp (can yield a measurement when run) | 67 (38 server + 23 client + 6 both) |
| Reference an oracle but have no direct javac path (indirect/shared runners — verify per driver) | 19 |
| Javac driver whose oracle ref was not mapped (`test_flat_generation_parity`) | 1 |
| No oracle reference (server-e2e, data-only, wrappers) — not oracle-blocked | 254 |
| **Total** | **648** |

- 275 distinct oracles block at least one driver. The highest-leverage
  repairs are shared helpers: `VanillaFaceSupportOracle` blocks 19 drivers,
  `VanillaAnchorOracle` 9, `VanillaRegistryCollisionScanOracle`,
  `ParityFont` and `VanillaProtocolClient` 8 each, `VanillaSuffocationOracle`
  7. Fixing the top 5 unblocks ~59 driver-oracle pairs (overlaps exist).
- 26 oracles are never referenced by any driver; 12 of them fail on all cps
  — lowest repair priority (dead code unless a batch adopts them).
- The 254 no-oracle drivers are not oracle-blocked, but that does not mean
  they pass: they run real servers / the game / data diffs and have their
  own failure modes (see the end-to-end sample, §5).

## The 17-harness check

Of the 9 harnesses the plan (§2) records as dying on oracle javac failures
in CI run 44441686979 (2026-11-19), **8 still fail** and 1 does not:

| Harness | Oracle | Now |
|---|---|---|
| chunk_generation | VanillaChunkGenerationOracle | FAIL (constructor-mismatch) |
| command_exec | VanillaCommandExecOracle | FAIL (missing-package) |
| component_patch | VanillaComponentPatchOracle | **PASS on both cps** |
| datafix | VanillaDataFixOracle | FAIL (missing-package) |
| first_person | VanillaFirstPersonOracle | FAIL (missing-package) |
| food | VanillaFoodOracle | FAIL (missing-package) |
| item_model | VanillaItemModelOracle | FAIL (missing-package) |
| options | VanillaOptionsOracle | FAIL (missing-package) |
| resource_pack | VanillaPackResourcesOracle | FAIL (missing-package) |

`component_patch` is the exception: its oracle compiles cleanly against the
1.21.7 jars on both classpaths, and the file is untouched since the original
1.21.7 port commit (be65dca7) — so the Nov 19 CI failure for that harness
was either misattributed in `status.json` or sat elsewhere in the harness
(the end-to-end sample runs it; see §5).

## End-to-end sample (drivers run via `tools/parity.py`)

Compile status is not run status. A 5-driver sample across the classes:

| Driver | Oracle status | End-to-end result |
|---|---|---|
| food | FAIL (server cp) | **javac failure, as measured** — `VanillaFoodOracle` fails on `net.minecraft.world.level.gamerules` |
| client_redstone_preview | PASS (client cp) | **Compiles, then the oracle throws at runtime**: `NullPointerException: Cannot invoke Holder.value() because this.dimensionTypeRegistration is null` — the oracle `Unsafe`-allocates a `ClientLevel` subclass without running the constructor, so 1.21.7's `Level` field stays null and `onPlace` dereferences it. Compiling ≠ running: runtime repair is a second layer on top of compile repair. |
| block_properties | PASS (server cp) | running at push time — cargo build of `world_probe` in progress |
| command_parse | PASS (server cp) | running at push time — cargo build of `client_probe` in progress |
| component_patch | PASS (server cp) | running at push time |

Sample status at push: 2/5 complete, 3/5 running (cargo probe builds).
This doc is updated when the remaining three land; the compile measurement
above is complete and final regardless.

## Driver defects found (Batch 0 scope)

1. **oracle_harness / `client_libraries()` omits jsr305 — javac crashes.**
   `tools/test_rust_parity.py:verify_artifacts()` appends
   `tools/jsr305-3.0.2.jar` to the *server* libraries with the comment that
   javac crashes completing symbols without it on pre-1.21.11 jars — but
   `tools/test_model_baking_parity.py:client_libraries()` (used by
   `oracle_harness.compile_oracle` and 48 files) does not. Reproduced:
   the client-cp fixpoint compile crashed with
   `CompletionFailure: class file for javax.annotation.Nullable not found`
   (javac rc=4). Whether a single-oracle compile crashes depends on the
   oracle (some complete a `Nullable`-referencing symbol, some do not), so
   this is a latent crash across the 86 oracle_harness drivers, not a
   deterministic one. Fix (Batch 0, driver fix): append jsr305 in
   `client_libraries()` — one point, fixes all 48 users.
2. **`.local/minecraft/1.21.7/` did not exist in any worktree** (the pilot's
   blocker, §3). Resolved — see §7. No driver could run VM-local before this.

## `.local/minecraft/1.21.7/` setup (recipe for later batches)

`verify_artifacts()` needs `manifest.json` (pinned sha1), `jars/client.jar`
+ `jars/server.jar` (manifest sha1s or `remapped.json` digests), the inner
`jars/server-game.jar` (bundle checksum or `remapped.json`), and
`libraries/server/` + `libraries/client/` (sha-verified). No worktree had
it; `lab.py prepare` would re-download + remap. Instead, reuse the complete
prepared directory the 1.21.7 port work already produced:

```sh
cd versions/1.21.7
mkdir -p .local/minecraft
cp --reflink=auto -r ~/workspace/mcir-work/e6-1.21.7/versions/1.21.7/game .local/minecraft/1.21.7
ln -s ~/workspace/minecraft-in-rust/versions/1.21.7/.local/horizon-client .local/horizon-client
python3 -c "import sys; sys.path.insert(0,'tools'); import test_rust_parity as t; t.verify_artifacts(); print('GATE OK')"
```

Source layout (223 MB, reflinked so ~0 new disk): `game/` is a prepared
`.local/minecraft/1.21.7/` — manifest sha1 `48c9b004…` (the pinned one),
named client + named inner server jars with `remapped.json`, both library
sets, `prepared.json`, `reports/`. Verified: the gate passes (44 server
libraries). Do NOT `cp -a` (source files are root-owned; ownership
preservation fails) — plain `cp -r` works. Never commit `.local`.

Unit tests use `HORIZON_TEST_RESOURCES=~/workspace/mcir-work/mob-spawn/resources`
(the reusable 125 MB 1.21.7 test data), per the batch brief.

## What the numbers mean (preliminary — full re-estimate in the final report)

- Batch 0's repair load is **297 files**, not the ~half the 17-sample
  suggested. The dominant work is per-oracle API translation (26.x → 1.21.7
  packages/constructors/symbols), verified against the named 1.21.7 jar —
  the same shape as the 1.21.11 branch's 16 oracle repairs, at ~19× the
  file count. The 1.21.11 repairs are carry-over candidates where the
  1.21.7 API matches (verified per file, never assumed).
- The §5 cost basis (Batch 0 = 20–60 agent-days, sweep total ~50–90) was
  built on "unknown until measured". The measurement now bounds it: 297
  files × per-file repair effort + a runtime-failure layer the compile
  pass cannot see (the sample's 1-for-1 runtime failure on a compiling
  oracle). The final Batch 0 report re-estimates from the *measured*
  repair rate once the first repair groups land.
- Sequencing for batches 3–12: the 67 oracle-clean drivers can run now;
  the 307 blocked drivers wait on Batch 0 repairs, shared helpers first
  (`VanillaFaceSupportOracle` unblocks 19 drivers alone). The 254
  no-oracle drivers were never oracle-blocked and can run in parallel with
  Batch 0.

## Per-oracle table

`docs/sweep/batch-0-oracles.json` — all 391 files: pass/fail per classpath,
first-error class + message + error count for failures. Raw big-compile logs
(6,199 server-cp / 3,916 client-cp errors) are in the measuring workdir's
`.local/measure-oracles/` (not committed — regenerable with
`docs/sweep/measure-oracles.py`).
