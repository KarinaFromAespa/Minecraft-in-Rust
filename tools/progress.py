#!/usr/bin/env python3
"""progress.py — regenerate the Progress table in README.md from the repo's data.

Reads versions/*/status.json (behaviour items, harness counts) and
versions/*/delta-verdict.json (verified templates and deltas, by tier) and
prints the markdown table the README carries. Run from the repo root and
paste the output into the README's Progress section.

The Phase 0 line in the README is hand-maintained prose, not generated:
it changes only when a milestone lands, and a milestone is a judgement
call, not a file count.

    python3 tools/progress.py
"""

from __future__ import annotations

import json
from pathlib import Path

TOTAL_RELEASES = 103   # roadmap §0: 11 era templates + 92 delta releases
TEMPLATES = 11
DELTAS = 92


def read_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def pct(done: int, total: int) -> str:
    if not total:
        return "—"
    value = 100 * done / total
    return "0%" if value == 0 else f"{value:.1f}%"


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    versions = sorted(p for p in (root / "versions").iterdir() if p.is_dir())

    behaviour_closed = behaviour_total = 0
    harness_pass = harness_total = 0
    profiled = 0
    verified_templates = verified_deltas = 0
    for version in versions:
        status = read_json(version / "status.json") or {}
        # The base (26.3) is upstream's client, not a port: its profile
        # is the baseline the ports are measured against, not progress.
        # Counting it would inflate the harness aggregate.
        if status.get("role") == "base":
            continue
        behaviour = status.get("behaviour") or {}
        behaviour_closed += behaviour.get("closed") or 0
        behaviour_total += behaviour.get("total") or 0
        harness = status.get("harness") or {}
        if harness.get("pass") is not None:
            profiled += 1
            harness_pass += harness["pass"]
            harness_total += harness.get("total", 0)

        verdict = read_json(version / "delta-verdict.json")
        if verdict and verdict.get("verified"):
            tier = (verdict.get("scalars") or {}).get("tier")
            if tier is None:
                tier_file = version / "tier.txt"
                tier = tier_file.read_text(encoding="utf-8").strip()[:1] if tier_file.exists() else ""
            if tier == "T":
                verified_templates += 1
            elif tier == "D":
                verified_deltas += 1
        elif status.get("status") == "parity-verified":
            verified_templates += 1
        elif status.get("status") == "delta-verified":
            verified_deltas += 1

    verified = verified_templates + verified_deltas
    rows = [
        ("Releases verified (templates at parity + deltas gated)",
         f"{verified} / {TOTAL_RELEASES}", pct(verified, TOTAL_RELEASES)),
        ("Era templates at full parity",
         f"{verified_templates} / {TEMPLATES}", pct(verified_templates, TEMPLATES)),
        ("Delta releases verified",
         f"{verified_deltas} / {DELTAS}", pct(verified_deltas, DELTAS)),
        ("Trees published on main",
         f"{len(versions)} / {TOTAL_RELEASES}", pct(len(versions), TOTAL_RELEASES)),
        ("Behaviour items closed",
         f"{behaviour_closed:,} / {behaviour_total:,}", pct(behaviour_closed, behaviour_total)),
        (f"Harnesses passing ({profiled} profiled version{'s' if profiled != 1 else ''})",
         f"{harness_pass} / {harness_total}", pct(harness_pass, harness_total)),
    ]
    print("| Measure | Done | % |")
    print("|---|---|---|")
    for measure, done, percent in rows:
        print(f"| {measure} | {done} | {percent} |")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
