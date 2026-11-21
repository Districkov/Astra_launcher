import os, sys

ROOT = r"C:\RAGEMP"
# Needles to locate the launch2.ip / domain-whitelist logic.
ASCII = [b"gta5rp", b"grandrp", b"launch2", b"launch2.ip", b"launch2.port", b"launch.ip"]
WIDE = [n.decode().encode("utf-16-le") for n in ASCII]

def scan_file(p):
    try:
        with open(p, "rb") as f:
            data = f.read()
    except Exception:
        return
    hits = []
    for label, needles in (("ASCII", ASCII), ("WIDE", WIDE)):
        for i, n in enumerate(needles):
            off = 0
            while True:
                j = data.find(n, off)
                if j < 0:
                    break
                ctx = data[max(0, j - 12): j + len(n) + 20]
                hits.append((label, ASCII[i].decode(), j, ctx))
                off = j + 1
    if hits:
        print(f"\n=== {p} ({len(data)} bytes) ===")
        for label, name, off, ctx in hits[:40]:
            print(f"  {label} '{name}' @0x{off:X}  ctx={ctx!r}")

for dirpath, _, files in os.walk(ROOT):
    for fn in files:
        if fn.lower().endswith((".exe", ".dll")):
            scan_file(os.path.join(dirpath, fn))
