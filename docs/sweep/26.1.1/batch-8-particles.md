# Batch 8 — Particles (26.1.1 parity sweep)

**Version:** 26.1.1 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.1.1`
(workdir `~/workspace/mcir-work/sweep-26.1.1`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 8 — all 117 particle types,
emitters, providers; 8 drivers found in the tree (see §10).

**Status: DRAFT — not committed.** Checks + triage only, per the batch
brief: no fixes, no edits under `versions/26.1.1/`, no cargo builds, no
commits. Everything below is static analysis: `javap` on the named
26.1.1 jars (`~/workspace/mcir-work/26.1.x/versions/26.1.1/jars/{client,server-game}.jar`;
the server jar is a Mojang bundler jar and was never javap'd), direct
bytecode parsing of `ParticleTypes.<clinit>` (via the `javap -c` dump),
source comparison against `versions/26.1.1/crates/` and against the
26.1 tree (`versions/26.1/`), data checks against the 26.1.1 datagen
reports and the prepared data in
`versions/26.1.1/.local/horizon-client/26.1.1/`, and oracle triage from
the measured Batch 0 table (`docs/sweep/26.1.1/batch-0-oracles.json`,
391 oracles, 189 pass / 202 fail-all) — not from import resolution. No
driver runs, no cargo tests. The coordinator commits these deliverables.

**Headline:** all ten 26.1 Batch 8 items reproduce in 26.1.1 — the four
behaviour defects (B8-G01–G04), the four needs-harness items
(B8-H01–H04, with H02 corrected: 11 non-26.1.1 kinds, not 5) and the
two oracle defects (B8-O01–O02) — because the particle files are
byte-identical to the 26.1 tree's or differ only in comments, and all
28 particle-relevant vanilla classes are byte-identical between the
26.1 and 26.1.1 jars. But the tree diff found what the 26.1 run could
not: **six new defects**, five of them the landed batches' systemic
pattern (the 26.1.1 tree carries 26.2/26.3-form behaviour the 26.1 tree
deliberately removed) — the particle-group cap is enforced (B8-G05),
chorus fruit / endermen / shulkers show 26.3-form directional teleport
trails vanilla 26.1.1 never sends (B8-G06–G08), and digging shows a
crack particle vanilla never shows (B8-G09). The sixth (B8-G10,
`/spreadplayers` always fails) is the Batch 7-routed flag, verified
fresh: it is shared with the 26.1 tree (missed by the 26.1 run) and is
the one major in this batch.

## §0 Execution status

Complete. All 26.1 re-checks (10) and all 1.21.7 re-checks (3) decided;
no UNVERIFIED carry-overs. The Batch 7-routed flag
(`/spreadplayers` tag read) verified fresh and minted as B8-G10. No
stalls: every jar needed was present, every class needed was
byte-identical to its 26.1 counterpart or freshly javap'd, and no
driver build was attempted (out of scope for this batch).

## §1 Scope

`docs/PARITY-SWEEP.md` §4 Batch 8: particle registry + type coverage,
wire codec, override limiter, entity event bursts, marker zoo, leaves,
dripstone + tags, oracle API existence. Method: registry comparison
(26.1.1's `particle_type` registry vs the port's `ParticleType`),
codec comparison, source comparison — plus, new for this run, a
tree-diff against the 26.1 tree for every particle file, which is what
surfaced B8-G05–G09. No cargo builds, no driver runs. Driver-coverage
gaps are needs-harness items (B8-Hxx), not behaviour gaps.

Files diffed 26.1 → 26.1.1 (particle scope):
`particle_type.rs` (comments only), `lan_particles.rs` (comments
only), `animate_tick.rs` (identical), `entity_data.rs` (identical),
`world_cmds.rs` (identical), all 8 particle drivers (identical), all 8
particle oracles (identical), `level_particles.rs` (REAL DIFF — the
2016/2017/2018 handlers, the breaking-block rework, two sound
removals), `particles.rs` (REAL DIFF — the cap), `game.rs` (2015–2020
dispatch), `remote.rs` (digging call), `consume.rs` + `items.rs` +
`teleport_consume.rs` (2017 path), `mobs/enderman.rs` (2018 send +
tag-form teleport), `mobs/shulker.rs` (identical — the 2016 send is
shared; only the client handler is new).

## §2 Method

- **Jar identity first.** 28 particle-relevant classes were
  md5-compared between the vanilla 26.1 and 26.1.1 jars (both
  client and server-game): `ParticleTypes`, `ParticleResources`,
  `ParticleGroup`, `ElderGuardianParticle`, `Blocks`, `LocalPlayer`,
  `PointedDripstoneBlock`, `MultiPlayerGameMode`,
  `ClientPacketListener`, `LeavesBlock`,
  `TintedParticleLeavesBlock`, `Shulker`, `EnderMan`,
  `ParticleEngine`, `ParticleUtils`, `Allay`, `Dolphin`, `Villager`,
  `TamableAnimal`, `Animal`, `Ocelot`, `AbstractHorse`,
  `AbstractVillager`, `LivingEntity`, `Particle`, `CampfireBlock`,
  `CampfireBlockEntity`, `ClientLevel`. **All 28 are byte-identical.**
  Where the 26.1 report's jar evidence rests on one of these classes,
  it carries to 26.1.1 verbatim; the carry basis is stated in each
  item. Every NEW claim in this batch was javap-verified fresh
  against the 26.1.1 jars (see §15).
- **Registry census from 26.1.1's own data.** Prepared
  `registries.json` vs datagen `registries.json` (identical, 117
  types, ids contiguous 0–116); `ParticleTypes.<clinit>` parsed from
  the `javap -c` dump (117 registrations in protocol-id order, 28
  limiter-true). `ParticleTypes.class` md5
  `8292544543edd947bd3398873d591f30` in both 26.1.1 jars (same as
  26.1).
- **Tree diff.** Every particle file was diffed against the 26.1
  tree; each diff hunk was checked against the 26.1.1 jar to decide
  which tree matches vanilla.
- **Oracle triage from the measured Batch 0 table**, with both
  failing particle oracles re-verified against the 26.1.1 jar
  (LocalPlayer ctor arity by `javap`; geyser absence by jar listing).
- **26.2-record corrections applied** (26.1 summary §"26.2-record
  corrections"): none of the three corrections touches Batch 8's
  scope (B3 horses/furnace/BreedGoal, B9-G11 recipe suggestions, B6
  feature_tags/dripstone_cluster). No correction was needed here.

## §3 Checks table (tally)

**Behaviour checks: 279 — 267 pass, 12 gap** (across 10 defect items,
B8-G01–B8-G10: 9 minor, 1 major). Counting rule: a behaviour check is
one comparison where BOTH the vanilla side (26.1.1 jar/data) and the
Rust side were actually compared; oracle-status checks are NOT counted
in the tally (per the common brief). Non-behaviour checks tallied: 0.
As-tallied 279 = 279 behaviour + 0 non-behaviour.

| Section | Total | Pass | Gap | Gaps |
|---|---|---|---|---|
| Registry / type coverage | 120 | 118 | 2 | B8-G01 (per-type coverage, kind set) |
| Wire codec | 20 | 19 | 1 | B8-G02 |
| Override limiter | 28 | 27 | 1 | B8-G01 (flag) |
| Entity event bursts | 30 | 30 | 0 | — |
| Marker zoo | 45 | 45 | 0 | — |
| Leaves | 12 | 11 | 1 | B8-G03 |
| Dripstone + tags | 7 | 6 | 1 | B8-G04 |
| Oracle API existence | 8 | 8 | 0 | — |
| Other passes | 2 | 2 | 0 | — |
| Level events 2015–2020 + digging (new) | 5 | 1 | 4 | B8-G06, G07, G08, G09 |
| Particle group cap (new) | 1 | 0 | 1 | B8-G05 |
| Routed check: /spreadplayers safety (B7 flag) | 1 | 0 | 1 | B8-G10 |
| **Total** | **279** | **267** | **12** | |

120+20+28+30+45+12+7+8+2+5+1+1 = 279; 267 + 12 = 279.

- **Oracle defects: 2** (B8-O01 — marker oracle, LocalPlayer ctor,
  Batch 0 G2; B8-O02 — providers oracle, 10 geyser-symbol errors).
  The other six particle-family oracles pass on their real classpath
  (measured, §9).
- **Needs-harness items: 4** (B8-H01–H04, §4): driver-coverage gaps,
  not behaviour gaps.
- **Drivers: 0 run, 8 inventoried** (7 particle drivers + 1
  particle-adjacent effect-sync driver). 2 of the 8 are oracle-blocked
  (B8-O01 blocks marker_particles; B8-O02 blocks particle_providers —
  which would also fail on its 11 non-26.1.1 kinds once repaired,
  B8-H02); the other 6 have compiling oracles (measured) but are
  unrun, not passing.
- **Unit tests:** 31 `#[test]`s in `particles.rs` exist (23 vanilla +
  8 mod-particle, same names as 26.1); unrun, blocked.

## §4 Defects

### B8-G01 — `elder_guardian` has no port implementation at all (MINOR, REPRODUCES from 26.1, recurs from 1.21.7)

**Checks failed (3):** per-type end-to-end coverage for
`elder_guardian`; its `override_limiter` flag; the providers-driver
kind set (which omits it).

**Evidence (jar, fresh for 26.1.1).** Vanilla 26.1.1 registers
`ELDER_GUARDIAN` as a `SimpleParticleType` (`ParticleTypes.<clinit>`,
wire id 17, `overrideLimiter=true` — parsed from the `javap -c` dump
of the 26.1.1 client jar) and registers a provider for it in
`ParticleResources.registerProviders()` — a dedicated
`ElderGuardianParticle$Provider` (offsets 296–300).
`ClientPacketListener.handleGameEvent` spawns it at the player's
position on game event `GUARDIAN_ELDER_EFFECT` (offset 520;
`addParticle(ELDER_GUARDIAN, ...)` at offset 530, plus the curse
sound at 574). All three classes are byte-identical to 26.1's, so the
26.1 evidence carries exactly; the offsets above were re-read from
the 26.1.1 jar.

