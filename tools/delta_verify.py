#!/usr/bin/env python3
"""delta_verify.py — decide whether one Minecraft release is delta-verified.

Implements the D1–D6 bar from DELTA-BAR.md as a single gate. A release is
delta-verified only if every check PASSES. Missing evidence is BLOCKED, and
blocked is never verified: the exit status distinguishes the two.

    python3 tools/delta_verify.py 1.21.10                 # judge recorded evidence
    python3 tools/delta_verify.py 1.21.10 --run           # run the build/tests/harnesses
    python3 tools/delta_verify.py 1.21.10 --run --download
    python3 tools/delta_verify.py 1.21.10 --json           # verdict to stdout
    python3 tools/delta_verify.py 1.21.10 --check D3       # one check, for a loop

Layout it expects (see DELTA-BAR.md §2):

    <tree>/                       lines/<release> or versions/<release>
      research-lock.json
      crates/horizon-core/src/version.rs
      tools/{version_port.py,parity.py,port_ledger.py}
      .local/horizon-client/<release>/        prepared runtime data
      .local/parity-runner/state.json         parity.py's own record

    <pin>/                        pins/<release>, or the tree itself if no pins/
      work-items.json             D3 ledger, hashes the report it was built from
      harness-subset.txt          D5: this era's regression subset
      allowed-failures.txt        any check: "name  # cause: … owner: …"
      playtest.md                 D6 signed playtest
      regen-determinism.json      D1: two regen runs' table hashes
      delta-verdict.json          written by this script

Exit: 0 verified · 1 failed · 3 blocked (evidence missing) · 2 usage error.
Stdlib only; no imports from the tree, so it runs from CI or a scratch checkout.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

SCHEMA = "delta-verdict/1"


class ConfigurationError(Exception):
    """The pin's own metadata is malformed — a config error, not a verdict."""
PASS, FAIL, BLOCKED = "pass", "fail", "blocked"
STATUS_EXIT = {PASS: 0, FAIL: 1, BLOCKED: 3}

PROTOCOL_PACKAGE = "horizon-protocol"
RUST_TABLES = (
    "crates/horizon-core/src/version.rs",
    "crates/horizon-protocol/src/ids.rs",
    "crates/horizon-client/src/lan/tracking.rs",
    "crates/horizon-client/src/lan/command_order.rs",
    "crates/horizon-client/src/lan/command_suggestions.rs",
    "crates/horizon-command/src/vanilla_order.rs",
)
# The six generators in versions/*/tools/port_regen.py GENERATORS, verified
# identical in 26.1, 26.2, 1.21.9, 1.21.10 and 1.21.11. A `regen --only`
# run records only the generators it ran, so D1 must check completeness
# explicitly instead of trusting the file's length.
EXPECTED_GENERATORS = (
    "version_consts",
    "packet_ids",
    "entity_tracking",
    "lan_command_order",
    "command_suggestions",
    "vanilla_command_order",
)
DATA_CHECKS = ("datafix", "save_roundtrip")  # D4
D3_RANKS = ("1", "2")
PLAYTEST_BOXES = 6


# ---------------------------------------------------------------- plumbing


def read_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return None
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"delta_verify: cannot read {path}: {error}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def sh(argv, cwd: Path, timeout: int = 3600, env: dict | None = None):
    merged = dict(os.environ)
    merged.setdefault("CARGO_BUILD_JOBS", "2")
    merged.update(env or {})
    try:
        run = subprocess.run(
            argv, cwd=cwd, env=merged, timeout=timeout,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        )
        return run.returncode, run.stdout
    except FileNotFoundError:
        return 127, f"{argv[0]}: not found"
    except subprocess.TimeoutExpired as expired:
        text = (expired.stdout or "") if isinstance(expired.stdout, str) else ""
        return 124, text + f"\n[delta_verify] timed out after {timeout}s"


def first_line(text: str) -> str:
    for line in text.splitlines():
        if line.strip():
            return line.strip()[:240]
    return ""


def allowed(pin: Path) -> dict[str, str]:
    """allowed-failures.txt: `name  # cause: … owner: …` → name to the line."""
    table, path = {}, pin / "allowed-failures.txt"
    if not path.exists():
        return table
    for number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        name = line.split("#", 1)[0].strip()
        comment = line.split("#", 1)[1] if "#" in line else ""
        if "cause:" not in comment or "owner:" not in comment:
            raise ConfigurationError(
                f"{path}:{number}: '{name}' is exempted without "
                "'cause: …' and 'owner: …' — an accepted gap needs both"
            )
        table[name] = comment.strip()
    return table


