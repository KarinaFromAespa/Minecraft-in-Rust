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
```

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
    sample.jar, proof/            only with --prove: decompiled sample output
```

## Design

1. **Resolve.** The version manifest (piston-meta) gives the per-version
   JSON; its `downloads.client` and `downloads.client_mappings` give the jar
   and mapping URLs. Every hop is verified against the SHA-1 Mojang
   publishes, so a corrupted or swapped file stops the run.
2. **Normalize.** The mapping table is read with Fabric's **mapping-io** —
   the programme's one intermediate representation. Formats plug in here:
   ProGuard txt today (mapping-io auto-detects it); SRG/TSRG/tiny at the
   same seam when older eras are wired up.
3. **Detect direction from the data.** Mojang's tables are ProGuard format
   (named → obfuscated on paper), but the front-end does not hard-code which
   tree namespace is which: the driver (`src/Remap.java`) looks up a probe
   class (`net/minecraft/client/Minecraft`) and takes the namespace that
   carries its real name as the target. `-Dmf.from=<ns>` / `-Dmf.to=<ns>`
   force a direction for future table types.
4. **Remap.** Fabric's **tiny-remapper** applies the table to the jar:
   classes, fields, methods (identified by obfuscated name + descriptor,
   with descriptors translated through the same table), plus all non-class
   resources copied through untouched.
5. **Report.** The driver dumps the full class table; the script intersects
   it with the jar's contents and prints coverage — how many of the jar's
   classes carry a mapped name, and how many actually change name.

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
  silently mishandled.
- **26.1+ needs nothing:** those jars ship unobfuscated and publish no
  mapping table; the front-end reports them as such.
- **Next (older eras, same seam):** MCPConfig/NeoForm SRG/TSRG tables for
  1.13–1.16.x community coverage, MCP SRG for ≤1.12.2, and Legacy
  Yarn/Ornithe tiny-format tables for the oldest releases. mapping-io
  already reads all of these formats; the work is per-source fetching,
  namespace wiring, and coverage reporting against gappier tables.
