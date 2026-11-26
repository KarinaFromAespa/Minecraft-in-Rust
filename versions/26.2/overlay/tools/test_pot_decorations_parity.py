#!/usr/bin/env python3
"""Game-path round-trip for `minecraft:pot_decorations`: NBT -> model -> NBT -> model.

The fixture path (item_model) reads raw JSON; this harness drives the game
path instead — `item_from_nbt` / `item_to_nbt` — and compares it against
vanilla's `PotDecorations.CODEC` on the same lists of item ids.

One documented equivalence: vanilla encodes `PotDecorations.EMPTY` as four
bricks, while the Rust model collapses all four sides undecorated to no
component, so the re-encoded component is absent. The two are equivalent
only because the decorated pot's default component is EMPTY — the oracle
pins that on its first output line, and the driver refuses to compare
unless both sides say `default EMPTY`. The normalisation below rewrites a
Rust `absent` to the four-brick list only when the decoded sides are all
empty, so it cannot mask a dropped or reordered sherd: those fail on the
sides fields, which are compared strictly.
"""
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess

try:
    from . import lab, test_rust_parity as base
except ImportError:
    import lab
    import test_rust_parity as base

BRICK = "minecraft:brick"
BRICK_LIST = '["minecraft:brick","minecraft:brick","minecraft:brick","minecraft:brick"]'
# All 23 pottery sherds in 26.2 (verified against the client jar's Items).
SHERDS = [
    "angler", "archer", "arms_up", "blade", "brewer", "burn", "danger",
    "explorer", "flow", "friend", "guster", "heart", "heartbreak", "howl",
    "miner", "mourner", "plenty", "prize", "scrape", "sheaf", "shelter",
    "skull", "snort",
]
SIDES = ["back", "left", "right", "front"]


def item(name):
    return f"minecraft:{name}_pottery_sherd"


def corpus():
    # Every sherd in every position, bricks elsewhere.
    for sherd in SHERDS:
        for i, side in enumerate(SIDES):
            ids = [BRICK] * 4
            ids[i] = item(sherd)
            yield (f"single-{sherd}-{side}", ids)
    yield ("all-brick", [BRICK] * 4)
    yield ("empty", [])
    yield ("short-1", [item("angler")])
    yield ("short-2", [item("angler"), item("blade")])
    yield ("short-3", [item("angler"), item("blade"), item("burn")])
    yield ("four-distinct", [item("angler"), item("blade"), item("burn"), item("danger")])
    yield ("four-same", [item("heart")] * 4)
    yield ("mixed-brick", [item("flow"), BRICK, item("prize"), BRICK])


