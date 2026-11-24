# Batch 8 — Particles (1.21.10 parity sweep)

**Version:** 1.21.10 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.10`
(workdir `~/workspace/mcir-work/sweep-1.21.10`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 8 — all 115 particle types,
emitters, providers; 8 drivers found in the tree (see §10).

**Status: DRAFT — not committed.** Checks + triage only, per the batch
brief: no fixes, no edits under `versions/1.21.10/`, no cargo builds, no
commits. Everything below is static analysis: `javap` on the named
1.21.10 jars (`~/workspace/mcir-work/e6-deltas/versions/1.21.10/jars/{client-named,server-game-named}.jar`;
the `server.jar` in that dir is a Mojang bundler jar and was never
javap'd), direct bytecode parsing of `ParticleTypes.<clinit>` (via the
`javap -c` dump) and of `Blocks.<clinit>` (a class-file parser written
for this batch, `evidence/blocks_parse.py`), source comparison against
`versions/1.21.10/crates/`, data checks against the 1.21.10 datagen
reports (`e6-deltas/versions/1.21.10/datagen/reports/`, since the
worktree has no prepared `.local/`), and oracle triage from this
batch's own single-file `javac` measurement of the 8 particle-family
oracles — Batch 0's measured table had not landed when this batch ran
(b0's evidence dir held only jar listings at the time), so there is no
Batch 0 table to triage from; the javac measurement here uses the
same "both" classpath Batch 0 defines (both named jars + the 114 game
library jars + jsr305) and the same standalone definition (each file
compiled alone with `-sourcepath`). No driver runs, no cargo tests.
The coordinator commits these deliverables.

**Headline:** all three 1.21.7 items reproduce, and 15 of the 16
26.1.2 items reproduce — the ten behaviour defects (B8-G01–G10), three
of the four needs-harness items (B8-H01–H03) and both oracle defects
(B8-O01–O02) — with one WITHDRAWN (B8-H04: `zombie_nautilus` and
`camel_husk` do not exist in 1.21.10, so there is nothing for the
entity driver to add). The 1.21.10 tree is a template-derived delta
port of 1.21.11, not a 26.x re-port, so the reproductions are not
inheritance — each was verified fresh against 1.21.10's own jar/data,
and several differ in detail from the 26.1.2 form: the group cap
(B8-G05) is a different defect here (vanilla 1.21.10 evicts the
oldest particle past 16,384; the port drops new ones and thins
probabilistically), the providers driver carries 13 non-1.21.10 kinds
not 11 (B8-H02, two new ones), and both oracle defects are worse than
in 26.1.2 (B8-O01: 6 errors, not 1; B8-O02: 25 errors, not 10).
**New in this run:** five more broken particle oracles (B8-O03–O07) —
7 of the 8 particle-family oracles fail to compile for 1.21.10,
because the oracle sources are 26.3-form (26.x packages, the
`world.attribute` package, jspecify, 26.x constructors) — and a new
needs-harness item (B8-H05: the entity driver sends `nautilus` cases
for an entity 1.21.10 does not have). No new behaviour defects were
found beyond the re-checks: the port's particle tables are otherwise
exact (see §6.5).

## §0 Execution status

Complete. All 1.21.7 re-checks (3, PRIMARY) and all 26.1.2 re-checks
(16, SECONDARY) decided; no UNVERIFIED carry-overs. The Batch 7-routed
flag carried forward as B8-G10 (`/spreadplayers` always fails, major)
was verified fresh against 1.21.10's own jar/data and reproduces. No
stalls: every jar needed was present, every class needed was
javap'd fresh against the 1.21.10 jars, and no driver build was
attempted (out of scope for this batch). One caveat: Batch 0's
measured oracle table had not landed, so the oracle verdicts rest on
this batch's own javac measurement (method in §2, §9) rather than the
Batch 0 table — the coordinator should reconcile against the Batch 0
table when it lands; the error counts here are single-file
standalone compiles on the "both" classpath and may differ from
Batch 0's per-classpath attribution, but a file that fails standalone
on "both" fails on every classpath.

## §1 Scope

`docs/PARITY-SWEEP.md` §4 Batch 8: particle registry + type coverage,
wire codec, override limiter, entity event bursts, marker zoo, leaves,
dripstone + tags, oracle API existence. Method: registry comparison
(1.21.10's `particle_type` registry vs the port's `ParticleType`),
codec comparison, source comparison — plus fresh `javap` verification
of every vanilla claim against 1.21.10's own jars (no md5 carry from
another version: the 1.21.10 tree is a 1.21.11-template delta, not a
26.x tree, so no 26.x jar evidence carries). No cargo builds, no
driver runs. Driver-coverage gaps are needs-harness items (B8-Hxx),
not behaviour gaps.

Files read in the 1.21.10 tree (particle scope): `particle_type.rs`,
`animate_tick.rs`, `entity_data.rs`, `world_cmds.rs`,
`level_particles.rs`, `particles.rs`, `game.rs`, `remote.rs`,
`consume.rs`, `items.rs`, `teleport_consume.rs`, `mobs/enderman.rs`,
`mobs/shulker.rs`, `mobs/guardian.rs` (elder curse param),
`blocks.rs`, `mesher.rs`, `mob_blocks.rs`, `creatures.rs`,
`interact.rs`, `adapter/ids.rs`; all 8 particle drivers; all 8
particle oracles.

## §2 Method

- **Jar identity first.** 1.21.10's own named jars, sha1-verified
  against the common brief (client-named
  `715f58165acfaf8b61ab921064f08c202b41bd0c`, server-game-named
  `ba1fc89ead95b8ebdb8727f84e2bd487533c8b7f`). Every vanilla claim
  was javap-verified fresh against these jars — nothing here rests
  on a 26.x or 1.21.7 jar finding. Where a 1.21.7 finding was
  checked for carry, the class was md5-compared first:
  `LeavesBlock`, `TintedParticleLeavesBlock` and
  `UntintedParticleLeavesBlock` are byte-identical between 1.21.7
  and 1.21.10, but `Blocks.class` differs, so the leaf
  registrations were parsed fresh from the 1.21.10 jar.
- **Registry census from 1.21.10's own data.** Datagen
  `registries.json` (115 types, ids contiguous 0–114);
  `ParticleTypes.<clinit>` parsed from the `javap -c` dump (115
  registrations in protocol-id order, 28 limiter-true, 0 mismatches
  vs the datagen registry). `ParticleTypes.class` md5 is identical
  in both 1.21.10 jars.
- **Leaves from a class-file parser.** `Blocks.<clinit>` was parsed
  directly (`evidence/blocks_parse.py`): each leaf name's `ldc` was
  associated with its registration lambda through the
  `BootstrapMethods` table, and each lambda's constructed class +
  float constant read out. All 11 leaf blocks resolved; the azalea
  colour was read as the `ColorParticleOption.create` int argument
  (−9399763).
- **Oracle measurement by this batch.** Each of the 8
  particle-family oracles was compiled alone with `javac -nowarn`
  on the "both" classpath (both named jars + 114 game library jars
  + `tools/jsr305-3.0.2.jar`) with `-sourcepath` on the test java
  root — the same standalone definition Batch 0's fixpoint uses.
  Logs in `evidence/javac-*.log`. Error counts are the javac
  counts; root causes were verified against the jar listings
  (package absence) and `javap` (constructor arities).
- **26.1.2-record corrections applied** (from the 26.1.2 summary):
  B8-H02 counts the kinds the driver's `cases()` generator actually
  yields, not just the `SIMPLE` list; the oracle-API check
  "Villager.handleEntityEvent" is attributed to `LivingEntity`
  (the method is declared there, inherited by Villager); B8-G10
  (`/spreadplayers`) was checked fresh in 1.21.10.

## §3 Checks table (tally)

**Behaviour checks: 274 — 262 pass, 12 gap** (across 10 defect items,
B8-G01–B8-G10: 9 minor, 1 major). Counting rule: a behaviour check is
one comparison where BOTH the vanilla side (1.21.10 jar/data) and the
Rust side were actually compared; oracle-status checks are NOT counted
in the tally (per the common brief). Non-behaviour checks tallied: 0.
As-tallied 274 = 274 behaviour + 0 non-behaviour.

| Section | Total | Pass | Gap | Gaps |
|---|---|---|---|---|
| Registry / type coverage | 118 | 116 | 2 | B8-G01 (per-type coverage, kind set) |
| Wire codec | 20 | 19 | 1 | B8-G02 |
| Override limiter | 28 | 27 | 1 | B8-G01 (flag) |
| Entity event bursts | 27 | 27 | 0 | — |
| Marker zoo | 45 | 45 | 0 | — |
| Leaves | 12 | 11 | 1 | B8-G03 |
| Dripstone + tags | 7 | 6 | 1 | B8-G04 |
| Oracle API existence | 8 | 8 | 0 | — |
| Other passes | 2 | 2 | 0 | — |
| Level events 2015–2020 + digging | 5 | 1 | 4 | B8-G06, G07, G08, G09 |
| Particle group cap | 1 | 0 | 1 | B8-G05 |
| Routed check: /spreadplayers safety (B7 flag) | 1 | 0 | 1 | B8-G10 |
| **Total** | **274** | **262** | **12** | |

118+20+28+27+45+12+7+8+2+5+1+1 = 274; 262 + 12 = 274.

(Tally vs the 26.1.2 template's 279: −2 in registry — 115 types, not
117 — and −3 in entity bursts — the driver's 3 `nautilus` pairs have
no vanilla 1.21.10 side, so they are driver scope (B8-H05), not
behaviour checks.)

- **Oracle defects: 7** (B8-O01 — marker oracle, 6 errors;
  B8-O02 — providers oracle, 25 errors; B8-O03 — block oracle, 8
  errors; B8-O04 — emitter oracle, 5 errors; B8-O05 — entity oracle,
  18 errors; B8-O06 — item oracle, 4 errors; B8-O07 — effect-sync
  oracle, 26 errors). Only `VanillaEffectParticlesOracle` compiles
  (measured, §9).
- **Needs-harness items: 4 live + 1 closed** (B8-H01–H03, H05, §4;
  B8-H04 closed as WITHDRAWN, §5): driver-coverage gaps, not
  behaviour gaps.
- **Drivers: 0 run, 8 inventoried** (7 particle drivers + 1
  particle-adjacent effect-sync driver). 7 of the 8 are
  oracle-blocked (B8-O01–O07); the 8th (`native_client_particles`)
  has no vanilla oracle. None is runnable as-is.
- **Unit tests:** 34 `#[test]`s in `particles.rs` exist (26 vanilla +
  8 mod-particle); unrun, blocked.

## §4 Defects

### B8-G01 — `elder_guardian` has no port implementation at all (MINOR, REPRODUCES from 1.21.7 and 26.1.2)

**Checks failed (3):** per-type end-to-end coverage for
`elder_guardian`; its `override_limiter` flag; the providers-driver
kind set (which omits it).

**Evidence (jar, fresh for 1.21.10).** Vanilla 1.21.10 registers
`ELDER_GUARDIAN` as a `SimpleParticleType` (`ParticleTypes.<clinit>`,
wire id 17, `overrideLimiter=true` — parsed from the `javap -c` dump
of the 1.21.10 client jar) and registers a provider for it in
`ParticleResources.registerProviders()` — a dedicated
`ElderGuardianParticle$Provider` (offsets 296–300, same as 26.x;
1.21.7 used the shared `MobAppearanceParticle` provider, which does
not exist in 1.21.10).
`ClientPacketListener.handleGameEvent` spawns it at the player's
position on game event `GUARDIAN_ELDER_EFFECT` (offset 544;
`addParticle(ELDER_GUARDIAN, ...)` at offset 554, plus the curse
sound at 598).

**The port.** `ParticleType` (`horizon-client-core/src/particle_type.rs`)
has 125 variants and no `ElderGuardian` (grep over
`particle_type.rs`, `lan_particles.rs`, `particles.rs`: zero hits).
Consequences, all verified in source:

- `received_particle` (`game/lan_particles.rs:469`, `from_name` at
  :488) resolves a server-sent particle by name and returns `None`
  for `elder_guardian` — a `LevelParticles` packet for it is silently
  dropped.
- `/particle minecraft:elder_guardian` fails with `particle.notFound`
  (`game/commands/world_cmds.rs:718`, `from_name` returns `None`);
  vanilla accepts it.
