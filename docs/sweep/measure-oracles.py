#!/usr/bin/env python3
"""Batch 0 measurement: javac-compilability of every Java oracle vs the 1.21.7 jars.

Why a fixpoint, not 391 individual javac runs: one javac invocation costs
~20s wall here (throttled VM), so per-file runs would take ~2h per classpath.
Instead, per classpath:
  1. Big-compile ALL files -> files with errors E, files without C.
  2. Big-compile C alone; files that fail there (they referenced a file in E,
     which is now absent) drop out of C; repeat until the compile passes.
  The fixpoint set is exactly the set of files that compile standalone with
  -sourcepath: a file fails standalone iff it has own errors or transitively
  pulls a file with errors, and the fixpoint drops exactly those.
Error attribution per failing file comes from the all-files big-compile log
(own errors) or, when the file itself is clean there, from its references to
failing parity files (cascade).

Classpaths (the three driver configurations):
  server: server-game.jar + server libraries + jsr305 (direct drivers)
  client: client.jar + client libraries (oracle_harness drivers)
  both:   server-game.jar + client.jar + both library sets (superset:
          compilable at all against 1.21.7?)

Outputs: results.json (per-file status + first error + class), drivers.json
(driver->oracle map), plus the raw big-compile logs.
"""
import json
import os
from pathlib import Path
import re
import subprocess
import sys

# Version tree under test. When run from its committed home (docs/sweep/),
# point HORIZON_VERSION_DIR at the version tree; when run from
# versions/<v>/.local/measure-oracles/ (the working copy) it is derived.
ROOT = Path(os.environ.get("HORIZON_VERSION_DIR", Path(__file__).resolve().parents[2]))
HERE = Path(__file__).resolve().parent
if HERE.name != "measure-oracles":
    # Committed copy (docs/sweep/): write outputs to the version tree's .local.
    HERE = ROOT / ".local" / "measure-oracles"
    HERE.mkdir(parents=True, exist_ok=True)
PARITY = ROOT / "mods/horizon/src/test/java/dev/horizonlab/parity"
SRCROOT = ROOT / "mods/horizon/src/test/java"

sys.path.insert(0, str(ROOT / "tools"))
import lab  # noqa: E402
import test_rust_parity as base  # noqa: E402
import test_model_baking_parity as baking  # noqa: E402

lock, jars, libraries = base.verify_artifacts()
client_libraries = baking.client_libraries()
JSR305 = ROOT / "tools/jsr305-3.0.2.jar"
CPS = {
    "server": os.pathsep.join([str(jars["server"])] + [str(p) for p in libraries]),
    # oracle_harness drivers use client jar + client libraries, WITHOUT jsr305
    # — and javac CRASHES (CompletionFailure for javax.annotation.Nullable)
    # without it on 1.21.7 jars. That omission is a Batch 0 driver defect,
    # recorded separately. The measurement adds jsr305 here so it measures
    # oracle compilability, not the harness bug.
    "client": os.pathsep.join([str(jars["client"])] + [str(p) for p in client_libraries]
                              + [str(JSR305)]),
    "both": os.pathsep.join([str(jars["client"]), str(jars["server"])]
                            + [str(p) for p in libraries] + [str(p) for p in client_libraries]),
}
JAVA_MAJOR = str(lock["minecraft"]["java_major"])
FILES = sorted(f.stem for f in PARITY.glob("*.java"))
FILE_SET = set(FILES)

ERR_RE = re.compile(r"^(\S+\.java):(\d+): error: (.*)$", re.M)


def big_compile(names, cp, tag, sourcepath=True):
    """Compile `names` together; returns (returncode, {file: [error lines]}, raw).

    Fixpoint iterations MUST run with sourcepath=False: with -sourcepath a
    clean file referencing a broken file pulls the broken file's source back
    in, and the errors land in the broken file (not in the clean set), so the
    fixpoint never converges. Without -sourcepath the missing helper is a
    cannot-find-symbol error in the referring (clean-set) file, which drops.
    """
    outdir = HERE / "classes" / tag
    outdir.mkdir(parents=True, exist_ok=True)
    cmd = ["javac", "-J-Xmx1g", "--release", JAVA_MAJOR, "-proc:none",
           "-Xmaxerrs", "10000", "-nowarn"]
    if sourcepath:
        cmd += ["-sourcepath", str(SRCROOT)]
    cmd += ["-cp", cp, "-d", str(outdir),
            *[str(PARITY / f"{n}.java") for n in names]]
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=900)
    raw = proc.stdout + proc.stderr
    (HERE / f"{tag}.log").write_text(raw)
    per_file = {}
    for m in ERR_RE.finditer(raw):
        per_file.setdefault(Path(m.group(1)).name, []).append(
            f"{m.group(2)}: {m.group(3)}")
    return proc.returncode, per_file, raw


def classify(first_error):
    msg = first_error
    if "package " in msg and "does not exist" in msg:
        return "missing-package"
    if "cannot find symbol" in msg:
        return "missing-symbol"
    if "incompatible types" in msg:
        return "incompatible-types"
    if "cannot access" in msg:
        return "cannot-access"
    if "method does not override or implement" in msg:
        return "no-override"
    if "unreported exception" in msg:
        return "unreported-exception"
    if "is not abstract and does not override" in msg:
        return "missing-override"
    if "class file for " in msg and "not found" in msg:
        return "missing-classfile"
    if "constructor " in msg and "cannot be applied" in msg:
        return "constructor-mismatch"
    return "other"


