#!/usr/bin/env python3
"""defect_suite.py — cross-version defect regression suite.

One shared suite that checks the DEFECT CLASSES found in 1.21.7 playtesting
against every version, so a failing check names the version and the defect.
The owner asked for it 2026-11-23 ("write test functions to check for such
issues, so that we can do test for other versions and know what is not
passing the test and fix those").

Each check is a function of (version, tree, jars): the expected values come
from EACH version's own vanilla jar/data, never hardcoded 1.21.7 values and
never the 26.3 base's form (the VANILLA-CHECK rule). Where a version's jar
is unavailable, the checks that need it report `unmeasured`, never pass.

Two kinds of check:

  data checks  (run everywhere, no compile) — compare what the version
               tree's code reads against what the version's own jar
               contains: fuel/brewing-fuel components, loot-table field
               names, advancement trigger keys, item special-model types,
               menu sprites, and (via the leakage audit) tags.
  rust checks  (registered in tools/defect_suite_rust.json) — the
               regression tests the fix agents wrote in the version
               crates. By default the suite only reports whether each
               test EXISTS in the tree (presence). With --crate-logs it
               reads the results out of a crate-tests run's logs, and
               with --run-rust it runs the tests itself with cargo test
               (opt-in: compiling horizon-client's test binary OOMs the
               VM — run those on CI or the laptop, never the VM).

Check ids (see docs/DEFECT-SUITE.md for the full list and how to add one):

  fuel-table        1.21.7 has no fuel component; fuel must come from
                    vanilla's FuelValues table (fixed a53b0c58).
  brewing-fuel      same class, tag-gated (fix in flight).
  loot-fields       loot tables name condition/function kinds and the
                    functions list differently per era (fixed e3bce30e).
  recipe-packets    recipe_book_add packets skipped as undecodable
                    (fix in flight; rust check, no test yet).
  advancement-keys  advancement triggers name their keys singular in
                    1.21.7-26.2, plural in 26.3 (fixed 59ba5f8d).
  bed-models        bed item models use a special model type the code
                    does not know (fix in flight) + rust checks.
  mob-models        entity models resolve from entity-models.json
                    (rust check, no test yet).
  undead-burn       zombies do not ignite promptly at sunrise
                    (rust check, no test yet).
  menu-sprites      every sprite the code references must exist in the
                    version's client jar or the prepare fallback
                    (fixed 20453af0).
  absent-tags       code reads a tag/attribute/gamerule the version's
                    data does not have (the leakage audit, baseline
                    ratchet — see docs/LEAKAGE-AUDIT.md).

Usage:

    python3 tools/defect_suite.py 1.21.7 --jar-dir jars/
    python3 tools/defect_suite.py 1.21.7 --jar jars/1.21.7-server.jar \
        --client-jar jars/1.21.7-client.jar --json
    python3 tools/defect_suite.py --all --jar-dir jars/ --report-only
    python3 tools/defect_suite.py 1.21.7 --jar-dir jars/ \
        --crate-logs crate-tests-45105507386/
    python3 tools/defect_suite.py 1.21.7 --jar-dir jars/ --run-rust \
        --checks fuel-table,loot-fields

Exit: 0 no failures · 1 one or more checks failed · 3 one or more
checks unmeasured (and none failed) · 2 usage error. --report-only
forces 0 (CI reports the defect list; it does not gate on it).

Stdlib only. The leakage-audit import is optional: without
tools/audit_26_3_leakage.py the absent-tags check reports `skip`.
"""

from __future__ import annotations

import argparse
import io
import json
import re
import subprocess
import sys
import zipfile
from collections import Counter, defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
try:
    import audit_26_3_leakage as leakage
except ImportError:  # pragma: no cover - the audit tool is a sibling
    leakage = None

VERSIONS_DIRNAME = "versions"
RUST_MANIFEST = "defect_suite_rust.json"
LEAKAGE_BASELINE = "leakage-baselines.json"
LEAKAGE_EXCEPTIONS = "leakage-exceptions.json"
FALLBACK_VERSION = "26.3"

# Statuses a check can report. `skip` means the check does not apply to
# this run (no client jar, no such crate); `unmeasured` means it applies
# but its oracle is unavailable — never a silent pass. `not-covered`
# means the rust tests for the class do not exist in this tree yet.
PASS, FAIL, SKIP, UNMEASURED, NOT_COVERED = (
    "pass", "fail", "skip", "unmeasured", "not-covered")


def _read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def _open_game_jar(jar_path: Path):
    """ZipFile for the game jar: the jar itself, or the bundler's inner jar."""
    if leakage is not None:
        return leakage._open_game_jar(jar_path)
    outer = zipfile.ZipFile(jar_path)
    names = outer.namelist()
    if any(n.startswith("data/minecraft/tags/") for n in names):
        return outer, "jar"
    inner = [n for n in names
             if n.startswith("META-INF/versions/") and n.endswith(".jar")]
    for candidate in inner:
        try:
            game = zipfile.ZipFile(io.BytesIO(outer.read(candidate)))
        except (zipfile.BadZipFile, KeyError):
            continue
        if any(n.startswith("data/minecraft/tags/") for n in game.namelist()):
            return game, f"jar:{candidate}"
    return outer, "jar"


class Ctx:
    """Everything a check needs: the version, its tree, its jars."""

    def __init__(self, repo: Path, version: str, server_jar=None,
                 client_jar=None, fallback_jar=None, jar_dir=None,
                 rust_manifest=None, crate_logs=None, run_rust=False,
                 leakage_baseline=None, leakage_exceptions=None):
        self.repo = repo
        self.version = version
        self.tree = repo / VERSIONS_DIRNAME / version
        self.jar_dir = Path(jar_dir) if jar_dir else None
        self.server_jar = self._resolve_jar(server_jar, "server")
        self.client_jar = self._resolve_jar(client_jar, "client")
        self.fallback_jar = self._resolve_jar(
            fallback_jar, "client", version=FALLBACK_VERSION)
        self.crate_logs = Path(crate_logs) if crate_logs else None
        self.run_rust = run_rust
        self.leakage_baseline = leakage_baseline
        self.leakage_exceptions = leakage_exceptions
        self.rust_manifest = self._load_rust_manifest(rust_manifest)
        self._server_game = None
        self._client_game = None
        self._fallback_game = None
        self._text_cache = {}
        self._test_ranges_cache = {}

    def _resolve_jar(self, explicit, kind, version=None):
        version = version or self.version
        if explicit:
            p = Path(explicit)
            return p if p.is_file() else None
        if self.jar_dir:
            for name in (f"{version}-{kind}.jar",
                         f"{version}-{kind}-named.jar"):
                p = self.jar_dir / name
                if p.is_file():
                    return p
        return None

    def _load_rust_manifest(self, path):
        if path:
            p = Path(path)
        else:
            p = Path(__file__).resolve().parent / RUST_MANIFEST
        if not p.is_file():
            return {}
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            return {}
        return data if isinstance(data, dict) else {}

    @property
    def server_game(self):
        """ZipFile of the version's server jar (bundler-aware), or None."""
        if self._server_game is None and self.server_jar is not None:
            try:
                self._server_game, _ = _open_game_jar(self.server_jar)
            except (zipfile.BadZipFile, OSError):
                self._server_game = False
        return self._server_game or None

    @property
    def client_game(self):
        if self._client_game is None and self.client_jar is not None:
            try:
                self._client_game, _ = _open_game_jar(self.client_jar)
            except (zipfile.BadZipFile, OSError):
                self._client_game = False
        return self._client_game or None

    @property
    def fallback_game(self):
        if self._fallback_game is None and self.fallback_jar is not None:
            try:
                self._fallback_game, _ = _open_game_jar(self.fallback_jar)
            except (zipfile.BadZipFile, OSError):
                self._fallback_game = False
        return self._fallback_game or None

    def is_test_line(self, rel: str, line: int) -> bool:
        """True when a tree-relative file:line is inside a test module.

        Per-file test ranges (the audit tool's tokenizer), cached —
        deliberately NOT the audit's TreeIndex, which loads the whole
        tree at construction; most checks need only a handful of files.
        """
        if rel not in self._test_ranges_cache:
            path = self.tree / rel
            if leakage is not None and leakage._is_test_file(path):
                self._test_ranges_cache[rel] = True
            elif leakage is not None:
                try:
                    ranges = leakage._test_ranges_for(_read(path))
                except Exception:
                    ranges = []
                self._test_ranges_cache[rel] = ranges
            else:
                self._test_ranges_cache[rel] = (
                    "_test" in rel or rel.endswith("_tests.rs"))
        cached = self._test_ranges_cache[rel]
        if cached is True:
            return True
        if isinstance(cached, bool):
            return cached
        return any(a <= line <= b for a, b in cached)

    def rs_files(self, *roots):
        """(rel, Path) for every .rs file under the given tree roots."""
        for root in roots:
            base = self.tree / root
            if not base.is_dir():
                continue
            for path in sorted(base.rglob("*.rs")):
                yield str(path.relative_to(self.tree)), path


