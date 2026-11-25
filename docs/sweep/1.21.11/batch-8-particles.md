# Batch 8 — Particles (1.21.11 parity sweep)

**Version:** 1.21.11 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.11`
(workdir `~/workspace/mcir-work/sweep-1.21.11`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 8 — all 115 particle types,
emitters, providers; 8 drivers found in the tree (see §10).

**Status: DRAFT — not committed.** Checks + triage only, per the batch
brief: no fixes, no edits under `versions/1.21.11/`, no cargo builds, no
commits. Everything below is static analysis: `javap` on the named
1.21.11 jars (`~/workspace/mcir-work/e6-deltas/versions/1.21.11/jars/{client-named,server-game-named}.jar`;
the `server.jar` in that dir is a Mojang bundler jar and was never
javap'd), direct bytecode parsing of `ParticleTypes.<clinit>` (via the
`javap -c` dump) and of `Blocks.<clinit>` (a class-file parser written
for the 1.21.10 batch, `evidence/blocks_parse.py`, re-run fresh against
the 1.21.11 jar), source comparison against `versions/1.21.11/crates/`,
data checks against the 1.21.11 datagen reports (`e6-deltas/versions/1.21.11/datagen/reports/`,
since the worktree has no prepared `.local/`), and oracle triage from
this batch's own single-file `javac` measurement of the 8 particle-family
oracles — Batch 0's measured table had not landed when this batch ran
(b00's evidence dir held only cargo-check artifacts at the time), so
there is no Batch 0 table to triage from; the javac measurement here uses
the same "both" classpath Batch 0 defines (both named jars + the game's
library jars + jsr305) and the same standalone definition (each file
compiled alone with `-sourcepath`). No driver runs, no cargo tests.
The coordinator commits these deliverables.

**Headline:** all ten 1.21.10 behaviour defects reproduce in the
template itself (B8-G01–G10, verified fresh against 1.21.11's own
jar/data — the template is where they live, and the 1.21.10 child
inherited them unchanged), and the particle registry is byte-for-byte
identical to 1.21.10's (115 types, same names, same protocol ids). But
vanilla changed between 1.21.10 and 1.21.11 in exactly the places this
batch touches, and the re-checks split on those changes. **The nautilus
family arrived** (nautilus, zombie_nautilus, camel_husk, parched — the
4 new entities): the entity driver's 3 nautilus pairs are now legitimate
behaviour checks (B8-H05 WITHDRAWN, +3 checks vs 1.21.10), while the
driver's omission of zombie_nautilus and camel_husk is a live gap again
(B8-H04 REVERSED — the 1.21.10 withdrawal does not carry). **The 26.x
entity subpackages arrived** (`animal/cow`, `animal/dolphin`,
`animal/equine`, `animal/feline`, `animal/nautilus`, `animal/parrot`,
`npc/villager`, `monster/zombie`) with the `world.attribute` package,
jspecify in the game libraries, and the 7-arg `TextureAtlasSprite`
constructor — so two oracles that failed for 1.21.10 now compile clean
(B8-O04 emitter, B8-O05 entity — FIXED), and the five still-broken ones
fail with far fewer errors (Marker 6→2, Providers 25→19, Block 8→7,
EffectSync 26→1; Item unchanged at 4). **New in this run:** nothing
beyond the re-checks — no new behaviour defect was found; the port's
particle tables are otherwise exact (see §6.5).

## §0 Execution status

Complete. All 1.21.10 re-checks (22, PRIMARY) and all 1.21.7 re-checks
(3, SECONDARY) decided; no UNVERIFIED carry-overs. The Batch 7-routed
flag carried forward as B8-G10 (`/spreadplayers` always fails, major)
was verified fresh against 1.21.11's own jar/data and reproduces. No
stalls: every jar needed was present, every class needed was
javap'd fresh against the 1.21.11 jars, and no driver build was
attempted (out of scope for this batch). Two caveats: Batch 0's
measured oracle table had not landed, so the oracle verdicts rest on
this batch's own javac measurement (method in §2, §9) rather than the
Batch 0 table — the coordinator should reconcile against the Batch 0
table when it lands; the error counts here are single-file
standalone compiles on the "both" classpath and may differ from
Batch 0's per-classpath attribution, but a file that fails standalone
on "both" fails on every classpath. Second, a transient disk-quota
error truncated the first `ParticleTypes` dump mid-`<clinit>`; it was
re-dumped complete (841 lines, 117 putstatics) and the parse below
uses the complete dump (§14).

## §1 Scope

`docs/PARITY-SWEEP.md` §4 Batch 8: particle registry + type coverage,
wire codec, override limiter, entity event bursts, marker zoo, leaves,
dripstone + tags, oracle API existence. Method: registry comparison
(1.21.11's `particle_type` registry vs the port's `ParticleType`),
codec comparison, source comparison — plus fresh `javap` verification
of every vanilla claim against 1.21.11's own jars (no md5 carry from
another version: 1.21.11 is the E6 template, not a 26.x tree, so no
26.x jar evidence carries; and 1.21.10's evidence does not carry
either — vanilla changed between 1.21.10 and 1.21.11 in the entity
packages, the attribute package, the libraries and the entity
registry). No cargo builds, no driver runs. Driver-coverage gaps are
needs-harness items (B8-Hxx), not behaviour gaps.

Files read in the 1.21.11 tree (particle scope): `particle_type.rs`,
`animate_tick.rs`, `entity_data.rs`, `world_cmds.rs`,
`level_particles.rs`, `particles.rs`, `game.rs`, `remote.rs`,
`consume.rs`, `items.rs`, `teleport_consume.rs`, `mobs/enderman.rs`,
`mobs/shulker.rs`, `mobs/guardian.rs` (elder curse param),
`blocks.rs`, `mesher.rs`, `mob_blocks.rs`, `creatures.rs`,
`interact.rs`, `adapter/ids.rs`; all 8 particle drivers; all 8
particle oracles.

## §2 Method

- **Jar identity first.** 1.21.11's own named jars, sha1-verified
  against the common brief (client-named
  `29bddf144a2f2907528b25d616770bfea67c4792`, server-game-named
  `7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2`, bundler server.jar
  `64bb6d763bed0a9f1d632ec347938594144943ed` — never javap'd). Every
  vanilla claim was javap-verified fresh against these jars — nothing
  here rests on a 26.x, 1.21.7 or 1.21.10 jar finding. Cross-version
  comparisons were made by re-reading the class, not by carrying a
  verdict: the particle registry was diffed name-by-name and
  id-by-id against 1.21.10's (identical), the entity registry diffed
  the same way (+4: camel_husk, nautilus, parched, zombie_nautilus),
  and the library list diffed against 1.21.10's version.json (107 vs
  115 entries — jspecify arrives in 1.21.11, Jackson/nimbus/httpclient
  leave).
- **Registry census from 1.21.11's own data.** Datagen
  `registries.json` (115 types, ids contiguous 0–114);
  `ParticleTypes.<clinit>` parsed from the `javap -c` dump (115
  registrations in protocol-id order, 28 limiter-true, 0 mismatches
  vs the datagen registry). `ParticleTypes.class` md5 is identical
  in both 1.21.11 jars (same size 15,634 in both listings).
- **Leaves from a class-file parser.** `Blocks.<clinit>` was parsed
  directly (`evidence/blocks_parse.py`, the 1.21.10 batch's parser,
  re-run unmodified against the 1.21.11 server-game jar): each leaf
  name's `ldc` was associated with its registration lambda through
  the `BootstrapMethods` table, and each lambda's constructed class +
  float constant read out. All 11 leaf blocks resolved.
- **Oracle measurement by this batch.** Each of the 8
  particle-family oracles was compiled alone with `javac -nowarn`
  (JDK 25, the standard env compiler — see §14 for the JDK 21
  cross-check) on the "both" classpath (both named jars + all 107
  of 1.21.11's own game library jars, downloaded from Mojang's
  library host per version.json and sha1-verified, `evidence/libs/`,
  + `tools/jsr305-3.0.2.jar` — the 1.21.11 tree has no jsr305 copy,
  so the 1.21.10 tree's identical artifact was used) with
  `-sourcepath` on the test java root — the same standalone
  definition Batch 0's fixpoint uses. Logs in `evidence/javac-*.log`.
  Error counts are the javac counts; root causes were verified
  against the jar listings (package absence) and `javap`
  (constructor arities).
- **26.1.2-record corrections applied** (from the 26.1.2 summary,
  carried through 1.21.10): B8-H02 counts the kinds the driver's
  `cases()` generator actually yields, not just the `SIMPLE` list;
  the oracle-API check "Villager.handleEntityEvent" is attributed to
  `LivingEntity` (the method is declared there, inherited by
  Villager); B8-G10 (`/spreadplayers`) was checked fresh in 1.21.11.
- **1.21.10 handover items applied** (from the 1.21.10 summary §"What
  the 1.21.11 run needs"): the particle limiter boolean is read as
  the `iconst` AFTER the name `ldc` (forward walk, §14); the holder
  shape was established by jar listing (1.21.x shape — no
  EntityTypes/BlockEntityTypes classes in either jar); the ServerLevel
  ctor arity was established by javap (11-arg with RandomSequences);
  oracle/harness/driver-status checks are NOT tallied as behaviour
  checks (§3); full `Vanilla*Oracle` names are used throughout.

## §3 Checks table (tally)

**Behaviour checks: 277 — 265 pass, 12 gap** (across 10 defect items,
B8-G01–B8-G10: 9 minor, 1 major). Counting rule: a behaviour check is
one comparison where BOTH the vanilla side (1.21.11 jar/data) and the
Rust side were actually compared; oracle-status checks are NOT counted
in the tally (per the common brief). Non-behaviour checks tallied: 0.
As-tallied 277 = 277 behaviour + 0 non-behaviour.

| Section | Total | Pass | Gap | Gaps |
|---|---|---|---|---|
| Registry / type coverage | 118 | 116 | 2 | B8-G01 (per-type coverage, kind set) |
| Wire codec | 20 | 19 | 1 | B8-G02 |
| Override limiter | 28 | 27 | 1 | B8-G01 (flag) |
| Entity event bursts | 30 | 30 | 0 | — |
| Marker zoo | 45 | 45 | 0 | — |
| Leaves | 12 | 11 | 1 | B8-G03 |
| Dripstone + tags | 7 | 6 | 1 | B8-G04 |
| Oracle API existence | 8 | 8 | 0 | — |
| Other passes | 2 | 2 | 0 | — |
| Level events 2015–2020 + digging | 5 | 1 | 4 | B8-G06, G07, G08, G09 |
| Particle group cap | 1 | 0 | 1 | B8-G05 |
| Routed check: /spreadplayers safety (B7 flag) | 1 | 0 | 1 | B8-G10 |
| **Total** | **277** | **265** | **12** | |

118+20+28+30+45+12+7+8+2+5+1+1 = 277; 265 + 12 = 277.

(Tally vs the 1.21.10 template's 274: +3 in entity bursts — the
driver's 3 `nautilus` pairs HAVE a vanilla 1.21.11 side (nautilus
inherits TamableAnimal 6/7 and Animal 18), so they are behaviour
checks here, not driver scope (B8-H05 withdrawn). Every other section
is unchanged in count.)

- **Oracle defects: 5 live** (B8-O01 — marker oracle, 2 errors;
  B8-O02 — providers oracle, 19 errors; B8-O03 — block oracle, 7
  errors; B8-O06 — item oracle, 4 errors; B8-O07 — effect-sync
  oracle, 1 error) **+ 2 fixed** (B8-O04 — emitter oracle, 0 errors;
  B8-O05 — entity oracle, 0 errors). `VanillaEffectParticlesOracle`
  also compiles (0 errors, as in 1.21.10). So 3 of the 8
  particle-family oracles compile for 1.21.11, vs 1 of 8 for 1.21.10
  (§9).
- **Needs-harness items: 4 live + 1 closed** (B8-H01–H04, §4;
  B8-H05 closed as WITHDRAWN, §5): driver-coverage gaps, not
  behaviour gaps.
- **Drivers: 0 run, 8 inventoried** (7 particle drivers + 1
  particle-adjacent effect-sync driver). 5 of the 8 are
  oracle-blocked (B8-O01–O03, O06, O07); 2 are oracle-clean but
  still unrun (emitter, entity — their oracles compile, the runs
  need prepared data + a build, §13); the 8th
  (`native_client_particles`) has no vanilla oracle. None is
  runnable as-is.
- **Unit tests:** 34 `#[test]`s in `particles.rs` exist (26 vanilla +
  8 mod-particle); unrun, blocked.

## §4 Defects

### B8-G01 — `elder_guardian` has no port implementation at all (MINOR, REPRODUCES from 1.21.7, 26.1.2 and 1.21.10)

