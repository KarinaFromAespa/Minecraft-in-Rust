# Batch 9 — Commands (26.1.2 parity sweep)

Date: 2026-11-25. Branch: `parity-sweep-26.1.2` (Batch 0 landed).
Scope: parsing, execution, selectors, functions, scoreboard — 22 drivers;
57 argument types (26.1.2's own list), 59 game rules.
Ledger slice: `slice.json` (this directory).

## 0. Execution status — read first

**The `command_parse` run MEASURED.** The 26.1.2 probe compiles: Batch 0
verified H01 does NOT reproduce (`cargo check --locked -j 1 -p
horizon-client-lib --example client_probe` exits 0), and this batch ran
the driver end-to-end via `tools/test_command_parse_parity.py` (§2):
34,542 commands × 3 permission levels (corpus regenerated from
26.1.2's tree export, md5-identical to the 26.1.1 run's,
`aa28f318ef41b52e2cc75a727e290317`), 3,162 / 3,148 / 1 mismatches
(perm 4 / 2 / 0); node diff 34 / 29 / 0 inputs. The run's outputs are
BYTE-IDENTICAL to the 26.1.1 run's (`vanilla.jsonl`, `rust.jsonl`,
`mismatches.jsonl` at all three permission levels, verified by `cmp`)
— the strongest possible confirmation that the 26.1.1→26.1.2 vanilla
delta touches nothing in the command surface.

Everything else in this report is static triage (method 2: javap +
source + oracle data + datagen), and it stands on its own: the
re-checks below were all verified without the run, and the run's
mismatch classification (§2) confirms them empirically where the
corpus reaches them.

**Headline: the 26.1.2 command surface is the 26.1.1 command surface.**
The 26.1.2 tree is a delta port of the 26.1.1 tree, and every file in
this batch's scope is byte-identical between the two trees — all of
`crates/horizon-command/src/` (`args.rs`, `codec.rs`, `component.rs`,
`selector.rs`, `tree_data.rs`, `tree_execute.rs`, `tree_extra.rs`,
`tree_misc.rs`, `tree_scoreboard.rs`, `tree_world.rs`, `vanilla.rs`,
`lib.rs`), `horizon-protocol/src/packets/play/commands.rs`,
`horizon-client-core/src/gamerules.rs` and `scoreboard.rs`, all five
in-scope oracle Java files, the `command_parse_fixture`, the driver,
and the corpus generator (all verified by `cmp` against the 26.1.1
tree). The 26.1.1→26.1.2 vanilla delta (DataVersion 4788→4790 + 6
changed classes) touches no command node, no registry, and no loot
data in scope. Every 26.1.1 verdict in this batch was nevertheless
re-verified fresh against 26.1.2's OWN jar/data — none was copied —
and **all 26 items land with the same verdict as 26.1.1**: 16
behaviour gaps reproduce (including all five command flips among the
twelve 26.1 regressions — G03, G05, G07b, G12, G13), 5 closures stand,
H01 stays fixed, all three in-scope oracles fail with the same error
counts as 26.1.1 (O01 3, O02 6, O03 13), and O04 stays closed. The
tally is unchanged: 24 as-tallied (7 pass, 17 gap) = 20 behaviour (6
pass, 14 gap) + 4 non-behaviour (1 pass, 3 gap).

Two corrections to the 26.1.1 record, both verified fresh for 26.1.2
(§6): (1) the dedicated-command list in the 26.1.1 report had `kick`
in and `transfer` out — backwards. `KickCommand.register` sits at
`Commands.<init>` offset 216, outside the `includeDedicated` guard,
while `TransferCommand.register` sits at offset 752, inside it; the
oracle's INTEGRATED tree export (76 top-level) has `kick` and lacks
`transfer`. The corrected dedicated list is `ban`, `ban-ip`,
`banlist`, `deop`, `op`, `pardon`, `pardon-ip`, `perf`, `save-all`,
`save-off`, `save-on`, `setidletimeout`, `stop`, `transfer`,
`whitelist`. The G02 outcome is unchanged (76 of 76 integrated
present, three extras) and G14 stands exactly as stated (the
dedicated tree's 14 commands are vanilla's 15 minus `perf`).
(2) Several line numbers in the 26.1.1 report were off by 1–2 (the
files are byte-identical, so the numbers below are the correct ones
for both trees): G12 `tree_misc.rs:171,188,202` and
`tree_execute.rs:240`; G18 `tree_misc.rs:179` and
`tree_execute.rs:224`; G10 `tree_world.rs:334`.

Prepared data used: `versions/26.1.2/.local/horizon-client/26.1.2/`,
including `commands.json`, `command-registries.json`
(VanillaCommandParseOracle export), `registries.json`, and
`assets/minecraft/lang/en_us.json` (md5-identical to the 26.1.2 client
jar's file, 7,886 keys). All behaviour claims are grounded in 26.1.2's
OWN jar/data — the named server-game jar
(`26.1.x/versions/26.1.2/jars/server-game.jar`, sha1
83eb1106f12f24c9127ce22070d71853157af127, Mojang-mapped), the 26.1.2
client jar (sha1 4e618f09a0c649dde3fdf829df443ce0b8831e65), the vanilla
26.1.2 datagen reports, or the oracle's own exports. The 26.3 base's
form was never trusted. (The `server.jar` in the same directory is a
Mojang bundler jar, sha1 97ccd4c0ed3f81bbb7bfacddd1090b0c56f9bc51 —
never javapped.)

Nothing in this batch touched production code (`crates/`); no oracle
was repaired (the three broken in-scope oracles are scoped in §4, not
fixed); no git commands were run. All line numbers are in the 26.1.2
tree unless stated otherwise.

## 2. The command_parse run (measured)

Run 2026-11-25 via `tools/test_command_parse_parity.py --allow-gaps`
(`b09/run-command-parse.sh`: taskset -c 0, CARGO_BUILD_JOBS=1). Corpus
34,542 commands (tree-walked from vanilla's integrated dispatcher +
hand-written edge cases), run at permission levels 4, 2 and 0 — 103,626
parse comparisons. The Java oracle compiled clean (5 warnings,
`compile-java.log`); the Rust `client_probe` build was interrupted in
its first attempt at 20 crates by a service restart (last:
`horizon-shaders`) and resumed from the warm cache — the remaining 22
crates compiled and cargo `Finished 'fast'` in 56m 56s with no errors
(41 distinct crates across both attempts; the long pole was
`horizon-client-render` at ~19 min and `horizon-client-lib` at ~20
min, both slowed by a competing 1.21.4-port build sharing the VM's
1.5 CPUs). Driver exit 0. Outputs:
`.local/command-parse-parity/report.json`,
`perm{4,2,0}/{vanilla,rust,mismatches}.jsonl`; analysis by
`b09/classify_mismatches.py` and `b09/node_diff.py` (outputs
`classify-output.txt`, `node-diff-output.txt` in this directory).
Preserved in `b09/run-artifacts/`: `report.json`, `corpus.txt`,
`tree.json`, the per-perm `mismatches.jsonl` + `node-diffs.json`,
both compile logs, and the driver logs. No oracle crashes (0 `crash`
entries in any `vanilla.jsonl`).

**The run reproduces the 26.1.1 run exactly.** Every number below is
identical to the 26.1.1 run's, and the output files are byte-identical
to the 26.1.1 run's (`cmp` on `vanilla.jsonl`, `rust.jsonl` and
`mismatches.jsonl` at all three permission levels). This is expected:
the corpus is md5-identical, the port's command code is
byte-identical, and the 26.1.1→26.1.2 vanilla delta touches no command
node — but it was measured, not assumed.

### 2.1 Per-permission results

| Perm | Commands | Match | Mismatches | ok-differs | cursor-differs | error-text-only | suggestions |
|---|---|---|---|---|---|---|---|
| 4 | 34,542 | 31,380 (90.8%) | 3,162 | 22 | 44 | 2,858 | 238 |
| 2 | 34,542 | 31,394 (90.9%) | 3,148 | 17 | 37 | 2,857 | 237 |
| 0 | 34,542 | 34,541 (100.0%) | 1 | 0 | 0 | 0 | 1 |
| Total | 103,626 | 97,315 | 6,311 | 39 | 81 | 5,715 | 476 |

