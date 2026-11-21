# Port notes: Minecraft 1.21.1

1.21.1 is the tenth adjacent delta port in the **Components era
(E6, 1.20.5–1.21.11)**, taken from 1.21.2. It is a
cross-protocol delta: protocol **768 → 767**, world version
4080 → 3955, data pack 48.0 and resource pack 34.0 — so the
pin moves, the packet set is rewired, the component ids are
re-pinned to 1.21.1's registry, the adapter ladder is re-based
to the 767 pin, and the parity oracles are ported to the
1.21.1 APIs (the 1.21.2 tag, model and registry reworks all
postdate this release). The measured delta is **633 ranked
work items**. The port is the mechanical layer: the pin is
set, all six generated tables are regenerated against
1.21.1's own jar, the verification gates pass, and the
remaining work is sized, ranked and documented. The behaviour
code in this tree is still the 26.3 base's, inherited through
1.21.11, 1.21.10, 1.21.9, 1.21.8, 1.21.7, 1.21.6, 1.21.5,
1.21.4, 1.21.3 and 1.21.2; the port is **in progress**, not
parity-verified.

## Provenance

- Base: the programme's completed 1.21.2 tree — the `port/1.21.2`
  branch at 3f5956e, which contains the 1.21.2 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle port),
  the jsr305 jar, the both-forms `gen_version_consts.py`, the
  jsr305 classpath fix in `tools/port_regen.py`, and the full
  ported oracle. No overlay was needed. Published 1.21.2:
  main@ead0c430.
- 1.21.1's facts, from its own version JSON and generated tables:
  protocol **767**, world (data) version **3955**, data pack
  format **48.0**, resource pack format **34.0**, Java **21**,
  stable release, released 2024-08-08.
- Prepare ran clean from the Mojang manifest (2026-11-19):
  mappings: classes **8,269 mapped / 8,269 total (8,243
  renamed)**, methods **73,432 mapped / 73,432 total (55,951
  renamed)**, fields **37,906 mapped / 37,906 total (37,678
  renamed)**, 0 placeholders; named jar 26,836,906 bytes;
  datagen ran. Nothing Mojang-derived is committed in this tree.
- Diff 1.21.2 → 1.21.1: **633 ranked work items** (rank 1: 10,
  rank 2: 407, rank 3: 163, rank 4: 51, rank 5: 2). Classes
  +172 −628, 11 moved, **2,259 changed; 2,033 of the changed
  or removed ones are ported or proved by Horizon**.
  Registries +2 −4, 16 changed; blocks +0 −24, 4 changed,
  states 27318 → 26684; items +0 −0; command nodes +11 −23,
  10 changed; data files +9 −355, 2,659 changed; assets +166
  −408, 157 changed; en_us keys +91 −182.

## The delta: a cross-protocol step down

Rank 1 is the version pin (`version.rs`), the component
types (+1 −11 on the wire and in saves), the protocol
(login/clientbound +1 −1, play/clientbound +2 −9 with 90
renumbered, play/serverbound +0 −2 with 56 renumbered), the
six table regenerations, the save-format item (DataFixers
4080 → 3955: 0 schemas, 1 fix added, 13 datafix classes
changed) and the version facts (protocol 768 → 767, world
4080 → 3955). Rank 2 is 407 harness reruns. Rank 3 is 163
items, rank 4 is 51, rank 5 is 2 unported classes.

