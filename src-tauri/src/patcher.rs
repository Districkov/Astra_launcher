/**
 * patcher.rs — RAGE:MP launcher core
 *
 * Ported from the open-source project `tiny-ragemp-launcher`
 * (https://github.com/winnerchester/tiny-ragemp-launcher, winnerchester).
 * The registry + updater-bypass technique is identical; this module adds a
 * built-in patch table (with version-aware offset rebasing), a hosts-file
 * bypass for the `ragemp_v.exe` domain whitelist, and an `index.rui`
 * auto-connect injector that uses a raw IP (which is *not* subject to the
 * whitelist).
 *
 * Flow:
 * 0. Ensure RAGE:MP is installed
 * 1. Resolve a connect target:  raw IP  or  whitelisted domain via hosts file
 * 2. Write registry  HKCU\Software\RAGE-MP\launch2.{ip,port}
 * 3. Patch updater.exe on disk (offline bypass, 6 patches)
 * 3b. (optional) Patch ragemp_v.exe on disk to neuter its launch2.ip
 *     whitelist, so a raw IP auto-connect works (connect_mode "raw")
 * 4. (optional) Inject auto_connect into rui\index.rui (raw IP method)
 * 5. Spawn updater.exe (cwd = RAGE:MP folder)
 */

use std::fs;
use std::io::Write;
use std::path::Path;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::logging::{log_error, log_info, log_warn};

/// A single binary patch for updater.exe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
    /// File offset — parsed as hex when prefixed with `0x`, otherwise decimal.
    pub offset: String,
    pub original: String,
    pub replacement: String,
}

/// Patch manifest downloaded from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchManifest {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub ragemp_version: String,
    #[serde(default)]
    pub gta_build: String,
    pub patches: Vec<Patch>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Built-in updater offline-bypass table
// ─────────────────────────────────────────────────────────────────────────────
//
// Source: tiny-ragemp-launcher README ("Updater Offline Bypass Patch").
// Verified byte-for-byte against updater.exe 967 680 bytes
// (RAGE:MP 1.1 / Sep 2025 build shipping with GTA V 1.0.3411).
//
// # | Offset | Original           | Patched              | Effect
// 1 | 0x9393 | A0 0F 00 00        | 32 00 00 00          | transfer timeout 4000ms → 50ms
// 2 | 0x9603 | 0F 8D BD 02 00 00  | E9 BE 02 00 00 90    | skip manifest retry loop
// 3 | 0x98CB | 0F 85 84 4B 00 00  | 90 90 90 90 90 90    | bypass connection error A
// 4 | 0x98D9 | 0F 85 76 4B 00 00  | 90 90 90 90 90 90    | bypass connection error B
// 5 | 0xA04D | 0F 8E DE 0A 00 00  | E9 DE 0A 00 00 90    | skip file downloads
// 6 | 0x99AA | 0F 84 96 04 00 00  | E9 82 11 00 00 90    | bypass manifest parse
//
// Offsets are absolute file offsets within one specific updater.exe build.
// A RAGE:MP update rebuilds the binary and shifts them, so we locate each
// patch by scanning for its `original` bytes and only trust the reference
// offset as a tie-breaker / sanity check.

/// Number of bytes considered significant when rebasing an offset by content
/// search. The full 6-byte sequence is used when available.
const ANCHOR_LEN: usize = 6;

/// The reference build the offsets below were taken from.
pub const UPSTREAM_PATCH_VERSION: &str = "1.1";
pub const UPSTREAM_PATCH_GTA_BUILD: &str = "1.0.3411";
/// Upstream project these patches were ported from (informational).
pub const UPSTREAM_PATCH_SOURCE: &str = "tiny-ragemp-launcher";

/// Exposed for the diagnostics panel so the UI can credit the upstream table.
pub fn upstream_patch_source() -> &'static str {
    UPSTREAM_PATCH_SOURCE
}

struct BuiltinPatch {
    name: &'static str,
    offset: u64,
    original: &'static str,
    replacement: &'static str,
}

const BUILTIN_PATCHES: &[BuiltinPatch] = &[
    BuiltinPatch {
        name: "updater_timeout_shorten",
        offset: 0x9393,
        original: "A0 0F 00 00",
        replacement: "32 00 00 00",
    },
    BuiltinPatch {
        name: "updater_skip_manifest_retry",
        offset: 0x9603,
        original: "0F 8D BD 02 00 00",
        replacement: "E9 BE 02 00 00 90",
    },
    BuiltinPatch {
        name: "updater_bypass_connection_error_a",
        offset: 0x98CB,
        original: "0F 85 84 4B 00 00",
        replacement: "90 90 90 90 90 90",
    },
    BuiltinPatch {
        name: "updater_bypass_connection_error_b",
        offset: 0x98D9,
        original: "0F 85 76 4B 00 00",
        replacement: "90 90 90 90 90 90",
    },
    BuiltinPatch {
        name: "updater_skip_file_downloads",
        offset: 0xA04D,
        original: "0F 8E DE 0A 00 00",
        replacement: "E9 DE 0A 00 00 90",
    },
    BuiltinPatch {
        name: "updater_bypass_manifest_parse",
        offset: 0x99AA,
        original: "0F 84 96 04 00 00",
        replacement: "E9 82 11 00 00 90",
    },
];

/// Built-in manifest used when no `patches_url` is configured on the server.
/// This keeps the launcher usable even if the patch-hosting endpoint is down.
pub fn builtin_patch_manifest() -> PatchManifest {
    PatchManifest {
        version: format!("builtin-{}", UPSTREAM_PATCH_VERSION),
        ragemp_version: UPSTREAM_PATCH_VERSION.to_string(),
        gta_build: UPSTREAM_PATCH_GTA_BUILD.to_string(),
        patches: BUILTIN_PATCHES
            .iter()
            .map(|b| Patch {
                name: b.name.to_string(),
                offset: format!("0x{:X}", b.offset),
                original: b.original.to_string(),
                replacement: b.replacement.to_string(),
            })
            .collect(),
    }
}

fn parse_hex_bytes(hex_str: &str) -> Vec<u8> {
    hex_str
        .split_whitespace()
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect()
}

