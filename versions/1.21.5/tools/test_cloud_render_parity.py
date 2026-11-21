#!/usr/bin/env python3
"""Compare fancy cloud exterior/interior faces, nearest-first mesh traversal and the
fixed 32-cell mesh radius of 1.21.5's CloudRenderer with 1.21.5."""
try:
    from . import lab, oracle_harness as harness
except ImportError:
    import lab, oracle_harness as harness


def make_input(build):
    cases = []
    for x in (-130, -2, -1, 0, 1, 2, 130):
        for z in (-130, -2, -1, 0, 1, 2, 130):
            for bottom in (-5, -4, 0, 1):
                for empty in range(16):
                    cases.append(f"0 {x} {z} {bottom} {empty}\n")
    for radius in range(11):
        for bottom in (-5, -4, 0, 1):
            cases.append(f"1 {radius} 0 {bottom} 0\n")
    # 1.21.5 has no Cloud Distance option: the mesh radius is the fixed 32
    # cells, so the range input is ignored and every case traverses it.
    for cloud_range in (2, 3, 4, 7, 8, 12, 32, 64, 127, 128):
        cases.append(f"2 {cloud_range} 0 -5 0\n")
    inputs = build / "inputs.txt"
    inputs.write_text("".join(cases))
    return inputs, len(cases)


def main():
    harness.differential("cloud-render-parity", "VanillaCloudRenderOracle", "cloud_fixture", make_input,
                         "Fancy cloud face culling, interior winding flags and mesh traversal; not full framebuffer parity.",
                         sources=[lab.ROOT / "crates/horizon-client-render/src/clouds.rs"])


if __name__ == "__main__":
    main()
