# Port notes: Minecraft 1.21.4

1.21.4 ("The Garden Awakens") is the seventh adjacent delta port in the
**Components era (E6, 1.20.5–1.21.11)**, taken from 1.21.5. It is a
cross-protocol delta: protocol **770 → 769**, world version 4325 →
4189, data pack 71.0 → 61.0 and resource pack 55.0 → 46.0 — and the
packet set changes both ways: 3 test-block packets are removed,
`minecraft:add_experience_orb` comes back, and 117 play clientbound
packets are renumbered, so this port needed a full packet rewire plus
the adapter re-base. The measured delta is **548 ranked work items**.
The port is the mechanical layer: the pin is set, the six generated
tables are regenerated against 1.21.4's own jar, the adapter ladder is
re-based to the new protocol, the component codec is re-pinned to the
1.21.4 registry, the verification gates pass, and the remaining work
is sized, ranked and documented. The behaviour code in this tree is
still the 26.3 base's, inherited through 1.21.11, 1.21.10, 1.21.9,
1.21.8, 1.21.7, 1.21.6 and 1.21.5; the port is **in progress**, not
parity-verified.

## Provenance

- Base: the programme's completed 1.21.5 tree — the `port/1.21.5`
  branch at fd79ef0, which contains the 1.21.5 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle port),
  the jsr305 jar, the both-forms `gen_version_consts.py`, the
  jsr305 classpath fix in `tools/port_regen.py`, and the full
  ported oracle. No overlay was needed. Published 1.21.5:
  main@7ff1eace.
- 1.21.4's facts, from its own version JSON and generated tables:
  protocol **769**, world (data) version **4189**, data pack
  format **61.0**, resource pack format **46.0**, Java **21**,
  stable release (2024-12-03).
- Prepare ran clean from the Mojang manifest (2026-11-19):
  **8,857 classes indexed, 72,101 methods**; mappings: classes
  **8,857 mapped / 8,857 total (8,829 renamed)**, methods
  **78,666 mapped / 78,666 total (59,634 renamed)**, fields
  **40,134 mapped / 40,134 total (39,880 renamed)**,
  0 placeholders; named jar 33,218,199 bytes; datagen ran.
  Nothing Mojang-derived is committed in this tree.
- Diff 1.21.5 → 1.21.4: **548 ranked work items** (rank 1: 10,
  rank 2: 348, rank 3: 149, rank 4: 39, rank 5: 2). Classes +205
  −496, 25 moved, **1,842 changed (9,139 methods); 1,628 of the
  changed or removed ones are ported or proved by Horizon**.
  Data files +17 −113 ~1,435; assets +80 −281 ~147; en_us keys
  +0 −161 ~44.

## The delta: test blocks, garden content and 31 components go away

Rank 1 is the version pin (`version.rs`), the six regenerate
items, the save-format item (DataFixers 4325 → 4189: 0 schemas,
4 fixes added, 93 datafix classes changed) and the version facts
(protocol 770 → 769, world 4325 → 4189). Rank 2 is 348 harness
reruns. Rank 3 is 149 unproved items. Rank 4 is 39 items. Rank 5
is 2 items.

The visible content changes are the removals of 1.21.5 features:

- **Test blocks**: the `test_block` and `test_instance_block`
  blocks (and their block entities, the `test_environment_definition_type`,
  `test_function` and `test_instance_type` registries), the
  `minecraft:set_test_block` / `minecraft:test_instance_block_action` /
  `minecraft:test_instance_block_status` packets.
- **Garden content**: bush, short/tall dry grass, cactus flower,
  wildflowers, leaf litter and firefly bush (blocks and items),
  the `firefly` and `tinted_leaves` particles, and 52 sound
  events (the registry gains 1).
- **Blue and brown eggs** (items 970/971).
- **31 data components**, among them the whole variant family
  (chicken/cow/pig/wolf/fox/frog/horse/llama/cat/parrot/rabbit/
  salmon/tropical-fish/mooshroom/axolotl/villager variants,
  wolf sound variant, wolf/sheep/shulker/cat collars and colours),
  `tooltip_display`, `weapon`, `blocks_attacks`,
  `potion_duration_scale`, `break_sound`, `provides_trim_material`
  and `provides_banner_patterns`. The pin gains
  `hide_additional_tooltip` (15) and `hide_tooltip` (16) in their
  place. The component registry is 67 entries (0–66).
- **Potion entities**: `splash_potion` (99) and `lingering_potion`
  (100) are gone; the single `potion` entity (99) is back.
- **The creaking heart** goes back to a boolean `active` (12
  states, active-major) from 1.21.5's `creaking_heart_state`
  (18 states).

The item id shift rule across the delta: ids <204 unchanged;
205–207 shift −1; 211–258 shift −4; 261–328 shift −6; 330–831
shift −7; 834–969 shift −9; 972+ shift −11. Entity ids above 100
shift −1 (99 is potion in both). Block ids: <133 unchanged;
136–266 shift −3; 268–865 shift −4; 868–1049 shift −6; 1052–1102
shift −8; >1103 shift −9. Attributes are unchanged. That is why
the adapter re-base and the test re-pins below are mechanical
but wide.

## Tables: 6 regenerated

| Generator | Result vs 1.21.5 |
|---|---|
| `version_consts` | regenerated (+8 −8 — the pin: 1.21.4 / 769 / 4189 / 61.0 / 46.0) |
| `packet_ids` | regenerated (+125 −129 — 3 packets removed, 1 added, 117 renumbered) |
| `entity_tracking` | regenerated (+1 −2) |
| `lan_command_order` | regenerated (+7 −86) |
| `command_suggestions` | regenerated (+0 −1) |
| `vanilla_command_order` | regenerated (+4 −1) |

The regen took three runs (commits c70f563, 306672cde4,
b658fe174c on `port/1.21.4`), because each run unblocked the
next: run 1 regenerated the first four tables and failed the two
oracle-command generators — `command_suggestions` on NBT API
errors in the oracles (1.21.4 tags have no `value()`,
`CompoundTag` has no `keySet()`), fixed in 6c88fc6, and
`vanilla_command_order` because `horizon-protocol` would not
compile until the packet rewire landed. Run 2 regenerated
`vanilla_command_order` and failed `command_suggestions` on
`TagParser.parseCompoundFully` (a later addition; 1.21.4 has
only `parseTag`) — fixed in b12fa91 (17 sites in 13 files).
Run 3 regenerated `command_suggestions`. The committed
`regen.json` lists all six generators with their run-1–3 statuses
combined (D1 completeness, per the gate change at main@15ff93a8).
A fresh regen reproduces all six committed tables byte-for-byte —
see `regen-determinism.json`, no overrides.

## Packet rewire: 3 packets removed, 1 restored, 117 renumbered

Commit 65307b5 on `port/1.21.4` removes the 3 test-block packets
(`SetTestBlock`, `TestInstanceBlockAction`,
`TestInstanceBlockStatus`, with `TestInstanceData`) and adds
`AddExperienceOrb` (varint id, 3 doubles, short value — verified
with `javap -c` against the 1.21.4 named jar). Two test-block
tests went with the removed packets; the `test_action` helper in
`step_1_21_6.rs` and the now-empty `translate::v1_21_6` module
went too. An `AddExperienceOrb` round-trip test was added later
(commit b191fd8).

Commit 56e787f reworks the game-side wire boundary around the
missing packets: `remote.rs` drops the `TestInstanceBlockStatus`
arm and gains an `AddExperienceOrb` arm (spawns the orb, then
overrides its value — `spawn_entity` already handles
`minecraft:experience_orb` with value 1); `lan_actions.rs`
drops the test-block arms; `test_blocks.rs` and
`test_instances.rs` drop the LAN send/apply functions; the
test-block editors (`test_block_edit.rs`,
`test_instance_edit.rs`) now apply their values locally and
return a new `Outcome::Close` ("close without sending") instead
of sending packets — `Game::send` is a no-op without a remote
connection, so the packets were LAN-only and local-only edits
are honest. The test-runner operations only the removed apply
function invoked (`save_test`, `export_test`, `run_test`,
`TestDef.description`, `track_and_start_runner`) are removed as
dead code: the packets arrived in 1.21.5 and stay absent down
the chain.

## Adapter re-base to 769

The adapter ladder (`crates/horizon-protocol/src/adapter/`) is
re-based from the 770 pin to 769 (commit 2d3e575 on
`port/1.21.4`; scripts `rebase769.py` and `fix_slot_display769.py`
in the Stream 1 workdir, modelled on the 1.21.5 `rebase770.py`).
The rebase was recovered from diffing the regenerated tables
against the 770 tree:

- INHERIT map: 1.21.3→PIN, 1.21.5→PIN, 1.21.6→PIN, 1.21.8→PIN.
- Potion renames: `splash_potion→potion` added for 1.21.5,
  1.21.6, 1.21.8, 1.21.10, 1.21.11, 26.1.2, 26.2
  (`lingering_potion` has no pin equivalent and drops to NONE);
  the old `potion→splash_potion` entries for 1.20.2/1.20.3/
  1.21.1/1.21.3 removed (the pin now has `potion`).
- Creaking-heart state aliases rewritten for 1.21.4's boolean
  `active` (12 states, active-major): both quiet states
  (dormant, disabled) map to `active=false, natural=false`,
  with dormant first so the generator's DOWN prefers it; the
  v1_21_3 BLOCK_STATE_DOWN patch inserts length-1 runs for the
  natural=true halves (they cannot merge into length-2 runs —
  two pin states share one old target).
- **Entity-data serializer remaps recomputed** (unlike 770/771):
  1.21.5 added 4 variant serializers (cow, wolf sound, pig,
  chicken) at 1.21.5-order ids 23, 25, 27, 28; the pin has 31
  serializers (0–30). V1_21_4's custom remap is removed
  (generated Same); V1_21_8/6/5, V1_21_10/11, V26_2 and V1_20_3
  gained the recomputed remaps; V1_21_3/1/20_6 dropped theirs
  (their order matches the pin).
- `adapter.rs` pin swap: V1_21_4 is now the pin identity static
  (quirks merged: `wolf_variant_holder`, `components_before_1_21_5`,
  `components_before_1_21_6`); V1_21_5 is a regular old version;
  the `from_protocol` test updated (774 five above pin, 773 four,
  772 three, 771 two, 770→V1_21_5, 769→Current).
- SlotDisplay: the `slot_display` registry is identical in 1.21.4
  and 1.21.5 (8 entries, same order), so the 770 fix values apply
  unchanged; the v1_20_3 ENTITY_TYPE_DOWN anchor (breeze_wind_charge
  still 18 in the 769 pin) is identical too.

## Component re-pin: the 1.21.4 registry

The `item::component` constants were still the 1.21.5 pin's ids
(commit 47d621a on `port/1.21.4`). All 67 entries of the 1.21.4
`data_component_type` registry are now pinned (0 custom_data …
66 container_loot), the 31 removed components are gone from the
constants (BLOCKS_ATTACKS, PROVIDES_TRIM_MATERIAL,
PROVIDES_BANNER_PATTERNS, CHICKEN_VARIANT deleted outright),
and `TOOLTIP_DISPLAY` is now an internal marker (−1003): 1.21.4
has no `tooltip_display` component.

**Tooltip design.** The pin expresses tooltip hiding as two Unit
components (`hide_tooltip`, `hide_additional_tooltip`) plus a
`show_in_tooltip` bool at the end of each TooltipProvider
component's codec (verified with `javap`: Unbreakable,
ItemEnchantments ×2, AdventureModePredicate ×2,
ItemAttributeModifiers, DyedItemColor, ArmorTrim, JukeboxPlayable
— the last was missing from the old-server flag list, a latent
bug fixed here). `Component::TooltipDisplay` stays the canonical
in-memory form (the client reads it via `stack.component` and the
NBT form `minecraft:tooltip_display`); the pin codec synthesizes
it at the stack level in `read_stack`/`read_template`/
`WirePatch::decode`, and `write_patch`/`DelimitedSlot::write`
split it back into the two hide booleans plus per-component show
flags. Writing to a 1.21.5+ server drops the hiding — the marker
has no wire id there (and the target's `tooltip_display` id is
15 in 1.21.5–1.21.10 but 18 in 1.21.11+, so it cannot be
hardcoded). See "Known gaps" below.

**`Quirks::PINNED` fixed.** It still said "the pinned 1.21.8's
own wire layouts" with `components_before_1_21_6: false` — stale
since 1.21.8. For the 1.21.4 pin it is now
`components_before_1_21_6: true, components_before_1_21_5: true,
wolf_variant_holder: true`, matching V1_21_4.quirks. (The
published 1.21.5 tree carries the same staleness — flagged for a
fix-up there; the 1.21.6/1.21.7 pins are unaffected, both
post-date 1.21.6.)

**Real code fixes surfaced during the re-pins** (commits
17f3a77, 47d621a, 557782e, b385cce, 9989409, 51307c8, d053132,
f51456c, af3cf6f, 5562ad8, b4a2f78, 14be1b1, d5c1710, db4813d,
b191fd8, ca20060):

- `ARGUMENT_TYPES` in `packets/play/commands.rs`: 55 → 54
  entries (`resource_selector` 47 removed).
- Particle codec re-pinned (real code, `entity_data.rs`): trail
  47, vibration 46, item 45, item_slime 48, shriek 101, sculk
  35/36, instant_effect 44 — and the Item arm now covers both
  `minecraft:item` and `minecraft:item_slime` (the 1.21.5 pin
  only covered item; item_slime fell through to None).
- Entity-data serializer constants re-pinned from the 1.21.4 jar
  (`javap -c` on `EntityDataSerializers.<clinit>`: 31 calls,
  SNIFFER before ARMADILLO); the four 1.21.5 variant serializers
  removed.
- `v1_21_5::write` now gates on the *target's* layout
  (`quirks_written().components_before_1_21_6`), so pin writes
  drop the 1.21.6 display/shearing fields the pin reader cannot
  parse; the `before_5` check is direction-aware (pin reads of
  equippable no longer consume a phantom interact byte).
- `DelimitedSlot::write` mirrors `write_patch`'s tooltip
  synthesis (it previously wrote the −1003 marker raw, which the
  delimited read rejected).
- Smithing-trim old-form read now requires an active UP session
  with `components_before_1_21_5`: the pin codec reads the old
  form only when the translator holds the trim-pattern registry
  (the game decodes pinned bytes with no such context).
- Legacy stacks: `from_tag` parses JSON-string text components
  to NBT (the 4290 unflatten fix does not run at data version
  4189 — see below), synthesizes the tooltip marker from
  `show_in_tooltip: false` fields and HideFlags, and reads
  `dyed_color` from its 1.20.5 compound form; `json_text` no
  longer double-encodes already-JSON strings; `legacy_tag` and
  `legacy_raw` re-pinned.
- The v1_20_3 block-entity oracles were wrong for this pin, not
  the code: the text-component unflatten fix is keyed 4290 (a
  1.21.5 fix), so at 4189 `CustomName` and sign messages stay
  JSON strings in NBT — the oracles' "after" fixtures were
  rewritten to the JSON-string form (commit af3cf6f).
- The shearing-sound placeholder is now `entity.sheep.shear`
  + 1 (1256): 1.21.4 has no `item.shears.snip` (1328 there is
  `block.resin.break`).
- Hash tables (`item/hash/raw.rs`, `gameplay.rs`) re-pinned to
  the 1.21.4 component ids; 23 removed-component cases dropped.
- 71 pinned component fixtures regenerated from the pin codec
  (TEMP-REGEN method); the legacy-slots fixtures likewise (39
  labels, 58 replacements).

The registry shifts forced wide test re-pins: diamond 845→836,
brick 963→954, calcite state 23344→23329, filled map 1042→1031,
tadpole 124→123, turtle 131→130, zombie villager 146→145, player
kind 148→147, angler sherd 1352→1341, bowl 839→830, iron ingot
851→842, emerald 846→837, explode sound 606→602, llama
128→127 (trader), saddle 800→793, shulker box 552→545,
decorated pot 307→301, chest 319→313, chain 378→371, music disc
cat 1232→1221, pigstep 1248→1237, mall 1238→1227, goat horn
1267→1256, beehive 1287→1276, sentry trim 1334→1323, blast
furnace station 1271→1260, beehive block 871→865, decorated pot
block 1091→1083, and the packet-id decode sites (block_update
is 9 pinned, 8 in 26.2; animate is 3 pinned, 2 in 26.2).

## Oracle port: 52 errors, then 2, then the datafix exporter

`horizon-prepare` (commits 76ce485, 556297a, 7d8ef28, 058ad0a)
ports the oracles to the 1.21.4 APIs — verified with `javap`
against the 1.21.4 named jar, not guessed:

- `Player.isCreative()` and `isSpectator()` are **abstract** in
  1.21.4, and `Player` has **no `gameMode()` method** (nor does
  `ServerPlayer` — it has a `gameMode` field). All 40
  `gameMode()` overrides were removed: classes that already had
  isCreative/isSpectator kept them; the rest gained `false`
  pairs; the fixture-driven ones derive from `mode`
  (BlockEntityAnim, HeldDrop, SpawnRules, NaturalSpawn, Raid);
  DamageSource's Human returns `getAbilities().instabuild`.
- `Entity.causeFallDamage` and `Block.fallOn` take a **float**
  distance in 1.21.4 (8 overrides + the fallOn call and its
  reflective lookup in VanillaBlockLandingOracle).
- `Level.getFreeMapId()` and `setMapData()` are abstract;
  `playSeededSound` takes a **Player** first parameter;
  `LevelAccessor.levelEvent` is `levelEvent(Player, int,
  BlockPos, int)`.
- `CompoundTag` has `getString` but no `getStringOr`;
  `VoxelShape.move` takes a Vec3 (or three doubles), not a
  BlockPos.
- **Model oracles** (VanillaCuboidModelOracle,
  VanillaBlockStateBindingOracle): 1.21.4 predates the 1.21.5
  model refactor — no `BlockStateModel`, `MultiPartModel`,
  `BlockStateDefinitions`, `SimpleUnbakedGeometry`, or
  `BlockModelDefinition.CODEC`. Ported to the pre-refactor API:
  `BlockElement`/`ItemTransform` public fields, `BlockFaceUV`
  (rotation in degrees on the uv object), `BlockModel`
  getters (`getElements`/`getParentLocation` via reflection —
  package-private), `BlockModelDefinition.fromJsonElement` +
  `instantiate(StateDefinition, String)`, `MultiVariant`/
  `MultiPart` roots with `visualEqualityGroup` (an IntList of
  matching selector indices, confirmed by bytecode), and the
  state-definition mapper straight from the block registry
  (`Registry.getValue`, which returns the value directly —
  `get` returns an Optional in 1.21.4).
- **Datafix exporter**: 1.21.4's `BlockStateData.register` takes
  SNBT strings, `(int, String, String...)`, not Dynamics — the
  hook descriptor, the `record` signature and the replay check
  (value equality, null-safe) were ported accordingly; the
  recorded strings are re-parsed with vanilla's own `parse`.

The first `horizon-prepare` run surfaced 52 javac errors in 6
files; the second run surfaced 2 more; the datafix exporter
then failed at runtime (twice — the SNBT register, then a
null replay entry). The fix batches above cover the latent
occurrences of the same API shapes across the whole oracle tree
(the exporters compile in batches, so each run's errors were
only the first batch).

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (11 m 56 s at `-j 1`, post-rebase run, with
  `CARGO_INCREMENTAL=0`; the only warning in the run was a dead
  `float` helper in `item/hash/raw.rs`, removed in ca20060).
- `cargo test -p horizon-protocol`: **424 pass / 2 fail** (426
  total — down 3 from 1.21.5's 429: the packet rewire removed 4
  test-block tests and added 1 `AddExperienceOrb` test). The 2
  failures are the same pre-existing sandbox-network pair in
  untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope.
- `horizon-prepare` completes end-to-end: exit 0, "Done in
  167.0 s", after the oracle port above.

## Harness subset: no local profile (skipped, CI to measure)

No local harness profile was measured. The 1.21.6 publish
attempted a local sweep and spent ~4 h 30 min on attempts that
produced no profile (the `fast`-profile horizon-client build
OOMs on this VM's 6 GB under 3-stream load); the programme has
since moved to measuring profiles on CI, and disk quota pressure
(~99G of ~100–103G on 2026-11-19) rules out the local sweep's
multi-GB build trees. The profile is to be measured by the CI
`port-verify` workflow on GitHub runners (16 GB) after merge.

`status.json` records the harness section as 0/0/0 of 17 with
this note, and the CI run will fill it in.

## Known gaps (mechanical-layer limits, for the behaviour port)

- **Tooltip hiding is lost writing to 1.21.5+ servers**: the
  marker is dropped by `write_patch`'s `has` filter (no DOWN
  synthesis — the target's `tooltip_display` id differs by
  version and the generated `Ids` struct has no name→id lookup).
- **Per-component show flags don't round-trip through saves**:
  `components_tag` keeps emitting the synthesized 1.21.5-form
  `minecraft:tooltip_display` compound, which a 1.21.4 save
  would carry as an unknown component (vanilla drops unknown
  components on load). The 1.21.4 save form (per-component
  `show_in_tooltip: false`, separate hide booleans) is
  behaviour-port work.
- **`skip_dropped` reads the EitherHolder form unconditionally**
  for the removed components, but 26.1+ sends them as bare
  holders — a 26.1+ stack carrying `damage_type`,
  `chicken/variant` or `provides_trim_material` may misparse.
  Same pre-existing pattern as `damage_type`; untested in this
  tree.
- **Smithing-trim DOWN-write old form not implemented**: the
  write side always writes the new form (pattern id); a
  pre-1.21.5 guest's recipe displays would need a reverse
  pattern→template-item lookup plus the trim-pattern context on
  the guest path.
- **Text components stay JSON strings in NBT at this pin**
  (the 4290 unflatten fix is a 1.21.5 fix): the pin codec reads
  and writes NBT compounds on the wire, and `from_tag` converts
  legacy JSON strings — but anything that reads a 1.21.4 save's
  text components as compounds will find strings instead.
- **Equip-on-interact is dropped on the pin wire**: the 1.21.4
  equippable layout has no interact field, so a 1.21.5 stack's
  `equip_on_interact=true` does not survive the pin codec (the
  DOWN write to a 1.21.5 target still round-trips it).
- **`/clone` still accepts a `strict` argument** (batch 16):
  `strict` is a 1.21.5 addition to CloneCommands (0 occurrences
  in 1.21.4's jar, present in 1.21.5's), but removing it needs
  the clone block-tick tests reworked — they depend on strict
  placement (no own tick) to assert the copied ticks — so it
  stays until a clone-focused item takes it. `/setblock` and
  `/fill` had their strict removed in batch 16.

## What remains, by cause

### Behaviour work from the diff (not started)

The 548 ranked items are the port's work list; **0 of 548 are
closed**. The work is the test-block/garden/egg/component
removals' behaviour consequences, the 1,842 changed classes
(1,628 concern Horizon), the save-format step (4325 → 4189:
0 schemas, 4 fixes, 93 datafix classes), and the 348 harness
reruns — unattempted. The behaviour code in this tree is still
the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21.5
unchanged — the same oracle failures as 1.21.5 are expected once
the profile is measured, but the counts are unmeasured for
1.21.4.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was skipped (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Status

Pin set (protocol 769, world 4189); 6 of 6 tables regenerated;
packet rewire done (3 packets removed, 1 restored, 117
renumbered) and the game-side wire boundary reworked; adapter
ladder re-based to 769; component codec re-pinned to the 1.21.4
registry (tooltip synthesis, `Quirks::PINNED` fixed); oracles
ported to the 1.21.4 APIs (44 files + 3 follow-ups); workspace
compiles (0 errors, 0 warnings); protocol tests 424 pass / 2 fail
(the sandbox-network pair); prepare verified ok end-to-end.
Harness profile: **not measured locally** (skipped — CI to
measure). D6 deferred (owner).

End main island (worldgen-verify): vanilla 1.21.4's
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

## Behaviour port — rank 1 (10 items, CLOSED 2026-11-24)

- Item 1 components (PORTED): the wire layer was already 1.21.4's
  (`crates/horizon-protocol/src/item/v1_21_4.rs` codec,
  `HIDE_ADDITIONAL_TOOLTIP = 15`, `HIDE_TOOLTIP = 16`, synthesized
  `TOOLTIP_DISPLAY = -1003`), but `DATA_COMPONENT_TYPES` in
  `crates/horizon-assets/src/item_model.rs` was the stale 26.3
  122-entry list, still containing all 31 removed components (the
  1.21.5 tree's list is stale the same way — the 1.21.5 port never
  rebuilt it). Rebuilt to the 67-entry 1.21.4 list, alphabetically
  sorted, jar-verified from `DataComponents.<clinit>` in the 1.21.4
  named client jar: 67 ids in registration order, [15] =
  hide_additional_tooltip, [16] = hide_tooltip, none of the 31
  removed names present, both added names present. Behaviour
  fallout — code that reads the removed components by name —
  belongs to the rank-2 items for those modules.
- Item 2 protocol (verified): all 131 play clientbound + 62 play
  serverbound ids in `crates/horizon-protocol/src/ids.rs` match the
  report's renumbering table exactly (0 errors across the 117
  renumbered, 3 added, 3 removed); zero test-block packet remnants
  in the file.
