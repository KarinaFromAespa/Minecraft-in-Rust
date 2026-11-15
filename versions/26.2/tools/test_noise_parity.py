#!/usr/bin/env python3
"""Compare 26.2 double-precision synth noise against both checksum-pinned vanilla JARs."""
import argparse
from collections import Counter
import itertools
import json
import math
import os
from pathlib import Path
import platform
import random
import struct
import subprocess

try:
    from . import lab, test_rust_parity
except ImportError:
    import lab
    import test_rust_parity


def number(value):
    return struct.pack(">d", value).hex()


def numbers(values):
    return " ".join(map(number, values))


def corpus():
    rng = random.Random(0x484f52495a4f4e4e4f495345)
    half = math.nextafter(2**24, 0)
    specials = [0.0, -0.0, 1.0, -1.0, .5, -.5, 1e-7, -1e-7,
                float.fromhex("0x0.0000000000001p-1022"), 1e-200, -1e-200,
                2**24, -(2**24), half, -half, math.nextafter(half, 0),
                math.nextafter(-half, -math.inf), 2**25, -(2**25),
                2147483647.0, -2147483648.0, 1e100, -1e100,
                float("inf"), float("-inf"), float("nan")]
    for value in specials:
        yield "wrap " + number(value)
    for _ in range(2000):
        yield "wrap " + (f"{rng.getrandbits(64):016x}" if rng.randrange(2) else number(rng.uniform(-1e10, 1e10)))
    bias_values = [0.0, -0.0, .5, -.5, 1.0, -1.0, 2.0, .1, 3.14159, 1e100, float("inf"), float("nan")]
    bias_factors = [0.0, .5, 1.0, -1.0, 10.0, 1e-200, float("inf"), float("nan")]
    for noise, factor in itertools.product(bias_values, bias_factors):
        yield "bias " + numbers([noise, factor])
    for _ in range(300):
        yield "bias " + numbers([rng.uniform(-10, 10), rng.uniform(-10, 10)])
    seeds = [-(2**63), -1, 0, 1, 2**63 - 1, *[rng.randrange(-2**63, 2**63) for _ in range(19)]]
    scales = [0.0, -0.0, .001, .125, .5, 1.0, 4.0, -.125, -4.0, 1e-200, 1e200, math.inf, -math.inf, math.nan]
    derivative_edges = [0.0, -0.0, 1.0, -1.0, math.inf, -math.inf, math.nan, 1e-300, 1e300]
    # ImprovedNoise: one octave, double sampling, smear, derivatives.
    improved_id = 0
    for stream in ["legacy", "xoroshiro"]:
        for scenario in range(12):
            seed = seeds[scenario % len(seeds)]
            yield f"create-improved {stream} {seed}"
            for step in range(24):
                if step % 4 == 0:
                    coords = [rng.choice(specials) for _ in range(3)]
                elif step % 4 == 1:
                    value = rng.choice(specials)
                    coords = [value, value, value]
                else:
                    magnitude = rng.choice([1, 16, 10000, 30_000_000, 2**31, 1e100])
                    coords = [rng.uniform(-magnitude, magnitude) for _ in range(3)]
                y_scale = scales[step % len(scales)]
                y_fudge = rng.choice(specials) if step % 3 == 0 else rng.uniform(-2, 2)
                initial = [rng.choice(derivative_edges) if step % 3 == 0 else rng.uniform(-1000, 1000) for _ in range(3)]
                yield f"query-improved {improved_id} " + numbers(coords + [y_scale, y_fudge] + initial)
            for step in range(8):
                coords = [rng.uniform(-2, 2) for _ in range(3)]
                y_scale = scales[step % len(scales)]
                y_fudge = rng.uniform(-2, 2)
                initial = [0.0, 0.0, 0.0]
                yield f"local-improved {improved_id} " + numbers(coords + [y_scale, y_fudge] + initial)
            improved_id += 1
    # PerlinNoise: new and legacy init, direct and octave-set forms.
    def legacy_direct_errors(first_octave, amplitudes):
        """Mirror the legacy initializer's throw order: the zero octave is
        built first, then octaves below it; a nonzero amplitude above the
        zero octave is never built, so the count check fails before the
        positive-octave check."""
        octaves = len(amplitudes)
        zero = -first_octave
        built = 0
        if 0 <= zero < octaves and amplitudes[zero] != 0.0:
            built += 1
        for i in range(zero - 1, -1, -1):
            if i < octaves and amplitudes[i] != 0.0:
                built += 1
        nonzero = sum(1 for a in amplitudes if a != 0.0)
        if built != nonzero:
            return True
        return zero < octaves - 1

    perlin_id = 0
    direct_cases = [
        (0, [-3, [1.0, 1.0, 1.0, 0.0]]),
        (0, [-2, [1.0]]),
        (0, [0, [1.0, 0.5, 0.25]]),
        (0, [-8, [1.0, 0.0, 1.0, 0.0, 1.0]]),
        (0, [2, [2.0, -1.0]]),
        (0, [-1, [0.0, 0.0]]),
        (0, [-4, []]),
        (0, [-16, [1.0] * 6]),
        (1, [-3, [1.0, 1.0, 1.0, 0.0]]),
        (1, [-2, [1.0]]),
        (1, [-8, [1.0, 0.0, 1.0]]),
        (1, [-1, [0.0, 0.0]]),
        (1, [1, [1.0]]),   # missing-levels error: zero octave discarded
        (1, [1, [0.0]]),   # positive-octaves error after the count check
        (1, [0, [1.0, 1.0]]),
        (1, [-4, []]),
    ]
    octave_cases = [
        (0, [0]),
        (0, [-2, -1, 0]),
        (0, [-15, -14, -13, -12, -11, -10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0]),
        (0, [-7, -6, -5, -4, -3, -2, -1, 0]),
        (0, [1]),
        (0, [0, 1, 2]),
        (0, [-3, 1]),
        (0, [5]),
        (0, []),            # empty-octaves error
        (1, [-15, -14, -13, -12, -11, -10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0]),
        (1, [-7, -6, -5, -4, -3, -2, -1, 0]),
        (1, [0]),
        (1, [-2, -1, 0]),
        (1, []),            # empty-octaves error
    ]
    for stream in ["legacy", "xoroshiro"]:
        for scenario, (init, (first_octave, amplitudes)) in enumerate(direct_cases):
            seed = seeds[scenario % len(seeds)]
            yield f"create-perlin {stream} {seed} {init} {first_octave} {len(amplitudes)} " + numbers(amplitudes)
            # Error cases create no sampler; only query the successes.
            if init == 1 and legacy_direct_errors(first_octave, amplitudes):
                continue
            for step in range(16):
                if step % 4 == 0:
                    coords = [rng.choice(specials) for _ in range(3)]
                else:
                    magnitude = rng.choice([1, 16, 10000, 2**31])
                    coords = [rng.uniform(-magnitude, magnitude) for _ in range(3)]
                y_scale = scales[step % len(scales)]
                y_fudge = rng.uniform(-2, 2)
                yield f"query-perlin {perlin_id} " + numbers(coords + [y_scale, y_fudge])
            yield f"perlin-maxbroken {perlin_id} " + number(rng.choice(scales))
            perlin_id += 1
        for scenario, (init, octaves) in enumerate(octave_cases):
            seed = seeds[(scenario + 5) % len(seeds)]
            yield f"create-perlin-octaves {stream} {seed} {init} {len(octaves)} " + " ".join(map(str, octaves))
            if not octaves:
                continue
            for step in range(12):
                magnitude = rng.choice([1, 16, 10000, 2**31])
                coords = [rng.uniform(-magnitude, magnitude) for _ in range(3)]
                y_scale = scales[step % len(scales)]
                y_fudge = rng.uniform(-2, 2)
                yield f"query-perlin {perlin_id} " + numbers(coords + [y_scale, y_fudge])
            yield f"perlin-maxbroken {perlin_id} " + number(rng.choice(scales))
            perlin_id += 1
    # SimplexNoise: 2D/3D without applied offsets.
    simplex_id = 0
    for stream in ["legacy", "xoroshiro"]:
        for scenario in range(8):
            seed = seeds[scenario % len(seeds)]
            yield f"create-simplex {stream} {seed}"
            for step in range(24):
                if step % 4 == 0:
                    coords = [rng.choice(specials) for _ in range(3)]
                elif step % 4 == 1:
                    value = rng.choice(specials)
                    coords = [value, value, value]
                else:
                    magnitude = rng.choice([1, 16, 10000, 2**31, 1e100])
                    coords = [rng.uniform(-magnitude, magnitude) for _ in range(3)]
                yield f"query-simplex {simplex_id} " + numbers(coords)
            simplex_id += 1
    # PerlinSimplexNoise: octave sets, both useNoiseStart forms.
    psimplex_id = 0
    psimplex_sets = [[0], [-2, -1, 0], [1], [0, 1, 2], [2, 3], [-3, -2, -1, 0, 1, 2, 3], [0, 2], [-1], []]
    for stream in ["legacy", "xoroshiro"]:
        for scenario, octaves in enumerate(psimplex_sets):
            seed = seeds[scenario % len(seeds)]
            yield f"create-perlin-simplex {stream} {seed} {len(octaves)} " + " ".join(map(str, octaves))
            if not octaves:
                continue
            for step in range(16):
                if step % 4 == 0:
                    coords = [rng.choice(specials) for _ in range(2)]
                else:
                    magnitude = rng.choice([1, 16, 10000, 2**31])
                    coords = [rng.uniform(-magnitude, magnitude) for _ in range(2)]
                yield f"query-perlin-simplex {psimplex_id} " + numbers(coords) + f" {step % 2}"
            psimplex_id += 1
    # NormalNoise: create + legacy Nether biome, value factor, sampling.
    for scenario in range(120):
        legacy = 1 if scenario % 4 == 0 else 0
        base = rng.randrange(-8, 3)
        count = rng.choice([0, 1, 2, 3, 4, 6])
        amplitudes = [rng.choice([0.0, -0.0, 1.0, .5, -1.0, 2.0, 1e-3, 1e16]) for _ in range(count)]
        if scenario % 31 == 0:
            amplitudes = [0.0] * count
        if scenario == 119:
            legacy, base, amplitudes = 1, 1, [1.0]  # legacy error case
        stream = "legacy" if scenario % 2 == 0 else "xoroshiro"
        seed = seeds[scenario % len(seeds)]
        points = [[0, 0, 0], [-17.25, 64.125, 32.5],
                  *[[rng.uniform(-1000, 1000) for _ in range(3)] for _ in range(4)]]
        yield (f"normal {stream} {seed} {legacy} {base} {len(amplitudes)} " + numbers(amplitudes)
               + f" {len(points)} " + numbers(sum(points, [])))
    # BlendedNoise: seeded + unseeded, overworld-like and extreme scales.
    blend_scales = [
        (1.0, 1.0, 80.0, 160.0, 1.0),
        (0.5, 2.0, 40.0, 80.0, 2.0),
        (2.0, 0.25, 160.0, 320.0, 0.5),
        (1.0, 1.0, 1.0, 1.0, 8.0),
    ]
    blend_points = [[0, 0, 0], [-17, 64, 32], [100, -60, -100], [1000000, -64, -1000000],
                    [-1000000, 320, 1000000], [7, 7, 7], [-1, -1, -1], [12345, 100, -54321]]
    for scenario, scales in enumerate(blend_scales):
        for stream in ["legacy", "xoroshiro"]:
            seed = seeds[scenario % len(seeds)]
            yield (f"blended {stream} {seed} " + numbers(scales)
                   + f" {len(blend_points)} " + " ".join(" ".join(map(str, p)) for p in blend_points))
    for scales in blend_scales[:2]:
        yield ("blended-unseeded " + numbers(scales)
               + f" {len(blend_points)} " + " ".join(" ".join(map(str, p)) for p in blend_points))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepare", action="store_true")
    args = parser.parse_args()
    build = lab.LOCAL / "rust-noise-parity"
    build.mkdir(parents=True, exist_ok=True)
    report_path = build / "report.json"
    report_path.unlink(missing_ok=True)
    if args.prepare:
        lab.prepare()
    lock, jars, libraries = test_rust_parity.verify_artifacts()
    commands = list(corpus())
    inputs = build / "inputs.txt"
    inputs.write_text("\n".join(commands) + "\n", encoding="utf-8")
    source = lab.ROOT / "mods/horizon/src/test/java/dev/horizonlab/parity/VanillaNoiseOracle.java"
    sources = [source, Path(__file__), lab.ROOT / "tools/test_rust_parity.py", lab.ROOT / "tools/lab.py",
               *sorted((lab.ROOT / "crates/horizon-core").rglob("*.rs")),
               lab.ROOT / "crates/horizon-core/Cargo.toml", lab.ROOT / "Cargo.toml", lab.ROOT / "Cargo.lock"]
    tested_sources = test_rust_parity.source_hashes(sources)
    classes = build / "classes"
    classes.mkdir(exist_ok=True)
    dependency_path = os.pathsep.join(map(str, libraries))
    run = test_rust_parity.run
    run(["javac", "--release", str(lock["minecraft"]["java_major"]), "-proc:none",
         "-cp", os.pathsep.join([str(jars["server"]), dependency_path]), "-d", str(classes), str(source)])
    run(["cargo", "build", "--release", "--bin", "horizon-noise-parity"])
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=lab.ROOT, text=True))
    binary = Path(metadata["target_directory"]) / "release" / ("horizon-noise-parity.exe" if os.name == "nt" else "horizon-noise-parity")
    candidate = build / "rust.txt"
    with inputs.open("rb") as stdin, candidate.open("wb") as stdout:
        run([str(binary)], stdin=stdin, stdout=stdout)
    references = []
    for side, jar in jars.items():
        reference = build / f"vanilla-{side}.txt"
        run(["java", "-Xmx512M", "-cp", os.pathsep.join([str(classes), str(jar), dependency_path]),
             "dev.horizonlab.parity.VanillaNoiseOracle", str(inputs), str(reference)])
        matched = test_rust_parity.compare(reference, candidate)
        references.append({"side": side, "jar_sha256": lab.digest(jar, "sha256"),
                           "output_sha256": lab.digest(reference, "sha256"), "matched_output_lines": matched})
        print(f"{side}: {matched:,} noise output lines match Rust", flush=True)
    test_rust_parity.ensure_sources_unchanged(sources, tested_sources)
    report = {"schema": 1, "minecraft": lock["minecraft"]["version"], "platform": platform.platform(),
              "java": subprocess.check_output(["java", "--version"], text=True).splitlines()[0],
              "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
              "manifest_sha1": lock["minecraft"]["manifest_sha1"], "commands": dict(Counter(line.split()[0] for line in commands)),
              "inputs_sha256": lab.digest(inputs, "sha256"), "rust_output_sha256": lab.digest(candidate, "sha256"),
              "sources_sha256": tested_sources, "references": references,
              "scope": "26.2 double-precision synth: coordinate wrap (long floor), NoiseUtils bias, ImprovedNoise offsets/permutations/smear/derivatives, PerlinNoise new + legacy initialization (direct and octave-set forms, error draw state), SimplexNoise 2D/3D without applied offsets, PerlinSimplexNoise octave construction incl. positive-octave seeding, NormalNoise value factor/maxValue/create + legacy Nether biome, BlendedNoise seeded + unseeded compute. Finite outputs and signed zeros compared by bits; NaNs canonicalized. No serialized noise definitions, density-function graphs, generated terrain, gameplay, rendering or performance claim."}
    lab.write_json(report_path, report)
    print(f"Scoped noise parity passed: {report_path}")


if __name__ == "__main__":
    main()
