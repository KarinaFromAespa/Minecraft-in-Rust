# Cross-version defect regression suite

The 1.21.7 playtest found a dozen defects that all share one shape: the
port carries a 26.3 form (a component, a field name, a key, a tag, a
model type) that the target version's own vanilla data does not have, so
the lookup silently degrades and something in the game quietly stops
working — furnaces that take no fuel, sheep that drop nothing, beds and
mobs that never render, a recipe book that stays empty. Each fix was
verified in 1.21.7, but nothing said whether the same defect lived in the
other 18 versions.

This suite is that check. One command runs every known defect class
against any version's own jar and tree, and a failing check names the
version and the defect:

```sh
python3 tools/defect_suite.py 1.21.11 --jar-dir jars/
python3 tools/defect_suite.py --all --jar-dir jars/ --report-only
```

It is dispatch-only CI, in the `defect-suite` job of
`.github/workflows/crate-tests.yml` (one version or `all`), which
downloads the version's server and client jars from Mojang, sha1-verified,
runs the suite, and uploads `defect-suite-<version>` with the JSON report.
The job reports the defect list; it does not gate on it — every version
fails something today, and the list is the point.

## The checks

| id | What it checks | Oracle | Fixed in 1.21.7 by |
|---|---|---|---|
| `fuel-table` | A version with no fuel component needs the FuelValues table fallback | server jar: no class mentions `cooking_fuel` | a53b0c58 |
| `brewing-fuel` | A tag-gated version needs tag-based brewing fuel | server jar: `brewing_fuel` item tag file present | in flight |
| `loot-fields` | Loot evaluators read the version's field names — container (`conditions` list vs `condition` object), kind keys (`condition`/`function` vs `type`), list key (`functions` vs `modifier`) | server jar: loot-table census | e3bce30e |
| `loot-tables` | Every loot-table id the code rolls exists in the version's own jar (harvest/ tables are 1.21.9-only; the 1.21.7 sweet-berry pick rolled a table that version never shipped) | server jar (client jar fallback): loot-table inventory | in flight |
| `recipe-packets` | `recipe_book_add` packets decode (all slot display kinds) | rust tests (none yet) | in flight |
| `advancement-keys` | Triggers read the version's key names (singular ≤26.2, plural in 26.3) | server jar: advancement census | 59ba5f8d, caef65c4 |
| `bed-models` | Every special item-model type the version uses has a decoder arm | client jar: item-model census | in flight |
| `mob-models` | Every entity has a resolvable model | rust tests (none yet) | in flight |
| `undead-burn` | Undead ignite promptly in daylight | rust tests (none yet) | in flight |
| `menu-sprites` | Every sprite the code references exists in the version's client jar or the prepare fallback | client jar + 26.3 client jar | 20453af0 |
| `absent-tags` | No tag/attribute/gamerule leaks new against the baseline | the leakage audit | ongoing |

Statuses: `pass`, `fail` (the defect is present), `skip` (the check
does not apply to this run — e.g. no client jar, so the sprite and
special-model checks cannot run), `unmeasured` (the check applies but
its oracle is unavailable — never a silent pass), `not-covered` (the
class has no rust test registered yet, or the registered test does not
exist in this tree).

Exit codes mirror the leakage audit: 0 no failures, 1 a check failed,
3 a check was unmeasured (and none failed), 2 usage error.
`--report-only` forces 0.

## The two kinds of check

**Data checks** (fuel, brewing, loot, advancement, bed item-models,
sprites, tags) run everywhere with no compile. They compare what the
version tree's code reads against what the version's own jar contains.
The oracle is always the version's own jar — never hardcoded 1.21.7
values, never the 26.3 base's form. That is what makes a check
cross-version: 26.3's data uses plural advancement keys and a fuel
component, so the same checks pass there while failing on 1.21.11.

**Rust checks** are the regression tests the fix agents write in the
version crates, registered in `tools/defect_suite_rust.json` by class,
crate, and test function name. The suite has three modes for them:

- Default: presence only — it reports whether each registered test
  exists in the tree. A missing test is `not-covered`, not a failure.
- `--crate-logs <dir>`: reads results out of a crate-tests run's logs
  (`<crate>.log` files, as uploaded by the crate-tests job). This is
  how CI results join the suite's report without recompiling. A
  downloaded artifact nests the logs one level down
  (`crate-tests-1.21.7/horizon-items.log`); the driver looks there
  too. `horizon-prepare` is not in the crate-tests suites, so a
  horizon-prepare test never has a log — it stays not-covered here.
- `--run-rust`: runs the registered tests with `cargo test` itself.
  Opt-in: compiling `horizon-client`'s test binary OOMs the VM, so
  run those crates on CI or the laptop, never the VM.

A manifest entry names the crate the test's source lives in. When the
test *runs* under a different cargo package, the entry carries a
`suite` field with that package name: `horizon-client-lib`'s lib root
is `../horizon-client/src/client_lib.rs`, so tests written in
`crates/horizon-client/src/` run as `horizon-client-lib` and their
results land in `horizon-client-lib.log`. Presence checks use the
source crate; `--crate-logs` and `--run-rust` use the suite.

## Adding a defect class

When a playtest (or a sweep) finds a new defect class:

1. **Fix it in one version first**, with a regression test, the way the
   furnace fix did. The fix is the spec for the check.
2. **Decide the check's kind.** If the defect is "code reads X, the
   version's data has Y", it is a data check: write a census function
   over the jar (loot tables, advancements, tags, item models — the
   existing ones are the pattern) and a code extractor for what the
   tree reads, and compare. If the defect is behavioural (timing, a
   packet round-trip, a render path), it is a rust check: write the
   regression test and register it in `tools/defect_suite_rust.json`.
