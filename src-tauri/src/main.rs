//! ASTRA Launcher — Backend (Rust / Tauri v2)
//! =========================================
//! RAGE:MP launcher with 3-step system:
//!   1. Client file verification + download (SHA-256)
//!   2. Server cache auto-download
//!   3. Memory patching (CREATE_SUSPENDED + WriteProcessMemory)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod client;
mod config;
mod discord;
mod downloader;
mod logging;
mod manifest;
mod patcher;
mod paths;
mod rui;
mod server;
mod username;
mod window;

use std::fs;

use tauri::command;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent, TrayIconBuilder};
use tauri::Manager;

use crate::config::LauncherConfig;
use crate::logging::{log_error, log_info, log_warn};
use crate::manifest::CheckResult;
use crate::patcher::{LaunchOptions, PatchManifest};

/// ── Global State ──

static CONFIG: once_cell::sync::Lazy<LauncherConfig> =
    once_cell::sync::Lazy::new(LauncherConfig::load);

/// ── Tauri Commands: Client ──
//
// NOTE ON ASYNC: in Tauri, a *synchronous* command runs on the **main thread**
// and blocks the UI while it executes. `find_rage_dir` walks several directory
// trees and spawns PowerShell to resolve `.lnk` shortcuts, and the RUI commands
// read a 15 MB archive — none of that may happen on the main thread. Every
// command that touches the filesystem or the registry is therefore `async` and
// hands the work to `spawn_blocking`.

/// Run a blocking closure off the main thread, mapping join errors to strings.
async fn blocking<T, F>(what: &'static str, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| format!("{} task failed: {}", what, e))?
}

#[command]
async fn check_client_installed() -> Result<bool, String> {
    blocking("check_client_installed", || {
        Ok(client::check_client_installed(&CONFIG))
    })
    .await
}

#[command]
async fn get_rage_dir() -> Result<String, String> {
    blocking("get_rage_dir", || {
        Ok(match client::find_rage_dir(&CONFIG) {
            Some(path) => path.to_string_lossy().to_string(),
            None => String::new(),
        })
    })
    .await
}

/// Save the RAGE:MP folder.
///
/// Returns the *normalised* path plus whether it looks like a real install —
/// the caller must not assume success. A file path is converted to its parent
/// folder (the picker used to be a file dialog).
#[command]
async fn set_rage_dir(path: String) -> Result<serde_json::Value, String> {
    blocking("set_rage_dir", move || {
        let (normalized, valid) = client::save_rage_dir(&path)?;
        Ok(serde_json::json!({
            "path": normalized,
            "valid": valid,
        }))
    })
    .await
}

#[command]
async fn auto_find_rage_dir() -> Result<Option<String>, String> {
    blocking("auto_find_rage_dir", || {
        match client::find_rage_dir(&CONFIG) {
            Some(path) => {
                let path_str = path.to_string_lossy().to_string();
                let _ = client::save_rage_dir(&path_str);
                Ok(Some(path_str))
            }
            None => Ok(None),
        }
    })
    .await
}

/// ── Tauri Commands: Manifest + Download ──

#[command]
async fn check_client_files() -> Result<CheckResult, String> {
    let rage_dir = client::find_rage_dir(&CONFIG).ok_or("CLIENT_NOT_FOUND")?;
    match manifest::fetch_manifest_or_skip(&CONFIG.client_manifest_url).await? {
        Some(manifest) => Ok(manifest::verify_files(&rage_dir, &manifest)),
        None => Ok(CheckResult {
            valid: true,
            missing: Vec::new(),
            outdated: Vec::new(),
            total_download_size: 0,
            skipped: true,
        }),
    }
}

#[command]
async fn download_client_files(window: tauri::Window) -> Result<(), String> {
    let rage_dir = client::find_rage_dir(&CONFIG).ok_or("CLIENT_NOT_FOUND")?;
    let manifest = match manifest::fetch_manifest_or_skip(&CONFIG.client_manifest_url).await? {
        Some(m) => m,
        None => {
            log_info!("No client manifest configured — nothing to download");
            return Ok(());
        }
    };
    let check = manifest::verify_files(&rage_dir, &manifest);

    if check.valid {
        return Ok(());
    }

    let mut files_to_download = check.missing;
    files_to_download.extend(check.outdated);

    downloader::download_files(&rage_dir, &files_to_download, &CONFIG.file_download_base_url, &window).await
}

/// ── Tauri Commands: Server Cache ──

#[command]
async fn check_server_cache() -> Result<serde_json::Value, String> {
    let rage_dir = client::find_rage_dir(&CONFIG).ok_or("CLIENT_NOT_FOUND")?;
    match cache::check_cache_needed_or_skip(&rage_dir, &CONFIG.cache_info_url).await? {
        Some((needed, local_ver, server_ver, _info)) => Ok(serde_json::json!({
            "needed": needed,
            "skipped": false,
            "reason": if needed {
                if local_ver.is_empty() { "first_run".to_string() } else { format!("local '{}' != server '{}'", local_ver, server_ver) }
            } else { "up_to_date".to_string() },
            "version_local": local_ver,
            "version_server": server_ver,
        })),
        None => Ok(serde_json::json!({
            "needed": false,
            "skipped": true,
            "reason": "cache endpoint not configured or unreachable — trusting installed client",
            "version_local": cache::get_local_cache_version(&rage_dir),
            "version_server": "",
        })),
    }
}

