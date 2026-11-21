# Third-Party Notices

This project incorporates code and technical findings derived from the
open-source project below. The original copyright notice and licence are
reproduced as required by the MIT Licence.

---

## tiny-ragemp-launcher

- **Source:** https://github.com/winnerchester/tiny-ragemp-launcher
- **Author:** winnerchester
- **Licence:** MIT

### What was ported

| Area | Location in this project | Description |
|------|--------------------------|-------------|
| `updater.exe` offline-bypass patch table | `src-tauri/src/patcher.rs` → `BUILTIN_PATCHES` | The six binary patches (offsets `0x9393`, `0x9603`, `0x98CB`, `0x98D9`, `0xA04D`, `0x99AA`) that skip the `cdn.rgsvc.io` check so `ragemp_v.exe` launches offline. |
| Registry direct-connect method | `src-tauri/src/patcher.rs` → `write_registry` | Writing `HKCU\Software\RAGE-MP\launch2.{ip,port}` and spawning `updater.exe` with the RAGE:MP folder as the working directory. |
| Domain-whitelist bypass | `src-tauri/src/patcher.rs` (hosts helpers), `src-tauri/src/rui.rs` | The finding that `ragemp_v.exe` only accepts `*.gta5rp.com` / `*.grandrp.com`, plus both documented workarounds: hosts-file mapping and raw-IP `launchGame(ip, port)` via the RUI interface. |
| RUI `index.rui` archive format and editor | `src-tauri/src/rui.rs` | A Rust port of `tools/build.py`: the two-table 265-entry archive layout, byte-exact repacking, and the `auto_connect`, `server_list`, `colors`, `texts`/`title` and image edit operations. |

### Upstream licence text

```
MIT License

Copyright (c) winnerchester

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Upstream disclaimer (reproduced)

> This project exists solely to keep a shutting-down launcher alive. It has
> **no commercial use and no monetization** behind it. I respect the rights of
> Rockstar Games (Take-Two Interactive) and RAGE:MP — if an official objection
> is raised, I will take the project down entirely. What is done here falls
> under **abandonware preservation**.

### Divergences from upstream

The Rust port is not a line-by-line translation. Intentional differences:

1. **Patch discovery by signature.** Upstream applies fixed offsets. This port
   validates the reference offset first, then falls back to a byte-signature
   scan so a newer `updater.exe` build can be patched after its offsets shift.
   Ambiguous or unknown builds are refused rather than mis-patched.
2. **Idempotent patching.** Already-applied patches are detected and skipped.
3. **Built-in patch table.** The table ships inside the binary so the launcher
   still works when the patch-hosting endpoint is unreachable.
4. **Longest-key-first colour replacement.** Upstream iterates its colour map in
   insertion order; this port sorts by descending key length so a short rule
   such as `#fff` cannot partially consume a longer one such as `#ffffff`.
5. **No Python dependency.** The archive editor is native Rust; there is no
   `tools/` Python step at runtime.
