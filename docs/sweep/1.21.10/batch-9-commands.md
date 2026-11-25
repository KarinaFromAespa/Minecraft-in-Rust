# Batch 9 — Commands (1.21.10 parity sweep)

Date: 2026-11-25. Branch: `parity-sweep-1.21.10` (Batch 0's oracle
measurement landed in `.local/measure-oracles/results.json`, Nov 24
22:25; the `docs/sweep/1.21.10/batch-0-oracles.json` doc had not landed
at write time — §4 cites the measured table directly).
Scope: parsing, execution, selectors, functions, scoreboard — 22 drivers;
57 argument types (1.21.10's own list), 59 game rules.
Ledger slice: `slice.json` (this directory).

## 0. Execution status — read first

**The `command_parse` run MEASURED.** The 1.21.10 probe compiles: this
batch's driver built it end-to-end via
`tools/test_command_parse_parity.py` (§2) — the build step (`cargo
build --profile fast -p horizon-client-lib --example client_probe`)
compiles AND links the probe, strictly stronger than Batch 0's `cargo
check`, which was still running when the VM rebooted at ~23:50 UTC Nov
24 and never completed. H01 does NOT reproduce. The run itself: 32,365
commands × 3 permission levels (corpus regenerated from 1.21.10's own
tree export, md5 `6fa6f661f641dc84f86c099184f1c644`), 3,325 / 3,312 / 1
mismatches (perm 4 / 2 / 0); node diff 157 / 153 / 0 inputs.

Everything else in this report is static triage (method 2: javap +
source + oracle data + datagen), and it stands on its own: the
re-checks below were all verified without the run, and the run's
mismatch classification (§2) confirms them empirically where the
corpus reaches them.

**Headline: the 1.21.10 command surface is a 26.3-era tree carried
against a 1.21.10 vanilla that predates it — with the era template's
adaptations only partly applied.** The 1.21.10 tree is a delta port of
the 1.21.11 era template (719/719, merged `fcb0e7e3`), and the template
did adapt some command code for the era (the `/worldborder` integer
time, the clock-free `/time`, the removed `stopwatch`, the
`fetchprofile` name+id form, the `vanilla_key` gamerule mapping for
persistence) — but the release tree still ships four 26.x-only
commands (`compute`, `posteffect`, `swing`, `unpublish`), the
26.3 snake_case gamerule table against vanilla's camelCase rules, the
26.3 62-entry argument-type table against vanilla's 57, and 1.21.7-era
parse code (particle/modifier handling, selector rollback) that the
26.2 port fixed but this tree never carried. Of the 17 primary 1.21.7
items, 13 reproduce (G01, G02, G03, G05, G06-partial, G07, G08, G09,
G12, G13, O01, O02, O03), 3 are closed/fixed (G04 inverted by a
vanilla change, G10 fixed in-tree, G11 closed against real vanilla),
and O04 reproduces as a prepared-data gap. Of the 26 secondary 26.1.2
items, the verdicts track 26.1.2's except where 1.21.10's vanilla
differs: G19 is WITHDRAWN (vanilla 1.21.10 has no `/swing` at all, so
the whole command folds into G02), G01 flips from pass to gap
(1.21.10's vanilla uses camelCase gamerules), and the oracle failures
take the 1.21.7 forms (O01 6 errors, O02 19 client errors) rather than
26.1.2's (3 and 6). One NEW oracle defect appears: `VanillaCommandExecOracle`
fails javac with 45 errors (B9-O05) — it was never adapted for 1.21.10,
so the linchpin `command_exec` driver is oracle-blocked on top of its
fixture blocker. The tally: 24 as-tallied (5 pass, 19 gap) = 20
behaviour (4 pass, 16 gap) + 4 non-behaviour (1 pass, 3 gap). B9-O05
is an additional measured oracle gap outside the as-tallied scheme
(26.1.2 precedent: new items are not counted as checks); counted, the
totals become 25 as-tallied (5 pass, 20 gap) with non-behaviour 1 pass
/ 4 gap.

Two corrections to the brief, both verified against 1.21.10's own
tree (§6): (1) the brief's drop-list parenthetical said `fetchprofile`
and `publish` were 26.x-only — both exist in vanilla 1.21.10's
integrated tree (`fetchprofile` with `name` + `id` branches only,
`publish` with `allowCommands` → `gamemode` → `port`), so their EXTRA
corpus cases were KEPT. (2) The brief's expected gamerule count
implied the 26.x form; vanilla 1.21.10 has 59 camelCase rules in the
tree (60 registered in the jar, `minecartMaxSpeed` feature-gated out).

Prepared data used: `versions/1.21.10/.local/horizon-client/1.21.10/`
(copied from the candidate-test prepared copy, 143 MB), including
`commands.json`, `command-registries.json`
(VanillaCommandParseOracle export), `registries.json` — but NO
`assets/` directory at all, so no lang file (B9-O04). The language leg
was completed for the run by extracting `assets/minecraft/lang/en_us.json`
from the 1.21.10 client jar (md5 `d166df536bf7f617599990f8437ba18c`,
jar-identical, 7,583 keys, contains `argument.entity.invalid`) —
exactly what the port's own `horizon-prepare languages` step does.
All behaviour claims are grounded in 1.21.10's OWN jar/data — the
named server-game jar
(`e6-deltas/versions/1.21.10/jars/server-game-named.jar`, sha1
ba1fc89ead95b8ebdb8727f84e2bd487533c8b7f, Mojang-mapped), the 1.21.10
client jar (`client-named.jar`, sha1
715f58165acfaf8b61ab921064f08c202b41bd0c), the vanilla 1.21.10 datagen
reports, or the oracle's own exports. The 26.3 base's form was never
trusted. (The `server.jar` in the same directory is a Mojang bundler
jar, sha1 95495a7f485eedd84ce928cef5e223b757d2f764 — never javap'd.)
DataVersion 4556, protocol 773, Java 21.

## 2. The command_parse run (measured)

Run via `b09/run-command-parse.sh` (the tree's
`tools/test_command_parse_parity.py`, byte-identical copy, from the
`b09/driver/` shadow dir; corpus generator adapted per §6). The first
launch (22:17 NPT Nov 24) was killed by a VM reboot at ~23:50 UTC Nov
24 during the cargo build (after ~232 crates); the driver was
relaunched at 23:51 UTC and resumed from the cached `target/` dir
(2.4 GB survived the reboot). The java steps re-ran in ~1 minute on
the fresh VM (vs ~10–11 min each under load the first time); the
resumed cargo build finished in 29m 31s (`Finished fast profile`,
`compile-rust.log`); the three permission runs completed 00:24 UTC
Nov 25. Corpus md5 `6fa6f661f641dc84f86c099184f1c644` (identical across
both launches — the regeneration is deterministic). Artifacts in
`b09/run-artifacts/` (`report.json`, `corpus.txt`, `tree.json`,
`registries.json`, per-perm `mismatches.jsonl` + `node-diffs.json`,
compile logs, `driver.log`, `classify.txt`, `node-diff.txt`); the
full per-perm `vanilla.jsonl` / `rust.jsonl` remain in the worktree's
`.local/command-parse-parity/` (not copied — ~130 MB; every divergence
is in the preserved mismatches files).

### 2.1 Per-permission results

| Perm | Commands | Match | Mismatches | ok-differs | cursor-differs | error-text-only | suggestions |
|---|---|---|---|---|---|---|---|
| 4 | 32,365 | 29,040 (89.7%) | 3,325 | 145 | 168 | 2,779 | 233 |
| 2 | 32,365 | 29,053 (89.8%) | 3,312 | 141 | 161 | 2,778 | 232 |
| 0 | 32,365 | 32,364 (100.0%) | 1 | 0 | 0 | 0 | 1 |
| Total | 97,095 | 90,457 | 6,638 | 286 | 329 | 5,557 | 466 |

Perm 2's mismatch set is perm 4's set minus the 13 perm-4-only inputs
(all `publish` — gated below level 4; at perm 2 both sides reject
`publish` identically with cursor 0); perm 0's single mismatch (the
empty-input root suggestion list) is also in perm 4's set. So **3,325
unique mismatching inputs**. 14 oracle-crash rows at perm 4 (10
`random` sequence cases — NPE in `RandomSequences`, the oracle has no
server state — + 4 `schedule clear` cases — NPE in `WorldData`) were
skipped by the driver's `compare()` (`if 'crash' in v: continue`); the
schedule skips hide a real divergence (G20, §2.5).

### 2.2 What the mismatches are (perm 4; perm 2 identical except publish)

| Command | Total | Composition | Maps to |
|---|---|---|---|
| execute | 2,923 | 2,723 error-text-only (2,703 SlotSource message + 20 codec error-text shape), 196 suggestions (all rust-only `slots`), 4 cursor-differs (`@@bad@@` slot tokens) | G12, G07a, G18 |
| gamerule | 248 | 124 ok-differs, 123 cursor-differs, 1 suggestions | G01 |
| item | 67 | 52 error-text-only (34 SlotSource + 18 codec), 3 ok-differs, 10 suggestions, 2 cursor-differs | G12, G07a/b, G18 |
| place | 22 | 11 ok-differs, 7 cursor-differs, 3 error-text-only, 1 suggestions | G03 |
| advancement | 22 | all cursor-differs: 8 `type=` accepted for @a/@p/@r, 14 duplicate-option wrong cursor | G09 |
| data | 17 | all suggestions, rust-only `compute` | G02 |
| publish | 13 | perm 4 only: 7 cursor-differs, 4 ok-differs, 1 error-text-only, 1 suggestions | G05 |
| bossbar | 4 | 2 ok-differs, 2 cursor-differs | G06a |
| recipe | 3 | all suggestions | G11 artifact (§2.4) |
| scoreboard | 3 | all suggestions | G21 |
| particle | 2 | 1 ok-differs, 1 cursor-differs | G08 |
| (empty input) | 1 | suggestions: root list | G02 |

Detail on the big commands. `execute`'s 2,723 error-text-only
mismatches are 2,703 × `Unknown slot '0'` vs `Can't find element
'minecraft:0' in registry 'minecraft:slot_source'` (G12) plus 20 codec
error-text-shape cases (G07a's footprint in error text: vanilla `No
key condition` / `No key function` vs Rust `No key type`, and
`Failed to parse structure` either-branch enumeration differences).
Its 196 suggestions are all the extra `slots` literal (G18). Its 4
cursor-differs are the `@@bad@@` slot tokens (vanilla `Unknown slot
'@@bad@@'` at cursor 66/69 vs Rust `Expected a valid unquoted string`
at 59/62). `gamerule`'s 248 are the name-form defect in full: every
camelCase vanilla rule fails in Rust (124 ok-differs: vanilla ok,
Rust errors; 123 cursor-differs: both error, different cursor) plus
the `gamerule ` root suggestion (vanilla 59 names, Rust 116 — 58
engine rules × 2 forms, plain + `minecraft:` prefix). `item`'s 3
ok-differs are the 2 modifier `function`-key cases (G07a) plus the
multi-entity `from entity` source (G07b: `item replace entity @s
container.0 from entity @a weapon.mainhand` parses in Rust, fails in
vanilla); its 10 suggestions are 1 × `fill`/`override` (G18) plus 9 ×
SlotSource `*` wildcards (`armor.*`, `container.*`, … — G12:
vanilla's `item_slot` suggests only concrete slots). `place`'s 11
ok-differs are all configured-feature ids vanilla accepts and Rust
rejects (wrong registry, G03). `bossbar`'s 4 are the G06a NBT-selector
cases (`{"nbt":"a","entity":"@x"}` decodes in vanilla, fails in Rust).

`publish` mismatches appear only at perm 4 (the command is gated
below it). Perm 0's single mismatch is the empty-input root suggestion
list: Rust's list is vanilla's plus exactly `compute`, `posteffect`,
`swing`, `unpublish` (G02). Note the corpus carries NO parse cases
for the four extra commands themselves (they were dropped from EXTRA
per the brief's "drop the 26.x-only commands" — §6): their only run
footprint is the root suggestion list at all three perms, which
confirms all four. The tree diff (§3.1, B9-G02) is the complete check
for them.

### 2.3 Node diff (B9-G16 confirmed)

Separate node-path comparison over the run's `vanilla.jsonl` vs
`rust.jsonl` (the driver's `compare()` strips nodes): **perm 4: 157
inputs, perm 2: 153, perm 0: 0** (of 32,365 each). At perm 4 the 157
split into 139 shape diffs where Rust emits no nodes at all (119
gamerule — G01: Rust fails the whole camelCase input; 11 place
feature — G03; 4 publish gamemode — G05; 2 item modifier — G07a; 2
bossbar NBT-selector — G06a; 1 particle — G08), 5 reverse-shape diffs
where vanilla emits no nodes (the snake_case gamerule EXTRA cases —
`gamerule keep_inventory`, `gamerule spawn_mobs`, … — which parse in
Rust and fail in vanilla), and 13 true label-rename inputs (12 item +
1 execute) carrying 29 label-pair occurrences: `slot` → `slots` (11),
`targets` → `target` (6), `pos` → `targetPos` (5), `sourceSlot` →
`sourceSlots` (4), `source` → `sourcePos` (2), `entities` → `source`
(1) — six of the nine G16 pairs, the same six pairs and the same
counts as 26.1.2. Perm 2 has 4 fewer (the publish gamemode diffs,
gated below level 4). The node diff is the only check that sees the
names the fixture actually emits (as opposed to the literals in the
tree builders) — G16 is confirmed end-to-end, not just statically.

### 2.4 Recipe suggestions are the predicted oracle artifact (B9-G11)

All 3 recipe mismatches at each of perms 4 and 2 are the artifact
predicted in §3.2 (B9-G11): the oracle suggests only `["*"]` (its
`Src.suggestRegistryElements` override returns empty for the RECIPE
registry, which is not in the RegistryAccess), while Rust suggests
`*` plus all 1,449 learnable recipes. Real vanilla suggests all 1,461
recipes (RECIPE special case, `CommandSourceStack.javap` offsets
0–36). Do not read these 3 mismatches as port defects; the real
divergence (learnable subset vs all recipes) is B9-G17, verified
statically by the jar census.

### 2.5 Run observations confirming the run-minted items (G20/G21; G19 withdrawn)

- **G20 (`/schedule clear`): NO run coverage — the oracle crashes.**
  The four distinguishing inputs (`schedule clear hello world`,
  `schedule clear hello world `, `schedule clear foo:bar`, `schedule
  clear `) all crash the oracle with an NPE in
  `ScheduleCommand.lambda$static$1` (`WorldData.overworldData()` on a
  null `getWorldData()`), so the driver's `compare()` skips them. The
  raw rows show the divergence plainly: vanilla parses `schedule
  clear hello world` OK (nodes `[schedule, clear, function]` — the
  node is named `function`), Rust errors at cursor 21 (`Incorrect
  argument for command` — the trailing `world` after the Id). G20
  stands on its static proof.
- **G21 (scoreboard criteria): confirmed, with TWO extra criteria.**
  The 3 scoreboard suggestion mismatches (one per corpus name form)
  show Rust suggesting 9,033 criteria against vanilla's 9,031; the
  rust-only entries are `minecraft.custom:minecraft.nautilus_one_cm`
  AND `minecraft.custom:minecraft.sleep_in_straw_bed` (vanilla-only:
  none). The static check had verified only the straw bed; the run
  caught the nautilus stat too. G21's evidence now cites both
  (vanilla 1.21.10's `Stats` class has neither).
- **G19: withdrawn** — no `/swing` cases in the corpus (dropped per
  the brief) and no `/swing` in vanilla 1.21.10; the command is a G02
  extra, confirmed via the root suggestion list (§2.2).
- **`data modify … compute`: YES, the helper is shared.** All 17
  `data` mismatches (per perm) are suggestions on `data modify …
  insert` inputs where Rust's source list is vanilla's plus
  `compute` (`['from', 'string', 'value']` vs `['compute', 'from',
  'string', 'value']`) — the `context_sources` helper
  (`tree_data.rs:32-48`) is shared with `/data`, same as 26.1.1/26.1.2
  (§7.8).
- **`fetchprofile` / `publish` EXTRA cases: no mismatches.** Both
  commands exist in vanilla 1.21.10 (brief's drop-list parenthetical
  was wrong — §0); their EXTRA cases parse identically on both sides
  except for `publish`'s missing gamemode (G05, 13 mismatches).

### 2.6 Error-text count (B9-O04 confirmation) and tally check

Error-text-only mismatches: **2,779 (perm 4) / 2,778 (perm 2)**.
Composition at perm 4: 2,737 SlotSource message (G12: 2,703 execute +
34 item), 42 codec/registry error-text shape (20 execute + 18 item +
3 place + 1 publish — G07a's footprint plus `Failed to parse
structure` either-branch enumeration differences). **0 raw-key
emissions in the whole run** (both perms' `rust.jsonl` scanned: no
error starting with `argument.` / `commands.` / `entity.`) — no
unresolvable-key artifact: the 1.21.7-style 86,241-mismatch blowup did
not recur, confirming O04's mitigation worked (fixture lang file
present and jar-identical after the extraction).

**Tally check: the run contradicts NO static verdict.** Every gap with
run coverage is confirmed empirically (G01, G02, G03, G05, G06a, G07,
G08, G09, G12, G16, G18, G21); every closure stands (G11's artifact
prediction confirmed exactly; G04/G10 have no run coverage and stand
on their static proofs); G13/G14/G15/G17/G20 have no run coverage and
stand on their static proofs. The 24-check tally (5 pass / 19 gap) is
UNCHANGED; G20/G21 are re-check items carried from 26.1.2, not new
checks, and B9-O05 is a new oracle item, not a counted check (see §6
for the tally scheme).

## 3. Defects

### 3.1 Open defects

### B9-G01 — major — `/gamerule` exposes the 26.3 snake_case table; vanilla 1.21.10 uses camelCase names

REPRODUCES — flipped from 26.1.2's pass, because 1.21.10's vanilla
still uses camelCase gamerule names. Vanilla 1.21.10's integrated tree
has 59 gamerule literals, all camelCase (`doFireTick`,
`keepInventory`, `commandModificationBlockLimit`, `maxCommandForkCount`,
`locatorBar`, …); the jar's `GameRules` clinit registers 60 keys, the
60th (`minecartMaxSpeed`) feature-gated out of the tree. The Rust
`/gamerule` tree is built from `available_rules()` snake_case engine
names plus `minecraft:`-prefixed forms (`vanilla.rs:315-326`), so
every one of vanilla's 59 names is unusable in the port and every one
of the port's 58 names is unknown to vanilla. The count differs by
one for a second reason: vanilla's fire pair (`doFireTick` +
`allowFireTicksAwayFromPlayer`) collapses into the single engine rule
`fire_spread_radius_around_player` (`gamerules.rs:66-72`, default 128),
so the engine table has 58 entries against vanilla's 59 keys (plus the
gated `MAX_MINECART_SPEED`, `gamerules.rs:130`, offered only when
`minecart_improvements` is on — matching vanilla's gating). The port
HAS the full camelCase mapping (`vanilla_key` / `from_vanilla_key`,
`gamerules.rs:151-230`, including all 26 renames and the
`vanilla_inverted` trio `raids` / `elytra_movement_check` /
`player_movement_check`) and fire-pair handling (`fire_radius` /
`fire_keys`, `gamerules.rs:236-260`) — but only for level.dat
persistence, not for the command tree. Sampled defaults match vanilla
exactly (`randomTickSpeed` 3, `maxEntityCramming` 24,
`maxCommandChainLength` 65536, `maxCommandForkCount` 65536,
`tntExplosionDropDecay` false, `keepInventory` false,
`snowAccumulationHeight` 1 — `GameRules.javap` clinit vs
`gamerules.rs`); the defect is the name form and the fire-pair
collapse, not the values. The corpus carries 255 gamerule cases (§2).

### B9-G02 — major — Four 26.x commands in the release tree; vanilla 1.21.10 has none of them

REPRODUCES — four extras here, not 26.1.2's three: `swing` joins the
list because vanilla 1.21.10 has no `/swing` at all. The full
top-level diff (§3.2, G02 bulk) shows the Rust release tree (78
literals) against vanilla 1.21.10's integrated tree (74 commands, from
the oracle's own INTEGRATED tree export): exact match except FOUR
extras — `compute`, `posteffect`, `swing`, `unpublish`:

- `posteffect`: no `PostEffectCommand` class in the 1.21.10
  server-game jar, no `posteffect` in the datagen `commands.json`
  dump or the oracle's tree export, no `PostEffectCommand.register`
  in `Commands.<init>`, and no `minecraft:post_effects` registry in
  1.21.10's `registries.json` — so the tree's
  `ArgType::Id("post_effects")` arguments (`tree_extra.rs:40,50`)
  reference a registry that does not exist. Registered
  unconditionally in `tree_extra::register` (`tree_extra.rs:36-53`).
- `unpublish`: no `UnpublishCommand` class in the 1.21.10 jar and no
  `unpublish` in the dump (`unpublish` arrived in 26.2). Registered
  unconditionally (`tree_extra.rs:83-85`, under the `// PublishCommand
  / UnpublishCommand` comment).
- `compute`: no `ComputeCommand` class in the 1.21.10 jar and no
  `compute` in the dump. Registered unconditionally
  (`tree_data.rs:51-69`, `compute (default | block <pos> | entity
  <target>) (float <provider> [<scale>] | integer <provider>)`).
  The `/data modify … compute` source is also present (the shared
  `context_sources` helper, `tree_data.rs:32-48`) — counted here per
  the 26.1.2 precedent, not as a new item.
- `swing`: no `SwingCommand` class in the 1.21.10 server-game jar
  (`Swing.javap` = "class not found"), no `swing` in the dump or the
  oracle's tree export, and no `swing_animation` argument type in
  vanilla's 57-entry table. Registered unconditionally with
  animation + duration arguments (`tree_world.rs:176-192`). (This
  subsumes the 26.1.2 secondary item B9-G19 — see §3.2.)

All four are gamemaster-only, but they are real divergences in the
release tree: on each, Rust parses and vanilla reports unknown
command. The corpus carries NO parse cases for the four extra commands
themselves (they were dropped from EXTRA per the brief's "drop the
26.x-only commands" — §6); their only run footprint is the
empty-input root suggestion list at all three permission levels
(vanilla 74 names, Rust 78 — the four extras, §2.2). The tree diff in
this section is the complete check for them.

### B9-G03 — minor — `/place feature` uses the wrong registry and accepts inline values

REPRODUCES — same as 26.1.2. Vanilla 1.21.10's `/place feature` takes
`ResourceKeyArgument.key(CONFIGURED_FEATURE)` (`Place.javap`:
`CONFIGURED_FEATURE` + `ResourceKeyArgument`; the oracle's tree
export: node `minecraft:resource_key`, registry
`minecraft:worldgen/configured_feature`). Rust uses
`ArgType::ResourceOrId("minecraft:worldgen/feature")`
(`tree_world.rs:60-64`) — the wrong registry, and a form that accepts
inline values vanilla rejects. The 26.1 tree's fix
(`ResKey("minecraft:worldgen/configured_feature")`) was never carried
to this tree.

### B9-G05 — minor — `/publish` is missing its gamemode argument

REPRODUCES — same as 26.1.2. Vanilla 1.21.10's `/publish` is
`[<allowCommands> [<gamemode> [<port>]]]` (`Publish.javap`:
`BoolArgumentType` + `GameModeArgument` + `IntegerArgumentType`; the
oracle's tree export: `allowCommands` → `gamemode` → `port`, integer
0–65535, required level 4). Rust is `[<allowCommands> [<port>]]`
(`tree_extra.rs:76-82`) — the gamemode node is missing entirely, so
`/publish true survival` fails to parse in Rust and `/publish true
25565` binds the port where vanilla would read a gamemode first.

### B9-G06 — minor — Item-component NBT with a bad selector decodes in vanilla, fails in Rust (partial)

REPRODUCES on part (a); part (b) is CLOSED. (a) Vanilla 1.21.10's
`EntityDataSource.compileSelector` catches `CommandSyntaxException`
and returns null (`EntityDataSource.javap`: exception table 0–21 →
22, `aconst_null` at offset 23) — the 1.21.7 form — so
`{"nbt":"a","entity":"@x"}` decodes in vanilla. Rust's nbt arm
requires `selector_string` to parse (`component.rs:278-299`), so the
same component fails. (b) Vanilla 1.21.10 HAS `ObjectContents` (both
jars, with `AtlasSprite` + `PlayerSprite`), so the Rust `object` arm
is correct and that half of the 1.21.7 item does not reproduce. One
check, gap on part (a).

### B9-G07 — minor — Item modifiers: inline JSON fails (wrong type key) AND multi-entity source

REPRODUCES in full — same as 26.1.2. (a) `codec.rs dispatch()` reads
`"type"` unconditionally, and the `minecraft:item_modifier` arm calls
it directly — inline `{"function":"minecraft:set_count","count":1}`
fails in Rust. Vanilla 1.21.10's loot data uses the `"function"` key
(`data/minecraft/loot_table/entities/zombie.json`:
`"function": "minecraft:set_count"`). The 26.2 port's fix (key-parameter
dispatch) was never carried here. (b) `/item … from entity <source>`
uses `ENTITIES` (multi, `tree_misc.rs:198`); vanilla uses a
single-entity source (oracle tree export: `from entity` node is
single-entity). Two parts, one item, both gap.

### B9-G08 — minor — Particle block_state compound with Name is rejected

REPRODUCES — same as 26.1.2. Vanilla 1.21.10's `StateHolder.codec`
dispatches on `"Name"` (`StateHolder.javap:353-362`: `ldc "Name"` at
offset 1, then `Codec.dispatch`), so the `{Name, Properties}` map
form is a valid block state inside particle options. Rust's
`particle_options` returns an error for ANY non-string block_state
(`args.rs:1011-1024`, with the `withAlternative(BlockState.CODEC,
…)` comment noting the intended shape but implementing the rejection).
The 26.2 port's `Name`-accepting form was never carried here.

### B9-G09 — minor — Selector option errors: wrong cursor on duplicates; type= accepted for @a/@p/@r

REPRODUCES (both parts). Vanilla 1.21.10 still has the OLD selector
parser — the option-state refactor (`InvertableSetOptionState`,
`SetOnceOptionState`) that closed this defect for 26.2 had NOT
happened yet in 1.21.10 (neither class exists in the 1.21.10
server-game jar). The Rust parser (`selector.rs`) implements the 26.2
option-state shape, so it diverges from vanilla 1.21.10 in both parts:

(a) Duplicate options: vanilla 1.21.10's `EntitySelectorOptions.get()`
(`net.minecraft.commands.arguments.selector.options.EntitySelectorOptions`,
note the `options` subpackage) runs the `canUse` gate BEFORE the
value is read — `parseOptions` calls `get()` at offset 65, right
after `readString`, before the `=` and value (`Selector.javap:651-720`)
— but does NOT roll back: only the unknown-option branch does
`setCursor(start)` (offsets 47–52); the inapplicable branch throws
`ERROR_INAPPLICABLE_OPTION` at the cursor as it stands, i.e. at the
END of the option name (before `=`). Rust rolls back to the
option-name START (`selector.rs:331-341`, `rollback(reader, start,
…)`). For `@e[type=pig,type=cow]` the cursors differ by the
option-name length (4 for `type`). Three-version shape: 1.21.7 vanilla
at the end of the VALUE, 1.21.10/26.1/26.1.1/26.1.2 vanilla at the end
of the NAME, 26.2 vanilla at the START — Rust reports at the start in
all cases, matching only 26.2.

(b) `@a[type=player]`: vanilla 1.21.10's `type` option `canUse` is
`!parser.isTypeLimited()` (`lambda$bootStrap$47`,
`SelectorOptions2.javap:862-872`), and `@a`/`@p`/`@r` all call
`limitToType(EntityType.PLAYER)` during `parseSelector` (offsets 165,
196, 225), which sets the `type` field — so vanilla 1.21.10 REJECTS
`@a[type=player]` with `ERROR_INAPPLICABLE_OPTION`. Rust pushes
`minecraft:player` into `s.types` for `@a`/`@p`/`@r`
(`selector.rs:218-242`) without marking `type_option`, so
`can_use("type")` stays true and Rust ACCEPTS it.

### B9-G12 — minor — SlotSource is back in the tree; vanilla 1.21.10 uses item_slot/item_slots

REPRODUCES — same as 26.1.2. Rust uses `ArgType::SlotSource`
everywhere in `/item` (`tree_misc.rs:171,188,202`) and `execute
if|unless items|slots` (`tree_execute.rs:240`); vanilla 1.21.10 uses
`minecraft:item_slot` / `minecraft:item_slots` (oracle tree export:
`/item` slot node `minecraft:item_slot`, `/execute if items` slot
node `minecraft:item_slots`). The `minecraft:slot_source` registry
does not exist in 1.21.10. The slot-name TABLE itself matches vanilla
exactly (Rust `slot_ranges()`, `args.rs:205-239`, vs vanilla
`SlotRanges` clinit, `SlotRanges.javap:158-304`: contents,
container.×54, hotbar.×9, inventory.×27, enderchest.×27,
villager.×8, horse.×15, weapon/weapon.mainhand/weapon.offhand/
weapon.*, armor.head/chest/legs/feet/body, armor.*, saddle,
horse.chest, player.cursor, player.crafting.×4) — the defect is the
argument type, not the names. The 26.1 tree's fix (`Slot`/`Slots`)
was never carried here.

### B9-G13 — minor — Argument-type table is 26.3's 62, not 1.21.10's 57; index 16 is team_color

REPRODUCES — same as 26.1.2, in the stronger 26.1.2 form (not
1.21.7's). Rust ships 26.3's 62-entry table (`commands.rs:25-87`, doc
comment says "registry of 26.3 in id order"), `minecraft:team_color`
at index 16, plus five 26.3-only types (`context_float_provider`,
`context_int_provider`, `slot_source`, `feature`, `swing_animation`)
shifting later ids (`dialog` 58 vs 55, `uuid` 61 vs 56). Vanilla
1.21.10's `ArgumentTypeInfos` has 57 types with `color` at index 16
(order: 6 brigadier, entity, game_profile, block_pos, column_pos,
vec3, vec2, block_state, block_predicate, item_stack, item_predicate,
color, hex_color, … dialog, uuid). The 1.21.10 adapter has
`command_argument_type: Remap::Same` (`adapter/v1_21_10_ids.rs:33`) —
no remap mitigates the shift, so the tree's network form mislabels
every argument type from index 16 on.

### B9-G14 — minor — `/perf` missing from the dedicated tree

REPRODUCES — same as 26.1.2. Vanilla 1.21.10 has 15 dedicated
commands (`ban`, `ban-ip`, `banlist`, `deop`, `op`, `pardon`,
`pardon-ip`, `perf`, `save-all`, `save-off`, `save-on`,
`setidletimeout`, `stop`, `transfer`, `whitelist` — the 15 in datagen
`commands.json` but not in the oracle's integrated tree);
`PerfCommand.register` sits at `Commands.<init>` offset 695, inside
the `includeDedicated` guard (offsets 636–747). Rust's dedicated tree
(`register_dedicated`) has 14 commands and lacks `perf`. (The
dedicated list here is the CORRECTED one: `kick` is integrated —
`KickCommand.register` at offset 216, outside the guard — and
`transfer` is dedicated, at offset 737 inside it; the 26.1.1 report
had them backwards and 26.1.2 corrected it.)

### B9-G15 — minor — Dev tree is missing six vanilla 1.21.10 dev commands

REPRODUCES — same as 26.1.2. Vanilla 1.21.10's dev block
(`Commands.<init>` offsets 565–635, under `DEBUG_DEV_COMMANDS` /
`IS_RUNNING_IN_IDE`) registers `raid`, `debugpath`,
`debugmobspawning`, `warden_spawn_tracker`, `spawnarmortrims`,
`serverpack`, plus `debugconfig` when dedicated (offset 632); `chase`
is under its own flag (`DEBUG_CHASE_COMMAND`, offset 552). Rust's dev
tree has only `raid` + `warden_spawn_tracker` (`vanilla.rs` `if dev`
blocks) — missing `debugpath`, `debugmobspawning`, `spawnarmortrims`,
`serverpack` (+ `debugconfig` dedicated, + `chase`). `jfr` is
registered unconditionally in Rust while vanilla gates it on
`JvmProfiler.INSTANCE.isAvailable()` (offsets 534–549) — the same
non-defect nit as 26.1.2, not counted.

### B9-G16 — minor — Argument node names differ from vanilla

REPRODUCES — nine label pairs, same shape as 26.1.2 (counted by
path). Proven statically from the tree-builder literals against
vanilla's oracle tree export, and confirmed empirically by the run's
node diff (§2):

| Path | Vanilla 1.21.10 (oracle `tree.json`) | Rust 1.21.10 |
|---|---|---|
| `/item replace block <…>` | `pos` | `targetPos` (`tree_misc.rs:185`) |
| `/item replace entity <…>` | `targets` | `target` (`tree_misc.rs:183`) |
| `/item replace … slot` | `slot` | `slots` (`tree_misc.rs:187`) |
| `/item replace … from block <…>` | `source` | `sourcePos` (`tree_misc.rs:199`) |
| `/item replace … from … sourceSlot` | `sourceSlot` | `sourceSlots` (`tree_misc.rs:201`) |
| `/item modify block <…>` | `pos` | `targetPos` (`tree_misc.rs:168`) |
| `/item modify entity <…>` | `targets` | `target` (`tree_misc.rs:166`) |
| `/execute if items entity <…>` | `entities` | `source` (`tree_execute.rs:226`) |
| `/execute if items block <…>` | `pos` | `sourcePos` (`tree_execute.rs:234`) |

(`from entity` agrees: both sides call it `source`
(`tree_misc.rs:197`). `/item modify` `slot` agrees on the name — only
the parser differs, see B9-G12. `/loot replace` names match vanilla:
`entities` → `slot`.) Node names surface in parse output (`nodes` in
the fixture's jsonl) and in client-side command feedback, so a rename
is a real (if cosmetic) divergence — minor. Run node-diff confirmation
(§2.3): six of the nine pairs confirmed empirically (slot→slots 11,
targets→target 6, pos→targetPos 5, sourceSlot→sourceSlots 4,
source→sourcePos 2, entities→source 1 occurrences at perm 4 — the
same six pairs and the same counts as 26.1.2); the `/item modify` and
`/execute if items block` pairs have no corpus case and stand on the
static proof.

### B9-G17 — minor — Recipe suggestions omit special recipes (Rust suggests the learnable subset; real vanilla suggests all recipes)

REPRODUCES — count 12 for 1.21.10 (26.1.2's was 24). Vanilla 1.21.10's
client jar carries 1,461 recipe files; the oracle's
`command-registries.json` lists 1,449 `learnable_recipes`; the 12
omitted are `armor_dye`, `banner_duplicate`, `book_cloning`,
`decorated_pot`, `firework_rocket`, `firework_star`,
`firework_star_fade`, `map_cloning`, `map_extending`, `repair_item`,
`shield_decoration`, `tipped_arrow` (1.21.10 has ONE `banner_duplicate`
recipe, not 26.x's 16 colour variants — hence 12, not 24). Real vanilla
suggests all recipes via the RECIPE special case in
`CommandSourceStack.suggestRegistryElements` (offsets 0–36); the Rust
side suggests the learnable subset. The run cannot test this
faithfully (§2.4): the oracle's `Src` override bypasses the special
case, so the run's recipe mismatches are artifacts and this item rests
on the static census.

### B9-G18 — minor — 26.3-era subcommands in the tree (`/item fill`, `/item override`, `/execute if|unless slots`)

REPRODUCES — same as 26.1.2. Rust has `/item fill` and `/item
override` (`tree_misc.rs:179`, `for mode in ["replace", "fill",
"override"]`) and `/execute if|unless slots` (`tree_execute.rs:224`,
`for kind in ["items", "slots"]`); vanilla 1.21.10 has neither —
`/item` is only `replace` + `modify`, and `/execute if/unless` has
`items` only (oracle tree export). These are invisible to the corpus
(vanilla's tree has no such nodes to walk) and were caught by the
static tree diff.

### B9-G20 — minor — `/schedule clear` takes a single id where vanilla takes a greedy string

REPRODUCES — same as 26.1.2. Rust `/schedule clear` takes
`ArgType::Id("schedules")` (`tree_misc.rs:49`); vanilla 1.21.10 takes
a greedy `brigadier:string` (oracle tree export). A schedule id with
spaces, or a prefix, parses in vanilla and fails in Rust.

### B9-G21 — minor — Scoreboard criterion suggestions include two criteria vanilla 1.21.10 lacks

REPRODUCES — same as 26.1.2, with a second extra criterion caught by
the run. Rust's custom-stats list (`scoreboard.rs:251,289-290`)
includes `nautilus_one_cm` and `sleep_in_straw_bed`; vanilla 1.21.10's
`Stats` class has `SLEEP_IN_BED` and no `SLEEP_IN_STRAW_BED` and no
`NAUTILUS` stat at all (`Stats.javap:139,491`). The run confirms both:
the 3 scoreboard suggestion mismatches show Rust suggesting 9,033
criteria against vanilla's 9,031, rust-only exactly
`minecraft.custom:minecraft.nautilus_one_cm` and
`minecraft.custom:minecraft.sleep_in_straw_bed` (§2.5).

### 3.2 Closed — the 1.21.7 defects, re-checked against 1.21.10

### B9-G04 — CLOSED (inverted by a vanilla change)

The 1.21.7 defect was that Rust used `Rotation` where vanilla 1.21.7
used `Angle` for `/spawnpoint` / `/setworldspawn`'s trailing argument.
Vanilla CHANGED since 1.21.7: 1.21.10 uses `RotationArgument`
(`SetSpawn.javap` / `SetWorldSpawn.javap`: 3 refs each, no
`AngleArgument`; the oracle's tree export names the node `rotation`
with parser `minecraft:rotation`), and Rust uses `ArgType::Rotation`
(`vanilla.rs:124,355,364`) — match. (Note: `RotationArgument` lives
in `net.minecraft.commands.arguments.coordinates` in 1.21.10, not in
`arguments`.)

### B9-G10 — FIXED in the 1.21.10 tree

The 1.21.7 defect was that Rust's `/worldborder` set/add time used a
`Time` argument where vanilla used a plain integer. The 1.21.10 tree
fixed it: `tree_world.rs:335-336` uses `integer(0)` with the comment
"1.21.10 takes a plain integer of ticks here
(`IntegerArgumentType.integer(0)`); the Time argument is 1.21.11's" —
matching vanilla 1.21.10 (`WorldBorder.javap`: `IntegerArgumentType`,
12 refs, no `TimeArgument`; oracle tree export: `brigadier:integer`
min 0).

### B9-G11 — CLOSED against real vanilla (run mismatches are the oracle artifact)

The 1.21.7 item claimed vanilla suggests no recipes for `/recipe` and
Rust suggests the learnable set. Against REAL vanilla 1.21.10 that
premise fails: `CommandSourceStack.suggestRegistryElements` HAS the
RECIPE special case (`CommandSourceStack.javap:1035-1071`, offsets
0–36: `Registries.RECIPE` → `RecipeManager.getRecipes()` →
`suggestResource`), so real vanilla suggests all 1,461 recipes. The
run's recipe mismatches come from the oracle's `Src` override
(`VanillaCommandParseOracle.java:93-99`), which looks the registry up
in the RegistryAccess and returns `Suggestions.empty()` when absent —
bypassing the special case, exactly as in 26.1.2. The real divergence
(the learnable subset vs all recipes) is B9-G17, verified statically.

### B9-G19 — WITHDRAWN (folds into B9-G02)

The 26.1.2 secondary item's premise was that vanilla has `/swing` and
Rust adds 26.3-era animation + duration arguments. That premise fails
for 1.21.10: vanilla 1.21.10 has NO `/swing` at all — no
`SwingCommand` class in the server-game jar, no `swing` in the datagen
dump or the oracle's tree export, no `swing_animation` argument type.
The whole command is a G02 extra; there is no separate G19 defect.
(26.1.2's vanilla HAS `/swing`, so G19 stands there — this withdrawal
is 1.21.10-specific.)

### G02 bulk (top-level diff)

Vanilla 1.21.10 integrated tree (oracle export, 74):
`advancement`, `attribute`, `bossbar`, `clear`, `clone`, `damage`,
`data`, `datapack`, `debug`, `defaultgamemode`, `dialog`,
`difficulty`, `effect`, `enchant`, `execute`, `experience`,
`fetchprofile`, `fill`, `fillbiome`, `forceload`, `function`,
`gamemode`, `gamerule`, `give`, `help`, `item`, `jfr`, `kick`,
`kill`, `list`, `locate`, `loot`, `me`, `msg`, `particle`, `place`,
`playsound`, `publish`, `random`, `recipe`, `reload`, `return`,
`ride`, `rotate`, `say`, `schedule`, `scoreboard`, `seed`,
`setblock`, `setworldspawn`, `spawnpoint`, `spectate`,
`spreadplayers`, `stopsound`, `summon`, `tag`, `team`, `teammsg`,
`teleport`, `tell`, `tellraw`, `test`, `tick`, `time`, `title`,
`tm`, `tp`, `trigger`, `version`, `w`, `waypoint`, `weather`,
`worldborder`, `xp` — all present in the Rust release tree. Rust release tree: 78 literals (grep of
`d.literal(root,…)` + loop-built `tell`/`w` aliases). **74 of 74
present, zero missing; four extras: `compute`, `posteffect`, `swing`,
`unpublish`.** Absent from vanilla's integrated tree (all verified
absent from the oracle export AND the jar): `swing`, `stopwatch`,
`compute`, `posteffect`, `unpublish`, `raid`, `perf`, `chase`,
`debugpath`, `debugmobspawning`, `warden_spawn_tracker`,
`spawnarmortrims`, `serverpack`, `debugconfig` (dev/dedicated-gated,
see G14/G15). Present in vanilla and correctly present in Rust
(notable): `fetchprofile` (name + id branches only — the entity branch
was correctly removed, `tree_extra.rs` comment), `publish`, `waypoint`,
`dialog`, `debug`, `test`, `jfr`, `kick`. `/datapack create` exists
in vanilla 1.21.10 — Rust's is correct, not an extra. `stopwatch` is
gone from the Rust tree entirely (no occurrences in
`horizon-command/src`, command or execute conditional). `/time` clock
subcommands removed (`vanilla.rs` comment: "no clocks in 1.21.10 —
the clock/timeline nodes were 26.x leakage, removed"). The oracle's
`registries.json` confirms: `clocks` 0, `timelines` 0,
`clock_time_markers` {}.

### Passes (verified, not re-diagnosed)

- **`/time set` pass HOLDS.** Both sides have `day|noon|night|midnight`
  literals + a `Time` argument; `add` and `query daytime|gametime|day`
  match; no clocks either side. (One behaviour check, pass.)
- **`/datapack create`** is present on both sides (see G02 bulk).
- **`fetchprofile`** name + id branches match vanilla (see G02 bulk).

### 3.3 Harness — B9-H01 FIXED for 1.21.10

FIXED — the probe compiles. The 26.1 defect was that
`game_loop_probe.rs` constructed a `FrameContext` with fields
(`focused`, `exclusive_fullscreen`) the tree's `window.rs` did not
declare. In the 1.21.10 tree the probe (`crates/horizon-client/examples/probe/game_loop_probe.rs:152-164`)
constructs `FrameContext` with exactly the seven fields the struct
declares (`crates/horizon-client-input/src/window.rs:56-64`:
`inactivity`, `iconified`, `now_ms`, `latest_input_ms`, `has_level`,
`has_screen`, `has_overlay`) — no `focused`/`exclusive_fullscreen`
anywhere. Batch 0's `cargo check --locked -j 1 -p
horizon-client-lib --example client_probe` was still running when the
VM rebooted at ~23:50 UTC Nov 24 and never completed; the verification
here is this batch's own driver build (§2), which compiles AND links
the probe (`cargo build --profile fast -p horizon-client-lib --example
client_probe`). One non-behaviour check, pass.

## 4. Oracle / fixture defects

Oracle results below are from Batch 0's measured table
(`versions/1.21.10/.local/measure-oracles/results.json`, Nov 24 22:25;
391 files, java_major 21, fixpoint big-compile per classpath). The
`docs/sweep/1.21.10/batch-0-oracles.json` doc had not landed at write
time; the error inventories below were extracted from the measured
logs (`all-server.log`, `all-client.log`) and re-verified against the
1.21.10 jars where noted.

### B9-O01 — minor — `VanillaScoreboardOracle` fails javac (6 errors) — blocks `scoreboard`

REPRODUCES in the 1.21.7 form (26.1.2's was 3 errors). All three
classpaths fail with the same 6 errors: line 44 ×3 (`packObjectives()`,
`packDisplaySlots()`, `packPlayerTeams()` — methods that do not exist
on 1.21.10's scoreboard classes), line 242 (`TeamColor` variable —
no `TeamColor` class in 1.21.10), line 436 ×2 (incompatible conditional
+ `TeamColor`). The repair is the 1.21.7 form's: drop the three pack
calls and replace `TeamColor` with `ChatFormatting`. Blocks the
`scoreboard` driver.

### B9-O02 — minor — `VanillaTeamRulesOracle` fails javac (19 client errors) — blocks `team_rules`

REPRODUCES in the 1.21.7 form (26.1.2's was 6 client errors). Client
classpath: 19 errors — line 14 (`Hud` class, `net.minecraft.client.gui`
— 1.21.10's HUD class is `Gui`, not `Hud`), line 24 (`net.minecraft.world.entity.monster.zombie`
package does not exist — zombies are not in a `monster.zombie`
subpackage in 1.21.10), line 51 (`Zombie` class), lines 60/65/70/75/80/
85/90 (SEVEN failed `@Override`s, not 1.21.7's five), line 101
(`LocalPlayer` constructor arity), lines 188–189 (`Hud` ×3), lines
197–201 (Stub type errors: `doTeamsAllowDamage(Stub)`,
`isInvisibleTo(Viewer)`, Stub→Entity/LivingEntity conversions).
Server classpath fails with 53 errors (expected — client-only
imports); the client classpath is operative. Blocks the `team_rules`
driver.

### B9-O03 — minor — `LocatorBarClient` fails javac (18 errors) — blocks `exploration_map_command`

REPRODUCES in an 18-error form (1.21.7's was 20, 26.1.2's 13). All
three classpaths fail with the same 18 errors: lines 222/231
(`ContainerInput` class — no such class in 1.21.10's
`net.minecraft.world.inventory`), lines 393–395/407/409/413
(`ClientboundUpdateAdvancementsPacket` has no `removed()` / `added()` /
`progress()` record accessors in 1.21.10), line 562 (`Action.STAB` —
no such constant), line 668 (failed `@Override`), lines 683/906/911
(`EntityTypes` class — 1.21.10 still uses `EntityType`), lines 690/916
(`ClientboundRemoveEntitiesPacket.entityIds()` — no such accessor),
lines 955/957 (`ClientboundSetEntityMotionPacket.id()` /
`movement()` — no such accessors), line 977
(`ServerboundAcceptTeleportationPacket` constructor arity). Blocks the
`exploration_map_command` driver.

### B9-O05 — major — `VanillaCommandExecOracle` fails javac (45 errors) — blocks `command_exec`

NEW for 1.21.10 (it passed in 26.1.2). All three classpaths fail with
the same 45 errors: the oracle was never adapted for 1.21.10 and is
still in its 26.3 form. Clusters: (1) 26.3-era imports —
`com.mojang.authlib.services` (line 8),
`net.minecraft.world.level.gamerules` (line 62 — 1.21.10's `GameRules`
is in `net.minecraft.world.level`, no `gamerules` subpackage),
`net.minecraft.world.entity.npc.villager` (lines 100/686/1111/1246/
1249), `animal.cow` / `animal.happyghast` / `animal.pig` (231/233/299),
`projectile.arrow` (650), `vehicle.boat` (865); (2) `EntityTypes`
class missing (231/233/299/300/538 — 1.21.10 uses `EntityType`); (3)
server-API drift — `DifficultySettings` (113), `Util` (121/130/1275),
`datapackWorldRegistries()` (123), `PrimaryLevelData` constructor
(126), `WorldDataAndGenSettings` (128), `ServicesKeySet` / `GameRules`
(135/136), `tickServer` override (156), `getRandomSequences()` (361),
`getRandomSequence()` (392), `setWeatherParameters()` (400),
`hasClientLoaded()` (603), `asLong()` (731/745/1340),
`getWeatherData()` (754), `ServerboundInteractPacket` constructor
(867/876/1112), `getDataStorage()` (913/1009),
`MapItemSavedData.tickCarriedBy` (979), `ContainerInput` (989/1121),
`teleportToPortalDestination()` (1326). Major because it blocks the
linchpin `command_exec` driver entirely — on top of that driver's
`effect_view:always` fixture blocker (§5). This is a NEW item (not in
the 1.21.7/26.1.2 re-check sets) and is NOT counted in the as-tallied
24; counted, the totals become 25 as-tallied (5 pass, 20 gap).

### B9-O04 — minor — Prepared data lacks the language file (fixture-data gap, mitigated for the run)

REPRODUCES as a prepared-data gap — the 1.21.7 condition (26.1.2's was
closed). The prepared copy used for the run
(`versions/1.21.10/.local/horizon-client/1.21.10/`, from
candidate-test) had `command-registries.json`, `commands.json` and
`registries.json` but NO `assets/` directory at all — no lang file.
The fixture (`command_parse_fixture.rs`) loads lang only from the
resources root via `lang::Language::load`, so without the file all
error text would be raw keys. The run was mitigated by extracting
`assets/minecraft/lang/en_us.json` from the 1.21.10 client jar
(md5 `d166df536bf7f617599990f8437ba18c`, jar-identical, 7,583 keys,
contains `argument.entity.invalid`; `deprecated.json` also extracted)
— exactly what the port's own `horizon-prepare languages` step does
(`sounds.rs:100`, `add_assets`; the client jar carries `en_us`
alone). The prepare flow itself produces the file, so the fix is a
re-prepare, not code. The error-text count in §2.6 confirms the
mitigation worked.

### Closed oracle / fixture defects

None — O01, O02, O03 all reproduce (in 1.21.7 forms), O04 reproduces
as a prepared-data gap, and O05 is new.

### Passing in-scope oracles (auditable coverage)

- `VanillaCommandParseOracle` — PASS on all three classpaths (Batch 0
  table); independently confirmed by this batch: it compiled clean (3
  warnings only) and exported `registries.json` (926 KB) +
  `tree.json` (135,919 bytes) for the run.
- `VanillaCommandSuggestionsExporter` — PASS on all three classpaths
  (Batch 0 table).

## 5. Driver table (22)

| Driver | Oracle | Status |
|---|---|---|
| command_parse | VanillaCommandParseOracle (pass) | **MEASURED — see §2.** 32,365 commands × 3 perms; 3,325 / 3,312 / 1 mismatches (perm 4 / 2 / 0); node diff 157 / 153 / 0 inputs. |
| command_exec | VanillaCommandExecOracle (FAIL, 45 errors) | Oracle-blocked — B9-O05 (NEW). Plus the known fixture blocker (same as 1.21.7/26.1/26.1.1/26.1.2/26.2): `/execute if predicate effect_view:always` — the `effect_view:*` predicates come from `tools/mob_effect_predicate_corpus.py`, imported by `command_exec_corpus.py:64`. Recorded, not chased. |
| scoreboard | VanillaScoreboardOracle (FAIL) | Oracle-blocked — B9-O01 (6 errors; the 1.21.7 form) |
| team_rules | VanillaTeamRulesOracle (FAIL) | Oracle-blocked — B9-O02 (19 client errors; the 1.21.7 form) |
| exploration_map_command | LocatorBarClient (FAIL) | Oracle-blocked — B9-O03 (18 errors) |
| command_damage_range | none (via command_exec driver) | Not run — game-binary e2e |
| command_reinforcements | none | Not run — game-binary e2e |
| fork_function_feedback | none | Not run — game-binary e2e (`--reference-only` Java first) |
| function_admission | none (via command_exec driver) | Not run — game-binary e2e |
| function_tags | none (via command_exec driver) | Not run — game-binary e2e |
| guest_advancement_selector | none | Not run — game-binary e2e (two network players) |
| knot_command_view | none | Not run — game-binary e2e |
| loaded_command | none (via command_exec driver) | Not run — game-binary e2e |
| mob_command_damage_vibration | none | Not run — game-binary e2e |
| selector_bounds | none (via command_exec driver) | Not run — game-binary e2e |
| selector_criterion | none | Not run — game-binary e2e |
| selector_empty_tag | none | Not run — game-binary e2e |
| selector_global_order | none (via command_exec driver) | Not run — game-binary e2e |
| stand_command_hurt | none | Not run — game-binary e2e |
| terminal_execute_fork | none (via command_exec driver) | Not run — game-binary e2e |
| tick_function_time | none (builds the game itself) | Not run — game-binary e2e |
| vehicle_command_damage | none | Not run — game-binary e2e |

"Game-binary e2e" = the driver runs a real Java reference (fixture
server or the command_exec oracle) and the built Horizon game
(`--profile fast`), comparing records. Each needs a full client build
plus a Java run; none was attempted, and each is a 10–30+ min job on
this VM even unblocked. `command_exec` is the linchpin: 7 of the 17
no-oracle drivers import it or shell out to it, so its oracle failure
(B9-O05) + fixture blocker gate the family. All 22 driver scripts exist
in the 1.21.10 tree (`tools/test_*_parity.py`, verified by name). The
tree carries 648 `test_*_parity.py` drivers in total; the 22 above are
the command family.

## 6. Method notes

- **Run + static.** The one measurement available (`command_parse`,
  §2) was run end-to-end — the 1.21.10 probe compiles, so the 26.1
  run's blocker is gone. Every verdict below that would have rested
  on run data ALSO rests on a static proof stated in its section
  (javap + source + datagen + oracle data); the run confirms rather
  than carries them. Where the corpus cannot reach a defect
  (dedicated/dev trees, special-recipe suggestions, node names the
  driver's `compare()` strips), the static proof is the verdict and
  the report says so.
- **Vanilla 1.21.10's own jar/data is the spec throughout.** Every
  behaviour claim above is grounded in one of: the oracle's exports
  (`commands.json`, `command-registries.json`, `tree.json`,
  `registries.json` — real vanilla 1.21.10 dispatcher output), `javap
  -c` on the named 1.21.10 server-game jar (Mojang-mapped, so no
  mappings front-end was needed) or the 1.21.10 client jar, the
  vanilla 1.21.10 datagen reports, or Brigadier 1.3.10 from the
  datagen libraries. The 26.3 base's form was never trusted — and
  this batch is the proof of why: the port itself trusted it, and the
  defects below are the result. Two places where trusting the 26.3
  form would have given the WRONG verdict: G04 (vanilla changed
  Angle→Rotation since 1.21.7) and G01 (1.21.10's vanilla uses
  camelCase gamerules, 26.x uses snake_case).
- **Corpus regeneration.** The corpus was regenerated from 1.21.10's
  own tree export with the tree's `tools/command_parse_corpus.py`
  (byte-identical to 26.1.2's), run from a shadow dir because the
  worktree is read-only; only the EXTRA list was adapted (16 dropped,
  16 added — §0). 32,365 commands, 494 EXTRA entries, 255 gamerule
  cases, md5 `6fa6f661f641dc84f86c099184f1c644`. The brief's drop-list
  parenthetical was wrong on `fetchprofile` and `publish` (both exist
  in vanilla 1.21.10) — their EXTRA cases were kept.
- **Tally scheme (stated explicitly — the 26.1.1 run needed
  restatements anyway).** 24 as-tallied (5 pass, 19 gap) = 20
  behaviour (4 pass, 16 gap) + 4 non-behaviour (1 pass, 3 gap: H01 +
  O01/O02/O03). The 20 behaviour checks are: G01–G13 with G07 split
  into its two parts (14 checks: G01, G02, G03, G04, G05, G06, G07a,
  G07b, G08, G09, G10, G11, G12, G13) + G14, G15, G16, G17, G18 (5) +
  the `/time set` inverted pass (1). Behaviour passes: G04, G10, G11
  + `/time set`. Behaviour gaps: G01, G02, G03, G05, G06 (partial —
  part (a) gap, part (b) closed), G07a, G07b, G08, G09, G12, G13 +
  G14, G15, G16, G17, G18. G19/G20/G21 are run-minted ITEMS (26.1.1
  precedent: minted at merge, not counted as checks); G19 is
  withdrawn here. B9-O05 is a NEW oracle item, likewise not counted
  as a check (counted: 25 as-tallied, 5 pass, 20 gap). The
  error-text-resolution pass is O04's confirmation, not a separate
  check. The behaviour-only restatement is the coordinator's job
  (merge.py).
- **Systemic pattern**: the 1.21.10 command surface carries 26.3-era
  tree code against a 1.21.10 vanilla that predates it, 26.3-era
  oracle code (`TeamColor`, `Hud`, `EntityTypes`, `gamerules`
  subpackage, `npc.villager` subpackage) against a 1.21.10 vanilla
  that predates those, and 1.21.7-era parse code (particle/modifier
  handling, selector rollback) that the 26.2 port fixed but neither
  this tree nor the 26.1-line trees carried. The era template DID
  adapt some command code (`/worldborder` integer time, clock-free
  `/time`, no `stopwatch`, `fetchprofile` name+id, the gamerule
  persistence mapping) — but the adaptations stop at the tree's edge:
  the release tree, the argument-type table, and the oracles were
  carried verbatim from 26.3. The remaining risk is execution, not
  parsing: 18 of 22 drivers are unrun e2e drivers, and `command_parse`
  only covers parse behaviour — and now the linchpin `command_exec`
  is oracle-blocked too (B9-O05).
- **What the corpus cannot see**: it walks vanilla's tree, so
  Rust-only extras appear only via the empty-input root suggestion
  list (which shows all four G02 extras at all three perms) — the
  26.1.2 run's hand-written parse cases for the extra commands
  (`posteffect`, `compute`, `unpublish`, `swing`) were dropped from
  EXTRA per this batch's brief ("drop the 26.x-only commands"), so
  the four extras have no parse-level run footprint here. The tree
  diff (§3, B9-G02) is the complete check for top-level extras; the
  corpus is the complete check for missing/wrong vanilla behaviour. The dedicated tree (`/perf`) and the dev tree are
  outside both: the corpus walks the integrated tree, and the dev tree
  is not built by `release_dispatcher()` — B9-G14/G15 were caught by
  the static diff against `Commands.<init>` bytecode, not by any run.
  The subcommand-level extras (B9-G18) are likewise outside the corpus:
  no corpus case walks `/item fill`, `/item override`, or `execute
  … slots`, because vanilla's tree has no such nodes to walk.
- **Counts**: 97,095 comparisons (32,365 commands × 3 permission
  levels), 90,457 match, 6,638 mismatches across 3,325 unique inputs
  (§2); node diff 157 / 153 / 0 inputs (perm 4 / 2 / 0). Driver table:
  1 measured + 4 oracle-blocked + 17 not run = 22.

## 7. Open questions / follow-ups for the coordinator

0. **The template-adaptation lesson, still unpaid.** The era template
   adapted some command code for 1.21.10 but stopped at the tree's
   edge: the release tree still ships four 26.x-only commands, the
   26.3 snake_case gamerule table, and the 26.3 62-entry argument-type
   table — and the oracles were carried verbatim from 26.3, so
   `VanillaCommandExecOracle` (45 errors) was never adapted at all.
   The fixes for G03, G05, G07b, G12, G13 are the 26.1 tree's forms,
   already written once: `ResKey("minecraft:worldgen/configured_feature")`
   for `/place feature`, the gamemode argument back in `/publish`,
   single-entity `from entity` source, `Slot`/`Slots` for `/item` and
   `execute items`, and the 57-entry argument-type table with `color`
   at index 16. Any fix carried across versions must be version-gated:
   26.2's vanilla wants `team_color`, the selector refactor, and
   `unpublish`/`compute`, so the 1.21.10 fixes must NOT be copied
   forward blindly — and 1.21.11's vanilla wants the `Time` argument
   for `/worldborder` that 1.21.10 correctly lacks.
1. **B9-O05 is the batch's biggest remainder after the run itself.**
   `command_exec` is the linchpin for 7 of the 17 no-oracle drivers,
   and its oracle needs a 45-error adaptation to 1.21.10's API (26.3
   subpackages, `EntityType` vs `EntityTypes`, server-API drift).
   Recommend adapting the oracle first (with `--allow-gaps` on the
   driver), then resolving the `effect_view:always` fixture blocker,
   then the dependents.
2. The 17 unrun e2e drivers (esp. `command_exec` + its 7 dependents)
   are the batch's real remainder: they need a full client build and
   the oracle + fixture blockers resolved.
3. `scoreboard` is the cheapest remaining oracle repair in the batch
   (B9-O01: 6 errors, two sites — drop the three pack calls,
   `TeamColor` → `ChatFormatting`) — but it is a repair, not a free
   run, and it takes the 1.21.7 form, not 26.1.2's 3-error form.
4. B9-O02 (`team_rules`) is a medium repair (19 client errors, fully
   scoped in §4) — the `Hud`→`Gui` rework and the seven failed
   overrides are the new work vs 26.2.
5. B9-G14 (`/perf` in the dedicated tree) and B9-G15 (dev tree) are
   both tree-completeness gaps found by the static diff; neither is
   visible to the `command_parse` corpus (integrated tree only). If
   dev-tree parity is out of scope, G15 can be withdrawn (the 26.2
   merge kept it as a minor gap).
6. B9-G09 (selector) is the batch's most instructive finding: the
   Rust selector parser implements the 26.2 option-state shape, but
   vanilla 1.21.10 still has the old parser. A fix here must target
   1.21.10's form (no rollback on inapplicable, `!isTypeLimited` for
   `type`), NOT the 26.2 form — and any fix carried across versions
   must be version-gated, since 26.2's vanilla wants the opposite
   behaviour for `@a[type=player]`.
7. B9-G07a (item-modifier `function` key) and B9-G08 (particle `Name`
   compound) are fixes the 26.2 port already made — both are direct
   carry-overs from the 26.2 tree (`codec.rs` key-parameter dispatch,
   `args.rs` `Name`-accepting `particle_options`). They are the
   cheapest behaviour fixes in the batch.
8. B9-G02's four extra commands are one-command removals each from
   the tree builders — but check whether anything else in the 1.21.10
   tree references the nonexistent `post_effects` registry before
   removing `posteffect` (the `ArgType::Id("post_effects")`
   arguments at `tree_extra.rs:40,50` die with the command), and
   whether `compute`'s `context_sources` helper (`tree_data.rs:32-48`)
   is shared with `/data` before removing `compute`. (Run answer,
   §2.5: yes, it IS shared — 17 `data modify …` suggestion
   mismatches, all rust-only `compute`, same as 26.1.1/26.1.2.)
9. The oracle's `Src.suggestRegistryElements` override
   (`VanillaCommandParseOracle.java:93-99`) bypasses vanilla's RECIPE
   special case, so the `command_parse` driver cannot test recipe
   suggestions faithfully: the oracle suggests nothing where real
   vanilla suggests all recipes. The run's recipe suggestion
   mismatches are artifacts of that override (see B9-G11, §3.2) —
   do not read them as port defects. A future oracle fix would be
   to special-case `Registries.RECIPE` in `Src` the way
   `CommandSourceStack` does (suggest from the oracle's own
   `learnable_recipes` export — or, more faithfully, from all
   `RecipeManager` recipes). Until then, recipe-suggestion
   divergences (B9-G17) are verifiable only statically.
10. B9-G16 (node names) is proven statically here and confirmed by the
    run's node diff (§2): the node diff is the only check that sees
    the names the fixture actually emits (as opposed to the literals
    in the tree builders).
11. B9-O04 (prepared data lacks the lang file) is a data-snapshot gap,
    not a code defect: the prepare flow produces the file. The
    coordinator may want to re-run prepare for the sweep worktree's
    `.local` so later batches inherit a complete copy — this batch
    completed its own copy by jar extraction for the run.
12. G01's fix is NOT just a rename table: the engine rule
    `fire_spread_radius_around_player` (an int, default 128) has no
    single vanilla counterpart — vanilla has the `doFireTick` /
    `allowFireTicksAwayFromPlayer` bool pair. The command tree needs
    to expose the pair (via the existing `fire_radius` / `fire_keys`
    mapping) while the engine keeps its single int.