**Checks failed (3):** per-type end-to-end coverage for
`elder_guardian`; its `override_limiter` flag; the providers-driver
kind set (which omits it).

**Evidence (jar, fresh for 1.21.11).** Vanilla 1.21.11 registers
`ELDER_GUARDIAN` as a `SimpleParticleType` (`ParticleTypes.<clinit>`,
wire id 17, `overrideLimiter=true` — parsed from the `javap -c` dump
of the 1.21.11 client jar) and registers a provider for it in
`ParticleResources.registerProviders()` — a dedicated
`ElderGuardianParticle$Provider` (offsets 296–300, same as 1.21.10).
`ClientPacketListener.handleGameEvent` spawns it at the player's
position on game event `GUARDIAN_ELDER_EFFECT` (offset 544;
`addParticle(ELDER_GUARDIAN, ...)` at offset 554, plus the curse
sound at 598 — the same offsets as 1.21.10).

**The port.** `ParticleType` (`horizon-client-core/src/particle_type.rs`)
has 125 variants and no `ElderGuardian` (grep over
`particle_type.rs`, `lan_particles.rs`, `particles.rs`: zero hits).
Consequences, all verified in source:

- `received_particle` (`game/lan_particles.rs:469`, `from_name` at
  :491) resolves a server-sent particle by name and returns `None`
  for `elder_guardian` — a `LevelParticles` packet for it is silently
  dropped.
- `/particle minecraft:elder_guardian` fails with `particle.notFound`
  (`game/commands/world_cmds.rs:718`, `from_name` returns `None`);
  vanilla accepts it.
- `override_limiter()` (`particle_type.rs:152-189`) has no entry for
  it (28 vanilla limiter types vs 27 real-1.21.11 Rust entries; the
  only 1.21.11 miss is `elder_guardian` — the Rust list's other 4
  entries are the geyser types, dead in 1.21.11).
- The providers driver does **not** send `elder_guardian` cases in
  the 1.21.11 tree (its 126-kind set omits the type entirely), so
  unlike 1.21.7 — where the driver sent 12 cases and the probe
  printed `no provider` — the driver is not blocked by this defect;
  it silently skips the type.

**Mitigation (why minor).** The one in-game path that matters — the
elder guardian's curse apparition — is re-implemented outside the
particle engine, same as in 1.21.7, 26.1.2 and 1.21.10: game event 10
sets `elder_apparition` (`game/remote.rs:4990`,
`game/creatures.rs:4186`) and `elder_apparition_geometry`
(`game/mob_blocks.rs:96`) draws the ghostly guardian with the vanilla
30-tick lifetime (`APPARITION_TICKS = 30`, `mob_blocks.rs:87`). So
the apparition works; what is missing is the particle-type plumbing
(`/particle`, server packets). Visual fidelity of the
re-implementation against vanilla's `ElderGuardianParticle` is
unverified statically — needs the driver run or an in-game look.

### B8-G02 — `/particle minecraft:flash` sends a malformed packet to LAN guests (MINOR, REPRODUCES from 1.21.7, 26.1.2 and 1.21.10)

**Checks failed (1):** the `Flash` arm of `hub_particle`.

**Evidence.** `Particle::read` (`horizon-protocol/src/entity_data.rs:185`,
State ids :214, Color ids :215) decodes wire id 42 (`flash`, verified
against the 1.21.11 registry — id 42 here as in 1.21.10, 41 in 1.21.7)
as `Color(i32)` — a flash particle on the wire always carries a
colour. But `hub_particle` (`game/lan_particles.rs:411`), which builds
the packet a LAN host sends, has no `Flash` arm in its `Color` match
(`EntityEffect | TintedLeaves` only, :440), so a flash particle is
built with `data: None`, and `Particle::write` (:297 — which writes
data by presence, not by id: `let Some(data) = &self.data else {
return }`) emits the id with no colour bytes. Any guest reading id 42
then consumes the next 4 bytes of the stream as the colour and
continues misaligned: packet desync (and likely a disconnect or a
dropped connection) for every guest in range.

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

### B8-G03 — Spruce leaves never drop falling-leaf particles (MINOR, REPRODUCES from 1.21.7, 26.1.2 and 1.21.10, driver-blocking)

**Checks failed (1):** the spruce entry of the per-block leaves check.

**Evidence (jar, fresh for 1.21.11).** Vanilla 1.21.11 registers
`SPRUCE_LEAVES` as a `TintedParticleLeavesBlock` with
`leafParticleChance = 0.01` — parsed fresh from the 1.21.11
`Blocks.<clinit>` with the class-file parser
(`evidence/blocks_parse.py`: `ldc "spruce_leaves"` →
`lambda$static$36` → `new TintedParticleLeavesBlock`, float `0.01F`;
same lambda number as 1.21.10).
`LeavesBlock.animateTick` calls
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

### B8-G04 — Level-event 1504 stalactite drip particle never fires: its gate checks for `SpeleothemBlock`, which does not exist in 1.21.11 (MINOR, REPRODUCES from 26.1.2 and 1.21.10)

**Checks failed (1):** the level-event 1504 gate in the dripstone
section.

**Evidence (jar + datapack, fresh).** Vanilla 1.21.11 has no
`SpeleothemBlock` class at all (0 entries in either jar;
`PointedDripstoneBlock` extends `Block` directly) and no
`speleothems` tag in the client jar's datapack (206 block tags, none
of them `speleothems`). Vanilla 1.21.11's `LevelEventHandler` routes
1504 to `PointedDripstoneBlock.spawnDripParticle(level, pos, state)`
(offset 3519, no class gate), and vanilla's
`PointedDripstoneBlock.maybeTransferFluid` sends level event 1504
when a stalactite drips fluid into a cauldron below (offsets 218 and
245 in `dripstone.txt`).

**The port.** The client routes 1504 to `dripstone_drip_particle`
(`game.rs:4377`), whose first act is the `is_stalactite` gate
(`game/level_particles.rs:243-244`): `has_class("SpeleothemBlock")`
(`horizon-items/src/blocks.rs:121`, an exact match over the prepared
class chain). In 1.21.11 that is false for every block, so the
function returns before spawning anything — the cauldron-fill drip
particle never appears. The ambient animateTick drip path is
unaffected: it is name-based (`AnimateTables.dripstone`, built from
`"pointed_dripstone"` by name, `animate_tick.rs:397-410`) and matches
vanilla (§8).

**Scope note.** Whether the port's *server* actually transfers fluid
to cauldrons in 1.21.11 (pilot G6's 1.21.11 status) is Batch 5's scope;
the client gate is dead regardless of server reachability, and the
defect is minted on the client side only. The same `SpeleothemBlock`
class assumption appears elsewhere in the tree (see §7) — those are
other batches' scope.

### B8-G05 — Particle engine drops new particles at the 16,384 cap (with probabilistic thinning above 12,288); vanilla 1.21.11 evicts the oldest instead (MINOR, REPRODUCES from 26.1.2 and 1.21.10 with the same vanilla form as 1.21.10)

**Checks failed (1):** the pending-particle drain in `tick_tracking`.

**Evidence (jar, fresh).** Vanilla 1.21.11's `ParticleGroup.add`
(`javap -c`, 1.21.11 client jar) is a plain queue add —
`particles.add(p)`, no size check, no probability roll — but the
queue itself is an `EvictingQueue.create(16384)` (constructor,
offset 5–8: `sipush 16384` at 5, `EvictingQueue.create` at 8): adding
past 16,384 evicts the *oldest* particle. There is no probabilistic
thinning in the last quarter and no level-random draw on add. Same
form as 1.21.10 (and unlike 26.1.2, where the queue was plain and no
cap was enforced at all).

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

### B8-G06 — Chorus fruit shows an extra 2017 directional teleport trail vanilla never sends (MINOR, REPRODUCES from 26.1.2 and 1.21.10; 26.3-form)

**Checks failed (1):** the level-event 2017 check.

**Evidence (jar, fresh).** Vanilla 1.21.11's
`TeleportRandomlyConsumeEffect` is a record with ONE field
(`diameter: float`; constructors `()` and `(float)`) — there is no
`directional_particles` field. Its `apply` calls
`LivingEntity.randomTeleport(x, y, z, true)`, which broadcasts
entity event 46 (`bipush 46`, offset 177 in `living.txt`);
`LivingEntity.handleEntityEvent` case 46 spawns 128 portal particles
at the entity (`sipush 128`, offsets 201/208; `PORTAL` at 389,
`addParticle` at 407). Vanilla 1.21.11's `LevelEventHandler` has NO
2017 case (79-case lookupswitch, 2000–2013 then 3000+).

**The port.** The 1.21.11 tree carries the 26.3 form:
`TeleportConsume` has a `directional_particles: bool` field
(`horizon-world/src/teleport_consume.rs:6`); `items.rs:157-160` reads
it from data with `unwrap_or(true)` — and the 1.21.11 data never
carries the field (`chorus_fruit`'s consumable is
`{"on_consume_effects": [{"type": "minecraft:teleport_randomly"}]}`,
0 occurrences of `directional_particles` in the datagen
`items.json`), so it defaults to true;
`teleport_consume.rs:109-121` then fires the event on every
successful teleport, and the client impl (`game/consume.rs:422-431`)
calls `level_event_at(2017, ...)`, which routes (`game.rs:4338-4341`)
to `consume_teleport_particles` → `block_teleportation(pos, data,
[127,127,127])` (`particles.rs:1859`) — 128 portal particles
interpolated along the old→new line. The port ALSO fires the
vanilla-correct entity event 46 (`consume.rs:188-194`,
`remote.rs:4774-4777` → `entity_teleport_particles`, 128 portal
particles at the entity). So a chorus-fruit teleport in the 1.21.11
port shows BOTH the correct 128-particle burst AND a second,
128-particle directional trail vanilla never shows. (Batch 7
independently flagged the 2017 trail as masked by its G09 in the
26.1.2 run — the trail fires only after a successful teleport; the
masking claim is Batch 7's, not verified here. The trail mechanism
itself is verified fresh here.)

### B8-G07 — Enderman teleport shows an extra 2018 directional portal trail vanilla never sends (MINOR, REPRODUCES from 26.1.2 and 1.21.10; 26.3-form)

**Checks failed (1):** the level-event 2018 check.

**Evidence (jar, fresh).** Vanilla 1.21.11's `EnderMan.teleport`
calls `randomTeleport(x, y, z, true)` (offset 110) → entity event 46
→ 128 portal particles at the entity. The only `addParticle` call in
`EnderMan` is the ambient aiStep emission (offset 90), not the
teleport. `LevelEventHandler` has NO 2018 case.

**The port.** The 1.21.11 tree's `teleport_to`
(`mobs/enderman.rs:80`, event 46 at :160-166, 2018 send at :174-184)
pushes BOTH the vanilla-correct entity event 46 AND
`MobEvent::LevelEvent { id: 2018, ... }` (when not silent), which
routes (`game.rs:4343-4346`) to `enderman_teleport_particles` →
`enderman_teleportation(pos, data, particle_level)`
(`particles.rs:1884`) — 128 portal particles along the old→new
line. Every enderman teleport in the 1.21.11 port shows the correct
burst plus a second, wrong trail. (The destination-selection half of
the enderman rework is Batch 3's scope; only the particle-visible
trail is minted here.)

### B8-G08 — Shulker teleport shows a 2016 directional trail; vanilla shows NO particles at all (MINOR, REPRODUCES from 26.1.2 and 1.21.10; 26.3-form)

**Checks failed (1):** the level-event 2016 check.

**Evidence (jar, fresh).** Vanilla 1.21.11's
`Shulker.teleportSomewhere` does: `unRide`, `setAttachFace`,
`playSound(SHULKER_TELEPORT)`, `setPos`, `gameEvent(TELEPORT)`, set
peek data, `setTarget`. No `randomTeleport`, no `addParticle`, no
`broadcastEntityEvent`, no level event anywhere in the class (0 such
references in the full `javap -c` dump of the 1.21.11 server-game
jar). A shulker teleport in vanilla 1.21.11 shows zero particles.
`LevelEventHandler` has NO 2016 case.

**The port.** `mobs/shulker.rs:366-371` pushes
`MobEvent::LevelEvent { id: 2016, ... }` on every teleport, and the
client handles it (`game.rs:4333-4336` →
`shulker_teleport_particles` → `block_teleportation(pos, data,
[8,8,8])`, `particles.rs:1859`), so every shulker teleport shows 128
portal particles along the old→new line that vanilla never shows.
Note the asymmetry with G06/G07: here there is no correct event-46
burst underneath — the trail is the ONLY particle output, and it is
entirely wrong.

### B8-G09 — Digging shows a crack particle (and a mistimed hit sound) vanilla never shows (MINOR, REPRODUCES from 26.1.2 and 1.21.10; 26.2-form breaking effects)

**Checks failed (1):** the digging crack-particle check (level
events 2019/2020 area).

**Evidence (jar, fresh).** Vanilla 1.21.11's `MultiPlayerGameMode`
never calls `addBreakingBlockEffect` anywhere in the class (0
occurrences in the full `javap -c` dump), and `LevelEventHandler`
has NO 2019/2020 cases. `startDestroyBlock` plays nothing and
spawns nothing. `continueDestroyBlock` plays ONLY the hit sound, on
destroy ticks 0, 4, 8, … (`destroyTicks % 4.0f == 0`, offsets
199–208; `getSoundType` at 212, volume `/ 8.0`, pitch `* 0.5`) — no
particle at any point while digging.
(`ClientLevel.addBreakingBlockEffect` exists as a method but is
never invoked on this path.)

**The port.** The 1.21.11 tree keeps the 26.2 form:
`breaking_block_effects` (`game/level_particles.rs:353-373`: hit
sound when `play_sound`, plus `crack_particle(pos, face, state)` —
one crumb on the mined face), called from `startDestroyBlock`
(`game/remote.rs:8171`, `breaking_block_effects(hit.pos, hit.face,
true)`) with the comment "startDestroyBlock plays the first hit and
crack itself; the server's level events 2019 and 2020 do the rest".
But the port's server never sends 2019/2020 (no sends in
`horizon-world` or `game/`), so the network dispatch
(`game.rs:4298-4306`) is dead and the continue path plays NOTHING —
no every-fourth-tick sound at all. Net effect vs vanilla: an extra
crack particle + a hit sound at the first hit (vanilla: silence),
then silence for the rest of the dig (vanilla: a hit sound every 4th
tick). The particle half (extra crumb) is minted here; the
sound-schedule half is flagged for Batch 10 (§7).

