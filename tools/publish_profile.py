#!/usr/bin/env python3
"""publish_profile.py — freeze a harness run into `versions/<id>/profile.json`.

The delta bar's D4 and D5 judge a published record of which harnesses ran and how
they ended. `parity.py` keeps that per worktree, unfrozen, in
`.local/parity-runner/state.json`; this turns it into one committed file per
release, stamped with the commit it belongs to, so `tools/delta_verify.py` can
refuse a profile from a different tree instead of judging stale evidence.

    python3 tools/publish_profile.py 1.21.10                 # find the state file
    python3 tools/publish_profile.py 1.21.10 --state a.json --state b.json
    python3 tools/publish_profile.py 1.21.10 --commit "$GITHUB_SHA"

    python3 tools/publish_profile.py 1.21.10 --out versions/1.21.10/profile.json

Exit: 0 published · 1 nothing to publish · 2 usage error.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

SCHEMA = "profile/1"
ALIASES = {"pass": "pass", "passed": "pass", "ok": "pass", "green": "pass",
           "fail": "fail", "failed": "fail", "red": "fail"}
PASS_KEYS = ("passing", "passed", "pass")
FAIL_KEYS = ("failing", "failed", "fail")
CONTAINERS = ("harnesses", "harness", "suite", "results", "tests", "summary")


def read_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def normalise(raw) -> dict[str, dict]:
    """Any parity.py state or profile shape → {harness name: record}."""
    if not isinstance(raw, dict):
        return {}
    for container in [raw] + [raw.get(key) for key in CONTAINERS]:
        if not isinstance(container, dict):
            continue
        buckets = {}
        for key in PASS_KEYS:
            if isinstance(container.get(key), list):
                buckets["pass"] = container[key]
                break
        for key in FAIL_KEYS:
            if isinstance(container.get(key), list):
                buckets["fail"] = container[key]
                break
        if buckets:
            return {
                str(name): {"status": status}
                for status, names in buckets.items()
                for name in names if isinstance(name, str) and name
            }
    entries = None
    for key in CONTAINERS:
        value = raw.get(key)
        if isinstance(value, (dict, list)):
            entries = value
            break
    if isinstance(entries, list):
        entries = {
            (entry.get("name") or entry.get("harness")): entry
            for entry in entries if isinstance(entry, dict)
        }
    if not isinstance(entries, dict):
        return {}
    harnesses = {}
    for name, value in entries.items():
        if not name:
            continue
        if isinstance(value, str):
            harnesses[name] = {"status": ALIASES.get(value.lower(), value.lower())}
        elif isinstance(value, dict):
            status = value.get("status") or value.get("verdict") or value.get("result")
            if status is None and "passed" in value:
                status = "pass" if value["passed"] else "fail"
            if status is None:
                continue
            record = {**value, "status": ALIASES.get(str(status).lower(), str(status).lower())}
            harnesses[name] = record
    return harnesses


def candidates(release: str) -> list[Path]:
    found = []
    ports = os.environ.get("HORIZON_PORTS")
    if ports:
        found.append(Path(ports) / release / ".local/parity-runner/state.json")
    found.append(Path("versions") / release / ".local/parity-runner/state.json")
    found.append(Path("lines") / release / ".local/parity-runner/state.json")
    found.extend(sorted(Path(".").glob(f"**/{release}/.local/parity-runner/state.json")))
    unique = []
    for path in found:
        if path.exists() and path not in unique:
            unique.append(path)
    return unique


def head_commit() -> str:
    run = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True)
    return run.stdout.strip() if run.returncode == 0 else ""


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("release")
    parser.add_argument("--state", action="append", default=[],
                        help="a state.json to merge (repeatable; default: find one)")
    parser.add_argument("--commit", default="", help="the commit this run belongs to (default: HEAD)")
    parser.add_argument("--out", default="", help="default: versions/<release>/profile.json")
    args = parser.parse_args(argv)

    sources = [Path(path) for path in args.state] or candidates(args.release)
    if not sources:
        print(f"publish_profile: no parity state for {args.release}; run tools/parity.py first",
              file=sys.stderr)
        return 1

    merged: dict[str, dict] = {}
    used = []
    for path in sources:
        raw = read_json(path)
        if raw is None:
            print(f"publish_profile: cannot read {path}", file=sys.stderr)
            return 2
        harnesses = normalise(raw)
        if harnesses:
            merged.update(harnesses)
            used.append(str(path))

    if not merged:
        print(f"publish_profile: {', '.join(used) or sources[0]} holds no harness results",
              file=sys.stderr)
        return 1

    commit = args.commit or head_commit()
    counts = {"pass": 0, "fail": 0, "other": 0}
    for record in merged.values():
        status = record.get("status")
        counts["pass" if status == "pass" else "fail" if status == "fail" else "other"] += 1

    profile = {
        "schema": SCHEMA,
        "release": args.release,
        "commit": commit,
        "published_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "sources": used,
        "summary": {"total": len(merged), **counts,
                    "passing": sorted(n for n, r in merged.items() if r.get("status") == "pass"),
                    "failing": sorted(n for n, r in merged.items() if r.get("status") == "fail")},
        "harnesses": dict(sorted(merged.items())),
    }

    out = Path(args.out or Path("versions") / args.release / "profile.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(profile, indent=2) + "\n", encoding="utf-8")
    print(f"{out}: {len(merged)} harnesses "
          f"({counts['pass']} pass, {counts['fail']} fail, {counts['other']} other) "
          f"at {commit[:12] or 'no commit'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
