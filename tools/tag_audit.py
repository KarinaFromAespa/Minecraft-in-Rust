#!/usr/bin/env python3
"""tag_audit.py — every tag literal in Rust must exist in the pinned version's prepared tag files.

Three of the four real divergences found in the 26.2 behaviour port were the
same bug wearing different hats: a tag that exists only in 26.3's data
(`#nether_portal_frame`, `#speeds_up_zombie_villager_curing`), so the Rust
lookup is always false and the behaviour silently never fires. That class is
mechanically findable, so it is checked here, beside D1 (data-versus-code
consistency is what D1 is for). It catches the dead-tag class only — not a
missing gate (item 371) or a wrong default (item 402). Partial net, not a
replacement for the ledger.

Design (three decisions; they decide whether this is useful or noise):

1. UNION ACROSS REGISTRIES. A tag name is dead only if it is absent from
   EVERY prepared tag file (blocks, items, fluids, entity types, damage,
   biomes, enchantments, ...). Per-registry comparison produces false
   positives, since one name can live in several registries.
2. ONLY TAG POSITIONS COUNT. `"#minecraft:x"` string literals are
   unambiguous; bare `tag("...")` / `is_tag("...")` / `mark_tag("...")` /
   `has_tag("...")` call sites are tags, as are the same family's
   `block_tag("...")` / `biome_tag("...")` and lookups into a map literally
   named `*_tags` / `tags` (`.get("...")`). A plain `named.obsidian` or
   `is_block(...)` is a block name, and sweeping every `minecraft:` literal
   would bury the real hits. Two refinements found by surveying the tree:
   a `tag("...")` literal must carry a namespace (the same name is also a
   registry-first lookup, `tag("item", name)`, and an NBT/SNBT helper, so a
   bare first argument is not a tag); a `#...` literal must carry a
   namespace too (model JSON uses `#side` texture references and `#80FF0000`
   colours). Literals in a namespace the prepared data does not carry
   (mod tags such as `#horizon:mace`) are out of scope, not dead.
3. BLOCK WHEN THE TAG DATA IS ABSENT, DON'T PASS. Missing prepared tag
   files — or a required registry (block, item, fluid, entity_type,
   damage_type) with no tags at all — blocks the check (exit 3), never a
   silent pass. A check that quietly no-ops when prepare didn't run is the
   failure mode D1 exists to avoid.

Test context: a dead tag inside a `#[cfg(test)]` module, a `#[test]` fn, a
`tests/` or `examples/` path, or a `*_test*.rs` / `*_fixture.rs` file is
reported separately and does not fail the audit — fixtures deliberately use
fake tags (e.g. `#minecraft:no_such_tag` pins unknown-tag handling). Only
dead tags in production code fail.

    python3 tools/tag_audit.py --tree versions/26.2
    python3 tools/tag_audit.py --tree versions/26.2 --prepared <dir> --json

Exit: 0 no dead tags in production code · 1 dead tag(s) in production code
· 3 blocked (tag data missing or incomplete) · 2 usage error.
Stdlib only; no imports from any version tree, so it runs from CI or a
scratch checkout.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

# Registries that must each contribute at least one tag, or the inventory is
# incomplete and the check blocks instead of risking false positives.
REQUIRED_REGISTRIES = ("block", "item", "fluid", "entity_type", "damage_type")

# Call sites whose first string argument is a tag name. `tag` is ambiguous
# (registry-first `tag("item", name)`, NBT `tag("{...}")`), so its literals
# must carry a namespace; the rest take the tag as their first argument by
# construction, so a bare name there is a real (if namespace-less) reference.
TAG_CALLS = ("tag", "is_tag", "has_tag", "mark_tag", "block_tag", "biome_tag")
NAMESPACE_REQUIRED = frozenset({"tag"})

NAMESPACE_RE = re.compile(r"^[a-z0-9._-]+$")
PATH_RE = re.compile(r"^[a-z0-9._/-]+$")

# A file is test context when any path part is tests/ or examples/, or its
# name marks it a test or a harness fixture.
TEST_FILE_RE = re.compile(r"(^|_)tests?(_|\.)|_fixture\.rs$")
TEST_PATH_PARTS = frozenset({"tests", "examples"})

# ---------------------------------------------------------------- inventory


def _tag_name_from_file(tags_dir: Path, path: Path, namespace: str):
    """(registry, tag id) for one file under data/<ns>/tags/, or None.

    The registry is the first path component — except `worldgen`, whose
    registries are two components deep (`worldgen/biome`, ...). The tag
    name is everything below the registry, `/` included, so
    `block/mineable/pickaxe.json` is `minecraft:mineable/pickaxe` and
    `worldgen/biome/has_structure/ancient_city.json` is
    `minecraft:has_structure/ancient_city`.
    """
    rel = path.relative_to(tags_dir)
    parts = list(rel.parts[:-1]) + [rel.stem]
    if not parts:
        return None
    if parts[0] == "worldgen":
        if len(parts) < 3:
            return None
        registry, name_parts = "worldgen/" + parts[1], parts[2:]
    else:
        registry, name_parts = parts[0], parts[1:]
    if not name_parts:
        return None
    return registry, f"{namespace}:" + "/".join(name_parts)


def load_tag_union(prepared: Path):
    """The prepared tag inventory: union of tag ids, per-registry counts.

    Sources, in order: `data/<ns>/tags/<registry>/*.json` (the canonical
    prepared tag files) and top-level `<registry>-tags.json` exports
    (`block-tags.json`, `damage-tags.json`), whose keys are tag ids. Both
    carry the same data in practice; the union makes either sufficient.
    Returns (union, per_registry, sources, namespaces).
    """
    union: set[str] = set()
    per_registry: dict[str, set[str]] = {}
    sources: dict[str, int] = {}
    namespaces: set[str] = set()

    def add(tag_id: str, registry: str):
        union.add(tag_id)
        per_registry.setdefault(registry, set()).add(tag_id)

    data_dir = prepared / "data"
    if data_dir.is_dir():
        for ns_dir in sorted(data_dir.iterdir()):
            if not ns_dir.is_dir():
                continue
            tags_dir = ns_dir / "tags"
            if not tags_dir.is_dir():
                continue
            namespaces.add(ns_dir.name)
            count = 0
            for path in sorted(tags_dir.rglob("*.json")):
                parsed = _tag_name_from_file(tags_dir, path, ns_dir.name)
                if parsed is None:
                    continue
                registry, tag_id = parsed
                add(tag_id, registry)
                count += 1
            if count:
                sources[f"data/{ns_dir.name}/tags"] = count
    for path in sorted(prepared.glob("*-tags.json")):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if not isinstance(data, dict):
            continue
        registry = path.name[: -len("-tags.json")]
        if registry == "damage":
            registry = "damage_type"
        count = 0
        for tag_id in data:
            if isinstance(tag_id, str) and ":" in tag_id:
                namespaces.add(tag_id.split(":", 1)[0])
                add(tag_id, registry)
                count += 1
        if count:
            sources[path.name] = count
    return union, {r: len(s) for r, s in per_registry.items()}, sources, sorted(namespaces)


# ---------------------------------------------------------------- Rust scan

# One regex drives the scan; gaps between matches (whitespace, operators,
# numbers) carry no information. Alternation order matters: comment and
# string openers must win over the ident that shares their first character
# (`r#"` is a raw string, not ident `r` + `#` + `"`).
_SCAN_RE = re.compile(
    r"//[^\n]*"  # line comment
    r"|/\*"  # block comment open (nesting handled by hand)
    r"|(?:br?|r)#*\""  # raw / byte-raw string opener
    r"|b?\""  # normal / byte string opener
    r"|'(?:\\.|[^'\\\n])'"  # char literal
    r"|'[A-Za-z_][A-Za-z0-9_]*"  # lifetime
    r"|(?P<ident>[A-Za-z_][A-Za-z0-9_]*)"
    r"|(?P<punct>[#\[\]{}().,])"
)


def _scan_string(text: str, pos: int):
    """(value, end) for the string starting at text[pos] ('"' or raw opener)."""
    if text[pos] == "r" or (text[pos] == "b" and text[pos + 1:pos + 2] == "r"):
        hashes = 0
        i = pos + 1
        if text[pos] == "b":
            i += 1
        while text[i] == "#":
            hashes += 1
            i += 1
        # text[i] is the opening quote.
        end = text.find('"' + "#" * hashes, i + 1)
        if end == -1:
            return text[i + 1:], len(text)
        return text[i + 1:end], end + 1 + hashes
    # Normal or byte string: escapes hide the closing quote.
    i = pos + 1 if text[pos] == "b" else pos
    i += 1  # opening quote
    out = []
    while i < len(text):
        ch = text[i]
        if ch == "\\":
            out.append(text[i:i + 2])
            i += 2
        elif ch == '"':
            return "".join(out), i + 1
        elif ch == "\n":
            return "".join(out), i  # unterminated; treat as closed
        else:
            out.append(ch)
            i += 1
    return "".join(out), len(text)


def tokenize_rust(text: str):
    """[(kind, value, line)] for idents, strings and structural punctuation.

    Comments are dropped (the two known dead tags now survive only in
    comments documenting their fixes — scanning them would flag the fix
    itself). Block comments nest in Rust, so they are skipped by hand.
    """
    tokens = []
    pos = 0
    line = 1
    end = len(text)
    while pos < end:
        match = _SCAN_RE.search(text, pos)
        if not match:
            break
        line += text.count("\n", pos, match.start())
        pos = match.start()
        found = match.group(0)
        if found.startswith("//"):
            pos = match.end()
        elif found == "/*":
            depth, i = 1, pos + 2
            while depth and i < end:
                open_at = text.find("/*", i)
                close_at = text.find("*/", i)
                if close_at == -1:
                    i = end
                    break
                if open_at != -1 and open_at < close_at:
                    depth += 1
                    i = open_at + 2
                else:
                    depth -= 1
                    i = close_at + 2
            line += text.count("\n", pos, i)
            pos = i
        elif found.endswith('"'):
            value, new_pos = _scan_string(text, pos)
            line += text.count("\n", pos, new_pos)
            tokens.append(("string", value, line - text.count("\n", pos, new_pos)))
            pos = new_pos
        elif found.startswith("'"):
            pos = match.end()
        elif match.group("ident") is not None:
            tokens.append(("ident", found, line))
            pos = match.end()
        else:
            tokens.append(("punct", found, line))
            pos = match.end()
    return tokens


def _test_ranges(tokens):
    """[(start_line, end_line)] of #[cfg(test)] / #[test] items.

    An attribute marks the next `{` as a test scope; a `;` first means a
    bodiless item (`mod foo;`) and clears the mark. Scopes close when the
    brace depth falls back below their opening depth.
    """
    ranges = []
    depth = 0
    pending = False
    stack: list[tuple[int, int]] = []  # (opening depth, start line)
    i = 0
    while i < len(tokens):
        kind, value, tok_line = tokens[i]
        if kind == "punct" and value == "#" and i + 1 < len(tokens) \
                and tokens[i + 1] == ("punct", "[", tok_line):
            # Read to the closing bracket; a `test` ident inside a cfg()
            # or a bare `test` marks the item.
            j, nested, idents = i + 2, 0, []
            while j < len(tokens):
                k2, v2, _ = tokens[j]
                if k2 == "punct" and v2 == "[":
                    nested += 1
                elif k2 == "punct" and v2 == "]":
                    if nested == 0:
                        break
                    nested -= 1
                elif k2 == "ident":
                    idents.append(v2)
                j += 1
            if "test" in idents and idents and idents[0] in ("cfg", "test"):
                pending = True
            i = j
        elif kind == "punct" and value == "{":
            depth += 1
            if pending:
                stack.append((depth, tok_line))
                pending = False
        elif kind == "punct" and value == "}":
            while stack and stack[-1][0] > depth - 1:
                _, start = stack.pop()
                ranges.append((start, tok_line))
            depth -= 1
        elif kind == "punct" and value == ";" and pending:
            pending = False
        i += 1
    return ranges


def _normalize(literal: str, namespace_required: bool, namespaces):
    """Canonical `ns:path` for a tag literal, or (None, reason)."""
    text = literal[1:] if literal.startswith("#") else literal
    if ":" in text:
        namespace, path = text.split(":", 1)
    elif namespace_required:
        return None, "no namespace"
    else:
        namespace, path = "minecraft", text
    if not NAMESPACE_RE.match(namespace) or not PATH_RE.match(path):
        return None, "not a resource location"
    if path.startswith("/") or path.endswith("/") or "//" in path:
        return None, "not a tag name"
    if namespace not in namespaces:
        return None, f"namespace {namespace} not in prepared data"
    return f"{namespace}:{path}", None


def scan_file(path: Path, rel: str):
    """Raw hits for one Rust file: (literal, line, test, form, ns_required).

    Skipped literals (found in a tag position but failing validation) are
    decided in audit(), where the prepared namespaces are known.
    """
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    is_test_file = (
        any(part in TEST_PATH_PARTS for part in path.parts)
        or TEST_FILE_RE.search(path.name) is not None
    )
    tokens = tokenize_rust(text)
    ranges = [] if is_test_file else _test_ranges(tokens)

    def in_test(tok_line: int) -> bool:
        return is_test_file or any(a <= tok_line <= b for a, b in ranges)

    hits = []
    for i, (kind, value, tok_line) in enumerate(tokens):
        if kind != "string":
            continue
        form = None
        namespace_required = False
        if value.startswith("#"):
            form = "#literal"
            namespace_required = True
        elif i >= 2 and tokens[i - 1][0] == "punct" and tokens[i - 1][1] == "(" \
                and tokens[i - 2][0] == "ident" and tokens[i - 2][1] in TAG_CALLS:
            form = tokens[i - 2][1] + "()"
            namespace_required = tokens[i - 2][1] in NAMESPACE_REQUIRED
        elif i >= 4 and tokens[i - 1][0] == "punct" and tokens[i - 1][1] == "(" \
                and tokens[i - 2][0] == "ident" and tokens[i - 2][1] == "get" \
                and tokens[i - 3][0] == "punct" and tokens[i - 3][1] == "." \
                and tokens[i - 4][0] == "ident" \
                and (tokens[i - 4][1] == "tags" or tokens[i - 4][1].endswith("_tags")):
            form = tokens[i - 4][1] + ".get()"
        if form is None:
            continue
        hits.append((value, tok_line, in_test(tok_line), form, namespace_required))
    return hits


def scan_tree(tree: Path):
    """(raw_hits, files_scanned) for every .rs file under crates/ + launcher/."""
    raw_hits = []
    files = 0
    for root in (tree / "crates", tree / "launcher"):
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*.rs")):
            files += 1
            rel = str(path.relative_to(tree))
            for value, tok_line, test, form, ns_required in scan_file(path, rel):
                raw_hits.append((value, rel, tok_line, test, form, ns_required))
    return raw_hits, files


# ---------------------------------------------------------------- audit


def audit(tree: Path, prepared: Path):
    """Full audit. Returns a result dict with status pass/fail/blocked."""
    union, per_registry, sources, namespaces = load_tag_union(prepared)
    inventory = {
        "total_tags": len(union),
        "namespaces": namespaces,
        "per_registry": dict(sorted(per_registry.items())),
        "sources": dict(sorted(sources.items())),
    }
    if not union:
        return {"status": "blocked",
                "reason": f"no prepared tag data at {prepared}",
                "inventory": inventory}
    missing = [r for r in REQUIRED_REGISTRIES if per_registry.get(r, 0) == 0]
    if missing:
        return {"status": "blocked",
                "reason": "prepared tag data lacks registry(ies): " + ", ".join(missing),
                "inventory": inventory}
    raw_hits, files = scan_tree(tree)
    literals, skipped = [], []
    for value, rel, tok_line, test, form, ns_required in raw_hits:
        tag_id, reason = _normalize(value, ns_required, namespaces)
        if tag_id is None:
            skipped.append({"file": rel, "line": tok_line, "literal": value,
                            "form": form, "reason": reason})
        else:
            literals.append({"tag": tag_id, "file": rel, "line": tok_line,
                             "test": test, "form": form})
    by_tag: dict[str, list[dict]] = {}
    for lit in literals:
        by_tag.setdefault(lit["tag"], []).append(lit)
    dead, dead_test_only = [], []
    for tag_id in sorted(by_tag):
        if tag_id in union:
            continue
        locations = by_tag[tag_id]
        entry = {"tag": tag_id, "locations": locations,
                 "production": any(not loc["test"] for loc in locations)}
        (dead if entry["production"] else dead_test_only).append(entry)
    return {
        "status": "fail" if dead else "pass",
        "inventory": inventory,
        "scanned": {"files": files, "literals": len(literals),
                    "distinct_tags": len(by_tag), "skipped": len(skipped)},
        "dead": dead,
        "dead_test_only": dead_test_only,
        "skipped": skipped,
    }


def resolve_prepared(tree: Path, release: str | None, given: str | None) -> Path | None:
    if given:
        return Path(given)
    base = tree / ".local" / "horizon-client"
    if release:
        candidate = base / release
        return candidate if candidate.is_dir() else None
    if base.is_dir():
        subdirs = [d for d in base.iterdir() if d.is_dir()]
        if len(subdirs) == 1:
            return subdirs[0]
    return None


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description="Check every tag literal in a version tree against its prepared tag files.")
    parser.add_argument("--tree", required=True, help="the release's tree (e.g. versions/26.2)")
    parser.add_argument("--release", help="release id, to find .local/horizon-client/<v>/")
    parser.add_argument("--prepared", help="prepared data dir (default: derived from --tree)")
    parser.add_argument("--json", action="store_true", help="print the full result as JSON")
    parser.add_argument("--verbose", action="store_true", help="also list skipped literals")
    args = parser.parse_args(argv)

    tree = Path(args.tree)
    prepared = resolve_prepared(tree, args.release, args.prepared)
    if prepared is None:
        print(f"tag_audit: blocked: no prepared data under {tree}/.local/horizon-client/",
              file=sys.stderr)
        return 3
    result = audit(tree, prepared)
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        inv = result["inventory"]
        print(f"tags: {inv['total_tags']} across {len(inv['per_registry'])} registries "
              f"({', '.join(f'{k} {v}' for k, v in inv['sources'].items())})")
        if result["status"] == "blocked":
            print(f"blocked: {result['reason']}")
        else:
            sc = result["scanned"]
            print(f"scanned: {sc['literals']} literals ({sc['distinct_tags']} distinct) "
                  f"in {sc['files']} files; {sc['skipped']} skipped")
            for entry in result["dead"]:
                locs = ", ".join(f"{loc['file']}:{loc['line']}" for loc in entry["locations"][:3])
                print(f"DEAD (production): {entry['tag']} at {locs}")
            for entry in result["dead_test_only"]:
                locs = ", ".join(f"{loc['file']}:{loc['line']}" for loc in entry["locations"][:3])
                print(f"dead (test only): {entry['tag']} at {locs}")
            if args.verbose:
                for skip in result["skipped"]:
                    print(f"skipped: {skip['file']}:{skip['line']} {skip['literal']!r} "
                          f"({skip['form']}, {skip['reason']})")
            print(f"tag_audit: {result['status']}")
    return {"pass": 0, "fail": 1, "blocked": 3}[result["status"]]


if __name__ == "__main__":
    sys.exit(main())
