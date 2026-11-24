# Batch 9 — Commands (26.2 parity sweep)

Date: 2026-11-24. Branch: `parity-sweep-26.2` (synced to main 732c800f).
Scope: parsing, execution, selectors, functions, scoreboard — 22 drivers;
57 argument types (26.2's own list), 59 game rules.
Ledger slice: `batch-09.json` (this directory).

## 0. Execution status — read first

**Prepare landed during this run.** When the batch started, no prepared
26.2 data existed and the brief recorded the `command_parse` driver run as
a needs-harness follow-up (B9-H). Mid-batch, `horizon-prepare` for 26.2
finished (2048.7 s, EXIT 0) and the coordinator directed the batch to run
`command_parse` directly rather than record B9-H. This batch therefore did
run one driver end-to-end: `command_parse` (§2), via
`tools/test_command_parse_parity.py` — oracle compiled clean, corpus
generated from the oracle's own tree export, Rust side via the
`client_probe` `command_parse_fixture` example (built `--profile fast`,
`-j 1`, `taskset -c 0`, `RUSTFLAGS=-C codegen-units=1`, alongside other
agents' cargo jobs on the shared 6 GB cell; the lib rustc was briefly
SIGSTOP'd once so another agent's check could finish first). The run
survived two interruptions: the first driver attempt was SIGTERM'd
mid-Java-steps (relaunched detached via setsid/nohup), and the VM
rebooted mid-build during `horizon-client-lib` codegen — the build
resumed from the warm cargo cache (only the lib + probe recompiled) and
the resumed build finished in 35m04s.

Prepared data used: `versions/26.2/.local/horizon-client/26.2/` (203 MB),
including `commands.json` (verified identical to the vanilla 26.2 datagen
dump), `command-registries.json` (VanillaCommandParseOracle export),
`registries.json`, and `assets/minecraft/lang/en_us.json` (md5-identical
to the 26.2 client jar's file, 8,123 keys). `sounds.json` is absent from
the prepared root (the VM lacks the official launcher's asset index
32.json) — expected, not a port defect, and irrelevant to this batch: no
command check needs it.

Nothing in this batch touched production code (`crates/`); no oracle was
repaired (the two broken in-scope oracles are scoped in §4, not fixed);
no git commits were made. All behaviour claims are grounded in 26.2's OWN
jar/data — the named server-game jar
(`26.1-adapters/versions/26.2/jars/server-game.jar`, Mojang-mapped),
the 26.2 client jar (`leakage-audit/jars/26.2-client.jar`), the vanilla
26.2 datagen reports, or the oracle's own exports. The 26.3 base's form
was never trusted — and the headline finding of this batch is how often
that mattered in reverse: six of the thirteen 1.21.7 defects are CLOSED
for 26.2 because vanilla 26.2 itself uses the 26.x form the 1.21.7 batch
flagged as carry-over.

## 1. Tally

**21 checks: 16 PASS, 5 GAP.** 3 game defects (0 major, 3 minor) +
2 oracle defects (0 major, 2 minor). No fixes — checks + triage only.

Check→item map: the 13 re-checks of the 1.21.7 defects (B9-G01–G13) are
all PASS for 26.2 — every one is closed, six of them because vanilla 26.2
changed to the form the port already ships (G04, G06a, G06b, G09, G10,
G13), the rest because the 26.2 port fixed them (G03, G05, G07, G08, G12)
or because the 26.x form was correct all along (G01, G02, G11). The two
1.21.7 passes were re-verified: the `/time set` literals check PASSES in
inverted form (vanilla 26.2 has no day/noon/night/midnight literals and
neither does the port), and the 57 argument-type list PASSES exactly.
The `command_parse` measurement PASSES with zero mismatches (§2). The
5 gaps are the two new tree gaps (B9-G14 dedicated `/perf`, B9-G15 dev
tree), the argument-name divergences found by the run's node check
(B9-G16), and the two still-broken oracles (B9-O02, B9-O03). B9-O01
(scoreboard oracle) and B9-O04 (fixture language) are CLOSED.

Driver tally (22 in scope): 1 measured (`command_parse`, run by this
batch — §2), 0 newly run beyond it, 2 oracle-blocked (`team_rules`,
`exploration_map_command`), 19 not run — all game-binary e2e drivers
needing a full `fast`-profile client build plus a Java reference run (§5).

## 2. The command_parse measurement (run by this batch)

Corpus: **34,581 commands** (tree-walk of the oracle's own 77-command
integrated tree export + hand-written edge cases: `swing @s`,
`stopwatch …`, `fetchprofile`, `unpublish`, `publish true survival
25565`, `compute`, `raid start 5`, `execute if stopwatch …`, 484
`/gamerule` variants). Run at permission levels 4, 2 and 0 against the
26.2 server-game jar oracle and the `fast`-profile `client_probe`
fixture. Report:
`versions/26.2/.local/command-parse-parity/report.json`.

| Permission | Commands | Match | Mismatches | Parse-OK (both sides) |
|---|---|---|---|---|
| 4 | 34,581 | 34,581 | **0** | 5,577 |
| 2 | 34,581 | 34,581 | **0** | 5,521 |
| 0 | 34,581 | 34,581 | **0** | 72 |
| **Total** | **103,743** | **103,743** | **0** | — |

**Zero mismatches across 103,743 parses** — parse success, error text,
error cursor, and end-of-input suggestions all agree exactly at every
permission level. Every mismatch class is empty: 0 OK-differs, 0
cursor-differs, 0 error-text-only, 0 suggestion mismatches. In
particular the error-text leg is clean (0 mismatches), confirming
B9-O04's closure: the prepared 26.2 root's jar-identical `en_us.json`
lets the fixture resolve every error key, and the 1.21.7 run's 86,241
error-text mismatches do not recur. The run independently confirms the
static closures: the 484 `/gamerule` cases all match (G01), the
`swing`/`stopwatch`/`fetchprofile`/`unpublish` edge cases all match
(G02), the `/time` clock cases all match (G02/G10), the `publish true
survival 25565` case matches (G05), and the inline-modifier cases
(`{"function":"minecraft:set_count","count":1}`) match (G07).

One check the driver's `compare()` does NOT perform is the parsed-node
path (it strips `nodes` before comparing). This batch compared nodes
separately: **14 node-path differences** at perm 4 and perm 2 (same 14;
0 at perm 0), all in `/item replace` and `/execute if items` — same
structure, different argument-name labels (see B9-G16, §3.1). Parse
behaviour on all 14 is identical (ok/cursor/suggestions agree); the
names surface only in the tree's network form. The node check also has
one blind spot: failed parses carry no nodes, so a name difference on a
path that never parses OK in the corpus is invisible to it — that is
exactly the `execute if items block` case (all 260 corpus variants fail
on the slots sample `0`, "Unknown slot '0'"), verified by code + datagen
dump instead and included in B9-G16.

## 3. Defects

### 3.1 Open defects

### B9-G14 — minor — `/perf` missing from the dedicated tree

Vanilla 26.2 registers `PerfCommand` inside the `includeDedicated` block
of `net.minecraft.commands.Commands` (javap: `PerfCommand.register` at
bytecode offset 710, between `PardonIpCommand` and `SaveAllCommand`,
inside the `includeDedicated` guard at 651–762). The command is
`perf start|stop`, owners permission
(`PerfCommand.register`: `literal("perf")`, `start`/`stop` executables;
datagen `commands.json` agrees). The Rust release (integrated) tree
correctly lacks it, but the dedicated tree
(`tree_extra.rs:register_dedicated`, 14 commands) lacks it too — the only
one of vanilla's 15 dedicated commands not registered. Single command,
dedicated-only, profiling use — minor.

### B9-G15 — minor — Dev tree is missing six vanilla 26.2 dev commands

Vanilla 26.2's dev block (`SharedConstants.DEBUG_DEV_COMMANDS ||
IS_RUNNING_IN_IDE`, `Commands` constructor offsets 580–632) registers
`raid`, `debugpath`, `debugmobspawning`, `warden_spawn_tracker`,
`spawnarmortrims`, `serverpack` (plus `debugconfig` when dedicated, plus
`chase` under the separate `DEBUG_CHASE_COMMAND` flag). The Rust dev tree
has only the two `if dev` blocks in `vanilla.rs:161,385` (`raid`,
`warden_spawn_tracker`). `debugpath`, `debugmobspawning`,
`spawnarmortrims`, `serverpack` (and `debugconfig`/`chase`) have no Rust
tree node at all (grep over `horizon-command/src` finds no mention).
Dev-tree only — no release player ever sees these — minor, and the
coordinator may rule dev-tree parity out of scope.

### B9-G16 — minor — Argument node names differ from vanilla in `/item replace` and `/execute if items`

Found by this batch's node-path comparison on the §2 run (the driver's
`compare()` strips `nodes`, so this is invisible to the driver itself).
The parse structure is identical — same length, same positions — only
the argument-name labels differ. Vanilla names from the 26.2 datagen
`commands.json` dump (confirmed by the oracle's live tree):

| Location | Vanilla 26.2 | Rust | Rust source |
|---|---|---|---|
| `/item replace block <pos>` | `pos` | `targetPos` | `tree_misc.rs:174,190` |
| `/item replace entity <targets>` | `targets` | `target` | `tree_misc.rs:172,188` |
| `/item replace … from block <source>` | `source` | `sourcePos` | `tree_misc.rs:211` |
| `/execute if items entity <entities>` | `entities` | `source` | `tree_execute.rs:230` |
| `/execute if items block <pos>` | `pos` | `sourcePos` | `tree_execute.rs:237` |

14 corpus commands show the difference (13 `/item replace`, 1
`/execute if items entity`); the `/execute if items block` row is
invisible to the corpus because all 260 block variants fail on the
slots sample (`0` is not a valid slot) and failed parses carry no
nodes — verified by code + dump instead. Parse behaviour on all 14 is
identical (ok/cursor/error/suggestions agree), so this is cosmetic:
the names surface only in the tree's network form (the `commands`
packet carries argument node names) and in tooling that reads node
paths. Vanilla is itself inconsistent here — `execute if data` uses
`source`/`sourcePos` and `execute store` uses `target`/`targetPos`,
both matching Rust — so the Rust author evidently copied the
`store`/`data` naming into `item`/`items` where vanilla uses different
names. The 1.21.7 tree carries the same names (`tree_misc.rs:167-200`),
so this is a shared upstream naming choice, not a 26.2 regression —
but it is a divergence from vanilla 26.2 all the same. Fix is a
five-name rename in the tree definitions.

### 3.2 Closed — the 1.21.7 defects, re-checked against 26.2 (withdrawn candidates)

Each of the thirteen 1.21.7 Batch 9 defects was re-verified against
26.2's own jar/data. All thirteen are CLOSED for 26.2; none is reused as
a 26.2 defect. The closures fall into three groups.

**Group A — vanilla 26.2 itself uses the 26.x form (the 1.21.7 defect's
vanilla premise does not hold in 26.2):**

- **B9-G01 — CLOSED.** Vanilla 26.2's
  `net.minecraft.world.level.gamerules.GameRules` (the class moved from
  `world.level` in 26.x) registers **59 snake_case rules** (javap static
  initializer: `advance_time`, `keep_inventory`, `doFireTick` absent —
  there is no fire-tick rule in 26.2 at all — `spawn_mobs` /
  `spawn_monsters` / `spawn_phantoms` as separate rules, `raids`,
  `elytra_movement_check`). The Rust table (`gamerules.rs:51-126`) is the
  same 58 ungated rules in the same order with the same defaults (sampled:
  `fire_spread_radius_around_player` 128/min −1, `max_block_modifications`
  32768/1, `players_nether_portal_default_delay` 80/0,
  `tnt_explosion_drop_decay` false, `max_command_forks` 65536/0), plus
  `max_minecart_speed` (8/1/1000) gated behind the
  `minecraft:minecart_improvements` feature flag — exactly as vanilla
  gates it (`FeatureFlags.MINECART_IMPROVEMENTS`, static initializer
  offset 391–412). The `/gamerule` tree is built from
  `available_rules(minecart_improvements)` (`vanilla.rs:301`). Nit, not a
  defect: the `MAX_MINECART_SPEED` comment says "(26.3)" — the rule is in
  26.2 too.
- **B9-G04 — CLOSED (inverted).** Vanilla 26.2's `SetSpawnCommand` and
  `SetWorldSpawnCommand` use `RotationArgument.rotation()` (yaw+pitch
  pair), not `AngleArgument` — javap shows `rotation` /
  `RotationArgument.rotation()` in both `register` methods. The Rust
  `ArgType::Rotation` (`vanilla.rs:341,350`) matches vanilla 26.2.
- **B9-G06 — CLOSED (both parts, both inverted).**
  (a) Vanilla 26.2's `EntityDataSource` holds a
  `CompilableString<EntitySelector>` decoded via
  `EntitySelector.COMPILABLE_CODEC`; `CompilableString$CommandParserHelper.apply`
  catches `CommandSyntaxException` and returns `DataResult.error`
  (exception table 9–69 → 70) — so decoding
  `{"nbt":"a","entity":"@x"}` FAILS in vanilla 26.2, the opposite of
  1.21.7 (where `compileSelector` caught the exception and returned
  null). The Rust `"nbt"` arm's selector check (`component.rs`, unchanged
  from 1.21.7) rejects it too — match.
  (b) Vanilla 26.2 HAS `ObjectContents`
  (`network/chat/contents/ObjectContents.class`, with `AtlasSprite` and
  `PlayerSprite` in `contents/objects/`) — the Rust `object` arm is
  correct for 26.2.
- **B9-G09 — CLOSED (both parts).** Vanilla 26.2 refactored the selector
  parser to the option-state shape the Rust code already implements
  (`InvertableSetOptionState`, `SetOnceOptionState`). (a) Duplicate
  options: the `canUse` gate now runs in `EntitySelectorOptions.get()`
  BEFORE the value is read, rolling back to the option-name start
  (`parseOptions` offsets 50–65; `rollbackAndThrow` to the saved start) —
  exactly the Rust behaviour (`selector.rs` `parse_options` checks
  `can_use` before `=` and rolls back to `start`). The 1.21.7 divergence
  (vanilla at end of value) came from the old parser's in-handler check.
  (b) `@a[type=player]`: `limitToType(PLAYER)` sets only the `type`
  field, never the option state, and the `type` option's `canUse` is
  `typeOption().canParseAny()` (state != SINGLE — bootstrap #31 →
  `lambda$bootStrap$41`), so vanilla 26.2 ACCEPTS it; the handler then
  succeeds (resolves `player`, `limitToType` no-op, marks SINGLE). Rust
  accepts it too (`selector.rs:215-242` pushes `minecraft:player` without
  marking the state; `handle("type")` replaces it). Both sides agree on
  acceptance — the 1.21.7 rejection is gone in 26.2.
- **B9-G10 — CLOSED (inverted).** Vanilla 26.2's `WorldBorderCommand`
  uses `TimeArgument.time(0)` for the `add`/`set` time argument (javap:
  2 call sites), not `IntegerArgumentType`. Rust
  (`tree_world.rs:318`, `ArgType::Time { min: 0 }`) matches — including
  the `d`/`s`/`t` suggestions, which are correct for a Time argument.
- **B9-G13 — CLOSED.** Vanilla 26.2's `ArgumentTypeInfos` bootstrap
  registers 57 types in exactly the Rust `ARGUMENT_TYPES` order
  (`horizon-protocol/src/packets/play/commands.rs:25-82`), and index 16
  is `team_color` in vanilla 26.2 (17th `register` call) — the name the
  1.21.7 batch flagged as 26.x carry-over is correct here.

**Group B — the 26.2 port fixed the defect (Rust differs from 1.21.7):**

- **B9-G03 — CLOSED.** Rust 26.2 uses
  `ArgType::ResKey("minecraft:worldgen/configured_feature")`
  (`tree_world.rs:65`; comment notes inline feature values are 26.3-only),
  matching vanilla 26.2's `PlaceCommand`
  (`ResourceKeyArgument.key(Registries.CONFIGURED_FEATURE)`).
- **B9-G05 — CLOSED.** Rust 26.2 `/publish` is
  `publish [<allowCommands> [<gamemode> [<port>]]]`
  (`tree_extra.rs:60-67`), matching vanilla 26.2's `PublishCommand`
  (`BoolArgumentType`, `GameModeArgument.gameMode`,
  `IntegerArgumentType(0, 65535)`).
- **B9-G07 — CLOSED (both parts).** (a) `codec.rs:204-212` dispatches
  `minecraft:item_modifier` on the `"function"` key
  (`LootItemFunctions.ROOT_CODEC` form), with a unit test proving
  `{"function":"minecraft:set_count","count":1}` decodes; vanilla 26.2
  loot data confirms the key (client jar
  `data/minecraft/loot_table/entities/zombie.json`:
  `"function": "minecraft:set_count"`). (b) `/item … from entity
  <source>` uses `ArgType::Entity { single: true, … }`
  (`tree_misc.rs:203-210`), matching vanilla 26.2's `ItemCommands`
  (`EntityArgument.entity()` for `source`, 2 call sites;
  `entities()` for `targets`).
- **B9-G08 — CLOSED.** Rust 26.2 `particle_options` (`args.rs:1054-1058`)
  accepts the compound form with a `Name` key; vanilla 26.2's
  `StateHolder.codec` dispatches on `"Name"` (javap:
  `Codec.dispatch("Name", …)`) — the `{Name, Properties}` map form.
- **B9-G12 — CLOSED.** The 26.3-only `SlotSource` argument type (whose
  SNBT fallback produced the 1.21.7 divergences) is gone from the 26.2
  tree: `execute if items` uses `ArgType::Slots`
  (`tree_execute.rs:239`), `/item` uses `ArgType::Slot`
  (`tree_misc.rs:176,192,213`). The Slot/Slots parse matches vanilla
  26.2's `SlotArgument` exactly: read-until-space
  (`lambda$parse$0`: `c != ' '`), `slot.unknown` /
  `slot.only_single_allowed` at the cursor after the word (same keys in
  the 26.2 `en_us.json`: "Unknown slot '%s'",
  "Only single slots allowed: got '%s'"), and the `slot_ranges()` table
  matches vanilla 26.2's `SlotRanges` entry for entry (container.×54,
  hotbar.×9, inventory.×27, enderchest.×27, mob.inventory.×8, horse.×15,
  weapon.*×2, armor.*×5, player.crafting.×4, singles at 98/99/100–103/
  105/106/499).

**Group C — the 26.x form was correct for 26.2 all along / masked:**

- **B9-G02 — CLOSED as stated.** Full top-level diff of the Rust release
  tree (77 literals from `tree_*.rs` + `vanilla.rs`, dev-gated
  `raid`/`warden_spawn_tracker`, Horizon-gated `pregen`, and
  dedicated-only commands excluded) against vanilla 26.2's dispatcher
  dump (datagen `commands.json`, 92 top-level; 77 integrated after
  removing the 15 dedicated-only per `Commands.<init>`): **exact set
  match, zero extras, zero missing.** `stopwatch`, `swing`,
  `fetchprofile`, `unpublish` are all in vanilla 26.2's tree;
  `/execute if|unless stopwatch` is in vanilla's tree; the `/time`
  clock subcommands (`pause`, `resume`, `rate`, `of <clock>`,
  `set <timemarker>`, `query <timeline>`) are all in vanilla 26.2's
  `/time` node, and the Rust `clock_nodes` (`vanilla.rs`) matches it node
  for node (including `query gametime` only on the default clock, not
  under `of`). `compute` and `posteffect` are gone from the Rust tree
  entirely (no occurrences in `horizon-command/src`). The only tree gap
  found is the dedicated-tree `/perf` (B9-G14, §3.1). Note: vanilla
  registers `jfr` only when the JVM profiler is available
  (`JvmProfiler.isAvailable()`); Rust registers it unconditionally —
  effectively always available on a normal JVM, not recorded as a defect.
- **B9-G11 — CLOSED.** The recipe instance is gone on both sides:
  vanilla 26.2 has no `minecraft:recipe` registry (absent from datagen
  `registries.json`; recipes live in the `RecipeManager`), so
  `ResourceKeyArgument.listSuggestions` → `SharedSuggestionProvider`
  finds nothing to suggest; Rust suggests from
  `reg.ids("minecraft:recipe")`, also empty (`args.rs:1744-1748`). The
  suggestion mechanism itself (`lib.rs:suggestion_context`, byte-identical
  to 1.21.7) mirrors Brigadier 1.3.10's `findSuggestionContext`
  (verified against the brigadier jar in `.local/datagen/libraries`):
  same inclusive-end range test, same parent-plus-start return — and the
  1.21.7 corpus proved the mechanism does not generally re-suggest (only
  16 suggestion mismatches in 96,963 parses, 12 of them the G10 Time
  symptom).

### Passes (verified, not re-diagnosed)

- **`/time set day|noon|night|midnight` — PASS in inverted form.**
  Vanilla 26.2's `/time set` has only the `time` and `timemarker`
  arguments (datagen `commands.json`) — the day/noon/night/midnight
  literals do not exist in 26.2 (they went with the clock rework). The
  26.2 Rust `clock_nodes` has none either (no matches for
  day/noon/night/midnight in `horizon-command/src`). Both sides agree:
  `time set day` parses only via the `timemarker` (resource-location)
  argument on both. The 1.21.7 pass ("literals present") does not
  transfer as stated; the 26.2 check is the matching absence, and it
  holds.
- **57 argument-type list — PASS.** Count 57, order identical to vanilla
  26.2's `ArgumentTypeInfos` registration order (javap static
  initializer), names identical including `minecraft:team_color` at
  index 16 (see B9-G13 above).
- **Error-text resolution at runtime — PASS.** The game resolves command
  error keys correctly (see B9-O04, §4).

## 4. Oracle / fixture defects

### B9-O02 — minor — `VanillaTeamRulesOracle` fails javac (2 client errors) — blocks `team_rules`

Batch 0's 26.2 measurement
(`versions/26.2/.local/measure-oracles/results.json`): client cp
`constructor-mismatch`, 2 errors (down from 19 in 1.21.7). The server-cp
failure (41 errors, `net.minecraft.client` absent) is expected — the
driver uses the client jar, so the client cp is the operative one.

- Line 101 (`Viewer()`): `super(null, null, null, null, null, null,
  false, null, null)` — 9 arguments. 26.2's `LocalPlayer` constructor
  takes 8: `(Minecraft, ClientLevel, ClientPacketListener, StatsCounter,
  ClientRecipeBook, Input, boolean, ChatAbilities)` (javap on the 26.2
  client jar). Drop one argument to match.
- Line 197: `own.doTeamsAllowDamage(other)` — no such method in 26.2
  (`Entity` has no `doTeamsAllowDamage`; the 26.2 form is
  `PlayerTeam.isAllowFriendlyFire()`). Rework the damage check against
  the team's friendly-fire flag.

Repair scope: small, in-family, NOT attempted (no fixes in this batch).
Needs an end-to-end driver run after repair (compiling ≠ running).

### B9-O03 — minor — `LocatorBarClient` fails javac (10 errors) — blocks `exploration_map_command`

Batch 0's 26.2 measurement: fails both classpaths, `missing-symbol`,
10 errors (down from 20 in 1.21.7). Causes, verified against the 26.2
client jar:

- Lines 393–395, 407, 409, 413: `packet.removed()` / `packet.added()` /
  `packet.progress()` on `ClientboundUpdateAdvancementsPacket` — 26.2
  uses getter names: `getRemoved()`, `getAdded()`, `getProgress()`
  (`shouldReset()` exists as-is, so line 405 is fine). Six call sites to
  rename.
- Line 668: `canUseCommandBlocks()` override — no such method on the
  supertype in 26.2. Drop or rework the override.
- Line 690: `removed.entityIds()` on `ClientboundRemoveEntitiesPacket`
  — 26.2's accessor is `getEntityIds()`.

Repair scope: per-symbol renames against the 26.2 client jar (the three
named fixes cover 8 of the 10 errors; the remaining 2 are unlisted in
the results file). Medium; NOT attempted — and out of family for a
commands batch (packet-client helper shared with the locator-bar
harness).

### Closed oracle / fixture defects

- **B9-O01 — CLOSED.** `VanillaScoreboardOracle` compiles clean on BOTH
  classpaths in Batch 0's 26.2 measurement (26.x has `TeamColor` and the
  `Scoreboard` packing methods the 1.21.7 oracle lacked). The
  `scoreboard` driver is unblocked at the oracle level; it still needs
  its driver run (§5).
- **B9-O04 — CLOSED.** The `command_parse` fixture loads language from
  the prepared-resources root (`command_parse_fixture.rs:147`,
  `lang::Language::load(&[&root], …)`; root = `lab.RESOURCES` =
  `.local/horizon-client/26.2/`). Unlike 1.21.7, the prepared 26.2 root
  DOES contain `assets/minecraft/lang/en_us.json` — md5-identical to
  the 26.2 client jar's file (8,123 keys, including every command error
  key the driver compares). The fixture's error-text leg is readable for
  26.2; the 86,241-mismatch artifact of the 1.21.7 run does not recur
  (confirmed by the §2 run: **0 error-text mismatches** across 103,743
  parses, 0 mismatches of any kind).
  Note: the coordinator's mid-batch update said lang files were still
  absent; direct verification shows `en_us.json` present and
  jar-identical (only `sounds.json` is genuinely absent, and no command
  check needs it).

## 5. Driver table (22)

| Driver | Oracle | Status |
|---|---|---|
| command_parse | VanillaCommandParseOracle (pass) | **Measured by this batch** (§2) |
| command_exec | VanillaCommandExecOracle (pass) | Not run. Needs full game build + Java integrated server. Known fixture blocker (same as 1.21.7): `/execute if predicate effect_view:always` — the `effect_view:*` predicates come from `tools/mob_effect_predicate_corpus.py`, imported by `command_exec_corpus.py:64`. Recorded, not chased. |
| scoreboard | VanillaScoreboardOracle (pass) | Not run — oracle now compiles (B9-O01 closed); needs its driver run (probe-based, no game-build cost). Cheapest remaining driver in the batch. |
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
plus a Java run; none was attempted, and each is a 10–30+ min job on
this VM even unblocked. `command_exec` is the linchpin: 7 of the 17
no-oracle drivers import it or shell out to it, so its fixture blocker
gates the family.

## 6. Method notes

- **Run before writing** was honoured: the one measurement available
  (`command_parse`, §2) was run and triaged, and the static checks were
  designed around it. Everything else is method 2 (ledger-item pattern:
  javap + source + oracle data), because no other driver could run.
- **Vanilla 26.2's own jar/data is the spec throughout.** Every
  behaviour claim above is grounded in one of: the oracle's exports
  (`commands.json`, `command-registries.json`, `tree.json`,
  `registries.json` — real vanilla 26.2 dispatcher output), `javap -c`
  on the named 26.2 server-game jar (Mojang-mapped, so no mappings
  front-end was needed) or the 26.2 client jar, the vanilla 26.2 datagen
  reports, or Brigadier 1.3.10 from the datagen libraries. The 26.3
  base's form was never trusted — and this batch is the mirror image of
  the 1.21.7 one: the dominant pattern is 26.x forms that ARE correct
  for 26.2 (gamerules, Rotation spawnpoints, Time worldborder,
  ObjectContents, `team_color`, the selector option-state refactor).
  The 26.2 port also fixed five 1.21.7 defects outright (G03, G05, G07,
  G08, G12) and removed the 26.3-only tree nodes (`compute`,
  `posteffect`, `SlotSource`).
- **Systemic pattern**: the 26.2 command surface is in markedly better
  shape than 1.21.7's was: 13 of 13 re-checked defects closed, 3 new
  minor defects found (two tree-completeness, neither in the release
  integrated tree, plus the argument-name labels), and the full
  `command_parse` corpus matches vanilla exactly (0 mismatches in
  103,743 parses). The remaining risk is execution, not parsing: 19 of
  22 drivers are unrun e2e drivers, and `command_parse` only covers
  parse behaviour.
- **What the corpus cannot see**: it walks vanilla's tree, so
  Rust-only extras appear only via hand-written edge cases (the 26.2
  corpus carries `swing`, `stopwatch`, `fetchprofile`, `unpublish`,
  `compute`, `raid start 5`, `publish true survival 25565`, `execute if
  stopwatch …`). The tree diff (§3, B9-G02) is the complete check for
  extras; the corpus is the complete check for missing/wrong vanilla
  behaviour. The dedicated tree (`/perf`) and the dev tree are outside
  both: the corpus walks the integrated tree, and the dev tree is not
  built by `release_dispatcher()` — B9-G14/G15 were caught by the static
  diff against `Commands.<init>` bytecode, not by any run.
- **Counts**: corpus 34,581 commands × 3 permission levels = 103,743
  parses; **0 mismatches** (perm 4: 34,581/34,581 match, 5,577 parse-OK;
  perm 2: 34,581/34,581, 5,521 OK; perm 0: 34,581/34,581, 72 OK). The
  run's only finding is the 14 node-path label differences (B9-G16),
  which the driver's compare excludes. No mismatch collapse was needed:
  there was nothing to collapse.

## 7. Open questions / follow-ups for the coordinator

1. The 19 unrun e2e drivers (esp. `command_exec` + its 7 dependents)
   are the batch's real remainder: they need a full client build and the
   `effect_view:always` fixture blocker resolved. Recommend running
   `command_exec` first (with `--allow-gaps`), then its dependents.
2. `scoreboard` is the cheapest remaining driver: its oracle compiles
   clean for 26.2 (B9-O01 closed) and the driver is probe-based — no
   game build needed.
3. B9-O02 (`team_rules`) is a small repair (2 client errors, fully
   scoped in §4) — a good first oracle repair; it unblocks a driver
   with no game-build cost.
4. B9-G14 (`/perf` in the dedicated tree) and B9-G15 (dev tree) are
   both tree-completeness gaps found by the static diff; neither is
   visible to the `command_parse` corpus (integrated tree only). If
   dev-tree parity is out of scope, G15 can be withdrawn.
5. B9-G16 (argument node names) is a five-name rename in
   `tree_misc.rs`/`tree_execute.rs`, cosmetic (network-form labels
   only). The 1.21.7 tree shares the names, so a fix could be carried
   across versions — but the 1.21.7 batch did not record it, so it is
   this batch's finding to carry.
6. The `/time` clock nodes, `stopwatch`/`swing`/`compute`/`posteffect`
   history is now resolved per version: 26.2's tree matches vanilla
   26.2 exactly at the top level. The same tree-diff method (Rust
   literals vs the version's `commands.json`, with the
   dedicated/integrated split read from `Commands.<init>` bytecode)
   applies cheaply to the E6 siblings and 26.1.x.

## Coordinator note

Reconciled Nov 24, 2026 against the measured Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`), the 26.2 tree/jars, and the
command_parse run's own artifacts
(`versions/26.2/.local/command-parse-parity/`):

- **command_parse run verified from the primary artifacts:** report.json
  reads perm 4/2/0 × 34,581 commands, 0 mismatches each (103,743 parses);
  corpus.txt has 34,581 lines; perm4/perm2/perm0 `mismatches.jsonl` files
  are empty; OK counts re-derived independently from the jsonl outputs
  (perm 4: 5,577, perm 2: 5,521, perm 0: 72 — rust == vanilla in all
  three); compile-rust.log shows `Finished 'fast' profile in 35m 04s`;
  the oracle compiled with only unchecked warnings. This is the sweep's
  strongest single verification.
- **B9-G16 verified by an independent node diff** over the run's
  rust.jsonl/vanilla.jsonl: 14 distinct inputs with node differences at
  perm 4 and perm 2 (13 `/item replace`, 1 `/execute if items entity`),
  4 label pairs (`pos`→`targetPos`, `targets`→`target`,
  `source`→`sourcePos`, `entities`→`source`), 17 occurrences; 0 at perm 0.
  The agent's "14 label differences" counts distinct inputs — confirmed.
- **B9-G14/G15 verified on both sides:** the port's dedicated tree has 14
  root literals (`register_dedicated`, no `perf`); `PerfCommand` exists in
  the 26.2 server-game jar. None of the six dev commands appears anywhere
  in `horizon-command/src`; all six classes (`DebugPathCommand`,
  `DebugMobSpawningCommand`, `SpawnArmorTrimsCommand`,
  `ServerPackCommand`, `DebugConfigCommand`, `ChaseCommand`) exist in the
  jar. **Scope decision:** B9-G15 is kept as a minor gap item — it is a
  verified divergence, and withdrawing it unilaterally would hide a
  finding; ruling dev-tree parity out of scope is for the fix decision,
  not the sweep.
- **Oracle claims verify:** VanillaTeamRulesOracle fails all three
  classpaths, constructor-mismatch, 2 errors on the client view (the
  Batch 0 table's n_errors is the client view; results.json confirms the
  server cp fails with 41 missing-package errors, as the report states);
  LocatorBarClient fails all three, missing-symbol, 10 errors;
  VanillaScoreboardOracle and VanillaCommandParseOracle pass all three.
- **Cross-batch overlap — dedup at merge:** B9-O03 (LocatorBarClient) is
  the SAME oracle defect as the LocatorBarClient member of B3-O04
  (10 errors); B3-O04 is canonical, B9-O03 becomes a cross-reference.
  B9-O02 (TeamRules) is a first per-batch record.
- **Lang correction (sweep-wide):** the agent is right — prepared
  `assets/minecraft/lang/en_us.json` IS present and md5-identical to the
  26.2 client jar's file (c7e1a5ca564d3afab2b4d025d0ac5298, 8,123 keys
  both), verified by the coordinator. This corrects the "lang files
  absent" statements in the Batch 0, Batch 3, Batch 5 and Batch 10 notes
  (prepare DOES import lang from the client jar; only `sounds.json` is
  genuinely absent, which needs the separate sounds step). B10-G02 (no
  sounds.json) is unaffected. The earlier batch reports are committed
  as-is; this correction is carried in the merged summary.
- **Build note:** the full command_parse driver run (fast-profile
  `horizon-client-lib` build, `-j 1`, `taskset -c 0`,
  `RUSTFLAGS=-C codegen-units=1`) went beyond the brief's no-builds
  rule, on the coordinator's direction to run command_parse rather than
  record B9-H. No OOM, no interference beyond one brief SIGSTOP to let
  another agent's check finish; the run survived a SIGTERM of the first
  attempt and a VM reboot mid-build. The report.json is preserved in the
  batch scratch dir (`batch-09/command-parse-report.json`) ahead of
  `.local` cleanup.
- **Tally arithmetic verified:** 13 re-checks (all pass) + 2 re-verified
  1.21.7 passes + 1 command_parse measurement (pass) + 5 gaps = 21
  checks; 16 pass, 5 gap. Driver table: 1 measured + 2 oracle-blocked +
  19 not run = 22. No corrections needed.