**The port.** `ParticleType` (`horizon-client-core/src/particle_type.rs`)
has 127 variants and no `ElderGuardian` (grep over
`particle_type.rs`, `lan_particles.rs`, `particles.rs`: zero hits).
Consequences, all verified in source:

- `received_particle` (`game/lan_particles.rs:466`, `from_name` at
  :488) resolves a server-sent particle by name and returns `None`
  for `elder_guardian` — a `LevelParticles` packet for it is silently
  dropped.
- `/particle minecraft:elder_guardian` fails with `particle.notFound`
  (`game/commands/world_cmds.rs:760`, `from_name` returns `None`);
  vanilla accepts it.
- `override_limiter()` (`particle_type.rs:154-189`) has no entry for
  it (28 vanilla limiter types vs 27 real-26.1.1 Rust entries; the
  only 26.1.1 miss is `elder_guardian` — the Rust list's other 4
  entries are the geyser types, dead in 26.1.1).
- The providers driver does **not** send `elder_guardian` cases in
  the 26.1.1 tree (its 120-kind set omits the type entirely), so
  unlike 1.21.7 — where the driver sent 12 cases and the probe
  printed `no provider` — the driver is not blocked by this defect;
  it silently skips the type.

**Mitigation (why minor).** The one in-game path that matters — the
elder guardian's curse apparition — is re-implemented outside the
particle engine, same as in 26.1: game event 10 sets
`elder_apparition` (`game/remote.rs:5001`,
`game/creatures.rs:4175`) and `elder_apparition_geometry`
(`game/mob_blocks.rs:79`) draws the ghostly guardian with the vanilla
30-tick lifetime (`APPARITION_TICKS = 30`, `mob_blocks.rs:70`). So
the apparition works; what is missing is the particle-type plumbing
(`/particle`, server packets). Visual fidelity of the
re-implementation against vanilla's `ElderGuardianParticle` is
unverified statically — needs the driver run or an in-game look.

### B8-G02 — `/particle minecraft:flash` sends a malformed packet to LAN guests (MINOR, REPRODUCES from 26.1, recurs from 1.21.7)

**Checks failed (1):** the `Flash` arm of `hub_particle`.

**Evidence.** `Particle::read` (`horizon-protocol/src/entity_data.rs:185`,
file byte-identical to 26.1's) decodes wire id 42 (`flash`, verified
against the 26.1.1 registry) as `Color(i32)` — a flash particle on the
wire always carries a colour. But `hub_particle`
(`game/lan_particles.rs:408`), which builds the packet a LAN host
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
(`world_cmds.rs:871`; vanilla's command requires the colour option,
the port does not) and calls `network_particle` (:925). Everywhere
else `Flash` is created client-side only (the firework explosion
flash, `particles.rs:3777`, via `self.create(...)`, never sent over
the network). The local player is unaffected (draws directly, never
reads the packet). So the trigger is exactly: a LAN host runs
`/particle minecraft:flash` with guests in range.

### B8-G03 — Spruce leaves never drop falling-leaf particles (MINOR, REPRODUCES from 26.1, recurs from 1.21.7, driver-blocking)

**Checks failed (1):** the spruce entry of the per-block leaves check.

**Evidence (jar).** Vanilla 26.1.1 registers `SPRUCE_LEAVES` as a
`TintedParticleLeavesBlock` with `leafParticleChance = 0.01` —
`Blocks.class` is byte-identical to 26.1's, so the 26.1 parse carries
exactly (`BlockItemIds.SPRUCE_LEAVES` + `lambda$static$27` constructs
`TintedParticleLeavesBlock(0.01F, …)`).
`LeavesBlock.animateTick` calls `makeFallingLeavesParticles`, which
rolls `nextFloat() < leafParticleChance` and spawns the falling leaf
unless the block below is face-full. So vanilla spruce drops tinted
leaves at 1% per animateTick, like oak.

**The port.** `classify` (`animate_tick.rs:151`, file byte-identical
to 26.1's) maps `"spruce_leaves"` to `Animate::Leaves(None, None,
false)` — no falling particle at all. Every other leaf block matches
vanilla (see §8, leaves table).

**Driver note.** The marker driver's own zoo comment says "the spruce
that never falls" (`test_marker_particles_parity.py:114`) — the
comment encodes the same wrong belief, but the driver compares against
the real vanilla oracle, so the driver is correct as a check and
**would fail on the spruce cases** (oracle emits tinted leaves, probe
emits none — the probe runs the port's real `animate_tick`,
`marker_particles_probe.rs:168`) once B8-O01 is repaired — the driver
is currently oracle-blocked, not merely unrun. The comment needs
correcting when the fix lands; the driver needs no change.

### B8-G04 — Level-event 1504 stalactite drip particle never fires: its gate checks for `SpeleothemBlock`, which does not exist in 26.1.1 (MINOR, REPRODUCES from 26.1)

**Checks failed (1):** the level-event 1504 gate in the dripstone
section.

**Evidence (jar + prepared data, fresh).** Vanilla 26.1.1 has no
`SpeleothemBlock` class at all (0 entries in either jar;
`PointedDripstoneBlock` extends `Block` directly) and no
`speleothems` tag in the client jar's datapack or the prepared
`block-tags.json`. The prepared data agrees:
`block-behaviours.json` gives `minecraft:pointed_dripstone` the class
chain `["PointedDripstoneBlock", "Block"]`. Vanilla 26.1.1's
`LevelEventHandler` routes 1504 to
`PointedDripstoneBlock.spawnDripParticle(level, pos, state)`
(offset 3495–3508, no class gate), and vanilla's
`PointedDripstoneBlock.maybeTransferFluid` sends level event 1504
when a stalactite drips fluid into a cauldron below (26.1 evidence,
class byte-identical).

**The port.** The client routes 1504 to `dripstone_drip_particle`
(`game.rs:4376-4377`), whose first act is the `is_stalactite` gate
(`game/level_particles.rs:243-244`): `has_class("SpeleothemBlock")`
(`horizon-items/src/blocks.rs:121`, an exact match over the prepared
class chain). In 26.1.1 that is false for every block, so the function
returns before spawning anything — the cauldron-fill drip particle
never appears. The ambient animateTick drip path is unaffected: it is
name-based (`AnimateTables.dripstone`, built from
`"pointed_dripstone"` by name, `animate_tick.rs:397-410`) and matches
vanilla (§8).

**Scope note.** Whether the port's *server* actually transfers fluid
to cauldrons in 26.1.1 (pilot G6's 26.1.1 status) is Batch 5's scope;
the client gate is dead regardless of server reachability, and the
defect is minted on the client side only. The same `SpeleothemBlock`
class assumption appears elsewhere in the tree (see §7) — those are
other batches' scope.

### B8-G05 — Particle engine enforces the 16,384 group cap vanilla 26.1.1 never enforces (MINOR, NEW — 26.1.1 regression vs the 26.1 tree)

**Checks failed (1):** the pending-particle drain in `tick_tracking`.

**Evidence (jar, fresh).** Vanilla 26.1.1's
`ParticleGroup.add` (`javap -c`, 26.1.1 client jar) is a plain queue
add — `particles.add(p)`, no size check, no probability roll. The
class declares `MAX_PARTICLES` (16384) but never reads it in `add`
(the cap enforcement is a 26.2 addition). `ParticleGroup.class` is
byte-identical to 26.1's.

**The port.** `tick_tracking` (`horizon-client-render/src/particles.rs:3880`,
the drain that moves every created particle from `pending` into the
live list — `create()` pushes to `pending`, :2087) enforces the cap
(:3925-3935): at 16,384 live particles every new particle is dropped
(`if size >= MAX_PARTICLES { continue; }`), and above 12,288 new
particles are dropped with probability `1 - ((16384-size)/4096)^2`,
drawing the level random for each rejection. The constant is declared
at :1349-1350 (`MAX_PARTICLES: usize = 16384`). The 26.1 tree
deliberately has NO cap here (its comment: "`ParticleGroup.add` in
26.1 is a plain queue add: the group cap (16384, with the last
quarter filling by decreasing probability) is 26.2-only — 26.1
declares MAX_PARTICLES but never enforces it"). The 26.1.1 tree
re-introduced the 26.2 form.

**Reachability (why minor).** The cap only bites in particle-heavy
scenes (above 12,288 live particles: big explosions, dense farms,
heavy `/particle` use). Below that threshold the port matches
vanilla exactly. When it bites, particles vanish that vanilla would
show, and the rejection draws consume level-random draws vanilla
never makes (RNG-order divergence for everything downstream in the
tick).

### B8-G06 — Chorus fruit shows an extra 2017 directional teleport trail vanilla never sends (MINOR, NEW — 26.1.1 regression vs the 26.1 tree)

**Checks failed (1):** the level-event 2017 check.

**Evidence (jar, fresh).** Vanilla 26.1.1's
`TeleportRandomlyConsumeEffect` is a record with ONE field
(`diameter: float`; constructors `()` and `(float)`) — there is no
`directional_particles` field. Its `apply` calls
`LivingEntity.randomTeleport(x, y, z, true)`, which broadcasts
entity event 46; `LivingEntity.handleEntityEvent` case 46 spawns 128
portal particles at the entity (offsets 201/392/410). Vanilla
26.1.1's `LevelEventHandler` has NO 2017 case (79-case lookupswitch,
2000–2013 then 3000+). No level event is involved anywhere in the
path.

**The port.** The 26.1.1 tree added the 26.3 form:
`TeleportConsume` gained a `directional_particles: bool` field
(`horizon-world/src/teleport_consume.rs:6`); `items.rs:124` reads it
from data with `unwrap_or(true)` — and the prepared 26.1.1 data
never carries the field (`chorus_fruit`'s consumable is just
`{"type": "minecraft:teleport_randomly"}`, 0 occurrences of
`directional_particles` in `items.json`), so it defaults to true;
`teleport_consume.rs:109-121` then fires the event on every
successful teleport, and the client impl (`game/consume.rs:422-431`)
calls `level_event_at(2017, ...)`, which routes (`game.rs:4338-4341`)
to `consume_teleport_particles` → `block_teleportation(pos, data,
[127,127,127])` (`particles.rs:1848`) — 128 portal particles
interpolated along the old→new line. The port ALSO fires the
vanilla-correct entity event 46 (`consume.rs:188-194`,
`remote.rs:4782-4784` → `entity_teleport_particles`, 128 portal
particles at the entity). So a chorus-fruit teleport in the 26.1.1
port shows BOTH the correct 128-particle burst AND a second,
128-particle directional trail vanilla never shows. The 26.1 tree
has no `directional_particles` anywhere and fires only event 46.
(Batch 7 independently flagged the 2017 trail as masked by its G09 —
the trail fires only after a successful teleport; the masking claim
is Batch 7's, not verified here. The trail mechanism itself is
verified fresh here.)

### B8-G07 — Enderman teleport shows an extra 2018 directional portal trail vanilla never sends (MINOR, NEW — 26.1.1 regression vs the 26.1 tree)

**Checks failed (1):** the level-event 2018 check.

**Evidence (jar, fresh).** Vanilla 26.1.1's `EnderMan.teleport`
calls `randomTeleport(x, y, z, true)` (offset 110) → entity event 46
→ 128 portal particles at the entity. The only `addParticle` call in
`EnderMan` is the ambient aiStep emission, not the teleport.
`LevelEventHandler` has NO 2018 case. The 26.1 tree's
`mobs/enderman.rs` documents this explicitly: "26.3 … added … the
level event 2018 portal trail; none of those exist in 26.1" and
"(26.3 also fires the level event 2018 portal trail; 26.1 has no
such event here.)"

**The port.** The 26.1.1 tree's `teleport_to`
(`mobs/enderman.rs:161-190`) pushes BOTH the vanilla-correct entity
event 46 AND `MobEvent::LevelEvent { id: 2018, ... }` (when not
silent), which routes (`game.rs:4343-4346`) to
`enderman_teleport_particles` →
`enderman_teleportation(pos, data, particle_level)`
(`particles.rs:1873`) — 128 portal particles along the old→new
line. Every enderman teleport in the 26.1.1 port shows the correct
burst plus a second, wrong trail. (The destination-selection half of
the 26.1.1 enderman rework — the `#entities_can_teleport_to` /
`#enderman_does_not_teleport_to` tag form — is Batch 3's scope,
recorded as B3-G14; only the particle-visible trail is minted here.)

