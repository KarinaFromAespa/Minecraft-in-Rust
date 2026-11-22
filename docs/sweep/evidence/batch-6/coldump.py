"""Dump block columns from an MCA chunk (own code, read-only)."""
import sys, struct
sys.path.insert(0, '/home/hatch/workspace/mcir-work/sweep-batch-6-deliverables')
from nbtmini import read_region_chunks

def bits_for_palette(n, minimum):
    if n <= 1: return 0
    return max(minimum, (n - 1).bit_length())

def expected_longs(count, bits):
    per = 64 // bits
    return (count + per - 1) // per

def unpack(data, bits, count):
    if not data or bits == 0: return [0]*count
    out = []
    per = 64 // bits
    mask = (1 << bits) - 1
    for idx in range(count):
        v = data[idx // per] & 0xFFFFFFFFFFFFFFFF
        out.append((v >> ((idx % per) * bits)) & mask)
    return out

def state_name(p):
    if isinstance(p, str): return p
    if isinstance(p, dict):
        for k in ('Name', 'id', ''):
            if k in p and isinstance(p[k], str): return p[k]
        for v in p.values():
            if isinstance(v, str): return v
    return '?'

def column_blocks(chunk, lx, lz):
    """Return dict y -> block name for one column."""
    out = {}
    for sec in chunk.get('sections', []):
        sy = sec.get('Y', -999)
        if sy > 127: sy -= 256
        if sy < -4 or sy > 19: continue
        bs = sec.get('block_states', {})
        pal = bs.get('palette', [])
        data = bs.get('data', [])
        if not pal: continue
        names = [state_name(p) for p in pal]
        nbits = bits_for_palette(len(pal), 4)
        idx = unpack(data, nbits, 4096) if data else [0]*4096
        y0 = (sy + 4) * 16
        for ly in range(16):
            # block_states order: y*256 + z*16 + x
            out[y0 + ly - 64] = names[idx[ly*256 + lz*16 + lx]]
    return out

def show(path, cx, cz, cols):
    chunks = read_region_chunks(path)
    c = chunks.get((cx, cz))
    if c is None:
        print(path, (cx, cz), 'MISSING'); return
    print(f'--- {path} chunk ({cx},{cz}) status={c.get("Status")}')
    for (lx, lz) in cols:
        col = column_blocks(c, lx, lz)
        # compress runs
        runs = []
        prev = None; start = None; last = None
        for y in sorted(col):
            b = col[y]
            if b != prev:
                if prev is not None: runs.append((start, last, prev))
                prev = b; start = y
            last = y
        if prev is not None: runs.append((start, last, prev))
        # print only runs above y=60 plus top air boundary
        print(f'  col local ({lx},{lz}):')
        for (a, b, name) in runs:
            if b >= 55:
                print(f'    y {a:4d}..{b:4d}: {name}')
