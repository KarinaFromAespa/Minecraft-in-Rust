# Port notes: Minecraft 1.20.6

1.20.6 is the twelfth adjacent delta port in the **Components era
(E6, 1.20.5–1.21.11)**, taken from 1.21. It is a cross-protocol
delta: protocol **767 → 766**, world version 3953 → 3839, data
pack 48.0 → 41.0, resource pack 34.0 → 32.0 — so the pin moves
the wire: the 1.21-only server-links and custom-report-details
packets go, the component registry loses `jukebox_playable`,
and the adapter ladder re-bases to the 766 pin. The measured
delta is **431 ranked work items**. The port is the mechanical
layer: the pin is set, the three stale generated tables are
regenerated against 1.20.6's own jar, the other three are
confirmed unchanged, the packet rewire, adapter re-base and
oracle port are done, the verification gates pass, and the
remaining work is sized, ranked and documented. The behaviour
code in this tree is still the 26.3 base's, inherited through
1.21.11, 1.21.10, 1.21.9, 1.21.8, 1.21.7, 1.21.6, 1.21.5,
1.21.4, 1.21.3, 1.21.2, 1.21.1 and 1.21; the port is **in
progress**, not parity-verified.

## Provenance

- Base: the programme's completed 1.21 tree — the `port/1.21`
  branch at 52a79646, which contains the 1.21 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle
  port), the 1.21.1 rewire, re-pin and oracle port, the 1.21
  version bump, the jsr305 jar, the both-forms
  `gen_version_consts.py`, the jsr305 classpath fix in
  `tools/port_regen.py`, and the full ported oracle. No overlay
  was needed. Published 1.21: main@ab2dad6d.
- 1.20.6's facts, from its own version JSON and generated tables:
  protocol **766**, world (data) version **3839**, data pack
  format **41.0**, resource pack format **32.0**, Java **21**,
  stable release, released 2024-04-29.
- Prepare ran clean from the Mojang manifest (2026-11-20):
  mappings: classes **8,206 mapped / 8,206 total (8,179
  renamed)**, methods **72,182 mapped / 72,182 total (55,032
  renamed)**, fields **37,286 mapped / 37,286 total (37,056
  renamed)**, 0 placeholders; datagen indexed 8,206 classes /
  66,542 methods. Nothing Mojang-derived is committed in this
  tree.
- Diff 1.21 → 1.20.6: **431 ranked work items** (rank 1: 10,
  rank 2: 275, rank 3: 118, rank 4: 26, rank 5: 2). Classes
  +97 −160, 22 moved, **1,294 changed (5,121 methods); 1,029
  of the changed or removed ones are ported or proved by
  Horizon**. Registries +3 −7, 11 changed (attribute +0 −9
  with 15 ids shifted; data_component_type −1 with 14
  shifted; item −3 with 208 shifted; sound_event +2 −6 with
  721 shifted); blocks +0 −0, 0 changed, states 26684 →
  26684; items +0 −0, 0 with changed default components;
  command nodes +9 −8.

## The delta: a cross-protocol step down

Rank 1 is the version pin (`version.rs`), the component types
(`jukebox_playable` goes; every component id from 42 up shifts
down by one), the protocol packets (config clientbound 17 →
15, play clientbound 124 → 122, 0 renumbered), the 6 table
regenerations, the save-format step (3953 → 3839) and the
version facts. Rank 2 is 275 harness-proved items, rank 3 is
118 ported-but-unproved items, rank 4 is 26 new-content items,
rank 5 is 2 unported items.

The content change is real but narrow: no block or item
changes, no state-id shifts; the movement is in the registries
(sound events shift heavily), the packet set (two packets
removed per side) and the component numbering.

## Tables: 3 regenerated, 3 verified unchanged

| Generator | Result vs 1.21 |
|---|---|
| `version_consts` | regenerated (the pin: 1.20.6 / 766 / 3839 / 41.0 / 32.0) |
| `packet_ids` | regenerated (+1 −9 — the removed packets) |
| `entity_tracking` | unchanged |
| `lan_command_order` | regenerated (+1 −1) |
| `command_suggestions` | unchanged |
| `vanilla_command_order` | unchanged |

The regen took two runs (commit 0ac07614 on `port/1.20.6`).
Run 1 failed only on `vanilla_command_order` (the
client_probe build OOMed at `-j 2` under 3-stream load);
run 2 retried it at `-j 1` and confirmed it unchanged. A
fresh regen reproduces all six committed tables byte-for-byte
— see `regen-determinism.json`, no overrides.

## Packet rewire