def _result(check_id, title, status, detail, evidence=None, notes=None):
    return {"id": check_id, "title": title, "status": status,
            "detail": detail, "evidence": evidence or [],
            "notes": notes or []}


# --------------------------------------------------------------------------
# Jar oracles

def _class_string_hits(game, needle: bytes) -> int:
    """How many .class files in the jar contain the byte string."""
    hits = 0
    for name in game.namelist():
        if not name.endswith(".class"):
            continue
        try:
            data = game.read(name)
        except KeyError:
            continue
        if needle in data:
            hits += 1
    return hits


def _jar_jsons(game, prefix, suffix=".json"):
    """(path, parsed JSON) for every JSON file under a jar prefix."""
    for name in game.namelist():
        if not (name.startswith(prefix) and name.endswith(suffix)):
            continue
        try:
            yield name, json.loads(game.read(name))
        except (ValueError, KeyError):
            continue


def loot_census(game):
    """The loot-table field-name form this version's data uses.

    Returns {role: {"old": n, "new": n}} for the four roles:
      cond_container  the conditions holder: a `conditions` list (old,
                      <=26.2) vs a single `condition` object (new, 26.3)
      cond_kind       a condition object's kind key: `condition` (old)
                      vs `type` (new)
      func_kind       a function object's kind key: `function` (old)
                      vs `type` (new)
      list_key        the functions list key: `functions` (old) vs
                      `modifier` (new)
    Counting is context-aware: kind keys are counted only on condition
    / function objects (elements of `conditions`/`terms` lists, values
    of `condition`/`term` keys, elements of `functions`/`modifier`
    lists), so a `type` on an entry or a number provider never counts
    as a kind key, and a string-valued `condition` key inside a
    condition object counts as the old kind key, not the container.
    """
    counts = {"cond_container": Counter(), "cond_kind": Counter(),
              "func_kind": Counter(), "list_key": Counter()}
    tables = 0

    def kind_of(obj):
        """old / new / None for a condition/function object's kind key."""
        if not isinstance(obj, dict):
            return None
        if isinstance(obj.get("condition"), str) \
                or isinstance(obj.get("function"), str):
            return "old"
        if isinstance(obj.get("type"), str):
            return "new"
        return None

    for _path, table in _jar_jsons(game, "data/minecraft/loot_table/"):
        if not isinstance(table, dict):
            continue
        tables += 1

        def walk(node):
            if isinstance(node, list):
                for item in node:
                    walk(item)
                return
            if not isinstance(node, dict):
                return
            for key, value in node.items():
                if key == "conditions" and isinstance(value, list):
                    counts["cond_container"]["old"] += 1
                    for cond in value:
                        kind = kind_of(cond)
                        if kind:
                            counts["cond_kind"][kind] += 1
                        walk(cond)
                elif key == "condition" and isinstance(value, dict):
                    counts["cond_container"]["new"] += 1
                    kind = kind_of(value)
                    if kind:
                        counts["cond_kind"][kind] += 1
                    walk(value)
                elif key == "terms" and isinstance(value, list):
                    for cond in value:
                        kind = kind_of(cond)
                        if kind:
                            counts["cond_kind"][kind] += 1
                        walk(cond)
                elif key == "term" and isinstance(value, dict):
                    kind = kind_of(value)
                    if kind:
                        counts["cond_kind"][kind] += 1
                    walk(value)
                elif key == "functions" and isinstance(value, list):
                    counts["list_key"]["old"] += 1
                    for fn in value:
                        kind = kind_of(fn)
                        if kind:
                            counts["func_kind"][kind] += 1
                        walk(fn)
                elif key == "modifier" and isinstance(value, list):
                    counts["list_key"]["new"] += 1
                    for fn in value:
                        kind = kind_of(fn)
                        if kind:
                            counts["func_kind"][kind] += 1
                        walk(fn)
                else:
                    walk(value)

        walk(table)
    return {"tables": tables,
            "cond_container": dict(counts["cond_container"]),
            "cond_kind": dict(counts["cond_kind"]),
            "func_kind": dict(counts["func_kind"]),
            "list_key": dict(counts["list_key"])}


def _form(counts):
    """old / new / both / none for an {"old": n, "new": n} count dict."""
    old, new = counts.get("old", 0), counts.get("new", 0)
    if old and new:
        return "both"
    if old:
        return "old"
    if new:
        return "new"
    return "none"


def advancement_census(game):
    """Per-trigger condition keys from the version's advancement JSONs.

    Returns {trigger: {key: uses}}. The keys a trigger's data uses are
    the keys the evaluator must read for that trigger.
    """
    keys = defaultdict(Counter)
    advancements = 0
    for _path, adv in _jar_jsons(game, "data/minecraft/advancement/"):
        if not isinstance(adv, dict):
            continue
        advancements += 1
        for crit in (adv.get("criteria") or {}).values():
            if not isinstance(crit, dict):
                continue
            trigger = crit.get("trigger", "")
            cond = crit.get("conditions") or {}
            if isinstance(cond, dict):
                for key in cond:
                    keys[trigger][key] += 1
    return {"advancements": advancements,
            "triggers": {t: dict(c) for t, c in keys.items()}}


def special_model_types(game):
    """The special item-model types the version's items use.

    Walks every assets/minecraft/items/*.json for {"type":
    "minecraft:special"} nodes and records their nested model.type,
    with an example item file per type. (1.21.7 bed items use
    minecraft:bed; the port's SpecialModel::decode has no bed arm.)
    """
    used = {}
    items = 0
    for path, doc in _jar_jsons(game, "assets/minecraft/items/"):
        items += 1

        def walk(node):
            if isinstance(node, list):
                for item in node:
                    walk(item)
                return
            if not isinstance(node, dict):
                return
            if node.get("type") == "minecraft:special":
                model = node.get("model")
                if isinstance(model, dict) \
                        and isinstance(model.get("type"), str):
                    used.setdefault(model["type"], path)
            for value in node.values():
                walk(value)

        walk(doc)
    return {"items": items, "types": used}


