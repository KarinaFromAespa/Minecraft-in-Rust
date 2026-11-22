# The parity sweep: every block, entity, particle and fluid vs vanilla

Owner-directed, 2026-11-22. The owner, playing the 1.21.7 port, found two live
divergences the D1–D6 gate had missed — no trees at the spawn chunks, and water
placed at height not falling — and asked for a systematic verification that
every block, entity, particle, fluid and related behaviour does the same thing
as vanilla, not just spot checks. The sweep runs across all seven versions
that have vanilla reference worlds (see below), piloted on 1.21.7 — the
version the owner is playing. This document is the sweep's inventory,
coverage map, methodology, batch plan and pilot report. The methodology is
version-neutral; the counts are per-version (see §1.2 and §7).

### Reference worlds (ground truth for worldgen checks)

Vanilla worlds archived in `KarinaFromAespa/vanilla-world-mc` (local clone
`~/workspace/vanilla-world-mc`), created by the owner 2026-11-21. Seeds
verified from the world files; 26.x seeds live in
`data/minecraft/world_gen_settings.dat`, 1.21.x seeds in level.dat's
WorldGenSettings compound.

| Version | World (LevelName) | Seed | DataVersion | Overworld chunk regions |
|---|---|---|---|---|
| 1.21.7 | Test World (`New World`) | 8257987075890829262 | 4438 | 16 |
| 1.21.10 | New World (`New World`) | 6826278913115239597 | 4556 | 16 |
| 1.21.11 | New World (`New World`) | -76134417269081163 | 4671 | 15 |
| 26.1 | 26.1 (`26_1`) | -1600712240839185950 | 4786 | 15 |
| 26.1.1 | 26.1.1 (`26_1_1`) | 3231603902411641524 | 4788 | 15 |
| 26.1.2 | 26.1.2 (`26_1_2`) | 4656297851632877335 | 4790 | 15 |
| 26.2 | 26.2 World (`26_2 World`) | -1514548643533799951 | 4903 | 12 |

All seven worlds are overworld-only — no nether or end ground truth exists
in any of them (see §7).

These worlds are the reference for **worldgen** checks (method 3, §3):
paired chunk comparisons at the same seed. **Behaviour** checks (methods 1–2)
use the Java-oracle jars per version (§7) — the worlds do not carry behaviour
evidence beyond what their chunks show.

Companion docs: `docs/ROADMAP.md` (programme plan, §9 cost model),
`docs/DELTA-BAR.md` (the D1–D6 gate this sweep sits beside, not inside).

## 0. The one-paragraph finding

