# Batch 8 — Particles (1.21.7 parity sweep)

**Version:** 1.21.7 · **Date:** 2026-11-22 · **Branch:** `parity-sweep` (workdir
`~/workspace/mcir-work/sweep-batch-8`, tip `4259cdb9`) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 8 — all 114 particle types, emitters,
providers; 9 drivers claimed by the plan, 8 found in the tree (see §5).

**Status: DRAFT — not committed.** An execution-risk notice on the setup step
blocks running, building, testing, or issuing git commands inside the repo
without the user's go-ahead (Batch 2 precedent), so no driver runs, no cargo
tests, and no commit/push happened in this batch. Everything below is static
analysis: `javap -c` on the named 1.21.7 client jar
(`stream3-e6-quick/jars/1.21.7-client-named.jar`) and server jar
(`leakage-audit/jars/1.21.7-server-named.jar`), direct class-file parsing
where `javap` was too slow under CPU throttle, source comparison against
`versions/1.21.7/crates/`, and data diffs against the prepared 1.21.7 data
(`~/workspace/mcir-work/mob-spawn/resources/`). The coordinator needs the
user's go-ahead to run the drivers/tests and to commit these deliverables to
`docs/sweep/batch-8-particles.md` and `pins/1.21.7/sweep/batch-08.json`.

**Restart recovery (second re-spawn):** the previous agent's setup work was
verified intact before any new work — clean tree, HEAD `4259cdb9`, with the
two inherited commits on top of `fb9b3daf`:
`c1f1428a` (repair of the six particle oracles) and `4259cdb9` (driver/probe
adaptation). Both are already on `origin/parity-sweep` (verified by fetch:
origin tip is `4259cdb9`), so nothing is at risk. No production code was
touched in this batch; no new commits were made (blocked — see above).

## Tally

- **Behaviour checks: 261 — 257 pass, 4 gap** (across 3 defect items,
  B8-G01–B8-G03, all minor).
- **Oracle defects: 0 new.** 6 oracle repairs inherited from the previous
  agent (`c1f1428a`); 1 compile-verified from its run log
  (VanillaItemParticlesOracle — see §4), the other 5 compile-unverified in
  this run because the execution-risk notice blocks `javac`.
- **Drivers: 0 run, 8 inventoried** (7 particle drivers + 1 particle-adjacent
  effect-sync driver). 2 of the 8 would fail on run for identified
  jar-verified reasons (B8-G01 blocks particle_providers; B8-G03 blocks
  marker_particles); the other 6 are unmeasured, not passing.
- **Unit tests:** 31 `#[test]`s in `particles.rs` exist (23 vanilla + 8
  mod-particle); unrun, blocked.

## 1. Defects

### B8-G01 — `elder_guardian` has no port implementation at all (MINOR, NEW, driver-blocking)

**Checks failed (2):** per-type end-to-end coverage for `elder_guardian`;
its `override_limiter` flag.

**Evidence (jar).** Vanilla 1.21.7 registers `ELDER_GUARDIAN` as a
`SimpleParticleType` (`ParticleTypes.<clinit>`, wire id 16,
`overrideLimiter=true`) and registers a provider for it in
`ParticleEngine.registerProviders()` — `MobAppearanceParticle$Provider`
(javap: `registerProviders` offset 281). There is no
`ElderGuardianParticle` class in 1.21.7; the provider is the shared
mob-appearance one. `ClientPacketListener.handleGameEvent` spawns it at the
player's position on game event 10 (`GUARDIAN_ELDER_EFFECT`), plus the curse
sound.

**The port.** `ParticleType` (`horizon-client-core/src/particle_type.rs`)
has 127 variants and no `ElderGuardian`. Consequences, all verified in
source:

- `received_particle` (`game/lan_particles.rs`) resolves a server-sent
  particle by name and returns `None` for `elder_guardian` — a
  `LevelParticles` packet for it is silently dropped.
- `/particle minecraft:elder_guardian` fails with `particle.notFound`
  (`game/commands/world_cmds.rs:761`, `from_name` returns `None`); vanilla
  accepts it.
- `override_limiter()` has no entry for it (28 vanilla limiter types vs 31
  Rust entries; the only 1.21.7 miss is `elder_guardian` — the 4 Rust
  extras are dead 26.2 geyser entries).
- The providers driver sends 12 `elder_guardian` cases (in `SIMPLE`); the
  probe's `options()` returns `None` for them and prints `no provider`
  while the oracle produces real particle output — **the driver cannot
  pass until this is fixed**.