def sprite_inventory(game):
    """The gui/sprites ids in a client jar (path after gui/sprites/)."""
    prefix = "assets/minecraft/textures/gui/sprites/"
    sprites = set()
    for name in game.namelist():
        if name.startswith(prefix) and name.endswith(".png"):
            sprites.add(name[len(prefix):-len(".png")])
    return sprites


# --------------------------------------------------------------------------
# Code extraction

# A JSON-index read of a string literal: `["key"]`, `.get("key")`,
# `["key"].as_str()`. Comments are stripped first (the audit tool's
# stripper) so a commented-out read never counts.
_INDEX_READ = re.compile(r"""(?:\[|\.get\()\s*"([A-Za-z0-9_]+)"\s*(?:\]|\))""")
_STRING_LITERAL = re.compile(r'"([^"\\]*)"')


def _production_text(ctx: Ctx, rel: str):
    """(comment-stripped text, newline offsets) for a file, cached.

    The stripper preserves offsets and newlines, so a match's line is
    found by bisecting the newline offsets — and matches the original
    file's line numbers, which test detection uses.
    """
    if rel in ctx._text_cache:
        return ctx._text_cache[rel]
    path = ctx.tree / rel
    text = _read(path)
    if not text:
        ctx._text_cache[rel] = ("", [])
        return ctx._text_cache[rel]
    if leakage is not None:
        clean = leakage._strip_comments(text)
    else:
        clean = re.sub(r"//[^\n]*", "", text)
    newlines = [i for i, ch in enumerate(clean) if ch == "\n"]
    ctx._text_cache[rel] = (clean, newlines)
    return ctx._text_cache[rel]


def _line_of(newlines, pos: int) -> int:
    """1-based line number of a char offset."""
    import bisect
    return bisect.bisect_right(newlines, pos) + 1


def _reads_in(ctx: Ctx, rel: str, text: str, newlines,
              start: int = 0, end=None, exclude=()):
    """Index-read literals in text[start:end], production lines only.

    Matches inside any (start, end) span in `exclude` are dropped —
    the loot check excludes the condition-evaluator spans from the
    container scan, so the kind line's `c["condition"]` never counts
    as a container read. Test fixtures carry both forms of every key
    and would mask a missing production read.
    """
    if end is None:
        end = len(text)
    lits = set()
    for match in _INDEX_READ.finditer(text, start, end):
        pos = match.start()
        if any(a <= pos < b for a, b in exclude):
            continue
        if ctx.is_test_line(rel, _line_of(newlines, pos)):
            continue
        lits.add(match.group(1))
    return lits


def read_literals(ctx: Ctx, rels, production_only=True):
    """String literals read by index/get in the given tree files.

    Returns {literal: [(file, line)]}. With production_only, lines
    inside test modules are dropped (test fixtures carry both forms
    of every key and would mask a missing production read).
    """
    found: dict[str, list] = defaultdict(list)
    for rel in rels:
        text, newlines = _production_text(ctx, rel)
        if not text:
            continue
        for match in _INDEX_READ.finditer(text):
            line = _line_of(newlines, match.start())
            if production_only and ctx.is_test_line(rel, line):
                continue
            found[match.group(1)].append((rel, line))
    return found


def find_identifier(ctx: Ctx, rel: str, identifier: str):
    """(file, line) of production uses of an identifier, or []."""
    text, newlines = _production_text(ctx, rel)
    if not text:
        return []
    pat = re.compile(r"\b" + re.escape(identifier) + r"\b")
    hits = []
    for match in pat.finditer(text):
        line = _line_of(newlines, match.start())
        if not ctx.is_test_line(rel, line):
            hits.append((rel, line))
    return hits


# --------------------------------------------------------------------------
# Data checks

def check_fuel_table(ctx: Ctx):
    """fuel-table: a version with no fuel component needs the table."""
    title = "furnace fuel comes from vanilla's FuelValues table"
    game = ctx.server_game
    if game is None:
        return _result("fuel-table", title, UNMEASURED,
                       "no server jar: cannot tell whether this version "
                       "has a fuel component")
    hits = _class_string_hits(game, b"cooking_fuel")
    has_component = hits > 0
    rel = "crates/horizon-items/src/items.rs"
    table = find_identifier(ctx, rel, "FUEL_VALUES")
    apply = find_identifier(ctx, rel, "apply_fuel_values")
    fallback = bool(table or apply)
    if has_component:
        return _result(
            "fuel-table", title, PASS,
            f"the jar has a cooking_fuel component ({hits} class(es) "
            f"mention it), so the component path works; "
            f"a FuelValues fallback is {'also ' if fallback else 'not '}present")
    if fallback:
        entries = 0
        text, _ = _production_text(ctx, rel)
        start = text.find("FUEL_VALUES")
        if start >= 0:
            entries = text.count('("minecraft:', start, start + 20000)
        return _result(
            "fuel-table", title, PASS,
            "the jar has no cooking_fuel component and the catalog "
            f"fills fuel from the FuelValues table ({entries} entries)",
            evidence=[f"{rel}:{line}" for _f, line in table[:1]])
    return _result(
        "fuel-table", title, FAIL,
        "the jar has no cooking_fuel component (no class mentions it) "
        "and the catalog has no FuelValues fallback: every item's fuel "
        "is None, so the furnace fuel slot rejects all fuel, hoppers "
        "and shift-click never route fuel, and burn time is 0 "
        "(the 1.21.7 defect, fixed there by a53b0c58)",
        evidence=[f"{rel}: no FUEL_VALUES / apply_fuel_values"])


def check_brewing_fuel(ctx: Ctx):
    """brewing-fuel: a tag-gated version needs tag-based fuel."""
    title = "brewing-stand fuel comes from the brewing_fuel tag"
    game = ctx.server_game
    if game is None:
        return _result("brewing-fuel", title, UNMEASURED,
                       "no server jar: cannot tell whether this version "
                       "gates brewing fuel on the tag or a component")
    tag_gated = ("data/minecraft/tags/item/brewing_fuel.json"
                 in game.namelist())
    rel = "crates/horizon-items/src/items.rs"
    text, newlines = _production_text(ctx, rel)
    # A tag-based population mentions brewing_fuel next to a tag lookup
    # (or under a BREWING_FUEL / apply_brewing_fuel name). A bare
    # `minecraft:brewing_fuel` component read does not count: the
    # component does not exist in a tag-gated version's data.
    tag_populated = False
    evidence = []
    # The code names are BREWING_FUEL_TAG and apply_brewing_fuel_values;
    # find_identifier matches whole identifiers, so the bare prefixes
    # ("BREWING_FUEL", "apply_brewing_fuel") never match them.
    for marker in ("BREWING_FUEL_TAG", "apply_brewing_fuel_values"):
        if find_identifier(ctx, rel, marker):
            tag_populated = True
            evidence.append(f"{rel}: {marker}")
    if not tag_populated:
        for match in re.finditer(r"brewing_fuel", text):
            line = _line_of(newlines, match.start())
            if ctx.is_test_line(rel, line):
                continue
            window = text[max(0, match.start() - 120):match.end() + 40]
            if re.search(r"\btags?\b.{0,80}brewing_fuel|"
                         r"brewing_fuel.{0,40}\btags?\b", window, re.S):
                tag_populated = True
                evidence.append(f"{rel}:{line} (tag lookup)")
                break
    if not tag_gated:
        return _result(
            "brewing-fuel", title, PASS,
            "the jar has no brewing_fuel item tag: this version gates "
            "brewing fuel on the component, so the component read works")
    if tag_populated:
        return _result(
            "brewing-fuel", title, PASS,
            "the jar gates brewing fuel on the brewing_fuel item tag "
            "and the catalog populates brewing fuel from the tag",
            evidence=evidence)
    return _result(
        "brewing-fuel", title, FAIL,
        "the jar gates brewing fuel on the brewing_fuel item tag "
        "(blaze powder) and the catalog has no tag-based population: "
        "the brewing-stand fuel slot reads only the brewing_fuel "
        "component, which this version does not have",
        evidence=[f"{rel}: no tag-based brewing-fuel population"])


