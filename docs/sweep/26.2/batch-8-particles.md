# Batch 8 — Particles (26.2 parity sweep)

**Version:** 26.2 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.2`
(workdir `~/workspace/mcir-work/sweep-26.2`, synced to main `732c800f`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 8 — all 125 particle types,
emitters, providers; 9 drivers claimed by the plan, 8 found in the tree
(see §5).

**Status: DRAFT — not committed.** Checks + triage only, per the batch
brief: no fixes, no edits under `versions/26.2/`, no cargo builds, no
commits. Everything below is static analysis: `javap` on the named 26.2
server-game jar (`26.1-adapters/versions/26.2/jars/server-game.jar`) and
the Mojang-mapped 26.2 client jar (`leakage-audit/jars/26.2-client.jar`),
direct class-file parsing of `Blocks.class` where `javap` would be too
slow, source comparison against `versions/26.2/crates/`, and data checks
against the 26.2 datagen reports
(`~/workspace/mcir-work/26.2/versions/26.2/datagen/reports/`). No driver
runs, no cargo tests, no prepared 26.2 data exists yet. The coordinator
commits these deliverables.

**Coordinator note (2026-11-24):** this batch ran before Batch 0 landed
and triaged oracles by import resolution only. Its oracle claims were
reconciled against the measured Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`) — see §4: one oracle defect
(B8-O01) was added that the import-level check could not see, and the
driver table in §5 was corrected to the measured verdicts. `horizon-prepare`
for 26.2 also finished after this batch ran (2048.7 s, EXIT 0);
`.local/horizon-client/26.2/` now holds the prepared data (203 MB), so the
data precondition for the driver runs in §8 is met — the runs themselves
remain follow-up work. Check-tally arithmetic: the §3 subsections sum to
281 checks / 276 pass; the 283 / 278 tally counts the two §3 "Other
passes" items (elder-guardian apparition re-implementation, bubble-column
animate arm) as checks 282–283.

**Headline:** all three 1.21.7 Batch 8 defects recur in 26.2 unchanged —
the 26.3-derived port still lacks the `elder_guardian` variant (B8-G01),
`hub_particle` still has no `Flash` arm (B8-G02), and spruce leaves still
drop no falling leaves (B8-G03). Two 26.2 differences from the 1.21.7 run
matter: the providers driver no longer sends `elder_guardian` cases (so
B8-G01 does not block it — it silently skips the type instead), and both
particle drivers carry 26.3-only entries (poplar leaves) that cannot run
against the 26.2 oracle (B8-H02, B8-H03).

## Tally

- **Behaviour checks: 283 — 278 pass, 5 gap** (across 3 defect items,
  B8-G01–B8-G03, all minor).
- **Oracle defects: 1** (B8-O01 — coordinator reconciliation, §4:
  `VanillaMarkerParticlesOracle` fails all three classpaths on the
  LocalPlayer constructor, Batch 0 G2; blocks the marker_particles
  driver). The other six particle oracles pass on their real classpath
  (measured, not import-level).
- **Needs-harness items: 4** (B8-H01–B8-H04, §1): driver-coverage gaps,
  not behaviour gaps.
- **Drivers: 0 run, 8 inventoried** (7 particle drivers + 1
  particle-adjacent effect-sync driver). 2 of the 8 would fail on run for
  identified reasons (B8-O01 oracle-blocks marker_particles — and B8-G03
  would fail its spruce cases once the oracle compiles; B8-H02 blocks
  particle_providers); the other 6 have compiling oracles (measured)
  but are unrun, not passing.
- **Unit tests:** 31 `#[test]`s in `particles.rs` exist (23 vanilla + 8
  mod-particle); unrun, blocked.

## 1. Defects

### B8-G01 — `elder_guardian` has no port implementation at all (MINOR, NEW, recurs from 1.21.7)

**Checks failed (3):** per-type end-to-end coverage for `elder_guardian`;
its `override_limiter` flag; the providers-driver kind set (which omits
it — see below).

**Evidence (jar).** Vanilla 26.2 registers `ELDER_GUARDIAN` as a
`SimpleParticleType` (`ParticleTypes.<clinit>`, wire id 24,
`overrideLimiter=true`) and registers a provider for it in
`ParticleResources.registerProviders()` — a dedicated
`ElderGuardianParticle$Provider` (26.2 has its own
`ElderGuardianParticle` class; 1.21.7 used the shared
`MobAppearanceParticle` provider). `ClientPacketListener.handleGameEvent`
spawns it at the player's position on game event
`GUARDIAN_ELDER_EFFECT` (`addParticle(ELDER_GUARDIAN, player.x/y/z, 0, 0,
0)`, bytecode offset 536), plus the curse sound.