#[command]
async fn download_server_cache(window: tauri::Window) -> Result<(), String> {
    let rage_dir = client::find_rage_dir(&CONFIG).ok_or("CLIENT_NOT_FOUND")?;
    let (_needed, _local_ver, _server_ver, cache_info) =
        match cache::check_cache_needed_or_skip(&rage_dir, &CONFIG.cache_info_url).await? {
            Some(v) => v,
            None => {
                log_info!("No cache endpoint configured — nothing to download");
                return Ok(());
            }
        };

    cache::download_cache(&rage_dir, &cache_info, &window).await
}

/// ── Tauri Commands: Patches + Launch ──

/// Resolve the patch table to use.
///
/// Priority: server-supplied manifest (when `patches_url` is configured and
/// reachable) → built-in table ported from tiny-ragemp-launcher. The built-in
/// fallback is what keeps the launcher working if the patch endpoint is down,
/// which is the whole point of the preserved project.
async fn resolve_patch_manifest() -> PatchManifest {
    if !CONFIG.patches_url.is_empty() {
        match patcher::download_patches(&CONFIG.patches_url).await {
            Ok(m) if !m.patches.is_empty() => {
                log_info!("Using server patch manifest v{}", m.version);
                return m;
            }
            Ok(_) => log_warn!("Server patch manifest was empty — falling back to built-in"),
            Err(e) => log_warn!("Server patch manifest unavailable ({}) — falling back to built-in", e),
        }
    } else {
        log_info!("patches_url not configured — using built-in patch table");
    }
    patcher::builtin_patch_manifest()
}

#[command]
async fn get_patch_manifest() -> Result<serde_json::Value, String> {
    let m = resolve_patch_manifest().await;
    Ok(serde_json::json!({
        "version": m.version,
        "ragemp_version": m.ragemp_version,
        "gta_build": m.gta_build,
        "count": m.patches.len(),
        "source": if m.version.starts_with("builtin") { "builtin" } else { "server" },
        "upstream": patcher::upstream_patch_source(),
        "patches": m.patches,
    }))
}

/// Report the current patch state of the installed updater.exe.
#[command]
async fn check_patch_status() -> Result<serde_json::Value, String> {
    let rage_dir = client::find_rage_dir(&CONFIG).ok_or("CLIENT_NOT_FOUND")?;
    let updater = rage_dir.join("updater.exe");
    let manifest = resolve_patch_manifest().await;

    let (applied, total) = patcher::patch_status(&updater, &manifest.patches)?;

    // Is the installed binary still the verified reference build?
    let on_reference = {
        let mut ok = true;
        let data = std::fs::read(&updater).map_err(|e| e.to_string())?;
        for p in &manifest.patches {
            let off = {
                let s = p.offset.trim();
                if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                    u64::from_str_radix(h, 16).unwrap_or(0) as usize
                } else {
                    s.parse().unwrap_or(0)
                }
            };
            let orig: Vec<u8> = p
                .original
                .split_whitespace()
                .filter_map(|x| u8::from_str_radix(x, 16).ok())
                .collect();
            if off + orig.len() > data.len() || data[off..off + orig.len()] != orig[..] {
                ok = false;
                break;
            }
        }
        ok
    };

    Ok(serde_json::json!({
        "applied": applied,
        "total": total,
        "patched": applied == total && total > 0,
        "reference_build": on_reference,
        "manifest_version": manifest.version,
        "updater_size": std::fs::metadata(&updater).map(|m| m.len()).unwrap_or(0),
    }))
}

/// Diagnostics for the two domain-whitelist workarounds.
#[command]
async fn get_whitelist_status() -> Result<serde_json::Value, String> {
    blocking("get_whitelist_status", || {
        let state = patcher::read_registry_state();

        let curated = CONFIG.hosts_domain.trim().to_string();
        let curated = if curated.is_empty() {
            // Derive the default domain we would map for this server.
            let ip = CONFIG
                .server_address
                .rsplit_once(':')
                .map(|(i, _)| i)
                .unwrap_or(&CONFIG.server_address);
            format!("astra-{}.gta5rp.com", ip.replace('.', "-"))
        } else {
            curated
        };

        let mapping = patcher::hosts_entry_for(&curated);

        Ok(serde_json::json!({
            "registry_ip": state.launch2_ip,
            "registry_port": state.launch2_port,
            "registry": state,
            "hosts_path": patcher::hosts_file_path().to_string_lossy(),
            "hosts_writable": patcher::hosts_writable(),
            "hosts_domain": curated,
            "hosts_domain_whitelisted": patcher::is_whitelisted_domain(&curated),
            "hosts_mapped": mapping.is_some(),
            "hosts_mapped_ip": mapping.map(|m| m.ip),
            "connect_mode": CONFIG.connect_mode,
            "rui_path": client::find_rage_dir(&CONFIG)
                .map(|d| d.join("rui").join("index.rui").to_string_lossy().to_string()),
        }))
    })
    .await
}