The programme already owns a 648-harness differential suite, but for 1.21.7
only 17 harnesses were ever run and only 2 passed; the other 631 drivers exist
in the tree and are unmeasured, and 9 of the 17 measured harnesses died on
Java-oracle compile errors against the 1.21.7 jars. The sweep's main job is
therefore *running* existing coverage and repairing its oracles — not writing
new checks from scratch. The genuinely uncovered areas are the 247 rank-3
ledger items with no harness at all (30% of the template's changed scope),
particles, fluid ticking and redstone ticking as measured categories, and the
~9,200 non-protocol unit tests, which no CI job runs. Nothing in D1–D6 asks
whether every block, entity, particle or fluid behaves as vanilla: the gate
is a delta audit (changed scope only, inherited answers in writing), and the
owner asked for a whole-version behavioural audit.

## 1. Inventory — the behaviour universe

### 1.1 1.21.7 (pilot version)

Counted 2026-11-22 from the vanilla 1.21.7 registry dump and the port's
prepared data (`mob-spawn/resources/`, cross-checked against
`.local/horizon-client/1.21.7/`). Data version 4438. Every multi-source count
agrees; the 5 shared JSON files are md5-identical between the resources copy
and the port's prepared data. Method: `len(registries.json[<registry>]
["entries"])` for synced registries; list/file counts for data-driven ones.
Full auditable write-up (since superseded by this section) was kept in the
working notes, not the repo.

### Headline counts

| Category | Count | Source |
|---|---|---|
| Block types | **1,105** | `registries.json` `minecraft:block` |
| Block states | **27,946** | 4 JSON dumps agree; 388 single-state blocks; largest: redstone_wire 1,296 |
| Block-entity types | **47** | `minecraft:block_entity_type` (173 distinct blocks carry one) |
| Entity types | **151** | 3 sources agree; 81 living mobs + 70 misc |
| Particle types | **114** | `minecraft:particle_type` |
| Fluid types | **5** | empty, flowing_water, water, flowing_lava, lava (16 water + 16 lava states, carried as block states) |
| Item types | **1,416** | `minecraft:item` + `items.json` |
| Biomes / enchantments / status effects / potions / damage types | 65 / 42 / 39 / 46 / 49 | data files + registries |
| Sound events | 1,728 | `minecraft:sound_event` (sweep checks wiring, not audio) |
| Structures / configured+placed features / loot tables / recipes / advancements / tags | 34 / 224+258 / 1,246 / 1,407 / 1,520 / 589 | data files, cross-checked vs prepared worldgen JSONs |

### Entity breakdown (vanilla `MobCategory`)

Monster 40, creature 32, water_ambient 4, water_creature 2,
underground_water_creature 1, axolotls 1, ambient 1, misc 70. The 70 misc
split into boats/rafts 20, minecarts 7, projectiles 20, item/XP 2, other 21 —
and vanilla puts iron_golem, snow_golem, villager and player in `misc`, so a
"mobs only" batch would lose them. 86 of 151 entity types carry attribute
sets.

### Other behaviour-bearing categories (sweep: yes)

Attributes 35, villager professions 15, POI types 21, game events 60,
menus 25, template pools 188, structure sets 20, configured carvers 4,
noises 60, density functions 35, noise settings 7, world presets 6, flat
level presets 9, data component types 96, command argument types 57 (matches
the 57-entry 1.21.7 list), AI brain registries (activities 26 / memory
modules 107 / sensors 28 / schedules 4), dimension types 4, chunk statuses
12, trial spawners 28, spawn placements 151 (all entity types covered).
Judged cosmetic/presentation only and OUT of the sweep: paintings 51, banner
patterns 43, trims 18+11, animal variants, instruments 8, jukebox songs 21,
entity models 356, mob animations 43, map decorations 35, creative tabs 14,
custom stats 76. PARTIAL: dialogs 3, advancements 1,520 (triggers only),
sound events (wiring only).

### Inventory caveats

- `block-behaviours.json` `items` holds only 965 entries, all BlockItem
  subclasses. The other **451 items** (food, tools, weapons, armour, spawn
  eggs, buckets, boats, potions, …) have no behaviour-class entry there —
  the sweep's item batches need another spec source for those 451
  (`items.json` data components) or accept the gap.
- `registries.json` `minecraft:block_type` (255 entries) has unverified
  provenance — not obviously a vanilla 1.21.7 synced registry. Do not size
  from it until confirmed against the vanilla jar or the exporter source.
- The port's runtime `.local` does not contain registries.json, items.json,
  entity-types.json, attributes.json, mob-data.json, entity-models.json or
  horizon-resources.json — those exist only in the resources copy. By design:
  the client receives registries from the server at login, so sweep registry
  counts rest on the resources copy.
- `worldgen/rules.json` is an empty dict — there is no standalone
  material-rule table; rules live inside noise settings / density functions.
- No biome/enchantment/damage-type/painting registry exists in
  registries.json — those are data-driven in 1.21.7, counted from files.

### 1.2 Other versions (per-version counts)

Counted 2026-11-22 with the same method as §1.1. 1.21.10, 1.21.11 and 26.1
from prepared data (cross-checked against the vanilla datagen dumps — all
headline counts agree exactly, which validates the fallback method).
26.1.1, 26.1.2 and 26.2 from the vanilla datagen dumps + jar datapacks
(marked †), because no prepared data exists for them (see §7). Data
versions: 1.21.10 = 4556, 1.21.11 = 4671, 26.1 = 4786, 26.1.1 = 4788,
26.1.2 = 4790, 26.2 = 4903.

| Category | 1.21.7 | 1.21.10 | 1.21.11 | 26.1 | 26.1.1† | 26.1.2† | 26.2† |
|---|---:|---:|---:|---:|---:|---:|---:|
| Block types | 1,105 | 1,166 | 1,166 | 1,168 | 1,168 | 1,168 | 1,196 |
| Block states | 27,946 | 29,671 | 29,671 | 29,873 | 29,873 | 29,873 | 32,366 |
| Block-entity types | 47 | 49 | 49 | 49 | 49 | 49 | 49 |
| Entity types | 151 | 153 | 157 | 157 | 157 | 157 | 158 |
| Particle types | 114 | 115 | 115 | 117 | 117 | 117 | 125 |
| Fluid types | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| Item types | 1,416 | 1,488 | 1,505 | 1,506 | 1,506 | 1,506 | 1,537 |
| Biomes | 65 | 65 | 65 | 65 | 65 | 65 | 66 |

Extended counts (same sources; † = fallback):

| Category | 1.21.7 | 1.21.10 | 1.21.11 | 26.1 | 26.1.1† | 26.1.2† | 26.2† |
|---|---:|---:|---:|---:|---:|---:|---:|
| Sound events | 1,728 | 1,771 | 1,838 | 1,902 | 1,902 | 1,902 | 1,968 |
| Status effects | 39 | 39 | 40 | 40 | 40 | 40 | 40 |
| Potions | 46 | 46 | 46 | 46 | 46 | 46 | 46 |
| Attributes | 35 | 35 | 35 | 35 | 35 | 35 | 40 |
| Enchantments | 42 | 42 | 43 | 43 | 43 | 43 | 43 |
| Damage types | 49 | 49 | 50 | 50 | 50 | 50 | 51 |
| Structures | 34 | 34 | 34 | 34 | 34 | 34 | 34 |
| Configured / placed features | 224/258 | 224/258 | 224/258 | 221/258 | 221/258 | 221/258 | 226/262 |
| Recipes | 1,407 | 1,461 | 1,470 | 1,515 | 1,515 | 1,515 | 1,585 |
| Loot tables | 1,246 | 1,320 | 1,324 | 1,326 | 1,326 | 1,326 | 1,355 |
| Advancements | 1,520 | 1,574 | 1,584 | 1,617 | 1,617 | 1,617 | 1,688 |
| Tags (all kinds) | 589 | 612 | 625 | 758 | 758 | 758 | 794 |
| Blocks w/ block entity | 173 | 201 | 201 | 201 | 201† | 201† | — |
| Entities w/ attributes | 86 | 88 | 92 | 92 | 92† | 92† | — |
| Items w/ behaviour class | 965 | 1,025 | 1,025 | 1,026 | — | — | — |

Notable deltas: 1.21.10→1.21.11 adds 4 entities (monster 40→44,
water_creature 2→3, creature 32→31) and 17 items, blocks unchanged.
**26.1.1 and 26.1.2 are registry-identical to 26.1** — genuine patch
releases, verified (datagen dumps byte-identical in every copy; jars are
distinct builds with world versions 4788/4790; protocol 775 and pack
versions unchanged; bundled datapacks content-identical). **26.2** adds the
cinnabar/sulfur block families (+28 blocks), the `sulfur_cube` entity, 8
geyser/sulfur particles, 5 attributes (35→40) and a 66th biome; its BE
types swap `bed` out for `potent_sulfur` (net still 49). The 451-item
behaviour-class gap from §1.1 persists in every version (463 without a
class in 1.21.10, 480 in 1.21.11 and 26.1).

Caveats for the † versions: the behaviour-class item split, entity
attribute sets, BE-block counts and entity MobCategory breakdowns need
prepared data (`block-behaviours.json`, `entity-types.json`,
`gameplay-facts.json`) and could not be counted — a prepare run produces
them. For 26.1.1/26.1.2 the 26.1 values are inferred by registry identity;
for 26.2 only the `sulfur_cube` delta is known and its category is
unverified. **Trap:** `26.2/pipeline/.local/datagen/` is 26.3's dump
(1,286 blocks), not 26.2's — 26.2's real datagen is at
`26.2/versions/26.2/datagen/reports/` (md5-verified distinct).

## 2. Coverage map — what is already verified, and what is not

Surveyed 2026-11-22 against the 1.21.7 tree, its pin and the behaviour
ledgers. "Measured" = run for 1.21.7 with a recorded result. "Drivers exist"
= a harness driver + (usually) a Java oracle exist in the tree but were never
run for this version.

### The 17-harness port-verify profile

The only published measurement for 1.21.7 is CI run 44441686979
(2026-11-19), recorded in `pins/1.21.7/profile.json`: **2 pass / 21 fail**
over 23 entries (17 harnesses + 6 datafix shards). Passing: block_properties
(block flags/solidity), lan (LAN discovery). Failing, by cause (from
`pins/1.21.7/status.json`): 9 of the 17 harnesses died on oracle javac
failures (chunk_generation, command_exec, component_patch, datafix — plus
its 6 shards — first_person, food, item_model, options, resource_pack),
5 real fails (command_parse, hotbar, light, save_roundtrip, sound_event),
1 harness Python bug (entity_components). All E6 siblings
(1.21.6/8/9) show the identical 2/23 — systematic oracle breakage at the era
boundary, not 1.21.7-specific parity. Even the 1.21.11 template's published
profile is only 5/23. The 1.21.7 pin is also D4-blocked: the profile is
stamped for d37f448e, not HEAD.

Correction to a standing note: the "4 red-but-non-blocking" set
(command_exec, hotbar, options, sound_event) is the **26.2** profile (19/23
pass), not 1.21.7's.

### The full suite: 648 drivers, 631 unmeasured

`versions/1.21.7/tools/test_*_parity.py`: **648** runnable drivers
(`parity.py` globs them), plus **391** Java oracles in
`mods/horizon/src/test/java/dev/horizonlab/parity/`. Each harness runs a Java
oracle (the version's own jars) and a Rust probe on the same inputs and
compares JSON. No full-sweep `state.json` exists for 1.21.7 anywhere. A warm
full sweep is ~970 harness-minutes (DELTA-BAR §9) — machine time, not agent
time; the agent cost is triage of what fails.

Driver counts by category (keyword match over the 648 names; overlaps exist):

| Category | Drivers | In the measured 17? |
|---|---|---|
| Entities / mobs | ~109 | entity_components only (fails, harness bug) |
| Worldgen / features | ~54 | chunk_generation only (oracle-broken) |
| Items / inventory / food | ~50 | item_model, food, hotbar, component_patch |
| Save / datafix / network | ~51 | save_roundtrip, datafix, lan |
| Block entities / containers | ~40 | none directly (save_roundtrip covers storage only) |
| Rendering / models | ~27 | item_model, first_person |
| Commands | ~22 | command_exec, command_parse |
| Fluids / water / lava | ~20 | **none** |
| UI / screens | ~19 | options only |
| Sound / music | ~16 | sound_event only |
| Redstone / pistons | ~10 | **none** |
| Particles | 9 | **none** |

Spot-checked oracles present in the 1.21.7 tree: VanillaFluidFlowOracle,
VanillaFluidBoxOracle, VanillaFluidOccupancyOracle, VanillaFluidShapeOracle,
VanillaEntityFluidOracle, VanillaLivingFluidOracle,
VanillaBlockParticlesOracle, VanillaEmitterParticlesOracle,
VanillaEntityParticlesOracle, VanillaItemParticlesOracle,
VanillaMarkerParticlesOracle, VanillaParticleProvidersOracle,
VanillaRedstoneCatalogOracle, VanillaRedstonePersistenceOracle,
VanillaClientRedstonePreviewOracle. Caveat: oracle *presence* is not oracle
*compilation* — how many of the 631 unmeasured drivers' oracles compile
against 1.21.7 is unknown until run.

### The behaviour-port ledgers

Ledgers are diff-driven, not behaviour-driven: each item is a set of changed
vanilla classes mapped to Rust files, ranked by whether a harness exists to
rerun (rank 2) or not (rank 3, "unproved"). Closure is counts only — no
per-harness pass/fail evidence in the ledger.

- **1.21.7's own ledger is nearly empty**: `pins/1.21.7/work-items.json` — 5
  items (3 rank-1, 1 rank-2, 1 rank-5), all closed, all version-facts /
  save-format / no-change verification. It covers essentially zero behaviour;
  rank-3+ "inherits the 1.21.8 delta's answers … unmeasured until the
  behaviour port reaches them."
- **1.21.11 template ledger** (the behavioural substance for the era): 815
  items, all closed — rank 1: 10, rank 2 (has harness): 492, rank 3
  (unproved, no harness): 247, rank 4 (new classes): 64, rank 5: 2.
  **322/815 items (40%) name no harness at all.** Rank-2 items reference 362
  distinct harnesses; most-referenced: mob_ai (107), trading (71),
  natural_spawn (61), entity_model (59), spawner (58), trial_spawner (56),
  vault (51), mob_attributes (42), chat_screen (33), finalize_spawn (33).
- **1.21.5 ledger**: 426 items — rank 2: 259, rank 3: 133, rank 4: 23. Same
  shape: ~31% unproved. **1.21.8 ledger**: 526 items — rank 2: 317, rank 3:
  163, rank 4: 35.
- Known ledger gap: item 760 — sign boards are not rendered, only sign text
  (no harness; fix on branch `fix/1.21.11-sign-boards`).

### In-tree Rust unit tests — 9,668 `#[test]`s, almost none gated