**The port.** `ParticleType` (`horizon-client-core/src/particle_type.rs`)
has 127 variants and no `ElderGuardian`. Consequences, all verified in
source:

- `received_particle` (`game/lan_particles.rs:460`, `from_name` at :482)
  resolves a server-sent particle by name and returns `None` for
  `elder_guardian` — a `LevelParticles` packet for it is silently
  dropped.
- `/particle minecraft:elder_guardian` fails with `particle.notFound`
  (`game/commands/world_cmds.rs:770`, `from_name` returns `None`);
  vanilla accepts it.
- `override_limiter()` has no entry for it (32 vanilla limiter types vs
  31 Rust entries; the only 26.2 miss is `elder_guardian` — no Rust
  extras this time, since the geyser entries are real in 26.2).
- The providers driver does **not** send `elder_guardian` cases in the
  26.2 tree (its `SIMPLE` list omits the type entirely), so unlike
  1.21.7 — where the driver sent 12 cases and the probe printed `no
  provider` — the driver is not blocked by this defect; it silently
  skips the type. The probe would still print `no provider` if it ever
  saw one (`particle_providers_probe.rs:40`, `from_name` → `None`;
  :184).

**Mitigation (why minor).** The one in-game path that matters — the elder
guardian's curse apparition — is re-implemented outside the particle
engine, same as in 1.21.7: game event 10 sets `elder_apparition`
(`game/remote.rs:5040-5043`, `game/creatures.rs:4160`) and
`elder_apparition_geometry` (`game/mob_blocks.rs:79`) draws the ghostly
guardian in front of the camera with the vanilla 30-tick lifetime
(`APPARITION_TICKS = 30`, `mob_blocks.rs:70`). So the apparition works;
what is missing is the particle-type plumbing (`/particle`, server
packets). Visual fidelity of the re-implementation against vanilla's
`ElderGuardianParticle` is unverified statically — needs the driver run
or an in-game look.

### B8-G02 — `/particle minecraft:flash` sends a malformed packet to LAN guests (MINOR, NEW, recurs from 1.21.7)

**Checks failed (1):** the `Flash` arm of `hub_particle`.

**Evidence.** `Particle::read` (`horizon-protocol/src/entity_data.rs:215`)
decodes wire id 49 (`flash`, verified against the 26.2 registry) as
`Color(i32)` — a flash particle on the wire always carries a colour. But
`hub_particle` (`game/lan_particles.rs:402`), which builds the packet a
LAN host sends, has no `Flash` arm in its `Color` match (`EntityEffect |
TintedLeaves` only, :431), so a flash particle is built with `data:
None`, and `Particle::write` — which writes data by presence, not by id
— emits the id with no colour bytes. Any guest reading id 49 then
consumes the next 4 bytes of the stream as the colour and continues
misaligned: packet desync (and likely a disconnect or a dropped
connection) for every guest in range.

**Reachability (narrow — why minor).** The only path that sends `Flash`
through `hub_particle` is `/particle minecraft:flash`: `cmd_particle`
builds `ParticleOptions::simple(Flash)` (no colour — vanilla's command
requires the colour option, the port does not; `world_cmds.rs:881`
catch-all) and calls `network_particle` (:935). Everywhere else `Flash`
is created client-side only (the firework explosion flash,
`particles.rs:3739`, via `self.create(...)`, never sent over the
network). The local player is unaffected (draws directly, never reads
the packet). So the trigger is exactly: a LAN host runs `/particle
minecraft:flash` with guests in range.

### B8-G03 — Spruce leaves never drop falling-leaf particles (MINOR, NEW, recurs from 1.21.7, driver-blocking)

**Checks failed (1):** the spruce entry of the per-block leaves check.

**Evidence (jar).** Vanilla 26.2 registers `SPRUCE_LEAVES` as a
`TintedParticleLeavesBlock` with `leafParticleChance = 0.01` — verified
by parsing `Blocks.class` directly: `BlockItemIds.SPRUCE_LEAVES` +
`lambda$static$27` constructs `TintedParticleLeavesBlock(0.01F, …)`.
`LeavesBlock.animateTick` calls `makeFallingLeavesParticles`, which
rolls `nextFloat() < leafParticleChance` and spawns the falling leaf
unless the block below is face-full (javap). So vanilla spruce drops
tinted leaves at 1% per animateTick, like oak.