- `override_limiter()` (`particle_type.rs:152-189`) has no entry for
  it (28 vanilla limiter types vs 27 real-1.21.10 Rust entries; the
  only 1.21.10 miss is `elder_guardian` — the Rust list's other 4
  entries are the geyser types, dead in 1.21.10).
- The providers driver does **not** send `elder_guardian` cases in
  the 1.21.10 tree (its 126-kind set omits the type entirely), so
  unlike 1.21.7 — where the driver sent 12 cases and the probe
  printed `no provider` — the driver is not blocked by this defect;
  it silently skips the type.

**Mitigation (why minor).** The one in-game path that matters — the
elder guardian's curse apparition — is re-implemented outside the
particle engine, same as in 1.21.7 and 26.1.2: game event 10 sets
`elder_apparition` (`game/remote.rs:4990`,
`game/creatures.rs:4187`) and `elder_apparition_geometry`
(`game/mob_blocks.rs:96`) draws the ghostly guardian with the vanilla
30-tick lifetime (`APPARITION_TICKS = 30`, `mob_blocks.rs:87`). So
the apparition works; what is missing is the particle-type plumbing
(`/particle`, server packets). Visual fidelity of the
re-implementation against vanilla's `ElderGuardianParticle` is
unverified statically — needs the driver run or an in-game look.

### B8-G02 — `/particle minecraft:flash` sends a malformed packet to LAN guests (MINOR, REPRODUCES from 1.21.7 and 26.1.2)

**Checks failed (1):** the `Flash` arm of `hub_particle`.

**Evidence.** `Particle::read` (`horizon-protocol/src/entity_data.rs:185`,
State ids :214, Color ids :215) decodes wire id 42 (`flash`, verified
against the 1.21.10 registry) as `Color(i32)` — a flash particle on the
wire always carries a colour. But `hub_particle`
(`game/lan_particles.rs:411`), which builds the packet a LAN host
sends, has no `Flash` arm in its `Color` match (`EntityEffect |
TintedLeaves` only, :437), so a flash particle is built with `data:
None`, and `Particle::write` — which writes data by presence, not by
id — emits the id with no colour bytes. Any guest reading id 42 then
consumes the next 4 bytes of the stream as the colour and continues
misaligned: packet desync (and likely a disconnect or a dropped
connection) for every guest in range.

**Reachability (narrow — why minor).** The only path that sends `Flash`
through `hub_particle` is `/particle minecraft:flash`: `cmd_particle`
builds `ParticleOptions::simple(ty)` for it via the catch-all
(`world_cmds.rs:829`; vanilla's command requires the colour option,
the port does not) and calls `network_particle` (:883). Everywhere
else `Flash` is created client-side only (the firework explosion
flash, `particles.rs:3785`, via `self.create(...)`, never sent over
the network). The local player is unaffected (draws directly, never
reads the packet). So the trigger is exactly: a LAN host runs
`/particle minecraft:flash` with guests in range.

### B8-G03 — Spruce leaves never drop falling-leaf particles (MINOR, REPRODUCES from 1.21.7 and 26.1.2, driver-blocking)

**Checks failed (1):** the spruce entry of the per-block leaves check.

**Evidence (jar, fresh for 1.21.10).** Vanilla 1.21.10 registers
`SPRUCE_LEAVES` as a `TintedParticleLeavesBlock` with
`leafParticleChance = 0.01` — parsed fresh from the 1.21.10
`Blocks.<clinit>` with this batch's class-file parser
(`evidence/blocks_parse.py`: `ldc "spruce_leaves"` →
`lambda$static$36` → `new TintedParticleLeavesBlock`, float `0.01F`).
`Blocks.class` differs from 1.21.7's, so this is a fresh parse, not a
carry. `LeavesBlock.animateTick` calls
`makeFallingLeavesParticles`, which rolls `nextFloat() <
leafParticleChance` and spawns the falling leaf unless the block
below is face-full. So vanilla spruce drops tinted leaves at 1% per
animateTick, like oak.

**The port.** `classify` (`animate_tick.rs:151`) maps
`"spruce_leaves"` to `Animate::Leaves(None, None, false)` — no
falling particle at all. Every other leaf block matches vanilla
(see §8, leaves table).

**Driver note.** The marker driver's own zoo comment says "the spruce
that never falls" (`test_marker_particles_parity.py:114`) — the
comment encodes the same wrong belief, but the driver compares against
the real vanilla oracle, so the driver is correct as a check and
**would fail on the spruce cases** (oracle emits tinted leaves, probe
emits none — the probe runs the port's real `animate_tick`,
`marker_particles_probe.rs:168`) once B8-O01 is repaired — the driver
is currently oracle-blocked, not merely unrun. The comment needs
correcting when the fix lands; the driver needs no change.

### B8-G04 — Level-event 1504 stalactite drip particle never fires: its gate checks for `SpeleothemBlock`, which does not exist in 1.21.10 (MINOR, REPRODUCES from 26.1.2)

**Checks failed (1):** the level-event 1504 gate in the dripstone
section.

**Evidence (jar + datapack, fresh).** Vanilla 1.21.10 has no
`SpeleothemBlock` class at all (0 entries in either jar;
`PointedDripstoneBlock` extends `Block` directly) and no
`speleothems` tag in the client jar's datapack (205 block tags, none
of them `speleothems`). Vanilla 1.21.10's `LevelEventHandler` routes
1504 to `PointedDripstoneBlock.spawnDripParticle(level, pos, state)`
(offset 3519, no class gate), and vanilla's
`PointedDripstoneBlock.maybeTransferFluid` sends level event 1504
when a stalactite drips fluid into a cauldron below (offsets 218 and
245 in `dripstone.txt`).

**The port.** The client routes 1504 to `dripstone_drip_particle`
(`game.rs:4388`), whose first act is the `is_stalactite` gate
(`game/level_particles.rs:243-244`): `has_class("SpeleothemBlock")`
(`horizon-items/src/blocks.rs:121`, an exact match over the prepared
class chain). In 1.21.10 that is false for every block, so the
function returns before spawning anything — the cauldron-fill drip
particle never appears. The ambient animateTick drip path is
unaffected: it is name-based (`AnimateTables.dripstone`, built from
`"pointed_dripstone"` by name, `animate_tick.rs:397-410`) and matches
vanilla (§8).

**Scope note.** Whether the port's *server* actually transfers fluid
to cauldrons in 1.21.10 (pilot G6's 1.21.10 status) is Batch 5's scope;
the client gate is dead regardless of server reachability, and the
defect is minted on the client side only. The same `SpeleothemBlock`
class assumption appears elsewhere in the tree (see §7) — those are
other batches' scope.

### B8-G05 — Particle engine drops new particles at the 16,384 cap (with probabilistic thinning above 12,288); vanilla 1.21.10 evicts the oldest instead (MINOR, REPRODUCES from 26.1.2 with a different vanilla form)

**Checks failed (1):** the pending-particle drain in `tick_tracking`.

**Evidence (jar, fresh).** Vanilla 1.21.10's `ParticleGroup.add`
(`javap -c`, 1.21.10 client jar) is a plain queue add —
`particles.add(p)`, no size check, no probability roll — but the
queue itself is an `EvictingQueue.create(16384)` (constructor,
offset 5–8): adding past 16,384 evicts the *oldest* particle. There
is no probabilistic thinning in the last quarter and no
level-random draw on add. This differs from 26.1.2, where the queue
was plain and no cap was enforced at all; in 1.21.10 a cap exists
but its mechanism is eviction, not rejection.

**The port.** `tick_tracking` (`horizon-client-render/src/particles.rs:3888`,
the drain that moves every created particle from `pending` into the
live list — `create()` pushes to `pending`, :2098) enforces the
26.2-form cap (:3933-3945): at 16,384 live particles every new
particle is dropped (`if size >= MAX_PARTICLES { continue; }`), and
above 12,288 new particles are dropped with probability
`1 - ((16384-size)/4096)^2`, drawing the level random for each
rejection. The constant is declared at :1358-1359
(`MAX_PARTICLES: usize = 16384`), and the comment
("`ParticleGroup.add`: the last quarter fills with decreasing
probability") names the 26.2 form.

**Reachability (why minor).** The cap only bites in particle-heavy
scenes (above 12,288 live particles: big explosions, dense farms,
heavy `/particle` use). Below that threshold the port matches
vanilla exactly. When it bites, the port keeps old particles and
drops new ones where vanilla keeps new ones and drops old ones, and
the rejection draws consume level-random draws vanilla never makes
(RNG-order divergence for everything downstream in the tick).

### B8-G06 — Chorus fruit shows an extra 2017 directional teleport trail vanilla never sends (MINOR, REPRODUCES from 26.1.2; 26.3-form)

**Checks failed (1):** the level-event 2017 check.

**Evidence (jar, fresh).** Vanilla 1.21.10's
`TeleportRandomlyConsumeEffect` is a record with ONE field
(`diameter: float`; constructors `()` and `(float)`) — there is no
`directional_particles` field. Its `apply` calls
`LivingEntity.randomTeleport(x, y, z, true)`, which broadcasts
entity event 46 (`bipush 46`, offset 177 in `living.txt`);
`LivingEntity.handleEntityEvent` case 46 spawns 128 portal particles
at the entity (`sipush 128`, offset 193; `PORTAL` at 381,
`addParticle` at 399). Vanilla 1.21.10's `LevelEventHandler` has NO
2017 case (79-case lookupswitch, 2000–2013 then 3000+). No level
event is involved anywhere in the path.

**The port.** The 1.21.10 tree carries the 26.3 form:
`TeleportConsume` has a `directional_particles: bool` field
(`horizon-world/src/teleport_consume.rs:6`); `items.rs:157-160` reads
it from data with `unwrap_or(true)` — and the 1.21.10 data never
carries the field (`chorus_fruit`'s consumable is
`{"on_consume_effects": [{"type": "minecraft:teleport_randomly"}]}`,
0 occurrences of `directional_particles` in the datagen
`items.json`), so it defaults to true;
`teleport_consume.rs:109-121` then fires the event on every
successful teleport, and the client impl (`game/consume.rs:422-431`)
calls `level_event_at(2017, ...)`, which routes (`game.rs:4349-4352`)
to `consume_teleport_particles` → `block_teleportation(pos, data,
[127,127,127])` (`particles.rs:1859`) — 128 portal particles
interpolated along the old→new line. The port ALSO fires the
vanilla-correct entity event 46 (`consume.rs:188-194`,
`remote.rs:4774-4777` → `entity_teleport_particles`, 128 portal
particles at the entity). So a chorus-fruit teleport in the 1.21.10
port shows BOTH the correct 128-particle burst AND a second,
128-particle directional trail vanilla never shows. (Batch 7
independently flagged the 2017 trail as masked by its G09 in the
26.1.2 run — the trail fires only after a successful teleport; the
masking claim is Batch 7's, not verified here. The trail mechanism
itself is verified fresh here.)

### B8-G07 — Enderman teleport shows an extra 2018 directional portal trail vanilla never sends (MINOR, REPRODUCES from 26.1.2; 26.3-form)

**Checks failed (1):** the level-event 2018 check.

**Evidence (jar, fresh).** Vanilla 1.21.10's `EnderMan.teleport`
calls `randomTeleport(x, y, z, true)` (offset 110) → entity event 46
→ 128 portal particles at the entity. The only `addParticle` call in
`EnderMan` is the ambient aiStep emission (offset 90), not the
teleport. `LevelEventHandler` has NO 2018 case.

**The port.** The 1.21.10 tree's `teleport_to`
(`mobs/enderman.rs:160-190`) pushes BOTH the vanilla-correct entity
event 46 (:160-166) AND `MobEvent::LevelEvent { id: 2018, ... }`
(:174-184, when not silent), which routes (`game.rs:4354-4357`) to
`enderman_teleport_particles` →
`enderman_teleportation(pos, data, particle_level)`
(`particles.rs:1884`) — 128 portal particles along the old→new
line. Every enderman teleport in the 1.21.10 port shows the correct
burst plus a second, wrong trail. (The destination-selection half of
the enderman rework is Batch 3's scope; only the particle-visible
trail is minted here.)

### B8-G08 — Shulker teleport shows a 2016 directional trail; vanilla shows NO particles at all (MINOR, REPRODUCES from 26.1.2; 26.3-form)

**Checks failed (1):** the level-event 2016 check.

**Evidence (jar, fresh).** Vanilla 1.21.10's
`Shulker.teleportSomewhere` does: `unRide`, `setAttachFace`,
`playSound(SHULKER_TELEPORT)`, `setPos`, `gameEvent(TELEPORT)`, set
peek data, `setTarget`. No `randomTeleport`, no `addParticle`, no
`broadcastEntityEvent`, no level event anywhere in the class (0 such
references in the full `javap -c` dump of the 1.21.10 server-game
jar). A shulker teleport in vanilla 1.21.10 shows zero particles.
`LevelEventHandler` has NO 2016 case.

**The port.** `mobs/shulker.rs:366-371` pushes
`MobEvent::LevelEvent { id: 2016, ... }` on every teleport, and the
client handles it (`game.rs:4344-4347` →
`shulker_teleport_particles` → `block_teleportation(pos, data,
[8,8,8])`, `particles.rs:1859`), so every shulker teleport shows 128
portal particles along the old→new line that vanilla never shows.
Note the asymmetry with G06/G07: here there is no correct event-46
burst underneath — the trail is the ONLY particle output, and it is
entirely wrong.

### B8-G09 — Digging shows a crack particle (and a mistimed hit sound) vanilla never shows (MINOR, REPRODUCES from 26.1.2; 26.2-form breaking effects)

**Checks failed (1):** the digging crack-particle check (level
events 2019/2020 area).

**Evidence (jar, fresh).** Vanilla 1.21.10's `MultiPlayerGameMode`
never calls `addBreakingBlockEffect` anywhere in the class (0
occurrences in the full `javap -c` dump), and `LevelEventHandler`
has NO 2019/2020 cases. `startDestroyBlock` plays nothing and
spawns nothing. `continueDestroyBlock` plays ONLY the hit sound, on
destroy ticks 0, 4, 8, … (`destroyTicks % 4.0f == 0`, offsets
199–208; `getSoundType` at 212, volume `/ 8.0`, pitch `* 0.5`,
offsets 243/252) — no particle at any point while digging.
(`ClientLevel.addBreakingBlockEffect` exists as a method but is
never invoked on this path.)

**The port.** The 1.21.10 tree keeps the 26.2 form:
`breaking_block_effects` (`game/level_particles.rs:353-373`: hit
sound when `play_sound`, plus `crack_particle(pos, face, state)` —
one crumb on the mined face), called from `startDestroyBlock`
(`game/remote.rs:8172`, `breaking_block_effects(hit.pos, hit.face,
true)`) with the comment "startDestroyBlock plays the first hit and
crack itself; the server's level events 2019 and 2020 do the rest".
But the port's server never sends 2019/2020 (no sends in
`horizon-world` or `game/`), so the network dispatch
(`game.rs:4309-4317`) is dead and the continue path plays NOTHING —
no every-fourth-tick sound at all. Net effect vs vanilla: an extra
crack particle + a hit sound at the first hit (vanilla: silence),
then silence for the rest of the dig (vanilla: a hit sound every 4th
tick). The particle half (extra crumb) is minted here; the
sound-schedule half is flagged for Batch 10 (§7).

### B8-G10 — `/spreadplayers` always fails: `spread_safe` requires a tag 1.21.10 does not have (MAJOR, REPRODUCES from 26.1.2; Batch 7-routed flag, verified fresh)

**Checks failed (1):** the `/spreadplayers` safety check (routed
from Batch 7 §8.6 / slice `out_of_scope_failures` in the 26.1.1 run).

**Evidence (jar + data, fresh).** Vanilla 1.21.10's
`SpreadPlayersCommand.Position.isSafe` (`javap -c`, 1.21.10
server-game jar) is `y < maxY && !state.liquid() &&
!state.is(BlockTags.FIRE)` — three conditions, no teleport tag.
The `#entities_can_teleport_to` tag does not exist in 1.21.10: 0
entries in either jar's datapack (205 block tags in the client jar,
none teleport-related).