- Items 3–8 regenerate (verified): `regen-determinism.json` shows all
  six generators reproduce the committed tables byte-identical, with
  no overrides.
- Item 9 save-format (PORTED from the 1.21.11 template's final state,
  with 1.21.4 adjustments — the full removal, not the 1.21.10 port's
  always-false stub): 1.21.4 has no `FileFixerUpper`
  (`net/minecraft/util/filefix` is absent from both named jars) and
  no `FILE_FIXING_REQUIRED` backup status (`LevelSummary$BackupStatus`
  is only NONE/DOWNGRADE/UPGRADE_TO_SNAPSHOT, javap-verified from the
  named client jar). Deleted fix_1493/4772/4773/4899,
  `level_dat_to_saved_data` and their helpers
  (`resolve`/`move_file`/`delete`/`dimensions`, `read_level`/
  `write_level`/`write_saved_data`/`read_saved_data`),
  `requires_file_fixing` and the `FILE_FIXES` constants; the
  whole-world upgrade follows the pre-26.x layout (overworld at the
  root, DIM-1/DIM1, `playerdata/`, `advancements/`, `stats/`);
  `current_world_gen_settings` reads `WorldGenSettings` from
  level.dat's `Data` only; `saved_data_type` keeps only the types
  1.21.4's `DataFixTypes` defines — the 1.21.11 set minus
  stopwatches (no `Stopwatch` class in the jar) and minus
  world_border (no world-border saved data in 1.21.4; the border
  lives in level.dat) — with `ForcedChunksSavedData.FILE_ID =
  "chunks"` javap-verified for the flat `chunks.dat` mapping.
  `NbtUtils.getDataVersion` in 1.21.4 has only the two-argument
  `(CompoundTag, int)` form (no `(Dynamic, int)` form yet).
  `WorldUpgrader` has the inner classes (`AbstractUpgrader`,
  `ChunkUpgrader`, `SimpleRegionStorageUpgrader`, `EntityUpgrader`,
  `PoiUpgrader`, `FileToUpgrade`, `DimensionToUpgrade`),
  javap-verified, so the 1.21.11 doc shape applies. The 4 "added"
  fixes are already implemented in the Rust table in their 1.21.4
  (split) forms: `BlockEntitySignTextStrictJsonFix` +
  `ItemWrittenBookPagesStrictJsonFix` at key(165, 0) (1.21.5
  consolidated them into `TextComponentStrictJsonFix`),
  `ObjectiveDisplayNameFix` + `TeamDisplayNameFix` at key(1514, 0)
  (1.21.5 consolidated them into `ScoreboardDisplayNameFix`).
  `DataFixers`: 1.21.4's fix versions (177 ≤ 4189) are a subset of
  1.21.5's (191 ≤ 4325) except key 2537 (`legacy_dimension_id`,
  kept per the 1.21.11/1.21.10 templates — the 1.21.5 tree lacks
  it); the 15 1.21.5-only versions are all > 4189 (4293–4314, the
  1.21.5-release fixes), so the CURRENT-gated fix table is correct
  as-is. The 93 changed datafix classes are mechanical noise only
  (sampled `FixWolfHealth` — a package move into `fixes/`;
  `BlockEntityIdFix` — `ID_MAP` public→private;
  `ItemStackTagFix` — the Typed→Dynamic API reshape plus lambda
  renumbering — no behaviour change), so the Rust fix
  implementations are untouched. Files:
  `datafix/{files,mod,progress,world_upgrader,tests}.rs`,
  `datafix/fixes_legacy.rs` (doc comments naming the 1.21.4 fix
  classes), `save.rs` (`needs_file_fixes` deleted,
  `world_gen_settings` reads level.dat only, test replaced with the
  level.dat-through-the-fix-pipeline form), `title.rs`
  (`WorldEntry.file_fixing` field deleted, `can_edit` /
  `should_backup` simplified, the file-fixing backup branch and
  the `selectWorld.upgrade_and_play` button gone, the backup
  prompts' file-fixing arms replaced with `unreachable!`, the
  file-fixing tooltip block deleted, `select_tooltip` loses its
  `selected` parameter), `title/tests.rs` (the file-fixing
  optimize test deleted). NOT in item 9: the `save.rs` settings
  store still writes `world_gen_settings.dat` / `world_clocks.dat`
  / `game_rules.dat` as saved-data files (26.x) instead of
  level.dat's `Data` — that rewire is rank-2 save_roundtrip work,
  as in 1.21.11.
- Item 10 facts (verified): `version.rs` reads 1.21.4, protocol 769,
  data version 4189, data pack 61.0, resource pack 46.0, Java 21,
  stable, build time Tue Dec 03 10:09:48 UTC 2024.

## Behaviour port — rank 2

Triage (2026-11-24): a batched javap jardiff of the 1.21.5 and
1.21.4 named client jars (1,565 rank-2/3 classes: 1,333 common, 7
added, 225 removed, 1,313 method-changed, 0 javap failures) crossed
with a per-module diff of the 1.21.5 and 1.21.4 trees (248 identical,
145 differs, 102 widely-named, 2 only-in-1.21.4) gives 42 no-change,
13 carry-over and 293 review. The semantic diff over the 962 changed
named methods of the review items classifies 719 real, 101 same, 131
missing-new, 11 missing-old; `real_diff.py` clusters the real ones by
shape for the review batches.

- Batch 1 (42 no-change): items 90, 172, 183, 190, 194, 233, 235,
  236, 243, 244, 247, 250, 257, 262, 263, 271, 273, 275, 282, 283,
  284, 285, 290, 291, 295, 296, 297, 298, 299, 300, 317, 318, 320,
  321, 322, 324, 325, 327, 328, 330, 331, 358. Each module is
  identical between the 1.21.5 and 1.21.4 trees (0 diff lines) and
  no named method changed 1.21.5→1.21.4 per jardiff (verified
  programmatically across all 42: 0 named-method hits, including
  class removals). Rank 2: 42/348.

- Batch 2 (8 of 13 carry-over): items 189, 216, 227, 229, 242,
  272, 294, 329. Each module's 1.21.5 form carried with jar
  evidence; no named method changed 1.21.5→1.21.4 for any of them.
  189 item_model_fixture.rs: the `oversized_in_gui` fixture field
  removed (1.21.6-only; `oversized` occurs only in RegionFile and
  RealmsDownloadLatestWorldScreen in both jars). 216
  remote_mob_data.rs: the HappyGhast STAYS_STILL/IS_LEASH_HOLDER
  arms removed (0 HappyGhast classes in the 1.21.4 jar); test
  renamed, happy ghast → cow. 227 perspective.rs: comment-only,
  the `camera_distance` comments reworded to 1.21.4 (the attribute
  is 1.21.6-only; the 1.21.4 distance is a flat 4× scale). 229
  game/recipe_book.rs: `Call::Stratum` removed (nextStratum added
  in 1.21.6; Stratum absent from both jars), with its fixture arm.
  242 collision_context.rs: `placement()`/`with_position()`
  removed (1.21.4 `CollisionContext` has only `empty`/`of`/`of`;
  `placementContext` arrived in 1.21.5, `withPosition` in 1.21.6),
  fixture/oracle/drivers trimmed; the `placement` flag stays,
  always false. Caveat: the oracle still calls
  `emptyWithFluidCollisions()` and `positionContext(y)`, absent
  from both jars — pre-existing in the 1.21.5 oracle too, and the
  collision harnesses are not in the 1.21.5 pin profile, so oracle
  compile status is unproven until a CI harness run. 272
  functions.rs: `create_pack`/`is_portable_name` and the
  `/datapack create` node removed (`create` is not a
  DataPackCommand literal in either jar). 294 lan.rs: the dialog
  sync, the waypoint fields and `lan_update_waypoints` call, the
  HappyGhast metadata arms and the dialog test fixtures removed;
  the frog-variant hunk KEPT (pig/cow/chicken variants are
  1.21.5-only), so lan.rs is otherwise byte-identical to 1.21.5.
  329 recipe_book.rs: the 1.21.5 read/write form carried,
  jar-verified (a missing list reads as empty, settings still
  load; all eight open/filter flags written unconditionally), with
  a new `a_book_without_lists_still_loads_its_settings` test.
  The waypoint/locator-bar system removal anchored at item 294
  spans game.rs (the contextual HUD bar → the direct jump/XP bar,
  the Game fields, the mod decls), client_lib.rs, game/packs.rs,
  game/resource_loader.rs, game/remote.rs, game/world_session.rs,
  game/lan_attributes.rs, game/mounts.rs (`mount_jumping`),
  game/commands.rs + commands/extra.rs + commands/admin.rs (the
  `/waypoint`, `/dialog` and `/version` commands),
  attribute_updates.rs, motion.rs (`MotionEvent::Waypoint`),
  item_tooltip.rs (the waypoint attributes, the invisibility
  mapping), gamerules.rs + save.rs (the `locator_bar` gamerule),
  lan/mod.rs, scene_motion_probe.rs, and the deleted files
  game/locator_bar.rs, game/lan_waypoints.rs, lan/waypoints.rs,
  locator_bar.rs. The dialog-system removal (dialog.rs,
  dialog_screen.rs, game/dialogs.rs, screens.rs, remote_chat.rs,
  packets/play.rs `TrackedWaypoint`/`DialogHolder`) is still in
  progress and is recorded under items 274/323. Rank 2: 50/348.

- Batch 2 continued (item 177): `crates/horizon-protocol/src/
  item/hash/raw.rs` verified against the jar-verified 67-entry
  1.21.4 component registry: the mechanical publish had already
  re-pinned every component arm to the 1.21.4 ids and dropped the
  1.21.5-only components (`potion_duration_scale`,
  `provides_trim_material`, `weapon`, `blocks_attacks`,
  `break_sound` — all absent from the 1.21.4 registry) from the
  arms and the test cases. The variant serializer arms (72–95,
  Horizon synthetic ids) are unchanged from 1.21.5; arms 74/84/
  85/86 reference registries 1.21.4 lacks (wolf_sound_variant,
  pig/cow/chicken variants — confirmed via `Registries` javap)
  and return None, the documented no-codec-form outcome; they
  stay because the default arm falls through to
  `gameplay::code` and removing them would change that path.
  The 13 mob-variant test cases the publish's regen dropped
  were restored with their 1.21.5 hashes (the arms are live code
  in 1.21.4, the CRC32C hashes version-independent;
  1.21.5/1.21.10/1.21.11 all keep them). Rank 2: 51/348.

- Batch 2 continued (items 274, 323): the dialog-system removal
  is closed. Item 274 (`packets/play.rs`): the `DialogHolder`
  enum + codec and the waypoint types (`waypoint_operation`,
  `WaypointId`, `WaypointTarget`, `TrackedWaypoint` + codec)
  removed; the `dialog_holder_encodes_registry_as_id_plus_one`
  test removed; the "1.21.5 has no dialog packets" comments
  fixed to "1.21.4" in play.rs, configuration.rs and adapter.rs.
  KEPT, jar-verified: the `AddExperienceOrb` packet + enum entry
  (1.21.4 HAS `ClientboundAddExperienceOrbPacket`; 1.21.5 does
  not — removed in 1.21.5, orbs spawn through AddEntity there)
  and the ABSENCE of the test-block packets (0 test-block classes
  in the 1.21.4 jar). Item 323 (`game/remote_chat.rs`): the
  1.21.5 form carried wholesale — the command-confirmation system
  removed (`CommandCheck` + `verify_command`, `CommandAccept`,
  `Behind`, `CommandPrompt`, `send_unattended_command`) and
  `send_unsigned_command` restored; jar-verified, 1.21.4's
  `ClientPacketListener` has `sendUnsignedCommand`, not
  `sendUnattendedCommand`. The removal spans game.rs (the
  `dialogs` and `command_prompt` fields, the `DialogWarning`
  screen, every `dialog_open` guard), remote_chat.rs,
  net_harness.rs, native_screen_observer.rs, chat.rs, books.rs
  (click-handler hunks), sign_edit.rs, screens.rs,
  server_links.rs, agent.rs, control.rs, core_tests.rs,
  debug_options.rs, play_keys.rs, key_codes.rs, play_input.rs,
  ime.rs, inventory_input.rs, pad.rs, pad_menu.rs,
  pointer_capture.rs, screen_backgrounds.rs, remote.rs
  (`ContainerClose`), narrator.rs (`edit_box_focused` hunk),
  net.rs (`send_custom_click`), widgets.rs (`CHECKBOX_BOX`),
  horizon-client-core text.rs + component.rs,
  horizon-client-render signs.rs + hud_layout.rs,
  horizon-command codec.rs + component.rs + tree_extra.rs (the
  `/dialog` and `/waypoint` commands), lan/data.rs (the
  `minecraft:dialog` registry + `pack_entries` + dialog tests),
  and the deleted files dialog.rs, dialog_screen.rs,
  game/dialogs.rs, game/quick_actions.rs. Final sweep: no
  `dialog_open`/`dialog_key`/`command_prompt`/`DialogHolder`/
  `TrackedWaypoint`/waypoint code refs remain (only the
  component.rs test proving 1.21.6 click actions are dropped,
  pathfinding waypoints, and comments). Rank 2: 53/348.

- Batch 2 continued (items 249, 326): the HappyGhast system
  removal is closed, and batch 2 is complete (13/13 carry-over).
  Item 249 (`mobs/animal_brain.rs`): the 1.21.5 form carried
  wholesale, jar-verified (HappyGhast 0 classes in the 1.21.4
  jar; `FollowTemptation` has only the 2 ctors in 1.21.4, no
  3-arg look-in-eyes form) — the Ghastling brain and the 1.21.6
  struct shapes (`Tracker.block_eye`, `AnimalPanic.fly`,
  `FollowTemptation.look_in_the_eyes`, `FollowAdult.player`,
  `B::reset`) removed. The removal spans mobs.rs, species.rs,
  riders.rs, entity_culling.rs, ai.rs, living.rs,
  living_tests.rs, anim.rs, render.rs, equine.rs, finalize.rs,
  drops.rs, leash.rs, mounts.rs, passengers.rs,
  remote_mob_use.rs, remote_vehicles.rs, remote.rs,
  creatures.rs, mob_sounds.rs + sound_engine.rs, anticheat.rs,
  dispenser.rs, entity_nbt.rs, scoreboard.rs + stats_screen.rs,
  boat_leads.rs, cem.rs, sound_tests.rs,
  block_update_fixture.rs, nether_fossil.rs (`placeDriedGhast`),
  animate_tick.rs, game.rs, and the deleted files
  mobs/happy_ghast.rs + game/ghasts.rs. KEPT, matching 1.21.5:
  the dried-ghast BLOCK handling (animate_tick, horizon-world
  level classes/fluids/place/shapes/tick/timeline) — data-driven
  dead code, the block is not in the 1.21.4 tables. Final sweep:
  0 HappyGhast/happy_ghast/ghastling refs in Rust code. Item 326
  (`mobs/parrot.rs`): the 1.21.5 form carried wholesale,
  jar-verified — 1.21.4's `Parrot.MOB_SOUND_MAP` has exactly 37
  entries, the ender dragon IS imitated (None key) and happy
  ghast/camel husk/parched/zombie horse/zombie nautilus are NOT
  in it. Rank 2: 55/348.

