#!/usr/bin/env python3
"""audit_26_3_leakage.py — catch 26.3-only logic leaking into older versions.

The 1.21.7 water bug was the pattern: `FlowingFluid.canHoldAnyFluid` read the
26.3 block tag `#washed_away_by_fluids`, which does not exist in 1.21.7, so
the lookup silently resolved to an empty set and water could not spread at
all. The parity-sweep pilot found the same pattern in four more defects
(bubble columns, lava fire spread, dripstone cauldrons) plus a gamerule-table
gap (no `doFireTick`). Every one of them is mechanically findable: the code
references a name that does not exist in the target version's data, and the
lookup degrades to empty/false instead of failing loudly.

This tool is the standing guard for that class. For one version (or all),
it extracts what the version tree's Rust code references and compares it
against what actually exists in that version's vanilla data:

  tags        block tags (the `tags!` macro list + `.tag("...")` literals)
              and fluid tags (`fluid_mask("...")` + `FluidTags` fields),
              checked against the version's tag inventory.
  attributes  environment attributes (the `ATTRIBUTES` table +
              `attributes["..."]` read sites), checked against the
              version's attribute registry (extracted from the jar's
              EnvironmentAttributes class strings, unioned with the
              biome/dimension JSON keys; prepared-data runs only have the
              JSON keys, a partial set). The attribute system arrived in
              1.21.11; older versions have no attributes at all, so every
              read there needs a legacy fallback (the tool lists the sites;
              it cannot verify fallbacks mechanically, so attribute findings
              are `review`, never `fail`).
  gamerules   the port's `RULES` table + `GameRules` struct fields + string
              reads, checked against the version's gamerule names (from the
              jar's language file). Mojang renamed the whole registry in
              1.21.11 (doFireTick -> fire_spread_radius_around_player, via
              a merge; doDaylightCycle -> advance_time; ...), so the check
              runs through the authoritative rename table (RULE_RENAMES,
              extracted from GameRuleRegistryFix), not raw names. Vanilla
              rules with no port equivalent are gating gaps; a string read
              of a rule the port's own table lacks is a leak (the read
              silently returns false/0). Note G4's triage: doFireTick IS
              covered at the table level (the merge is exact), so the
              lava-ignition gap is in the code path, not the table.

Design decisions (what keeps this signal, not noise):

1. USED vs DEFINED. A tag field that is defined but never read is harmless
   (the 1.21.7 water fix left `tags.washed_away_by_fluids` defined but
   unused). Only a missing tag that production code actually reads is a
   `leak`. Defined-but-unused is a `dead_definition` note; test-only use is
   reported separately.
2. UNION ACROSS REGISTRIES (as in tag_audit.py). A tag name is missing only
   if absent from EVERY registry's inventory — one name can live in several
   registries, and the `.tag()` lookup is registry-agnostic.
3. BLOCK WHEN THE DATA IS ABSENT, DON'T PASS. A version with no usable data
   source is `unmeasured`, never clean. The tool never guesses a version's
   data from another version's.
4. VANILLA DATA IS GROUND TRUTH. The reference inventories come from the
   version's own vanilla jar (bundler-aware) or its prepared `.local` data,
   never from 26.3 or a neighbouring version.
5. EXCEPTIONS ARE EXPLICIT. Tags the port deliberately synthesizes (e.g.
   the 1.21.7 `supports_*` tags, synthesized at worldgen time by the trees
   fix) live in an exceptions file with a reason and a code pointer, and
   are reported as `accepted`, not hidden.

Data sources, in priority order per version:
  --prepared DIR     a `.local/horizon-client/<v>/` directory
  --jar PATH         a vanilla server/client jar (bundler-aware)
  auto-discovery     `<tree>/.local/horizon-client/<v>/`, then
                     `<tree>/.local/minecraft/<v>/*.jar`, then `--jar-dir`
                     (`<v>-server.jar`, `<v>-server-named.jar`,
                     `<v>-client-named.jar`, `<v>-client.jar`)

    python3 tools/audit_26_3_leakage.py 1.21.7
    python3 tools/audit_26_3_leakage.py 1.21.7 --jar jars/1.21.7-server-named.jar --json
    python3 tools/audit_26_3_leakage.py --all --jar-dir jars/
    python3 tools/audit_26_3_leakage.py --all --json --exceptions tools/leakage-exceptions.json
    python3 tools/audit_26_3_leakage.py --all --jar-dir jars/ --baseline tools/leakage-baselines.json

Exit: 0 no leaks (review notes may exist) · 1 leak(s) found · 3 one or more
versions unmeasured (and no leaks) · 2 usage error.

Every version currently fails (all 19 trees carry 26.3 leakage), so a
blocking check cannot demand zero leaks on day one. Instead CI runs with
--baseline tools/leakage-baselines.json: the run fails only on leaks that
are NEW against the recorded baseline (and reports leaks fixed since the
baseline as progress). The baseline is a ratchet, not a pardon — it
records today's debt so no new leak lands silently, and it shrinks as
fixes land. Regenerate it with --write-baseline after a fix lands (and
review the diff: a baseline that grows is a regression, not an update).
Stdlib only, except an optional import of the sibling tag_audit.py for its
Rust tokenizer (test-context classification) and prepared-data inventory
loader; without it, test detection falls back to file-level heuristics and
the prepared inventory is walked directly.
"""

from __future__ import annotations

import argparse
import io
import json
import re
import sys
import zipfile
from pathlib import Path

# --------------------------------------------------------------------------
# Constants

VERSIONS_DIRNAME = "versions"

# The environment-attribute system arrived in 1.21.11 (verified: 1.21.11
# biome JSONs carry "attributes" keys, 1.21.10 and below do not). Kept as a
# sanity check only — the tool measures presence from the data, it does not
# trust this constant.
ATTRIBUTE_SYSTEM_SINCE = "1.21.11"

