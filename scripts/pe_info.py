import sys, math, pefile

def entropy(b):
    if not b: return 0.0
    from collections import Counter
    c = Counter(b); n = len(b)
    return -sum((v/n) * math.log2(v/n) for v in c.values())

FILES = sys.argv[1:] or [r"C:\RAGEMP\multiplayer.dll", r"C:\RAGEMP\multiplayer_g9.dll",
                         r"C:\RAGEMP\ragemp_v.exe", r"C:\RAGEMP\updater.exe", r"C:\RAGEMP\vinjector.dll"]

for path in FILES:
    print(f"\n===== {path} =====")
    try:
        pe = pefile.PE(path, fast_load=True)
    except Exception as e:
        print("  PE error:", e); continue
    print(f"  imagebase=0x{pe.OPTIONAL_HEADER.ImageBase:X} entry=0x{pe.OPTIONAL_HEADER.AddressOfEntryPoint:X}")
    for s in pe.sections:
        name = s.Name.rstrip(b"\x00").decode("ascii", "replace")
        raw = s.get_data()
        ent = entropy(raw)
        flag = "  <-- HIGH ENTROPY (packed?)" if ent > 7.2 and s.SizeOfRawData > 0x1000 else ""
        print(f"  {name:10} VA=0x{s.VirtualAddress:X} vsize=0x{s.Misc_VirtualSize:X} raw=0x{s.SizeOfRawData:X} ent={ent:.2f}{flag}")
    # quick packer signatures
    low = open(path, "rb").read().lower()
    for sig in (b"vmp0", b"vmp1", b"vmprotect", b"themida", b"enigma", b"upx0", b"upx1"):
        if sig in low:
            print(f"  signature: {sig.decode(errors='replace')}")
