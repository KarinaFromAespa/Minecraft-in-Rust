# Minecraft in Rust — Java Edition 26.1.1

**Minecraft: Java Edition 26.1.1, rewritten in Rust.** This directory is one
version of the [Minecraft-in-Rust](../../README.md) programme: every major
Java Edition release, 1.0 → 26.3, rewritten in Rust for desktop — clean-room,
each version specified and verified against its own jar.

26.1.1 is a patch release on the 26.1 line. It was delta-ported from the
programme's 26.1 rewrite ([versions/26.1](../26.1/)) through the
version-port pipeline: the delta is a DataVersion bump and three changed
classes, and the tree otherwise stands as 26.1's port did. **The port is in
progress, with a partial parity profile** —
[PORT-NOTES-26.1.1.md](PORT-NOTES-26.1.1.md) in this directory is the
authoritative record of the delta, what the pipeline regenerated, and what
remains blocked. Its status is also tracked in
[VERSIONS.md](../../VERSIONS.md).

## Base and credit

This codebase derives from the **horizon-rust** project, whose version-port
pipeline (`tools/version_port.py`, design in `docs/version-ports.md`) and
parity-harness method this programme builds on. The code is used here as a
foundation under its MIT licence. This directory is part of an independent
programme; it is not the Horizon project, and its releases, community and
website are separate. Upstream: <https://github.com/ludovic111/horizon>.

## How this version is held to account

- **Vanilla is the specification.** Behaviour is reimplemented from the
  version's own jar and checked against it with parity harnesses: a Java
  oracle and a Rust probe run identical inputs and their outputs are diffed.
- **Nothing Mojang-derived is committed.** Jars, mappings and decompiled
  sources are development-time reference only. Players prepare this
  version's data from their own copy of the game (`horizon-prepare`).
- The version pins itself with `research-lock.json` in this directory
  (version, jar hashes, Java major).

## Working in this tree

- The codebase map for agents is [AGENTS.md](AGENTS.md); the working method
  is [CONTRIBUTING.md](CONTRIBUTING.md); design and architecture notes are
  under `docs/`.
- Build from source with the Cargo workspace in this directory
  (`Cargo.toml`); the tree's own docs describe the pipeline and the
  per-version porting flow.

## Licence

MIT — see [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
Unofficial: not affiliated with Mojang or Microsoft. You play it with your
own copy of Java Edition.