fn parse_offset(offset_str: &str) -> u64 {
    let s = offset_str.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).unwrap_or(0)
    } else {
        s.parse().unwrap_or(0)
    }
}

/// Locate the real position of a patch inside a possibly-updated binary.
///
/// Returns the offset to use, or an error explaining why the patch cannot be
/// applied. Resolution order:
///   1. If the reference offset already holds `original` → use it (exact build).
///   2. Otherwise scan the binary for `original`:
///        - exactly one match  → rebase to it (login/log scan) and warn.
///        - no match, but the reference offset already holds `replacement`
///          → already patched, report as no-op.
///        - otherwise → hard error (incompatible build).
fn resolve_patch_offset(data: &[u8], patch: &Patch) -> Result<Option<usize>, String> {
    let reference = parse_offset(&patch.offset) as usize;
    let original = parse_hex_bytes(&patch.original);
    let replacement = parse_hex_bytes(&patch.replacement);

    if original.is_empty() || replacement.is_empty() {
        return Err(format!("invalid patch data for '{}'", patch.name));
    }
    if replacement.len() > original.len() {
        return Err(format!(
            "patch '{}' replacement ({} bytes) longer than original ({} bytes)",
            patch.name,
            replacement.len(),
            original.len()
        ));
    }

    // 1. Reference offset matches → exact known build.
    if reference + original.len() <= data.len()
        && &data[reference..reference + original.len()] == original.as_slice()
    {
        return Ok(Some(reference));
    }

    // Already patched at the reference site?
    if reference + replacement.len() <= data.len()
        && &data[reference..reference + replacement.len()] == replacement.as_slice()
    {
        log_warn!("Patch '{}' already applied at 0x{:X}", patch.name, reference);
        return Ok(None);
    }

    // 2. Content scan for `original`.
    let needle = &original[..original.len().min(ANCHOR_LEN)];
    let mut hits: Vec<usize> = Vec::new();
    if needle.len() >= 2 {
        let mut i = 0usize;
        while i + needle.len() <= data.len() {
            if &data[i..i + needle.len()] == needle {
                hits.push(i);
                if hits.len() > 8 {
                    break;
                }
            }
            i += 1;
        }
    }

    match hits.len() {
        0 => Err(format!(
            "patch '{}' not found — updater.exe is an unsupported build \
             (expected offset 0x{:X})",
            patch.name, reference
        )),
        1 => {
            let found = hits[0];
            log_warn!(
                "Patch '{}' rebased 0x{:X} → 0x{:X} (updater.exe differs from reference build)",
                patch.name,
                reference,
                found
            );
            Ok(Some(found))
        }
        n => Err(format!(
            "patch '{}' is ambiguous: {} candidate sites ({:X?}) — updater.exe build is not recognised",
            patch.name, n, hits
        )),
    }
}