**The port.** `spread_safe` (`game/commands/world_cmds.rs:2091-2114`)
returns `y < max_y && !liquid && teleportable`, where `teleportable`
(:2100-2109) requires membership in `#entities_can_teleport_to` —
always false in 1.21.10. So `spread_safe` is ALWAYS false. In the
spread loop (:1978-1986), every collision-free iteration
re-randomizes every point (none is ever "safe"), `collisions` stays
true, and the loop runs until `iteration >= 10_000`, when the command
fails with `commands.spreadplayers.failed.entities` (or `.teams`,
:1991-1999). **`/spreadplayers` cannot succeed in the 1.21.10 port
on any terrain.** Commands-area; minted here on the coordinator's
routing via Batch 7 — the merge may re-route it to Batch 9's ledger.

### Needs-harness items (driver coverage, not behaviour gaps)

- **B8-H01 — providers driver omits `vibration` (REPRODUCES).** The
  port implements vibration end to end (`Particle::read` id 48,
  `hub_particle` arm, `make()` arm, `ParticleOptions::vibration`),
  and the oracle's `options()` handles it
  (`VanillaParticleProvidersOracle.java:155-157`, the `VIBRATION`
  branch), but the driver's 126-kind set never sends a vibration
  case — so the vibration provider's numerics have no driver
  coverage. Add vibration cases (with a target + duration) to
  `test_particle_providers_parity.py`.
- **B8-H02 — providers driver includes 13 kinds 1.21.10 lacks
  (REPRODUCES, count 13, not 11).** The driver's `SIMPLE` list
  carries `red/orange/yellow_poplar_leaves` (26.3-only),
  `sulfur_bubbles` and `sulfur_cube_goo` (26.2-only) AND
  `pause_mob_growth` and `reset_mob_growth` (not particle types in
  1.21.10 at all — 7 kinds in `SIMPLE`), and the driver's `cases()`
  generator also yields `noxious_gas`, `noxious_gas_cloud`,
  `geyser`, `geyser_plume`, `geyser_base`, `geyser_poof`
  (`test_particle_providers_parity.py:94-105`, 6 more kinds, 72+
  cases). None of the 13 is in the 1.21.10 registry (115 types, no
  geyser/sulfur/noxious/poplar; `pause_mob_growth` /
  `reset_mob_growth` are not in the registry either — and not in the
  port's `ParticleType` enum, so the probe has no provider for them
  either). The oracle resolves kinds by registry name and throws
  `AssertionError("unknown particle " + id)` for an unregistered
  type (`VanillaParticleProvidersOracle.java:131`), so the driver
  cannot run against the 1.21.10 oracle as-is: it would die on the
  first such case. Trim all 13 kinds for 1.21.10.
- **B8-H03 — marker driver zoo includes `red_poplar_leaves`
  (26.3-only) (REPRODUCES).** The zoo (`test_marker_particles_parity.py`,
  `zoo_states()`, :118) sends
  `red_poplar_leaves[distance=3,...]` cases, but no such block exists
  in 1.21.10 — the only zoo name absent from the 1.21.10
  `blocks.json` (1,166 blocks) — so the oracle cannot build that
  world. (The 1.21.10 port DOES have poplar `classify` arms
  (`animate_tick.rs:144-146`), but they are dead: no poplar block
  exists to classify.) Trim the zoo case for 1.21.10.
- **B8-H04 — WITHDRAWN, closed (see §5).** The 26.1.2 item (entity
  driver omits live entities `zombie_nautilus` and `camel_husk`)
  has no 1.21.10 premise: neither entity exists in 1.21.10's entity
  registry (153 types; no `nautilus`, no `zombie_nautilus`, no
  `camel_husk`). The driver's omission is correct for 1.21.10.