def check_ok(notes, evidence=None, metrics=None):
    result = {"status": PASS, "notes": list(notes), "evidence": list(evidence or [])}
    if metrics:
        result["metrics"] = metrics
    return result


def check_bad(note, evidence=None):
    return {"status": FAIL, "notes": [note], "evidence": list(evidence or [])}


def check_stop(note, evidence=None):
    return {"status": BLOCKED, "notes": [note], "evidence": list(evidence or [])}


# ---------------------------------------------------------------- discovery


def resolve_tree(release: str, given: str | None) -> Path:
    if given:
        return Path(given).resolve()
    here = Path.cwd()
    for candidate in (here / "lines" / release, here / "versions" / release, here):
        if (candidate / "research-lock.json").exists() or (
            candidate / "crates" / "horizon-core" / "src" / "version.rs"
        ).exists():
            return candidate.resolve()
    raise SystemExit(
        f"delta_verify: no tree for {release}; pass --tree lines/{release} "
        f"or versions/{release}"
    )


def resolve_pin(release: str, given: str | None, tree: Path) -> Path:
    """Where this release's metadata lives.

    Two layouts are supported, and the second is the recommended one: pin
    metadata beside the version (`versions/<id>/`) rather than in a second
    per-version tree, or `lines/<era>/pins/<release>/` when code and pins split.
    """
    if given:
        return Path(given).resolve()
    repo = tree.parent.parent
    for candidate in (
        repo / "pins" / release,
        repo / "versions" / release,
        tree / "pins" / release,
        tree,
    ):
        if candidate.is_dir() and any(candidate.iterdir()):
            return candidate.resolve()
    return tree


def find_report(tree: Path, release: str) -> Path | None:
    """version_port.py's ranked report for the diff that produced this pin."""
    roots = [tree / ".local", tree, tree.parent / ".local", tree.parent]
    override = os.environ.get("HORIZON_PORTS")
    if override:
        roots.insert(0, Path(override))
    for root in roots:
        if not root.is_dir():
            continue
        for path in sorted(root.rglob(f"diff-*-{release}/report.json")):
            return path
    return None


def find_sibling(tree: Path, release: str, name: str) -> Path | None:
    report = find_report(tree, release)
    if report and (report.with_name(name)).exists():
        return report.with_name(name)
    roots = [tree / ".local", tree, tree.parent / ".local", tree.parent]
    for root in roots:
        if not root.is_dir():
            continue
        hits = sorted(root.rglob(f"diff-*-{release}/{name}"))
        if hits:
            return hits[0]
    return None


# ---------------------------------------------------------------- the checks