def normalise_empty(rust_path):
    """Rewrite a Rust `absent` to the four-brick list, only when the decoded
    sides are all empty (see the module docstring). Keeps the raw output as
    `rust-raw.txt` for debugging."""
    raw = rust_path.read_text(encoding="utf-8").splitlines()
    (rust_path.parent / "rust-raw.txt").write_text(
        "\n".join(raw) + "\n", encoding="utf-8"
    )
    out = []
    for line in raw:
        fields = line.split("\t")
        if len(fields) == 4 and fields[1] == "-,-,-,-" and fields[2] == "absent":
            fields[2] = BRICK_LIST
        out.append("\t".join(fields))
    rust_path.write_text("\n".join(out) + "\n", encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepare", action="store_true")
    args = parser.parse_args()
    build = lab.LOCAL / "rust-pot-decorations-parity"
    build.mkdir(parents=True, exist_ok=True)
    report_path = build / "report.json"
    report_path.unlink(missing_ok=True)
    if args.prepare:
        lab.prepare()
    lock, jars, libraries = base.verify_artifacts()
    oracle = lab.ROOT / "mods/horizon/src/test/java/dev/horizonlab/parity/VanillaPotDecorationsOracle.java"
    sources = [
        oracle,
        Path(__file__),
        lab.ROOT / "tools/test_rust_parity.py",
        lab.ROOT / "tools/lab.py",
        lab.ROOT / "Cargo.toml",
        lab.ROOT / "Cargo.lock",
        lab.ROOT / "crates/horizon-items/Cargo.toml",
        *sorted((lab.ROOT / "crates/horizon-items/src").rglob("*.rs")),
        lab.ROOT / "crates/horizon-items/examples/pot_decorations_fixture.rs",
    ]
    tested_sources = base.source_hashes(sources)
    classes = build / "classes"
    classes.mkdir(exist_ok=True)

    def run(command, name):
        with (build / (name + ".log")).open("w", encoding="utf-8") as log:
            base.run(list(map(str, command)), stdout=log, stderr=subprocess.STDOUT)

    dependency_path = os.pathsep.join(map(str, libraries))
    run(
        [
            "javac",
            "--release",
            lock["minecraft"]["java_major"],
            "-proc:none",
            "-sourcepath",
            str(lab.ROOT / "mods/horizon/src/test/java"),
            "-cp",
            str(jars["server"]) + os.pathsep + dependency_path,
            "-d",
            classes,
            oracle,
        ],
        "compile-java",
    )

    def java(side, mode, path):
        run(
            [
                "java",
                "-Xmx512M",
                "-cp",
                os.pathsep.join([str(classes), str(jars[side]), dependency_path]),
                "dev.horizonlab.parity.VanillaPotDecorationsOracle",
                mode,
                path,
            ],
            side + "-" + mode,
        )

    fixtures = list(corpus())
    inputs = build / "inputs.txt"
    inputs.write_text(
        "".join(
            f"{label}\t{json.dumps(ids, separators=(',', ':'))}\n"
            for label, ids in fixtures
        ),
        encoding="utf-8",
    )
    fixture_paths = [inputs]
    tested_fixtures = base.source_hashes(fixture_paths)
    run(
        ["cargo", "build", "--locked", "-p", "horizon-items", "--example", "pot_decorations_fixture"],
        "compile-rust",
    )
    metadata = json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=lab.ROOT,
            text=True,
        )
    )
    binary = Path(metadata["target_directory"]) / "debug/examples" / (
        "pot_decorations_fixture.exe" if os.name == "nt" else "pot_decorations_fixture"
    )
    run([binary, inputs, build / "rust.txt"], "rust")
    normalise_empty(build / "rust.txt")
    references = []
    for side, jar in jars.items():
        java(side, side, build)
        reference = build / f"{side}.txt"
        lines = reference.read_text(encoding="utf-8").splitlines()
        if not lines or lines[0] != "default EMPTY":
            raise SystemExit(
                f"{side}: decorated pot default is not EMPTY "
                f"({lines[0] if lines else 'no output'}) — the harness's "
                "EMPTY rule no longer holds; re-examine the equivalence"
            )
        # The header pins an assumption; it is not compared output.
        reference.write_text("\n".join(lines[1:]) + "\n", encoding="utf-8")
        matched = base.compare(reference, build / "rust.txt")
        references.append(
            {
                "side": side,
                "jar_sha256": lab.digest(jar, "sha256"),
                "output_sha256": lab.digest(reference, "sha256"),
                "matched_output_lines": matched,
            }
        )
        print(f"{side}: {matched:,} pot decorations output lines match Rust", flush=True)
    base.ensure_sources_unchanged(sources, tested_sources)
    base.ensure_sources_unchanged(fixture_paths, tested_fixtures)
    report = {
        "schema": 1,
        "minecraft": lock["minecraft"]["version"],
        "manifest_sha1": lock["minecraft"]["manifest_sha1"],
        "platform": platform.platform(),
        "java": subprocess.check_output(["java", "--version"], text=True).splitlines()[0],
        "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
        "fixtures": len(fixtures),
        "sources_sha256": tested_sources,
        "fixtures_sha256": tested_fixtures,
        "rust_output_sha256": lab.digest(build / "rust.txt", "sha256"),
        "references": references,
        "scope": "Game-path pot_decorations round-trip (item_from_nbt/item_to_nbt) against vanilla's PotDecorations.CODEC: every sherd in every position, all-brick, empty and short lists, full and mixed combos. Decoded sides, the re-encoded list and the second decode are all compared. Documented equivalence: vanilla encodes EMPTY as four bricks while the Rust model collapses it to no component; the driver rewrites Rust 'absent' to the four-brick list only when the decoded sides are all empty, and asserts the decorated pot's default component is EMPTY on both jars first (raw Rust output kept as rust-raw.txt). Codec rejections (the 26.3 named-side map, lists over four entries, unknown item ids) are outside the differential comparison and are pinned by horizon-items unit tests instead. The block-entity sherds path (block_entity_nbt, spill, block_entity_item) is a separate surface and is not covered here.",
    }
    lab.write_json(report_path, report)
    print(f"Scoped pot decorations parity passed: {report_path}")


if __name__ == "__main__":
    main()