**The port.** `classify` (`animate_tick.rs:149`) maps `"spruce_leaves"`
to `Animate::Leaves(None, None)` — no falling particle at all. Every
other leaf block matches vanilla (see §3, leaves table).

**Driver note.** The marker driver's own zoo comment says "the spruce
that never falls" — the comment encodes the same wrong belief, but the
driver compares against the real vanilla oracle, so the driver is
correct as a check and **would fail on the spruce cases** (oracle emits
tinted leaves, probe emits none) once B8-O01 is repaired — the driver is
currently oracle-blocked, not merely unrun. The comment needs correcting
when the fix lands; the driver needs no change.

### Needs-harness items (driver coverage, not behaviour gaps)

- **B8-H01 — providers driver omits `vibration`.** The port implements
  vibration end to end (`Particle::read` id 55, `hub_particle` arm,
  `make()` arm, `ParticleOptions::vibration`), and the oracle's
  `options()` handles it (`VanillaParticleProvidersOracle.java`, the
  `VIBRATION` branch), but the driver's kind set never sends a vibration
  case — so the vibration provider's numerics have no driver coverage.
  Add vibration cases (with a target + duration) to
  `test_particle_providers_parity.py`.
- **B8-H02 — providers driver includes the 3 poplar leaves
  (26.3-only).** The driver's `SIMPLE` list carries
  `red/orange/yellow_poplar_leaves`, which are not in the 26.2 registry.
  The oracle resolves kinds by name and throws
  `AssertionError("unknown particle " + id)` for an unregistered type
  (`VanillaParticleProvidersOracle.java:131`), so the driver cannot run
  against the 26.2 oracle as-is: it would die on the first poplar case
  (12 cases per kind). Trim the 3 kinds for 26.2.
- **B8-H03 — marker driver zoo includes `red_poplar_leaves`
  (26.3-only).** The zoo (`test_marker_particles_parity.py`,
  `zoo_states()`) sends `red_poplar_leaves[distance=3,...]` cases, but
  no such block exists in 26.2 — the oracle cannot build that world.
  The port is correct to have no classify arm for it (26.2 has no
  poplar trees; `animate_tick.rs:40-41` says so). Trim the zoo case for
  26.2.
- **B8-H04 — entity driver omits live 26.2 entities `zombie_nautilus`
  and `camel_husk`.** Both are real 26.2 entity types and both inherit
  the 6/7/18 bursts (`ZombieNautilus extends AbstractNautilus extends
  TamableAnimal`; `CamelHusk extends Camel extends AbstractHorse`), and
  the port's `entity_event_burst` covers them via its tamable/horse
  sets — but the driver's `EVENTS` table has no entries for them (6
  untested pairs). Add them to `test_entity_particles_parity.py`.

## 2. Cross-references (not re-minted)