def check_d1(tree: Path, pin: Path, release: str, run: bool, download: bool):
    """D1 — pin regenerated from this release's own jar, prepare green, determinism."""
    evidence, notes = [], []
    lock_path = tree / "research-lock.json"
    lock = read_json(lock_path)
    if not lock:
        return check_stop("no research-lock.json", [str(lock_path)])
    pinned = lock.get("minecraft", {}).get("version")
    if pinned != release:
        return check_bad(f"research-lock pins {pinned!r}, not {release!r}", [str(lock_path)])
    for key in ("client_sha1", "server_sha1", "java_major"):
        if key not in lock.get("minecraft", {}):
            return check_bad(f"research-lock.minecraft.{key} missing", [str(lock_path)])
    notes.append(f"lock pins {release} (java {lock['minecraft']['java_major']})")
    evidence.append(str(lock_path))

    version_rs = tree / "crates/horizon-core/src/version.rs"
    if not version_rs.exists():
        return check_stop("no crates/horizon-core/src/version.rs", [str(version_rs)])
    text = version_rs.read_text(encoding="utf-8", errors="replace")
    found = {
        "id": re.search(r'VERSION_ID:\s*&str\s*=\s*"([^"]+)"', text),
        "protocol": re.search(r"PROTOCOL_VERSION:\s*i32\s*=\s*(\d+)", text),
        "data": re.search(r"DATA_VERSION:\s*i32\s*=\s*(\d+)", text),
    }
    if not all(found.values()):
        return check_bad("version.rs is missing VERSION_ID/PROTOCOL_VERSION/DATA_VERSION",
                         [str(version_rs)])
    if found["id"].group(1) != release:
        return check_bad(f"version.rs VERSION_ID is {found['id'].group(1)!r}", [str(version_rs)])
    if int(found["protocol"].group(1)) <= 0 or int(found["data"].group(1)) <= 0:
        return check_bad("version.rs has an unregenerated protocol/data version", [str(version_rs)])
    notes.append(
        f"version.rs: protocol {found['protocol'].group(1)}, data {found['data'].group(1)}"
    )
    evidence.append(str(version_rs))

    regen_path = find_sibling(tree, release, "regen.json")
    regen = read_json(regen_path) if regen_path else None
    if regen is None:
        return check_stop("no regen.json: run `version_port.py regen` and keep its output",
                          evidence)
    bad = [
        f"{name}: {entry.get('what') or entry.get('error') or 'failed'}"
        for name, entry in (regen.get("generators") or {}).items()
        if str(entry.get("status", entry.get("ok", ""))).lower() in ("fail", "failed", "error")
        or entry.get("what") or entry.get("error")
    ]
    if bad:
        return check_bad("generators failed: " + "; ".join(bad[:4]), evidence + [str(regen_path)])
    generated = regen.get("generators") or {}
    missing = sorted(set(EXPECTED_GENERATORS) - set(generated))
    if missing:
        return check_stop(
            "regen.json is a partial run: missing " + ", ".join(missing)
            + "; run a full `version_port.py regen` (no --only) and keep its output",
            evidence + [str(regen_path)],
        )
    notes.append(f"regen.json: all {len(generated)} generators clean")
    evidence.append(str(regen_path))

    determinism_path = pin / "regen-determinism.json"
    determinism = read_json(determinism_path)
    if determinism is None:
        return check_stop(
            "no regen-determinism.json: hash the COMMITTED tables as `committed`, do one fresh "
            "`version_port.py regen <v> --worktree A` as `fresh`, and compare",
            evidence,
        )
    # `committed` is the tree as shipped; `fresh` is one regeneration. Comparing two
    # fresh runs to each other passes two identically wrong regens, which is exactly
    # how a hand-corrected table survives unnoticed.
    committed = determinism.get("committed") or determinism.get("run1") or {}
    fresh = determinism.get("fresh") or determinism.get("run2") or {}
    if not committed or not fresh:
        return check_stop("regen-determinism.json needs `committed` and `fresh` table hashes",
                          [str(determinism_path)])
    if set(committed) != set(fresh):
        return check_bad("regen-determinism.json: the two runs cover different tables",
                         [str(determinism_path)])
    missing = [t for t in RUST_TABLES if t not in committed]
    if missing:
        return check_stop("regen-determinism.json omits tables: " + ", ".join(missing[:3]),
                          [str(determinism_path)])
    drift = sorted(t for t in committed if committed[t] != fresh[t])
    overrides = {
        entry.get("table"): entry
        for entry in (determinism.get("overrides") or [])
        if isinstance(entry, dict) and entry.get("table")
    }
    unjustified = [t for t, entry in overrides.items()
                   if not entry.get("reason") or not entry.get("owner")]
    if unjustified:
        return check_bad("declared table override without cause or owner: "
                         + ", ".join(unjustified[:3]), [str(determinism_path)])
    undeclared = [t for t in drift if t not in overrides]
    if undeclared:
        return check_bad(
            f"a fresh regen differs from the committed tables at {len(undeclared)} table(s) with "
            "no declared override: " + ", ".join(undeclared[:3]),
            [str(determinism_path)],
        )
    if overrides:
        notes.append(
            f"{len(committed) - len(overrides)} tables reproduce exactly; "
            f"{len(overrides)} declared hand override(s): "
            + ", ".join(sorted(overrides)[:3])
        )
    else:
        notes.append(f"all {len(committed)} tables reproduce exactly")
    stale = [t for t in overrides if t not in drift]
    if stale:
        notes.append(f"declared override no longer differs from the generator: {len(stale)}")
    evidence.append(str(determinism_path))

    prepared = tree / ".local" / "horizon-client" / release
    if run:
        argv = [sys.executable, "tools/version_port.py", "prepare", release]
        if download:
            argv.append("--download")
        code, output = sh(argv, tree, timeout=7200)
        evidence.append(f"version_port.py prepare → exit {code}")
        if code != 0:
            return check_bad(f"prepare failed (exit {code}): {first_line(output)}", evidence)
        notes.append("prepare ran end to end (exit 0)")
    elif prepared.is_dir():
        files = sum(1 for _ in prepared.rglob("*") if _.is_file())
        if files < 50:
            return check_bad(f"prepared data at {prepared} holds only {files} files", [str(prepared)])
        notes.append(f"prepared data present ({files} files)")
        evidence.append(str(prepared))
    else:
        return check_stop(f"no prepared data at {prepared}; rerun with --run", evidence)

    return check_ok(notes, evidence)


