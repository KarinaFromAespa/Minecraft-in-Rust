#!/usr/bin/env python3
"""Classify command_parse mismatches (Batch 0 sample output) into patterns."""
import json, re, sys
from collections import Counter, defaultdict

BASE = "/home/hatch/workspace/mcir-work/sweep-batch-0/versions/1.21.7/.local/command-parse-parity"

def classify(rec):
    v, r = rec.get("vanilla", {}), rec.get("rust", {})
    kind = rec.get("kind", "?")
    if kind != "parse":
        return ("kind:" + kind, None)
    vok, rok = v.get("ok"), r.get("ok")
    vcur, rcur = v.get("cursor"), r.get("cursor")
    verr, rerr = v.get("error"), r.get("error")
    if vok != rok:
        return ("OK_DIFFERS", (vok, rok, verr, rerr))
    if vcur != rcur:
        return ("CURSOR_DIFFERS", (vcur, rcur, verr, rerr))
    if verr != rerr:
        # error text differs but ok+cursor agree: bucket by error pair shape
        if rerr and re.fullmatch(r"[a-z0-9_.]+", rerr or "") and verr and not re.fullmatch(r"[a-z0-9_.]+", verr or ""):
            return ("ERR_TEXT_KEY_VS_RESOLVED", (rerr, verr))
        return ("ERR_TEXT_OTHER", (verr, rerr))
    return ("OTHER_FIELD_DIFF", (v, r))

def main():
    total = Counter()
    errpair = Counter()
    okdiff = Counter()
    cursordiff = Counter()
    other = Counter()
    kindc = Counter()
    examples = defaultdict(list)
    for perm in ("perm4", "perm2", "perm0"):
        path = f"{BASE}/{perm}/mismatches.jsonl"
        with open(path) as f:
            for line in f:
                rec = json.loads(line)
                kindc[rec.get("kind", "?")] += 1
                cls, detail = classify(rec)
                total[cls] += 1
                if cls == "ERR_TEXT_KEY_VS_RESOLVED":
                    errpair[detail] += 1
                    if len(examples[("key", detail[0])]) < 2:
                        examples[("key", detail[0])].append(rec["in"])
                elif cls == "ERR_TEXT_OTHER":
                    other[detail] += 1
                elif cls == "OK_DIFFERS":
                    okdiff[(detail[0], detail[1])] += 1
                    if len(examples["okdiff"]) < 15:
                        examples["okdiff"].append(rec)
                elif cls == "CURSOR_DIFFERS":
                    cursordiff[(detail[0], detail[1])] += 1
                    if len(examples["cursordiff"]) < 15:
                        examples["cursordiff"].append(rec)
                else:
                    if len(examples["otherfield"]) < 10:
                        examples["otherfield"].append(rec)
    print("=== kind counts ===")
    for k, c in kindc.most_common():
        print(f"  {k}: {c}")
    print("=== class totals ===")
    for k, c in total.most_common():
        print(f"  {k}: {c}")
    print("=== ERR_TEXT_KEY_VS_RESOLVED: top rust keys (count, key, sample vanilla) ===")
    bykey = Counter()
    sample_v = {}
    for (rerr, verr), c in errpair.items():
        bykey[rerr] += c
        sample_v.setdefault(rerr, verr)
    for k, c in bykey.most_common(40):
        print(f"  {c:6d}  {k}  | vanilla e.g.: {sample_v[k]!r}")
    print("=== ERR_TEXT_OTHER: top pairs ===")
    for (verr, rerr), c in other.most_common(25):
        print(f"  {c:6d}  vanilla={verr!r} rust={rerr!r}")
    print("=== OK_DIFFERS combos ===")
    for k, c in okdiff.most_common():
        print(f"  {c:6d}  vanilla_ok={k[0]} rust_ok={k[1]}")
    print("=== CURSOR_DIFFERS combos (top 15) ===")
    for k, c in cursordiff.most_common(15):
        print(f"  {c:6d}  vanilla_cursor={k[0]} rust_cursor={k[1]}")
    print("=== OK_DIFFERS examples ===")
    for rec in examples["okdiff"]:
        print("  ", json.dumps(rec)[:400])
    print("=== CURSOR_DIFFERS examples ===")
    for rec in examples["cursordiff"]:
        print("  ", json.dumps(rec)[:400])
    print("=== OTHER_FIELD_DIFF examples ===")
    for rec in examples["otherfield"]:
        print("  ", json.dumps(rec)[:500])

if __name__ == "__main__":
    main()