# The loot evaluator files, per version tree. loot.rs (horizon-world)
# and the loot code in items.rs (horizon-items) are separate evaluators
# — the block-drops fix touched both — so each is checked on its own;
# a file that reads no loot field at all is not an evaluator here.
_LOOT_FILES = ["crates/horizon-world/src/loot.rs",
               "crates/horizon-items/src/items.rs"]


def _loot_file_reads(ctx: Ctx, rel: str):
    """What one evaluator file reads, per role — or None.

    Role-scoped, because the same literal means different things in
    different roles: `condition` is the old kind key inside `fn
    condition` but the new container key in the pool/entry code, so a
    file-level scan conflates them (it passed 1.21.11, whose loot.rs
    reads the new container key and the new kind key against old-form
    data). Kind reads come from the `fn condition` /
    `fn condition_object` / `fn modify` / `fn modifiers` / `fn run`
    bodies; container reads from the rest of the file; list reads are
    unambiguous anywhere.
    """
    text, newlines = _production_text(ctx, rel)
    if not text:
        return None
    # Kind reads come from the condition / function evaluators. In
    # odds.rs the kind line lives in `fn condition_object`, a helper
    # `fn condition` delegates to, so both bodies count for the kind
    # role — and both spans are excluded from the container scan, so
    # the kind line's `c["condition"]` never counts as a container read.
    kind_spans = []
    kind_lits = set()
    for name in ("condition", "condition_object"):
        span = _fn_span(text, name)
        if span:
            kind_spans.append(span)
            kind_lits |= _reads_in(ctx, rel, text, newlines,
                                   span[0], span[1])
    # In loot.rs/odds.rs `fn modify` is a dispatcher and the kind
    # line lives in `fn run`; in items.rs it is `fn modifiers`.
    func_lits = set()
    for name in ("modify", "modifiers", "run"):
        span = _fn_span(text, name)
        if span:
            func_lits |= _reads_in(ctx, rel, text, newlines,
                                   span[0], span[1])
    outside_lits = _reads_in(ctx, rel, text, newlines,
                             exclude=kind_spans)
    reads = {
        "cond_container": {"old": "conditions" in outside_lits,
                           "new": "condition" in outside_lits},
        "cond_kind": {"old": "condition" in kind_lits,
                      "new": "type" in kind_lits},
        "func_kind": {"old": "function" in func_lits,
                      "new": "type" in func_lits},
        "list_key": {"old": "functions" in outside_lits,
                     "new": "modifier" in outside_lits},
    }
    if not any(any(role.values()) for role in reads.values()):
        return None  # no loot reads at all: not an evaluator
    return reads


def check_loot_fields(ctx: Ctx):
    """loot-fields: the loot evaluators read the version's field names."""
    title = "loot evaluators read this version's field names"
    game = ctx.server_game
    if game is None:
        return _result("loot-fields", title, UNMEASURED,
                       "no server jar: cannot census this version's "
                       "loot-table field names")
    census = loot_census(game)
    if census["tables"] == 0:
        return _result("loot-fields", title, UNMEASURED,
                       "the server jar holds no loot tables")
    roles = ("cond_container", "cond_kind", "func_kind", "list_key")
    forms = {role: _form(census[role]) for role in roles}
    rels = list(_LOOT_FILES)
    rels += [rel for rel, _ in
             ctx.rs_files("crates/horizon-world/src/loot")]
    failures = []
    per_file = []
    for rel in rels:
        reads = _loot_file_reads(ctx, rel)
        if reads is None:
            continue
        short = rel.split("/")[-1]
        per_file.append(f"{short}: container "
                        f"{'conditions' if reads['cond_container']['old'] else ''}"
                        f"{' condition' if reads['cond_container']['new'] else ''}; "
                        f"cond-kind "
                        f"{'condition' if reads['cond_kind']['old'] else ''}"
                        f"{' type' if reads['cond_kind']['new'] else ''}; "
                        f"func-kind "
                        f"{'function' if reads['func_kind']['old'] else ''}"
                        f"{' type' if reads['func_kind']['new'] else ''}; "
                        f"list "
                        f"{'functions' if reads['list_key']['old'] else ''}"
                        f"{' modifier' if reads['list_key']['new'] else ''}")
        for role in roles:
            form = forms[role]
            if form in ("old", "both") and not reads[role]["old"]:
                failures.append(f"{short}: {role} — data uses the old "
                                f"form but the evaluator never reads it")
            if form in ("new", "both") and not reads[role]["new"]:
                failures.append(f"{short}: {role} — data uses the new "
                                f"form but the evaluator never reads it")
    detail = ("loot census over "
              f"{census['tables']} tables: container "
              f"{forms['cond_container']} {census['cond_container']}, "
              f"condition kind {forms['cond_kind']} "
              f"{census['cond_kind']}, function kind "
              f"{forms['func_kind']} {census['func_kind']}, list key "
              f"{forms['list_key']} {census['list_key']}; "
              + "; ".join(per_file))
    if failures:
        return _result(
            "loot-fields", title, FAIL,
            detail + " (the 1.21.7 defect: blocks and entities dropped "
            "nothing — fixed there by e3bce30e)",
            evidence=failures)
    return _result("loot-fields", title, PASS, detail)


# Advancement triggers whose condition keys differ across eras, and the
# candidate keys the evaluator might read for each. The census decides
# which candidate the version's data uses; the trigger's own evaluation
# code must read that one. (The table is deliberately small: only keys
# that changed form. A trigger whose data uses a key no code reads at
# all is a different defect — an unevaluated trigger — and this check
# does not claim it.)
ADVANCEMENT_KEY_CANDIDATES = {
    "minecraft:recipe_crafted": ["recipe_id", "recipes"],
    "minecraft:crafter_recipe_crafted": ["recipe_id", "recipes"],
    "minecraft:recipe_unlocked": ["recipe", "recipes"],
    "minecraft:player_generates_container_loot": ["loot_table",
                                                  "loot_tables"],
    "minecraft:enter_block": ["block", "blocks"],
    "minecraft:bee_nest_destroyed": ["block", "blocks"],
    "minecraft:slide_down_block": ["block", "blocks"],
}

_ADVANCEMENTS_RS = "crates/horizon-client/src/advancements.rs"
_ADV_TRIGGER_FILES = [
    "crates/horizon-client/src/game/advancements.rs",
    "crates/horizon-client/src/game/machines/crafter.rs",
]
_CONDITION_HELPER = re.compile(r"adv::([a-z_]+_condition)\s*\(")


