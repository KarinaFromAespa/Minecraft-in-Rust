# 26.3-Leakage Audit

The standing guard against the bug class that broke 1.21.7's water: code
carried over from the 26.3 base reads a tag, attribute or gamerule that
does not exist in the target version's data, and the lookup silently
degrades (empty set / false / 0) instead of failing loudly. The tool is
`tools/audit_26_3_leakage.py`; this document is its first findings report
and the proposal for where it hooks into the workflow.

## The incident

1.21.7's `FlowingFluid.canHoldAnyFluid` read the 26.3 block tag
`#washed_away_by_fluids`. The tag does not exist in 1.21.7, so it resolved
to an empty set and water could not spread at all (fixed in ee7ab396 by
restoring the vanilla list form). The parity-sweep pilot found the same
pattern in four more defects (G1 bubble columns, G3 lava fire spread via
`#blocks_lava_fire_spread`, G5 fluid spreading, G6 dripstone cauldrons via
`#speleothems`) plus G4 (lava ignition ignoring `doFireTick`). Every one of
them is mechanically findable: the code references a name that does not
exist in the target version's data. The owner ordered a guard so this
class is caught by a tool instead of in playtests.

## The tool

```
python3 tools/audit_26_3_leakage.py 1.21.7
python3 tools/audit_26_3_leakage.py 1.21.7 --jar jars/1.21.7-server-named.jar --json
python3 tools/audit_26_3_leakage.py --all --jar-dir jars/
python3 tools/audit_26_3_leakage.py --all --jar-dir jars/ --baseline tools/leakage-baselines.json
```

For one version (or all), it extracts what the version tree's Rust code
references and compares it against what actually exists in that version's
own vanilla data:

| Family | Code side | Data side |
|---|---|---|
| Tags | `tags!` macro list, `FluidTags` fields, `tag()` / `is_tag()` / `has_tag()` / `block_tag()` / `biome_tag()` literals, `<registry>_tags.get()` map reads, `#minecraft:…` literals | Tag inventory from the jar (bundler-aware) or prepared `.local` data, unioned across registries |
| Attributes | `ATTRIBUTES` table + `attributes["…"]` read sites | The version's attribute registry (from the jar's `EnvironmentAttributes` class strings) ∪ biome/dimension JSON keys |
| Gamerules | `RULES` table (+ standalone `Rule` consts), `GameRules` struct fields, `.int()` / `.bool()` / `.set()` string reads, `gamerules::rule()` | The jar's language file (`gamerule.*` keys), through Mojang's 1.21.11 rename table |

Data sources, in priority order per version: `--prepared DIR`, `--jar
PATH`, the tree's own `.local/horizon-client/<v>/`, `.local/minecraft/<v>/`
jars, `--jar-dir` (`<v>-server.jar`, `<v>-server-named.jar`,
`<v>-client-named.jar`, `<v>-client.jar`). The source used is recorded in
the output. **A version with no usable data source is `unmeasured`, never
clean.** The tool never guesses one version's data from another's.

Exit codes (mirroring `tag_audit.py`): 0 no leaks (review notes may
exist) · 1 leak(s) found · 3 one or more versions unmeasured (and no
leaks) · 2 usage error. With `--baseline`, exit 1 only on leaks that are
new against the baseline.

### Severities

- `leak` (fails): a missing tag the production code actually reads, or a
  gamerule read naming a rule the port's own table lacks (the read
  silently returns false/0).
- `review` (does not fail): an attribute read in a version without the
  attribute system, or an attribute/table entry the version's registry
  lacks, or a vanilla gamerule with no port equivalent. A human triages
  these: attribute reads may have a legacy fallback, and removed rules
  may be genuinely unrepresentable.
- `dead_definition`: a tag field defined but never read (harmless — the
  1.21.7 water fix left `washed_away_by_fluids` defined but unused).
- `test_only`: a missing tag referenced only from tests/fixtures.
- `note`: informational (26.3-only gamerules carried into the version,
  the size of the unpopulated `ATTRIBUTES` table, port-invented rules).