The content change is real: the pale oak family and the
creaking heart (24 blocks, all 1.21.2 additions) go, four
registries go (`consume_effect_type`, `recipe_book_category`,
`recipe_display`, `slot_display`), two registries arrive
(`armor_material`, `instrument` — both data-driven in
1.21.2, code-registered here), and 1.21.1's attributes still
use the `generic.*` names (the rename to bare names is
1.21.2's).

## Tables: 6 regenerated

| Generator | Result vs 1.21.2 |
|---|---|
| `version_consts` | regenerated (+8 −8 — the pin: 1.21.1 / 767 / 3955 / 48.0 / 34.0) |
| `packet_ids` | regenerated (+153 −171 — the rewired packet set) |
| `entity_tracking` | regenerated (+2 −22 — boat/chest_boat/potion map to substitutes) |
| `lan_command_order` | regenerated (+4 −6) |
| `command_suggestions` | regenerated (+34 −0) |
| `vanilla_command_order` | regenerated (+0 −1) |

The regen took three runs (commit a6e5ead on `port/1.21.1`).
Run 1 regenerated the first three tables and failed the three
command-order generators (the client_probe build OOM-killed
at `-j 2` under 3-stream load, then the oracle compile).
Runs 2–3 at `-j 1` completed them. A fresh regen reproduces
all six committed tables byte-for-byte — see
`regen-determinism.json`, no overrides.

## Packet rewire

The 1.21.2 packets go and the 1.21.1 forms return
(6ecf7b7, f8d7777, 6b6023e, c3ab5d6, 248ef73, 613e1e9):
login `login_finished` → `game_profile` (profile + bool,
no session id); the three recipe-book split packets →
the single `recipe` packet; `entity_position_sync`,
`move_minecart_along_track` and `player_rotation` go;
`set_cursor_item`/`set_held_slot` → `set_carried_item`;
teleports, negative windows and the missing book sync
are rewired in the game, the LAN host and the bots. The
tree also fixes a latent bug all E6 trees carry:
`LoginFinished { profile, session: u128 }` is wrong for
their native wire (profile-only 1.21.2–1.21.11) — this
tree uses GameProfile + bool.

## Component re-pin

The component ids are re-pinned to 1.21.1's 57-kind
registry (9bba749): `fire_resistant` (21) is in;
`item_model`, `consumable`, `use_remainder`,
`use_cooldown`, `damage_resistant`, `enchantable`,
`equippable`, `repairable`, `glider`, `tooltip_style`
and `death_protection` are out (negative markers). The
internal form is the pin form for food (6-field combined),
instrument (3-field, use-duration in ticks) and tool
(no trailing bool), and the newest form for attributes
(the established v1_21_5 pattern). The hash modules are
re-pinned the same way (506409e), and three real bugs the
re-pin exposed are fixed (9d95d15): brick was still
1.21.3's id (945, the 1.21.1 brick is 921), `skip_dropped`
read damage_resistant as a holder set (it is a TagKey, a
bare string), and `skip_dropped` read equippable in its
1.21.6+ form (the skip is now session-aware). The 1.20.3
and 1.20.5 legacy fixtures are re-pinned to the 767 pin
(e39de62, 5d4b74c), including the pre-1.21 food remainder
and UUID/name attribute conversions, which were previously
unreachable at this pin (the v1_21_1 step is inert when
the pin IS 1.21.1).

## Adapter re-base

The adapter ladder is re-based to the 767 pin (a8c120a):
the 1.20.6 → 1.21.1 tables (ITEM_UP, DATA_COMPONENT_TYPE_UP,
SOUND_EVENT_UP, ATTRIBUTE_UP) and the session, chunk and
entity tests are re-pinned. Zero entities share an id
between 1.21.1 and 26.2; boat, chest_boat and potion map
to substitutes (oak_boat, oak_chest_boat, splash_potion),
not drops.

## Oracle port

The parity oracles need a full 1.21.2 → 1.21.1 API port
(683c529, 2f50978, adbcd59): 66 javac errors across 19
files, plus one runtime failure. The 1.21.2 tag rework
is undone (`Registry.get` for `getValue`,
`getHolderOrThrow`, single-arg TagLoader lookups,
`MappedRegistry.bindTags` — no `LoadResult` or
`prepareTagReload`); events use `getLocation()`;
solidity, light and offset queries take
`(BlockGetter, BlockPos)`; `shouldRenderFace` takes its
5-arg form over a two-block getter; the Level/GameRules
forms are the 1.21.1 ones (no-arg GameRules, 9-arg Level
ctor, `Explosion` from `explode`, `getRecipeManager`
instead of `RecipeAccess`, no `FuelValues`,
`updateEntityAfterFallOn`). The blockstate binding
oracle replays `BlockStateModelLoader`'s 1.21.1 selection
(multipart first, partial variant predicates, the missing
model on variant overlap) since `instantiate` /
`UnbakedBlockStateModel` do not exist yet; the
simple-block oracle builds a `RegistryAccess` for its
`RegistryOps` since a bare `Registry` is not a
`RegistryLookup`; the entity model exporter reads the
private `Cube.polygons` and package-private
`Polygon`/`Vertex` fields reflectively; cuboid elements
report light 0 (element light emission arrived in
1.21.2); the occlusion-shape lookup takes
`(state, level, pos)`. `compile-exporters` passes both
sides (client 21.5 s, server 23.0 s).

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (at `-j 1`, with `CARGO_INCREMENTAL=0`).
- `cargo test -p horizon-protocol`: **420 pass / 2 fail** (422
  total). The 2 failures are the same pre-existing
  sandbox-network pair in untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope.
- `horizon-prepare` completes end-to-end: exit 0, "Done in
  428.4 s".

## Harness subset: no local profile (skipped, CI to measure)

No local harness profile was measured. The 1.21.6 publish
attempted a local sweep and spent ~4 h 30 min on attempts that
produced no profile (the `fast`-profile horizon-client build
OOMs on this VM's 6 GB under 3-stream load); the programme has
since moved to measuring profiles on CI, and disk quota pressure
(~98–100G of ~100–103G on 2026-11-19) rules out the local
sweep's multi-GB build trees. The profile is to be measured by
the CI `port-verify` workflow on GitHub runners (16 GB) after
merge.

`status.json` records the harness section as 0/0/0 of 17 with
this note, and the CI run will fill it in.

## Known gaps (mechanical-layer limits, for the behaviour port)

Inherited from 1.21.2, plus the new 1.21.1 findings:

- **Tooltip hiding is lost writing to 1.21.5+ servers**: the
  marker is dropped by `write_patch`'s `has` filter (no DOWN
  synthesis — the target's `tooltip_display` id differs by
  version and the generated `Ids` struct has no name→id lookup).
- **Per-component show flags don't round-trip through saves**:
  `components_tag` keeps emitting the synthesized 1.21.5-form
  `minecraft:tooltip_display` compound, which a 1.21.1 save
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
- **Blockstate binding overlap**: vanilla 1.21.1 assigns the
  missing model to a state claimed by two variants; the Rust
  `bind.rs` keeps the newer variant (1.21.4+ semantics). The
  oracle reports vanilla; the harness will flag the gap.
- **Cuboid light emission**: 1.21.1's parser ignores
  `light_emission` (no such field); the Rust parser rejects
  out-of-range values (1.21.2+ validation). The oracle reports
  light 0 for every element.

## What remains, by cause

### Behaviour work from the diff (not started)

The 633 ranked items are the port's work list; **0 of 633 are
closed**. The work is the save-format step (4080 → 3955: 0
schemas, 1 fix, 13 datafix classes), the component-type and
protocol items, the 407 harness reruns, and the 163 + 51 + 2
rank-3/4/5 items — unattempted. The behaviour code in this
tree is still the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from
1.21.2 — the same oracle failures as 1.21.2 are expected
once the profile is measured, but the counts are unmeasured
for 1.21.1.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was skipped (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Status

Pin set (protocol 767, world 3955); 6 of 6 tables regenerated;
packet rewire, component re-pin, adapter re-base and oracle
port done; workspace compiles (0 errors, 0 warnings);
protocol tests 420 pass / 2 fail (the sandbox-network pair);
prepare verified ok end-to-end. Harness profile: **not
measured locally** (skipped — CI to measure). D6 deferred
(owner).

End main island (worldgen-verify): vanilla 1.21.1's
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
