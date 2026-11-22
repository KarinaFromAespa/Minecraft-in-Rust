# Batch 9 — Commands (1.21.7 parity sweep)

Date: 2026-11-22. Branch: `parity-sweep` (deliverables held as uncommitted
drafts — see §0). Scope: parsing, execution, selectors, functions,
scoreboard — 22 drivers; 57 argument types (1.21.7's own list).
Ledger slice: `pins/1.21.7/sweep/batch-09.json`.

## 0. Execution status — read first

**No driver was run by this batch.** Mid-setup, an execution-risk notice
blocked repo execution and git commands in the workdir (the in-flight
`git checkout parity-sweep` was stopped; the workdir
`~/workspace/mcir-work/sweep-batch-9` is left mid-checkout on main at
`a2f95487`). Per the Batch 2 precedent, this batch did static analysis
only (source + data + `javap` on the named 1.21.7 jars) and saved its
deliverables as drafts under
`~/workspace/mcir-work/sweep-batch-9-deliverables/`, uncommitted, awaiting
the user's go-ahead. Nothing in this batch touched production code
(`crates/`); no oracle was repaired (compiling is blocked too) — the three
oracle defects below are scoped, not fixed.

What this batch DID measure: Batch 0's end-to-end `command_parse` sample
(86,987 mismatches over 96,963 corpus parses) was triaged in full from the
saved `mismatches.jsonl` files — every mismatch classified, every distinct
defect verified against the Rust sources and, where it mattered, against
the vanilla 1.21.7 jar's bytecode. The 57 argument-type list and the
`/time set` literals were verified as instructed. The other 21 drivers are
classified by what blocks them (§5).

## 1. Tally

**18 checks: 2 PASS, 16 GAP.** 13 game defects (2 major, 11 minor) +
4 oracle/fixture defects (1 major, 3 minor). No fixes — checks + triage
only. Check→item map: the 2 passes are the `/time set` literals check
and the error-text-resolution check (game exonerated; the fixture defect
behind it is B9-O04). The 16 gaps are B9-G01–B9-G13 (13 behaviour
checks, each one defect) + B9-O01–B9-O03 (3 oracle-compilability
checks). The 18 unrun drivers are not counted as checks — they are
listed in §5 with their blockers.

Driver tally (22 in scope): 1 measured (`command_parse`, by Batch 0, data
triaged here), 0 newly run, 3 oracle-blocked (`scoreboard`,
`team_rules`, `exploration_map_command`), 18 not run — all game-binary
e2e drivers needing a full `fast`-profile client build plus a Java
reference run (§5).

## 2. The command_parse triage (Batch 0 measurement, classified here)

Source: `versions/1.21.7/.local/command-parse-parity/` in the Batch 0
workdir — `report.json` + `perm{4,2,0}/mismatches.jsonl` (27,350 / 27,401 /
32,236 mismatches over 32,321 corpus commands per permission level;
4,971/32,321 match at perm 4). Classification script:
`sweep-batch-9-deliverables/classify_cmdparse.py` (+ `classify_cmdparse2.py`
for the OK/cursor sub-patterns); full output in
`sweep-batch-9-deliverables/cmdparse-classification.txt`.

| Class | Count | Meaning |
|---|---|---|
| Error text only (ok + cursor agree; vanilla resolved message vs Rust translation key) | 86,241 | Fixture defect B9-O04 — NOT a game defect (§2.1) |
| Error text, other key forms (`particle.*`, `argument.scoreboardDisplaySlot.*` — same key-vs-resolved shape) | 70 | Same fixture defect B9-O04 |
| OK differs (parse accept/reject disagreement) | 315 | Real defects B9-G01–G03, G05–G08, G10 (§2.2) |
| Cursor differs (same ok, different error position) | 345 | Real defects, mostly B9-G06/G09/G10/G12 (§2.3) |
| Suggestions differ | 16 | Real defects B9-G10/G11 (§2.4) |
| **Total** | **86,987** | |

### 2.1 The 86,311 error-text mismatches are a fixture defect (B9-O04)

The Rust probe emits the raw translation key (`command.unknown.command`,
`argument.resource_or_id.no_such_element`, …) where vanilla emits the
resolved Brigadier message. Root cause, verified in source: the
`command_parse_fixture` probe loads its language only from the prepared
resources root (`lab.RESOURCES` =
`.local/horizon-client/1.21.7`), which contains **no lang files at all**
— so `lang::get(key)` falls back to the key itself
(`horizon-client-core/src/lang.rs`, `get()` returns the key on a miss;
`Text::translate` resolves at construction time). The game itself is
exonerated: it loads language from the pack root built on the vanilla
asset root (`game/packs.rs`, `startup_root(vanilla_root, …)`), and the
1.21.7 client jar's `assets/minecraft/lang/en_us.json` (7,392 keys)
contains every key with exactly the vanilla text the oracle produced
(e.g. `command.unknown.command` → "Unknown or incomplete command, see
below for error"). Fix direction (for the fix decision, not this batch):
load the fixture's language from the vanilla asset root too, or resolve
keys in the comparison. Until then the driver's error-text comparison is
unreadable — 99.2% of its mismatches are this one artifact.

Top keys: `command.unknown.command` 55,949;
`command.unknown.argument` 19,824;
`argument.resource_or_id.no_such_element` 7,348;
`argument.literal.incorrect` 254; `argument.player.entities` 200;
`command.expected.separator` 190; the rest <200 each across ~40 keys.

### 2.2 OK_DIFFERS (315) — real parse divergences

Vanilla-OK / Rust-FAIL (264 — Rust rejects valid commands):

| Commands | N | Defect |
|---|---|---|
| `gamerule <rule>` — every rule name | 220 | B9-G01 |
| `place feature minecraft:acacia` (+ variants) | 22 | B9-G03 |
| `spawnpoint`/`setworldspawn … 0` (single angle) | 8 | B9-G04 |
| `publish true survival` | 4 | B9-G05 |
| `bossbar … {"nbt":"a","entity":"@x"}` | 4 | B9-G06 |
| `item replace … {"function":"minecraft:set_count","count":1}` | 4 | B9-G07 |
| `particle minecraft:block{block_state:{Name:"minecraft:stone"}}` | 2 | B9-G08 |

Vanilla-FAIL / Rust-OK (51 — Rust accepts invalid commands):

| Commands | N | Defect |
|---|---|---|
| `stopwatch …` | 10 | B9-G02 |
| `gamerule keep_inventory` (snake_case) | 8 | B9-G01 |
| `swing @s` | 6 | B9-G02 |
| `bossbar … {"object":"atlas"/"player",…}` | 8 | B9-G06 |
| `time set nope` | 4 | B9-G02 (timemarker node) |
| `fetchprofile …` | 4 | B9-G02 |
| `execute if stopwatch …` | 2 | B9-G02 |
| `spawnpoint @s ~ ~ ~ ~ ~` / `setworldspawn 0 0 0 90 0` | 4 | B9-G04 |
| `worldborder set 100 10s` | 2 | B9-G10 |
| `item replace … from entity @a …` (multi-entity source) | 2 | B9-G07 |
| `unpublish` | 1 | B9-G02 |

### 2.3 CURSOR_DIFFERS (345)

| Pattern | N | Defect |
|---|---|---|
| Bossbar component cases: vanilla fails later ("Incorrect argument", cursor at end of input); Rust fails on the component itself (cursor at component start) | 252 | B9-G06 (same root as §2.2) |
| Selector duplicate option: Rust reports the error at the option-name start, vanilla at the end of the duplicated value | 28 | B9-G09 |
| Selector inapplicable option on `@a`/`@p`/`@r`: vanilla errors at the option; Rust accepts the selector and fails later with a generic error | 16 | B9-G09 |
| `Unknown slot '@@bad@@'` / `Unknown slot ''`: different error + cursor | 10 | B9-G12 |
| `swing`/`compute` unknown-command cursor (0 vs end-of-literal) | 8 | B9-G02 (downstream: Rust has the command, vanilla doesn't) |
| Gamerule value errors (Rust fails at cursor 9 before reaching the value) | 12 | B9-G01 (downstream) |
| `place feature` / `publish` follow-on cursor diffs | 8 | B9-G03/B9-G05 (downstream) |
| `time set  1` (double space) off-by-one | 2 | B9-G02 (downstream of timemarker) |
| Bossbar invalid-component trailing-space cases | 8 | B9-G06 |
| `publish true extra` ("Unknown game mode") | 1 | B9-G05 (downstream) |

### 2.4 Suggestions (16)

- `worldborder` (12): after a complete time argument (`worldborder add 0
  0`), vanilla suggests nothing; Rust suggests `["d","s","t"]` — B9-G10
  (downstream of the wrong Time arg type).
- `summon pig` (2): tooltip text — vanilla resolves ("Pig"), Rust shows
  the key (`entity.minecraft.pig`) — B9-O04, same fixture cause.
- `recipe give @s minecraft:stick` (2): vanilla suggests nothing after a
  complete recipe id; Rust suggests prefix matches — B9-G11.

## 3. Defects

### B9-G01 — MAJOR — `/gamerule` rejects every 1.21.7 rule name; accepts 26.3 names

`crates/horizon-client-core/src/gamerules.rs` is the **26.3** gamerule
table with **snake_case** names (`keep_inventory`, `advance_time`,
`block_drops`, `spawn_mobs`, `max_block_modifications`, …); the
`/gamerule` tree is built from it (`horizon-command/src/vanilla.rs:301`).
Vanilla 1.21.7's rules are **camelCase** — jar-verified via `javap -c` on
`net.minecraft.world.level.GameRules` (56 rules: `keepInventory`,
`doFireTick`, `allowFireTicksAwayFromPlayer`, `doDaylightCycle`,
`commandModificationBlockLimit`, `spawnChunkRadius`, …). Every vanilla
rule name is rejected at cursor 9 (220 corpus cases); every 26.3 name is
wrongly accepted (8 cases, e.g. `keep_inventory`). The table also renames
rules (`advance_time` for `doDaylightCycle`), inverts some
(`elytra_movement_check` for `disableElytraMovementCheck`, `raids` for
`disableRaids`), splits others (`spawn_mobs`/`spawn_monsters`/
`spawn_phantoms` for `doMobSpawning`), and drops `doFireTick` entirely —
the last is also the pilot's G4 (gamerule-table gap); this defect is the
parser-visible whole-namespace consequence. Related: the leakage audit's
26.3-leak class; the E6 re-base triage should carry the 1.21.7 table.

### B9-G02 — MAJOR — 26.x commands and subcommands in the 1.21.7 release tree

Full top-level diff of the Rust tree (`d.literal(root, …)` across
`tree_*.rs`, plus redirects) against vanilla's tree export
(`tree.json`, the oracle's dump of the real 1.21.7 dispatcher, 73
top-level commands): Rust has 85 top-level literals; after removing
dev-gated (`raid`, `warden_spawn_tracker` — `if dev`, correctly excluded
from `release_dispatcher()`), Horizon-gated (`pregen`), dedicated-only
(`ban`, `banlist`, `setidletimeout`, `transfer`, `whitelist` —
`register_dedicated` only, correctly absent from the integrated tree),
and redirects (`tell`/`w` → `msg`, present), the genuine 26.x extras in
the release tree are:

- `/stopwatch` (`tree_misc.rs:293`), `/swing` (`tree_world.rs:170`),
  `/fetchprofile` (`tree_extra.rs:54`), `/unpublish`
  (`tree_extra.rs:83`), `/compute` (`tree_data.rs:51`), `/posteffect`
  (`tree_extra.rs:36`) — none in vanilla 1.21.7's tree; corpus-proven for
  stopwatch/swing/fetchprofile/unpublish/compute.
- `/time` clock subcommands: `pause`, `resume`, `rate`, `of <clock>`,
  `set <timemarker>`, `query <timeline>` (`vanilla.rs:480`
  `clock_nodes`) — vanilla's `/time` has only `add`/`query`/`set`, and
  `set` has only `day`/`noon`/`night`/`midnight`/`time` (tree.json).
  The `timemarker` node is why Rust accepts `time set nope`.
- `/execute … stopwatch …` (`tree_execute.rs:193`) — no `stopwatch`
  child in vanilla's `execute` node.

(`waypoint` and `dialog` are in vanilla 1.21.7's tree — not defects.)

### B9-G03 — minor — `/place feature` looks up the wrong registry

`tree_world.rs:62` uses
`ArgType::ResourceOrId("minecraft:worldgen/feature")` — the feature-*type*
registry. Vanilla 1.21.7's `PlaceCommand` uses
`Registries.CONFIGURED_FEATURE` (javap: `PlaceCommand` references
`CONFIGURED_FEATURE`; `placeFeature` takes a `ConfiguredFeature`).
`minecraft:acacia` is a configured feature (224 ids in the oracle's
`by_registry`), not a feature type — so every `/place feature <name>`
fails with `no_such_element` (22 cases). The fixture data is correct;
the registry string is wrong.

### B9-G04 — minor — `spawnpoint`/`setworldspawn` parse the angle as a rotation

`vanilla.rs:341,349` use `ArgType::Rotation` (yaw+pitch pair) for the
trailing angle. Vanilla uses `AngleArgument` (single float) — javap:
`SetSpawnCommand` and `SetWorldSpawnCommand` both call
`AngleArgument.angle()` / `getAngle()`. Rust rejects a single value
(`rotation.incomplete`, 8 cases) and accepts two values vanilla rejects
(4 cases).

### B9-G05 — minor — `/publish` is missing the gamemode argument

`tree_extra.rs:77-82`: `publish [<allowCommands>] [<port>]`. Vanilla
1.21.7's `PublishCommand` is `publish [<allowCommands>] [<gamemode>]
[<port>]` (javap: `BoolArgumentType.bool`, then
`GameModeArgument.gameMode`, then `IntegerArgumentType.integer(0,
65535)`). `publish true survival` fails in Rust with
`parsing.int.expected` (4 cases + follow-on cursor diffs).

### B9-G06 — minor — Chat-component decoder: too strict on NBT, too lax on objects

`horizon-command/src/component.rs`, two sub-defects, both jar-verified:

- (a) NBT components with an entity source are rejected when the selector
  string doesn't parse (`selector_string` check at the `"nbt"` arm).
  Vanilla accepts `{"nbt":"a","entity":"@x"}`: `EntityDataSource`'s
  `compileSelector` catches `CommandSyntaxException` and returns null
  (javap: exception table `0–21 → 22`, `aconst_null; areturn`) — the
  codec never fails on a bad selector. 4 OK-diff cases + 252 cursor-diff
  cases + 8 trailing-space cases.
- (b) `object` components (`{"object":"atlas",…}`,
  `{"object":"player",…}`) are accepted by Rust but rejected by vanilla
  ("No matching codec found"). Vanilla 1.21.7 has **no object component
  type at all**: the jar's `network/chat/contents/` holds only
  `PlainText`, `Translatable`, `Keybind`, `Score`, `Selector`, `Nbt`
  (+ data sources) — no `ObjectContents`. The `object` arm is a 26.x
  carry-over. 8 cases.

### B9-G07 — minor — Item modifiers: inline JSON fails; multi-entity source accepted

- (a) `codec.rs:296` decodes `minecraft:item_modifier` via `dispatch()`,
  which reads the `"type"` key (`codec.rs:52`). Loot item modifiers use
  the `"function"` key (`{"function":"minecraft:set_count","count":1}`)
  — so every inline modifier fails with `failed_to_parse` (4 cases).
  Vanilla accepts it.
- (b) `/item … from entity <source>` uses `ENTITIES` (multi) in
  `tree_misc.rs`; vanilla's `ItemCommands` uses `EntityArgument.entity()`
  (single) for the source — "Only one entity is allowed, but the provided
  selector allows more than one" (2 cases; the target correctly stays
  multi).

### B9-G08 — minor — Particle `block_state` compound with `Name` is rejected

`args.rs:997` `particle_options`: for `minecraft:block`, any non-string
`block_state` is an error. Vanilla accepts the BlockState-codec SNBT form
`block_state:{Name:"minecraft:stone"}` (2 cases). The Rust code requires
either a string or a compound with `id`; the `Name`-form compound is the
form vanilla's codec takes.

### B9-G09 — minor — Selector option errors: wrong cursor; `type=` accepted for `@a`/`@p`/`@r`

`selector.rs`, two sub-defects:

- (a) Duplicate-option errors roll back to the option-name start
  (`rollback(…, start, …)` in `parse_options`); vanilla reports the error
  at the end of the duplicated value (28 cases, e.g. `@e[type=pig,
  type=cow]`: vanilla cursor 34, Rust 30).
- (b) `@a`/`@p`/`@r` push `minecraft:player` into `s.types` but never
  mark `type_option` consumed (`parse_selector`, `selector.rs:215-242`),
  so `can_use("type")` stays true and `@a[type=player]` parses — vanilla
  rejects it as inapplicable ("Option 'type' isn't applicable here", 16
  cases; Rust then fails later with a generic error).

### B9-G10 — minor — `/worldborder` time is a Time argument; vanilla uses an integer

`tree_world.rs:318` worldborder `set`/`add` time uses `ArgType::Time`
(float + `d`/`s`/`t` suffix). Vanilla 1.21.7's `WorldBorderCommand` uses
`IntegerArgumentType.integer(0)` (javap: 4 call sites, seconds). Rust
accepts `worldborder set 100 10s` (2 cases) and suggests `["d","s","t"]`
after a complete time argument where vanilla suggests nothing (12
suggestion cases).

### B9-G11 — minor — Suggestions offered for complete valid arguments

`recipe give @s minecraft:stick`: vanilla suggests nothing (start 0, end
0, empty list); Rust suggests prefix matches
(`minecraft:stick_from_bamboo_item`, `minecraft:sticky_piston`) at the
argument start (2 cases). Same shape as B9-G10's suggestion symptom:
vanilla offers no completions once the last argument is complete and
valid; Rust re-suggests. (The `summon pig` tooltip cases are B9-O04, not
this defect — the lists agree, only the tooltip resolution differs.)

### B9-G12 — minor — Slot error text/cursor divergences

`execute if items … @@bad@@`: vanilla "Unknown slot '@@bad@@'" at end of
input; Rust reports an SNBT error mid-input (8 cases). `item replace
block … <empty slot>`: vanilla "Unknown slot ''"; Rust reports
`no_such_element` one position later (2 cases). The slot-argument error
path (`args.rs` Slot/SlotSource parsing) doesn't match vanilla's
`SlotRanges`/slot-argument reporting.

### B9-G13 — minor — Argument-type table carries the 26.x name at index 16

`horizon-protocol/.../play/commands.rs` `ARGUMENT_TYPES` is 57 entries,
count and order verified identical to vanilla 1.21.7's
`ArgumentTypeInfos` registration order (javap, static initializer) —
**the 481494bd fix holds**. One entry carries the 26.x name:
`minecraft:team_color` where vanilla 1.21.7's registry has
`minecraft:color` (index 16; `minecraft:color` is in the oracle's
57-entry list, `team_color` is not). Functionally harmless for network
decode (numeric ids; index 16 decodes to `TeamColor` either way), but a
latent inconsistency for anything comparing names against vanilla's
registry.

### Passes (verified, not re-diagnosed)

- **`/time set day|noon|night|midnight` literals hold** (481494bd):
  present in `clock_nodes` (`vanilla.rs:482-485`) and in vanilla's tree
  (`tree.json`: `time.set` children `day/midnight/night/noon/time`).
  No corpus mismatch involves them.
- **57 argument-type list holds** (481494bd): count 57, order identical
  to vanilla's `ArgumentTypeInfos` registration order (javap); see
  B9-G13 for the single name drift.
- **Error-text resolution at runtime**: the game resolves command error
  keys correctly (see B9-O04) — the 86,241 mismatches are not game
  behaviour.

## 4. Oracle / fixture defects

### B9-O01 — minor — `VanillaScoreboardOracle` fails javac (6 errors) — blocks `scoreboard`

`batch-0-oracles.json` / `results-final.json`: fails on both classpaths,
`missing-symbol`, errors at lines 44 (×3), 242, 436 (×2). Causes,
verified against the jar:

- Line 44 (`store()`): calls `packObjectives()`, `packDisplaySlots()`,
  `packPlayerTeams()` — none exist on 1.21.7's `Scoreboard` (only
  `packPlayerScores()` is there; packing lives in
  `ScoreboardSaveData.pack()`). `ScoreboardSaveData$Packed` itself
  exists with the same four components.
- Lines 242/436: `TeamColor` — no such class in 1.21.7;
  `PlayerTeam.getColor()`/`setColor()` use `ChatFormatting`.
  (Line 436 also has an incompatible-types error from the same cause.)

Repair scope (small, in-family, not attempted — compiling is blocked):
pack via `ScoreboardSaveData`, `TeamColor` → `ChatFormatting` at 242/436.
Needs an end-to-end driver run after repair (compiling ≠ running).

### B9-O02 — minor — `VanillaTeamRulesOracle` fails javac (19 client errors) — blocks `team_rules`

Client cp: `missing-symbol` at 14, 51; 26.x-only package
`net.minecraft.world.entity.monster.zombie` at 24; five `method does not
override` errors at 60–80 (26.x overrides of `Entity`/
`LivingEntityRenderer` methods). Server cp fails harder (53 errors,
`net.minecraft.client` absent — the driver uses the client jar, so the
client cp is the operative one). Repair scope: re-home the zombie import
to `world.entity.monster`, rework the five overrides against 1.21.7's
`Entity`/`LivingEntityRenderer` APIs, resolve the two missing symbols.
Medium; not attempted.

### B9-O03 — minor — `LocatorBarClient` fails javac (20 errors) — blocks `exploration_map_command`

`missing-symbol` at 222, 231, 393–395, 407, 409, 413, … (20 server-cp
errors; also fails on client cp). `test_exploration_map_command_parity.py`
compiles it as its Java packet client (line 39) and runs a real Java
server + the game. Repair scope: 20 missing symbols against the 1.21.7
client jar — needs per-symbol javap work; out of family for a commands
batch (packet-client helper shared with the locator-bar harness).
Not attempted.

### B9-O04 — MAJOR — `command_parse` fixture has no language data (probe defect, not a game defect)

See §2.1. The fixture loads language only from the prepared-resources
root, which has no `assets/minecraft/lang/en_us.json`, so all 86,241
error-text comparisons (and the 2 `summon` tooltip cases) render as raw
keys. The game resolves correctly at runtime. Fix direction: load the
fixture language from the vanilla asset root (as the game does via
`startup_root`) or resolve keys before comparing. Until fixed, the
`command_parse` driver's error-text leg is unreadable and its mismatch
count (86,987) overstates the real divergence by ~99%.

## 5. Driver table (22)

| Driver | Oracle | Status |
|---|---|---|
| command_parse | VanillaCommandParseOracle (pass) | **Measured by Batch 0; triaged here** (§2) |
| command_exec | VanillaCommandExecOracle (pass, repaired 69e93c10) | Not run (execution-risk block). Needs full game build + Java integrated server. Known fixture blocker (brief): `/execute if predicate effect_view:always` — the `effect_view:*` predicates come from `tools/mob_effect_predicate_corpus.py`, imported by `command_exec_corpus.py:64`. Recorded, not chased. |
| scoreboard | VanillaScoreboardOracle (FAIL) | Oracle-blocked — B9-O01 |
| team_rules | VanillaTeamRulesOracle (FAIL) | Oracle-blocked — B9-O02 |
| exploration_map_command | LocatorBarClient (FAIL) | Oracle-blocked — B9-O03 |
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
plus a Java run; none was attempted under the execution-risk block, and
each is a 10–30+ min job on this VM even unblocked. `command_exec` is the
linchpin: 7 of the 17 no-oracle drivers import it or shell out to it, so
its fixture blocker gates the family.

## 6. Method notes

- **Run before writing** was honoured where a run existed: the only
  measurement available (`command_parse`, Batch 0) was triaged before any
  static check was designed. Everything else is method 2 (ledger-item
  pattern: javap + source + oracle data), because no driver could run.
- **Vanilla 1.21.7's own jar/data is the spec throughout.** Every
  behaviour claim above is grounded in one of: the oracle's
  `vanilla.jsonl`/`tree.json`/`registries.json` (real vanilla 1.21.7
  dispatcher output), `javap -c` on the named 1.21.7 server jar
  (`server-game.jar`), or the client jar's `en_us.json`. The 26.3 base's
  form was never trusted — and the dominant defect pattern in this batch
  is exactly 26.x carry-over: the gamerule table (G01), the command tree
  (G02), the object component type (G06b), `TeamColor` (O01),
  `team_color` (G13).
- **Systemic pattern**: 6 of the 13 game defects (G01, G02, G06b, G13,
  and the `time` clock nodes inside G02) are "26.x form shipped for a
  1.21.7 version" — the same bug class as the pilot's tag defects and
  the leakage audit's findings, in the command tree rather than in
  tags. The other 7 are local argument/type errors (wrong registry,
  wrong argument type, wrong key, wrong cursor).
- **What the corpus cannot see**: it walks vanilla's tree, so 26.x-only
  commands appear only via hand-written edge cases (`stopwatch`,
  `swing`, `fetchprofile`, `unpublish`, `compute` were caught that way;
  `posteffect` and the `/time` clock nodes were caught by the static
  tree diff, not the corpus). The tree diff (§3, B9-G02) is the complete
  check for extras; the corpus is the complete check for missing/wrong
  vanilla behaviour.
- **Counts**: the corpus has 32,321 commands × 3 permission levels =
  96,963 parses; 9,976 match. The 86,987 mismatches collapse to 13 game
  defects + 1 fixture defect. Do not quote 86,987 as a defect count.

## 7. Open questions / follow-ups for the coordinator

1. The 18 unrun e2e drivers (esp. `command_exec` + its 7 dependents)
   are the batch's real remainder: they need the execution-risk
   go-ahead, a full client build, and the `effect_view:always` fixture
   blocker resolved. Recommend running `command_exec` first (with
   `--allow-gaps`), then its dependents.
2. B9-O01 (`scoreboard`) is the cheapest oracle repair in the batch
   (6 errors, fully scoped in §4) — a good first repair once compiling
   is allowed; it unblocks a driver with no game-build cost (probe-based).
3. B9-O04 (fixture language) should be fixed before the next
   `command_parse` run, or the driver will report ~86k mismatches again
   and hide the real 600-odd.
4. G01 (gamerule table) overlaps the pilot's G4 and the E6 re-base: the
   1.21.7 camelCase table is needed by the parser, the runtime
   (`GameRules` value map), and the save format — a version-table
   decision, not a parser-local fix. Same table work likely serves
   1.21.5/1.21.6/1.21.8/1.21.9/1.21.10 (each needs its own jar check).
5. The `/time` clock nodes and `stopwatch`/`swing`/`compute`/`posteffect`
   suggest the command tree was ported from 26.3 wholesale; a tree-level
   audit against each version's `tree.json` (as done here for 1.21.7)
   would catch the same class in the E6 siblings cheaply.
