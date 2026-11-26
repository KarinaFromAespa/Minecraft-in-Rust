#!/usr/bin/env python3
"""Assemble a full version tree from commons + overlay chain.

Implements docs/restructure.md (commons + thin per-version overlays).

Layout (post-cutover; pre-cutover the base is versions/26.3):
  commons/                 One full copy of the upstream base tree (26.3).
  versions/<id>/
    overlay/               Only the files that differ from the assembled
                           parent chain, at tree-relative paths, plus a
                           `.deletions` manifest (one tree-relative path
                           per line) for files the version removes.
    research-lock.json, PORT-NOTES-<id>.md, status.json, README.md,
    AGENTS.md, ROADMAP.md, work-items.json, rank*-progress.json,
    regen-determinism.json, harness-subset.txt, tier.txt, diff-*/
                           Per-version metadata: NOT part of the overlay.
                           Assembly copies them from versions/<id>/ last,
                           so the assembled tree is byte-identical to the
                           published tree.

Commands:
  assemble <version> --into DIR [--base DIR] [--repo DIR] [--force]
      Build the full tree for <version> in DIR.
  assemble --extract <version> --from DIR [--repo DIR] [--dry-run]
      Diff an assembled (possibly edited) tree back into the overlay and
      the version's metadata files. The inverse of assembly.
  assemble --extract-all [--repo DIR] [--ref REF]
      (Re)generate every version's overlay/ from the committed trees,
      using git plumbing. Used for the migration itself.
  assemble --chain <version> [--repo DIR]
      Print the overlay chain for a version.
  assemble --list [--repo DIR]
      Print every version and its chain.

The base is resolved as: --base, else <repo>/commons if it exists, else
<repo>/versions/26.3 (the pre-cutover fallback, noted on stderr).

File enumeration uses `git ls-files --cached --others --exclude-standard`
when the source lives in a git checkout, so ignored build output (.local/,
target/, __pycache__/, *.jar, runtime files) is never copied, while
tracked-but-ignored files (e.g. tools/jsr305-3.0.2.jar) are. Outside a
git checkout it falls back to a filesystem walk with the same ignore set.
"""
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys

DELETIONS = ".deletions"

# Per-version metadata: files that stay at versions/<id>/ and are copied
# into the assembled tree last (never part of an overlay). PORT-NOTES-*.md
# matches any version's notes file: the parent's notes must never be
# treated as a tree file the child deleted.
METADATA_EXACT = {
    "README.md", "AGENTS.md", "ROADMAP.md", "research-lock.json",
    "status.json", "work-items.json",
    "rank1-progress.json", "rank2-progress.json", "rank3-progress.json",
    "rank4-progress.json", "rank5-progress.json",
    "regen-determinism.json", "harness-subset.txt", "tier.txt",
}


def is_metadata(relpath):
    """True if relpath is per-version metadata, not tree content."""
    top = relpath.split("/", 1)[0]
    if top.startswith("diff-"):
        return True
    base = os.path.basename(relpath)
    if base.startswith("PORT-NOTES-") and base.endswith(".md"):
        return True
    return relpath in METADATA_EXACT


# Paths never copied into an assembled tree (build output / runtime state).
IGNORE_DIRS = {
    ".git", ".local", "target", "__pycache__", ".venv", ".cache",
    "decompiled", "mappings", "minecraft", "assets", "run", "logs",
    "dist", "saves", "backups", "screenshots", ".horizon-cache",
    "crash-reports", "debug", "exporters",
}
IGNORE_SUFFIXES = (".pyc", ".pyo", ".jar", ".class", ".jfr", ".hprof")
IGNORE_FILES = {
    "options.txt", "debug-profile.json", "horizon.txt",
    "horizon-graphics.txt", "horizon-controller.txt", "horizon-extras.txt",
    "horizon-player.txt", "servers.dat", ".DS_Store",
}


def _ignored(relpath):
    parts = relpath.replace(os.sep, "/").split("/")
    if any(p in IGNORE_DIRS for p in parts):
        return True
    name = parts[-1]
    if name in IGNORE_FILES:
        return True
    return name.endswith(IGNORE_SUFFIXES)


def git_blob_hash(data):
    """The git blob id for these bytes (also what ls-tree reports)."""
    return hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()


def git_mode_for(path):
    """Git mode string for a filesystem entry: 120000/100755/100644."""
    if os.path.islink(path):
        return "120000"
    if os.access(path, os.X_OK):
        return "100755"
    return "100644"


