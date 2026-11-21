import socket, os, re

HOST = "89.35.130.191"
print("== reverse DNS ==")
try:
    print(" ", socket.gethostbyaddr(HOST))
except Exception as e:
    print("  ", type(e).__name__, e)

# find master/announce/CDN URLs in client binaries (ascii + utf16)
ROOT = r"C:\RAGEMP"
NEEDLES = [b"rage.mp", b"master", b"cdn.", b"announce", b"https://", b"secure"]
for dirpath, _, files in os.walk(ROOT):
    for fn in files:
        if not fn.lower().endswith((".exe", ".dll", ".json", ".xml", ".js")):
            continue
        p = os.path.join(dirpath, fn)
        try:
            data = open(p, "rb").read()
        except Exception:
            continue
        found = []
        for n in NEEDLES:
            for enc_name, needle in (("A", n), ("W", n.decode().encode("utf-16-le"))):
                off = 0
                while True:
                    j = data.find(needle, off)
                    if j < 0: break
                    # grab a printable-ish window
                    ctx = data[max(0, j-40): j+80]
                    printable = "".join(chr(c) if 32 <= c < 127 else "." for c in ctx)
                    if "rage.mp" in printable.lower() or "master" in printable.lower() or "announce" in printable.lower() or "cdn." in printable.lower():
                        found.append(f"    {enc_name} {n.decode()} @0x{j:X}: {printable}")
                    off = j + 1
                    if len(found) > 30: break
        if found:
            print(f"\n== {p} ==")
            for line in found[:20]:
                print(line)
