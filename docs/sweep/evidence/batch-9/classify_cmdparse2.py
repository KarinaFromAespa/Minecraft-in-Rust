#!/usr/bin/env python3
"""Second pass: bucket OK_DIFFERS and CURSOR_DIFFERS by sub-pattern."""
import json, re
from collections import Counter, defaultdict

BASE = "/home/hatch/workspace/mcir-work/sweep-batch-0/versions/1.21.7/.local/command-parse-parity"

ok_vtrue = Counter()   # vanilla ok, rust fail: bucket by first token + rust error
ok_vfalse = Counter()  # vanilla fail, rust ok: bucket by first token + vanilla error
ok_vtrue_ex = defaultdict(list)
ok_vfalse_ex = defaultdict(list)
cur_pat = Counter()
cur_ex = defaultdict(list)
sugg = Counter()
sugg_ex = []

def firsttok(s):
    return s.split(" ", 1)[0] if s else ""

for perm in ("perm4", "perm2", "perm0"):
    with open(f"{BASE}/{perm}/mismatches.jsonl") as f:
        for line in f:
            rec = json.loads(line)
            if rec.get("kind") == "suggestions":
                sugg[firsttok(rec["in"])] += 1
                if len(sugg_ex) < 20:
                    sugg_ex.append(rec)
                continue
            v, r = rec.get("vanilla", {}), rec.get("rust", {})
            if v.get("ok") != r.get("ok"):
                if v.get("ok"):
                    key = (firsttok(rec["in"]), r.get("error"))
                    ok_vtrue[key] += 1
                    if len(ok_vtrue_ex[key]) < 2:
                        ok_vtrue_ex[key].append(rec["in"])
                else:
                    key = (firsttok(rec["in"]), v.get("error"))
                    ok_vfalse[key] += 1
                    if len(ok_vfalse_ex[key]) < 2:
                        ok_vfalse_ex[key].append(rec["in"])
            elif v.get("cursor") != r.get("cursor"):
                # sub-pattern: what kind of input?
                s = rec["in"]
                if re.search(r"@\w+\[[^\]]*(\w+)=[^,\]]*,\1=", s):
                    pat = "selector duplicate option"
                elif re.search(r"@a\[|@p\[|@r\[", s) and "isn't applicable" in (v.get("error") or ""):
                    pat = "selector inapplicable option on @a/@p/@r"
                elif "isn't applicable" in (v.get("error") or ""):
                    pat = "selector inapplicable option (other)"
                else:
                    pat = "other: " + (v.get("error") or "")[:60]
                cur_pat[pat] += 1
                if len(cur_ex[pat]) < 3:
                    cur_ex[pat].append(rec)

print("=== OK_DIFFERS: vanilla OK, rust FAIL (264) ===")
for (tok, err), c in ok_vtrue.most_common(30):
    print(f"  {c:4d}  {tok}  rust_err={err}  e.g. {ok_vtrue_ex[(tok, err)][:1]}")
print("=== OK_DIFFERS: vanilla FAIL, rust OK (51) ===")
for (tok, err), c in ok_vfalse.most_common(30):
    print(f"  {c:4d}  {tok}  vanilla_err={err}  e.g. {ok_vfalse_ex[(tok, err)][:1]}")
print("=== CURSOR_DIFFERS patterns (345) ===")
for pat, c in cur_pat.most_common(20):
    print(f"  {c:4d}  {pat}")
    for rec in cur_ex[pat][:2]:
        print("       ", json.dumps(rec)[:300])
print("=== SUGGESTIONS by command (16) ===")
for tok, c in sugg.most_common():
    print(f"  {c:4d}  {tok}")