def repo_root(explicit=None):
    if explicit:
        return os.path.abspath(explicit)
    return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def load_chains(repo):
    with open(os.path.join(repo, "tools", "overlay-chains.json")) as f:
        return json.load(f)


def resolve_base(repo, explicit=None):
    if explicit:
        return os.path.abspath(explicit)
    commons = os.path.join(repo, "commons")
    if os.path.isdir(commons):
        return commons
    print("note: no commons/ yet; using versions/26.3 as the base "
          "(pre-cutover layout)", file=sys.stderr)
    return os.path.join(repo, "versions", "26.3")


def list_source_files(source_dir, repo):
    """Files to copy out of source_dir: [(relpath, abspath)].

    Inside a git checkout, git ls-files (cached + untracked, honouring
    .gitignore) is the exact set. Otherwise a filesystem walk with the
    ignore set above. Symlinks are listed, never followed.
    """
    source_dir = os.path.abspath(source_dir)
    rel_to_repo = os.path.relpath(source_dir, repo)
    try:
        out = subprocess.run(
            ["git", "-C", repo, "ls-files", "--cached", "--others",
             "--exclude-standard", "--", rel_to_repo],
            capture_output=True, text=True, check=True).stdout
        result = []
        for line in out.splitlines():
            if not line:
                continue
            abspath = os.path.join(repo, line)
            rel = os.path.relpath(abspath, source_dir)
            result.append((rel, abspath))
        return result
    except (subprocess.CalledProcessError, FileNotFoundError):
        result = []
        for dirpath, dirnames, filenames in os.walk(source_dir):
            dirnames[:] = [d for d in dirnames
                           if not _ignored(os.path.relpath(
                               os.path.join(dirpath, d), source_dir))]
            for name in filenames:
                abspath = os.path.join(dirpath, name)
                rel = os.path.relpath(abspath, source_dir)
                if not _ignored(rel):
                    result.append((rel, abspath))
        return result


def copy_entry(src, dst):
    """Copy one file/symlink, preserving the executable bit."""
    parent = os.path.dirname(dst)
    if parent:
        os.makedirs(parent, exist_ok=True)
    if os.path.lexists(dst):
        if os.path.isdir(dst) and not os.path.islink(dst):
            shutil.rmtree(dst)
        else:
            os.remove(dst)
    if os.path.islink(src):
        os.symlink(os.readlink(src), dst)
    else:
        shutil.copyfile(src, dst)
        st = os.stat(src)
        os.chmod(dst, st.st_mode & 0o777)


def apply_overlay(overlay_dir, dest):
    """Apply one overlay directory onto dest. Returns (files, deletions)."""
    files = 0
    deletions = []
    manifest = os.path.join(overlay_dir, DELETIONS)
    if os.path.isfile(manifest):
        with open(manifest) as f:
            deletions = [ln.strip() for ln in f if ln.strip()]
    for rel, abspath in list_source_files(overlay_dir, _repo_of(overlay_dir)):
        if rel == DELETIONS or is_metadata(rel):
            continue
        copy_entry(abspath, os.path.join(dest, rel))
        files += 1
    for rel in deletions:
        target = os.path.join(dest, rel)
        if os.path.lexists(target):
            if os.path.isdir(target) and not os.path.islink(target):
                shutil.rmtree(target)
            else:
                os.remove(target)
    return files, deletions


def _repo_of(path):
    """Best-effort repo root for a path inside a checkout (walk up to .git)."""
    cur = os.path.abspath(path)
    while True:
        if os.path.isdir(os.path.join(cur, ".git")) or \
                os.path.isfile(os.path.join(cur, ".git")):
            return cur
        parent = os.path.dirname(cur)
        if parent == cur:
            return os.path.abspath(path)
        cur = parent


def copy_metadata(version, repo, dest):
    """Copy the version's metadata files from versions/<v>/ into dest."""
    srcdir = os.path.join(repo, "versions", version)
    copied = []
    for name in sorted(os.listdir(srcdir)):
        if not is_metadata(name):
            continue
        src = os.path.join(srcdir, name)
        dst = os.path.join(dest, name)
        if os.path.isdir(src) and not os.path.islink(src):
            if os.path.lexists(dst):
                shutil.rmtree(dst)
            shutil.copytree(src, dst, symlinks=True,
                            ignore=shutil.ignore_patterns(
                                ".git", "__pycache__"))
            for dirpath, _, filenames in os.walk(dst):
                copied.extend(os.path.relpath(
                    os.path.join(dirpath, fn), dest) for fn in filenames)
        elif os.path.lexists(src):
            copy_entry(src, dst)
            copied.append(name)
    return copied