def _fn_span(text: str, fn_name: str):
    """(body_start, body_end) of `fn <name>(` in comment-stripped text.

    Skips trait declarations (`fn block(...);` — no body): the scan
    from the signature tracks `()`/`[]` depth, so the `;` in an array
    type (`[i32; 3]`) never counts as a declaration terminator, and
    the first `{` at depth 0 is the body's open brace. (advancements.rs
    declares `fn block` in the World trait long before defining
    `pub fn block`.) Returns None when no definition is found.
    """
    if leakage is None:
        return None
    # The name may carry a generic parameter list (`fn condition<R:
    # RandomSource>(...` in items.rs) before the `(`.
    pat = re.compile(r"\bfn\s+" + re.escape(fn_name)
                     + r"\s*(?:<[^<>]*>)?\s*\(")
    for match in pat.finditer(text):
        # The match consumes the opening `(` of the parameter list.
        depth = 1
        brace = -1
        i = match.end()
        while i < len(text):
            ch = text[i]
            if ch in "([":
                depth += 1
            elif ch in ")]":
                depth -= 1
            elif ch == "{" and depth == 0:
                brace = i
                break
            elif ch == ";" and depth == 0:
                break  # a declaration; keep looking for the definition
            i += 1
        if brace < 0:
            continue
        _body, end = leakage._brace_block(text, brace)
        return brace + 1, end - 1
    return None


def _fn_body(text: str, fn_name: str):
    """The `{...}` body of `fn <name>(` in comment-stripped text."""
    span = _fn_span(text, fn_name)
    if span is None:
        return None
    return text[span[0]:span[1]]


def _trigger_closure_literals(ctx: Ctx, trigger: str):
    """Index-read literals in a trigger's evaluation code.

    Each trigger is evaluated by a `self.trigger("<id>", move |c, w|
    {...})` closure in game/advancements.rs (or crafter.rs for the
    crafter trigger); the condition-key read is inline there
    (`c["blocks"]`) or one helper call away (`adv::block_condition`,
    whose body is followed). Attribution is per trigger: a `block`
    read in a location predicate must not mask an enter_block that
    reads `blocks` — the key-level scan did exactly that on 1.21.11.
    Returns (literals, found): found is False when no evaluation site
    was located at all (the caller falls back to the file-level set).
    """
    literals: set[str] = set()
    found = False
    call = re.compile(r'self\.trigger\("'
                      + re.escape(trigger) + r'"')
    for rel in _ADV_TRIGGER_FILES:
        text, _ = _production_text(ctx, rel)
        if not text:
            continue
        for match in call.finditer(text):
            found = True
            closure = re.search(r"move\s*\|[^|]*\|\s*\{",
                                text[match.end():match.end() + 400])
            if not closure:
                continue
            brace = text.find("{", match.end() + closure.start())
            if brace < 0 or leakage is None:
                continue
            body, _end = leakage._brace_block(text, brace)
            literals.update(_INDEX_READ.findall(body))
            # One level of helper indirection: adv::*_condition, and
            # adv::block — slide_down_block evaluates its block through
            # the BlockPredicate evaluator, which reads ["blocks"] for
            # the object form (right for a predicate, wrong for the
            # trigger's {"block": ...} condition).
            helpers_text, _ = _production_text(ctx, _ADVANCEMENTS_RS)
            for helper in _CONDITION_HELPER.findall(body):
                helper_body = _fn_body(helpers_text, helper)
                if helper_body:
                    literals.update(_INDEX_READ.findall(helper_body))
            if re.search(r"adv::block\s*\(", body):
                block_body = _fn_body(helpers_text, "block")
                if block_body:
                    literals.update(_INDEX_READ.findall(block_body))
    return literals, found


def check_advancement_keys(ctx: Ctx):
    """advancement-keys: triggers read the version's key names."""
    title = "advancement triggers read this version's key names"
    game = ctx.server_game
    if game is None:
        return _result("advancement-keys", title, UNMEASURED,
                       "no server jar: cannot census this version's "
                       "advancement condition keys")
    census = advancement_census(game)
    if census["advancements"] == 0:
        return _result("advancement-keys", title, UNMEASURED,
                       "the server jar holds no advancement JSONs")
    # recipe_unlocked has no trigger closure: it is evaluated through
    # the index built by index_recipes in advancements.rs.
    helpers_text, _ = _production_text(ctx, _ADVANCEMENTS_RS)
    index_body = _fn_body(helpers_text, "index_recipes") or ""
    index_literals = set(_INDEX_READ.findall(index_body))
    # File-level fallback for any trigger whose evaluation site cannot
    # be located (a renamed trigger function, a moved file).
    file_literals = read_literals(
        ctx, [_ADVANCEMENTS_RS] + _ADV_TRIGGER_FILES)
    failures = []
    notes = []
    checked = 0
    for trigger, candidates in ADVANCEMENT_KEY_CANDIDATES.items():
        used = census["triggers"].get(trigger, {})
        data_keys = [k for k in candidates if used.get(k, 0) > 0]
        if not data_keys:
            continue  # this version's data never constrains the trigger
        checked += 1
        if trigger == "minecraft:recipe_unlocked":
            read = index_literals
        else:
            read, found = _trigger_closure_literals(ctx, trigger)
            if not found:
                read = set(file_literals)
                notes.append(f"{trigger}: no evaluation site found; "
                             "fell back to file-level reads")
        if not any(k in read for k in data_keys):
            failures.append(
                f"{trigger}: data uses {', '.join(data_keys)} "
                f"({sum(used[k] for k in data_keys)} criteria) but the "
                f"trigger reads only "
                f"{', '.join(k for k in candidates if k in read) or 'neither'}")
    detail = (f"census over {census['advancements']} advancements, "
              f"{checked} keyed triggers checked")
    if failures:
        return _result(
            "advancement-keys", title, FAIL,
            detail + "; " + "; ".join(failures)
            + " (the 1.21.7 defect: wrong advancements fired, or none "
            "did — fixed there by 59ba5f8d and caef65c4)",
            evidence=failures, notes=notes)
    return _result("advancement-keys", title, PASS,
                   detail + "; every keyed trigger's data key is read",
                   notes=notes)


def check_bed_models(ctx: Ctx):
    """bed-models: every special item-model type has a decoder arm.

    The data half of the beds defect: 1.21.7's bed items use the
    `minecraft:bed` special model type, which SpecialModel::decode has
    no arm for, so all 16 bed item definitions fail to load ("unknown
    special model type") and the bed item is invisible in the
    inventory. The block-render half is a rust check (bed-models in
    the manifest) — the block model resolves through prepare's
    model-states export, which this tool cannot reproduce without
    Java and the client jar's blockstates.
    """
    title = "every special item-model type has a decoder arm"
    game = ctx.client_game
    if game is None:
        return _result("bed-models", title, SKIP,
                       "no client jar: the special-model census needs "
                       "the version's item models (assets/minecraft/"
                       "items/*.json)")
    rel = "crates/horizon-assets/src/item_model/special.rs"
    path = ctx.tree / rel
    if not path.is_file():
        return _result("bed-models", title, SKIP,
                       f"no {rel} in this tree")
    oracle = special_model_types(game)
    text, _ = _production_text(ctx, rel)
    # The match arms of SpecialModel::decode: `"minecraft:xxx" =>`.
    arms = set(re.findall(r'"(minecraft:[a-z_0-9]+)"\s*=>', text))
    missing = {t: f for t, f in oracle["types"].items() if t not in arms}
    detail = (f"census over {oracle['items']} item models: "
              f"{len(oracle['types'])} special types used, "
              f"{len(arms)} decoder arms")
    if missing:
        return _result(
            "bed-models", title, FAIL,
            detail + "; missing arms: "
            + ", ".join(f"{t} (e.g. {f.split('/')[-1]})"
                        for t, f in sorted(missing.items()))
            + " — items using them fail to load and render invisible",
            evidence=[f"{t}: used by {f}" for t, f in
                      sorted(missing.items())])
    return _result("bed-models", title, PASS,
                   detail + "; every used special type decodes")