**Mitigation (why minor).** The one in-game path that matters — the elder
guardian's curse apparition — is re-implemented outside the particle engine:
game event 10 sets `elder_apparition` (`game/remote.rs:4990`,
`game/creatures.rs:4175`) and `elder_apparition_geometry`
(`game/mob_blocks.rs`) draws the ghostly guardian in front of the camera
with the vanilla 30-tick lifetime. So the apparition works; what is missing
is the particle-type plumbing (`/particle`, server packets, the driver).
Visual fidelity of the re-implementation against vanilla's
`MobAppearanceParticle` is unverified statically — needs the driver run or
an in-game look.

### B8-G02 — `/particle minecraft:flash` sends a malformed packet to LAN guests (MINOR, NEW)

**Checks failed (1):** the `Flash` arm of `hub_particle`.

**Evidence.** `Particle::read` (`horizon-protocol/src/entity_data.rs`)
decodes wire id 41 (`flash`, verified against the 1.21.7 registry) as
`Color(i32)` — a flash particle on the wire always carries a colour. But
`hub_particle` (`game/lan_particles.rs`), which builds the packet a LAN host
sends, has no `Flash` arm in its `Color` match (`EntityEffect |
TintedLeaves` only), so a flash particle is built with `data: None`, and
`Particle::write` — which writes data by presence, not by id — emits the id
with no colour bytes. Any guest reading id 41 then consumes the next 4 bytes
of the stream as the colour and continues misaligned: packet desync (and
likely a disconnect or a dropped connection) for every guest in range.

**Reachability (narrow — why minor).** The only path that sends `Flash`
through `hub_particle` is `/particle minecraft:flash`: `cmd_particle`
builds `ParticleOptions::simple(Flash)` (no colour — vanilla's command
requires the colour option, the port does not) and calls
`network_particle`. Everywhere else `Flash` is created client-side only
(the firework explosion flash, `particles.rs:3777`), never sent over the
network. The local player is unaffected (draws directly, never reads the
packet). So the trigger is exactly: a LAN host runs `/particle
minecraft:flash` with guests in range.

### B8-G03 — Spruce leaves never drop falling-leaf particles (MINOR, NEW, driver-blocking)

**Checks failed (1):** the spruce entry of the per-block leaves check.

**Evidence (jar).** Vanilla 1.21.7 registers `SPRUCE_LEAVES` as a
`TintedParticleLeavesBlock` with `leafParticleChance = 0.01` — verified by
parsing `Blocks.class` directly: the `BootstrapMethods` table maps the
registration lambda `lambda$static$36` to spruce, and its construction is
`TintedParticleLeavesBlock(0.01F, …)`. `LeavesBlock.animateTick` calls
`makeFallingLeavesParticles`, which rolls `nextFloat() < leafParticleChance`
and spawns the falling leaf unless the block below is face-full (javap).
So vanilla spruce drops tinted leaves at 1% per animateTick, like oak.

**The port.** `classify` (`animate_tick.rs:151`) maps `"spruce_leaves"` to
`Animate::Leaves(None, None, false)` — no falling particle at all. Every
other leaf block matches vanilla (see §3, leaves table).

**Driver note.** The marker driver's own zoo comment says "the spruce that
never falls" — the comment encodes the same wrong belief, but the driver
compares against the real vanilla oracle, so the driver is correct as a
check and **would fail on the spruce cases** (oracle emits tinted leaves,
probe emits none). The comment needs correcting when the fix lands; the
driver needs no change.

## 2. Cross-references (not re-minted)

- **G1 (pilot) — bubble columns never form.** The client-side animate arm
  for bubble columns exists and is correct (`Animate::BubbleColumn`,
  `animate_tick.rs:1660` — `BubbleColumnUp` particles + whirlpool/upwards
  ambient sounds), but it can never trigger in-game because no
  `bubble_column` block states ever form server-side (G1). No new defect:
  the particle-visible consequence of G1, not a particle defect.
- **G6 (pilot) — dripstone never transfers fluid to cauldrons.** Server-side
  only. The *client-side* drip particles are unaffected: vanilla 1.21.7's
  `PointedDripstoneBlock.animateTick` uses a hardcoded
  `Blocks.POINTED_DRIPSTONE` check via `getFluidAboveStalactite` (javap —
  no `SPELEOTHEMS` reference anywhere in the class), and the port's
  `dripstone_drip` (`animate_tick.rs:1807`) matches it by name, with the
  same 0.12 roll and 0.02 cauldron roll. The marker driver's scope note
  about binding "the speleothem tag the stalactite branch needs" is
  harmless: the tag does not exist in either 1.21.7 jar (verified by jar
  listing), so binding yields empty, matching vanilla.