def assemble(version, into, repo, base=None, force=False):
    """Build the full tree for version into `into`. Returns a stats dict."""
    chains = load_chains(repo)
    if version not in chains["chains"]:
        raise SystemExit(f"unknown version: {version}")
    base = resolve_base(repo, base)
    if not os.path.isdir(base):
        raise SystemExit(f"base tree not found: {base}")
    into = os.path.abspath(into)
    if os.path.exists(into):
        if not force or not os.path.isdir(into):
            raise SystemExit(f"destination exists: {into} (use --force)")
    os.makedirs(into, exist_ok=True)
    stats = {"base": base, "layers": [], "metadata": []}
    base_files = [(rel, abspath)
                  for rel, abspath in list_source_files(base, repo)
                  if not is_metadata(rel)]
    for rel, abspath in base_files:
        copy_entry(abspath, os.path.join(into, rel))
    stats["base_files"] = len(base_files)
    for layer in chains["chains"][version]:
        overlay_dir = os.path.join(repo, "versions", layer, "overlay")
        if not os.path.isdir(overlay_dir):
            raise SystemExit(f"missing overlay: {overlay_dir}")
        n_files, deletions = apply_overlay(overlay_dir, into)
        stats["layers"].append(
            {"layer": layer, "files": n_files, "deletions": len(deletions)})
    stats["metadata"] = copy_metadata(version, repo, into)
    return stats


def _walk_map(root, apply_ignores=True):
    """Map of relpath -> (git-mode, blob-sha) for a directory on disk.

    With apply_ignores (extract), build output and runtime state are
    skipped. Without it (verification), every file present is compared,
    including tracked-but-ignored files such as tools/jsr305-3.0.2.jar.
    """
    result = {}
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames
                       if not apply_ignores or not _ignored(os.path.relpath(
                           os.path.join(dirpath, d), root))]
        for name in filenames:
            abspath = os.path.join(dirpath, name)
            rel = os.path.relpath(abspath, root)
            if apply_ignores and _ignored(rel):
                continue
            if os.path.islink(abspath):
                data = os.readlink(abspath).encode()
            else:
                with open(abspath, "rb") as f:
                    data = f.read()
            result[rel] = (git_mode_for(abspath), git_blob_hash(data))
    return result


def _git_tree_map(repo, ref, subdir):
    """Map of relpath -> (mode, blob-sha) for a committed directory."""
    out = subprocess.run(
        ["git", "-C", repo, "ls-tree", "-r", ref, "--", subdir],
        capture_output=True, text=True, check=True).stdout
    result = {}
    prefix = subdir.rstrip("/") + "/"
    for line in out.splitlines():
        meta, path = line.split("\t", 1)
        mode, _typ, sha = meta.split()
        result[path[len(prefix):]] = (mode, sha)
    return result


def _disk_entry(abspath):
    if os.path.islink(abspath):
        return ("120000", git_blob_hash(os.readlink(abspath).encode()))
    with open(abspath, "rb") as f:
        data = f.read()
    return (git_mode_for(abspath), git_blob_hash(data))


def _same_file(a, b):
    if not os.path.lexists(a):
        return False
    if os.path.islink(a) or os.path.islink(b):
        return os.path.islink(a) and os.path.islink(b) and \
            os.readlink(a) == os.readlink(b)
    if os.path.isdir(a) or os.path.isdir(b):
        return False
    with open(a, "rb") as f:
        da = f.read()
    with open(b, "rb") as f:
        db = f.read()
    return da == db and (os.stat(a).st_mode & 0o111) == \
        (os.stat(b).st_mode & 0o111)


def _apply_overlay_to_map(base_map, overlay_dir):
    result = dict(base_map)
    manifest = os.path.join(overlay_dir, DELETIONS)
    if os.path.isfile(manifest):
        with open(manifest) as f:
            for ln in f:
                rel = ln.strip()
                if rel:
                    result.pop(rel, None)
    if os.path.isdir(overlay_dir):
        for rel, abspath in list_source_files(
                overlay_dir, _repo_of(overlay_dir)):
            if rel == DELETIONS or is_metadata(rel):
                continue
            result[rel] = _disk_entry(abspath)
    return result


def _parent_map(repo, version):
    """What assembly without this version's own overlay would produce."""
    chains = load_chains(repo)["chains"]
    base = resolve_base(repo)
    result = {}
    for rel, abspath in list_source_files(base, repo):
        if not is_metadata(rel):
            result[rel] = _disk_entry(abspath)
    for layer in chains[version][:-1]:
        result = _apply_overlay_to_map(
            result, os.path.join(repo, "versions", layer, "overlay"))
    return result