### B8-G10 — `/spreadplayers` always fails: `spread_safe` requires a tag 1.21.11 does not have (MAJOR, REPRODUCES from 26.1.2 and 1.21.10; Batch 7-routed flag, verified fresh)

**Checks failed (1):** the `/spreadplayers` safety check (routed
from Batch 7 §8.6 / slice `out_of_scope_failures` in the 26.1.1 run).

**Evidence (jar + data, fresh).** Vanilla 1.21.11's
`SpreadPlayersCommand.Position.isSafe` (`javap -c`, 1.21.11
server-game jar) is `y < maxY && !state.liquid() &&
!state.is(BlockTags.FIRE)` — three conditions, no teleport tag.
The `#entities_can_teleport_to` tag does not exist in 1.21.11: 0
entries in either jar's datapack (206 block tags in the client jar,
none teleport-related).

**The port.** `spread_safe` (`game/commands/world_cmds.rs:2091-2114`)
returns `y < max_y && !liquid && teleportable`, where `teleportable`
(:2100-2109) requires membership in `#entities_can_teleport_to` —
always false in 1.21.11. So `spread_safe` is ALWAYS false. In the
spread loop (:1978-1986), every collision-free iteration
re-randomizes every point (none is ever "safe"), `collisions` stays
true, and the loop runs until `iteration >= 10_000`, when the command
fails with `commands.spreadplayers.failed.entities` (or `.teams`,
:1991-1999). **`/spreadplayers` cannot succeed in the 1.21.11 port
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
- **B8-H02 — providers driver includes 13 kinds 1.21.11 lacks
  (REPRODUCES, count 13).** The driver's `SIMPLE` list
  carries `red/orange/yellow_poplar_leaves` (26.3-only),
  `sulfur_bubbles` and `sulfur_cube_goo` (26.2-only) AND
  `pause_mob_growth` and `reset_mob_growth` (not particle types in
  1.21.11 at all — 7 kinds in `SIMPLE`), and the driver's `cases()`
  generator also yields `noxious_gas`, `noxious_gas_cloud`,
  `geyser`, `geyser_plume`, `geyser_base`, `geyser_poof`
  (`test_particle_providers_parity.py:94-105`, 6 more kinds, 72+
  cases). None of the 13 is in the 1.21.11 registry (115 types, no
  geyser/sulfur/noxious/poplar; `pause_mob_growth` /
  `reset_mob_growth` are not in the registry either — and not in the
  port's `ParticleType` enum, so the probe has no provider for them
  either). The oracle resolves kinds by registry name and throws
  `AssertionError("unknown particle " + id)` for an unregistered
  type (`VanillaParticleProvidersOracle.java:131`), so the driver
  cannot run against the 1.21.11 oracle as-is: it would die on the
  first such case. Trim all 13 kinds for 1.21.11.
- **B8-H03 — marker driver zoo includes `red_poplar_leaves`
  (26.3-only) (REPRODUCES).** The zoo (`test_marker_particles_parity.py`,
  `zoo_states()`, :118) sends
  `red_poplar_leaves[distance=3,...]` cases, but no such block exists
  in 1.21.11 — the only zoo name absent from the 1.21.11
  `blocks.json` (1,166 blocks) — so the oracle cannot build that
  world. (The 1.21.11 port DOES have poplar `classify` arms
  (`animate_tick.rs:144-146`), but they are dead: no poplar block
  exists to classify.) Trim the zoo case for 1.21.11.
- **B8-H04 — entity driver omits `zombie_nautilus` and `camel_husk`
  (REVERSED from the 1.21.10 WITHDRAWN verdict — the gap is LIVE
  again in 1.21.11).** The 26.1.2 item asked for
  `zombie_nautilus`/`camel_husk` cases in the entity driver; the
  1.21.10 run withdrew it because neither entity exists in 1.21.10.
  Vanilla 1.21.11 ADDED both: the entity registry has 157 types
  including `minecraft:zombie_nautilus` and `minecraft:camel_husk`,
  the classes exist in both jars
  (`world/entity/animal/nautilus/ZombieNautilus`,
  `world/entity/animal/camel/CamelHusk`), and both inherit
  particle-burst entity events — `ZombieNautilus` extends
  `AbstractNautilus` extends `TamableAnimal` (events 6/7 via
  `TamableAnimal.handleEntityEvent`, 18 via `Animal`), `CamelHusk`
  extends `Camel` (events 6/7/18 via `AbstractHorse`/`Animal`).
  Neither class overrides `handleEntityEvent` or calls `addParticle`
  itself (0 such refs in either `javap -c` dump), so the inherited
  bursts are the whole story. The driver's `EVENTS` table
  (`test_entity_particles_parity.py:17`, 12 entities, 30 pairs)
  omits both, and the oracle's `TYPES` map
  (`VanillaEntityParticlesOracle.java:47-50`) omits both too — so
  the two entities' bursts have no driver coverage at all, while
  the port's `entity_event_burst` DOES carry arms for them
  (`particles.rs:631`, :641: `zombie_nautilus` in the tamable set,
  `camel_husk` in the horse set). Add both entities (events 6, 7,
  18) to the driver's `EVENTS` and the oracle's `TYPES` for 1.21.11.
- **B8-H05 — WITHDRAWN, closed (see §5).** The 1.21.10 item (entity
  driver sends `nautilus` cases for an entity 1.21.10 lacks) has no
  1.21.11 premise: `nautilus` EXISTS in 1.21.11 (157-type entity
  registry; `Nautilus` class in both jars, extending
  `AbstractNautilus` extends `TamableAnimal`, so events 6/7/18
  inherit the tamable/animal bursts). The driver's 3 nautilus pairs
  are legitimate behaviour checks here (counted in §3, all pass).

### Oracle defects

- **B8-O01 — `VanillaMarkerParticlesOracle` does not compile for
  1.21.11 (REPRODUCES, 2 errors, not 6).** This batch's javac
  measurement (single-file, "both" classpath with 1.21.11's own
  libraries, log `evidence/javac-VanillaMarkerParticlesOracle.log`):
  2 errors. The 26.1.2/1.21.10 break reproduces — :44, the oracle's
  `Player` subclass calls `super(...)` with 9 arguments, the 26.3
  form, but 1.21.11's `LocalPlayer` constructor takes **7**
  (`Minecraft, ClientLevel, ClientPacketListener, StatsCounter,
  ClientRecipeBook, Input, boolean` — no `ChatAbilities`, which is
  26.x) — plus the missing `bindComponents(DataComponentMap)` at
  :218 (`Holder$Reference` has no such method in 1.21.11, verified
  by javap). GONE vs 1.21.10: the three `world.attribute` errors
  (:104, :116, :233) and the dead `@Override` (:116) — the
  `net.minecraft.world.attribute` package EXISTS in 1.21.11 (55
  entries, incl. `EnvironmentAttributeSystem`), and the oracle's
  attribute references now resolve. Blocks the marker_particles
  driver entirely (before it can reach the B8-G03 spruce cases).
