# Batch 8 — Particles (26.1 parity sweep)

**Version:** 26.1 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.1`
(workdir `~/workspace/mcir-work/sweep-26.1`, synced to main) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 8 — all 117 particle types,
emitters, providers; 9 drivers claimed by the plan, 8 found in the tree
(see §5).

**Status: DRAFT — not committed.** Checks + triage only, per the batch
brief: no fixes, no edits under `versions/26.1/`, no cargo builds, no
commits. Everything below is static analysis: `javap` on the named 26.1
jars (`~/workspace/mcir-work/26.1.x/versions/26.1/jars/{client,server-game}.jar`;
the server jar is a Mojang bundler jar and was never javap'd), direct
class-file parsing of `Blocks.class` where `javap` would be too slow,
source comparison against `versions/26.1/crates/`, data checks against
the 26.1 datagen reports and the prepared data in
`versions/26.1/.local/horizon-client/26.1/`, and oracle triage from the
measured Batch 0 table (`docs/sweep/26.1/batch-0-oracles.json`, 199/391
oracles compile) — not from import resolution. No driver runs, no cargo
tests. The coordinator commits these deliverables.

**Headline:** all three 1.21.7 Batch 8 defects recur in 26.1 unchanged —
the 26.3-derived port still lacks the `elder_guardian` variant (B8-G01),
`hub_particle` still has no `Flash` arm (B8-G02), and spruce leaves still
drop no falling leaves (B8-G03). One new 26.1 defect was found while
verifying dripstone: the level-event 1504 stalactite drip particle never
fires, because its gate checks for the `SpeleothemBlock` class, which does
not exist in 26.1 (B8-G04). And one new oracle defect: the providers
oracle does not compile for 26.1 at all — it references the geyser API,
which 26.1 predates (B8-O02), so the `particle_providers` driver is
oracle-blocked in addition to carrying 5 kinds the 26.1 registry lacks
(B8-H02). The marker oracle is broken the same way as in 26.2
(B8-O01, LocalPlayer constructor, Batch 0 G2).

## Tally

- **Behaviour checks: 272 — 266 pass, 6 gap** (across 4 defect items,
  B8-G01–B8-G04, all minor). Counting rule: a behaviour check is one
  comparison where BOTH the vanilla side (26.1 jar/data) and the Rust
  side were actually compared; oracle-status checks are NOT counted in
  the tally (per the common brief — this differs from the 26.2 run's
  Batch 6). Per-section counts (see §3): registry/type coverage 120
  (118 pass, 2 gap), wire codec 20 (19 pass, 1 gap), override limiter
  28 (27 pass, 1 gap), entity event bursts 30 (30 pass), marker zoo 45
  (45 pass), leaves 12 (11 pass, 1 gap), dripstone + tags 7 (6 pass, 1
  gap), oracle API existence 8 (8 pass), other passes 2 (2 pass).
  120+20+28+30+45+12+7+8+2 = 272; 266 + 6 = 272.
- **Oracle defects: 2** (B8-O01 — marker oracle, LocalPlayer ctor,
  Batch 0 G2; B8-O02 — providers oracle, 10 geyser-symbol errors, new
  for 26.1). The other six particle-family oracles pass on their real
  classpath (measured, §4).
- **Needs-harness items: 4** (B8-H01–B8-H04, §1): driver-coverage gaps,
  not behaviour gaps.
- **Drivers: 0 run, 8 inventoried** (7 particle drivers + 1
  particle-adjacent effect-sync driver). 2 of the 8 are oracle-blocked
  (B8-O01 blocks marker_particles; B8-O02 blocks particle_providers —
  which would also fail on its 5 non-26.1 kinds once repaired, B8-H02);
  the other 6 have compiling oracles (measured) but are unrun, not
  passing.
- **Unit tests:** 31 `#[test]`s in `particles.rs` exist (23 vanilla + 8
  mod-particle, same names as 26.2/1.21.7); unrun, blocked.

## 1. Defects

### B8-G01 — `elder_guardian` has no port implementation at all (MINOR, NEW, recurs from 1.21.7)

**Checks failed (3):** per-type end-to-end coverage for `elder_guardian`;
its `override_limiter` flag; the providers-driver kind set (which omits
it — see below).

**Evidence (jar).** Vanilla 26.1 registers `ELDER_GUARDIAN` as a
`SimpleParticleType` (`ParticleTypes.<clinit>`, wire id 17,
`overrideLimiter=true`) and registers a provider for it in
`ParticleResources.registerProviders()` — a dedicated
`ElderGuardianParticle$Provider` (26.1 has its own
`ElderGuardianParticle` class; 1.21.7 used the shared
`MobAppearanceParticle` provider). `ClientPacketListener.handleGameEvent`
spawns it at the player's position on game event
`GUARDIAN_ELDER_EFFECT` (`addParticle(ELDER_GUARDIAN, player.x/y/z, 0, 0,
0)`, bytecode offset 530), plus the curse sound (offset 574).

