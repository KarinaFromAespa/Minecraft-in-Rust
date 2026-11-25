# 1.21.11 parity sweep — summary

**Version:** 1.21.11 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.11`
(branched from main `d148da50`) · **Ledger:** `pins/1.21.11/sweep-items.json`
(merged from the 13 per-batch slices in `pins/1.21.11/sweep/batch-NN.json`).
Per-batch reports live in this directory (`batch-N-*.md`); the universe
inventory is `inventory.md` / `inventory-1.21.11.json`.

Checks + triage only. No fixes were made in any batch; no production code
under `versions/1.21.11/` was touched. Every `fix` field in the merged ledger
is null. Batches ran 2026-11-25 (06:33–09:15 NPT); the merge landed the
same morning.

## Headline

| | Total | Pass | Gap | Accepted-gap |
|---|---:|---:|---:|---:|
| Checks as tallied | 9,810 | 8,143 | 1,667 | 0 |
| Checks, behaviour only | 9,766 | 8,132 | 1,634 | 0 |

The two rows differ by 44 non-behaviour checks three batches count in
their as-tallied tallies, each per a 26.2 precedent the coordinator
accepted: Batch 4 counts 37 driver-status checks (9 pass / 28 gap),
Batch 3 counts 3 driver-code-defect checks (1 pass / 2 gap), Batch 9
counts 3 oracle items + 1 harness blocker as checks (1 pass / 3 gap:
H01 + O01/O02/O03). No other batch counts oracle-status checks. Quote
the behaviour-only row when comparing against other versions' behaviour
tallies.

**Ledger items:** 417 raw across the slices, **394 unique** after dedup
(23 whole-item duplicates folded, 0 minted at merge — see Dedup
below). **305 live** (191 gap + 104 oracle-defect + 10 needs-harness),
**77 live major** (55 gap + 22 oracle-defect by status; the 55 includes
the 10 Batch 0 repair groups, all major). Non-live: 55 fixed (1.21.7 /
1.21.10 defects withdrawn for 1.21.11), 31 closed (1.21.7 / 1.21.10
defects re-checked and closed), 3 accepted-gap.

**Comparison with the 1.21.10 sweep** (9,711 checks, 8,015 pass, 1,696
gap, 379 items, 307 live, 75 live major), **the 26.1.2 sweep** (9,855
checks, 8,260 pass, 1,595 gap, 302 items, 262 live, 61 live major),
**the 26.1.1 sweep** (9,851 checks, 8,254 pass, 1,596 gap, 300 items,
262 live, 60 live major), **the 26.1 sweep** (10,016 checks, 8,502
pass, 1,514 gap, 201 items, 186 live, 50 live major), **the 26.2 sweep**
(10,374 checks, 10,058 pass, 314 gap, 113 items, 93 live, 20 live
major) **and the 1.21.7 sweep** (10,341 checks, 7,730 pass, 2,609 gap,
208 items, 35 major): 1.21.11 ran 99 more checks than 1.21.10 (B12's
registry count is larger — 8,290 vs 8,248, because 1.21.11 ships more
data-driven files, including the `game_rule` and `environment_attribute`
registries 1.21.10 lacks; B9 ran 25 vs 24) and found 29 fewer gaps
(1,667 vs 1,696), 15 more unique items (394 vs 379), 2 fewer live
(305 vs 307) and 2 more live majors (77 vs 75). On paper 1.21.11 is
marginally cleaner than its first child — and the dirt is the same
dirt: the template's own defects, confirmed at the source. The
1.21.7 fixes that were supposed to carry into the E6 template never
did (fluid spreading, the supports_* synthesis, the motion-tag
derivation, the palette reader), and here they are found in the
template itself, not inherited. What is genuinely new here is the
loading stick (B10-G16, code-verified in the template), the B6-O09
ServerLevel ctor group, ten new oracle defects (B7-O31–O40), the
ComponentPatch repair (O12 FIXED), the O13 format-adaptation group,
and — where vanilla changed — the template being cleaner than its
first child: the gamerule table (B9-G01 withdrawn), the Time argument
(B9-G10 closed), `nautilus_one_cm` (B9-G21 halved), no `/swing`
(B9-G19 doubly withdrawn). The oracle measurement is the first
improvement in the series: 127/391 compile (32.5%), up from 104
(26.6%) in 1.21.10, with 23 new passes and 0 new fails.

## Per-batch tallies

| Batch | Scope | Checks | Pass | Gap | Items |
|---|---|---:|---:|---:|---:|
| 0 | Oracle measurement | 391 | 127 | 264 | 13 |
| 1 | Fluids + falling blocks | 95 | 78 | 17 | 20 |
| 2 | Redstone + pistons | 53 | 46 | 7 | 21 |
| 3 | Entities / mobs | 99 | 65 | 34 | 59 |
| 4 | Block entities + containers | 96 | 42 | 54 | 63 |
| 5 | Block behaviour | 130 | 104 | 26 | 37 |
| 6 | Worldgen / features | 37 | 22 | 15 | 28 |
| 7 | Items | 45 | 31 | 14 | 59 |
| 8 | Particles | 277 | 265 | 12 | 22 |
| 9 | Commands | 25 | 7 | 18 | 28 |
| 10 | Rendering, sound, UI | 25 | 13 | 12 | 19 |
| 11 | Rank-3 triage (247 items) | 247 | 243 | 4 | 40 |
| 12 | Data-driven registries | 8,290 | 7,100 | 1,190 | 8 |
| **Total** | | **9,810** | **8,143** | **1,667** | **417 raw / 394 unique** |

B3/B4/B9 count non-behaviour checks in their as-tallied tallies (see
Headline); their behaviour splits are B3 96 (64/32), B4 59 (33/26),
B9 21 (6/15).

## The story: 1.21.11 is the template itself — and the dirt is at the source

Step 0 (provenance, verified before any batch was briefed) established
that 1.21.11 IS the E6 template. The behaviour port itself is 815/815,
merged 9de51367 with CI proof run 44989513836 (profile 5 pass / 18 fail
of 23; D1 fail on 21 dead tags, D2 pass, D3 pass 502/502). The holder
shape is 1.21.x (no EntityTypes/BlockEntityTypes in either jar),
confirmed by jar listing. The jars are the named game jars
(client-named 10,291 classes, server-game-named 7,163 classes, sha1s
verified); the `server.jar` in the same directory is a Mojang bundler
(4 classes) and was never javap'd. DataVersion 4671, protocol 774,
Java 21, data pack 94.1, resource pack 75.0.

The sweep's central finding is that the 1.21.10 sweep's carry-over
failure is not a carry-over failure at all — it is a template failure.
The 1.21.7 fixes never reached the template, and the template is where
they are missing:

- **The loading stick EXISTS** (B10-G16, major, NEW): `remote_loading()`
  exists at game/remote.rs:10906 but none of the three pause sites is
  gated (Escape play_keys.rs:78-88, `pauses_game()`
  pointer_capture.rs:47-52, focus-loss pause pointer_capture.rs:72-95).
  Code-verified by the coordinator; the deadlock itself was not
  reproduced headless. Propagation is a separate owner decision.
- **No fluid spreading at all** (B1-G05, major): the template's own
  defect, not an inheritance. The port's fluid code still reads 26.x
  tags that 1.21.11 never shipped.
- **Seven blocks can never survive anywhere** (B5-G37–G42 + B12-G09,
  all major): sugar cane, cocoa, azalea, mangrove propagule (both
  forms), nether sprouts, nether wart, frogspawn. The survival code
  reads eleven supports_* tag names that do not exist in 1.21.11; no
  `synthesize_legacy_support_tags` code exists in the template tree.
  Each block's vanilla form was javap-verified fresh.
- **The client cannot start** (B12-G04, major): `Assets::load`
  hard-requires three 26.x-only motion tags
  (`blocks_motion_in_heightmap`, `..._no_leaves`, `blocks_fluid_flow`)
  with hard errors; 1.21.11 never shipped them and no synthesis or
  derivation exists in the tree. A second, independent startup
  blocker lives in B3-G22 (major): `MotionResources::load` requires
  the absent `#suppresses_bounce` / `#causes_suffocation` tags.