def _cargo(tree: Path, argv, log_names, run: bool, pin: Path):
    """Run the command, or read the first of several accepted log names."""
    if run:
        code, output = sh(argv, tree, timeout=7200)
        return code, output, None
    for name in log_names:
        log = pin / "ci-out" / name
        if log.exists():
            return 0, log.read_text(encoding="utf-8", errors="replace"), log
    return None, None, None


def check_d2(tree: Path, pin: Path, release: str, run: bool, timeout: int):
    """D2 — workspace compiles clean, protocol tests green except the sandbox pair."""
    evidence, notes = [], []
    exempt = allowed(pin)

    code, output, log = _cargo(
        tree, ["cargo", "check", "--workspace", "--all-targets"],
        ("cargo-check.log", "workspace-check.log"), run, pin,
    )
    if output is None:
        narrow = pin / "ci-out" / "protocol-check.log"
        if narrow.exists():
            return check_stop(
                "only ci-out/protocol-check.log is recorded: that compiles horizon-protocol alone "
                "and cannot show the workspace builds — record a `cargo check --workspace "
                "--all-targets` step", [str(narrow)],
            )
        return check_stop("no ci-out/cargo-check.log; rerun with --run", evidence)
    if log:
        evidence.append(str(log))
    errors = len(re.findall(r"^error(\[E\d+\])?:", output, re.M))
    warnings = len(re.findall(r"^warning:", output, re.M))
    if code != 0 or errors:
        return check_bad(f"cargo check: {errors} error(s), exit {code}", evidence)
    if warnings:
        return check_bad(f"cargo check: {warnings} warning(s) (bar is 0/0)", evidence)
    notes.append("cargo check --workspace --all-targets: 0 errors, 0 warnings")

    code, output, log = _cargo(
        tree, ["cargo", "test", "-p", PROTOCOL_PACKAGE],
        ("cargo-test-protocol.log", "protocol-test.log", "cargo-test.log"), run, pin,
    )
    if output is None:
        return check_stop(
            "no protocol-test log (cargo-test-protocol.log / protocol-test.log); the workflow has "
            "no test step yet — add one, or rerun with --run", evidence,
        )
    if log:
        evidence.append(str(log))
    blocks = re.findall(r"^failures:\n((?:\s{4}\S+\n)+)", output, re.M)
    failures = sorted({name for block in blocks for name in block.split()})
    passed = re.search(r"test result: ok\. (\d+) passed", output)
    if code != 0 and not failures:
        return check_bad(f"cargo test -p {PROTOCOL_PACKAGE} failed (exit {code})", evidence)
    unexplained = [f for f in failures if not any(token in f for token in exempt)]
    if unexplained:
        return check_bad(
            f"{len(unexplained)} protocol test(s) failing without an accepted gap: "
            + ", ".join(unexplained[:3]),
            evidence,
        )
    notes.append(
        f"protocol tests: {passed.group(1) if passed else 'ran'}"
        + (f", {len(failures)} allowed" if failures else ", none failing")
    )
    return check_ok(notes, evidence)