Static counts: horizon-client 4,385 (gameplay), horizon-client-render 2,139
(rendering/screens, incl. particles.rs 31 tests), horizon-world 562
(worldgen, fluids 11 total, redstone.rs 1 test), horizon-protocol 436 (packet
codecs — **the only crate D2 runs**), horizon-shaders 329, horizon-server 179,
horizon-port-pack 169, horizon-core 163, horizon-optifine 155,
horizon-client-input 128, horizon-items 111, horizon-client-core 83, all
others ~830. D2 runs only `cargo test -p horizon-protocol` plus `cargo
check` — the other ~9,200 tests have no CI record for 1.21.7 and are not
part of the gate. Caveat: unit tests check Rust code against itself/fixtures,
not against vanilla, except where they pin oracle-derived values.

### Coverage map by category

| Category | Existing check | Fraction covered for 1.21.7 |
|---|---|---|
| Blocks — definitions/flags | block_properties harness, green | ~all flags/solidity, measured |
| Blocks — behaviour (ticks, interaction, survival, updates) | ~30 block_* drivers; horizon-world unit tests | Drivers exist; **measured 0** |
| Block entities | ~40 drivers; save_roundtrip fills one of every type | Drivers exist; **measured ~0**; save_roundtrip really fails |
| Entities / mobs | ~109 drivers, deepest area | Drivers exist; **measured ~0**; the no-mobs bug shipped past all of them. Known gap (owner-reported 2026-11-22): undead mobs do not burn in daylight — a skeleton in direct sunlight does not catch fire where vanilla burns it. Fix in progress, branch `fix/1.21.7-undead-burn` (not re-diagnosed here; Batch 3 verifies the fix against the oracle). |
| Particles | 9 drivers + 7 oracles; 31 unit tests | **Uncovered as measured** — none in the 17 or the E6 D5 subset |
| Fluids | ~20 drivers + oracles; 11 unit tests | **Uncovered as measured** — none in the 17; the owner's water bug sits here |
| Items | ~50 drivers; 111 unit tests | 4 measured, all failing; rest unmeasured |
| Redstone | 10 drivers; 1 unit test | **Uncovered as measured**; D6 redstone checkbox has no playtest.md |
| Worldgen / features | ~54 drivers; ad-hoc beardifier/post-decorate checks outside the gate | chunk_generation measured but oracle-broken; rest unmeasured |
| Commands | 22 drivers; 52 unit tests | 2 measured, both failing; rest unmeasured |
| Rendering | ~27 drivers; 2,139 unit tests | 2 measured, both oracle-failing; sign boards have no harness |
| Sound | 16 drivers | 1 measured, really failing (missing sounds.json) |
| UI / screens | 19 drivers; heavy unit tests | 1 measured (options), oracle-failing |
| Save / datafix / network | datafix, save_roundtrip, lan, 432 protocol tests (D2, green) | lan + protocol green; datafix/save_roundtrip failing |