- **Vanilla-written worlds load as void** (B11-G10, major): no
  `from_palette_nbt` anywhere in the tree; `from_nbt` reads strict
  `"id"` only; datafix 5006 never runs at DataVersion 4671; the
  reference world uses `Name`/`Properties` palettes throughout.
  Verified fresh at all three links (reader removed + save.rs maps
  unresolvable entries to air + reference-world palettes).
  Statically derived (not demonstrated in-game).
- **No 1.21.11 world can generate at all** (B6 cascade, 20 live
  majors): empty carver/feature bundles (G06) → 7 missing density
  arms (G13) → `material_rule` (G11) → `spawn_target` (G12), with
  aquifers (G09) behind them. Plus ore veins never generate (G18).
- **Both loot engines dead** (B12-G01/G02, both major): items.rs
  makes every condition false (1,190/1,324 tables affected, post-fix
  residual again exactly 102); loot.rs ignores all conditions and
  functions (demonstrated live with `loot_fill` — 7 tables × seeds
  1–5, the second loot_fill run in the programme).
- **Splash water never douses** (B1-G03, major, canonical of the
  3-way fold): vanilla gates on `PotionContents.is(Potions.WATER)`;
  the port gates all four effects on potion tags that do not exist
  in 1.21.11 (there is no PotionTags class in either jar and 0 potion
  tag files).