/// Save the preferred connect method (`raw` | `hosts`).
#[command]
async fn set_connect_mode(mode: String, hosts_domain: String) -> Result<(), String> {
    if mode != "raw" && mode != "hosts" {
        return Err(format!("unsupported connect mode '{}'", mode));
    }
    if mode == "hosts" && !patcher::is_whitelisted_domain(&hosts_domain) {
        return Err(format!(
            "'{}' is not whitelisted — it must end in .gta5rp.com or .grandrp.com",
            hosts_domain
        ));
    }
    blocking("set_connect_mode", move || {
        CONFIG.set_connect_mode(&mode, &hosts_domain);
        log_info!("Connect mode set to '{}' (domain='{}')", mode, hosts_domain);
        Ok(())
    })
    .await
}

/// Save which binary the launcher should hand control to.
#[command]
async fn set_launch_target(target: String) -> Result<serde_json::Value, String> {
    blocking("set_launch_target", move || {
        let t = patcher::LaunchTarget::parse(&target);
        CONFIG.set_launch_target(t.as_str());
        log_info!("Launch target set to '{}'", t.as_str());

        // Report whether the chosen binary is actually present, so the UI can warn.
        let available = client::find_rage_dir(&CONFIG).map(|dir| {
            let exe = match t {
                patcher::LaunchTarget::Updater => "updater.exe",
                patcher::LaunchTarget::RagempV => "ragemp_v.exe",
            };
            dir.join(exe).exists()
        });

        Ok(serde_json::json!({
            "target": t.as_str(),
            "binary_present": available,
        }))
    })
    .await
}

/// Toggle the rui auto-connect injection (off by default — it has been
/// reported to break the CEF launcher UI with "404 error: File not found").
#[command]
fn set_rui_auto_connect(enabled: bool) -> Result<serde_json::Value, String> {
    CONFIG.set_rui_auto_connect(enabled);
    log_info!("RUI auto-connect set to '{}'", enabled);
    Ok(serde_json::json!({
        "rui_auto_connect": enabled,
        "rui_injected": client::find_rage_dir(&CONFIG).map(|d| rui::is_injected(&d)).unwrap_or(false),
    }))
}