def extract(version, from_dir, repo, dry_run=False):
    """Diff an assembled tree back into overlay/ + metadata files."""
    chains = load_chains(repo)
    if version not in chains["chains"]:
        raise SystemExit(f"unknown version: {version}")
    from_dir = os.path.abspath(from_dir)
    if not os.path.isdir(from_dir):
        raise SystemExit(f"no such directory: {from_dir}")
    parent_map = _parent_map(repo, version)
    actual = _walk_map(from_dir)
    overlay_dir = os.path.join(repo, "versions", version, "overlay")
    changes = {"overlay_write": [], "overlay_delete": [],
               "metadata_write": [], "metadata_delete": [],
               "deletions_add": []}

    for rel, entry in sorted(actual.items()):
        if is_metadata(rel):
            target = os.path.join(repo, "versions", version, rel)
            if not _same_file(target, os.path.join(from_dir, rel)):
                changes["metadata_write"].append(rel)
                if not dry_run:
                    parent = os.path.dirname(target)
                    if parent:
                        os.makedirs(parent, exist_ok=True)
                    copy_entry(os.path.join(from_dir, rel), target)
            continue
        if rel not in parent_map or parent_map[rel] != entry:
            changes["overlay_write"].append(rel)
            if not dry_run:
                copy_entry(os.path.join(from_dir, rel),
                           os.path.join(overlay_dir, rel))

    # Metadata present in versions/<v>/ but gone from the assembled tree.
    srcdir = os.path.join(repo, "versions", version)
    for name in sorted(os.listdir(srcdir)):
        if not is_metadata(name):
            continue
        src = os.path.join(srcdir, name)
        if os.path.isdir(src) and not os.path.islink(src):
            if not any(p == name or p.startswith(name + "/")
                       for p in actual):
                changes["metadata_delete"].append(name)
                if not dry_run:
                    shutil.rmtree(src)
        elif not _same_file(src, os.path.join(from_dir, name)):
            if name not in actual:
                changes["metadata_delete"].append(name)
                if not dry_run:
                    os.remove(src)

    # Parent-chain files absent from the assembled tree become deletions.
    changes["deletions_add"] = sorted(
        p for p in set(parent_map) - set(actual) if not is_metadata(p))
    if not dry_run:
        _prune_overlay(version, repo, parent_map, actual, changes)
    return changes


def _prune_overlay(version, repo, parent_map, actual, changes):
    overlay_dir = os.path.join(repo, "versions", version, "overlay")
    os.makedirs(overlay_dir, exist_ok=True)
    for rel, abspath in list_source_files(overlay_dir, repo):
        if rel == DELETIONS:
            continue
        if rel not in actual:
            # Gone from the assembled tree: drop the overlay copy (the
            # .deletions manifest, reconciled below, records it if the
            # parent chain has it).
            os.remove(abspath)
            changes["overlay_delete"].append(rel)
        elif parent_map.get(rel) == _disk_entry(abspath):
            # Now identical to the parent chain: the overlay copy is
            # redundant.
            os.remove(abspath)
            changes["overlay_delete"].append(rel)
    deletions = sorted(p for p in set(parent_map) - set(actual)
                       if not is_metadata(p))
    manifest = os.path.join(overlay_dir, DELETIONS)
    if deletions:
        with open(manifest, "w") as f:
            f.write("\n".join(deletions) + "\n")
    elif os.path.isfile(manifest):
        os.remove(manifest)
    changes["deletions_add"] = deletions
    for dirpath, dirnames, filenames in os.walk(overlay_dir, topdown=False):
        if dirpath != overlay_dir and not dirnames and not filenames:
            os.rmdir(dirpath)


