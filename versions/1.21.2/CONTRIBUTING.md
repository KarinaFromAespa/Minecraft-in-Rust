# Contributing

This tree is one version of the **Minecraft-in-Rust** programme (repository
root: [../../README.md](../../README.md)). The one rule that shapes
everything is that **it plays exactly like vanilla**.

## How work happens here

- **Vanilla is the specification.** Behaviour is reimplemented clean-room
  from this version's own jar — never copied from decompiled sources, which
  are a reading aid only. The running jar, exercised through the parity
  harnesses, is the only truth.
- **Parity harnesses are the gate.** A Java oracle and a Rust probe run
  identical inputs and their outputs are diffed. Never delete or weaken a
  harness that pins vanilla behaviour; a version that passes everything
  and diverges somewhere unmeasured is the failure mode this programme
  exists to avoid.
- **Never commit Mojang's files** — game jars, assets, decompiled source,
  prepared resources. They live in `.local/`, which is git-ignored, and are
  regenerated from the player's own copy of the game.
- **Every version documents itself.** The port's state — what was done, the
  real harness profile, what remains blocked and why — is recorded in the
  version directory's port notes and reflected in the root
  [VERSIONS.md](../../VERSIONS.md). Honest partial results are expected;
  polished claims without harness numbers are not.
- **Finished work is committed.** Work lands on the repository's single
  `main` branch, in the version's own `versions/<id>/` directory, under the
  repository owner's identity and without agent co-author lines. Nothing
  finished is left sitting in a working tree.

## Where things are

[AGENTS.md](AGENTS.md) is the codebase map: crates, subsystems, build and
test commands, and the harness layout. The program-wide plan is the root
[docs/ROADMAP.md](../../docs/ROADMAP.md). Where this page or AGENTS.md —
both inherited with the codebase this programme builds on — conflicts with
the programme documents at the repository root, the root documents win.