- **B8-H05 — NEW: entity driver includes `nautilus`, which 1.21.10
  lacks (needs-harness).** The driver's `EVENTS` table
  (`test_entity_particles_parity.py:17`) sends `nautilus: [6, 7,
  18]` — 3 pairs for a 26.x-only entity that is not in 1.21.10's
  entity registry (153 types, no nautilus) and has no class in
  either 1.21.10 jar. The oracle cannot build a nautilus (its
  `Nautilus` import is one of B8-O05's 18 errors), so the driver
  cannot run those cases as-is. (The port's `entity_event_burst`
  also carries dead `nautilus` / `zombie_nautilus` / `camel_husk`
  arms — unreachable from a 1.21.10 server, not a behaviour gap.)
  Trim the nautilus entries for 1.21.10.

### Oracle defects

- **B8-O01 — `VanillaMarkerParticlesOracle` does not compile for
  1.21.10 (REPRODUCES, 6 errors, not 1).** This batch's javac
  measurement (single-file, "both" classpath, log
  `evidence/javac-VanillaMarkerParticlesOracle.log`): 6 errors.
  The 26.1.2 break reproduces — :44, the oracle's `Player` subclass
  calls `super(...)` with 9 arguments, the 26.3 form, but 1.21.10's
  `LocalPlayer` constructor takes **7** (`Minecraft, ClientLevel,
  ClientPacketListener, StatsCounter, ClientRecipeBook, Input,
  boolean` — no `ChatAbilities`, which is 26.x) — plus three
  references to the 26.x package `net.minecraft.world.attribute`
  (:104, :116, :233; 1.21.10 keeps attributes under
  `net.minecraft.world.entity.ai.attributes`), a dead `@Override`
  at :116, and a missing `bindComponents(DataComponentMap)` at
  :218. Blocks the marker_particles driver entirely (before it can
  reach the B8-G03 spruce cases).
- **B8-O02 — `VanillaParticleProvidersOracle` does not compile for
  1.21.10 (REPRODUCES, 25 errors, not 10).** This batch's javac
  measurement: 25 errors. The 26.1.2 geyser break reproduces — 10
  of the 25 are the geyser references (`GeyserParticleOptions` ×3,
  `GeyserBaseParticleOptions` ×3, `GEYSER`/`GEYSER_PLUME`/
  `GEYSER_BASE`/`GEYSER_POOF` ×4; :143-146, :164-165; zero `geyser`
  entries in either 1.21.10 jar) — plus 26.x-only symbols the
  26.1.2 run never hit: `BlockStateModelSet` ×2, `ItemStackTemplate`
  ×2, `Nullable` ×4 (the `org.jspecify.annotations` import at :46 —
  no jspecify jar exists in 1.21.10's game libraries), the
  `net.minecraft.client.resources.model.sprite` package (:22; absent
  from 1.21.10 — sprites live in `client.renderer.texture`),
  `Material` ×3 (:115, :116, :124), and a `TextureAtlasSprite`
  constructor mismatch (:107; 1.21.10's ctor is the 6-arg protected
  form). Blocks the particle_providers driver entirely. Repair is
  bigger than the 26.1.2 deletion (geyser branches + cast helpers);
  the sprite/Material/jspecify breaks need 1.21.10-form rewrites.
  Do not repair here.
- **B8-O03 — NEW: `VanillaBlockParticlesOracle` does not compile
  for 1.21.10 (8 errors).** `BlockStateModelSet` ×2 (:13, :44),
  the `net.minecraft.client.resources.model.sprite` package (:17),
  `Material` ×3 (:45, :47), a `TextureAtlasSprite` constructor
  mismatch (:42), a dead `@Override` (:47). All 26.x-form. Blocks
  the block_particles driver entirely.
- **B8-O04 — NEW: `VanillaEmitterParticlesOracle` does not compile
  for 1.21.10 (5 errors).** The `net.minecraft.world.entity.animal.cow`
  package (:23; absent from 1.21.10 — `Cow` is at
  `net.minecraft.world.entity.animal.Cow`) and the `Cow` symbol ×4
  (:120, :126). Blocks the emitter_particles driver entirely.
- **B8-O05 — NEW: `VanillaEntityParticlesOracle` does not compile
  for 1.21.10 (18 errors).** Ten bad 26.x package imports (:17-26:
  `animal.cow`, `animal.dolphin`, `animal.equine` ×2,
  `animal.feline` ×2, `animal.nautilus`, `animal.parrot`,
  `npc.villager` — all absent from 1.21.10; the classes live in
  `world.entity.animal.*` / `world.entity.npc.*` directly, and
  `Nautilus` does not exist at all) plus 8 missing-symbol errors
  for the entity classes (:47-50). Blocks the entity_particles
  driver entirely.
- **B8-O06 — NEW: `VanillaItemParticlesOracle` does not compile
  for 1.21.10 (4 errors).** A `ServerLevel` constructor mismatch
  (:28) and three missing symbols (:29, :31: `bindComponents`,
  `count()`, `item()` — the 26.x item-template API). Blocks the
  item_particles driver entirely.
- **B8-O07 — NEW: `VanillaEffectSyncOracle` does not compile for
  1.21.10 (26 errors).** The `net.minecraft.world.entity.monster.zombie`
  package (:100; absent — `Zombie` is at
  `net.minecraft.world.entity.monster.Zombie`), a cascade of 21
  errors from the unresolvable `Zombie` superclass (:102-110:
  dead `@Override`s, `super` from a static context), a `ServerLevel`
  constructor mismatch (:113), and missing effect methods (:148:
  `setGlowingTag`, `getSharedFlag`, `hasEffect`, `isCurrentlyGlowing`,
  `removeEffectParticles`, `setInvisible`, `setSharedFlag`,
  `updateInvisibilityStatus`). Blocks the effect_sync driver
  entirely (its other oracle, `VanillaEffectParticlesOracle`,
  compiles — see §9).

## §5 Re-check verdicts

### 1.21.7 items (PRIMARY — all 3 decided)

| 1.21.7 item | Verdict | 1.21.10 item |
|---|---|---|
| B8-G01 elder_guardian | **REPRODUCES** | B8-G01 (fresh jar evidence: `<clinit>` id 17/limiter, `ElderGuardianParticle$Provider`, game-event offsets re-read from the 1.21.10 jar; provider is the dedicated class here, not 1.21.7's shared `MobAppearanceParticle`) |
| B8-G02 flash | **REPRODUCES** | B8-G02 (`entity_data.rs` ids verified fresh against the 1.21.10 registry — flash is id 42 here, 41 in 1.21.7; `hub_particle` still has no Flash arm) |
| B8-G03 spruce | **REPRODUCES** | B8-G03 (`Blocks.<clinit>` parsed fresh — `Blocks.class` differs from 1.21.7's; spruce is `TintedParticleLeavesBlock(0.01F)` in 1.21.10 too) |

All three recur unchanged, as they did in 26.2, 26.1, 26.1.1 and
26.1.2.

### 26.1.2 items (SECONDARY — all 16 decided)

| 26.1.2 item | Verdict | 1.21.10 item |
|---|---|---|
| B8-G01 elder_guardian | **REPRODUCES** | B8-G01 (fresh jar evidence; provider is the dedicated `ElderGuardianParticle$Provider`, same as 26.1.2) |
| B8-G02 flash | **REPRODUCES** | B8-G02 (flash id 42 in both; `hub_particle` still has no Flash arm) |
| B8-G03 spruce | **REPRODUCES** | B8-G03 (`Blocks.<clinit>` parsed fresh for 1.21.10) |
| B8-G04 1504 gate | **REPRODUCES** | B8-G04 (gate unchanged; no `SpeleothemBlock` class or `speleothems` tag in 1.21.10; 1504 case verified fresh) |
| B8-G05 group cap | **REPRODUCES** (different vanilla form) | B8-G05 (vanilla 1.21.10 caps by eviction via `EvictingQueue(16384)`, not by rejection; the port's drop-new + probabilistic thinning matches neither 26.1.2's plain queue nor 1.21.10's eviction) |
| B8-G06 2017 chorus | **REPRODUCES** | B8-G06 (`teleport_consume.rs`/`items.rs`/`consume.rs` carry the 26.3 form; `TeleportRandomlyConsumeEffect` record + no-2017 verified fresh; 0 `directional_particles` in 1.21.10 `items.json`) |
| B8-G07 2018 enderman | **REPRODUCES** | B8-G07 (`mobs/enderman.rs` pushes 46 + 2018; `EnderMan.teleport` → event 46 verified fresh) |
| B8-G08 2016 shulker | **REPRODUCES** | B8-G08 (`mobs/shulker.rs` + `game.rs` keep the 2016 send + handler; `Shulker` has 0 particle/event refs, verified fresh) |
| B8-G09 digging crack | **REPRODUCES** | B8-G09 (`level_particles.rs`/`remote.rs` keep the 26.2 form; `MultiPlayerGameMode` 0 `addBreakingBlockEffect` calls + 4th-tick sound verified fresh) |
| B8-G10 /spreadplayers | **REPRODUCES** | B8-G10 (`world_cmds.rs` requires the absent tag; `isSafe` = `y < maxY && !liquid && !is(FIRE)` verified fresh; tag absent from both jars) |
| B8-H01 vibration omitted | **REPRODUCES** | B8-H01 (driver 126-kind set re-counted, vibration absent; oracle handles it) |
| B8-H02 11 non-26.1 kinds | **REPRODUCES** (count 13) | B8-H02 — 13 kinds (7 in SIMPLE incl. new `pause_mob_growth`/`reset_mob_growth` + 6 in `cases()` sections, re-counted fresh) |
| B8-H03 red_poplar zoo | **REPRODUCES** | B8-H03 (zoo re-counted: 37 distinct names incl. red_poplar; only red_poplar absent from 1.21.10's `blocks.json`) |
| B8-H04 zombie_nautilus/camel_husk | **WITHDRAWN** | B8-H04, closed (neither entity exists in 1.21.10 — no `nautilus`, no `zombie_nautilus`, no `camel_husk` in the 153-type entity registry or either jar; the driver's omission is correct) |
| B8-O01 marker oracle | **REPRODUCES** (6 errors, not 1) | B8-O01 (this batch's javac measurement + fresh LocalPlayer javap: 7-arg ctor, not 8) |
| B8-O02 providers oracle | **REPRODUCES** (25 errors, not 10) | B8-O02 (this batch's javac measurement + fresh geyser-absence check; 10 geyser errors + 15 more 26.x-form errors) |

### Batch 7-routed flag

`/spreadplayers` tag read (`world_cmds.rs:2100-2109`): **VERIFIED
REAL again, minted as B8-G10** (major). Vanilla 1.21.10's `isSafe`
checks `y < maxY && !liquid && !is(FIRE)`; the port requires the
absent `#entities_can_teleport_to` tag, so the command always fails
after 10,000 iterations.

### New items minted in this run (not re-checks)

- B8-H05 (entity driver includes `nautilus`, absent from 1.21.10).
- B8-O03–O07 (five more particle oracles that do not compile for
  1.21.10: block, emitter, entity, item, effect-sync).

## §6 Systemic findings

1. **The 1.21.10 tree carries the same five 26.2/26.3-form
   reversions as the 26.1.2 tree, by template inheritance, not by
   delta.** B8-G05 (cap), B8-G06 (2017), B8-G07 (2018), B8-G08 (2016
   handler), B8-G09 (breaking effects) are all the same story: the
   1.21.11 era template shipped the newer form, and the 1.21.10
   delta port carries it on unchanged. The 26.1 tree's own comments
   remain the repair reference for all five (the cap removal, the
   `breaking_block_sound` form, the absence of the 2016/2017/2018
   handlers and senders) — with one correction for 1.21.10: the cap
   repair is NOT "remove the cap" (the 26.1 fix), it is "evict the
   oldest" (vanilla 1.21.10's `EvictingQueue` form).
2. **The directional-teleport events are one feature, three
   defects.** 2016/2017/2018 share the `block_teleportation` /
   `enderman_teleportation` draw path and the 26.3 design (a 128
   portal-particle trail along the old→new line). They differ in
   severity of wrongness: 2017 and 2018 ADD a trail on top of the
   correct event-46 burst (double particles); 2016 REPLACES nothing
   — vanilla shows no particles at all for a shulker teleport, so
   the trail is pure invention. A single fix (remove the three
   handlers + their senders) closes all three.
3. **The 1.21.10 tree's level-event surface is wider than vanilla's
   in exactly the 2015–2020 band.** Vanilla 1.21.10's
   `LevelEventHandler` handles 79 event ids; 2015–2020 are all
   absent. The port dispatches all six. 2015 is a harmless
   port-internal channel (same visible result as vanilla's
   `addParticle` loop: 128 portal particles lerped along the line);
   2016/2017/2018 are live and wrong; 2019/2020 dispatch is dead
   (no sender) but the client-side digging call that replaced them
   is live and wrong (B8-G09).
4. **The oracle sources are 26.3-form wholesale — the 1.21.10
   sweep's oracle problem is bigger than the 26.x runs'.** In
   26.1.2, 2 of 8 particle oracles failed; in 1.21.10, 7 of 8 fail
   (92 errors total), and the causes are all the same shape: the
   oracle sources import 26.x packages (`animal.cow/dolphin/equine/
   feline/nautilus/parrot`, `npc.villager`, `monster.zombie`,
   `world.attribute`, `client.resources.model.sprite`), 26.x classes
   (`BlockStateModelSet`, `ItemStackTemplate`, geyser options),
   a library 1.21.10 does not ship (jspecify), and 26.x constructor
   forms (`LocalPlayer` 9-arg, `ServerLevel`, `TextureAtlasSprite`).
   The 1.21.10 classes mostly exist under their 1.21.x names, so
   most of these are mechanical re-pointings — except `Nautilus`
   and the geyser API, which do not exist at all. Until Batch 0's
   repair pass lands, no particle driver can run for 1.21.10.
5. **The port's particle tables are otherwise exact.** All 115
   types covered end-to-end except elder_guardian, all 18
   data-carrying wire ids correct, all 28 limiter flags correct
   except elder_guardian, all 27 real entity-burst pairs and all 45
   marker-zoo blocks covered, all 11 leaf blocks' chances verified
   — the gaps are narrow and enumerated, not broad drift. The 11
   extra types in the Rust enum are dead but unreachable.

## §7 Cross-references (not re-minted)

- **G1 (pilot) — bubble columns never form.** The client-side
  animate arm for bubble columns exists and is correct
  (`Animate::BubbleColumn`, `animate_tick.rs:1660` —
  `BubbleColumnUp` particles + whirlpool/upwards ambient sounds),
  but it can never trigger in-game because no `bubble_column`
  block states ever form server-side (G1, 1.21.7 pilot; 1.21.10
  server-side status is Batch 5's scope, not verified here). No new
  defect: the particle-visible consequence of G1, not a particle
  defect.
- **G6 (pilot) — dripstone never transfers fluid to cauldrons.**
  Server-side only; 1.21.10 server-side status is Batch 5's scope.
  The *client-side* drip particles are unaffected except for
  B8-G04's gate: vanilla 1.21.10's
  `PointedDripstoneBlock.animateTick` rolls 0.12 and climbs
  same-block stalactite via `findRootBlock` (no tag involved —
  there is no `SpeleothemBlock` and no `speleothems` tag in
  1.21.10), and the port's `dripstone_drip`
  (`animate_tick.rs:1807`) matches it by name, with the same 0.12
  roll and 0.02 cauldron rule. The marker oracle's scope comment
  about binding "the speleothem tag the stalactite branch needs"
  (`VanillaMarkerParticlesOracle.java:136`) is stale for 1.21.10 —
  the oracle binds tags from the jar's datapack, which has no such
  tag, and no code path reads it — but the comment is not a defect.
- **B12-G04 pattern — 26.x-only types in the `ParticleType` table.**
  Same table-leak *pattern* as B12-G04, but the 11 extra types in
  the Rust enum (8 26.2-only: geyser family, sulfur bubbles,
  noxious gas pair, sulfur cube goo; 3 26.3-only: poplar leaves)
  are unreachable — ids are name-resolved from the server's
  registry, and a 1.21.10 server never sends those names — so they
  are not minted as a defect. Dead table entries, not a behaviour
  gap. The 4 geyser entries in `override_limiter()` are likewise
  dead.
- **The `SpeleothemBlock` assumption elsewhere in the tree.** B8-G04
  is the particle-visible instance; the same 26.2+-only class name
  gates other behaviour: `mesher.rs:910`
  (`has_class("SpeleothemBlock")` selects the 0.125 horizontal
  random offset for dripstone — dead in 1.21.10, so dripstone meshes
  with the 0.25 default; vanilla 1.21.10's `getMaxHorizontalOffset`
  is `SHAPE_BASE.min(X)`, i.e. the 0.125 branch) and the
  server-side `horizon-world` speleothem/falling code
  (`level/speleothem.rs`, `level/falling.rs:44/50`,
  `level/classes.rs:219` — Batch 5's scope). The projectile-hit
  path is fine: it matches by block name, not class
  (`creatures.rs:5084` in 26.1.2; not re-verified here).
- **3003 wax-on sound missing from the level-event path → Batch 10
  (sound).** Vanilla 1.21.10's 3003 case spawns WAX_ON particles
  (3–5) AND plays `HONEYCOMB_WAX_ON` via `playLocalSound`
  (offsets 3374/3390, verified fresh). The 1.21.10 tree kept the
  particles (`block_particle_event`, `level_particles.rs:35-38`,
  routed from `game.rs:4378`) but the event path plays no sound.
  The item-use, dispenser, remote and copper-golem wax sounds are
  all still present (`game.rs:8043` + callers :8354/:8373,
  `dispenser.rs:2235`, `remote.rs:8802/8810`,
  `copper_golems.rs:508`), so the missing case is specifically a
  client receiving the 3003 event for someone else's waxing (LAN
  guest view). Sound defect, not a particle defect — flagged for
  Batch 10 with this evidence, not minted here.
- **2002/2007 splash-potion break sound missing → Batch 10
  (sound).** Vanilla 1.21.10's 2002/2007 case plays
  `SPLASH_POTION_BREAK` itself, after the particles (offset 1530,
  verified fresh). The 1.21.10 tree instead maps it to events
  1053/1054 (`game.rs:4500`, `:4519`) — which vanilla 1.21.10 never
  sends (26.3 split the sound into 1053/1054; 1.21.10's
  lookupswitch has no such cases). The potion-splash particles
  still fire (`potion_splash_particles`, `game.rs:4368/4373`).
  Sound defect — flagged for Batch 10, not minted here.
- **Digging hit-sound schedule (sound half of B8-G09) → Batch 10.**
  The particle half (extra crumb) is B8-G09; the sound half (hit
  sound at first hit instead of every 4th tick, then silence) is a
  sound defect with the same evidence — flagged for Batch 10.
- **Enderman teleport destination selection → Batch 3.** The
  1.21.10 `mobs/enderman.rs` grounds the teleport on its own
  destination checks; the tag-form question is Batch 3's scope, not
  verified here; only the particle-visible 2018 trail is minted
  here (B8-G07).
- **`CONSUMABLE_DOES_NOT_TELEPORT_TO` avoidance tag → Batch 3.**
  `teleport_consume.rs:36` documents the 26.3-form forbidden tag;
  the tag is absent from both 1.21.10 jars and the prepared data.
  The destination-selection consequence is Batch 3's scope, not
  verified here; only the particle-visible 2017 trail is minted
  here (B8-G06).
- **B8-G10 routing.** `/spreadplayers` is commands-area (Batch 9);
  it is minted here only because the coordinator routed Batch 7's
  flag to Batch 8. The merge may re-route it to Batch 9's ledger.

## §8 What passed (per area)

### Registry and type coverage (118 checks: 116 pass, 2 gap)

- 115 particle types: `registries.json` count == jar `ParticleTypes`
  `<clinit>` registration count (115), and the `<clinit>` order equals
  the `protocol_id` of every type (verified by parsing the bytecode;
  0 mismatches vs the datagen registry — `elder_guardian` present at
  id 17, no geyser/sulfur/noxious/poplar entries).
  `ParticleTypes.class` is byte-identical (md5) between the client
  and server-game jars.
- Engine `make()` (`particles.rs:2216`, outer match at :2277) has an
  outer match arm for all 125 `ParticleType` variants (verified fresh
  by script: 125/125 variants covered, no outer catch-all — the `_ =>`
  arms in the body are all nested matches) — every enum variant has a
  provider implementation. The only engine-coverage gap is the missing
  `elder_guardian` variant itself (B8-G01).
- Vanilla providers: `ParticleResources.registerProviders()` in the
  1.21.10 client jar registers a provider for all 115 types, including
  `ELDER_GUARDIAN` (`ElderGuardianParticle$Provider`, offsets
  296–300). (Field-name quirk, same as 26.1.2: the fields are
  `TRIAL_SPAWNER_DETECTED_PLAYER(_OMINOUS)` but register under the
  names `trial_spawner_detection(_ominous)`.)
- The 11 Rust-enum extras (8 26.2-only geyser/sulfur + 3 26.3-only
  poplar leaves) are absent from the 1.21.10 jar's `ParticleTypes` —
  confirmed unreachable from a 1.21.10 server.
- Gap: providers-driver kind set (126 kinds) misses `elder_guardian`
  (B8-G01) and `vibration` (B8-H01), and carries the 13 non-1.21.10
  kinds (B8-H02).
- `STONE = 1` in the providers driver is correct for 1.21.10: state
  id 1 is stone in the 1.21.10 datagen `blocks.json` (the "in 26.3"
  comment is stale; the value is right).

### Wire codec (20 checks: 19 pass, 1 gap)

- All 18 data-carrying particle wire ids match `Particle::read`'s
  hardcoded ids exactly (verified fresh against the 1.21.10 registry,
  0 mismatches): State 1/2/29/109/113 (block, block_marker,
  falling_dust, dust_pillar, block_crumble), Color 21/36/42
  (entity_effect, tinted_leaves, flash), dust 14,
  dust_color_transition 15, effect 16, instant_effect 46,
  dragon_breath 8, sculk_charge 38, shriek 103, trail 49, item 47,
  vibration 48. `ids::particle(id)` is the identity when no version
  adapter is active (`adapter/ids.rs`), so these are the native
  1.21.10 ids. Note the ids differ from 26.1.2's (dust_pillar 109
  vs 111, block_crumble 113 vs 115, shriek 103 vs 105) — the read
  code's comment names the 1.21.11 order, which 1.21.10 shares. The
  read code carries no geyser arms — correct for 1.21.10 (comment:
  "The geyser particles a later release adds do not exist in
  1.21.11"), though the `Geyser`/`GeyserBase` data variants still
  exist in the enum and `write` still writes them if present
  (unreachable from a 1.21.10 read; `hub_particle` can still build
  them from a `ParticleOptions::geyser`, but no 1.21.10 path
  creates one).
- `received_particle` resolves ids to names through the server-sent
  registry (not a hardcoded table) — correct for all 114 implemented
  types.
- Gap: B8-G02 (Flash write path).
- Note (not a defect): `hub_particle` deliberately returns `None` for
  item and trail particles (`lan_particles.rs`, `network_particle`
  doc: "a particle whose data isn't sent from here (items, trails)")
  — a LAN host's `/particle item|trail` draws locally but sends
  nothing to guests. Documented behaviour, same as 26.1.2; the command
  does not parse trail options either. Not minted.

### Override limiter (28 checks: 27 pass, 1 gap)

- Vanilla limiter-true set (28 types, from the `<clinit>` booleans,
  parsed fresh from the 1.21.10 jar) vs the Rust `override_limiter()`
  list: identical except `elder_guardian` (B8-G01). The Rust list's
  4 extra entries are the geyser types — dead in 1.21.10,
  unreachable.

### Entity event bursts (27 checks: 27 pass)

- Every (entity, event) pair in the entity driver's `EVENTS` table
  that has a vanilla 1.21.10 side (11 entities, 27 pairs, re-counted
  fresh — the 3 `nautilus` pairs have no 1.21.10 side, B8-H05) is
  covered by the port's `entity_event_burst` (`particles.rs:627`):
  villager 12/13/14/42, cow 18, wolf/cat 6/7/18, parrot 6/7,
  horse/llama/camel 6/7/18, ocelot 40/41/18, dolphin 38, allay 18 —
  with the right particle kinds and burst shapes, verified against
  the 1.21.10 jar (`Villager.handleEntityEvent` 12→heart, 13→angry,
  14→happy, 42→splash; `TamableAnimal` 7→heart/6→smoke;
  `Animal` 18→heart ×7 (0.02 gaussian, y+0.5); `Ocelot`
  41→heart/40→smoke; `Dolphin` 38→happy_villager;
  `Allay` 18→3× heart; `AbstractHorse` 6/7→taming particles).
  Counts/positions/RNG draws need the driver run; the mapping table
  is exact.
- The port's table also carries `nautilus`, `zombie_nautilus` and
  `camel_husk` arms — dead in 1.21.10 (none of the three exists in
  the 1.21.10 entity registry): unreachable, not a defect.

### Marker zoo (45 checks: 45 pass)

- Every real 1.21.10 block name in the marker driver's zoo (36 names,
  re-counted: 37 distinct names in `zoo_states()` minus
  `red_poplar_leaves`) + neighbour (6: dried_ghast, beehive,
  bee_nest, redstone_ore, deepslate_redstone_ore, pointed_dripstone)
  + campfire (2: campfire, soul_campfire) + wither_rose (1) cases
  has a `classify` arm in `animate_tick.rs`, including the guarded
  ones (lit furnaces, powered levers/repeaters, honey-level-5 hives,
  charged respawn anchors, active sculk sensors, down-pointing
  dripstone tips, lit candles/candle cakes). Positions/RNG draws
  need the driver run; coverage is complete. (`red_poplar_leaves`
  is the 37th zoo name; it has a classify arm in this tree but the
  block does not exist in 1.21.10 — B8-H03, a driver-scope item, not
  a port gap.)
- The unlit variants in the zoo (unlit redstone torches, furnaces,
  smokers) correctly map to `Animate::None`.

### Leaves (12 checks: 11 pass, 1 gap)

Per-block `leafParticleChance` and class — parsed fresh from the
1.21.10 `Blocks.<clinit>` (this batch's class-file parser; the
azalea colour read from the lambda's `ColorParticleOption.create`
argument):

| Block | Vanilla 1.21.10 | Port | Verdict |
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
- Note: vanilla 1.21.10 has no `CherryLeavesBlock`/`PaleOakLeavesBlock`
  classes — cherry/pale-oak are `UntintedParticleLeavesBlock` instances,
  and the port's `FallingParticlesLeavesBlock` comment names the 26.3
  class; the mechanism is the 1.21.10 one.

### Dripstone and tags (7 checks: 6 pass, 1 gap)

- `speleothems` does NOT exist in 1.21.10 (absent from the client
  jar's datapack; no `SpeleothemBlock` class) — a real difference
  from 26.2, where it held `pointed_dripstone` + `sulfur_spike`.
  The port's animateTick drip path does not need it (name-based
  table), so this is a pass for the animate path; the class-gated
  level-event path is the gap (B8-G04).
- `washed_away_by_fluids` and `blocks_lava_fire_spread` are both
  absent from the 1.21.10 datapack — confirms the pilot's tag
  inventory pattern for 1.21.10.
- `triggers_ambient_dried_ghast_block_sounds` exists (soul_sand,
  soul_soil), as do the two desert ambient-sound tags
  (`triggers_ambient_desert_sand_block_sounds`: sand, red sand;
  `triggers_ambient_desert_dry_vegetation_block_sounds`: #terracotta,
  sand, red sand) — the tags the port's `AnimateTables` binds are
  real in 1.21.10. (The poplar tag it also binds,
  `required_for_poplar_leaf_ambience`, has no 1.21.10 referent — dead
  path, no poplar blocks exist.)
- Vanilla dripstone animateTick path matches the port (0.12 roll,
  same-block climb, 0.02 cauldron rule, mud-as-water, 5/16 − 0.0625
  drip height).
- Gap: B8-G04 (level-event 1504 gate).

### Oracle API existence (8 checks: 8 pass)

All vanilla methods the oracles call exist in the 1.21.10 jars with the
expected signatures (javap on the named jars, fresh):
`ParticleUtils.spawnParticlesOnBlockFaces` /
`spawnParticlesAlongAxis` (emitter, `net.minecraft.util.ParticleUtils`),
`LivingEntity.spawnItemParticles` (item),
`LivingEntity.handleEntityEvent` (entity — the method is declared on
`LivingEntity` and inherited by Villager; the 26.1 record's
attribution to `Villager` was corrected by the 26.1.1 run),
`CampfireBlock.animateTick` / `makeParticles` /
`CampfireBlockEntity.particleTick` (marker), `ClientLevel.animateTick`
(marker loop). This is existence only — the oracles' *use* of them is
what the drivers verify, and 7 of the 8 oracles do not compile (§9).

### Other passes

- Engine `make()` exhaustiveness (see above — 125/125 arms, fresh).
- Game-event-10 apparition re-implementation present (§4, B8-G01).
- Bubble-column animate arm present (§7, G1).

### Level events 2015–2020 + digging (5 checks: 1 pass, 4 gap)

- **2015 dragon egg: pass.** Vanilla 1.21.10 has no 2015 level event,
  but `DragonEggBlock.teleport` spawns 128 portal particles lerped
  between the old and new block positions via `Level.addParticle`
  (loop bound `sipush 128`, `Mth.lerp` on x/y/z — verified fresh).
  The port's server sends the port-internal 2015 event
  (`horizon-world/src/level/interact.rs:594`) and the client draws
  128 portal particles lerped along the same line
  (`block_teleportation`, `[16,8,16]`). Mechanism differs,
  visible result matches. (Exact RNG-draw equivalence needs a run.)
- **2016 shulker: gap — B8-G08.** **2017 consume: gap — B8-G06.**
  **2018 enderman: gap — B8-G07.**
- **Digging crack particles: gap — B8-G09.** (The 2019/2020 network
  dispatch itself is dead — no sender — so it is not a separate
  check; the live defect is the client-side `remote.rs:8172` call.)

### Particle group cap (1 check: 0 pass, 1 gap)

- Gap: B8-G05 (the `tick_tracking` pending drain enforces the
  26.2-form drop-new + probabilistic cap; vanilla 1.21.10 evicts
  the oldest via `EvictingQueue`).

### Routed check: /spreadplayers safety (1 check: 0 pass, 1 gap)

- Gap: B8-G10 (Batch 7-routed flag, verified fresh).

## §9 Oracle repairs and defects

Triage is from this batch's own javac measurement (method in §2) —
Batch 0's measured table had not landed when this batch ran, so
there is no Batch 0 table to triage from. Each oracle was compiled
alone on the "both" classpath (both named 1.21.10 jars + 114 game
library jars + jsr305) with `-sourcepath` on the test java root;
logs in `evidence/javac-*.log`. A file that fails standalone on
"both" fails on every classpath. The coordinator should reconcile
these counts against the Batch 0 table when it lands — in
particular, Batch 0's per-classpath attribution may split some of
these errors differently, but the own-error root causes below were
each verified against the 1.21.10 jars (package absence by jar
listing, constructor arity by `javap`).

### B8-O01 — `VanillaMarkerParticlesOracle` BROKEN, 6 compile errors (MINOR)

Measured: 6 errors. The 26.1.2 break reproduces — :44, the oracle's
`Player` subclass calls `super(null, null, null, null, null, null,
false, null, null)` — 9 arguments, the 26.3 form
(`VanillaMarkerParticlesOracle.java:44`); 1.21.10's `LocalPlayer`
constructor takes 7 (`Minecraft, ClientLevel, ClientPacketListener,
StatsCounter, ClientRecipeBook, Input, boolean` — javap on the
1.21.10 client jar; no `ChatAbilities`). New breaks beyond 26.1.2:
three references to the 26.x package
`net.minecraft.world.attribute` (:104, :116, :233), a dead
`@Override` at :116, and a missing
`bindComponents(DataComponentMap)` at :218. **Blocks the
marker_particles driver entirely** — before it can reach the B8-G03
spruce cases. Repair is the 1.21.10-form LocalPlayer construction
plus re-pointing the attribute references to
`net.minecraft.world.entity.ai.attributes`; do not repair here
(checks + triage only).

### B8-O02 — `VanillaParticleProvidersOracle` BROKEN, 25 compile errors (MINOR)

Measured: 25 errors. The 26.1.2 geyser break reproduces — 10 of the
25 are the geyser references (`GeyserParticleOptions` ×3,
`GeyserBaseParticleOptions` ×3, `ParticleTypes.GEYSER` /
`GEYSER_PLUME` / `GEYSER_BASE` / `GEYSER_POOF` ×4; :143-146,
:164-165; zero `geyser` entries in either 1.21.10 jar) — plus 15
more 26.x-form errors: `BlockStateModelSet` ×2 (:16, :34),
`ItemStackTemplate` ×2, `Nullable` ×4 (the `org.jspecify.annotations`
import at :46 — no jspecify jar in 1.21.10's game libraries), the
`net.minecraft.client.resources.model.sprite` package (:22),
`Material` ×3 (:115, :116, :124), a `TextureAtlasSprite`
constructor mismatch (:107), and further missing symbols (:83,
:100, :113, :122, :150). **Blocks the particle_providers driver
entirely.** The providers oracle's *mechanism* fits 1.21.10 — it
builds `new ParticleResources()`, reads the private `spriteSets`
field by reflection, and calls the public `getProviders()` — all
present in the 1.21.10 client jar (`ParticleResources` holds
`registerProviders()` in 1.21.10). Repair is bigger than the 26.1.2
deletion (geyser branches + cast helpers); the sprite/Material/
jspecify breaks need 1.21.10-form rewrites, and the driver trim
(B8-H02) goes with it; do not repair here.

### B8-O03 — `VanillaBlockParticlesOracle` BROKEN, 8 compile errors (MINOR, NEW)

Measured: 8 errors — `BlockStateModelSet` ×2 (:13, :44), the
`net.minecraft.client.resources.model.sprite` package (:17),
`Material` ×3 (:45, :47), a `TextureAtlasSprite` constructor
mismatch (:42; 1.21.10's ctor is the 6-arg protected form), a dead
`@Override` (:47). All 26.x-form. **Blocks the block_particles
driver entirely.**

### B8-O04 — `VanillaEmitterParticlesOracle` BROKEN, 5 compile errors (MINOR, NEW)

Measured: 5 errors — the `net.minecraft.world.entity.animal.cow`
package (:23; absent from 1.21.10 — `Cow` is at
`net.minecraft.world.entity.animal.Cow`) and the `Cow` symbol ×4
(:120, :126). **Blocks the emitter_particles driver entirely.**

### B8-O05 — `VanillaEntityParticlesOracle` BROKEN, 18 compile errors (MINOR, NEW)

Measured: 18 errors — ten bad 26.x package imports (:17-26) plus 8
missing-symbol errors for the entity classes (:47-50). The classes
live in `world.entity.animal.*` / `world.entity.npc.*` directly in
1.21.10, except `Nautilus`, which does not exist at all. **Blocks
the entity_particles driver entirely.**

### B8-O06 — `VanillaItemParticlesOracle` BROKEN, 4 compile errors (MINOR, NEW)

Measured: 4 errors — a `ServerLevel` constructor mismatch (:28) and
three missing symbols (:29, :31: `bindComponents`,
`count()`, `item()` — the 26.x item-template API). **Blocks the
item_particles driver entirely.**

### B8-O07 — `VanillaEffectSyncOracle` BROKEN, 26 compile errors (MINOR, NEW)

Measured: 26 errors — the
`net.minecraft.world.entity.monster.zombie` package (:100; absent —
`Zombie` is at `net.minecraft.world.entity.monster.Zombie`), a
cascade of 21 errors from the unresolvable `Zombie` superclass
(:102-110), a `ServerLevel` constructor mismatch (:113), and missing
effect methods (:148). **Blocks the effect_sync driver entirely**
(its other oracle, `VanillaEffectParticlesOracle`, compiles).

### Oracles assessed PASSING for 1.21.10 (this batch's measurement)

- `VanillaEffectParticlesOracle` — compiles clean (0 errors) on the
  "both" classpath.

For comparison, the 26.2 run had only the marker oracle failing
(B8-O01); 26.1, 26.1.1 and 26.1.2 add the providers oracle (B8-O02);
1.21.10 adds five more (B8-O03–O07) because the oracle sources are
26.3-form and 1.21.10 predates the 26.x package moves, the
`world.attribute` package, jspecify, and the 26.x constructor forms.

Static evidence for 1.21.10 (supporting the measured passes/fails,
not a substitute for them):

- The 8 oracle APIs this batch's scope touches all exist (§8).
- The providers oracle's mechanism fits 1.21.10: it builds
  `new ParticleResources()`, reads the private `spriteSets` field by
  reflection, and calls the public `getProviders()` — all present in
  the 1.21.10 client jar (`ParticleResources` holds
  `registerProviders()` in 1.21.10; `ParticleEngine` has no such
  method). It resolves kinds by registry name, which is why the 13
  non-1.21.10 kinds would throw (B8-H02).
- The marker oracle's `bindBlockTags()` reads tags from the jar's
  datapack at runtime, so the absent `speleothems` tag is simply
  unbound — no code path reads it in 1.21.10.

## §10 Driver inventory

The plan (§4) claims 9 particle drivers; the tree has 7 particle-named
drivers plus 1 particle-adjacent driver = **8**. The 9th is unaccounted
for: possibly the plan counted `tests/test_cushion_particles.py` (a
Python unit test for Horizon's cushion feature, not a parity driver) or
double-counted. No 9th parity driver exists under any name in
`versions/1.21.10/tools/`.

| Driver | Oracle (1.21.10 status, MEASURED by this batch) | Probe | Status |
|---|---|---|---|
| block_particles | BlockParticles: FAILS, 8 errors (B8-O03) | block_particles_fixture.rs | BLOCKED (B8-O03) — oracle does not compile |
| emitter_particles | EmitterParticles: FAILS, 5 errors (B8-O04) | emitter_particles_probe.rs | BLOCKED (B8-O04) — oracle does not compile |
| entity_particles | EntityParticles: FAILS, 18 errors (B8-O05) | entity_particles_probe.rs | BLOCKED (B8-O05) — oracle does not compile; driver also sends nautilus cases (B8-H05) |
| item_particles | ItemParticles: FAILS, 4 errors (B8-O06) | item_particles_fixture.rs (horizon-world) | BLOCKED (B8-O06) — oracle does not compile |
| marker_particles | MarkerParticles: FAILS, 6 errors (B8-O01) | marker_particles_probe.rs | BLOCKED (B8-O01) — oracle does not compile; would also fail on spruce cases (B8-G03) once repaired; red_poplar_leaves zoo cases un-runnable (B8-H03) |
| native_client_particles | none (Horizon-only Host API contract, no vanilla oracle) | game fixture | UNRUN; out of vanilla scope |
| particle_providers | ParticleProviders: FAILS, 25 errors (B8-O02) | particle_providers_probe.rs | BLOCKED (B8-O02) — oracle does not compile; would also FAIL on the 13 non-1.21.10 kinds — oracle throws `AssertionError("unknown particle …")` (B8-H02); does not test elder_guardian (B8-G01) or vibration (B8-H01) |
| effect_sync (adjacent) | EffectParticles: PASSES (0 errors); EffectSync: FAILS, 26 errors (B8-O07) | effect_sync_fixture.rs | BLOCKED (B8-O07) — one of its two oracles does not compile |

`parity.py` was not run (blocked — static checks only, no cargo).
There is no prepared 1.21.10 data in the worktree (no
`.local/horizon-client/1.21.10/`), so the data precondition for the
driver runs in §13 is NOT met — the runs need a prepare first (Batch
12's scope), on top of the oracle repairs. 0 of the 8 drivers are
runnable as-is; 7 wait on oracle repairs (B8-O01–O07) plus the
B8-H02/H03/H05 trims, and the 8th has no vanilla oracle.

## §11 Unit tests

34 `#[test]`s in `crates/horizon-client-render/src/particles.rs`,
unrun (blocked — no cargo builds in this batch):

- Vanilla (26): `legacy_spell_providers_use_velocity_rgb_and_their_original_alpha_rules`,
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
  `explosion_tracker_emits_debris_only_in_air`,
  `constant_240_particles_snapshot_to_240_not_full_bright`,
  `flame_and_portal_light_go_to_their_12111_channels`,
  `glow_family_renders_on_the_translucent_layer` (the last three are
  new vs the 26.1.2 tree's 23 vanilla tests — 1.21.11-era additions
  carried into the 1.21.10 tree).
- Horizon mod particles (not vanilla scope, 8):
  `a_mod_particles_json_becomes_its_sprite_set`,
  `mod_behaviour_validates_declarations`,
  `mod_velocity_adds_the_spawn_velocity_and_jitters_deterministically`,
  `mod_animation_modes_pick_their_sprites`,
  `mod_curves_clamp_and_interpolate_over_lifetime`,
  `mod_animation_advances_in_the_tick_with_and_without_curves`,
  `mod_particle_ticks_motion_curves_and_removal`,
  `fading_mod_particles_use_the_blended_layer`.

Note: `consume_teleport_unpacks_offsets_within_127` and
`egg_teleport_unpacks_offsets_and_preserves_seeded_draw_order` test
the teleport-unpack helpers the 2017/2015 paths use — they do not
test whether the events should fire at all.

## §12 Not covered

- **All driver numerics.** Per-tick provider physics (positions, motion,
  age, lifetime, size, colour, roll, sprite, UVs, light), emitter RNG
  draws, entity-burst counts/positions, block-particle placement, marker
  loop exactness — only the drivers can verify these, and none ran (7
  of 8 are oracle-blocked; the 8th has no vanilla oracle). The static
  checks above bound the *tables and mappings*, not the numerics.
- **Translucent-layer table.** `ParticleType::translucent_layer()` (which
  types vanilla renders in the translucent layer) was not verified — it
  needs per-class `getRenderType` checks across ~20 particle classes.
  (Same scope decision as the 26.1.2, 26.1.1, 26.1, 26.2 and 1.21.7
  batches.) Note the 1.21.10 tree's own comment: the Glow family is
  TRANSLUCENT in 1.21.10, OPAQUE in 26.x — the table is version-shaped
  and a re-verification here would need the 1.21.10 classes.
- **Sprite sets / particle textures.** Whether the engine resolves every
  type's sprite set from the 1.21.10 assets needs prepared data + a run.
- **Level events other than 1504, 2015–2020 and 3002–3005.**
  `level_particles.rs` handlers (dragon egg, shulker/ender teleport,
  growth, turtle/sniffer eggs, torch burnout, dragon fireball, potion
  splashes, `level_event_at`) exist but their numerics are unverified;
  only 3002–3005 are in the emitter driver. 1504's gate was verified
  statically (B8-G04) but its drip numerics need a run. 2015's
  visible result matches vanilla (128 lerped portal particles) but
  exact RNG-draw equivalence needs a run.
- **Apparition visual fidelity.** The elder-guardian re-implementation
  (§4) needs an in-game look or the providers driver.
- **The 9th driver.** The plan's count of 9 does not match the tree's 8;
  if a 9th driver exists somewhere else (another version tree, a renamed
  file), it was not found here.
- **Dripstone climb limit.** Vanilla's `getFluidAboveStalactite` passes
  11 to `findRootBlock`; the port climbs 10 steps (`1..11`). Whether
  the off-by-one reading matches needs the marker driver run.
- **G1/G6 server-side status for 1.21.10.** Whether bubble columns form
  and dripstone transfers fluid server-side in the 1.21.10 port is Batch
  5's scope, not this batch's.
- **The 3003/2002 sound halves and the digging sound schedule** are
  Batch 10's scope (§7); the particle halves were verified here.
- **Enderman/shulker teleport destination selection** is Batch 3's
  scope; only the particle-visible trails were verified here.
- **No prepared 1.21.10 data exists in the worktree** (no
  `.local/horizon-client/1.21.10/`), so data checks used the datagen
  reports (`e6-deltas/versions/1.21.10/datagen/reports/`) instead of
  prepared data — the prepared-data cross-checks the 26.1.2 run did
  (prepared `registries.json` == datagen, `block-tags.json`,
  `block-behaviours.json`, `items.json`) could not be repeated here
  for the prepared side. The tag-absence claims rest on the jar
  datapack (205 block tags listed), and the `items.json` claim
  (chorus_fruit consumable, 0 `directional_particles`) rests on the
  datagen report.
- **Batch 7's G09 masking claim for the 2017 trail** (the trail fires
  only after a successful teleport, which never happens on the player
  path) was not verified here; the trail mechanism itself was verified
  fresh. If G09 is fixed without removing the trail, the trail becomes
  visible — Batch 7's warning stands as recorded.
- **Batch 0's measured oracle table had not landed** when this batch
  ran; the oracle verdicts rest on this batch's own javac measurement
  (§9). Reconcile against the Batch 0 table when it lands.

## §13 What unblocks the rest

1. Prepared 1.21.10 data does NOT exist in the worktree — a prepare
   (Batch 12's scope) is a precondition for every driver run in this
   batch, on top of the oracle repairs below.
2. Repair the seven broken oracles (follow-up, not this batch): B8-O01
   (LocalPlayer 7-arg construction + `world.attribute` re-pointing
   for `VanillaMarkerParticlesOracle`), B8-O02 (geyser deletion +
   sprite/Material/jspecify rework for
   `VanillaParticleProvidersOracle`), B8-O03 (sprite/Material rework
   for `VanillaBlockParticlesOracle`), B8-O04 (Cow import for
   `VanillaEmitterParticlesOracle`), B8-O05 (entity package imports
   + Nautilus removal for `VanillaEntityParticlesOracle`), B8-O06
   (`ServerLevel` ctor + item-template rework for
   `VanillaItemParticlesOracle`), B8-O07 (Zombie import +
   `ServerLevel` ctor + effect methods for
   `VanillaEffectSyncOracle`). Only `VanillaEffectParticlesOracle`
   compiles as-is (measured, §9). Reconcile the repair list against
   Batch 0's table when it lands.
3. Fix B8-G01 (add the `ElderGuardian` variant + `ElderGuardianParticle`
   provider), B8-G03 (spruce leaves arm) and B8-G04 (the 1504 gate —
   match by name or by `PointedDripstoneBlock`, not `SpeleothemBlock`)
   on fix branches, then run `particle_providers` and
   `marker_particles`; fix B8-G02 (Flash arm in `hub_particle`) with
   them or separately. B8-G03 is driver-blocking; B8-G02 and B8-G04 are
   not covered by any driver; B8-G01 blocks nothing in the 1.21.10 tree
   because the driver skips the type — the fix should add
   `elder_guardian` to the driver's kind set too, or the type stays
   untested.
4. Fix the five inherited 26.2/26.3-form reversions (B8-G05–G09) — the
   26.1 tree is the repair reference for four of them (the
   `breaking_block_sound` form, the absence of the 2016/2017/2018
   handlers and senders); for B8-G05 the repair is evict-oldest, not
   cap-removal (vanilla 1.21.10's `EvictingQueue` form). Fix B8-G10 by
   restoring vanilla's `isSafe` (`y < maxY && !liquid && !is(FIRE)`)
   in `spread_safe`. None of B8-G05–G10 is covered by any driver; all
   need in-game verification (particle-heavy scene for the cap; chorus
   fruit, enderman and shulker teleports; digging; `/spreadplayers`).
5. Trim the non-1.21.10 entries from the drivers for 1.21.10 (B8-H02: 13
   kinds in the providers driver — 3 poplars + 2 sulfur + 2 noxious +
   4 geyser + `pause_mob_growth` + `reset_mob_growth`; B8-H03:
   `red_poplar_leaves` in the marker zoo; B8-H05: `nautilus` in the
   entity driver), add the missing coverage (B8-H01: vibration), then
   run the drivers (`block`, `emitter`, `entity`, `item`,
   `effect_sync`); the item driver needs a longer cargo-build window
   than its 900 s timeout under load (or a warm target dir) —
   1.21.7 precedent.
6. Run the 34 `particles.rs` unit tests (`cargo test -p
   horizon-client-render --lib particles::`, dev profile, `-j 1`).
7. Route the Batch 10 sound flags (§7: 3003 wax-on, 2002/2007
   potion-break, digging sound schedule) and the B8-G10 routing
   question (commands-area; Batch 9's ledger?) to the coordinator.

## §14 Method notes

- The 1.21.10 **client-named** jar is Mojang-mapped —
  `net/minecraft/client/particle/ParticleEngine.class` is present by
  its mapped name — and its `ParticleTypes.class` is byte-identical
  (md5) to the server-game-named jar's. The `server.jar` in the jars
  dir is a Mojang bundler jar (never javap'd); the named game jars are
  `e6-deltas/versions/1.21.10/jars/{client-named,server-game-named}.jar`.
- **No md5 carry from 26.x.** Every vanilla claim was javap-verified
  fresh against the 1.21.10 jars. The one cross-version md5 check
  (1.21.7 vs 1.21.10 leaves classes, byte-identical) was used only to
  confirm the leaf *classes* are unchanged; the *registrations* were
  still parsed fresh because `Blocks.class` differs.
- **The `ParticleTypes.<clinit>` parse** (evidence, inline in this
  batch's scratch) walks the `javap -c` dump anchored on `putstatic`
  lines, reading the name (`ldc`/`ldc_w` string) and the limiter boolean
  (`iconst_0`/`iconst_1`) from the lines above each. Three gotchas,
  all carried from the 26.1.2 run: constant indices above 255 use
  `ldc_w`, not `ldc`; the regex for the wide form is `ldc(_w)?`, not
  `ldc_w?`; and the limiter boolean must be read as the `iconst`
  *after* the name `ldc` — a backwards walk that takes the first
  `iconst` before the name picks up the *previous* registration's
  boolean. `CODEC`/`STREAM_CODEC` putstatics at the end of `<clinit>`
  are not registrations and must be excluded (117 putstatics, 115
  registrations).
- **The `Blocks.<clinit>` parse** (`evidence/blocks_parse.py`) parses
  the class file directly (constant pool + `BootstrapMethods` +
  bytecode with a full opcode-length table): each leaf name's `ldc`
  is associated with its registration lambda through the
  `BootstrapMethods` table, and each lambda's constructed class and
  float constant are read out. 1.21.x registers by name string
  (`ldc "spruce_leaves"`), unlike 26.x's `BlockItemIds` form. The
  parser was validated by resolving all 11 leaf blocks to the
  expected classes and chances, and by the azalea colour reading
  exactly −9399763 (0xFF70922D).
- **Oracle measurement.** Single-file `javac -nowarn` per oracle on
  the "both" classpath, ~60–90 s per file under load (8 files,
  ~11 min total). The classpath was both named jars + all 114 jars
  under `game/libraries/` + `tools/jsr305-3.0.2.jar`. The jspecify
  error is real, not a classpath gap: no jspecify jar exists in
  1.21.10's `game/libraries/` or `launcher/libraries/`.
- `javap` on the small classes ran 5–40 s each under load; batches of
  2–8 classes per exec kept the work moving. `javap -c` on the big
  classes (`LivingEntity`, `ClientPacketListener`) ran in background
  batches. `javap -c` on `Blocks` was not used (1.21.7 warning:
  exceeds 280 s under throttle) — the class-file parser replaced it.
- `LevelEventHandler` lives at `net.minecraft.client.renderer.LevelEventHandler`
  in 1.21.10 (same as 26.x). The entity classes moved packages
  between 1.21.x and 26.x: `Animal` is
  `net.minecraft.world.entity.animal.Animal` (not
  `net.minecraft.world.entity.Animal`), `Allay` is
  `...animal.allay.Allay`, `AbstractHorse` is
  `...animal.horse.AbstractHorse` — the first javap attempts with
  the 26.x packages failed with "class not found".
- No production code (`crates/`) was touched. No commits were made in
  this batch. Only the two deliverable files were written (plus
  evidence/ scratch: javap dumps, the registry census, the
  `ParticleTypes` `<clinit>` parse, `blocks_parse.py`, the javac
  logs, and the jar listings).

## §15 Evidence index

Worktree `~/workspace/mcir-work/sweep-1.21.10`, version tree
`versions/1.21.10/` unless stated. Jars:
`~/workspace/mcir-work/e6-deltas/versions/1.21.10/jars/{client-named,server-game-named}.jar`.
Scratch: `~/workspace/mcir-work/sweep-1.21.10-batches/b08/evidence/`.

**Jar (fresh, 1.21.10):**
- `evidence/particle_types.txt` — `ParticleTypes` `javap -c`
  (`<clinit>`; 115 registrations in protocol-id order, 28
  limiter-true; `ELDER_GUARDIAN` id 17, limiter true).
- `evidence/particle_types.json` — the parsed registrations
  (115, with limiter flags).
- `evidence/level_event_handler.txt` — `LevelEventHandler` `javap -c`
  (79-case lookupswitch: NO 2015–2020, NO 1053/1054; 3003 case:
  WAX_ON at offset 3374 + HONEYCOMB_WAX_ON at 3390; 2002 case
  SPLASH_POTION_BREAK at offset 1530; 1504 case:
  `PointedDripstoneBlock.spawnDripParticle` at offset 3519).
- `evidence/mpgm.txt` — `MultiPlayerGameMode` `javap -c`
  (`addBreakingBlockEffect`: 0 calls; hit sound every 4th tick,
  `destroyTicks % 4.0f == 0`, offsets 199–208; volume /8.0 at 243,
  pitch *0.5 at 252; `startDestroyBlock` plays/spawns nothing).
- `evidence/living.txt` — `LivingEntity.randomTeleport` broadcasts
  entity event 46 (bipush 46 at offset 177); `handleEntityEvent`
  case 46 → 128 PORTAL particles (offsets 193/381/399);
  `handleEntityEvent` declared on `LivingEntity`;
  `spawnItemParticles` present.
- `evidence/shulker.txt` — `Shulker.teleportSomewhere`: no particles,
  no entity/level event (0 such refs in the whole class).
- `evidence/enderman.txt` — `EnderMan.teleport` → `randomTeleport(…,
  true)` (offset 110); the only `addParticle` is ambient aiStep
  (offset 90).
- `evidence/dragonegg.txt` — `DragonEggBlock.teleport`: 128 PORTAL
  lerped old→new (loop bound `sipush 128`, offset 135).
- `evidence/spread.txt` — `SpreadPlayersCommand$Position.isSafe`:
  `y < maxY && !liquid && !is(FIRE)`.
- `evidence/teleport_consume.txt` — `TeleportRandomlyConsumeEffect`:
  record with only a `diameter` field.
- `evidence/localplayer.txt` — `LocalPlayer` ctor: 7 args.
- `evidence/particlegroup.txt` — `ParticleGroup.add`: plain queue add
  into an `EvictingQueue.create(16384)` (constructor offsets 5–8).
- `evidence/particle_resources.txt` —
  `ElderGuardianParticle$Provider` registered (offsets 296–300).
- `evidence/clientpacketlistener.txt` — `ClientPacketListener.handleGameEvent`:
  GUARDIAN_ELDER_EFFECT offset 544, ELDER_GUARDIAN addParticle offset
  554, curse sound offset 598.
- `evidence/particleutils.txt` — `ParticleUtils` ×2 methods.
- `evidence/campfire.txt` — `CampfireBlock.animateTick` /
  `makeParticles`, `CampfireBlockEntity.particleTick`.
- `evidence/clientlevel.txt` — `ClientLevel.animateTick`.
- `evidence/dripstone.txt` — `PointedDripstoneBlock.animateTick`
  (0.12 roll, offset 19), `maybeTransferFluid` sends 1504 (offsets
  218/245), `findRootBlock` called with 11, same-block test in
  `lambda$findRootBlock$6`, 0.02 cauldron rule in
  `lambda$animateTick$0`.
- `evidence/entity_*.txt` — `Villager` (12→heart, 13→angry,
  14→happy, 42→splash), `TamableAnimal` (7→heart, 6→smoke),
  `Animal` (18→7× heart, 0.02 gaussian, y+0.5), `Ocelot`
  (41→heart, 40→smoke), `Dolphin` (38→happy_villager), `Allay`
  (18→3× heart), `AbstractHorse` (7→heart, 6→smoke),
  `AbstractVillager` (`addParticlesAroundSelf`).
- `evidence/blocks_parse.py` + the §8 leaves table — all 11 leaf
  registrations parsed from the 1.21.10 `Blocks.<clinit>` (spruce →
  `lambda$static$36` → `TintedParticleLeavesBlock(0.01F)`; azalea
  colour −9399763).
- `evidence/client-jar-list.txt` / `evidence/server-jar-list.txt` —
  jar listings: 0 entries for `SpeleothemBlock`, `Geyser`,
  `EntityTypes`, `BlockEntityTypes`, `SulfurCube`, `Poplar`,
  `MobAppearanceParticle`; 0 `entities_can_teleport_to` /
  `consumable_does_not_teleport_to` / `speleothems` /
  `washed_away_by_fluids` / `blocks_lava_fire_spread` /
  `required_for_poplar_leaf_ambience` datapack entries; the three
  ambient-sound tags present; the 26.x animal/monster/attribute/
  sprite packages all absent; `Cow`/`Dolphin`/`Cat`/`Ocelot`/
  `Parrot` at `world.entity.animal.*`, `Villager` at
  `world.entity.npc.*`, `Zombie` at `world.entity.monster.*`.
- `evidence/javac-*.log` — the 8 single-file oracle compiles
  (7 fail: 8/5/18/4/6/25/26 errors; `VanillaEffectParticlesOracle`
  passes, 0 errors).
- `evidence/jarclasses/` — `Blocks`/`LeavesBlock`/
  `TintedParticleLeavesBlock`/`UntintedParticleLeavesBlock` extracted
  from the 1.21.7 and 1.21.10 server jars for the md5 comparison
  (leaves classes byte-identical; `Blocks.class` differs).

**Data (1.21.10's own):**
- Datagen `datagen/reports/registries.json`:
  `minecraft:particle_type` (115 entries, ids contiguous 0–114;
  elder_guardian 17, flash 42, vibration 48, portal 58, note 56; no
  geyser/sulfur/noxious/poplar); `minecraft:entity_type` (153
  entries; no nautilus/zombie_nautilus/camel_husk).
- Datagen `datagen/reports/blocks.json` (1,166 blocks): state id 1 is
  `minecraft:stone`; `red_poplar_leaves` absent (the only zoo name
  absent).
- Datagen `datagen/reports/items.json`: chorus_fruit consumable is
  `{"on_consume_effects": [{"type": "minecraft:teleport_randomly"}]}`;
  0 occurrences of `directional_particles`.
- No prepared `.local/horizon-client/1.21.10/` exists in the worktree
  (see §12).

**Port source (1.21.10 tree):**
- `crates/horizon-client-core/src/particle_type.rs` — 125 variants
  (macro list), no `ElderGuardian`; `override_limiter` :152-189 (31
  entries: 27 real + 4 geyser).
- `crates/horizon-client/src/game/lan_particles.rs` —
  `hub_particle` :411, Color match :437 (no Flash arm);
  `received_particle` :469, `from_name` :488; `Item | Trail => None`;
  `network_particle` :406.
- `crates/horizon-client/src/game/commands/world_cmds.rs` —
  `cmd_particle` :708, `particle.notFound` :718, catch-all
  `simple(ty)` :829, `network_particle` call :883; `spread_safe`
  :2091-2114, tag read :2100-2109, spread loop :1978-1986, failure
  :1991-1999.
- `crates/horizon-client/src/animate_tick.rs` — leaves :140-151
  (spruce :151), poplar arms :144-146, azalea :148-150, dripstone
  table :397-410, `dripstone_drip` :1807, bubble column :1660,
  tag bindings :315-322/:418-421.
- `crates/horizon-client/src/game/level_particles.rs` —
  `dragon_egg_particles` :15, `shulker_teleport_particles` :20,
  `consume_teleport_particles` :23, `enderman_teleport_particles`
  :28, `block_particle_event` :35 (3003→WaxOn :38),
  `is_stalactite` :243-244, `dripstone_drip_particle` :256,
  `breaking_block_effects` :353-373, `potion_splash_particles` :378.
- `crates/horizon-client/src/game.rs` — 2019/2020 dispatch
  :4309-4317, 2015 :4339-4342, 2016 :4344-4347, 2017 :4349-4352, 2018
  :4354-4357, 2002/2007 :4368/4373, 3002..=3005 :4378, 1504 :4388,
  1053|1054 :4500/:4519, `wax_on_sound` :8043 (callers :8354/:8373).
- `crates/horizon-client/src/game/remote.rs` — digging call :8172,
  event-46 handler :4774-4777, game event 10 apparition set :4990,
  wax sounds :8802/:8810.
- `crates/horizon-client/src/game/consume.rs` — `portal_particles`
  :208, `entity_teleport_particles` :228,
  `check_position_and_teleport` event 46 :188-194,
  `directional_particles` :422-431.
- `crates/horizon-client/src/game/creatures.rs` — apparition set
  :4187. `crates/horizon-client/src/game/mob_blocks.rs` —
  `APPARITION_TICKS = 30` :87, `elder_apparition_geometry` :96.
- `crates/horizon-client/src/mobs/enderman.rs` — event 46 :160-166,
  2018 send :174-184. `crates/horizon-client/src/mobs/shulker.rs` —
  2016 send :366-371.
- `crates/horizon-client-render/src/particles.rs` —
  `entity_event_burst` :627, `create` :2065
  (`pending.push` :2098), `make` :2216 (outer match :2277, 125/125
  arms), `block_teleportation` :1859,
  `enderman_teleportation` :1884, `tick_tracking` :3888, cap
  :3933-3945, `MAX_PARTICLES` :1358-1359, firework flash :3785, 34
  tests.
- `crates/horizon-protocol/src/entity_data.rs` — `Particle::read`
  :185 (State ids :214, Color ids :215, vibration :265). `adapter/ids.rs` —
  identity when no adapter is active.
- `crates/horizon-items/src/items.rs:157-160` — `directional_particles`
  default true. `crates/horizon-world/src/teleport_consume.rs` —
  field :6, 2017 fire :109-121. `crates/horizon-items/src/blocks.rs:121`
  — `has_class` exact match.
- `crates/horizon-client-render/src/mesher.rs:910` —
  `SpeleothemBlock` offset gate (cross-ref).
- `crates/horizon-world/src/level/interact.rs:594` — 2015 send.
- Drivers: `tools/test_particle_providers_parity.py` — SIMPLE :26-38
  (103 kinds), kinds :74-76 (126 total distinct), geyser/noxious sections
  :94-105, STONE :23; `tools/test_marker_particles_parity.py` —
  zoo_states :76 (37 distinct names), red_poplar :118, spruce
  comment :114, neighbour_cases :168, campfire_cases :37,
  wither_rose_cases :156; `tools/test_entity_particles_parity.py` —
  EVENTS :17 (12 entities, 30 pairs).
- Oracles: `mods/horizon/src/test/java/dev/horizonlab/parity/` —
  `VanillaMarkerParticlesOracle.java:44` (9-arg super call),
  :104/:116/:233 (world.attribute), :218 (bindComponents),
  :136 (stale speleothem comment);
  `VanillaParticleProvidersOracle.java:131` (unknown-particle
  throw), :143-146/:164-165 (geyser refs), :22 (sprite package),
  :46 (jspecify), :107 (TextureAtlasSprite), :115/116/124 (Material);
  `VanillaBlockParticlesOracle.java:13/:17/:42/:44-47`;
  `VanillaEmitterParticlesOracle.java:23/:120/:126`;
  `VanillaEntityParticlesOracle.java:17-26/:47-50`;
  `VanillaItemParticlesOracle.java:28-31`;
  `VanillaEffectSyncOracle.java:100/:102-110/:113/:148`.
- Batch 0 table: not landed at the time of this batch (b0/evidence
  held only jar listings); oracle verdicts rest on this batch's own
  javac measurement (§9).

## §16 Verdict

**274 behaviour checks: 262 pass, 12 gap. 22 items: 10 behaviour
defects (9 minor, 1 major), 4 live needs-harness items + 1 closed
(WITHDRAWN), 7 oracle defects.**

All 3 of the 1.21.7 Batch 8 items reproduce in 1.21.10, and 15 of the
16 26.1.2 items reproduce — the ten behaviour defects, three
needs-harness items and both oracle defects — with B8-H04 WITHDRAWN
(`zombie_nautilus`/`camel_husk` do not exist in 1.21.10). The
reproductions were verified fresh against 1.21.10's own jar/data, not
carried: the group cap (B8-G05) is a different defect here (eviction,
not no-cap), the providers driver carries 13 non-1.21.10 kinds, not
11 (B8-H02), and both oracle defects are worse than in 26.1.2
(B8-O01: 6 errors; B8-O02: 25 errors). New in this run: five more
broken particle oracles (B8-O03–O07) — 7 of the 8 particle-family
oracles fail to compile for 1.21.10, because the oracle sources are
26.3-form — and a new needs-harness item (B8-H05: the entity driver
sends `nautilus` cases for an entity 1.21.10 lacks). No new behaviour
defects were found beyond the re-checks: the port's particle tables
are otherwise exact. Nothing is UNVERIFIED; the Batch 7-routed flag
is decided (real, minted as B8-G10). The batch is complete and ready
for the coordinator to land — with two reconciliation notes: the
oracle verdicts rest on this batch's own javac measurement because
Batch 0's table had not landed, and there is no prepared 1.21.10
data in the worktree, so the driver runs wait on a prepare as well
as the oracle repairs.