/// Download patches from the server.
pub async fn download_patches(patches_url: &str) -> Result<PatchManifest, String> {
    if patches_url.is_empty() {
        return Err("Patches URL not configured".to_string());
    }

    log_info!("Downloading patches from {}", patches_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client
        .get(patches_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch patches: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Patches server returned status {}", resp.status()));
    }

    let manifest: PatchManifest = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse patches: {}", e))?;

    log_info!(
        "Patches v{}: {} patches, RAGE:MP={}",
        manifest.version,
        manifest.patches.len(),
        manifest.ragemp_version,
    );

    for patch in &manifest.patches {
        log_info!("  - {} @ {}", patch.name, patch.offset);
    }

    Ok(manifest)
}

/// Apply patches to updater.exe on disk.
///
/// The operation is **idempotent**: patches already present are detected and
/// skipped, so repeated launches never double-apply or corrupt the binary.
/// Returns the number of patches actually written this run.
pub fn apply_patches(updater_path: &Path, patches: &[Patch]) -> Result<usize, String> {
    log_info!("Applying {} patches to {:?}", patches.len(), updater_path);

    let mut data =
        fs::read(updater_path).map_err(|e| format!("Failed to read updater.exe: {}", e))?;

    let mut applied = 0usize;

    for patch in patches {
        let replacement = parse_hex_bytes(&patch.replacement);

        // Ok(Some(off)) → original bytes present, patch needed at `off`.
        // Ok(None)      → replacement already present, nothing to do.
        //
        // Err → this patch does not exist in the installed updater.exe build
        // (RAGE:MP shipped a new updater). Applying the rest would leave the
        // binary in a half-patched state, which is worse than leaving it
        // untouched — the file is only written when `applied > 0`, so bailing
        // out here keeps the original bytes intact and lets the launch go on
        // with an unpatched updater instead of failing outright.
        let target = match resolve_patch_offset(&data, patch) {
            Ok(Some(off)) => off,
            Ok(None) => continue,
            Err(e) => {
                log_warn!(
                    "Patch set does not match this updater.exe build ({}); \
                     skipping all patches and launching unpatched",
                    e
                );
                return Ok(0);
            }
        };

        // Re-validate the write window before touching the buffer.
        if target + replacement.len() > data.len() {
            return Err(format!(
                "patch '{}' resolved offset 0x{:X} exceeds file size {}",
                patch.name,
                target,
                data.len()
            ));
        }

        data[target..target + replacement.len()].copy_from_slice(&replacement);
        applied += 1;
        log_info!("Applied patch '{}' at 0x{:X}", patch.name, target);
    }

    if applied > 0 {
        fs::write(updater_path, &data)
            .map_err(|e| format!("Failed to write patched updater.exe: {}", e))?;
    }

    log_info!("{} patch(es) applied to {:?}", applied, updater_path);
    Ok(applied)
}

/// Report how many patches are already present, without modifying anything.
/// Returns (applied, total).
pub fn patch_status(updater_path: &Path, patches: &[Patch]) -> Result<(usize, usize), String> {
    if !updater_path.exists() {
        return Err("updater.exe not found".to_string());
    }
    let data = fs::read(updater_path).map_err(|e| format!("Failed to read updater.exe: {}", e))?;

    let mut done = 0usize;
    for patch in patches {
        // Ok(None) means the *replacement* bytes are present → already patched.
        // Err means neither form exists → unknown build, do not count.
        if let Ok(None) = resolve_patch_offset(&data, patch) {
            done += 1;
        }
    }
    Ok((done, patches.len()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Domain-whitelist bypass (hosts file)
// ─────────────────────────────────────────────────────────────────────────────
//
// `ragemp_v.exe` only accepts `*.gta5rp.com` / `*.grandrp.com` for launch2.ip.
// Mapping a whitelisted domain to the real server IP in the hosts file makes
// the client pass the check and still connect to us.
//
// Writing C:\Windows\...\etc\hosts requires administrator rights, so the
// check/write can be performed by re-launching ourselves elevated with
// `--hosts-map` / `--hosts-clean`.

pub const HOSTS_MARKER: &str = "# astra-launcher";

#[cfg(target_os = "windows")]
pub fn hosts_file_path() -> std::path::PathBuf {
    std::path::PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
}

#[cfg(not(target_os = "windows"))]
pub fn hosts_file_path() -> std::path::PathBuf {
    std::path::PathBuf::from("/etc/hosts")
}

/// A `domain -> ip` mapping we own (tagged with `HOSTS_MARKER`).
#[derive(Debug, Clone)]
pub struct HostsEntry {
    pub ip: String,
    pub domain: String,
}

/// Parse our own marked lines out of a hosts file body.
fn parse_our_hosts_entries(body: &str) -> Vec<HostsEntry> {
    let mut out = Vec::new();
    for line in body.lines() {
        if !line.contains(HOSTS_MARKER) {
            continue;
        }
        // Strip the marker and any comment that follows it, then split.
        let content = line.split('#').next().unwrap_or("").trim();
        let mut parts = content.split_whitespace();
        if let (Some(ip), Some(domain)) = (parts.next(), parts.next()) {
            out.push(HostsEntry {
                ip: ip.to_string(),
                domain: domain.to_string(),
            });
        }
    }
    out
}

/// Read the *current* IP mapping for a domain, if we own it.
pub fn hosts_entry_for(domain: &str) -> Option<HostsEntry> {
    let path = hosts_file_path();
    let body = fs::read_to_string(&path).ok()?;
    parse_our_hosts_entries(&body)
        .into_iter()
        .find(|e| e.domain.eq_ignore_ascii_case(domain))
}

/// Remove any of our previously-written lines. Returns the cleaned body.
pub fn strip_our_hosts_lines(body: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for line in body.lines() {
        if line.contains(HOSTS_MARKER) {
            continue;
        }
        kept.push(line);
    }
    let mut result = kept.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Pure transform: produce the new hosts body for the given entries.
/// Existing foreign lines are preserved; our own lines are replaced.
pub fn build_hosts_body(current: &str, entries: &[HostsEntry]) -> String {
    let mut body = strip_our_hosts_lines(current);
    if !entries.is_empty() {
        if !body.ends_with('\n') {
            body.push('\n');
        }
        body.push_str(&format!(
            "{} astra-launcher: RAGE:MP direct-connect mapping\n",
            HOSTS_MARKER
        ));
        for e in entries {
            body.push_str(&format!("{}\t{}\t{}\n", e.ip, e.domain, HOSTS_MARKER));
        }
    }
    body
}

/// Whether we can currently write the hosts file (probe by opening for append).
pub fn hosts_writable() -> bool {
    let path = hosts_file_path();
    fs::OpenOptions::new().append(true).open(&path).is_ok()
}

/// Apply (or clear, when `entries` is empty) our hosts mappings.
/// Requires administrator rights; call from an elevated process.
pub fn apply_hosts_mapping(entries: &[HostsEntry]) -> Result<(), String> {
    let path = hosts_file_path();
    let current = fs::read_to_string(&path).unwrap_or_default();
    let new_body = build_hosts_body(&current, entries);

    if new_body == current {
        log_info!("hosts file already up to date");
        return Ok(());
    }

    fs::write(&path, new_body)
        .map_err(|e| format!("Failed to write hosts file (need administrator): {}", e))?;

    log_info!(
        "hosts file updated: {} mapping(s) {}",
        entries.len(),
        if entries.is_empty() { "removed" } else { "written" }
    );
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Registry
// ─────────────────────────────────────────────────────────────────────────────
//
// Value names discovered by extracting **both ASCII and UTF-16LE** strings from
// `ragemp_v.exe` (registry names are wide strings, so an ASCII-only scan gives
// false negatives).
//
//   ragemp_v.exe reads : launch.ip, launch.port, launch.type, launch.password,
//                        launch.data0, launch.cefPort, game_v_path, rage_path
//   community launchers: launch2.ip, launch2.port
//
// We write BOTH families. Writing the extra values is harmless and makes the
// launcher work regardless of which name a given RAGE:MP build actually reads.

/// Registry key holding all RAGE:MP state.
pub const REG_KEY: &str = "Software\\RAGE-MP";

/// Value names read by `ragemp_v.exe` itself.
pub const RV_LAUNCH_IP: &str = "launch.ip";
pub const RV_LAUNCH_PORT: &str = "launch.port";
pub const RV_LAUNCH_TYPE: &str = "launch.type";

/// Value names used by the community launchers (tiny-ragemp-launcher et al).
pub const RV_LAUNCH2_IP: &str = "launch2.ip";
pub const RV_LAUNCH2_PORT: &str = "launch2.port";

/// `launch.type` value that means "connect straight to launch.ip".
pub const LAUNCH_TYPE_CONNECT: &str = "connect";

/// A snapshot of every direct-connect value we care about.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistryState {
    pub launch_ip: Option<String>,
    pub launch_port: Option<String>,
    pub launch_type: Option<String>,
    pub launch2_ip: Option<String>,
    pub launch2_port: Option<String>,
}

impl RegistryState {
    /// True when the `launch.*` family (what `ragemp_v.exe` reads) is set.
    pub fn has_launch_family(&self) -> bool {
        self.launch_ip.is_some()
    }
    /// True when the `launch2.*` family (what community launchers write) is set.
    pub fn has_launch2_family(&self) -> bool {
        self.launch2_ip.is_some()
    }
}

/// UTF-16, NUL-terminated — the encoding the Win32 registry API expects.
#[cfg(target_os = "windows")]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Little-endian bytes of a NUL-terminated UTF-16 string, for REG_SZ.
#[cfg(target_os = "windows")]
fn reg_sz_bytes(s: &str) -> Vec<u8> {
    s.encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(|c| c.to_le_bytes())
        .collect()
}

/// Write every direct-connect value for `ip`/`port`.
///
/// Both the `launch.*` and `launch2.*` families are written, plus
/// `launch.type = connect` so `ragemp_v.exe` knows this is a direct join.
#[cfg(target_os = "windows")]
pub fn write_registry(ip: &str, port: &str) -> Result<(), String> {
    use windows::Win32::System::Registry::*;

    log_info!(
        "Writing registry ({}): launch.ip={}, launch.port={}, launch.type={}, launch2.ip={}, launch2.port={}",
        REG_KEY,
        ip,
        port,
        LAUNCH_TYPE_CONNECT,
        ip,
        port
    );

    let values: [(&str, &str); 5] = [
        (RV_LAUNCH_IP, ip),
        (RV_LAUNCH_PORT, port),
        (RV_LAUNCH_TYPE, LAUNCH_TYPE_CONNECT),
        (RV_LAUNCH2_IP, ip),
        (RV_LAUNCH2_PORT, port),
    ];

    unsafe {
        let mut hkey = HKEY::default();

        let result = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            windows::core::w!("Software\\RAGE-MP"),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut hkey,
            None,
        );

        if result.is_err() {
            return Err(format!("Failed to create registry key: {}", result.to_hresult()));
        }

        for (name, value) in values {
            let name_w = to_wide(name);
            let bytes = reg_sz_bytes(value);

            let result = RegSetValueExW(
                hkey,
                windows::core::PCWSTR(name_w.as_ptr()),
                0,
                REG_SZ,
                Some(&bytes),
            );

            if result.is_err() {
                let _ = RegCloseKey(hkey);
                return Err(format!(
                    "Failed to write {}: {}",
                    name,
                    result.to_hresult()
                ));
            }
        }

        let _ = RegCloseKey(hkey);
    }

    log_info!("Registry written successfully");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn write_registry(_ip: &str, _port: &str) -> Result<(), String> {
    Err("Registry operations only supported on Windows".to_string())
}

/// Remove every direct-connect value we manage (used by "clear" / diagnostics).
#[cfg(target_os = "windows")]
pub fn clear_registry() -> Result<(), String> {
    use windows::Win32::System::Registry::*;

    let names = [
        RV_LAUNCH_IP,
        RV_LAUNCH_PORT,
        RV_LAUNCH_TYPE,
        RV_LAUNCH2_IP,
        RV_LAUNCH2_PORT,
    ];

    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            windows::core::w!("Software\\RAGE-MP"),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        )
        .is_err()
        {
            return Ok(()); // nothing to clear
        }

        for name in names {
            let name_w = to_wide(name);
            // Ignore "value not found" — the goal is simply to end up absent.
            let _ = RegDeleteValueW(hkey, windows::core::PCWSTR(name_w.as_ptr()));
        }

        let _ = RegCloseKey(hkey);
    }

    log_info!("Registry direct-connect values cleared");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_registry() -> Result<(), String> {
    Ok(())
}

/// Read one REG_SZ value.
#[cfg(target_os = "windows")]
unsafe fn read_value(hkey: windows::Win32::System::Registry::HKEY, name: &str) -> Option<String> {
    use windows::Win32::System::Registry::*;

    let name_w = to_wide(name);
    let name_pc = windows::core::PCWSTR(name_w.as_ptr());

    let mut ty = REG_NONE;
    let mut size: u32 = 0;
    if RegQueryValueExW(hkey, name_pc, None, Some(&mut ty), None, Some(&mut size)).is_err() {
        return None;
    }
    if size == 0 || size > 8192 {
        return None;
    }

    let mut buf = vec![0u8; size as usize];
    if RegQueryValueExW(
        hkey,
        name_pc,
        None,
        Some(&mut ty),
        Some(buf.as_mut_ptr()),
        Some(&mut size),
    )
    .is_err()
    {
        return None;
    }

    let u16s: Vec<u16> = buf
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&c| c != 0)
        .collect();

    let s = String::from_utf16(&u16s).ok()?;
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Read every direct-connect value.
#[cfg(target_os = "windows")]
pub fn read_registry_state() -> RegistryState {
    use windows::Win32::System::Registry::*;

    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            windows::core::w!("Software\\RAGE-MP"),
            0,
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return RegistryState::default();
        }

        let state = RegistryState {
            launch_ip: read_value(hkey, RV_LAUNCH_IP),
            launch_port: read_value(hkey, RV_LAUNCH_PORT),
            launch_type: read_value(hkey, RV_LAUNCH_TYPE),
            launch2_ip: read_value(hkey, RV_LAUNCH2_IP),
            launch2_port: read_value(hkey, RV_LAUNCH2_PORT),
        };

        let _ = RegCloseKey(hkey);
        state
    }
}

#[cfg(not(target_os = "windows"))]
pub fn read_registry_state() -> RegistryState {
    RegistryState::default()
}

// ─────────────────────────────────────────────────────────────────────────────
// Process spawning
// ─────────────────────────────────────────────────────────────────────────────

/// Generic spawn of an executable inside the RAGE:MP folder, no console window.
#[cfg(target_os = "windows")]
fn spawn_in(ragemp_dir: &Path, exe: &str) -> Result<u32, String> {
    let path = ragemp_dir.join(exe);

    if !path.exists() {
        return Err(format!("{} not found in RAGE:MP directory", exe));
    }

    log_info!("Spawning {} from {:?}", exe, ragemp_dir);

    let child = std::process::Command::new(path.to_string_lossy().to_string())
        .current_dir(ragemp_dir)
        .creation_flags(0x00000008) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| {
            log_error!("Failed to spawn {}: {}", exe, e);
            format!("Failed to spawn {}: {}", exe, e)
        })?;

    let pid = child.id();
    log_info!("{} spawned with PID={}", exe, pid);
    Ok(pid)
}

/// Spawn `updater.exe` **elevated** (UAC) and return its PID.
///
/// `ragemp_v.exe` refuses to start unless `updater.exe` has been launched as
/// an administrator at least once (it then shows the dialog "Please run
/// updater.exe as admin."). The six binary patches only disable the CDN
/// check inside `updater.exe` — they do **not** satisfy this "already run
/// as admin" requirement. So whenever `updater.exe` is present (the common
/// case), the launcher must start it elevated; with the patches applied it
/// completes initialization quickly (no CDN hang) and itself spawns
/// `ragemp_v.exe`, which is what actually starts the game.
#[cfg(target_os = "windows")]
pub fn spawn_updater(ragemp_dir: &Path) -> Result<u32, String> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::GetProcessId;
    use windows::Win32::UI::Shell::{
        ShellExecuteExW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
    };
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

    let updater = ragemp_dir.join("updater.exe");
    if !updater.exists() {
        return Err("updater.exe not found in RAGE:MP directory".to_string());
    }

    let to_wide = |s: &str| -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    };
    let verb = to_wide("runas");
    let file = to_wide(&updater.to_string_lossy());
    let dir = to_wide(&ragemp_dir.to_string_lossy());

    unsafe {
        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_FLAG_NO_UI,
            lpVerb: PCWSTR(verb.as_ptr()),
            lpFile: PCWSTR(file.as_ptr()),
            lpDirectory: PCWSTR(dir.as_ptr()),
            nShow: SW_HIDE.0,
            ..Default::default()
        };

        if let Err(e) = ShellExecuteExW(&mut info) {
            log_error!("elevated updater launch failed: {}", e);
            return Err(format!(
                "Не удалось запустить updater.exe от администратора (UAC): {}",
                e
            ));
        }

        let handle = info.hProcess;
        let pid = GetProcessId(handle);
        let _ = CloseHandle(handle);

        if pid == 0 {
            return Err("elevated updater launch returned no process id".to_string());
        }
        log_info!("updater.exe launched elevated, PID={}", pid);
        Ok(pid)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn spawn_updater(_ragemp_dir: &Path) -> Result<u32, String> {
    Err("updater.exe spawning only supported on Windows".to_string())
}