Perm 2's mismatch set is perm 4's set minus the 14 perm-4-only inputs
(`publish` 13 + `unpublish` 1 — both gated below level 4; at perm 2
both sides reject `publish` identically with cursor 0); perm 0's
single mismatch (the empty-input root suggestion list) is also in
perm 4's set. So **3,162 unique mismatching inputs**.

### 2.2 What the mismatches are (perm 4; perm 2 identical except publish/unpublish)

| Command | Total | Composition | Maps to |
|---|---|---|---|
| execute | 3,006 | 2,802 error-text-only (2,782 SlotSource message + 20 codec error-text shape), 200 suggestions (all rust-only `slots`), 4 cursor-differs (`@@bad@@` slot tokens) | G12, G07a, G18 |
| item | 65 | 51 error-text-only (33 SlotSource + 18 codec), 3 ok-differs, 10 suggestions, 1 cursor-differs | G12, G07a/b, G18 |
| advancement | 22 | all cursor-differs: 8 `type=` accepted for @a/@p/@r, 14 duplicate-option wrong cursor | G09 |
| place | 20 | 11 ok-differs, 5 cursor-differs, 3 error-text-only, 1 suggestions | G03 |
| data | 17 | all suggestions, rust-only `compute` | G02 |
| publish | 13 | perm 4 only: 7 cursor-differs, 4 ok-differs, 1 error-text-only, 1 suggestions | G05 |
| swing | 4 | 1 cursor-differs, 1 error-text-only, 2 suggestions | G19 |
| recipe | 3 | all suggestions | G11 artifact (§2.4) |
| scoreboard | 3 | all suggestions | G21 |
| schedule | 2 | all ok-differs | G20 |
| particle | 2 | 1 ok-differs, 1 cursor-differs | G08 |
| posteffect | 2 | cursor-differs | G02 |
| (empty input) | 1 | suggestions: root list | G02 |
| compute | 1 | cursor-differs | G02 |
| unpublish | 1 | perm 4 only: ok-differs | G02 |

Detail on the two big commands. `execute`'s 2,802 error-text-only
mismatches are 2,782 × `Unknown slot '0'` vs `Can't find element
'minecraft:0' in registry 'minecraft:slot_source'` (G12) plus 20 codec
error-text-shape cases (5 distinct pairs × 4: vanilla `No key
condition` / `No key function` vs Rust `No key type` — G07a's
footprint in error text). Its 200 suggestions are all the extra
`slots` literal (G18). `item`'s 3 ok-differs are the 2 modifier
`function`-key cases (G07a) plus the multi-entity `from entity` source
(G07b: `item replace entity @s container.0 from entity @a
weapon.mainhand` parses in Rust, fails in vanilla); its 10 suggestions
are 1 × `fill`/`override` (G18) plus 9 × SlotSource `*` wildcards
(`armor.*`, `container.*`, … — G12: vanilla's `item_slot` suggests
only concrete slots).

`publish` mismatches appear only at perm 4 (the command is gated
below it); `unpublish` likewise. Perm 0's single mismatch is the
empty-input root suggestion list: Rust's list is vanilla's plus
exactly `compute`, `posteffect`, `unpublish` (G02).

### 2.3 Node diff (B9-G16 confirmed)

The driver's `compare()` strips `nodes`, so `node_diff.py` compared
them independently. Perm 4: **34 inputs** with node differences out of
34,542; perm 2: **29**; perm 0: **0**. Full lists in
`perm{4,2,0}/node-diffs.json` (preserved in `run-artifacts/`).

Label pairs (perm 4; perm 2 identical): `slot` → `slots` 11,
`targets` → `target` 6, `pos` → `targetPos` 5, `sourceSlot` →
`sourceSlots` 4, `source` → `sourcePos` 2, `entities` → `source` 1.
Six of G16's nine static pairs are confirmed empirically; the other
three (`/item modify` pos/targets, `/execute if items block`
pos→sourcePos) have no corpus case reaching them and stand on the
static proof. No label pair beyond the static nine.

Shape diffs (one side parses, the other fails — 22 at perm 4, 17 at
perm 2): `place feature` 11 (G03), `schedule clear` 2 (G20),
`publish` 4 perm-4-only (G05), item-modifier 2 (G07a), `particle` 1
(G08), item multi-entity `from entity` 1 (G07b — Rust parses, vanilla
fails), `unpublish` 1 perm-4-only (G02 — Rust parses).

**Node-diff verdict: G16 CONFIRMED** — the fixture emits the 26.3-era
names (`slots`, `target`, `targetPos`, `sourceSlots`, `sourcePos`,
`source`) where vanilla 26.1.2 emits `slot`, `targets`, `pos`,
`sourceSlot`, `source`, `entities`.

### 2.4 Recipe suggestions are the predicted oracle artifact (B9-G11)

6 recipe suggestion mismatches (3 per perm at 4/2), all the predicted
form: the oracle suggests `["*"]` (or nothing) while Rust suggests
its 1,491-entry learnable list — the oracle's `Src.suggestRegistryElements`
override bypasses vanilla's RECIPE special case (B9-G11, §3.2). NOT
port defects; no item minted. G17 (the 24 special recipes Rust omits)
remains statically verified only — no corpus case covers a
special-recipe prefix.

### 2.5 Run observations confirming the run-minted items (G19/G20/G21)

1. **`/swing` animation + duration** (8 mismatches: 4 per perm at 4/2).
   Rust adds an `animation` argument (StrEnum none/whack/stab) and a
   `duration` (Time) under each hand literal
   (`tree_world.rs:176-192`); vanilla 26.1.2's `/swing` ends at the
   hand literal (`tree.json`: swing → targets → mainhand/offhand, all
   executable). Same family as G18 (26.3-era tree additions) but
   argument-level, not subcommand-level — B9-G19, re-verified for
   26.1.2. Also the source of the run's only raw-key errors
   (`argument.swing_animation.invalid`, a key absent from the 26.1.2
   lang file): 2 unique inputs, 4 emissions across perms 4 and 2.
2. **`schedule clear` takes a greedy string in vanilla** (4 mismatches:
   2 per perm). Vanilla's clear argument is `brigadier:string` greedy
   (`tree.json`), so `schedule clear hello world` parses; Rust takes
   `ArgType::Id("schedules")` (`tree_misc.rs:49`) and fails at the
   space (cursor 21). Single-token ids match (`schedule clear
   foo:bar` parses on both). B9-G20, re-verified for 26.1.2.
3. **Scoreboard criterion suggestion** (6 mismatches: 3 per perm).
   Rust suggests one extra criterion,
   `minecraft.custom:minecraft.sleep_in_straw_bed` (9,133 vs 9,132
   entries; order otherwise identical). Vanilla 26.1.2 has no such
   criterion. B9-G21, re-verified for 26.1.2.

Also answered by the run: §7.7's open question — yes, `compute`'s
`context_sources` helper IS shared with `/data` (17 `data modify …`
suggestion mismatches, all rust-only `compute`); counted under G02,
not a new item.

### 2.6 Error-text count (B9-O04 confirmation) and tally check

Error-text-only mismatches: **2,858 (perm 4) / 2,857 (perm 2)**.
Composition at perm 4: 2,815 SlotSource message (G12: 2,782 execute +
33 item), 38 codec error-text shape (G07a: 20 execute + 18 item), 3
place-feature registry message (G03), 1 publish (G05), 1 swing (raw
key). 4 raw-key emissions in the whole run (2 unique swing inputs,
each at perms 4 and 2 — the only bare-key errors anywhere in the
run; §2.5) — no unresolvable-key artifact: the 1.21.7-style 86,241-mismatch blowup
did not recur, confirming O04's closure (fixture lang file present
and jar-identical).