def check_d3(tree: Path, pin: Path, release: str):
    """D3 — rank-1 and rank-2 work items closed or accepted with a cause and owner."""
    report_path = find_report(tree, release)
    report = read_json(report_path) if report_path else None
    if report is None:
        return check_stop("no diff-<prev>-<release>/report.json: run `version_port.py report`")
    if "ranks" not in report and "items" not in report:
        return check_bad("report.json holds no ranks or items — not a port report", [str(report_path)])
    # version_port.py's report step nests the rank totals under
    # summary.ranks and lists the items at top level; no real report
    # carries a top-level "ranks" key. Read top-level ranks first
    # (back-compat), then summary.ranks (the tool's own totals), then
    # derive the totals from the items list. Reading only the top
    # level made every rank total 0 and failed D3 on any real pin.
    ranks = {str(k): int(v) for k, v in (report.get("ranks") or {}).items()}
    if not ranks:
        summary = report.get("summary")
        if isinstance(summary, dict):
            ranks = {str(k): int(v) for k, v in (summary.get("ranks") or {}).items()}
    if not ranks and isinstance(report.get("items"), list):
        from collections import Counter
        ranks = {str(k): v for k, v in Counter(
            str(i.get("rank")) for i in report["items"]
            if isinstance(i, dict) and i.get("rank") is not None).items()}
    ledger_path = pin / "work-items.json"
    ledger = read_json(ledger_path)
    if ledger is None:
        return check_stop("no work-items.json: record closure per rank in the pin", [str(report_path)])
    if ledger.get("release") != release:
        return check_bad(f"work-items.json is for {ledger.get('release')!r}", [str(ledger_path)])
    recorded = ledger.get("generated_from")
    if recorded and recorded != sha256(report_path):
        return check_bad("work-items.json was built from a different report.json", [str(ledger_path)])
    notes, evidence = [], [str(report_path), str(ledger_path)]
    for rank in D3_RANKS:
        total = ranks.get(rank, 0)
        entry = ledger.get(f"rank{rank}") or {}
        if int(entry.get("total", -1)) != total:
            return check_bad(
                f"rank-{rank}: ledger says {entry.get('total')!r} items, report has {total}",
                evidence,
            )
        accepted = entry.get("accepted") or []
        for gap in accepted:
            if not gap.get("cause") or not gap.get("owner"):
                return check_bad(f"rank-{rank}: an accepted gap lacks cause or owner", evidence)
        closed = int(entry.get("closed", 0)) + len(accepted)
        if closed != total:
            return check_bad(
                f"rank-{rank}: {total - closed} of {total} item(s) neither closed nor accepted",
                evidence,
            )
        notes.append(f"rank-{rank}: {entry.get('closed', 0)} closed, {len(accepted)} accepted, {total} total")
    if ranks.get("1", 0) == 0:
        notes.append("report has no rank-1 items (mechanically identical release)")
    inherited = ledger.get("rank3plus")
    if not inherited:
        return check_stop("work-items.json must say what rank-3+ items inherit (e.g. the template)")
    notes.append(f"rank-3+: {inherited}")
    closed = sum(int((ledger.get(f"rank{r}") or {}).get("closed", 0)) for r in D3_RANKS)
    total = sum(ranks.get(r, 0) for r in D3_RANKS)
    accepted = sum(len((ledger.get(f"rank{r}") or {}).get("accepted") or []) for r in D3_RANKS)
    return check_ok(notes, evidence, {"items_closed": closed, "items_total": total,
                                      "items_accepted": accepted})


ALIASES = {"pass": "pass", "passed": "pass", "ok": "pass", "green": "pass",
           "fail": "fail", "failed": "fail", "red": "fail"}
PASS_KEYS = ("passing", "passed", "pass")
FAIL_KEYS = ("failing", "failed", "fail")


def _records(container) -> dict:
    """name → record from a record-style container ({} for a summary container)."""
    if isinstance(container, list):
        if not all(isinstance(entry, dict) for entry in container):
            return {}
        container = {(e.get("name") or e.get("harness")): e for e in container}
    if not isinstance(container, dict):
        return {}
    if container and all(isinstance(value, list) for value in container.values()):
        return {}  # a summary: passing/failing lists, not records
    out = {}
    for name, value in container.items():
        if not isinstance(name, str) or not name:
            continue
        if isinstance(value, str):
            out[name] = {"status": ALIASES.get(value.lower(), value.lower())}
        elif isinstance(value, dict):
            status = value.get("status") or value.get("verdict") or value.get("result")
            if status is None and "passed" in value:
                status = "pass" if value["passed"] else "fail"
            if status is None:
                continue
            out[name] = {**value,
                         "status": ALIASES.get(str(status).lower(), str(status).lower())}
    return out


def _buckets(container) -> dict:
    """name → record from passing/failing lists."""
    if not isinstance(container, dict):
        return {}
    out = {}
    for status, keys in (("pass", PASS_KEYS), ("fail", FAIL_KEYS)):
        for key in keys:
            names = container.get(key)
            if isinstance(names, list):
                for name in names:
                    if isinstance(name, str) and name:
                        out.setdefault(name, {"status": status})
                break
    return out


def normalise_profile(raw):
    """Any published profile shape → {"harnesses": {...}}, records beating summaries.

    A profile may carry both a summary (`passing`/`failing` lists) and a map of
    harness records. The map wins where they disagree and the summary fills gaps,
    so a harness whose status is neither pass nor fail — an oracle that no longer
    compiles, a flaky rerun — keeps that status instead of vanishing into
    "unmeasured". Reading the summary first would report the wrong reason.
    """
    if not isinstance(raw, dict):
        return None
    found = dict(_buckets(raw))
    for key in ("harnesses", "harness", "suite", "summary"):
        value = raw.get(key)
        if isinstance(value, dict):
            for name, record in _buckets(value).items():
                found.setdefault(name, record)
    for key in ("harnesses", "harness", "suite", "results", "tests"):
        value = raw.get(key)
        if isinstance(value, (dict, list)):
            found.update(_records(value))
    return {"harnesses": found} if found else None