# Mojang's gamerule registry rework (landed in 1.21.11 — GameRuleRegistryFix
# is present in the 1.21.11+ jars, absent in 1.21.10 and below): every rule
# was renamed from its old camelCase name to a `minecraft:<snake_case>` id,
# several with a new word (doDaylightCycle -> advance_time) and three with
# inverted sense (disableElytraMovementCheck -> elytra_movement_check).
# The port's table uses the NEW names, so a raw name diff against an old
# version's vanilla rules would read every rename as a gap on one side and
# an invention on the other. This map is the authoritative old -> new
# table, extracted from Mojang's GameRuleRegistryFix in the 26.3 jar
# (javap, 2026-11-22) and cross-checked against the port's own
# transcription of it (horizon-world/src/datafix/fixes_c.rs, GAME_RULES —
# identical, 59 pairs). Only true renames are listed; identity names need
# no entry. Two old names map to command_blocks_work (Mojang renamed it
# twice).
RULE_RENAMES = {
    "do_daylight_cycle": "advance_time",
    "do_weather_cycle": "advance_weather",
    "do_tile_drops": "block_drops",
    "do_entity_drops": "entity_drops",
    "do_mob_loot": "mob_drops",
    "natural_regeneration": "natural_health_regeneration",
    "announce_advancements": "show_advancement_messages",
    "spawner_blocks_enabled": "spawner_blocks_work",
    "do_mob_spawning": "spawn_mobs",
    "do_insomnia": "spawn_phantoms",
    "do_patrol_spawning": "spawn_patrols",
    "do_trader_spawning": "spawn_wandering_traders",
    "do_warden_spawning": "spawn_wardens",
    "do_vines_spread": "spread_vines",
    "command_modification_block_limit": "max_block_modifications",
    "max_command_fork_count": "max_command_forks",
    "max_command_chain_length": "max_command_sequence_length",
    "minecart_max_speed": "max_minecart_speed",
    "snow_accumulation_height": "max_snow_accumulation_height",
    "spawn_radius": "respawn_radius",
    "disable_elytra_movement_check": "elytra_movement_check",
    "disable_player_movement_check": "player_movement_check",
    "disable_raids": "raids",
    "do_immediate_respawn": "immediate_respawn",
    "do_limited_crafting": "limited_crafting",
    "enable_command_blocks": "command_blocks_work",
    "command_blocks_enabled": "command_blocks_work",
}

# The fire rules were MERGED, not renamed: doFireTick +
# allowFireTicksAwayFromPlayer became one int rule, fire_spread_radius_
# around_player (0 = fire never spreads, 128 = near players only, -1 =
# everywhere). A version's fire rules are covered if the port has the
# merged rule. Note the consequence for the parity sweep's G4: the table
# covers doFireTick exactly, so G4 is a CODE gap (the lava-ignition path
# never consults the rule), not a table gap — this tool does not flag it.
RULE_MERGE = {
    "fire_spread_radius_around_player": {"do_fire_tick",
                                         "allow_fire_ticks_away_from_player"},
}

# Vanilla rules Mojang REMOVED in the registry rework (no new name at all):
# the port's 1.21.11+ registry cannot represent them, so a pre-rework
# version that has them shows a genuine (if minor) gap. For rework versions
# the lang file still carries their stale keys; the audit suppresses the
# finding there (see has_gamerule_rework).
RULES_REMOVED_IN_REGISTRY_REWORK = {
    "spawn_chunk_radius",
    "entities_with_passengers_can_use_portals",
    "game_loop_function",
}

# Gamerule names the port invented (not vanilla in any version): reported as
# informational, never as leaks. As of the 26.3 registry check every port
# rule traces to a vanilla rule via RULE_RENAMES/RULE_MERGE, so this set is
# currently empty — it exists so a future port-invented rule is a deliberate
# list entry, not a silent diff.
INVENTED_RULES = set()

NAMESPACE_RE = re.compile(r"^[a-z0-9._-]+$")
PATH_RE = re.compile(r"^[a-z0-9._/-]+$")
TEST_FILE_RE = re.compile(r"(^|_)tests?(_|\.)|_fixture\.rs$")
TEST_PATH_PARTS = frozenset({"tests", "examples"})

# --------------------------------------------------------------------------
# Optional import of the sibling tag_audit.py: its Rust tokenizer (test-range
# classification for files with hits) and its prepared-data inventory loader.
# Without it, test detection is file-level only and the prepared inventory
# is walked directly. The tag-literal scan is regex-based either way.

sys.path.insert(0, str(Path(__file__).resolve().parent))
try:
    import tag_audit as _tag_audit
except Exception:  # pragma: no cover - sibling missing
    _tag_audit = None


def _is_test_file(path: Path) -> bool:
    return (any(part in TEST_PATH_PARTS for part in path.parts)
            or TEST_FILE_RE.search(path.name) is not None)


def _test_ranges_for(text: str):
    """[(start_line, end_line)] of #[cfg(test)] / #[test] items, or []."""
    if _tag_audit is None:
        return []
    try:
        return _tag_audit._test_ranges(_tag_audit.tokenize_rust(text))
    except Exception:
        return []


# --------------------------------------------------------------------------
# Vanilla data inventories