- **The gamerule table is RIGHT** (B9-G01, withdrawn): vanilla
  1.21.11's `game_rule` registry uses snake_case ids — the port's
  table matches exactly (59 = 59 names, defaults match, 0 gamerule
  mismatches in 492 run cases). The 1.21.10 defect was a vanilla
  change between the versions, not a port defect.
- **Four 26.x-only commands in the release tree** (B9-G02, major):
  `compute`, `posteffect`, `unpublish` — vanilla 1.21.11 has no
  `/swing` at all (B9-G19 doubly withdrawn).

The oracle layer is the first improvement in the series: 391 files,
127 pass / 264 fail-all (32.5%), 289 drivers blocked, all 9 harness
oracles passing. 23 new passes vs 1.21.10, 0 new fails. The new O10
group is now B6-O09 (ServerLevel ctor, 4 oracles — the 85-file O10
group is the Batch 0 view). The `VanillaCommandExecOracle` compiles
clean now (B9-O05 FIXED), so `command_exec` is one fixture away from
running. The ComponentPatch oracle is FIXED (O12 — the tree carries
the 1.21.7 repair form). But jsr305 is WORSE than 1.21.10: it is
absent from the tree entirely (O2), not just from the server path.

It is not all dirt. Genuinely fixed here, several of them firsts
across the sweep programme:

- **Redstone wire FIXED** (B2-G01): the class-alias mechanism
  dispatches wire correctly — the fix originates in the template.
- **Sign boards FIXED** (B10-G01): signs render boards — originated
  in the template (ac8dd9dc).
- **Fire tick PASSES** (B5): no doFireTick rule to honour — the port
  is correct by construction.