- Batch 3 (items 11 + 155): the entity save form ported to
  1.21.4's lists, jar-verified — the first version in the
  programme BELOW 1.21.5, so the ArmorItems/HandItems list form
  has no template and was written from the 1.21.4 jar.
  `Mob.addAdditionalSaveData` writes exactly 12 keys: the four
  lists (always, empty slots as empty compounds),
  body_armor_item + body_armor_drop_chance (only with a body
  item), DeathLootTable/Seed, CanPickUpLoot, LeftHanded, NoAI,
  PersistenceRequired — no `drop_chances` compound (no
  `DropChances` class), no leash/home keys. Equines and camels
  save `SaddleItem`, pigs and striders a `Saddle` boolean, and
  there is no SADDLE slot. Armor stands write the same lists.
  `FallDistance` is a FLOAT in 1.21.4 (`putFloat`; 1.21.5
  renamed it to a double `fall_distance`) — renamed across
  entity_nbt.rs, save.rs (player), persist.rs, commands/data.rs,
  projectile_fields.rs, snbt.rs, and the cushions /
  structure-mob / saved-tag tests; the movement-predicate
  `fall_distance` JSON field is untouched (a separate item).
  Phantom: `AX`/`AY`/`AZ` + `Size`, anchor defaulting to
  BlockPos.ZERO. PrimedTnt: no owner saved. Bucket variants
  ride inside `bucket_entity_data` (axolotl `Variant`, salmon
  `type`, tropical fish `BucketVariantTag` packed) — the
  separate variant components do not exist in the 1.21.4
  registry and their reads are gone. `getBooleanOr` does not
  exist in 1.21.4 (all sites now `getBoolean`). Item 155
  (`mobs/vex.rs`): the plain-field owner form, jar-verified
  (Vex saves BoundX/BoundY/BoundZ + LifeTicks only; no
  `EntityReference` class). No-change, jar-verified: mooshroom
  `Type`, wandering trader `wander_target`, dolphin, piglin,
  ominous spawner, warden anger/listener, the NeutralMob
  AngerTime/AngryAt and AgeableMob Age/ForcedAge forms, and all
  the getXOr/codec/playSound/isDay/snapTo/putUUID reshapes.
  Tests: 4 new list-form round-trip tests (zombie gear, drop
  chances, armor stand, salmon/tropical buckets); the horse,
  wolf, TNT, axolotl, spawn-egg, phantom, raider, mannequin,
  armor-stand, advancement and item-command tests converted to
  the list form. NOT in this item, with jar evidence recorded
  for their own items: finalize.rs (item 117) — 1.21.4's
  `saves_variant_name` is Wolf/Cat/Frog only (chicken writes
  only EggLayTime, cow/pig write no variant key) and
  `sound_variants` is empty for all species (0 `sound_variant`
  strings in the jar); beehive.rs (item 79) —
  IGNORED_BEE_TAGS is the 30-entry 1.21.4 set
  (ArmorDropChances/HandDropChances, `FallDistance`); phantom
  goal code (item 401); the tropical-fish bucket tooltip
  (item 22). The nautilus's `equipment`-compound saddle save
  (mobs/nautilus.rs) is dead code — no nautilus in 1.21.4, no
  spawn path — left as the 1.21.10 port left unremoved species.
  SpawnData's `equipment` field is real in 1.21.4
  (`EquipmentTable`), so spawner.rs is untouched. Rank 2:
  57/348.

  Batch-2 compile follow-ups (same commit): the batch-2
  `cargo check` never reached the horizon-client lib — it died
  on the GAME_RULES length error in horizon-client-lib first —
  so the batch-3 check surfaced 19 missing-method errors from
  the system removals, all fixed to the 1.21.5 form. The deleted
  `game/ghasts.rs` held the generic entity-collision helpers, so
  `game/entity_collisions.rs` is restored from 1.21.5 (Shulker/
  boat only, no ghast arms) with its test; `game/locator_bar.rs`
  held `own_uuid`, restored into `game/names.rs` with its `uuid`
  helper; the `close_dialogs` (5), `dialog_scroll` (1),
  `track_experience_display` (2) calls are gone with the
  `experience_display` field, and `Remote::player_entity` (whose
  only caller was the deleted locator bar) is gone too. The
  /version command's `version_lines` went with cmd_version, so
  the F3 DumpVersion debug action and its key binding are gone
  (ACTIONS 20 -> 19). `AXOLOTL_VARIANTS` is test-only now.
  Full `cargo check -p horizon-client` green (EXIT 0, 0 errors)
  after these fixes; the check's one warning (the dead
  `Remote::player_entity`) was fixed by removing the method, and
  the next batch's check re-verifies the tree warning-free.

  Batch 4 (items 12 + 13). Item 12 (game.rs): DecoratedPotBlock
  and TripWireBlock `playerWillDestroy` setBlock flags 260 -> 4
  (1.21.4's `UPDATE_NONE` is 4 — `Block` has no
  `UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS`, and the Rust
  `set_block` skips block-entity side effects at flag 256, so
  260 changed behaviour); ComposterBlock `handleFill` particle
  bounds 0.1875/0.625 -> 0.13124999403953552/0.737500011920929
  (game/composter.rs, which has no report item of its own; the
  three gaussians are x0.02 in both versions, the diff's reorder
  was a local-renumbering artifact). No-change, jar-verified:
  BeehiveBlock `playerWillDestroy` (`preventsBlockDrops` is
  `instabuild`, same as `isCreative`) and `playerDestroy`
  (`Containers.updateNeighboursAfterDestroy` is a trivial
  wrapper), ShulkerBoxBlock `playerWillDestroy`, the
  Scaffolding/Shulker shape renames, `handleKeybinds`/
  `startUseItem` (`hasInfiniteMaterials` == `isCreative`),
  MouseHandler (same scaled-pos math), `storeGameTypes` (same
  NBT), the RespawnConfig message rule (send iff the point
  moved, both versions), the Entity-vs-Player overloads
  (levelEvent, mayInteract, BucketItem, candle extinguish),
  `loadAndSpawnParentVehicle` (RootVehicle contains-check),
  `openItemGui`, `getSituationalMusic`, the IDE-only world
  screen branch. Item 13 (game/remote.rs):
  `handleMinecartAlongTrack` gains the 1.21.4
  `isControlledByLocalInstance` early return (the local player
  riding the cart skips the server's steps; 1.21.5 dropped the
  guard); the game-event type is `ARROW_HIT_PLAYER` (id 6 in
  both jars — the 1.21.5 rename is name-only). No-change,
  jar-verified: the game/sound event overloads, the
  isLocalInstanceAuthoritative/isControlledByLocalInstance and
  setSelectedSlot renames, the login chat-index reset, the
  portal/spinning effect rename (the respawn intensity copy is
  unmodeled), the chat-signature error path (unmodeled),
  `hasInfiniteMaterials` == creative, the effect renames, and
  the pose-priority flip (FALL_FLYING-first vs SLEEPING-first
  is unobservable — the states are mutually exclusive, and the
  Rust guest pose already checks gliding before swimming).
  Routed with jar evidence: the player respawn flat save form,
  `enteredNetherPosition`, no `raid_omen_position`, forced
  chunks via `chunks.dat` (`Forced` long array — no
  `TicketStorage` in 1.21.4) and the ticket lifecycle (item 23);
  the `canDestroyBlock` tool-component check (item 119 —
  1.21.4's Tool record has no `canDestroyBlocksInCreative`);
  KeyboardInput normalization + the sprint 0.8 impulse
  threshold (item 15 batch — `keyboard_direction` normalizes,
  the 1.21.5 form; 1.21.4 stores raw impulses, needs forward
  >= 0.8 to start a sprint on land, > 1e-5 underwater);
  `Player.aiStep` swing/head placement and the fall-damage
  formula (item 15); SetPassengers on `startRiding` (riding
  items); `tickChunk` thunder (horizon-world tick item);
  `Raids.tick()` (item 21); the scaffolding placement
  collision (shape item — the Rust `is_placement` flag is
  always false, so behaviour already matches); the
  `onProjectileHit` pot crack 260 -> 4 in creatures.rs:5090
  (item 17). Rank 2: 59/348.

  Batch 5 (items 14 + 212). Item 14 (entities.rs): the
  lava-cauldron contact test is the 1.21.4
  `isEntityInsideContent` form — feet below the lava surface
  (0.9375) and top above a quarter block, no horizontal test
  beyond the cell overlap, the rim and feet do not count (the
  1.21.6 filled-shape test, its `VoxelShape`/portal-travel
  imports and the `OnceLock` shape are gone; the 1.21.5
  lava-cauldron test is carried, adapted to 1.21.4);
  `LavaCauldronBlock.entityInside` is lavaHurt only — the hurt
  itself ignites for 15 s, there is no `lavaIgnite` in 1.21.4.
  The orb-spawn system drops everything 1.21.6 added:
  `ExperienceOrb::unstuck` and its tests are gone (no
  `unstuckIfPossible` in 1.21.4), `spawn_orbs`/`add_orb` are
  the plain 1.21.5 forms (`spawn_orbs_toward`/`add_orb_toward`/
  `unstuck_orb` removed; yaw is a `nextDouble`, the motion's
  x/z factors the widened `0.2F`/`0.1F` literals), the stuck
  orb's `moveTowardsClosestSpace` nudge is unconditional (the
  1.21.6 next-move-collision guard removed), `/summon
  experience_orb` spawns exactly at `pos`, and the thrown
  experience bottle awards plain (its `splash` loses the
  `motion` parameter; both callers updated). Orb NBT: Health
  reads `getShort` with no default — 0 when absent, not 5
  (jar-verified; 5 is the 1.21.5 `getShortOr` form). Also:
  `AttributeMap.save` (not `pack`), and the happy-ghast
  comments are gone from four sites. No-change, jar-verified:
  the remote packet-velocity application at spawn
  (`SetEntityMotion` overwrites with the same value before the
  entity's first tick, so applying the packet velocity is
  unobservable), `collidedWithShapeMovingFrom`/`collide`/
  `isColliding` (API-only), the in-memory `fall_distance` f64
  (its save form was ported in batch 3), and the 1.21.5-only
  methods absent from 1.21.4 (`snapTo`, `propagateFallToPass-
  engers`, `lavaIgnite`, `followNearbyPlayer`,
  `scanForMerges`, `applyComponentsFromItemStack`, and the
  rest — none is modelled in the 1.21.4 tree). Item 212
  (game/item_entities.rs): its module carries the orb-spawn
  port above; its own two methods are no-change —
  `FurnaceResultSlot.checkTakeAchievements` (the `onCraftedBy`
  overload gains the Level arg; unmodelled in Rust) and
  `SculkBlock.attemptUseCharge` (full bytecode identical
  except the `playSound` Entity-vs-Player overload).
  Routed with jar evidence: `ServerEntity.sendChanges`
  (lan/tracking — no rank-2 item covers it); the `baseTick`
  frozen handling (items 187/15); `causeFallDamage`/
  `checkFallDamage` (landing.rs, items 126/296);
  `applyEffectsFromBlocks` (items 15/126/36/66);
  `checkInsideBlocks` (items 15/36); `extinguishFire`/
  `igniteForTicks` (items 55/15/36/160); `isInLiquid` (item
  126); the `teleportTo` pitch clamp [-90, 90] (item 42);
  `NameTagItem`'s `canBeNameTagged` check (item 17);
  `HoneyBlock` entityInside/fallOn (item 52);
  `placePortalTicket` (likely unmodelled in the client).
  `cargo check -p horizon-client` green (EXIT 0, 0 errors).
  Rank 2: 61/348.

  Batch 6 (item 15). Item 15 (motion.rs + the input/sprint/fall
  system, which has no rank-2 item of its own): the input model is
  1.21.4's — `KeyboardInput.tick` stores RAW impulses, so
  `keyboard_direction` returns them unnormalized (1.21.5
  normalizes into `moveVector`; the normalization in 1.21.4
  happens later, in `moveRelative`'s `getInputVector`, which the
  Rust already implements), and `modify_input` drops the
  square-movement correction (`square_movement` deleted;
  1.21.5's `modifyInput` only — 1.21.4 has no `modifyInput`, the
  0.2 item-use and sneaking multipliers apply directly to the
  impulses in aiStep and the 0.98 damping lives in
  `LivingEntity.aiStep`). Movement-equivalence checked by hand:
  raw + no-square reproduces vanilla 1.21.4 exactly in every
  case (full diagonal, item-use, sneak, straight); the old
  normalized + square form matched too for keyboard input, but
  boosted analog diagonals. Sprint is 1.21.4's
  `LocalPlayer.aiStep`: the start threshold is
  `hasEnoughImpulseToStartSprinting` (0.8 on land, any forward
  impulse underwater, on the raw impulse — a half-pushed stick
  no longer starts a sprint on land), the stop block keeps
  `hasForwardImpulse` (> 1e-5), `canStart` forbids fall-flying
  outright (1.21.5 allows it underwater), the start block needs
  the ground (the vehicle's `onGround` for a passenger, threaded
  through `sprint_input_tick_riding` from `driven_mount`) or
  water — no mid-air sprint start (1.21.5 drops the gate) — the
  sprint key also starts a surface swim, the double-tap window
  resets on shift and item use but NOT on backward (the
  backward reset is 1.21.5's), and the fresh-press edge is the
  previous tick's `hasEnoughImpulse`. Three new sprint tests pin
  the threshold, the ground gate and the backward rule. Fall
  damage is 1.21.4's ceil form: `calculate_fall_damage`
  (living_fall.rs) computes `(fall - safe) * multiplier` in
  float, times the fall-damage multiplier, ceiled (1.21.5
  floors `calculateFallPower`); the landing-particle power in
  living_landing.rs ceils too, with the magnitude in float
  (0.2 + power / 15, capped 2.5). A fall-flying player glides
  on past a climbable (1.21.5's `travelFallFlying` climbable
  early-out removed). `checkInsideBlocks` axis order follows
  the RESOLVED move in all three sweep sites (motion.rs,
  entities.rs `orb_lava_contacts` — its `wanted` parameter is
  gone — falling_block.rs `inside_blocks`): 1.21.4 traverses
  straight-line (`boxTraverseBlocks`) and the resolved delta is
  the closer approximation; 1.21.5's `axisStepOrder` takes the
  resolved delta explicitly. Cauldrons: the layered short-circuit
  is carried (entityInside fires on sweep overlap; the content
  check lives in `layered_cauldron_inside`, now taking the
  content top and applying `isEntityInsideContent` — feet below
  the top, head above a quarter block), `getContentHeight`
  (not `getPixelContentHeight`), and the lava-cauldron comment;
  the nether-portal visual-slab test is NOT carried — jar-verified,
  1.21.4's NetherPortalBlock has no
  `getEntityInsideCollisionShape` override (added in 1.21.5),
  so portals fire on whole-block overlap, as the tree already
  does. Same jar check: the base shape is `Shapes.block()` in
  both versions; only end portals and tripwires override it in
  1.21.4 (powder snow's override is 1.21.5's too). The
  `was_on_fire` snapshot in `applyEffectsFromBlocks` is NOT
  carried — 1.21.5-only; 1.21.4 sets -20 whenever not inside
  fire/lava, alive, with `remainingFireTicks <= 0`, which the
  tree's `apply_fire_contact` already does. `AttributeMap.save`
  (not `pack`) in two comments. Drowning: baseTick spawns the
  eight bubbles directly in 1.21.4 (no entity event 67, no
  `makeDrownParticles`); the `MotionEvent::Drown` doc says so
  now. No-change, jar-verified: `MobEffectInstance` save/load
  delegate to the CODEC in 1.21.4 too (the lowercase effect-NBT
  shape stands; the `readAdditionalSaveData` difference is the
  Java call shape only); `LivingEntity.tick`'s run/oRun and
  `animStep` (animation, unmodelled in motion.rs);
  `onAttributeUpdated`'s SCALE branch (1.21.4 refreshes
  dimensions in `tick` instead — same net behaviour);
  `getItemBySlot`/`setItemSlot` (abstract in 1.21.4, concrete
  via EntityEquipment in 1.21.5 — API-only);
  `getKillCredit`/`getLastHurtByMob`/
  `resolvePlayerResponsibleForDamage` (EntityReference vs
  direct fields — same behaviour); `pushEntities` (same
  behaviour, `getEntities` vs `getPushableEntities`);
  `handleRelativeFrictionAndCalculateMovement` (the powder-snow
  check reads the block state directly in 1.21.4, same
  behaviour); `causeFallDamage` (super first, then damage, in
  both — only float/double signatures differ); `breakItem`
  (BREAK_SOUND component vs `getBreakingSound` — same sound);
  `collectEquipmentChanges` (per-slot fields vs the map — same
  behaviour); the MobEffects renames (MINING_FATIGUE/
  DIG_SLOWDOWN, RESISTANCE/DAMAGE_RESISTANCE, JUMP_BOOST/JUMP);
  `tickEffects` (one `tick` vs the tickServer/tickClient
  split); `travel`/`travelRidden` (the
  `isControlledByLocalInstance` early return and the no-arg
  `travelFallFlying` — same values); `LayeredCauldronBlock.
  entityInside` (bytecode-identical); the aiStep
  lerp/InterpolationHandler difference (interpolation is
  unmodelled in motion.rs in both trees). Pre-existing gaps in
  BOTH trees, not port differences: the run-stop's
  `(isInWater && !isUnderWater)` term is missing from
  `should_stop`; blindness is not modelled in sprint
  `Conditions`; the aiStep-start `shouldStopSprinting` check
  (fall-flying/blindness/moving-slowly/using-item) is not
  modelled in the sprint phase. Routed with jar evidence:
  `canUseSlot` (base false in 1.21.4, true in 1.21.5; Player
  overrides it in 1.21.4) and the dispenser `mob_can_use_slot`
  default (item 39); `canFreeze`'s BODY-slot check — leather
  horse armor protects in 1.21.4 (item 187, mobs/ai.rs);
  `getItemBlockingWith`/`hurtServer` shield blocking
  (BLOCK animation + 5-tick delay; `hurtCurrentlyUsedShield`;
  event 29 broadcast) and `entityEventForEquipmentBreak`
  (7 slots, no saddle/68) (item 126); the mob fall-damage
  formula in `mobs/living.rs` (same floor -> ceil port as the
  player path) (item 126); `onEquipItem`/`releaseUsingItem`/
  `onEffectAdded`/`onEffectsRemoved` (items 126/55);
  `handleEntityEvent` 29/30 shield sounds and the missing 67/68
  cases — the LAN event-67 broadcast in lan_actions.rs (item 28)
  and creatures.rs (item 17) should send particles instead, and
  remote.rs's event-67 handling (item 13, closed) is dead in
  1.21.4 play; the swing/head placement (1.21.5's Player.aiStep
  calls `updateSwingTime` and sets `yHeadRot = yRot`; 1.21.4
  does neither — swing advances in `LivingEntity.tick`) (item
  164); `Player.attack`'s sweep ordering (knockback + hurt
  before the server check in 1.21.4; `hurtServer` inside it in
  1.21.5) (item 13, closed — check whether batch 4 covered
  it); the `makeDrownParticles` doc names in combat.rs (item
  210) and mobs.rs (item 55). Follow-up hunks in closed modules:
  the axis-order fix in entities.rs (item 14) and the
  `modifyInput` doc in game.rs (item 12).
  `cargo check -p horizon-client -p horizon-world
  -p horizon-client-input --tests` green (EXIT 0, 0 errors;
  the lib check green too). The --tests run caught two
  test-compile errors from earlier batches (fixed above);
  batches from here run the --tests check, not just the lib.
  Rank 2: 62/348.

  Batch 7 (items 16 + 17 + 18). Item 16 (game/game_tests.rs):
  NO-CHANGE on all three tree-diff hunks. The 1.21.4 jar has no
  TestInstanceBlock classes at all (1.21.5 has 10): 1.21.4's
  game-test framework is the structure-block model
  (`GameTestInfo` with `TestFunction` + `StructureBlockEntity`;
  `prepareTestStructure` returns GameTestInfo and calls
  `StructureUtils.prepareTestStructure`), while 1.21.5's is the
  test-instance-block model (`GameTestInstance` +
  `TestInstanceBlockEntity`, `createTestInstanceBlock` +
  `placeStructure`). The 1.21.4 tree's runner is therefore a
  1.21.5-form system on 1.21.4 data, but it is safe: no panic
  path was found (`set_test_block_state` returns early when
  `default_state` is None; `prepare_test_structure` inserts a
  default TestInstanceBlock block entity when absent), and the
  runner is a dev-only subsystem (`/test`), not player
  gameplay. `track_and_start_runner` (16 lines, 1.21.5-only) is
  dead in 1.21.4 — the 1.21.4 tree's test_instances.rs has no
  test-instance-block action packet and no `track_and_start`
  call, and the 1.21.4 test calls `track_and_start_batches`
  directly — so it is not added. `prepare_test_structure`'s
  `bool` return (false when the block is gone; both callers
  check it) fits 1.21.4's structure-block model, where the
  block can be absent; 1.21.5's `()` form requires the block
  entity. `say` stays `say` (1.21.5 renamed it `say_report`
  because `Game::say` is the narrator's; the 1.21.4 tree has
  no `Game::say`). A rewrite of the runner to the
  structure-block model is a separate job, recorded as a known
  limitation, not a behaviour-port item.

  Item 17 (game/creatures.rs): PORTED. The ravager roar
  broadcast (event 69) is gone: neither 1.21.4's nor 1.21.5's
  Ravager broadcasts it (`roar()` has no broadcastEntityEvent;
  `handleEntityEvent` handles only event 4), and 1.21.4's
  `Ravager.aiStep` fires the roar from the entity's own tick
  (`roarTick == 10`). The creatures.rs handler skips the LAN
  broadcast for event 69; remote.rs (item 13 follow-up) gates
  the event-69 handler on 1.21.6+ servers and fires the roar
  client-side when `anim_ticks == 10` for 1.21.5-or-older
  servers (ravager.rs, item 107 follow-up, comments only:
  AttributeMap.pack -> save, roar doc, event-69 comment). The
  wolf armor shear gate is the 1.21.5 form: 1.21.4's
  `Wolf.mobInteract` shears branch has NO sneak check — shears
  + owned + wearing body armor + (creative if the armor has
  PREVENT_ARMOR_CHANGE) — so the `!sneaking` gate became
  `(creative || !binding_curse)`. `shear_saddle` is deleted
  (mounts.rs: `Mob.canShearEquipment` exists in neither 1.21.4
  nor 1.21.5 — a 1.21.6 addition), and the
  PLAYER_SHEARED_EQUIPMENT trigger fire is deleted (the trigger
  exists in neither version's `Criteria`). The spawn egg on a
  spawner is the 1.21.5 form (type only): 1.21.4's
  `SpawnEggItem.useOn` calls only `Spawner.setEntityId(type,
  random)` — no data, no op check — so `spawn_egg_on_spawner`
  returns a String (block_entity_item.rs, item 129 follow-up),
  `use_spawn_egg_on_spawner` drops its data param
  (game/spawner.rs, item 152 follow-up), `set_entity_data`
  leaves the spawner trait + impl + its test (mobs/spawner.rs,
  unowned module), `override_entity_data` leaves the trial
  spawner trait + impl (mobs/trial_spawner.rs, item 140
  follow-up), and the dispenser.rs call site takes the 1-arg
  form (it was still calling the old 2-arg signature — a
  compile error caught before the check). The decorated-pot
  crack is flags 4, not 260 (routed from batch 4; 1.21.4's
  `DecoratedPotBlock.onProjectileHit` uses `iconst_4`). The
  LAN drowning broadcast is particles, not event 67 (routed
  from batch 6; 1.21.4's `LivingEntity.baseTick` spawns the
  eight BUBBLE particles directly): creatures.rs drops the
  `lan_entity_event(67)` call, combat.rs's `drown_particles`
  gains a `host: Option<&LanHost>` param and sends each bubble
  to LAN (item 210 follow-up), and lan_actions.rs's guest
  Drown handler calls it with `Some(host)` (item 28
  follow-up); the `DROWN_EVENT` const stays for remote.rs's
  1.21.5+ receive handler. The spawn-egg offspring takes only
  the egg's custom name, after the add (1.21.4's
  `spawnOffspringFromSpawnEgg`: `moveTo` then
  `addFreshEntityWithPassengers`, then CUSTOM_NAME ->
  `setCustomName`; 1.21.5's `snapTo` is byte-identical to
  1.21.4's `moveTo` — a pure rename — and its
  `applyComponentsFromItemStack` puts all components on before
  the add). The breeding coin is drawn from the mother's
  random, and only for Fox/Cat/Wolf: 1.21.4's
  `Chicken.getBreedOffspring` is create-only (no draw, no
  variant), Pig/Cow likewise, while Fox/Cat/Wolf draw
  `this.random.nextBoolean()`; the variant-pick branch now
  covers Fox only (Pig/Cow/Chicken have no variants in
  1.21.4). NO-CHANGE, jar-verified: the potion splash centres
  on the hit location (1.21.4's `ThrowableProjectile.tick`
  calls `setPos(hitResult.getLocation())` before
  `hitTargetOrDeflectSelf`, identical in 1.21.5 — the 1.21.4
  tree is right and the 1.21.5 tree's `p.previous` form
  contradicts BOTH jars, a 1.21.5-port deviation flagged to the
  parent, out of scope here); the guardian's
  `isInWaterOrBubble` vs `isInWater` (the Rust's block facts
  classify bubble columns as water, so `in_water` is already
  true there); the dragon fireball's `setPotionDurationScale
  0.25f` (1.21.5-only, unobservable — the cloud's only effect
  is instant HARM, which has no duration); the thrown egg
  (1.21.4's onHit is `setAge` + `moveTo` + add, no variant);
  the egg's PROJECTILE_SHOOT_POWER (1.5f in both — the
  `use()` difference is literal-vs-constant only);
  `EntityType#by` (string + parse + getOptional vs CODEC —
  same result); `updateCustomEntityTag` (1.21.4 takes a Player
  param with a null check, 1.21.5 a LivingEntity with an
  instanceof check — same op-only behaviour in every path the
  Rust models; the dispenser path is identical in both trees,
  a shared gap, not a port difference);
  `DistanceManager#inEntityTickingRange` (a pure rename:
  `tickingTicketsTracker`/`TickingTracker` vs
  `simulationChunkTracker`/`SimulationChunkTracker`);
  TurtleEggBlock's randomTick (snapTo -> moveTo rename),
  fallOn (double vs float — API-only) and decreaseEggs
  (playSound overload); AmethystBlock, Parrot, WitherBoss,
  EnderpearlItem, SnowballItem, WitherRoseBlock (playSound
  Entity-vs-Player overloads only). DEFERRED to batch 8 (the
  leash-system port, unowned): the LeashBreak `sound` field —
  1.21.4's `Leashable` is the LEGACY system
  (`legacyElasticRangeLeashBehaviour`, 10.0/6.0 distances, 0.4
  pull, identical to 1.21.5; `dropLeash` plays no sound),
  while the 1.21.4 tree carries the 1.21.6 Wrench/spring
  system (SNAP=12, `spring`, `elastic_pull`, quad-leash APIs,
  `HolderState.center`), and the creatures.rs hunk cannot be
  done in isolation (the event shape is shared with leash.rs
  producers). Routed: the potion splash's dead `age` param
  (item 26, potions.rs); chicken/pig/cow variant finalize
  (empty registries in 1.21.4 — item 117, finalize.rs);
  `leads.rs` quad-leash call sites (batch 8 / item 463, rank
  3). The 1.21.5 tree's breed has no chicken-variant
  inheritance (the 1.21.5 jar has it) — a second 1.21.5-port
  deviation, flagged to the parent.

  Item 18 (mobs/tame.rs): NO-CHANGE except one comment carry.
  Wolf sound variants do not exist in 1.21.4 (no
  `wolf_sound_variant` registry data in the 1.21.4 jar;
  1.21.5 has sad/puglin/grumpy/...): `getHurtSound` returns
  WOLF_HURT directly (the species table's
  "entity.wolf.hurt" — correct), `defineSynchedData` uses the
  WOLF_VARIANT registry with a RED default collar (the Rust
  defaults to `tame::RED` when CollarColor is absent), and
  `Wolf#get` (the 1.21.5-only component accessor) has no Rust
  equivalent. `getTexture` is the 1.21.4 direct form
  (`tameTexture`/`angryTexture`/`wildTexture` — the Rust's
  `tame::texture` builds `entity/wolf/{file}{_tame|_angry}
  {_baby}`; 1.21.5's assetInfo resolves to the same paths).
  `readAdditionalSaveData` reads "variant" as a string and
  "CollarColor" as an int (entity_nbt.rs does both, with the
  RED default). `tick`'s `isInWaterRainOrBubble` collapses to
  the Rust's `in_water || in_rain` (bubble columns are water
  in the block facts). The one tree-diff hunk, a test comment,
  takes the 1.21.5 form: vanilla writes `AngerTime`/`AngryAt`
  in both versions. ROUTED (found here, owned elsewhere): the
  sound-variant assignment is wrong for 1.21.4 in TWO places
  — finalize.rs's `sound()` (item 117) assigns
  `Some(r.random())` unconditionally, and `random()` returns
  0 for an empty registry, so every 1.21.4 wolf gets
  sound_variant 0 = "angry" (the hardcoded list is sorted
  alphabetically: angry, big, classic, ...); spawning.rs's
  `finalize` (item 188) assigns from the HARDCODED list length
  (`sound_variants(species).len()`), so every spawned wolf/pig/
  cow/chicken/cat gets a random 1.21.5 sound variant. The fix
  (don't assign when the version has no such registry) and the
  entity_nbt.rs sound_variant save/load question (item 11 is
  closed; the save writes "sound_variant" whenever the species
  has a hardcoded list, defaulting to "classic") belong to
  items 117/188.
  `cargo check -p horizon-client -p horizon-world
  -p horizon-client-input` green (EXIT 0, 2m44s; the first run
  failed on a missing `LanHost` import in combat.rs, fixed)
  and the --tests check green (EXIT 0, 6m19s, 0 errors). The
  nine rustfmt-checked leaf files are clean (ravager.rs,
  mounts.rs, block_entity_item.rs, mobs/spawner.rs,
  mobs/trial_spawner.rs, combat.rs, lan_actions.rs,
  dispenser.rs, tame.rs); creatures.rs, remote.rs and
  game/spawner.rs are module roots and were not formatted.
  Rank 2: 65/348.

  Batch 8 (the leash-system port, unowned — deferred from item
  17; closes no numbered item). 1.21.4's `Leashable` is the
  LEGACY system, identical to 1.21.5's (jar-verified:
  `legacyElasticRangeLeashBehaviour`, LEASH_TOO_FAR_DIST=10.0,
  LEASH_ELASTIC_DIST=6.0, 0.4 pull; `dropLeash` plays no
  sound), while the 1.21.4 tree carried the 1.21.6 Wrench/
  spring system (SNAP=12, `spring`, `elastic_pull`, quad-leash
  APIs, `HolderState.center`). Carried the 1.21.5 legacy system
  wholesale: mobs/leash.rs, game/boat_leads.rs and
  game/leads.rs copied from the 1.21.5 tree (version mentions
  in comments adapted 1.21.5 -> 1.21.4; rustfmt drift in the
  three copied files cleaned — 9 small hunks). `MobEvent::
  LeashBreak` loses its `sound` field (mobs.rs, item 55
  follow-up) and `break_leash` loses its third arg
  (creatures.rs handler + bucket call — completing item 17's
  deferred hunk — and the call sites in beehives.rs,
  boat_passengers.rs, copper_golems.rs, knot_identity_tests.rs,
  llamas.rs, minecarts.rs, nautili.rs, sulfur_cubes.rs).
  equine.rs and camel.rs gain `handle_leash_at_distance`
  (the 1.21.5 form, distance > 6.0 gate inside) in place of
  the dead `on_elastic_leash_pull` (its only caller was the old
  leash.rs). Two methods the old leads.rs had are 1.21.6-only
  and were removed with their call sites, jar-verified:
  `drop_all_leash_connections` (neither 1.21.4's nor 1.21.5's
  `FireworkRocketItem.use` calls it — the leash-drop block
  leaves boost_firework) and `forget_untied_knots`
  (`notifyLeasheeRemoved` exists in neither version — a knot
  outlives its last lead, so lan_entities.rs drops the call).
  Dead code the port left behind is removed: `Boat.leash_spin`
  (`LeashData.angularMomentum`) and `Boat.in_liquid`
  (for `Leashable.angularFriction`) — neither exists in the
  1.21.5 tree's boats.rs. Routed: equine.rs's stand-counter
  hunks (counter 1 vs 20, counting up vs down — a separate
  behaviour difference, not leash) to item 205; the chest-boat
  sneak hunk in boats.rs (1.21.4's `AbstractChestBoat.interact`
  checks `isSecondaryUseActive` first, matching the 1.21.5
  form) to item 57; leads.rs's carry to item 463 (rank 3 —
  the quad sites are gone, the rank-3 agent verifies the rest);
  camel.rs's leash hunk to item 395 (rank 3 — it was camel.rs's
  only tree diff). `cargo check -p horizon-client
  -p horizon-world -p horizon-client-input` green (EXIT 0,
  0 warnings) and the --tests check green (EXIT 0, 7m18s).
  Rank 2: still 65/348 (batch 8 closes no numbered item).

  Batch 9 (items 19 + 20 + 21). Item 19 (game/native_hooks.rs):
  NO-CHANGE, comment-only carry. The tree diff is 46 comment
  lines naming 1.21.6 features (the dialog system, the locator
  bar, the `now_playing` toast, the happy ghast) that do not
  exist in 1.21.4; the 1.21.5 tree's comments are correct for
  1.21.4, so the file was copied wholesale (12,786 lines, no
  code diff). The jar diffs are API-only or modelled elsewhere:
  `destroyBlock`'s `canDestroyBlock` vs `canAttackBlock` (item
  119, already routed); the BLOCKS_ATTACKS component in
  `getUseAnimation`/`getUseDuration`/`use` (items 126/55);
  the enderpearl `unRide` (already modelled — `stop_riding`
  before the teleport in creatures.rs); `handleBlockBreakAction`
  / `useItemOn` / `BlockItem#canPlace` / `Item#hurtEnemy` /
  `Level#playSound` (overload and accessor reshaping, same
  behaviour).

  Item 20 (game/minecarts.rs): PORTED. `prime_cart` and
  `explode_cart` drop their `tnt_explodes` gamerule gates:
  `RULE_TNT_EXPLODES` does NOT exist in 1.21.4 — it was added
  in 1.21.5 (verified: the 1.21.4 jar has no such field in
  `GameRules`, and no references to it in `MinecartTNT`,
  `PrimedTnt`, `TntBlock`, `ServerLevel` or `Explosion`, while
  1.21.5's `TntBlock` has 3 and its `MinecartTNT.primeFuse` /
  `explode` check it). In 1.21.4 a TNT minecart primes and
  explodes regardless of any gamerule. Two comment carries:
  `CompoundTag`'s (not `ValueInput`'s) numeric getters, and
  `cart_lava_tick` no longer names `lavaIgnite` (absent in
  1.21.4 — the cart takes `lavaHurt` twice and never burns
  because it has no `baseTick`). NO-CHANGE, jar-verified: the
  fall damage (`(fall / 10)^2` at fall >= 3); the display
  offset/block (the Rust returns the default without a custom
  display — the 1.21.4 `hasCustomDisplay` form; display.rs is
  identical in both trees); the destroy fuse (two
  `nextInt(20)` draws); the render `mulPose`, `getDisplay-
  BlockState`, `causeFallDamage` and `primeFuse()` shapes
  (API-only); the `getKnownMovement` NaN guard and the
  interpolation check (unmodelled). SHARED GAP (both trees,
  not a port difference): `hurt_vehicle` passes `fire=false`,
  so a burning projectile never primes a TNT cart — 1.21.4's
  `hurtServer` checks any `Projectile`, 1.21.5's only
  `AbstractArrow`, both requiring on-fire plus a speed
  threshold; the `Projectile` struct has `on_fire()`, so the
  fix is feasible later. ROUTED: the cross-cutting
  `tnt_explodes` removal — the rule is defined in
  gamerules.rs (no item) and read in 13 files (arrows.rs,
  game.rs, chat.rs, creatures.rs, dispenser.rs, explosion.rs,
  sulfur_cubes.rs, tnt.rs, sulfur_cube.rs, save.rs,
  datafix/fixes_c.rs, plus the minecarts.rs sites done here)
  — to item 66 (tnt.rs) as its natural home.

  Item 21 (raid.rs): NO-CHANGE. The tree diff is empty, and
  every jar diff is API-only: `level` as a method parameter
  (1.21.5) vs the `this.level` field (1.21.4) in
  `findRandomSpawnPos`, `playSound`, `spawnGroup`, `tick`,
  `updateRaiders`, `createOrExtendRaid` and `Raids.tick`;
  `raidMap` as fastutil `Int2ObjectMap` vs `java.util.Map`;
  `getNumGroups` returns the same 3/5/7 for EASY/NORMAL/HARD
  (only the switch compilation differs);
  `getOminousBannerInstance` hides the banner patterns via
  `TOOLTIP_DISPLAY.withHidden` (1.21.5) vs
  `HIDE_ADDITIONAL_TOOLTIP` (1.21.4) — same result, and the
  banner item itself is unmodelled in raid.rs;
  `findRandomSpawnPos` is local-slot renumbering plus
  instruction reordering with identical operations and
  constants (0.22f, PI, 32.0f, the same random draws).
  `cargo check -p horizon-client -p horizon-world
  -p horizon-client-input` green (EXIT 0, 1m28s) and the
  --tests check green (EXIT 0, 3m31s, 0 errors). minecarts.rs,
  native_hooks.rs and minecarts/display.rs are rustfmt-clean.
  Rank 2: 68/348.

  Batch 10 (items 22 + 23 + 24). Item 22 (item_tooltip.rs): PORTED.
  Tooltip hiding is the core 1.21.4-vs-1.21.5 system difference:
  1.21.5 uses the TooltipDisplay component, 1.21.4 uses HIDE_TOOLTIP
  / HIDE_ADDITIONAL_TOOLTIP unit components plus per-component
  showInTooltip flags (verified: 1.21.4's DataComponents has both
  HIDE components and has NO WEAPON and NO TOOLTIP_DISPLAY
  component). TooltipDisplay gains a hide_tooltip field, set in
  for_stack from minecraft:hide_tooltip; hides_tooltip() now returns
  hide_tooltip OR the tooltip_display check (kept, since a 1.21.5+
  server can still send it). In 1.21.4 play stacks never carry
  tooltip_display, so the old gating was a no-op; hide_tooltip now
  hides all gated lines (lore, bees, honey, fireworks, jukebox,
  enchantments, dyed, ominous bottle, bundle, instrument, tropical
  fish, map, spawn egg, block state, written book). Comment carry:
  ItemStack.addModifierTooltip (the 1.21.5 form; the tree named the
  1.21.6 ItemAttributeModifiers.Display.Default.apply). NO-CHANGE:
  the EQUIPPABLE equipOnInteract branch and the WEAPON component
  (both 1.21.5-only, unmodelled), PotionContents.addPotionTooltip
  (the Rust's brewing::tooltip builds the same "EffectName II
  (duration)" output), inventoryTick / onCraftedBy (API-only).
  Recorded gaps, not fixed: hide_additional_tooltip (needs the
  appendHoverText/component split the Rust does not model — the
  details fn mixes both); trim, pot decorations, potions and
  attributes do not check shows(); lore ordering (Rust puts lore
  right after the name, 1.21.4 after enchantments/dyed) is a shared
  deviation in both trees.
  Item 23 (save.rs): PORTED. (1) The player respawn point is FLAT
  fields in 1.21.4 — SpawnX/SpawnY/SpawnZ (int), SpawnAngle (float),
  SpawnForced (bool), SpawnDimension (string) — not the 1.21.5
  respawn compound (RespawnData does not exist in 1.21.4; verified:
  no RespawnData class, ServerPlayer saves the flat fields).
  read_player and player_tag rewritten to the flat form; the stale-
  keys list drops the flat keys (plus the 1.21.5 compound,
  defensively). (2) Forced chunks live in data/chunks.dat, not
  chunk_tickets.dat: 1.21.4 has ForcedChunksSavedData (FILE_ID
  "chunks", TAG_FORCED "Forced", a long array of ChunkPos.asLong
  values under data/, no minecraft/ subfolder) and no TicketStorage
  at all (1.21.5 has TicketStorage and no ForcedChunksSavedData).
  load_forced_chunks reads the Forced long array; save_chunk_tickets
  is renamed save_forced_chunks and drops its portal param;
  load_portal_tickets returns empty (portal tickets are memory-only
  in 1.21.4); is_forced_ticket / is_portal_ticket removed; the
  portal_tickets_fixture probe is deleted (its case —
  chunk_tickets.dat persistence — does not exist at this pin; no
  other tree reference); saving.rs call sites updated; the
  PortalTicket doc fixed. The forced-chunks test is rewritten for
  chunks.dat (the foreign-ticket preservation it asserted is a
  1.21.5 TicketStorage behaviour with no 1.21.4 counterpart). (3)
  Legacy numeric dimensions carried: DimensionType.parseLegacy
  exists in BOTH 1.21.4 and 1.21.5, so read_player now maps -1/0/1,
  with the 1.21.5 test carried. (4) The test_instance_block POI
  mapping removed (no test instance blocks in 1.21.4). (5) Conduit
  comments fixed to putUUID/getUUID (the code already wrote/reads an
  int array of four — correct either way). NO-CHANGE:
  world_gen_settings (level.dat only — correct for 1.21.x),
  FallDistance (Float — correct). ROUTED: needs_file_fixes / the
  file_fixing backup prompt to item 400 (title.rs, rank 3) — the
  1.21.4 tree's title.rs has no file_fixing at all, so the fn
  belongs with that feature; enteredNetherPosition persistence
  (modelled in-memory as adv.marks.nether, not persisted to
  player.dat — a cross-module follow-up; the field is a 3-double
  Vec3 list in 1.21.4's player.dat). The playerdata.rs
  PrepareSpawnTask comment now names SpawnAngle (not
  respawnData.yaw/pitch).
  Item 24 (game/stations.rs): PORTED. The tree diff is empty and the
  jar diffs are the 1.21.5 TooltipDisplay refactor moving tooltip
  code from item classes into components — output unchanged (spot-
  checked: 1.21.4's WrittenBookItem.appendHoverText emits
  book.byAuthor + generation, identical to 1.21.5's
  WrittenBookContent.addToTooltip; Item.appendHoverText is a no-op
  in both jars, so the smithing-template super-call difference is
  nil; FireworkExplosion is the addShapeNameTooltip helper vs
  inline, same output). The one real difference: JukeboxPlayable is
  a {song, show_in_tooltip} record, and 1.21.4's addToTooltip checks
  showInTooltip itself (1.21.5 leaves it to TooltipDisplay).
  jukebox_line (item_tooltip.rs) now reads the song from the record
  and honours show_in_tooltip, and jukebox.rs jukebox_playable is
  fixed the same way — both had misread the record compound as an
  inline song (no description key, so the tooltip line never showed
  and the jukebox lost the song id for an explicitly stored disc;
  plain discs still worked through the item default).
  game/jukebox.rs has no item at any rank, so its fix lands here.
  ItemLore / OminousBottleAmplifier / SuspiciousStewEffects
  addToTooltip are bytecode-identical. Lib check green (EXIT 0,
  2m14s); --tests check green (EXIT 0, 3m14s). Rank 2: 71/348.

  Batch 11 (items 25 + 26 + 27). Item 25 (menu.rs): NO-CHANGE. The
  tree diff is empty. The Inventory compartments difference
  (1.21.4's List<NonNullList> vs 1.21.5's flat items +
  EntityEquipment) is structural only: the Rust models the player
  inventory as a flat 36-slot vec plus separate armor/offhand
  fields, never as compartments, so contains / dropAll / setItem /
  tick all agree in behaviour, and native_inventory_tick(slot,
  selected) is already the 1.21.4 form (1.21.5 passes an
  EquipmentSlot). AnvilMenu's hasInfiniteMaterials vs
  getAbilities().instabuild is the same value (the Rust passes
  creative through; cost >= 40 && !creative). The onCraftedBy
  signature in the result-slot achievements is API-only. MapItem's
  onCraftedPostProcess / scaleMap run on any Level in 1.21.4 and
  require a ServerLevel in 1.21.5 — the Rust runs post-processing
  server-side with no gate, matching 1.21.4.
  Item 26 (game/potions.rs): PORTED. (1) The splash system is
  carried from the 1.21.5 tree, jar-verified against 1.21.4's
  ThrownPotion.applySplash: entities in the potion's box inflated
  (4, 2, 4), distanceToSqr from the potion's position with a 16
  cutoff, full strength for the directly hit entity else 1 -
  sqrt(d2)/4. splash_intensity(from, direct, pos, bounds) replaces
  splash_distance_sqr (the 1.21.6 box-to-box + age-margin form); the
  landed tuple carries from (t.position) instead of age, which also
  resolves the dead-age routing from item 17. ENTITY_HIT_MARGIN
  (0.3) is added to projectiles.rs —
  DEFAULT_ENTITY_HIT_RESULT_MARGIN is 0.3f in both 1.21.4 and
  1.21.5, and computeMargin exists in neither. The creatures.rs
  splash call site passes at as from (the potion's position at the
  hit IS the hit location after setPos — the 1.21.5 tree's
  p.previous there contradicts both jars, as batch 7 found); the
  xp_bottle test call site is updated the same way. (2) The cloud
  particle is carried: the Particle key + load_particle (updateColor
  recolour) + particle_options replace the 1.21.6 custom_particle
  form (both jars use "Particle"). (3) Cloud NBT load defaults are
  1.21.4's plain getInt/getFloat zeros (Radius 0, Duration 0,
  WaitTime 0, ReapplicationDelay 0), not 1.21.5's getOr defaults.
  (4) There is no potion_duration_scale in 1.21.4 — no such
  DataComponent, no cloud field: the load pre-scales a named
  potion's effects by /4 (the mapDuration lambda, plain integer
  division) and sets duration_scale 1.0 so custom effects stay
  unscaled; the save drops the key; the round-trip test is updated.
  (5) The creeper cloud is unscaled: its effects reach the cloud
  through addEffect, i.e. custom effects, which 1.21.4 applies
  unscaled (1.21.5's setPotionDurationScale(0.25) is the 1.21.5
  form). (6) Both duration != -1 guards are removed from
  tick_potion_cloud: 1.21.4 has neither, so an infinite cloud
  discards at age >= wait - 1 and a duration on use makes it finite.
  NO-CHANGE: the InfestedMobEffect snapTo/moveTo rename; the
  MobEffectInstance tick split (the Rust has a single tick); the XP
  bottle's position() vs getLocation() (the same point after
  setPos); the playSound / mayInteract overloads; the
  ThrowablePotionItem factory. ROUTED: the 1.21.6 computeMargin ramp
  also leaks into arrows.rs (item 36), fireworks.rs, fishing.rs and
  trident.rs — their items verify. The 1.21.5 files potions.rs,
  cloud_particle.rs and cloud_nbt.rs were copied wholesale, then the
  1.21.4 edits above applied; rustfmt drift in the two copied leaf
  files cleaned (potions.rs is a module root and was not formatted).
  Item 27 (widely named: network/codec): NO-CHANGE. 20 vanilla
  classes (all anonymous ByteBufCodecs$N inner classes), 153
  methods, 0 named in Horizon; module_status missing-both — no
  module file in either tree, nothing to port. Codec behaviour lives
  in horizon-protocol, mechanically ported for 769. Lib check green
  (EXIT 0, 1m26s); --tests check green (EXIT 0, 1m45s). Rank 2:
  74/348.

  Batch 12 (items 28 + 29 + 30). Item 28 (lan_actions.rs):
  PORTED, and it surfaced two 769 wire-format bugs the mechanical
  layer had missed. (1) The missing-respawn yaw carry: the 1.21.4
  tree now sets the guest's yaw to 0.0 when no respawn block is
  found, as the 1.21.5 tree does (TeleportTransition.
  missingRespawnBlock is byte-identical in both jars). The jar also
  sets pitch 0, but neither tree resets pitch — a shared gap.
  (2) UpdateAdvancements: 1.21.4's packet has exactly four fields
  (reset, added, removed, progress); the trailing showAdvancements
  bool is a 1.21.5 addition, and the 769 codec was reading it,
  desyncing every advancement packet. The show field is gone from
  the codec, from the manual LAN writer (lan/advancements.rs
  ::update loses its show param; hub.rs::flush_advancements loses
  its show param; both callers passed true) and from the toast gate
  in game/advancements.rs (1.21.4 toasts on completion, subject to
  show_toast + toast_allowed). (3) ContainerClick: 1.21.4 sends full
  ItemStacks for the changed slots and the carried item (the map
  codec is SHORT keys + ItemStack.OPTIONAL_STREAM_CODEC); 1.21.5
  sends HashedStack hashes. The codec now uses Slot, remote.rs
  sends to_network stacks directly (hashed_stack deleted, with its
  Hashed + SyncedNames imports), and lan/containers.rs compares the
  claimed full stacks (hashed_matches -> claimed_matches, item +
  count as before). The 769 adapter steps are updated to match:
  step_1_21_4's up-step passes clicks through with id remapping
  instead of hashing, and translate/v1_21_4's test literal is a
  full stack now (its down-step still drops the predictions).
  Hashed/HashedSlot stay in item.rs: the custom-component wire test
  still uses Hashed::create as a comparison vehicle. Verified that
  the tree's own paths encode the internal structs raw
  (net.rs Session::send does w.send(&packet.encode()), no
  translation), so the internal layout must be 769 — the fix is
  required, not optional. NO-CHANGE: the PlayerCommand action
  numbers (1.21.4 has no PRESS/RELEASE_SHIFT_KEY, so 5/6/3/4/1/2
  are right and the crouch test via PlayerInput flags is right);
  the TestInstanceBlockAction / SetTestBlock handlers (correctly
  absent — handleTestInstanceBlockAction is 1.21.5-only); the
  handleMovePlayer awaiting-teleport early return; handlePickItem-
  FromEntity (getEntity, no part resolution); handleSetCreative-
  ModeSlot, handleUseItemOn, tick, tryPickItem (API-only);
  handleBundleItemSelectedPacket (threading boilerplate).
  ROUTED: the structure-block strict flag (a 1.21.5 addition) to
  item 394 (rank 3) — the packet bit is wire-harmless (a 1.21.4
  client never sets bit 8 of the flags byte), but the entity field,
  the edit-screen STRICT toggle and the load_settings param need
  removing there. KNOWN GAP (programme-level, recorded not fixed):
  the older-clients adapter chain (client_session.rs) still applies
  the 770->769 steps at this 769 pin — newer_layout_down runs every
  step for a 1.21.4 peer, so the show-flag strip and the other
  770-layout arms would corrupt 769 payloads; the chain needs
  gating on the pin (or the older-clients feature dropping for
  ported trees). accept_older_clients is off by default and the
  feature is admitted unfinished, so this bites nobody today.
  Item 29 (widely named: nbt): NO-CHANGE. 33 vanilla classes, 145
  methods, 0 named in Horizon; module_status missing-both — the
  same shape as item 27. Item 30 (fox.rs): NO-CHANGE, comment-only.
  isBrightOutside does not exist in 1.21.4, and 1.21.5's form is
  byte-identical to 1.21.4's isDay (!hasFixedTime && skyDarken < 4),
  so the StrollThroughVillageGoal and getAmbientSound differences
  are pure renames. Fox save/load already match 1.21.4 (Type string,
  Trusted as a list of int-array UUIDs, boolean flags); addTrusted-
  UUID's slot behaviour verified against the jar (slot 0 if free,
  else replaces slot 1). Comment carries only: getTrustedEntities
  -> getTrustedUUIDs, addTrustedEntity -> addTrustedUUID, no
  clearTrusted in 1.21.4, TRUSTED_LIST_CODEC -> the manual
  NbtUtils.createUUID list (fox.rs + entity_nbt.rs). CaveVines#use
  is a playSound overload; registerGoals is an ldc/ldc_w
  constant-pool difference. Lib check green (EXIT 0, 2m19s);
  protocol --tests check green (EXIT 0, 3m37s); full --tests check
  green (EXIT 0, 59m34s — slow: the incremental cache had been
  deleted to free quota, and the VM was contending with another
  agent's horizon-prepare). The first --tests run failed E0308 on
  the translate/v1_21_4 test literal (Hashed carried stack); the
  retry hit the disk quota (104G used) and was unblocked by deleting
  the 1.21.4 target/debug/incremental cache (4.3G, regenerable).
  rustfmt: 5 of 7 leaf files clean; remote.rs had one drift hunk in
  the edited region (fixed by hand) and one pre-existing drift hunk
  at 5551 (left alone); tests.rs has one pre-existing drift hunk at
  926 (left alone). Rank 2: 77/348.

  Batch 13 (items 31 + 32 + 33). Item 31 (villager_brain.rs):
  NO-CHANGE. The tree diff is empty and every jar difference is
  the Holder-vs-direct profession/type API (customServerAiStep,
  playWorkSound, registerBrainGoals, setVillagerData,
  wantsToPickUp) with identical behaviour. getTypeName: 1.21.4
  builds entity.minecraft.villager.<profession-path>, exactly what
  Mob::command_name emits. getBreedOffspring was undetermined by
  the differ and was verified by javap: one nextDouble, <0.5 the
  biome's type, <0.75 the mother's, else the father's — exactly the
  Rust form. HarvestFarmland#tick and SecondaryPoiSensor#doTick
  are bytecode-identical. Item 32 (block_entity_nbt.rs): PORTED.
  (1) The loadAllItems slot default is carried: a missing Slot
  reads 0 (getByte), so the 1.21.4 tree's drop of such entries was
  wrong; unparseable stacks drop (orElse(EMPTY)). (2) The
  TrialSpawner comment is carried (TrialSpawnerData exists in
  1.21.4; TrialSpawnerStateData.Packed is the 26.3 form).
  (3) canonical() no longer writes a "components" key at all:
  1.21.4's saveWithoutMetadata MERGES the encoded components into
  the tag itself (the lambda calls tag.merge), while 1.21.5 stores
  them under "components" — so a 1.21.4 save never carries the key
  and neither tree's form was right (the 1.21.4 tree always wrote
  an empty compound; the 1.21.5 tree omitted it only when empty).
  The every_type_has_defaults test now asserts no components key.
  (4) The structure-block "strict" flag is removed from
  canonical() (no strict field in 1.21.4's StructureBlockEntity;
  the entity/UI side stays routed to item 394). NO-CHANGE:
  CompoundTag#merge (same recursive behaviour; no Rust equivalent
  in this module), LockCode#addToTag (encode+put vs store, the same
  "lock" compound — fromTag reads the "lock" key in both versions),
  tryLoadLootTable (codec vs manual reads, same values), the skull's
  lowercase "custom_name" (jar-verified: putString "custom_name"),
  CustomName top-level strings (BaseContainerBlockEntity writes
  them with putString in 1.21.4). KNOWN GAP: canonical() builds
  from scratch, so top-level component keys from a 1.21.4 save are
  dropped rather than merged back. Item 33 (nbt_path.rs):
  NO-CHANGE, comment-only. 1.21.4 has no NumberTag/intValue
  /longValue — the numeric tags extend NumericTag with getAsInt/
  getAsLong — but the semantics are identical (Double getAsInt/
  getAsLong floor, Float getAsInt floors, Float getAsLong
  truncates), exactly the Rust int_value/long_value forms; only the
  doc comments named the 1.21.5 methods. compareNbt (entrySet vs
  getAllKeys), getDataVersion (contains+getInt vs getIntOr) and
  readBlockState (AIR default when Name is missing, same
  properties loop) are API-only with identical behaviour. Lib check
  green (EXIT 0, 2m06s); --tests check green (EXIT 0, 3m08s).
  Both edited leaf files rustfmt-clean. Rank 2: 80/348.
  Batch 14 (items 34 + 35 + 36). Item 34 (animate_tick.rs):
  PORTED. The ambient desert sound system does not exist in 1.21.4
  (no AmbientDesertBlockSoundsPlayer in the client jar, no desert
  sound tags in 1.21.4 data, no DryGrass blocks, DeadBushBlock has
  no animateTick, block.sand.idle in neither sounds.json) — removed
  wholesale, not carried: Animate::Falling loses its sand flag,
  the DeadBush/DryGrass variants and their tick arms are gone, the
  desert_sand_sounds / desert_dry_vegetation_sounds / terracotta
  tables are gone, sand_sound_nearby and
  desert_dry_vegetation_sounds are gone, and block_animate_tick
  loses its orphaned player param (the DryGrass arm was its only
  user). Neither tree's form was right for 1.21.4 (the 1.21.4 tree
  carried the 1.21.6 three-tag system, the 1.21.5 tree the 1.21.5
  single-tag one). NO-CHANGE: the SoundSource AMBIENT-vs-BLOCKS
  differences (eyeblossom, hanging moss, lava/water fluids) are
  unobservable — sound() carries no category; Eyeblossom
  tryChangingState is unmodelled (1.21.5 uses
  CreakingHeartBlock.isNaturalNight, 1.21.4 ServerLevel.isDay with
  the open-check inverted); leaf drip (LeavesBlock
  #makeDrippingWaterParticles, missing-new) is unmodelled; lava/
  water entityInside lives in entities.rs/fluids.rs, not here.
  ROUTED to item 80: the lava randomTick fire gate — 1.21.4 has
  only doFireTick, no RULE_ALLOWFIRETICKAWAYFROMPLAYERS, and the
  Rust models no doFireTick at all (can_spread_fire_around gates
  only on the nonexistent fire_spread_radius_around_player rule,
  default -1 = always); item 82 owns the fire tick's shared gate.
  Item 35 (mesher/fluid.rs): NO-CHANGE, comment-only. The only
  named method is Shapes#blockOccludes [missing-new], which is
  1.21.4 Shapes#blockOccudes renamed (the typo was fixed in
  1.21.5) — 1.21.4 LiquidBlockRenderer.isFaceOccludedByState calls
  blockOccudes with the same logic (empty-shape and full-block
  shortcuts, fluid box vs face occlusion shape). The Rust's
  coverage() implements the test directly from shape boxes; only
  its doc comment named the 1.21.5 spelling. Item 36 (arrows.rs):
  PORTED. (1) The computeMargin ramp is removed — flat
  ENTITY_HIT_MARGIN (0.3) from the first tick; computeMargin
  exists in neither jar (routed from batch 11). (2) The
  powder-snow fire clear is carried: 1.21.4 applyEffectsFromBlocks
  extinguishes via isInPowderSnow, which the arrow's
  apply_block_effects does not model (Extinguish only for water),
  so a direct is_powder_snow(state) arm is added. (3) The
  contain-check no longer zeroes velocity — setDeltaMovement(ZERO)
  after setInGround is a 1.21.5 addition; 1.21.4 only sticks (the
  in-ground branch returns before any move, so the retained
  velocity is inert until the arrow falls out). (4)
  is_pushed_by_fluid now always returns true — AbstractArrow has
  no override in 1.21.4 (the !isInGround override is 1.21.5);
  behaviourally a no-op today, since the stuck-arrow tick returns
  before the fluid tick. (5) The bubble column: the gate is
  isAir above (1.21.4), not the 1.21.5 collision-shape/fluid
  gate, and the velocities are clamped per onAboveBubbleCol
  (drag max(-0.9, vy-0.03), lift min(1.8, vy+0.1)) and
  onInsideBubbleColumn (drag max(-0.3, vy-0.03), lift min(0.7,
  vy+0.06), fall reset). (6) The FireIgnite comment is carried
  (the inline entityInside form; igniteForTicks is a max, matching
  the Rust). NO-CHANGE verified: EnderMan hurtServer (the
  ThrownPotion class rename only); the arrow save/load forms match
  1.21.4's manual NBT (pickup byte ordinal, damage an
  unconditional double, SoundEvent a registry-key string, item/
  weapon via save, item falling back to getDefaultPickupItem);
  ARROW_HIT_PLAYER is already the 1.21.4 name in remote.rs; the
  on-fire flag sync is covered by synced_fire. The tnt_explodes
  refs stay, routed to item 66. Lib check green (EXIT 0, 2m08s);
  --tests check green (EXIT 0, 3m21s). All three edited files
  rustfmt-clean (arrows.rs checked, not reformatted — it declares
  the client_tests file module). Rank 2: 83/348.
  Batch 15 (items 37 + 38 + 39 + 40). Item 37 (mobs/render.rs):
  PORTED. jeb_wool_rgb gains the entity-id offset (ticks/25 + id)
  — jar-verified in 1.21.4 SheepWoolLayer.render (floor(ageInTicks)
  /25 + state.id, mod 16, ARGB.lerp with (age%25 + frac)/25); the
  1.21.4 tree carried the 1.21.6 ColorLerper form without the id
  (1.21.5 moved the computation to SheepRenderState.getWoolColor
  with the same formula). The caller passes m.id and the tests are
  extended (id 1 starts at orange, id 16 wraps to white). NO-CHANGE:
  the mulPose Quaternionfc-vs-Quaternionf renames (AllayModel,
  CrossedArmsItemLayer) are API-only; the four setupRotations,
  SheepRenderer.extractRenderState and WitchItemLayer are
  bytecode-identical; the updateForLiving left-hand bool is
  internal to the Java item-model resolver (the Rust already
  distinguishes hands). Item 38 (game/maps.rs): NO-CHANGE, tree
  diff empty. CompassItem inventoryTick/useOn and EmptyMapItem.use
  are API-only (ServerLevel cast vs isClientSide, playSound
  overloads, local-slot reshuffles). The MapItemSavedData setDirty
  differences (checkBanners, tickCarriedBy, toggleBanner call
  setDirty in 1.21.5, not in 1.21.4) are server-save concerns with
  no Rust equivalent — the Rust models banners/frames as render
  data only. MapFrame entityId() vs getEntityId() is a
  record-accessor rename; ExplorationMapFunction's MapItem.create
  param type is API-only. Item 39 (game/dispenser.rs): PORTED.
  (1) shear_off_leashes removed — tryShearLivingEntity is
  byte-identical in both jars (Shearable + game event, no leash
  handling); the 1.21.6 leash-cutting arm does not exist in 1.21.4.
  (2) mob_can_use_slot rewritten to the 1.21.4 form (routed from
  batch 6): the LivingEntity base returns false, Mob allows every
  slot but BODY, Horse/Llama/Wolf allow them all; the
  Pig/AbstractHorse saddle arms (grown/tame) are 1.21.5 additions,
  and skeleton/zombie horses lose the body slot (no override —
  they cannot wear horse armour in 1.21.4). (3)
  mob_can_dispenser_equip rewritten: Pig/Strider have no override
  in 1.21.4 (just canPickUpLoot), AbstractHorse is (BODY && tamed)
  || loot — the saddle arms are 1.21.5, so a 1.21.4 dispenser
  cannot saddle a pig, strider or horse (Animal/AgeableMob/
  PathfinderMob add no override either). (4) Arrow
  #doPostHurtEffects in potions.rs: the named potion's effects at
  an eighth (integer division, min 1 tick), the custom effects
  UNSCALED — the Rust scaled every effect by
  potion_duration_scale, which does not exist in 1.21.4 (the
  lambda is i/8, verified). (5) The setGuaranteedDrop comment is
  carried (1.21.4: setDropChance(slot, 2.0F), verified equivalent
  via DropChances.withGuaranteedDrop). NO-CHANGE: MobBucketItem
  #spawn (createDefaultStackConfig param type), the two same-
  methods; the Player canUseSlot comment is fixed (every slot but
  BODY). Item 40 (game/frame_geometry.rs): NO-CHANGE, tree diff
  empty. Every SkyRenderer difference is render-pipeline API with
  identical geometry: buildEndSky/buildStars build GPU buffers in
  1.21.5 vs writing into a passed VertexConsumer in 1.21.4 (same
  vertices: sky disc at +/-100 with UV 16, 1500 stars from the
  same random draws); renderSkyDisc/renderStars use RenderPass/
  GpuBuffer vs VertexBuffer.drawWithRenderType; the sun/moon/
  sunrise diffs are mulPose signature renames. The Rust models
  frames/transforms only, with no buffer-API references. Lib check
  green (EXIT 0, 2m01s); --tests check green (EXIT 0, 3m16s).
  render.rs and dispenser.rs rustfmt-clean; potions.rs has only
  two pre-existing drift hunks (lines 260, 2004) outside the
  edited regions. Rank 2: 87/348.
  Batch 16 (items 41 + 42 + 43 + 44). Item 41 (game/raids.rs):
  NO-CHANGE, tree diff empty. bright_outside() is !hasFixedTime
  && skyDarken < 4 — exactly 1.21.4's isDay form (1.21.5's
  isBrightOutside is the same formula, a pure rename; the comment
  now says so). PatrolSpawner.tick and tick_village_siege return
  () — the 1.21.4 boolean/count returns are unmodelled and
  unobservable. The raider RaidId is saved directly as Int
  (raider.rs:713), the 1.21.4 form (1.21.5 looks it up via
  Raids.getId). PathfindToRaidGoal.recruitNearby, VillageSiege
  #trySpawn (snapTo vs moveTo, same), Raider.aiStep/die and
  pickUpItem are API-only or same-logic. Item 42
  (game/commands.rs): PORTED. (1) /setblock and /fill now notify
  neighbours via update_neighbors_at — 1.21.4's
  ServerLevel.blockUpdated, plain neighbour notification —
  replacing the 1.21.6 update_neighbours_on_block_set
  (affectNeighborsAfterRemoval exists in neither 1.21.4's nor
  1.21.5's Block class; 1.21.4's Level.setBlock with flags=2 runs
  neither blockUpdated nor the analog-output update, and the
  commands send the notice themselves). The commands.rs wrapper
  is replaced (it now reads the current state for the block);
  fill's updated list is plain positions, no old states. (2)
  strict REMOVED from /setblock and /fill: jar-verified a 1.21.5
  addition (0 occurrences of "strict" in 1.21.4's
  SetBlockCommand, FillCommand and CloneCommands; present in
  1.21.5's; 1.21.4's setBlock has no boolean param and
  FillCommand$Mode has only the filter field). block_input_state
  always normalizes (1.21.4's BlockInput.place does
  updateFromNeighbourShapes + overwriteWithDefinedProperties);
  the dispatcher trees drop the strict literal; 44 test command
  strings drop the token (all verified safe: fully-specified
  properties are restored by overwriteWithDefinedProperties, the
  rest have no neighbour-dependent properties, the fence sites
  assert nothing about connections) and the 2 note tests use
  explicit instrument=harp instead (one renamed). (3) The
  debug-world check REMOVED from /setblock, /fill and /clone
  (isDebug appears 0 times in 1.21.4's three command classes, 1
  time each in 1.21.5's). Clone keeps its strict (CloneCommands
  is outside item 42's classes, and the clone block-tick tests
  depend on its strict semantics — a routed gap, see below);
  place_state keeps its strict param for clone, and /place
  template's strict (structure settings) is untouched.
  NO-CHANGE: the teleportTo pitch clamp is already present
  (pitch.clamp(-90, 90) at commands.rs:1698, matching 1.21.4's
  Mth.clamp in teleportTo — routed from batch 5); /advancement
  is same-behaviour (a shared helper vs inline messaging);
  FillCommand.wrapWithMode is 1.21.5-only; the fill
  destroy-count (affected) matches 1.21.4's DESTROY filter; the
  block-entity clearing in place_state (remove without side
  effects on a block change) is 1.21.4's Clearable.tryClear.
  Item 43 (protocol/item/skip.rs): NO-CHANGE. All jar diffs are
  ByteBufCodecs anonymous-class numbering only ($24 vs $23, $28
  vs $26, $29 vs $27, $21 vs $20; fromCodecWithRegistries,
  idMapper, registry are same) — zero behaviour. The tree diff
  is just the component-id table, already re-pinned for 1.21.4
  by the mechanical layer. Item 44 (widely named:
  client/resources/model): NO-CHANGE. 15 vanilla classes, 85
  methods, 0 named in Horizon; module_status missing-both — the
  same shape as items 27 and 29. Lib check green (EXIT 0,
  2m32s); --tests check green (EXIT 0, 4m06s). vanilla.rs,
  entity_die_tests.rs and clone_block_ticks_tests.rs
  rustfmt-clean; data.rs had 3 drift hunks from the strict drops
  (fixed by hand to rustfmt's form) and terrain_cmds.rs 1 (fixed
  the same way); commands.rs, raids.rs and the other test files
  checked only (module roots / file-module declarers), with only
  pre-existing drift hunks elsewhere (creatures.rs 3893/6110/
  7088, persist.rs 1559, commands/extra.rs 127 — left alone).
  Rank 2: 91/348.
  Batch 17 (items 45 + 46 + 47 + 48). Item 45 (render/camera.rs):
  PORTED. (1) Camera distance is the flat 4.0 * scale (1.21.4's
  Camera.setup computes 4.0f * LivingEntity.getScale(), no
  camera_distance attribute, no mount consideration; the 1.21.5
  form carries). (2) depth_far is renderDistance * 4 with no
  cloud-range max (1.21.4's GameRenderer.getDepthFar; three call
  sites drop the cloud_range arg). (3) The spin is now 1.21.4's
  single spinning intensity: ScreenEffects keeps spinning /
  o_spinning / confusion_animation_tick (incremented per tick);
  tick merges the portal ramp (+0.0125), the Confusion ramp
  (+1/150) and the decay (-0.05), clamped 0..1, per
  LocalPlayer.handleConfusionTransitionEffect (read in full:
  portal step also closes containers and plays PORTAL_TRIGGER at
  intensity 0 — the nether_portal.rs gate now reads the pre-tick
  spinning intensity). spinning_matrix uses angle =
  (confusionAnimationTick + partial) * (7 if Confusion else 20)
  and intensity = lerp * scale^2. The Nausea blend fields,
  tick_spinning and nausea_blend_added are gone: BlendState
  exists in 1.21.4's MobEffectInstance and the UpdateMobEffect
  packet carries the flag-8 shouldBlend bit, but no 1.21.4
  renderer reads the blend factor (grep over LocalPlayer,
  GameRenderer, Gui, FogRenderer, LevelRenderer: 0 calls), so
  the remote.rs nausea branch is removed with it.
  (4) camera_overlays: the portal swirl shows when intensity > 0
  without Confusion, the confusion swirl when Confusion is
  present and screenEffectScale < 1 (alpha = intensity *
  (1 - scale)), per Gui.renderCameraOverlays. NO-CHANGE: the
  bobHurt/bobView mulPose renames, processBlurEffect (no
  PostChain in the Rust), render/renderLevel GPU API,
  takeAutoScreenshot and the tick null-check reorder are
  API-only; spinning_effect_matrix is identical in both jars.
  Tick granularity: confusionAnimationTick advances per game
  tick here, per frame in vanilla — the same approximation the
  1.21.5 form already made. Item 46 (game/mounts.rs): PORTED.
  The plain-carrot horse-diet arm is removed (neither 1.21.4's
  nor 1.21.5's AbstractHorse.handleEating mentions Items.CARROT
  — a 1.21.6 addition; the test asserts eating(carrot) == None)
  and the jump-meter comment now cites Gui.renderJumpMeter
  (1.21.4 has the private method, no JumpableVehicleBar class).
  NO-CHANGE: Strider mobInteract already checks
  stack.is(Items.SADDLE) (the 1.21.4 form, not 1.21.5's
  isEquippableInSlot); the strider finalize saddles + drop
  chance 2.0 (the equipSaddle outcome); Camel handleEating/tick
  and Strider floatStrider/spawnJockey are playSound overloads
  or renames. The stand-counter test (counter 20) is LEFT for
  item 205: the logic lives in equine.rs (stand_if_possible
  sets 20; 1.21.4's jar sets standCounter = 1) and changing the
  mounts.rs test alone would break it. Item 47
  (entity_nbt/hanging.rs): PORTED. Hanging entities save and
  load the tile position as TileX/TileY/TileZ ints (1.21.4's
  BlockAttachedEntity.addAdditionalSaveData /
  readAdditionalSaveData), not the 1.21.5 block_pos int array
  the Rust wrote. Load order: Tile keys, then block_pos (kept
  for saves this port wrote before the fix), then the summon
  position's block. Tests updated (Tile keys round-trip,
  block_pos absent from saves). NO-CHANGE: ItemFrame Facing as
  a 3D byte (default 0 = DOWN), ItemRotation plain getByte,
  ItemDropChance default 1.0 (read only when present),
  Invisible/Fixed plain booleans, the Item compound check — all
  already the 1.21.4 form; Painting variant-as-string + 2D
  facing byte match the VARIANT_CODEC encoding. The Cushion
  block_pos (entity_nbt.rs) is a 26.x entity, out of scope.
  Item 48 (game/scoreboard.rs): PORTED. TamableAnimal.getTeam
  is owner-first: a tamed pet answers its owner's team, else
  its own (1.21.4 checks getOwner().getTeam() first, then
  Animal.getTeam; the Rust had the 1.21.5 own-first order).
  getRootOwner is gone (a 1.21.5 addition; 1.21.4 uses the
  direct getOwner()): root_owner and root_owner_name are
  replaced by owner_name resolving the direct owner, and a pet
  owned by a pet now answers that pet's team. Tests updated
  (both-teams case answers the owner's; pet-owned-by-pet
  answers the pet's team). NO-CHANGE: Owner save/load is
  already the 1.21.4 putUUID form (IntArray + uuid_from_tag);
  the legacy username conversion (OldUsersConverter) is a
  shared gap in both trees; tame.rs's is_owner/owned_by already
  use the direct owner. Routed: the narrator rename
  (say_system_now -> say_now, say_system_queued -> say,
  say_system_chat_queued -> say_chat, and sayChat's missing
  empty check — all jar-verified 1.21.4 behaviour, GameNarrator
  has only sayChat/say/sayNow) spans 15 files with no owning
  item; it goes to item 299 (horizon-narrator crate) and the
  scoreboard.rs:885 call site stays say_system_now until then.
  Lib check green (EXIT 0, 1m18s, one warning fixed: the unused
  screen_effects import in remote.rs); --tests check green
  (EXIT 0, 2m08s). camera.rs, hud_overlays.rs,
  screen_effects.rs, kinds.rs, hanging.rs, nether_portal.rs,
  render_tick.rs, fov.rs and screen_backgrounds.rs
  rustfmt-clean (fov.rs and screen_backgrounds.rs needed one
  hand fix each in the edited regions); agent.rs, mounts.rs and
  scoreboard.rs clean; remote.rs, potions.rs, entity_nbt.rs and
  game.rs checked only, with only pre-existing drift hunks
  elsewhere (remote.rs 5549, potions.rs 260/2004 — left alone).
  Rank 2: 95/348.
  Batch 18 (items 49 + 50 + 51 + 52). Item 49
  (game/armor_stands.rs): PORTED. (1) The PlayerHead special
  model is removed: 1.21.4's data uses `head` + `kind: player`
  (items/player_head.json, identical in 1.21.5) and neither jar
  has a PlayerHeadSpecialRenderer (only SkullSpecialRenderer,
  byte-identical sizes) — the 1.21.5 tree's form carries. The
  variant is gone from horizon-assets special.rs (enum, parse
  and serialize arms); argument() reads the profile for
  Head{kind: player}; armor_stands.rs and mob_blocks.rs use
  Head for all kinds; item_render/special.rs renders a player
  head inside the Head arm (profile skin, translucent) and the
  extents arm is dropped (skull_layer("player") is already
  "player"). (2) The happy-ghast phrase is removed from the
  tick comment (batch 2's removal). NO-CHANGE:
  ArmorStandItem.useOn is API-only (createDefaultStackConfig
  LivingEntity-vs-Player param, snapTo/moveTo, playSound
  overload); swapWithEquipmentSlot already matches 1.21.4
  (equip_from_hand checks no canBeEquippedBy — the 1.21.5
  addition — and canUseSlot is vacuous for the four armour
  slots); equipOnInteract exists only in the protocol layer,
  correctly dropped on the 769 wire; the armor-stand save
  already uses HandItems/ArmorItems. Item 50 (widely named:
  world/level/block): NO-CHANGE. 26 vanilla classes, 77
  methods, 0 named in Horizon — the same shape as items 27, 29
  and 44. Item 51 (lan/data.rs): NO-CHANGE, comment-only.
  ListTag.addAndUnwrap / wrapIfNeeded are 1.21.5-only, but the
  behaviour (mixed-list {"": ...} wrappers) exists in 1.21.4
  inside NbtOps$HeterogenousListCollector (own wrapElement /
  isWrapper, javap-verified) and the Rust's nbt.rs already
  wraps/unwraps the same way, so only the lan/data.rs citations
  change. Item 52 (world/level/mod.rs): PORTED. (1) The dead
  update_neighbours_on_block_set is removed (23 lines): no
  callers since batch 16 removed the commands.rs call, the
  1.21.6 form, absent from the 1.21.5 tree. (2) Removal
  citations carried to the 1.21.4 trigger:
  BlockEntity.preRemoveSideEffects does not exist in 1.21.4
  (only setRemoved) — LevelChunk.setBlockState calls
  BlockState.onRemove, and LecternBlock.onRemove + popBook and
  JukeboxBlock.onRemove exist (all javap-verified). The
  removal.rs module doc, PreRemove doc, lectern_book_position
  doc (LecternBlock.popBook) and the level/mod.rs:796 doc now
  cite onRemove; the effects themselves (contents spill,
  jukebox pop, lectern book, dropItemStack math) are identical
  in both versions. The PreRemove / pre_remove identifiers
  stay (internal names). NO-CHANGE: BellBlock.onProjectileHit
  is a local-slot reshuffle (same logic; the Rust takes no
  entity param); the BlockEntity load/save codec-vs-MapCodec
  differences are the item-32 components form (already
  handled); the update.rs:348 piston preRemoveSideEffects
  comment belongs to that module's item. Lib check green
  (EXIT 0, 1m54s, no warnings); --tests check green (EXIT 0,
  2m06s). armor_stands.rs, special.rs (both), mob_blocks.rs,
  lan/data.rs and removal.rs rustfmt-clean
  (item_render/special.rs needed one hand fix in the edited
  region: the `if let` brace); level/mod.rs checked only
  (module root, no drift in the edited regions). Rank 2:
  99/348.
  Batch 19 (items 53 + 54 + 55 + 56). Item 53
  (render/structure_block.rs): PORTED. The load defaults are the
  1.21.4 plain-get forms: ignoreEntities and showboundingbox
  default to false (plain getBoolean, not 1.21.5's
  getBooleanOr(true)) and posY to 0 (plain getInt, not
  DEFAULT_POS (0,1,0)); the constructor defaults in new() stay
  (0,1,0)/true/true — those are the field initializers
  (javap-verified), and load overwrites them. The load tests now
  assert the 1.21.4 defaults. NO-CHANGE on the render box:
  BoundingBoxRenderable does not exist in 1.21.4 (a 1.21.5
  addition), but 1.21.4's StructureBlockRenderer gates match
  render_mode exactly (Save/Load only; the box iff Save or
  showBoundingBox; invisible blocks iff Save && showAir), and
  the mirror/rotation box math is the same source, so
  render_mode / renderable_box stand as the ported equivalents.
  strict stays — routed to item 394 (rank 3). Item 54
  (mobs/spawn_selection.rs): NO-CHANGE, comment-only. All diffs
  are API-only — WeightedList vs WeightedRandomList (rename),
  SpawnerData record accessors vs fields (same values), snapTo
  vs moveTo; SpawnerData#equals is 1.21.5-only. One citation
  updated (WeightedRandomList.getRandom). Item 55 (mobs.rs):
  PORTED. (1) A mob's fire_ticks starts at -1 (the Entity
  constructor sets -getFireImmuneTicks(), javap-verified), so
  the first lightning strike ignites a fresh mob. (2)
  extinguish_fire sets exactly 0 (clearFire), not min(0).
  (3) The 1.21.4 post-check in ai.rs tick: a non-burning alive
  entity not inside fire or lava leaves -1 after the pass —
  EVERY pass, not only when a fire went out in it. 1.21.4's
  applyEffectsFromBlocks has no wasOnFire gate (1.21.5 gates
  the reset on it); the flag predicate is the #fire tag or
  lava (the synthetic lambda, javap-verified). The wet-fire
  test expectations move 0 -> -1. (4) The dead leash_spin
  field is removed (write-only; the creaking.rs write is gone
  with it). (5) The travelFlying comments are fixed (no such
  method in either jar) and the LEAD_BREAK comment is carried
  (1.21.4 plays no sound for a snapped lead). NO-CHANGE: the
  Cow 0.4 volume was already correct (Cow.getSoundVolume is
  1.21.4-only); the RemotePlayer lerp math is exactly 1.21.4's
  lerpPositionAndRotationStep — InterpolationHandler does not
  exist in 1.21.4 at all, and the citations in remote.rs,
  entities.rs, fishing.rs, end.rs, interactions.rs and
  dragon.rs are a known naming lag, behaviour verified
  identical, deliberately not swept. KNOWN GAP: the
  entities.rs fire_ticks: 0 inits (6 sites) plus arrows.rs and
  fluids.rs stay 0, matching the 1.21.5 tree's scoping — the
  jar's -1 applies to all entities. Item 56
  (game/block_ticks.rs): PORTED. (1) The portal-ticket expiry
  gate is removed: 1.21.4's Ticket.timedOut is a pure age
  check (now - createdTick > timeout) and canTicketExpire is a
  1.21.6 addition — tick_portal is bare (the 1.21.5 tree form),
  portal_ticket_can_expire is deleted from game.rs and the
  levels.rs doc is fixed; the tick_portal doc now cites
  DistanceManager.purgeStaleTickets. (2) The tickChunks
  shuffle is added: 1.21.4's ServerChunkCache.tickChunks runs
  Util.shuffle(tickingChunks, level.random) before ticking
  (1.21.5 dropped it) — the Rust shuffles scratch.ticking with
  the level random in the explosion.rs pattern. Lib check
  green (EXIT 0, 17m12s, no warnings); --tests check green
  (EXIT 0, 53m32s, no warnings) — both slow because the disk
  quota forced deleting target/debug/incremental (3.6G,
  regenerable) mid-check and the cache rebuilt cold. One
  rustfmt incident: a bare `rustfmt <file>` piped to diff
  formatted game.rs and mobs.rs IN PLACE and recursed into
  19 submodule files; all unintended changes were reverted
  with git checkout and the two roots' edits re-applied, so
  the commit holds only the batch's 10 files. (Never pipe
  rustfmt's output for a module root — --check only.)
  creaking.rs, structure_block.rs, spawn_selection.rs,
  block_ticks.rs, ticket_levels.rs, levels.rs and
  wet_fire_tests.rs rustfmt-clean; ai.rs clean by --check;
  mobs.rs and game.rs checked only (module roots). Rank 2:
  103/348.
  Batch 20 (items 57 + 58 + 59 + 60). Item 57 (game/boats.rs):
  PORTED. (1) The chest-boat sneak hunk is carried (routed from
  batch 8): AbstractChestBoat.interact checks isSecondaryUseActive
  in both jars, byte-identical — a sneaking player opens the
  chest instead of the leash step, even with a lead in hand.
  (2) The bubble-column wobble's phase is the level's game time
  in 1.21.4 (10.0f * sin(0.5f * gameTime)), not the boat's
  tickCount (1.21.5) — tick_bubble_wobble takes game_time, the
  dead tick_count field is removed, both call sites (boats.rs
  and remote_vehicles.rs) pass world.game_time, and the tests
  advance the game time. NO-CHANGE: onAboveBubbleColumn is
  onAboveBubbleCol in 1.21.4 (a rename); the
  getGroundFriction / pushEntitiesUp move(Vec3i)-vs-move(DDD)
  difference is API-only; fallDistance float-vs-double is left
  at f64 (the 1.21.5 port's scoping); the tick lerp is the
  InterpolationHandler naming lag (item 55). Item 58
  (game/interaction_shapes.rs): NO-CHANGE, comment-only. The
  values are identical — 1.21.4 builds the cauldron/hopper
  shapes with literal Block.box calls (javap-verified: cauldron
  INSIDE box(2,4,2,14,16,14), hopper INSIDE plus the
  *_INTERACTION_SHAPE statics) and 1.21.5 with the column/boxZ
  helpers (1.21.5-only), so only the citations change. fallOn
  float-vs-double and the spawnDestroyParticles Player-vs-Entity
  overload are API-only. Item 59 (game/advancements.rs):
  PORTED. (1) The AdvancementToast fade is carried wholesale —
  the bytecode is identical in both jars: the label draws at
  full opacity until 1.5 s, then the title lines fade in with
  (fade * 252) | 0x40 (never below alpha 64); the 1.21.4 tree's
  1.21.6 form (label fades out, lines fade in from 0) is
  replaced, with advancement_title_alpha and its test added.
  (2) Toast sounds are un-deduped: 1.21.4 has no
  playedToastSounds set — every visibility change plays
  ui.toast.in/out (1.21.5 dedups each event once per update).
  NO-CHANGE: the showAdvancements toast gate was already
  removed (batch 12); the AdvancementRewards#grant playSound
  overload and ItemPredicate#test are API-only/same. Item 60
  (merchant.rs): NO-CHANGE, comment-only.
  DataComponentPredicate (1.21.4) and
  DataComponentExactPredicate (1.21.5) have identical subset
  semantics — both check Objects.equals for each expected
  component, extras allowed (javap-verified); only the getter
  interface differs, so only the citation changes.
  selectEnchantment's WeightedRandom form differs only in the
  weight-function argument (same selection); the Rust's
  enchant.rs select_with does its own weighted pick. Lib check
  green (EXIT 0, 7m32s, no warnings); --tests check green
  (EXIT 0, 4m35s, no warnings). boats.rs needed one hand fix
  in the edited region (the wobble assignment's line break);
  interaction_shapes.rs, advancements.rs and merchant.rs
  rustfmt-clean; remote_vehicles.rs has only pre-existing drift
  elsewhere (lines 282-293 — left alone). Rank 2: 107/348.
  Batch 21 (items 61 + 62 + 63 + 64). Item 61
  (game/native_block_entity_draw.rs): NO-CHANGE. The Rust's
  PoseStack is a plain Vec<Mat4> (push clones the top, pop
  guarded to len > 1) — behaviour-identical to both jars'
  push/pop; the List+lastIndex (1.21.5) vs Deque (1.21.4)
  internals are unobservable through it, and rotateAround's
  matrix+normal split equals Pose.rotateAround. ModelPart's
  mulPose(Quaternionfc)-vs-(Quaternionf) is API-only. Item 62
  (render/chunk/block_entity.rs): NO-CHANGE, comment-only. The
  jukebox load/save already is the 1.21.4 form — no
  isSameItemSameComponents same-record check (1.21.5-only), the
  clock is honoured only with a record plus the clock key
  (JukeboxSong.fromStack), the save writes the clock only when
  the song exists; RecordItem parses via ItemStack.parse when
  the key is present, else the item empties (and a playing song
  stops). The one edit: TrialSpawnerStateData ->
  TrialSpawnerData — the 1.21.4 jar has only TrialSpawnerData
  (the StateData name is 26.3-era). ROUTED: the same misnomer
  in mobs/trial_spawner.rs (items 88/140/245/268/315/339),
  game/trial_spawner.rs (item 245) and lan/chunks.rs (items
  163/252) belongs to those items. Item 63
  (horizon-command/snbt.rs): NO-CHANGE, comment-only.
  parseAsArgument / parseCompoundAsArgument / parseFully are
  1.21.5-only names, but the Rust's three functions implement
  1.21.4's shapes — parse_tag_argument is TagParser.readValue
  via NbtTagArgument.parse (one value at the cursor),
  parse_fully is TagParser.parseTag (one value, trailing data
  is an error). Citations updated to the 1.21.4 names. Item 64
  (game/natural_spawning.rs): NO-CHANGE.
  close_enough_for_spawning is ChunkMap.
  euclideanDistanceSquared < 16384 with the chunk-centre offset
  in both (the Vec3-field vs Entity.getX/getZ difference is
  API-only); the TriState-vs-boolean hasPlayersNearby is
  unmodelled — the Rust computes closeness directly from player
  positions, which is what 1.21.5's TriState.DEFAULT falls back
  to; collectSpawningChunks and forEachBlockTickingChunk
  (1.21.5-only) have no Rust counterpart to change. No
  citations to fix. --tests check green (EXIT 0, 16m08s, no
  warnings) for horizon-client-render + horizon-command.
  block_entity.rs and snbt.rs rustfmt-clean. Rank 2: 111/348.
  Batch 22 (items 65 + 66 + 67 + 68). Item 65 (widely named:
  client/renderer/block/model): NO-CHANGE. 10 vanilla classes,
  62 methods, 0 named in Horizon — the same shape as items 27,
  29, 44 and 50. Item 66 (game/tnt.rs): PORTED — the
  cross-cutting `tnt_explodes` removal (routed from batch 9).
  RULE_TNT_EXPLODES does not exist in 1.21.4 (added in 1.21.5;
  jar-verified: 1.21.4's GameRules has only
  RULE_TNT_EXPLOSION_DROP_DECAY). Removed across 14 files: the
  gamerules.rs registration, the save.rs default (57 -> 56),
  the dead fixes_c.rs rename mapping (tntExplodes can never
  appear in a pre-rename save), the tnt_prime_allowed gate,
  the PrimedTnt tick's despawn-without-blast, the useItemOn
  disabled-message gate, the arrow scene field plus the TNT
  prime gate, the explosion wasExploded gate, the three
  dispenser gates, the three sulfur_cubes gates, the
  sulfur_cube thread-local rule sharing (set_tnt_explodes
  deleted; prime/tickFuse always behave as on; the two
  rule-off tests removed, the remaining fuse test's module
  renamed fuse_tests), and the chat.rs/game.rs rule-sharing
  calls. 1.21.4's TntBlock calls explode() directly (prime()
  is 1.21.5-only) and playerWillDestroy checks isCreative()
  (not getAbilities().instabuild). One warning fixed on the
  way: the Text import in sulfur_cubes.rs went unused with the
  disabled-message block. Item 67 (game/beacon.rs): PORTED.
  BeaconRenderer.getViewDistance is a flat 256 blocks in
  1.21.4 (1.21.5 culls at the effective render distance times
  16) — beacon_geometry's cull is now the constant 256.0.
  NO-CHANGE: the beacon decode reads no CustomName in either
  tree (a shared gap); the beam sections are (colour, height)
  tuples in both (the BeaconBeamOwner$Section vs BeaconBeamSection
  class move is API-only); shouldRender's BlockEntity-vs-
  BeaconBlockEntity getBlockPos is API-only. Item 68
  (world/level/hooks.rs): NO-CHANGE. ButtonBlock#getShape is a
  Function table in 1.21.5 vs the explicit face/facing/powered
  switch in 1.21.4 with identical shapes (data-driven in the
  Rust, no code to change); ButtonBlock#entityInside is same;
  SpongeBlock#tryAbsorbWater is a playSound Player-vs-Entity
  overload; PistonHeadBlock#playerWillDestroy's
  preventsBlockDrops() is exactly getAbilities().instabuild in
  1.21.5 (javap-verified), and the concept is unmodelled in
  both trees. Lib check green (EXIT 0, 2m12s) after a runtime
  restart lost the first run's result (empty log; re-ran from
  scratch); --tests check green (EXIT 0, 7m09s, no warnings).
  gamerules.rs, fixes_c.rs, tnt.rs, arrow_probe.rs,
  arrows_client_tests.rs, explosion.rs, dispenser.rs,
  sulfur_cubes.rs, chat.rs and beacon.rs rustfmt-clean;
  arrows.rs and sulfur_cube.rs needed one hand fix each in the
  edited regions (a match-arm collapse, a stray blank line);
  save.rs, creatures.rs and game.rs checked only (module
  roots / pre-existing drift elsewhere — left alone). Rank 2:
  115/348.
  Batch 23 (items 69 + 70 + 71 + 72). Item 69
  (assets/item_model.rs): NO-CHANGE. The tree diff is only the
  DATA_COMPONENT_TYPES table, already the 1.21.4 list (67
  entries, with hide_tooltip / hide_additional_tooltip and
  map_color). GrassColor#get's inline math matches the Rust's
  biome.rs sample (clamped temperature, downfall x temperature,
  (y<<8|x) index, bounds-checked fallback);
  ItemStackRenderState.isLeftHand is write-only in 1.21.4
  (nothing reads it), so the Rust correctly carries no such
  field (the display context already has the left-hand
  variants). Item 70 (world/level/bonemeal.rs): PORTED.
  GrassBlock.performBonemeal takes the biome's FIRST flower
  feature in 1.21.4 (flowers.get(0)) and a random one in
  1.21.5 (flowers.get(random.nextInt(size))) —
  grass_bonemeal_effect now takes list[0]. NO-CHANGE: the
  spreadable-neighbour helper mirrors 1.21.5's
  getSpreadableNeighbourPos extraction, but its callers are
  dead in 1.21.4 (no BushBlock.performBonemeal, no FireflyBush)
  so the shuffle flag is unobservable here. KNOWN SHARED GAP:
  the Rust's flower list is built from the
  can_spawn_from_bone_meal tag, not the biome's flower-features
  list (both trees). Item 71 (data_cmd.rs): NO-CHANGE,
  comment-only. NumericTag#doubleValue/intValue/shortValue are
  1.21.5-only names for getAsDouble/getAsInt/getAsShort
  (identical behaviour); the Rust's numeric() cites getAsDouble
  now. Item 72 (game/allays.rs): NO-CHANGE. Allay.travel's
  1.21.4 isControlledByLocalInstance gate is a client-side/no-AI
  guard — the server tick only travels AI-enabled mobs, so it
  is already implied (comment recorded on allay::travel);
  duplicateAllay snapTo/moveTo is a rename; mobInteract's
  playSound overloads are API-only;
  updateDuplicationCooldown's unconditional decrement matches
  1.21.4 (the 1.21.5 server-gate + setDuplicationCooldown form
  differs), and the DATA_CAN_DUPLICATE sync has no Rust
  counterpart in either tree (a shared gap). --tests check
  green (EXIT 0, 4m10s, no warnings) for horizon-client +
  horizon-world. bonemeal.rs, data_cmd.rs and allay.rs
  rustfmt-clean. Rank 2: 119/348.
  Batch 24 (items 73 + 74 + 75 + 76). Item 73 (widely named:
  client/renderer): NO-CHANGE. 7 vanilla classes, 53 methods, 0
  named in Horizon — the same shape as items 27, 29, 44, 50
  and 65. Item 74 (world/level/sculk.rs): NO-CHANGE. All four
  diffs are Level.playSound Player-vs-Entity overloads
  (SculkSensorBlock#activate/tick, SculkSpreader$ChargeCursor
  #update, SculkVeinBlock#attemptPlaceSculk) — API-only, same
  sounds. Item 75 (game/cat_spawner.rs): NO-CHANGE.
  Cat.finalizeSpawn's moon/tag pick already matches 1.21.4 —
  tame::cat_variant draws uniformly over 10 variants (11 on a
  full moon), which is exactly Registry.getRandomElementOf over
  the verified tag data (default_spawns = the first 10 in
  registry order; full_moon_spawns = those 10 + all_black); the
  swamp-hut ALL_BLACK + persistence rule is already applied on
  the spawner path (spawn_cat checks cats_spawn_as_black). The
  CatSpawner#tick int-vs-void return is unobservable (the count
  is ignored). ROUTED GAP (item 188, spawning.rs): an
  egg/summon-spawned cat inside a hut does not get ALL_BLACK —
  finalize has no world handle for the structure check. Item 76
  (game/persist.rs): PORTED. (1) PistonMovingBlockEntity.
  saveAdditional writes facing as an INT in 1.21.4
  (get3DDataValue + putInt); 1.21.5 stores a byte via
  Direction.LEGACY_ID_CODEC (a BYTE codec, javap-verified) —
  the Rust wrote a byte; now Tag::Int. The load already reads
  it via number() and maps through Direction::ALL (Down/Up/
  North/South/West/East = from3DDataValue). (2) The piston tick
  places the moved block with flags 84 in 1.21.4, not 340 —
  340 adds 256 SKIP_BLOCK_ENTITY_SIDEEFFECTS, a 1.21.5-only
  flag. NO-CHANGE: the blockState NbtUtils form,
  progress/extending/source plain gets (same defaults), and
  preRemoveSideEffects absent in 1.21.4 (already handled in
  batch 18, item 52). --tests check green (EXIT 0, 6m06s, no
  warnings) for horizon-client + horizon-world. pistons.rs
  rustfmt-clean; persist.rs checked only (module root,
  pre-existing drift elsewhere — left alone). Rank 2: 123/348.
  Batch 25 verification (2026-11-26, restart after the agent stopped
  on a context limit at c4295402). The WIP touched only the item-80
  gamerule plumbing and one item-79 comment; it closed no item and
  updated neither work-items.json nor rank2-progress.json, so the
  committed counts stand (rank 2: 123/348). VERIFIED against the
  1.21.4 named server jar: (1) RULE_DOFIRETICK exists in 1.21.4
  GameRules and no FIRE_SPREAD/ALLOWFIRE rule does;
  LavaFluid.randomTick and FireBlock.tick both read RULE_DOFIRETICK
  only, while 1.21.5's LavaFluid.randomTick reads RULE_DOFIRETICK
  and RULE_ALLOWFIRETICKAWAYFROMPLAYERS — so the WIP's replacement
  of the fire_spread_radius_around_player rule with doFireTick and
  of can_spread_fire_around with the bare do_fire_tick read is
  correct; `canSpreadFireAround` is a Horizon name, it occurs in
  neither jar. (2) 1.21.4 CampfireBlock has the VIRTUAL_FENCE_POST
  field and 1.21.5 the SHAPE_VIRTUAL_POST field, so the beehive.rs
  comment correction is correct. REMAINING in batch 25: item 77
  (gfx.rs WorldBorderRenderer.render), item 78
  (natural_spawn_fixture.rs Slime/Zombie), item 79 (beehive.rs:
  CampfireBlock.entityInside, placeLiquid, Occupant.of,
  IGNORED_BEE_TAGS), item 80 (fluids.rs: BarrierBlock
  canPlaceLiquid/pickupBlock, FlowingFluid.tick lava gate).
  Rank 2: 123/348.
  Batch 25 (2026-11-26, restart continued): items 77, 78, 79, 80, all
  PORTED. Item 77 (gfx.rs WorldBorderRenderer.render): the Rust was the
  1.21.5 cached-buffer form (BorderRenderer with needs_rebuild/last_border,
  rebuildWorldBorderBuffer's 16 vertices, closestBorder nearest-first draw
  order). 1.21.4's WorldBorderRenderer has only render(WorldBorder, Vec3,
  double, double) (javap vs the 1.21.4 client jar): it emits the wall as
  one-block segments in a fixed east, west, south, north order, with no
  closestBorder (that method does not exist on 1.21.4's WorldBorder) and no
  0.01 inset (no RENDERING_OFFSET in either 1.21.4 or 1.21.5). Fixed: sides()
  now returns the fixed [east 3, west 1, south 0, north 2] order filtered by
  proximity; closest_border + java_double_compare removed; the 0.01 OFFSET
  inset removed (the wall now sits exactly on the border bounds); the
  far-outside early return removed (1.21.4 has only the deep-inside return;
  nothing is drawn far outside either way). Kept the port's cached
  single-quad buffer and texture-matrix scroll offset: the same picture as
  1.21.4's per-frame segmented emission, since the texture wraps and u is
  linear in the wall. Docs corrected (prepareDynamicTransforms is not a
  vanilla method in either version). Item 78 (natural_spawn_fixture.rs
  Slime/Zombie): PORTED — Zombie.canHoldItem in 1.21.4 checks Items.EGG (the
  single item); 1.21.5 checks ItemTags.EGGS. The #minecraft:eggs tag does not
  exist in 1.21.4 (no eggs.json in the jar's data, no EGGS field in 1.21.4's
  ItemTags class — javap-verified; both exist in 1.21.5), so the Rust's
  item_in_tag("minecraft:eggs") was always false: a baby zombie riding a
  chicken would wrongly pick up an egg. mob_looting.rs now checks item ==
  "minecraft:egg" (the binding is &*stack.item, as stack.item is Arc<str>).
  NO-CHANGE: Slime.readAdditionalSaveData (getInt/getBoolean vs
  getIntOr/getBooleanOr with the same 0/false defaults); Zombie.finalizeSpawn
  (Chicken.moveTo in 1.21.4 vs snapTo in 1.21.5 — the Rust's create() sets
  position + previous + yaw the same way, only the snapTo comment corrected
  to moveTo; the Halloween-head drop chance is a direct
  armorDropChances[HEAD] = 0.0 store in 1.21.4, which the Rust's
  set_drop_chance already does); Zombie.readAdditionalSaveData (identical
  defaults; the DrownedConversionTime arm matches on all producible inputs).
  Item 79 (beehive.rs): PORTED — IGNORED_BEE_TAGS is now the exact 1.21.4
  30-entry set from BeehiveBlockEntity's <clinit> (javap-verified, in order);
  the Rust had 25 (the 1.21.5 set): removed drop_chances, equipment,
  fall_distance, sleeping_pos; added ArmorDropChances, ArmorItems,
  FallDistance, HandDropChances, HandItems, HurtByTimestamp, SleepingX,
  SleepingY, SleepingZ. NO-CHANGE: Occupant.of's nectar read,
  CampfireBlock.entityInside, placeLiquid. Item 80 (fluids.rs): PORTED — the
  LevelAccess trait default can_spread_fire_around returned true with the
  stale "(fire-spread radius around players)" comment (a 26.x concept); it
  now returns self.game_rules().do_fire_tick, matching 1.21.4, where
  LavaFluid.randomTick, FireBlock.tick and lightning fire all spread only
  while doFireTick is on (javap-verified; the GameLevel override already did
  this). Default rules have do_fire_tick=true, so fixtures keep returning
  true. NO-CHANGE: BarrierBlock canPlaceLiquid/pickupBlock, FlowingFluid.tick
  (value PartialEq != matches 1.21.4's .equals()). cargo check green for
  horizon-client-render + horizon-world (5m15s, no warnings) and
  horizon-client (1m04s, no warnings). --tests: horizon-client-render lib
  2053 passed / 4 failed, horizon-world lib 444 passed / 3 failed,
  horizon-client lib 1263 passed / 4 failed — all 11 failures proven
  pre-existing by re-running them with the batch stashed (still failing).
  horizon-client-lib lib test target does not compile on this tree: 6 errors
  in block_entity_item.rs tests (spawn_egg_on_spawner called with 2
  arguments, defined with 1) — also proven pre-existing the same way;
  test-health item, CI crate-tests will show it. The border test
  near_wall_is_drawn_and_tinted passes with the new fixed order. Rank 2:
  127/348.
  Batch 26 (2026-11-26, restart continued): items 81, 82, 83, 84.
  Item 81 (horizon-command/src/item.rs): NO-CHANGE. 1.21.4's
  ItemPredicateArgument has its own parse/listSuggestions over its
  grammarWithContext field (grammar.parseForCommands + Util.allOf +
  Result::test; suggestions via grammar.parseForSuggestions); 1.21.5's
  extends ParserBasedArgument and inherits the same two calls, so the
  entry points are observably identical. ComponentPredicateParser.
  createGrammar has the same 10 lambdas and rule structure in both
  versions; the ResourceLookupRule constructor and
  lambda$createGrammar$9's Scope parameter are internal packrat
  plumbing. Decode is the Tag form in 1.21.4 (ComponentWrapper/
  PredicateWrapper decode(reader, RegistryOps<Tag>, Tag); Context
  passes a Tag) and the Rust's decode_component/decode_predicate take
  &Tag — the 1.21.4 form; 1.21.5's Dynamic form validates the same
  values through the same codec. ItemParser holds TagParser<Tag> +
  RegistryOps<Tag> in 1.21.4 (passed as parameters in 1.21.5); the
  Rust's ItemState parses SNBT to a Tag and validates with
  decode_component — same unknown-item/component/repeated/malformed
  errors. The pseudo maps hold only count in both versions (javap
  <clinit>), which the Rust handles. Item 82 (level/tick.rs): PORTED —
  FireBlock.tick writes the aged fire with flags 4 in 1.21.4 (javap
  offset 179-180, iconst_4) and 260 in 1.21.5; the Rust had
  set_block(pos, state, 260) at tick.rs:652, now
  super::flags::UPDATE_INVISIBLE (260 is UPDATE_NONE = 256 + 4, and
  256 is the 1.21.5-only UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS bit).
  The prime_tnt doc now names 1.21.4's TntBlock.explode (1.21.5
  renamed it prime and gated it on the tntExplodes rule, which 1.21.4
  does not have); behaviour already matched. NO-CHANGE: bootStrap's
  seven 1.21.5-only flammable blocks (SHORT_DRY_GRASS, TALL_DRY_GRASS,
  WILDFLOWERS, LEAF_LITTER, CACTUS_FLOWER, FIREFLY_BUSH, BUSH) do not
  exist in the 1.21.4 jar, and the Rust's fire odds come from the
  per-version block-behaviours.json; the tick gate (doFireTick via
  can_spread_fire_around after item 80) and the rest of fire_tick
  already match 1.21.4. Item 83 (game/dragon.rs): PORTED (comment
  only) — breath_affect_mob cited setPotionDurationScale(0.25) for the
  flame phase and DragonFireball.onHit; that method does not exist in
  1.21.4 (no such method on 1.21.4's AreaEffectCloud; 1.21.5 added it
  with DEFAULT_POTION_DURATION_SCALE). 1.21.4's quarter is
  AreaEffectCloud.serverTick's mapDuration(duration / 4); the comment
  now says so. Behaviour verified matching throughout: EndCrystal
  read/add save data (beam_target as an IntArray of 3 via NbtUtils,
  ShowBottom default true, write IntArray + ShowBottom); hurtServer
  identical (ldc/ldc_w noise only); EnderDragon.aiStep is the 1.21.4
  lerpSteps form (tick_dragon + tick_remote_dragon step 1/nth toward
  the sent position/yaw), not 1.21.5's InterpolationHandler;
  onCrystalDestroyed identical in both versions (Player-or-nearest,
  10.0 explosion on the head when nearest, phase strafe callback);
  readAdditionalSaveData matches on producible inputs (Phase::from_id
  clamps like getById; DragonDeathTime contains-guarded);
  sanitizeScale identical (returns 1.0);
  DragonSittingFlamingPhase.doServerTick differs only in the
  1.21.5-only setPotionDurationScale call and the HARM->INSTANT_DAMAGE
  rename — the scale only affects non-instant effect durations and
  both clouds carry only instant damage, and the Rust's instant
  magnitudes (3.0 flame / 6.0 fireball at the cloud's 0.5 scale) match
  1.21.4 exactly (cloud passes 0.5 to applyInstantenousEffect,
  javap-verified); DragonStrafePlayerPhase.doServerTick identical;
  EnderDragonRenderer/DragonFireballRenderer render methods identical
  (constant-pool noise only; extractAdditionalHitboxes is 1.21.5-only).
  Noted edge (no fix): a non-numeric DragonPhase tag coerces to
  0/Holding in 1.21.4 (contains + getInt) while the Rust keeps Hover —
  not producible by vanilla saves. Item 84 (hud.rs): NO-CHANGE.
  PackSelectionScreen.loadPackIcon is identical except the
  DynamicTexture constructor (API-only). The Rust's icon_texture
  reproduces the naming exactly: Util.sanitizeName lowercases
  (Locale.ROOT) then replaces chars failing
  ResourceLocation.validPathChar ([a-z0-9_-./]) with '_', and the sha1
  is Hashing.sha1().hashUnencodedChars(id) (UTF-16LE) — both
  javap-verified. TextureAtlas.dumpContents writes name.png + name.txt
  in both versions; the 1.21.5 change is the writeAsPNG overload (gl id
  vs GpuTexture + identity operator) — API-only; dumpSpriteNames sorts
  by key and writes '%s\tx=%d\ty=%d\tw=%d\th=%d' with getX/getY/
  contents width/height, which the Rust's dump_atlas matches.
  TextureAtlas upload/cycleAnimationFrames/getHeight/constructor differ
  only in the GL-id vs GpuTexture backend (1.21.5 adds a null-texture
  guard and passes the texture to tickAndUpload/uploadFirstFrame); the
  Rust's wgpu atlas + SpriteAnimator reproduce the observable
  behaviour. cargo check green for horizon-world + horizon-client
  (1m02s, no warnings). --tests: horizon-world lib 444 passed / 3
  failed, horizon-client lib 1263 passed / 4 failed — the same 7
  pre-existing failures as batch 25 (datafix x3, lan_particles,
  lan_screen, launch_api, spill). NOTE: the horizon-client lib run was
  interrupted by a runtime restart mid-run (session proc_897ba7b003e7,
  empty recorded log); the prebuilt test binary it had produced was
  re-run directly and completed in 13s with the baseline result.
  Rank 2: 131/348.
  Batch 27 (2026-11-26, restart continued): items 85, 86, 87, 88.
  Item 85 (game/mob_blocks.rs): NO-CHANGE.
  LivingEntityRenderer.setupRotations is 135 lines in both versions
  (only constant-pool indices differ) and
  CustomHeadLayer.translateToHead is 32 vs 32 with an empty diff.
  extractRenderState differs only in one call: 1.21.4's
  ItemModelResolver.updateForLiving(headItem, headStack,
  ItemDisplayContext.HEAD, false, entity) passes a boolean the
  1.21.5 4-arg form drops — stored into
  ItemStackRenderState.isLeftHand; a head item is never in a hand, so
  the picture is identical. extractAdditionalHitboxes is 1.21.5-only.
  Noted pre-existing gap (no fix — identical vs 1.21.5, not a version
  delta): a player named Dinnerbone/Grumm should flip upside-down
  when their cape is shown (isEntityUpsideDown reads getName() and
  requires isModelPartShown(CAPE)), but the Rust flips only on
  Mob.custom_name, which comes solely from DATA_CUSTOM_NAME/NBT and is
  never set from a player's username, so players with those usernames
  never flip. The cape bit is tracked (hidden_model_parts bit 0) but
  not consulted by the flip; fixing needs username plumbing into the
  flip path — a backlog item, not a port delta. Item 86
  (game/stats.rs): NO-CHANGE. ServerStatsCounter: 1.21.4 has
  fromJson(JsonObject), getKey(Stat), getStat(StatType, String) plus
  parseLocal/toJson lambdas; 1.21.5 has createTypedStatsCodec +
  lambdas instead. Verified by javap: 1.21.4's toJson hand-builds
  {"stats": {type-id: {stat-id: int}}, "DataVersion": <current>}; the
  Rust's to_json writes exactly that shape, and DATA_VERSION = 4189 is
  1.21.4's data version. PhantomSpawner.tick: 1.21.4 returns int (spawn
  count), 1.21.5 returns void; ServerLevel.tickCustomSpawners
  pops/discards the return in 1.21.4, so observably identical. The
  Rust's spawn_phantoms matches; gamerule ids are correctly mapped by
  the datafix layer (("doInsomnia", "minecraft:spawn_phantoms"),
  ("doMobSpawning", "minecraft:spawn_mobs")). Item 87 (mobs/undead.rs):
  PORTED — ZombifiedPiglin.customServerAiStep: 1.21.4 (after
  maybeAlertOthers) does `if (isAngry()) this.lastHurtByPlayerTime =
  this.tickCount;` — 1.21.5 has no such block. An angry piglin counts
  as recently-hurt-by-a-player for loot/XP for as long as it stays
  angry. Fix in zombified_piglin_step (after the anger if/else so a
  piglin whose anger just hit 0 doesn't refresh): `if m.extra.anger >
  0 { m.ai.last_hurt_by_player = 100; }`. Caveats: the Rust's
  single-countdown model can't express vanilla's tick-stamp tail —
  after anger ends the Rust's window is 100 ticks, vanilla's is
  ~age-in-ticks; an angry-at-a-non-player death credits the local
  player for statistics in the Rust (vanilla credits nobody, identity
  null) while loot/XP match. The setTarget companion (1.21.4-only `if
  (target instanceof Player) setLastHurtByPlayer(player)`) is subsumed
  in practice: a new player target starts anger on the next AI step
  and the refresh then covers loot/XP/credit; only uncovered window is
  ≤1 tick between target-set and the next step, plus the
  target-cleared-before-the-step case. MoveThroughVillageGoal.canUse
  (191 vs 191, empty diff) and Drowned.okTarget (17 vs 17, empty diff)
  identical; Drowned.travel's 1.21.4-only
  isControlledByLocalInstance() guard is vacuous (all simulated mobs
  are locally controlled). Two new tests (piglin_anger_tests,
  horizon-client-lib): an angry piglin counts as hurt by a player
  (timer 100 after one step), a calm piglin keeps no player-kill
  memory — both pass. Item 88 (widely named: world/entity): PORTED —
  InsideBlockEffectApplier/StepBasedCollector is 1.21.5-only: the
  Rust's arrows.rs Collector deduped effects per step and applied them
  in fixed type order, but 1.21.4's Entity.applyEffectsFromBlocks calls
  BlockState.entityInside + entity.onInsideBlock immediately per block
  in sweep order, with no collector. The divergent cases are exactly
  water before an igniter (water-then-fire: 1.21.4 leaves the arrow
  burning at 160, collector clears to 0; water-then-lava likewise 300
  vs 0); fire/lava ignites commute (both max). Fix:
  apply_block_effects now passes &mut fire_ticks into
  check_inside_blocks, which applies fire/lava/water inline per block
  in sweep order; the Effect enum, EFFECT_ORDER and Collector were
  deleted. Two new tests (arrows::client_tests, horizon-client-lib,
  server path, 1 tick): water-then-fire leaves the arrow burning
  (fire_ticks 159) and water-then-lava (299) — both pass. The rest of
  the item is NO-CHANGE by hand-checked normalized bytecode diffs:
  AgeableMob (explicit 0-initialization / getIntOr default 0),
  ConversionType$1.convert (same sequence; only the
  lastHurtByPlayerTime field rename), EntityAttachments.scale/
  Builder.build (refactor only), EntitySelector lambdas (identical),
  EquipmentSlot$Type/Group (1.21.5 adds SADDLE type + group; the Rust
  has neither, matching 1.21.4), Leashable/LeashData (same NBT format;
  the 1.21.5 CODEC is 1.21.5-only), Marker (1.21.4 keeps a data
  compound; the Rust preserves it via persist.raw),
  SpawnPlacements.checkSpawnRules (same), DropChances/EntityEquipment
  (1.21.5-only container move; same constants/behaviour),
  EntityReference (1.21.5-only; the Rust resolves owners by UUID, the
  1.21.4 form). Mob$1 is a switch-map class in both jars (the anonymous
  Container with getTheItem/setTheItem is AbstractHorse$1, the
  saddle-slot ContainerSingleItem built in the AbstractHorse
  constructor, removed in 1.21.5 when the saddle became a real
  equipment slot; the Rust's HorseInventoryMenu already models the
  saddle slot). EntityEvent: shared events have identical values;
  1.21.4 has ATTACK_BLOCKED=29 and SHIELD_DISABLED=30 (1.21.5 has
  DROWN_PARTICLES=67, SADDLE_BREAK=68 instead). The local-player shield
  sounds are already correct; the gap is the server-to-client broadcast
  + remote handling of 29/30 (never broadcast, remote handler lacks
  them) — a LAN-only gap, recorded, no fix in this batch (needs
  defender entity-id plumbing plus care to avoid double-play).
  DROWN_EVENT=67 handler is dead code in 1.21.4 (no 1.21.4 server ever
  sends it) — left as a cleanup candidate. cargo check green for
  horizon-client (1m39s). --tests: piglin 2/2, arrow 2/2 (both
  horizon-client-lib). Rank 2: 135/348.
  Test-health (2026-11-26, alongside batch 27): crate-tests FAILED on
  runs 45818664429 and 45823137761 with the same 11 failures (protocol
  item hash, world datafix x3, render packs x3, client lan_particles,
  lan_screen, launch_api, spill). All 11 fixed and verified locally:
  protocol skip.rs gains the synthetic mob-variant ids 72-88/90-95 as
  a varint (1.21.4 has 67 component types; PAINTING_VARIANT=89 is a
  1.21.5 id); block_entity_item.rs spawn-egg test re-scoped to the
  1.21.4 type-only form (neither version has Spawner.setEntityData,
  and SpawnEggItem.useOn calls only setEntityId); datafix fix_display
  parses 1.20 display.Name/Lore JSON into components instead of copying
  raw strings; the three datafix tests re-scoped to 1.21.4 (level data
  keeps Difficulty/GameRules/Spawn, chunk palette keeps Name, both
  fixes past 4189); the three pack tests re-scoped to formats 46/61
  (1.21.4 has no min/max_format or minors); launch-reference.md
  regenerated (61/46.0/1.21.4); the vibration test's particle id 58
  corrected to 46 (1.21.4's ParticleTypes order); remote GameRules
  button follows gamemaster (the 1.21.5 tree has the same test/implementation
  mismatch — a pre-existing inconsistency in the parent, not a delta).
  The spill test's cause was a real data-loss bug, not a test issue:
  canonical() dropped a placed block entity's components (it only
  dropped the `components` key, never merging them as 1.21.4's
  saveWithoutMetadata does), so a placed ominous banner lost its
  item_name/rarity/tooltip_display on save and on load. Fixed by
  merging the components into the tag as top-level `minecraft:*` keys
  in canonical(), reading them back through a new
  block_entity_components() helper in decode_banner and the copper
  golem statue branch, and using the same helper in
  collected_components() (pick-block). Local verification after the
  fixes: horizon-protocol 423 passed / 2 failed (both network tests,
  proven pre-existing by the stash test — the VM proxy intercepts one,
  EPERM on a UDP bind the other; the item-hash test passes),
  horizon-world 447/0, horizon-client-render 2055/2 (overlays fixed
  after the run; the other failure is the local HDR/environment one,
  still being identified), horizon-client 1267/0. Fixing the
  block_entity_item compile error also made horizon-client-lib compile
  for the first time on this tree: 2407 passed / 24 failed. Ten of the
  24 were 1.21.4-only and are fixed here: the world-gen settings
  cluster (7) — the writer stores settings in
  data/minecraft/world_gen_settings.dat (the 1.21.5 form) but
  current_world_gen_settings() only read level.dat, so
  current_world_gen_settings() + generators() now fall back to the
  file (level.dat wins when it carries the settings); the TNT owner
  test (1.21.4's PrimedTnt.addAdditionalSaveData writes only
  block_state/explosion_power/fuse, javap-verified — the test asserted
  an owner the 1.21.5 form writes); the hub self-pose test
  (first_tick hand-wrote a trailing showAdvancements bool, a 1.21.5
  addition — removed; flushDirty already omitted it); the server-packs
  header test (asserted 97.1, the client correctly sends 46.0).
  After those, horizon-client-lib is 2417/14 — exactly the 14 that
  fail identically on the 1.21.5 parent tree (item_sync x2, lan
  advancements x5, projectiles, realms, save chunks x4, title
  optimization), pre-existing programme test-health debt, not
  1.21.4 regressions.
  Batch 28 (2026-11-26, partial: items 89, 90, 91; item 92 held on the
  parent's CI-green priority steer — no more rank-2 items opened until
  CI is green). Item 89 (horizon-assets/item_bake.rs): PORTED.
  QuadCollection is 1.21.5-only; 1.21.4's ItemRenderer.renderModelLists
  draws each culled direction in Direction order (down, up, north,
  south, west, east), then the unculled quads, while the Rust ordered
  unculled first then north/south/east/west/up/down (the
  QuadCollection.getAll order). Fix in bake_resolved: culled quads in
  Direction::ALL order, then unculled; comments now cite
  renderModelLists. The order is the draw order (item_render.rs emits
  baked.quads in list order), so translucent quads blended in the wrong
  order. ItemTransform.apply matches 1.21.4 exactly (left-hand
  negation, translate-rotate-scale then -0.5 in LayerRenderState.render);
  ItemTransform$Deserializer defaults/clamps match (translation x0.0625
  clamp +/-5, scale clamp +/-4). New test quads_follow_the_client_draw_order:
  a cube's culls read [Down, Up, North, South, West, East, None] —
  horizon-assets item_bake 3/3 pass. Item 90
  (horizon-client-render/block_entity_render.rs): NO-CHANGE (already
  seeded closed in rank2-progress.json; verified this session). All
  eight renderers' typed render bodies are identical between 1.21.4
  and 1.21.5 modulo the 1.21.5 interface changes (7th Vec3 camera
  parameter, mulPose(Quaternionf)->Quaternionfc); translateSignText,
  renderBanner, translateBase and ShulkerBox render likewise. The Rust
  threads the camera in (equivalent to 1.21.4's
  Minecraft.getInstance() lookup); sign_text_near matches
  isOutlineVisible (distance squared < 256.0). LayerDefinitions.createRoots
  delta is 1.21.5-only layers (cold/warm chicken/cow/pig,
  camel/donkey/horse/mule/skeleton/zombie-horse saddle layers, sheep
  wool undercoat) owned by other items; BlockEntityRenderers
  TEST_INSTANCE_BLOCK registration owned by items 163/465. No
  TestInstance arm in the Rust's block_entity_render.rs. Item 91
  (horizon-client/stations.rs): PORTED — smithing trim was broken.
  The Rust required the recipe JSON's "pattern" field (the 1.21.5
  form), but 1.21.4's trim recipes carry no pattern (verified against
  the 1.21.4 server jar's recipe JSONs) — the pattern is resolved from
  the template stack at craft time via TrimPatterns.getFromTemplate
  (scan the trim_pattern registry for the entry whose template_item is
  the template stack's item). All 1.21.4 trim recipes were dropped, so
  smithing trim did nothing. Fix: Smithing::Trim drops the pattern
  field and makes template/addition Optional<Ingredient> (1.21.4's
  fields are all Optional, like the transform's); StationRecipes gains
  trim_patterns loaded from data/minecraft/trim_pattern/*.json
  (template_item -> pattern id); the match resolves the pattern from
  the template stack and trim_demo from the template ingredient's
  first item. SmithingTransformRecipe.assemble already matches 1.21.4
  (transmuteCopy + component patch; matches() identical);
  EnchantingTableBlock SHAPE is identical (1.21.5's
  Block.column(16,12,16) == 1.21.4's box(0,0,0,16,12,16),
  data-driven in the Rust). Tests: the additions-trim-material test
  re-scoped (registry-supplied pattern) and
  smithing_trim_reads_its_pattern_from_the_template_item (new;
  1.21.4-form JSON + trim_pattern/coast.json loads, applies the coast
  trim, and a template with no registry pattern trims nothing) —
  stations suite 26/26 passes. Rank 2: 137/348.
  CI 45851997564 (dispatched Nov 26 ~21:43 NPT on head c3e46cf9):
  completed FAILURE at 12m25s — but differently from runs 45818664429
  and 45823137761. All 11 fixes from the test-health batch are green
  in CI (protocol 425/0, world 447/0, render 2057/0, client 1267/0,
  items, core pass; the two local-only protocol network failures pass
  in CI). The only failures are the 14 horizon-client-lib tests
  shared with the 1.21.5 parent (item_sync x2, lan advancements x5,
  projectiles, realms, save chunks x4, title optimization) — 2417/14,
  identical to the local run and to the parent tree. Next: fix those
  14 (each verified against the 1.21.4 jar, not silently re-scoped)
  before opening more rank-2 items.
  Test-health, round 2 (2026-11-26, after batch 28): crate-tests run
  45851997564 failed only on horizon-client-lib, 2417/14 — exactly the
  14 that fail identically on the 1.21.5 parent tree. All 14 are now
  fixed and verified locally; the lib suite is 2431/0. Each fix was
  verified against the 1.21.4 jar, not silently re-scoped:
  item_sync (2): the painting test used the 26.x component id 109 — the
  Rust's internal id is 89 (1.21.5's PAINTING_VARIANT; 1.21.4 has no
  such component, verified: DataComponents has no painting/sign entries)
  — changed 109 to 89 in the test. The sign test used 26.x ids 118/119;
  1.21.4 has no sign-text components (neither does 1.21.5), the text is
  carried internally under markers -1001/-1002 that write_patch never
  emits — re-scoped to pin that a sign with text writes as a plain sign
  and reads back with no text on it. lan::advancements (5): the book
  writer emitted the 26.2 recipe-display form — Tag slot display plus a
  HolderSet, and ItemStack results item-first. 1.21.4's TagSlotDisplay
  holds a bare tag id (javap: TagKey<Item> in both 1.21.4 and 1.21.5;
  the pinned quirk tag_slot_display_is_id is true) and its
  ItemStack.STREAM_CODEC is count-first (v1_21_4::read_stack reads count
  then item). New slot_display() writes Ingredient.display()'s form
  (tag to Tag+string, one item to Item+id, several to Composite of
  Items; verified against Ingredient.display, displayForSingleItem and
  AbstractCookingRecipe.display in the jar) and the result closures now
  write count first (the owned-content test, count 2 item 1, catches the
  swap). The three furnace tests' expectations moved from Tag(HolderSet)
  to Item(id), the 1.21.4 form. projectiles (1): a real port gap —
  settings() had lost the 26.3 fallback where a registered mod thrown
  type takes its template's (snowball's) tracking settings; restored it
  in lan/tracking.rs. realms (1): test-only — the client already sends
  1.21.4/769/Java 21, the test expected 26.3/777/Java 25. save (4): the
  blending fix was registered at key 4997, past 4189, so it never ran —
  Mojang's DataFixers registers BlendingDataFix at schema 4185
  (between addSchema(4185) and addSchema(4187)), re-keyed to 4185; the
  same wrong 4997 sat in record_is_old, and vanilla's
  IOWorker.isOldChunk compares against sipush 4185 — fixed to 4185.
  The legacy-dimension fallback now sets the overworld for an unknown
  numeric (parseLegacy errors there; the load falls back to overworld).
  The older-records palette assertion moved from `id` to `Name`
  (the grass to short_grass rename fires at fix 3692; the Name to id
  restructure is a 26.x fix, past 4189). title (1): the optimization
  fixture set DataVersion 4950, newer than 4189 — the upgrader never
  downgrades — re-scoped to a 1.20.1 world (3465), which stamps to 4189.
  Local verification after the fixes: horizon-client-lib 2431/0.

  Item 92 (horizon-client/game/lan_entities.rs): NO-CHANGE — verified
  against the 1.21.4 and 1.21.5 named client jars (javap -p -c,
  normalized bytecode). Display: 1.21.5 refactored pos-rot interpolation
  onto InterpolationHandler; 1.21.4 keeps PosRotInterpolationTarget with
  lerpTo/cancelLerp/lerpTarget* (steps = getPosRotInterpolationDuration,
  clamped 0..59). The Rust already implements the 1.21.4 form:
  RemoteTextDisplay.move_to sets steps = teleport_duration.clamp(0, 59)
  and snaps at 0, tick() eases (target - position) / steps per tick;
  text_display.rs has teleport_duration default 0, entity-data id 10,
  clamp 0..59. add/readAdditionalSaveData write the same fields with the
  same defaults in both versions (only the codec API style differs);
  createInterpolatedRenderState is byte-identical. Pinned by the existing
  test metadata_and_position_interpolation_keep_their_separate_clocks
  (duration 2: halfway after one tick, arrived after two).
  MinecartFurnace: no semantic change — 1.21.5's DEFAULT_FUEL (0) and
  DEFAULT_PUSH (Vec3.ZERO) only name 1.21.4's field defaults; the Rust
  save/load (PushX/PushZ doubles, Fuel short, defaults 0) and DATA_ID_FUEL
  (id 13, boolean, true when fuel > 0) already match. TrackedEntity:
  1.21.5 adds broadcastIgnorePlayers and ServerEntity's
  broadcastWithIgnore BiConsumer; broadcast itself is identical, and the
  Rust models neither addition — correct for 1.21.4. Rank 2: 138/348.

  Item 93 (horizon-client-render/collisions.rs): comment-only carry-over.
  The strider liquid shape doc cited `Strider.getLiquidCollisionShape`
  (exists in neither jar; both versions have only
  `Strider.canStandOnFluid`) and `Block.column(16.0, 0.0, 8.0)`
  (1.21.5-only; `Block.column` does not exist in 1.21.4). The shape itself
  is already the 1.21.4 form: `from_cube([1, 2, 1], &[true, false])` is
  the bottom half-box = `LiquidBlock.STABLE_SHAPE`
  (`Block.box(0, 0, 0, 16, 8, 16)`), which
  `LiquidBlock.getCollisionShape` returns for a level-0 source when
  `CollisionContext.canStandOnFluid` (strider: lava) and the context is
  above it. `CollisionContext` placement (1.21.5-only) was already handled
  by item 242; `LiquidBlock.pickupBlock` is body-identical (only the
  parameter type widens, unused). Rank 2: 139/348.

  Item 94 (horizon-client-render/jigsaw_block.rs): comment-only
  carry-over. The two `JointType` docs cited
  `StructureTemplate.getDefaultJointType` (1.21.5-only as a named method);
  in 1.21.4 the rule (front-facing axis horizontal to ALIGNED, else
  ROLLABLE) is inline in `getJointType`, which
  `JointType::default_for_front` already implements exactly.
  `load`/`loadPalette`/`getJointType` differ only by the 1.21.5 NBT API
  refactor. `placeInWorld`'s barrier flags (1.21.4: 20 +
  `Clearable.tryClear`; 1.21.5: 820) belong to item 240 (`processor.rs`
  is in its files list) — NOT fixed here. `placeEntities` `moveTo`
  (1.21.4) vs `snapTo` (1.21.5) is a pure rename: byte-identical bodies.
  Rank 2: 140/348.

  Item 95 (horizon-client/advancements.rs): comment-only carry-over.
  The `potions()` and custom-data docs cited `PotionsPredicate` /
  `CustomDataPredicate.matches` (1.21.5's `core/component/predicates/`
  package, absent in 1.21.4); in 1.21.4 they are
  `advancements/critereon/ItemPotionsPredicate` /
  `ItemCustomDataPredicate`, with the same matching. `NbtPredicate.matches`
  changed signature (ItemStack to DataComponentGetter) with identical
  semantics; `PlayerPredicate` differs only in the `gameMode` access and
  the recipe codec construction; `AdvancementProgress` differs only in
  codec-lambda implementation (`Util.mapValues` vs a manual stream
  collect); `RecipeCraftedTrigger$TriggerInstance` is codec-only.
  Findings NOT fixed here (shared with the 1.21.5 tree, not a delta):
  `potions()` is called only from `BrewedPotionTrigger` with the
  trigger's single-Holder `potion` condition but reads a `v["potions"]`
  HolderSet form, so a specific-potion condition never filters; its
  `effects` check matches no predicate in either version's jar.
  Rank 2: 141/348.

  Item 96 (horizon-client/game/crossbow.rs): PORTED — `CrossbowItem`
  charge/load timing is the 1.21.5 refactor. In 1.21.4 the projectiles
  load in `releaseUsing`, not in `onUseTick` at full charge.
  `tick_crossbow` no longer loads at `now >= 1.0` (start/mid sounds
  unchanged); new `release_crossbow` (from `release_used_item`, slot
  threaded through from both callers) implements 1.21.4's
  `releaseUsing`: `getPowerForTime` = min(held/charge, 1); below 1 loads
  nothing; already-charged returns true; else `load_crossbow`
  (`tryLoadProjectiles`: multishot 3, creative intangible, ammo consumed
  when non-creative) + end sound with pitch `1/(rand*0.5+1)+0.2` at
  volume 1. `getUseDuration` stays 72000: with `useOnRelease` true
  (identical in both versions) the charge+3 duration is unobservable
  client-side. Tooltip: `appendHoverText` (1.21.4-only; 1.21.5 moved it
  to `ChargedProjectiles.addToTooltip`, which the 1.21.4 component
  lacks): first projectile only as `item.minecraft.crossbow.projectile`
  ("Projectile:" per the 1.21.4 lang) + display name; advanced +
  firework rocket appends the rocket's own lines indented with two
  spaces. `Stray.getArrow` is a rename only (`MOVEMENT_SLOWDOWN` to
  `SLOWNESS`, 600 ticks; the Rust already has it).
  `BowItem.releaseUsing` differs only in the `Level.playSound` overload.
  New ignored test `a_crossbow_loads_on_release_not_on_full_charge`
  (needs prepared resources) pins the release-load + early-release
  cases. Rank 2: 142/348.

  Item 97 (horizon-client/game/powder_snow.rs): PORTED — the powder
  snow inside shape is the 1.21.5 rule in the Rust, not 1.21.4's. In
  1.21.4 `BlockBehaviour.getEntityInsideCollisionShape` returns the whole
  block by default and `PowderSnowBlock` does not override it (its
  collision-shape override arrived in 1.21.5), so
  `Entity.checkInsideBlocks` fires `entityInside` on plain whole-block
  overlap. `falling_block.rs` required `fall_distance <= 2.5` or a
  collision with the 0.9-high FALLING top, and `motion.rs` fell through
  to the same shape test when falling fast; both now fire on plain sweep
  overlap (fall distance still reset, stuck multiplier 0.9/1.5/0.9).
  `getCollisionShape` was already the 1.21.4 form (`is_placement` is
  dead — the placement flag is always false in 1.21.4; `fallDistance`
  f64 vs f32 is exact widening, same 2.5 threshold). `entityInside`'s
  client parts are byte-identical; the fire-melt tail is inline in
  1.21.4 vs the applier's runBefore EXTINGUISH in 1.21.5, so the
  `powder_snow.rs` doc cited the 1.21.5 form — fixed. `pickupBlock` is
  body-identical (Player vs LivingEntity param, unused); `fallOn`
  differs only in the float/double param (same 4.0/7.0 sound
  thresholds). Rank 2: 143/348.

  Item 98 (horizon-world/worldgen/generator_decoration.rs): PORTED —
  `WorldGenRegion.setBlock` in 1.21.4 never reads its flags int
  (`setBlockState(pos, state, false)`; the postprocess mark happens
  whenever `hasPostProcess`). 1.21.5 passes the flags through and skips
  the mark when `flags & 16` (UPDATE_KNOWN_SHAPE) is set. The Rust's
  `World::set_block` had the 1.21.5 gate, reachable with bit 16 via the
  structure processor (`processor.rs:1614` passes `flags & !1 | 16` into
  `update_from_neighbour_shapes`), so structure-placed mushrooms,
  magma and soul sand lost their postprocess marks; the gate is removed
  and the flags param is now ignored. The `hasPostProcess` set is
  identical in both jars (brown/red mushroom + magma block; soul sand
  is not in the set in either version, so its `postprocess_above` entry
  is version-independent and left as is). `getBlockEntity` differs only
  in `getString` vs `getStringOr`; `levelEvent`/`playSound` only widen
  Player to Entity; `applyImplicitComponents` differs only in the
  parameter type; `placeSpike` differs only in `moveTo` vs `snapTo`
  (pure rename). Rank 2: 144/348.

  Item 99 (horizon-client-render/block_entity_anim.rs): comment-only
  carry-over. The three renderers' `render` methods are body-identical
  modulo the 1.21.5-only 7th Vec3 camera parameter (slot renumbering
  only; `Quaternionf` to `Quaternionfc` is a JOML API rename).
  `EndGatewayBlock.entityInside` is byte-identical (the 1.21.5 applier
  parameter is unused). `BellBlockEntity.tick` differs only in the
  `Level.playSound` overload. `EnchantingTableBlockEntity`
  load/saveAdditional differ only in the NBT API form. `BlockTags`:
  1.21.5 adds 6 tags (`camels_spawnable_on`,
  `dry_vegetation_may_place_on`, `edible_for_sheep`,
  `plays_ambient_desert_block_sounds`, `replaceable_by_mushrooms`,
  `sword_instantly_mines`) and drops `dead_bush_may_place_on`. One doc
  fix: `ConduitBlockEntity`'s method is `updateDestroyTarget` in BOTH
  jars; the 1.21.4 tree's doc said `updateAndAttackTarget`.
  Findings NOT fixed here (1.21.5-only tags referenced in the 1.21.4
  tree — recorded for their owning items): sheep eat via
  `minecraft:edible_for_sheep` (`goals.rs`, item 88) but 1.21.4's
  `EatBlockGoal` uses `IS_TALL_GRASS` = `BlockStatePredicate.forBlock`
  (SHORT_GRASS) or GRASS_BLOCK below, so sheep never eat short grass
  here; camels spawn via `minecraft:camels_spawnable_on`
  (`spawn_rules.rs`, item 88) but 1.21.4 has no `checkCamelSpawnRules`
  and no CAMEL entry in `SpawnPlacements` (camels spawn only via desert
  village structures), so camels should never naturally spawn here;
  huge mushrooms replace via `minecraft:replaceable_by_mushrooms`
  (`tree_blocks.rs`/`vegetation.rs`, items 252/253) but 1.21.4's
  `AbstractHugeMushroomFeature` uses `isAir || is(LEAVES)` in its space
  check and no tag in `place`, so mushrooms only replace air here.
  Also: `block_entity_nbt.rs` writes `CustomName` via the 1.21.5 NBT
  codec form (`storeNullable`, `ComponentSerialization.CODEC`) while
  1.21.4 writes a JSON string (`Component$Serializer.toJson`) —
  recorded for the block-entity-NBT items (notably item 163).
  Rank 2: 145/348.

  Item 100 (horizon-client-render/options_file.rs): PORTED —
  `Options.startedCleanly` is 1.21.5-only: neither 1.21.4's `Options`
  nor its `Minecraft` class contains the field (no `startedCleanly`,
  `resetting fullscreen`, or `unexpected shutdown` strings). The Rust's
  unclean-start recovery (init: read flag, set false, save;
  `onGameLoadFinished`: set true, save; leave fullscreen + clear
  `fullscreenResolution` when false) is the 1.21.5 mechanism; removed
  entirely: the `startedCleanly` entry from the options TABLE,
  `unclean_start_recovery`, the `game.rs` init block and both
  `onGameLoadFinished` markers, and the `core_tests` test that pinned
  the 1.21.5 behavior (a 1.21.5-behavior test, not a 1.21.4 one). The
  two `save()` calls are kept as plain persistence points. Everything
  else in `Options` is NBT-API-only: `dataFix`, `load`'s
  fancyGraphics-to-graphicsMode migration and `fullscreenResolution`
  store, `save`'s `fullscreenResolution` write (the Rust keeps the
  string verbatim and writes it back — the 1.21.4 form in effect),
  `processOptions` (identical except the `startedCleanly` line),
  constructor (`soundSourceVolumes` EnumMap factory form only).
  Rank 2: 146/348.

  Item 101 (horizon-client/game/falling_damage.rs): carry-over, one doc
  fix. `Damage::from_nbt` is already the 1.21.4 read (`HurtEntities` or
  the anvil default, `FallHurtAmount` or 0.0, `FallHurtMax` or 40 —
  equivalent to 1.21.5's `getBooleanOr`/`getFloatOr`/`getIntOr` with the
  anvil default); `impact()` is `Mth.ceil(fallDistance - 1.0F)` with the
  same floor/min/max and `0.05 + steps * 0.05` wear chance;
  `wear_anvil`/cancelDrop match; `finish_falling_block`'s `TileEntityData`
  merge is 1.21.4's `getAllKeys` copy loop (1.21.5's `forEach` copies the
  same tag per key); `causeFallDamage` returns false at the end in 1.21.4
  and the Rust consumes no boolean. The float-vs-double fallDistance
  param is provably the same ceil for float inputs. Doc fix:
  `finish_falling_block` cited 1.21.4's tick as 1.21.11. Known gap, NOT
  fixed here: the tree stores `Entity.fallDistance` as f64 everywhere
  (the 1.21.5 form; 1.21.4's field is float) — narrowing it is a
  tree-wide decision, like the `level/mod.rs` flags question.
  Rank 2: 147/348.

  Item 102 (horizon-client/mobs/bee.rs): PORTED — `Bee.customServerAiStep`
  in 1.21.4 reads `Entity.isInWaterOrBubble` (water, or the bubble column
  at the bee's feet — `Entity.isInBubbleColumn` reads the block state at
  the entity's block position); 1.21.5 narrowed it to `isInWater`. The
  Rust used `m.fluids.in_water` only, so a bee in a bubble column never
  drowned here; `server_ai_step` now also checks the block at the bee's
  feet for `minecraft:bubble_column`, with a regression test (5 ticks in
  a column count `under_water` to 5; out of it resets to 0).
  `isNightOrRaining` is NOT a delta: 1.21.4 `isNight()` equals 1.21.5
  `isDarkOutside()` (`isDay()` and `isBrightOutside()` are byte-identical:
  `!hasFixedTime() && skyDarken < 4`), and the Rust's `bees_stay_in_hive`
  day-time range 12542..23460 reproduces `skyDarken >= 4` exactly
  (verified numerically against `Level.updateSkyBrightness` with
  `Mth.cos`'s table and f32 rounding); its doc cited the non-existent
  `EnvironmentAttributes.BEES_STAY_IN_HIVE`, fixed to cite
  `Bee.isNightOrRaining`. `TorchflowerCropBlock` is identical:
  `getBonemealAgeIncrease` returns 1 in both; `getShape`'s `SHAPES` vs
  `SHAPE_BY_AGE` hold the same boxes (age 0: 5,0,5,11,6,11; age 1:
  5,0,5,11,10,11 — 1.21.5 builds them via `Block.column(6, 0, 6 +
  age*4)`); add/readAdditionalSaveData differ only in the NBT API form.
  Rank 2: 148/348.

  Item 103 (horizon-client/mobs/zombie_villager.rs): PORTED —
  `ZombieVillager.readAdditionalSaveData` calls `startConverting(player,
  time)` when `ConversionTime` is present, re-applying the cure's strength
  for the remaining time; the Rust's `read_nbt` set the cure but not the
  effect, so a loaded converting zombie villager lost its strength.
  `read_nbt` now adds `minecraft:strength` for the cure's remaining time
  (and drops weakness, as `startConverting` does) after the saved
  `active_effects` are loaded, with a round-trip test (a saved cure comes
  back with strength 4000). `startConverting` itself already matches:
  `DAMAGE_BOOST`'s registry id is `minecraft:strength` in 1.21.4 (the
  1.21.5 change was a field rename only); weakness removed; entity event
  16. `finishConversion`'s lambdas are identical modulo numbering
  (`!EnchantmentHelper.has` `PREVENT_ARMOR_CHANGE`;
  `dropPreservedEquipment` moves cursed stacks to villager slots
  index+300 — the Rust's `convert_to_villager` matches, verified against
  `Mob.dropPreservedEquipment`); `defineSynchedData`'s
  `createDefaultVillagerData()` is `new VillagerData(PLAINS, NONE, 1)`;
  `finalizeSpawn`'s byBiome/setType equals withType; `setVillagerData`
  clears offers on a profession change (`villager.rs`);
  `VillagerData`/`Offers`/`Xp` load in `entity_nbt.rs`. One comment fix:
  the strength amplifier is `Math.min(difficulty - 1, 0)`, 0 except on
  Peaceful where vanilla gives -1 (3 less melee damage) — this keeps 0
  everywhere. Rank 2: 149/348.

  Item 104 (horizon-client-render/block_entities.rs): carry-over.
  The ticker-visible behaviour of `LevelChunk` is identical in both
  versions. `postProcessGeneration` differs only in jump offsets;
  `clearAllBlockEntities` is identical; `promotePendingBlockEntity`
  differs only in `getString` vs `getStringOr` (same `DUMMY` check);
  `replaceWithPacketData` decodes the heightmaps in the packet reader in
  1.21.5 vs from the `CompoundTag` in 1.21.4, same data, and its
  `BlockEntityTagOutput` lambda is identical (`getBlockEntity(IMMEDIATE)`,
  type check, `loadWithComponents`). The `setBlockState` differences are
  all outside this file's ticker bookkeeping: 1.21.4 takes a boolean and
  calls `oldState.onRemove` server-side unconditionally, while 1.21.5
  takes flags and calls `preRemoveSideEffects` (unless flags & 256),
  `affectNeighborsAfterRemoval` for rails, and `onPlace` only when
  !(flags & 512) — the 1.21.5-only flags
  (`UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS` 256, `UPDATE_SKIP_ON_PLACE`
  512) already present in `level/mod.rs`, whose 1.21.5 form is a known
  open parent decision (item 52). The block-entity create/remove/mismatch
  path inside `setBlockState` is identical in both jars.
  Rank 2: 150/348.

  Item 105 (horizon-client-render/command_block.rs): PORTED.
  `BaseCommandBlock.save` in 1.21.4 writes `LastOutput` as a JSON string
  (`Component$Serializer.toJson`) and `LastExecution` only while
  `updateLastExecution && lastExecution > 0`; the Rust wrote the codec
  compound and any value but -1. `save` now writes
  `Tag::String(serde_json::to_string(component))` and skips
  `LastExecution` at 0. `load` reads `CustomName`/`LastOutput` only from
  String tags (1.21.4's `getString` drops a compound) and parses the JSON
  before `comp_text`: a malformed `LastOutput` becomes a literal of the
  parse error, as vanilla's `catch (Throwable)` does, and an unparsable
  `CustomName` is no name (`parseCustomNameSafe`); `name()` parses the
  stored JSON string the same way. `load` otherwise matches: `track_output`
  and `update_last_execution` default true, `LastExecution` -1 unless
  present and updating. `CommandBlockEntity.loadAdditional` is NBT-API-only
  noise (`getBooleanOr` vs `getBoolean`, same false defaults); the 1.21.5
  constructor's explicit powered/auto/conditionMet false inits are field
  defaults in 1.21.4; `isAutomatic`/`onModeSwitch`/`setAutomatic` differ
  in spacing only. Test
  `components_save_as_json_strings_and_zero_execution_is_not_kept` covers
  the styled round-trip, the skipped zero, and the dropped compound forms.
  Follow-up: crate-tests 45892679369 failed on
  `a_command_cart_saves_its_base_command_block`, whose fixture fed a bare
  non-JSON `LastOutput` string and expected it back verbatim (the old
  1.21.5-form literal read). 1.21.4's load calls the strict
  `Component$Serializer.fromJson` (`JsonParser.parseString`), so a bare
  string loads as a literal of the parse error; a vanilla-written save
  never holds one (`toJson` quotes it). The fixture now uses the JSON-string
  form. Rank 2: 151/348.

  Item 106 (horizon-client/crafting.rs): PORTED, two fixes. (1) The
  special-recipes loader gap: all 11 1.21.4 special JSONs carry only
  `type` + `category`, and the loader read ingredients from the JSON and
  dropped them; three types (`crafting_special_armordye`,
  `crafting_special_mapcloning`, `crafting_special_tippedarrow`) had no
  loader arm at all, so armor dyeing, map cloning and tipped arrows could
  not be crafted. Ported the 1.21.7 fix (b680e303) with every hardcoded
  input verified against the 1.21.4 server jar: `ing_or`/`result_or`/`dyes`
  fallbacks; armor dye targets `#minecraft:dyeable` with the sixteen dyes
  (1.21.4 has no `#minecraft:dyes` tag); banner duplication `#minecraft:banners`
  (vanilla tests `instanceof BannerItem`); shield decoration `#minecraft:banners`
  + shield; book cloning source `written_book` + material `writable_book`
  (1.21.4 has NO `#minecraft:book_cloning_target` tag — the 1.21.7
  fallback would have resolved to an empty ingredient here); firework
  rocket paper/gunpowder/firework_star, 3 out (assemble `iconst_3`);
  firework star `SHAPE_BY_ITEM` (large_ball fire_charge, burst feather,
  star gold_nugget, creeper `#minecraft:skulls`) with gunpowder/diamond/
  glowstone_dust; star fade firework_star; map extending filled_map +
  paper (`Items.MAP` in `MapExtendingRecipe` is only the `ShapedRecipe`
  dummy result); map cloning as a transmute, filled_map + 1..8 map,
  `add_count`; decorated pot `#minecraft:decorated_pot_ingredients`;
  tipped arrow as an imbue, lingering potion + arrow, 8 out (assemble
  `bipush 8`). `BookCloningRecipe` itself already matched
  (`WRITTEN_BOOK`/`WRITABLE_BOOK` items; `tryCraftCopy` caps generation
  at 2, which the Rust's `[0, 1]` bounds default reproduces; remaining
  items stop at the first written book). Test
  `special_recipes_load_from_type_and_category_only_jsons` loads all 11.
  (2) `DyedItemColor`: 1.21.4's record is `(rgb, showInTooltip)` with
  `CODEC` the record plus a bare-int alternative and `STREAM_CODEC`
  int+bool; the Rust kept a bare `i32`, so a compound `dyed_color`
  decoded to `None` (the colour was lost) and the line could never hide.
  `StackData` gains `dyed_color_hidden` (default shown); the decode
  accepts an int or `{rgb, show_in_tooltip}` in both decode paths; the
  encode writes the record when hidden; `dyed_line` gates on it (the
  packet path already hid it through `TooltipDisplay`); `applyDyes`
  preserves the flag, as vanilla does (it keeps the existing
  `showInTooltip`); the cauldron wash clears it. Only the loot
  `ToggleTooltips` function and explicit components set it false.
  `Recipe.recipeBookCategory` is static-init only (`KEY_CODEC` added in
  1.21.5): no change. Rank 2: 152/348.

  Item 107 (horizon-client/mobs/ravager.rs): carry-over.
  `CropBlock.entityInside` and `PitcherCropBlock.entityInside` are the
  same trample in both versions (ServerLevel + Ravager + mobGriefing ->
  `destroyBlock(pos, true, entity)`; the super call is a no-op — 1.21.4
  has no `VegetationBlock`, and neither `BushBlock` nor `DoublePlantBlock`
  defines `entityInside`). The Rust's deflated-bounds sweep breaks
  wheat/carrots/potatoes/beetroots/torchflower_crop/pitcher_crop with drops
  under mobGriefing, matching. The shape deltas are real but NOT in Rust
  source: 1.21.4's `PitcherCropBlock` keeps per-age
  `UPPER_SHAPE_BY_AGE` (upper: ages 0/1/2/4 `box(3,0,3,13,15,13)`, age 3
  `box(3,0,3,13,11,13)`) while 1.21.5 computes the upper half as
  `column(w, 0, max(0, v2-16))` (ages 0/1 empty, age 2
  `box(3,0,3,13,1,13)`, age 3 `box(3,0,3,13,12,13)`, age 4
  `box(3,0,3,13,16,13)`); the lower halves and both collision shapes
  (`BULB box(5,-1,5,11,3,11)`, `CROP box(3,-1,3,13,5,13)`) are identical,
  and 1.21.4's upper-half collision falls back to
  `BlockBehaviour.getCollisionShape` (the visual shape) while 1.21.5
  returns `Shapes.empty()` for the upper half. Shapes live in the
  prepare-derived `cached-collision.nbt` (`VanillaCollisionOracle` vs the
  1.21.4 jar), so the 1.21.4 tree gets the 1.21.4 shapes by construction.
  `CropBlock`'s `SHAPE_BY_AGE` is identical in both
  (`box(0,0,0,16,2+2*age,16)`, ages 0..7). Rank 2: 153/348.

  Item 108 (horizon-protocol/buf.rs): carry-over. `ByteTag`/`EndTag`/
  `StreamCodec` behaviour is identical in both versions. `ByteTag` is a
  plain class in 1.21.4 (field `data`, package-private ctor) vs a Record
  in 1.21.5 (field `value`); the wire form is one byte either way, the
  `valueOf` cache and `write` are identical, `getAsX` vs `xValue` are
  renames only. `EndTag.getAsString` differs only in construction (1.21.4
  returns `getAsString`, 1.21.5 builds via
  `StringTagVisitor.visitEnd`) with the same empty result.
  `StreamCodec.composite` overloads (1..8 fields) differ only in
  anonymous-class numbering. Verified the constants the Rust cites:
  `EndTag.sizeInBytes()` = 8 (`MIN_TAG_COST`), `ByteTag.sizeInBytes()` =
  9, and `FriendlyByteBuf.readNbt` passes
  `NbtAccounter.create(2097152L)` in both jars. The `packet!` macro
  mirrors `composite` (fields in order); the NBT limits already match.
  Rank 2: 154/348.