**Tally check: the run contradicts NO static verdict.** Every gap with
run coverage is confirmed empirically (G02, G03, G05, G07, G08, G09,
G12, G16, G18, G19, G20, G21); every closure stands (G11's artifact
prediction confirmed exactly); G13/G14/G15/G17 have no run coverage
and stand on their static proofs. The 24-check tally (7 pass / 17 gap)
is UNCHANGED; G19/G20/G21 are re-check items carried from 26.1.1, not
new checks (see §6 for the tally scheme).

## 3. Defects

### 3.1 Open defects

### B9-G02 — major — Three 26.x commands in the release tree; vanilla 26.1.2 has none of them

REPRODUCES — same three extras as 26.1.1. The full top-level diff
(§3.2, G02 bulk) shows the Rust release tree (79 literals) against
vanilla 26.1.2's integrated tree (76 commands, from the oracle's own
INTEGRATED tree export): exact match except THREE extras —
`posteffect`, `unpublish`, `compute`:

- `posteffect`: no `PostEffectCommand` class in the 26.1.2
  server-game jar, no `posteffect` in the datagen `commands.json`
  dump or the oracle's tree export, no `PostEffectCommand.register`
  in `Commands.<init>`, and no `minecraft:post_effects` registry in
  26.1.2's `registries.json` — so the tree's
  `ArgType::Id("post_effects")` arguments (`tree_extra.rs:40,50`)
  reference a registry that does not exist. Registered
  unconditionally in `tree_extra::register` (`tree_extra.rs:36`).
- `unpublish`: no `UnpublishCommand` class in the 26.1.2 jar and no
  `unpublish` in the dump (`unpublish` arrived in 26.2). The 26.1
  port never added it; the 26.1.1/26.1.2 port registers it
  unconditionally (`tree_extra.rs:83-85`, under the `// PublishCommand
  / UnpublishCommand` comment).
- `compute`: no `ComputeCommand` class in the 26.1.2 jar and no
  `compute` in the dump. The 26.1 port had removed it entirely; the
  26.1.1/26.1.2 port registers it unconditionally
  (`tree_data.rs:51-69`, `compute (default | block <pos> | entity
  <target>) (float <provider> [<scale>] | integer <provider>)`).

All three are gamemaster-only, but they are real divergences in the
release tree: on each, Rust parses and vanilla reports unknown
command. The corpus carries edge cases for all three (`posteffect`,
`posteffect @s minecraft:x`, `compute`, `unpublish`) — see §2 for the
run's verdict on them.

### B9-G03 — minor — `/place feature` uses the wrong registry and accepts inline values

REPRODUCES — a 26.1 closure that flipped for 26.1.1 and stays flipped
for 26.1.2. Vanilla 26.1.2's `PlaceCommand` uses
`ResourceKeyArgument.key(Registries.CONFIGURED_FEATURE)` (javap:
`CONFIGURED_FEATURE` + `ResourceKeyArgument.key` at offsets 25–28;
the oracle's tree export agrees: the `feature` node is
`minecraft:resource_key` with registry
`minecraft:worldgen/configured_feature`). The 26.1 port matched this
(`ArgType::ResKey("minecraft:worldgen/configured_feature")`, with a
comment noting inline feature values are 26.3-only). The 26.1.2 port
uses `ArgType::ResourceOrId("minecraft:worldgen/feature")`
(`tree_world.rs:60-64`) — the wrong registry key AND an argument type
that accepts an inline SNBT value (`args.rs:140-141,1661-1663`,
`resource_or_id`), which vanilla 26.1.2 rejects. So `/place feature
{...}` parses in Rust and fails in vanilla, and feature suggestions
come from the wrong registry.

### B9-G05 — minor — `/publish` is missing its gamemode argument