The 1.21-only packets go (commit 9593a64c): `server_links`
in both the configuration and play clientbound packets, and
`custom_report_details` (unreferenced in Rust). The LAN host
stops sending links; the ServerLink *types* stay for game code
(`dialog.rs`, the `server_links` module, `dialog_screen.rs`).
`ids.rs` loses `CUSTOM_REPORT_DETAILS` + `SERVER_LINKS` from
both clientbound enums. No `custom_report_details` references
remain in the candidate's Rust code.

## Adapter re-base to the 766 pin

Commit 79b07708 (`rebase766.py`, modelled on `rebase767.py`):
all 27 adapter files regenerated (13 `v*.rs` + 13 `v*_ids.rs`
+ `pinned_ids.rs`), the SlotDisplay eight-entry versions
rewritten, and `Quirks::PINNED` hand-set for 1.20.6
(`components_before_1_21: true`, `wolf_variant_holder:
false`). Verified before the re-base: the
EntityDataSerializers registration order is identical between
1.20.6 and 1.21.1 (same 31 serializers, same order), and the
entity types are identical too (130 entries, bogged at 11 in
both), so the hand-written serializer runs and the breeze
anchor carry over verbatim.

## First pin below 1.21: the item codec

1.20.6 is the first pin that predates 1.21, and three codec
dispatch sites assumed the pin was 1.21.1+ (commit 387f835e):

- `Component::read` routed attribute modifiers through the
  UUID converter only with an active pre-1.21 session; the
  pin's own wire is the UUID form too, so the read now keys
  off the effective quirks. `read_attributes` /
  `write_attributes` treat no active session as the pin
  itself (identity attribute mapping) instead of dropping
  every modifier.
- `TOOLTIP_FLAGS` / `ADDITIONAL` in `v1_21_4.rs` are re-pinned
  to the 1.20.6 component ids (`jukebox_playable` drops out of
  `TOOLTIP_FLAGS` — the pin has no such component).