- **B10-G05 — first-person probe swap-ticker columns.** No overlap: no
  particle driver touches the first-person probe or oracle. Noted, no
  action.
- **B12-G04 — 26.x-only effect in the MobEffects table.** Same table-leak
  *pattern* as the 14 extra 26.x particle types in the Rust `ParticleType`
  enum (see §3), but the particle extras are unreachable — ids are
  name-resolved from the server's registry, and a 1.21.7 server never sends
  those names — so they are not minted as a defect. Dead table entries,
  not a behaviour gap.

## 3. What passed (per area)

### Registry and type coverage (117 checks: 116 pass, 1 gap)

- 114 particle types: `registries.json` count == jar `ParticleTypes`
  `<clinit>` registration count (114), and the `<clinit>` order equals the
  `protocol_id` of every type (verified by parsing the bytecode).
- Providers-driver kind set == registry exactly (114/114): the driver's
  `SIMPLE` list + 18 optioned kinds cover every 1.21.7 type, no more, no
  fewer.
- Engine `make()` (`particles.rs:2266`) has an outer match arm for all 127
  `ParticleType` variants (no catch-all) — every enum variant has a
  provider implementation. The only engine-coverage gap is the missing
  `elder_guardian` variant itself (B8-G01).
- The 14 Rust-enum extras (`copper_fire_flame`, the geyser family,
  `noxious_gas`, `noxious_gas_cloud`, the three poplar leaves,
  `pause_mob_growth`, `reset_mob_growth`, `sulfur_bubbles`,
  `sulfur_cube_goo`) are absent from the 1.21.7 jar's `ParticleTypes` —
  confirmed 26.x-only, unreachable from a 1.21.7 server.

### Wire codec (20 checks: 19 pass, 1 gap)

- All 18 data-carrying particle wire ids match `Particle::read`'s hardcoded
  ids exactly: State 1/2/28/108/112 (block, block_marker, falling_dust,
  dust_pillar, block_crumble), Color 20/35/41 (entity_effect,
  tinted_leaves, flash), dust 13, dust_color_transition 14, effect 15,
  instant_effect 45, dragon_breath 7, sculk_charge 37, shriek 102, trail
  48, item 46, vibration 47.
- `received_particle` resolves ids to names through the server-sent
  registry (not a hardcoded table) — correct for all 113 implemented types.
- Gap: B8-G02 (Flash write path).

### Override limiter (28 checks: 27 pass, 1 gap)

- Vanilla limiter-true set (28 types, from `<clinit>` booleans) vs the
  Rust `override_limiter()` list: identical except `elder_guardian`
  (B8-G01). The 4 Rust extras are the dead 26.2 geyser entries.

### Entity event bursts (25 checks: 25 pass)

- Every (entity, event) pair in the entity driver's `EVENTS` table is
  covered by the port's `entity_event_burst` (`particles.rs:620`):
  villager 12/13/14/42, cow 18, wolf/cat 6/7/18, parrot 6/7, horse/llama/
  camel 6/7/18, ocelot 40/41/18, dolphin 38, allay 18 — with the right
  particle kinds (heart, angry_villager, happy_villager, splash, smoke).
  Counts/positions/RNG draws need the driver run; the mapping table is
  exact. The port's table also carries dead 26.x arms (`nautilus`,
  `zombie_nautilus`, `camel_husk`) — unreachable, harmless.

### Marker zoo (43 checks: 43 pass)

- Every block name in the marker driver's zoo + neighbour + campfire cases
  (43 names) has a `classify` arm in `animate_tick.rs`, including the
  guarded ones (lit furnaces, powered levers/repeaters, honey-level-5
  hives, charged respawn anchors, active sculk sensors, down-pointing
  dripstone tips). The 26.x arms in the table (`copper_torch`,
  `potent_sulfur`, poplar leaves) are unreachable in 1.21.7 — the table is
  keyed by registry name, so blocks that do not exist in 1.21.7 never map.
  Positions/RNG draws need the driver run; coverage is complete.

### Leaves (12 checks: 11 pass, 1 gap)

Per-block `leafParticleChance` and class, verified by parsing `Blocks.class`
(`BootstrapMethods` → registration lambda → construction) against the port's
`classify` table:

| Block | Vanilla 1.21.7 | Port | Verdict |
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
  equivalence needs the driver run.)