def commit_agrees(stamped: str, actual: str) -> bool:
    if not stamped or not actual:
        return True
    short = min(len(stamped), len(actual), 40)
    if short < 7:
        return True
    return stamped[:short][:7] == actual[:7] or actual.startswith(stamped) or stamped.startswith(actual)


def _parity_state(tree: Path, pin: Path, commit: str = ""):
    """Published profile first (the frozen record), then the local run state.

    Returns (state, path, reason). A profile stamped with another commit is not
    evidence about this tree, so it is refused rather than downgraded.
    """
    profile = pin / "profile.json"
    if not profile.exists():
        profile = pin / "ci-out" / "profile.json"
    if profile.exists():
        raw = read_json(profile)
        stamped = (raw or {}).get("commit") or ""
        if not commit_agrees(stamped, commit):
            return None, profile, (
                f"{profile.name} is stamped for commit {stamped[:12]}, HEAD is {commit[:12]} "
                "— re-record the sweep for this tree"
            )
        state = normalise_profile(raw or {})
        if state:
            return state, profile, ""
    for candidate in (tree / ".local/parity-runner/state.json", pin / "ci-out/state.json"):
        raw = read_json(candidate)
        if raw:
            state = normalise_profile(raw)
            if state:
                return state, candidate, ""
    return None, None, ""


def _judge_harnesses(state, names, exempt):
    harnesses = state.get("harnesses") or {}
    failing, unknown = [], []
    for name in names:
        record = harnesses.get(name)
        if record is None:
            unknown.append(name)
        elif record.get("status") != "pass" and not any(token in name for token in exempt):
            status = record.get("status")
            failing.append(name if status == "fail" else f"{name}({status})")
    return failing, unknown


def check_d4(tree: Path, pin: Path, release: str, run: bool, timeout: int, commit: str = ""):
    """D4 — data compatibility: datafix ladder and save round-trip."""
    exempt = allowed(pin)
    if run:
        code, output = sh(
            [sys.executable, "tools/parity.py", *DATA_CHECKS, "--timeout", str(timeout)],
            tree, timeout=timeout * len(DATA_CHECKS) + 300,
        )
        if code != 0:
            return check_bad(f"parity.py {DATA_CHECKS} exited {code}: {first_line(output)}")
    state, path, reason = _parity_state(tree, pin, commit)
    if state is None:
        return check_stop(reason or "no profile.json or .local/parity-runner/state.json; "
                                  "rerun with --run", [str(path)] if path else [])
    failing, unknown = _judge_harnesses(state, DATA_CHECKS, exempt)
    if unknown:
        return check_stop(f"not measured: {', '.join(unknown)}", [str(path)])
    if failing:
        return check_bad(f"data compatibility failing: {', '.join(failing)}", [str(path)])
    return check_ok([f"{', '.join(DATA_CHECKS)}: pass"], [str(path)])


def check_d5(tree: Path, pin: Path, release: str, run: bool, timeout: int, commit: str = ""):
    """D5 — this era's regression subset is green (or exempted with cause and owner)."""
    subset_path = pin / "harness-subset.txt"
    if not subset_path.exists():
        return check_stop("no harness-subset.txt: the era's regression subset is undefined",
                          [str(subset_path)])
    names = [
        line.split("#", 1)[0].strip()
        for line in subset_path.read_text(encoding="utf-8").splitlines()
        if line.split("#", 1)[0].strip()
    ]
    if len(names) < 5:
        return check_bad(f"harness-subset.txt names only {len(names)} harnesses", [str(subset_path)])
    exempt = allowed(pin)
    if run:
        code, output = sh(
            [sys.executable, "tools/parity.py", *names, "--timeout", str(timeout)],
            tree, timeout=timeout * min(len(names), 8) + 900,
        )
        if code != 0 and not (tree / ".local/parity-runner/state.json").exists():
            return check_bad(f"parity.py exited {code}: {first_line(output)}")
    state, path, reason = _parity_state(tree, pin, commit)
    if state is None:
        return check_stop(reason or "no profile.json or .local/parity-runner/state.json; "
                                  "rerun with --run", [str(path)] if path else [])
    failing, unknown = _judge_harnesses(state, names, exempt)
    if unknown:
        return check_stop(f"{len(unknown)} of {len(names)} subset harnesses unmeasured: "
                          + ", ".join(unknown[:4]), [str(path)])
    if failing:
        return check_bad(f"{len(failing)} of {len(names)} subset harnesses failing: "
                         + ", ".join(failing[:4]), [str(path)])
    exempted = sum(1 for name in names if any(token in name for token in exempt))
    return check_ok([f"{len(names)}/{len(names)} subset harnesses pass"],
                    [str(path), str(subset_path)],
                    {"suite_pass": len(names) - exempted, "suite_total": len(names),
                     "suite_exempt": exempted})