### B8-G08 — Shulker teleport shows a 2016 directional trail; vanilla shows NO particles at all (MINOR, NEW — 26.1.1 regression vs the 26.1 tree)

**Checks failed (1):** the level-event 2016 check.

**Evidence (jar, fresh).** Vanilla 26.1.1's
`Shulker.teleportSomewhere` does: `unRide`, `setAttachFace`,
`playSound(SHULKER_TELEPORT)`, `setPos`, `gameEvent(TELEPORT)`, set
peek data, `setTarget`. No `randomTeleport`, no `addParticle`, no
`broadcastEntityEvent`, no level event anywhere in the class. A
shulker teleport in vanilla 26.1.1 shows zero particles.
`LevelEventHandler` has NO 2016 case.

**The port.** `mobs/shulker.rs:366-371` (byte-identical to 26.1's)
pushes `MobEvent::LevelEvent { id: 2016, ... }` on every teleport.
In the 26.1 tree this event is dropped — `game.rs` has no 2016 arm —
so the 26.1 port accidentally matches vanilla (no particles). The
26.1.1 tree added the handler (`game.rs:4333-4336` →
`shulker_teleport_particles` → `block_teleportation(pos, data,
[8,8,8])`, `particles.rs:1848`), so every shulker teleport now shows
128 portal particles along the old→new line that vanilla never
shows. Note the asymmetry with G06/G07: here there is no correct
event-46 burst underneath — the trail is the ONLY particle output,
and it is entirely wrong.

### B8-G09 — Digging shows a crack particle (and a mistimed hit sound) vanilla never shows (MINOR, NEW — 26.1.1 regression vs the 26.1 tree)

**Checks failed (1):** the digging crack-particle check (level
events 2019/2020 area).

**Evidence (jar, fresh).** Vanilla 26.1.1's `MultiPlayerGameMode`
never calls `addBreakingBlockEffect` anywhere in the class (0
occurrences in the full `javap -c` dump), and `LevelEventHandler`
has NO 2019/2020 cases. `startDestroyBlock` plays nothing and
spawns nothing. `continueDestroyBlock` plays ONLY the hit sound, on
destroy ticks 0, 4, 8, … (`destroyTicks % 4.0 == 0`, volume
`(volume+1)/8`, pitch `* 0.5`, offsets 197–262) — no particle at
any point while digging. (`ClientLevel.addBreakingBlockEffect`
exists as a method but is never invoked on this path.)

**The port.** The 26.1.1 tree replaced the 26.1 tree's sound-only
`breaking_block_sound` with `breaking_block_effects`
(`game/level_particles.rs:351-373`: hit sound when `play_sound`,
plus `crack_particle(pos, face, state)` — one crumb on the mined
face), and calls it from `startDestroyBlock`
(`game/remote.rs:8180`, `breaking_block_effects(hit.pos, hit.face,
true)`) with the comment "startDestroyBlock plays the first hit and
crack itself; the server's level events 2019 and 2020 do the rest".
But the port's server never sends 2019/2020 (no sends in
`horizon-world`), so the network dispatch (`game.rs:4298-4306`) is
dead and the continue path plays NOTHING — no every-fourth-tick
sound at all. Net effect vs vanilla: an extra crack particle + a
hit sound at the first hit (vanilla: silence), then silence for the
rest of the dig (vanilla: a hit sound every 4th tick). The 26.1
tree is correct here: start plays nothing, continue plays the
sound-only `breaking_block_sound` on ticks 1, 5, 9, …
(`remote.rs:8233`). The particle half (extra crumb) is minted here;
the sound-schedule half is flagged for Batch 10 (§7).

### B8-G10 — `/spreadplayers` always fails: `spread_safe` requires a tag 26.1.1 does not have (MAJOR, NEW — shared with the 26.1 tree, missed by the 26.1 run; Batch 7-routed flag, verified fresh)

**Checks failed (1):** the `/spreadplayers` safety check (routed
from Batch 7 §8.6 / slice `out_of_scope_failures`).

**Evidence (jar + data, fresh).** Vanilla 26.1.1's
`SpreadPlayersCommand.Position.isSafe` (`javap -c`, 26.1.1
server-game jar) is `y < maxY && !state.liquid() &&
!state.is(BlockTags.FIRE)` — three conditions, no teleport tag.
The `#entities_can_teleport_to` tag does not exist in 26.1.1: 0
entries in either jar's datapack, absent from the prepared
`block-tags.json` (no `teleport`-containing block tag at all).

