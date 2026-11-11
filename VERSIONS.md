# Version index — all 103 major releases

Every major Minecraft Java Edition release, grouped by architecture era (see `docs/ROADMAP.md` §2).
Each version's rewritten Rust code lives on `main` in its own directory, `versions/<id>/`, with its documentation alongside the code.
Status values: `planned` · `in progress` · `parity-verified` · `frozen`.

## E7 — Unobfuscated (26.x) (5)

| Version | Location | Status |
|---|---|---|
| 26.3 | `versions/26.3` | base — imported (upstream snapshot, unmodified) |
| 26.2 | `versions/26.2` | completion pass landed: protocol rewire closed, 6/6 tables, prepare ok, workspace compiles; CI port-verify v5 profile 3 pass / 0 oracle / 14 fail (see `versions/26.2/PORT-NOTES-26.2.md`) |
| 26.1.2 | `versions/26.1.2` | in progress — completion pass + adapter re-base carried forward: protocol rewire closed, 6/6 tables, workspace compiles, protocol tests 437/2; patch delta of 26.1.1 (DataVersion bump, 6 classes incl. handleAttack), see `versions/26.1.2/PORT-NOTES-26.1.2.md` |
| 26.1.1 | `versions/26.1.1` | in progress — completion pass + adapter re-base carried forward: protocol rewire closed, 6/6 tables, workspace compiles, protocol tests 437/2; patch delta of 26.1 (DataVersion bump, 3 classes), see `versions/26.1.1/PORT-NOTES-26.1.1.md` |
| 26.1 | `versions/26.1` | completion pass landed: protocol rewire closed, 6/6 tables, prepare ok, workspace compiles; 1.20.2–1.21.11 adapters re-based to the 775 pin, protocol tests 437 pass / 2 fail (sandbox-network pair only); CI port-verify v5 profile 3 pass / 0 oracle / 14 fail (see `versions/26.1/PORT-NOTES-26.1.md`) |

## E6 — Components (14)

| Version | Location | Status |
|---|---|---|
| 1.21.11 | `versions/1.21.11` | in progress — protocol packet-set rewire closed: nine 26.3-only packets rewired to their 1.21.11 mechanisms from the jar, 6/6 tables regenerated (vanilla_command_order +5 −26); cross-version adapter ladder re-based to the 774 pin (all tables regenerated, 19 adapter + 2 legacy-item failures closed), workspace compiles 0 errors / 0 warnings, protocol tests 438 pass / 2 fail (sandbox-network pair only); oracle bootstrap reworked to 1.21.11's mechanisms, horizon-prepare bundle compiles and runs end-to-end against the named jars, 9 of 17 harnesses measured (all fail, named causes); see `versions/1.21.11/PORT-NOTES-1.21.11.md` |
| 1.21.10 | `versions/1.21.10` | planned |
| 1.21.9 | `versions/1.21.9` | planned |
| 1.21.8 | `versions/1.21.8` | planned |
| 1.21.7 | `versions/1.21.7` | planned |
| 1.21.6 | `versions/1.21.6` | planned |
| 1.21.5 | `versions/1.21.5` | planned |
| 1.21.4 | `versions/1.21.4` | planned |
| 1.21.3 | `versions/1.21.3` | planned |
| 1.21.2 | `versions/1.21.2` | planned |
| 1.21.1 | `versions/1.21.1` | planned |
| 1.21 | `versions/1.21` | planned |
| 1.20.6 | `versions/1.20.6` | planned |
| 1.20.5 | `versions/1.20.5` | planned |

## E5 — Height & config (15)

| Version | Location | Status |
|---|---|---|
| 1.20.4 | `versions/1.20.4` | planned |
| 1.20.3 | `versions/1.20.3` | planned |
| 1.20.2 | `versions/1.20.2` | planned |
| 1.20.1 | `versions/1.20.1` | planned |
| 1.20 | `versions/1.20` | planned |
| 1.19.4 | `versions/1.19.4` | planned |
| 1.19.3 | `versions/1.19.3` | planned |
| 1.19.2 | `versions/1.19.2` | planned |
| 1.19.1 | `versions/1.19.1` | planned |
| 1.19 | `versions/1.19` | planned |
| 1.18.2 | `versions/1.18.2` | planned |
| 1.18.1 | `versions/1.18.1` | planned |
| 1.18 | `versions/1.18` | planned |
| 1.17.1 | `versions/1.17.1` | planned |
| 1.17 | `versions/1.17` | planned |