/// Full picture of what the next launch will actually do.
///
/// Answers the question "which path is in effect, and is it set up correctly?"
#[command]
async fn diagnose_launch() -> Result<serde_json::Value, String> {
    blocking("diagnose_launch", || {
        let rage_dir = client::find_rage_dir(&CONFIG);
        let target = patcher::LaunchTarget::parse(&CONFIG.launch_target);
        let reg = patcher::read_registry_state();

        let server_addr = CONFIG.server_address.clone();
        let (ip, port) = match server_addr.rsplit_once(':') {
            Some((i, p)) => (i.to_string(), p.to_string()),
            None => (server_addr.clone(), "22005".to_string()),
        };

        // Which binaries exist on disk?
        let (updater_present, client_present, rage_dir_str) = match &rage_dir {
            Some(d) => (
                d.join("updater.exe").exists(),
                d.join("ragemp_v.exe").exists(),
                d.to_string_lossy().to_string(),
            ),
            None => (false, false, String::new()),
        };

        let target_binary_present = match target {
            patcher::LaunchTarget::Updater => updater_present,
            patcher::LaunchTarget::RagempV => client_present,
        };

        // Is the registry in the state the chosen target needs?
        let registry_ok = reg.has_launch_family() && reg.has_launch2_family();
        let registry_matches_server = reg.launch_ip.as_deref() == Some(ip.as_str())
            && reg.launch_port.as_deref() == Some(port.as_str());

        // A raw IP is filtered by ragemp_v.exe's whitelist unless the rui
        // auto-connect bootstrap is doing the connecting instead.
        let rui_injected = rage_dir.as_ref().map(|d| rui::is_injected(d)).unwrap_or(false);
        let rui_bypass_enabled = CONFIG.connect_mode == "raw" && CONFIG.rui_auto_connect;
        let raw_ip_needs_bypass = CONFIG.connect_mode == "raw";
        let raw_ip_covered = rui_injected;
        // Before the first ASTRA launch, registry values and the RUI bootstrap
        // are expected to be absent: launch_patched writes/injects them as part
        // of the Play action. Distinguish that pending state from a failed
        // post-launch setup so diagnostics do not raise a false alarm.
        let rui_pending = rui_bypass_enabled && !rui_injected && !registry_matches_server;

        let mut notes: Vec<String> = Vec::new();
        if rage_dir.is_none() {
            notes.push("RAGE:MP не найден — укажите путь в настройках".to_string());
        }
        // RAGE:MP's default game port is 22005. 30120 is a non-standard port of another client, and
        // is almost always a leftover from the legacy config schema — and because
        // this value is written straight into `launch.port`, a wrong port means the
        // client connects to nothing.
        if port == "30120" {
            notes.push(
                "Порт 30120 — нестандартный порт. Для RAGE:MP обычно 22005, иначе игра подключится не туда"
                    .to_string(),
            );
        }
        if !target_binary_present {
            notes.push(format!(
                "Бинарник для выбранного пути ({}) не найден",
                match target {
                    patcher::LaunchTarget::Updater => "updater.exe",
                    patcher::LaunchTarget::RagempV => "ragemp_v.exe",
                }
            ));
        }
        if raw_ip_needs_bypass && !raw_ip_covered {
            if !CONFIG.rui_auto_connect {
                notes.push(
                    "Режим raw, но автоподключение отключено (патч rui убран из-за ошибки 404) — прямой IP может быть отброшен белым списком ragemp_v.exe. Включите rui_auto_connect в конфиге или переключитесь на режим hosts"
                        .to_string(),
                );
            } else if rui_pending {
                notes.push(
                    "До первого запуска через ASTRA автоподключение ещё не внедрено; оно будет добавлено при нажатии «Играть»".to_string(),
                );
            } else {
                notes.push(
                    "Прямой IP без внедрённого автоподключения будет отброшен белым списком".to_string(),
                );
            }
        }
        if !reg.has_launch_family() && reg.has_launch2_family() {
            notes.push(
                "В реестре только launch2.* — ragemp_v.exe читает launch.*, возможно потребуется повторный запуск"
                    .to_string(),
            );
        }
        if !reg.has_launch2_family() && reg.has_launch_family() {
            notes.push("В реестре только launch.* — старые лаунчеры читают launch2.*".to_string());
        }

        Ok(serde_json::json!({
            "rage_dir": rage_dir_str,
            "target": target.as_str(),
            "target_binary_present": target_binary_present,
            "updater_present": updater_present,
            "client_present": client_present,
            "server_ip": ip,
            "server_port": port,
            "connect_mode": CONFIG.connect_mode,
            "rui_auto_connect": CONFIG.rui_auto_connect,
            "registry": reg,
            "registry_ok": registry_ok,
            "registry_matches_server": registry_matches_server,
            "rui_injected": rui_injected,
            "rui_pending": rui_pending,
            "raw_ip_covered": raw_ip_covered,
            "notes": notes,
        }))
    })
    .await
}

/// Clear the direct-connect registry values (diagnostics / troubleshooting).
#[command]
async fn clear_launch_registry() -> Result<(), String> {
    blocking("clear_launch_registry", || patcher::clear_registry()).await
}

/// Validate and save the server address (`ip:port` or bare host).
///
/// This value is written into the registry as `launch.ip` / `launch.port`, so a
/// typo here silently breaks direct connect — hence the validation.
#[command]
async fn set_server_address(address: String) -> Result<serde_json::Value, String> {
    let trimmed = address.trim().to_string();
    if trimmed.is_empty() {
        return Err("Адрес не может быть пустым".to_string());
    }

    blocking("set_server_address", move || {
        let (host, port) = match trimmed.rsplit_once(':') {
            Some((h, p)) => (h.trim(), p.trim()),
            None => (trimmed.as_str(), "22005"),
        };

        if host.is_empty() {
            return Err("Не указан адрес сервера".to_string());
        }
        // Hostname or IPv4 — reject anything with spaces or scheme prefixes.
        if host.contains(' ') || host.contains('/') {
            return Err(format!("Некорректный адрес: '{}'", host));
        }
        if host.chars().all(|c| c.is_ascii_digit() || c == '.') {
            let octets: Vec<&str> = host.split('.').collect();
            if octets.len() != 4 || octets.iter().any(|o| o.parse::<u8>().is_err()) {
                return Err(format!("Некорректный IP-адрес: '{}'", host));
            }
        }

        let port_num: u16 = port
            .parse()
            .map_err(|_| format!("Некорректный порт: '{}'", port))?;
        if port_num == 0 {
            return Err("Порт не может быть нулевым".to_string());
        }

        let mut cfg = (*CONFIG).clone();
        cfg.server_address = format!("{}:{}", host, port_num);
        cfg.save();

        log_info!("Server address set to {}", cfg.server_address);

        Ok(serde_json::json!({
            "server_address": cfg.server_address,
            "host": host,
            "port": port_num,
            "rage_port_warning": port_num == 30120,
        }))
    })
    .await
}