def referenced_parity_files(name):
    """Other parity-dir classes this file references by simple name."""
    text = (PARITY / f"{name}.java").read_text(errors="replace")
    refs = set()
    for token in re.findall(r"\b([A-Z][A-Za-z0-9_]+)\b", text):
        if token in FILE_SET and token != name:
            refs.add(token)
    return refs


def measure_cp(tag):
    print(f"--- classpath: {tag}", flush=True)
    rc, errors, raw = big_compile(FILES, CPS[tag], f"all-{tag}")
    (HERE / f"big-compile-{tag}.log").write_text(raw)
    err_files = {f[:-5] for f in errors}
    print(f"  all-files compile: rc={rc} files-with-own-errors={len(err_files)}", flush=True)
    # Fixpoint: compile the clean set alone until it passes.
    clean = [n for n in FILES if n not in err_files]
    iteration = 0
    while clean:
        iteration += 1
        rc2, errors2, _ = big_compile(clean, CPS[tag], f"clean-{tag}-{iteration}",
                                      sourcepath=False)
        if rc2 == 0:
            break
        dropped = {f[:-5] for f in errors2} & set(clean)
        print(f"  clean-set iteration {iteration}: {len(dropped)} cascade drops", flush=True)
        if not dropped:
            raw2 = (HERE / f"clean-{tag}-{iteration}.log").read_text()
            raise RuntimeError(f"fixpoint stuck on {tag} (rc={rc2}, "
                               f"{len(errors2)} files with errors, "
                               f"raw tail: {raw2[-1500:]})")
        clean = [n for n in clean if n not in dropped]
    passing = set(clean)
    print(f"  standalone pass: {len(passing)}/{len(FILES)}", flush=True)
    results = {}
    for n in FILES:
        if n in passing:
            results[n] = {"ok": True}
        else:
            own = errors.get(f"{n}.java")
            if own:
                results[n] = {"ok": False, "class": classify(own[0]),
                              "first_error": own[0], "n_errors": len(own),
                              "errors": own[:8]}
            else:
                broken_refs = sorted(referenced_parity_files(n) - passing)
                results[n] = {"ok": False, "class": "cascade",
                              "first_error": "depends on broken: " + ", ".join(broken_refs),
                              "cascade_via": broken_refs}
    return results


def main():
    print(f"files: {len(FILES)}  java_major: {JAVA_MAJOR}", flush=True)
    print(f"server jar: {jars['server']}", flush=True)
    print(f"client jar: {jars['client']}", flush=True)
    print(f"server libraries: {len(libraries)}  client libraries: {len(client_libraries)}",
          flush=True)
    tags = sys.argv[1:] or ["server", "client", "both"]
    all_results = {}
    if (HERE / "results.json").exists():
        all_results = json.loads((HERE / "results.json").read_text())["files"]
        print(f"loaded prior results.json ({len(all_results)} files)", flush=True)
    for tag in tags:
        per = measure_cp(tag)
        for n, r in per.items():
            all_results.setdefault(n, {})[tag] = r
        # Checkpoint after each classpath so a crash keeps completed passes.
        out = {"java_major": JAVA_MAJOR,
               "server_jar": str(jars["server"]),
               "client_jar": str(jars["client"]),
               "method": "fixpoint big-compile per classpath (see script header)",
               "files": all_results}
        (HERE / "results.json").write_text(json.dumps(out, indent=1))

    # Driver -> oracle map.
    drivers = {}
    for d in sorted((ROOT / "tools").glob("test_*_parity.py")):
        text = d.read_text(errors="replace")
        refs = {m.group(1) for m in re.finditer(r"([A-Za-z0-9_]+)\.java", text)}
        refs |= {m.group(1) for m in
                 re.finditer(r"['\"](Vanilla[A-Za-z0-9_]+|Parity[A-Za-z0-9_]+)['\"]", text)}
        refs = sorted(r for r in refs if r in FILE_SET)
        drivers[d.stem] = {
            "oracles": refs,
            "uses_oracle_harness": "oracle_harness" in text,
            "has_javac": "javac" in text,
            "uses_client_jar": bool(re.search(r"jars\[['\"]client['\"]\]", text)),
            "uses_server_jar": bool(re.search(r"jars\[['\"]server['\"]\]", text)),
        }
    n_oracle_drivers = sum(1 for v in drivers.values() if v["oracles"])
    print(f"drivers mapped: {len(drivers)} ({n_oracle_drivers} reference an oracle)",
          flush=True)

    out = {"java_major": JAVA_MAJOR,
           "server_jar": str(jars["server"]),
           "client_jar": str(jars["client"]),
           "method": "fixpoint big-compile per classpath (see script header)",
           "files": all_results}
    (HERE / "results.json").write_text(json.dumps(out, indent=1))
    (HERE / "drivers.json").write_text(json.dumps(drivers, indent=1))

    from collections import Counter
    for tag in ("server", "client", "both"):
        npass = sum(1 for r in all_results.values() if r[tag]["ok"])
        classes = Counter(r[tag]["class"] for r in all_results.values()
                          if not r[tag]["ok"])
        print(f"{tag}: {npass} pass / {len(FILES) - npass} fail; classes: {dict(classes)}",
              flush=True)
    any_pass = sum(1 for r in all_results.values()
                   if any(r[t]["ok"] for t in ("server", "client", "both")))
    print(f"pass on at least one cp: {any_pass}; fail on all three: {len(FILES) - any_pass}",
          flush=True)


if __name__ == "__main__":
    main()