- `accepted`: listed in `tools/leakage-exceptions.json` with a reason —
  reported, not hidden, and not failing. Nothing is excepted yet.

### Design decisions (what keeps this signal, not noise)

1. **Used vs defined.** Only a missing tag that production code reads is
   a leak. Defined-but-unused is a note. Usage covers the alias form
   (`let t = &self.b.tags;` then `t.<field>`).
2. **Union across registries.** A tag name is missing only if absent from
   every registry — one name can live in several, and `.tag()` lookups
   are registry-agnostic.
3. **Vanilla data is ground truth.** Inventories come from the version's
   own jar or prepared data, never from 26.3 or a neighbour.
4. **The gamerule check runs through Mojang's rename table.** In 1.21.11
   Mojang renamed the whole gamerule registry from camelCase to
   `minecraft:<snake_case>` ids (`doDaylightCycle` → `advance_time`,
   `disableElytraMovementCheck` → `elytra_movement_check`, and the fire
   rules merged into `fire_spread_radius_around_player`). The port's
   table uses the new names, so a raw name diff would read every rename
   as a gap and an invention. The tool's `RULE_RENAMES` is the
   authoritative old→new table, extracted from Mojang's
   `GameRuleRegistryFix` (javap, 26.3 jar) and cross-checked against the
   port's own transcription (`horizon-world/src/datafix/fixes_c.rs`,
   `GAME_RULES` — identical, 59 pairs). With it, the port's table
   matches the 26.3 registry exactly (59/59).
5. **Attribute findings are review-only.** The tool lists read sites; it
   cannot verify legacy fallbacks mechanically.
6. **Exceptions are explicit.** Tags the port deliberately synthesizes
   belong in `tools/leakage-exceptions.json` with a reason and a code
   pointer — and only when the synthesis covers every flagged read site
   (see the `supports_*` triage below).

## Findings — per-version table

Run 2026-11-22 against branch `tools/leakage-audit` at 9d8b7557 (main
8a3f87b6 + tool), jar data source for all 19 versions. Counts are
findings, not unique names (three tags appear twice — once as a block tag
and once as a fluid tag — see the 1.21.7 note).

| Version | Status | Leaks | Review | Dead defs | Test-only | Notes | Tags in data | Attributes | Gamerules |
|---|---|---|---|---|---|---|---|---|---|
| 1.20.5 | fail | 90 | 9 | 2 | 10 | 10 | 430 | 0 (absent) | 51 |
| 1.20.6 | fail | 90 | 9 | 2 | 10 | 10 | 430 | 0 (absent) | 51 |
| 1.21 | fail | 84 | 10 | 2 | 8 | 10 | 466 | 0 (absent) | 52 |
| 1.21.1 | fail | 84 | 10 | 2 | 8 | 10 | 466 | 0 (absent) | 52 |
| 1.21.2 | fail | 79 | 10 | 2 | 6 | 8 | 493 | 0 (absent) | 54 |
| 1.21.3 | fail | 79 | 10 | 2 | 6 | 8 | 493 | 0 (absent) | 54 |
| 1.21.4 | fail | 77 | 10 | 2 | 6 | 8 | 498 | 0 (absent) | 54 |
| 1.21.5 | fail | 70 | 10 | 2 | 6 | 6 | 514 | 0 (absent) | 56 |
| 1.21.6 | fail | 70 | 10 | 2 | 6 | 6 | 524 | 0 (absent) | 57 |
| 1.21.7 | fail | 62 | 10 | 3 | 7 | 6 | 524 | 0 (absent) | 57 |
| 1.21.8 | fail | 69 | 9 | 3 | 6 | 5 | 524 | 0 (absent) | 57 |
| 1.21.9 | fail | 65 | 10 | 1 | 6 | 1 | 539 | 0 (absent) | 63 |
| 1.21.10 | fail | 65 | 10 | 1 | 6 | 1 | 539 | 0 (absent) | 63 |
| 1.21.11 | fail | 63 | 8 | 1 | 5 | 0 | 552 | 45 | 63 |
| 26.1 | fail | 23 | 5 | 2 | 5 | 0 | 678 | 48 | 63 |
| 26.1.1 | fail | 23 | 5 | 2 | 5 | 0 | 678 | 48 | 63 |
| 26.1.2 | fail | 23 | 5 | 2 | 5 | 0 | 678 | 48 | 63 |
| 26.2 | review | 0 | 2 | 2 | 5 | 0 | 710 | 48 | 63 |
| 26.3 | fail | 1 | 0 | 1 | 5 | 0 | 792 | 51 | 63 |