/// Relaunch *this* executable elevated (UAC) with the given CLI arguments.
///
/// We reuse the same binary instead of shipping a separate helper, so there is
/// no sidecar to package. The elevated instance short-circuits in
/// `handle_cli_mode()` before Tauri ever initialises.
#[cfg(target_os = "windows")]
fn elevate_self(args: &[&str]) -> Result<(), String> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

    let exe = std::env::current_exe()
        .map_err(|e| format!("cannot resolve own path: {}", e))?;

    let params = args.join(" ");
    let to_wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };

    let verb = to_wide("runas");
    let file = to_wide(&exe.to_string_lossy());
    let param = to_wide(&params);

    unsafe {
        // ShellExecuteW returns as soon as the process has *started*; the
        // caller polls the hosts file to learn the real outcome.
        let result = ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR(param.as_ptr()),
            PCWSTR::null(),
            SW_HIDE,
        );

        // Values <= 32 indicate failure.
        let code = result.0 as isize;
        if code <= 32 {
            let msg = match code {
                5 => "administrator rights were refused (UAC prompt cancelled)".to_string(),
                _ => format!("ShellExecuteW failed with code {}", code),
            };
            log_error!("elevation failed: {}", msg);
            return Err(msg);
        }
    }

    log_info!("relaunched elevated: {}", params);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn elevate_self(_args: &[&str]) -> Result<(), String> {
    Err("hosts file editing is only supported on Windows".to_string())
}

/// Return the server IP for the configured address.
fn server_ip() -> String {
    CONFIG
        .server_address
        .rsplit_once(':')
        .map(|(ip, _)| ip.to_string())
        .unwrap_or_else(|| CONFIG.server_address.clone())
}