3. **Name the rust test version-neutrally** (`defect_<class>_<what>`,
   e.g. `defect_fuel_coal_burns_1600`) so the same name ports to every
   version. The one exception is the 1.21.7-named loot test, which
   predates the convention.
4. **Add the check to `CHECKS` in `tools/defect_suite.py`** with its
   id, title, and function, add the row to the table above, and prove
   it: it must pass on the version where the fix landed and fail (or
   report not-covered) on a version that still has the defect.

Data-check contracts the code must satisfy (so a fix is recognised):

- `fuel-table`: the fallback is a `FUEL_VALUES` table plus
  `apply_fuel_values()` in `crates/horizon-items/src/items.rs`.
- `brewing-fuel`: the tag-based population is a `BREWING_FUEL` /
  `apply_brewing_fuel` name, or a `brewing_fuel` tag lookup, in
  `items.rs`. A bare `minecraft:brewing_fuel` component read does not
  count — the component does not exist in a tag-gated version.
- `loot-fields`: each loot evaluator — `crates/horizon-world/src/
  loot.rs`, the loot code in `items.rs`, and `loot/odds.rs` — must
  read the field names the version's data uses, in four roles: the
  conditions container (`conditions` list vs a single `condition`
  object), the condition kind key (`condition` vs `type`), the
  function kind key (`function` vs `type`), and the functions list
  key (`functions` vs `modifier`). The extraction is role-scoped:
  kind reads come from the `fn condition` / `fn condition_object` /
  `fn modify` / `fn modifiers` / `fn run` bodies, container reads
  from the rest of the file — a file-level scan conflates the two,
  because `condition` is the old kind key inside `fn condition` but
  the new container key in the pool/entry code (that conflation
  passed 1.21.11, whose loot.rs reads the new container and kind
  keys against old-form data). Reading both forms (the dual-read the
  1.21.7 fix uses) passes everywhere. Test lines are excluded:
  fixtures carry both forms of every key and would mask a missing
  production read.
- `advancement-keys`: each trigger's own evaluation closure (the
  `self.trigger("<id>", ...)` call in `game/advancements.rs` or
  `game/machines/crafter.rs`, plus one level of `adv::*_condition` /
  `adv::block` helper) must read the key that trigger's data uses.
  `recipe_unlocked` is evaluated through the index built by
  `index_recipes`, so that function's body is checked instead. If a
  trigger's evaluation site cannot be located, the check falls back
  to file-level reads and says so in its notes.
- `bed-models`: every `minecraft:special` model type the version's
  items use needs a `"minecraft:xxx" =>` arm in
  `SpecialModel::decode` (`crates/horizon-assets/src/item_model/
  special.rs`).
- `loot-tables`: a literal loot-table id counts when it is rolled —
  an argument to `sequence_loot`, `InteractLoot`, `drop_gift_loot`,
  `native_use_drops` or `roll`, a `let table = ...` / `table: ...` /
  `const TABLE` binding, a `create_chest` argument, or a `LootTable:`
  NBT string (the literal's line, or ~600 chars before it for
  multi-line calls). `format!` ids are dynamic prefixes: at least one
  table must exist under the prefix. Excluded: `==`/`!=` comparisons
  and `matches!` guards (they name an id to recognise, not a table to
  roll), `attributes[...]` reads (biome/dimension attribute ids),
  test modules, examples, and datafix files (historical ids by
  design). The oracle is the server jar, falling back to the client
  jar, which carries the same loot_table tree. Behaviour beyond
  existence is not asserted here — what vanilla drops instead is
  pinned in each version's own unit tests (registered under
  `loot-tables` in `tools/defect_suite_rust.json`). See
  `docs/LOOT-TABLE-AUDIT.md` for the per-version audit, the exact
  arrival versions (harvest/brush/carve/charged_creeper/turtle_grow
  in 1.21.9, till/ in 26.3), and the known gaps (dynamic per-entity
  ids, data-driven transformer loot ids, block drops).
- `menu-sprites`: a referenced sprite must exist in the version's
  client jar, or the prepare fallback (`fallback_menu_sprites` in
  `horizon-prepare`) must be present and the sprite must exist in the
  26.3 client jar. Dynamic (`format!`) references are checked by
  prefix: at least one sprite must exist under the referenced prefix.

## Known gaps

- **Mob models** have no data check yet. Entity models resolve from
  the prepare-generated `entity-models.json`, which this tool cannot
  reproduce without Java and the client jar. The designed follow-up
  is a prepare-time check: after prepare runs, verify every species'
  model key (from `mobs/species.rs` and `mobs/render.rs`) exists in
  `entity-models.json`. Until that lands, the class is rust-only.
- **Bed block models** are the same: the block model resolves through
  prepare's model-states export. The suite's data check covers the
  bed *item* model (the special-model type); the block render is
  rust-only.
- **Recipe packets** are rust-only: the slot-display id table lives in
  the protocol code, and the registry order that assigns the ids is
  not recoverable from the jar without the registry dump.
- The suite checks the defect classes it knows. A version that passes
  every check is not proven defect-free — it is proven free of these
  eleven classes.

## Reading a failure

A failing check prints its evidence: the jar census that defines the
expected form, and the code site that disagrees. For example,
`advancement-keys` on 1.21.11 reports that `minecraft:recipe_crafted`
data uses `recipe_id` (27 criteria) but the code reads only `recipes`
— the fix is the 1.21.7 one (59ba5f8d), ported to the version. The
JSON report (`--json`, and the CI artifact) carries the full evidence
list for every check.