REPRODUCES — a 26.1 closure that flipped for 26.1.1 and stays flipped
for 26.1.2. Vanilla 26.1.2's `PublishCommand` is `publish
[<allowCommands> [<gamemode> [<port>]]]` (javap: `BoolArgumentType`
at offset 31, `GameModeArgument.gameMode` at 50,
`IntegerArgumentType(0, 65535)` at 72; the oracle's tree export
agrees: `allowCommands` → `gamemode` → `port`). The 26.1 port matched
this. The 26.1.2 port is `publish [<allowCommands> [<port>]]`
(`tree_extra.rs:76-82`) — the gamemode argument is gone, so `publish
true survival 25565` (a corpus case) fails to parse in Rust where
vanilla accepts it.

### B9-G07 — minor — Item modifiers: inline JSON fails (wrong type key) AND multi-entity source

REPRODUCES — in FULL for 26.1.2; in 26.1 only part (a) reproduced.

(a) The 26.1.2 `dispatch()` in `codec.rs:41-67` reads the `"type"` key
unconditionally, and the `minecraft:item_modifier` arm
(`codec.rs:296-298`) calls it directly — so
`{"function":"minecraft:set_count","count":1}` fails with a missing
`"type"` key error. Vanilla 26.1.2 loot data uses the `"function"` key
(client jar `data/minecraft/loot_table/entities/zombie.json`:
`"function": "minecraft:set_count"`, `"function":
"minecraft:enchanted_count_increase"` — verified in the 26.1.2 jar;
the 26.1.1→26.1.2 delta touches no loot data). The 26.2 port fixed
this (key-parameter dispatch); the 26.1.2 port carries the 1.21.7
form, as 26.1.1 did.

(b) `/item … from entity <source>` uses `ENTITIES` (multi-entity,
`tree_misc.rs:198`) in 26.1.2. Vanilla 26.1.2's `ItemCommands` uses
`EntityArgument.entity()` for `source` — the oracle's tree export
shows the `source` node as `minecraft:entity` with `{'type':
'entities', 'amount': 'single'}`. The 26.1 port had fixed this
(single-entity source); the 26.1.1/26.1.2 port reverted it. So `/item
replace block … from entity @e …` parses in Rust and fails in
vanilla.

### B9-G08 — minor — Particle block_state compound with Name is rejected

REPRODUCES. Rust 26.1.2 `particle_options` (`args.rs:1011-1024`)
returns an error for ANY non-string `block_state` value — the compound
branch builds an error mentioning the `id` key and returns it.
Vanilla 26.1.2's `StateHolder.codec` dispatches on `"Name"` (javap:
`Codec.dispatch("Name", …)`, `ldc "Name"` at offset 1) — the `{Name,
Properties}` map form is the form vanilla's codec takes, so
`block_state:{Name:"minecraft:stone"}` is valid in vanilla and
rejected by the port. The 26.2 port fixed this; the 26.1.2 port
carries the 1.21.7 form, as 26.1.1 did.

### B9-G09 — minor — Selector option errors: wrong cursor on duplicates; type= accepted for @a/@p/@r

REPRODUCES (both parts). Vanilla 26.1.2 still has the OLD selector
parser — the option-state refactor (`InvertableSetOptionState`,
`SetOnceOptionState`) that closed this defect for 26.2 had NOT
happened yet in 26.1.2 (neither class exists in the 26.1.2
server-game jar). The Rust parser (`selector.rs`) implements the 26.2
option-state shape, so it diverges from vanilla 26.1.2 in both parts:

(a) Duplicate options: vanilla 26.1.2's `EntitySelectorOptions.get()`
(`net.minecraft.commands.arguments.selector.options.EntitySelectorOptions`,
note the `options` subpackage) runs the `canUse` gate BEFORE the
value is read (offsets 17–27) but does NOT roll back: only the
unknown-option branch does `setCursor(start)` (offsets 47–52); the
inapplicable branch throws `ERROR_INAPPLICABLE_OPTION` at the cursor
as it stands, i.e. at the END of the option name (before `=`).
Rust rolls back to the option-name START (`selector.rs:331-341`,
`rollback(reader, start, …)`). For `@e[type=pig,type=cow]` the
cursors differ by the option-name length (4 for `type`). Three-version
shape: 1.21.7 vanilla at the end of the VALUE, 26.1/26.1.1/26.1.2
vanilla at the end of the NAME, 26.2 vanilla at the START — Rust
reports at the start in all three, matching only 26.2.

(b) `@a[type=player]`: vanilla 26.1.2's `type` option `canUse` is
`!parser.isTypeLimited()` (`lambda$bootStrap$42`), and `@a`/`@p`/`@r`
all call `limitToType(EntityType.PLAYER)` during `parseSelector`
(offsets 165, 195, 224), which sets the `type` field — so vanilla
26.1.2 REJECTS `@a[type=player]` with `ERROR_INAPPLICABLE_OPTION`.
Rust pushes `minecraft:player` into `s.types` for `@a`/`@p`/`@r`
(`selector.rs:218-242`) without marking `type_option`, so
`can_use("type")` stays true and Rust ACCEPTS it.

### B9-G12 — minor — SlotSource is back in the tree; vanilla 26.1.2 uses item_slot/item_slots

REPRODUCES — a 26.1 closure that flipped for 26.1.1 and stays flipped
for 26.1.2. Vanilla 26.1.2's `/item` and `/execute if items` use
`minecraft:item_slot` (single) and `minecraft:item_slots` (multi) —
`SlotArgument`, read-until-space (`lambda$parse$0`: `c != ' '`),
`slot.unknown` / `slot.only_single_allowed` at the cursor after the
word, and the `slot_ranges()` table matches vanilla 26.1.2's
`SlotRanges` entry for entry (contents; container. ×54, hotbar. ×9,
inventory. ×27, enderchest. ×27, mob.inventory. ×8, horse. ×15,
weapon.*×2, armor.*×5, player.crafting. ×4, singles saddle /
horse.chest / player.cursor — all verified against the javap). The
26.1 port had removed the 26.3-only `SlotSource` argument type and
used `ArgType::Slot` / `ArgType::Slots`. The 26.1.2 port uses
`ArgType::SlotSource` everywhere in `/item`
(`tree_misc.rs:171,188,202`) and in `execute if|unless items|slots`
(`tree_execute.rs:240`): a slot range name, else a slot-source id or
inline value (`args.rs:1647-1660`, `resource_or_id` on
`minecraft:slot_source` — a registry that does not exist in 26.1.2).
So a non-range word is accepted by Rust as a slot-source id/inline
value where vanilla errors `slot.unknown`, the single-vs-multi
distinction (`slot.only_single_allowed`) is lost, and the node names
differ too (see B9-G16: `slots` vs `slot`, `sourceSlots` vs
`sourceSlot`).

### B9-G13 — minor — Argument-type table is 26.3's 62, not 26.1.2's 57; index 16 is team_color

REPRODUCES — a 26.1 closure that flipped for 26.1.1 and stays flipped
for 26.1.2. Vanilla 26.1.2's `ArgumentTypeInfos` bootstrap
(`net.minecraft.commands.synchronization.ArgumentTypeInfos`)
registers 57 types (57 `register` calls in the javap), and index 16
is `color` (17th `register` call, `ldc "color"` at bytecode offset
267). The 26.1 port shipped exactly that list. The 26.1.2 port ships
26.3's table (`horizon-protocol/src/packets/play/commands.rs:25-87`,
doc comment: "The `minecraft:command_argument_type` registry of 26.3
in id order"): 62 entries, `minecraft:team_color` at index 16, plus
five 26.3-only types vanilla 26.1.2 never registered —
`context_float_provider`, `context_int_provider`, `slot_source`,
`feature`, `swing_animation` — which shift every later protocol id
(`dialog` 58 vs 55, `uuid` 61 vs 56). The first 16 entries agree; the
divergence starts at index 16.

### B9-G14 — minor — `/perf` missing from the dedicated tree

Carried from 26.2, verified fresh for 26.1.2. Vanilla 26.1.2 registers
`PerfCommand` inside the `includeDedicated` block of
`net.minecraft.commands.Commands` (javap: `PerfCommand.register` at
bytecode offset 710, between `PardonIpCommand` and `SaveAllCommand`,
inside the `includeDedicated` guard at 651–762 — the same offsets as
26.1, 26.1.1 and 26.2). The command is `perf start|stop`, owners
permission (datagen `commands.json` agrees). The Rust release
(integrated) tree correctly lacks it, but the dedicated tree
(`tree_extra.rs:register_dedicated`, 14 commands) lacks it too — the
only one of vanilla's 15 dedicated commands not registered (the
corrected dedicated list is in §0: `kick` is integrated and
`transfer` is dedicated, both correctly placed in the Rust tree).
Single command, dedicated-only, profiling use — minor.

### B9-G15 — minor — Dev tree is missing six vanilla 26.1.2 dev commands

Carried from 26.2, verified fresh for 26.1.2. Vanilla 26.1.2's dev
block (`SharedConstants.DEBUG_DEV_COMMANDS || IS_RUNNING_IN_IDE`,
`Commands` constructor offsets 580–632) registers `raid`,
`debugpath`, `debugmobspawning`, `warden_spawn_tracker`,
`spawnarmortrims`, `serverpack` (plus `debugconfig` when dedicated,
plus `chase` under the separate `DEBUG_CHASE_COMMAND` flag — the same
layout as 26.1, 26.1.1 and 26.2). The Rust dev tree has only the two
`if dev` blocks in `vanilla.rs:161,385` (`raid`,
`warden_spawn_tracker`). `debugpath`, `debugmobspawning`,
`spawnarmortrims`, `serverpack` (and `debugconfig`/`chase`) have no
Rust tree node at all (grep over `horizon-command/src` finds no
mention). Dev-tree only — no release player ever sees these — minor,
and the coordinator may rule dev-tree parity out of scope (the 26.2
merge kept B9-G15 as a minor gap item on the grounds that withdrawing
it unilaterally would hide a finding).

### B9-G16 — minor — Argument node names differ from vanilla

REPRODUCES — nine label pairs for 26.1.2 (26.1's seven, plus two new
ones from the SlotSource rework — `slots` vs `slot`, `sourceSlots` vs
`sourceSlot`). Proven statically from the tree-builder literals
against vanilla's oracle tree export, and confirmed empirically by
the run's node diff (§2):

| Path | Vanilla 26.1.2 (oracle `tree.json`) | Rust 26.1.2 |
|---|---|---|
| `/item replace block <…>` | `pos` | `targetPos` (`tree_misc.rs:186`) |
| `/item replace entity <…>` | `targets` | `target` (`tree_misc.rs:184`) |
| `/item replace … slot` | `slot` | `slots` (`tree_misc.rs:188`) |
| `/item replace … from block <…>` | `source` | `sourcePos` (`tree_misc.rs:200`) |
| `/item replace … from … sourceSlot` | `sourceSlot` | `sourceSlots` (`tree_misc.rs:202`) |
| `/item modify block <…>` | `pos` | `targetPos` (`tree_misc.rs:169`) |
| `/item modify entity <…>` | `targets` | `target` (`tree_misc.rs:167`) |
| `/execute if items entity <…>` | `entities` | `source` (`tree_execute.rs:231`) |
| `/execute if items block <…>` | `pos` | `sourcePos` (`tree_execute.rs:238`) |

(`from entity` agrees: both sides call it `source`. `/item modify`
`slot` agrees on the name — only the parser differs, see B9-G12.)
Node names surface in parse output (`nodes` in the fixture's jsonl)
and in client-side command feedback, so a rename is a real (if
cosmetic) divergence — minor. Run node-diff confirmation (§2.3): six
of the nine pairs confirmed empirically (slot→slots 11,
targets→target 6, pos→targetPos 5, sourceSlot→sourceSlots 4,
source→sourcePos 2, entities→source 1 occurrences at perm 4); the
`/item modify` and `/execute if items block` pairs have no corpus
case and stand on the static proof.

### B9-G17 — minor — Recipe suggestions omit special recipes (Rust suggests the learnable subset; real vanilla suggests all recipes)

REPRODUCES for 26.1.2 (same shape as 26.1.1, count 24). Real vanilla
26.1.2 DOES suggest recipe names:
`CommandSourceStack.suggestRegistryElements` has a
`Registries.RECIPE` special case (javap offsets 0–36) suggesting from
`RecipeManager.getRecipes()` — ALL recipes. Rust suggests from the
oracle-exported `learnable_recipes` list (`args.rs:1722-1723`), which
the oracle builds as the non-special recipes only
(`VanillaCommandParseOracle.java:279-280`, filtering
`!h.value().isSpecial()`): 1,491 entries, against 1,515 recipe files
in the 26.1.2 client jar. The 24 omitted are the 16 banner duplicates,
`book_cloning`, `decorated_pot`, `firework_rocket`,
`firework_star`, `firework_star_fade`, `map_extending`,
`repair_item`, `shield_decoration` (special types
`crafting_special_bannerduplicate`, `crafting_special_bookcloning`,
`crafting_decorated_pot`, `crafting_special_firework_*`,
`crafting_special_mapextending`, `crafting_special_repairitem`,
`crafting_special_shielddecoration`). So Rust omits the 24 special
recipes that real vanilla suggests — e.g. completing `recipe give @s
minecraft:black_banner_duplicate` suggests the id in real vanilla and
nothing in Rust. No corpus case covers a special-recipe prefix, so the
run does not demonstrate this either; it is verified by jar + source
+ counts. Note: when the run shows recipe suggestion mismatches, they
are a DIFFERENT thing — an oracle artifact (see B9-G11 below — the
oracle's `Src` override bypasses the RECIPE special case and suggests
nothing, while Rust suggests its learnable list). Suggestions only,
special recipes only — minor.

### B9-G18 — minor — 26.3-era subcommands in the tree (`/item fill`, `/item override`, `/execute if|unless slots`)

REPRODUCES for 26.1.2 (new in 26.1.1; the 26.1 port had none of
these). The Rust tree registers three subcommands vanilla 26.1.2
never had:

- `/item fill` and `/item override`: `tree_misc.rs:179` builds
  `for mode in ["replace", "fill", "override"]`; vanilla 26.1.2's
  `/item` node has only `modify` and `replace` (oracle `tree.json`).
  No `fill`/`override` handling exists in vanilla 26.1.2's
  `ItemCommands`.
- `/execute if slots` and `/execute unless slots`:
  `tree_execute.rs:224` builds `for kind in ["items", "slots"]`
  inside `conditionals`, which is shared by `if` and `unless`
  (`tree_execute.rs:89`); vanilla 26.1.2's `/execute if` and `/execute
  unless` nodes have `items` but no `slots`.

All three parse in Rust and fail as unknown subcommands in vanilla.
They are 26.3-form carry-overs in the re-ported tree — the same
systemic cause as G02's three extra top-level commands, recorded
separately because they are subcommand-level and G02 is the top-level
diff. Minor: gamemaster-only, no vanilla equivalent to confuse
players with.

### B9-G19 — minor — `/swing` accepts 26.3-era animation + duration arguments vanilla 26.1.2 lacks

REPRODUCES for 26.1.2 (minted from the 26.1.1 run; re-verified fresh).
Rust adds an `animation` argument (StrEnum none/whack/stab) and a
`duration` (Time) under each hand literal (`tree_world.rs:176-192`);
vanilla 26.1.2's `/swing` ends at the hand literal (oracle
`tree.json`: swing → targets → mainhand/offhand, all executable, no
children). See §2.5 for the run's verdict. Also the source of the
run's only raw-key errors (`argument.swing_animation.invalid`, a key
absent from the 26.1.2 lang file). 26.3-era arguments carried over in
the re-ported tree, same systemic cause as G18 (recorded separately:
argument-level, not subcommand-level). Minor: gamemaster-only, extra
args accepted, no vanilla behaviour broken.

### B9-G20 — minor — `/schedule clear` takes a single id where vanilla takes a greedy string

REPRODUCES for 26.1.2 (minted from the 26.1.1 run; re-verified fresh).
Vanilla's clear argument is `brigadier:string` greedy (oracle
`tree.json`: schedule → clear → function, parser `brigadier:string`,
type `greedy`), so `schedule clear hello world` parses; Rust takes
`ArgType::Id("schedules")` (`tree_misc.rs:49`) and fails at the space
(cursor 21). Single-token ids match (`schedule clear foo:bar` parses
on both). See §2.5 for the run's verdict. Minor: multi-word schedule
names are an edge case, single-token use unaffected.

### B9-G21 — minor — Scoreboard criterion suggestions include one criterion vanilla 26.1.2 lacks

REPRODUCES for 26.1.2 (minted from the 26.1.1 run; re-verified fresh).
Rust's custom-stats list (`horizon-client-core/src/scoreboard.rs:290`)
includes `sleep_in_straw_bed`; vanilla 26.1.2's `Stats` has
`SLEEP_IN_BED` and no `SLEEP_IN_STRAW_BED`, and the criterion is
absent from vanilla 26.1.2's suggestion registry (the oracle's
suggestion list comes from vanilla's own registry). See §2.5 for the
run's verdict (26.1.1: Rust suggested 9,133 criteria vs vanilla's
9,132). Minor: one extra suggestion, no parse difference.

### 3.2 Closed — the 1.21.7 defects, re-checked against 26.1.2

Each of the thirteen 1.21.7 Batch 9 defects was re-verified against
26.1.2's own jar/data. Five are CLOSED for 26.1.2; eight reproduce in
whole or part (G02, G03, G05, G07, G08, G09, G12, G13 — §3.1). The
closures fall into two groups.

**Group A — vanilla 26.1.2 itself uses the 26.x form (the 1.21.7 defect's
vanilla premise does not hold in 26.1.2):**

- **B9-G01 — CLOSED.** Vanilla 26.1.2's
  `net.minecraft.world.level.gamerules.GameRules` registers **59
  snake_case rules** (javap static initializer: `advance_time`,
  `keep_inventory`, `doFireTick` absent — there is no fire-tick rule
  in 26.1.2 at all — `spawn_mobs` / `spawn_monsters` /
  `spawn_phantoms` as separate rules, `raids`,
  `elytra_movement_check`). The Rust table
  (`horizon-client-core/src/gamerules.rs:50-126`) is the same 58
  ungated rules in the same order with the same defaults (sampled
  against the javap clinit: `fire_spread_radius_around_player`
  128/min −1, `max_block_modifications` 32768/1, `max_command_forks`
  65536/0, `max_command_sequence_length` 65536/0,
  `max_entity_cramming` 24/0, `random_tick_speed` 3/0,
  `players_nether_portal_default_delay` 80/0,
  `tnt_explosion_drop_decay` false), plus `max_minecart_speed`
  (8/1/1000) gated behind the `minecraft:minecart_improvements`
  feature flag — exactly as vanilla gates it
  (`FeatureFlags.MINECART_IMPROVEMENTS`, static initializer offset
  403–406). The `/gamerule` tree is built from
  `available_rules(minecart_improvements)` (`vanilla.rs:301`). (One
  ordering nit, same as 26.1.1: vanilla registers the gated rule at
  position 27 in the clinit, Rust chains it last via
  `available_rules` — the 58 ungated rules match in order exactly.)
  Nit, not a defect: the `MAX_MINECART_SPEED` comment says "(26.3)"
  — the rule is in 26.1.2 too (same nit as 26.1, 26.1.1 and 26.2).
- **B9-G04 — CLOSED (inverted).** Vanilla 26.1.2's `SetSpawnCommand`
  and `SetWorldSpawnCommand` use `RotationArgument.rotation()` (yaw+pitch
  pair), not `AngleArgument` — javap shows `rotation` /
  `RotationArgument.rotation()` in both `register` methods. The Rust
  `ArgType::Rotation` (`vanilla.rs:341,350`) matches vanilla 26.1.2.
- **B9-G06 — CLOSED (both parts, both inverted).**
  (a) Vanilla 26.1.2's `EntityDataSource` holds a
  `CompilableString<EntitySelector>` decoded via
  `EntitySelector.COMPILABLE_CODEC`;
  `CompilableString$CommandParserHelper.apply`
  (`net.minecraft.util.CompilableString$CommandParserHelper`) catches
  `CommandSyntaxException` and returns `DataResult.error` (exception
  table 9–69 → 70) — so decoding `{"nbt":"a","entity":"@x"}` FAILS in
  vanilla 26.1.2, the opposite of 1.21.7 (where `compileSelector`
  caught the exception and returned null). The Rust `"nbt"` arm's
  selector check (`component.rs:278-299`, `selector_string` must
  parse) rejects it too — match.
  (b) Vanilla 26.1.2 HAS `ObjectContents`
  (`network/chat/contents/ObjectContents.class`, with `AtlasSprite`
  and `PlayerSprite` in `contents/objects/`) — the Rust `object` arm
  (`component.rs:297-314`) is correct for 26.1.2.
- **B9-G10 — CLOSED (inverted).** Vanilla 26.1.2's `WorldBorderCommand`
  uses `TimeArgument.time(0)` for the `add`/`set` time argument
  (javap: call sites at offsets 51, 110, 299), not
  `IntegerArgumentType`. Rust (`tree_world.rs:334`, `ArgType::Time {
  min: 0 }`) matches — including the `d`/`s`/`t` suggestions, which
  are correct for a Time argument.

**Group C — the 26.x form was correct for 26.1.2 all along / masked:**

- **B9-G02 (bulk) — CLOSED as stated, except `posteffect`,
  `unpublish`, `compute` (§3.1).** Full top-level diff of the Rust
  release tree (79 literals from `tree_*.rs` + `vanilla.rs`, counting
  the `tell`/`w` loop at `vanilla.rs:408-409`; dev-gated
  `raid`/`warden_spawn_tracker`, Horizon-gated `pregen`, and
  dedicated-only commands excluded) against vanilla 26.1.2's
  integrated tree (oracle `tree.json` export, 76 top-level; datagen
  `commands.json` has 91 top-level, 76 integrated after removing the
  15 dedicated-only per `Commands.<init>`): **76 of 76 vanilla
  integrated commands present, zero missing; three extras,
  `posteffect`, `unpublish`, `compute`.** `stopwatch`, `swing`,
  `fetchprofile` are all in vanilla 26.1.2's tree; `/execute
  if|unless stopwatch` is in vanilla's tree (and the Rust `stopwatch`
  node, `tree_execute.rs:193-196`, matches: `id` → `range`
  float_range → redirect); the `/time` clock subcommands (`pause`,
  `resume`, `rate`, `of <clock>`, `set <timemarker>`, `query
  <timeline>`) are all in vanilla 26.1.2's `/time` node, and the Rust
  `clock_nodes` (`vanilla.rs:476-505`) matches it node for node
  (including `query gametime` only on the default clock, not under
  `of`, and `timeline repetition` under both). Note: vanilla
  registers `jfr` only when the JVM profiler is available
  (`JvmProfiler.INSTANCE != null`, `Commands.<init>` offsets
  549–564); Rust registers it unconditionally — effectively always
  available on a normal JVM, not recorded as a defect (same as 26.1,
  26.1.1 and 26.2).
- **B9-G11 — CLOSED (with the 26.1 run's correction to the 26.2
  record).** The 1.21.7 defect as stated — "vanilla suggests nothing
  for a complete recipe argument, Rust re-suggests" — does NOT hold
  against REAL vanilla 26.1.2. Real vanilla 26.1.2 DOES suggest
  recipes: `CommandSourceStack.suggestRegistryElements` has a
  `Registries.RECIPE` special case (javap offsets 0–36) suggesting
  from `RecipeManager.getRecipes()`, even though there is no
  `minecraft:recipe` registry (absent from datagen `registries.json`;
  recipes live in the `RecipeManager`). For the case in the corpus
  (`recipe give @s minecraft:stick`), the two suggestion lists agree
  by static comparison: real vanilla suggests `minecraft:stick`,
  `minecraft:stick_from_bamboo_item`, `minecraft:sticky_piston`
  (prefix matches among `getRecipes()`), and Rust suggests exactly
  the same three from its `learnable_recipes` list
  (`args.rs:1722-1723`, 1,491 entries). Note for the run: it WILL
  show recipe suggestion mismatches, but they are an ORACLE artifact,
  not a port defect — the oracle's `Src` class overrides
  `suggestRegistryElements` (`VanillaCommandParseOracle.java:93-99`)
  to look the registry up in the `RegistryAccess` and return
  `Suggestions.empty()` when absent, bypassing vanilla's RECIPE
  special case, so the oracle suggests nothing while Rust suggests
  its learnable list. (The 26.2 closure — "vanilla suggests nothing"
  — was based on the same oracle behaviour; real vanilla 26.2 has the
  same special case, so 26.2's Rust, which suggests nothing, is the
  one that diverges from real vanilla there.) The residual real
  divergence for 26.1.2 is the special recipes Rust omits — recorded
  as B9-G17 (§3.1). The suggestion mechanism
  (`lib.rs:suggestion_context`, byte-identical to 26.1.1's and 26.2's,
  which was verified against Brigadier 1.3.10's `findSuggestionContext`)
  mirrors Brigadier.

### Passes (verified, not re-diagnosed)

- **`/time set day|noon|night|midnight` — PASS in inverted form.**
  Vanilla 26.1.2's `/time set` has only the `time` and `timemarker`
  arguments (oracle `tree.json`) — the day/noon/night/midnight
  literals do not exist in 26.1.2 (they went with the clock rework).
  The 26.1.2 Rust `clock_nodes` has none either (no matches for
  day/noon/night/midnight in `horizon-command/src`). Both sides agree:
  `time set day` parses only via the `timemarker` (resource-location)
  argument on both. The 1.21.7 pass ("literals present") does not
  transfer as stated; the 26.1.2 check is the matching absence, and
  it holds. (This pass is the 20th behaviour check in the tally;
  see §6.)
- **Error-text resolution at runtime — PASS (static).** The game
  resolves command error keys correctly (see B9-O04, §4 — the fixture
  language file is present and jar-identical); the run's error-text
  mismatch count is reported in §2.

### 3.3 Harness — B9-H01 FIXED for 26.1.2

**B9-H01 — CLOSED (fixed).** In 26.1 the `client_probe` example did
not compile (`game_loop_probe.rs:159-160` set `focused` and
`exclusive_fullscreen` on a `FrameContext` that lacked both fields),
blocking the `command_parse` run. In 26.1.1/26.1.2 the port re-ported
`FrameContext` from the 26.3 form
(`crates/horizon-client-input/src/window.rs:56-66`, with `pub focused:
bool` and `pub exclusive_fullscreen: bool`), so the same two probe
lines are valid initialisers and the probe file is unmodified. Batch 0
verified this with `cargo check` (EXIT 0, including a forced recheck
with rewritten example artifacts); this batch verified it with a full
`cargo build --profile fast -p horizon-client-lib --example
client_probe` as part of the driver run (§2) and ran the
`command_parse` driver end-to-end on it. H01 blocks no harness for
26.1.2.

## 4. Oracle / fixture defects

### B9-O01 — minor — `VanillaScoreboardOracle` fails javac (3 errors) — blocks `scoreboard`

REPRODUCES for 26.1.2 (same as 26.1 and 26.1.1; it passed in 26.2 —
26.1.2 predates `TeamColor`). Batch 0's 26.1.2 measurement
(`docs/sweep/26.1.2/batch-0-oracles.json`, verified identical to the
26.1.1 table): fails all three classpaths, `missing-symbol`, 3
errors, first at line 242. The two `TeamColor` sites fail:

- Line 242 (`t.getColor().map(TeamColor::getSerializedName)`):
  `cannot find symbol` — there is NO `TeamColor` class in the 26.1.2
  server jar (verified by jar listing). 26.1.2's
  `PlayerTeam.getColor()` returns `ChatFormatting` (javap:
  `PlayerTeam.javap`).
- Line 436 (`t.setColor(… Optional.of(TeamColor.byName(…)))`):
  `incompatible types` + `cannot find symbol` — 26.1.2's
  `PlayerTeam.setColor` takes a bare `ChatFormatting`, not an
  `Optional` (javap), and `TeamColor.byName` does not exist.

Repair scope: small, in-family, NOT attempted (no fixes in this
batch): `TeamColor::getSerializedName` → `ChatFormatting`'s own
`getSerializedName` (it implements `StringRepresentable`),
`TeamColor.byName` → `ChatFormatting.getByName`, drop the `Optional`
wrapper at line 436. Needs an end-to-end driver run after repair
(compiling ≠ running).

### B9-O02 — minor — `VanillaTeamRulesOracle` fails javac (6 client errors) — blocks `team_rules`

REPRODUCES for 26.1.2 (same as 26.1 and 26.1.1; worse than 26.2's 2
errors: 26.1.2 predates the `Hud` split). Batch 0's 26.1.2
measurement: fails all three classpaths, `missing-symbol`, 6 errors,
first at line 14. The server-cp failure is expected — the driver uses
the client jar, so the client cp is the operative one.

- Line 14 (`import net.minecraft.client.gui.Hud`): `cannot find
  symbol` — no `Hud` class in the 26.1.2 client jar (verified by jar
  listing). The `Hud` split happened after 26.1.2; 26.1.2's `Gui` has
  no `hud` field and the HUD-hidden flag is `Options.hideGui` (public
  boolean).
- Lines 188 (×2), 189: the three `Hud` uses (`Hud hud =
  alloc(Hud.class)`, `put(hud, Hud.class, "isHidden", …)`,
  `put(gui, Gui.class, "hud", hud)`) — all downstream of the missing
  import.
- Line 101 (`Viewer()`): `super(null, null, null, null, null, null,
  false, null, null)` — 9 arguments. 26.1.2's `LocalPlayer`
  constructor takes 8: `(Minecraft, ClientLevel, ClientPacketListener,
  StatsCounter, ClientRecipeBook, Input, boolean, ChatAbilities)`
  (javap on the 26.1.2 client jar, `LocalPlayer.javap`). Drop one
  argument to match (same as 26.2).
- Line 197: `own.doTeamsAllowDamage(other)` — no such method in
  26.1.2 (`Entity` has no `doTeamsAllowDamage`; the 26.1.2 form is
  `PlayerTeam.isAllowFriendlyFire()`, which exists). Rework the damage
  check against the team's friendly-fire flag (same as 26.2).

Repair scope: medium (larger than 26.2's), in-family, NOT attempted.
Needs an end-to-end driver run after repair.

### B9-O03 — minor — `LocatorBarClient` fails javac (13 errors) — blocks `exploration_map_command`

REPRODUCES for 26.1.2 (same as 26.1 and 26.1.1; worse than 26.2's 10
errors: 26.1.2 predates both the `EntityTypes` move and the 6-arg
teleport-accept packet). Batch 0's 26.1.2 measurement: fails all
three classpaths, `missing-symbol`, 13 errors, first at line 393.
Causes, verified against the 26.1.2 client jar:

- Lines 393–395, 407, 409, 413: `packet.removed()` / `packet.added()`
  / `packet.progress()` on `ClientboundUpdateAdvancementsPacket` —
  26.1.2 uses getter names: `getRemoved()`, `getAdded()`,
  `getProgress()` (`shouldReset()` exists as-is, so line 405 is fine).
  Six call sites to rename (same as 26.2).
- Line 668: `canUseCommandBlocks()` override — no such method on the
  supertype in 26.1.2 (`GameProtocols$Context`
  (`net.minecraft.network.protocol.game.GameProtocols$Context`) has
  only `hasInfiniteMaterials()`). Drop the override (same as 26.2).
- Lines 683, 906, 911: `net.minecraft.world.entity.EntityTypes.PLAYER`
  — no `EntityTypes` class in 26.1.2 (registration still lives in
  `EntityType`, the 1.21.x shape). Use `EntityType.PLAYER`. Three
  sites (new vs 26.2).
- Lines 690, 916: `removed.entityIds()` / `rm.entityIds()` on
  `ClientboundRemoveEntitiesPacket` — 26.1.2's accessor is
  `getEntityIds()`. Two sites (same as 26.2).
- Line 977: `new ServerboundAcceptTeleportationPacket(t.id(), x, y, z,
  yaw, pitch)` — no suitable constructor: 26.1.2's packet has only the
  1-arg `(int)` constructor (the 6-arg form arrived later). Send the
  id-only packet (new vs 26.2).

Repair scope: per-symbol renames against the 26.1.2 client jar (the
five named fixes cover all 13 errors). Medium; NOT attempted — and
out of family for a commands batch (packet-client helper shared with
the locator-bar harness; the 26.1 merge recorded the B3/B9 member
fold for this oracle — expect the same overlap for 26.1.2's Batch 3).

### Closed oracle / fixture defects

- **B9-O04 — CLOSED.** The `command_parse` fixture loads language from
  the prepared-resources root (`command_parse_fixture.rs:147`,
  `lang::Language::load(&[&root], …)`; root = `lab.RESOURCES` =
  `.local/horizon-client/26.1.2/`). Unlike 1.21.7, the prepared
  26.1.2 root DOES contain `assets/minecraft/lang/en_us.json` —
  md5-identical to the 26.1.2 client jar's file
  (`98deafdb5e90e4438db30853494e1aeb`, 7,886 keys, including every
  command error key the driver compares). The fixture's error-text
  leg is readable for 26.1.2: the 1.21.7 defect was the file's
  ABSENCE, and the file is present and jar-identical here, so the
  86,241-mismatch artifact of the 1.21.7 run cannot recur on that
  account. Run confirmation (§2.6): error-text-only mismatches are
  2,858 (perm 4) / 2,857 (perm 2) — all genuine message differences
  (SlotSource G12, codec G07a, place G03), with 4 raw-key
  emissions in the whole run (2 unique swing inputs, both
  `argument.swing_animation.invalid`, the 26.3-era key); no unresolvable-key artifact,
  and the 1.21.7-style blowup did not recur.

### Passing in-scope oracles (auditable coverage)

From the Batch 0 measured table (`docs/sweep/26.1.2/batch-0-oracles.json`,
verified identical to the 26.1.1 table):
`VanillaCommandParseOracle` (pass all three classpaths),
`VanillaCommandExecOracle` (pass all three),
`VanillaCommandSuggestionsExporter` (pass all three). The
`command_parse`/`command_exec` linchpin drivers are oracle-clean for
26.1.2 from the start.

## 5. Driver table (22)

| Driver | Oracle | Status |
|---|---|---|
| command_parse | VanillaCommandParseOracle (pass) | **MEASURED — see §2.** 34,542 commands × 3 perms; 3,162 / 3,148 / 1 mismatches (perm 4 / 2 / 0); node diff 34 / 29 / 0 inputs. Outputs byte-identical to the 26.1.1 run. |
| command_exec | VanillaCommandExecOracle (pass) | Not run. Needs full game build + Java integrated server. Known fixture blocker (same as 1.21.7/26.1/26.1.1/26.2): `/execute if predicate effect_view:always` — the `effect_view:*` predicates come from `tools/mob_effect_predicate_corpus.py`, imported by `command_exec_corpus.py:64`. Recorded, not chased. |
| scoreboard | VanillaScoreboardOracle (FAIL) | Oracle-blocked — B9-O01 (3 errors; 26.2's cheapest remaining driver is blocked again for 26.1.2) |
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
gates the family. All 22 driver scripts exist in the 26.1.2 tree
(`tools/test_*_parity.py`, verified by name).

## 6. Method notes

- **Run + static.** The one measurement available (`command_parse`,
  §2) was run end-to-end — the 26.1.2 probe compiles, so the 26.1
  run's blocker is gone. Every verdict below that would have rested
  on run data ALSO rests on a static proof stated in its section
  (javap + source + datagen + oracle data); the run confirms rather
  than carries them. Where the corpus cannot reach a defect
  (dedicated/dev trees, special-recipe suggestions, node names the
  driver's `compare()` strips), the static proof is the verdict and
  the report says so.
- **Vanilla 26.1.2's own jar/data is the spec throughout.** Every
  behaviour claim above is grounded in one of: the oracle's exports
  (`commands.json`, `command-registries.json`, `tree.json`,
  `registries.json` — real vanilla 26.1.2 dispatcher output), `javap
  -c` on the named 26.1.2 server-game jar (Mojang-mapped, so no
  mappings front-end was needed) or the 26.1.2 client jar, the
  vanilla 26.1.2 datagen reports, or Brigadier 1.3.10 from the
  datagen libraries. The 26.3 base's form was never trusted — and
  this batch is the proof of why: the port itself trusted it, and the
  defects below are the result.
- **Tally scheme (stated explicitly — the 26.1.1 run needed
  restatements anyway).** 24 as-tallied (7 pass, 17 gap) = 20
  behaviour (6 pass, 14 gap) + 4 non-behaviour (1 pass, 3 gap: H01 +
  O01/O02/O03). The 20 behaviour checks are: G01–G13 with G07 split
  into its two parts (14 checks: G01, G02, G03, G04, G05, G06, G07a,
  G07b, G08, G09, G10, G11, G12, G13) + G14, G15, G16, G17, G18 (5) +
  the `/time set` inverted pass (1). Behaviour passes: G01, G04, G06,
  G10, G11 + `/time set`. Behaviour gaps: G02, G03, G05, G07a, G07b,
  G08, G09, G12, G13 + G14, G15, G16, G17, G18. G19/G20/G21 are
  run-minted ITEMS (26.1.1 precedent: minted at merge, not counted as
  checks); the error-text-resolution pass is O04's confirmation, not
  a separate check. The behaviour-only restatement is the
  coordinator's job (merge.py).
- **Systemic pattern**: unchanged from 26.1.1 — the 26.1.2 command
  surface IS the 26.1.1 command surface (every in-scope file
  byte-identical, §0). The port carries 26.3-era tree code against a
  26.1.2 vanilla that predates it, 26.2-era oracle code (`TeamColor`,
  `Hud`, `EntityTypes`) against a 26.1.2 vanilla that predates those,
  and 1.21.7-era parse code (particle/modifier handling) that the
  26.2 port fixed but neither the 26.1, the 26.1.1 nor the 26.1.2
  port carried. The remaining risk is execution, not parsing: 18 of
  22 drivers are unrun e2e drivers, and `command_parse` only covers
  parse behaviour.
- **26.1.1-record corrections (from this run).** (1) The dedicated
  list: `kick` is integrated (registered unconditionally at
  `Commands.<init>` offset 216) and `transfer` is dedicated
  (registered inside the `includeDedicated` block at offset 752) —
  the 26.1.1 report had them backwards. The oracle's INTEGRATED tree
  export confirms (76 top-level: `kick` in, `transfer` out). G02's
  outcome and G14 are unaffected. (2) Line numbers: G12
  `tree_misc.rs:171,188,202` / `tree_execute.rs:240`, G18
  `tree_misc.rs:179` / `tree_execute.rs:224`, G10
  `tree_world.rs:334` — the 26.1.1 report's numbers for these were
  off by 1–2; the files are byte-identical, so the numbers here are
  correct for both trees.
- **What the corpus cannot see**: it walks vanilla's tree, so
  Rust-only extras appear only via hand-written edge cases (the
  corpus carries `posteffect`, `posteffect @s minecraft:x`,
  `compute`, `unpublish`, `raid start 5`). The tree diff (§3, B9-G02)
  is the complete check for top-level extras; the corpus is the
  complete check for missing/wrong vanilla behaviour. The dedicated
  tree (`/perf`) and the dev tree are outside both: the corpus walks
  the integrated tree, and the dev tree is not built by
  `release_dispatcher()` — B9-G14/G15 were caught by the static diff
  against `Commands.<init>` bytecode, not by any run. The
  subcommand-level extras (B9-G18) are likewise outside the corpus:
  no corpus case walks `/item fill`, `/item override`, or `execute
  … slots`, because vanilla's tree has no such nodes to walk.
- **Counts**: 103,626 comparisons (34,542 commands × 3 permission
  levels), 97,315 match, 6,311 mismatches across 3,162 unique inputs
  (§2); node diff 34 / 29 / 0 inputs (perm 4 / 2 / 0). Driver table:
  1 measured + 3 oracle-blocked + 18 not run = 22.

## 7. Open questions / follow-ups for the coordinator

0. **The re-port lesson, still unpaid.** Five 26.1 closures flipped
   because the 26.1.1 port re-ported command code from the 26.3 form
   instead of carrying the 26.1 tree's adaptations (G03, G05, G07b,
   G12, G13) — and the 26.1.2 delta port inherited them verbatim.
   The fixes are the 26.1 tree's forms, already written once:
   `ResKey("minecraft:worldgen/configured_feature")` for `/place
   feature`, the gamemode argument back in `/publish`, single-entity
   `from entity` source, `Slot`/`Slots` for `/item` and `execute
   items`, and the 57-entry argument-type table with `color` at index
   16. Any fix carried across versions must be version-gated: 26.2's
   vanilla wants `team_color`, the selector refactor, and
   `unpublish`/`compute`, so the 26.1.2 fixes must NOT be copied
   forward blindly.
1. The 18 unrun e2e drivers (esp. `command_exec` + its 7 dependents)
   are the batch's real remainder: they need a full client build and
   the `effect_view:always` fixture blocker resolved. Recommend
   running `command_exec` first (with `--allow-gaps`), then its
   dependents.
2. `scoreboard` is NOT the cheapest remaining driver for 26.1.2 (it
   was for 26.2): its oracle fails again on `TeamColor` (B9-O01). The
   repair is small (two sites, `TeamColor` → `ChatFormatting`), so it
   is still the cheapest oracle repair in the batch — but it is a
   repair, not a free run.
3. B9-O02 (`team_rules`) is a medium repair (6 client errors, fully
   scoped in §4) — the `Hud` rework is the new work vs 26.2.
4. B9-G14 (`/perf` in the dedicated tree) and B9-G15 (dev tree) are
   both tree-completeness gaps found by the static diff; neither is
   visible to the `command_parse` corpus (integrated tree only). If
   dev-tree parity is out of scope, G15 can be withdrawn (the 26.2
   merge kept it as a minor gap).
5. B9-G09 (selector) is the batch's most instructive finding: the
   Rust selector parser implements the 26.2 option-state shape, but
   vanilla 26.1.2 still has the old parser. A fix here must target
   26.1.2's form (no rollback on inapplicable, `!isTypeLimited` for
   `type`), NOT the 26.2 form — and any fix carried across versions
   must be version-gated, since 26.2's vanilla wants the opposite
   behaviour for `@a[type=player]`.
6. B9-G07a (item-modifier `function` key) and B9-G08 (particle `Name`
   compound) are fixes the 26.2 port already made — both are direct
   carry-overs from the 26.2 tree (`codec.rs` key-parameter dispatch,
   `args.rs` `Name`-accepting `particle_options`). They are the
   cheapest behaviour fixes in the batch.
7. B9-G02's three extra commands are one-command removals each from
   the tree builders — but check whether anything else in the 26.1.2
   tree references the nonexistent `post_effects` registry before
   removing `posteffect` (the `ArgType::Id("post_effects")`
   arguments at `tree_extra.rs:40,50` die with the command), and
   whether `compute`'s `context_sources` helper (`tree_data.rs:32-48`)
   is shared with `/data` before removing `compute`. (Run answer,
   §2.5: yes, it IS shared — 17 `data modify …` suggestion
   mismatches, all rust-only `compute`, same as 26.1.1.)
8. The oracle's `Src.suggestRegistryElements` override
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
9. B9-G16 (node names) is proven statically here and confirmed by the
   run's node diff (§2): the node diff is the only check that sees
   the names the fixture actually emits (as opposed to the literals
   in the tree builders).