### Uncovered areas (the sweep's target list)

1. 631 of 648 harness drivers never run for 1.21.7.
2. Particles: no measured coverage (9 drivers exist, none run).
3. Fluid ticking/flow: no measured coverage (20 drivers exist, none run;
   unit tests thin).
4. Redstone ticking: no measured coverage (tick-by-tick circuit harness
   unrun; no playtest.md).
5. Mob AI/spawning: unmeasured (~109 drivers; the only measured one fails on
   a harness bug; the no-mobs bug shipped anyway).
6. Block-entity ticking: unmeasured (tickers/storage drivers unrun;
   save_roundtrip really fails).
7. Rank-3 gaps: 247/815 (30%) of the template's changed scope has no harness
   — closed by jar review, not by a run; 1.21.7 inherits them unmeasured.
8. Sign-board rendering: no harness (ledger item 760).
9. ~9,200 non-protocol unit tests: not gated or CI-recorded for 1.21.7.
10. No human playtest record — no `pins/1.21.7/playtest.md` (D6
    blocked/deferred); the owner's playtesting is the only human verification
    and has already found 5 bugs the gate missed (black sky, no mobs, spawn
    trees, water not falling, undead mobs not burning in daylight).
11. Oracle compilability for the 631 is unknown — expect a real
    oracle-porting tax before the drivers yield measurements.
12. Undead mobs do not burn in daylight (owner-reported 2026-11-22, in-game:
    a skeleton in direct sunlight during the day does not catch fire where
    vanilla burns it) — fix in progress on branch
    `fix/1.21.7-undead-burn`; Batch 3 (entities/mobs) verifies the fix
    against the oracle.

### Gate vs what the owner asked for

D1 checks the pin, six regenerated tables and the tag audit (dead-tag class
only — "not a missing gate or a wrong default"). D2 checks compilation and
protocol codecs. D3 checks closure of the *changed-scope* ledger — five
version-facts items for 1.21.7, everything else inherited "in writing". D4
checks datafix + save round-trip, D5 an 8-harness frozen subset, D6 one
signed playtest. Nothing in D1–D6 asks whether every block, entity, particle
or fluid behaves as vanilla. ROADMAP §P3 states the blind spot outright: "A
version can pass every harness and still diverge somewhere no harness
looks." The gate is blind exactly where the owner's five in-game findings
live — black sky, mob absence, spawn-chunk trees, falling water, undead
mobs not burning in daylight — because those are unmeasured drivers, rank-3
gaps, or human-visible behaviour with no automated check. The sweep is therefore a separate audit
that feeds the gate (new measurements, new harnesses, new ledger items), not
a replacement for it.

## 3. Methodology

### Check methods, in priority order

