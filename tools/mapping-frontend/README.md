# Mappings front-end

Pipeline step 3 of the per-version playbook (see `docs/ROADMAP.md` §4-P1):
every spec tool downstream — Vineflower decompiles, bytecode queries, the
Java exporters, oracle compilation — assumes readable class names, and every
Minecraft jar before 26.1 is obfuscated. This front-end is the step that
remaps an obfuscated jar to its real names, once, so the rest of the pipeline
consumes the named jar unchanged.

## Usage

```sh
python3 tools/mapping-frontend/frontend.py 1.21.11          # fetch + remap + stats
python3 tools/mapping-frontend/frontend.py 1.21.11 --prove  # + decompile a sample
python3 tools/mapping-frontend/frontend.py 1.21.11 --force  # refetch everything
python3 tools/mapping-frontend/frontend.py 1.21.11 --to-namespace named --side server
```

Direction is normally detected from the table (per-side probe classes, see
Design §3). `--to-namespace` / `--from-namespace` force one side explicitly;
`--side` picks which probe set to try (`auto` by default: client probes
first, then server probes). The driver itself takes the same choices as
system properties: `-Dmf.to=<ns>`, `-Dmf.from=<ns>`, `-Dmf.side=<side>` (and
`-Dmf.report=<path>` to place the JSON coverage report elsewhere).

The version id is a parameter; anything with official Mojang mappings works
the same way. Requirements: Python 3.10+, a JDK (`javac`, for the one-time
driver compile) and a Java 17+ runtime. Set `JAVAC`/`JAVA` to override tool
discovery (PATH, then JDKs installed under `~/workspace/tools/`).

Everything downloaded or produced lands in `.local/mapping-frontend/`
(git-ignored — jars, mappings and decompiled output are development-time
reference only and are **never committed**):

```
.local/mapping-frontend/
  version_manifest_v2.json        Mojang's manifest (cached, refreshed daily)
  deps/                           pinned tool jars (see below)
  build/classes/                  the compiled remapper driver
  <version>/
    <version>.json                the version's metadata (SHA-1 verified)
    client.jar                    the obfuscated client jar (SHA-1 verified)
    client-mappings.txt           the official mapping table (SHA-1 verified)
    client-named.jar              the output: deobfuscated jar
    mapped-classes.tsv            obfuscated -> named, one line per class
    mapped-classes.json           coverage report (see Design §5)
    sample.jar, proof/            only with --prove: decompiled sample output
```

## Design

1. **Resolve.** The version manifest (piston-meta) gives the per-version
   JSON; its `downloads.client` and `downloads.client_mappings` give the jar
   and mapping URLs. Every hop is verified against the SHA-1 Mojang
   publishes, so a corrupted or swapped file stops the run.
2. **Normalize.** The mapping table is read with Fabric's **mapping-io** —
   the programme's one intermediate representation. Formats plug in here:
   ProGuard txt, SRG/TSRG, tiny v1/v2 (mapping-io auto-detects the format).
   Multi-namespace tables are first-class: the driver resolves a source and
   a target namespace and remaps along the chain between them (see §4).
3. **Detect direction from the data.** Mojang's tables are ProGuard format
   (named → obfuscated on paper), but the front-end does not hard-code which
   tree namespace is which: the driver (`src/Remap.java`) looks up probe
   classes and takes the namespace that carries a probe's real name as the
   target. Probes are per side, tried in order, the first that
   discriminates decides:
   - client: `net/minecraft/client/Minecraft`,
     `net/minecraft/client/player/LocalPlayer`;
   - server: `net/minecraft/server/MinecraftServer`,
     `net/minecraft/server/level/ServerLevel`,
     `net/minecraft/server/players/PlayerList`.
   The source namespace is then the one where the deciding probe class
   carries a different name; with several candidates the truly obfuscated
   one (probe name not under `net/minecraft/`) wins. `-Dmf.from=<ns>` /
   `-Dmf.to=<ns>` force a direction — either may be given alone, and both
   given skips detection entirely (this is how the pipeline's server-jar
   path drives the driver). If no probe can decide, the driver fails naming
   every probe tried and what each found, instead of guessing.
   Note: `MinecraftServer` is identity-mapped in Mojang's 1.21.11 tables
   (kept unobfuscated), so it never discriminates on its own — it verifies
   the table is server-side, and the obfuscated server classes behind it
   (`ServerLevel` → `axf`, `PlayerList` → `bbz` in 1.21.11) decide.
4. **Remap.** Fabric's **tiny-remapper** applies the table to the jar:
   classes, fields, methods (identified by obfuscated name + descriptor,
   with descriptors translated through the same table), plus all non-class
   resources copied through untouched. When the table carries an
   intermediate namespace (tiny v2 `official`/`intermediary`/`named`), the
   driver runs one pass per leg of the chain, so a member missing a named
   name still lands on its intermediate name instead of staying obfuscated.