- **Beds place correctly** (B11-G20, fixed — 1.21.7's B11-G01): beds
  place foot+head and sleep works. But beds are INVISIBLE (B10-G03,
  major) — the renderer doesn't draw them.
- **H01 FIXED**: the client_probe compiles AND links (cargo check
  EXIT=0 in 20m13s, rmeta present) — so `command_parse` ran
  end-to-end, the only end-to-end driver run in this sweep besides
  `loot_fill`.
- **Lib tests UNVERIFIED, not broken** (B11-H03): no crate-tests run
  has ever tested 1.21.11 — CI status is unknown, not a failure.

## Live majors (77)

**Startup / world-loading (6):** B12-G04 (Assets::load motion tags —
client cannot start), B3-G22 (MotionResources tags — suffocation
startup blocker), B11-G10 (vanilla worlds load as void), B12-G01
(items.rs loot engine dead — every condition false), B12-G02
(loot.rs loot engine dead — conditions/functions ignored), B10-G16
(the loading stick exists).

**Worldgen cascade (20):** B6-G01, B6-G02, B6-G03, B6-G06, B6-G09,
B6-G11, B6-G12, B6-G13, B6-G14, B6-G16, B6-G18 (ore veins never
generate) + the nine B6 oracle groups O01–O09 (densityfunction,
material/rule, feature, spawn, ServerLevel ctor, and template-API
drift).

**Survival / blocks (11):** B5-G37–G42 (six supports_* blocks),
B12-G09 (frogspawn), B5-G04 (fungi/roots), B5-G13 (dripstone tag half
dead), B5-G21 (snow/ice), B5-G22 (composter).

**Fluids (3):** B1-G05 (no fluid spreading at all), B1-G03 (splash
water never douses — canonical of the 3-way fold), B1-G01
(dripstone system half dead).

**Redstone / pistons (5):** B2-G02 (push-reaction loader matches no
1.21.11 value — every state loads as PushPull), B2-G03 (is_pushable
lacks vanilla's hardcoded obsidian-family refusal), B2-O01
(BlockUpdate, 38 errors), B2-O02 (RedstoneCatalog), B2-O06
(FaceSupport).

**Entities / block entities (12):** B3-G14 (endermen never teleport),
B4-G01 (furnaces never light), B4-G02 (brewing never brews), B4-G16
(cartography), B4-G30 (composter), B4-O06 (Dispenser), B4-O17
(ContainerScreen), B4-O23 (NaturalSpawn), B4-O24, B7-G07 (tool
transforms), B7-G08 (campfire douse), B7-G09 (chorus fruit).

**Commands / UI / sound (10):** B9-G02 (26.x-only commands), B9-O01
(Scoreboard, 3 errors), B9-O02 (TeamRules, 6 errors), B9-O03
(LocatorBarClient, 17 errors), B8-G10 (/spreadplayers always fails),
B10-G02 (no sounds.json), B10-G03 (beds invisible), B10-O03 (render
family, 18 oracles), B10-O04 (sound family), B10-O05 (UI family, 11
oracles).

**Oracle families (10 B0 groups):** O1 (LocalPlayer ctor), O2 (jsr305
crash), O3 (fuelValues), O4 (densityfunction), O5 (EntityTypes holder
move), O6 (26.3/1.21.11-only packages), O7 (264 fail / 289 drivers
blocked), O8 (runtime layer unmeasured), O9 (re-ported 26.3-form
sources), O10 (ServerLevel ctor, 85 files).

Full list in the ledger (`totals.live_major_ids`).

## Strongest verifications

- **Commands (B9):** the run — 34,492-command corpus regenerated from
  1.21.11's own tree export (md5
  `9e9597f33c9c5bc9e2112f0bda481720`), perm 4: 3,172 mismatches (90.8%
  match), perm 2: 3,158, perm 0: 1 (root suggestion list only). 14
  oracle-crash rows per perm (`random` + `schedule clear` NPEs) — so
  G20 rests on static evidence alone. Recipe suggestion mismatches
  are the predicted oracle artifacts. The run contradicted NO static
  verdict. A VM reboot at ~03:10 UTC killed the first driver build at
  232 crates; relaunched from the warm cache, it finished clean in
  52m00s, probe linked.
- **Registries (B12):** prepared data byte-faithful (5,096/5,096 files
  identical). `loot_fill` WAS run — G02 demonstrated live, 7 tables ×
  seeds 1–5. This batch ran horizon-prepare itself (build 14m39s exit
  0, run 1,373.9s exit 0); the prepared 1.21.11 tree now exists in the
  worktree's `.local` (552 MB, later 665 MB).
- **Worldgen (B6):** the construction cascade verified link by link
  against code + prepared data + jar; the flagship flip —
  VanillaChunkGenerationOracle PASSES all three classpaths, confirmed
  against the B0 table. Reference-world census: 0 snow / no frozen
  peaks / 0 sculk (seed -76134417269081163).
- **Entities (B3):** the suffocation rule is fixed but the load path
  still requires the absent tags (G22) — the fix and the blocker
  distinguished by direct code read. All 42 in-scope oracles
  cross-checked vs the B0 table: 0 disagreements.
- **Items (B7):** every 1.21.10 behaviour defect reproduces verbatim;
  the oracle triage was MEASURED (own full javac fixpoint: 127/391,
  matching B0 exactly). Ten NEW oracle defects O31–O40.
- **Rank-3 (B11):** class-table jar half run (265/271 live, 6
  jar-dead all inert); G10 verified end-to-end by the coordinator
  (reader removed + save.rs maps unresolvable entries to air +
  reference world uses `Name`/`Properties` palettes with zero strict
  `id` entries). Lib-test status UNVERIFIED (B11-H03), not breakage.
- **Fluids (B1):** censuses use the real 1.21.10 prepare as the
  prepared side (no 1.21.11 prepare existed at the time), via a
  jar-equivalence chain — stated in the report, not hidden.
- **Particles (B8):** the B8 parser gotcha observed (the limiter
  boolean is the `iconst` AFTER the name `ldc`); G10 (/spreadplayers
  always fails) is the batch's only major. Nautilus family arrived:
  H04 reversed to a live gap, H05 withdrawn.
- **Block behaviour (B5):** the G17/G18 corrections confirmed fresh
  against 1.21.11's own jar/tree (chorus ceiling, bell isProperHit —
  both never existed); the B5-O02 flip applied at landing (Portal
  claimed FIXED on symbol-level evidence, but the B0 table shows
  fail-all — it depends on the broken VanillaBlockUpdateOracle).