def check_d6(pin: Path, release: str, commit: str):
    """D6 — signed playtest: world loads, a vanilla client of this version joins."""
    path = pin / "playtest.md"
    if not path.exists():
        return check_stop("no playtest.md: the play gate has not been recorded", [str(path)])
    text = path.read_text(encoding="utf-8", errors="replace")
    problems = []
    if not re.search(rf"release:\s*{re.escape(release)}\b", text):
        problems.append(f"does not name release {release}")
    stamped = re.search(r"commit:\s*([0-9a-f]{7,40})", text)
    if not stamped:
        problems.append("no `commit:` line")
    elif commit and not commit.startswith(stamped.group(1)) and not stamped.group(1).startswith(commit[:7]):
        problems.append(f"commit {stamped.group(1)} is not HEAD ({commit[:12]})")
    if not re.search(r"tester:\s*\S+", text):
        problems.append("no `tester:` line")
    boxes = re.findall(r"^\s*[-*] \[([ xX])\]", text, re.M)
    if len(boxes) < PLAYTEST_BOXES:
        problems.append(f"{len(boxes)} checklist boxes, need at least {PLAYTEST_BOXES}")
    elif any(box == " " for box in boxes):
        problems.append(f"{boxes.count(' ')} checklist box(es) unticked")
    evidence = re.findall(r"^\s*evidence:\s*(\S+)", text, re.M)
    if not evidence:
        problems.append("no `evidence:` path (log, screenshot or capture)")
    else:
        missing = [e for e in evidence if not (pin / e).exists() and not Path(e).exists()]
        if missing:
            problems.append("evidence path missing: " + ", ".join(missing[:2]))
    if problems:
        return check_bad("playtest.md: " + "; ".join(problems), [str(path)])
    return check_ok([f"playtest signed by {re.search(r'tester:\s*(\S+)', text).group(1)}, "
                     f"{len(boxes)} checks, commit {stamped.group(1)}"], [str(path)])


# ---------------------------------------------------------------- driver


def git_head(tree: Path) -> str:
    code, output = sh(["git", "rev-parse", "HEAD"], tree, timeout=30)
    return output.strip() if code == 0 else ""


def version_key(release: str):
    return tuple(int(part) if part.isdigit() else 0 for part in release.split("."))


def fmt_ratio(pair, empty="—"):
    if pair[0] is None or pair[1] is None:
        return empty
    return f"{pair[0]}/{pair[1]}"


def print_index(roots) -> int:
    """Emit VERSIONS.md rows from the verdicts, so the index is generated, not written."""
    found = {}
    for root in roots:
        if not Path(root).is_dir():
            continue
        for path in Path(root).glob("**/delta-verdict.json"):
            verdict = read_json(path)
            if verdict and verdict.get("release"):
                found[verdict["release"]] = verdict
    if not found:
        print("delta_verify: no delta-verdict.json under " + ", ".join(map(str, roots)),
              file=sys.stderr)
        return 3
    print("| Release | Tier | Subset | Items | Accepted | Status |")
    print("|---|---|---|---|---|---|")
    for release in sorted(found, key=version_key):
        verdict = found[release]
        scalars = verdict.get("scalars") or {}
        print(verdict.get("index_line") or (
            f"| {release} | {scalars.get('tier', 'D')} | "
            f"{fmt_ratio((scalars.get('suite_pass'), scalars.get('suite_total'))) } | "
            f"{fmt_ratio((scalars.get('items_closed'), scalars.get('items_total'))) } | "
            f"{scalars.get('items_accepted', 0)} | {verdict.get('state', 'unknown')} |"
        ))
    return 0