1. **Run the existing harness driver** (`python3 tools/parity.py <name>` in
   the version tree). The driver runs a Java oracle (the version's own jars)
   and a Rust probe on the same inputs and compares JSON. This is the
   primary method wherever a driver exists — 648 of them do. A "pass" is the
   driver green; a "gap" is a failing driver with the JSON diff as evidence.
2. **Ledger item pattern** for behaviour with no driver (the rank-3 class):
   one item per behaviour, triaged against the decompiled vanilla jar
   (named jars via the mappings front-end; `javap` for the logic), closed by
   a new Rust test pinning the oracle-derived value — or recorded as an
   accepted gap with cause + owner, per the gate's rule. A "pass" is the
   test green; the test becomes the harness for that behaviour.
3. **In-game probe** where automation cannot reach: a scripted scenario on a
   fixture server or the owner's laptop playtest (owner-initiated only —
   D6 stays deferred unless he starts one). A "pass" is paired numbers
   matching vanilla (same seed, same coordinates, same inputs). Used for
   render feel, sound wiring, and any behaviour whose only oracle is a
   running game.
4. **Data diff** for data-driven registries (recipes, loot tables,
   advancements, tags, enchantments, worldgen JSONs): byte-level comparison
   of the prepared data against the vanilla data files. A "pass" is a clean
   diff; a difference is either a port defect or a declared override with a
   reason (the D1 overrides pattern).

### Rules the sweep follows

- **Run before writing.** A driver that exists and is green settles its
  area; a driver that exists and fails is a gap with evidence already
  attached. New checks are written only where no driver exists.
- **Match the stage.** The port's `generate_proto` output is terrain-level
  (before structure placement/decoration); vanilla region files are
  post-decorate. Comparing them directly is valid only where structure
  templates contain no air (ancient city: 0 air) — trial chambers and
  villages carve air during placement, so a terrain-vs-post-decorate gap
  there is a measurement artifact, not a density bug (the −6.4
  "beardifier residual" of 2026-11-22).
- **Use the right ground truth.** The VM vanilla server world under
  `terrain2/vanilla/world` is the WRONG seed (−8343112210435233944) — all
  comparisons against it are invalid. The owner's Prism worlds in
  `vanilla-world-mc` are the correct ground truth for their seeds.
- **One item per behaviour, ledgered.** Sweep findings go into a per-version
  sweep ledger (`pins/<v>/sweep-items.json`, same shape as
  `work-items.json`: item, method, evidence, status) so the sweep's
  coverage is countable and the gate can read it later. Fixes are separate
  work, branched per the repo rules, and never bundled into the sweep
  branch.
- **Oracle failures are findings, not noise.** A driver that fails because
  its Java oracle does not compile against the version's jars is recorded
  as an oracle defect with the javac error, and counts toward Batch 0 —
  it is never silently skipped or marked as a parity pass.

### Oracle limitations (known)

- `VanillaStructureDensityOracle.java` imports the 26.x-only densityfunction
  API (`SamplerContext`/`DensityVolume`) and cannot compile against
  1.21.7/1.21.11 jars — that harness does not run for 1.21.7. Verify density
  behaviour against the named jar's bytecode directly instead, or via the
  chunk-generation comparison.
- 9 of the 17 measured harnesses died on javac errors against 1.21.7 jars.
  Oracle compilability for the 631 unmeasured drivers is unknown until run
  — Batch 0 exists to measure it.
- Categories with no oracle path yet: the 247 rank-3 items (no harness),
  sign-board rendering (no harness), and human-visible behaviour whose only
  oracle is a running game (render feel, some sound wiring). Those take
  methods 2–3 above.
- VM-local harness runs need `.local/minecraft/<v>/` populated (manifest +
  jars): the drivers' `verify_artifacts()` gate requires the pinned
  manifest sha1 from `research-lock.json`, and no worktree currently has
  the versioned directory — the pilot could not run its harness drivers
  for this reason and fell back to javap + unit tests (§6). Batch 0 must
  include this setup step, or VM-local driver runs stay blocked and the
  drivers only run on CI.

## 4. Batch plan

Batches are ordered by risk × evidence: live divergences first, then the
deepest unmeasured systems, then the long tail. Sizes are check items
(drivers to run + registry entries to verify), given for 1.21.7;
per-version sizes come from §1.2. Costs use the rates in §5.

| Batch | Scope | Size | Why this order |
|---|---|---|---|
| 0 — Oracle repair | Measure oracle compilability across all 648 drivers; repair the broken oracles | ~648 drivers to run; repairs TBD by measurement (17-sample suggests ~half may need work) | Prerequisite: without it, no driver yields a measurement. The 17-sample (9/17 javac-broken) is the only evidence; the true rate is measured, not assumed. |
| 1 — Fluids + falling blocks (PILOT) | Falling blocks, lava, water-source mechanics (falling water excluded — owned by `fix/1.21.7-water-flow`) | ~20 fluid drivers + falling-block checks; see §6 | The owner just found a live water bug here; smallest driver family, proves the method. |
| 2 — Redstone + pistons | Redstone ticking (tick-by-tick circuits), pistons, observers, comparators | 10 drivers + circuit probes | Highest-risk unmeasured system: tick-order-dependent, quasi-connectivity, push order. No measured coverage at all. |
| 3 — Entities / mobs | Mob AI, spawning, trading, attributes, projectiles, vehicles | ~109 drivers; 151 entity types | Deepest driver family; the no-mobs bug shipped past all of it. Spawning rules (151 placements) are data-checked. |
| 4 — Block entities + containers | BE ticking/storage/render, menus, crafting/furnace/brewing logic | ~40 drivers; 47 BE types; 25 menus | save_roundtrip (the only measured touchpoint) really fails; tickers unrun. |
| 5 — Block behaviour | Block ticks, interaction, survival, support, updates, breaking | ~30 drivers; 1,105 blocks grouped by behaviour class (class chains in `block-behaviours.json`) | Largest registry; grouped by Java class chain so 16 colours of one block are one check, not 16. |
| 6 — Worldgen / features | Chunk generation, biomes, features, structures, carvers, density | ~54 drivers; 482 features; 34 structures; 65 biomes | The spawn-tree divergence lives here (being fixed separately); the feature catalogue is otherwise unverified. |
| 7 — Items | Item use, food, tools, armour, buckets, spawn eggs, components | ~50 drivers; 1,416 items (451 non-block items need a spec source — see §1) | The 451-item behaviour-class gap must be resolved first (items.json components as spec). |
| 8 — Particles | All 114 particle types, emitters, providers | 9 drivers; 114 types | Small, self-contained, zero measured coverage. |
| 9 — Commands | Parsing, execution, selectors, functions, scoreboard | 22 drivers; 57 argument types | 2 measured, both failing (1 real, 1 oracle). |
| 10 — Rendering, sound, UI | Models, block-entity rendering, screens, sound wiring, options | ~27 + 16 + 19 drivers; 1,728 sound events (wiring only) | Includes the sign-board gap (no harness — needs a new check) and the missing sounds.json failure. |
| 11 — Rank-3 gaps | The 247 template items with no harness: triage for new harnesses or in-game probes | 247 items | The long tail of "closed by jar review" — each needs a real check or a recorded accepted gap. |
| 12 — Data-driven registries | Recipes, loot tables, advancements, tags, enchantments, effects, damage types | 1,407 + 1,246 + 1,520 + 589 + 42 + 39 + 49 entries | Mostly mechanical data diffs (method 4); behaviour spot-checks where a diff disagrees. |

