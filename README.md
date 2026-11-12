# Minecraft in Rust

Every major Minecraft Java Edition release — **1.0 → 26.3, 103 releases** — rewritten
in Rust for desktop. Each version's behaviour is a reference reimplementation from
decompiled jars, verified against its own jar with parity harnesses, the same method
that produced the 26.3 rewrite.

## Base

The foundation is the upstream desktop **horizon-rust** project, whose version-port
pipeline (`tools/version_port.py`, design in its `docs/version-ports.md`) was built to
turn each new Minecraft release into Rust. This programme runs that pipeline
*backward* through history and keeps every version as a finished line.

The full plan — era decomposition, the per-version pipeline, the hard problems and
their workarounds, sequencing and gates — is in **[docs/ROADMAP.md](docs/ROADMAP.md)**.

## Organization

- **`main` is the only branch.** It holds the plan, the version index, the shared
  pipeline tooling, and every version's rewritten code.
- **Each version lives in its own directory: `versions/<id>/`**
  (e.g. `versions/1.21.11`, `versions/1.8.9`), with its port notes and parity
  status documented alongside the code.
- **[VERSIONS.md](VERSIONS.md)** is the index: all 103 releases grouped by
  architecture era, each with its location and status.
- Era lines are the maintained units; a finished era eventually **freezes** —
  a parity-verified old release stays correct forever.

## Rules

- **Nothing Mojang-derived is ever committed.** Jars, mappings and decompiled
  sources are development-time reference only; they live in `.local/`, which is
  git-ignored. The product is only the reimplemented Rust code.
- Players prepare each version's data from **their own** Minecraft copy
  (`horizon-prepare`), exactly as the 26.3 rewrite works today.
- Every version directory pins itself with a `research-lock.json` (version, jar
  hashes, Java major) — the version a build targets is never a guess.

## Status

Phase 0's **tooling milestone** is met: the mappings front-end is wired into the
pipeline's prepare paths, and obfuscated releases now prepare end-to-end through
it. The **gate itself is still open** — as written in the roadmap §5 it requires
a 1.21.11 parity profile ≈ its baseline (baseline: the 26.3 control profile,
not yet run) and the front-end remapping a gappy SRG jar (1.12.2) without
manual rescue; neither condition is done yet. Each version directory carries a
machine-readable `status.json` (harness counts, behaviour items, last CI run);
the scalars in `VERSIONS.md` come from it.
`versions/26.3/` holds the base rewrite; `versions/26.1/` and
`versions/26.2/` have both completed their completion passes (protocol
rewires closed, all six tables, prepare green, workspaces compile), and the
completion has been carried forward into the `versions/26.1.1/` patch
delta; `versions/26.1.2/` is the second delta. Full harness profiles are
produced per tree by the CI route below.
`versions/1.21.11/` opens the Components era (E6): the oracle bootstrap
rework is done, all six tables regenerated, the protocol packet-set
rewire closed (nine 26.3-only packets rewired to their 1.21.11
mechanisms from the jar), horizon-prepare runs end to end against the
named jars; the behaviour port remains, documented in its port notes. The adapter
ladder re-base landed (protocol tests 438 pass / 2 fail). Verification runs for published trees are dispatched to GitHub Actions
(`port-verify` workflow). See `VERSIONS.md` for the per-version state, the
roadmap §5 for sequencing and §6 for the remaining owner decisions.