**The port.** `spread_safe` (`game/commands/world_cmds.rs:2132-2154`,
file byte-identical to 26.1's) returns `y < max_y && !liquid &&
teleportable`, where `teleportable` (:2141-2146) requires membership
in `#entities_can_teleport_to` — always false in 26.1.1. So
`spread_safe` is ALWAYS false. In the spread loop (:2018-2026),
every collision-free iteration re-randomizes every point (none is
ever "safe"), `collisions` stays true, and the loop runs until
`iteration >= 10_000`, when the command fails with
`commands.spreadplayers.failed.entities` (or `.teams`, :2030-2043).
**`/spreadplayers` cannot succeed in the 26.1.1 port on any
terrain.** Shared with the 26.1 tree (same code); the 26.1 run's
Batch 9 never covered `/spreadplayers` safety (no mention in the
26.1 B9 report/slice), and the 26.1.1 Batch 9 slice does not cover
it either — this is the first record of the defect. Routed to Batch
8 by the coordinator via Batch 7's flag; it sits in the commands
area, so the merge may re-route it to Batch 9's ledger.

### Needs-harness items (driver coverage, not behaviour gaps)

- **B8-H01 — providers driver omits `vibration` (REPRODUCES).** The
  port implements vibration end to end (`Particle::read` id 48,
  `hub_particle` arm, `make()` arm at `particles.rs:3146`,
  `ParticleOptions::vibration`), and the oracle's `options()`
  handles it (`VanillaParticleProvidersOracle.java`, the `VIBRATION`
  branch), but the driver's 120-kind set never sends a vibration
  case — so the vibration provider's numerics have no driver
  coverage. Add vibration cases (with a target + duration) to
  `test_particle_providers_parity.py`.
- **B8-H02 — providers driver includes 11 kinds 26.1.1 lacks
  (REPRODUCES, corrected count).** The driver's `SIMPLE` list
  carries `red/orange/yellow_poplar_leaves` (26.3-only),
  `sulfur_bubbles` and `sulfur_cube_goo` (26.2-only) — the 5 the
  26.1 report named — AND the driver's `cases()` generator also
  yields `noxious_gas`, `noxious_gas_cloud`, `geyser`,
  `geyser_plume`, `geyser_base`, `geyser_poof`
  (`test_particle_providers_parity.py:94-105`, 6 more kinds, 72+
  cases), which the 26.1 report wrongly said were never sent ("The
  geyser kinds are NOT in the driver's kind set … so they need no
  trim" — incorrect; they are yielded by the same `cases()`
  generator `make_cases` consumes). None of the 11 is in the 26.1.1
  registry (117 types, no geyser/sulfur/noxious/poplar). The oracle
  resolves kinds by registry name and throws
  `AssertionError("unknown particle " + id)` for an unregistered
  type (`VanillaParticleProvidersOracle.java:131`), so the driver
  cannot run against the 26.1.1 oracle as-is: it would die on the
  first such case. Trim all 11 kinds for 26.1.1. (Correction to the
  26.1 record: the 26.1 run's H02 undercounts by 6 kinds; the driver
  file is byte-identical between the trees, so the 11-kind count
  applies to 26.1 as well.)
- **B8-H03 — marker driver zoo includes `red_poplar_leaves`
  (26.3-only) (REPRODUCES).** The zoo (`test_marker_particles_parity.py`,
  `zoo_states()`, :118) sends
  `red_poplar_leaves[distance=3,...]` cases, but no such block exists
  in 26.1.1 — the oracle cannot build that world. (Unlike 26.2, the
  26.1.1 port DOES have poplar `classify` arms
  (`animate_tick.rs:144-146`), but they are dead: no poplar block
  exists to classify.) Trim the zoo case for 26.1.1.
- **B8-H04 — entity driver omits live 26.1.1 entities
  `zombie_nautilus` and `camel_husk` (REPRODUCES).** Both are real
  26.1.1 entity types (protocol ids 152 and 20 — the entity
  registry is identical to 26.1's, verified via the inventory) and
  both inherit the 6/7/18 bursts (`ZombieNautilus extends
  AbstractNautilus extends TamableAnimal`; `CamelHusk extends Camel
  extends AbstractHorse` — the classes are byte-identical to 26.1's),
  and the port's `entity_event_burst` covers them via its
  tamable/horse sets — but the driver's `EVENTS` table
  (`test_entity_particles_parity.py:17`, 12 entities, 30 pairs) has
  no entries for them (6 untested pairs). Add them to
  `test_entity_particles_parity.py`.

### Oracle defects

- **B8-O01 — `VanillaMarkerParticlesOracle` does not compile for
  26.1.1 (REPRODUCES).** Batch 0 measured table
  (`docs/sweep/26.1.1/batch-0-oracles.json`): FAILS on all three
  classpaths, 1 error, class constructor-mismatch — "44: constructor
  LocalPlayer in class LocalPlayer cannot be applied to given
  types". This is Batch 0 group G2. Verified fresh against the
  26.1.1 jar: the oracle's `Player` subclass calls `super(...)`
  with 9 arguments, the 26.3 form
  (`VanillaMarkerParticlesOracle.java:44`); 26.1.1's `LocalPlayer`
  constructor takes 8 (`Minecraft, ClientLevel,
  ClientPacketListener, StatsCounter, ClientRecipeBook, Input,
  boolean, ChatAbilities` — `javap` on the 26.1.1 client jar;
  `LocalPlayer.class` byte-identical to 26.1's). Blocks the
  marker_particles driver entirely (before it can reach the B8-G03
  spruce cases).
- **B8-O02 — `VanillaParticleProvidersOracle` does not compile for
  26.1.1 (REPRODUCES).** Batch 0 measured table: FAILS on all three
  classpaths, 10 errors, class missing-symbol, first error "164:
  cannot find symbol". All 10 errors are references to the geyser
  API, which 26.1.1 predates: `GeyserParticleOptions` (3 refs: :144
  constructor, :164 type + cast), `GeyserBaseParticleOptions` (3
  refs: :146 constructor, :165 type + cast),
  `ParticleTypes.GEYSER` / `GEYSER_PLUME` (:143),
  `ParticleTypes.GEYSER_BASE` / `GEYSER_POOF` (:145). Verified
  fresh: zero `geyser` entries in either 26.1.1 jar, and no
  `GEYSER*` fields in `ParticleTypes` (the `<clinit>` parse).
  Blocks the particle_providers driver entirely. Repair is a
  deletion (the geyser branches + the two cast helpers — the driver
  never sends geyser kinds in its main 120-kind set, though it DOES
  send them in its geyser section, B8-H02 — so the trim and the
  repair go together); do not repair here.

## §5 Re-check verdicts

### 26.1 items (PRIMARY — all 10 decided)

| 26.1 item | Verdict | 26.1.1 item |
|---|---|---|
| B8-G01 elder_guardian | **REPRODUCES** | B8-G01 (fresh jar evidence: `<clinit>` id 17/limiter, provider, game-event offsets re-read from the 26.1.1 jar) |
| B8-G02 flash | **REPRODUCES** | B8-G02 (`entity_data.rs` + `world_cmds.rs` identical; `lan_particles.rs` comment-only diff) |
| B8-G03 spruce | **REPRODUCES** | B8-G03 (`animate_tick.rs` identical; `Blocks.class` byte-identical) |
| B8-G04 1504 gate | **REPRODUCES** | B8-G04 (gate unchanged; `PointedDripstoneBlock` chain + `LevelEventHandler` 1504 case verified fresh) |
| B8-H01 vibration omitted | **REPRODUCES** | B8-H01 (driver identical; 120-kind set re-counted) |
| B8-H02 5 non-26.1 kinds | **REPRODUCES with correction** | B8-H02 — 11 kinds, not 5 (the 26.1 report missed the driver's geyser/noxious sections, :94-105) |
| B8-H03 red_poplar zoo | **REPRODUCES** | B8-H03 (zoo re-counted: 37 distinct names incl. red_poplar) |
| B8-H04 zombie_nautilus/camel_husk | **REPRODUCES** | B8-H04 (EVENTS re-counted: 12 entities, 30 pairs, neither present) |
| B8-O01 marker oracle | **REPRODUCES** | B8-O01 (Batch 0 table + fresh LocalPlayer javap) |
| B8-O02 providers oracle | **REPRODUCES** | B8-O02 (Batch 0 table + fresh geyser-absence check) |

### 1.21.7 items (all 3 decided)

| 1.21.7 item | Verdict | 26.1.1 item |
|---|---|---|
| B8-G01 elder_guardian | **REPRODUCES** | B8-G01 |
| B8-G02 flash | **REPRODUCES** | B8-G02 |
| B8-G03 spruce leaves | **REPRODUCES** | B8-G03 |

All three recur unchanged, as they did in 26.2 and 26.1.

### Batch 7-routed flag

`/spreadplayers` tag read (`world_cmds.rs:2141-2146`): **VERIFIED
REAL, minted as B8-G10** (major). Vanilla 26.1.1's `isSafe` checks
`y < maxY && !liquid && !is(FIRE)`; the port requires the absent
`#entities_can_teleport_to` tag, so the command always fails after
10,000 iterations. (The coordinator's follow-up called this the
"particle command path"; the code is the `/spreadplayers` path in
`world_cmds.rs` — no particle-command tag read exists. The finding
itself is as Batch 7 recorded it.)

## §6 Systemic findings

1. **The 26.1.1 tree reverted five 26.1 particle fixes to their
   26.2/26.3 forms.** B8-G05 (cap), B8-G06 (2017), B8-G07 (2018),
   B8-G08 (2016 handler), B8-G09 (breaking effects) are all the
   same story: the 26.1 tree deliberately removed or reworked a
   26.2/26.3-form behaviour with a comment saying why, and the
   26.1.1 tree ships the newer form instead. The 26.1 tree's own
   comments are the repair reference for all five (the cap comment
   at 26.1 `particles.rs`, the enderman 2018 comments, the
   `breaking_block_sound` form, the 3003/2002 sound blocks). This
   matches the landed batches' systemic finding (B1: 4, B2: 1, B3:
   18+4, B5: 15, B6: 3 regressions of the same shape).
2. **The directional-teleport events are one feature, three
   defects.** 2016/2017/2018 share the `block_teleportation` /
   `enderman_teleportation` draw path and the 26.3 design (a 128
   portal-particle trail along the old→new line). They differ in
   severity of wrongness: 2017 and 2018 ADD a trail on top of the
   correct event-46 burst (double particles); 2016 REPLACES nothing
   — vanilla shows no particles at all for a shulker teleport, so
   the trail is pure invention. A single fix (remove the three
   handlers + their senders) closes all three.
3. **The 26.1.1 tree's level-event surface is wider than vanilla's
   in exactly the 2015–2020 band.** Vanilla 26.1.1's
   `LevelEventHandler` handles 79 event ids; 2015–2020 are all
   absent. The port dispatches all six. 2015 is a harmless
   port-internal channel (same visible result as vanilla's
   `addParticle` loop: 128 portal particles lerped along the line);
   2016/2017/2018 are live and wrong; 2019/2020 dispatch is dead
   (no sender) but the client-side digging call that replaced them
   is live and wrong (B8-G09).
4. **Two 26.1-record corrections from this batch.** (a) B8-H02
   undercounts: the providers driver sends 11 non-26.1 kinds, not
   5 — the geyser/noxious sections (`cases()`, driver :94-105) were
   missed. (b) The oracle-API check "Villager.handleEntityEvent"
   is misattributed: the method is declared on `LivingEntity`
   (inherited by Villager); the check still passes.
5. **The port's particle tables are otherwise exact.** All 117
   types covered end-to-end except elder_guardian, all 18
   data-carrying wire ids correct, all 28 limiter flags correct
   except elder_guardian, all 30 entity-burst pairs and all 45
   marker-zoo blocks covered, all 11 leaf blocks' chances verified
   — the gaps are narrow and enumerated, not broad drift. The 11
   extra types in the Rust enum are dead but unreachable.

## §7 Cross-references (not re-minted)

- **G1 (pilot) — bubble columns never form.** The client-side
  animate arm for bubble columns exists and is correct
  (`Animate::BubbleColumn`, `animate_tick.rs:1660` —
  `BubbleColumnUp` particles + whirlpool/upwards ambient sounds),
  but it can never trigger in-game because no `bubble_column`
  block states ever form server-side (G1, 1.21.7 pilot; 26.1.1
  server-side status is Batch 5's scope, not verified here). No new
  defect: the particle-visible consequence of G1, not a particle
  defect.
- **G6 (pilot) — dripstone never transfers fluid to cauldrons.**
  Server-side only; 26.1.1 server-side status is Batch 5's scope.
  The *client-side* drip particles are unaffected except for
  B8-G04's gate: vanilla 26.1.1's
  `PointedDripstoneBlock.animateTick` rolls 0.12 and climbs
  same-block stalactite via `findRootBlock` (no tag involved —
  there is no `SpeleothemBlock` and no `speleothems` tag in
  26.1.1), and the port's `dripstone_drip`
  (`animate_tick.rs:1807`) matches it by name, with the same 0.12
  roll and 0.02 cauldron rule. The marker oracle's scope comment
  about binding "the speleothem tag the stalactite branch needs"
  (`VanillaMarkerParticlesOracle.java:136`) is stale for 26.1.1 —
  the oracle binds tags from the jar's datapack, which has no such
  tag, and no code path reads it — but the comment is not a defect.
- **B12-G04 pattern — 26.x-only types in the `ParticleType` table.**
  Same table-leak *pattern* as B12-G04, but the 11 extra types in
  the Rust enum (8 26.2-only: geyser family, sulfur bubbles,
  noxious gas pair, sulfur cube goo; 3 26.3-only: poplar leaves)
  are unreachable — ids are name-resolved from the server's
  registry, and a 26.1.1 server never sends those names — so they
  are not minted as a defect. Dead table entries, not a behaviour
  gap. The 4 geyser entries in `override_limiter()` are likewise
  dead. (Note: the 26.1.1 tree REMOVED the 26.1 tree's doc comment
  documenting the 8 26.2-only variants — the variants themselves
  are unchanged.)
- **The `SpeleothemBlock` assumption elsewhere in the tree.** B8-G04
  is the particle-visible instance; the same 26.2+-only class name
  gates other behaviour: `mesher.rs:910`
  (`has_class("SpeleothemBlock")` selects the 0.125 horizontal
  random offset for dripstone — dead in 26.1.1, so dripstone meshes
  with the 0.25 default; vanilla 26.1.1's `getMaxHorizontalOffset`
  is `SHAPE_BASE.min(X)`, i.e. the 0.125 branch) and the
  server-side `horizon-world` speleothem/falling code
  (`level/speleothem.rs`, `level/falling.rs:54`,
  `level/classes.rs:215` — Batch 5's scope). The projectile-hit
  path is fine: it matches by block name, not class
  (`creatures.rs:5084`).
- **3003 wax-on sound missing from the level-event path → Batch 10
  (sound).** Vanilla 26.1.1's 3003 case spawns WAX_ON particles
  (3–5) AND plays `HONEYCOMB_WAX_ON` via `playLocalSound`
  (offsets 3358–3391). The 26.1.1 tree kept the particles
  (`block_particle_event`, `level_particles.rs:35-38`, routed from
  `game.rs:4367`) but dropped the sound the 26.1 tree played from
  the event (`level_particles.rs:35` in 26.1). The item-use,
  dispenser, remote and copper-golem wax sounds are all still
  present (`game.rs:8020` + callers :8331/:8350,
  `dispenser.rs:2229`, `remote.rs:8783/8791`,
  `copper_golems.rs:436`), so the missing case is specifically a
  client receiving the 3003 event for someone else's waxing (LAN
  guest view). Sound defect, not a particle defect — flagged for
  Batch 10 with this evidence, not minted here.
- **2002/2007 splash-potion break sound missing → Batch 10
  (sound).** Vanilla 26.1.1's 2002/2007 case plays
  `SPLASH_POTION_BREAK` itself, after the particles (offsets
  1524–1547). The 26.1.1 tree dropped the sound the 26.1 tree
  played from the event and instead maps it to events 1053/1054
  (`game.rs:4489`) — which vanilla 26.1.1 never sends (26.3 split
  the sound into 1053/1054; 26.1.1's lookupswitch has no such
  cases). The potion-splash particles still fire
  (`potion_splash_particles`, `game.rs:4360/4365`). Sound defect —
  flagged for Batch 10, not minted here.
- **Digging hit-sound schedule (sound half of B8-G09) → Batch 10.**
  The particle half (extra crumb) is B8-G09; the sound half (hit
  sound at first hit instead of every 4th tick, then silence) is a
  sound defect with the same evidence — flagged for Batch 10.
- **Enderman teleport destination selection → Batch 3 (B3-G14).**
  The 26.1.1 `mobs/enderman.rs` grounds the teleport on the absent
  `#entities_can_teleport_to` / `#enderman_does_not_teleport_to`
  tags (26.3 form); vanilla 26.1.1 scans for blocks-motion ground
  (`EnderMan.teleport`, `blocksMotion` loop). Already recorded as
  B3-G14; only the particle-visible 2018 trail is minted here
  (B8-G07).
- **`CONSUMABLE_DOES_NOT_TELEPORT_TO` avoidance tag → Batch 3.**
  `teleport_consume.rs:36` documents the 26.3-form forbidden tag;
  the tag is absent from both 26.1.1 jars and the prepared data.
  The destination-selection consequence is Batch 3's scope, not
  verified here; only the particle-visible 2017 trail is minted
  here (B8-G06).
- **B8-G10 routing.** `/spreadplayers` is commands-area (Batch 9);
  it is minted here only because the coordinator routed Batch 7's
  flag to Batch 8. The merge may re-route it to Batch 9's ledger.

## §8 What passed (per area)

### Registry and type coverage (120 checks: 118 pass, 2 gap)

- 117 particle types: `registries.json` count == jar `ParticleTypes`
  `<clinit>` registration count (117), and the `<clinit>` order equals
  the `protocol_id` of every type (verified by parsing the bytecode;
  0 mismatches vs the prepared registries — prepared == datagen
  exactly, `elder_guardian` present at id 17, no geyser/sulfur/
  poplar/noxious entries). `ParticleTypes.class` is byte-identical
  (md5 `8292544543edd947bd3398873d591f30`) between the client and
  server-game jars.
- Engine `make()` (`particles.rs:2205`, outer match at :2264) has an
  outer match arm for all 127 `ParticleType` variants (verified fresh
  by script: 127/127 variants covered, no outer catch-all — the `_ =>`
  arms in the body are all nested matches) — every enum variant has a
  provider implementation. The only engine-coverage gap is the missing
  `elder_guardian` variant itself (B8-G01).
- Vanilla providers: `ParticleResources.registerProviders()` in the
  26.1.1 client jar registers a provider for all 117 types, including
  `ELDER_GUARDIAN` (`ElderGuardianParticle$Provider`, offsets
  296–300). (Field-name quirk, same as 26.1: the fields are
  `TRIAL_SPAWNER_DETECTED_PLAYER(_OMINOUS)` but register under the
  names `trial_spawner_detection(_ominous)`.)
- The 11 Rust-enum extras (8 26.2-only geyser/sulfur + 3 26.3-only
  poplar leaves) are absent from the 26.1.1 jar's `ParticleTypes` —
  confirmed unreachable from a 26.1.1 server.
- Gap: providers-driver kind set (120 kinds) misses `elder_guardian`
  (B8-G01) and `vibration` (B8-H01), and carries the 11 non-26.1.1
  kinds (B8-H02).
- `STONE = 1` in the providers driver is correct for 26.1.1: state
  id 1 is stone in the 26.1.1 datagen `blocks.json` (the "in 26.3"
  comment is stale; the value is right).

### Wire codec (20 checks: 19 pass, 1 gap)

- All 18 data-carrying particle wire ids match `Particle::read`'s
  hardcoded ids exactly (verified fresh against the prepared 26.1.1
  registry, 0 mismatches): State 1/2/29/111/115 (block,
  block_marker, falling_dust, dust_pillar, block_crumble), Color
  21/36/42 (entity_effect, tinted_leaves, flash), dust 14,
  dust_color_transition 15, effect 16, instant_effect 46,
  dragon_breath 8, sculk_charge 38, shriek 105, trail 49, item 47,
  vibration 48. `ids::particle(id)` is the identity when no version
  adapter is active (`adapter/ids.rs`, file identical to 26.1's), so
  these are the native 26.1.1 ids. The read code carries no geyser
  arms — correct for 26.1.1 (comment: "The geyser particles a later
  release adds do not exist in 26.1"), though the `Geyser`/`GeyserBase`
  data variants still exist in the enum and `write` still writes
  them if present (unreachable from a 26.1.1 read; `hub_particle`
  can still build them from a `ParticleOptions::geyser`, but no
  26.1.1 path creates one).
- `received_particle` resolves ids to names through the server-sent
  registry (not a hardcoded table) — correct for all 116 implemented
  types.
- Gap: B8-G02 (Flash write path).
- Note (not a defect): `hub_particle` deliberately returns `None` for
  item and trail particles (`lan_particles.rs`, `network_particle`
  doc: "a particle whose data isn't sent from here (items, trails)")
  — a LAN host's `/particle item|trail` draws locally but sends
  nothing to guests. Documented behaviour, same as 26.1; the command
  does not parse trail options either. Not minted.

### Override limiter (28 checks: 27 pass, 1 gap)

- Vanilla limiter-true set (28 types, from the `<clinit>` booleans,
  parsed fresh from the 26.1.1 jar) vs the Rust `override_limiter()`
  list: identical except `elder_guardian` (B8-G01). The Rust list's
  4 extra entries are the geyser types — dead in 26.1.1,
  unreachable.

### Entity event bursts (30 checks: 30 pass)

- Every (entity, event) pair in the entity driver's `EVENTS` table
  (12 entities, 30 pairs, re-counted fresh) is covered by the port's
  `entity_event_burst` (`particles.rs:620`, region byte-identical to
  26.1's): villager 12/13/14/42, cow 18, wolf/cat 6/7/18, parrot 6/7,
  nautilus 6/7/18, horse/llama/camel 6/7/18, ocelot 40/41/18, dolphin
  38, allay 18 — with the right particle kinds and burst shapes,
  verified against the 26.1.1 jar (the entity classes are
  byte-identical to 26.1's, so the 26.1 jar verification carries:
  `Villager.handleEntityEvent` 12→heart, 13→angry, 14→happy,
  42→splash; `TamableAnimal` 7→heart/6→smoke, 7 particles × 0.02
  gaussian; `Animal` 18→heart ×7; `Ocelot` 41→heart/40→smoke;
  `Dolphin` 38→happy_villager ×7 (0.01 gaussian, y+0.2); `Allay`
  18→3× heart (0.02 gaussian, y+0.5); `AbstractHorse` 6/7→taming
  particles; `AbstractVillager.addParticlesAroundSelf` 5 × 0.02).
  Counts/positions/RNG draws need the driver run; the mapping table
  is exact.
- The port's table also carries `nautilus`, `zombie_nautilus` and
  `camel_husk` arms — live and correct in 26.1.1 (unlike 1.21.7,
  where they were dead 26.x arms): `AbstractNautilus extends
  TamableAnimal`, `ZombieNautilus extends AbstractNautilus`,
  `CamelHusk extends Camel`. The driver just doesn't test the last
  two (B8-H04).

### Marker zoo (45 checks: 45 pass)

- Every real 26.1.1 block name in the marker driver's zoo (36 names,
  re-counted: 37 distinct names in `zoo_states()` minus
  `red_poplar_leaves`) + neighbour (6: dried_ghast, beehive,
  bee_nest, redstone_ore, deepslate_redstone_ore, pointed_dripstone)
  + campfire (2: campfire, soul_campfire) + wither_rose (1) cases
  has a `classify` arm in `animate_tick.rs` (file byte-identical to
  26.1's), including the guarded ones (lit furnaces, powered
  levers/repeaters, honey-level-5 hives, charged respawn anchors,
  active sculk sensors, down-pointing dripstone tips, lit
  candles/candle cakes). Positions/RNG draws need the driver run;
  coverage is complete. (`red_poplar_leaves` is the 37th zoo name;
  it has a classify arm in this tree but the block does not exist in
  26.1.1 — B8-H03, a driver-scope item, not a port gap.)
- The unlit variants in the zoo (unlit redstone torches, furnaces,
  smokers) correctly map to `Animate::None`.

### Leaves (12 checks: 11 pass, 1 gap)

Per-block `leafParticleChance` and class — `Blocks.class` is
byte-identical to 26.1's (vanilla side carries exactly) and
`animate_tick.rs` is byte-identical to 26.1's (port side carries
exactly); the spruce line was re-read fresh
(`animate_tick.rs:151`):

| Block | Vanilla 26.1.1 | Port | Verdict |
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
- Note: vanilla 26.1.1 has no `CherryLeavesBlock`/`PaleOakLeavesBlock`
  classes — cherry/pale-oak are `UntintedParticleLeavesBlock` instances,
  and the port's `FallingParticlesLeavesBlock` comment names the 26.3
  class; the mechanism is the 26.1.1 one.

### Dripstone and tags (7 checks: 6 pass, 1 gap)

- `speleothems` does NOT exist in 26.1.1 (absent from the client
  jar's datapack and the prepared `block-tags.json`; no
  `SpeleothemBlock` class) — a real difference from 26.2, where it
  held `pointed_dripstone` + `sulfur_spike`. The port's animateTick
  drip path does not need it (name-based table), so this is a pass
  for the animate path; the class-gated level-event path is the gap
  (B8-G04).
- `washed_away_by_fluids` and `blocks_lava_fire_spread` are both
  absent from the 26.1.1 prepared tags — confirms the pilot's tag
  inventory pattern for 26.1.1.
- `triggers_ambient_dried_ghast_block_sounds` exists (soul_sand,
  soul_soil), as do the two desert ambient-sound tags
  (`triggers_ambient_desert_sand_block_sounds`: sand, red sand;
  `triggers_ambient_desert_dry_vegetation_block_sounds`: #terracotta,
  sand, red sand) — the tags the port's `AnimateTables` binds are
  real in 26.1.1. (The poplar tag it also binds,
  `required_for_poplar_leaf_ambience`, has no 26.1.1 referent — dead
  path, no poplar blocks exist.)
- Vanilla dripstone animateTick path matches the port (0.12 roll,
  same-block climb, 0.02 cauldron rule, mud-as-water, 5/16 − 0.0625
  drip height).
- Gap: B8-G04 (level-event 1504 gate).

### Oracle API existence (8 checks: 8 pass)

All vanilla methods the oracles call exist in the 26.1.1 jars with the
expected signatures (javap on the named jars, fresh):
`ParticleUtils.spawnParticlesOnBlockFaces` /
`spawnParticlesAlongAxis` (emitter, `net.minecraft.util.ParticleUtils`
in 26.x), `LivingEntity.spawnItemParticles` (item),
`LivingEntity.handleEntityEvent` (entity — CORRECTION to the 26.1
report, which attributed it to `Villager`: the method is declared on
`LivingEntity` and inherited by Villager; the check still passes),
`CampfireBlock.animateTick` / `makeParticles` /
`CampfireBlockEntity.particleTick` (marker), `ClientLevel.animateTick`
(marker loop). This is existence only — the oracles' *use* of them is
what the drivers verify.

### Other passes

- Engine `make()` exhaustiveness (see above — 127/127 arms, fresh).
- Game-event-10 apparition re-implementation present (§4, B8-G01).
- Bubble-column animate arm present (§7, G1).

### Level events 2015–2020 + digging (5 checks: 1 pass, 4 gap) — NEW

- **2015 dragon egg: pass.** Vanilla 26.1.1 has no 2015 level event,
  but `DragonEggBlock.teleport` spawns 128 portal particles lerped
  between the old and new block positions via `Level.addParticle`
  (loop bound `sipush 128`, `Mth.lerp` on x/y/z). The port's
  server sends the port-internal 2015 event
  (`horizon-world/src/level/interact.rs:595`, shared with 26.1)
  and the client draws 128 portal particles lerped along the same
  line (`block_teleportation`, `[16,8,16]`). Mechanism differs,
  visible result matches. (Exact RNG-draw equivalence needs a run.)
- **2016 shulker: gap — B8-G08.** **2017 consume: gap — B8-G06.**
  **2018 enderman: gap — B8-G07.**
- **Digging crack particles: gap — B8-G09.** (The 2019/2020 network
  dispatch itself is dead — no sender — so it is not a separate
  check; the live defect is the client-side `remote.rs:8180` call.)

### Particle group cap (1 check: 0 pass, 1 gap) — NEW

- Gap: B8-G05 (the `tick_tracking` pending drain enforces the cap
  vanilla never enforces).

### Routed check: /spreadplayers safety (1 check: 0 pass, 1 gap) — NEW

- Gap: B8-G10 (Batch 7-routed flag, verified fresh).

## §9 Oracle repairs and defects

Triage is from the measured Batch 0 table
(`docs/sweep/26.1.1/batch-0-oracles.json`, 391 oracles, 2026-11-24) —
NOT from import resolution. Both failing particle oracles were
re-verified against the 26.1.1 jar by this batch (javap + jar
listing). All 8 particle-family oracle sources are byte-identical to
26.1's.

### B8-O01 — `VanillaMarkerParticlesOracle` BROKEN, 1 compile error (MINOR)

Measured: fails on all three classpaths, 1 error, class
constructor-mismatch — "44: constructor LocalPlayer in class LocalPlayer
cannot be applied to given types". This is Batch 0 group G2 (the
LocalPlayer constructor mismatch). Verified fresh: the oracle's
`Player` subclass calls `super(null, null, null, null, null, null,
false, null, null)` — 9 arguments, the 26.3 form
(`VanillaMarkerParticlesOracle.java:44`); 26.1.1's `LocalPlayer`
constructor takes 8 (`Minecraft, ClientLevel, ClientPacketListener,
StatsCounter, ClientRecipeBook, Input, boolean, ChatAbilities` —
javap on the 26.1.1 client jar). **Blocks the marker_particles driver
entirely** — before it can reach the B8-G03 spruce cases. Repair is
the G2 fix (align the `LocalPlayer` construction with 26.1.1's
constructor), shared with the other G2 oracles; do not repair here
(checks + triage only).

### B8-O02 — `VanillaParticleProvidersOracle` BROKEN, 10 compile errors (MINOR)

Measured: fails on all three classpaths, 10 errors, class
missing-symbol — first error "164: cannot find symbol". All 10 errors
are references to the geyser API, which 26.1.1 predates:
`GeyserParticleOptions` (3 refs: :144 constructor, :164 type + cast),
`GeyserBaseParticleOptions` (3 refs: :146 constructor, :165 type +
cast), `ParticleTypes.GEYSER` / `GEYSER_PLUME` (:143),
`ParticleTypes.GEYSER_BASE` / `GEYSER_POOF` (:145). Verified fresh:
zero `geyser` entries in either 26.1.1 jar, and no `GEYSER*` fields
in `ParticleTypes`. **Blocks the particle_providers driver
entirely.** Repair is a deletion (the geyser branches + the two cast
helpers — the driver never sends geyser kinds in its main 120-kind
set, though it DOES send them in its geyser section, B8-H02 — so the
trim and the repair go together); do not repair here.

### Oracles assessed PASSING for 26.1.1 (Batch 0 measurement)

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

For comparison, the 26.2 run had only the marker oracle failing
(B8-O01); 26.1 and 26.1.1 add the providers oracle (B8-O02) because
both predate the geyser API that 26.2 already had.

Static evidence for 26.1.1 (supporting the measured passes, not a
substitute for them):

- The 8 oracle APIs this batch's scope touches all exist (§8).
- The providers oracle's mechanism fits 26.1.1: it builds
  `new ParticleResources()`, reads the private `spriteSets` field by
  reflection, and calls the public `getProviders()` — all present in
  the 26.1.1 client jar (`ParticleResources` holds
  `registerProviders()` in 26.1.1; `ParticleEngine` has no such
  method). It resolves kinds by registry name, which is why the 11
  non-26.1.1 kinds would throw (B8-H02).
- The marker oracle's `bindBlockTags()` reads tags from the jar's
  datapack at runtime, so the absent `speleothems` tag is simply
  unbound — no code path reads it in 26.1.1.

## §10 Driver inventory

The plan (§4) claims 9 particle drivers; the tree has 7 particle-named
drivers plus 1 particle-adjacent driver = **8**. The 9th is unaccounted
for: possibly the plan counted `tests/test_cushion_particles.py` (a
Python unit test for Horizon's cushion feature, not a parity driver) or
double-counted. No 9th parity driver exists under any name in
`versions/26.1.1/tools/` (all 8 driver files byte-identical to 26.1's).

| Driver | Oracle (26.1.1 status, MEASURED) | Probe | Status |
|---|---|---|---|
| block_particles | BlockParticles: passes client+both (server-only failure is client packages, expected) | block_particles_fixture.rs | UNRUN |
| emitter_particles | EmitterParticles: passes client+both (server-only failure is client packages, expected) | emitter_particles_probe.rs | UNRUN |
| entity_particles | EntityParticles: passes client+both (server-only failure is client packages, expected) | entity_particles_probe.rs | UNRUN; driver omits zombie_nautilus/camel_husk (B8-H04) |
| item_particles | ItemParticles: passes all three classpaths | item_particles_fixture.rs (horizon-world) | UNRUN |
| marker_particles | MarkerParticles: FAILS all three classpaths, 1 error (LocalPlayer ctor, B8-O01) | marker_particles_probe.rs | BLOCKED (B8-O01) — oracle does not compile; would also fail on spruce cases (B8-G03) once repaired; red_poplar_leaves zoo cases un-runnable (B8-H03) |
| native_client_particles | none (Horizon-only Host API contract, no vanilla oracle) | game fixture | UNRUN; out of vanilla scope |
| particle_providers | ParticleProviders: FAILS all three classpaths, 10 errors (geyser symbols, B8-O02) | particle_providers_probe.rs | BLOCKED (B8-O02) — oracle does not compile; would also FAIL on the 11 non-26.1.1 kinds — oracle throws `AssertionError("unknown particle …")` (B8-H02); does not test elder_guardian (B8-G01) or vibration (B8-H01) |
| effect_sync (adjacent) | EffectParticles: passes all three; EffectSync: passes all three | effect_sync_fixture.rs | UNRUN; scope is effect particle/visibility sync |

`parity.py` was not run (blocked — static checks only, no cargo).
Prepared 26.1.1 data exists (`.local/horizon-client/26.1.1/`, Batch 0
prepare), so the data precondition for the driver runs in §13 is met —
the runs themselves remain follow-up work. 6 of the 8 drivers are
runnable once their probes build; the other 2 wait on the B8-O01 /
B8-O02 oracle repairs plus the B8-H02/H03 trims.

## §11 Unit tests

31 `#[test]`s in `crates/horizon-client-render/src/particles.rs`
(same names as 26.1 — verified by diff of test names), unrun (blocked
— no cargo builds in this batch):

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

Same 31 tests as the 26.1, 26.2 and 1.21.7 batches (same names, same
file). Note: `consume_teleport_unpacks_offsets_within_127` and
`egg_teleport_unpacks_offsets_and_preserves_seeded_draw_order` test
the teleport-unpack helpers the 2017/2015 paths use — they do not
test whether the events should fire at all.

## §12 Not covered

- **All driver numerics.** Per-tick provider physics (positions, motion,
  age, lifetime, size, colour, roll, sprite, UVs, light), emitter RNG
  draws, entity-burst counts/positions, block-particle placement, marker
  loop exactness — only the drivers can verify these, and none ran. The
  static checks above bound the *tables and mappings*, not the numerics.
- **Translucent-layer table.** `ParticleType::translucent_layer()` (which
  types vanilla renders in the translucent layer) was not verified — it
  needs per-class `getRenderType` checks across ~20 particle classes.
  (Same scope decision as the 26.1, 26.2 and 1.21.7 batches.)
- **Sprite sets / particle textures.** Whether the engine resolves every
  type's sprite set from the 26.1.1 assets needs prepared data + a run.
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
- **Dripstone climb limit.** Vanilla's `findRootBlock` passes 11 to
  `findBlockVertical`; the port climbs 10 steps (`1..11`). Whether the
  off-by-one reading matches needs the marker driver run.
- **G1/G6 server-side status for 26.1.1.** Whether bubble columns form
  and dripstone transfers fluid server-side in the 26.1.1 port is Batch
  5's scope, not this batch's.
- **The 3003/2002 sound halves and the digging sound schedule** are
  Batch 10's scope (§7); the particle halves were verified here.
- **Enderman/shulker teleport destination selection** is Batch 3's
  scope (B3-G14 covers the enderman tag form); only the
  particle-visible trails were verified here.
- **`sounds.json` is genuinely absent** from the prepared data (the
  separate `horizon-prepare sounds` step was not run — Batch 0). Sound
  names in the particle paths were not checked against prepared sound
  data. `en_us.json` IS present (verified in
  `.local/horizon-client/26.1.1/assets/minecraft/lang/`).
- **Batch 7's G09 masking claim for the 2017 trail** (the trail fires
  only after a successful teleport, which never happens on the player
  path) was not verified here; the trail mechanism itself was verified
  fresh. If G09 is fixed without removing the trail, the trail becomes
  visible — Batch 7's warning stands as recorded.

## §13 What unblocks the rest

1. Prepared 26.1.1 data exists (Batch 0 — done); `sounds.json` remains
   absent, which does not block the particle drivers.
2. Repair the two broken oracles (follow-up, not this batch): B8-O01
   (the G2 LocalPlayer-constructor fix for
   `VanillaMarkerParticlesOracle`) and B8-O02 (delete the geyser
   branches + cast helpers from `VanillaParticleProvidersOracle`).
   Six of eight particle-family oracles already compile on their real
   classpath (measured, §9).
3. Fix B8-G01 (add the `ElderGuardian` variant + `ElderGuardianParticle`
   provider), B8-G03 (spruce leaves arm) and B8-G04 (the 1504 gate —
   match by name or by `PointedDripstoneBlock`, not `SpeleothemBlock`)
   on fix branches, then run `particle_providers` and
   `marker_particles`; fix B8-G02 (Flash arm in `hub_particle`) with
   them or separately. B8-G03 is driver-blocking; B8-G02 and B8-G04 are
   not covered by any driver; B8-G01 blocks nothing in the 26.1.1 tree
   because the driver skips the type — the fix should add
   `elder_guardian` to the driver's kind set too, or the type stays
   untested.
4. Fix the five 26.1.1 regressions (B8-G05–G09) — the 26.1 tree is the
   repair reference for all five (the cap removal, the
   `breaking_block_sound` form, the absence of the 2016/2017/2018
   handlers and senders). Fix B8-G10 by restoring vanilla's `isSafe`
   (`y < maxY && !liquid && !is(FIRE)`) in `spread_safe`. None of
   B8-G05–G10 is covered by any driver; all need in-game verification
   (particle-heavy scene for the cap; chorus fruit, enderman and
   shulker teleports; digging; `/spreadplayers`).
5. Trim the non-26.1.1 entries from the drivers for 26.1.1 (B8-H02: 11
   kinds in the providers driver — 3 poplars + 2 sulfur + 2 noxious +
   4 geyser; B8-H03: `red_poplar_leaves` in the marker zoo), add the
   missing coverage (B8-H01: vibration; B8-H04: zombie_nautilus,
   camel_husk), then run the drivers (`block`, `emitter`, `entity`,
   `item`, `effect_sync`); the item driver needs a longer cargo-build
   window than its 900 s timeout under load (or a warm target dir) —
   1.21.7 precedent.
6. Run the 31 `particles.rs` unit tests (`cargo test -p
   horizon-client-render --lib particles::`, dev profile, `-j 1`).
7. Route the Batch 10 sound flags (§7: 3003 wax-on, 2002/2007
   potion-break, digging sound schedule) and the B8-G10 routing
   question (commands-area; Batch 9's ledger?) to the coordinator.

## §14 Method notes

- The 26.1.1 **client** jar is Mojang-mapped —
  `net/minecraft/client/particle/ParticleEngine.class` is present by
  its mapped name — and its `ParticleTypes.class` is byte-identical
  (md5) to the server-game jar's. The leakage-audit 26.1.x server jars
  are bundler jars (never javap'd); the named game jars are
  `26.1.x/versions/26.1.1/jars/{client,server-game}.jar`.
- **Jar-identity carry.** All 28 particle-relevant classes checked
  are byte-identical between vanilla 26.1 and 26.1.1 (script:
  `evidence/jar_compare.py`). Where this report says a 26.1 jar
  finding "carries", that md5 identity is the basis — not an
  assumption. Every NEW claim (cap, 2016/2017/2018, digging,
  spreadplayers, dragon egg, shulker, enderman) was javap-verified
  fresh against the 26.1.1 jars.
- 26.x registration shapes differ from 1.21.7 in two places this batch
  had to adapt to:
  - `Blocks.<clinit>` registers via `register(BlockItemId, Function,
    Properties)` with the name from a `getstatic BlockItemIds.<NAME>`
    (no `ldc` name strings) — the 26.1 batch's class-file parser
    associates `BlockItemIds` fields with their `invokedynamic`
    lambdas instead of string constants. Not re-run here:
    `Blocks.class` is byte-identical to 26.1's, so the 26.1 parse
    carries exactly.
  - Providers live in `ParticleResources.registerProviders()` (private),
    not `ParticleEngine` — and 26.1.1 has a dedicated
    `ElderGuardianParticle$Provider` where 1.21.7 used the shared
    `MobAppearanceParticle` provider.
- The `ParticleTypes.<clinit>` parse (evidence, inline in this batch's
  scratch) walks the `javap -c` dump anchored on `putstatic` lines,
  reading the name (`ldc`/`ldc_w` string) and the limiter boolean
  (`iconst_0`/`iconst_1`) backwards from each. Two gotchas: constant
  indices above 255 use `ldc_w`, not `ldc` (a forward parser that
  only matches `ldc` silently drops 83 of 117 registrations); and the
  regex for the wide form is `ldc(_w)?`, not `ldc_w?` (the latter
  requires a literal underscore and matches nothing).
- `javap` on the small classes ran 5–40 s each under load; batches of
  2–5 classes per exec kept the work moving. `javap -c` on `Blocks`
  was not used (1.21.7 warning: exceeds 280 s under throttle) —
  unnecessary here given the md5 identity.
- The providers oracle's 10 errors were NOT re-reproduced with a
  single-file javac run (the 26.1 batch did that); the Batch 0
  measured table (10 errors, first at :164) plus the fresh
  geyser-absence check (0 entries in either jar) plus the
  byte-identical oracle source establish the same breakage.
- No production code (`crates/`) was touched. No commits were made in
  this batch. Only the two deliverable files were written (plus
  evidence/ scratch: javap dumps, the registry census, the
  `ParticleTypes` `<clinit>` parse, and `jar_compare.py`).

## §15 Evidence index

Worktree `~/workspace/mcir-work/sweep-26.1.1`, version tree
`versions/26.1.1/` unless stated. Jars:
`~/workspace/mcir-work/26.1.x/versions/26.1.1/jars/{client,server-game}.jar`.
Scratch: `~/workspace/mcir-work/sweep-26.1.1-batches/b08/evidence/`.

**Jar (fresh, 26.1.1):**
- `evidence/particle_types.txt` — `ParticleTypes` `javap -c`
  (`<clinit>` from line 333; 117 registrations in protocol-id order,
  28 limiter-true; `ELDER_GUARDIAN` id 17, limiter true). md5
  `8292544543edd947bd3398873d591f30` in both jars.
- `evidence/level_event_handler.txt` — `LevelEventHandler` `javap -c`
  (79-case lookupswitch, lines 116–235: NO 2015–2020; 3003 case
  offsets 3358–3391: WAX_ON + HONEYCOMB_WAX_ON; 2002/2007 case offset
  1191, SPLASH_POTION_BREAK at 1524–1547; 1504 case offset 3495:
  `PointedDripstoneBlock.spawnDripParticle`).
- `evidence/mpgm.txt` — `MultiPlayerGameMode` `javap -c`
  (`addBreakingBlockEffect`: 0 calls; hit sound every 4th tick,
  offsets 197–262).
- `evidence/living.txt` / `living_client.txt` —
  `LivingEntity.randomTeleport` broadcasts entity event 46;
  `handleEntityEvent` case 46 → 128 PORTAL particles.
- `evidence/shulker.txt` — `Shulker.teleportSomewhere`: no particles,
  no entity/level event (offsets 143–219).
- `evidence/enderman.txt` — `EnderMan.teleport` → `randomTeleport(…,
  true)` (offset 110); the only `addParticle` is ambient aiStep.
- `evidence/dragonegg.txt` — `DragonEggBlock.teleport`: 128 PORTAL
  lerped old→new (loop bound `sipush 128`, offsets 142–147).
- `evidence/spread.txt` — `SpreadPlayersCommand$Position.isSafe`:
  `y < maxY && !liquid && !is(FIRE)` (lines 202–262).
- `LocalPlayer` ctor: 8 args (javap, fresh). `ParticleGroup.add`:
  plain queue add (javap `-c`, fresh). `ParticleResources`:
  `ElderGuardianParticle$Provider` registered (offsets 296–300).
  `ClientPacketListener.handleGameEvent`: GUARDIAN_ELDER_EFFECT
  offset 520, ELDER_GUARDIAN addParticle offset 530, curse sound
  offset 574. `TeleportRandomlyConsumeEffect`: record with only a
  `diameter` field. Oracle APIs: `ParticleUtils` ×2,
  `LivingEntity.spawnItemParticles`,
  `LivingEntity.handleEntityEvent`, `CampfireBlock.animateTick` /
  `makeParticles`, `CampfireBlockEntity.particleTick`,
  `ClientLevel.animateTick` (all javap-verified fresh).
- `evidence/jar_compare.py` + output — 28 classes byte-identical
  between vanilla 26.1 and 26.1.1; 0 entries for `SpeleothemBlock`,
  `Geyser`, `EntityTypes`, `BlockEntityTypes`, `SulfurCube`, `Poplar`
  in either 26.1.1 jar; 0 `entities_can_teleport_to` /
  `consumable_does_not_teleport_to` datapack entries.

**Data (26.1.1's own):**
- Prepared `.local/horizon-client/26.1.1/registries.json` ==
  datagen `registries.json` for `minecraft:particle_type` (117
  entries, ids contiguous 0–116; elder_guardian 17, flash 42,
  vibration 48, portal 60, note 58; no geyser/sulfur/noxious/poplar).
- `.local/horizon-client/26.1.1/items.json` (1,506 items):
  chorus_fruit consumable is `{"type":
  "minecraft:teleport_randomly"}` only; 0 occurrences of
  `directional_particles`.
- `.local/horizon-client/26.1.1/block-tags.json`: `speleothems`,
  `required_for_poplar_leaf_ambience`, `washed_away_by_fluids`,
  `blocks_lava_fire_spread`, `entities_can_teleport_to`,
  `consumable_does_not_teleport_to` all ABSENT; the three ambient
  tags present.
- `.local/horizon-client/26.1.1/block-behaviours.json`:
  pointed_dripstone class chain `["PointedDripstoneBlock", "Block"]`.
- Datagen `blocks.json`: state id 1 is `minecraft:stone`.
- `sounds.json` absent from prepared data; `en_us.json` present.

**Port source (26.1.1 tree):**
- `crates/horizon-client-core/src/particle_type.rs` — 127 variants
  (macro list), no `ElderGuardian`; `override_limiter` :154-189 (31
  entries: 27 real + 4 geyser).
- `crates/horizon-client/src/game/lan_particles.rs` —
  `hub_particle` :408, Color match :437 (no Flash arm);
  `received_particle` :466, `from_name` :488; `Item | Trail => None`.
- `crates/horizon-client/src/game/commands/world_cmds.rs` —
  `cmd_particle` :750, `particle.notFound` :760, catch-all
  `simple(ty)` :871, `network_particle` :925; `spread_safe`
  :2132-2154, tag read :2141-2146, spread loop :2018-2026, failure
  :2030-2043.
- `crates/horizon-client/src/animate_tick.rs` — leaves :140-151
  (spruce :151), poplar arms :144-146, dripstone table :397-410,
  `dripstone_drip` :1807, bubble column :1660.
- `crates/horizon-client/src/game/level_particles.rs` —
  `dragon_egg_particles` :15, `shulker_teleport_particles` :20,
  `consume_teleport_particles` :23, `enderman_teleport_particles`
  :28, `block_particle_event` :35 (3003→WaxOn :38),
  `is_stalactite` :243-244, `dripstone_drip_particle` :256,
  `breaking_block_effects` :351-373, `potion_splash_particles` :378.
- `crates/horizon-client/src/game.rs` — 2019/2020 dispatch
  :4298-4306, 2015 :4328-4331, 2016 :4333-4336, 2017 :4338-4341, 2018
  :4343-4346, 2002/2007 :4355-4365, 3002..=3005 :4367, 1504
  :4376-4377, 1053|1054 :4489, `wax_on_sound` :8020 (callers
  :8331/:8350).
- `crates/horizon-client/src/game/remote.rs` — digging call :8180,
  event-46 handler :4782-4784, wax sounds :8783/:8791.
- `crates/horizon-client/src/game/consume.rs` — `portal_particles`
  :208, `entity_teleport_particles` :228,
  `check_position_and_teleport` :142-194 (event 46 :188-194),
  `directional_particles` :422-431.
- `crates/horizon-client/src/mobs/enderman.rs` — event 46 :161-166,
  2018 send :174-184. `crates/horizon-client/src/mobs/shulker.rs` —
  2016 send :366-371.
- `crates/horizon-client-render/src/particles.rs` —
  `entity_event_burst` :620, `add` :1797, `create` :2054
  (`pending.push` :2087), `make` :2205 (outer match :2264, 127/127
  arms), `block_teleportation` :1848,
  `enderman_teleportation` :1873, `tick_tracking` :3880, cap
  :3925-3935, `MAX_PARTICLES` :1349-1350, firework flash :3777, 31
  tests.
- `crates/horizon-protocol/src/entity_data.rs` — `Particle::read`
  :185 (Color ids :215, vibration :265). `adapter/ids.rs` —
  identity, file identical to 26.1's.
- `crates/horizon-items/src/items.rs:124` — `directional_particles`
  default true. `crates/horizon-world/src/teleport_consume.rs` —
  field :6, 2017 fire :109-121. `crates/horizon-items/src/blocks.rs:121`
  — `has_class` exact match.
- `crates/horizon-client-render/src/mesher.rs:910` —
  `SpeleothemBlock` offset gate (cross-ref).
- `crates/horizon-world/src/level/interact.rs:595` — 2015 send
  (shared with 26.1).
- Drivers: `tools/test_particle_providers_parity.py` — SIMPLE :26-38
  (103 kinds), kinds :74-76 (120 total), geyser/noxious sections
  :94-105, STONE :23; `tools/test_marker_particles_parity.py` —
  zoo_states :76 (37 distinct names), red_poplar :118, spruce
  comment :114, neighbour_cases :168, campfire_cases :37,
  wither_rose_cases :156; `tools/test_entity_particles_parity.py` —
  EVENTS :17 (12 entities, 30 pairs).
- Oracles: `mods/horizon/src/test/java/dev/horizonlab/parity/` —
  `VanillaMarkerParticlesOracle.java:44` (9-arg super call),
  `VanillaParticleProvidersOracle.java:131` (unknown-particle
  throw), :143-146/:164-165 (geyser refs).
- Batch 0 table: `docs/sweep/26.1.1/batch-0-oracles.json`
  (Marker: fail-all, 1 error, constructor-mismatch; Providers:
  fail-all, 10 errors, missing-symbol; other six particle oracles
  pass on their real classpath).
- 26.1 tree diffs: `particle_type.rs`, `lan_particles.rs`
  (comment-only); `animate_tick.rs`, `entity_data.rs`,
  `world_cmds.rs`, all 8 drivers, all 8 oracles (byte-identical);
  `level_particles.rs`, `particles.rs`, `game.rs`, `remote.rs`,
  `consume.rs`, `items.rs`, `teleport_consume.rs`,
  `mobs/enderman.rs` (real diffs — the B8-G05–G09 evidence).

## §16 Verdict

**279 behaviour checks: 267 pass, 12 gap. 16 items: 10 behaviour
defects (9 minor, 1 major), 4 needs-harness, 2 oracle defects.**

All 10 of the 26.1 Batch 8 items reproduce in 26.1.1, and all 3
1.21.7 items with them — the particle core is unchanged from 26.1,
and vanilla changed nothing in scope (all 28 relevant classes
byte-identical). The new work in this batch is the tree diff, and it
paid: five 26.1.1 regressions where the tree reverted 26.1's
deliberate removals (B8-G05–G09), plus one whole-command failure the
26.1 run missed (B8-G10, `/spreadplayers`, major). Two corrections
to the 26.1 record are recorded here (H02: 11 kinds, not 5; the
entity-API attribution). Nothing is UNVERIFIED; the Batch 7-routed
flag is decided (real, minted as B8-G10). The batch is complete and
ready for the coordinator to land.
