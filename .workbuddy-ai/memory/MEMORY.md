# ASTRA Launcher — project notes

## Critical server identity (CORRECTED 2026-10-16)
- **Real RAGE:MP server: `89.35.130.191:22005`** (NOT `185.176.94.21` — that was a
  wrong IP used during all of dev and was the true cause of "doesn't see server").
- Public identity: `astra-rp.fun` (CDN `cdn.astra-rp.fun`).
- Connect method: **`connect_mode: "hosts"`** with a whitelisted
  `*.gta5rp.com` fake domain (`astra-89-35-130-191.gta5rp.com`). Rationale:
  `ragemp_v.exe` whitelist only accepts `*.gta5rp.com` / `*.grandrp.com`; a raw IP
  in the registry is deliberately rejected and drops you to the server browser.

## Hard-won facts (do not re-litigate)
- Editing `rui\index.rui` (any variant) makes this client build show a black "404
  File not found" CEF window. `rui_auto_connect` is OFF by default and must stay
  off. The client validates the archive by a mechanism not in the file.
- The hosts mapping must be written by ASTRA's **«Применить»** button
  (`apply_whitelist_bypass`, native UAC elevation) — the agent cannot write
  `C:\Windows\System32\drivers\etc\hosts` itself.
- Config (`config.json`) is loaded once at ASTRA startup; edit it only while
  ASTRA is closed, then restart.

## Build / version
- Tauri (Rust backend) + React/Svelte TS frontend. Unsigned NSIS installer.
- Version chain: v1.11.5 -> v1.12.0 -> v1.13.0..v1.13.4 (current installer at
  repo root: `ASTRA_Launcher_x64-setup.exe`). Repo defaults updated for v1.13.5.