Batch 0 runs first and unblocks all the others; batches 1–2 can start in
parallel with it (their drivers' oracles are individually verifiable, as the
pilot shows). Batches 3–12 are ordered but independent once Batch 0 lands —
they can run as parallel streams, one agent per batch, each with its own
workdir and sweep-ledger slice.

### Per-version execution

Every batch runs **per version** — the sweep is seven sweeps sharing one
method, not one sweep applied seven times. Per version, the batch sequence
is: (a) readiness gate — prepared data + oracle jars present (see §7; a
version missing either waits for a prepare run before its Batch 0); (b)
Batch 0 (oracle repair for that version's jars); (c) batches 1–12 in the
order above, against that version's drivers and registries. The pilot
(§6) runs on 1.21.7 only and proves the method; the other six versions
reuse its checklists.

Which reference applies where, per batch:

| Batch family | Reference for the check |
|---|---|
| Worldgen / features (6) | The version's reference world (§0): paired chunk comparisons at that world's seed. The 1.21.7 world is also paired with the port's Test 1 world at the same seed. |
| Everything else (1–5, 7–11) | The version's Java-oracle jars (named jars where they exist; see §7 for per-version availability). The reference worlds do not carry behaviour evidence. |
| Data-driven registries (12) | The vanilla data files from the version's own jars/prepared data, byte-compared against the port's prepared data. |

A version's sweep ledger (`pins/<v>/sweep-items.json`) records per-batch
status for that version only — no cross-version inheritance. A batch that
passes on 1.21.7 says nothing about 26.2; the registries, the drivers and
the oracles all differ.

## 5. Cost estimate

What is measured vs estimated, stated plainly:

- **Measured (programme records, Nov 15 correction from the committed
  sample):** 47 of 108 behaviour items changed (43.5%); **92.5 minutes per
  changed item; ~12 items/day per agent.** This rate is for FIX work
  (diagnose + change + test), not for checks. It is not in ROADMAP.md — the
  repo's planning figure there (§9, A2) is 30 items/port-day, which is an
  estimate that counts unchanged/no-op closes; the Nov 15 figure is the
  measured rate for items that actually change code.
- **Measured (DELTA-BAR §9):** a warm full sweep is ~970 harness-minutes of
  machine time; ~1.5 min per harness warm, ×8–10 cold.
- **Estimated (this plan):** triage at ~15 min per driver (read the failure,
  classify oracle-bug vs parity-gap vs harness-bug); new-harness design at
  ~1 h per rank-3 item on average (wide range — some are jar-review-only,
  some need a driver); data-diff triage at ~5 min per disagreeing entry.

On those rates, the 1.21.7 sweep (checks + triage, EXCLUDING fixes) is
roughly:

| Cost | Basis | Estimate |
|---|---|---|
| Run all 648 drivers | 970 harness-minutes warm (machine) | ~16 runner-hours, parallelisable across the CI slice matrix; ~1 agent-day of driving/triage setup |
| Triage 648 driver results | ~15 min/driver | ~160 agent-hours ≈ 9 agent-days |
| Batch 0 oracle repair | Unknown until measured; 17-sample suggests ~half the drivers' oracles may need work; ROADMAP A3 puts era-break oracle repair at 20–80% of the corpus | **20–60 agent-days — the least-evidenced number in this plan; re-estimate after Batch 0 measures the true javac-failure rate** |
| Rank-3 new harnesses (247) | ~1 h/item average | ~14 agent-days |
| Data-diff triage (Batch 12) | ~5 min/disagreeing entry; disagreement rate unknown, assume 5% | ~3 agent-days |
| **Sweep total (checks + triage, no fixes)** | | **~50–90 agent-days for 1.21.7** |

Fixes are extra and gap-count-dependent: each gap found costs 92.5 min at
the measured fix rate. The pilot (§6) gives the first sample: 6 defects in
a 45-check family (~9 agent-hours for that family's fixes if each defect is
one changed item). If that rate held across the sweep, the fix bill would
dominate the check bill; if the drivers are mostly green once their
oracles compile, it would not. **Do not budget the fix bill until Batch 0 +
more families bound the gap rate.**

Per-version reuse and the seven-version total: see §7.

## 6. Pilot — fluids + falling blocks (1.21.7)

Run 2026-11-22, checks only — no fixes, no commits. Scope: falling blocks,
lava, water-source mechanics. Excluded by scope (owned by
`fix/1.21.7-water-flow`): vertical water fall and horizontal water flow
spread — not checked as water behaviour, not counted. Full gap list with
per-check evidence is in the working notes, not the repo.

### Tally

**45 checks: 33 PASS, 12 GAP checks across 6 distinct defects.**
Family: 25 falling blocks (sand, red sand, gravel, 3 anvils, dragon egg, 16
concrete powders, suspicious sand/gravel); water + lava source/flowing;
waterlogging acceptance across all 1,105 blocks (355 waterloggable, zero
mismatches); bubble columns (soul sand / magma).

### The systemic pattern (read first)

Four of the six defects share one root cause: the 1.21.7 port carries 26.x
tag-based logic for lookups whose tags do not exist in vanilla 1.21.7, and
every such lookup silently degrades to empty/false (block tags:
`unwrap_or_default` at `level/data.rs:397`; fluid tags: empty mask at
`level/data.rs:824-835`). Verified against the vanilla 1.21.7 jar itself (it
bundles `data/minecraft/tags/`): `speleothems`, `washed_away_by_fluids`,
`blocks_lava_fire_spread`, `enables_bubble_column_push_up`,
`enables_bubble_column_drag_down` do not exist as block tags, and
`bubble_column_can_occupy` does not exist as a fluid tag (only `lava.json`,
`water.json`). The prepared `.local` data is faithful to the jar on this
point — the port code, not the data, is the 26.x carry-over. This is the
same bug class as the D1 tag audit's dead tags, but in the other direction:
code that *reads* a tag that never existed in this version, instead of code
that reads a tag from a newer version. Batch 0's oracle repair does not
catch it; the sweep's driver runs do.

### The six gaps

- **G5 — NO fluid spreading at all, water or lava (MAJOR).** Vanilla
  `FlowingFluid.canHoldAnyFluid` is hardcoded (air passes); the port uses
  the absent tag `washed_away_by_fluids`, so only liquid containers pass
  and `spread` never fires. Affects 5 checks: effective lava flow, stone
  via spread, cobblestone via spread, 2×2 source, waterlogged-water
  spreading out. RAN: `lava_meets_water` fails (air where cobblestone
  should be), plus 6 more fluid tests fail. The spread numbers themselves
  match vanilla — they just never execute. **Cross-reference: this same
  root cause covers the owner's water-fall symptom owned by
  `fix/1.21.7-water-flow` — coordinate so it is not fixed twice.**
- **G1 — Bubble columns never form; existing ones decay to water (MAJOR).**
  Formation/survival key on absent bubble-column tags; no SoulSand/Magma
  hook exists in `level/`. Vanilla trigger confirmed by javap. The
  entity-motion side is correct but moot while no column can exist.
- **G6 — Dripstone never transfers fluid to cauldrons.** All stalactite
  predicates require the absent tag `minecraft:speleothems`. Constants and
  delay match vanilla but never run. RAN: `dripstone_fills_cauldrons`
  fails (`cauldron[]` vs `water_cauldron[level=1]`). Same cause also kills
  falling stalactites (out of scope, noted).
- **G2 — Concrete powder falling into water does not solidify on contact.**
  Vanilla runs the landing branch at the first water cell (javap); the
  port always passes `stuck_in_water=false`, so powder sinks to the
  lakebed. Landing/neighbour solidification otherwise exact.
- **G3 — Lava random-tick fire spread walks through solid blocks.** Vanilla
  stops at the first `blocksMotion()` block; the port stops only for the
  absent `blocks_lava_fire_spread` tag. The walk also perturbs the RNG
  stream after the first motion-blocking block.
- **G4 — Lava ignition ignores doFireTick.** The port carries the 26.3
  gamerule table with no `doFireTick`; with it off, vanilla lava never
  ignites but the port still does near players. A gamerule-table gap, not
  lava-specific — it affects fire spread generally and needs a
  version-table decision, not a lava-local fix.

### Tests run

`cargo test -p horizon-world --lib level::tests::` (dev profile, `-j 1`):
**77 passed, 15 failed.** 8 in-scope failures, all explained by G5/G6. 7
out-of-scope failures (copper chest, crops, doors/beds, levers, pistons,
redstone, snowy grass) — **not diagnosed**; hypothesis only that the same
tag pattern may extend to them. `snowy_biomes_freeze_and_snow` passed
(199 s). The release-profile test build was OOM-killed under 3-stream load
— release-profile test builds under load are fragile; use dev profile for
sweep checks.

### Method notes / limitations

- The Java-oracle harness drivers (`test_falling_block_parity.py`,
  `test_block_update_parity.py`) could NOT be run: no worktree has
  `.local/minecraft/1.21.7/` (manifest + jars), and `verify_artifacts()`
  requires the pinned manifest sha1 from `research-lock.json`. The named
  jars exist elsewhere but the harness gate cannot be satisfied without
  `lab.py prepare` (network download + writes into the shared symlinked
  `.local`), out of scope for a checks-only pilot. Instead every vanilla
  method in scope was verified directly with `javap -c` on the named
  1.21.7 server jar, plus exhaustive jar scans (waterlogging 355/355;
  falling classes 23/23, 2/2, 1/1, 16/16, 3/3; tag inventory 196 block /
  2 fluid tags), plus Rust unit tests. No fixture-server runs — the
  javap + unit-test evidence settled every check. **Consequence for the
  sweep: Batch 0 must include the `.local/minecraft/<v>/` setup step, or
  VM-local harness runs stay blocked and the drivers only run on CI.**
- `IGNITED_BY_LAVA` flag table: mechanism-verified, not exhaustively
  diffed. Torch/fence/rail landing breaks are mechanism-covered (generic
  `canBeReplaced`), not harness-covered.
- Adjacent observation (not a pilot finding): prepared `.local` renames
  four 1.21.7 tool tags to their 26.x `mineable/*` names — affects
  mining/tool code, not this family.

### Fix order (suggested, not done)

G5 → G1 → G6 → G2 → G3 → G4. G5 first: it breaks all water/lava flow, and
its fix (a 1.21.7 `canHoldAnyFluid` per the bytecode, not a tag) is also
the likely root cause of the sibling water-fall bug — coordinate with
`fix/1.21.7-water-flow`. G1/G3/G6 share one fix shape (replace the tag
lookup with the 1.21.7 hardcoded rule). G4 needs the version-table decision.

### What the pilot says about the fix bill

6 defects in a 45-check family. At the measured 92.5 min per changed item
(§5), this family's fixes cost ~9 agent-hours if each defect is one changed
item — a first sample, not a projection: do not extrapolate it to 648
drivers until more families are measured.

## 7. Multi-version execution

### Readiness per version

Surveyed 2026-11-22 (prepared data, oracle jars, driver counts via git
ls-tree on origin/main confirmed on disk, pin status, world seeds verified
from the world files).

| Version | Prepared data | Oracle jars | Drivers / oracles | Pin on main | Reference world | Verdict |
|---|---|---|---|---|---|---|
| 1.21.7 | YES | named + obfuscated | 648 / 391 | YES (2/23, d37f448e) | YES, 16 regions | **START NOW** |
| 1.21.10 | YES (2 copies) | named + obfuscated | 648 / 391 | NO | YES, 16 regions | **START NOW** |
| 1.21.11 | YES | named + obfuscated | 648 / 391 | NO (pin on `behaviour/1.21.11`) | YES, 15 regions | **START NOW** |
| 26.1 | YES | obfuscated only | 648 / 391 | NO | YES, 15 regions | **START NOW*** |
| 26.1.1 | NO | obfuscated only | 648 / 391 | NO | YES, 15 regions | **NEEDS PREPARE** |
| 26.1.2 | NO | obfuscated only | 648 / 391 | NO | YES, 15 regions | **NEEDS PREPARE** |
| 26.2 | NO (deleted) | obfuscated only | 656 / 391 | YES (19/23, 8f5f13d6) | YES, 12 regions | **NEEDS PREPARE** |

\* 26.1 has everything except named jars. **No named jar exists on the VM
for any 26.x version** (searched all 26.x workdirs; the mapping-frontend
holds named jars for 1.21.x only). The remap tooling is present
(tiny-remapper, mapping-io, asm, vineflower), so named 26.x jars are
producible by a remap run — follow-up work, not done here. Note 26.2's CI
profile is 19/23 green, so the oracle-compile path for 26.x works in CI;
the VM sweep needs that jar setup reproduced before Batch 0 can run
drivers for a 26.x version.

"NEEDS PREPARE" means a `horizon prepare` run for that version (the jars
are already on the VM, so no download is needed for the jars themselves;
prepare runs datagen + the exporters and produces
`.local/horizon-client/<v>/`). The sweep's Rust probes read prepared data
at runtime, so prepare is a genuine prerequisite for *running* drivers,
not just for counting.

Two scope notes: (1) **all seven reference worlds are overworld-only**
(12–16 chunk regions each) — no nether or end ground truth exists in
`vanilla-world-mc`, so Batch 6 (worldgen) has no vanilla comparison for
the Nether and End; that needs fresh vanilla exploration or a
fixture-server comparison. (2) **26.2 has 656 drivers**, not 648 (+12
feature drivers, −4 removed, net +8 — mostly vegetation/feature parity);
every other version has 648.

### Rollout order

1. **1.21.7** — pilot version. Everything present: prepared data, named
   jars, reference world, owner playing it. Proves the method.
2. **1.21.11** — the E6 template: 815-item behaviour ledger, the era's
   reference port. Its pin/profile lands with the behaviour-branch merge.
3. **26.2** — the most-measured version (19/23 profile) and the E7 line's
   completed port. Needs a prepare run first (`.local` deleted).
4. **1.21.10** — prepared data and named jars exist; a mechanical publish,
   never profiled — the sweep is its first measurement.
5. **26.1, then 26.1.1, 26.1.2** — mechanical publishes, no pins; the 26.1
   line is the E7 era base, so its oracle repair also serves the era.
   26.1 first: it has prepared data, while 26.1.1/26.1.2 need a prepare
   run; the patches are registry-identical to 26.1, so their sweeps are
   mostly re-runs of its drivers.

### What carries across versions

- The counting method (§1) is reusable; the numbers are not — every
  version's inventory must be re-run (or at least re-verified) from that
  version's prepared data.
- The driver set and the oracle repairs (Batch 0) carry across versions
  within an era; era breaks need the A3-style oracle repair the ROADMAP
  already budgets.
- The sweep ledger shape (`pins/<v>/sweep-items.json`) is version-neutral;
  each version gets its own ledger.
- The pilot's checklists (per-family: what to run, what "pass" looks like)
  become the batch templates for the other families and the other versions.

### Cost across versions

§5 estimates ~50–90 agent-days for 1.21.7 (checks + triage, no fixes).
A second version reuses the method, the driver set and the within-era
oracle repairs, so its cost is Batch 0 (delta oracle repair only) +
re-running the drivers + its own rank-3 triage — estimated at roughly a
third of the first version's cost, unmeasured. Era breaks (E6→E7) pay the
A3 oracle-repair cost again. The seven-version total is therefore roughly
2–3× the single-version cost, not 7× — but the multiplier is an estimate
until the second version is timed.

## 8. What this document does not do

- It does not change the D1–D6 gate. The sweep feeds the gate (new
  measurements, new harnesses, new ledger items); a gate change needs the
  owner's explicit word, per the programme's standing rule.
- It does not fix anything. Fixes are follow-up work on their own branches,
  sequenced by the main session — including the live divergences found
  during playtesting: the two that triggered this sweep (spawn trees:
  `fix/1.21.7-trees`; falling water: `fix/1.21.7-water-flow`) and undead
  mobs not burning in daylight (`fix/1.21.7-undead-burn`, reported after
  the sweep started).
- It does not claim a version is verified. A version is sweep-complete when
  its sweep ledger shows every batch measured — that is a future state, not
  this document.