- **B8-O02 — `VanillaParticleProvidersOracle` does not compile for
  1.21.11 (REPRODUCES, 19 errors, not 25).** This batch's javac
  measurement: 19 errors. The geyser break reproduces in full — 10
  of the 19 are the geyser references (`GeyserParticleOptions` ×3,
  `GeyserBaseParticleOptions` ×3, `ParticleTypes.GEYSER` /
  `GEYSER_PLUME` / `GEYSER_BASE` / `GEYSER_POOF` ×4; :143-146,
  :164-165; zero `geyser` entries in either 1.21.11 jar) — plus `BlockStateModelSet` ×2
  (:16, :113), `ItemStackTemplate` ×2 (:34, :150), the
  `net.minecraft.client.resources.model.sprite` package (:22),
  `Material` ×4 (:115×2 incl. a dead `@Override`, :116, :124).
  GONE vs 1.21.10 (6 fewer): the 4 jspecify `Nullable` errors (the
  `org.jspecify.annotations` import now resolves — jspecify 1.0.0
  ships in 1.21.11's game libraries) and the `TextureAtlasSprite`
  constructor mismatch (the oracle's 7-arg `super(...)` call
  matches 1.21.11's 7-arg protected ctor; 1.21.10's was 6-arg),
  plus one further missing symbol. Blocks the particle_providers
  driver entirely. Repair is bigger than the 26.1.2 deletion
  (geyser branches + cast helpers); the sprite/Material breaks need
  1.21.11-form rewrites. Do not repair here.
- **B8-O03 — `VanillaBlockParticlesOracle` does not compile for
  1.21.11 (REPRODUCES, 7 errors, not 8).** `BlockStateModelSet` ×2
  (:13, :44), the `net.minecraft.client.resources.model.sprite`
  package (:17), `Material` ×4 (:45×2, :47, plus a dead `@Override`
  at :47). All 26.x-form. GONE vs 1.21.10: the `TextureAtlasSprite`
  constructor mismatch — the oracle's 7-arg `super(...)` matches
  1.21.11's 7-arg protected ctor. Blocks the block_particles driver
  entirely.
- **B8-O04 — `VanillaEmitterParticlesOracle` COMPILES for 1.21.11
  (FIXED, 0 errors).** The 1.21.10 break (5 errors: the
  `net.minecraft.world.entity.animal.cow` package + the `Cow`
  symbol) is gone because the package EXISTS in 1.21.11 — `Cow` is
  at `net.minecraft.world.entity.animal.cow.Cow` in both jars, the
  26.x subpackage form the oracle imports. This batch's javac
  measurement: exit 0, 0 errors. The emitter_particles driver is
  no longer oracle-blocked (it still needs prepared data + a build
  to run, §13).
- **B8-O05 — `VanillaEntityParticlesOracle` COMPILES for 1.21.11
  (FIXED, 0 errors).** The 1.21.10 break (18 errors: ten bad 26.x
  package imports + 8 missing symbols) is gone because the
  packages EXIST in 1.21.11 — `animal/cow`, `animal/dolphin`,
  `animal/equine`, `animal/feline`, `animal/nautilus`,
  `animal/parrot`, `npc/villager` are all real packages in both
  jars, and `Nautilus` exists as a class. This batch's javac
  measurement: exit 0, 0 errors. The entity_particles driver is no
  longer oracle-blocked (it still needs prepared data + a build to
  run, §13; and it still omits zombie_nautilus/camel_husk, B8-H04).
- **B8-O06 — `VanillaItemParticlesOracle` does not compile for
  1.21.11 (REPRODUCES, 4 errors, same count).** A `ServerLevel`
  constructor mismatch (:28 — the oracle passes 10 args, 1.21.11's
  ctor takes 11 with `RandomSequences`) and three missing symbols
  (:29, :31: `ItemStack.item()`, `ItemStack.count()`,
  `bindComponents` — the 26.x item-template API; `bindComponents`
  does not exist on `Holder$Reference` in 1.21.11). Blocks the
  item_particles driver entirely.
- **B8-O07 — `VanillaEffectSyncOracle` does not compile for
  1.21.11 (REPRODUCES, 1 error, not 26).** The single remaining
  error is the `ServerLevel` constructor mismatch (:113 — same
  10-vs-11-arg break as B8-O06). GONE vs 1.21.10 (25 fewer): the
  `monster.zombie` package break (`Zombie` is at
  `net.minecraft.world.entity.monster.zombie.Zombie` in 1.21.11),
  the 21-error cascade from the unresolvable `Zombie` superclass,
  and the missing effect methods (:148: `setGlowingTag`,
  `getSharedFlag`, `hasEffect`, `isCurrentlyGlowing`,
  `removeEffectParticles`, `setInvisible`, `setSharedFlag`,
  `updateInvisibilityStatus` — all present in 1.21.11). Blocks the
  effect_sync driver entirely (its other oracle,
  `VanillaEffectParticlesOracle`, compiles — see §9).

## §5 Re-check verdicts

### 1.21.10 items (PRIMARY — all 22 decided)

| 1.21.10 item | Verdict | 1.21.11 item |
|---|---|---|
| B8-G01 elder_guardian | **REPRODUCES** | B8-G01 (fresh jar evidence: `<clinit>` id 17/limiter, `ElderGuardianParticle$Provider`, game-event offsets re-read from the 1.21.11 jar — same offsets as 1.21.10) |
| B8-G02 flash | **REPRODUCES** | B8-G02 (`entity_data.rs` ids verified fresh against the 1.21.11 registry — flash is id 42 here as in 1.21.10; `hub_particle` still has no Flash arm; `Particle::write` writes data by presence, :297) |
| B8-G03 spruce | **REPRODUCES** | B8-G03 (`Blocks.<clinit>` parsed fresh — spruce is `TintedParticleLeavesBlock(0.01F)` in 1.21.11 too, same lambda number) |
| B8-G04 1504 gate | **REPRODUCES** | B8-G04 (gate unchanged; no `SpeleothemBlock` class or `speleothems` tag in 1.21.11; 1504 case verified fresh at offset 3519) |
| B8-G05 group cap | **REPRODUCES** (same vanilla form as 1.21.10) | B8-G05 (vanilla 1.21.11 caps by eviction via `EvictingQueue(16384)`, same as 1.21.10; the port's drop-new + probabilistic thinning matches neither) |
| B8-G06 2017 chorus | **REPRODUCES** | B8-G06 (`teleport_consume.rs`/`items.rs`/`consume.rs` carry the 26.3 form; `TeleportRandomlyConsumeEffect` record + no-2017 verified fresh; 0 `directional_particles` in 1.21.11 `items.json`) |
| B8-G07 2018 enderman | **REPRODUCES** | B8-G07 (`mobs/enderman.rs` pushes 46 + 2018; `EnderMan.teleport` → event 46 verified fresh) |
| B8-G08 2016 shulker | **REPRODUCES** | B8-G08 (`mobs/shulker.rs` + `game.rs` keep the 2016 send + handler; `Shulker` has 0 particle/event refs, verified fresh) |
| B8-G09 digging crack | **REPRODUCES** | B8-G09 (`level_particles.rs`/`remote.rs` keep the 26.2 form; `MultiPlayerGameMode` 0 `addBreakingBlockEffect` calls + 4th-tick sound verified fresh) |
| B8-G10 /spreadplayers | **REPRODUCES** | B8-G10 (`world_cmds.rs` requires the absent tag; `isSafe` = `y < maxY && !liquid && !is(FIRE)` verified fresh; tag absent from both jars) |
| B8-H01 vibration omitted | **REPRODUCES** | B8-H01 (driver 126-kind set re-counted, vibration absent; oracle handles it at :155-157) |
| B8-H02 13 non-1.21.10 kinds | **REPRODUCES** (count 13) | B8-H02 — 13 kinds (7 in SIMPLE incl. `pause_mob_growth`/`reset_mob_growth` + 6 in `cases()` sections, re-counted fresh against the 1.21.11 registry) |
| B8-H03 red_poplar zoo | **REPRODUCES** | B8-H03 (zoo re-counted: 37 distinct names incl. red_poplar; only red_poplar absent from 1.21.11's `blocks.json`) |
| B8-H04 zombie_nautilus/camel_husk | **REVERSED — the 1.21.10 WITHDRAWN verdict does not carry** | B8-H04, needs-harness (vanilla 1.21.11 ADDED both entities — 157-type registry, classes in both jars, inherited 6/7/18 bursts — and the driver + oracle both omit them; the 26.1.2 gap is live again) |
| B8-H05 nautilus cases | **WITHDRAWN** | B8-H05, closed (nautilus EXISTS in 1.21.11 — registry + class + inherited bursts; the 3 driver pairs are legitimate behaviour checks now, counted in §3, all pass) |
| B8-O01 marker oracle | **REPRODUCES** (2 errors, not 6) | B8-O01 (this batch's javac measurement + fresh LocalPlayer javap: 7-arg ctor; the world.attribute breaks are gone — the package exists in 1.21.11) |
| B8-O02 providers oracle | **REPRODUCES** (19 errors, not 25) | B8-O02 (this batch's javac measurement + fresh geyser-absence check; jspecify + TextureAtlasSprite breaks gone) |
| B8-O03 block oracle | **REPRODUCES** (7 errors, not 8) | B8-O03 (this batch's javac measurement; TextureAtlasSprite break gone) |
| B8-O04 emitter oracle | **FIXED** | B8-O04, fixed (compiles clean, 0 errors — the `animal.cow` package exists in 1.21.11) |
| B8-O05 entity oracle | **FIXED** | B8-O05, fixed (compiles clean, 0 errors — the 26.x entity packages + `Nautilus` exist in 1.21.11) |
| B8-O06 item oracle | **REPRODUCES** (4 errors, same) | B8-O06 (this batch's javac measurement; ServerLevel ctor + item-template API, unchanged) |
| B8-O07 effect-sync oracle | **REPRODUCES** (1 error, not 26) | B8-O07 (this batch's javac measurement; only the ServerLevel ctor remains — zombie package + effect methods all exist now) |

### 1.21.7 items (SECONDARY — all 3 decided)

| 1.21.7 item | Verdict | 1.21.11 item |
|---|---|---|
| B8-G01 elder_guardian | **REPRODUCES** | B8-G01 (fresh jar evidence: `<clinit>` id 17/limiter, `ElderGuardianParticle$Provider`, game-event offsets re-read from the 1.21.11 jar; provider is the dedicated class here, not 1.21.7's shared `MobAppearanceParticle` — which does not exist in 1.21.11) |
| B8-G02 flash | **REPRODUCES** | B8-G02 (`entity_data.rs` ids verified fresh against the 1.21.11 registry — flash is id 42 here, 41 in 1.21.7; `hub_particle` still has no Flash arm) |
| B8-G03 spruce | **REPRODUCES** | B8-G03 (`Blocks.<clinit>` parsed fresh — spruce is `TintedParticleLeavesBlock(0.01F)` in 1.21.11 too) |

All three recur unchanged, as they did in 26.2, 26.1, 26.1.1, 26.1.2
and 1.21.10.

### Batch 7-routed flag

`/spreadplayers` tag read (`world_cmds.rs:2100-2109`): **VERIFIED
REAL again, minted as B8-G10** (major). Vanilla 1.21.11's `isSafe`
checks `y < maxY && !liquid && !is(FIRE)`; the port requires the
absent `#entities_can_teleport_to` tag, so the command always fails
after 10,000 iterations.

### New items minted in this run (not re-checks)

None. B8-H04 is a re-check whose verdict reversed (see §5 table), not
a new item; no new behaviour defect was found beyond the re-checks.

## §6 Systemic findings

1. **The template is where the defects live.** All ten behaviour
   defects reproduce in 1.21.11 itself, verified fresh — the 1.21.10
   child inherited them unchanged from this template, not the other
   way round. The five 26.2/26.3-form reversions (B8-G05 cap, B8-G06
   2017, B8-G07 2018, B8-G08 2016 handler, B8-G09 breaking effects)
   are the template's own: the 1.21.11 port shipped the newer form,
   and the 1.21.10 delta carried it on. The 26.1 tree's own comments
   remain the repair reference for all five (the cap removal, the
   `breaking_block_sound` form, the absence of the 2016/2017/2018
   handlers and senders) — with the 1.21.10 correction carried over:
   the cap repair is NOT "remove the cap" (the 26.1 fix), it is
   "evict the oldest" (vanilla 1.21.11's `EvictingQueue` form, same
   as 1.21.10's).
2. **The directional-teleport events are one feature, three
   defects.** 2016/2017/2018 share the `block_teleportation` /
   `enderman_teleportation` draw path and the 26.3 design (a 128
   portal-particle trail along the old→new line). They differ in
   severity of wrongness: 2017 and 2018 ADD a trail on top of the
   correct event-46 burst (double particles); 2016 REPLACES nothing
   — vanilla shows no particles at all for a shulker teleport, so
   the trail is pure invention. A single fix (remove the three
   handlers + their senders) closes all three.
3. **The 1.21.11 tree's level-event surface is wider than vanilla's
   in exactly the 2015–2020 band.** Vanilla 1.21.11's
   `LevelEventHandler` handles 79 event ids in its main switch
   (82 distinct across both switches); 2015–2020 are all
   absent. The port dispatches all six. 2015 is a harmless
   port-internal channel (same visible result as vanilla's
   `addParticle` loop: 128 portal particles lerped along the line);
   2016/2017/2018 are live and wrong; 2019/2020 dispatch is dead
   (no sender) but the client-side digging call that replaced them
   is live and wrong (B8-G09).
4. **Vanilla 1.21.11 moved halfway to 26.x, and the oracle problem
   shrank with it.** In 1.21.10, 7 of 8 particle oracles failed (92
   errors total); in 1.21.11, 5 of 8 fail (33 errors total) — because
   vanilla 1.21.11 adopted the 26.x entity subpackages, the
   `world.attribute` package, jspecify, the 7-arg
   `TextureAtlasSprite` ctor, the `monster.zombie` package and the
   effect methods the 26.3-form oracle sources assume. What remains
   is the genuinely-26.x-only residue: the geyser API (10 errors in
   B8-O02), `BlockStateModelSet` (2 + 2), `ItemStackTemplate` (2),
   the sprite `Material` package (4 + 4), the 26.x `ServerLevel`
   10-arg call (B8-O06, B8-O07), the 26.x `LocalPlayer` 9-arg call
   (B8-O01), and `bindComponents` (B8-O01, B8-O06) — none of which
   exists in 1.21.11. The two newly-compiling oracles (emitter,
   entity) mean two drivers are oracle-clean for the first time in
   any 1.21.x sweep; they still need prepared data + a build to run.
5. **The port's particle tables are otherwise exact.** All 115
   types covered end-to-end except elder_guardian, all 18
   data-carrying wire ids correct, all 28 limiter flags correct
   except elder_guardian, all 30 entity-burst pairs (incl. the 3
   nautilus pairs, new as checks this run) and all 45 marker-zoo
   blocks covered, all 11 leaf blocks' chances verified — the gaps
   are narrow and enumerated, not broad drift. The 11 extra types in
   the Rust enum are dead but unreachable.

## §7 Cross-references (not re-minted)

- **G1 (pilot) — bubble columns never form.** The client-side
  animate arm for bubble columns exists and is correct
  (`Animate::BubbleColumn`, `animate_tick.rs:1660` —
  `BubbleColumnUp` particles + whirlpool/upwards ambient sounds),
  but it can never trigger in-game because no `bubble_column`
  block states ever form server-side (G1, 1.21.7 pilot; 1.21.11
  server-side status is Batch 5's scope, not verified here). No new
  defect: the particle-visible consequence of G1, not a particle
  defect.
- **G6 (pilot) — dripstone never transfers fluid to cauldrons.**
  Server-side only; 1.21.11 server-side status is Batch 5's scope.
  The *client-side* drip particles are unaffected except for
  B8-G04's gate: vanilla 1.21.11's
  `PointedDripstoneBlock.animateTick` rolls 0.12 and climbs
  same-block stalactite via `findRootBlock` (no tag involved —
  there is no `SpeleothemBlock` and no `speleothems` tag in
  1.21.11), and the port's `dripstone_drip`
  (`animate_tick.rs:1807`) matches it by name, with the same 0.12
  roll and 0.02 cauldron rule. The marker oracle's scope comment
  about binding "the speleothem tag the stalactite branch needs"
  (`VanillaMarkerParticlesOracle.java:136`) is stale for 1.21.11 —
  the oracle binds tags from the jar's datapack, which has no such
  tag, and no code path reads it — but the comment is not a defect.
- **B12-G04 pattern — 26.x-only types in the `ParticleType` table.**
  Same table-leak *pattern* as B12-G04, but the 11 extra types in
  the Rust enum (8 26.2-only: geyser family, sulfur bubbles,
  noxious gas pair, sulfur cube goo; 3 26.3-only: poplar leaves)
  are unreachable — ids are name-resolved from the server's
  registry, and a 1.21.11 server never sends those names — so they
  are not minted as a defect. Dead table entries, not a behaviour
  gap. The 4 geyser entries in `override_limiter()` are likewise
  dead.
- **The `SpeleothemBlock` assumption elsewhere in the tree.** B8-G04
  is the particle-visible instance; the same 26.2+-only class name
  gates other behaviour: `mesher.rs:910`
  (`has_class("SpeleothemBlock")` selects the 0.125 horizontal
  random offset for dripstone — dead in 1.21.11, so dripstone meshes
  with the 0.25 default; vanilla 1.21.11's `getMaxHorizontalOffset`
  is `SHAPE_BASE.min(X)`, i.e. the 0.125 branch) and the
  server-side `horizon-world` speleothem/falling code
  (`level/speleothem.rs`, `level/falling.rs:44/50`,
  `level/classes.rs:219` — Batch 5's scope). The projectile-hit
  path is fine: it matches by block name, not class
  (`creatures.rs:5084` in 26.1.2; not re-verified here).
- **3003 wax-on sound missing from the level-event path → Batch 10
  (sound).** Vanilla 1.21.11's 3003 case spawns WAX_ON particles
  (3–5) AND plays `HONEYCOMB_WAX_ON` via `playLocalSound`
  (offsets 3374/3390, verified fresh). The 1.21.11 tree kept the
  particles (`block_particle_event`, `level_particles.rs:35-38`,
  routed from `game.rs:4367`) but the event path plays no sound.
  The item-use, dispenser, remote and copper-golem wax sounds are
  all still present (`game.rs:8032` + callers :8343/:8362,
  `dispenser.rs:2235`, `remote.rs:8795`,
  `copper_golems.rs:318`), so the missing case is specifically a
  client receiving the 3003 event for someone else's waxing (LAN
  guest view). Sound defect, not a particle defect — flagged for
  Batch 10 with this evidence, not minted here.
- **2002/2007 splash-potion break sound missing → Batch 10
  (sound).** Vanilla 1.21.11's 2002/2007 case plays
  `SPLASH_POTION_BREAK` itself, after the particles (offset 1530,
  verified fresh). The 1.21.11 tree instead maps it to events
  1053/1054 (`game.rs:4489`, `:4508`) — which vanilla 1.21.11 never
  sends (26.3 split the sound into 1053/1054; 1.21.11's
  lookupswitch has no such cases). The potion-splash particles
  still fire (`potion_splash_particles`, `game.rs:4357/4362`).
  Sound defect — flagged for Batch 10, not minted here.
- **Digging hit-sound schedule (sound half of B8-G09) → Batch 10.**
  The particle half (extra crumb) is B8-G09; the sound half (hit
  sound at first hit instead of every 4th tick, then silence) is a
  sound defect with the same evidence — flagged for Batch 10.
- **Enderman teleport destination selection → Batch 3.** The
  1.21.11 `mobs/enderman.rs` grounds the teleport on its own
  destination checks; the tag-form question is Batch 3's scope, not
  verified here; only the particle-visible 2018 trail is minted
  here (B8-G07).
- **`CONSUMABLE_DOES_NOT_TELEPORT_TO` avoidance tag → Batch 3.**
  `teleport_consume.rs:36` documents the 26.3-form forbidden tag;
  the tag is absent from both 1.21.11 jars and the prepared data.
  The destination-selection consequence is Batch 3's scope, not
  verified here; only the particle-visible 2017 trail is minted
  here (B8-G06).
- **B8-G10 routing.** `/spreadplayers` is commands-area (Batch 9);
  it is minted here only because the coordinator routed Batch 7's
  flag to Batch 8. The merge may re-route it to Batch 9's ledger.
- **`Parched` (new 1.21.11 entity) has no particle bursts.** It
  extends `AbstractSkeleton`, has no `handleEntityEvent` override
  and no particle refs; the driver sends no parched cases and the
  port has no parched arm — correctly, no gap.

## §8 What passed (per area)

### Registry and type coverage (118 checks: 116 pass, 2 gap)

- 115 particle types: `registries.json` count == jar `ParticleTypes`
  `<clinit>` registration count (115), and the `<clinit>` order equals
  the `protocol_id` of every type (verified by parsing the bytecode;
  0 mismatches vs the datagen registry — `elder_guardian` present at
  id 17, no geyser/sulfur/noxious/poplar entries).
  `ParticleTypes.class` is the same size (15,634 bytes) in the client
  and server-game jars. The registry is identical to 1.21.10's:
  same 115 names, same protocol ids (diffed name-by-name and
  id-by-id).
- Engine `make()` (`particles.rs:2216`, outer match at :2277) has an
  outer match arm for all 125 `ParticleType` variants (verified fresh
  by script: 125/125 variants covered, no outer catch-all — the `_ =>`
  arms in the body are all nested matches) — every enum variant has a
  provider implementation. The only engine-coverage gap is the missing
  `elder_guardian` variant itself (B8-G01).
- Vanilla providers: `ParticleResources.registerProviders()` in the
  1.21.11 client jar registers a provider for all 115 types, including
  `ELDER_GUARDIAN` (`ElderGuardianParticle$Provider`, offsets
  296–300). (Field-name quirk, same as 26.1.2 and 1.21.10: the fields
  are `TRIAL_SPAWNER_DETECTED_PLAYER(_OMINOUS)` but register under
  the names `trial_spawner_detection(_ominous)`.)
- The 11 Rust-enum extras (8 26.2-only geyser/sulfur + 3 26.3-only
  poplar leaves) are absent from the 1.21.11 jar's `ParticleTypes` —
  confirmed unreachable from a 1.21.11 server.
- Gap: providers-driver kind set (126 kinds) misses `elder_guardian`
  (B8-G01) and `vibration` (B8-H01), and carries the 13 non-1.21.11
  kinds (B8-H02).
- `STONE = 1` in the providers driver is correct for 1.21.11: state
  id 1 is stone in the 1.21.11 datagen `blocks.json` (the "in 26.3"
  comment is stale; the value is right).

### Wire codec (20 checks: 19 pass, 1 gap)

- All 18 data-carrying particle wire ids match `Particle::read`'s
  hardcoded ids exactly (verified fresh against the 1.21.11 registry,
  0 mismatches): State 1/2/29/109/113 (block, block_marker,
  falling_dust, dust_pillar, block_crumble), Color 21/36/42
  (entity_effect, tinted_leaves, flash), dust 14,
  dust_color_transition 15, effect 16, instant_effect 46,
  dragon_breath 8, sculk_charge 38, shriek 103, trail 49, item 47,
  vibration 48. `ids::particle(id)` is the identity when no version
  adapter is active (`adapter/ids.rs`), so these are the native
  1.21.11 ids. Note the ids differ from 26.1.2's (dust_pillar 109
  vs 111, block_crumble 113 vs 115, shriek 103 vs 105) — the read
  code's comment names the 1.21.11 order, which 1.21.10 shares. The
  read code carries no geyser arms — correct for 1.21.11 (comment:
  "The geyser particles a later release adds do not exist in
  1.21.11"), though the `Geyser`/`GeyserBase` data variants still
  exist in the enum and `write` still writes them if present
  (unreachable from a 1.21.11 read; `hub_particle` can still build
  them from a `ParticleOptions::geyser`, but no 1.21.11 path
  creates one).
- `received_particle` resolves ids to names through the server-sent
  registry (not a hardcoded table) — correct for all 114 implemented
  types.
- Gap: B8-G02 (Flash write path).
- Note (not a defect): `hub_particle` deliberately returns `None` for
  item and trail particles (`lan_particles.rs`, `network_particle`
  doc: "a particle whose data isn't sent from here (items, trails)")
  — a LAN host's `/particle item|trail` draws locally but sends
  nothing to guests. Documented behaviour, same as 26.1.2 and 1.21.10;
  the command does not parse trail options either. Not minted.

### Override limiter (28 checks: 27 pass, 1 gap)

- Vanilla limiter-true set (28 types, from the `<clinit>` booleans,
  parsed fresh from the 1.21.11 jar — the limiter boolean read as the
  `iconst` AFTER the name `ldc`, per the parser gotcha) vs the Rust
  `override_limiter()` list: identical except `elder_guardian`
  (B8-G01). The Rust list's 4 extra entries are the geyser types —
  dead in 1.21.11, unreachable.

### Entity event bursts (30 checks: 30 pass)

- Every (entity, event) pair in the entity driver's `EVENTS` table
  (12 entities, 30 pairs, re-counted fresh — the 3 `nautilus` pairs
  HAVE a vanilla 1.21.11 side now, B8-H05 withdrawn) is covered by
  the port's `entity_event_burst` (`particles.rs:627`):
  villager 12/13/14/42, cow 18, wolf/cat 6/7/18, parrot 6/7,
  nautilus 6/7/18, horse/llama/camel 6/7/18, ocelot 40/41/18,
  dolphin 38, allay 18 — with the right particle kinds and burst
  shapes, verified against the 1.21.11 jar (`Villager.handleEntityEvent`
  12→heart, 13→angry, 14→happy, 42→splash; `TamableAnimal` 7→heart/6→smoke;
  `Animal` 18→heart ×7 (0.02 gaussian, y+0.5); `Ocelot`
  41→heart/40→smoke; `Dolphin` 38→happy_villager;
  `Allay` 18→3× heart; `AbstractHorse` 6/7→taming particles;
  `Nautilus` inherits 6/7/18 via `AbstractNautilus` →
  `TamableAnimal`/`Animal` — no override, no particle refs of its
  own). Counts/positions/RNG draws need the driver run; the mapping
  table is exact.
- The port's table also carries `zombie_nautilus` and `camel_husk`
  arms — LIVE in 1.21.11 (both entities exist with inherited bursts),
  but the driver does not send cases for them (B8-H04), so they are
  untested, not wrong.

### Marker zoo (45 checks: 45 pass)

- Every real 1.21.11 block name in the marker driver's zoo (36 names,
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
  block does not exist in 1.21.11 — B8-H03, a driver-scope item, not
  a port gap.)
- The unlit variants in the zoo (unlit redstone torches, furnaces,
  smokers) correctly map to `Animate::None`.

### Leaves (12 checks: 11 pass, 1 gap)

Per-block `leafParticleChance` and class — parsed fresh from the
1.21.11 `Blocks.<clinit>` (this batch's class-file parser; the
azalea colour read from the lambda's `ColorParticleOption.create`
argument in the 1.21.10 run as −9399763, and the port's fixed tint
`0x70922D` matches it):

| Block | Vanilla 1.21.11 | Port | Verdict |
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
- Note: vanilla 1.21.11 has no `CherryLeavesBlock`/`PaleOakLeavesBlock`
  classes — cherry/pale-oak are `UntintedParticleLeavesBlock` instances,
  and the port's `FallingParticlesLeavesBlock` comment names the 26.3
  class; the mechanism is the 1.21.11 one.

### Dripstone and tags (7 checks: 6 pass, 1 gap)

- `speleothems` does NOT exist in 1.21.11 (absent from the client
  jar's datapack; no `SpeleothemBlock` class) — a real difference
  from 26.2, where it held `pointed_dripstone` + `sulfur_spike`.
  The port's animateTick drip path does not need it (name-based
  table), so this is a pass for the animate path; the class-gated
  level-event path is the gap (B8-G04).
- `washed_away_by_fluids` and `blocks_lava_fire_spread` are both
  absent from the 1.21.11 datapack — confirms the pilot's tag
  inventory pattern for 1.21.11.
- `triggers_ambient_dried_ghast_block_sounds` exists (soul_sand,
  soul_soil), as do the two desert ambient-sound tags
  (`triggers_ambient_desert_sand_block_sounds`: sand, red sand;
  `triggers_ambient_desert_dry_vegetation_block_sounds`: #terracotta,
  sand, red sand) — the tags the port's `AnimateTables` binds are
  real in 1.21.11 (contents read fresh from the jar). (The poplar tag
  it also binds, `required_for_poplar_leaf_ambience`, has no 1.21.11
  referent — dead path, no poplar blocks exist.)
- Vanilla dripstone animateTick path matches the port (0.12 roll,
  same-block climb, 0.02 cauldron rule, mud-as-water, 5/16 − 0.0625
  drip height).
- Gap: B8-G04 (level-event 1504 gate).

### Oracle API existence (8 checks: 8 pass)

All vanilla methods the oracles call exist in the 1.21.11 jars with the
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
what the drivers verify, and 5 of the 8 oracles do not compile (§9).

### Other passes

- Engine `make()` exhaustiveness (see above — 125/125 arms, fresh).
- Game-event-10 apparition re-implementation present (§4, B8-G01).
- Bubble-column animate arm present (§7, G1).

### Level events 2015–2020 + digging (5 checks: 1 pass, 4 gap)

- **2015 dragon egg: pass.** Vanilla 1.21.11 has no 2015 level event,
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
  check; the live defect is the client-side `remote.rs:8171` call.)

### Particle group cap (1 check: 0 pass, 1 gap)

- Gap: B8-G05 (the `tick_tracking` pending drain enforces the
  26.2-form drop-new + probabilistic cap; vanilla 1.21.11 evicts
  the oldest via `EvictingQueue`).

### Routed check: /spreadplayers safety (1 check: 0 pass, 1 gap)

- Gap: B8-G10 (Batch 7-routed flag, verified fresh).

## §9 Oracle repairs and defects

Triage is from this batch's own javac measurement (method in §2) —
Batch 0's measured table had not landed when this batch ran, so
there is no Batch 0 table to triage from. Each oracle was compiled
alone on the "both" classpath (both named 1.21.11 jars + all 107 of
1.21.11's own game library jars, downloaded from Mojang per
version.json and sha1-verified, + jsr305) with `-sourcepath` on the
test java root; logs in `evidence/javac-*.log`. A file that fails
standalone on "both" fails on every classpath. The coordinator should
reconcile these counts against the Batch 0 table when it lands — in
particular, Batch 0's per-classpath attribution may split some of
these errors differently, but the own-error root causes below were
each verified against the 1.21.11 jars (package absence by jar
listing, constructor arity by `javap`). Note the javac-version
sensitivity (§14): JDK 25 reports cascading errors JDK 21 suppresses
(providers: 19 vs 7, same root causes); the counts below are the JDK
25 counts, comparable to the 1.21.10 run's.

### B8-O01 — `VanillaMarkerParticlesOracle` BROKEN, 2 compile errors (MINOR)

Measured: 2 errors (JDK 25; JDK 21 agrees). The 26.1.2/1.21.10 break
reproduces — :44, the oracle's `Player` subclass calls `super(null,
null, null, null, null, null, false, null, null)` — 9 arguments, the
26.3 form (`VanillaMarkerParticlesOracle.java:44`); 1.21.11's
`LocalPlayer` constructor takes 7 (`Minecraft, ClientLevel,
ClientPacketListener, StatsCounter, ClientRecipeBook, Input, boolean`
— javap on the 1.21.11 client jar; no `ChatAbilities`) — plus the
missing `bindComponents(DataComponentMap)` at :218. GONE vs 1.21.10:
the three `world.attribute` references (:104, :116, :233) and the
dead `@Override` (:116) — the package exists in 1.21.11.
**Blocks the marker_particles driver entirely** — before it can
reach the B8-G03 spruce cases. Repair is the 1.21.11-form LocalPlayer
construction plus a `bindComponents` replacement; do not repair here
(checks + triage only).

### B8-O02 — `VanillaParticleProvidersOracle` BROKEN, 19 compile errors (MINOR)

Measured: 19 errors (JDK 25; JDK 21 reports 7 — same root causes,
fewer cascades). The geyser break reproduces in full — 10 of the 19
are the geyser references (`GeyserParticleOptions` ×3,
`GeyserBaseParticleOptions` ×3, `ParticleTypes.GEYSER` /
`GEYSER_PLUME` / `GEYSER_BASE` / `GEYSER_POOF` ×4; :143-146,
:164-165; zero `geyser` entries in either 1.21.11 jar) — plus
`BlockStateModelSet` ×2 (:16, :113), `ItemStackTemplate` ×2 (:34,
:150), the `net.minecraft.client.resources.model.sprite` package
(:22), `Material` ×4 (:115×2 incl. a dead `@Override`, :116, :124).
GONE vs 1.21.10: the 4 jspecify `Nullable` errors (jspecify 1.0.0
ships in 1.21.11's game libraries — verified in version.json and on
the classpath), the `TextureAtlasSprite` constructor mismatch (the
oracle's 7-arg `super(...)` matches 1.21.11's 7-arg protected ctor),
and one further missing symbol. **Blocks the particle_providers
driver entirely.** The providers oracle's *mechanism* fits 1.21.11 —
it builds `new ParticleResources()`, reads the private `spriteSets`
field by reflection, and calls the public `getProviders()` — all
present in the 1.21.11 client jar (`ParticleResources` holds
`registerProviders()` in 1.21.11). Repair is bigger than the 26.1.2
deletion (geyser branches + cast helpers); the sprite/Material/
jspecify breaks need 1.21.11-form rewrites, and the driver trim
(B8-H02) goes with it; do not repair here.

### B8-O03 — `VanillaBlockParticlesOracle` BROKEN, 7 compile errors (MINOR)

Measured: 7 errors — `BlockStateModelSet` ×2 (:13, :44), the
`net.minecraft.client.resources.model.sprite` package (:17),
`Material` ×4 (:45×2, :47, plus a dead `@Override` at :47). All
26.x-form. GONE vs 1.21.10: the `TextureAtlasSprite` constructor
mismatch (1.21.11's ctor is the 7-arg protected form the oracle
calls). **Blocks the block_particles driver entirely.**

### B8-O04 — `VanillaEmitterParticlesOracle` FIXED, 0 compile errors

Measured: compiles clean (exit 0). The 1.21.10 break (5 errors: the
`net.minecraft.world.entity.animal.cow` package + the `Cow` symbol)
is gone — `Cow` is at `net.minecraft.world.entity.animal.cow.Cow`
in 1.21.11, the form the oracle imports. **The emitter_particles
driver is no longer oracle-blocked** (it still needs prepared data +
a build to run, §13).

### B8-O05 — `VanillaEntityParticlesOracle` FIXED, 0 compile errors

Measured: compiles clean (exit 0). The 1.21.10 break (18 errors: ten
bad 26.x package imports + 8 missing symbols) is gone — the packages
(`animal.cow/dolphin/equine/feline/nautilus/parrot`,
`npc.villager`) and the classes (incl. `Nautilus`) all exist in
1.21.11. **The entity_particles driver is no longer oracle-blocked**
(it still needs prepared data + a build to run, §13; and it still
omits zombie_nautilus/camel_husk, B8-H04).

### B8-O06 — `VanillaItemParticlesOracle` BROKEN, 4 compile errors (MINOR)

Measured: 4 errors, same as 1.21.10 — a `ServerLevel` constructor
mismatch (:28: the oracle passes 10 args, 1.21.11's ctor takes 11
with `RandomSequences`) and three missing symbols (:29, :31:
`ItemStack.item()`, `ItemStack.count()`, `bindComponents` — the
26.x item-template API). **Blocks the item_particles driver
entirely.**

### B8-O07 — `VanillaEffectSyncOracle` BROKEN, 1 compile error (MINOR)

Measured: 1 error — the `ServerLevel` constructor mismatch (:113,
same 10-vs-11-arg break as B8-O06). GONE vs 1.21.10: the
`net.minecraft.world.entity.monster.zombie` package break (`Zombie`
is at `net.minecraft.world.entity.monster.zombie.Zombie` in
1.21.11), the 21-error cascade from the unresolvable `Zombie`
superclass, and the missing effect methods (:148 in 1.21.10 — all
present in 1.21.11). **Blocks the effect_sync driver entirely**
(its other oracle, `VanillaEffectParticlesOracle`, compiles).

### Oracles assessed PASSING for 1.21.11 (this batch's measurement)

- `VanillaEffectParticlesOracle` — compiles clean (0 errors) on the
  "both" classpath (as in 1.21.10).
- `VanillaEmitterParticlesOracle` — compiles clean (0 errors);
  FIXED vs 1.21.10 (B8-O04).
- `VanillaEntityParticlesOracle` — compiles clean (0 errors);
  FIXED vs 1.21.10 (B8-O05).

For comparison, the 26.2 run had only the marker oracle failing
(B8-O01); 26.1, 26.1.1 and 26.1.2 add the providers oracle (B8-O02);
1.21.10 had 7 of 8 failing (92 errors); 1.21.11 has 5 of 8 failing
(33 errors) because vanilla 1.21.11 adopted the 26.x entity
subpackages, the `world.attribute` package, jspecify, the 7-arg
`TextureAtlasSprite` ctor, the `monster.zombie` package and the
effect methods — leaving only the genuinely-26.x-only residue
(geyser, `BlockStateModelSet`, `ItemStackTemplate`, sprite
`Material`, the 26.x `ServerLevel`/`LocalPlayer` call forms,
`bindComponents`).

Static evidence for 1.21.11 (supporting the measured passes/fails,
not a substitute for them):

- The 8 oracle APIs this batch's scope touches all exist (§8).
- The providers oracle's mechanism fits 1.21.11: it builds
  `new ParticleResources()`, reads the private `spriteSets` field by
  reflection, and calls the public `getProviders()` — all present in
  the 1.21.11 client jar (`ParticleResources` holds
  `registerProviders()` in 1.21.11; `ParticleEngine` has no such
  method). It resolves kinds by registry name, which is why the 13
  non-1.21.11 kinds would throw (B8-H02).
- The marker oracle's `bindBlockTags()` reads tags from the jar's
  datapack at runtime, so the absent `speleothems` tag is simply
  unbound — no code path reads it in 1.21.11.
- The oracle sources in the 1.21.11 tree differ from the 1.21.10
  tree's ONLY by the `ResourceLocation` → `Identifier` rename (4
  files: marker, providers, block, emitter); the other 4
  (entity, item, effect-sync, effect-particles) are byte-identical.
  The oracle verdict changes are all vanilla-side (1.21.11's jars
  and libraries), not source-side.

## §10 Driver inventory

The plan (§4) claims 9 particle drivers; the tree has 7 particle-named
drivers plus 1 particle-adjacent driver = **8**. The 9th is unaccounted
for: possibly the plan counted `tests/test_cushion_particles.py` (a
Python unit test for Horizon's cushion feature, not a parity driver) or
double-counted. No 9th parity driver exists under any name in
`versions/1.21.11/tools/`.

| Driver | Oracle (1.21.11 status, MEASURED by this batch) | Probe | Status |
|---|---|---|---|
| block_particles | BlockParticles: FAILS, 7 errors (B8-O03) | block_particles_fixture.rs | BLOCKED (B8-O03) — oracle does not compile |
| emitter_particles | EmitterParticles: PASSES, 0 errors (B8-O04 fixed) | emitter_particles_probe.rs | UNRUN — oracle clean; needs prepared data + build |
| entity_particles | EntityParticles: PASSES, 0 errors (B8-O05 fixed) | entity_particles_probe.rs | UNRUN — oracle clean; needs prepared data + build; driver also omits zombie_nautilus/camel_husk (B8-H04) |
| item_particles | ItemParticles: FAILS, 4 errors (B8-O06) | item_particles_fixture.rs (horizon-world) | BLOCKED (B8-O06) — oracle does not compile |
| marker_particles | MarkerParticles: FAILS, 2 errors (B8-O01) | marker_particles_probe.rs | BLOCKED (B8-O01) — oracle does not compile; would also fail on spruce cases (B8-G03) once repaired; red_poplar_leaves zoo cases un-runnable (B8-H03) |
| native_client_particles | none (Horizon-only Host API contract, no vanilla oracle) | game fixture | UNRUN; out of vanilla scope |
| particle_providers | ParticleProviders: FAILS, 19 errors (B8-O02) | particle_providers_probe.rs | BLOCKED (B8-O02) — oracle does not compile; would also FAIL on the 13 non-1.21.11 kinds — oracle throws `AssertionError("unknown particle …")` (B8-H02); does not test elder_guardian (B8-G01) or vibration (B8-H01) |
| effect_sync (adjacent) | EffectParticles: PASSES (0 errors); EffectSync: FAILS, 1 error (B8-O07) | effect_sync_fixture.rs | BLOCKED (B8-O07) — one of its two oracles does not compile |

`parity.py` was not run (blocked — static checks only, no cargo).
There is no prepared 1.21.11 data in the worktree (no
`.local/horizon-client/1.21.11/`), so the data precondition for the
driver runs in §13 is NOT met — the runs need a prepare first (Batch
12's scope), on top of the oracle repairs. 0 of the 8 drivers are
runnable as-is; 5 wait on oracle repairs (B8-O01–O03, O06, O07) plus
the B8-H02/H03/H04 trims/additions, 2 wait only on prepared data +
a build (emitter, entity), and the 8th has no vanilla oracle.

## §11 Unit tests

34 `#[test]`s in `crates/horizon-client-render/src/particles.rs`,
unrun (blocked — no cargo builds in this batch) — the same 34 as the
1.21.10 tree:

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
  `glow_family_renders_on_the_translucent_layer`.
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
  loop exactness — only the drivers can verify these, and none ran (5
  of 8 are oracle-blocked; 2 are oracle-clean but need prepared data +
  a build; the 8th has no vanilla oracle). The static checks above
  bound the *tables and mappings*, not the numerics.
- **Translucent-layer table.** `ParticleType::translucent_layer()` (which
  types vanilla renders in the translucent layer) was not verified — it
  needs per-class `getRenderType` checks across ~20 particle classes.
  (Same scope decision as the 26.1.2, 26.1.1, 26.1, 26.2, 1.21.7 and
  1.21.10 batches.) Note the 1.21.11 tree's own comment: the Glow family
  is TRANSLUCENT in 1.21.11, OPAQUE in 26.x — the table is version-shaped
  and a re-verification here would need the 1.21.11 classes.
- **Sprite sets / particle textures.** Whether the engine resolves every
  type's sprite set from the 1.21.11 assets needs prepared data + a run.
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
- **G1/G6 server-side status for 1.21.11.** Whether bubble columns form
  and dripstone transfers fluid server-side in the 1.21.11 port is Batch
  5's scope, not this batch's.
- **The 3003/2002 sound halves and the digging sound schedule** are
  Batch 10's scope (§7); the particle halves were verified here.
- **Enderman/shulker teleport destination selection** is Batch 3's
  scope; only the particle-visible trails were verified here.
- **No prepared 1.21.11 data exists in the worktree** (no
  `.local/horizon-client/1.21.11/`), so data checks used the datagen
  reports (`e6-deltas/versions/1.21.11/datagen/reports/`) instead of
  prepared data — the prepared-data cross-checks the 26.1.2 run did
  (prepared `registries.json` == datagen, `block-tags.json`,
  `block-behaviours.json`, `items.json`) could not be repeated here
  for the prepared side. The tag-absence claims rest on the jar
  datapack (206 block tags listed), and the `items.json` claim
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
- **The azalea colour** was not re-read from the 1.21.11 lambda in
  this run (the 1.21.10 run read −9399763 from the 1.21.10 jar; the
  1.21.11 parse resolved the azalea lambdas to
  `UntintedParticleLeavesBlock(0.01F)` with the same shape, and the
  port's fixed tint `0x70922D` matches the 1.21.10 reading). The
  leaves-table pass for azalea rests on the class + chance + the
  port's tint matching the 1.21.10 colour reading, not a fresh
  1.21.11 colour read.

## §13 What unblocks the rest

1. Prepared 1.21.11 data does NOT exist in the worktree — a prepare
   (Batch 12's scope) is a precondition for every driver run in this
   batch, on top of the oracle repairs below.
2. Repair the five broken oracles (follow-up, not this batch): B8-O01
   (LocalPlayer 7-arg construction + `bindComponents` replacement
   for `VanillaMarkerParticlesOracle`), B8-O02 (geyser deletion +
   sprite/Material rework for `VanillaParticleProvidersOracle`),
   B8-O03 (sprite/Material rework for
   `VanillaBlockParticlesOracle`), B8-O06 (`ServerLevel` ctor +
   item-template rework for `VanillaItemParticlesOracle`), B8-O07
   (`ServerLevel` ctor for `VanillaEffectSyncOracle` — a one-line
   fix now, down from 26 errors). `VanillaEffectParticlesOracle`,
   `VanillaEmitterParticlesOracle` and `VanillaEntityParticlesOracle`
   compile as-is (measured, §9). Reconcile the repair list against
   Batch 0's table when it lands.
3. Fix B8-G01 (add the `ElderGuardian` variant + `ElderGuardianParticle`
   provider), B8-G03 (spruce leaves arm) and B8-G04 (the 1504 gate —
   match by name or by `PointedDripstoneBlock`, not `SpeleothemBlock`)
   on fix branches, then run `particle_providers` and
   `marker_particles`; fix B8-G02 (Flash arm in `hub_particle`) with
   them or separately. B8-G03 is driver-blocking; B8-G02 and B8-G04 are
   not covered by any driver; B8-G01 blocks nothing in the 1.21.11 tree
   because the driver skips the type — the fix should add
   `elder_guardian` to the driver's kind set too, or the type stays
   untested.
4. Fix the five inherited 26.2/26.3-form reversions (B8-G05–G09) — the
   26.1 tree is the repair reference for four of them (the
   `breaking_block_sound` form, the absence of the 2016/2017/2018
   handlers and senders); for B8-G05 the repair is evict-oldest, not
   cap-removal (vanilla 1.21.11's `EvictingQueue` form, same as
   1.21.10's). Fix B8-G10 by restoring vanilla's `isSafe`
   (`y < maxY && !liquid && !is(FIRE)`) in `spread_safe`. None of
   B8-G05–G10 is covered by any driver; all need in-game verification
   (particle-heavy scene for the cap; chorus fruit, enderman and
   shulker teleports; digging; `/spreadplayers`).
5. Trim the non-1.21.11 entries from the drivers for 1.21.11 (B8-H02: 13
   kinds in the providers driver — 3 poplars + 2 sulfur + 2 noxious +
   4 geyser + `pause_mob_growth` + `reset_mob_growth`; B8-H03:
   `red_poplar_leaves` in the marker zoo), add the missing coverage
   (B8-H01: vibration; B8-H04: zombie_nautilus + camel_husk in the
   entity driver AND the entity oracle's `TYPES` map), then run the
   drivers (`block`, `emitter`, `entity`, `item`, `effect_sync`);
   the item driver needs a longer cargo-build window than its 900 s
   timeout under load (or a warm target dir) — 1.21.7 precedent.
   The emitter and entity drivers are oracle-clean already — they are
   the first particle drivers runnable for a 1.21.x version once a
   prepare exists.
6. Run the 34 `particles.rs` unit tests (`cargo test -p
   horizon-client-render --lib particles::`, dev profile, `-j 1`).
7. Route the Batch 10 sound flags (§7: 3003 wax-on, 2002/2007
   potion-break, digging sound schedule) and the B8-G10 routing
   question (commands-area; Batch 9's ledger?) to the coordinator.

## §14 Method notes

- The 1.21.11 **client-named** jar is Mojang-mapped —
  `net/minecraft/client/particle/ParticleEngine.class` is present by
  its mapped name — and its `ParticleTypes.class` is the same size
  (15,634 bytes) in both jars. The `server.jar` in the jars dir is a
  Mojang bundler jar (never javap'd); the named game jars are
  `e6-deltas/versions/1.21.11/jars/{client-named,server-game-named}.jar`.
- **No md5 carry from 26.x or 1.21.10.** Every vanilla claim was
  javap-verified fresh against the 1.21.11 jars. Cross-version
  comparisons were re-reads, not carries: the particle registry was
  diffed against 1.21.10's (identical names + ids), the entity
  registry diffed (+4), the library list diffed (107 vs 115).
- **The `ParticleTypes.<clinit>` parse** (evidence, inline in this
  batch's scratch) walks the `javap -c` dump anchored on `putstatic`
  lines, reading the name (`ldc`/`ldc_w` string) and the limiter boolean
  (`iconst_0`/`iconst_1`) from the lines after each name. Three gotchas,
  all carried from the 26.1.2 run: constant indices above 255 use
  `ldc_w`, not `ldc`; the regex for the wide form is `ldc(_w)?`, not
  `ldc_w?`; and the limiter boolean must be read as the `iconst`
  *after* the name `ldc` — a backwards walk that takes the first
  `iconst` before the name picks up the *previous* registration's
  boolean. `CODEC`/`STREAM_CODEC` putstatics at the end of `<clinit>`
  are not registrations and must be excluded (117 putstatics, 115
  registrations). A transient disk-quota error truncated the first
  dump mid-`<clinit>` (754 lines, ending at `wax_on`); the dump was
  re-run complete (841 lines, 117 putstatics) and the parse uses the
  complete dump.
- **The `Blocks.<clinit>` parse** (`evidence/blocks_parse.py`, the
  1.21.10 batch's parser, re-run unmodified against the 1.21.11
  server-game jar) parses the class file directly (constant pool +
  `BootstrapMethods` + bytecode with a full opcode-length table):
  each leaf name's `ldc` is associated with its registration lambda
  through the `BootstrapMethods` table, and each lambda's constructed
  class and float constant are read out. 1.21.x registers by name
  string (`ldc "spruce_leaves"`), unlike 26.x's `BlockItemIds` form.
  The parser resolved all 11 leaf blocks for 1.21.11 (spruce →
  `lambda$static$36` → `TintedParticleLeavesBlock(0.01F)`, the same
  lambda number as 1.21.10).
- **Oracle measurement.** Single-file `javac -nowarn` per oracle on
  the "both" classpath, ~20–40 s per file under load (8 files,
  ~3.5 min total). The classpath was both named jars + all 107 of
  1.21.11's own game library jars (downloaded from Mojang's library
  host per version.json, sha1-verified, `evidence/libs/`) +
  `tools/jsr305-3.0.2.jar` (the 1.21.11 tree has no jsr305 copy, so
  the 1.21.10 tree's identical artifact was used). **javac-version
  sensitivity:** JDK 25 reports cascading errors JDK 21 suppresses —
  the providers oracle measures 19 errors under JDK 25 vs 7 under
  JDK 21, with identical root causes (the GEYSER field refs, the
  `ItemStackTemplate` use at :150 and the extra `Material` errors
  appear only under JDK 25). The counts in this report are the JDK
  25 counts, comparable to the 1.21.10 run's (whose 25-error
  providers count is consistent with JDK 25's fuller reporting).
  The jspecify resolution is real, not a classpath gap: jspecify
  1.0.0 is in 1.21.11's version.json libraries and on the classpath.
- `javap` on the small classes ran 5–40 s each under load; batches of
  2–8 classes per exec kept the work moving. `javap -c` on the big
  classes (`LivingEntity`, `ClientPacketListener`) ran in background
  batches. `javap -c` on `Blocks` was not used (1.21.7 warning:
  exceeds 280 s under throttle) — the class-file parser replaced it.
- `LevelEventHandler` lives at `net.minecraft.client.renderer.LevelEventHandler`
  in 1.21.11 (same as 26.x and 1.21.10). The entity classes moved
  packages between 1.21.10 and 1.21.11: in 1.21.11 `Cow` is
  `...animal.cow.Cow`, `Dolphin` is `...animal.dolphin.Dolphin`,
  `Ocelot` is `...animal.feline.Ocelot`, `Allay` is
  `...animal.allay.Allay`, `AbstractHorse` is
  `...animal.equine.AbstractHorse`, `Villager` is
  `...npc.villager.Villager`, `Zombie` is
  `...monster.zombie.Zombie` — the 26.x subpackage forms, which is
  why the 26.3-form oracle imports now resolve.
- The oracle sources in the 1.21.11 tree differ from the 1.21.10
  tree's only by the `ResourceLocation` → `Identifier` rename (4
  files); the other 4 are byte-identical. All oracle-verdict changes
  vs 1.21.10 are vanilla-side.
- No production code (`crates/`) was touched. No commits were made in
  this batch. Only the two deliverable files were written (plus
  evidence/ scratch: javap dumps, the registry census, the
  `ParticleTypes` `<clinit>` parse, `blocks_parse.py`, the javac
  logs, the jar listings, and the downloaded 1.21.11 library jars).

## §15 Evidence index

Worktree `~/workspace/mcir-work/sweep-1.21.11`, version tree
`versions/1.21.11/` unless stated. Jars:
`~/workspace/mcir-work/e6-deltas/versions/1.21.11/jars/{client-named,server-game-named}.jar`.
Scratch: `~/workspace/mcir-work/sweep-1.21.11-batches/b08/evidence/`.

**Jar (fresh, 1.21.11):**
- `evidence/particle_types.txt` — `ParticleTypes` `javap -c`
  (`<clinit>`; 115 registrations in protocol-id order, 28
  limiter-true; `ELDER_GUARDIAN` id 17, limiter true).
- `evidence/particle_types.json` — the parsed registrations
  (115, with limiter flags).
- `evidence/level_event_handler.txt` — `LevelEventHandler` `javap -c`
  (79-case main lookupswitch + 3-case switch, 82 distinct: NO
  2015–2020, NO 1053/1054; 3003 case: WAX_ON at offset 3374 +
  HONEYCOMB_WAX_ON at 3390; 2002 case SPLASH_POTION_BREAK at offset
  1530; 1504 case: `PointedDripstoneBlock.spawnDripParticle` at
  offset 3519).
- `evidence/mpgm.txt` — `MultiPlayerGameMode` `javap -c`
  (`addBreakingBlockEffect`: 0 calls; hit sound every 4th tick,
  `destroyTicks % 4.0f == 0`, offsets 199–208; volume /8.0, pitch
  *0.5; `startDestroyBlock` plays/spawns nothing).
- `evidence/living.txt` — `LivingEntity.randomTeleport` broadcasts
  entity event 46 (bipush 46 at offset 177); `handleEntityEvent`
  case 46 → 128 PORTAL particles (offsets 201/208, PORTAL at 389,
  `addParticle` at 407); `handleEntityEvent` declared on
  `LivingEntity`; `spawnItemParticles` present.
- `evidence/shulker.txt` — `Shulker.teleportSomewhere`: no particles,
  no entity/level event (0 such refs in the whole class).
- `evidence/enderman.txt` — `EnderMan.teleport` → `randomTeleport(…,
  true)` (offset 110); the only `addParticle` is ambient aiStep
  (offset 90).
- `evidence/dragonegg.txt` — `DragonEggBlock.teleport`: 128 PORTAL
  lerped old→new (loop bound `sipush 128`, offset 135).
- `evidence/spread.txt` + `evidence/spread_pos.txt` —
  `SpreadPlayersCommand$Position.isSafe`:
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
- `evidence/campfire.txt` + `evidence/campfire_be.txt` —
  `CampfireBlock.animateTick` / `makeParticles`,
  `CampfireBlockEntity.particleTick`.
- `evidence/clientlevel.txt` — `ClientLevel.animateTick`.
- `evidence/serverlevel.txt` — `ServerLevel` ctor: 11 args with
  `RandomSequences`.
- `evidence/textureatlas.txt` — `TextureAtlasSprite` ctor: protected
  7-arg form.
- `evidence/holder_ref.txt` — `Holder$Reference`: no `bindComponents`.
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
- `evidence/entity_nautilus.txt` + `evidence/abstractnautilus.txt` —
  `Nautilus` extends `AbstractNautilus` extends `TamableAnimal`; no
  `handleEntityEvent` override, no particle refs (inherits 6/7/18).
- `evidence/entity_znautilus.txt` — `ZombieNautilus` extends
  `AbstractNautilus`; no override, no particle refs (inherits 6/7/18).
- `evidence/entity_camelhusk.txt` — `CamelHusk` extends `Camel`; no
  override, no particle refs (inherits 6/7/18).
- `evidence/blocks_parse.py` + the §8 leaves table — all 11 leaf
  registrations parsed from the 1.21.11 `Blocks.<clinit>` (spruce →
  `lambda$static$36` → `TintedParticleLeavesBlock(0.01F)`).
- `evidence/client-jar-list.txt` / `evidence/server-jar-list.txt` —
  jar listings: 0 entries for `SpeleothemBlock`, `Geyser`,
  `EntityTypes`, `BlockEntityTypes`, `SulfurCube`, `Poplar`,
  `MobAppearanceParticle`, `BlockStateModelSet`, `ItemStackTemplate`;
  0 `entities_can_teleport_to` / `consumable_does_not_teleport_to` /
  `speleothems` / `washed_away_by_fluids` /
  `blocks_lava_fire_spread` / `required_for_poplar_leaf_ambience`
  datapack entries; the three ambient-sound tags present (contents
  read fresh); the `world.attribute` package present (55 entries);
  the 26.x animal/monster/npc subpackages present
  (`animal/cow`, `animal/dolphin`, `animal/equine`, `animal/feline`,
  `animal/nautilus`, `animal/parrot`, `npc/villager`,
  `monster/zombie`); the `client.resources.model.sprite` package
  absent; `Nautilus`/`ZombieNautilus`/`CamelHusk`/`Parched` classes
  present.
- `evidence/javac-*.log` — the 8 single-file oracle compiles
  (JDK 25: 2/19/7/0/0/4/1/0 errors; `VanillaEmitterParticlesOracle`,
  `VanillaEntityParticlesOracle` and `VanillaEffectParticlesOracle`
  pass, 0 errors).
- `evidence/jarclasses/` — `ParticleTypes.class` extracted from
  both jars: md5 `480f81fe64c3ec67656f185bdfc91759` in both.
- `evidence/libs/` — 1.21.11's own 107 game library jars,
  downloaded from Mojang per version.json, sha1-verified
  (`download.log`: ok=107 fail=0); includes jspecify 1.0.0.
- `evidence/classpath.txt` — the "both" classpath used for the
  javac measurement (110 entries).

**Data (1.21.11's own):**
- Datagen `datagen/reports/registries.json`:
  `minecraft:particle_type` (115 entries, ids contiguous 0–114;
  elder_guardian 17, flash 42, vibration 48, portal 58, note 56; no
  geyser/sulfur/noxious/poplar — identical to 1.21.10's names + ids);
  `minecraft:entity_type` (157 entries; nautilus, zombie_nautilus,
  camel_husk, parched — the 4 new vs 1.21.10's 153);
  `minecraft:environment_attribute` (45 entries, new vs 1.21.10);
  `minecraft:game_rule` (59 entries, new vs 1.21.10).
- Datagen `datagen/reports/blocks.json` (1,166 blocks): state id 1 is
  `minecraft:stone`; `red_poplar_leaves` absent (the only zoo name
  absent).
- Datagen `datagen/reports/items.json` (1,505 items): chorus_fruit
  consumable is `{"on_consume_effects": [{"type":
  "minecraft:teleport_randomly"}]}`; 0 occurrences of
  `directional_particles`.
- No prepared `.local/horizon-client/1.21.11/` exists in the worktree
  (see §12).

**Port source (1.21.11 tree):**
- `crates/horizon-client-core/src/particle_type.rs` — 125 variants
  (macro list), no `ElderGuardian`; `override_limiter` :152-189 (31
  entries: 27 real + 4 geyser).
- `crates/horizon-client/src/game/lan_particles.rs` —
  `hub_particle` :411, Color match :440 (no Flash arm);
  `received_particle` :469, `from_name` :491; `Item | Trail => None`;
  `network_particle` :401.
- `crates/horizon-client/src/game/commands/world_cmds.rs` —
  `cmd_particle` :708, `particle.notFound` :718, catch-all
  `simple(ty)` :829, `network_particle` call :883; `spread_safe`
  :2091-2114, tag read :2100-2109, spread loop :1978-1986, failure
  :1991-1999.
- `crates/horizon-client/src/animate_tick.rs` — leaves :140-151
  (spruce :151), poplar arms :144-146, azalea :148-150, dripstone
  table :397-410, `dripstone_drip` :1807, bubble column :1660,
  tag bindings :315-322.
- `crates/horizon-client/src/game/level_particles.rs` —
  `dragon_egg_particles` :15, `shulker_teleport_particles` :20,
  `consume_teleport_particles` :24, `enderman_teleport_particles`
  :29, `block_particle_event` :35 (3003→WaxOn :38),
  `is_stalactite` :243-244, `dripstone_drip_particle` :256,
  `breaking_block_effects` :353-373, `potion_splash_particles` :378.
- `crates/horizon-client/src/game.rs` — 2019/2020 dispatch
  :4298-4306, 2015 :4328, 2016 :4333, 2017 :4338, 2018 :4343,
  2002/2007 :4357/:4362, 3002..=3005 :4367, 1504 :4377,
  1053|1054 :4489/:4508, `wax_on_sound` :8032 (callers :8343/:8362).
- `crates/horizon-client/src/game/remote.rs` — digging call :8171,
  event-46 handler :4774-4777, game event 10 apparition set :4990,
  wax sounds :8795.
- `crates/horizon-client/src/game/consume.rs` — `portal_particles`
  :208, `entity_teleport_particles` :228,
  `check_position_and_teleport` event 46 :188-194,
  `directional_particles` :422-431.
- `crates/horizon-client/src/game/creatures.rs` — apparition set
  :4186. `crates/horizon-client/src/game/mob_blocks.rs` —
  `APPARITION_TICKS = 30` :87, `elder_apparition_geometry` :96.
- `crates/horizon-client/src/mobs/enderman.rs` — `teleport_to` :80,
  event 46 :160-166, 2018 send :174-184.
  `crates/horizon-client/src/mobs/shulker.rs` — 2016 send :366-371.
- `crates/horizon-client-render/src/particles.rs` —
  `entity_event_burst` :627, `create` :2065
  (`pending.push` :2098), `make` :2216 (outer match :2277, 125/125
  arms), `block_teleportation` :1859,
  `enderman_teleportation` :1884, `tick_tracking` :3888, cap
  :3933-3945, `MAX_PARTICLES` :1358-1359, firework flash :3785, 34
  tests.
- `crates/horizon-protocol/src/entity_data.rs` — `Particle::read`
  :185 (State ids :214, Color ids :215, vibration :265),
  `Particle::write` :297 (data by presence). `adapter/ids.rs` —
  identity when no adapter is active.
- `crates/horizon-items/src/items.rs:157-160` — `directional_particles`
  default true. `crates/horizon-world/src/teleport_consume.rs` —
  field :6, 2017 fire :109-121. `crates/horizon-items/src/blocks.rs:121`
  — `has_class` exact match.
- `crates/horizon-client-render/src/mesher.rs:910` —
  `SpeleothemBlock` offset gate (cross-ref).
- `crates/horizon-world/src/level/interact.rs:594` — 2015 send.
- Drivers: `tools/test_particle_providers_parity.py` — SIMPLE :26-38
  (103 kinds), kinds :74-76 (120 in the list + 6 in `cases()`
  sections = 126 total distinct), geyser/noxious sections
  :94-105, STONE :23; `tools/test_marker_particles_parity.py` —
  zoo_states :76 (37 distinct names), red_poplar :118, spruce
  comment :114, neighbour_cases :168, campfire_cases :37,
  wither_rose_cases :156; `tools/test_entity_particles_parity.py` —
  EVENTS :17 (12 entities, 30 pairs).
- Oracles: `mods/horizon/src/test/java/dev/horizonlab/parity/` —
  `VanillaMarkerParticlesOracle.java:44` (9-arg super call),
  :104/:116/:233 (world.attribute — now resolving),
  :218 (bindComponents), :136 (stale speleothem comment);
  `VanillaParticleProvidersOracle.java:131` (unknown-particle
  throw), :143-146/:164-165 (geyser refs), :22 (sprite package),
  :46 (jspecify — now resolving), :107 (TextureAtlasSprite — now
  matching), :115/116/124 (Material), :155-157 (vibration);
  `VanillaBlockParticlesOracle.java:13/:17/:42/:44-47`;
  `VanillaEmitterParticlesOracle.java:23/:120/:126` (now resolving);
  `VanillaEntityParticlesOracle.java:17-26/:47-50` (now resolving);
  `VanillaItemParticlesOracle.java:28-31`;
  `VanillaEffectSyncOracle.java:100/:102-110/:113/:148` (only :113
  still failing).
- Batch 0 table: not landed at the time of this batch (b00/evidence
  held only cargo-check artifacts); oracle verdicts rest on this
  batch's own javac measurement (§9).

## §16 Verdict

**277 behaviour checks: 265 pass, 12 gap. 22 items: 10 behaviour
defects (9 minor, 1 major), 4 live needs-harness items + 1 closed
(WITHDRAWN), 5 oracle defects + 2 fixed oracles.**

All 10 of the 1.21.10 Batch 8 behaviour defects reproduce in 1.21.11
itself — the template is where they live, and the 1.21.10 child
inherited them unchanged — as do all 3 of the 1.21.7 items. The
reproductions were verified fresh against 1.21.11's own jar/data, not
carried: the particle registry is identical to 1.21.10's (115 types,
same names, same ids), the group cap is the same eviction-form defect
as 1.21.10, and the providers driver carries the same 13 non-1.21.11
kinds. But vanilla changed between 1.21.10 and 1.21.11 in the places
this batch touches, and three re-checks split on those changes: the
nautilus family arrived, so the entity driver's nautilus cases are
now legitimate behaviour checks (B8-H05 WITHDRAWN, +3 checks) while
the zombie_nautilus/camel_husk omission is a live gap again (B8-H04
REVERSED from the 1.21.10 withdrawal); and the 26.x entity
subpackages, the `world.attribute` package, jspecify and the 7-arg
`TextureAtlasSprite` ctor arrived, so two oracles that failed for
1.21.10 now compile clean (B8-O04 emitter, B8-O05 entity — FIXED)
and the five still-broken ones fail with far fewer errors (33 total,
vs 92 for 1.21.10). No new behaviour defects were found beyond the
re-checks: the port's particle tables are otherwise exact. Nothing is
UNVERIFIED; the Batch 7-routed flag is decided (real, minted as
B8-G10). The batch is complete and ready for the coordinator to land
— with two reconciliation notes: the oracle verdicts rest on this
batch's own javac measurement because Batch 0's table had not landed,
and there is no prepared 1.21.11 data in the worktree, so the driver
runs wait on a prepare as well as the oracle repairs.
