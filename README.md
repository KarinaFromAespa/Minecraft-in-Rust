# Minecraft in Rust

Every major Minecraft Java Edition release — **1.0 → 26.3, 103 releases** — rewritten
in Rust for desktop. Clean-room: each version's behaviour is reimplemented from its own
spec and verified against its own jar with parity harnesses, the same method that
produced the 26.3 rewrite.

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
  git-ignored. The product is only the clean-room Rust code.
- Players prepare each version's data from **their own** Minecraft copy
  (`horizon-prepare`), exactly as the 26.3 rewrite works today.
- Every version directory pins itself with a `research-lock.json` (version, jar
  hashes, Java major) — the version a build targets is never a guess.

## Status

Phase 0's gate is met: the mappings front-end is wired into the pipeline's
prepare paths, and obfuscated releases now prepare end-to-end through it.
`versions/26.3/` holds the base rewrite; `versions/26.1/` has completed its
completion pass (protocol rewire closed, all six tables, prepare green) and
`versions/26.1.1/` is its first patch delta; `versions/26.2/` is published
with a partial profile and its completion pass is running.
`versions/1.21.11/` opens the Components era (E6): prepare proven through
the front-end, pin set, 3 of 6 tables regenerated; the remaining tables wait
on a registry-bootstrap rework documented in its port notes. Verification
runs for published trees are dispatched to GitHub Actions
(`port-verify` workflow). See `VERSIONS.md` for the per-version state, the
roadmap §5 for sequencing and §6 for the remaining owner decisions.