("Attributes" is the registry size extracted from the jar; 0 means the
version predates the attribute system. "Tags in data" is the union tag
count across registries; "Gamerules" is the lang-file rule count.)

Reading the table:

- **The gradient is the story.** Leak counts fall as versions get newer
  (90 at 1.20.5 → 62 at 1.21.7 → 23 at 26.1 → 0 at 26.2): older versions
  lack more of the 26.3 systems. The identical counts in pairs
  (1.20.5/1.20.6, 1.21/1.21.1, 1.21.2/1.21.3, 1.21.9/1.21.10,
  26.1/26.1.1/26.1.2) are the mechanical ports sharing a tree and an
  inventory.
- **1.21.7 (62 leaks)** is the best-measured version: it has the water
  fix (so `washed_away_by_fluids` is a dead definition, not a leak) and
  the motion-tag derivation (so the `blocks_motion` family is not
  referenced at all). Its 62 leaks are 59 unique tag ids — three
  (`supports_frogspawn`, `supports_lily_pad`,
  `supports_sugar_cane_adjacently`) are counted twice, once as a block
  tag and once as a fluid tag, because the code reads both registries.
- **1.21.8 (69)** has 8 leaks 1.21.7 lacks: `washed_away_by_fluids`
  (the water carry-over is paused), the `blocks_motion` family +
  `blocks_fluid_flow`, `can_glide_through`, `causes_suffocation`,
  `suppresses_bounce` (1.21.7's launch-crash fixes removed those
  references; 1.21.8's tree still reads the tags). 1.21.7 has one leak
  1.21.8 lacks (`copper`, read by 1.21.7's `golems.rs`).
- **1.21.11 (63)** still carries `washed_away_by_fluids` as a live leak
  (the 1.21.11 water carry-over is pending) and the `blocks_motion`
  family. Its 8 review findings are the 6 attributes 26.x added after
  1.21.11 (`block_light_tint`, `night_vision_color`,
  `ambient_light_color`, `straw_bed_rule`, `natural_mob_spawns`,
  `creature_world_gen_spawn_probability`): the 1.21.11 tree registers
  26.3's 51-entry `ATTRIBUTES` table, but 1.21.11 vanilla has only 45.
- **26.1/26.1.1/26.1.2 (23 each) are NOT clean**, contrary to the
  "26.x shares the 26.3 systems" assumption: they lack the newest tags
  (`washed_away_by_fluids`, the `blocks_motion` family, `speleothems`,
  the teleport tags, the sulfur tags). The 26.x systems arrived
  gradually; 26.1 predates several of them.
- **26.2 is the only clean tree** (0 leaks; 2 review findings — reads of
  the 26.3-only attributes `natural_mob_spawns` and
  `creature_world_gen_spawn_probability` in `biome.rs`).
- **26.3 itself has 1 leak**: `minecraft:consumable_does_not_teleport_to`,
  read by `mobs/fox.rs:438`. The 26.3 jar has `cat_`, `enderman_` and
  `shulker_does_not_teleport_to`, `entities_can_teleport_to` and
  `dangerous_for_teleportation` — but no `consumable_` variant. The fox
  teleport guard is dead in the base. (Two further 26.3 hits from the
  first run — `#minecraft:overworld/nether_carver_replaceables` — were
  migration data in `horizon-port-pack/src/upgrade/`, not lookups; the
  tool now excludes that module.)

## Known positives verified

1.21.7 before/after, same jar, same tool:

| Check | Pre-fix (37c59445) | Post water fix (ee7ab396) | Post trees fix (8a3f87b6) |
|---|---|---|---|
| `washed_away_by_fluids` (G5) | **leak** (`fluids.rs:360`) | dead_definition | dead_definition |
| `speleothems` (G6) | leak | leak | leak |
| `blocks_lava_fire_spread` (G3) | leak | leak | leak |
| `enables_bubble_column_push_up` / `drag_down` (G1) | leak | leak | leak |
| `bubble_column_can_occupy` (G1, fluid) | leak | leak | leak |
| Total leaks | 63 | 62 | 62 |

The tool measures the water fix exactly: the delta between the pre-fix
and post-fix runs is precisely one finding, and it moves from `leak` to
`dead_definition` (the field is still defined in `data.rs:426`, just no
longer read). G1/G3/G6 remain live leaks at the current tip — no fix has
landed for them yet.

G4 (`doFireTick`) needs a careful triage, because the tool deliberately
does NOT flag it — and that is correct. Mojang merged `doFireTick` +
`allowFireTicksAwayFromPlayer` into `fire_spread_radius_around_player`
(0 = never, 128 = near players, −1 = everywhere), and the merge is exact
for 1.21.7's three states. The port's table HAS the merged rule, and the
port's own datafix (`fixes_c.rs`, `game_rule_registry`) implements the
merge faithfully, so saves migrate correctly. G4 is a CODE gap — the
lava-ignition path never consults the rule — not a table gap, and this
tool audits tables and references, not behaviour. The parity sweep owns
G4.

## The `supports_*` triage (why nothing is excepted yet)

The trees fix (56e86819) synthesizes eleven `supports_*` tags from their
1.21.7 equivalents — but only inside `GeneratorData::load`
(`worldgen/generator.rs`), i.e. only for worldgen decoration. The runtime
`Tags` struct resolves from a separate load (`built.tags` in
`level/data.rs`), which the synthesis never touches — and the prepared
`.local` `block-tags.json` (194 tags, exactly the jar's set) confirms no
synthesis happens at prepare time either. The runtime survival checks in
`level/survive.rs` read the same tags directly
(`Class::Vegetation => b.in_set(ground, &t.supports_vegetation)` and
twenty-odd siblings), so every one of them still resolves to an empty
set at runtime: hand-planted saplings, crops, cacti and sugar cane fail
survival even though worldgen-placed trees now decorate. The PORT-NOTES
entry for the fix says gameplay survival "uses the separate class-based
implementation in `level/survive.rs`" — true in structure, but that
implementation delegates to the missing tags. The `supports_*` leaks are
therefore real, and the exceptions file deliberately does not list them.
An exception becomes appropriate only when a fix covers the runtime
reads too (extend the synthesis to the runtime tag load, or restore the
vanilla list form the way the water fix did) — at which point the
exception entry records the code pointer.

## False-positive analysis

What the tool was tuned NOT to flag, and the evidence for each:

- **Defined-but-unused tags** (`washed_away_by_fluids` post-fix,
  `convertible_to_mud`, `lanterns`): reported as `dead_definition`, not
  leaks. Verified: the only post-fix references to
  `washed_away_by_fluids` outside `data.rs` are comments.
- **Test fixtures** (`is_goblin`, `is_moon`, `no_such_tag`, `t/level_1`,
  …): `test_only`, via file-level (`tests/`, `*_tests.rs`) and
  `#[cfg(test)]` / `#[test]` range detection.
- **Comments and raw strings.** Comment text is stripped before
  scanning (the post-fix `washed_away_by_fluids` comments do not
  count as uses). Raw-string contents are blanked: they hold JSON blobs
  and test data, not code — this removed six test-fixture hits and the
  JSON-embedded tags in `horizon-port-pack`'s `carry.rs`/`oracle.rs`.
- **Migration data.** `horizon-port-pack/src/upgrade/` is excluded: its
  tag strings name old-version data by design (the carver
  replaceables).
- **NBT/property reads that look like gamerule reads.** `.int()` /
  `.bool()` / `.get()` are generic method names; an early version of
  the gamerule scanner produced 61 junk `unknown_rule_read` findings
  from NBT compounds (`r.get("CustomName")`), server properties
  (`r.int("server-port", 25565)`) and datafix code. The scanner now
  requires the GameRules API shape (single-argument `.int()`/`.bool()`
  on a rules-like receiver, `.set()`/`.set_string()`, or
  `gamerules::rule()`), drops `.get()` entirely (the GameRules `.get`
  is only ever called with a variable in this tree), and skips
  namespaced names (save-data keys). Result: zero `unknown_rule_read`
  findings on any version.
- **Gamerule renames.** Without the rename table, 1.21.7 showed 15
  spurious gamerule gaps (every rename read as a gap). With Mojang's
  table, 1.21.7's genuine gamerule gaps are exactly two: the removed
  rules `spawn_chunk_radius` and
  `entities_with_passengers_can_use_portals` (see below).
- **Stale lang keys.** The lang file keeps keys for renamed/removed
  rules (`commandBlocksEnabled`, `enableCommandBlocks` — both renamed
  to `command_blocks_work`; the two removed rules above). The rename
  table absorbs the renamed keys; the removed-rule findings are
  suppressed for rework versions (jars containing
  `GameRuleRegistryFix`, i.e. 1.21.11+), where the rules genuinely do
  not exist.
- **The `mineable/*` tags.** The pilot noted prepared data renames four
  tool tags; checked directly, the 1.21.7 jar already has
  `mineable/axe` (and no `axe.json`), so there is no discrepancy to
  flag.

Two findings classes deserve a reader's judgement rather than blind
trust:

- **"Dead feature" leaks.** Some flagged tags belong to features the
  version lacks entirely (copper golems, shelves, sulfur cubes,
  nautili in 1.21.7). The code path is inert because the blocks/items
  do not exist — but the reference is still 26.3 logic in the tree, and
  it becomes live the moment the feature is (partially) back-ported.
  They are leaks by the tool's definition; triage them as a group when
  the feature lands or is deliberately excluded.
- **Negative checks.** `cannot_support_seagrass` / `cannot_support_kelp`
  / `cannot_support_snow_layer` are exclusion tags: with the tag
  empty, the exclusion never fires, so the block survives where
  vanilla would reject it. The failure mode is inverted (too
  permissive, not too strict) but it is still a leak.

## Validation

- **Against `tag_audit.py`.** The tag-literal scan was compared with
  `tag_audit.scan_tree`'s tokenizer pass on 1.21.7: the production
  missing-tag sets are identical. (The only diffs are test-only: one
  malformed `enchantable/` literal the tokenizer keeps and this scan
  rejects, and raw-string fixtures this scan sees and the tokenizer
  does not — both quarantined as `test_only`.) `tag_audit` cannot see
  the struct-field references at all, which is why this tool exists:
  of 1.21.7's 62 leaks, 36 come via the `tags!`/`FluidTags` fields
  (four of those also have a literal read of the same tag).
- **Against prepared data.** The 1.21.7 `.local` `block-tags.json` has
  exactly the jar's 194 tags and none of the flagged ones — the
  jar-based findings are not an artefact of the data source. (The
  prepared-data path is exercised the same way: `--prepared`.)
- **Against the port's own tables.** The gamerule rename table matches
  the port's `fixes_c.rs` `GAME_RULES` table exactly (59 pairs), and
  the port's rule table matches the 26.3 registry exactly (59/59). The
  attribute registry extraction recovers 51 ids for 26.3, 48 for 26.2,
  45 for 1.21.11 — matching each version's `EnvironmentAttributes`
  class — and 0 for 1.21.7 (no system).
- **Before/after.** The water fix moves exactly one finding, `leak` →
  `dead_definition` (table above).

## Known limitations

- **Data-driven tag names are not audited.** `Class::AttachedStem`
  reads `data.support_tag`, a per-block tag name that comes from the
  prepared data, not from code. If that name is wrong for the version,
  this tool does not see it. (Same for any tag name that arrives via a
  variable rather than a literal.)
- **Attribute fallbacks are not verified.** Attribute findings are
  `review` for exactly this reason — the mob fix's legacy fallback at
  `biome.rs:244` is the model of what a human confirms.
- **Behaviour gaps are out of scope.** G4 (lava ignition not consulting
  the fire rule) is invisible to a reference audit; the parity sweep
  owns behaviour.
- **Prepared-data runs have a partial attribute inventory** (biome/dimension
  JSON keys only — 13 of 51 for 26.3). The jar source is authoritative
  for attributes; prefer it when the attribute family matters.
- **The tool reads the tree as committed.** Uncommitted local changes
  (including a live stream's workdir state) are audited as-is; the
  baseline ratchet is what keeps a work-in-progress honest.

## Recommended workflow hook

**A CI job that runs the audit per version with the jar source, gated on
the baseline ratchet — not a change to the D1–D6 gate.**

Reasoning:

1. **Every version fails today.** A zero-leak gate would block all 19
   trees on day one, including 26.2's clean tag set (which still carries
   review items) — it would be disabled within a week. The baseline
   (`tools/leakage-baselines.json`, generated from the run in this
   document) records today's debt per version, and CI fails only when a
   run adds a leak that is not in the baseline. Fixes shrink the
   baseline; the ratchet only ever tightens. This is the same shape as
   the programme's existing tolerance for known-red harnesses: record
   the state, forbid regression.
2. **CI, not a pre-merge ritual.** The repo merges by branch push, not
   by pull request, so a "run it before you merge" rule depends on
   human discipline at exactly the moment attention is elsewhere. A CI
   job runs the same way every time. The job belongs next to
   port-verify (same trigger: a version tree changes), or as a small
   standalone workflow (`leakage-audit.yml`) that takes a version
   input and runs on push to `versions/**` — the standalone workflow
   is the cheaper first step, since port-verify's judge stages the
   tree as a fresh single-commit repo and the audit wants the jar
   (downloaded from Mojang's piston-meta, ~60 MB, no Java needed —
   the tool is stdlib-only Python and runs in ~40 s per version).
   Either way it must run with `--baseline` from the first day, or it
   is either red forever or green by ignoring everything.
3. **Not a D-gate change.** Folding the audit into `delta_verify.py`
   (a D7, or part of D1) is the natural end state — D1 already owns
   the tag audit — but gate changes are Freebuff's call by programme
   discipline, and this tool is one run old. The right sequence is:
   run it in CI as its own job, let one port cycle prove the signal
   (no false-positive fires, at least one real leak caught before
   merge), then propose the gate integration to Freebuff with that
   evidence. The baseline file is the artefact that makes the
   proposal concrete: "D1 gains a no-new-leaks check" is a one-line
   gate change once the job exists.
4. **Where the human looks.** The job's output should post the
   per-version summary table (the one at the top of this document) as
   a job summary, so a reviewer sees the counts without opening a log.
   New leaks list their file:line; review findings stay visible but
   non-blocking.

What this does not replace: the parity sweep (behaviour), `tag_audit.py`
(D1's prepared-data tag check — keep both; they see different reference
forms), or the vanilla-check rule (a human still verifies behaviour
against the version's own jar — the tool only checks that the names
exist).

## Reproducing this report

```
# jars are NOT committed; download or symlink them into jars/ first
# (see tools/audit_26_3_leakage.py --help; the run above used the 19
# vanilla server/client jars, sha1-verified from piston-meta)
python3 tools/audit_26_3_leakage.py --all --jar-dir jars/ --json > /tmp/audit.json
python3 tools/audit_26_3_leakage.py --all --jar-dir jars/ --baseline tools/leakage-baselines.json
```
