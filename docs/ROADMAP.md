# Every Minecraft Version in Rust — Roadmap (Desktop)

Prepared 2026-11-11 for V-Cool; target reframed 2026-11-12 (see §0).
Scope: all 103 major Java Edition releases, 1.0 → 26.3, rewritten in Rust
for desktop — a reference reimplementation from decompiled jars.
**The deliverable is 11 template releases at full parity and 92
delta-verified releases** — not 103 rewrites. "All versions" is a
statement about reachability, correctness of behaviour changes and data
compatibility, not about 103 independent behavioural clones.
Snapshots/pre-releases (~820 more entries in Mojang's manifest) are out
of scope unless the owner says otherwise (open decision, §6); the
pipeline below would cover them unchanged, as deltas.

## 0. Verdict — the target, restated

> **Every major Java Edition release, 1.0 → 26.3, runs in Rust on desktop
> from the player's own copy of that release. Eleven releases — the era
> templates — are held to full one-to-one behavioural parity with their
> own jar. The other ninety-two are guaranteed correct on everything
> that release changed, correct on data, and honest about the rest in a
> signed ledger.**

What changed from the 2026-11-11 draft, and why:

1. **The unit of maintenance is the era line: 8 lines, 11 template
   trees.** E5 needs two (1.18.2 for the height break, 1.20.4 for the
   configuration phase) and E0 needs three (1.6.4, 1.2.5 for Anvil, 1.0
   because it has no server jar). Releases inside an era are pins over
   the era line, never forks. Eight, eleven and ninety-two are the only
   counts used below.
2. **Two acceptance bars, written down** (§8). A build either passes
   the template bar or the delta bar; there is no third, softer state,
   and no status word that means "compiles".
3. **Cost is estimated before work starts, per port** (§9): ~2,510
   port-days likely case, ~8.4 agent-years, ~2.8 years wall-clock at
   three concurrent streams.
4. **Verification is budgeted as compute** (§10): ~7,900 CPU-hours /
   ~2,000 runner-hours, about $1k hosted, against a 648-harness suite
   rather than the 281 the earlier drafts assumed. Money is not the
   constraint; oracle labour and cold-sweep latency are.

What we explicitly stop claiming: 103 versions at behavioural identity
with vanilla. That bar, applied to all 103 releases, is ~2× the cost of
this plan with no additional user-visible correctness inside an era,
and it is not reachable inside this decade by any team.

Three guarantees hold for **every** release, template or delta:

- **G1 — Pin.** The tree builds only against that release:
  `research-lock.json` (version, jar hashes, Java major) plus the six
  regenerated tables, all derived from that release's own jar.
- **G2 — Prepare.** `horizon-prepare` runs end to end from the player's
  install and produces the runtime data set for that release.
- **G3 — Play.** A vanilla client of that exact version joins a Rust
  LAN world of that version, and a world saved by that version loads
  in the Rust build and round-trips back.

Three more hold for the **eleven templates** (T) and are replaced by
the delta bar (D) elsewhere (§8).

The evidence is in this repo and it all points the same way: the
mechanical ladder is cheap, the behaviour is not, and verification is
the ceiling. Measured figures are marked ⓜ, estimates ⓔ, live
external sources ⓤ.

| Observation | Figure | Consequence |
|---|---|---|
| Adjacent same-era port, 26.3 → 26.1, differs in ⓜ | 135 of 5,914 files | The tree is a copy; the port is a patch, and the patch is small |
| …yet that release lists ⓜ | 795 ranked work items (3,281 classes / 18,452 methods changed) | The patch that *compiles* is ~1/20th of the patch that *behaves* |
| Era-opening 1.21.11 | ⓜ 815 items (10/492/247/64/2), 148 files touched, 0 behaviour items closed | A day of agent work bought the scaffolding, not the port |
| CI profile, 17-harness subset — 1.21.11 ⓜ | 2 pass / 2 oracle-broken / 13 fail | "In progress" is accurate; "port" is not yet |
| CI profile — 26.1 and 26.2 ⓜ | 3 pass / 14 fail each | Same, one era up |
| Harness suite in the 26.3 tree ⓜ | **648** runnable `test_*_parity.py` scripts (`parity.py` globs them); the README's 281 is its 2026-10-13 audit count, since outgrown | The verification surface is ~2.3× what both plans budgeted |
| Upstream 26.3 project status ⓤ | **"Player-visible gaps: 0 open of the 833 the port ledger found"** — every subsystem listed at 0 (live README, 2026-11-12) | One version *did* reach its audit's zero. That is the template argument and its price tag: ~830 items closed per version, so 92 versions at that bar is ~76,000 items |
| Harness oracles broken by one era boundary (1.21.11) ⓜ | 6 of the 17 profiled (35%): chunk-generation stages, first-person swing, component types | Oracle porting, not harness writing, is the recurring tax |

Two conclusions follow. **First**, per-release behavioural identity
does not scale: the cheapest possible port still carries ~800 ranked
items, so 92 of them at full fidelity is 92 rewrites wearing a delta's
clothes. **Second**, the scarce resource is the oracle corpus, which
must compile against each era's jar — so the programme should pay that
cost eleven times, against eleven templates, and never again inside an
era.

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
  - **Parity harness framework** (648 harnesses): a Java oracle (the version's own
    classes) and a Rust probe run identical inputs; JSON outputs are diffed. Only the
    oracle sources' API usage is version-specific; the framework is not.
- Tree proportions (audit of the tree): roughly **60–65% of the
  semantics are version-specific behaviour** (hand-ported), **35–40% generic engine**
  (renderer, mesher, UI, audio, `jmath`/RNG/noise) that carries across nearby versions.
  This programme is desktop only.

## 2. Decomposition: 103 releases → 8 eras → 11 templates → ~45 protocols

Protocol versions are shared across patch releases (all of 1.8.x = PV 47; 1.20.5 =
1.20.6 = 766; 26.1/26.1.1/26.1.2 = 775), so the 103 releases collapse to roughly
**45 distinct protocol versions** (approximate, from the wiki protocol table).
Era boundaries sit where the game's *internals* change, not where features land.
Each era has one template release held to full parity (two for E5, three for E0);
every other release in the era is a delta over its template (§8.2).

| Era | Releases | Template release(s) | Why there / interior gate | Java | Spec material |
|---|---|---|---|---|---|
| E7 Unobfuscated | 26.1–26.3 (5) | **26.3** (imported base) | Upstream snapshot; needs a truth pass, not a port | 25 | None needed |
| E6 Components | 1.20.5–1.21.11 (14) | **1.21.11** | Top of era; obfuscated → mappings front-end | 21 | Mojang official mappings |
| E5 Height & config | 1.17–1.20.4 (15) | **1.18.2** + **1.20.4** | 1.18 world-height restructure; 1.20.2 configuration phase is its own interior gate | 16→17 | Mojang official |
| E4 Mojmap plateau | 1.15–1.16.5 (9) | **1.16.5** | Datapack worldgen begins; mappings change character | 8 | Mojang official + Yarn + MCPConfig |
| E3 Flattening | 1.13–1.14.4 (8) | **1.14.4** | Largest behavioural delta in the game's history; official mappings from 1.14.4 | 8 | Split: 1.13–1.14.3 community only; official from 1.14.4 |
| E2 Late numeric-ID | 1.9–1.12.2 (14) | **1.12.2** | ID+meta data model; three chunk wire/storage shapes | 8 | MCP golden age (hole: no SRG for 1.10.1) |
| E1 Netty protocol | 1.7.2–1.8.9 (19) | **1.8.9** | Protocol rewrite + JSON models; MCP SRG per release | 8 | MCP SRG per release |
| E0 Pre-Netty legacy | 1.0–1.6.4 (19) | **1.6.4** + **1.2.5** + **1.0** | Anvil at 1.2.1, client/server merge at 1.3, no server jar at 1.0 | 8 (era: 5–7) | Thinnest: early MCP, Ornithe |

Eleven template releases (8 era lines); 92 deltas.

## 3. The standard per-version pipeline (the playbook every agent runs)

From the tree's own `version_port.py` flow; class = regenerate / adapt / rewrite.

| # | Step | Class |
|---|---|---|
| 1 | Fetch the version JSON + client/server jars from Mojang's manifest; sha1-verify | mechanical |
| 2 | Write the version's `research-lock.json`; select its JDK (8/16/17/21/25, read from the version JSON — never hardcode) | mechanical |
| 3 | **Deobfuscate** (pre-26.1 only — the new front-end, §4-P1) → `lab.py prepare` + datagen reports + class index | regenerate |
| 4 | Diff old..new (facts, packets, registries, blocks, components, commands, changed classes by bytecode hash; decompile only changed classes) | mechanical |
| 5 | Regen the 6 committed tables (`version.rs`, packet `ids.rs`, entity tracking, command tables ×3: `command_order.rs`, `command_suggestions.rs`, `vanilla_order.rs`) on branch `port/<version>` | regenerate |
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
- **Adopted shape (2026-11-12 reframe).** Under the 11-template target the
  published repo's current shape — six full copies of one tree, 35,493 files,
  888 MB, ~140 files differing between adjacent versions — is structurally
  wrong: 103 copies is ~14 GB of duplicated source and 103 places for an
  engine fix to land. The target shape is 11 real Rust trees (one per
  template) plus per-release pins holding only metadata:

  ```
  main
  ├── programme/            plan, VERSIONS.md, shared tooling, mappings front-end
  ├── lines/26.3/           ← the only real Rust trees (11)
  ├── lines/1.21.11/
  ├── …
  ├── pins/1.21.10/         research-lock.json, 6 regenerated tables, diff report,
  ├── pins/1.21.9/          ledger, profile.json, accepted-gaps.md — no crate copies
  └── …
  ```

  A delta release is `lines/<era template>` + `pins/<release>` applied by the
  regen step. Builds consume the pin; no tree is forked until a release
  genuinely diverges, at which point it is a template and the era boundary
  was drawn wrong. Note: `docs/restructure.md` sketches a different
  mechanism for the same goal (one commons + overlays + `assemble.py`);
  the two must be reconciled into one execution plan before the
  restructure runs — the gate (`tools/delta_verify.py`) already resolves
  both layouts, so the decision is about the tree shape, not the tooling.
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
- **The oracle tax (2026-11-12 reframe).** The binding constraint is verification,
  not code, and inside verification the binding constraint is the oracle corpus:
  1.21.11 broke 6 of 17 profiled oracles at one era boundary, and every oracle
  must compile against each era's jar. That is why the programme pays the oracle
  cost eleven times, against eleven templates, and never again inside an era —
  and why the version-parameterised oracle layer (§5, A1) is funded in Phase 0.
  The second control is D3, enforced literally (§8.2): a delta closes its
  rank-1 items and its rank-2 items or accepts them with cause and owner, and
  inherits the template's answers for everything else. If "delta-verified"
  drifts into "close all rank-2 items", the plan becomes the 103-rewrite plan
  and the cost model (§9) doubles.
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

Order is unchanged from the 2026-11-11 draft — it was right. What changes is
the exit condition of each phase and the factory/work split inside it.

- **Phase 0 — Prove the loop (in flight).**
  Retargeted by the 2026-11-12 reframe: 1.21.11 reaches **T1–T4** only
  (pin, prepare, build, oracle corpus compiling). The gate moves off
  "prepare runs" and onto "oracle corpus compiles and the full suite
  produces a complete, measured profile — no unmeasured harnesses".
  *Exit: a complete 1.21.11 profile exists, whatever its pass count.*
  (The old gate's second condition — the front-end remapping a gappy
  SRG jar (1.12.2) without manual rescue — moves to Phase 3, where the
  MCP eras need it; the front-end work already done is not wasted.)
- **Phase 1 — Mojmap belt (E6 → E5 → E4).**
  1.21.11 to T; then 1.20.4 across the configuration-phase gate; 1.18.2
  across the height break; 1.16.5. Deltas for E6/E5/E4 run behind each
  template as factory work at the delta bar.
  *Exit per era: template at T, every delta at D, ledger published.*
- **Phase 2 — Flattening (E3).** 1.14.4 to T, then 1.13.x deltas on community mappings. *Exit:
  a 1.12.2 world upgraded through the datafixer ladder loads identically in both.*
- **Phase 3 — MCP eras (E2, E1).** 1.12.2 and 1.8.9 to T. The SRG/TSRG
  front-end source step and the MCP CSV join land here, proven on the
  1.12.2 gappy-SRG gate. *Exit: cross-era LAN playtest per template.*
- **Phase 4 — Legacy (E0).** 1.6.4, 1.2.5 and 1.0 to T, with client-as-oracle under Xvfb for
  1.0–1.2.5. *Exit: owner's fidelity decision recorded in the E0 ledger.*

**Cross-cutting, runs from Phase 0 onward (the acceleration that pays for itself):**

- **A1 — Version-parameterised oracle layer.** ⓔ 40–60 port-days, one Java source tree with
  reflective/`MethodHandle` shims for the APIs that move every era (registry loading, level
  construction, chunk stages, component access, swing/first-person). Collapses template oracle
  repair from 25–225 port-days per era to ~10–20, and is the single highest-leverage build after
  the mappings front-end (which is done).
- **A2 — Ordinal harness profiles.** Stop measuring with a 17-harness subset that was chosen for one experiment. Define, per era, a fixed regression subset (~40 harnesses, scoped to that
  era's behaviour surface) and use exactly that subset for every delta in the era.
- **A3 — Sharded `port-verify`.** One job with a 300-minute cap cannot hold a full sweep; see §10.
- **A4 — Content-hash pass cache across version trees.** Upstream already shares passes between
  worktrees by content hash ⓜ; extend the key to include the pin so deltas inherit template
  passes whose inputs are unchanged. This is the difference between a 40-minute and a
  1,700-CPU-hour cold sweep.

After Phase 4: 11 templates at T, 92 deltas at D. Snapshots, if ever wanted, slot into their era as deltas.

## 6. Decisions for the owner

1. **Delta bar enforcement (D3).** Adopted by this rewrite: the rank-1 /
   rank-2 with accepted-gaps rule as written in §8.2. Everything in §9
   scales off this answer; changing it later re-opens the cost model.
2. **E0 fidelity.** Full parity for 1.0–1.6.4 (336–756 port-days) or
   harness-covered core plus a published accepted-gap ledger (~half)?
   **Open — recommended the latter.** This is also the lever that would
   hold the template count at 9 (demoting 1.2.5 and 1.0 to gated
   deltas, ~250 port-days).
3. **Snapshot scope.** 103 major releases only (recommended — snapshots
   enter as deltas later at ~9 port-days each if wanted). **Open.**
4. **Repo shape.** Adopted in target (§4-P2): one tree per era template
   with `pins/<release>`. Execution timing vs the in-flight E6 chain is
   still open, as is reconciling the two sketches (`docs/restructure.md`
   vs §4-P2).
5. **Streams.** One agent stream (~8 years) or three with one reviewer
   (~2.5–3 years)? **Open.** The plan's wall-clock, not its
   correctness, depends on this.
6. **Terminology.** Retire "clean room" in favour of "reference
   reimplementation, oracle-verified"? **Done** (2026-11-12, batch 1).
7. **Phase 0 exit.** Re-gate Phase 0 on "1.21.11 oracle corpus
   compiles + complete measured profile"? **Adopted by this rewrite**
   (see §5, Phase 0).

## 7. Distribution posture (unchanged from Horizon's, kept explicit)

Nothing Mojang-derived ever ships: jars, mappings, decompiled sources are
development-time reference only (the mappings' own license forbids redistributing
them; the EULA governs the rest). Players run `horizon-prepare` against **their
own** Minecraft copy per version, exactly as 26.3 works today. The product is
only ever the reimplemented Rust code and the player's own prepared data.

**Language.** Reading Vineflower output and reimplementing from it is a
**reference reimplementation verified against a running oracle** —
legitimate, and accurately described that way. Upstream's own
`docs/cleanroom/` shows what clean room means there (two isolated teams,
a spec as the only channel); this workflow is not that, and a clean-room
claim that can be disproved by opening the port notes is worse than no
claim at all. The term is retired from this repo's own docs.

## 8. The two acceptance bars

### 8.1 Template bar (T) — eleven releases

All of T1–T7 must hold, evidenced by a recorded run:

- **T1 Pin & prepare.** Six tables regenerated from that release's jar; `horizon-prepare` green
  end to end from a staged launcher directory (as `port-verify` does today).
- **T2 Build.** `cargo check --workspace --all-targets`: 0 errors, 0 warnings.
- **T3 Protocol.** `cargo test -p horizon-protocol` green except the two sandbox-network tests
  (out of scope by policy, recorded by name).
- **T4 Oracle corpus.** Every oracle in the suite compiles against that release's jar. No
  `oracle no longer compiles` verdicts may remain in the profile.
- **T5 Harness suite green.** The full suite (not a 17-harness subset) passes, or every failure
  is listed in that release's ledger as an accepted gap with a named cause and an owner. No
  unmeasured harnesses.
- **T6 Ledger closed.** `port_ledger.py` gaps for that release: every gap is ported, justified
  as not-a-port, or an accepted gap with a cause. The count is published.
- **T7 Play gate.** G3 above, plus a recorded playtest checklist (world load, movement, combat,
  containers, redstone sample, worldgen seed match, server join).

### 8.2 Delta bar (D) — ninety-two releases

Cheaper by construction, and still falsifiable. All of D1–D6, implemented
by `tools/delta_verify.py` and specified in `docs/DELTA-BAR.md`:

- **D1 Pin & prepare.** As T1. Tables regenerated from that release's jar; the
  committed tables match a fresh regen (determinism check).
- **D2 Build & protocol.** As T2/T3.
- **D3 Changed-scope behaviour.** Every rank-1 work item closed, and every rank-2 item either
  closed or listed as an accepted gap. "Changed scope" means the items the diff ranks 1–2;
  rank-3+ items inherit the era template's answer unless that release changed them. **This is
  the definition that makes 92 deltas affordable, and it must be enforced literally.**
- **D4 Data compatibility.** Save round-trip in both directions plus the `datafix` harness shard
  green for this release (this is what actually breaks silently between releases).
- **D5 Harness subset.** The era's regression subset — the ~40 harnesses whose scope the release
  touched, selected by `parity.py --changed` — green or accepted-gap-listed.
- **D6 Play gate.** G3 for that release.

A release that meets D1–D6 is **delta-verified**. It may not be claimed as parity-verified, and
should not be.

### 8.3 Status vocabulary

Two axes, both numbers, no adjectives:

```
versions/1.21.11   D/T: T   profile: 17/17   ledger: 743/743   gaps-accepted: 0
versions/1.21.10   D/T: D   profile: 40/40   ledger: 112/531    gaps-accepted: 419
versions/1.20.6    D/T: D   profile: 38/40   ledger: 96/510     gaps-accepted: 412
```

The only verbs are `prepared`, `delta-verified`, `parity-verified`, `frozen`. A tree that
compiles and prepares but has not met its bar is `prepared`, however much of the ladder is done.
`VERSIONS.md` rows are generated from `delta-verdict.json` files
(`tools/delta_verify.py --index`); rows migrate to generated verdicts as
releases are gated, and the hand-maintained scalars retire row by row.

## 9. Cost model per port

Not lines of Rust, and not prompts: **port-days**, where one port-day is
one agent session at the throughput this programme has actually
demonstrated. §10 estimates the machine side.

### 9.1 Anchors (measured where marked)

| # | Anchor | Value | Source |
|---|---|---|---|
| A1 | Mechanical ladder for one adjacent, officially-mapped release (pin, 6 tables, prepare, compile, protocol rewire) | 0.2–1.0 port-day | ⓜ 5 trees / 35 commits / ~2 days, incl. protocol rewires that closed 438 protocol tests |
| A2 | Behaviour work per adjacent port | 795–815 ranked items ⓜ; closing rate ⓔ 20–60 items/port-day | ⓜ work-item counts; ⓔ rate (see note) |
| A3 | Oracle repair per era break | ⓔ 20–80% of the oracle corpus, 0.25–1.5 port-days each | ⓜ 6 of 17 profiled oracles broken at 1.21.11 |
| A4 | Harness subset run | 17 harnesses, ≤300 min job cap ⓜ; ~15 min/harness cold ⓜ | `port-verify.yml`, port notes |
| A5 | Warm harness cost on an established tree | ~1.5 harness-minutes ⓜ | upstream: 1,061 harnesses → 1,592 harness-minutes |
| A6 | Suite size | **648** runnable `test_*_parity.py` files ⓜ (`parity.py` globs them); the README's 281 is upstream's narrower, stale figure | `tools/parity.py`, `ls tools/test_*_parity.py` |

**Note on A2.** The closing rate is the least evidenced number here and the one that decides the
programme. Use 30 items/port-day as the planning figure until one adjacent delta is timed end to
end (§11); the range 20–60 is honest, and 815 items is the floor from a release *five months*
away from the base.

### 9.2 Per-port estimates (port-days, low / likely / high)

| Port type | n | Mechanical | Behaviour | Oracle repair | Verification & ledger | Total each | Subtotal |
|---|---|---|---|---|---|---|---|
| E7 truth pass (26.3) | 1 | 0 | 0 / 15 / 40 | 0 | 0 / 5 / 10 | 0 / 20 / 50 | 0 / 20 / 50 |
| E6 template (1.21.11) | 1 | 1 | 20 / 27 / 41 | 25 / 45 / 80 | 5 / 10 / 15 | 51 / 83 / 137 | |
| E5 templates (1.18.2, 1.20.4) | 2 | 1 | 45 / 60 / 90 | 40 / 60 / 100 | 10 / 15 / 20 | 96 / 136 / 211 | 192 / 272 / 422 |
| E4 template (1.16.5) | 1 | 1 | 20 / 27 / 41 | 20 / 40 / 70 | 5 / 10 / 15 | 46 / 78 / 127 | |
| E3 template (1.14.4) | 1 | 1 | 60 / 80 / 120 | 60 / 120 / 200 | 15 / 25 / 35 | 136 / 226 / 356 | |
| E2 template (1.12.2) | 1 | 1 | 50 / 70 / 100 | 50 / 100 / 170 | 15 / 20 / 30 | 116 / 191 / 301 | |
| E1 template (1.8.9) | 1 | 1 | 50 / 70 / 100 | 50 / 95 / 160 | 15 / 20 / 30 | 116 / 186 / 291 | |
| E0 templates (1.6.4, 1.2.5, 1.0) | 3 | 2 | 30 / 45 / 70 | 60 / 90 / 140 | 20 / 30 / 40 | 112 / 167 / 252 | 336 / 501 / 756 |
| **Template total (11 releases)** | 11 | | | | | | **~995 / 1,555 / 2,440** |
| **Delta release (D bar)** | 92 | 0.5 | 3 / 5 / 10 | 1 / 2 / 4 | 1 / 2 / 3 | 5.5 / 9.5 / 17.5 | **~506 / 874 / 1,610** |
| Shared tooling (A1–A4, mappings front-end upkeep) | — | | | | | 40 / 80 / 140 | |
| **Programme total** | 103 | | | | | | **~1,540 / 2,510 / 4,190** |

Likely case: **~2,510 port-days ≈ 8.4 agent-years**. At three concurrent agent streams with one
reviewer, **~2.8 years wall-clock**; single stream, ~8.4 years. Compare the current plan's
implicit target — 103 releases each carrying ~800 closed items at ~30/day — which lands at
~4,300–4,700 port-days and never converges because every one of those 103 trees then needs
maintaining.

### 9.3 What the model punishes, and how

The four cells that decide whether this finishes:

1. **Behaviour on deltas (506–1,610 port-days).** Controlled only by the literal enforcement of
   D3. If "delta-verified" drifts into "close all rank-2 items", the plan becomes the 103-rewrite
   plan and the model doubles.
2. **Oracle repair at templates (465–1,300 port-days).** Controlled by A1. This is why A1 is
   funded in Phase 0, not later.
3. **E0 (336–756 port-days for 3 templates).** The smallest games with the worst material. If
   the owner's fidelity bar for E0 is "harness-covered core + accepted gaps", it halves; if it is
   full parity, it does not.
4. **The 800-item floor (A2).** If timed runs show 10 items/port-day rather than 30, multiply the
   behaviour column by three and the programme becomes a decade-scale single-stream effort. This
   is the one number to measure before committing Phase 2.

## 10. Verification compute budget

### 10.1 Measured anchors

- Warm harness: **~1.5 harness-minutes** ⓜ (1,061 selected harnesses = 1,592 harness-minutes).
- Cold harness on a fresh version tree: **~15 minutes** ⓜ (port notes; probe rebuild + that
  version's JVM, no shared pass cache).
- Current CI: one job, `timeout-minutes: 300` ⓜ, 17-harness subset ⓜ. The current workflow
  cannot host a full sweep at any version. (An earlier draft said CI "still leaves 8 of 17
  unmeasured" — that was a local run under VM contention; CI measured all 17.)
- Upstream fleet: Mac mini (14 slots), desktop (8), playhorizon (3 slots on 4 CPUs) ⓜ — ~25
  concurrent harnesses, one queue, passes shared by content hash ⓜ.

### 10.2 Arithmetic

| Line | Warm | Cold (per new tree) |
|---|---|---|
| **Full sweep, 648 runnable harnesses** (`parity.py` globs `test_*_parity.py`) | 648 × 1.5 = **~970 harness-min** (16 CPU-h) | 648 × 15 = **~9,700 harness-min** (162 CPU-h) |
| Full sweep at the floor of 281 (if that is really the oracle-backed subset) | ~420 harness-min (7 CPU-h) | ~4,200 harness-min (70 CPU-h) |
| Era regression subset (~40) | ~60 harness-min (1 CPU-h) | ~600 harness-min (10 CPU-h) |
| Cold-seed multiplier incl. probe rebuilds, oracle javac, data prep | ×1 | **×8–10** vs warm (observed: 40 min warm-equivalent → 300 min timeout) |

Programme total under this plan:

| Activity | Sweeps | Unit | CPU-hours |
|---|---|---|---|
| Template full sweeps (11 releases × 12 sweeps during behaviour work) | 132 | 16 CPU-h warm | ~2,140 |
| Template cold seeds (11 releases × 2) | 22 | 162 CPU-h | ~3,560 |
| Delta sweeps (92 × 3 sweeps of the ~40 subset) | 276 | 1 CPU-h | ~280 |
| Delta cold seeds (92 × 1, offset by A4 caching) | 92 | 10 CPU-h | ~920 (→ ~200 with content-hash reuse) |
| Flake re-runs (~15% allowance ⓔ) | — | ~15% | ~1,040 |
| **Total** | | | **~7,900 CPU-hours (~7,200 with A4 reuse)** |

One full sweep of all 103 releases is **648 × 103 ≈ 66,700 harness executions** — the number to
quote when someone asks what "verified" costs.

At 4 cores per runner: **~2,000 runner-hours**. GitHub-hosted 4-core Linux is roughly
$0.008/min ≈ $0.48/h, so **~$950 hosted**, or effectively free on the existing fleet. Set the
budget at **$3,500 / 2,500 runner-hours** with headroom, and treat CI money as a non-issue.

### 10.3 The real constraint is shape, not spend

Three consequences, all engineering:

1. **Shard the workflow.** A warm full sweep is ~970 harness-min against a 300-minute job cap, so
   ~10 matrix jobs of ~65 harnesses each (~97 min warm) fit; a cold seed (~975 min per slice)
   needs ~20 slices or a self-hosted runner before any template can reach T5. Note that
   `PARITY_SHARD` is a different lever: it splits one expensive harness's cases (`datafix` runs
   as six shards ⓜ) and does not split the harness set.
2. **Pre-warm per version.** The ×8–10 cold multiplier is probe rebuilds, oracle `javac` and data
   prep — all cacheable. A template must publish its built probes, compiled oracles and prepared
   data set as an artifact, and deltas must consume them. This is the difference between 2,880 and
   ~400 CPU-hours on cold seeds.
3. **Wall-clock is the cost, not dollars.** Iteration latency on a cold tree is ~5 hours; a
   behaviour port needs 10–30 iterations. That is why A4 (cross-tree pass cache keyed by pin) and
   A3 (sharding) are schedule items, not optimisations.

### 10.4 Storage and materials

- Prepared data per version: ~366 MB ⓜ. Keep the active era + its predecessor hot (~1 GB); cold
  versions keep `research-lock.json`, diff reports, ledger and profile (~2 MB each, ~200 MB
  total).
- Trees: 11 × ~140 MB ≈ **1.5 GB**, versus ~14 GB for 103 copies ⓜ under the current shape.
- Nothing Mojang-derived is ever committed or uploaded; artefacts are CI-local, as today.

## 11. Risks that invalidate the model, with triggers

| Risk | Trigger to watch | Response |
|---|---|---|
| Behaviour rate below 20 items/port-day | Time one adjacent delta end to end (see below) | Re-scope D3 to rank-1 + player-visible rank-2 only; publish the reduced bar |
| Oracle corpus resists parameterisation (A1 fails) | >80 port-days to get 1.21.11's corpus compiling | Accept per-era oracle forks; add ~150 port-days to the plan |
| Old mappings too gappy to diff (E2–E0) | A community-mapping era produces >20% unmatched classes | Structural-key diffs only (bytecode hash + descriptor), as §4-P1 already allows |
| E0 client-as-oracle doesn't hold headless | 1.0–1.2.5 fail under Xvfb | Logic-only harnesses for E0 + playtest gate; document the fidelity drop |
| Agent throughput is the only throughput | Fleet saturation, `PARITY_MACHINE_JOBS` queuing | Add runners before adding scope |
| Legal framing challenged | Any public claim of "clean room" | Retire the term (§7) |

**The single experiment to run before committing Phases 2–4:** finish one adjacent delta —
`1.21.10` against `1.21.11` — to the D bar, and record: port-days consumed, rank-1/2 items closed,
subset profile, and cold/warm sweep minutes. That one run converts A2 and A4 from assumptions to measurements, and it is the cheapest way to
falsify this document. **In flight** (E6 delta agent; cost-experiment metrics
recorded in `versions/1.21.10/PORT-NOTES-1.21.10.md`).

### 11.1 What that run must record

Eight fields, because each one either confirms or moves a cell of §9 — and all but the first two
land in files the delta gate already reads.

| Field | Feeds | Where it goes |
|---|---|---|
| Wall-clock port-days, template untouched | A2 rate | probe notes, one line |
| Items closed, by rank (rank-1 first, then rank-2) | A2 rate | `pins/<v>/work-items.json` |
| Rank-2 items *not* closed, each with cause + owner | D3 | same file, `accepted[]` |
| Harness subset profile (measured / pass / exempt) | D5, A4 | `.local/parity-runner/state.json` |
| Warm sweep minutes, cold sweep minutes | §10.2 | probe notes |
| Cold-seed cost: probe build, oracle javac, prepare | §10.3 | probe notes, split out |
| `datafix` + `save_roundtrip` verdict | D4 | `state.json` |
| Where the time actually went (top three by hours) | §9.3 | probe notes |

The last one is the one that matters most and the one nobody records: a delta that takes 14
days because it closed 90 rank-2 items tells a different story from one that takes 14 days
because the oracle corpus needed rework. Only the second changes the plan.

## Appendix — provenance of every number used

ⓜ measured in this tree or this repository · ⓔ estimate · ⓤ live external source, not in this tree.

| Figure | Kind | Where it comes from |
|---|---|---|
| 135 / 5,914 files differ, 26.3 → 26.1 | ⓜ measured here | `diff -rq versions/26.3 versions/26.1` |
| 148 files differ, 26.1 → 1.21.11 | ⓜ measured here | `diff -rq versions/26.1 versions/1.21.11` |
| 886 MB repo, 5,913 files/tree, 3,094 `.rs` | ⓜ measured here | `du`, `find` |
| 648 runnable harnesses; 281 is the stale audit count | ⓜ measured here | `ls tools/test_*_parity.py \| wc -l`; `parity.py`'s glob; README's "Last audited 2026-10-13" |
| "0 open of the 833" player-visible gaps at 26.3 | ⓤ live fetch 2026-11-12 | `raw.githubusercontent.com/horizon-rust/horizon/main/README.md`, §Project status; the pinned tree's README snapshot does not contain the sentence |
| 66,700 harness executions in one sweep of 103 releases | ⓔ arithmetic on ⓜ | 648 × 103 |
| 795 / 815 ranked work items | ⓜ repo | `PORT-NOTES-26.1.md`, `PORT-NOTES-1.21.11.md` |
| 2/17, 3/17 pass profiles | ⓜ repo (CI runs 43114684827, 43062079306) | port notes, `VERSIONS.md` |
| ~1.5 harness-min warm; ~15 min cold | ⓜ upstream docs / port notes | `docs/parity-workflow.md` harness-minute stats |
| 300-minute job cap, 17-harness subset | ⓜ repo | `.github/workflows/port-verify.yml` |
| 366 MB prepared data | ⓜ repo | `docs/ROADMAP.md` §4-P6 |
| 20–60 items/port-day; 0.25–1.5 pd/oracle; template splits | ⓔ estimates | this document; falsify via §11 |

---
*Evidence base: Mojang version manifest + per-version JSONs (fetched 2026-11-11);
minecraft.wiki protocol/data-version tables; mappings-license headers from Mojang's
own mapping files; audit of the Horizon tree (pin, protocol crate, prepare/REA
tooling, parity framework, `docs/version-ports.md` incl. the 26.2→26.3 replay);
upstream `ludovic111/horizon` main (pins 26.3; version-port tooling is upstream's).
Reframe evidence (2026-11-12): this repo's own recorded CI profiles, the 648-harness
count from the 26.3 tree, and the live upstream README fetch for the 833 figure.*

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

## Addendum — target reframe adopted (2026-11-12, owner's decision)

- The deliverable is **11 template releases at full parity + 92
  delta-verified releases** (§0, §8), replacing "103 versions at
  behavioural identity". Cost model §9 (~2,510 port-days likely) and
  compute budget §10 (~7,900 CPU-hours) adopted with it.
- Phase 0 re-gated on 1.21.11 reaching T1–T4 (pin, prepare, build,
  oracle corpus compiling) plus a complete measured profile; the
  1.12.2 SRG proof moves to Phase 3. The delta bar is implemented by
  `tools/delta_verify.py` + `docs/DELTA-BAR.md`.
- Corrections applied to the redraft at adoption: the repo-shape
  diagram and §10.4 now say 11 trees (the draft said 9 in two places);
  snapshot scope is recorded as still open, not decided; the "8 of 17
  unmeasured" line is corrected (that was a local run, CI measured all
  17); the flake allowance is marked an estimate, not a measured rule.
- Still open: E0 fidelity (§6.2), snapshot scope (§6.3), streams
  (§6.5), restructure execution timing (§6.4).
