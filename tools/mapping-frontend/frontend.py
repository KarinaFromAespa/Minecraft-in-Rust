#!/usr/bin/env python3
"""Mappings front-end: turn an obfuscated Minecraft client jar into a named one.

Pipeline step 3 of the programme's per-version playbook (docs/ROADMAP.md):
before any spec tooling (decompile, bytecode queries, exporters) can read a
pre-26.1 jar, the jar has to be remapped from its obfuscated names to real
names using that version's mapping table.

What this script does, for a given version id (e.g. 1.21.11):

  1. Resolve the version through Mojang's official version manifest
     (piston-meta) -> per-version JSON, verifying SHA-1 at every hop.
  2. Download the client jar and the official client mappings into
     .local/mapping-frontend/<version>/ (git-ignored, never committed).
  3. Remap obfuscated -> named with Fabric's tiny-remapper, reading the
     mapping table through Fabric's mapping-io (the normalization layer:
     ProGuard txt today; SRG/TSRG/tiny plug in at the same seam later).
     The remapper driver is src/Remap.java, compiled on first use.
  4. Emit .local/mapping-frontend/<version>/client-named.jar and print
     coverage statistics (how many of the jar's classes carry mapped names).
  5. With --prove: decompile a small sample of well-known classes from the
     named jar with Vineflower and show that the output uses real names.

Currently supported mapping source: Mojang's official ProGuard tables,
published for releases 1.14.4 through 1.21.11. Versions without official
tables (everything older, and the unobfuscated 26.x line which needs no
remapping) are reported as unsupported rather than silently mishandled.

Usage:
    python3 frontend.py <version> [--prove] [--force]

Requirements: Python 3.10+, a JDK (javac) for the one-time driver compile
and a Java 17+ runtime. Set JAVAC/JAVA to override tool discovery.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
import urllib.request
import zipfile
from pathlib import Path

MANIFEST_URL = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
FABRIC_MAVEN = "https://maven.fabricmc.net"
MAVEN_CENTRAL = "https://repo1.maven.org/maven2"

# Pinned third-party tool dependencies (downloaded into .local, never committed).
DEPS = {
    "tiny-remapper-0.15.0.jar": f"{FABRIC_MAVEN}/net/fabricmc/tiny-remapper/0.15.0/tiny-remapper-0.15.0.jar",
    "mapping-io-0.7.1.jar": f"{FABRIC_MAVEN}/net/fabricmc/mapping-io/0.7.1/mapping-io-0.7.1.jar",
    "asm-9.9.1.jar": f"{MAVEN_CENTRAL}/org/ow2/asm/asm/9.9.1/asm-9.9.1.jar",
    "asm-commons-9.9.1.jar": f"{MAVEN_CENTRAL}/org/ow2/asm/asm-commons/9.9.1/asm-commons-9.9.1.jar",
    "asm-tree-9.9.1.jar": f"{MAVEN_CENTRAL}/org/ow2/asm/asm-tree/9.9.1/asm-tree-9.9.1.jar",
    "asm-util-9.9.1.jar": f"{MAVEN_CENTRAL}/org/ow2/asm/asm-util/9.9.1/asm-util-9.9.1.jar",
    "asm-analysis-9.9.1.jar": f"{MAVEN_CENTRAL}/org/ow2/asm/asm-analysis/9.9.1/asm-analysis-9.9.1.jar",
    "vineflower-1.12.0.jar": f"{MAVEN_CENTRAL}/org/vineflower/vineflower/1.12.0/vineflower-1.12.0.jar",
}
REMAPPER_DEPS = [name for name in DEPS if name != "vineflower-1.12.0.jar"]

# Well-known classes used for the --prove decompile sample (named form).
PROOF_CLASSES = [
    "net/minecraft/world/level/Level",
    "net/minecraft/client/Minecraft",
    "net/minecraft/world/entity/Entity",
    "net/minecraft/core/BlockPos",
    "net/minecraft/world/item/ItemStack",
    "net/minecraft/server/MinecraftServer",
]

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]
WORK_DIR = REPO_ROOT / ".local" / "mapping-frontend"
DRIVER_SRC = SCRIPT_DIR / "src" / "Remap.java"


def sha1_of(path: Path) -> str:
    h = hashlib.sha1()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def fetch(url: str, dest: Path, sha1: str | None = None, force: bool = False) -> Path:
    """Download url to dest (atomically), verifying sha1 when given.

    Skips the download when dest already exists and matches.
    """
    if dest.exists() and not force:
        if sha1 is None or sha1_of(dest) == sha1:
            print(f"  cached  {dest.name}")
            return dest
        print(f"  sha1 mismatch, refetching {dest.name}")
    dest.parent.mkdir(parents=True, exist_ok=True)
    tmp = dest.with_suffix(dest.suffix + ".part")
    req = urllib.request.Request(url, headers={"User-Agent": "mapping-frontend/1.0"})
    print(f"  fetch   {url}")
    with urllib.request.urlopen(req) as resp, open(tmp, "wb") as fh:
        shutil.copyfileobj(resp, fh, length=1 << 20)
    if sha1 is not None:
        actual = sha1_of(tmp)
        if actual != sha1:
            tmp.unlink(missing_ok=True)
            raise SystemExit(f"SHA-1 mismatch for {url}: expected {sha1}, got {actual}")
    tmp.replace(dest)
    print(f"  saved   {dest} ({dest.stat().st_size} bytes, sha1 ok)")
    return dest


def find_tool(env_var: str, name: str) -> str:
    """Locate a Java tool (javac/java): env override, PATH, bundled tools."""
    override = os.environ.get(env_var)
    if override:
        return override
    on_path = shutil.which(name)
    if on_path:
        return on_path
    tools_dir = Path.home() / "workspace" / "tools"
    if tools_dir.is_dir():
        # Prefer full JDKs over bare JREs (a JRE has java but no javac).
        candidates = sorted(tools_dir.glob(f"*/bin/{name}"),
                            key=lambda p: ("jre" in p.parent.parent.name, str(p)))
        if candidates:
            return str(candidates[0])
    raise SystemExit(
        f"could not find {name}: set {env_var}, add it to PATH, "
        f"or install a JDK under ~/workspace/tools/")


def ensure_deps(force: bool) -> Path:
    deps_dir = WORK_DIR / "deps"
    print("tool dependencies:")
    for name, url in DEPS.items():
        fetch(url, deps_dir / name, force=force)
    return deps_dir


def ensure_driver_compiled(deps_dir: Path, javac: str) -> Path:
    classes_dir = WORK_DIR / "build" / "classes"
    class_file = classes_dir / "Remap.class"
    if class_file.exists() and class_file.stat().st_mtime >= DRIVER_SRC.stat().st_mtime:
        return classes_dir
    classes_dir.mkdir(parents=True, exist_ok=True)
    cp = os.pathsep.join(str(deps_dir / n) for n in ("mapping-io-0.7.1.jar", "tiny-remapper-0.15.0.jar"))
    print("compiling remapper driver (src/Remap.java):")
    subprocess.run([javac, "-cp", cp, "-d", str(classes_dir), str(DRIVER_SRC)],
                   check=True)
    return classes_dir


def resolve_version(version: str, force: bool) -> dict:
    manifest_path = WORK_DIR / "version_manifest_v2.json"
    stale = (not manifest_path.exists()
             or time.time() - manifest_path.stat().st_mtime > 24 * 3600)
    if stale or force:
        fetch(MANIFEST_URL, manifest_path, force=True)
    else:
        print(f"  cached  {manifest_path.name}")
    manifest = json.loads(manifest_path.read_text())
    entry = next((v for v in manifest["versions"] if v["id"] == version), None)
    if entry is None:
        raise SystemExit(f"version {version} not found in Mojang's version manifest")
    version_json = fetch(entry["url"], WORK_DIR / version / f"{version}.json",
                         sha1=entry["sha1"], force=force)
    return json.loads(version_json.read_text())


def class_names_in_jar(jar: Path) -> set[str]:
    with zipfile.ZipFile(jar) as zf:
        return {name[:-len(".class")] for name in zf.namelist()
                if name.endswith(".class")}


def prove(version_dir: Path, named_jar: Path, deps_dir: Path, java: str) -> None:
    print("proof: decompiling a sample of classes with Vineflower 1.12.0")
    present = class_names_in_jar(named_jar)
    sample = [c for c in PROOF_CLASSES if c in present]
    if not sample:
        raise SystemExit("proof failed: none of the expected named classes are in the output jar")
    sample_jar = version_dir / "sample.jar"
    with zipfile.ZipFile(named_jar) as zin, \
            zipfile.ZipFile(sample_jar, "w", zipfile.ZIP_DEFLATED) as zout:
        for cls in sample:
            zout.writestr(cls + ".class", zin.read(cls + ".class"))
    proof_dir = version_dir / "proof"
    shutil.rmtree(proof_dir, ignore_errors=True)
    proof_dir.mkdir(parents=True)
    subprocess.run([java, "-jar", str(deps_dir / "vineflower-1.12.0.jar"),
                    str(sample_jar), str(proof_dir)],
                   check=True, capture_output=True, text=True)
    for cls in sample:
        java_file = proof_dir / (cls + ".java")
        status = "decompiled" if java_file.exists() else "MISSING"
        print(f"  {status}: {cls}.java")
    level_java = proof_dir / "net/minecraft/world/level/Level.java"
    if level_java.exists():
        head = [ln for ln in level_java.read_text(errors="replace").splitlines()
                if "class Level" in ln or ln.startswith("package")][:3]
        print("  evidence from Level.java:")
        for ln in head:
            print(f"    {ln}")
    print(f"  full proof output (not committed): {proof_dir}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("version", help="Minecraft version id, e.g. 1.21.11")
    parser.add_argument("--prove", action="store_true",
                        help="also decompile a sample of classes with Vineflower")
    parser.add_argument("--force", action="store_true",
                        help="refetch downloads even when cached copies verify")
    args = parser.parse_args()

    version_dir = WORK_DIR / args.version
    version_dir.mkdir(parents=True, exist_ok=True)

    print(f"== mapping front-end: {args.version} ==")
    print("resolving version through Mojang's manifest:")
    info = resolve_version(args.version, args.force)
    downloads = info["downloads"]
    if "client_mappings" not in downloads:
        raise SystemExit(
            f"{args.version} publishes no official client mappings, so this "
            "front-end cannot remap it. Supported today: Mojang's official "
            "ProGuard tables (releases 1.14.4-1.21.11). Unobfuscated versions "
            "(26.x) need no remapping; older versions await SRG/TSRG support.")

    print("downloading client jar and official mappings:")
    client_jar = fetch(downloads["client"]["url"], version_dir / "client.jar",
                       sha1=downloads["client"]["sha1"], force=args.force)
    mappings = fetch(downloads["client_mappings"]["url"],
                     version_dir / "client-mappings.txt",
                     sha1=downloads["client_mappings"]["sha1"], force=args.force)

    deps_dir = ensure_deps(args.force)
    javac = find_tool("JAVAC", "javac")
    java = find_tool("JAVA", "java")
    classes_dir = ensure_driver_compiled(deps_dir, javac)

    named_jar = version_dir / "client-named.jar"
    stats_file = version_dir / "mapped-classes.tsv"
    cp = os.pathsep.join([str(classes_dir)]
                         + [str(deps_dir / n) for n in REMAPPER_DEPS])
    print("remapping obfuscated -> named:")
    subprocess.run([java, "-cp", cp, "Remap", str(mappings), str(client_jar),
                    str(named_jar), str(stats_file)], check=True)

    jar_classes = class_names_in_jar(client_jar)
    pairs = [ln.split("\t") for ln in stats_file.read_text().splitlines() if "\t" in ln]
    mapped = {f for f, _ in pairs}
    renamed = sum(1 for f, t in pairs if f != t)
    covered = jar_classes & mapped
    print("coverage:")
    print(f"  class files in client jar : {len(jar_classes)}")
    print(f"  classes in mapping table  : {len(pairs)}")
    print(f"  jar classes with a mapping: {len(covered)} "
          f"({100.0 * len(covered) / len(jar_classes):.2f}%)")
    print(f"  classes actually renamed  : {renamed}")
    print(f"  named jar                 : {named_jar} ({named_jar.stat().st_size} bytes)")

    if args.prove:
        prove(version_dir, named_jar, deps_dir, java)
    return 0


if __name__ == "__main__":
    sys.exit(main())