def extract_all(repo, ref="HEAD"):
    """Regenerate every overlay/ from the committed trees (migration)."""
    chains = load_chains(repo)["chains"]
    order = ["26.2", "26.1", "26.1.1", "26.1.2", "1.21.11", "1.21.10",
             "1.21.9", "1.21.8", "1.21.7", "1.21.6", "1.21.5", "1.21.4",
             "1.21.3", "1.21.2", "1.21.1", "1.21", "1.20.6", "1.20.5"]
    base_map = {rel: entry for rel, entry in
                _git_tree_map(repo, ref, "versions/26.3").items()
                if not is_metadata(rel)}
    batch = subprocess.Popen(
        ["git", "-C", repo, "cat-file", "--batch"],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    report = {}
    try:
        for version in order:
            chain = chains[version]
            parent_map = dict(base_map)
            for layer in chain[:-1]:
                parent_map = _apply_overlay_to_map(
                    parent_map,
                    os.path.join(repo, "versions", layer, "overlay"))
            tree_map = _git_tree_map(repo, ref, f"versions/{version}")
            overlay_dir = os.path.join(repo, "versions", version, "overlay")
            if os.path.isdir(overlay_dir):
                shutil.rmtree(overlay_dir)
            os.makedirs(overlay_dir)
            written, deletions = [], []
            for rel in sorted(set(tree_map) - set(parent_map)):
                if is_metadata(rel):
                    continue
                written.append(rel)
                _write_blob(batch, tree_map[rel], overlay_dir, rel)
            for rel in sorted(set(parent_map) - set(tree_map)):
                if is_metadata(rel):
                    continue
                deletions.append(rel)
            for rel in sorted(set(tree_map) & set(parent_map)):
                if is_metadata(rel):
                    continue
                if tree_map[rel] != parent_map[rel]:
                    written.append(rel)
                    _write_blob(batch, tree_map[rel], overlay_dir, rel)
            if deletions:
                with open(os.path.join(overlay_dir, DELETIONS), "w") as f:
                    f.write("\n".join(deletions) + "\n")
            report[version] = {"files": len(written),
                               "deletions": len(deletions)}
    finally:
        batch.stdin.close()
        batch.wait()
    return report


def _write_blob(batch, entry, overlay_dir, rel):
    mode, sha = entry
    batch.stdin.write((sha + "\n").encode())
    batch.stdin.flush()
    header = batch.stdout.readline().decode().strip().split()
    size = int(header[2])
    data = batch.stdout.read(size)
    batch.stdout.read(1)  # trailing newline
    dst = os.path.join(overlay_dir, rel)
    parent = os.path.dirname(dst)
    if parent:
        os.makedirs(parent, exist_ok=True)
    if mode == "120000":
        os.symlink(data.decode(), dst)
    else:
        with open(dst, "wb") as f:
            f.write(data)
        if mode == "100755":
            os.chmod(dst, 0o755)


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("version", nargs="?", help="version to assemble")
    p.add_argument("--into", help="destination directory for assembly")
    p.add_argument("--from", dest="from_dir",
                   help="assembled tree to extract from")
    p.add_argument("--extract", action="store_true",
                   help="extract an assembled tree back into overlay/")
    p.add_argument("--extract-all", action="store_true",
                   help="regenerate every overlay/ from committed trees")
    p.add_argument("--chain", action="store_true",
                   help="print the overlay chain for the version")
    p.add_argument("--list", action="store_true", help="list all chains")
    p.add_argument("--base", help="base tree directory (default: commons/ "
                   "or versions/26.3)")
    p.add_argument("--repo", help="repo root (default: this script's repo)")
    p.add_argument("--ref", default="HEAD", help="ref for --extract-all")
    p.add_argument("--force", action="store_true",
                   help="assemble into an existing directory")
    p.add_argument("--dry-run", action="store_true",
                   help="report extract changes without writing")
    args = p.parse_args(argv)
    repo = repo_root(args.repo)

    if args.list:
        chains = load_chains(repo)["chains"]
        for v in sorted(chains):
            print(f"{v}: {' + '.join(chains[v]) or '(commons itself)'}")
        return 0
    if args.chain:
        chains = load_chains(repo)["chains"]
        if args.version not in chains:
            raise SystemExit(f"unknown version: {args.version}")
        if chains[args.version]:
            print("commons + " + " + ".join(chains[args.version]))
        else:
            print("commons (no overlay)")
        return 0
    if args.extract_all:
        report = extract_all(repo, args.ref)
        for v, r in report.items():
            print(f"{v}: overlay {r['files']} files, "
                  f"{r['deletions']} deletions")
        return 0
    if args.extract:
        if not args.version or not args.from_dir:
            p.error("--extract needs a version and --from DIR")
        changes = extract(args.version, args.from_dir, repo, args.dry_run)
        for key, items in changes.items():
            print(f"{key}: {len(items)}")
            for item in items[:20]:
                print(f"  {item}")
        return 0
    if not args.version or not args.into:
        p.error("assemble needs a version and --into DIR")
    stats = assemble(args.version, args.into, repo, args.base, args.force)
    print(f"assembled {args.version} from {stats['base']} "
          f"({stats['base_files']} files)")
    for layer in stats["layers"]:
        print(f"  + {layer['layer']}: {layer['files']} files, "
              f"{layer['deletions']} deletions")
    print(f"  + metadata: {len(stats['metadata'])} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
