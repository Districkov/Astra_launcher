import sys, pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_64

PATH = sys.argv[1] if len(sys.argv) > 1 else r"C:\RAGEMP\vinjector.dll"
NEEDLE = b"launch.ip"

pe = pefile.PE(PATH, fast_load=True)
base = pe.OPTIONAL_HEADER.ImageBase
data = open(PATH, "rb").read()

# locate string file offset
fo = data.find(NEEDLE)
if fo < 0:
    print("needle not found"); sys.exit(1)
print(f"file offset of '{NEEDLE.decode()}' = 0x{fo:X}")

# file offset -> VA
va = None
for s in pe.sections:
    start = s.PointerToRawData
    end = start + s.SizeOfRawData
    if start <= fo < end:
        va = base + s.VirtualAddress + (fo - start)
        print(f"in section {s.Name.rstrip(chr(0).encode()).decode()} VA=0x{va:X}")
        break
if va is None:
    print("no section for offset"); sys.exit(1)

# disassemble .text, find rip-relative refs to va
text = next((s for s in pe.sections if s.Name.rstrip(b"\x00") == b".text"), None)
if not text:
    print("no .text"); sys.exit(1)
tdata = text.get_data()
tva = base + text.VirtualAddress
md = Cs(CS_ARCH_X86, CS_MODE_64)
md.detail = True

print(f"\nscanning .text (VA 0x{tva:X}, {len(tdata)} bytes) for refs to 0x{va:X} ...")
refs = []
for insn in md.disasm(tdata, tva):
    for op in insn.operands:
        if op.type == 3:  # X86_OP_MEM
            if op.mem.base == 41 and op.mem.index == 0:  # RIP
                target = insn.address + insn.size + op.mem.disp
                if target == va:
                    refs.append(insn.address)
print(f"found {len(refs)} ref(s):", [hex(r) for r in refs])

# print disassembly around each ref
for r in refs:
    print(f"\n---- context around 0x{r:X} ----")
    off = r - tva
    start = max(0, off - 0x60)
    end = min(len(tdata), off + 0x80)
    for insn in md.disasm(tdata[start:end], tva + start):
        mark = " <== REF" if insn.address == r else ""
        print(f"0x{insn.address:X}: {insn.mnemonic:8} {insn.op_str}{mark}")
