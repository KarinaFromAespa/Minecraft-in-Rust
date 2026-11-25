# Loot-table audit — every loot-table id the code rolls, per version

The 1.21.7 playtest found that right-clicking a ripe sweet berry bush
dropped nothing. The cause was not the bush and not the drop machinery:
the port's use handler rolled `minecraft:harvest/sweet_berry_bush`, but
1.21.7 ships no `harvest/` loot-table directory at all, so the lookup
missed and the pick silently dropped nothing — while breaking the bush,
which rolls the existing `blocks/sweet_berry_bush` table, worked fine.

This audit enumerates EVERY loot-table id referenced by every completed
version tree and checks each one against that version's own jar. The
check that makes it permanent is `loot-tables` in
`tools/defect_suite.py` (see `docs/DEFECT-SUITE.md`): it extracts the
ids the code rolls and asserts each exists in the version's own data.
Per-version differences come from the version's own jar, never a
hardcoded list — a table that exists in 1.21.9 but not 1.21.7 passes in
1.21.9 and fails in 1.21.7 while the code still references it.

## When the tables arrived (from the jars, not from memory)

| directory / table | first version shipping it |
|---|---|
| `harvest/beehive`, `harvest/cave_vine`, `harvest/sweet_berry_bush` | 1.21.9 |
| `brush/armadillo` | 1.21.9 |
| `carve/pumpkin` | 1.21.9 |
| `charged_creeper/root` (+ `creeper`, `piglin`, `skeleton`, `wither_skeleton`, `zombie`) | 1.21.9 |
| `gameplay/turtle_grow` | 1.21.9 |
| `till/rooted_dirt` | 26.3 only |

So every version from 1.21.4 through 1.21.8 lacks all seven of the
1.21.9 tables, and every version before 26.3 lacks `till/rooted_dirt`.
The berries fix agent's "between 1.21.4 and 1.21.10" is now pinned:
the tables arrived in **1.21.9**.

## Findings per version

Tree state: main `cdc5fa68` (the berries fix and the vine/beehive fix
were on unmerged branches when this ran — see "Fix status" below).

| version | refs | missing | broken references |
|---|---|---|---|
| 1.21.5 | 49 | 7 | `brush/armadillo`, `carve/pumpkin`, `charged_creeper/root`, `gameplay/turtle_grow`, `harvest/beehive`, `harvest/cave_vine`, `harvest/sweet_berry_bush` |
| 1.21.6 | 49 | 7 | same seven |
| 1.21.7 | 50 | 7 | same seven |
| 1.21.8 | 43 | 1 | `brush/armadillo` — latent, see below |
| 1.21.9 | 49 | 0 | — |
| 1.21.10 | 49 | 0 | — |
| 1.21.11 | 49 | 0 | — |
| 26.1 | 50 | 1 | `till/rooted_dirt` — latent, see below |
| 26.1.1 | 49 | 0 | — |
| 26.1.2 | 49 | 0 | — |
| 26.2 | 51 | 1 | `till/rooted_dirt` — latent, see below |
| 26.3 | 49 | 0 | — |

Line numbers below are the 1.21.7 tree's; 1.21.5/1.21.6 carry the same
references at nearby lines (dispenser 2064/2065 in 1.21.5, drops 465).

## What vanilla does instead (verified against the 1.21.7 jar)

Every broken id below is a 1.21.9 table standing in for behaviour
1.21.7 does directly in code — no class in the 1.21.7 jar mentions
any of these ids, and the bytecode drops the items outright. The fix
pattern for each is the same as the berries fix: drop directly, play
the sound, reset the state.

| id | code sites (1.21.7) | vanilla 1.21.7 behaviour | live effect |
|---|---|---|---|
| `harvest/sweet_berry_bush` | `horizon-world/src/level/interact.rs:335` | `SweetBerryBushBlock.useWithoutItem`: 1+random(2) berries, +1 at age 3, pick sound, bush resets to age 1 | right-click pick drops nothing |
| `harvest/cave_vine` | `horizon-world/src/level/interact.rs:314` | `CaveVines.use`: 1 glow berry directly, clears `berries` | picking glow berries drops nothing |
| `harvest/beehive` | `horizon-client/src/game/beehives.rs:270-271`, `game/dispenser.rs:1805-1806` | shear: fixed 3 honeycomb (`SHEARED_HONEYCOMB_COUNT`), shear sound, shears take damage | shearing a full hive drops nothing (honey-bottle path unaffected) |
| `brush/armadillo` | `horizon-client/src/game/creatures.rs:7332`, `game/dispenser.rs:2132-2133` | `Armadillo.brushOffScute`: 1 armadillo scute via `spawnAtLocation` | brushing an armadillo drops no scute |
| `carve/pumpkin` | `horizon-client/src/game/pumpkin.rs:45` | `PumpkinBlock` carve: pumpkin seeds directly (4, per the 1.21.9 table encoding the same behaviour) | carving a pumpkin with shears drops no seeds |
| `charged_creeper/root` | `horizon-client/src/mobs/drops.rs:466` | heads drop directly in code: `LivingEntity` spawns `SKELETON_SKULL` / `ZOMBIE_HEAD` / `PIGLIN_HEAD` / `CREEPER_HEAD` by victim type, `Creeper` its own head | charged-creeper victims drop no heads |
| `gameplay/turtle_grow` | `horizon-client/src/game/creatures.rs:3642,7161`, `mobs/ai.rs:1666` | `Turtle.ageBoundaryReached`: 1 turtle scute via `spawnAtLocation` | baby turtles growing up drop no scute |
| `till/rooted_dirt` | `horizon-client/src/block_transform.rs:352` (26.1), `:346` (26.2) | tilling converts rooted dirt to dirt and drops hanging roots directly (`HoeItem`/`RootedDirtBlock`); the table form exists only in 26.3 | latent — see below |

