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

- **`main`** (this branch) is the programme home: the plan, the version index, and
  the shared pipeline tooling as it is built (mappings front-end first).
- **Each version's rewritten code lives on its own branch: `version/<id>`**
  (e.g. `version/1.21.11`, `version/1.8.9`). All branches share history, so the repo
  stays one organized tree instead of 103 copies of a codebase.
- **[VERSIONS.md](VERSIONS.md)** is the index: all 103 releases grouped by
  architecture era, each with its branch and status.
- Era lines are the maintained units; a finished era eventually **freezes** —
  a parity-verified old release stays correct forever.

## Rules

- **Nothing Mojang-derived is ever committed.** Jars, mappings and decompiled
  sources are development-time reference only; they live in `.local/`, which is
  git-ignored. The product is only the clean-room Rust code.
- Players prepare each version's data from **their own** Minecraft copy
  (`horizon-prepare`), exactly as the 26.3 rewrite works today.
- Every version branch pins itself with a `research-lock.json` (version, jar
  hashes, Java major) — the version a build targets is never a guess.

## Status

Founding state. Phase 0 (prove the loop on 26.1, build the mappings front-end,
prove it on 1.21.11) has not started yet; see the roadmap §5 for sequencing
and §6 for the remaining owner decisions.
