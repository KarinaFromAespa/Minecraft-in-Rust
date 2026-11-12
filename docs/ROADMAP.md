# Every Minecraft Version in Rust — Roadmap (Desktop)

Prepared 2026-11-11 for V-Cool. Scope: all 103 major Java Edition releases, 1.0 → 26.3,
rewritten in Rust for desktop — a reference reimplementation from decompiled jars,
each version verified against its own jar.
Snapshots/pre-releases (~820 more entries in Mojang's manifest) are out of scope unless
the owner says otherwise; the pipeline below would cover them unchanged.

## 0. Verdict

Feasible, and most of the machinery already exists — because the base project was
designed for exactly this. The programme decomposes into **one shared pipeline, 8 era
projects, and cheap per-release delta ports inside each era**. It is not 103 rewrites.
The one genuinely missing piece of engineering is a **mappings front-end** that lets the
existing tooling read obfuscated jars (everything before 26.1). Build that once, early,
against an easy target, and the rest is sequencing and stamina.

## 1. Foundation: build on upstream desktop Horizon

- The base is **upstream horizon-rust (`ludovic111/horizon`)** — the desktop project.
  Its own design doc (`docs/version-ports.md`) states the
  goal verbatim: *"every Minecraft Java release is turned into Rust by Horizon with as
  little human work as possible."* Upstream already runs this pipeline **forward**
  (each new release). This programme runs it **backward** to every old release.
- What already exists and is validated:
  - **Centralized version pin.** `research-lock.json` (version, jar sha1s, Java major)
    + generated `crates/horizon-core/src/version.rs` (VERSION_ID, PROTOCOL_VERSION,
    DATA_VERSION, pack formats). Everything else aliases these.
  - **Data is runtime-loaded, not compiled in.** Registries, block/item facts, loot,
    biomes are exported from the version's own jars by `horizon-prepare`
    (Mojang's data generator `--reports` + Horizon's Java exporters) into
    `.local/horizon-client/<version>/`. New version data mostly *regenerates*.
  - **`tools/version_port.py`** — detect → download → prepare → diff → regen →
    ranked work items, on a `port/<version>` branch. Validated on the 26.2 → 26.3
    replay: **499 work items** (only 10 rank-1: protocol 776→777, data version
    4903→5023, pack formats, `ids.rs` regen = 131 renumbered packets, DataFixers),
    prepare 9 s, diff 6 s. Adjacent-version ports are a bounded, largely mechanized job.
  - **Parity harness framework** (~330 harnesses): a Java oracle (the version's own
    classes) and a Rust probe run identical inputs; JSON outputs are diffed. Only the
    oracle sources' API usage is version-specific; the framework is not.
- Tree proportions (audit of the tree): roughly **60–65% of the
  semantics are version-specific behaviour** (hand-ported), **35–40% generic engine**
  (renderer, mesher, UI, audio, `jmath`/RNG/noise) that carries across nearby versions.
  This programme is desktop only.

## 2. Decomposition: 103 releases → 8 eras → ~45 protocols

Protocol versions are shared across patch releases (all of 1.8.x = PV 47; 1.20.5 =
1.20.6 = 766; 26.1/26.1.1/26.1.2 = 775), so the 103 releases collapse to roughly
**45 distinct protocol versions** (approximate, from the wiki protocol table).
Era boundaries sit where the game's *internals* change, not where features land:

| Era | Versions (n) | Structural character | Java | Spec material | Distance from 26.3 base |
|---|---|---|---|---|---|
| E7 Unobfuscated | 26.1–26.3 (5) | Mojang ships unobfuscated; year numbering | 25 | None needed | **Base era — done** |
| E6 Components | 1.20.5–1.21.11 (14) | Item NBT → data components (1.20.5) is the one deep break | 21 | Mojang official mappings | Delta ports + one serialization exception |
| E5 Height & config | 1.17–1.20.4 (15) | 1.18 world height/chunk restructure; 1.20.2 protocol Configuration phase | 16→17 | Mojang official | Moderate ports; two internal breaks |
| E4 Mojmap plateau | 1.15–1.16.5 (9) | Datapack-driven worldgen begins | 8 | Mojang official + Yarn + MCPConfig | Moderate ports |
| E3 Flattening | 1.13–1.14.4 (8) | Numeric IDs abolished; block states; per-section palettes | 8 | Split: 1.13–1.14.3 community only; official from 1.14.4 | Structural |
| E2 Late numeric-ID | 1.9–1.12.2 (14) | ID+meta data model; paletted chunk *wire* format (1.9) | 8 | MCP golden age (hole: no SRG for 1.10.1) | Structural rewrite of the data model |
| E1 Netty protocol | 1.7.2–1.8.9 (19) | Netty protocol rewrite (PV resets); JSON block models (1.8) | 8 | MCP SRG per release | Structural rewrite |
| E0 Pre-Netty legacy | 1.0–1.6.4 (19) | Pre-Netty protocol; client/server merge at 1.3; Anvil at 1.2.1 | 8 (era: 5–7) | Thinnest: early MCP, Ornithe | Rewrite-grade; smallest game, worst docs |

## 3. The standard per-version pipeline (the playbook every agent runs)

From the tree's own `version_port.py` flow; class = regenerate / adapt / rewrite.

| # | Step | Class |
|---|---|---|
| 1 | Fetch the version JSON + client/server jars from Mojang's manifest; sha1-verify | mechanical |
| 2 | Write the version's `research-lock.json`; select its JDK (8/16/17/21/25, read from the version JSON — never hardcode) | mechanical |
| 3 | **Deobfuscate** (pre-26.1 only — the new front-end, §4-P1) → `lab.py prepare` + datagen reports + class index | regenerate |
| 4 | Diff old..new (facts, packets, registries, blocks, components, commands, changed classes by bytecode hash; decompile only changed classes) | mechanical |
| 5 | Regen the 8 committed tables (`version.rs`, packet `ids.rs`, entity tracking, command tables ×3, block-state data) on branch `port/<version>` | regenerate |
| 6 | Re-run `horizon-prepare` (runtime data: registries, gameplay facts, items, worldgen bundles, assets) | regenerate if exporters compile |
| 7 | Fix Java exporters/oracles that no longer compile against the new jar (javac names each moved API) | adapt |
| 8 | Run the full parity suite vs the previous version's baseline; triage newly-failing | mechanical run + triage |
| 9 | Port the changed/new behaviour the diff and failing harnesses identify (incl. new DataFixer schemas) | adapt (nearby) / rewrite (era breaks) |
| 10 | Gate: pin-bump merge + playtest — world loads; a vanilla client of that version joins a Horizon LAN world of that version | gate |

## 4. The hard problems — issue, solution, hindrance, workaround, efficiency verdict

### P1. The obfuscation wall (the programme's one real blocker)
- **Issue:** The whole spec pipeline (Vineflower decompile, REA/JADX bytecode queries,
  Java exporters, oracle compilation) assumes readable names. It *stops* on any jar
  with `client_mappings` — i.e. **every version before 26.1**. As shipped, the pipeline
  covers 5 of 103 releases.
- **Solution:** Build a **mappings front-end**: a step before `lab.py prepare` that
  remaps the obfuscated jar to named form using that era's mapping tables — Mojang
  official (1.14.4–1.21.11), MCPConfig/NeoForm SRG→official (1.13–1.21.x), MCP SRG
  (≤1.12.2), Legacy Yarn (1.3–1.13.2), Ornithe (oldest). Normalize all formats
  (ProGuard txt / SRG / TSRG / tiny) to one intermediate, remap once, and every
  downstream tool consumes the deobfuscated jar unchanged.
- **Hindrance that remains:** Old community mappings have gaps — partial member names,
  one known hole (no SRG zip for 1.10.1), and names that are community-assigned, not
  Mojang's, for everything ≤1.16. Cross-version diffs then compare different name
  systems at era boundaries.
- **Workaround:** Treat names as *hints*, identity as *structural*: the diff step
  already keys classes by bytecode hash and methods by descriptor; where a name is
  missing, REA queries by descriptor still work, and the parity oracle (the running
  jar) never depended on names at all. For 1.10.1 specifically, interpolate mappings
  from 1.10/1.10.2 (adjacent, tiny delta) and hand-verify the residue.
- **Efficient enough?** **Yes — this is the highest-leverage build in the programme.**
  One front-end unlocks 98 versions; every alternative (hand-reading obfuscated
  bytecode per version) is strictly worse. Build it against 1.21.11 first, where
  Mojang's mappings are complete, so the tool itself is verified on perfect input
  before it ever meets a gappy one.

### P2. One version per build → N lines to maintain forever
- **Issue:** The pin is compiled in; upstream explicitly scopes multi-version-in-one
  build **out**. A finished programme = up to 103 branches/builds. Every later engine
  fix would land N times, or the lines diverge into 103 fossils.
- **Solution:** Follow the tree's own model — branch per version (`port/<version>`) —
  but maintain at **era granularity**: 8 era lines are the living codebases; releases
  inside an era are pins regenerated from the era line, not independent forks.
  Version-agnostic crates (core math, renderer, UI, audio) are the "commons": fixes
  land on the newest line and are cherry-picked *down* era lines by a standing
  commons-sync pass, with the parity suite as the regression net. A concrete
  restructure sketch implementing this (commons + thin per-version overlays +
  an assemble step) is in `docs/restructure.md` — proposal, not yet executed.
- **Hindrance:** Cherry-picks across era boundaries will sometimes conflict where
  behaviour genuinely diverged (that's the point of the eras). Some fixes simply
  won't apply to old eras — correctly.
- **Workaround:** Accept a **freeze policy**: an era line is actively maintained while
  its era is being built and for a soak period after; then it freezes as a finished
  artefact (releases don't change — a frozen, parity-verified 1.8.9 stays correct
  forever, unlike a live service). Maintenance cost is thereby bounded by design.
- **Efficient enough?** **Yes, with the freeze policy; no, without it.** The
  alternative — one multi-protocol binary (Stevenarella-style per-protocol layers,
  or ViaVersion-style runtime translation) — caps fidelity at "can join and play",
  which contradicts the programme's purpose (behavioural identity, harness-verified).

### P3. Behaviour is hand-ported; silent divergence is the standing risk
- **Issue:** Data regenerates; behaviour doesn't. ~60% of the tree mirrors specific
  vanilla classes, and the parity net has holes the pipeline itself flags (on the
  26.2→26.3 replay, only 173 of 407 rank-2/3 work items had a comment-named changed
  method — the rest are leads, not certainties). A version can pass every harness and
  still diverge somewhere no harness looks.
- **Solution:** Keep the existing doctrine and scale it: diff ranks work items;
  every rank-3 gap (ported code with no harness) gets either a new harness or an
  explicit "accepted gap" entry in that version's ledger — no silent unknowns.
- **Hindrance:** Harness-writing is the slowest, least automatable work in the
  programme, and it scales with behaviour surface (worst in E6–E3 where the game is
  biggest).
- **Workaround:** Prioritize harnesses by *player-visible* behaviour (movement,
  combat, worldgen seeds, redstone, mob AI) and let cosmetic/long-tail systems ride
  on the diff + playtest gate. Sample-based fuzzing against the oracle server
  (random inputs, compare state) covers ground harnesses don't, cheaply.
- **Efficient enough?** **Yes for E7–E3; qualified for E2–E0** — there the spec
  material is thinner (P1 gaps), so the fidelity bar per era is a real decision for
  the owner (§6): full parity, or "harness-covered core + documented gaps".

### P4. Old-version oracles: dead auth, a five-JDK matrix, headless clients
- **Issue:** Behaviour verification needs each version's jar *running*. Legacy auth
  servers are dead; pre-1.7.6 skin services are gone; each era needs its own JVM
  (8/16/17/21/25); pre-1.13 clients are LWJGL 2 and dislike headless displays.
- **Solution:** The **server jar is the primary oracle** — present for every release
  from 1.2.5 up, runs headless natively, its datagen `--reports` dump machine-readable
  ground truth, and `online-mode=false` removes the dead auth dependency entirely
  (it never calls Mojang). The five JDKs are all current, obtainable releases —
  a matrix to manage, not a blocker; the version JSON states which one each
  version wants.
- **Hindrance:** 1.0 has no server jar (client-only), and pre-1.3 single-player is
  client-internal, so the server-oracle pattern doesn't cover E0's first releases.
  Client-side behaviours (rendering, input feel) can't be oracle-tested headlessly
  at scale in old eras.
- **Workaround:** For 1.0–1.2.5, run the *client* as its own oracle in offline
  single-player under Xvfb (the integrated/internal world needs no auth), and keep
  those versions' harness set logic-only where possible. Client-feel items go to
  the playtest gate (§3 step 10) instead of automated parity in every era.
- **Efficient enough?** **Yes.** Servers carry ~all of the behavioural surface the
  harnesses test (the existing suite already works this way); the E0 exception is
  3 small, old, simple releases, hand-carried once.

### P5. Decompiled code is not truth
- **Issue:** Decompilers lose `throws` clauses and generic signatures; FernFlower-era
  output historically needed hundreds of hand patches to even compile (MCP's
  exceptor `.exc` files exist for this reason). Any automation that treats a
  decompile as ground truth imports its errors.
- **Solution:** The tree's existing doctrine, kept absolute: decompiled sources and
  REA are **locating tools** (find the class/method, read the algorithm); the
  **running jar via the parity oracle is the only truth**. Nothing decompiled is
  ever recompiled into the product or shipped (also a hard legal line, §5).
- **Hindrance:** Reading-aid quality degrades on old bytecode (Java 5/6-era class
  files, stripped generics) — RE on E0–E2 is slower per class than on E6.
- **Workaround:** Lean harder on REA's descriptor-level queries and on oracle
  probing (ask the running server, don't read the code) for old eras; budget RE
  time inversely to era age — old versions are *smaller* games with *worse* docs,
  so per-version totals stay bounded.
- **Efficient enough?** **Yes** — this is precisely how 26.3 was built; the
  workaround is the method, not a compromise of it.

### P6. Programme logistics: materials, compute, pacing
- **Issue:** Each version's local materials (jars, libraries, decompiled sources,
  reports) run to hundreds of MB (26.3's shared set: 366 MB); ×103 versions plus
  build trees is hundreds of GB if hoarded. This VM's CPU (1.5 shared cores) cannot
  run 103 versions' builds/harnesses. And the owner's weekly token limit paces how
  much agent work can run per week (current stop until Mon Nov 16, 2026).
- **Solution:** Materials are **reproducible, not precious**: jars re-download from
  Mojang's CDN in seconds, decompiles regenerate, so keep only the active era's
  materials hot; cold versions keep just their `research-lock.json`, diff reports,
  parity baselines and the branch. Builds and full parity runs go to **CI**
  (per-branch GitHub Actions, desktop), agents do code + triage; this VM is the
  coordinator, not the factory.
- **Hindrance:** CI time becomes the throughput ceiling for step 8 (full-suite
  runs); era waves must queue behind it.
- **Workaround:** The parity shim already caches oracle results by input hash —
  Rust-only changes don't re-pay the JVM side — and era waves run their suites
  staggered, template first, deltas after (deltas re-run only affected scopes
  via `tools/parity.py --changed`).
- **Efficient enough?** **Yes.** Nothing here needs new infrastructure — it needs
  the discipline of not hoarding and of treating CI as the factory floor.
  The pacing constraint is real but it's a schedule fact, not a technical risk:
  the phases in §5 are sized to run wave-per-reset-cycle.

## 5. Sequencing — eras in build order, with gates

Order rationale: build new machinery against the easiest verifiable target first;
cross each structural boundary once, deliberately; never start with the worst
tooling (chronological order from 1.0 would do exactly that).

- **Phase 0 — Prove the loop + build the front-end.**
  26.1 end-to-end through the existing pipeline (validates `version_port.py` in our
  hands), then the **mappings front-end (P1) proven on 1.21.11** (perfect official
  mappings; E6's template). *Gate: 1.21.11 parity profile ≈ its baseline; front-end
  remaps a gappy SRG jar (1.12.2) without manual rescue.*
- **Phase 1 — The Mojmap belt (E6 → E5 → E4).** Era templates 1.21.11, then 1.20.4
  (crossing components *upward* in reverse — the 1.20.5 boundary gets its own
  mini-gate), 1.18.2 (height break), 1.16.5; in-era releases as delta ports after
  each template. *Gate per era: template passes suite + playtest; deltas are then
  factory work.*
- **Phase 2 — The Flattening (E3).** Template 1.14.4 (official mappings), then down
  through 1.13.x on community mappings. This is the single largest behavioural
  delta in the game's history; it gets a phase to itself. *Gate: a 1.12.2 world
  upgraded through the new DataFixer ladder loads identically in both.*
- **Phase 3 — The MCP eras (E2, E1).** Templates 1.12.2, 1.8.9. Numeric-ID data
  model returns as a *runtime* model (the registry machinery is ID-based anyway;
  the work is in behaviour and chunk formats, of which there are three distinct
  wire/storage shapes to support). *Gate: cross-era LAN playtest per template.*
- **Phase 4 — Legacy (E0).** 1.6.4 down to 1.0. Smallest games, worst material,
  client-as-oracle exceptions (P4). By now every tool has been hardened twice.
  *Gate: owner's fidelity-bar decision for this era (§6) applied and documented.*

After Phase 4: 103/103. Snapshots, if ever wanted, slot into their era as deltas.

## 6. Decisions for the owner

1. **Scope line:** 103 major releases (this plan) — snapshots in or out?
2. **Repo shape:** one upstream-based repo, `port/<version>` branches, era lines
   maintained + freeze policy (recommended, §4-P2) — or separate repos per era?
3. **Fidelity bar for old eras (E2–E0):** full parity (slowest, most certain) or
   harness-covered core + documented accepted gaps (faster, honest)?
4. **Base checkout:** work from a fresh clone of upstream `ludovic111/horizon`
   main (recommended), used as the base as-is.
5. **Start:** Phase 0 begins after the Nov 16 limit reset, on your word.

## 7. Distribution posture (unchanged from Horizon's, kept explicit)

Nothing Mojang-derived ever ships: jars, mappings, decompiled sources are
development-time reference only (the mappings' own license forbids redistributing
them; the EULA governs the rest). Players run `horizon-prepare` against **their
own** Minecraft copy per version, exactly as 26.3 works today. The product is
only ever the reimplemented Rust code and the player's own prepared data.

---
*Evidence base: Mojang version manifest + per-version JSONs (fetched 2026-11-11);
minecraft.wiki protocol/data-version tables; mappings-license headers from Mojang's
own mapping files; audit of the Horizon tree (pin, protocol crate, prepare/REA
tooling, parity framework, `docs/version-ports.md` incl. the 26.2→26.3 replay);
upstream `ludovic111/horizon` main (pins 26.3; version-port tooling is upstream's).*

## Addendum — adopted setup (2026-11-11, owner's decisions)

- Programme repo: **KarinaFromAespa/Minecraft-in-Rust** (this repo). All rewrite
  commits are made under the KarinaFromAespa account only.
- Organization: `main` is the only branch — the programme home (this plan,
  VERSIONS.md, shared tooling) and every version's rewritten code, each version
  in its own directory `versions/<id>/` with its documentation alongside.
- §6 decisions settled by this setup: (2) repo shape — one repo, single main
  branch, versions as directories, era lines with the freeze policy; (4) base —
  a fresh clone of upstream `ludovic111/horizon` per version when its work starts.
  Still open: (1) snapshot scope, (3) the old-era fidelity bar, (5) the
  Phase 0 start, on the owner's word.