def _camel_to_snake(name: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


def _tag_id_from_jar_path(path: str):
    """(registry, tag id) for data/<ns>/tags/<registry>/...json, or None."""
    parts = path.split("/")
    # data/<ns>/tags/<registry...>/<name>.json
    if len(parts) < 5 or parts[0] != "data" or parts[2] != "tags":
        return None
    namespace = parts[1]
    rest = parts[3:]
    name = rest[-1][:-len(".json")]
    if rest[0] == "worldgen":
        if len(rest) < 3:
            return None
        registry, name_parts = "worldgen/" + rest[1], rest[2:-1] + [name]
    else:
        registry, name_parts = rest[0], rest[1:-1] + [name]
    return registry, f"{namespace}:" + "/".join(name_parts)


def _open_game_jar(jar_path: Path):
    """ZipFile for the game jar: the jar itself, or the bundler's inner jar."""
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


def load_inventory_from_jar(jar_path: Path):
    """Tag/attribute/gamerule inventories from a vanilla jar.

    Returns (inventory, detail). `inventory` has keys: tags (set of tag ids),
    per_registry (registry -> count), attributes_present (bool),
    attribute_ids (set of attribute ids used by biome/dimension JSONs),
    gamerules (set of snake_case gamerule names, empty if no lang file).
    """
    game, _source = _open_game_jar(jar_path)
    names = game.namelist()
    tags: set[str] = set()
    per_registry: dict[str, int] = {}
    for path in names:
        if not (path.startswith("data/") and "/tags/" in path
                and path.endswith(".json")):
            continue
        parsed = _tag_id_from_jar_path(path)
        if parsed is None:
            continue
        registry, tag_id = parsed
        tags.add(tag_id)
        per_registry[registry] = per_registry.get(registry, 0) + 1
    # Environment attributes: the registry is code (EnvironmentAttributes'
    # <clinit> registers each id as a string constant), so the full set
    # comes from a class-file string scan, not from biome JSONs (which only
    # carry the ids some biome/dimension actually sets — 13 of 51 in 26.3).
    # The scan is scoped to the registry class: the class with the most
    # attribute-path strings that includes a `visual/` id (BuiltInLootTables
    # also carries many `gameplay/` strings, but no `visual/` ones). Works
    # on obfuscated jars too — obfuscation renames classes, not strings.
    attribute_ids: set[str] = set()
    attributes_present = False
    attr_pat = re.compile(rb"(?:visual|audio|gameplay)/[a-z0-9_/]+")
    best_class: set[str] = set()
    for path in names:
        if not path.endswith(".class"):
            continue
        try:
            found = {m.group(0).decode()
                     for m in attr_pat.finditer(game.read(path))}
        except (KeyError, ValueError):
            continue
        if found and any(f.startswith("visual/") for f in found) \
                and len(found) > len(best_class):
            best_class = found
    if best_class:
        attributes_present = True
        attribute_ids.update(f"minecraft:{a}" for a in best_class)
    # Biome/dimension JSONs: any "attributes" key confirms the system, and
    # their keys join the set (covers data-only ids, if any).
    for path in names:
        if not (path.startswith("data/minecraft/worldgen/")
                and path.endswith(".json")):
            continue
        try:
            data = json.loads(game.read(path))
        except (ValueError, KeyError):
            continue
        if isinstance(data, dict) and isinstance(data.get("attributes"), dict):
            attributes_present = True
            attribute_ids.update(str(k) for k in data["attributes"])
    gamerules: set[str] = set()
    lang_candidates = [n for n in names if n.endswith("lang/en_us.json")]
    for lang_path in lang_candidates:
        try:
            lang = json.loads(game.read(lang_path))
        except (ValueError, KeyError):
            continue
        for key in lang:
            if key.startswith("gamerule.") and "." not in key[len("gamerule."):]:
                gamerules.add(_camel_to_snake(key[len("gamerule."):]))
        if gamerules:
            break
    # The 1.21.11 registry rework: GameRuleRegistryFix exists in the jar
    # iff this version is at/after the rework (1.21.11+). Rework versions
    # still carry stale lang keys for the removed rules.
    has_gamerule_rework = any("GameRuleRegistryFix" in n for n in names)
    return {
        "tags": tags,
        "per_registry": per_registry,
        "attributes_present": attributes_present,
        "attribute_ids": attribute_ids,
        "gamerules": gamerules,
        "has_gamerule_rework": has_gamerule_rework,
    }, {"tag_files": len(tags)}


def load_inventory_from_prepared(prepared: Path):
    """Tag/attribute/gamerule inventories from a .local prepared directory."""
    tags: set[str] = set()
    per_registry: dict[str, int] = {}
    if _tag_audit is not None:
        union, per_reg, _sources, _namespaces = _tag_audit.load_tag_union(prepared)
        tags = set(union)
        per_registry = dict(per_reg)
    else:  # pragma: no cover - fallback when tag_audit is absent
        data_dir = prepared / "data"
        if data_dir.is_dir():
            for ns_dir in sorted(data_dir.iterdir()):
                tags_dir = ns_dir / "tags"
                if not tags_dir.is_dir():
                    continue
                for path in sorted(tags_dir.rglob("*.json")):
                    rel = path.relative_to(tags_dir)
                    parts = list(rel.parts[:-1]) + [rel.stem]
                    if parts[0] == "worldgen":
                        registry, name_parts = "worldgen/" + parts[1], parts[2:]
                    else:
                        registry, name_parts = parts[0], parts[1:]
                    if name_parts:
                        tags.add(f"{ns_dir.name}:" + "/".join(name_parts))
                        per_registry[registry] = per_registry.get(registry, 0) + 1
        for path in sorted(prepared.glob("*-tags.json")):
            try:
                data = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, ValueError):
                continue
            if isinstance(data, dict):
                registry = path.name[:-len("-tags.json")]
                for tag_id in data:
                    if isinstance(tag_id, str) and ":" in tag_id:
                        tags.add(tag_id)
                        per_registry[registry] = per_registry.get(registry, 0) + 1
    attribute_ids: set[str] = set()
    attributes_present = False
    data_dir = prepared / "data"
    if data_dir.is_dir():
        for path in sorted(data_dir.rglob("*.json")):
            if "worldgen" not in path.parts:
                continue
            try:
                data = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, ValueError):
                continue
            if isinstance(data, dict) and isinstance(data.get("attributes"), dict):
                attributes_present = True
                attribute_ids.update(str(k) for k in data["attributes"])
    gamerules: set[str] = set()
    for lang_path in sorted(prepared.rglob("en_us.json")):
        try:
            lang = json.loads(lang_path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        for key in lang:
            if key.startswith("gamerule.") and "." not in key[len("gamerule."):]:
                gamerules.add(_camel_to_snake(key[len("gamerule."):]))
        if gamerules:
            break
    return {
        "tags": tags,
        "per_registry": per_registry,
        "attributes_present": attributes_present,
        "attribute_ids": attribute_ids,
        "gamerules": gamerules,
        # Prepared data cannot show whether the rework landed (no jar to
        # check), so removed-rule findings are kept, with their note.
        "has_gamerule_rework": False,
    }, {"tag_files": len(tags)}


# --------------------------------------------------------------------------
# Code extraction

def _brace_block(text: str, start: int):
    """(body, end) of the `{...}` block opening at text[start] == '{'."""
    depth, i, body_start = 0, start, start + 1
    in_str, esc = False, False
    while i < len(text):
        ch = text[i]
        if in_str:
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
        elif ch == '"':
            in_str = True
        elif ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return text[body_start:i], i + 1
        i += 1
    return text[body_start:], len(text)


def _bracket_block(text: str, start: int):
    """(body, end) of the `[...]` block opening at text[start] == '['."""
    depth, i, body_start = 0, start, start + 1
    in_str, esc = False, False
    while i < len(text):
        ch = text[i]
        if in_str:
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
        elif ch == '"':
            in_str = True
        elif ch == "[":
            depth += 1
        elif ch == "]":
            depth -= 1
            if depth == 0:
                return text[body_start:i], i + 1
        i += 1
    return text[body_start:], len(text)


def _strip_comments(text: str) -> str:
    """Text with // and /* */ comments blanked (offsets preserved).

    String literals, raw strings and char literals are respected, so a
    comment marker inside a literal stays inert; lifetimes ('a) are not
    mistaken for char literals. Newlines are kept, so line numbers survive.
    """
    out = []
    i, n = 0, len(text)
    while i < n:
        ch = text[i]
        if ch == "/" and i + 1 < n and text[i + 1] == "/":
            j = text.find("\n", i)
            j = n if j == -1 else j
            out.append(" " * (j - i))
            i = j
        elif ch == "/" and i + 1 < n and text[i + 1] == "*":
            j = text.find("*/", i + 2)
            j = n if j == -1 else j + 2
            out.append("".join(" " if c != "\n" else "\n"
                               for c in text[i:j]))
            i = j
        elif ch == "r" and i + 1 < n and text[i + 1] in '"#':
            # Raw string r"..." / r#"..."# / r##"..."##: blank the whole
            # token (contents are data — JSON blobs, regexes — not code, so
            # a tag-like string inside one is not a lookup).
            hashes = 0
            j = i + 1
            while j < n and text[j] == "#":
                hashes += 1
                j += 1
            if j < n and text[j] == '"':
                end = text.find('"' + "#" * hashes, j + 1)
                end = n if end == -1 else end + 1 + hashes
                out.append("".join(" " if c != "\n" else "\n"
                                   for c in text[i:end]))
                i = end
            else:
                out.append(ch)
                i += 1
        elif ch == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    j += 1
                    break
                j += 1
            out.append(text[i:j])
            i = j
        elif ch == "'":
            # char literal 'x' / '\n' / '\u{...}' — not a lifetime ('a).
            j = i + 1
            if j < n and text[j] == "\\":
                while j < n and text[j] != "'":
                    j += 1
                j = min(j + 1, n)
                out.append(text[i:j])
                i = j
            elif j + 1 < n and text[j + 1] == "'":
                out.append(text[i:j + 2])
                i = j + 2
            else:
                out.append(ch)
                i += 1
        else:
            out.append(ch)
            i += 1
    return "".join(out)


def _read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def parse_tags_macro(tree: Path):
    """[(field, tag_name, line)] from the `tags!` macro in level/data.rs."""
    path = tree / "crates/horizon-world/src/level/data.rs"
    text = _read(path)
    match = re.search(r"\btags!\s*\{", text)
    if not match:
        return []
    body, _ = _brace_block(text, match.end() - 1)
    base_line = text.count("\n", 0, match.start()) + 1
    out = []
    for m in re.finditer(r"(\w+)\s*=\s*\"([^\"]+)\"\s*,", body):
        out.append((m.group(1), m.group(2),
                    base_line + body.count("\n", 0, m.start())))
    return out


def parse_fluid_tags(tree: Path):
    """[(field, tag_id, line)] from the FluidTags construction in data.rs."""
    path = tree / "crates/horizon-world/src/level/data.rs"
    text = _read(path)
    out = []
    for m in re.finditer(
            r"(\w+)\s*:\s*fluid_mask\(\s*\"(minecraft:[^\"]+)\"\s*\)", text):
        out.append((m.group(1), m.group(2),
                    text.count("\n", 0, m.start()) + 1))
    return out


def _rs_files(tree: Path):
    for root in (tree / "crates", tree / "launcher"):
        if root.is_dir():
            yield from sorted(root.rglob("*.rs"))


class TreeIndex:
    """One-pass load of a version tree's Rust sources.

    Files are read once and shared by every scanner; comment-stripping and
    test-range tokenization are lazy and cached per file (the two expensive
    steps — a naive per-scanner pass is ~5x slower).
    """

    def __init__(self, tree: Path):
        self.tree = tree
        self.files: dict[str, dict] = {}
        for path in _rs_files(tree):
            text = _read(path)
            if not text:
                continue
            rel = str(path.relative_to(tree))
            self.files[rel] = {"text": text, "clean": None,
                               "is_test": _is_test_file(path),
                               "ranges": None}

    def clean(self, rel: str) -> str:
        entry = self.files[rel]
        if entry["clean"] is None:
            entry["clean"] = _strip_comments(entry["text"])
        return entry["clean"]

    def is_test_line(self, rel: str, line: int) -> bool:
        entry = self.files[rel]
        if entry["is_test"]:
            return True
        if entry["ranges"] is None:
            entry["ranges"] = _test_ranges_for(entry["text"])
        return any(a <= line <= b for a, b in entry["ranges"])


def scan_field_usage(idx: TreeIndex, struct_name: str, fields: set[str]):
    """{field: [(file, line, test)]} for `<struct>.<field>` reads.

    Covers direct reads (`b.tags.leaves`, `self.b.fluid_tags.water`) and the
    local-alias form (`let t = &self.b.tags;` then `t.leaves`). The struct's
    own definition file is scanned too, but its macro/constructor lines do
    not match the read pattern.
    """
    usage: dict[str, list] = {f: [] for f in fields}
    direct = re.compile(r"\b" + re.escape(struct_name) + r"\.([A-Za-z_][A-Za-z0-9_]*)")
    alias_def = re.compile(
        r"let\s+(\w+)\s*=\s*&?[^;]*\." + re.escape(struct_name) + r"\s*;")
    for rel, entry in idx.files.items():
        if f".{struct_name}" not in entry["text"]:
            continue
        clean = idx.clean(rel)
        aliases = set(alias_def.findall(clean))
        alias_re = (re.compile(r"\b(?:" + "|".join(re.escape(a) for a in aliases)
                               + r")\.([A-Za-z_][A-Za-z0-9_]*)")
                    if aliases else None)
        seen: set[tuple[int, str]] = set()
        for i, line in enumerate(clean.split("\n"), 1):
            for m in direct.finditer(line):
                if m.group(1) in usage and (i, m.group(1)) not in seen:
                    seen.add((i, m.group(1)))
                    usage[m.group(1)].append(
                        (rel, i, idx.is_test_line(rel, i)))
            if alias_re:
                for m in alias_re.finditer(line):
                    if m.group(1) in usage and (i, m.group(1)) not in seen:
                        seen.add((i, m.group(1)))
                        usage[m.group(1)].append(
                            (rel, i, idx.is_test_line(rel, i)))
    return usage


def scan_tag_literals(idx: TreeIndex):
    """[(tag_id, file, line, test, form)] for tag string literals.

    Regex scan over comment-stripped text (fast; validated against
    tag_audit.scan_tree's tokenizer pass on 1.21.7 — identical
    production missing-tag sets, 2026-11-22; the only diffs are
    test-only: one malformed `enchantable/` literal tag_audit keeps and
    this scan rejects, and six test fixtures inside raw strings this
    scan sees and the tokenizer does not). Covers the call
    forms (tag/is_tag/has_tag/mark_tag/block_tag/biome_tag, including
    bare `tag(...)` and multi-line calls), the map forms
    (`<registry>_tags.get("...")`, e.g. item_tags/block_tags/feature_tags),
    and `#minecraft:...` literals. The scan runs over the whole file text
    (not per line) so multi-line calls are caught.
    """
    out = []
    call_re = re.compile(
        r"\b(?:tag|is_tag|has_tag|mark_tag|block_tag|biome_tag)"
        r"\s*\(\s*\"([^\"]+)\"")
    map_re = re.compile(r"\b(?:\w+_)?tags\s*\.\s*get\(\s*\"([^\"]+)\"")
    hash_re = re.compile(r"\"(#[A-Za-z0-9._:/-]+)\"")
    for rel, entry in idx.files.items():
        # horizon-port-pack's upgrade module migrates OLD-version data: its
        # tag strings name old-version things by design (e.g. the carver
        # replaceable tags it compares old configs against), not lookups
        # against this version's data.
        if rel.startswith("crates/horizon-port-pack/src/upgrade/"):
            continue
        text = entry["text"]
        # Pre-filter: every call form contains `tag(`, every map form
        # contains `tags`, every #-literal contains `"#`.
        if ("tag(" not in text and "tags" not in text
                and '"#' not in text):
            continue
        clean = idx.clean(rel)
        for m in call_re.finditer(clean):
            value = m.group(1)
            if ":" in value and _valid_tag_id(value):
                line = clean.count("\n", 0, m.start(1)) + 1
                call = re.match(r"\w+", m.group(0)).group(0)
                out.append((value, rel, line,
                            idx.is_test_line(rel, line), f"{call}()"))
        for m in map_re.finditer(clean):
            value = m.group(1)
            if ":" in value and _valid_tag_id(value):
                line = clean.count("\n", 0, m.start(1)) + 1
                map_name = re.match(r"\w+", m.group(0)).group(0)
                out.append((value, rel, line,
                            idx.is_test_line(rel, line), f"{map_name}.get()"))
        for m in hash_re.finditer(clean):
            value = m.group(1)[1:]
            if ":" in value and _valid_tag_id(value):
                line = clean.count("\n", 0, m.start(1)) + 1
                out.append((value, rel, line,
                            idx.is_test_line(rel, line), "#literal"))
    return out


def _valid_tag_id(value: str) -> bool:
    namespace, name = value.split(":", 1)
    if not NAMESPACE_RE.match(namespace) or not PATH_RE.match(name):
        return False
    return not (name.startswith("/") or name.endswith("/")
                or "//" in name)


def parse_attributes_table(tree: Path):
    """[(attr_id, line)] from the ATTRIBUTES table (ids without namespace)."""
    path = tree / "crates/horizon-world/src/environment_attributes.rs"
    text = _read(path)
    match = re.search(r"pub static ATTRIBUTES", text)
    if not match:
        return []
    brace = text.find("{", match.start())
    if brace == -1:
        return []
    body, _ = _brace_block(text, brace)
    base_line = text.count("\n", 0, brace) + 1
    return [(m.group(1), base_line + body.count("\n", 0, m.start()))
            for m in re.finditer(r"\ba\(\s*\"([^\"]+)\"", body)]


def scan_attribute_reads(idx: TreeIndex):
    """[(attr_id, file, line, test)] for `attributes[\"minecraft:...\"]`."""
    out = []
    read_re = re.compile(r"attributes\[\s*\"(minecraft:[^\"]+)\"")
    for rel, entry in idx.files.items():
        if "attributes[" not in entry["text"]:
            continue
        for i, line in enumerate(idx.clean(rel).split("\n"), 1):
            for m in read_re.finditer(line):
                out.append((m.group(1), rel, i, idx.is_test_line(rel, i)))
    return out


def parse_gamerules_table(tree: Path):
    """[(rule_name, file, line)] from the RULES + GAME_RULES tables."""
    out = []
    path = tree / "crates/horizon-client-core/src/gamerules.rs"
    text = _read(path)
    match = re.search(r"pub const RULES", text)
    if match:
        eq = text.find("=", match.start())
        bracket = text.find("&[", eq) if eq != -1 else -1
        if bracket != -1:
            body, _ = _bracket_block(text, bracket + 1)
            base_line = text.count("\n", 0, bracket) + 1
            for m in re.finditer(r"\b[bi]\(\s*\"([^\"]+)\"", body):
                out.append((m.group(1), str(path.relative_to(tree)),
                            base_line + body.count("\n", 0, m.start())))
    save = tree / "crates/horizon-client/src/save.rs"
    text = _read(save)
    match = re.search(r"const GAME_RULES", text)
    if match:
        eq = text.find("=", match.start())
        bracket = text.find("[", eq) if eq != -1 else -1
        if bracket != -1:
            body, _ = _bracket_block(text, bracket)
            base_line = text.count("\n", 0, bracket) + 1
            for m in re.finditer(r"\(\s*\"([^\"]+)\"\s*,", body):
                out.append((m.group(1), str(save.relative_to(tree)),
                            base_line + body.count("\n", 0, m.start())))
    # Standalone Rule consts outside the RULES array (e.g. MAX_MINECART_SPEED,
    # gated behind the minecart-improvements feature flag).
    text = _read(path)
    for m in re.finditer(r":\s*Rule\s*=\s*[bi]\(\s*\"([^\"]+)\"", text):
        out.append((m.group(1), str(path.relative_to(tree)),
                    text.count("\n", 0, m.start()) + 1))
    return out


def parse_world_gamerules_struct(tree: Path):
    """[(field, line)] from horizon-world's `GameRules` struct fields."""
    path = tree / "crates/horizon-world/src/level/mod.rs"
    text = _read(path)
    match = re.search(r"pub struct GameRules\s*\{", text)
    if not match:
        return []
    body, _ = _brace_block(text, match.end() - 1)
    base_line = text.count("\n", 0, match.start()) + 1
    return [(m.group(1), base_line + body.count("\n", 0, m.start()))
            for m in re.finditer(r"pub\s+(\w+)\s*:", body)]


def scan_gamerule_reads(idx: TreeIndex):
    """[(rule_name, file, line, test)] for string-based gamerule reads.

    Precision matters more than recall here: `.int(`/`.bool(`/`.get(` are
    generic method names (NBT compounds, properties readers, JSON). The
    GameRules API is `.int("name")`/`.bool("name")` (single string arg) on
    a `rules`/`game_rules`/`*_rules` receiver or on `r` (the struct-
    construction site in chat.rs), plus `.set(`/`.set_string(` and the
    `gamerules::rule("name")` lookup. `.get("...")` is deliberately
    excluded: every string-literal `.get(` on a rules-like receiver in
    the tree is an NBT read (the GameRules `.get` is only ever called
    with a variable). Namespaced names (`minecraft:...`) are save-data
    keys read by the datafix, not GameRules API reads.
    """
    out = []
    single_re = re.compile(
        r"(\w+)\.(int|bool)\(\s*\"([^\"]+)\"\s*\)")
    set_re = re.compile(
        r"(\w+)\.(set|set_string)\(\s*\"([^\"]+)\"\s*,")
    rule_re = re.compile(r"gamerules::rule\(\s*\"([^\"]+)\"")
    rules_receivers = {"rules", "game_rules"}
    for rel, entry in idx.files.items():
        text = entry["text"]
        if ".int(\"" not in text and ".bool(\"" not in text \
                and ".set(\"" not in text and ".set_string(\"" not in text \
                and "gamerules::rule(" not in text:
            continue
        for i, line in enumerate(idx.clean(rel).split("\n"), 1):
            for m in single_re.finditer(line):
                recv, name = m.group(1), m.group(3)
                if ":" in name:
                    continue
                if recv == "r" or recv in rules_receivers \
                        or recv.endswith("_rules"):
                    out.append((name, rel, i, idx.is_test_line(rel, i)))
            for m in set_re.finditer(line):
                recv, name = m.group(1), m.group(3)
                if ":" in name:
                    continue
                if recv in rules_receivers or recv.endswith("_rules"):
                    out.append((name, rel, i, idx.is_test_line(rel, i)))
            for m in rule_re.finditer(line):
                if ":" not in m.group(1):
                    out.append((m.group(1), rel, i,
                                idx.is_test_line(rel, i)))
    return out


# --------------------------------------------------------------------------
# Audit

JAR_CANDIDATES = ("{v}-server.jar", "{v}-server-named.jar",
                  "{v}-client-named.jar", "{v}-client.jar")


def resolve_data_source(tree: Path, version: str, prepared_arg, jar_arg,
                        jar_dir):
    """(kind, path, inventory, detail) — kind in jar/prepared/none."""
    if prepared_arg:
        prepared = Path(prepared_arg)
        if not prepared.is_dir():
            return ("none", prepared, None,
                    f"--prepared directory not found: {prepared}")
        inv, detail = load_inventory_from_prepared(prepared)
        if not inv["tags"]:
            return ("none", prepared, None,
                    f"no tag data in prepared dir {prepared}")
        return ("prepared", prepared, inv, detail)
    if jar_arg:
        jar = Path(jar_arg)
        if not jar.is_file():
            return ("none", jar, None, f"--jar file not found: {jar}")
        inv, detail = load_inventory_from_jar(jar)
        if not inv["tags"]:
            return ("none", jar, None, f"no tag data in jar {jar}")
        return ("jar", jar, inv, detail)
    local = tree / ".local" / "horizon-client" / version
    if local.is_dir():
        inv, detail = load_inventory_from_prepared(local)
        if inv["tags"]:
            return ("prepared", local, inv, detail)
    mc_dir = tree / ".local" / "minecraft" / version
    if mc_dir.is_dir():
        for jar in sorted(mc_dir.glob("*.jar")):
            try:
                inv, detail = load_inventory_from_jar(jar)
            except (zipfile.BadZipFile, OSError):
                continue
            if inv["tags"]:
                return ("jar", jar, inv, detail)
    if jar_dir:
        for pattern in JAR_CANDIDATES:
            jar = Path(jar_dir) / pattern.format(v=version)
            if jar.is_file():
                try:
                    inv, detail = load_inventory_from_jar(jar)
                except (zipfile.BadZipFile, OSError):
                    continue
                if inv["tags"]:
                    return ("jar", jar, inv, detail)
    return ("none", None, None,
            "no data source: no .local prepared data, no jar found "
            "(pass --prepared or --jar, or --jar-dir)")


def load_exceptions(path):
    if not path:
        return {}
    try:
        data = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise SystemExit(f"audit_26_3_leakage: bad exceptions file: {exc}")
    return data if isinstance(data, dict) else {}


def load_baseline(path):
    """{version: set(leak names)} from a baseline file (keys starting
    with '_' are metadata, not versions)."""
    if not path:
        return {}
    try:
        data = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise SystemExit(f"audit_26_3_leakage: bad baseline file: {exc}")
    if not isinstance(data, dict):
        raise SystemExit("audit_26_3_leakage: bad baseline file: not an object")
    return {version: set(names) for version, names in data.items()
            if not version.startswith("_")
            and isinstance(names, list)}


def _finding(family, kind, name, severity, locations, note=""):
    return {"family": family, "kind": kind, "name": name,
            "severity": severity, "locations": locations, "note": note}


def audit_version(tree: Path, version: str, prepared_arg=None, jar_arg=None,
                  jar_dir=None, exceptions=None):
    """Full audit for one version tree. Returns a result dict."""
    exceptions = exceptions or {}
    version_exceptions = exceptions.get(version, {})
    kind, source, inventory, detail = resolve_data_source(
        tree, version, prepared_arg, jar_arg, jar_dir)
    result = {
        "version": version,
        "tree": str(tree),
        "data_source": {"kind": kind,
                        "path": str(source) if source else None,
                        "detail": detail if kind == "none" else None},
        "status": "blocked",
        "inventory": {},
        "findings": [],
        "counts": {},
    }
    if inventory is None:
        result["reason"] = detail
        return result
    tags = inventory["tags"]
    namespaces = {t.split(":", 1)[0] for t in tags} | {"minecraft"}
    result["inventory"] = {
        "tags_total": len(tags),
        "per_registry": inventory["per_registry"],
        "attributes_present": inventory["attributes_present"],
        "attribute_ids": len(inventory["attribute_ids"]),
        "gamerules": len(inventory["gamerules"]),
        "data_source": kind,
    }
    findings: list[dict] = []

    def excepted(family, name):
        reason = (version_exceptions.get(family, {}) or {}).get(name)
        return reason

    idx = TreeIndex(tree)

    # --- tags: literal uses first (they count toward struct-field severity:
    # a macro field can be unused while a literal read of the same tag is
    # live — e.g. `copper` in 1.21.7: field unused, but golems.rs reads it
    # via block_tags.get()).
    by_tag: dict[str, list] = {}
    for tag_id, rel, line, test, form in scan_tag_literals(idx):
        if tag_id.split(":", 1)[0] not in namespaces:
            continue  # mod namespace: out of scope, not dead
        by_tag.setdefault(tag_id, []).append(
            {"file": rel, "line": line, "test": test, "form": form})

    # Literal forms that read the BLOCK registry (merged into a macro tag's
    # severity); other registries' literals (item_tags, feature_tags, ...)
    # are different tags that happen to share a name. Fluid tags have no
    # literal lookup form in the tree, so only #-literals merge there.
    BLOCK_LITERAL_FORMS = {"tag()", "has_tag()", "block_tag()",
                           "tags.get()", "block_tags.get()", "#literal"}
    FLUID_LITERAL_FORMS = {"#literal"}

    def _tag_severity(tag_id, field_uses, note_field, forms):
        """(severity, locations, note) for a missing tag, merging the
        struct-field uses and the registry-compatible literal uses of the
        same tag id."""
        lit = [loc for loc in by_tag.get(tag_id, [])
               if loc["form"] in forms]
        lit_prod = [loc for loc in lit if not loc["test"]]
        field_prod = [u for u in field_uses if not u[2]]
        reason = excepted("tags", tag_id)
        if reason is not None:
            severity = "accepted"
        elif field_prod or lit_prod:
            severity = "leak"
        elif field_uses or lit:
            severity = "test_only"
        else:
            severity = "dead_definition"
        locations = [{"file": f, "line": ln} for f, ln, _t in field_uses]
        locations += [{"file": loc["file"], "line": loc["line"]}
                      for loc in lit]
        via = []
        if field_prod or field_uses:
            via.append(f"field `{note_field}`")
        if lit:
            via.append(f"literal ({lit[0]['form']})")
        note = reason or ("tag " + " + ".join(via) if via else
                          f"tag field `{note_field}` (defined, never read)")
        return severity, locations, note

    # --- tags: macro-defined block tags (+ usage) -------------------------
    macro = parse_tags_macro(tree)
    fields = {field for field, _name, _line in macro}
    usage = scan_field_usage(idx, "tags", fields)
    for field, name, line in macro:
        tag_id = f"minecraft:{name}"
        if tag_id in tags:
            continue
        uses = usage.get(field, [])
        severity, locations, note = _tag_severity(
            tag_id, uses, field, BLOCK_LITERAL_FORMS)
        if not locations:
            locations = [{"file": "crates/horizon-world/src/level/data.rs",
                          "line": line}]
        findings.append(_finding(
            "tags", "missing_tag", tag_id, severity, locations, note))
    # --- tags: fluid tags --------------------------------------------------
    fluid = parse_fluid_tags(tree)
    ffields = {field for field, _tag, _line in fluid}
    fusage = scan_field_usage(idx, "fluid_tags", ffields)
    for field, tag_id, line in fluid:
        if tag_id in tags:
            continue
        uses = fusage.get(field, [])
        severity, locations, note = _tag_severity(
            tag_id, uses, field, FLUID_LITERAL_FORMS)
        if not locations:
            locations = [{"file": "crates/horizon-world/src/level/data.rs",
                          "line": line}]
        findings.append(_finding(
            "tags", "missing_tag", tag_id, severity, locations, note))
    # --- tags: string literals (tag() calls, #-literals) -------------------
    macro_names = {f"minecraft:{n}" for _f, n, _ln in macro}
    fluid_names = {t for _f, t, _ln in fluid}
    for tag_id in sorted(by_tag):
        if tag_id in tags:
            continue
        if tag_id in macro_names or tag_id in fluid_names:
            continue  # already reported via the struct-field path
        locs = by_tag[tag_id]
        prod = [loc for loc in locs if not loc["test"]]
        reason = excepted("tags", tag_id)
        if reason is not None:
            severity = "accepted"
        elif prod:
            severity = "leak"
        else:
            severity = "test_only"
        findings.append(_finding(
            "tags", "missing_tag", tag_id, severity,
            [{"file": loc["file"], "line": loc["line"]} for loc in locs],
            reason or f"tag literal ({locs[0]['form']})"))

    # --- attributes ---------------------------------------------------------
    attr_table = parse_attributes_table(tree)
    reads = scan_attribute_reads(idx)
    if not inventory["attributes_present"]:
        by_attr: dict[str, list] = {}
        for attr_id, rel, line, test in reads:
            if not test:
                by_attr.setdefault(attr_id, []).append(
                    {"file": rel, "line": line})
        for attr_id in sorted(by_attr):
            reason = excepted("attributes", attr_id)
            findings.append(_finding(
                "attributes", "attribute_system_absent", attr_id,
                "accepted" if reason else "review", by_attr[attr_id],
                reason or ("this version predates the environment-attribute "
                           "system (arrived 1.21.11); verify a legacy "
                           "fallback exists at each read site")))
        if attr_table:
            findings.append(_finding(
                "attributes", "table_without_system",
                f"{len(attr_table)} attributes registered",
                "note", [],
                "the ATTRIBUTES table is 26.3's registry; no data in this "
                "version will ever populate it (values come from defaults "
                "and legacy synthesis only)"))
    else:
        known = inventory["attribute_ids"]
        for attr_id, line in attr_table:
            full = f"minecraft:{attr_id}"
            if full not in known:
                reason = excepted("attributes", full)
                findings.append(_finding(
                    "attributes", "attribute_not_in_version_data", full,
                    "accepted" if reason else "review",
                    [{"file": "crates/horizon-world/src/environment_attributes.rs",
                      "line": line}],
                    reason or "registered but unused by any biome/dimension "
                              "in this version's data (may be a registry "
                              "attribute no biome uses — triage manually)"))
        by_attr = {}
        for attr_id, rel, line, test in reads:
            if not test and attr_id not in known:
                by_attr.setdefault(attr_id, []).append(
                    {"file": rel, "line": line})
        for attr_id in sorted(by_attr):
            reason = excepted("attributes", attr_id)
            findings.append(_finding(
                "attributes", "attribute_not_in_version_data", attr_id,
                "accepted" if reason else "review", by_attr[attr_id],
                reason or "read but unused by any biome/dimension in this "
                          "version's data"))

    # --- gamerules ----------------------------------------------------------
    table_rules = {name for name, _f, _ln in parse_gamerules_table(tree)}
    struct_fields = {f for f, _ln in parse_world_gamerules_struct(tree)}
    port_rules = table_rules | struct_fields
    vanilla_rules = set(inventory["gamerules"])
    if vanilla_rules:
        # A vanilla rule is covered if the port has it under its own name,
        # under its 26.x rename, or (for the fire rules) under the merged
        # rule. The lang inventory is old-camelCase names, snake_cased.
        merge_targets = {target for target, _srcs in RULE_MERGE.items()}
        merged_sources = {src for srcs in RULE_MERGE.values() for src in srcs}
        covered_vanilla = set()
        for rule in vanilla_rules:
            if rule in port_rules:
                covered_vanilla.add(rule)
            elif RULE_RENAMES.get(rule) in port_rules:
                covered_vanilla.add(rule)
            elif rule in merged_sources and merge_targets & port_rules:
                covered_vanilla.add(rule)
        for rule in sorted(vanilla_rules - covered_vanilla):
            # Rework versions carry stale lang keys for the removed rules;
            # the rules do not exist there, so there is nothing to flag.
            if rule in RULES_REMOVED_IN_REGISTRY_REWORK \
                    and inventory.get("has_gamerule_rework"):
                continue
            reason = excepted("gamerules", rule)
            if rule in RULES_REMOVED_IN_REGISTRY_REWORK:
                note = ("vanilla rule Mojang removed in the 1.21.11 registry "
                        "rework — the port's 1.21.11+ registry cannot "
                        "represent it")
            else:
                note = ("vanilla rule with no port equivalent: behaviour "
                        "gated on it cannot be honoured")
            findings.append(_finding(
                "gamerules", "missing_from_table", rule,
                "accepted" if reason else "review", [],
                reason or note))
        # A port rule is explained if any of its vanilla names (itself, its
        # old name(s) under the rename table, or the merge sources) exist
        # in this version's vanilla set.
        old_names: dict[str, set[str]] = {}
        for old, new in RULE_RENAMES.items():
            old_names.setdefault(new, set()).add(old)
        for target, srcs in RULE_MERGE.items():
            old_names.setdefault(target, set()).update(srcs)
        for rule in sorted(port_rules - vanilla_rules):
            if rule in INVENTED_RULES:
                severity, note = "note", "port-invented rule (not vanilla)"
            elif old_names.get(rule, set()) & vanilla_rules:
                continue  # rename/merge whose old name is this version's
            else:
                severity, note = ("note",
                                  "26.3-only rule carried into this version")
            reason = excepted("gamerules", rule)
            findings.append(_finding(
                "gamerules", "not_in_vanilla", rule,
                "accepted" if reason else severity, [],
                reason or note))
    # String reads naming a rule the port's own table lacks: the read can
    # never return a value.
    for rule, rel, line, test in scan_gamerule_reads(idx):
        if test or rule in port_rules:
            continue
        reason = excepted("gamerules", rule)
        findings.append(_finding(
            "gamerules", "unknown_rule_read", rule,
            "accepted" if reason else "leak",
            [{"file": rel, "line": line}],
            reason or "code reads a rule the port's table does not define"))

    counts: dict[str, int] = {}
    for finding in findings:
        counts[finding["severity"]] = counts.get(finding["severity"], 0) + 1
    result["findings"] = findings
    result["counts"] = counts
    if counts.get("leak"):
        result["status"] = "fail"
    elif counts.get("review"):
        result["status"] = "review"
    else:
        result["status"] = "pass"
    return result


# --------------------------------------------------------------------------
# CLI

def _version_key(version: str):
    parts = []
    for chunk in re.split(r"[.-]", version):
        parts.append((0, int(chunk)) if chunk.isdigit() else (1, chunk))
    return parts


def _print_human(result: dict):
    version = result["version"]
    inv = result["inventory"]
    source = result["data_source"]
    print(f"== {version} [{result['status']}] "
          f"(data: {source['kind']}"
          f"{' ' + source['path'] if source['path'] else ''})")
    if result["status"] == "blocked":
        print(f"   blocked: {result.get('reason')}")
        return
    print(f"   inventory: {inv['tags_total']} tags "
          f"({inv['per_registry'].get('block', 0)} block, "
          f"{inv['per_registry'].get('fluid', 0)} fluid), "
          f"attributes {'present' if inv['attributes_present'] else 'absent'} "
          f"({inv['attribute_ids']} ids in data), "
          f"{inv['gamerules']} gamerules")
    counts = result["counts"]
    if not counts:
        print("   clean: no findings")
        return
    print("   counts: " + ", ".join(f"{k} {v}" for k, v in sorted(counts.items())))
    for finding in result["findings"]:
        if finding["severity"] in ("note", "accepted", "dead_definition",
                                   "test_only"):
            continue
        locs = ", ".join(f"{loc['file']}:{loc['line']}"
                         for loc in finding["locations"][:3])
        print(f"   {finding['severity'].upper():6} [{finding['family']}] "
              f"{finding['name']} {locs}")
        if finding["note"]:
            print(f"          {finding['note']}")
    for severity in ("accepted", "dead_definition", "test_only", "note"):
        names = [f["name"] for f in result["findings"]
                 if f["severity"] == severity]
        if names:
            print(f"   {severity}: {len(names)} — "
                  + ", ".join(names[:12])
                  + (" ..." if len(names) > 12 else ""))


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description="Audit a version tree for 26.3-only logic (tags, "
                    "environment attributes, gamerules) it references but "
                    "its own vanilla data does not have.")
    parser.add_argument("version", nargs="?",
                        help="version id (e.g. 1.21.7); omit with --all")
    parser.add_argument("--all", action="store_true",
                        help="audit every version under versions/")
    parser.add_argument("--tree", help="version tree path "
                        "(default: versions/<version> under the repo root)")
    parser.add_argument("--root", default=".",
                        help="repo root (default: current directory)")
    parser.add_argument("--prepared", help="prepared data dir for the version")
    parser.add_argument("--jar", help="vanilla jar for the version")
    parser.add_argument("--jar-dir", help="directory of <v>-server.jar files")
    parser.add_argument("--exceptions",
                        help="exceptions JSON file (see tools/leakage-exceptions.json)")
    parser.add_argument("--baseline",
                        help="baseline JSON file: fail only on leaks that "
                             "are new against it (see tools/leakage-baselines.json)")
    parser.add_argument("--write-baseline",
                        help="write the current leak sets as a new baseline "
                             "file and exit 0 (review the diff before "
                             "committing it)")
    parser.add_argument("--json", action="store_true",
                        help="print the full result as JSON")
    args = parser.parse_args(argv)
    if bool(args.version) == bool(args.all):
        parser.error("give exactly one of <version> or --all")
    root = Path(args.root)
    exceptions = load_exceptions(args.exceptions)
    baseline = load_baseline(args.baseline)
    versions = []
    if args.all:
        versions_dir = root / VERSIONS_DIRNAME
        if not versions_dir.is_dir():
            parser.error(f"no {VERSIONS_DIRNAME}/ under {root}")
        versions = sorted((d.name for d in versions_dir.iterdir()
                           if d.is_dir() and (d / "Cargo.toml").exists()),
                          key=_version_key)
    else:
        versions = [args.version]
    results = []
    for version in versions:
        tree = Path(args.tree) if args.tree else root / VERSIONS_DIRNAME / version
        if not tree.is_dir():
            print(f"audit_26_3_leakage: no tree at {tree}", file=sys.stderr)
            return 2
        results.append(audit_version(
            tree, version, prepared_arg=args.prepared, jar_arg=args.jar,
            jar_dir=args.jar_dir, exceptions=exceptions))
    if args.write_baseline:
        out = {"_note": "Recorded leak sets (finding names) per version. "
                        "CI fails only on leaks NEW against this file. "
                        "Regenerate after a fix lands and review the diff: "
                        "a baseline that grows is a regression."}
        for result in results:
            leaks = sorted({f["name"] for f in result["findings"]
                            if f["severity"] == "leak"})
            out[result["version"]] = leaks
        Path(args.write_baseline).write_text(
            json.dumps(out, indent=2) + "\n", encoding="utf-8")
        print(f"wrote baseline for {len(results)} versions to "
              f"{args.write_baseline}")
        return 0
    if baseline:
        for result in results:
            current = {f["name"] for f in result["findings"]
                       if f["severity"] == "leak"}
            recorded = baseline.get(result["version"], set())
            result["baseline"] = {
                "new_leaks": sorted(current - recorded),
                "fixed_leaks": sorted(recorded - current),
            }
    if args.json:
        print(json.dumps({"results": results}, indent=2))
    else:
        for result in results:
            _print_human(result)
            if "baseline" in result:
                new = result["baseline"]["new_leaks"]
                fixed = result["baseline"]["fixed_leaks"]
                if new:
                    print(f"   NEW vs baseline ({len(new)}): "
                          + ", ".join(new[:10])
                          + (" ..." if len(new) > 10 else ""))
                if fixed:
                    print(f"   fixed vs baseline ({len(fixed)}): "
                          + ", ".join(fixed[:10])
                          + (" ..." if len(fixed) > 10 else ""))
        if args.all:
            print("\n== summary ==")
            header = f"{'version':<9} {'status':<8} {'leak':>4} {'review':>6} " \
                     f"{'notes':>5} {'data':<9}"
            if baseline:
                header += f" {'new':>4} {'fixed':>5}"
            print(header)
            for result in results:
                counts = result["counts"]
                notes = sum(counts.get(k, 0) for k in
                            ("note", "accepted", "dead_definition", "test_only"))
                line = (f"{result['version']:<9} {result['status']:<8} "
                        f"{counts.get('leak', 0):>4} {counts.get('review', 0):>6} "
                        f"{notes:>5} {result['data_source']['kind']:<9}")
                if "baseline" in result:
                    line += (f" {len(result['baseline']['new_leaks']):>4} "
                             f"{len(result['baseline']['fixed_leaks']):>5}")
                print(line)
    statuses = {r["status"] for r in results}
    if baseline:
        if any(r.get("baseline", {}).get("new_leaks") for r in results):
            return 1
    elif "fail" in statuses:
        return 1
    if "blocked" in statuses:
        return 3
    return 0


if __name__ == "__main__":
    sys.exit(main())