- **Rendering (B10):** the loading stick code-verified by the
  coordinator (three pause sites, none gated on `remote_loading()`);
  the sign-board fix verified (G01, originated in the template);
  six oracles that failed for 1.21.10 now pass (FirstPerson,
  ItemModel, PackResources, SoundEvent, ZombieCure, EntityParticles).
- **Redstone (B2):** the wire fix verified (class-alias mechanism,
  originates in the template); the G11/G12 corrections verified
  against 1.21.7's own bytecode (both never existed — a third
  confirmation).
- **Oracle measurement (B0):** 391 files, 127 pass / 264 fail-all
  (32.5%), 289 drivers blocked; first improvement in the series (23
  new passes, 0 new fails vs 1.21.10). H01 rerun finished 02:13 UTC:
  `cargo check` EXIT=0, 20m13s, rmeta present.

## Needs-harness (10)

All minor: B2-H01 (tick-by-tick circuit equivalence + piston corpus
driver runs), B2-H02 (the 4 level::tests need a cargo test run), B6-H
(paired port-vs-vanilla chunk comparison — stage rule applies),
B8-H01 (providers driver omits vibration), B8-H02 (providers driver
includes 13 kinds 1.21.11 lacks), B8-H03 (marker driver zoo includes
red_poplar_leaves), B8-H04 (entity driver omits zombie_nautilus and
camel_husk, which exist in 1.21.11 with inherited burst events),
B11-H01 (class-table chain-liveness half cannot run — no
block-behaviours.json), B11-H02 (cauldron interaction table vs Rust —
carries from 1.21.7), B11-H03 (no crate-tests run has ever tested
1.21.11).

## Dedup record