/// Spawn `ragemp_v.exe` (the actual client) directly.
///
/// This skips `updater.exe` — and therefore the CDN check — entirely.
/// `ragemp_v.exe` reads `launch.ip` / `launch.port` / `launch.type` from the
/// registry itself, so no updater round-trip is required.
///
/// Caveat: `ragemp_v.exe` contains the strings `Failed to start updater` and
/// `Please run updater.exe as admin.`, so it may try to spawn the updater on
/// its own. We deliberately do **not** delete or rename `updater.exe` — it is
/// left in place so that any such self-update attempt still finds it.
#[cfg(target_os = "windows")]
pub fn spawn_client(ragemp_dir: &Path) -> Result<u32, String> {
    spawn_in(ragemp_dir, "ragemp_v.exe")
}

#[cfg(not(target_os = "windows"))]
pub fn spawn_client(_ragemp_dir: &Path) -> Result<u32, String> {
    Err("ragemp_v.exe spawning only supported on Windows".to_string())
}

/// Which binary the launcher hands control to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchTarget {
    /// Official path: patch `updater.exe`, then run it. It performs the CDN
    /// check (which our patches bypass) and finally starts the client.
    Updater,
    /// Direct path: run `ragemp_v.exe` straight away, skipping the updater and
    /// the CDN check completely.
    RagempV,
}

