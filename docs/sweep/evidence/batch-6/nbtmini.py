"""Minimal NBT + MCA reader (own code, read-only). Parses chunk structures refs/starts."""
import struct, zlib, gzip, sys, os

def parse_tag(b, i, t):
    if t == 0: return None, i
    if t == 1: return struct.unpack('>b', b[i:i+1])[0], i+1
    if t == 2: return struct.unpack('>h', b[i:i+2])[0], i+2
    if t == 3: return struct.unpack('>i', b[i:i+4])[0], i+4
    if t == 4: return struct.unpack('>q', b[i:i+8])[0], i+8
    if t == 5: return struct.unpack('>f', b[i:i+4])[0], i+4
    if t == 6: return struct.unpack('>d', b[i:i+8])[0], i+8
    if t == 7:
        n = struct.unpack('>i', b[i:i+4])[0]; return bytes(b[i+4:i+4+n]), i+4+n
    if t == 8:
        n = struct.unpack('>H', b[i:i+2])[0]; return b[i+2:i+2+n].decode('utf-8', 'replace'), i+2+n
    if t == 9:
        et = b[i]; n = struct.unpack('>i', b[i+1:i+5])[0]; i += 5; out = []
        for _ in range(n):
            v, i = parse_tag(b, i, et); out.append(v)
        return out, i
    if t == 10:
        out = {}
        while True:
            tt = b[i]; i += 1
            if tt == 0: return out, i
            nl = struct.unpack('>H', b[i:i+2])[0]; name = b[i+2:i+2+nl].decode('utf-8', 'replace'); i += 2+nl
            v, i = parse_tag(b, i, tt); out[name] = v
    if t == 11:
        n = struct.unpack('>i', b[i:i+4])[0]; return list(struct.unpack('>%di' % n, b[i+4:i+4+4*n])), i+4+4*n
    if t == 12:
        n = struct.unpack('>i', b[i:i+4])[0]; return list(struct.unpack('>%dq' % n, b[i+4:i+4+8*n])), i+4+8*n
    raise ValueError('tag %d at %d' % (t, i))

def parse_nbt(b):
    assert b[0] == 10
    nl = struct.unpack('>H', b[1:3])[0]
    v, _ = parse_tag(b, 3+nl, 10)
    return v

def read_region_chunks(path):
    raw = open(path, 'rb').read()
    out = {}
    for slot in range(1024):
        off = struct.unpack('>I', raw[slot*4:slot*4+4])[0]
        sector, count = off >> 8, off & 0xFF
        if sector == 0 or count == 0: continue
        start = sector*4096
        length = struct.unpack('>I', raw[start:start+4])[0]
        comp = raw[start+4]
        payload = raw[start+5:start+4+length]
        try:
            if comp == 2: nb = zlib.decompress(payload)
            elif comp == 1: nb = gzip.decompress(payload)
            elif comp == 3: nb = payload
            else: continue
            c = parse_nbt(nb)
            out[(c.get('xPos'), c.get('zPos'))] = c
        except Exception as e:
            print('WARN', path, slot, e, file=sys.stderr)
    return out