_SPRITE_LITERAL = re.compile(r'"(gui/sprites/[A-Za-z0-9_/.\-{][^"]*)"')
_SPRITE_FORMAT = re.compile(r'gui/sprites/([A-Za-z0-9_/.\-]*)')


def _sprite_references(ctx: Ctx):
    """(literal sprite ids, dynamic prefixes) the code references.

    Literal ids come from full `"gui/sprites/..."` strings; dynamic
    ones from format! strings, cut at the first `{` — a prefix the
    jar must have at least one sprite under.
    """
    literals: set[str] = set()
    prefixes: set[str] = set()
    roots = ["crates/horizon-client/src",
             "crates/horizon-client-render/src"]
    for rel, _path in ctx.rs_files(*roots):
        text, newlines = _production_text(ctx, rel)
        if not text:
            continue
        for match in _SPRITE_LITERAL.finditer(text):
            line = _line_of(newlines, match.start())
            if ctx.is_test_line(rel, line):
                continue
            raw = match.group(1)
            if "{" in raw:
                prefix = raw.split("{", 1)[0]
                if prefix.startswith("gui/sprites/") \
                        and len(prefix) > len("gui/sprites/"):
                    prefixes.add(prefix[len("gui/sprites/"):])
            else:
                literals.add(raw[len("gui/sprites/"):])
    return literals, prefixes


def check_menu_sprites(ctx: Ctx):
    """menu-sprites: every referenced sprite exists or is covered."""
    title = "every referenced menu sprite exists or is covered"
    game = ctx.client_game
    if game is None:
        return _result("menu-sprites", title, SKIP,
                       "no client jar: the sprite census needs the "
                       "version's textures (assets/minecraft/textures/"
                       "gui/sprites/)")
    own = sprite_inventory(game)
    fallback_rel = "crates/horizon-prepare/src/lib.rs"
    has_fallback = bool(find_identifier(ctx, fallback_rel,
                                        "fallback_menu_sprites"))
    fallback_sprites: set[str] = set()
    fallback_verified = False
    if has_fallback and ctx.fallback_game is not None:
        fallback_sprites = sprite_inventory(ctx.fallback_game)
        fallback_verified = True
    literals, prefixes = _sprite_references(ctx)
    missing = []
    covered = 0
    for sprite in sorted(literals):
        if sprite in own:
            continue
        if has_fallback and sprite in fallback_sprites:
            covered += 1
            continue
        if has_fallback and not fallback_verified:
            covered += 1  # the fallback covers it; unverified which jar
            continue
        missing.append(sprite)
    missing_prefixes = [p for p in sorted(prefixes)
                        if not any(s.startswith(p) for s in own)
                        and not (has_fallback and any(
                            s.startswith(p) for s in fallback_sprites))]
    detail = (f"{len(literals)} literal sprite references, "
              f"{len(prefixes)} dynamic prefixes; version jar holds "
              f"{len(own)} sprites; prepare fallback "
              f"{'present' if has_fallback else 'ABSENT'}"
              + (f" ({len(fallback_sprites)} sprites in the "
                 f"{FALLBACK_VERSION} jar)"
                 if fallback_verified else
                 " (fallback jar not supplied — fallback coverage "
                 "unverified)" if has_fallback else "")
              + f"; {covered} covered by the fallback")
    failures = ([f"missing sprite: gui/sprites/{s}" for s in missing]
                + [f"missing sprite prefix: gui/sprites/{p}"
                   for p in missing_prefixes])
    if failures:
        return _result(
            "menu-sprites", title, FAIL,
            detail + "; " + "; ".join(failures[:10])
            + (" ..." if len(failures) > 10 else "")
            + " (the 1.21.7 defect: menus showed raw keys and missing "
            "icons — fixed there by the prepare-time fallback, 20453af0)",
            evidence=failures)
    return _result("menu-sprites", title, PASS,
                   detail + "; every referenced sprite resolves")


def check_absent_tags(ctx: Ctx):
    """absent-tags: the leakage audit, baseline ratchet.

    Reuses tools/audit_26_3_leakage.py (imported, not copied): the
    check fails only on leaks NEW against tools/leakage-baselines.json,
    exactly like the leakage-audit CI job. The total leak count is
    reported as evidence — it is the version's standing debt, not a
    failure of this run.
    """
    title = "no new 26.3-only tag/attribute/gamerule leaks"
    if leakage is None:
        return _result("absent-tags", title, SKIP,
                       "tools/audit_26_3_leakage.py is not available")
    tools = Path(__file__).resolve().parent
    baseline = leakage.load_baseline(
        ctx.leakage_baseline or str(tools / LEAKAGE_BASELINE))
    exceptions = leakage.load_exceptions(
        ctx.leakage_exceptions or str(tools / LEAKAGE_EXCEPTIONS))
    result = leakage.audit_version(
        ctx.tree, ctx.version, jar_arg=(str(ctx.server_jar)
                                        if ctx.server_jar else None),
        jar_dir=(str(ctx.jar_dir) if ctx.jar_dir else None),
        exceptions=exceptions)
    if result["status"] == "blocked":
        return _result("absent-tags", title, UNMEASURED,
                       result.get("reason", "no usable data source"))
    leaks = {f["name"] for f in result["findings"]
             if f["severity"] == "leak"}
    recorded = baseline.get(ctx.version, set())
    new = sorted(leaks - recorded)
    fixed = sorted(recorded - leaks)
    counts = result["counts"]
    detail = (f"audit: {counts.get('leak', 0)} leaks, "
              f"{counts.get('review', 0)} review notes, "
              f"{len(new)} new vs baseline, {len(fixed)} fixed vs baseline")
    if new:
        return _result("absent-tags", title, FAIL,
                       detail + "; new leaks: " + ", ".join(new[:10])
                       + (" ..." if len(new) > 10 else ""),
                       evidence=[f"new leak: {name}" for name in new],
                       notes=[f"fixed vs baseline: {name}"
                              for name in fixed])
    return _result("absent-tags", title, PASS, detail,
                   notes=[f"fixed vs baseline: {name}" for name in fixed])


# --------------------------------------------------------------------------
# Rust checks (the regression tests, via the manifest)

def _test_present(ctx: Ctx, crate: str, test: str) -> bool:
    """Whether `fn <test>(` exists in the crate's sources."""
    base = ctx.tree / "crates" / crate / "src"
    if not base.is_dir():
        return False
    pat = re.compile(r"fn\s+" + re.escape(test) + r"\s*[\(<]")
    for path in base.rglob("*.rs"):
        if pat.search(_read(path)):
            return True
    return False


def _parse_crate_log(text: str):
    """{test name: ok|FAILED} from a `cargo test --lib` log."""
    results = {}
    pat = re.compile(r"^test\s+(\S+)\s+\.\.\.\s+(ok|FAILED)", re.M)
    for match in pat.finditer(text):
        full, outcome = match.group(1), match.group(2)
        results[full.rsplit("::", 1)[-1]] = outcome
    return results


