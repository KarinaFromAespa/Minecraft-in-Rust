# Batch 9 — Commands (26.1 parity sweep)

Date: 2026-11-24. Branch: `parity-sweep-26.1` (synced to main).
Scope: parsing, execution, selectors, functions, scoreboard — 22 drivers;
57 argument types (26.1's own list), 59 game rules.
Ledger slice: `slice.json` (this directory).

## 0. Execution status — read first

**The `command_parse` run is BLOCKED, not measured.** This batch
attempted it four times via `tools/test_command_parse_parity.py` (§2):
the first three attempts were killed by infrastructure (two VM reboots,
one unexplained process kill), and the fourth reached the end of the
`fast`-profile build only for the `client_probe` example to fail to
compile — 2 errors, both in `game_loop_probe.rs`, a probe module the
26.1 port carried verbatim from 26.3 against a 26.1 `FrameContext`
that deliberately lacks the two fields it sets (B9-H01, §3.3). The
failure is deterministic: relaunching fails identically at the same
point. There is no `report.json`, no per-permission output, and no
mismatch data for 26.1. What the attempts DID produce, and what is
preserved in this directory: the oracle compiled clean, the 34,542-
command corpus was generated (identically, three times) from the
oracle's own 26.1 tree export, and the `horizon-client-lib` library
itself compiled — only the `client_probe` example binary is blocked.
A two-line fix to the probe file (deleting the two fields, exactly as
the 26.2 port's adapted probe does) unblocks the run; the incremental
rebuild after the fix is minutes, not hours, because the library is
already built. The fix was NOT attempted: this batch is checks +
triage only, and the probe file is `versions/26.1/` source.

Everything else in this report is static triage (method 2:
javap + source + oracle data + datagen), and it stands on its own:
the four reproducing 1.21.7 defects, the four new defects, and the
three broken oracles were all verified without the run. Where the
26.2 report leans on run data, this report says so explicitly and
gives the static evidence instead — in particular B9-G16 (node names),
whose 26.2 form came from the run's node diff, is proven here from
the tree-builder literals against vanilla's datagen dump.

Prepared data used: `versions/26.1/.local/horizon-client/26.1/`,
including `commands.json`, `command-registries.json`
(VanillaCommandParseOracle export), `registries.json`, and
`assets/minecraft/lang/en_us.json` (md5-identical to the 26.1 client
jar's file, 7,886 keys). `sounds.json` is absent from the prepared root
(the VM lacks the official launcher's asset index) — expected, not a
port defect, and irrelevant to this batch: no command check needs it.

Nothing in this batch touched production code (`crates/`); no oracle
was repaired (the three broken in-scope oracles are scoped in §4, not
fixed); no git commits were made. All behaviour claims are grounded in
26.1's OWN jar/data — the named server-game jar
(`26.1.x/versions/26.1/jars/server-game.jar`, Mojang-mapped), the 26.1
client jar, the vanilla 26.1 datagen reports, or the oracle's own
exports. The 26.3 base's form was never trusted — and the headline
finding of this batch is how much that mattered in the other direction
from 26.2: **26.1 is OLDER than 26.2 in exactly the ways this batch
measures.** Four of the thirteen 1.21.7 defects REPRODUCE for 26.1
(G02 in part, G07 in part, G08, G09) because the 26.2 port fixed them
or vanilla 26.2 changed — and 26.1 has neither the fixes nor the
vanilla changes. The 26.1 port carries the 26.2-form selector parser
against a vanilla 26.1 that still has the old parser, the 1.21.7-form
particle/item-modifier code, and a `/posteffect` command vanilla 26.1
never had.

## 1. Tally

**23 checks: 11 PASS, 12 GAP.** 8 game defects (1 major, 7 minor) +
3 oracle defects (0 major, 3 minor) + 1 harness blocker (major,
B9-H01 — the `command_parse` fixture does not compile). No fixes —
checks + triage only. The `command_parse` measurement is NOT counted:
it was attempted four times and blocked every time (§2), so no
measurement exists to pass or fail.

Check→item map: the 13 re-checks of the 1.21.7 defects (B9-G01–G13)
split 9 pass / 4 gap for 26.1 — G02 (posteffect only), G07 (item
modifier `function` key only), G08 (particle `Name` compound) and G09
(selector option errors, both parts) reproduce; the other nine close.
The two 1.21.7 passes were re-verified: the `/time set` literals check
PASSES in inverted form (vanilla 26.1 has no day/noon/night/midnight
literals and neither does the port), and the 57 argument-type list
PASSES exactly (index 16 is `minecraft:color` on both sides — 26.1
predates the `team_color` rename). The remaining 8 gaps are the two
tree gaps carried from 26.2 (B9-G14 dedicated `/perf`, B9-G15 dev
tree), the argument-name labels (B9-G16 — proven statically from the
tree literals vs the datagen dump, §3.1), the recipe special-recipe
omission (B9-G17), the three still-broken oracles (B9-O01 scoreboard,
B9-O02 team rules, B9-O03 locator bar), and the harness blocker
(B9-H01). B9-O04 (fixture language) is CLOSED. Arithmetic: 13
re-checks (9 pass, 4 gap) + 2 re-verified passes + 8 new gap items =
23 checks; 11 pass, 12 gap.

Driver tally (22 in scope): 0 measured, 1 build-blocked
(`command_parse` — B9-H01, §2), 3 oracle-blocked (`scoreboard`,
`team_rules`, `exploration_map_command`), 18 not run — all game-binary
e2e drivers needing a full `fast`-profile client build plus a Java
reference run (§5).

Counting rule: behaviour checks only (both the vanilla side and the
Rust side actually compared). Oracle-status checks are not counted in
the tally; oracle defects that reproduce are counted once each as gap
checks, following the 26.2 run's convention. The harness blocker
(B9-H01) is counted once as a gap check on the same principle — it is
a verified defect in the 26.1 tree, not a status note — and it takes
the place the measurement check would have held: the batch verified
the blocker instead of the measurement. If the coordinator prefers
the blocker untallied, the tally is 22 checks (11 pass, 11 gap).

## 2. The command_parse run — BLOCKED (B9-H01)

There is no measurement table for 26.1: the run never reached a
single parse. What follows is the attempt record, so the coordinator
can see exactly how far it got and what unblocks it.

**What was ready.** The corpus: **34,542 commands** (tree-walk of the
oracle's own 76-command integrated tree export + hand-written edge
cases: `swing @s`, `stopwatch …`, `fetchprofile`, `unpublish`,
`publish true survival 25565`, `compute`, `raid start 5`, `execute if
stopwatch …`, 484 `/gamerule` variants, `posteffect`, `posteffect @s
minecraft:x`, `recipe give @s minecraft:stick`). It was generated
three times (once per driver launch that reached the corpus step) and
came out byte-identical every time (2,095,436 bytes, md5
`aa28f318ef41b52e2cc75a727e290317`) — the corpus is deterministic and
preserved in this directory as `command-parse-corpus.txt`, with the
oracle's `tree.json` and `registries.json` beside it. The oracle
compiled clean (`VanillaCommandParseOracle`, javac `--release 25`,
no errors) and the Java registries/tree exports ran. The
`horizon-client-lib` library itself compiled (its rlib/rmeta are in
`target/fast/deps/`). The ONLY thing that failed is the
`client_probe` example binary — and only because of one probe module
it links unconditionally.

**Attempt record (all times UTC 2026-11-24).** The driver was launched
detached (`setsid nohup`, `run-command-parse.sh`: `--profile fast`,
`-j 1`, `taskset -c 0`, `RUSTFLAGS=-C codegen-units=1`) four times:

1. 07:36 — killed by a VM reboot at ~07:45 (`up 9 min` at 07:53:57),
   during the early crates (5 compiled).
2. 07:54 (pid 6046) — java steps redone by 08:04, cargo to 92 crates
   by 08:22, then the driver AND cargo died at ~08:22:37 with no
   error, no reboot (`up 53 min`), and no OOM trace in the journal —
   the process tree was reaped externally, most plausibly shared-cell
   memory pressure (another agent's `cargo check` on the 1.21.10 port
   was running on the same 6 GB cell later the same morning; the
   `horizon-client-lib` rustc peaks near 2.9 GB RSS).
3. 08:38 (pid 21595) — resumed from the warm cargo cache, 189 crates
   by 09:46 (`horizon-telemetry`), then a SECOND VM reboot at ~09:47
   (`up 0 min` at 09:47:07).
4. 09:47 (pid 1870) — resumed again; `horizon-client-lib` compiled
   10:13–10:28 (rustc peaking at 2,882 MB RSS); then the
   `client_probe` example failed at 10:28–10:31 and the driver exited
   `compile-rust failed (101)`.

**The failure (deterministic, not a flake).** The driver's build step
is `cargo build --profile fast -p horizon-client-lib --example
client_probe` (`test_command_parse_parity.py:67`). The example is a
single binary (`crates/horizon-client/examples/probe/main.rs`) that
declares every probe module unconditionally (`mod game_loop_probe;`
line 59, `mod command_parse_fixture;` line 34) — there is no feature
gate or per-probe build, so one broken module blocks the
`command_parse` fixture even though the fixture never executes it.
The errors (complete — exactly two, `command-parse-driver.log`):

    error[E0560]: struct `FrameContext` has no field named `focused`
       --> crates/horizon-client/examples/probe/game_loop_probe.rs:159:9
    error[E0560]: struct `FrameContext` has no field named `exclusive_fullscreen`
       --> crates/horizon-client/examples/probe/game_loop_probe.rs:160:9
    error: could not compile `horizon-client-lib` (example "client_probe") due to 2 previous errors

**Cause.** 26.1's `game_loop_probe.rs` is byte-identical to 26.3's —
the 26.1 port carried the probe file verbatim. But 26.1's
`FrameContext` (`horizon-client-input/src/window.rs:56`) deliberately
has only 7 fields (`inactivity`, `iconified`, `now_ms`,
`latest_input_ms`, `has_level`, `has_screen`, `has_overlay`); the
doc comment on `throttle_reason` says "26.3 also throttles an
unfocused window in exclusive fullscreen" — i.e. `focused` and
`exclusive_fullscreen` are 26.3-only by design. The 26.2 port adapted
its probe (26.2's `game_loop_probe.rs` has neither field, and its
`on_move` call takes only the absolute position, with a comment
explaining the 26.2/26.3 difference); 1.21.11's was adapted too. The
26.1 port adapted neither. Note 26.1's `on_move` still takes the
26.3 6-argument form (`mouse.rs:183`), so the probe's `move` handler
typechecks — the fix is ONLY the two field lines. The same
unadapted probe file is in 26.1.2, 26.1.1 and 1.21.7, so those
versions' `command_parse` runs would hit the same wall; 26.2 and
1.21.11 are clean.

**What unblocks it.** Delete the two lines (`focused: …`,
`exclusive_fullscreen: false`) from the `FrameContext` literal in
`game_loop_probe.rs:159-160`, exactly as the 26.2 port's probe does.
Two lines, in a probe file, zero behaviour impact on command parsing
(the game-loop probe is never executed by the `command_parse`
fixture). After the fix the incremental rebuild is minutes (the
library is already compiled; only the example re-links), and the
driver then runs the three permission levels from the preserved
corpus. The fix was NOT attempted by this batch (checks + triage
only; the file is `versions/26.1/` source). A patched-copy build
outside the worktree was considered and rejected: cargo fingerprints
are path-bound, so a copied tree needs a full cold rebuild (2+ hours
on this VM, likely interrupted again), and it would deviate from the
mandated run-the-driver method. The coordinator's call.

**Consequences for this report.** Every verdict below is static
(javap + source + datagen + oracle data). The run-dependent
confirmations the 26.2 report leans on are absent: there is no
mismatch count, no per-permission table, no error-text confirmation,
and no node diff from run output. B9-G16 (node names) is proven
statically instead (§3.1). The `node_diff.py` and
`classify_mismatches.py` scripts in this directory are ready for the
rerun whenever the fix lands.

## 3. Defects

### 3.1 Open defects

### B9-G02 (partial) — minor — `/posteffect` in the release tree; vanilla 26.1 has no such command

REPRODUCES in part (1 of the 1.21.7 extras remains). The full top-level
diff (§3.2, G02) shows the Rust release tree (77 literals) against
vanilla 26.1's integrated tree (76 commands): exact match except
`posteffect`. Vanilla 26.1 has no `PostEffectCommand` class in the
server-game jar, no `posteffect` in the datagen `commands.json` dump,
and no `PostEffectCommand.register` in `Commands.<init>`; there is not
even a `minecraft:post_effects` registry in 26.1's `registries.json`,
so the tree's `ArgType::Id("post_effects")` argument
(`tree_extra.rs:40,50`) references a registry that does not exist.
The Rust tree registers it unconditionally in `tree_extra::register`
(`tree_extra.rs:36`, called by `registered_with_features` for every
tree including `release_dispatcher()`). The 26.2 port removed it; the
26.1 port did not. The corpus carries two `posteffect` edge cases
(`posteffect`, `posteffect @s minecraft:x`) for the rerun — on them
Rust parses and vanilla reports unknown command, by the tree diff
above. (The run itself is blocked, §2.) Minor: one command,
gamemaster-only, no vanilla equivalent to confuse players with — but
it is a real divergence in the release tree, and the `post_effects`
argument can never resolve.

### B9-G07 (partial) — minor — Item modifiers: inline JSON fails (wrong type key)

REPRODUCES in part (a only; b is closed). The 26.1 `dispatch()` in
`codec.rs:41-67` reads the `"type"` key unconditionally, and the
`minecraft:item_modifier` arm (`codec.rs:171-173`) calls it directly —
so `{"function":"minecraft:set_count","count":1}` fails with a missing
`"type"` key error. Vanilla 26.1 loot data uses the `"function"` key
(client jar `data/minecraft/loot_table/entities/zombie.json`:
`"function": "minecraft:set_count"`, `"function":
"minecraft:enchanted_count_increase"`). The 26.2 port fixed this:
26.2's `dispatch` takes the key as a parameter and the item-modifier
arm dispatches on `"function"` (`with_alternative`, 26.2
`codec.rs:203-212`, with a unit test proving the inline form decodes).
The 26.1 port carries the 1.21.7 form. (The run that would flag it
empirically is blocked, §2 — the verdict here rests on the source +
loot data.) Part (b) is closed: `/item … from entity <source>`
uses `ArgType::Entity { single: true, … }` (`tree_misc.rs:200-208`),
matching vanilla 26.1's `ItemCommands` (`EntityArgument.entity()` for
`source`, 2 call sites; `entities()` for `targets`).

### B9-G08 — minor — Particle block_state compound with Name is rejected

REPRODUCES. Rust 26.1 `particle_options` (`args.rs:1048-1060`) returns
an error for ANY non-string `block_state` value — the compound branch
builds an error mentioning the `id` key and returns it. Vanilla 26.1's
`StateHolder.codec` dispatches on `"Name"` (javap:
`Codec.dispatch("Name", …)`, `StateHolder.java` `codec` method, `ldc
"Name"` at offset 1) — the `{Name, Properties}` map form is the form
vanilla's codec takes, so `block_state:{Name:"minecraft:stone"}` is
valid in vanilla and rejected by the port. The 26.2 port fixed this
(26.2 `args.rs` accepts a compound with a string `Name` and returns
`Ok(())`, with a comment noting 26.2's codec dispatches on `Name`
while 26.3's map form uses `id`); the 26.1 port carries the 1.21.7
form. (The run that would flag it empirically is blocked, §2 — the
verdict here rests on the source + javap.)

### B9-G09 — minor — Selector option errors: wrong cursor on duplicates; type= accepted for @a/@p/@r

REPRODUCES (both parts). Vanilla 26.1 still has the OLD selector
parser — the option-state refactor (`InvertableSetOptionState`,
`SetOnceOptionState`) that closed this defect for 26.2 had NOT
happened yet in 26.1 (neither class exists in 26.1's
`EntitySelectorOptions`/`EntitySelectorParser`). The Rust parser
(`selector.rs`) implements the 26.2 option-state shape, so it diverges
from vanilla 26.1 in both parts:

(a) Duplicate options: vanilla 26.1's `EntitySelectorOptions.get()`
runs the `canUse` gate BEFORE the value is read (called from
`parseOptions` right after the option name is read), but — unlike
26.2 — it does NOT roll back: only the unknown-option branch does
`setCursor(start)`; the inapplicable branch throws
`ERROR_INAPPLICABLE_OPTION` at the cursor as it stands, i.e. at the
END of the option name (before `=`). Rust rolls back to the option-name
START (`selector.rs:333-341`, `rollback(reader, start, …)`). For
`@e[type=pig,type=cow]` the cursors differ by the option-name length
(4 for `type`). Note the three-version shape: 1.21.7 vanilla reported
at the end of the VALUE (check in the handler), 26.1 vanilla at the end
of the NAME (check in `get()`, no rollback), 26.2 vanilla at the START
of the name (`rollbackAndThrow`) — and Rust reports at the start in all
three, matching only 26.2.

(b) `@a[type=player]`: vanilla 26.1's `type` option `canUse` is
`!parser.isTypeLimited()` (`lambda$bootStrap$42`), and `@a`/`@p`/`@r`
all call `limitToType(EntityType.PLAYER)` during `parseSelector`
(offsets 165, 195, 224), which sets the `type` field — so vanilla
26.1 REJECTS `@a[type=player]` with `ERROR_INAPPLICABLE_OPTION`. Rust
pushes `minecraft:player` into `s.types` for `@a`/`@p`/`@r`
(`selector.rs:218-242`) without marking `type_option`, so
`can_use("type")` stays true and Rust ACCEPTS it (the `handle("type")`
comment at `selector.rs:591` — "limitToType replaces the player type
of @a/@p/@r" — records the 26.2 intent). (The run that would flag
both parts empirically is blocked, §2 — the verdict here rests on
the javap + source.)

### B9-G14 — minor — `/perf` missing from the dedicated tree

Carried from 26.2, verified fresh for 26.1. Vanilla 26.1 registers
`PerfCommand` inside the `includeDedicated` block of
`net.minecraft.commands.Commands` (javap: `PerfCommand.register` at
bytecode offset 710, between `PardonIpCommand` and `SaveAllCommand`,
inside the `includeDedicated` guard at 651–762 — the same offsets as
26.2). The command is `perf start|stop`, owners permission
(datagen `commands.json` agrees). The Rust release (integrated) tree
correctly lacks it, but the dedicated tree
(`tree_extra.rs:register_dedicated`, 14 commands) lacks it too — the
only one of vanilla's 15 dedicated commands not registered. Single
command, dedicated-only, profiling use — minor.

### B9-G15 — minor — Dev tree is missing six vanilla 26.1 dev commands

Carried from 26.2, verified fresh for 26.1. Vanilla 26.1's dev block
(`SharedConstants.DEBUG_DEV_COMMANDS || IS_RUNNING_IN_IDE`, `Commands`
constructor offsets 580–632) registers `raid`, `debugpath`,
`debugmobspawning`, `warden_spawn_tracker`, `spawnarmortrims`,
`serverpack` (plus `debugconfig` when dedicated, plus `chase` under
the separate `DEBUG_CHASE_COMMAND` flag — the same layout as 26.2).
The Rust dev tree has only the two `if dev` blocks in
`vanilla.rs:161,385` (`raid`, `warden_spawn_tracker`). `debugpath`,
`debugmobspawning`, `spawnarmortrims`, `serverpack` (and
`debugconfig`/`chase`) have no Rust tree node at all (grep over
`horizon-command/src` finds no mention). Dev-tree only — no release
player ever sees these — minor, and the coordinator may rule dev-tree
parity out of scope (the 26.2 merge kept B9-G15 as a minor gap item on
the grounds that withdrawing it unilaterally would hide a finding).

### B9-G16 — minor — Argument node names differ from vanilla

REPRODUCES — proven statically (the 26.2 form of this item came from
the run's node diff; the 26.1 run is blocked, §2, so the comparison
here is the tree-builder literals against vanilla's datagen dump —
and the literals ARE what the fixture would report, since the fixture
reads the node names from the built tree). Five label pairs, the same
five as 26.2, plus the `/item modify` pair the 26.2 run never saw
(no 26.2 corpus case parsed far enough into `modify` to carry nodes):

| Path | Vanilla 26.1 (datagen `commands.json`) | Rust 26.1 |
|---|---|---|
| `/item replace block <…>` | `pos` | `targetPos` (`tree_misc.rs:190`) |
| `/item replace entity <…>` | `targets` | `target` (`tree_misc.rs:188`) |
| `/item replace … from block <…>` | `source` | `sourcePos` (`tree_misc.rs:211`) |
| `/item modify block <…>` | `pos` | `targetPos` (`tree_misc.rs:174`) |
| `/item modify entity <…>` | `targets` | `target` (`tree_misc.rs:172`) |
| `/execute if items entity <…>` | `entities` | `source` (`tree_execute.rs:230`) |
| `/execute if items block <…>` | `pos` | `sourcePos` (`tree_execute.rs:237`) |

(`from entity` agrees: both sides call it `source`.) The vanilla
names are from the datagen `commands.json` dump of vanilla 26.1's own
dispatcher; the Rust names are the string literals in the tree
builders. Node names surface in parse output (`nodes` in the
fixture's jsonl) and in client-side command feedback, so a rename is
a real (if cosmetic) divergence — minor. The rerun's node diff
(`node_diff.py`, ready in this directory) will confirm empirically;
the corpus covers all seven paths (187 `/item replace`, 20 `/item
modify`, 523 `execute if items` cases).

### B9-G17 — minor — Recipe suggestions omit special recipes (Rust suggests the learnable subset; real vanilla suggests all recipes)

NEW for 26.1 (not a 1.21.7 item; the 26.2 port does not have the
learnable-recipes suggestion branch at all). Real vanilla 26.1 DOES
suggest recipe names: `CommandSourceStack.suggestRegistryElements`
has a `Registries.RECIPE` special case (javap offsets 0–36) that
suggests from `server.getRecipeManager().getRecipes()` — ALL recipes.
Rust suggests from the oracle-exported `learnable_recipes` list
(`args.rs:1747-1748`), which the oracle builds as the non-special
recipes only (`VanillaCommandParseOracle.java:279-280`, filtering
`!h.value().isSpecial()`): 1,491 entries, against 1,515 recipe files
in the 26.1 client jar, of which 23 are `crafting_special` (banner
duplicates, book cloning, armour dyeing, …). So Rust omits the ~23
special recipes that real vanilla suggests — e.g. completing
`recipe give @s minecraft:black_banner_duplicate` suggests the id in
real vanilla and nothing in Rust. No corpus case covers a
special-recipe prefix, so a future run would not demonstrate this
either; it is verified by jar + source + counts. Note: when the run
eventually happens, its recipe suggestion mismatches will NOT be this
defect — they will be an oracle artifact (see B9-G11 below — the
oracle's `Src` override bypasses the RECIPE special case and suggests
nothing, while Rust suggests its learnable list). Suggestions only,
special recipes only — minor.

### 3.2 Closed — the 1.21.7 defects, re-checked against 26.1

Each of the thirteen 1.21.7 Batch 9 defects was re-verified against
26.1's own jar/data. Nine are CLOSED for 26.1; four reproduce in whole
or part (G02, G07, G08, G09 — §3.1). The closures fall into three
groups.

**Group A — vanilla 26.1 itself uses the 26.x form (the 1.21.7 defect's
vanilla premise does not hold in 26.1):**

- **B9-G01 — CLOSED.** Vanilla 26.1's
  `net.minecraft.world.level.gamerules.GameRules` registers **59
  snake_case rules** (javap static initializer: `advance_time`,
  `keep_inventory`, `doFireTick` absent — there is no fire-tick rule in
  26.1 at all — `spawn_mobs` / `spawn_monsters` / `spawn_phantoms` as
  separate rules, `raids`, `elytra_movement_check`). The Rust table
  (`gamerules.rs:51-126`) is the same 58 ungated rules in the same
  order with the same defaults (sampled against the javap clinit:
  `fire_spread_radius_around_player` 128/min −1,
  `max_block_modifications` 32768/1, `max_command_forks` 65536/0,
  `max_command_sequence_length` 65536/0, `max_entity_cramming` 24/0,
  `random_tick_speed` 3/0, `players_nether_portal_default_delay`
  80/0, `tnt_explosion_drop_decay` false), plus `max_minecart_speed`
  (8/1/1000) gated behind the `minecraft:minecart_improvements`
  feature flag — exactly as vanilla gates it
  (`FeatureFlags.MINECART_IMPROVEMENTS`, static initializer offset
  403–406). The `/gamerule` tree is built from
  `available_rules(minecart_improvements)` (`vanilla.rs:301`). Nit,
  not a defect: the `MAX_MINECART_SPEED` comment says "(26.3)" — the
  rule is in 26.1 too (same nit as 26.2).
- **B9-G04 — CLOSED (inverted).** Vanilla 26.1's `SetSpawnCommand` and
  `SetWorldSpawnCommand` use `RotationArgument.rotation()` (yaw+pitch
  pair), not `AngleArgument` — javap shows `rotation` /
  `RotationArgument.rotation()` in both `register` methods. The Rust
  `ArgType::Rotation` (`vanilla.rs:341,350`) matches vanilla 26.1.
- **B9-G06 — CLOSED (both parts, both inverted).**
  (a) Vanilla 26.1's `EntityDataSource` holds a
  `CompilableString<EntitySelector>` decoded via
  `EntitySelector.COMPILABLE_CODEC`; `CompilableString$CommandParserHelper.apply`
  catches `CommandSyntaxException` and returns `DataResult.error`
  (exception table 9–69 → 70) — so decoding
  `{"nbt":"a","entity":"@x"}` FAILS in vanilla 26.1, the opposite of
  1.21.7 (where `compileSelector` caught the exception and returned
  null). The Rust `"nbt"` arm's selector check
  (`component.rs:278-299`, `selector_string` must parse) rejects it
  too — match.
  (b) Vanilla 26.1 HAS `ObjectContents`
  (`network/chat/contents/ObjectContents.class`, with `AtlasSprite`
  and `PlayerSprite` in `contents/objects/`) — the Rust `object` arm
  (`component.rs:300-314`) is correct for 26.1.
- **B9-G10 — CLOSED (inverted).** Vanilla 26.1's `WorldBorderCommand`
  uses `TimeArgument.time(0)` for the `add`/`set` time argument
  (javap: call sites at offsets 51, 110, 299), not
  `IntegerArgumentType`. Rust (`tree_world.rs:335`, `ArgType::Time {
  min: 0 }`) matches — including the `d`/`s`/`t` suggestions, which
  are correct for a Time argument.

**Group B — the 26.1 port fixed the defect (Rust differs from 1.21.7):**

- **B9-G03 — CLOSED.** Rust 26.1 uses
  `ArgType::ResKey("minecraft:worldgen/configured_feature")`
  (`tree_world.rs:65`; comment notes inline feature values are
  26.3-only), matching vanilla 26.1's `PlaceCommand`
  (`ResourceKeyArgument.key(Registries.CONFIGURED_FEATURE)`).
- **B9-G05 — CLOSED.** Rust 26.1 `/publish` is
  `publish [<allowCommands> [<gamemode> [<port>]]]`
  (`tree_extra.rs:76-85`), matching vanilla 26.1's `PublishCommand`
  (`BoolArgumentType`, `GameModeArgument.gameMode`,
  `IntegerArgumentType(0, 65535)` — javap offsets 31, 50, 69–72).
  The corpus carries the case `publish true survival 25565` for the
  rerun (the run itself is blocked, §2).
- **B9-G07 (part b) — CLOSED.** See §3.1: the multi-entity source is
  fixed (single-entity source); only part (a) reproduces.
- **B9-G12 — CLOSED.** The 26.3-only `SlotSource` argument type (whose
  SNBT fallback produced the 1.21.7 divergences) is gone from the 26.1
  tree: `execute if items` uses `ArgType::Slots`
  (`tree_execute.rs:239`), `/item` uses `ArgType::Slot`
  (`tree_misc.rs:174,190,211`). The Slot/Slots parse matches vanilla
  26.1's `SlotArgument` exactly: read-until-space
  (`lambda$parse$0`: `c != ' '`), `slot.unknown` /
  `slot.only_single_allowed` at the cursor after the word (same keys
  in the 26.1 `en_us.json`: "Unknown slot '%s'",
  "Only single slots allowed: got '%s'"), and the `slot_ranges()`
  table matches vanilla 26.1's `SlotRanges` entry for entry
  (contents 0; container. 0×54, hotbar. 0×9, inventory. 9×27,
  enderchest. 200×27, mob.inventory. 300×8, horse. 500×15,
  weapon.*×2, armor.*×5, player.crafting. 500×4, singles at 98/99/
  100–103/105/106/499).
- **B9-G13 — CLOSED.** Vanilla 26.1's `ArgumentTypeInfos` bootstrap
  registers 57 types in exactly the Rust `ARGUMENT_TYPES` order
  (`horizon-protocol/src/packets/play/commands.rs:25-82`), and index
  16 is `color` in vanilla 26.1 (17th `register` call, `ldc "color"`)
  — the name the 1.21.7 batch flagged as 26.x carry-over (`team_color`)
  is NOT what 26.1 uses; the 26.1 port ships `minecraft:color` at
  index 16, matching vanilla 26.1. Note the three-version shape:
  1.21.7 `color`, 26.1 `color`, 26.2 `team_color` — the rename
  happened between 26.1 and 26.2.

**Group C — the 26.x form was correct for 26.1 all along / masked:**

- **B9-G02 (bulk) — CLOSED as stated, except `posteffect` (§3.1).**
  Full top-level diff of the Rust release tree (77 literals from
  `tree_*.rs` + `vanilla.rs`, dev-gated `raid`/`warden_spawn_tracker`,
  Horizon-gated `pregen`, and dedicated-only commands excluded)
  against vanilla 26.1's dispatcher dump (datagen `commands.json`, 91
  top-level; 76 integrated after removing the 15 dedicated-only per
  `Commands.<init>`): **76 of 76 vanilla integrated commands present,
  zero missing; one extra, `posteffect`.** `stopwatch`, `swing`,
  `fetchprofile` are all in vanilla 26.1's tree; `/execute if|unless
  stopwatch` is in vanilla's tree (and the Rust `stopwatch` node,
  `tree_execute.rs:193-196`, matches: `id` → `range` float_range →
  redirect); the `/time` clock subcommands (`pause`, `resume`, `rate`,
  `of <clock>`, `set <timemarker>`, `query <timeline>`) are all in
  vanilla 26.1's `/time` node, and the Rust `clock_nodes`
  (`vanilla.rs:476-505`) matches it node for node (including `query
  gametime` only on the default clock, not under `of`, and `timeline
  repetition` under both). `compute` is gone from the Rust tree
  entirely (no occurrences in `horizon-command/src`), and `unpublish`
  was never added for 26.1 (only a test comment at `tests.rs:915`
  noting 26.1 has no `UnpublishCommand` — correctly, since vanilla
  26.1 has neither the class nor the command; `unpublish` arrived in
  26.2). Note: vanilla registers `jfr` only when the JVM profiler is
  available (`JvmProfiler.INSTANCE != null`, `Commands.<init>` offsets
  549–564); Rust registers it unconditionally — effectively always
  available on a normal JVM, not recorded as a defect (same as 26.2).
- **B9-G11 — CLOSED (with a correction to the 26.2 record).** The
  1.21.7 defect as stated — "vanilla suggests nothing for a complete
  recipe argument, Rust re-suggests" — does NOT hold against REAL
  vanilla 26.1. Real vanilla 26.1 DOES suggest recipes:
  `CommandSourceStack.suggestRegistryElements` has a
  `Registries.RECIPE` special case (javap offsets 0–36) suggesting
  from `RecipeManager.getRecipes()`, even though there is no
  `minecraft:recipe` registry (absent from datagen `registries.json`;
  recipes live in the `RecipeManager`). For the case in the corpus
  (`recipe give @s minecraft:stick`), the two suggestion lists agree
  by static comparison: real vanilla suggests `minecraft:stick`,
  `minecraft:stick_from_bamboo_item`, `minecraft:sticky_piston`
  (prefix matches among `getRecipes()`), and Rust suggests exactly
  the same three from its `learnable_recipes` list
  (`args.rs:1747-1748`, 1,491 entries). Note for the rerun: it WILL
  show recipe suggestion mismatches, but they will be an ORACLE
  artifact, not a port defect — the oracle's `Src` class overrides
  `suggestRegistryElements` (`VanillaCommandParseOracle.java:93-99`)
  to look the registry up in the `RegistryAccess` and return
  `Suggestions.empty()` when absent, bypassing vanilla's RECIPE
  special case, so the oracle suggests nothing while Rust suggests
  its learnable list. (The 26.2 closure — "vanilla suggests nothing"
  — was based on the same oracle behaviour; real vanilla 26.2 has the
  same special case, so 26.2's Rust, which suggests nothing, is the
  one that diverges from real vanilla there.) The residual real
  divergence for 26.1 is the special recipes Rust omits — recorded
  as B9-G17 (§3.1). The suggestion mechanism itself
  (`lib.rs:suggestion_context`, byte-identical to 26.2's, which was
  verified against Brigadier 1.3.10's `findSuggestionContext`)
  mirrors Brigadier.

### Passes (verified, not re-diagnosed)

- **`/time set day|noon|night|midnight` — PASS in inverted form.**
  Vanilla 26.1's `/time set` has only the `time` and `timemarker`
  arguments (datagen `commands.json`) — the day/noon/night/midnight
  literals do not exist in 26.1 (they went with the clock rework). The
  26.1 Rust `clock_nodes` has none either (no matches for
  day/noon/night/midnight in `horizon-command/src`). Both sides agree:
  `time set day` parses only via the `timemarker` (resource-location)
  argument on both. The 1.21.7 pass ("literals present") does not
  transfer as stated; the 26.1 check is the matching absence, and it
  holds.
- **57 argument-type list — PASS.** Count 57, order identical to
  vanilla 26.1's `ArgumentTypeInfos` registration order (javap
  `bootstrap`: 57 `register` calls, ids paired in order, exactly the
  datagen `registries.json` protocol-id order), names identical
  including `minecraft:color` at index 16 (see B9-G13 above).
- **Error-text resolution at runtime — PASS.** The game resolves command
  error keys correctly (see B9-O04, §4).

### 3.3 Harness blocker

### B9-H01 — major — The `client_probe` example does not compile on 26.1, blocking `command_parse`

The `command_parse` fixture is the `client_probe` example's
`command_parse_fixture` mode, and the example does not compile on the
committed 26.1 tree: `game_loop_probe.rs:159-160` sets `focused` and
`exclusive_fullscreen` on a `FrameContext` literal, and 26.1's
`FrameContext` has neither field (E0560 × 2, §2). The probe file is
byte-identical to 26.3's — the 26.1 port carried it verbatim — while
26.1's `FrameContext` deliberately omits the two 26.3-only fields.
The 26.2 and 1.21.11 ports adapted their probe files; 26.1, 26.1.1,
26.1.2 and 1.21.7 did not. The fix is two deleted lines (§2); it was
not attempted here (checks + triage only). Major, because it blocks
the batch's core measurement and any other driver that builds the
`client_probe` example — not because of any gameplay impact (the
game-loop probe is test scaffolding, not shipped game code). Counted
once as a gap check (§1).

## 4. Oracle / fixture defects

### B9-O01 — minor — `VanillaScoreboardOracle` fails javac (3 errors) — blocks `scoreboard`

REPRODUCES for 26.1 (it passed in 26.2 — 26.1 predates `TeamColor`).
Batch 0's 26.1 measurement (`docs/sweep/26.1/batch-0-oracles.json`,
full errors in
`versions/26.1/.local/measure-oracles/results.json`): fails all three
classpaths, `missing-symbol`, 3 errors. The 1.21.7 pack-method errors
(lines 44) are gone — 26.1's `Scoreboard` HAS the packing methods —
but the two `TeamColor` sites fail:

- Line 242 (`t.getColor().map(TeamColor::getSerializedName)`):
  `cannot find symbol` — there is NO `TeamColor` class in the 26.1
  server jar (verified: no `TeamColor` entry). 26.1's
  `PlayerTeam.getColor()` returns `ChatFormatting`.
- Line 436 (`t.setColor(… Optional.of(TeamColor.byName(…)))`):
  `incompatible types` + `cannot find symbol` — 26.1's
  `PlayerTeam.setColor` takes a bare `ChatFormatting`, not an
  `Optional`, and `TeamColor.byName` does not exist.

Repair scope: small, in-family, NOT attempted (no fixes in this
batch): `TeamColor::getSerializedName` → `ChatFormatting`'s own
`getSerializedName` (it implements `StringRepresentable`),
`TeamColor.byName` → `ChatFormatting.getByName`, drop the `Optional`
wrapper at line 436. Needs an end-to-end driver run after repair
(compiling ≠ running).

### B9-O02 — minor — `VanillaTeamRulesOracle` fails javac (6 client errors) — blocks `team_rules`

REPRODUCES for 26.1 (worse than 26.2's 2 errors: 26.1 predates the
`Hud` split). Batch 0's 26.1 measurement: client cp
`missing-symbol`, 6 errors. The server-cp failure (41 errors,
`net.minecraft.client` absent) is expected — the driver uses the
client jar, so the client cp is the operative one.

- Line 14 (`import net.minecraft.client.gui.Hud`): `cannot find
  symbol` — no `Hud` class in the 26.1 client jar (verified). The
  `Hud` split happened after 26.1; 26.1's `Gui` has no `hud` field
  and the HUD-hidden flag is `Options.hideGui` (public boolean).
- Lines 188 (×2), 189: the three `Hud` uses (`Hud hud =
  alloc(Hud.class)`, `put(hud, Hud.class, "isHidden", …)`,
  `put(gui, Gui.class, "hud", hud)`) — all downstream of the missing
  import.
- Line 101 (`Viewer()`): `super(null, null, null, null, null, null,
  false, null, null)` — 9 arguments. 26.1's `LocalPlayer` constructor
  takes 8: `(Minecraft, ClientLevel, ClientPacketListener, StatsCounter,
  ClientRecipeBook, Input, boolean, ChatAbilities)` (javap on the 26.1
  client jar). Drop one argument to match (same as 26.2).
- Line 197: `own.doTeamsAllowDamage(other)` — no such method in 26.1
  (`Entity` has no `doTeamsAllowDamage`; the 26.1 form is
  `PlayerTeam.isAllowFriendlyFire()`, which exists). Rework the damage
  check against the team's friendly-fire flag (same as 26.2).

Repair scope: medium (larger than 26.2's), in-family, NOT attempted.
Needs an end-to-end driver run after repair.

### B9-O03 — minor — `LocatorBarClient` fails javac (13 errors) — blocks `exploration_map_command`

REPRODUCES for 26.1 (worse than 26.2's 10 errors: 26.1 predates both
the `EntityTypes` move and the 6-arg teleport-accept packet). Batch
0's 26.1 measurement: fails all three classpaths, `missing-symbol`,
13 errors (full list from
`versions/26.1/.local/measure-oracles/big-compile-client.log` — the
results file lists only the first 8). Causes, verified against the
26.1 client jar:

- Lines 393–395, 407, 409, 413: `packet.removed()` / `packet.added()`
  / `packet.progress()` on `ClientboundUpdateAdvancementsPacket` —
  26.1 uses getter names: `getRemoved()`, `getAdded()`,
  `getProgress()` (`shouldReset()` exists as-is, so line 405 is fine).
  Six call sites to rename (same as 26.2).
- Line 668: `canUseCommandBlocks()` override — no such method on the
  supertype in 26.1 (`GameProtocols$Context` has only
  `hasInfiniteMaterials()`). Drop the override (same as 26.2).
- Lines 683, 906, 911: `net.minecraft.world.entity.EntityTypes.PLAYER`
  — no `EntityTypes` class in 26.1 (registration still lives in
  `EntityType`, the 1.21.x shape). Use `EntityType.PLAYER`. Three
  sites (new vs 26.2).
- Lines 690, 916: `removed.entityIds()` / `rm.entityIds()` on
  `ClientboundRemoveEntitiesPacket` — 26.1's accessor is
  `getEntityIds()`. Two sites (same as 26.2).
- Line 977: `new ServerboundAcceptTeleportationPacket(t.id(), x, y, z,
  yaw, pitch)` — no suitable constructor: 26.1's packet has only the
  1-arg `(int)` constructor (the 6-arg form arrived later). Send the
  id-only packet (new vs 26.2).

Repair scope: per-symbol renames against the 26.1 client jar (the five
named fixes cover all 13 errors). Medium; NOT attempted — and out of
family for a commands batch (packet-client helper shared with the
locator-bar harness; the 26.2 merge recorded B3-O04 as canonical for
this oracle — expect the same overlap for 26.1's Batch 3).

### Closed oracle / fixture defects

- **B9-O04 — CLOSED.** The `command_parse` fixture loads language from
  the prepared-resources root (`command_parse_fixture.rs:147`,
  `lang::Language::load(&[&root], …)`; root = `lab.RESOURCES` =
  `.local/horizon-client/26.1/`). Unlike 1.21.7, the prepared 26.1 root
  DOES contain `assets/minecraft/lang/en_us.json` — md5-identical to
  the 26.1 client jar's file (`98deafdb5e90e4438db30853494e1aeb`,
  7,886 keys, including every command error key the driver compares).
  The fixture's error-text leg is readable for 26.1: the 1.21.7
  defect was the file's ABSENCE, and the file is present and
  jar-identical here, so the 86,241-mismatch artifact of the 1.21.7
  run cannot recur on that account. (The run confirmation — 0
  error-text mismatches — is unavailable because the run is blocked,
  §2; the closure rests on the md5 + the fixture's load path.)

### Passing in-scope oracles (auditable coverage)

From the Batch 0 measured table (`docs/sweep/26.1/batch-0-oracles.json`):
`VanillaCommandParseOracle` (pass all three classpaths),
`VanillaCommandExecOracle` (pass all three),
`VanillaCommandSuggestionsExporter` (pass all three). The
`command_parse`/`command_exec` linchpin drivers are oracle-clean for
26.1 from the start.

## 5. Driver table (22)

| Driver | Oracle | Status |
|---|---|---|
| command_parse | VanillaCommandParseOracle (pass) | **BLOCKED — build failure** (B9-H01, §2): the `client_probe` example does not compile on 26.1 (2 errors in `game_loop_probe.rs`). Corpus generated (34,542 commands, preserved) and oracle clean; no parses run. |
| command_exec | VanillaCommandExecOracle (pass) | Not run. Needs full game build + Java integrated server. Known fixture blocker (same as 1.21.7/26.2): `/execute if predicate effect_view:always` — the `effect_view:*` predicates come from `tools/mob_effect_predicate_corpus.py`, imported by `command_exec_corpus.py:64`. Recorded, not chased. |
| scoreboard | VanillaScoreboardOracle (FAIL) | Oracle-blocked — B9-O01 (3 errors; 26.2's cheapest remaining driver is blocked again for 26.1) |
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

- **Run before writing** was attempted and blocked: the one
  measurement available (`command_parse`, §2) was attempted four
  times across ~2h30m of build wall-time (two VM reboots, one
  unexplained process kill, then a deterministic compile failure)
  before the blocker was diagnosed as B9-H01. Everything in this
  report is therefore method 2 (ledger-item pattern: javap + source
  + oracle data + datagen), and every verdict that would have rested
  on run data instead rests on a static proof stated in its section.
  The corpus, the oracle exports, and the analysis scripts
  (`node_diff.py`, `classify_mismatches.py`) are preserved in this
  directory for the rerun.
- **Vanilla 26.1's own jar/data is the spec throughout.** Every
  behaviour claim above is grounded in one of: the oracle's exports
  (`commands.json`, `command-registries.json`, `tree.json`,
  `registries.json` — real vanilla 26.1 dispatcher output), `javap -c`
  on the named 26.1 server-game jar (Mojang-mapped, so no mappings
  front-end was needed) or the 26.1 client jar, the vanilla 26.1
  datagen reports, or Brigadier 1.3.10 from the datagen libraries. The
  26.3 base's form was never trusted — and this batch is the reverse
  of the 26.2 one: the dominant pattern is 26.2 forms that are NOT
  correct for 26.1 (the selector option-state refactor, `TeamColor`,
  `team_color`, the `Hud` split, `EntityTypes`, the 6-arg
  teleport-accept packet), plus 1.21.7 forms the 26.1 port never fixed
  (particle `Name` compound, item-modifier `function` key,
  `/posteffect`).
- **Systemic pattern**: the 26.1 command surface is in markedly worse
  shape than 26.2's was: 4 of 13 re-checked defects reproduce (vs 0
  of 13 for 26.2), all three in-scope oracles fail (vs 1 of 3 for
  26.2 — scoreboard passed there), and the `command_parse` fixture
  does not even compile (B9-H01). The port sits between versions: it
  carries 26.2-era code (selector option-state, `TeamColor` in the
  oracle, `Hud` in the oracle) against a 26.1 vanilla that predates
  those, 1.21.7-era code (particle/modifier handling) that the 26.2
  port fixed but the 26.1 port never carried, and 26.3-era probe code
  (`game_loop_probe.rs`) against a 26.1 `FrameContext` that predates
  it. The remaining risk is execution, not parsing: 18 of 22 drivers
  are unrun e2e drivers, and `command_parse` only covers parse
  behaviour.
- **What the corpus cannot see**: it walks vanilla's tree, so
  Rust-only extras appear only via hand-written edge cases (the 26.1
  corpus carries `posteffect`, `posteffect @s minecraft:x`,
  `compute`, `unpublish`, `raid start 5`). The tree diff (§3, B9-G02)
  is the complete check for extras; the corpus is the complete check
  for missing/wrong vanilla behaviour. The dedicated tree (`/perf`)
  and the dev tree are outside both: the corpus walks the integrated
  tree, and the dev tree is not built by `release_dispatcher()` —
  B9-G14/G15 were caught by the static diff against `Commands.<init>`
  bytecode, not by any run.
- **Counts**: 23 checks (11 pass, 12 gap) — 13 re-checks (9 pass, 4
  gap) + 2 re-verified passes + 8 new gap items (G14, G15, G16, G17,
  O01, O02, O03, H01). 22 items in the slice (G01–G17, H01, O01–O04);
  O04 is an item but not a check (oracle closure, per the 26.2
  convention), and the 2 re-verified passes are checks without items.
  Driver table: 0 measured + 1 build-blocked + 3 oracle-blocked +
  18 not run = 22.

## 7. Open questions / follow-ups for the coordinator

0. **B9-H01 first.** The two-line probe fix (delete `focused` /
   `exclusive_fullscreen` from the `FrameContext` literal in
   `game_loop_probe.rs:159-160`) unblocks the `command_parse` rerun
   and any other `client_probe`-based driver. After the fix, the
   rerun is cheap: the library is already compiled, the corpus is
   preserved (`command-parse-corpus.txt`), and the driver resumes
   from the warm cache (`run-command-parse.sh` in this directory;
   `node_diff.py` + `classify_mismatches.py` are ready for the
   output). The rerun's expected findings, from the static verdicts:
   `posteffect` ok-differs (G02), inline-modifier ok-differs (G07a),
   particle `Name` ok-differs (G08), selector cursor-differs +
   `@a[type=player]` ok-differs (G09), recipe suggestion mismatches
   that are ORACLE artifacts (G11 — see item 8), and the 7 node-label
   pairs (G16). The same unadapted probe file is in 26.1.1, 26.1.2
   and 1.21.7 — their batches (or a single cross-version fix) will
   hit the same wall.
1. The 18 unrun e2e drivers (esp. `command_exec` + its 7 dependents)
   are the batch's real remainder: they need a full client build and
   the `effect_view:always` fixture blocker resolved. Recommend running
   `command_exec` first (with `--allow-gaps`), then its dependents.
2. `scoreboard` is NOT the cheapest remaining driver for 26.1 (it was
   for 26.2): its oracle fails again on `TeamColor` (B9-O01). The
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
5. B9-G09 (selector) is the batch's most instructive finding: the Rust
   selector parser implements the 26.2 option-state shape, but vanilla
   26.1 still has the old parser. A fix here must target 26.1's form
   (no rollback on inapplicable, `!isTypeLimited` for `type`), NOT
   the 26.2 form — and any fix carried across versions must be
   version-gated, since 26.2's vanilla wants the opposite behaviour
   for `@a[type=player]`.
6. B9-G07a (item-modifier `function` key) and B9-G08 (particle `Name`
   compound) are fixes the 26.2 port already made — both are direct
   carry-overs from the 26.2 tree (`codec.rs` key-parameter dispatch,
   `args.rs` `Name`-accepting `particle_options`). They are the
   cheapest behaviour fixes in the batch.
7. B9-G02's `posteffect` is a one-command removal from
   `tree_extra::register` — but check whether anything else in the
   26.1 tree references the nonexistent `post_effects` registry before
   removing it (the `ArgType::Id("post_effects")` arguments at
   `tree_extra.rs:40,50` die with the command).
8. The oracle's `Src.suggestRegistryElements` override
   (`VanillaCommandParseOracle.java:93-99`) bypasses vanilla's
   RECIPE special case, so the `command_parse` driver cannot test
   recipe suggestions faithfully: the oracle suggests nothing where
   real vanilla suggests all recipes. The rerun's recipe suggestion
   mismatches will be artifacts of that override (see B9-G11, §3.2)
   — do not read them as port defects. A future oracle fix would be
   to special-case `Registries.RECIPE` in `Src` the way
   `CommandSourceStack` does (suggest from the oracle's own
   `learnable_recipes` export — or, more faithfully, from all
   `RecipeManager` recipes). Until then, recipe-suggestion
   divergences (B9-G17) are verifiable only statically.
9. B9-G16 (node names) is proven statically here, but the rerun's
   node diff is still worth doing when H01 lands: it is the only
   check that sees the names the fixture actually emits (as opposed
   to the literals in the tree builders), and it may surface further
   label divergences in commands the static diff did not walk
   argument-by-argument.