- The component constants, `skip.rs`, the component hash, the
  1.20.3 legacy converter, the session/layout/translate/recipe
  fixtures and the 1.20.5/1.21.1/1.21.3/1.21.4/1.21.5
  component fixtures are all re-pinned to the 1.20.6 numbering
  (commits ff4a75de, 406c765f, 141a6ec2, 35e52ffd, cd28e7e1,
  406c0744, 8aa16c67, c9ef7d73, af4466f1, 23ea989d, a19292c4,
  f44bef4b). The 1.20.3 converter also gained a real fix: its
  `attributes()` writer now derives the modifier id from
  UUID+name when the datafixed entry has no `id` (the 1.21
  uuid→id datafix runs at data version 3945, past the 766
  pin's 3839).

Two conversions are lossy through the pin's wire **by
construction**, and the fixtures document it rather than
hiding it: arbitrary modifier ids (the write derives a stable
UUID from the id, and the read falls back to
`minecraft:<uuid>` — vanilla's own 1.21 datafix loses the name
the same way) and food use remainders (the pin's food has no
remainder field). The newer-version round-trips stay
byte-exact.

## Oracle port: 13 files

Commit 8312f6a4. 1.20.6 predates
`ResourceLocation.parse` / `withDefaultNamespace` (use the
constructor), the 1.21 attribute sync split (one dirty set),
UUID-less attribute modifier ids (derive the UUID from the id
string with `nameUUIDFromBytes`, the same convention the Rust
`modifier_uuid` uses), `DamageSource` weapon items and
`isDirect` (constant false, `!isIndirect`), public
`VoxelShape.getCoords` (reflect it) and public
`Entity.getBlockPosBelowThatAffectsMyMovement` (expose it on
the test Actor), and the 1.21.2 model rework
(`BlockElementFace` is a plain class with public final fields).
The equippable rewrite's shearing-sound stand-in is re-pinned
to the pin's `entity.sheep.shear` (1226); the old 1256 was
stale even at the 1.21.1 pin, where 1255 is the skeleton
ambient. `compile-exporters` passes both sides (client 12.3 s,
server 17.9 s).

The harness-time oracles (chunk_generation, command_exec and
the rest) keep their 1.21 forms and fail honestly as oracle
failures on CI, as at 1.21.1 — only the exporters
`compile-exporters` builds are ported.

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (at `-j 2`).
- `cargo test -p horizon-protocol`: **420 pass / 2 fail** (422
  total). The 2 failures are the same pre-existing
  sandbox-network pair in untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope. All 6 real failures from the first run (393
  pass / 29 fail) were fixture re-pins and the three codec
  fixes above.
- `horizon-prepare compile-exporters`: both sides pass (see
  "Oracle port").

## Harness subset: no local profile (skipped, CI to measure)

No local harness profile was measured. The 1.21.6 publish
attempted a local sweep and spent ~4 h 30 min on attempts that
produced no profile (the `fast`-profile horizon-client build
OOMs on this VM's 6 GB under 3-stream load); the programme has
since moved to measuring profiles on CI, and disk quota pressure
(~97G of ~100–103G on 2026-11-20) rules out the local
sweep's multi-GB build trees. The profile is to be measured by
the CI `port-verify` workflow on GitHub runners (16 GB) after
merge.

`status.json` records the harness section as 0/0/0 of 17 with
this note, and the CI run will fill it in.

## Known gaps (mechanical-layer limits, for the behaviour port)

All inherited unchanged from 1.21, plus the two lossy
conversions above:

- **Tooltip hiding is lost writing to 1.21.5+ servers**: the
  marker is dropped by `write_patch`'s `has` filter (no DOWN
  synthesis — the target's `tooltip_display` id differs by
  version and the generated `Ids` struct has no name→id lookup).
- **Per-component show flags don't round-trip through saves**:
  `components_tag` keeps emitting the synthesized 1.21.5-form
  `minecraft:tooltip_display` compound, which a 1.20.6 save
  would carry as an unknown component (vanilla drops unknown
  components on load).
- **`skip_dropped` reads the EitherHolder form unconditionally**
  for the removed components, but 26.1+ sends them as bare
  holders — a 26.1+ stack carrying `damage_type`,
  `chicken/variant` or `provides_trim_material` may misparse.
- **Smithing-trim DOWN-write old form not implemented**: the
  write side always writes the new form (pattern id); a
  pre-1.21.5 guest's recipe displays would need a reverse
  pattern→template-item lookup plus the trim-pattern context on
  the guest path.
- **Text components stay JSON strings in NBT at this pin**
  (the 4290 unflatten fix is a 1.21.5 fix).
- **Equip-on-interact is dropped on the pin wire**: the 1.21.2
  equippable layout has no interact field.
- **The native pick-item hook has no call site at this pin**:
  it exists only so the native mod API generator finds its
  signature.
- **Entity-data PARTICLE path**: an unmapped particle still
  hits the −1/mod-sentinel collision (only LevelParticles
  drops).
- **1.21.2+ damage_resistant is dropped** rather than
  converting to the pin's fire_resistant unit.
- **1.21.2+ consume effects are LOST** when a newer server's
  food is expanded to the pin form (the older_layout FOOD arm
  uses defaults); instrument durability/description are dropped
  by the Up conversion; a 1.21.11 tool with creative=false
  cannot hash to its 1.21.11 value at this pin; predicates with
  non-empty matchers always resync (the pin cannot hash
  matchers it dropped).
- **Recipe-book sync gap** (1.21.1 single `recipe` packet vs
  split packets), bundle-local selection, tick-end-per-server-tick
  — documented behaviour gaps.
- **Blockstate binding overlap**: vanilla 1.20.6 assigns the
  missing model to a state claimed by two variants; the Rust
  `bind.rs` keeps the newer variant (1.21.4+ semantics). The
  oracle reports vanilla; the harness will flag the gap.
- **Cuboid light emission**: 1.20.6's parser ignores
  `light_emission` (no such field); the Rust parser rejects
  out-of-range values (1.21.2+ validation). The oracle reports
  light 0 for every element.

## What remains, by cause

### Behaviour work from the diff (not started)

The 431 ranked items are the port's work list; **0 of 431 are
closed**. The work is the save-format step (3953 → 3839), the
275 rank-2 harness-proved items, the 118 rank-3 items, the 26
rank-4 new-content items and the 2 rank-5 items — unattempted.
The behaviour code in this tree is still the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21
unchanged — the same oracle failures as 1.21 are expected once
the profile is measured, but the counts are unmeasured for
1.20.6.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was skipped (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Status

Pin set (protocol 766, world 3839); 6 of 6 tables accounted
for (3 regenerated, 3 verified unchanged); packet rewire done
(server_links + custom_report_details removed); adapter ladder
re-based to the 766 pin; item codec re-pinned and taught the
pin predates 1.21; oracles ported to the 1.20.6 APIs (13 files);
workspace compiles (0 errors, 0 warnings); protocol tests 420
pass / 2 fail (the sandbox-network pair); exporters compile on
both sides. Harness profile: **not measured locally** (skipped
— CI to measure). D6 deferred (owner).

End main island (worldgen-verify): vanilla 1.20.6's
`minecraft:end_islands` includes the central island cone in the
function itself, but the tree carried the 26.3 outer-only sampler
and did not decode the `end_islands` name at all, so the End
pipeline failed to build. Carried 26.2's fix:
`GraphNode::EndIslands { central }`, with `end_islands`
decoding to central: true and `end_outer_islands` to central:
false; the cone formula was verified against the 1.21.11 named
jar's EndIslandDensityFunction bytecode. The end-to-end worldgen
test (`prepared_world_builds_and_generates_chunks`, run by the
worldgen-verify CI job) now generates two chunks per dimension
and dumps them as JSON; it caught this gap.