## E4 — Mojmap plateau (9)

| Version | Location | Status |
|---|---|---|
| 1.16.5 | `versions/1.16.5` | planned |
| 1.16.4 | `versions/1.16.4` | planned |
| 1.16.3 | `versions/1.16.3` | planned |
| 1.16.2 | `versions/1.16.2` | planned |
| 1.16.1 | `versions/1.16.1` | planned |
| 1.16 | `versions/1.16` | planned |
| 1.15.2 | `versions/1.15.2` | planned |
| 1.15.1 | `versions/1.15.1` | planned |
| 1.15 | `versions/1.15` | planned |

## E3 — Flattening (8)

| Version | Location | Status |
|---|---|---|
| 1.14.4 | `versions/1.14.4` | planned |
| 1.14.3 | `versions/1.14.3` | planned |
| 1.14.2 | `versions/1.14.2` | planned |
| 1.14.1 | `versions/1.14.1` | planned |
| 1.14 | `versions/1.14` | planned |
| 1.13.2 | `versions/1.13.2` | planned |
| 1.13.1 | `versions/1.13.1` | planned |
| 1.13 | `versions/1.13` | planned |

## E2 — Late numeric-ID (14)

| Version | Location | Status |
|---|---|---|
| 1.12.2 | `versions/1.12.2` | planned |
| 1.12.1 | `versions/1.12.1` | planned |
| 1.12 | `versions/1.12` | planned |
| 1.11.2 | `versions/1.11.2` | planned |
| 1.11.1 | `versions/1.11.1` | planned |
| 1.11 | `versions/1.11` | planned |
| 1.10.2 | `versions/1.10.2` | planned |
| 1.10.1 | `versions/1.10.1` | planned |
| 1.10 | `versions/1.10` | planned |
| 1.9.4 | `versions/1.9.4` | planned |
| 1.9.3 | `versions/1.9.3` | planned |
| 1.9.2 | `versions/1.9.2` | planned |
| 1.9.1 | `versions/1.9.1` | planned |
| 1.9 | `versions/1.9` | planned |

## E1 — Netty protocol (19)

| Version | Location | Status |
|---|---|---|
| 1.8.9 | `versions/1.8.9` | planned |
| 1.8.8 | `versions/1.8.8` | planned |
| 1.8.7 | `versions/1.8.7` | planned |
| 1.8.6 | `versions/1.8.6` | planned |
| 1.8.5 | `versions/1.8.5` | planned |
| 1.8.4 | `versions/1.8.4` | planned |
| 1.8.3 | `versions/1.8.3` | planned |
| 1.8.2 | `versions/1.8.2` | planned |
| 1.8.1 | `versions/1.8.1` | planned |
| 1.8 | `versions/1.8` | planned |
| 1.7.10 | `versions/1.7.10` | planned |
| 1.7.9 | `versions/1.7.9` | planned |
| 1.7.8 | `versions/1.7.8` | planned |
| 1.7.7 | `versions/1.7.7` | planned |
| 1.7.6 | `versions/1.7.6` | planned |
| 1.7.5 | `versions/1.7.5` | planned |
| 1.7.4 | `versions/1.7.4` | planned |
| 1.7.3 | `versions/1.7.3` | planned |
| 1.7.2 | `versions/1.7.2` | planned |

## E0 — Pre-Netty legacy (19)

| Version | Location | Status |
|---|---|---|
| 1.6.4 | `versions/1.6.4` | planned |
| 1.6.2 | `versions/1.6.2` | planned |
| 1.6.1 | `versions/1.6.1` | planned |
| 1.5.2 | `versions/1.5.2` | planned |
| 1.5.1 | `versions/1.5.1` | planned |
| 1.4.7 | `versions/1.4.7` | planned |
| 1.4.5 | `versions/1.4.5` | planned |
| 1.4.6 | `versions/1.4.6` | planned |
| 1.4.4 | `versions/1.4.4` | planned |
| 1.4.2 | `versions/1.4.2` | planned |
| 1.3.2 | `versions/1.3.2` | planned |
| 1.3.1 | `versions/1.3.1` | planned |
| 1.2.5 | `versions/1.2.5` | planned |
| 1.2.4 | `versions/1.2.4` | planned |
| 1.2.3 | `versions/1.2.3` | planned |
| 1.2.2 | `versions/1.2.2` | planned |
| 1.2.1 | `versions/1.2.1` | planned |
| 1.1 | `versions/1.1` | planned |
| 1.0 | `versions/1.0` | planned |