- Note: vanilla 1.21.7 has no `CherryLeavesBlock`/`PaleOakLeavesBlock`
  classes — cherry/pale-oak are `UntintedParticleLeavesBlock` instances,
  and the port's `FallingParticlesLeavesBlock` comment names the 26.x
  class; the mechanism is the 1.21.7 one.

### Dripstone and tags (4 checks: 4 pass)

- `speleothems`, `washed_away_by_fluids`, `blocks_lava_fire_spread` are all
  absent from both 1.21.7 jars (jar listing) — confirms the pilot's tag
  inventory; no new tag finding.
- Vanilla dripstone animateTick is tag-free (hardcoded
  `POINTED_DRIPSTONE`); the port matches by name.

### Oracle API existence (8 checks: 8 pass)

All vanilla methods the repaired oracles call exist in the 1.21.7 jars with
the expected signatures (javap/`strings` on the class files):
`ParticleUtils.spawnParticlesOnBlockFaces` /
`spawnParticlesAlongAxis` (emitter), `LivingEntity.spawnItemParticles`
(item), `Villager.handleEntityEvent` (entity),
`CampfireBlock.animateTick` / `makeParticles` /
`CampfireBlockEntity.particleTick` (marker), `ClientLevel.animateTick`
(marker loop). This is existence only — the oracles' *use* of them is what
the drivers verify.

### Other passes

- Engine `make()` exhaustiveness (see above).
- Item oracle compiles — evidence from the previous agent's run log (§4).
- Game-event-10 apparition re-implementation present (§1, B8-G01).
- Bubble-column animate arm present (§2, G1).

## 4. Oracle repairs and defects

**No new oracle defects.** Batch 0's pre-repair table (from
`docs/sweep/batch-0-oracles.json`) had all six particle oracles failing on
all classpaths: Block 41 errors, Emitter 19, Entity 21, Item 3, Marker 42,
Providers 116 (mostly missing 26.x packages on the 1.21.7 classpath).
`VanillaEffectParticlesOracle` passed Batch 0 on all classpaths.

The previous agent repaired all six in `c1f1428a` (test-only Java,
committed separately, pushed) and adapted the drivers/probes in `4259cdb9`.
Compile verification in *this* run:

- **VanillaItemParticlesOracle — compiles.** Evidence from the previous
  agent's own run, in the workdir's `.local`: `rust-item-particles-parity/
  compile-java.log` is empty (javac exit 0) and `inputs.txt` (2.5 MB) was
  written after it, so compilation succeeded on 2026-11-22 ~11:18 NPT. The
  driver then died in the *Rust* step: the release build of the
  `world_probe` example timed out at the driver's 900 s limit
  (`parity-runner/logs/item_particles.log`) — an environment/load failure
  under 3-stream load, not a code failure. No comparison ever ran.
- The other five repaired oracles (Block, Emitter, Entity, Marker,
  Providers) are **compile-unverified in this run**: the execution-risk
  notice blocks `javac`, and the oracle sources are not in this workdir's
  sparse checkout (`mods/` is not checked out), so they could not even be
  re-read statically — only their commit (`c1f1428a`, verified via
  `git show --stat` before the notice) attests to them. First thing to do
  when execution resumes: compile all six and record the result; the
  previous agent's repairs were javap-verified against the 1.21.7 jar per
  its brief, so the expectation is green, but it is not yet evidence.

## 5. Driver inventory

