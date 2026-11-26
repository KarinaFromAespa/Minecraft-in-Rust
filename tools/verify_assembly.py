#!/usr/bin/env python3
"""Verify byte-identical assembly for every version (restructure gate).

For each version, assembles commons + overlay chain + metadata into a
scratch directory (one version at a time; scratch is deleted after the
diff) and compares the result against the published tree's tracked
files (git ls-tree). The comparison is on (git-mode, git-blob-sha) per
path, so content, executable bit and symlinks are all covered.

Usage:
  verify_assembly.py [--version V] [--repo DIR] [--keep] [--base DIR]

Exit code 0 iff every checked version assembles byte-identically.
--keep leaves failed scratch dirs in place and prints their paths.
"""
import argparse
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import assemble


def _published_map(repo, ref, version):
    return assemble._git_tree_map(repo, ref, f"versions/{version}")


def verify_one(version, repo, ref="HEAD", base=None, keep=False):
    """Assemble version into scratch and diff vs its published tree."""
    scratch = tempfile.mkdtemp(prefix=f"asm-{version}-")
    try:
        assemble.assemble(version, scratch, repo, base, force=True)
        actual = assemble._walk_map(scratch, apply_ignores=False)
        expected = _published_map(repo, ref, version)
        problems = []
        for rel in sorted(set(expected) | set(actual)):
            if rel not in expected:
                problems.append(f"EXTRA in assembled tree: {rel}")
            elif rel not in actual:
                problems.append(f"MISSING from assembled tree: {rel}")
            elif expected[rel] != actual[rel]:
                problems.append(
                    f"DIFFERS: {rel} "
                    f"(published {expected[rel]}, assembled {actual[rel]})")
        return problems, scratch
    finally:
        if keep:
            print(f"  scratch kept at {scratch}", file=sys.stderr)
        else:
            shutil.rmtree(scratch, ignore_errors=True)


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--version", help="verify one version only")
    p.add_argument("--repo", help="repo root (default: this script's repo)")
    p.add_argument("--ref", default="HEAD",
                   help="ref of the published trees")
    p.add_argument("--base", help="base tree directory")
    p.add_argument("--keep", action="store_true",
                   help="keep scratch dirs (for debugging failures)")
    args = p.parse_args(argv)
    repo = assemble.repo_root(args.repo)
    chains = assemble.load_chains(repo)["chains"]
    versions = [args.version] if args.version else sorted(chains)
    failed = 0
    for version in versions:
        problems, _scratch = verify_one(
            version, repo, args.ref, args.base, args.keep)
        if problems:
            failed += 1
            print(f"{version}: FAIL ({len(problems)} mismatches)")
            for problem in problems[:20]:
                print(f"  {problem}")
            if len(problems) > 20:
                print(f"  ... and {len(problems) - 20} more")
        else:
            print(f"{version}: OK (byte-identical)")
    print(f"{len(versions) - failed}/{len(versions)} versions "
          f"assemble byte-identically")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