impl Default for LaunchTarget {
    fn default() -> Self {
        // Updater first: it is the known-good path. Direct launch is opt-in
        // until the user has confirmed it works on their build.
        LaunchTarget::Updater
    }
}

impl LaunchTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            LaunchTarget::Updater => "updater",
            LaunchTarget::RagempV => "ragemp_v",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "ragemp_v" | "client" | "direct" => LaunchTarget::RagempV,
            _ => LaunchTarget::Updater,
        }
    }
}

/// Options controlling how `launch_ragemp` connects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchOptions {
    /// `"raw"` — write the raw server IP (relies on a `ragemp_v.exe` patch or
    /// the rui auto-connect bypass).
    /// `"hosts"` — write a whitelisted domain resolved via the hosts file.
    #[serde(default = "default_connect_mode")]
    pub connect_mode: String,
    /// Whitelisted domain to use when `connect_mode == "hosts"`.
    #[serde(default)]
    pub hosts_domain: String,
    /// Which binary to hand control to.
    #[serde(default)]
    pub target: LaunchTarget,
    /// Inject the rui auto-connect block into `rui\index.rui`.
    /// Off by default — opt-in (it has been reported to break the CEF UI).
    #[serde(default)]
    pub rui_auto_connect: bool,
    /// Binary patches applied to `ragemp_v.exe` itself (the game client), to
    /// neuter its `launch2.ip` whitelist so a raw IP auto-connect works. Empty
    /// by default — only effective when `connect_mode == "raw"` and the bytes
    /// match the installed client build.
    #[serde(default)]
    pub ragemp_v_patches: Vec<Patch>,
}

fn default_connect_mode() -> String {
    "raw".to_string()
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            connect_mode: default_connect_mode(),
            hosts_domain: String::new(),
            target: LaunchTarget::default(),
            rui_auto_connect: false,
            ragemp_v_patches: Vec::new(),
        }
    }
}