23 whole-item folds (duplicate removed from the merged list, kept in
its slice): splash water 3-way (B7-G10, B11-G05 → B1-G03), mud
(B7-G11 → B5-G19), spawner setEntityData (B7-G17 → B3-G39), dripstone
tag half (B11-G04 → B5-G13), snow-layer tags (B11-G14 → B5-G08),
loot condition() (B4-G26 → B12-G01), cauldron 3-way (B5-O01, B11-O01
→ B1-O02), FallingBlock (B5-O05 → B1-O03), LavaCauldronItem 3-way
(B7-O07, B11-O02 → B1-O07), dispenser 3-way (B5-O06, B7-O01 →
B4-O06), BlockUpdate (B7-O02 → B2-O01), FaceSupport (B5-O09 →
B2-O06), VehicleContainer (B7-O10 → B4-O21), EnchantDurability
cascade (B7-O29 → B4-O19), render/UI singletons → B10 families
(B5-O07, B5-O08, B4-O16 → B10-O03; B4-O18, B4-O08 → B10-O05).

13 oracle member folds (group item stays; the member folds into the
earlier canonical item): see `dedup.oracle_member_folds` in the
ledger. Cascade/dependency mentions are NOT members — BlockUpdate
named in B4-O03/B4-O19/B7-O29/B7-O30/B3-O07 as the cascade source,
FaceSupport named in B2-O07/B3-O07 as the cascade source, and
RegistryFluid named in B1-O04 as a shared-symbol cross-reference get
no fold.

0 items minted at merge: B12-G09 (frogspawn) was already minted in
the B12 slice.

Kept separate (cross-referenced, not folded): B11-G07 vs B4-G01
(furnace fuel — B11 is the recipe-book fuel list, B4 is the furnace
lighting; same root cause, different behaviours — the 1.21.10
precedent); B12-G04 vs B3-G22 (startup blockers — different loaders,
different tags); B1-G01 vs B5-G13 (dripstone — B1 covers the
fluid-transfer/cauldron side plus the blockToGrowOn link cause).

Differences from the 1.21.10 folds, each decided from the 1.21.11
evidence: B4-G26 → B12-G01 and B11-G14 → B5-G08 are new; B12-G03 →
B5-G37 does not apply (no B12-G03 exists — the census item was never
minted); B3-O08 → B1-O06 does not apply (B3-O08 is CLOSED in
1.21.11); B7-O09 → B10-O03 does not apply (B7-O09 is FIXED in
1.21.11, and B10-O03 already lists ItemModel as passing).

## Coordinator corrections (during reconciliation)

1. **B5-O02: fixed → oracle-defect (at landing).** B5 claimed
   VanillaPortalOracle is FIXED on symbol-level evidence. The B0
   table shows it fails on all three classpaths (it depends on the
   broken VanillaBlockUpdateOracle, 38 errors). Flipped in the report
   and the slice (11 edits; PRIMARY 28 REPRODUCE / 1 FIXED / 2 NEVER
   EXISTED; SECONDARY 11 / 9 / 2).
2. **B7-O11: fixed → oracle-defect (worker-corrected).** B7 claimed
   KnockbackClient is fixed; its own reconciliation found 7 new
   errors. Corrected in the slice before landing.
3. **B3's oracle figure: 142 → 127 (at landing).** B3's report quoted
   142/391 — a single-round count, not the fixpoint method. Corrected
   to B0's 127/391 (the 15-file gap is cascade dependents).
4. **B9-G01 withdrawal verified (coordinator).** The prepared
   registries.json shows 59 `minecraft:game_rule` entries with
   snake_case ids — vanilla 1.21.11 uses snake_case, the port's table
   matches exactly. The withdrawal stands.
5. **H01 timeline.** The first H01 run died ~01:02 UTC (compaction
   window); the rerun finished 02:13 UTC, `cargo check` EXIT=0,
   20m13s, rmeta present. H01 is confirmed (probe compiles AND
   links).