/// Apply the hosts-based whitelist bypass (requires administrator rights).
#[command]
async fn apply_whitelist_bypass() -> Result<serde_json::Value, String> {
    let ip = server_ip();
    let domain = {
        let d = CONFIG.hosts_domain.trim().to_string();
        if d.is_empty() {
            format!("astra-{}.gta5rp.com", ip.replace('.', "-"))
        } else {
            d
        }
    };

    if !patcher::is_whitelisted_domain(&domain) {
        return Err(format!(
            "'{}' is not whitelisted — it must end in .gta5rp.com or .grandrp.com",
            domain
        ));
    }

    // Try a direct write first — if the launcher already has the rights (or
    // the user runs it as administrator) no UAC prompt is needed.
    let entries = vec![patcher::HostsEntry {
        ip: ip.clone(),
        domain: domain.clone(),
    }];

    let direct = patcher::apply_hosts_mapping(&entries);
    if let Err(e) = &direct {
        log_info!("direct hosts write failed ({}), requesting elevation", e);
        elevate_self(&["--hosts-apply", &ip, &domain])?;
    }

    // Poll — either the write landed instantly, or the elevated instance is
    // still starting up behind a UAC prompt.
    let mut mapped = false;
    for _ in 0..40 {
        if let Some(e) = patcher::hosts_entry_for(&domain) {
            if e.ip == ip {
                mapped = true;
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    if !mapped {
        return Err(
            "hosts mapping was not written — the UAC prompt was likely cancelled".to_string(),
        );
    }

    CONFIG.set_connect_mode("hosts", &domain);

    log_info!("Whitelist bypass active: {} → {}", domain, ip);
    Ok(serde_json::json!({
        "success": true,
        "domain": domain,
        "ip": ip,
        "connect_mode": "hosts",
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tauri Commands: RUI archive editor
// ─────────────────────────────────────────────────────────────────────────────

/// Path of the persisted RUI editor config.
fn rui_config_path() -> std::path::PathBuf {
    paths::launcher_data_dir().join("rui_config.json")
}

/// Report the state of the installed `rui\index.rui`.
///
/// Uses [`rui::inspect`], which reads only the header tables — a full
/// [`rui::Archive::parse`] would copy all 265 files (15 MB) just to render a
/// status line.
#[command]
async fn get_rui_status() -> Result<serde_json::Value, String> {
    blocking("get_rui_status", || {
        let Some(rage_dir) = client::find_rage_dir(&CONFIG) else {
            return Ok(serde_json::json!({ "available": false, "reason": "CLIENT_NOT_FOUND" }));
        };
        let path = rui::rui_path(&rage_dir);

        if !path.exists() {
            return Ok(serde_json::json!({
                "available": false,
                "reason": "RUI_NOT_FOUND",
                "path": path.to_string_lossy(),
            }));
        }

        let data =
            std::fs::read(&path).map_err(|e| format!("failed to read index.rui: {}", e))?;
        let inspected = rui::inspect(&data);

        let (entries, contiguous, app_js, index_html) = match &inspected {
            Ok(i) => (
                i.entries,
                i.contiguous,
                i.app_js_index,
                i.index_html_index,
            ),
            Err(_) => (0, false, None, None),
        };

        Ok(serde_json::json!({
            "available": true,
            "path": path.to_string_lossy(),
            "size": data.len(),
            "parsable": inspected.is_ok(),
            "parse_error": inspected.as_ref().err(),
            "entries": entries,
            "contiguous": contiguous,
            "app_js_index": app_js,
            "index_html_index": index_html,
            "injected": rui::is_injected(&rage_dir),
            "backup_exists": rui::backup_path(&path).exists(),
            "backup_path": rui::backup_path(&path).to_string_lossy(),
        }))
    })
    .await
}

/// Load the saved RUI editor config (returns defaults when absent).
#[command]
async fn get_rui_config() -> Result<serde_json::Value, String> {
    blocking("get_rui_config", || {
        let path = rui_config_path();
        if !path.exists() {
            return Ok(serde_json::json!({
                "title": null,
                "auto_connect": { "enabled": false, "ip": "", "port": 22005, "name": "AutoConnect", "delay_ms": 2000 },
                "server_list": { "override": false, "servers": [] },
                "colors": {},
                "texts": [],
                "images": {},
            }));
        }
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read rui_config.json: {}", e))?;
        serde_json::from_str(&raw).map_err(|e| format!("invalid rui_config.json: {}", e))
    })
    .await
}

/// Persist the RUI editor config.
#[command]
async fn save_rui_config(config: serde_json::Value) -> Result<(), String> {
    blocking("save_rui_config", move || {
        // Validate by round-tripping through the strongly-typed struct.
        let _: rui::RuiConfig = serde_json::from_value(config.clone())
            .map_err(|e| format!("invalid RUI config: {}", e))?;

        let path = rui_config_path();
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&config).unwrap_or_default(),
        )
        .map_err(|e| format!("failed to write rui_config.json: {}", e))?;
        log_info!("RUI config saved to {:?}", path);
        Ok(())
    })
    .await
}

/// Apply an RUI editor config to the installed `index.rui`.
///
/// Reads, rewrites and re-verifies a 15 MB archive — must never run on the
/// main thread.
#[command]
async fn apply_rui_config(config: serde_json::Value) -> Result<serde_json::Value, String> {
    blocking("apply_rui_config", move || {
        let rage_dir = client::find_rage_dir(&CONFIG).ok_or("CLIENT_NOT_FOUND")?;
        let parsed: rui::RuiConfig = serde_json::from_value(config)
            .map_err(|e| format!("invalid RUI config: {}", e))?;

        let report = rui::apply_config(&rage_dir, &parsed)?;
        serde_json::to_value(report).map_err(|e| format!("failed to encode report: {}", e))
    })
    .await
}

/// Preview what an RUI config would change, without writing anything.
///
/// Parses the full 15 MB archive — off the main thread.
#[command]
async fn preview_rui_config(config: serde_json::Value) -> Result<serde_json::Value, String> {
    blocking("preview_rui_config", move || {
    let rage_dir = client::find_rage_dir(&CONFIG).ok_or("CLIENT_NOT_FOUND")?;
    let path = rui::rui_path(&rage_dir);
    if !path.exists() {
        return Err("index.rui not found".to_string());
    }

    let parsed: rui::RuiConfig = serde_json::from_value(config)
        .map_err(|e| format!("invalid RUI config: {}", e))?;

    let data = std::fs::read(&path).map_err(|e| format!("failed to read index.rui: {}", e))?;
    let archive = rui::Archive::parse(&data)?;

    // Mirror apply_config's logic but never touch the file.
    let mut changes: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    if let Some(i) = archive.app_js_index() {
        let before = archive.blobs[i].clone();
        let mut blob = before.clone();

        if parsed.auto_connect.enabled {
            match rui::apply_auto_connect(&blob, &parsed.auto_connect) {
                Ok(next) => {
                    blob = next;
                    changes.push(format!(
                        "auto-connect -> {}:{} ({}ms)",
                        parsed.auto_connect.ip, parsed.auto_connect.port, parsed.auto_connect.delay_ms
                    ));
                }
                Err(e) => warnings.push(format!("auto-connect: {}", e)),
            }
        }
        if parsed.server_list.override_list {
            match rui::apply_server_list(&blob, &parsed.server_list) {
                Ok(next) => {
                    blob = next;
                    changes.push(format!("server list override: {} entr(ies)", parsed.server_list.servers.len()));
                }
                Err(e) => warnings.push(format!("server list: {}", e)),
            }
        }
        if !parsed.colors.is_empty() {
            let (next, n) = rui::apply_colors(&blob, &parsed.colors);
            blob = next;
            if n > 0 {
                changes.push(format!("{} colour replacement(s)", n));
            }
        }
        if !parsed.texts.is_empty() {
            let (next, n) = rui::apply_texts(&blob, &parsed.texts);
            blob = next;
            if n > 0 {
                changes.push(format!("{} text replacement(s)", n));
            }
        }
        if blob == before {
            warnings.push("app.js would be unchanged".to_string());
        }
    } else {
        warnings.push("app.js not found in archive".to_string());
    }

    if let Some(i) = archive.index_html_index() {
        if let Some(title) = &parsed.title {
            let (_, applied) = rui::apply_title(&archive.blobs[i], title);
            if applied {
                changes.push(format!("title -> {}", title));
            }
        }
    }

    for key in parsed.images.keys() {
        match key.parse::<usize>() {
            Ok(idx) if idx < archive.blobs.len() => {
                changes.push(format!("image idx{} replaced", idx));
            }
            Ok(idx) => warnings.push(format!("image index {} out of range", idx)),
            Err(_) => warnings.push(format!("image key '{}' is not an index", key)),
        }
    }

    Ok(serde_json::json!({
        "changes": changes,
        "warnings": warnings,
    }))
    })
    .await
}

/// Restore `rui\index.rui` from the backup made by the archive editor.
#[command]
async fn restore_rui() -> Result<serde_json::Value, String> {
    blocking("restore_rui", || {
        let rage_dir = client::find_rage_dir(&CONFIG).ok_or("CLIENT_NOT_FOUND")?;
        let path = rui::rui_path(&rage_dir);
        let restored = rui::restore_backup(&path)?;
        Ok(serde_json::json!({ "success": true, "restored": restored }))
    })
    .await
}

/// Remove our hosts entries (requires administrator rights).
#[command]
async fn remove_whitelist_bypass() -> Result<serde_json::Value, String> {
    if let Err(e) = patcher::apply_hosts_mapping(&[]) {
        log_info!("direct hosts clean failed ({}), requesting elevation", e);
        elevate_self(&["--hosts-clean"])?;
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    }

    // Spin the config back to the raw-IP method.
    let domain = CONFIG.hosts_domain.clone();
    if !domain.is_empty() && patcher::hosts_entry_for(&domain).is_none() {
        CONFIG.set_connect_mode("raw", &domain);
    }

    Ok(serde_json::json!({ "success": true, "connect_mode": "raw" }))
}

#[command]
async fn download_ragemp(window: tauri::Window) -> Result<serde_json::Value, String> {
    let rage_dir = client::find_rage_dir(&CONFIG)
        .unwrap_or_else(|| {
            // Default: %LocalAppData%\RageMP
            let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
            std::path::PathBuf::from(local_appdata).join("RageMP")
        });

    let result = patcher::download_ragemp(
        &CONFIG.ragemp_download_url,
        &rage_dir,
        &window,
    ).await?;

    // Update config with the rage dir
    let mut config = (*CONFIG).clone();
    config.rage_dir = rage_dir.to_string_lossy().to_string();
    config.save();

    Ok(serde_json::json!({
        "success": true,
        "path": result,
        "rage_dir": rage_dir.to_string_lossy(),
    }))
}

#[command]
async fn launch_patched() -> Result<serde_json::Value, String> {
    let rage_dir = client::find_rage_dir(&CONFIG).ok_or("CLIENT_NOT_FOUND")?;

    // Check updater.exe exists
    let updater = rage_dir.join("updater.exe");
    if !updater.exists() {
        return Err("updater.exe not found in RAGE:MP directory".to_string());
    }

    // updater.exe is no longer patched: the binary patches were tied to one
    // specific build, broke the client's install-integrity check ("installation
    // is incomplete") and are not needed for auto-connect — the rui injection
    // does that job. Passing an empty table keeps the file pristine.

    // Parse server address (ip:port)
    let server_addr = &CONFIG.server_address;
    let (ip, port) = if let Some(colon_pos) = server_addr.rfind(':') {
        (
            server_addr[..colon_pos].to_string(),
            server_addr[colon_pos + 1..].to_string(),
        )
    } else {
        (server_addr.clone(), "22005".to_string())
    };

    let opts = LaunchOptions {
        connect_mode: if CONFIG.connect_mode.is_empty() {
            "raw".to_string()
        } else {
            CONFIG.connect_mode.clone()
        },
        hosts_domain: CONFIG.hosts_domain.clone(),
        target: patcher::LaunchTarget::parse(&CONFIG.launch_target),
        rui_auto_connect: CONFIG.rui_auto_connect,
        ragemp_v_patches: CONFIG.ragemp_v_patches.clone(),
    };

    // Launch: resolve target → registry → patch → (rui) → spawn
    let result = patcher::launch_ragemp(&rage_dir, &ip, &port, &[], &opts)?;

    let pid = result.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
    log_info!("RAGE:MP launched with PID={}", pid);
    Ok(result)
}

/// ── Tauri Commands: Server ──

#[command]
async fn get_server_status() -> Result<serde_json::Value, String> {
    server::get_server_status(&CONFIG).await
}

#[command]
async fn get_server_ping() -> Result<serde_json::Value, String> {
    server::get_server_ping(&CONFIG).await
}

/// ── Tauri Commands: Discord ──

#[command]
async fn set_discord_rpc(state: String, details: String, large_text: String) -> Result<bool, String> {
    // Opening the Discord IPC pipe blocks until it connects or times out.
    blocking("set_discord_rpc", move || {
        discord::set_discord_rpc(&state, &details, &large_text)
    })
    .await
}

/// ── Tauri Commands: Username ──

#[command]
async fn get_username() -> Result<String, String> {
    blocking("get_username", username::get_username).await
}

#[command]
async fn save_username(name: String) -> Result<(), String> {
    blocking("save_username", move || username::save_username(&name)).await
}

/// ── Tauri Commands: Config ──

#[command]
async fn get_launcher_config() -> Result<serde_json::Value, String> {
    blocking("get_launcher_config", || {
        Ok(serde_json::json!({
            "server_address": CONFIG.server_address,
            "client_manifest_url": CONFIG.client_manifest_url,
            "cache_info_url": CONFIG.cache_info_url,
            "patches_url": CONFIG.patches_url,
            "file_download_base_url": CONFIG.file_download_base_url,
            "ragemp_download_url": CONFIG.ragemp_download_url,
            "rage_dir": CONFIG.rage_dir,
            "gta5_path": CONFIG.gta5_path,
            "connect_mode": CONFIG.connect_mode,
            "hosts_domain": CONFIG.hosts_domain,
            "launch_target": CONFIG.launch_target,
        }))
    })
    .await
}

/// ── Tauri Commands: Onboarding ──

#[command]
async fn is_onboarding_complete() -> Result<bool, String> {
    blocking("is_onboarding_complete", || {
        Ok(paths::onboarding_flag_path().exists())
    })
    .await
}

#[command]
async fn complete_onboarding() -> Result<(), String> {
    blocking("complete_onboarding", || {
        fs::write(
            paths::onboarding_flag_path(),
            chrono::Utc::now().to_rfc3339(),
        )
        .map_err(|e| format!("Failed to write onboarding flag: {}", e))
    })
    .await
}

/// ── Tauri Commands: Window ──

#[command]
fn show_main_window(window: tauri::Window) -> Result<(), String> {
    window::show_main_window(&window)
}

#[command]
async fn create_embedded_webview(url: String, window: tauri::Window) -> Result<(), String> {
    window::create_embedded_webview(url, &window).await
}

#[command]
fn close_embedded_webview(window: tauri::Window) -> Result<(), String> {
    window::close_embedded_webview(&window)
}

#[command]
fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window::window_minimize(&window)
}

#[command]
fn window_close(window: tauri::Window) -> Result<(), String> {
    window::window_close(&window)
}

/// ── Entry Point ──

/// Handle the elevated-helper invocations.
///
/// When the launcher needs to write the hosts file it relaunches *itself*
/// with `--hosts-apply` / `--hosts-clean`. That second instance must not boot
/// the UI — it performs the write and exits with a status code:
///   0 = success, 2/3 = bad arguments, 5 = write denied.
fn handle_cli_mode() -> Option<i32> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--hosts-apply") => {
            let ip = args.get(2).cloned().unwrap_or_default();
            let domain = args.get(3).cloned().unwrap_or_default();
            if ip.trim().is_empty() {
                return Some(2);
            }
            if domain.trim().is_empty() {
                return Some(3);
            }
            let entries = vec![patcher::HostsEntry { ip, domain }];
            Some(match patcher::apply_hosts_mapping(&entries) {
                Ok(()) => 0,
                Err(e) => {
                    log_error!("elevated hosts write failed: {}", e);
                    5
                }
            })
        }
        Some("--hosts-clean") => Some(match patcher::apply_hosts_mapping(&[]) {
            Ok(()) => 0,
            Err(e) => {
                log_error!("elevated hosts clean failed: {}", e);
                5
            }
        }),
        _ => None,
    }
}

fn main() {
    // Elevated helper mode — never initialise Tauri here.
    if let Some(code) = handle_cli_mode() {
        std::process::exit(code);
    }

    log_info!("ASTRA Launcher v{} (RAGE:MP) started", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // System tray
            let show_item = MenuItemBuilder::with_id("show", "Show launcher").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("ASTRA Launcher")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Client
            check_client_installed,
            get_rage_dir,
            set_rage_dir,
            auto_find_rage_dir,
            // Manifest + Download
            check_client_files,
            download_client_files,
            // Server Cache
            check_server_cache,
            download_server_cache,
            // Patches + Launch
            get_patch_manifest,
            check_patch_status,
            download_ragemp,
            launch_patched,
            // Domain whitelist bypass
            get_whitelist_status,
            set_connect_mode,
            apply_whitelist_bypass,
            remove_whitelist_bypass,
            // Launch path selection + diagnostics
            set_launch_target,
            set_rui_auto_connect,
            diagnose_launch,
            clear_launch_registry,
            set_server_address,
            // RUI archive editor
            get_rui_status,
            get_rui_config,
            save_rui_config,
            apply_rui_config,
            preview_rui_config,
            restore_rui,
            // Server
            get_server_status,
            get_server_ping,
            // Discord
            set_discord_rpc,
            // Username
            get_username,
            save_username,
            // Config
            get_launcher_config,
            // Onboarding
            is_onboarding_complete,
            complete_onboarding,
            // Window
            show_main_window,
            create_embedded_webview,
            close_embedded_webview,
            window_minimize,
            window_close,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Tauri");
}