/// Full launch flow: resolve target → registry → patch → (rui) → spawn.
pub fn launch_ragemp(
    ragemp_dir: &Path,
    server_ip: &str,
    server_port: &str,
    patches: &[Patch],
    opts: &LaunchOptions,
) -> Result<serde_json::Value, String> {
    // 1. Resolve the value that goes into launch2.ip.
    let connect_host = if opts.connect_mode == "hosts" {
        let domain = if opts.hosts_domain.trim().is_empty() {
            format!("astra-{}", &server_ip.replace('.', "-"))
        } else {
            opts.hosts_domain.trim().to_string()
        };
        // Require the whitelisted suffix.
        if !is_whitelisted_domain(&domain) {
            return Err(format!(
                "'{}' is not a whitelisted domain — must end in .gta5rp.com or .grandrp.com",
                domain
            ));
        }
        // Verify the hosts mapping is actually in place.
        match hosts_entry_for(&domain) {
            Some(e) if e.ip == server_ip => {
                log_info!("hosts mapping verified: {} → {}", domain, server_ip);
            }
            _ => {
                return Err(format!(
                    "hosts mapping for '{}' is missing — apply the whitelist bypass first",
                    domain
                ));
            }
        }
        domain
    } else {
        server_ip.to_string()
    };

    // 2. Write registry (both launch.* and launch2.* families).
    write_registry(&connect_host, server_port)?;

    // 3. Patch updater.exe.
    //
    //    Only the Updater target actually runs the patched binary. For the
    //    direct target we still patch it when the file is present, so that any
    //    self-update attempt `ragemp_v.exe` makes will not hang on the dead CDN.
    let updater = ragemp_dir.join("updater.exe");
    let applied = if patches.is_empty() {
        log_warn!("no patches supplied — leaving updater.exe untouched");
        0
    } else if updater.exists() {
        apply_patches(&updater, patches)?
    } else if opts.target == LaunchTarget::Updater {
        return Err("updater.exe not found in RAGE:MP directory".to_string());
    } else {
        log_info!("updater.exe absent — direct launch does not need it");
        0
    };

    // 3b. Optionally patch ragemp_v.exe (the game client) to neuter its
    //     `launch2.ip` whitelist, so a raw IP auto-connect (connect_mode "raw")
    //     works without the fake-domain hosts hack — which otherwise breaks the
    //     secure connection. The updater spawns ragemp_v.exe, so patch it on
    //     disk before we hand control over.
    if !opts.ragemp_v_patches.is_empty() {
        let client = ragemp_dir.join("ragemp_v.exe");
        if client.exists() {
            match apply_patches(&client, &opts.ragemp_v_patches) {
                Ok(n) => log_info!("Applied {} ragemp_v.exe patch(es)", n),
                Err(e) => log_warn!("ragemp_v.exe patch failed: {}", e),
            }
        } else {
            log_warn!("ragemp_v.exe not found — skipping ragemp_v patch");
        }
    }

    // 4. Optional RUI auto-connect injection. Off by default: this client build
    //    shows a "404 File not found" CEF page after the archive is edited.
    //    The injection rewrites the archive tables, so it does not depend on
    //    bundle minification the way an in-place byte splice would.
    let rui_injected = if opts.rui_auto_connect {
        let port_num: u16 = server_port.parse().unwrap_or(22005);
        match crate::rui::apply_upstream_auto_connect(
            ragemp_dir,
            server_ip,
            port_num,
            "ASTRA",
            2000,
        ) {
            Ok(ok) => ok,
            Err(e) => {
                log_warn!("rui auto-connect injection failed: {}", e);
                false
            }
        }
    } else {
        false
    };

    // 5. Hand control to the chosen binary.
    //
    //    `ragemp_v.exe` refuses to run unless `updater.exe` has been launched
    //    as an administrator at least once ("Please run updater.exe as
    //    admin."). The 6-patch bypass only disables the CDN check; it does
    //    **not** satisfy this requirement. So whenever `updater.exe` is
    //    present — the common case — both launch targets run the *patched*
    //    `updater.exe` elevated. With the patches applied it completes
    //    initialization quickly (CDN is bypassed) and spawns `ragemp_v.exe`
    //    itself, which is what actually starts the game. Only when
    //    `updater.exe` is genuinely missing does the `ragemp_v` target truly
    //    skip it.
    let (pid, spawned) = if updater.exists() {
        if opts.target == LaunchTarget::RagempV {
            log_warn!(
                "ragemp_v.exe требует, чтобы updater.exe был запущен от админа; \
                 запускаю elevated updater.exe вместо прямого spawn"
            );
        }
        (spawn_updater(ragemp_dir)?, "updater.exe")
    } else if opts.target == LaunchTarget::Updater {
        return Err("updater.exe not found in RAGE:MP directory".to_string());
    } else {
        (spawn_client(ragemp_dir)?, "ragemp_v.exe")
    };

    log_info!(
        "RAGE:MP launched: PID={}, spawned={}, server={}:{}, mode={}, patches={}, rui_ac={}",
        pid,
        spawned,
        connect_host,
        server_port,
        opts.connect_mode,
        applied,
        rui_injected
    );

    Ok(serde_json::json!({
        "success": true,
        "pid": pid,
        "server": format!("{}:{}", server_ip, server_port),
        "connect_host": connect_host,
        "connect_mode": opts.connect_mode,
        "target": opts.target.as_str(),
        "spawned": spawned,
        "patches_applied": applied,
        "rui_auto_connect": rui_injected,
    }))
}