5. **Report.** The driver dumps the full class table (`mapped-classes.tsv`,
   unchanged format) and a JSON coverage report (`mapped-classes.json`):
   for classes, methods and fields — total entries, how many carry a name
   in the target namespace, how many actually change name, how many target
   names are still placeholders (SRG `func_`/`field_` names or Yarn
   intermediary `class_`/`method_`/`field_` names), and how many members
   fell back to an intermediate name for want of a named one. Class-level
   coverage alone says nothing about members, and for gappy tables the
   member numbers are the health check. The script also intersects the
   class table with the jar's contents and prints jar-side coverage.

Tool dependencies are pinned and fetched on first run (into `.local/deps`,
never committed): tiny-remapper 0.15.0 + mapping-io 0.7.1 (Fabric Maven —
the versions tiny-remapper itself declares), ASM 9.9.1 (Maven Central),
Vineflower 1.12.0 (Maven Central, proof step only).

## Proof: 1.21.11

Verified end-to-end on the first target (Java 21, matching the version's
own runtime requirement):

- Client jar: 31,152,600 bytes, SHA-1 `ba2df812c2d12e0219c489c4cd9a5e1f0760f5bd`;
  official mappings: 11,779,287 bytes, SHA-1 `031a68bebf55d824f66d6573d8c752f0e1bf232a`.
- The obfuscated jar's entries are names like `fzu$a.class`, `gds.class`.
- **Coverage: 10,291 of 10,291 class files in the jar carry a mapped name
  (100.00%).** 10,229 classes are renamed; the 62 that keep their name are
  `com/mojang/blaze3d/**` library classes, which ship unobfuscated.
  Member-level coverage (new in the generalized driver): **89,557 of
  89,557 methods** mapped (67,024 renamed) and **45,679 of 45,679 fields**
  mapped (45,233 renamed), **0 placeholder names** — the official table is
  complete at every level.
- **Server side, same tool, no overrides:** the 1.21.11 server jar
  (bundler inner jar, 7,163 classes, SHA-256 verified on extraction)
  remaps through direction detection alone — the server probe chain
  (`MinecraftServer` ambiguous as documented, `ServerLevel` decides) —
  with **7,163 of 7,163 classes** mapped (7,145 renamed), 66,676 of 66,676
  methods and 32,405 of 32,405 fields mapped, 0 placeholders. The output
  is byte-identical in content to the override-driven path
  (`-Dmf.from=target -Dmf.to=source`), which the pipeline keeps using.
- **Multi-namespace chain, synthetic test:** a hand-built tiny v2 table
  (`official`/`intermediary`/`named`, 2 classes) remaps through the chain
  — a method with an intermediary name but no named name lands on its
  intermediate name (`method_457`, counted as fallback), an SRG-style
  `func_12345_a` named name is carried through and counted as a
  placeholder, and `-Dmf.to=intermediary` remaps a single leg to the
  intermediate names. The 1.21.11 client regression above re-runs against
  the generalized driver and produces content-identical output (same
  29,233 entries, 0 differing bytes; only zip entry order varies
  run-to-run).
- Vineflower 1.12.0 decompiles the named jar with real names throughout —
  classes, fields, methods, and the types inside descriptors. From
  `net/minecraft/world/level/Level.java` (957 lines):

  ```java
  package net.minecraft.world.level;
  public abstract class Level implements LevelAccessor, AutoCloseable {
     public static final Codec<ResourceKey<Level>> RESOURCE_KEY_CODEC = ...
     protected final List<TickingBlockEntity> blockEntityTickers = ...
  ```

  and `net/minecraft/client/Minecraft.java` yields `hasShiftDown()`,
  `updateTitle()`, `isGameLoadFinished()`, …

Known limit of the input, not of the tool: Mojang's official tables carry
no parameter or local-variable names, so decompiled output shows synthetic
parameter names (`$$0`). Class, field, method and type names — everything
the spec pipeline keys on — are complete.

## Supported tables, and what comes next

- **Now:** Mojang's official ProGuard tables — published for every release
  from **1.14.4 through 1.21.11**, which is exactly the range this front-end
  accepts. Versions outside it are refused with an explanation, never
  silently mishandled. The driver itself already reads SRG/TSRG/tiny v1/v2
  tables (mapping-io) and remaps multi-namespace tables through their
  intermediate namespace; what is missing for older eras is the per-source
  fetching, not the remap machinery.
- **26.1+ needs nothing:** those jars ship unobfuscated and publish no
  mapping table; the front-end reports them as such.
- **Next (older eras, same seam):** MCPConfig/NeoForm SRG/TSRG tables for
  1.13–1.16.x community coverage, MCP SRG for ≤1.12.2, and Legacy
  Yarn/Ornithe tiny-format tables for the oldest releases. The remaining
  work is per-source fetching and namespace wiring; coverage reporting
  against gappier tables is already in place (member counts + placeholder
  counts, see Design §5).
- **MCP field/method CSV join (plug-in point, not yet built):** MCP-era
  names come from separate field and method CSVs joined to SRG, which is
  not a single-table read. The join belongs in a fetcher step that produces
  **one joined table** (SRG + CSVs merged into a single tiny/TSRG table)
  before the driver runs — the driver consumes any table mapping-io reads,
  so no driver change is needed. That step is part of the 1.12.2 SRG-proof
  work, not of this front-end generalization.