def _cargo_env():
    """The VM's cargo/rustup homes (AGENTS.md), when they exist."""
    import os
    env = dict(os.environ)
    home = Path.home()
    rustup = home / "workspace" / "tools" / "rustup"
    cargo = home / "workspace" / "tools" / "cargo"
    if rustup.is_dir():
        env.setdefault("RUSTUP_HOME", str(rustup))
    if cargo.is_dir():
        env.setdefault("CARGO_HOME", str(cargo))
        bin_dir = str(cargo / "bin")
        env["PATH"] = bin_dir + os.pathsep + env.get("PATH", "")
    jdk = home / "workspace" / "tools" / "jdk-25" / "bin"
    if jdk.is_dir():
        env["PATH"] = str(jdk) + os.pathsep + env["PATH"]
    return env


def _run_rust_test(ctx: Ctx, crate: str, test: str):
    """Run one registered test with cargo test; (outcome, detail).

    Outcome is ok / FAILED / not-covered (the filter matched no test)
    / error (cargo itself failed). Single-threaded, one test at a time:
    the VM OOMs on the big crates' test compiles, so this is opt-in
    and the doc says where each crate may run.
    """
    cmd = ["cargo", "test", "-p", crate, "--lib", "--color=never",
           test]
    try:
        proc = subprocess.run(
            cmd, cwd=str(ctx.tree), capture_output=True, text=True,
            timeout=3600, env=_cargo_env())
    except (subprocess.TimeoutExpired, OSError) as exc:
        return "error", f"cargo test failed to run: {exc}"
    out = proc.stdout + proc.stderr
    parsed = _parse_crate_log(out)
    if test in parsed:
        return parsed[test], f"cargo test -p {crate} --lib {test}"
    summary = re.search(r"test result: \w+\. (\d+) passed", out)
    if summary and summary.group(1) == "0":
        return "not-covered", f"no test matched the filter {test!r}"
    if proc.returncode != 0:
        tail = "\n".join(out.strip().splitlines()[-5:])
        return "error", f"cargo test exited {proc.returncode}: {tail}"
    return "not-covered", f"no result line for {test!r} in the log"


def check_rust_class(ctx: Ctx, check_id: str, title: str):
    """One class's registered rust tests, in the mode the run asked for.

    Default: presence only (skip unless a test is missing, which is
    not-covered). --crate-logs: results from the logs. --run-rust:
    run the tests. A class with no registered tests is not-covered —
    the test still needs writing (see docs/DEFECT-SUITE.md).
    """
    entries = ctx.rust_manifest.get(check_id, [])
    if not entries:
        return _result(check_id, title, NOT_COVERED,
                       "no regression test is registered for this class "
                       "yet — write one (see docs/DEFECT-SUITE.md) and "
                       "add it to tools/defect_suite_rust.json")
    # An entry's `suite` is the cargo package the test RUNS under,
    # which differs from the source crate when a suite's lib root
    # lives in another crate's sources: horizon-client-lib's lib root
    # is ../horizon-client/src/client_lib.rs, so tests written in
    # crates/horizon-client/src/ run as horizon-client-lib and their
    # results land in horizon-client-lib.log, not horizon-client.log.
    # Presence still uses `crate` (the source directory).
    if ctx.crate_logs is not None:
        logs = {}
        for suite in {e.get("suite", e["crate"]) for e in entries}:
            # A downloaded crate-tests artifact nests the logs one
            # level down (crate-tests-1.21.7/horizon-items.log), so
            # look there too before calling the log missing.
            log = ctx.crate_logs / f"{suite}.log"
            if not log.is_file():
                nested = sorted(ctx.crate_logs.glob(f"*/{suite}.log"))
                log = nested[0] if nested else log
            logs[suite] = _parse_crate_log(_read(log)) if log.is_file() \
                else None
        outcomes = []
        for entry in entries:
            crate, test = entry["crate"], entry["test"]
            suite = entry.get("suite", crate)
            if logs.get(suite) is None:
                outcomes.append((test, crate, "not-covered",
                                 f"no {suite}.log in {ctx.crate_logs}"))
            elif test in logs[suite]:
                outcomes.append((test, crate, logs[suite][test], ""))
            else:
                outcomes.append((test, crate, "not-covered",
                                 "no result line in the log"))
        return _rust_verdict(check_id, title, outcomes, "crate logs")
    if ctx.run_rust:
        outcomes = []
        for entry in entries:
            crate, test = entry["crate"], entry["test"]
            suite = entry.get("suite", crate)
            if not _test_present(ctx, crate, test):
                outcomes.append((test, crate, "not-covered",
                                 "no such test in this tree"))
                continue
            outcome, detail = _run_rust_test(ctx, suite, test)
            outcomes.append((test, crate, outcome, detail))
        return _rust_verdict(check_id, title, outcomes, "cargo test")
    # Presence only.
    missing = [e for e in entries
               if not _test_present(ctx, e["crate"], e["test"])]
    present = [e for e in entries
               if _test_present(ctx, e["crate"], e["test"])]
    if missing:
        return _result(
            check_id, title, NOT_COVERED,
            f"{len(present)}/{len(entries)} registered tests exist in "
            "this tree; missing: "
            + ", ".join(f"{e['crate']}::{e['test']}" for e in missing)
            + " — rerun with --crate-logs or --run-rust for results",
            evidence=[f"missing: {e['crate']}::{e['test']}"
                      for e in missing])
    return _result(
        check_id, title, SKIP,
        f"all {len(entries)} registered tests exist in this tree; "
        "results need --crate-logs (a crate-tests run) or --run-rust",
        evidence=[f"present: {e['crate']}::{e['test']}"
                  for e in present])


def _rust_verdict(check_id, title, outcomes, mode):
    failed = [o for o in outcomes if o[2] == "FAILED"]
    missing = [o for o in outcomes if o[2] == "not-covered"]
    errors = [o for o in outcomes if o[2] == "error"]
    passed = [o for o in outcomes if o[2] == "ok"]
    detail = (f"{mode}: {len(passed)} passed, {len(failed)} failed, "
              f"{len(missing)} not covered, {len(errors)} errors")
    evidence = [f"{crate}::{test}: {outcome}"
                + (f" ({note})" if note else "")
                for test, crate, outcome, note in outcomes]
    if failed or errors:
        return _result(check_id, title, FAIL,
                       detail + "; " + "; ".join(
                           f"{t} {o}" for t, _c, o, _n in failed + errors),
                       evidence=evidence)
    if missing:
        return _result(check_id, title, NOT_COVERED, detail,
                       evidence=evidence)
    return _result(check_id, title, PASS, detail, evidence=evidence)


# --------------------------------------------------------------------------
# Registry and runner

# (id, title, data check or None). A class with a data check is verified
# by it; when the rust manifest also lists tests for the class, the rust
# result combines with the data result (a rust failure fails the class).
# A class with no data check is verified by its rust tests alone, and is
# not-covered until those tests exist.
CHECKS = [
    ("fuel-table",
     "furnace fuel comes from vanilla's FuelValues table",
     check_fuel_table),
    ("brewing-fuel",
     "brewing-stand fuel comes from the brewing_fuel tag",
     check_brewing_fuel),
    ("loot-fields",
     "loot evaluators read this version's field names",
     check_loot_fields),
    ("recipe-packets",
     "recipe_book_add packets decode (all slot display kinds)",
     None),
    ("advancement-keys",
     "advancement triggers read this version's key names",
     check_advancement_keys),
    ("bed-models",
     "bed item models decode and beds render",
     check_bed_models),
    ("mob-models",
     "every entity has a resolvable model",
     None),
    ("undead-burn",
     "undead ignite promptly in daylight",
     None),
    ("menu-sprites",
     "every referenced menu sprite exists or is covered",
     check_menu_sprites),
    ("absent-tags",
     "no new 26.3-only tag/attribute/gamerule leaks",
     check_absent_tags),
]