- **G1 (pilot) — bubble columns never form.** The client-side animate arm
  for bubble columns exists and is correct (`Animate::BubbleColumn`,
  `animate_tick.rs:1653` — `BubbleColumnUp` particles + whirlpool/upwards
  ambient sounds), but it can never trigger in-game because no
  `bubble_column` block states ever form server-side (G1, 1.21.7 pilot;
  26.2 server-side status is Batch 5's scope, not verified here). No new
  defect: the particle-visible consequence of G1, not a particle defect.
- **G6 (pilot) — dripstone never transfers fluid to cauldrons.**
  Server-side only; 26.2 server-side status is Batch 5's scope. The
  *client-side* drip particles are unaffected: vanilla 26.2's
  `PointedDripstoneBlock.animateTick` rolls 0.12 and climbs same-block
  stalactite via `findRootBlock` (`state.is(state.getBlock())` — not
  tag-based, despite `SpeleothemBlock` using the `speleothems` tag for
  growth/falling), and the port's `dripstone_drip`
  (`animate_tick.rs:1760`) matches it by name, with the same 0.12 roll
  and 0.02 cauldron rule. The marker driver's scope note about binding
  "the speleothem tag the stalactite branch needs" is correct for 26.2:
  the tag exists here (`pointed_dripstone` + `sulfur_spike`), unlike
  1.21.7 where it was absent.
- **B12-G04 — 26.x-only effect in the MobEffects table.** Same
  table-leak *pattern* as the 3 extra 26.3 poplar-leaf types in the Rust
  `ParticleType` enum (see §3), but the particle extras are unreachable
  — ids are name-resolved from the server's registry, and a 26.2 server
  never sends those names — so they are not minted as a defect. Dead
  table entries, not a behaviour gap. (Note: in 26.2 the enum carries
  only 3 extras, vs 14 in 1.21.7 — the geyser/sulfur/copper-flame/
  pause/reset types are real in 26.2.)

## 3. What passed (per area)

### Registry and type coverage (128 checks: 126 pass, 2 gap)

- 125 particle types: `registries.json` count == jar `ParticleTypes`
  `<clinit>` registration count (125), and the `<clinit>` order equals
  the `protocol_id` of every type (verified by parsing the bytecode).
- Engine `make()` (`particles.rs:2167`) has an outer match arm for all
  127 `ParticleType` variants (no outer catch-all — the `_ =>` arms in
  the body are all nested matches) — every enum variant has a provider
  implementation. The only engine-coverage gap is the missing
  `elder_guardian` variant itself (B8-G01).
- Vanilla providers: `ParticleResources.registerProviders()` in the 26.2
  client jar registers a provider for all 125 types, including
  `ELDER_GUARDIAN` (`ElderGuardianParticle$Provider`). (Field-name
  quirk: the fields are `TRIAL_SPAWNER_DETECTED_PLAYER(_OMINOUS)` but
  register under the names `trial_spawner_detection(_ominous)`.)
- The 3 Rust-enum extras (`red/orange/yellow_poplar_leaves`) are absent
  from the 26.2 jar's `ParticleTypes` — confirmed 26.3-only,
  unreachable from a 26.2 server.
- Gap: providers-driver kind set (126 kinds) misses `elder_guardian`
  (B8-G01) and `vibration` (B8-H01), and carries the 3 poplars (B8-H02).
- `STONE = 1` in the providers driver is correct for 26.2: state id 1
  is stone in the 26.2 datagen `blocks.json`.

### Wire codec (24 checks: 23 pass, 1 gap)

- All 22 data-carrying particle wire ids match `Particle::read`'s
  hardcoded ids exactly: State 1/2/36/118/122 (block, block_marker,
  falling_dust, dust_pillar, block_crumble), Color 28/43/49
  (entity_effect, tinted_leaves, flash), dust 21,
  dust_color_transition 22, effect 23, instant_effect 53,
  dragon_breath 15, sculk_charge 45, shriek 112, trail 56, item 54,
  vibration 55, geyser 7/10 (geyser, geyser_plume),
  geyser_base 8/9 (geyser_base, geyser_poof). `ids::particle(id)` is the
  identity when no version adapter is active, so these are the native
  26.2 ids. (Note: the doc comment on `Particle` says "26.3 registry
  order" but the ids match 26.2 exactly; the inner comment says "as of
  26.2".)
- `received_particle` resolves ids to names through the server-sent
  registry (not a hardcoded table) — correct for all 124 implemented
  types.
- Gap: B8-G02 (Flash write path).

### Override limiter (32 checks: 31 pass, 1 gap)

- Vanilla limiter-true set (32 types, from `<clinit>` booleans) vs the
  Rust `override_limiter()` list: identical except `elder_guardian`
  (B8-G01). No Rust extras (the 26.2 geyser entries are real here).

### Entity event bursts (30 checks: 30 pass)

- Every (entity, event) pair in the entity driver's `EVENTS` table is
  covered by the port's `entity_event_burst` (`particles.rs:620`):
  villager 12/13/14/42, cow 18, wolf/cat 6/7/18, parrot 6/7, nautilus
  6/7/18, horse/llama/camel 6/7/18, ocelot 40/41/18, dolphin 38, allay
  18 — with the right particle kinds and burst shapes, verified against
  the 26.2 jar: `Villager.handleEntityEvent` 12→heart, 13→angry, 14→
  happy, 42→splash; `TamableAnimal` 7→heart/6→smoke, 7 particles ×
  0.02 gaussian; `Animal` 18→heart ×7; `Ocelot` 41→heart/40→smoke;
  `Dolphin` 38→happy_villager ×7; `Allay` 18→3× heart; `AbstractHorse`
  6/7→taming particles. Counts/positions/RNG draws need the driver run;
  the mapping table is exact.
- The port's table also carries `nautilus`, `zombie_nautilus` and
  `camel_husk` arms — live and correct in 26.2 (unlike 1.21.7, where
  they were dead 26.x arms): `AbstractNautilus extends TamableAnimal`,
  `ZombieNautilus extends AbstractNautilus`, `CamelHusk extends Camel`.
  The driver just doesn't test the last two (B8-H04).

### Marker zoo (42 checks: 42 pass)

- Every real 26.2 block name in the marker driver's zoo + neighbour +
  campfire cases (42 names) has a `classify` arm in `animate_tick.rs`,
  including the guarded ones (lit furnaces, powered levers/repeaters,
  honey-level-5 hives, charged respawn anchors, active sculk sensors,
  down-pointing dripstone tips, lit candles/candle cakes). Positions/RNG
  draws need the driver run; coverage is complete. (`red_poplar_leaves`
  is the 43rd zoo name; it has no arm because the block does not exist
  in 26.2 — B8-H03, a driver-scope item, not a port gap.)

### Leaves (12 checks: 11 pass, 1 gap)

Per-block `leafParticleChance` and class, verified by parsing
`Blocks.class` (`BlockItemIds` + `BootstrapMethods` → registration
lambda → construction) against the port's `classify` table:

| Block | Vanilla 26.2 | Port | Verdict |
|---|---|---|---|
| oak / birch / jungle / acacia / dark_oak | Tinted, 0.01 | TintedLeaves, 0.01 | pass |
| spruce | Tinted, 0.01 | **none** | **gap — B8-G03** |
| cherry | Untinted cherry_leaves, 0.1 | CherryLeaves, 0.1 | pass |
| pale_oak | Untinted pale_oak_leaves, 0.02 | PaleOakLeaves, 0.02 | pass |
| mangrove | Tinted (MangroveLeavesBlock), 0.01 | TintedLeaves, 0.01 | pass |
| azalea / flowering_azalea | Untinted, fixed `ColorParticleOption(TINTED_LEAVES, -9399763)`, 0.01 | TintedLeaves, fixed tint `0x70922D`, 0.01 | pass (same particle + colour; class differs) |

- Mechanism: vanilla `makeFallingLeavesParticles` rolls
  `nextFloat() < leafParticleChance`, then skips if the block below is
  face-full; the port does `next_float() < chance && !sturdy(below)` —
  same draw, same shape of below-check. (Exact `sturdy` vs `isFaceFull`
  equivalence needs the driver run.) The drip check also matches:
  vanilla spawns unless `canOcclude && isFaceSturdy(UP)` below; the
  port spawns when `!can_occlude || !sturdy(below)`.
- Note: vanilla 26.2 has no `CherryLeavesBlock`/`PaleOakLeavesBlock`
  classes — cherry/pale-oak are `UntintedParticleLeavesBlock` instances,
  and the port's `FallingParticlesLeavesBlock` comment names the 26.3
  class; the mechanism is the 26.2 one.

### Dripstone and tags (5 checks: 5 pass)

- `speleothems` EXISTS in 26.2 (`pointed_dripstone`, `sulfur_spike`) —
  a real difference from 1.21.7, where it was absent. It is used by
  `SpeleothemBlock` for growth/falling logic, not by the client-side
  drip path: `findRootBlock` climbs same-block stalactite, and
  `animateTick` lives only on `PointedDripstoneBlock` — `SulfurSpikeBlock`
  defines no `animateTick` and `SpeleothemBlock` has none, so sulfur
  spikes correctly have no drip particles, matching the port (no
  sulfur_spike arm).
- `washed_away_by_fluids` and `blocks_lava_fire_spread` are both absent
  from the 26.2 client jar datapack — confirms the pilot's tag inventory
  pattern for 26.2.
- `triggers_ambient_dried_ghast_block_sounds` exists (soul_sand,
  soul_soil), as do the two desert ambient-sound tags — the tags the
  port's `AnimateTables` binds are real in 26.2.
- Vanilla dripstone animateTick path matches the port (0.12 roll,
  same-block climb, 0.02 cauldron rule).

### Oracle API existence (8 checks: 8 pass)

All vanilla methods the oracles call exist in the 26.2 jars with the
expected signatures (javap on the named jars):
`ParticleUtils.spawnParticlesOnBlockFaces` /
`spawnParticlesAlongAxis` (emitter, `net.minecraft.util.ParticleUtils`
in 26.x), `LivingEntity.spawnItemParticles` (item),
`Villager.handleEntityEvent` (entity),
`CampfireBlock.animateTick` / `makeParticles` /
`CampfireBlockEntity.particleTick` (marker), `ClientLevel.animateTick`
(marker loop). This is existence only — the oracles' *use* of them is
what the drivers verify.

### Other passes

- Engine `make()` exhaustiveness (see above).
- Game-event-10 apparition re-implementation present (§1, B8-G01).
- Bubble-column animate arm present (§2, G1).

## 4. Oracle repairs and defects

**Coordinator reconciliation (2026-11-24):** the batch agent ran before
Batch 0 landed and wrote this section from import resolution only ("no
26.2 Batch 0 measurement exists yet"). The measurement now exists
(`docs/sweep/26.2/batch-0-oracles.json`, 391 oracles, 2026-11-24), and it
changes one verdict: the marker oracle does not compile. The import-level
evidence below is retained — it holds for the six oracles that pass — but
it is no longer the authority; the measured table is.

### B8-O01 — `VanillaMarkerParticlesOracle` BROKEN, 1 compile error (MINOR, coordinator-added)

Measured: fails on all three classpaths, 1 error, class
constructor-mismatch — "44: constructor LocalPlayer in class LocalPlayer
cannot be applied to given types". This is Batch 0 group G2 (the
LocalPlayer constructor mismatch, 17 oracles). Import resolution cannot
see a constructor mismatch, which is why the agent's static check missed
it. **Blocks the marker_particles driver entirely** — before it can reach
the B8-G03 spruce cases. Repair is the G2 fix (align the `LocalPlayer`
construction with 26.2's constructor), shared with the 16 other G2
oracles; do not repair here (checks + triage only).

### Oracles assessed PASSING for 26.2 (Batch 0 measurement)

- `VanillaBlockParticlesOracle` — passes client+both; fails server cp
  only on client packages (`com.mojang.blaze3d.platform`, expected for a
  client oracle).
- `VanillaEmitterParticlesOracle` — passes client+both; server-only
  failure is client packages (expected).
- `VanillaEntityParticlesOracle` — passes client+both; server-only
  failure is client packages (expected).
- `VanillaParticleProvidersOracle` — passes client+both; server-only
  failure is client packages (expected).
- `VanillaItemParticlesOracle` — passes all three classpaths.
- `VanillaEffectParticlesOracle` — passes all three classpaths.
- `VanillaEffectSyncOracle` — passes all three classpaths.

For comparison, the 1.21.7 pre-repair list had Block 41 errors, Emitter
19, Entity 21, Item 3, Marker 42, Providers 116 — mostly missing 26.x
packages on the 1.21.7 classpath; only `VanillaEffectParticlesOracle`
passed. For 26.2 the 26.x layout is native, so six of seven particle
oracles compile as-is.

Static evidence for 26.2 (supporting the measured passes, not a substitute
for them):

- All 55 explicit `net.minecraft` imports across the 7 particle oracles
  + `VanillaEffectParticlesOracle` exist as classes in the 26.2 jars
  (server-game + client); all 10 inline fully-qualified references
  resolve too. No oracle references a 26.3-only class or a poplar /
  elder-guardian symbol.
- The providers oracle's mechanism fits 26.2: it builds
  `new ParticleResources()`, reads the private `spriteSets` field by
  reflection, and calls the public `getProviders()` — all present in
  the 26.2 client jar (`ParticleResources` holds `registerProviders()`
  in 26.2; `ParticleEngine` has no such method). It resolves kinds by
  registry name, which is why the 26.3-only poplar kinds would throw
  (B8-H02).
- The 8 oracle APIs this batch's scope touches all exist (§3).
- (Coordinator: the compile run this bullet called for is done — Batch 0
  measurement, above. The expectation it records proved right: the
  structural mismatch that sank the 1.21.7 oracles does not apply here,
  because the 26.2 jars have the 26.x layout. Only the marker oracle
  fails, on the G2 constructor mismatch.)

## 5. Driver inventory

The plan (§4) claims 9 particle drivers; the tree has 7 particle-named
drivers plus 1 particle-adjacent driver = **8**. The 9th is unaccounted
for: possibly the plan counted `tests/test_cushion_particles.py` (a
Python unit test for Horizon's cushion feature, not a parity driver) or
double-counted. No 9th parity driver exists under any name in
`versions/26.2/tools/` (grep over all `test_*_parity.py` for
particle-oracle references finds only these).

| Driver | Oracle (26.2 status, MEASURED — coordinator) | Probe | Status |
|---|---|---|---|
| block_particles | BlockParticles: passes client+both (server-only failure is client packages, expected) | block_particles_fixture.rs | UNRUN |
| emitter_particles | EmitterParticles: passes client+both (server-only failure is client packages, expected) | emitter_particles_probe.rs | UNRUN |
| entity_particles | EntityParticles: passes client+both (server-only failure is client packages, expected) | entity_particles_probe.rs | UNRUN; driver omits zombie_nautilus/camel_husk (B8-H04) |
| item_particles | ItemParticles: passes all three classpaths | item_particles_fixture.rs (horizon-world) | UNRUN |
| marker_particles | MarkerParticles: FAILS all three classpaths, 1 error (LocalPlayer ctor, B8-O01) | marker_particles_probe.rs | BLOCKED (B8-O01) — oracle does not compile; would also fail on spruce cases (B8-G03) once repaired; red_poplar_leaves zoo cases un-runnable (B8-H03) |
| native_client_particles | none (Horizon-only Host API contract, no vanilla oracle) | game fixture | UNRUN; out of vanilla scope |
| particle_providers | ParticleProviders: passes client+both (server-only failure is client packages, expected) | particle_providers_probe.rs | UNRUN; would FAIL on the 3 poplar kinds — oracle throws `AssertionError("unknown particle …")` (B8-H02); does not test elder_guardian (B8-G01) or vibration (B8-H01) |
| effect_sync (adjacent) | EffectParticles: passes all three; EffectSync: passes all three | effect_sync_fixture.rs | UNRUN; scope is effect particle/visibility sync |

`parity.py` was not run (blocked — static checks only, no cargo). No
prepared 26.2 data existed when this batch ran (`.local` deleted;
inventory §7 verdict NEEDS PREPARE), so no driver could run yet even with
execution allowed. (Coordinator: prepare has since landed — see the note
at the top — so the data precondition is now met; 7 of the 8 drivers are
one oracle repair (B8-O01) plus the B8-H02/H03 trims away from runnable.)

## 6. Unit tests

31 `#[test]`s in `crates/horizon-client-render/src/particles.rs` (lines
5165–6042), unrun (blocked — no cargo builds in this batch):

- Vanilla: `legacy_spell_providers_use_velocity_rgb_and_their_original_alpha_rules`,
  `particle_light_cache_reuses_exact_blocks_and_fails_open_on_collisions`,
  `subpixel_culling_uses_the_frame_projection_and_fails_open`,
  `disabled_culled_render_matches_the_regular_particle_path`,
  `living_teleport_particles_span_the_old_and_new_boxes`,
  `egg_teleport_unpacks_offsets_and_preserves_seeded_draw_order`,
  `consume_teleport_unpacks_offsets_within_127`,
  `egg_teleport_options_roll_before_distance_rejection`,
  `mining_particles_follow_slab_and_trapdoor_height`,
  `destroyed_slab_uses_its_shape_and_particle_density`,
  `empty_outline_emits_no_terrain_particles_or_random_draws`,
  `every_hit_face_consumes_three_position_draws`,
  `multipart_destroy_burst_uses_each_box_and_caps_widths`,
  `spore_blossom_air_stops_at_its_particle_limit`,
  `a_vibration_option_makes_the_signal_its_provider_draws`,
  `particle_names_round_trip`, `terrain_crumb_samples_a_mirrored_quarter`,
  `legacy_trails_choose_a_lifetime_for_each_particle_from_the_level_random`,
  `sizes_follow_vanilla_curves`, `lifetime_alpha_interpolates`,
  `explosion_tracker_tracks_and_ignores_zero_blocks`,
  `explosion_tracker_caps_debris_at_512_across_multiple_explosions`,
  `explosion_tracker_emits_debris_only_in_air` (23 vanilla tests).
- Horizon mod particles (not vanilla scope): `a_mod_particles_json_becomes_its_sprite_set`,
  `mod_behaviour_validates_declarations`,
  `mod_velocity_adds_the_spawn_velocity_and_jitters_deterministically`,
  `mod_animation_modes_pick_their_sprites`,
  `mod_curves_clamp_and_interpolate_over_lifetime`,
  `mod_animation_advances_in_the_tick_with_and_without_curves`,
  `mod_particle_ticks_motion_curves_and_removal`,
  `fading_mod_particles_use_the_blended_layer`.

Same 31 tests as the 1.21.7 batch (same names, same file).

## 7. Not covered

- **All driver numerics.** Per-tick provider physics (positions, motion,
  age, lifetime, size, colour, roll, sprite, UVs, light), emitter RNG
  draws, entity-burst counts/positions, block-particle placement, marker
  loop exactness — only the drivers can verify these, and none ran. The
  static checks above bound the *tables and mappings*, not the numerics.
- **Translucent-layer table.** `ParticleType::translucent_layer()` (which
  types vanilla renders in the translucent layer) was not verified — it
  needs per-class `getRenderType` checks across ~20 particle classes.
  (Same scope decision as the 1.21.7 batch.)
- **Sprite sets / particle textures.** Whether the engine resolves every
  type's sprite set from the 26.2 assets needs prepared data + a run.
- **Level events other than 3002–3005.** `level_particles.rs` handlers
  (dragon egg, shulker/ender teleport, growth, turtle/sniffer eggs, torch
  burnout, dragon fireball, potion splashes, `level_event_at`) exist but
  their numerics are unverified; only 3002–3005 are in the emitter driver.
- **Apparition visual fidelity.** The elder-guardian re-implementation
  (§1) needs an in-game look or the providers driver.
- **Oracle compilation** for 26.2 (§4).
- **The 9th driver.** The plan's count of 9 does not match the tree's 8;
  if a 9th driver exists somewhere else (another version tree, a renamed
  file), it was not found here.
- **Dripstone climb limit.** Vanilla's `findRootBlock` passes 11 to
  `findBlockVertical`; the port climbs 10 steps (`1..11`). Whether the
  off-by-one reading matches needs the marker driver run.
- **G1/G6 server-side status for 26.2.** Whether bubble columns form and
  dripstone transfers fluid server-side in the 26.2 port is Batch 5's
  scope, not this batch's.

## 8. What unblocks the rest

1. Prepare 26.2 data (Batch 0 setup — no prepared 26.2 data exists).
2. (Coordinator: done — Batch 0 measured all seven particle oracles +
   the effect oracles against the 26.2 classpath; see §4. Six of seven
   pass on their real classpath. What remains is the B8-O01 repair —
   the G2 LocalPlayer-constructor fix for `VanillaMarkerParticlesOracle`.)
3. Fix B8-G01 (add the `ElderGuardian` variant + `ElderGuardianParticle`
   provider) and B8-G03 (spruce leaves arm) on fix branches, then run
   `particle_providers` and `marker_particles`; fix B8-G02 (Flash arm in
   `hub_particle`) with them or separately. B8-G03 is driver-blocking;
   B8-G02 is not covered by any driver; B8-G01 blocks nothing in the
   26.2 tree because the driver skips the type — the fix should add
   `elder_guardian` to the driver's `SIMPLE` list too, or the type stays
   untested.
4. Trim the 26.3-only entries from the drivers for 26.2 (B8-H02: 3
   poplar kinds in the providers driver; B8-H03: `red_poplar_leaves` in
   the marker zoo), add the missing coverage (B8-H01: vibration;
   B8-H04: zombie_nautilus, camel_husk), then run the drivers
   (`block`, `emitter`, `entity`, `item`, `effect_sync`); the item
   driver needs a longer cargo-build window than its 900 s timeout under
   load (or a warm target dir) — 1.21.7 precedent.
5. Run the 31 `particles.rs` unit tests (`cargo test -p
   horizon-client-render --lib particles::`, dev profile, `-j 1`).

## 9. Method notes

- The 26.2 **client** jar (`leakage-audit/jars/26.2-client.jar`) is
  Mojang-mapped — `net/minecraft/client/particle/ParticleEngine.class`
  is present by its mapped name — and its `ParticleTypes.class` is
  byte-identical (md5) to the server-game jar's. The
  `26.1-adapters/versions/26.2/jars/client.jar` is the same file. The
  leakage-audit `26.2-server.jar` is a bundler jar (never javap'd).
- 26.x registration shapes differ from 1.21.7 in two places this batch
  had to adapt to:
  - `Blocks.<clinit>` registers via `register(BlockItemId, Function,
    Properties)` with the name from a `getstatic BlockItemIds.<NAME>`
    (no `ldc` name strings) — the class-file parser associates
    `BlockItemIds` fields with their `invokedynamic` lambdas instead of
    string constants.
  - Providers live in `ParticleResources.registerProviders()` (private),
    not `ParticleEngine` — and 26.2 has a dedicated
    `ElderGuardianParticle$Provider` where 1.21.7 used the shared
    `MobAppearanceParticle` provider.
- `javap -c` on `Blocks` was not used (1.21.7 warning: exceeds 280 s
  under throttle). The Python class-file reader
  (`/tmp/b8/parse_blocks.py`, constant pool + `BootstrapMethods` +
  bytecode walk with a full opcode-length table) resolves the leaves
  associations in ~2 s. One gotcha found while parsing
  `ParticleTypes`: the first 34 registrations use `ldc` and the rest
  `ldc_w` — a `ldc_w?` regex silently drops the `ldc` ones (the `_`
  is mandatory in that pattern); use `ldc(_w)?`.
- `javap` on the small classes ran 5–30 s each under load; batches of
  2–3 classes per exec kept the work moving.
- No production code (`crates/`) was touched. No commits were made in
  this batch. Only the two deliverable files were written.