**The port.** `ParticleType` (`horizon-client-core/src/particle_type.rs`)
has 127 variants and no `ElderGuardian`. Consequences, all verified in
source:

- `received_particle` (`game/lan_particles.rs:466`, `from_name` at :488)
  resolves a server-sent particle by name and returns `None` for
  `elder_guardian` — a `LevelParticles` packet for it is silently
  dropped.
- `/particle minecraft:elder_guardian` fails with `particle.notFound`
  (`game/commands/world_cmds.rs:760`, `from_name` returns `None`);
  vanilla accepts it.
- `override_limiter()` has no entry for it (28 vanilla limiter types vs
  27 real-26.1 Rust entries; the only 26.1 miss is `elder_guardian` —
  the Rust list's other 4 entries are the geyser types, dead in 26.1).
- The providers driver does **not** send `elder_guardian` cases in the
  26.1 tree (its kind set omits the type entirely), so unlike 1.21.7 —
  where the driver sent 12 cases and the probe printed `no provider` —
  the driver is not blocked by this defect; it silently skips the type.
  The probe would still print `no provider` if it ever saw one
  (`particle_providers_probe.rs:40`, `from_name` → `None`; :184).

**Mitigation (why minor).** The one in-game path that matters — the elder
guardian's curse apparition — is re-implemented outside the particle
engine, same as in 1.21.7/26.2: game event 10 sets `elder_apparition`
(`game/remote.rs:5032-5043`, `game/creatures.rs:4175`) and
`elder_apparition_geometry` (`game/mob_blocks.rs:79`) draws the ghostly
guardian in front of the camera with the vanilla 30-tick lifetime
(`APPARITION_TICKS = 30`, `mob_blocks.rs:70`). So the apparition works;
what is missing is the particle-type plumbing (`/particle`, server
packets). Visual fidelity of the re-implementation against vanilla's
`ElderGuardianParticle` is unverified statically — needs the driver run
or an in-game look.

### B8-G02 — `/particle minecraft:flash` sends a malformed packet to LAN guests (MINOR, NEW, recurs from 1.21.7)

**Checks failed (1):** the `Flash` arm of `hub_particle`.

**Evidence.** `Particle::read` (`horizon-protocol/src/entity_data.rs:185`)
decodes wire id 42 (`flash`, verified against the 26.1 registry) as
`Color(i32)` — a flash particle on the wire always carries a colour. But
`hub_particle` (`game/lan_particles.rs:408`), which builds the packet a
LAN host sends, has no `Flash` arm in its `Color` match (`EntityEffect |
TintedLeaves` only, :437), so a flash particle is built with `data:
None`, and `Particle::write` — which writes data by presence, not by id
— emits the id with no colour bytes. Any guest reading id 42 then
consumes the next 4 bytes of the stream as the colour and continues
misaligned: packet desync (and likely a disconnect or a dropped
connection) for every guest in range.

**Reachability (narrow — why minor).** The only path that sends `Flash`
through `hub_particle` is `/particle minecraft:flash`: `cmd_particle`
builds `ParticleOptions::simple(Flash)` (no colour — vanilla's command
requires the colour option, the port does not; `world_cmds.rs:871`
catch-all) and calls `network_particle` (:925). Everywhere else `Flash`
is created client-side only (the firework explosion flash,
`particles.rs:3775`, via `self.create(...)`, never sent over the
network). The local player is unaffected (draws directly, never reads
the packet). So the trigger is exactly: a LAN host runs `/particle
minecraft:flash` with guests in range.

### B8-G03 — Spruce leaves never drop falling-leaf particles (MINOR, NEW, recurs from 1.21.7, driver-blocking)

**Checks failed (1):** the spruce entry of the per-block leaves check.

**Evidence (jar).** Vanilla 26.1 registers `SPRUCE_LEAVES` as a
`TintedParticleLeavesBlock` with `leafParticleChance = 0.01` — verified
by parsing `Blocks.class` directly: `BlockItemIds.SPRUCE_LEAVES` +
`lambda$static$27` constructs `TintedParticleLeavesBlock(0.01F, …)`.
`LeavesBlock.animateTick` calls `makeFallingLeavesParticles`, which
rolls `nextFloat() < leafParticleChance` and spawns the falling leaf
unless the block below is face-full (javap). So vanilla spruce drops
tinted leaves at 1% per animateTick, like oak.

**The port.** `classify` (`animate_tick.rs:151`) maps `"spruce_leaves"`
to `Animate::Leaves(None, None, false)` — no falling particle at all.
Every other leaf block matches vanilla (see §3, leaves table).

**Driver note.** The marker driver's own zoo comment says "the spruce
that never falls" — the comment encodes the same wrong belief, but the
driver compares against the real vanilla oracle, so the driver is
correct as a check and **would fail on the spruce cases** (oracle emits
tinted leaves, probe emits none — the probe runs the port's real
`animate_tick`, `marker_particles_probe.rs:168`) once B8-O01 is repaired
— the driver is currently oracle-blocked, not merely unrun. The comment
needs correcting when the fix lands; the driver needs no change.

### B8-G04 — Level-event 1504 stalactite drip particle never fires: its gate checks for `SpeleothemBlock`, which does not exist in 26.1 (MINOR, NEW, 26.1-only)

**Checks failed (1):** the level-event 1504 gate in the dripstone
section.

**Evidence (jar + prepared data).** Vanilla 26.1 has no
`SpeleothemBlock` class at all (0 entries in either jar;
`PointedDripstoneBlock` extends `Block` directly) and no `speleothems`
tag in the client jar's datapack. The prepared data agrees:
`block-behaviours.json` gives `minecraft:pointed_dripstone` the class
chain `["PointedDripstoneBlock", "Block"]`. Vanilla 26.1's
`PointedDripstoneBlock.maybeTransferFluid` sends level event 1504 when
a stalactite drips fluid into a cauldron below (`dripstone.txt`, offsets
218/245), and the port's server does the same
(`horizon-world/src/level/speleothem.rs:306/308`).

**The port.** The client routes 1504 to `dripstone_drip_particle`
(`game.rs:4352-4353`), whose first act is the `is_stalactite` gate
(`game/level_particles.rs:237-244`): `has_class("SpeleothemBlock")`
(`horizon-items/src/blocks.rs:121`, an exact match over the prepared
class chain). In 26.1 that is false for every block, so the function
returns before spawning anything — the cauldron-fill drip particle
never appears. The ambient animateTick drip path is unaffected: it is
name-based (`AnimateTables.dripstone`, built from
`"pointed_dripstone"` by name, `animate_tick.rs:397-410`) and matches
vanilla (§3).

**Scope note.** Whether the port's *server* actually transfers fluid to
cauldrons in 26.1 (pilot G6's 26.1 status) is Batch 5's scope; the
client gate is dead regardless of server reachability, and the defect
is minted on the client side only. The same `SpeleothemBlock` class
assumption appears elsewhere in the tree (see §2) — those are other
batches' scope.

### Needs-harness items (driver coverage, not behaviour gaps)

- **B8-H01 — providers driver omits `vibration`.** The port implements
  vibration end to end (`Particle::read` id 48, `hub_particle` arm,
  `make()` arm at `particles.rs:3146`, `ParticleOptions::vibration`),
  and the oracle's `options()` handles it
  (`VanillaParticleProvidersOracle.java`, the `VIBRATION` branch), but
  the driver's kind set never sends a vibration case — so the vibration
  provider's numerics have no driver coverage. Add vibration cases
  (with a target + duration) to
  `test_particle_providers_parity.py`.
- **B8-H02 — providers driver includes 5 kinds 26.1 lacks (3 poplar
  leaves + 2 sulfur particles).** The driver's `SIMPLE` list carries
  `red/orange/yellow_poplar_leaves` (26.3-only),
  `sulfur_bubbles` and `sulfur_cube_goo` (26.2-only), none of which are
  in the 26.1 registry. The oracle resolves kinds by name and throws
  `AssertionError("unknown particle " + id)` for an unregistered type
  (`VanillaParticleProvidersOracle.java:131`), so the driver cannot run
  against the 26.1 oracle as-is: it would die on the first such case
  (12 cases per kind). Trim the 5 kinds for 26.1. (26.2 had only the 3
  poplars; the 2 sulfur kinds are a 26.1-only addition. The geyser
  kinds are NOT in the driver's kind set — `options()` has geyser arms
  but `kinds` never sends them — so they need no trim.)
- **B8-H03 — marker driver zoo includes `red_poplar_leaves`
  (26.3-only).** The zoo (`test_marker_particles_parity.py`,
  `zoo_states()`) sends `red_poplar_leaves[distance=3,...]` cases, but
  no such block exists in 26.1 — the oracle cannot build that world.
  (Unlike 26.2, the 26.1 port DOES have poplar `classify` arms
  (`animate_tick.rs:144-146`), but they are dead: no poplar block
  exists to classify.) Trim the zoo case for 26.1.
- **B8-H04 — entity driver omits live 26.1 entities `zombie_nautilus`
  and `camel_husk`.** Both are real 26.1 entity types (protocol ids
  152 and 20) and both inherit the 6/7/18 bursts (`ZombieNautilus
  extends AbstractNautilus extends TamableAnimal`; `CamelHusk extends
  Camel extends AbstractHorse`), and the port's `entity_event_burst`
  covers them via its tamable/horse sets — but the driver's `EVENTS`
  table has no entries for them (6 untested pairs). Add them to
  `test_entity_particles_parity.py`.

## 2. Cross-references (not re-minted)

- **G1 (pilot) — bubble columns never form.** The client-side animate arm
  for bubble columns exists and is correct (`Animate::BubbleColumn`,
  `animate_tick.rs:1660` — `BubbleColumnUp` particles + whirlpool/upwards
  ambient sounds), but it can never trigger in-game because no
  `bubble_column` block states ever form server-side (G1, 1.21.7 pilot;
  26.1 server-side status is Batch 5's scope, not verified here). No new
  defect: the particle-visible consequence of G1, not a particle defect.
- **G6 (pilot) — dripstone never transfers fluid to cauldrons.**
  Server-side only; 26.1 server-side status is Batch 5's scope. The
  *client-side* drip particles are unaffected except for B8-G04's gate:
  vanilla 26.1's `PointedDripstoneBlock.animateTick` rolls 0.12 and
  climbs same-block stalactite via `findRootBlock` (no tag involved —
  there is no `SpeleothemBlock` and no `speleothems` tag in 26.1), and
  the port's `dripstone_drip` (`animate_tick.rs:1807`) matches it by
  name, with the same 0.12 roll and 0.02 cauldron rule. The marker
  oracle's scope comment about binding "the speleothem tag the
  stalactite branch needs" (`VanillaMarkerParticlesOracle.java:136`)
  is stale for 26.1 — the oracle binds tags from the jar's datapack,
  which has no such tag, and no code path reads it — but the comment is
  not a defect.
- **B12-G04 pattern — 26.x-only types in the `ParticleType` table.**
  Same table-leak *pattern* as B12-G04, but the 11 extra types in the
  Rust enum (8 26.2-only: geyser family, sulfur bubbles, noxious gas
  pair, sulfur cube goo; 3 26.3-only: poplar leaves) are unreachable —
  ids are name-resolved from the server's registry, and a 26.1 server
  never sends those names — so they are not minted as a defect. Dead
  table entries, not a behaviour gap. The 4 geyser entries in
  `override_limiter()` are likewise dead. (Note: in 26.1 the enum
  carries 11 extras, vs 3 in 26.2 — the geyser/sulfur types are real in
  26.2 but not here. The enum's header comment documents the 8
  26.2-only variants explicitly.)
- **The `SpeleothemBlock` assumption elsewhere in the tree.** B8-G04 is
  the particle-visible instance; the same 26.2+-only class name gates
  other behaviour: `mesher.rs:910` (`has_class("SpeleothemBlock")`
  selects the 0.125 horizontal random offset for dripstone — dead in
  26.1, so dripstone meshes with the 0.25 default; vanilla 26.1's
  `getMaxHorizontalOffset` is `SHAPE_BASE.min(X)`, i.e. the 0.125
  branch) and the server-side `horizon-world` speleothem/falling code
  (`level/speleothem.rs`, `level/falling.rs:54`, `level/classes.rs:215`
  — Batch 5's scope). The projectile-hit path is fine: it matches by
  block name, not class (`creatures.rs:5084`).

## 3. What passed (per area)

### Registry and type coverage (120 checks: 118 pass, 2 gap)

- 117 particle types: `registries.json` count == jar `ParticleTypes`
  `<clinit>` registration count (117), and the `<clinit>` order equals
  the `protocol_id` of every type (verified by parsing the bytecode;
  the prepared `.local` registries agree exactly — 117 entries, 0
  mismatches vs datagen, `elder_guardian` present, no geyser/sulfur/
  poplar/noxious entries). `ParticleTypes.class` is byte-identical
  (md5 `8292544543edd947bd3398873d591f30`) between the client and
  server-game jars.
- Engine `make()` (`particles.rs:2203`, outer match at :2264) has an
  outer match arm for all 127 `ParticleType` variants (no outer
  catch-all — the `_ =>` arms in the body are all nested matches) —
  every enum variant has a provider implementation. The only
  engine-coverage gap is the missing `elder_guardian` variant itself
  (B8-G01).
- Vanilla providers: `ParticleResources.registerProviders()` in the 26.1
  client jar registers a provider for all 117 types, including
  `ELDER_GUARDIAN` (`ElderGuardianParticle$Provider`). (Field-name
  quirk, same as 26.2: the fields are
  `TRIAL_SPAWNER_DETECTED_PLAYER(_OMINOUS)` but register under the
  names `trial_spawner_detection(_ominous)`.)
- The 11 Rust-enum extras (8 26.2-only geyser/sulfur + 3 26.3-only
  poplar leaves) are absent from the 26.1 jar's `ParticleTypes` —
  confirmed unreachable from a 26.1 server.
- Gap: providers-driver kind set (120 kinds) misses `elder_guardian`
  (B8-G01) and `vibration` (B8-H01), and carries the 5 non-26.1 kinds
  (B8-H02).
- `STONE = 1` in the providers driver is correct for 26.1: state id 1
  is stone in the 26.1 datagen `blocks.json` (the "in 26.3" comment is
  stale; the value is right).

### Wire codec (20 checks: 19 pass, 1 gap)

- All 18 data-carrying particle wire ids match `Particle::read`'s
  hardcoded ids exactly: State 1/2/29/111/115 (block, block_marker,
  falling_dust, dust_pillar, block_crumble), Color 21/36/42
  (entity_effect, tinted_leaves, flash), dust 14,
  dust_color_transition 15, effect 16, instant_effect 46,
  dragon_breath 8, sculk_charge 38, shriek 105, trail 49, item 47,
  vibration 48. `ids::particle(id)` is the identity when no version
  adapter is active (`adapter/ids.rs:318-325`), so these are the native
  26.1 ids. The read code carries no geyser arms — correct for 26.1
  (comment: "The geyser particles a later release adds do not exist in
  26.1"), though the `Geyser`/`GeyserBase` data variants still exist
  in the enum and `write` still writes them if present (unreachable
  from a 26.1 read; `hub_particle` can still build them from a
  `ParticleOptions::geyser`, but no 26.1 path creates one).
- `received_particle` resolves ids to names through the server-sent
  registry (not a hardcoded table) — correct for all 116 implemented
  types.
- Gap: B8-G02 (Flash write path).
- Note (not a defect): `hub_particle` deliberately returns `None` for
  item and trail particles (`lan_particles.rs:456`,
  `network_particle` doc: "a particle whose data isn't sent from here
  (items, trails)") — a LAN host's `/particle item|trail` draws locally
  but sends nothing to guests. Documented behaviour, same as 26.2;
  the command does not parse trail options either. Not minted.

### Override limiter (28 checks: 27 pass, 1 gap)

- Vanilla limiter-true set (28 types, from `<clinit>` booleans) vs the
  Rust `override_limiter()` list: identical except `elder_guardian`
  (B8-G01). The Rust list's 4 extra entries are the geyser types —
  dead in 26.1, unreachable.

### Entity event bursts (30 checks: 30 pass)

- Every (entity, event) pair in the entity driver's `EVENTS` table is
  covered by the port's `entity_event_burst` (`particles.rs:620`):
  villager 12/13/14/42, cow 18, wolf/cat 6/7/18, parrot 6/7, nautilus
  6/7/18, horse/llama/camel 6/7/18, ocelot 40/41/18, dolphin 38, allay
  18 — with the right particle kinds and burst shapes, verified against
  the 26.1 jar: `Villager.handleEntityEvent` 12→heart, 13→angry, 14→
  happy, 42→splash; `TamableAnimal` 7→heart/6→smoke, 7 particles ×
  0.02 gaussian; `Animal` 18→heart ×7; `Ocelot` 41→heart/40→smoke;
  `Dolphin` 38→happy_villager ×7 (0.01 gaussian, y+0.2); `Allay`
  18→3× heart (0.02 gaussian, y+0.5); `AbstractHorse` 6/7→taming
  particles; `AbstractVillager.addParticlesAroundSelf` 5 × 0.02.
  Counts/positions/RNG draws need the driver run; the mapping table is
  exact.
- The port's table also carries `nautilus`, `zombie_nautilus` and
  `camel_husk` arms — live and correct in 26.1 (unlike 1.21.7, where
  they were dead 26.x arms): `AbstractNautilus extends TamableAnimal`,
  `ZombieNautilus extends AbstractNautilus`, `CamelHusk extends Camel`.
  The driver just doesn't test the last two (B8-H04).

### Marker zoo (45 checks: 45 pass)

- Every real 26.1 block name in the marker driver's zoo (36 names) +
  neighbour (6: dried_ghast, beehive, bee_nest, redstone_ore,
  deepslate_redstone_ore, pointed_dripstone) + campfire (2: campfire,
  soul_campfire) + wither_rose (1) cases has a `classify` arm in
  `animate_tick.rs`, including the guarded ones (lit furnaces, powered
  levers/repeaters, honey-level-5 hives, charged respawn anchors,
  active sculk sensors, down-pointing dripstone tips, lit
  candles/candle cakes). Positions/RNG draws need the driver run;
  coverage is complete. (`red_poplar_leaves` is the 37th zoo name; it
  has a classify arm in this tree but the block does not exist in 26.1
  — B8-H03, a driver-scope item, not a port gap.)
- The unlit variants in the zoo (unlit redstone torches, furnaces,
  smokers) correctly map to `Animate::None`.

### Leaves (12 checks: 11 pass, 1 gap)

Per-block `leafParticleChance` and class, verified by parsing
`Blocks.class` (`BlockItemIds` + `BootstrapMethods` → registration
lambda → construction) against the port's `classify` table:

| Block | Vanilla 26.1 | Port | Verdict |
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
- Note: vanilla 26.1 has no `CherryLeavesBlock`/`PaleOakLeavesBlock`
  classes — cherry/pale-oak are `UntintedParticleLeavesBlock` instances,
  and the port's `FallingParticlesLeavesBlock` comment names the 26.3
  class; the mechanism is the 26.1 one.

### Dripstone and tags (7 checks: 6 pass, 1 gap)

- `speleothems` does NOT exist in 26.1 (absent from the client jar's
  datapack; no `SpeleothemBlock` class) — a real difference from 26.2,
  where it held `pointed_dripstone` + `sulfur_spike`. The port's
  animateTick drip path does not need it (name-based table), so this is
  a pass for the animate path; the class-gated level-event path is the
  gap (B8-G04).
- `washed_away_by_fluids` and `blocks_lava_fire_spread` are both absent
  from the 26.1 client jar datapack — confirms the pilot's tag inventory
  pattern for 26.1.
- `triggers_ambient_dried_ghast_block_sounds` exists (soul_sand,
  soul_soil), as do the two desert ambient-sound tags
  (`triggers_ambient_desert_sand_block_sounds`: sand, red sand;
  `triggers_ambient_desert_dry_vegetation_block_sounds`: #terracotta,
  sand, red sand) — the tags the port's `AnimateTables` binds are real
  in 26.1. (The poplar tag it also binds,
  `required_for_poplar_leaf_ambience`, has no 26.1 referent — dead
  path, no poplar blocks exist.)
- Vanilla dripstone animateTick path matches the port (0.12 roll,
  same-block climb, 0.02 cauldron rule, mud-as-water, 5/16 − 0.0625
  drip height).
- Gap: B8-G04 (level-event 1504 gate).

### Oracle API existence (8 checks: 8 pass)

All vanilla methods the oracles call exist in the 26.1 jars with the
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

Triage is from the measured Batch 0 table
(`docs/sweep/26.1/batch-0-oracles.json`, 391 oracles, 2026-11-24) —
NOT from import resolution. Both failing particle oracles were
re-verified against the 26.1 jar by this batch (javap + a single-file
javac reproduction for the providers oracle). Supporting import check:
all 95 distinct `net.minecraft` references across the 8 particle-family
oracles resolve as classes/packages in the 26.1 jars — which is exactly
why import resolution cannot see these two failures (the marker failure
is a constructor arity mismatch; the providers failure is missing
symbols behind a wildcard import).

### B8-O01 — `VanillaMarkerParticlesOracle` BROKEN, 1 compile error (MINOR)

Measured: fails on all three classpaths, 1 error, class
constructor-mismatch — "44: constructor LocalPlayer in class LocalPlayer
cannot be applied to given types". This is Batch 0 group G2 (the
LocalPlayer constructor mismatch, 20 oracles carry the error, 11 as
first error). Verified: the oracle's `Player` subclass calls
`super(null, null, null, null, null, null, false, null, null)` — 9
arguments, the 26.3 form (`VanillaMarkerParticlesOracle.java:44`);
26.1's `LocalPlayer` constructor takes 8 (`Minecraft, ClientLevel,
ClientPacketListener, StatsCounter, ClientRecipeBook, Input, boolean,
ChatAbilities` — javap on the 26.1 client jar). **Blocks the
marker_particles driver entirely** — before it can reach the B8-G03
spruce cases. Repair is the G2 fix (align the `LocalPlayer`
construction with 26.1's constructor), shared with the other G2
oracles; do not repair here (checks + triage only).

### B8-O02 — `VanillaParticleProvidersOracle` BROKEN, 10 compile errors (MINOR, new for 26.1 — it passed for 26.2)

Measured: fails on all three classpaths, 10 errors, class
missing-symbol — first error "164: cannot find symbol". All 10 errors
are references to the geyser API, which 26.1 predates: `GeyserParticleOptions`
(3 refs: :144 constructor, :164 type + cast), `GeyserBaseParticleOptions`
(3 refs: :146 constructor, :165 type + cast), `ParticleTypes.GEYSER` /
`GEYSER_PLUME` (:143), `ParticleTypes.GEYSER_BASE` / `GEYSER_POOF`
(:145). Verified: zero `geyser` entries in either 26.1 jar, and no
`GEYSER*` fields in `ParticleTypes`. Reproduced with a single-file
javac run against the 26.1 client classpath (11 errors there — the 11th
is `ParityData` at :214, an artifact of compiling one file alone; the
Batch 0 big compile resolves it, leaving the 10 geyser errors).
**Blocks the particle_providers driver entirely.** Repair is a deletion
(the geyser branches + the two cast helpers — the driver never sends
geyser kinds, so nothing is lost), not a translation; do not repair here.

### Oracles assessed PASSING for 26.1 (Batch 0 measurement)

- `VanillaBlockParticlesOracle` — passes client+both; fails server cp
  only on client packages (`com.mojang.blaze3d.platform`, expected for
  a client oracle).
- `VanillaEmitterParticlesOracle` — passes client+both; server-only
  failure is client packages (expected).
- `VanillaEntityParticlesOracle` — passes client+both; server-only
  failure is client packages (expected).
- `VanillaItemParticlesOracle` — passes all three classpaths.
- `VanillaEffectParticlesOracle` — passes all three classpaths.
- `VanillaEffectSyncOracle` — passes all three classpaths.

For comparison, the 26.2 run had only the marker oracle failing (B8-O01);
26.1 adds the providers oracle (B8-O02) because 26.1 predates the geyser
API that 26.2 already had. The 1.21.7 pre-repair list had Block 41
errors, Emitter 19, Entity 21, Item 3, Marker 42, Providers 116 — mostly
missing 26.x packages on the 1.21.7 classpath; only
`VanillaEffectParticlesOracle` passed. For 26.1 the 26.x layout is
native, so six of eight particle-family oracles compile as-is.

Static evidence for 26.1 (supporting the measured passes, not a substitute
for them):

- All 95 distinct `net.minecraft` references across the 8 particle
  oracles resolve as classes or packages in the 26.1 jars (server-game
  + client). No oracle references a 26.3-only class or a poplar /
  elder-guardian symbol by explicit import — the providers oracle's
  geyser references hide behind the `net.minecraft.core.particles.*`
  wildcard import, which is why only the compile catches them.
- The providers oracle's mechanism fits 26.1: it builds
  `new ParticleResources()`, reads the private `spriteSets` field by
  reflection, and calls the public `getProviders()` — all present in
  the 26.1 client jar (`ParticleResources` holds `registerProviders()`
  in 26.1; `ParticleEngine` has no such method). It resolves kinds by
  registry name, which is why the 5 non-26.1 kinds would throw
  (B8-H02).
- The 8 oracle APIs this batch's scope touches all exist (§3).
- The marker oracle's `bindBlockTags()` reads tags from the jar's
  datapack at runtime, so the absent `speleothems` tag is simply
  unbound — no code path reads it in 26.1.

## 5. Driver inventory

The plan (§4) claims 9 particle drivers; the tree has 7 particle-named
drivers plus 1 particle-adjacent driver = **8**. The 9th is unaccounted
for: possibly the plan counted `tests/test_cushion_particles.py` (a
Python unit test for Horizon's cushion feature, not a parity driver) or
double-counted. No 9th parity driver exists under any name in
`versions/26.1/tools/` (grep over all `test_*_parity.py` for
particle-oracle references finds only these).

| Driver | Oracle (26.1 status, MEASURED) | Probe | Status |
|---|---|---|---|
| block_particles | BlockParticles: passes client+both (server-only failure is client packages, expected) | block_particles_fixture.rs | UNRUN |
| emitter_particles | EmitterParticles: passes client+both (server-only failure is client packages, expected) | emitter_particles_probe.rs | UNRUN |
| entity_particles | EntityParticles: passes client+both (server-only failure is client packages, expected) | entity_particles_probe.rs | UNRUN; driver omits zombie_nautilus/camel_husk (B8-H04) |
| item_particles | ItemParticles: passes all three classpaths | item_particles_fixture.rs (horizon-world) | UNRUN |
| marker_particles | MarkerParticles: FAILS all three classpaths, 1 error (LocalPlayer ctor, B8-O01) | marker_particles_probe.rs | BLOCKED (B8-O01) — oracle does not compile; would also fail on spruce cases (B8-G03) once repaired; red_poplar_leaves zoo cases un-runnable (B8-H03) |
| native_client_particles | none (Horizon-only Host API contract, no vanilla oracle) | game fixture | UNRUN; out of vanilla scope |
| particle_providers | ParticleProviders: FAILS all three classpaths, 10 errors (geyser symbols, B8-O02) | particle_providers_probe.rs | BLOCKED (B8-O02) — oracle does not compile; would also FAIL on the 5 non-26.1 kinds — oracle throws `AssertionError("unknown particle …")` (B8-H02); does not test elder_guardian (B8-G01) or vibration (B8-H01) |
| effect_sync (adjacent) | EffectParticles: passes all three; EffectSync: passes all three | effect_sync_fixture.rs | UNRUN; scope is effect particle/visibility sync |

`parity.py` was not run (blocked — static checks only, no cargo).
Prepared 26.1 data exists (`.local/horizon-client/26.1/`, 190 MB, Batch
0 prepare, exit 0), so the data precondition for the driver runs in §8
is met — the runs themselves remain follow-up work. 6 of the 8 drivers
are runnable once their probes build; the other 2 wait on the B8-O01 /
B8-O02 oracle repairs plus the B8-H02/H03 trims.

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

Same 31 tests as the 1.21.7 and 26.2 batches (same names, same file).

## 7. Not covered

- **All driver numerics.** Per-tick provider physics (positions, motion,
  age, lifetime, size, colour, roll, sprite, UVs, light), emitter RNG
  draws, entity-burst counts/positions, block-particle placement, marker
  loop exactness — only the drivers can verify these, and none ran. The
  static checks above bound the *tables and mappings*, not the numerics.
- **Translucent-layer table.** `ParticleType::translucent_layer()` (which
  types vanilla renders in the translucent layer) was not verified — it
  needs per-class `getRenderType` checks across ~20 particle classes.
  (Same scope decision as the 1.21.7 and 26.2 batches.)
- **Sprite sets / particle textures.** Whether the engine resolves every
  type's sprite set from the 26.1 assets needs prepared data + a run.
- **Level events other than 1504 and 3002–3005.** `level_particles.rs`
  handlers (dragon egg, shulker/ender teleport, growth, turtle/sniffer
  eggs, torch burnout, dragon fireball, potion splashes,
  `level_event_at`) exist but their numerics are unverified; only
  3002–3005 are in the emitter driver. 1504's gate was verified
  statically (B8-G04) but its drip numerics need a run.
- **Apparition visual fidelity.** The elder-guardian re-implementation
  (§1) needs an in-game look or the providers driver.
- **The 9th driver.** The plan's count of 9 does not match the tree's 8;
  if a 9th driver exists somewhere else (another version tree, a renamed
  file), it was not found here.
- **Dripstone climb limit.** Vanilla's `findRootBlock` passes 11 to
  `findBlockVertical`; the port climbs 10 steps (`1..11`). Whether the
  off-by-one reading matches needs the marker driver run.
- **G1/G6 server-side status for 26.1.** Whether bubble columns form and
  dripstone transfers fluid server-side in the 26.1 port is Batch 5's
  scope, not this batch's.
- **`sounds.json` is genuinely absent** from the prepared data (the
  separate `horizon-prepare sounds` step was not run — Batch 0). Sound
  names in the particle paths were not checked against prepared sound
  data. `en_us.json` IS present (verified in
  `.local/horizon-client/26.1/assets/minecraft/lang/`).

## 8. What unblocks the rest

1. Prepared 26.1 data exists (Batch 0 — done); `sounds.json` remains
   absent, which does not block the particle drivers.
2. Repair the two broken oracles (follow-up, not this batch): B8-O01
   (the G2 LocalPlayer-constructor fix for
   `VanillaMarkerParticlesOracle`) and B8-O02 (delete the geyser
   branches + cast helpers from `VanillaParticleProvidersOracle`).
   Six of eight particle-family oracles already compile on their real
   classpath (measured, §4).
3. Fix B8-G01 (add the `ElderGuardian` variant + `ElderGuardianParticle`
   provider), B8-G03 (spruce leaves arm) and B8-G04 (the 1504 gate —
   match by name or by `PointedDripstoneBlock`, not `SpeleothemBlock`)
   on fix branches, then run `particle_providers` and
   `marker_particles`; fix B8-G02 (Flash arm in `hub_particle`) with
   them or separately. B8-G03 is driver-blocking; B8-G02 and B8-G04 are
   not covered by any driver; B8-G01 blocks nothing in the 26.1 tree
   because the driver skips the type — the fix should add
   `elder_guardian` to the driver's kind set too, or the type stays
   untested.
4. Trim the non-26.1 entries from the drivers for 26.1 (B8-H02: 5 kinds
   in the providers driver — 3 poplars + 2 sulfur; B8-H03:
   `red_poplar_leaves` in the marker zoo), add the missing coverage
   (B8-H01: vibration; B8-H04: zombie_nautilus, camel_husk), then run
   the drivers (`block`, `emitter`, `entity`, `item`, `effect_sync`);
   the item driver needs a longer cargo-build window than its 900 s
   timeout under load (or a warm target dir) — 1.21.7 precedent.
5. Run the 31 `particles.rs` unit tests (`cargo test -p
   horizon-client-render --lib particles::`, dev profile, `-j 1`).

## 9. Method notes

- The 26.1 **client** jar is Mojang-mapped —
  `net/minecraft/client/particle/ParticleEngine.class` is present by
  its mapped name — and its `ParticleTypes.class` is byte-identical
  (md5) to the server-game jar's. The leakage-audit 26.1.x server jars
  are bundler jars (never javap'd); the named game jars are
  `26.1.x/versions/26.1/jars/{client,server-game}.jar`.
- 26.x registration shapes differ from 1.21.7 in two places this batch
  had to adapt to:
  - `Blocks.<clinit>` registers via `register(BlockItemId, Function,
    Properties)` with the name from a `getstatic BlockItemIds.<NAME>`
    (no `ldc` name strings) — the class-file parser associates
    `BlockItemIds` fields with their `invokedynamic` lambdas instead of
    string constants.
  - Providers live in `ParticleResources.registerProviders()` (private),
    not `ParticleEngine` — and 26.1 has a dedicated
    `ElderGuardianParticle$Provider` where 1.21.7 used the shared
    `MobAppearanceParticle` provider.
- `javap -c` on `Blocks` was not used (1.21.7 warning: exceeds 280 s
  under throttle). The Python class-file reader
  (`evidence/parse_blocks.py`, constant pool + `BootstrapMethods` +
  bytecode walk with a full opcode-length table) resolves the leaves
  associations in ~2 s. Two gotchas found while writing it:
  `invokedynamic`'s operand is a constant-pool index to an
  `InvokeDynamic` entry (tag 18), which *then* holds the bootstrap
  index — reading it as the bootstrap index directly overflows the
  table; and `ldc2_w` (0x14) takes a 2-byte index, not 1.
- `javap` on the small classes ran 5–40 s each under load; batches of
  2–5 classes per exec kept the work moving.
- The providers oracle's 10 errors were reproduced with a single-file
  `javac --release 25` run against the 26.1 client classpath
  (client + server-game jars, all 26.1 libraries, jsr305); the 11th
  error in that run is the same-package `ParityData` helper, which the
  Batch 0 big compile resolves — the 10 geyser errors are the real
  breakage.
- No production code (`crates/`) was touched. No commits were made in
  this batch. Only the two deliverable files were written (plus
  evidence/ scratch: javap dumps, the registry census, the
  `ParticleTypes` `<clinit>` parse, and `parse_blocks.py`).