CHECK_IDS = [c[0] for c in CHECKS]


def _combine(data_result, rust_result):
    """Merge a data result with its class's rust result."""
    if rust_result is None:
        return data_result
    merged = dict(data_result)
    merged["evidence"] = (data_result["evidence"]
                          + ["rust: " + e for e in rust_result["evidence"]])
    merged["notes"] = data_result["notes"] + rust_result["notes"]
    merged["detail"] = (data_result["detail"]
                        + " | rust: " + rust_result["detail"])
    if data_result["status"] == PASS:
        if rust_result["status"] == FAIL:
            merged["status"] = FAIL
        elif rust_result["status"] == NOT_COVERED:
            merged["status"] = NOT_COVERED
        # rust PASS or SKIP (present, not run) leaves the data PASS.
    return merged


def run_version(ctx: Ctx, only=None):
    """Run the suite for one version; returns the version result dict."""
    results = []
    for check_id, title, data_fn in CHECKS:
        if only and check_id not in only:
            continue
        data_result = data_fn(ctx) if data_fn else None
        rust_result = None
        if ctx.rust_manifest.get(check_id):
            rust_result = check_rust_class(ctx, check_id, title)
        if data_result is None:
            # Rust-only class: the rust result IS the check (not-covered
            # when no test is registered yet).
            if rust_result is None:
                rust_result = check_rust_class(ctx, check_id, title)
            results.append(rust_result)
        else:
            results.append(_combine(data_result, rust_result))
    summary = Counter(r["status"] for r in results)
    return {
        "version": ctx.version,
        "tree": str(ctx.tree),
        "data_sources": {
            "server_jar": str(ctx.server_jar) if ctx.server_jar else None,
            "client_jar": str(ctx.client_jar) if ctx.client_jar else None,
            "fallback_jar": (str(ctx.fallback_jar)
                             if ctx.fallback_jar else None),
        },
        "checks": results,
        "summary": dict(summary),
    }


def _print_human(result: dict):
    version = result["version"]
    print(f"== {version} ==")
    for check in result["checks"]:
        status = check["status"].upper()
        print(f"  {status:<11} {check['id']:<17} {check['title']}")
        if check["status"] in (FAIL, UNMEASURED, NOT_COVERED):
            print(f"      {check['detail']}")
            for ev in check["evidence"][:5]:
                print(f"      - {ev}")
            if len(check["evidence"]) > 5:
                print(f"      - ... ({len(check['evidence']) - 5} more)")
    summary = result["summary"]
    parts = [f"{n} {s}" for s, n in sorted(summary.items())]
    print(f"{version}: " + ", ".join(parts))
    print()


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description="Cross-version defect regression suite: check the "
                    "defect classes found in 1.21.7 playtesting against "
                    "any version's own jar and tree.")
    parser.add_argument("versions", nargs="*", metavar="VERSION",
                        help="version id(s) (e.g. 1.21.7); omit with --all")
    parser.add_argument("--all", action="store_true",
                        help="check every version under versions/")
    parser.add_argument("--root", default=".",
                        help="repo root (default: current directory)")
    parser.add_argument("--jar", help="vanilla server jar for the version")
    parser.add_argument("--client-jar", help="vanilla client jar")
    parser.add_argument("--fallback-jar",
                        help="26.3 client jar (the menu-sprite fallback "
                             "source); default: <jar-dir>/26.3-client.jar")
    parser.add_argument("--jar-dir", help="directory of <v>-server.jar, "
                        "<v>-client.jar files")
    parser.add_argument("--checks", help="comma-separated check ids to "
                        "run (default: all)")
    parser.add_argument("--rust-manifest",
                        help="rust test manifest "
                             "(default: tools/defect_suite_rust.json)")
    parser.add_argument("--crate-logs", help="directory of crate-tests "
                        "logs (<crate>.log) to read rust results from")
    parser.add_argument("--run-rust", action="store_true",
                        help="run the registered rust tests with cargo "
                             "test (opt-in; the big crates OOM the VM)")
    parser.add_argument("--leakage-baseline",
                        help="leakage baseline JSON (default: "
                             "tools/leakage-baselines.json)")
    parser.add_argument("--leakage-exceptions",
                        help="leakage exceptions JSON (default: "
                             "tools/leakage-exceptions.json)")
    parser.add_argument("--json", action="store_true",
                        help="print the full result as JSON")
    parser.add_argument("--report-only", action="store_true",
                        help="exit 0 regardless of failures (CI reports "
                             "the defect list; it does not gate on it)")
    args = parser.parse_args(argv)
    if bool(args.versions) == bool(args.all):
        parser.error("give version(s) or --all, not both")
    if args.jar and len(args.versions) != 1 and not args.all:
        parser.error("--jar applies to a single version")
    if args.client_jar and len(args.versions) != 1 and not args.all:
        parser.error("--client-jar applies to a single version")
    root = Path(args.root)
    if args.all:
        versions_dir = root / VERSIONS_DIRNAME
        if not versions_dir.is_dir():
            parser.error(f"no {VERSIONS_DIRNAME}/ under {root}")
        versions = sorted(
            (d.name for d in versions_dir.iterdir()
             if d.is_dir() and (d / "Cargo.toml").exists()),
            key=(leakage._version_key if leakage is not None
                 else lambda v: v))
    else:
        versions = args.versions
    only = None
    if args.checks:
        only = [c.strip() for c in args.checks.split(",") if c.strip()]
        unknown = [c for c in only if c not in CHECK_IDS]
        if unknown:
            parser.error(f"unknown check id(s): {', '.join(unknown)} "
                         f"(known: {', '.join(CHECK_IDS)})")
    results = []
    for version in versions:
        tree = root / VERSIONS_DIRNAME / version
        if not tree.is_dir():
            print(f"defect_suite: no tree at {tree}", file=sys.stderr)
            return 2
        ctx = Ctx(root, version,
                  server_jar=args.jar, client_jar=args.client_jar,
                  fallback_jar=args.fallback_jar, jar_dir=args.jar_dir,
                  rust_manifest=args.rust_manifest,
                  crate_logs=args.crate_logs, run_rust=args.run_rust,
                  leakage_baseline=args.leakage_baseline,
                  leakage_exceptions=args.leakage_exceptions)
        results.append(run_version(ctx, only))
    if args.json:
        print(json.dumps({"results": results}, indent=2))
    else:
        for result in results:
            _print_human(result)
        if len(results) > 1:
            print("== summary ==")
            header = f"{'version':<9} " + " ".join(
                f"{s:>11}" for s in
                (PASS, FAIL, SKIP, UNMEASURED, NOT_COVERED))
            print(header)
            for result in results:
                summary = result["summary"]
                line = f"{result['version']:<9} " + " ".join(
                    f"{summary.get(s, 0):>11}" for s in
                    (PASS, FAIL, SKIP, UNMEASURED, NOT_COVERED))
                print(line)
    if args.report_only:
        return 0
    statuses = {c["status"] for r in results for c in r["checks"]}
    if FAIL in statuses:
        return 1
    if UNMEASURED in statuses:
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
