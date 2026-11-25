# Batch 9 — Commands (1.21.11 parity sweep)

Date: 2026-11-25. Branch: `parity-sweep-1.21.11` (Batch 0's oracle
measurement landed in `docs/sweep/1.21.11/batch-0-oracles.json` and
`batch-0-measurement.md` — §4 cites the measured table directly).
Scope: parsing, execution, selectors, functions, scoreboard — 22 drivers;
57 argument types (1.21.11's own list), 59 game rules (data-defined,
`minecraft:game_rule` registry).
Ledger slice: `slice.json` (this directory).

## 0. Execution status — read first

**The `command_parse` run MEASURED.** 34,492-command corpus
(md5 `9e9597f33c9c5bc9e2112f0bda481720`); perm 4: 3,172 mismatches
(90.8% match), perm 2: 3,158, perm 0: 1 (the root suggestion list —
the three extra commands). Full results in §2.

Everything else in this report is static triage (method 2: javap +
source + oracle data + datagen), and it stands on its own: the
re-checks below were all verified without the run, and the run's
mismatch classification (§2) confirms them empirically where the
corpus reaches them.

**Headline: the 1.21.11 template's command surface is a 26.3-era tree
carried against a 1.21.11 vanilla that predates it in some places and
matches it in others — and the template is CLEANER than its first
child in the places vanilla changed.** The 1.21.10 child fixed things
the template did not (the gamerule persistence mapping, the
`/worldborder` integer time), and diverged from the template in one
place the other way (the child ADDED `/swing` to its tree; the
template never had it — and vanilla 1.21.11 has no `/swing` either),
and vanilla changed between 1.21.10 and 1.21.11 in the template's
favour in three places: the `game_rule` registry arrived with
snake_case ids (G01 withdrawn — the port's table now matches vanilla
exactly, 59 = 59 names, defaults match), the nautilus custom stat
arrived (G21 halved — only `sleep_in_straw_bed` is extra now), and
`Action.STAB` arrived (O03 down to 17 errors). The template's own
defects are the familiar 26.3 carry-overs: three extra top-level
commands (`compute`, `posteffect`, `unpublish` — `swing` is absent
from BOTH sides here), the 26.3 62-entry argument-type table against
vanilla's 57, SlotSource in place of `item_slot`/`item_slots`, and
1.21.7-era parse code (particle/modifier handling, selector rollback)
that the 26.2 port fixed but this tree never carried. Of the 27
primary 1.21.10 items, 16 reproduce (G02, G03, G05, G06-partial, G07,
G08, G09, G12, G13, G14, G15, G16, G17, G18, G20, G21-partial),
3 are oracle gaps in NEW forms (O01 3 errors, O02 6 client errors,
O03 17 errors — all the 26.1.2 forms, not 1.21.10's 1.21.7 forms),
and 8 are closed/fixed (G01 withdrawn by the game_rule registry,
G04 closed by the Rotation form, G10 closed — vanilla 1.21.11 uses
the Time argument the tree ships, G11 closed against real vanilla,
G19 doubly withdrawn — no `/swing` on either side, O04 closed —
prepared data HAS the lang file, jar-identical, O05 fixed — the
`VanillaCommandExecOracle` compiles clean on all three classpaths,
H01 fixed — the probe compiles). Of the 17 secondary 1.21.7 items,
the verdicts track the 1.21.10 run's except where 1.21.11's vanilla
differs: G01 flips from gap to withdrawn (snake_case is now
vanilla's form), G10 flips from gap to closed (Time is now vanilla's
form), and O04 flips from gap to closed (the lang file is present).
The run adds one NEW item (B9-G22, the container component). The
tally: 25 as-tallied (7 pass, 18 gap) = 21 behaviour (6 pass,
15 gap) + 4 non-behaviour (1 pass, 3 gap: H01 + O01/O02/O03). The
21st behaviour check is `/stopwatch` — new in 1.21.11, verified
statically and empirically (0 mismatches in 1,870 cases); the
1.21.10 scheme had 20 behaviour checks, and the extra pass is
stated here so the count is auditable.

Two corrections to the brief, both verified against 1.21.11's own
tree (§6): (1) the brief's "1.21.10 corpus + scripts are preserved"
framing understates the tree delta — the 1.21.11 tree's own corpus
generator carries EXTRA cases the 1.21.10 adaptation dropped
(stopwatch, `execute if stopwatch`) because vanilla 1.21.11 HAS
`/stopwatch` (integrated, `StopwatchCommand` in the jar) — those
cases were KEPT, not dropped. (2) The brief said prepared
horizon-client data "does NOT exist anywhere on the VM — you must
build it (run a prepare)"; by the time this batch started, Batch 12
was already running a prepare into the shared worktree `.local` — it
finished EXIT=0 (1,373.9 s) with the lang file included, so this
batch used the shared prepared data instead of running a second,
conflicting prepare.

Prepared data used: `versions/1.21.11/.local/horizon-client/1.21.11/`
(built by Batch 12's prepare run, EXIT=0 Nov 25), including
`commands.json`, `command-registries.json`
(VanillaCommandParseOracle export — identical keys to this batch's
own oracle export, 1,458 learnable recipes), `registries.json`, and
`assets/minecraft/lang/en_us.json` (md5
`1460272285b8f2671c56c538113d1308`, jar-identical, 7,767 keys,
contains `argument.entity.invalid`) — so NO lang mitigation was
needed (B9-O04 closed). All behaviour claims are grounded in
1.21.11's OWN jar/data — the named server-game jar
(`e6-deltas/versions/1.21.11/jars/server-game-named.jar`, sha1
7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2, Mojang-mapped), the 1.21.11
client jar (`client-named.jar`, sha1
29bddf144a2f2907528b25d616770bfea67c4792), the vanilla 1.21.11
datagen reports, or the oracle's own exports. The 26.3 base's form
was never trusted. (The `server.jar` in the same directory is a
Mojang bundler jar, sha1 64bb6d763bed0a9f1d632ec347938594144943ed —
never javap'd.) DataVersion 4671, protocol 774, Java 21.

## 1. Scope

Parsing, execution, selectors, functions, scoreboard — 22 drivers
(§5); 57 argument types (1.21.11's own
`ArgumentTypeInfos` registration order, verified by javap — §3,
B9-G13); 59 game rules (the `minecraft:game_rule` registry,
snake_case ids, verified by javap + the oracle's `by_registry`
export — §3, B9-G01); 75 integrated commands in vanilla 1.21.11's
tree (oracle export — 74 in 1.21.10, + `/stopwatch`); 15 dedicated
commands; 7 dev commands (+ `chase` under its own flag).

## 2. The command_parse run (measured)

**Measured end-to-end.** Driver:
`tools/test_command_parse_parity.py --allow-gaps` (byte-identical
copy in `b09/driver/`), corpus regenerated from 1.21.11's own tree
export: **34,492 commands, md5
`9e9597f33c9c5bc9e2112f0bda481720`** (the preserved corpus md5
matches the preview exactly), 492 gamerule cases, 1,870 stopwatch
cases, 0 swing cases. Fast-profile `client_probe` fixture — compiled
AND linked (`Finished` in 52m00s). The first build was interrupted
by a VM reboot at ~03:10 UTC (uptime reset; 232 crates done,
`horizon-protocol` mid-compile); the relaunch resumed from the warm
batch-local cargo cache and finished clean. The oracle compiled
clean (`VanillaCommandParseOracle`, all three classpaths — Batch 0).
Prepared data: the shared Batch 12 prepare (EXIT=0), lang file
present. Run artifacts preserved in `run-artifacts/` (`report.json`,
`corpus.txt`, `registries.json`, `tree.json`, per-perm
`mismatches.jsonl` + `node-diffs.json` + `rust.jsonl` +
`vanilla.jsonl`, compile logs, `driver.log`, `classify.txt`,
`node-diff.txt`).

Results (`run-artifacts/report.json`):

| Permission | Commands | Mismatches | Match |
|---|---|---|---|
| 4 | 34,492 | 3,172 | 31,320 (90.8%) |
| 2 | 34,492 | 3,158 | 31,334 (90.8%) |
| 0 | 34,492 | 1 | 34,491 (99.997%) |

Perm 2 differs from perm 4 only by the perm-4-only `/publish`
cases (14). Perm 0's single mismatch is the empty-input root
suggestion list (vanilla 75 names, Rust 78 — the three extras, G02).

Classification (perm 4; `run-artifacts/classify.txt`):

| Class | Perm 4 | Perm 2 | Perm 0 |
|---|---|---|---|
| error-text-only | 2,859 | 2,858 | 0 |
| suggestions | 236 | 235 | 1 |
| cursor-differs | 52 | 45 | 0 |
| ok-differs | 25 | 20 | 0 |

- **The error-text-only bulk (2,859) is one cause: G12's slot
  error text.** Vanilla reports `Unknown slot '0'`; Rust reports
  `Can't find element 'minecraft:0' in registry
  'minecraft:slot_source'` — same cursor throughout (2,803 in
  `/execute` + 52 in `/item` + 3 in `/place` + 1 in `/publish`).
  The Rust error text itself names the non-existent registry.
- **Suggestions (236):** `/execute` `slots` extra 200 (G18 — every
  one is the `slots` literal, vanilla has only `items`);
  `/item` 10 (`fill`/`override` in the subcommand list + wildcard
  slot forms `armor.*`/`container.*` that vanilla's `item_slot`
  never suggests — G18 + G12); `/data` 17 (`compute` in the
  modify-source list — G02); `/recipe` 3 (the G17 artifacts, see
  below); `/scoreboard` 3 (G21 — the only difference is the extra
  `minecraft.custom:minecraft.sleep_in_straw_bed`); root list 1
  (G02); `/place feature` 1 (G03 — vanilla suggests
  configured_feature ids, Rust suggests feature ids); `/publish` 1
  (G05 — vanilla suggests the four gamemodes, Rust suggests
  nothing).
- **Cursor-differs (52):** `/advancement` 22 — all
  `@a[type=player]` (G09b: vanilla cursor 25, `Option 'type' isn't
  applicable here`; Rust cursor 33, fails later);
  `/place` 7 (G03 — Rust fails at the feature id, vanilla parses
  it and fails later); `/give` 7 (G22 container — vanilla reports
  the malformed component at cursor 40, Rust fails later);
  `/publish` 7 (G05); `/execute` 4 (G12 — `@@bad@@` slot: vanilla
  `Unknown slot '@@bad@@'` at the slot end, Rust `Expected a valid
  unquoted string` earlier); `/bossbar` 2 (G06a trailing-token
  variants); `/item` 2 (G12 — an empty slot, and vanilla's `Only
  single slots allowed: got 'armor.*'` where Rust accepts the
  wildcard); `/particle` 1 (G08 trailing-space variant — Rust
  cursor −1, the particle-options error carries no cursor).
- **Ok-differs (25):** `/place` 11 (G03 — vanilla accepts
  `minecraft:acacia`, `minecraft:ore_diamond_buried`, etc. as
  configured features; Rust rejects them from
  `minecraft:worldgen/feature`); `/publish` 5 (G05 — `publish true
  survival` parses in vanilla, `Expected integer` in Rust);
  `/item` 3 (G07a ×2 — `{"function":"minecraft:set_count",...}`
  parses in vanilla, `No key type` in Rust; G07b ×1 —
  `/item … from entity @a …` parses in Rust, vanilla `Only one
  entity is allowed`); `/give` 3 (G22 container); `/bossbar` 2
  (G06a — `{"nbt":"a","entity":"@x"}` parses in vanilla, fails in
  Rust); `/particle` 1 (G08 — `block_state:{Name:"minecraft:stone"}`
  parses in vanilla, `No key id` in Rust).

**Oracle crashes skipped by `compare()` (not counted as
mismatches): 14 at perm 4 and perm 2, 13 at perm 0.** All `random`
NPEs (`RandomSequences.forAllSequences` — the stand-in server has no
random sequences) plus `schedule clear` NPEs
(`WorldData.overworldData()` — no world data in the stand-in
server). Same 14 as 1.21.10. Consequence: **G20 is NOT empirically
confirmed by the run** — every `schedule clear` input crashes the
oracle before a parse result exists, so the greedy-string-vs-id
difference rests on static evidence alone. The `random` crashes are
oracle-fixture noise, not port defects (the Rust side parses all of
them).

**Recipe suggestion mismatches are artifacts, as predicted (§3,
B9-G11/B9-G17):** the oracle's `Src` override bypasses vanilla's
RECIPE special case, so the oracle suggests only `["*"]` while Rust
suggests `*` + the 1,458 learnable recipes. Do not read them as port
defects.

**Zero-mismatch confirmations:** `/gamerule` 0 mismatches across
492 cases (G01 withdrawn — the snake_case table matches vanilla);
`/stopwatch` 0 mismatches across 1,870 cases (new pass — the
command matches vanilla in shape and suggestions);
`/schedule` 0 mismatches on the non-`clear` subcommands.

**Raw error text: zero raw-key emissions** — every error in the
run's `rust.jsonl` is translated text (lang file present — O04
closed empirically).

**Node diff (`run-artifacts/node-diff.txt`, `node_diff.py`): 37
inputs with node differences at perm 4 out of 34,492** — 29
label-pair occurrences across six pairs (`slot`→`slots` 11,
`targets`→`target` 6, `pos`→`targetPos` 5,
`sourceSlot`→`sourceSlots` 4, `source`→`sourcePos` 2,
`entities`→`source` 1) plus shape diffs where one side fails to
parse (place feature 11, publish gamemode 4, bossbar nbt 2, item
modifier 2, particle 1, give container 3, item multi-entity 1,
publish port 1). The ninth G16 pair (`/execute if items block`
`pos`→`sourcePos`) is masked in the node diff because the slot
argument fails first (G12) — its static evidence stands.

**New run-found item: B9-G22** — the `minecraft:container`
component accepts entries without `slot`/`item` (10 `give`
mismatches: 3 ok-differs + 7 cursor-differs). See §3.1.

## 3. Defects

### 3.1 Open defects

### B9-G02 — major — Three 26.x commands in the release tree; vanilla 1.21.11 has none of them

REPRODUCES — three extras here, not 1.21.10's four: `swing` is absent
from BOTH sides in 1.21.11 (see B9-G19). The full top-level diff
(§3.2, G02 bulk) shows the Rust release tree (78 literals) against
vanilla 1.21.11's integrated tree (75 commands, from the oracle's own
INTEGRATED tree export): exact match except THREE extras — `compute`,
`posteffect`, `unpublish`:

- `posteffect`: no `PostEffectCommand` class in the 1.21.11
  server-game jar, no `posteffect` in the oracle's tree export, no
  `minecraft:post_effects` registry in 1.21.11's `by_registry` — so
  the tree's `ArgType::Id("post_effects")` arguments
  (`tree_extra.rs:40,50`) reference a registry that does not exist.
  Registered unconditionally in `tree_extra::register`
  (`tree_extra.rs:36-53`).
- `unpublish`: no `UnpublishCommand` class in the 1.21.11 jar and no
  `unpublish` in the tree export (`unpublish` arrived in 26.2).
  Registered unconditionally (`tree_extra.rs:83-85`, under the
  `// PublishCommand / UnpublishCommand` comment).
- `compute`: no `ComputeCommand` class in the 1.21.11 jar and no
  `compute` in the tree export. Registered unconditionally
  (`tree_data.rs:51-69`, `compute (default | block <pos> | entity
  <target>) (float <provider> [<scale>] | integer <provider>)`).
  The `/data modify … compute` source is also present (the shared
  `context_sources` helper, `tree_data.rs:32-48`) — counted here per
  the 26.1.2 precedent, not as a new item.

All three are gamemaster-only, but they are real divergences in the
release tree: on each, Rust parses and vanilla reports unknown
command. The corpus carries NO parse cases for the three extra
commands themselves (they were dropped from EXTRA per the brief's
"drop the 26.x-only commands" — §6); their only run footprint is the
empty-input root suggestion list at all three permission levels
(vanilla 75 names, Rust 78 — the three extras, §2). The tree diff
in this section is the complete check for them.

### B9-G03 — minor — `/place feature` uses the wrong registry and accepts inline values

REPRODUCES — same as 1.21.10. Vanilla 1.21.11's `/place feature` takes
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

REPRODUCES — same as 1.21.10. Vanilla 1.21.11's `/publish` is
`[<allowCommands> [<gamemode> [<port>]]]` (`Publish.javap`:
`BoolArgumentType` + `GameModeArgument` + `IntegerArgumentType`;
the oracle's tree export: `allowCommands` → `gamemode` → `port`,
integer 0–65535, required level 4). Rust is `[<allowCommands>
[<port>]]` (`tree_extra.rs:76-82`) — the gamemode node is missing
entirely, so `/publish true survival` fails to parse in Rust and
`/publish true 25565` binds the port where vanilla would read a
gamemode first.

### B9-G06 — minor — Item-component NBT with a bad selector decodes in vanilla, fails in Rust (partial)

REPRODUCES on part (a); part (b) is CLOSED. (a) Vanilla 1.21.11's
`EntityDataSource.compileSelector` catches `CommandSyntaxException`
and returns null (`EntityDataSource.javap`:
`net.minecraft.network.chat.contents.data.EntityDataSource` — moved
into a `data` subpackage in 1.21.11; exception table 0–21 → 22,
`aconst_null` at offset 23) — the 1.21.7 form — so
`{"nbt":"a","entity":"@x"}` decodes in vanilla. Rust's nbt arm
requires `selector_string` to parse (`component.rs:278-299`), so the
same component fails. (b) Vanilla 1.21.11 HAS `ObjectContents` (both
jars, with `AtlasSprite` + `PlayerSprite`), and the Rust `object` arm
(`component.rs:297-309`, `atlas` + `player` kinds) matches it, so
that half of the 1.21.7 item does not reproduce. One check, gap on
part (a).

### B9-G07 — minor — Item modifiers: inline JSON fails (wrong type key) AND multi-entity source

REPRODUCES in full — same as 1.21.10. (a) `codec.rs dispatch()` reads
`"type"` unconditionally (`codec.rs:50-51`), and the
`minecraft:item_modifier` arm calls it directly (`codec.rs:296`) —
inline `{"function":"minecraft:set_count","count":1}` fails in Rust.
Vanilla 1.21.11's loot data uses the `"function"` key
(`data/minecraft/loot_table/entities/zombie.json`:
`"function": "minecraft:set_count"`). The 26.2 port's fix
(key-parameter dispatch) was never carried here. (b) `/item … from
entity <source>` uses `ENTITIES` (multi, `tree_misc.rs:198`); vanilla
uses a single-entity source (oracle tree export: `from entity` node
is `minecraft:entity`, `amount: single`). Two parts, one item, both
gap.

### B9-G08 — minor — Particle block_state compound with Name is rejected

REPRODUCES — same as 1.21.10. Vanilla 1.21.11's `StateHolder.codec`
dispatches on `"Name"` (`StateHolder.javap:356`: `ldc "Name"` at
offset 1, then `Codec.dispatch`), so the `{Name, Properties}` map
form is a valid block state inside particle options. Rust's
`particle_options` returns an error for ANY non-string block_state
(`args.rs:1011-1024`, with the `withAlternative(BlockState.CODEC,
…)` comment noting the intended shape but implementing the
rejection). The 26.2 port's `Name`-accepting form was never carried
here.

### B9-G09 — minor — Selector option errors: wrong cursor on duplicates; type= accepted for @a/@p/@r

REPRODUCES (both parts). Vanilla 1.21.11 still has the OLD selector
parser — the option-state refactor (`InvertableSetOptionState`,
`SetOnceOptionState`) that closed this defect for 26.2 had NOT
happened yet in 1.21.11 (neither class exists in the 1.21.11
server-game jar). The Rust parser (`selector.rs`) implements the 26.2
option-state shape, so it diverges from vanilla 1.21.11 in both parts:

(a) Duplicate options: vanilla 1.21.11's `EntitySelectorOptions.get()`
(`net.minecraft.commands.arguments.selector.options.EntitySelectorOptions`,
note the `options` subpackage) runs the `canUse` gate BEFORE the
value is read — `parseOptions` calls `get()` at offset 65, right
after `readString`, before the `=` and value (`Selector.javap:661-730`)
— but does NOT roll back: only the unknown-option branch does
`setCursor(start)` (offsets 47–52); the inapplicable branch throws
`ERROR_INAPPLICABLE_OPTION` at the cursor as it stands, i.e. at the
END of the option name (before `=`). Rust rolls back to the
option-name START (`selector.rs:331-341`, `rollback(reader, start,
…)`). For `@e[type=pig,type=cow]` the cursors differ by the
option-name length (4 for `type`). Three-version shape: 1.21.7 vanilla
at the end of the VALUE, 1.21.10/1.21.11/26.1-line vanilla at the end
of the NAME, 26.2 vanilla at the START — Rust reports at the start in
all cases, matching only 26.2.

(b) `@a[type=player]`: vanilla 1.21.11's `type` option `canUse` is
`!parser.isTypeLimited()` (`lambda$bootStrap$47`,
`SelectorOptions2.javap:862-872`), and `@a`/`@p`/`@r` all call
`limitToType(EntityType.PLAYER)` during `parseSelector` (offsets 165,
196, 225), which sets the `type` field — so vanilla 1.21.11 REJECTS
`@a[type=player]` with `ERROR_INAPPLICABLE_OPTION`. Rust pushes
`minecraft:player` into `s.types` for `@a`/`@p`/`@r`
(`selector.rs:218-242`) without marking `type_option`, so
`can_use("type")` stays true and Rust ACCEPTS it.

### B9-G12 — minor — SlotSource is back in the tree; vanilla 1.21.11 uses item_slot/item_slots

REPRODUCES — same as 1.21.10. Rust uses `ArgType::SlotSource`
everywhere in `/item` (`tree_misc.rs:171,188,202`) and `execute
if|unless items|slots` (`tree_execute.rs:240`); vanilla 1.21.11 uses
`minecraft:item_slot` / `minecraft:item_slots` (oracle tree export:
`/item` slot node `minecraft:item_slot`, `/execute if items` slot
node `minecraft:item_slots`). The `minecraft:slot_source` registry
does not exist in 1.21.11 (no such key in the oracle's `by_registry`;
the registry that DOES exist is `minecraft:slot_source_type`, with
6 entries — `contents`, `empty`, `filtered`, `group`, `limit_slots`,
`slot_range` — a different registry). The slot-name TABLE itself
matches vanilla exactly (Rust `slot_ranges()`, `args.rs:205-239`, vs
vanilla `SlotRanges` clinit — `net.minecraft.world.inventory.SlotRanges`
in 1.21.11, moved from `commands.arguments` — `SlotRanges.javap:158-304`:
contents, container.×54, hotbar.×9, inventory.×27, enderchest.×27,
villager.×8, horse.×15, weapon/weapon.mainhand/weapon.offhand/
weapon.*, armor.head/chest/legs/feet/body, armor.*, saddle,
horse.chest, player.cursor, player.crafting.×4) — the defect is the
argument type, not the names. The fix form exists IN THIS TREE:
`/loot` uses `ArgType::Slot` (`tree_world.rs:351`,
`network.rs:234-235` maps `minecraft:item_slot` → `Slot`) — it was
never carried to `/item` or `/execute`.

### B9-G13 — minor — Argument-type table is 26.3's 62, not 1.21.11's 57; index 16 is team_color

REPRODUCES — same as 1.21.10, in the stronger 26.1.2 form (not
1.21.7's). Rust ships 26.3's 62-entry table (`commands.rs:25-87`, doc
comment says "registry of 26.3 in id order"), `minecraft:team_color`
at index 16, plus five 26.3-only types (`context_float_provider`,
`context_int_provider`, `slot_source`, `feature`, `swing_animation`)
shifting later ids (`dialog` 58 vs 55, `uuid` 61 vs 56). Vanilla
1.21.11's `ArgumentTypeInfos` has 57 types with `color` at index 16
(order: 6 brigadier, entity, game_profile, block_pos, column_pos,
vec3, vec2, block_state, block_predicate, item_stack, item_predicate,
color, hex_color, … dialog, uuid — `ArgumentTypeInfos.javap`,
verified in full, 57 `ldc` names). The 1.21.11 adapter has
`command_argument_type: Remap::Same` (`adapter/v1_21_11_ids.rs:33`) —
no remap mitigates the shift, so the tree's network form mislabels
every argument type from index 16 on.

### B9-G14 — minor — `/perf` missing from the dedicated tree

REPRODUCES — same as 1.21.10. Vanilla 1.21.11 has 15 dedicated
commands (`ban`, `ban-ip`, `banlist`, `deop`, `op`, `pardon`,
`pardon-ip`, `perf`, `save-all`, `save-off`, `save-on`,
`setidletimeout`, `stop`, `transfer`, `whitelist` — the 15 in the
`includeDedicated` guard, `Commands.<init>` offsets 642–754);
`PerfCommand.register` sits at offset 702, inside the guard.
Rust's dedicated tree (`register_dedicated`, `tree_extra.rs:215+`)
has 14 commands and lacks `perf`. (The dedicated list here is the
CORRECTED one: `kick` is integrated — `KickCommand.register` at
offset 216, outside the guard — and `transfer` is dedicated, at
offset 744 inside it; `publish` at offset 765 is OUTSIDE the guard —
integrated.)

### B9-G15 — minor — Dev tree is missing six vanilla 1.21.11 dev commands

REPRODUCES — same as 1.21.10. Vanilla 1.21.11's dev block
(`Commands.<init>` offsets 572–642, under `DEBUG_DEV_COMMANDS` /
`IS_RUNNING_IN_IDE`) registers `raid`, `debugpath`,
`debugmobspawning`, `warden_spawn_tracker`, `spawnarmortrims`,
`serverpack`, plus `debugconfig` when dedicated (offset 639);
`chase` is under its own flag (`DEBUG_CHASE_COMMAND`, offset 559).
Rust's dev tree has only `raid` + `warden_spawn_tracker`
(`vanilla.rs` `if dev` blocks, lines 174 and 398) — missing
`debugpath`, `debugmobspawning`, `spawnarmortrims`, `serverpack`
(+ `debugconfig` dedicated, + `chase`). `jfr` is registered
unconditionally in Rust while vanilla gates it on
`JvmProfiler.INSTANCE.isAvailable()` (offsets 541–549) — the same
non-defect nit as 1.21.10, not counted.

### B9-G16 — minor — Argument node names differ from vanilla

REPRODUCES — nine label pairs, same shape as 1.21.10 (counted by
path). Proven statically from the tree-builder literals against
vanilla's oracle tree export, and confirmed empirically by the run's
node diff (§2):

| Path | Vanilla 1.21.11 (oracle `tree.json`) | Rust 1.21.11 |
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
`entities` → `slot` for entity, `targetPos` → `slot` for block —
vanilla's `/loot replace block` node really is `targetPos`, not
`pos`.) Node names surface in parse output (`nodes` in the fixture's
jsonl) and in client-side command feedback, so a rename is a real (if
cosmetic) divergence — minor. The run's node diff (§2) confirms
eight of the nine pairs empirically (29 label-pair occurrences);
the ninth (`/execute if items block` `pos`→`sourcePos`) is masked
because the slot argument fails first (G12) — its static evidence
stands.

### B9-G17 — minor — Recipe suggestions omit special recipes (Rust suggests the learnable subset; real vanilla suggests all recipes)

REPRODUCES — count 12 for 1.21.11 (same as 1.21.10). Vanilla 1.21.11's
client jar carries 1,470 recipe files; the oracle's
`command-registries.json` lists 1,458 `learnable_recipes`; the 12
omitted are `armor_dye`, `banner_duplicate`, `book_cloning`,
`decorated_pot`, `firework_rocket`, `firework_star`,
`firework_star_fade`, `map_cloning`, `map_extending`, `repair_item`,
`shield_decoration`, `tipped_arrow` (1.21.11 has ONE `banner_duplicate`
recipe, not 26.x's 16 colour variants — hence 12, not 24). Real vanilla
suggests all recipes via the RECIPE special case in
`CommandSourceStack.suggestRegistryElements`
(`CommandSourceStack.javap:1004-1012`, offsets 0–36); the Rust side
suggests the learnable subset. The run cannot test this faithfully
(§2): the oracle's `Src` override bypasses the special case, so the
run's recipe mismatches are artifacts and this item rests on the
static census.

### B9-G18 — minor — 26.3-era subcommands in the tree (`/item fill`, `/item override`, `/execute if|unless slots`)

REPRODUCES — same as 1.21.10. Rust has `/item fill` and `/item
override` (`tree_misc.rs:179`, `for mode in ["replace", "fill",
"override"]`) and `/execute if|unless slots` (`tree_execute.rs:224`,
`for kind in ["items", "slots"]`); vanilla 1.21.11 has neither —
`/item` is only `replace` + `modify`, and `/execute if/unless` has
`items` only (oracle tree export). Note vanilla 1.21.11's
`/execute if/unless` DOES have `stopwatch` (alongside `items`) — and
so does Rust (`tree_execute.rs:193`) — that one matches. These extras
are invisible to the corpus (vanilla's tree has no such nodes to
walk) and were caught by the static tree diff.

### B9-G20 — minor — `/schedule clear` takes a single id where vanilla takes a greedy string

REPRODUCES — same as 1.21.10. Rust `/schedule clear` takes
`ArgType::Id("schedules")` (`tree_misc.rs:49`); vanilla 1.21.11 takes
a greedy `brigadier:string` (oracle tree export: node named
`function`, parser `brigadier:string`, `type: greedy`). A schedule id
with spaces, or a prefix, parses in vanilla and fails in Rust.

### B9-G21 — minor — Scoreboard criterion suggestions include one criterion vanilla 1.21.11 lacks

REPRODUCES, HALVED — one extra criterion here, not 1.21.10's two:
vanilla 1.21.11's `custom_stat` registry HAS `nautilus_one_cm` (77
custom stats; the nautilus stat arrived in 1.21.11), so that half of
the 1.21.10 item is closed by a vanilla change. Rust's custom-stats
list (`scoreboard.rs:228-291`, 78 entries) still includes
`sleep_in_straw_bed`; vanilla 1.21.11 has no such stat (no `straw`
entry in `custom_stat`; `Stats` has `SLEEP_IN_BED` only). Full diff
of the Rust list against vanilla's 77 custom stats: exactly one
extra (`sleep_in_straw_bed`), zero missing. (The tree even knows:
`stats_screen.rs:801` documents "no `sleep_in_straw_bed` (the straw
bed is a later addition)" — but `scoreboard.rs` still lists it.)
The run confirms it empirically: the only difference in the
scoreboard suggestion lists is the extra
`minecraft.custom:minecraft.sleep_in_straw_bed` in Rust's list
(3 suggestion mismatches at perm 4, same at perm 2; nothing
missing).

### B9-G22 — minor — NEW (run-found) — Item `minecraft:container` component accepts entries without `slot`/`item`; vanilla 1.21.11 rejects them

Minted from the run (10 `give` mismatches: 3 ok-differs + 7
cursor-differs, same at perm 2). Vanilla 1.21.11's
`ContainerComponent` codec requires each entry to be a
`ContainerItem` with a `slot` (int) and an `item` (ItemStack):
`give @s minecraft:shulker_box[container=[{id:"minecraft:stone",count:64}]]`
fails in vanilla with `Malformed 'minecraft:container' component:
'No key item in MapLike[{count:64,id:"minecraft:stone"}]; No key
slot in MapLike[{count:64,id:"minecraft:stone"}]'` (cursor 40).
Rust ACCEPTS it (`decode_component` in `item.rs:25-95` has NO
`minecraft:container` arm — it falls through to `_ => Ok(())`,
so any container value parses, including the 26.x `{id, count}`
shape and the `components:{...}`-nested variant). The Rust codebase
itself knows the 1.21.11 shape — `snbt.rs:1027` tests
`{{slot:0,item:{item}}}` — but the parse-time decoder never
enforces it. Same class as G07a/G08 (component codec validation
gap), distinct root cause. Affects every item argument carrying a
container component (`/give`, `/item`, `/loot`, `/summon` with
equipment), not just `/give`. Fix form: a `minecraft:container` arm
in `decode_component` requiring a list of compounds each with an
int `slot` and a decodable `item`.

### 3.2 Closed — the 1.21.10 defects, re-checked against 1.21.11

### B9-G01 — WITHDRAWN (vanilla changed: the game_rule registry arrived with snake_case ids)

The 1.21.10 defect was that Rust exposed the 26.3 snake_case gamerule
table while vanilla 1.21.10 used camelCase names. Vanilla CHANGED in
1.21.11: game rules are now registry entries in
`minecraft:game_rule` (59 entries, `BuiltInRegistries.GAME_RULE`),
registered by `GameRules`' clinit with SNAKE_CASE ids
(`GameRules.javap`, `net.minecraft.world.level.gamerules.GameRules` —
the class moved from `world.level` to the `gamerules` subpackage):
`advance_time`, `keep_inventory`, `fire_spread_radius_around_player`,
… — the exact set the port ships. Name-for-name diff of the 59
vanilla ids against the port's `RULES` + `MAX_MINECART_SPEED`
(`gamerules.rs:52-130`): 59 = 59, zero differences either way. Sampled
defaults match vanilla exactly (`randomTickSpeed`→`random_tick_speed`
3, `maxEntityCramming`→`max_entity_cramming` 24,
`maxCommandChainLength`→`max_command_sequence_length` 65536,
`maxCommandForkCount`→`max_command_forks` 65536,
`maxBlockModifications`→`max_block_modifications` 32768 min 1,
`maxSnowAccumulationHeight`→`max_snow_accumulation_height` 1 (0..8),
`fire_spread_radius_around_player` 128 (min −1),
`max_minecart_speed` 8 (1..1000), `players_sleeping_percentage` 100,
`respawn_radius` 10, `players_nether_portal_default_delay` 80,
`players_nether_portal_creative_delay` 0). The oracle's tree export
confirms the command surface too: vanilla's `/gamerule` tree has 116
literals = 58 available rules × 2 forms (plain + `minecraft:`
prefix), `max_minecart_speed` feature-gated out — exactly the shape
the Rust tree builds (`vanilla.rs:311-326`). The fire-pair collapse
that cost 1.21.10 a count difference is gone: vanilla 1.21.11 has no
`doFireTick` / `allowFireTicksAwayFromPlayer` pair at all — the single
`fire_spread_radius_around_player` int IS vanilla's form now. Note
the template does NOT carry the 1.21.10 child's `vanilla_key` /
`from_vanilla_key` camelCase mapping (`gamerules.rs` has no such
functions) — it doesn't need one: the engine table IS vanilla's
table now, including for level.dat persistence
(`minecraft:<snake_case>` keys, `gamerules.rs:236-260`). The corpus
carries 492 gamerule cases (§2), including camelCase negative
controls (`gamerule doFireTick`, `gamerule minecraft:keepInventory
true`) that fail on BOTH sides.

### B9-G04 — CLOSED (inverted by a vanilla change)

The 1.21.7 defect was that Rust used `Rotation` where vanilla 1.21.7
used `Angle` for `/spawnpoint` / `/setworldspawn`'s trailing argument.
Vanilla CHANGED since 1.21.7: 1.21.11 uses `RotationArgument`
(`SetSpawn.javap` / `SetWorldSpawn.javap`: 2 refs each, no
`AngleArgument`; the oracle's tree export names the node `rotation`
with parser `minecraft:rotation`), and Rust uses `ArgType::Rotation`
(`vanilla.rs:347-364`) — match. (Note: `RotationArgument` lives in
`net.minecraft.commands.arguments.coordinates` in 1.21.11, not in
`arguments` — `RotationArgument.javap` is "class not found" there.)

### B9-G10 — CLOSED (inverted by a vanilla change — the Time argument IS 1.21.11's form)

The 1.21.7 defect was that Rust's `/worldborder` set/add time used a
`Time` argument where vanilla used a plain integer; the 1.21.10 tree
"fixed" it by switching to `integer(0)` — with the comment "the Time
argument is 1.21.11's". That comment is confirmed: vanilla 1.21.11's
`WorldBorderCommand` uses `TimeArgument.time(0)` for set/add time
(`WorldBorder.javap`: 3 `TimeArgument` refs at offsets 51, 110, 299 —
set, add, warning time; the `IntegerArgumentType` ref at offset 268
is the warning distance), and the oracle's tree export names the node
`time` with parser `minecraft:time`, min 0. The 1.21.11 tree uses
`ArgType::Time { min: 0 }` (`tree_world.rs:313,333`) — match. The
1.21.10 "fix" must NOT be carried to the template: it would break
1.21.11.

### B9-G11 — CLOSED against real vanilla (run mismatches are the oracle artifact)

The 1.21.7 item claimed vanilla suggests no recipes for `/recipe` and
Rust suggests the learnable set. Against REAL vanilla 1.21.11 that
premise fails: `CommandSourceStack.suggestRegistryElements` HAS the
RECIPE special case (`CommandSourceStack.javap:1004-1012`, offsets
0–36: `Registries.RECIPE` → `RecipeManager.getRecipes()` →
`suggestResource`), so real vanilla suggests all 1,470 recipes. The
run's recipe mismatches come from the oracle's `Src` override
(`VanillaCommandParseOracle.java:96-105`), which looks the registry
up in the RegistryAccess and returns `Suggestions.empty()` when absent
— bypassing the special case, exactly as in 1.21.10/26.1.2. The real
divergence (the learnable subset vs all recipes) is B9-G17, verified
statically.

### B9-G19 — WITHDRAWN (doubly: no `/swing` in vanilla 1.21.11 AND none in the tree)

The 26.1.2 secondary item's premise was that vanilla has `/swing` and
Rust adds 26.3-era animation + duration arguments. That premise fails
twice for 1.21.11: vanilla 1.21.11 has NO `/swing` at all — no
`SwingCommand` class in the server-game jar, no `swing` in the
oracle's tree export, no `swing_animation` argument type in vanilla's
57-entry table — AND the 1.21.11 tree has no `/swing` either (the
`rotate_swing` builder, `tree_world.rs:155-171`, registers only
`/rotate`; no `"swing"` literal anywhere in `horizon-command/src`).
The 1.21.10 child ADDED `/swing` to its tree
(`tree_world.rs:176-192` there) — a child divergence from the
template, not a template defect. There is no G19 defect in 1.21.11
at all. (26.1.2's vanilla HAS `/swing`, so G19 stands there — this
withdrawal is 1.21.11-specific.)

### G02 bulk (top-level diff)

Vanilla 1.21.11 integrated tree (oracle export, 75):
`advancement`, `attribute`, `bossbar`, `clear`, `clone`, `damage`,
`data`, `datapack`, `debug`, `defaultgamemode`, `dialog`,
`difficulty`, `effect`, `enchant`, `execute`, `experience`,
`fetchprofile`, `fill`, `fillbiome`, `forceload`, `function`,
`gamemode`, `gamerule`, `give`, `help`, `item`, `jfr`, `kick`,
`kill`, `list`, `locate`, `loot`, `me`, `msg`, `particle`, `place`,
`playsound`, `publish`, `random`, `recipe`, `reload`, `return`,
`ride`, `rotate`, `say`, `schedule`, `scoreboard`, `seed`,
`setblock`, `setworldspawn`, `spawnpoint`, `spectate`,
`spreadplayers`, `stopsound`, `stopwatch`, `summon`, `tag`, `team`,
`teammsg`, `teleport`, `tell`, `tellraw`, `test`, `tick`, `time`,
`title`, `tm`, `tp`, `trigger`, `version`, `w`, `waypoint`,
`weather`, `worldborder`, `xp` — all present in the Rust release
tree. Rust release tree: 78 literals (grep of
`d.literal(root,…)` across the tree builders, minus the 14
`register_dedicated` commands, minus dev-gated `raid` /
`warden_spawn_tracker`, minus Horizon-gated `pregen`).
**75 of 75 present, zero missing; three extras: `compute`,
`posteffect`, `unpublish`.** Absent from vanilla's integrated tree
(all verified absent from the oracle export AND the jar): `compute`,
`posteffect`, `unpublish`, `swing`, `raid`, `perf`, `chase`,
`debugpath`, `debugmobspawning`, `warden_spawn_tracker`,
`spawnarmortrims`, `serverpack`, `debugconfig` (dev/dedicated-gated,
see G14/G15). Present in vanilla and correctly present in Rust
(notable): `fetchprofile` (name + id branches only — the entity branch
was correctly removed, `tree_extra.rs` comment: "1.21.11 has only
the `name` and `id` branches (26.1 added `entity <entity>`)"),
`publish`, `waypoint`, `dialog`, `debug`, `test`, `jfr`, `kick`,
`stopwatch` (NEW vs 1.21.10 — `create`/`query`/`restart`/`remove`
with a `resource_location` id and a `double` scale, matching
`StopwatchCommand`; the Rust `ArgType::Id("stopwatches")` parses
identically — `args.rs:1394` reads a bare id, no registry lookup —
and suggests from the fixture's `stopwatches` list,
`args.rs:1748`), `me` with its `action` argument
(`EmoteCommands` registers `/me`, not a new `/emote` command).
`/datapack create` exists in vanilla 1.21.11 — Rust's is correct,
not an extra. `/time` clock subcommands are absent on both sides
(`vanilla.rs` comment: "no clocks in 1.21.11"; the oracle's
`registries.json` confirms: `clocks` 0 — though `timelines` has 4
entries, `/time` does not use them). The oracle's `registries.json`
confirms: `clocks` 0, `timelines` 4 (`day`, `early_game`, `moon`,
`villager_schedule`), `clock_time_markers` {}.

### Passes (verified, not re-diagnosed)

- **`/time set` pass HOLDS.** Both sides have `day|noon|night|midnight`
  literals + a `Time` argument; `add` and `query daytime|gametime|day`
  match; no clocks either side. (One behaviour check, pass.)
- **`/stopwatch` pass (NEW for 1.21.11).** Both sides have
  `create|query|restart|remove <id>` with an optional `double` scale
  on query; the id parses as a bare resource location on both sides.
  The run confirms it empirically: 0 mismatches across 1,870
  stopwatch cases (perm 4 and perm 2). One behaviour check, pass.
- **`/datapack create`** is present on both sides (see G02 bulk).
- **`fetchprofile`** name + id branches match vanilla (see G02 bulk).

### 3.3 Harness — B9-H01 FIXED for 1.21.11

FIXED — the probe compiles. The 26.1 defect was that
`game_loop_probe.rs` constructed a `FrameContext` with fields
(`focused`, `exclusive_fullscreen`) the tree's `window.rs` did not
declare. In the 1.21.11 tree the probe constructs `FrameContext` with
exactly the seven fields the struct declares (no
`focused`/`exclusive_fullscreen` anywhere). Two independent
verifications: the coordinator's `cargo check --locked -j 1 -p
horizon-client-lib --example client_probe` EXIT=0 in 20m13s
(`b00/h01-cargo-check.log`, 02:13 UTC Nov 25, 0 errors, probe rmeta
present), AND this batch's own driver build (§2), which compiles AND
links the probe (`cargo build --profile fast -p horizon-client-lib
--example client_probe`). One non-behaviour check, pass.

## 4. Oracle / fixture defects

| Id | Oracle | Status | Blocks |
|---|---|---|---|
| B9-O01 | `VanillaScoreboardOracle` | FAIL all, 3 errors (the 26.1.2 form) | `scoreboard` |
| B9-O02 | `VanillaTeamRulesOracle` | FAIL all, 6 errors (client cp; the 26.1.2 form) | `team_rules` |
| B9-O03 | `LocatorBarClient` | FAIL all, 17 errors (1.21.10's form minus STAB) | `exploration_map_command` |
| B9-O04 | lang file for error text | **CLOSED** — prepared data has the lang file | — |
| B9-O05 | `VanillaCommandExecOracle` | **FIXED** — passes all three classpaths | — |

All five verified against Batch 0's oracle table
(`docs/sweep/1.21.11/batch-0-oracles.json`, measured Nov 25) and the
raw `all-client.log` / `all-server.log` in
`versions/1.21.11/.local/measure-oracles/`.

### B9-O01 — oracle-defect — `VanillaScoreboardOracle` fails with 3 errors (26.1.2 form)

The 26.1.2 form, NOT 1.21.10's 6-error 1.21.7 form. Line 242:
`cannot find symbol: variable TeamColor` — the oracle calls
`t.getColor().map(TeamColor::getSerializedName).orElse(null)`, but
`TeamColor` (the 26.x class) does not exist in 1.21.11 at all, and
`PlayerTeam.getColor()` returns `ChatFormatting` directly
(`PlayerTeam.javap`), not `Optional<TeamColor>`. Line 436 ×2:
`TeamColor.byName` — `cannot find symbol: variable TeamColor` — plus
`incompatible types: bad type in conditional expression` (`no
instance(s) of type variable(s) T exist so that Optional<T> conforms
to ChatFormatting`), because `setColor` takes a bare `ChatFormatting`.
GONE (adapted in the 1.21.11 source): the line-44 pack-call errors
(`Scoreboard.addPlayerToTeam`, `Team.equals` — the 1.21.11 source no
longer makes those calls). Repair scope: `TeamColor` →
`ChatFormatting` at both call sites, dropping the `Optional`
handling. Blocks `scoreboard`.

### B9-O02 — oracle-defect — `VanillaTeamRulesOracle` fails with 6 errors (26.1.2 form)

Line 14: `cannot find symbol: class Hud` — no `net.minecraft.client.gui.Hud`
in 1.21.11; the HUD class is `Gui` (`Gui.javap`). Line 101: the
`LocalPlayer` 9-arg constructor does not exist in 1.21.11 (the 1.21.7
form had a different arity). Lines 188 ×2 + 189: `Hud` references
in the draw calls. Line 197: `cannot find symbol: method
doTeamsAllowDamage(Stub)` — no such method in 1.21.11. GONE
(adapted): the `monster.zombie` package error (1.21.11 HAS the
`monster.zombie` package — `MobCategory` exists here) and the seven
`@Override` errors. Blocks `team_rules`.

### B9-O03 — oracle-defect — `LocatorBarClient` fails with 17 errors

Lines 222/231: `ContainerInput` — no such class in 1.21.11 (the
`ContainerInput` interface arrived in 26.x). Lines 393–395/407/409/413:
`ClientboundUpdateAdvancementsPacket` has no `removed()`/`added()`/
`progress()` accessors in 1.21.11. Line 668: failed `@Override` on
`canUseCommandBlocks`. Lines 683/906/911: `EntityTypes` — 1.21.11 uses
`EntityType` (the `EntityTypes` holder class arrived in 26.1).
Lines 690/916: `ClientboundRemoveEntitiesPacket.entityIds()`. Lines
955/957: `ClientboundSetEntityMotionPacket.id()`/`movement()`. Line
977: `ServerboundAcceptTeleportationPacket` constructor arity. The
1.21.10 line-562 `Action.STAB` error is GONE — `Action.STAB` EXISTS
in 1.21.11 (verified via javap on the client jar; the stab animation
arrived in 1.21.11). Blocks `exploration_map_command`.

### B9-O04 — CLOSED (prepared data HAS the lang file, jar-identical)

The 26.1.2 defect was that no prepared `en_us.json` existed, so the
Rust side emitted raw `argument.*` keys instead of translated text.
For 1.21.11 the prepared data HAS `assets/minecraft/lang/en_us.json`
(md5 `1460272285b8f2671c56c538113d1308`, 7,767 keys, contains
`argument.entity.invalid`), md5-identical to the 1.21.11 client jar's
own file — so no jar-extraction mitigation was needed, and the run
emitted translated text (§2). No oracle defect, no run artifact.

### B9-O05 — FIXED (the `VanillaCommandExecOracle` compiles clean)

The 1.21.10 item's 45-error form does NOT reproduce: Batch 0 measures
`VanillaCommandExecOracle` PASS on all three classpaths (client,
server-game, server-bundler). The 1.21.11 oracle source has no
`authlib.services` / `EntityTypes` references and uses the
`gamerules` / `permissions` packages that exist in 1.21.11. The
`command_exec` driver is therefore NOT oracle-blocked for 1.21.11 —
its remaining blocker is the fixture one
(`/execute if predicate effect_view:always`, §5).

## 5. Driver table (22)

| Driver | Status |
|---|---|
| `command_parse` | **MEASURED** — 34,492 commands; perm 4: 3,172 mismatches (90.8% match), perm 2: 3,158, perm 0: 1 (root suggestion list only). See §2. |
| `command_exec` | NOT RUN — fixture-blocked only: the corpus's `/execute if predicate effect_view:always` case (`mob_effect_predicate_corpus.py:39`, imported by `command_exec_corpus.py:64`) needs a datapack fixture that does not exist. The ORACLE is clean for 1.21.11 (B9-O05 fixed), so this is a one-fixture job, not an oracle repair. |
| `scoreboard` | NOT RUN — oracle-blocked (B9-O01) |
| `team_rules` | NOT RUN — oracle-blocked (B9-O02) |
| `exploration_map_command` | NOT RUN — oracle-blocked (B9-O03) |
| `command_damage_range` | NOT RUN — no oracle; e2e, unrun this batch |
| `command_reinforcements` | NOT RUN — no oracle; e2e, unrun this batch |
| `fork_function_feedback` | NOT RUN — no oracle; e2e, unrun this batch |
| `function_admission` | NOT RUN — no oracle; e2e, unrun this batch |
| `function_tags` | NOT RUN — no oracle; e2e, unrun this batch |
| `guest_advancement_selector` | NOT RUN — no oracle; e2e, unrun this batch |
| `knot_command_view` | NOT RUN — no oracle; e2e, unrun this batch |
| `loaded_command` | NOT RUN — no oracle; e2e, unrun this batch |
| `mob_command_damage_vibration` | NOT RUN — no oracle; e2e, unrun this batch |
| `selector_bounds` | NOT RUN — no oracle; e2e, unrun this batch |
| `selector_criterion` | NOT RUN — no oracle; e2e, unrun this batch |
| `selector_empty_tag` | NOT RUN — no oracle; e2e, unrun this batch |
| `selector_global_order` | NOT RUN — no oracle; e2e, unrun this batch |
| `stand_command_hurt` | NOT RUN — no oracle; e2e, unrun this batch |
| `terminal_execute_fork` | NOT RUN — no oracle; e2e, unrun this batch |
| `tick_function_time` | NOT RUN — no oracle; e2e, unrun this batch |
| `vehicle_command_damage` | NOT RUN — no oracle; e2e, unrun this batch |

22 drivers: 1 measured, 1 fixture-blocked (oracle clean), 3
oracle-blocked, 17 unrun e2e (no oracle). The 1.21.11 tree has 648
`test_*_parity.py` drivers total (same as 1.21.10). All 17 no-oracle
e2e driver names were verified fresh against the 1.21.11 tree (all
exist); 7 of them reference `command_exec`.

## 6. Method notes

- **Verification ladder, in order:** (1) vanilla 1.21.11's own
  server-game jar (javap on named classes; the bundler `server.jar`
  was never javap'd), (2) vanilla 1.21.11's own client jar for
  client-side classes (`Action.STAB`, `TeamColor` absence) and data
  (`data/minecraft/loot_table/entities/zombie.json` for the
  `"function"` key, the 1,470 recipe files for the G17 census),
  (3) the oracle's own exports (`registries.json` for the
  `game_rule`/`custom_stat`/`slot_source_type` registries,
  `tree.json` for vanilla's integrated tree), (4) the port's own
  source (`horizon-command`, `horizon-client-core`,
  `horizon-protocol`), (5) the run's own outputs. The 26.3 base's
  form was never trusted — and three times it would have misled:
  G01 (26.3's snake_case table IS vanilla 1.21.11's form, but for a
  different reason — the registry — not because 26.3's form carried
  over), G10 (26.3's Time form matches 1.21.11's, but the 1.21.10
  "fix" to integer would have broken it), G21 (`nautilus_one_cm`
  arrived in 1.21.11 — a 26.3-only stat in 1.21.10, a vanilla stat
  here).
- **Corpus regenerated from 1.21.11's own tree export.** The 1.21.10
  corpus was NOT reused (its `6fa6f661` md5 is for a different
  vanilla). The generator started from the 1.21.11 tree's own
  `tools/command_parse_corpus.py` — which carries EXTRA cases the
  1.21.10 adaptation dropped — and the EXTRA list was adapted for
  1.21.11: dropped `swing` (4), `compute`, `unpublish`, `posteffect`
  (2), `raid start 5`, `time of minecraft:overworld set 5`; kept
  stopwatch (5) + `execute if stopwatch …`, fetchprofile + publish
  cases; added 27 1.21.11 edge cases (gamerule snake_case/camelCase
  variants incl. `fire_spread_radius_around_player 64`,
  `max_minecart_speed` gated cases, `/place feature {}` inline
  values, slot-error cases, `spawnpoint`/`setworldspawn … 90 0`,
  `/schedule clear hello world` / trailing-space / empty, `publish
  true survival`, `publish true 25565`, `worldborder … 10t`,
  `worldborder warning time 5s`, stopwatch trailing-space/extra-token
  cases). EXTRA entries: 511. Corpus preview: 34,492 commands, md5
  `9e9597f33c9c5bc9e2112f0bda481720`, 492 gamerule cases, 1,870
  stopwatch cases, 0 swing cases (`run-artifacts/corpus-preview.txt`).
  The run's corpus md5 matches the preview exactly, and the run's
  `corpus.txt` is preserved in `run-artifacts/`.
- **Prepared data was shared, not built by this batch.** Batch 12's
  prepare was already running into the shared worktree `.local` when
  this batch started (pid 49916, lock file present); a second
  prepare would have conflicted. It finished EXIT=0 (1,373.9 s) with
  the lang file included. This batch used the shared prepared data
  and ran its own oracle export (registries + tree) independently
  (`run-oracle-export.py`, all steps exit 0; `oracle-export/`).
  The prepared `command-registries.json` matches this batch's oracle
  export exactly (same keys, 1,458 learnable recipes).
- **The driver's `compare()` strips `nodes`**, so argument-name label
  differences (B9-G16) are invisible to it — the separate
  `node_diff.py` comparison over the run's `rust.jsonl` /
  `vanilla.jsonl` is the check for G16 (§2). The driver's
  `compare()` also skips oracle-crash rows; the run's crash census
  is in §2.
- **The run's recipe suggestion mismatches are artifacts** — the
  oracle's `Src` override bypasses vanilla's RECIPE special case
  (§3, B9-G11) — and must not be read as port defects. The real
  divergence is B9-G17, verified statically.
- **The 1.21.10 report's three-version shape note for G09** (1.21.7
  at the end of the VALUE, 1.21.10/1.21.11/26.1-line at the end of
  the NAME, 26.2 at the START) was verified fresh for 1.21.11:
  `parseOptions` calls `get()` at offset 65, after `readString`,
  before `=`; only the unknown-option branch does `setCursor(start)`.
- **Package moves in 1.21.11** (javap evidence in `evidence/`):
  `GameRules` is in `net.minecraft.world.level.gamerules` (not
  `world.level`); `EntityDataSource` is in
  `net.minecraft.network.chat.contents.data`; `SlotRanges` is in
  `net.minecraft.world.inventory`; `RotationArgument` is in
  `net.minecraft.commands.arguments.coordinates`;
  `EntitySelectorOptions` is in the `selector.options` subpackage.
  The permissions package (`net.minecraft.server.permissions`,
  with `Permission` / `PermissionCheck` / `LevelBasedPermissionSet`)
  arrived in 1.21.11 — `Commands.LEVEL_GAMEMASTERS` and
  `hasPermission` are used by the stopwatch command.

## 7. Open questions

1. **Should the 1.21.10 child's `vanilla_key` / `from_vanilla_key`
   gamerule mapping be carried to the template?** No — the template
   doesn't need it (G01 withdrawn: the engine table IS vanilla's
   table now). But the child's mapping exists because 1.21.10's
   vanilla used camelCase — the mapping is a 1.21.10-only shim, not
   template material. No action.
2. **Should the 1.21.10 child's `/worldborder` integer-time "fix" be
   carried to the template?** NO — it would BREAK 1.21.11 (G10:
   vanilla 1.21.11 uses `TimeArgument`). The child's fix is
   1.21.10-specific. Flagged here so no one carries it by rote.
3. **Should the 1.21.10 child's `/swing` tree entry be carried to the
   template?** NO — vanilla 1.21.11 has no `/swing` (G19 withdrawn
   doubly). The child's entry is a child-only divergence.
4. **`/stopwatch` suggestion behaviour** — clean: 0 mismatches
   across 1,870 stopwatch cases at perm 4 and perm 2 (the id
   suggests from the fixture's `stopwatches` list on both sides).
   No open question.
5. **`command_exec` is one fixture away from running** — the oracle is
   clean for 1.21.11 (B9-O05 fixed); only the `effect_view:always`
   datapack fixture blocks it. A follow-up batch could run it with
   that single fixture added.
6. **The `max_minecart_speed` gamerule is feature-gated out of the
   oracle's tree export** (58 of 59 rules available) — the port's
   tree includes it unconditionally (`gamerules.rs:130`). Whether
   the feature flag (`minecart_improvements`) should gate it in the
   port's tree is a template-level question, not a Batch 9 defect
   (the port's `RULES` table is vanilla's table; the tree's
   availability under a disabled feature flag is a separate
   behaviour). Left open.