## Latent, not live

Two flags are wrong ids that do not currently break a drop:

- **1.21.8 `brush/armadillo`** (`game/creatures.rs:7367`): the 1.21.8
  port already drops the scute directly ("1.21.8's brushOffScute
  spawns one scute outright … there is no table to roll here") and
  only passes the id as metadata to `native_use_drops`, which never
  rolls it — it only hands the id to native-mod hooks. The scute
  drops; the id is a one-line cleanup.
- **26.1 / 26.2 `till/rooted_dirt`**: the in-tree transformer data
  names the table, but `game.rs` special-cases it (`if loot ==
  Some("minecraft:till/rooted_dirt")`, 26.1 `:7941`, 26.2 `:7947`)
  and pops the hanging roots directly — the id is never rolled.
  The fix is data-side (drop the id from the transformer data, or
  leave the special-case as the fix and remove the id).

1.21.8 already drops directly for the other six paths (berries and
vines in `interact.rs`, 3 honeycomb in `beehives.rs`, turtle scute in
`mobs/ai.rs`, charged-creeper heads via the `killedEntity` hook), so
it carries none of the live defects — the 1.21.8 port knew about the
1.21.9 tables and worked around them.

## Fix status

- `harvest/sweet_berry_bush` — fixed on `fix/1.21.7-sweet-berries`
  (tip `61126d68`, awaiting CI/merge at audit time). The check run
  against that tree clears the flag: 49 refs, 6 missing.
- `harvest/cave_vine` + `harvest/beehive` — being fixed on
  `fix/1.21.7-harvest-drops` (separate agent, same direct-drop
  pattern).
- `brush/armadillo`, `carve/pumpkin`, `charged_creeper/root`,
  `gameplay/turtle_grow` — **unfixed anywhere**. No fix was started
  here; this audit reports them. The fix pattern is established
  (direct drop, mirroring the vanilla bytecode above), and the same
  four ids are broken identically in 1.21.5 and 1.21.6 — a fix in one
  E6 version ports to the others.
- `till/rooted_dirt` (26.1/26.2) — latent; fix is data-side, see above.

## What the check covers, and what it does not

The `loot-tables` check extracts loot-table ids from production Rust
(`crates/*/src`): literal ids at loot call sites (`sequence_loot`,
`InteractLoot`, `drop_gift_loot`, `native_use_drops`, `roll`,
`let table = ...`, `table: ...`, `const TABLE`, `create_chest`,
`LootTable:` NBT) and `format!` dynamic prefixes (checked by "at
least one table under the prefix", the `menu-sprites` precedent).
Excluded by design, because they name ids to recognise rather than
tables to roll: `==`/`!=` comparisons and `matches!` guards (a
datafix matching an old id; the till special-case), `attributes[...]`
reads (biome/dimension attribute ids such as
`gameplay/natural_mob_spawns`), test modules, examples, and datafix
files (historical ids by design).

Known gaps (the check asserts existence only):

- **Dynamic per-entity ids.** `death_loot_table` builds
  `{namespace}:entities/{path}` from the mob's registry id, and
  `items.rs` looks up `minecraft:entities/{name}`. The check verifies
  the `entities/` prefix exists, not each id: a 26.x-only entity id
  leaking into an old tree would roll a missing table. Per-id
  verification needs the tree's entity registry data, which the
  suite does not load — same shape as the leakage audit's job.
- **Data-driven ids.** The `minecraft:block_transformer` registry
  loads transform definitions (including their `loot` ids) from the
  prepared resources at runtime. The check sees only the in-tree
  definitions (26.1/26.2) and the in-tree special-case (1.21.7) —
  it cannot see a wrong id that lives only in prepared data.
- **Block drops.** Block breaking rolls `blocks/<id>` dynamically;
  not every block has a table (air, fluids, technical blocks), so a
  per-id check needs the "which blocks should have tables" rule
  first. The `loot-fields` check covers the evaluator side.

## Running it

```sh
# One version (server jar; the client jar works too — it carries the
# same loot_table tree — and is the fallback when no server jar exists)
python3 tools/defect_suite.py 1.21.7 --jar jars/1.21.7-server.jar \
    --checks loot-tables

# Every version, report-only (the CI defect-suite job's shape)
python3 tools/defect_suite.py --all --jar-dir jars/ \
    --checks loot-tables --report-only
```

No Rust toolchain, stdlib only. It runs in the `defect-suite` job
of `.github/workflows/crate-tests.yml`, which downloads the version's
server and client jars from Mojang (sha1-verified) — no workflow
change was needed. Exit codes follow the suite: 1 on failure,
3 unmeasured, 0 clean (or `--report-only` forces 0).
