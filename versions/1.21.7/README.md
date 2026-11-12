# Minecraft in Rust — Java Edition 1.21.7

**Minecraft: Java Edition 1.21.7, rewritten in Rust.** This directory is one
version of the [Minecraft-in-Rust](../../README.md) programme: every major
Java Edition release, 1.0 → 26.3, rewritten in Rust for desktop — a reference
reimplementation, each version specified and verified against its own jar.

1.21.7 is the fourth adjacent delta port in the programme's **Components era
(E6, 1.20.5–1.21.11)**, taken from 1.21.8: a same-protocol hotfix delta
(protocol 772 on both sides; world version 4440 → 4438), so the adapter
ladder and packet set are unchanged — the measured delta is just 5 ranked
work items: the version pin, the save-format step (0 schemas, 0 fixes, 0
datafix classes changed), and one changed class (`SharedConstants`, 0
methods). Two of the six generated tables regenerate against 1.21.7's own
jar (version constants, and packet ids with a header-only change); the
other four reproduce byte-for-byte. The remaining work is sized, ranked
and documented.
[PORT-NOTES-1.21.7.md](PORT-NOTES-1.21.7.md) in this directory is the
authoritative record of the stage reached, the numbers behind it, and the
residual. Its status is also tracked in
[VERSIONS.md](../../VERSIONS.md).

## Base and credit

This codebase derives from the **horizon-rust** project, whose version-port
pipeline (`tools/version_port.py`, design in `docs/version-ports.md`) and
parity-harness method this programme builds on. The code is used here as a
foundation under its MIT licence. This directory is part of an independent
programme; it is not the Horizon project, and its releases, community and
website are separate. Upstream: <https://github.com/ludovic111/horizon>.

The mappings front-end (`tools/mapping-frontend/` at the repository root) is
this programme's own work: it deobfuscates a release's jars with Mojang's
published mapping tables so the pipeline can specify an obfuscated version
from its own jar, exactly as it does an unobfuscated one.

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
