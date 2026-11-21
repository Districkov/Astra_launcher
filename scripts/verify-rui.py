import struct

HEADER_END, TABLE2_BASE = 8488, 4240
src = open(r'C:\RAGEMP\rui\index.rui', 'rb').read()
u32 = lambda d, o: struct.unpack_from('<I', d, o)[0]


def parse(data):
    ents = []
    for k in range(132):
        ents.append(((2 * k + 1) * 16, (2 * k + 2) * 16))
    for k in range(133):
        ents.append((TABLE2_BASE + (2 * k) * 16, TABLE2_BASE + (2 * k + 1) * 16))
    full = [(sp, op, u32(data, sp), u32(data, op)) for sp, op in ents]
    full.sort(key=lambda e: e[3])          # sort by OFFSET (e[3]), not size
    blobs = [data[e[3]:e[3] + e[2]] for e in full]
    last = full[-1]
    return full, blobs, data[last[3] + last[2]:]


def serialize(full, blobs, trailing, header):
    offs = []
    cur = HEADER_END
    for b in blobs:
        offs.append(cur)
        cur += len(b)
    hdr = bytearray(header)
    for i, (sp, op, _, _) in enumerate(full):
        struct.pack_into('<I', hdr, sp, len(blobs[i]))
        struct.pack_into('<I', hdr, op, offs[i])
    return bytes(hdr) + b''.join(blobs) + trailing


header = src[:HEADER_END]
full, blobs, trailing = parse(src)

out = serialize(full, blobs, trailing, header)
print('source size  :', len(src))
print('repacked     :', len(out))
print('BYTE-EXACT   :', out == src)

app_i = next(i for i, b in enumerate(blobs) if b.startswith(b'(functio'))
anchor = b'_.servers.refresh)}'
print('anchor count :', blobs[app_i].count(anchor))

inj = (b'_.servers.refresh),setTimeout(()=>{rageApi.isALauncher&&this.launchGame('
       b'{ip:"89.35.130.191",port:22005,name:"ASTRA AutoConnect"},!0)},2000)}')

blobs2 = list(blobs)
blobs2[app_i] = blobs2[app_i].replace(anchor, inj)
out2 = serialize(full, blobs2, trailing, header)
print()
print('after inject :', len(out2), '(delta', len(out2) - len(src), 'bytes)')

full3, blobs3, trailing3 = parse(out2)
# offset + size == next offset
contig = all(full3[i][3] + full3[i][2] == full3[i + 1][3] for i in range(len(full3) - 1))
print('re-parse contiguous  :', contig)
print('entry count preserved:', len(blobs3) == len(blobs) == 265)
print('injection present    :', b'ASTRA AutoConnect' in blobs3[app_i])
print('other files intact   :', all(blobs3[i] == blobs[i] for i in range(len(blobs)) if i != app_i))
print('trailing preserved   :', trailing3 == trailing)