/// The `ragemp_v.exe` domain whitelist, as documented upstream.
pub fn is_whitelisted_domain(domain: &str) -> bool {
    let d = domain.trim().to_ascii_lowercase();
    (d == "gta5rp.com" || d.ends_with(".gta5rp.com"))
        || (d == "grandrp.com" || d.ends_with(".grandrp.com"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn patch(offset: u64, original: &str, replacement: &str) -> Patch {
        Patch {
            name: "test".into(),
            offset: format!("0x{:X}", offset),
            original: original.into(),
            replacement: replacement.into(),
        }
    }

    #[test]
    fn builtin_table_matches_upstream_documentation() {
        let m = builtin_patch_manifest();
        assert_eq!(m.patches.len(), 6, "upstream documents exactly six patches");

        // Spot-check the two most distinctive entries.
        let p2 = &m.patches[1];
        assert_eq!(p2.offset, "0x9603");
        assert_eq!(p2.original, "0F 8D BD 02 00 00");
        assert_eq!(p2.replacement, "E9 BE 02 00 00 90");

        let p6 = &m.patches[5];
        assert_eq!(p6.offset, "0x99AA");
        assert_eq!(p6.original, "0F 84 96 04 00 00");
        assert_eq!(p6.replacement, "E9 82 11 00 00 90");
    }

    #[test]
    fn replacement_never_grows_the_binary() {
        for p in builtin_patch_manifest().patches {
            let o = parse_hex_bytes(&p.original).len();
            let r = parse_hex_bytes(&p.replacement).len();
            assert!(r <= o, "{}: {} > {}", p.name, r, o);
        }
    }

    #[test]
    fn resolves_at_reference_offset() {
        let p = patch(0x10, "AA BB CC", "11 22 33");
        let mut data = vec![0u8; 0x40];
        data[0x10..0x13].copy_from_slice(&[0xAA, 0xBB, 0xCC]);
        assert_eq!(resolve_patch_offset(&data, &p).unwrap(), Some(0x10));
    }

    #[test]
    fn rebases_when_the_build_shifts() {
        // Same signature, different absolute position (a newer updater build).
        let p = patch(0x10, "AA BB CC", "11 22 33");
        let mut data = vec![0u8; 0x80];
        data[0x40..0x43].copy_from_slice(&[0xAA, 0xBB, 0xCC]);
        assert_eq!(resolve_patch_offset(&data, &p).unwrap(), Some(0x40));
    }

    #[test]
    fn detects_already_patched() {
        let p = patch(0x10, "AA BB CC", "11 22 33");
        let mut data = vec![0u8; 0x40];
        data[0x10..0x13].copy_from_slice(&[0x11, 0x22, 0x33]);
        assert_eq!(resolve_patch_offset(&data, &p).unwrap(), None);
    }

    #[test]
    fn errors_on_unknown_build() {
        let p = patch(0x10, "AA BB CC", "11 22 33");
        let data = vec![0x77u8; 0x40];
        assert!(resolve_patch_offset(&data, &p).is_err());
    }

    #[test]
    fn rejects_ambiguous_signature() {
        // Signature absent from the reference offset, present twice elsewhere.
        let p = patch(0x10, "AA BB", "11 22");
        let mut data = vec![0u8; 0x40];
        data[0x20..0x22].copy_from_slice(&[0xAA, 0xBB]);
        data[0x30..0x32].copy_from_slice(&[0xAA, 0xBB]);
        assert!(resolve_patch_offset(&data, &p).is_err());
    }

    #[test]
    fn reference_offset_wins_over_scan() {
        // When the reference site matches we trust it, even if the same bytes
        // appear elsewhere — this is the exact-build fast path.
        let p = patch(0x10, "AA BB", "11 22");
        let mut data = vec![0u8; 0x40];
        data[0x10..0x12].copy_from_slice(&[0xAA, 0xBB]);
        data[0x30..0x32].copy_from_slice(&[0xAA, 0xBB]);
        assert_eq!(resolve_patch_offset(&data, &p).unwrap(), Some(0x10));
    }

    #[test]
    fn whitelist_matches_upstream_rules() {
        for ok in [
            "gta5rp.com",
            "myserver.gta5rp.com",
            "grandrp.com",
            "play.grandrp.com",
            "MY.GTA5RP.COM",
        ] {
            assert!(is_whitelisted_domain(ok), "{} should pass", ok);
        }
        for bad in [
            "play.xxxx.com",
            "evil-gta5rp.com",
            "gta5rp.com.evil.com",
            "notgrandrp.com",
            "185.176.94.21",
        ] {
            assert!(!is_whitelisted_domain(bad), "{} should be rejected", bad);
        }
    }

    #[test]
    fn hosts_body_adds_and_removes_our_lines() {
        let base = "127.0.0.1 localhost\n";
        let entries = vec![HostsEntry {
            ip: "185.176.94.21".into(),
            domain: "astra.gta5rp.com".into(),
        }];

        let added = build_hosts_body(base, &entries);
        assert!(added.contains("127.0.0.1 localhost"), "foreign lines preserved");
        assert!(added.contains("185.176.94.21\tastra.gta5rp.com"));

        // Re-applying must not duplicate our lines.
        let again = build_hosts_body(&added, &entries);
        assert_eq!(again.matches("astra.gta5rp.com").count(), 1);

        // Clearing removes only our lines.
        let cleared = build_hosts_body(&added, &[]);
        assert!(!cleared.contains(HOSTS_MARKER));
        assert!(cleared.contains("127.0.0.1 localhost"));
    }

    // NOTE: the RUI payload/archive tests moved to `rui.rs`, which now owns
    // the archive format. Patching and rui editing are deliberately separate
    // concerns.

    #[test]
    fn launch_target_parses_aliases() {
        assert_eq!(LaunchTarget::parse("ragemp_v"), LaunchTarget::RagempV);
        assert_eq!(LaunchTarget::parse("client"), LaunchTarget::RagempV);
        assert_eq!(LaunchTarget::parse("direct"), LaunchTarget::RagempV);
        assert_eq!(LaunchTarget::parse("RAGEMP_V"), LaunchTarget::RagempV);
        assert_eq!(LaunchTarget::parse("  ragemp_v  "), LaunchTarget::RagempV);

        // Anything unrecognised (including empty) falls back to the safe path.
        assert_eq!(LaunchTarget::parse("updater"), LaunchTarget::Updater);
        assert_eq!(LaunchTarget::parse(""), LaunchTarget::Updater);
        assert_eq!(LaunchTarget::parse("garbage"), LaunchTarget::Updater);
    }

    #[test]
    fn launch_target_default_is_the_official_path() {
        // Direct launch must be opt-in — it is unverified on new builds.
        assert_eq!(LaunchTarget::default(), LaunchTarget::Updater);
    }

    #[test]
    fn launch_target_roundtrips_through_str() {
        for t in [LaunchTarget::Updater, LaunchTarget::RagempV] {
            assert_eq!(LaunchTarget::parse(t.as_str()), t);
        }
    }

    #[test]
    fn registry_state_family_detection() {
        let mut s = RegistryState::default();
        assert!(!s.has_launch_family());
        assert!(!s.has_launch2_family());

        s.launch2_ip = Some("1.2.3.4".into());
        assert!(s.has_launch2_family(), "launch2 family set");
        assert!(!s.has_launch_family(), "launch family still unset");

        s.launch_ip = Some("1.2.3.4".into());
        assert!(s.has_launch_family(), "launch family now set");
    }

    #[test]
    fn registry_value_names_match_the_binary() {
        // These names were extracted from ragemp_v.exe — changing them silently
        // would break direct connect.
        assert_eq!(RV_LAUNCH_IP, "launch.ip");
        assert_eq!(RV_LAUNCH_PORT, "launch.port");
        assert_eq!(RV_LAUNCH_TYPE, "launch.type");
        assert_eq!(RV_LAUNCH2_IP, "launch2.ip");
        assert_eq!(RV_LAUNCH2_PORT, "launch2.port");
        assert_eq!(LAUNCH_TYPE_CONNECT, "connect");
        assert_eq!(REG_KEY, "Software\\RAGE-MP");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn reg_sz_encoding_is_utf16le_nul_terminated() {
        let bytes = reg_sz_bytes("22005");
        // 5 chars * 2 bytes + 2-byte terminator
        assert_eq!(bytes.len(), 12);
        assert_eq!(&bytes[..2], &[0x32, 0x00]); // '2'
        assert_eq!(&bytes[10..], &[0x00, 0x00]); // NUL
    }

    /// Smoke test against the **real** installed `C:\RAGEMP\updater.exe`.
    /// Confirms the built-in patch table actually fits the binary shipped with
    /// the game (signature-based offsets, version-agnostic). Copies to a temp
    /// file so the real binary is never modified. Skipped silently when the
    /// path does not exist (CI / machines without the game).
    #[test]
    #[cfg(target_os = "windows")]
    fn builtin_patches_apply_to_real_rage_install() {
        use std::fs;
        let real = r"C:\RAGEMP\updater.exe";
        if !std::path::Path::new(real).exists() {
            eprintln!("[skip] {} not found", real);
            return;
        }
        let tmp = std::env::temp_dir().join("astra-updater-patchcheck.exe");
        fs::copy(real, &tmp).expect("copy real updater.exe to temp");

        let manifest = builtin_patch_manifest();

        let (before, _total) = patch_status(&tmp, &manifest.patches).expect("status before");
        let newly_applied = apply_patches(&tmp, &manifest.patches).expect("apply patches");
        let (after, total) = patch_status(&tmp, &manifest.patches).expect("status after");

        // The patch table must fit the shipped binary, and applying must be
        // idempotent regardless of the starting state.
        assert_eq!(after, total, "after patches: {}/{}", after, total);
        assert_eq!(
            after as i64 - before as i64,
            newly_applied as i64,
            "applied count inconsistent with status delta ({} -> {})",
            before,
            after
        );

        let _ = fs::remove_file(&tmp);
        eprintln!(
            "[ok] built-in patch table fits real C:\\RAGEMP\\updater.exe: {}/{} (was {}/{} before apply)",
            after, total, before, total
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RAGE:MP download
// ─────────────────────────────────────────────────────────────────────────────

/// Download RAGE:MP installer to target directory.
pub async fn download_ragemp(
    download_url: &str,
    target_dir: &Path,
    window: &tauri::Window,
) -> Result<String, String> {
    if download_url.is_empty() {
        return Err("RAGE:MP download URL not configured".to_string());
    }

    log_info!("Downloading RAGE:MP from {} to {:?}", download_url, target_dir);

    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create RAGE:MP directory: {}", e))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server returned status {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);

    let filename = response
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.split("filename=")
                .last()
                .map(|s| s.trim().trim_matches('"').to_string())
        })
        .or_else(|| download_url.rsplit('/').next().map(|s| s.to_string()))
        .unwrap_or_else(|| "RAGE-MP.zip".to_string());

    let dest_path = target_dir.join(&filename);
    let is_zip = filename.to_lowercase().ends_with(".zip");

    if is_zip {
        let zip_path = std::env::temp_dir().join("astra_ragemp.zip");
        let mut file =
            fs::File::create(&zip_path).map_err(|e| format!("Failed to create temp file: {}", e))?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Read error: {}", e))?;
            file.write_all(&chunk).map_err(|e| format!("Write error: {}", e))?;
            downloaded += chunk.len() as u64;

            let percent = if total_size > 0 {
                (downloaded as f64 / total_size as f64 * 100.0) as u32
            } else {
                0
            };

            let _ = window.emit(
                "ragemp-download-progress",
                serde_json::json!({
                    "phase": "download",
                    "downloaded": downloaded,
                    "total": total_size,
                    "percent": percent,
                    "filename": filename,
                }),
            );
        }

        drop(file);

        log_info!("Extracting RAGE:MP zip to {:?}", target_dir);
        let _ = window.emit(
            "ragemp-download-progress",
            serde_json::json!({ "phase": "extract", "percent": 0 }),
        );

        let zip_file = fs::File::open(&zip_path).map_err(|e| format!("Failed to open zip: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(zip_file).map_err(|e| format!("Failed to read zip: {}", e))?;

        let total_files = archive.len();
        let mut extracted = 0;

        for i in 0..total_files {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {}", e))?;

            let outpath = match entry.enclosed_name() {
                Some(path) => target_dir.join(path),
                None => continue,
            };

            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
            }
            if entry.is_dir() {
                continue;
            }

            let mut outfile =
                fs::File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Failed to write file: {}", e))?;

            extracted += 1;
            let percent = (extracted as f64 / total_files as f64 * 100.0) as u32;
            let _ = window.emit(
                "ragemp-download-progress",
                serde_json::json!({
                    "phase": "extract",
                    "percent": percent,
                    "extracted": extracted,
                    "total_files": total_files,
                }),
            );
        }

        let _ = fs::remove_file(&zip_path);
        log_info!("Extracted {} files to {:?}", extracted, target_dir);
    } else {
        let mut file =
            fs::File::create(&dest_path).map_err(|e| format!("Failed to create file: {}", e))?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Read error: {}", e))?;
            file.write_all(&chunk).map_err(|e| format!("Write error: {}", e))?;
            downloaded += chunk.len() as u64;

            let percent = if total_size > 0 {
                (downloaded as f64 / total_size as f64 * 100.0) as u32
            } else {
                0
            };

            let _ = window.emit(
                "ragemp-download-progress",
                serde_json::json!({
                    "phase": "download",
                    "downloaded": downloaded,
                    "total": total_size,
                    "percent": percent,
                    "filename": filename,
                }),
            );
        }

        log_info!("Downloaded RAGE:MP to {:?}", dest_path);
    }

    Ok(dest_path.to_string_lossy().to_string())
}