CHECK_ORDER = ("D1", "D2", "D3", "D4", "D5", "D6")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description="Decide whether one Minecraft release meets the delta bar (D1–D6).",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("release", nargs="?", help="e.g. 1.21.10 (omit with --index)")
    parser.add_argument("--index", action="store_true",
                        help="print VERSIONS.md rows from every delta-verdict.json found")
    parser.add_argument("--tree", help="the release's tree (default: lines/<v> or versions/<v>)")
    parser.add_argument("--pin", help="the pin directory (default: pins/<v>, else the tree)")
    parser.add_argument("--run", action="store_true",
                        help="run prepare, cargo check/test and the harnesses instead of "
                             "reading recorded output")
    parser.add_argument("--download", action="store_true", help="with --run: re-download the jars")
    parser.add_argument("--timeout", type=int, default=1800, help="seconds per harness")
    parser.add_argument("--check", action="append", choices=CHECK_ORDER,
                        help="only these checks (repeatable)")
    parser.add_argument("--json", action="store_true", help="print the verdict to stdout")
    parser.add_argument("--quiet", action="store_true", help="write the verdict, print nothing")
    args = parser.parse_args(argv)

    if args.index:
        roots = [args.pin] if args.pin else ["pins", "versions", "lines"]
        return print_index(roots)
    if not args.release:
        parser.error("release is required unless --index is given")

    tree = resolve_tree(args.release, args.tree)
    pin = resolve_pin(args.release, args.pin, tree)
    commit = git_head(tree)
    wanted = set(args.check or CHECK_ORDER)

    runners = {
        "D1": lambda: check_d1(tree, pin, args.release, args.run, args.download),
        "D2": lambda: check_d2(tree, pin, args.release, args.run, args.timeout),
        "D3": lambda: check_d3(tree, pin, args.release),
        "D4": lambda: check_d4(tree, pin, args.release, args.run, args.timeout, commit),
        "D5": lambda: check_d5(tree, pin, args.release, args.run, args.timeout, commit),
        "D6": lambda: check_d6(pin, args.release, commit),
    }
    checks = {}
    for name in CHECK_ORDER:
        checks[name] = runners[name]() if name in wanted else check_stop("not run (--check)")

    statuses = [entry["status"] for entry in checks.values()]
    if FAIL in statuses:
        verdict_state = "failed"
    elif BLOCKED in statuses:
        verdict_state = "blocked"
    else:
        verdict_state = "verified"

    tier_file = pin / "tier.txt"
    tier = tier_file.read_text(encoding="utf-8").strip()[:1].upper() if tier_file.exists() else "D"
    if tier not in ("T", "D"):
        raise ConfigurationError(f"{tier_file}: tier must be T (template) or D (delta), got {tier!r}")

    metrics = {}
    for entry in checks.values():
        metrics.update(entry.get("metrics") or {})
    scalars = {
        "tier": tier,
        "suite_pass": metrics.get("suite_pass"),
        "suite_total": metrics.get("suite_total"),
        "suite_exempt": metrics.get("suite_exempt", 0),
        "items_closed": metrics.get("items_closed"),
        "items_total": metrics.get("items_total"),
        "items_accepted": metrics.get("items_accepted", 0),
    }
    checked_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    index_line = (
        f"| {args.release} | {tier} | {fmt_ratio((scalars['suite_pass'], scalars['suite_total']))} | "
        f"{fmt_ratio((scalars['items_closed'], scalars['items_total']))} | "
        f"{scalars['items_accepted']} | {verdict_state} {checked_at[:10]} |"
    )

    verdict = {
        "schema": SCHEMA,
        "release": args.release,
        "commit": commit,
        "tree": str(tree),
        "pin": str(pin),
        "mode": "run" if args.run else "recorded",
        "verified": verdict_state == "verified",
        "state": verdict_state,
        "checked_at": checked_at,
        "scalars": scalars,
        "index_line": index_line,
        "checks": checks,
    }
    out = pin / "delta-verdict.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(verdict, indent=2) + "\n", encoding="utf-8")

    if args.json:
        print(json.dumps(verdict, indent=2))
    if not args.quiet and not args.json:
        head = {"verified": "DELTA-VERIFIED", "failed": "NOT DELTA-VERIFIED",
                "blocked": "BLOCKED (evidence missing — not verified)"}[verdict_state]
        print(f"{args.release}: {head}")
        for name in CHECK_ORDER:
            entry = checks[name]
            print(f"  {name} {entry['status']:<8} {'; '.join(entry['notes'])[:150]}")
        print(f"  verdict: {out}")

    return {"verified": 0, "failed": STATUS_EXIT[FAIL], "blocked": STATUS_EXIT[BLOCKED]}[verdict_state]


if __name__ == "__main__":
    try:
        sys.exit(main())
    except ConfigurationError as error:
        print(f"delta_verify: {error}", file=sys.stderr)
        sys.exit(2)
    except KeyboardInterrupt:
        sys.exit(130)