The plan (§4) claims 9 particle drivers; the tree has 7 particle-named
drivers plus 1 particle-adjacent driver = **8**. The 9th is unaccounted for:
possibly the plan counted `tests/test_cushion_particles.py` (a Python unit
test for Horizon's cushion feature, not a parity driver) or double-counted.
No 9th parity driver exists under any name in `tools/` (grep over all
`test_*_parity.py` for particle-oracle references finds only these).

| Driver | Oracle (Batch 0 → now) | Probe | Status |
|---|---|---|---|
| block_particles | BlockParticles: 41e → repaired `c1f1428a` | block_particles_fixture.rs | UNRUN (blocked) |
| emitter_particles | EmitterParticles: 19e → repaired | emitter_particles_probe.rs | UNRUN (blocked) |
| entity_particles | EntityParticles: 21e → repaired | entity_particles_probe.rs | UNRUN (blocked) |
| item_particles | ItemParticles: 3e → repaired; **compiles** (log evidence) | item_particles_fixture.rs (horizon-world) | UNRUN — previous run died on the 900 s cargo build timeout |
| marker_particles | MarkerParticles: 42e → repaired | marker_particles_probe.rs (adapted `4259cdb9`) | UNRUN (blocked); would FAIL on spruce cases (B8-G03) |
| native_client_particles | none (Horizon-only Host API contract, no vanilla oracle) | game fixture | UNRUN (blocked); out of vanilla scope |
| particle_providers | ParticleProviders: 116e → repaired | particle_providers_probe.rs (adapted `4259cdb9`) | UNRUN (blocked); would FAIL on elder_guardian cases (B8-G01) |
| effect_sync (adjacent) | EffectParticles: **pass** (Batch 0); EffectSync oracle separate | effect_sync_fixture.rs | UNRUN (blocked); scope is effect particle/visibility sync |

`parity.py` was not run (blocked). The `.local/minecraft/1.21.7/`
prerequisite the pilot flagged (`verify_artifacts()` gate) is still absent
from this workdir — the previous agent's item-particles run got past it
(the log shows the driver reached the cargo step), so the gate is
satisfiable here; the run died on build time, not on artifacts.

## 6. Unit tests

31 `#[test]`s in `crates/horizon-client-render/src/particles.rs` (lines
5202–6089), unrun (blocked):

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

## 7. Not covered

- **All driver numerics.** Per-tick provider physics (positions, motion,
  age, lifetime, size, colour, roll, sprite, UVs, light), emitter RNG
  draws, entity-burst counts/positions, block-particle placement, marker
  loop exactness — only the drivers can verify these, and none ran. The
  static checks above bound the *tables and mappings*, not the numerics.
- **Translucent-layer table.** `ParticleType::translucent_layer()` (which
  types vanilla renders in the translucent layer) was not verified — it
  needs per-class `getRenderType` checks across ~20 particle classes, and
  each `javap` invocation runs 10–75 s under this VM's CPU throttle.
- **Sprite sets / particle textures.** Whether the engine resolves every
  type's sprite set from the 1.21.7 assets needs prepared data + a run.
- **Level events other than 3002–3005.** `level_particles.rs` handlers
  (dragon egg, shulker/ender teleport, growth, turtle/sniffer eggs, torch
  burnout, dragon fireball, potion splashes, `level_event_at`) exist but
  their numerics are unverified; only 3002–3005 are in the emitter driver.
- **Apparition visual fidelity.** The elder-guardian re-implementation
  (§1) needs an in-game look or the providers driver.
- **Five repaired oracles' compilation** (§4).
- **The 9th driver.** The plan's count of 9 does not match the tree's 8;
  if a 9th driver exists somewhere else (another version tree, a renamed
  file), it was not found here.

## 8. What unblocks the rest

1. User go-ahead to execute in the repo (lifts the execution-risk block).
2. Compile the six repaired oracles (`javac` via any one driver, or
   directly) and record the result — expected green, unverified.
3. Fix B8-G01 (add the `ElderGuardian` variant + `MobAppearanceParticle`
   provider) and B8-G03 (spruce leaves arm) on fix branches, then run
   `particle_providers` and `marker_particles`; fix B8-G02 (Flash arm in
   `hub_particle`) with them or separately. B8-G01/B8-G03 are
   driver-blocking; B8-G02 is not covered by any driver.
4. Run the remaining drivers (`block`, `emitter`, `entity`, `item`,
   `effect_sync`); the item driver needs a longer cargo-build window than
   its 900 s timeout under load (or a warm target dir).
5. Run the 31 `particles.rs` unit tests (`cargo test -p
   horizon-client-render --lib particles::`, dev profile, `-j 1`).

## 9. Method notes

- The named 1.21.7 **client** jar is at
  `~/workspace/mcir-work/stream3-e6-quick/jars/1.21.7-client-named.jar`
  (the leakage-audit jar dir holds the server jar only); the server jar at
  `~/workspace/mcir-work/leakage-audit/jars/1.21.7-server-named.jar`.
  `javap` on the full jars is very slow here (10–75 s per class); the
  leaves work instead parsed `Blocks.class` directly with a small Python
  class-file reader (constant pool + `BootstrapMethods` + bytecode scans),
  which runs in under a second — recommended for any follow-up bytecode
  association work.
- `javap -c` on `Blocks` exceeds 280 s under throttle (timed out twice);
  do not use it — use the class-file parser.
- The sparse checkout in this workdir (`docs`, `pins`, `tools`,
  `versions/1.21.7`) does not include `mods/`, so the repaired oracle
  sources could not be re-read after the execution-risk notice blocked
  `git show`. The commit contents were verified before the notice.
- No production code (`crates/`) was touched. No commits were made in this
  batch. The two inherited commits are already pushed to
  `origin/parity-sweep`.