6. **Disk.** The merge.py write hit the VM quota at 104G used; 8.7G
   freed per the standing rule (b00/target 5.8G + b09/target 2.9G
   cargo build caches — H01 closed with rmeta evidence in the B0
   report, B9 landed with its artifacts preserved outside target/).

**Oracle reconciliation:** 106 singleton oracle items checked against
the landed Batch 0 table — 0 real contradictions. The 3 apparent
flags (B7-O09 ItemModel, B8-O04 EmitterParticles, B8-O05
EntityParticles) claim FIXED while the table says fail/pass/pass: all
three are client oracles whose evidence explicitly says they pass on
client + both; the server-classpath failure is the expected artifact
for client-only classes.

## Ledger corrections owed to other ledgers (parent action)

- **1.21.10 ledger:** B4-G26 (the 1.21.10 report's
  test_crafter_drop_name pass was wrong — condition() reads "type"
  only; the 1.21.11 B4-G26/B12-G01 pair proves it) and B11-G16 NEVER
  EXISTED (the 1.21.10 tree reads `dyeable` too; the agent cited the
  :190 doc comment, not the :8-11 code).
- **1.21.7 ledger:** B2-G11/G12 NEVER EXISTED (1.21.7's own bytecode
  contradicts both; confirmed a third time in this run) and B10-G04
  NEVER EXISTED extends to 1.21.7 (the 1.21.7 tree has all five
  geometry functions with live call sites).
- **26.1.2/26.1.1/26.1 ledgers:** B5-G17 chorus ceiling NEVER EXISTED
  (getMaxY() is 319 in both jars) and B5-G18 bell + B10-G04 BE
  renderers NEVER EXISTED — corrections owed "whenever those are next
  touched".

## Programme close: this is the last sweep

1.21.11 is the LAST sweep in the owner's programme (every version
from 26.2 down; 1.21.7 already swept). There is no next run, and no
handover list. Programme totals across the seven sweeps: **69,958
checks** (26.2: 10,374; 26.1: 10,016; 26.1.1: 9,851; 26.1.2: 9,855;
1.21.10: 9,711; 1.21.11: 9,810; 1.21.7: 10,341) and **1,897 unique
ledger items** (113 + 201 + 300 + 302 + 379 + 394 + 208).

What the programme established, version by version:

- **The template is the defect.** 1.21.11's sweep confirms at the
  source what the 1.21.10 sweep inferred from inheritance: fluid
  spreading, the supports_* synthesis, the motion-tag derivation, the
  palette reader, and the loading stick are all missing in the
  template itself. Any fix programme starts here, not in the
  children.
- **The oracle layer is the binding constraint.** 391 oracle files;
  264 fail javac in 1.21.11 (67.5%), 289 of 648 drivers blocked.
  The measurement improved for the first time in this run (127 pass,
  up from 104), and `command_parse` now runs end-to-end with the
  probe compiling AND linking — but the runtime layer beyond
  command_parse and loot_fill is still unmeasured.
- **Where vanilla changed, the template is cleaner than its child.**
  The gamerule registry, the Time argument, `nautilus_one_cm`, the
  permissions package, the `game_rule` and `environment_attribute`
  registries — all arrived in 1.21.11 and the port matches them. The
  1.21.10 defects in those areas were vanilla changes, not port
  defects.
- **Fixes are a separate decision.** This sweep, like the six before
  it, is checks + triage only. The live majors across the seven
  ledgers — the loading stick, the palette reader, both loot engines,
  the worldgen cascade, the supports_* blocks, the startup blockers —
  are recorded, not fixed. The E6 re-base propagation (triage done,
  249 units, ~45 agent-days) and the water carry-over remain held.
  The loading-stick propagation to 1.21.11 and the template versions
  is a separate owner decision.

Standing items carried forward (parent actions, not this sweep):
the ledger corrections listed above; the 1.21.4 behaviour port (live,
87/548 at the last check); the 1.21.7 playtest defects (armour
rendering, shield direction, firework tooltip — fix in flight,
build-9 held for it).
