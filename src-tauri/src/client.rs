use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::config::LauncherConfig;
use crate::logging::{log_info, log_warn};
use crate::paths::settings_path;

/// Does this folder look like a RAGE:MP install?
///
/// `GTA5.exe` is what the launcher actually needs, but a RAGE:MP folder is
/// also recognisable by `ragemp_v.exe` — accepting either makes detection far
/// more forgiving when the user points at the right folder.
fn is_rage_dir(dir: &Path) -> bool {
    dir.join("GTA5.exe").exists() || dir.join("ragemp_v.exe").exists()
}

/// Turn a user-supplied path into a directory.
///
/// The path picker used to be a *file* dialog, so people ended up with values
/// like `C:\RAGEMP\uninstall.exe` stored as the client folder. If we are handed
/// a file, fall back to its parent directory.
pub fn normalize_rage_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path.trim());
    if p.is_file() {
        if let Some(parent) = p.parent() {
            return parent.to_path_buf();
        }
    }
    p
}

/// Settings keys that may hold the client folder, in priority order.
///
/// `rage_dir` is what we write; `rage_client_path` is the legacy key present in
/// existing installs — reading both avoids losing a working path on upgrade.
const RAGE_DIR_KEYS: [&str; 2] = ["rage_dir", "rage_client_path"];

/// Look up the saved client folder in `settings.json`.
fn rage_dir_from_settings() -> Option<PathBuf> {
    let data = fs::read_to_string(settings_path()).ok()?;
    let settings: serde_json::Value = serde_json::from_str(&data).ok()?;

    for key in RAGE_DIR_KEYS {
        if let Some(raw) = settings.get(key).and_then(|v| v.as_str()) {
            if raw.trim().is_empty() {
                continue;
            }
            let dir = normalize_rage_path(raw);
            if dir.exists() && is_rage_dir(&dir) {
                return Some(dir);
            }
        }
    }
    None
}

pub fn find_rage_dir(config: &LauncherConfig) -> Option<PathBuf> {
    // 1. User-saved path from config.json
    if !config.rage_dir.is_empty() {
        let path = normalize_rage_path(&config.rage_dir);
        if path.exists() && is_rage_dir(&path) {
            return Some(path);
        }
    }

    // 2. User-saved path from settings.json
    if let Some(dir) = rage_dir_from_settings() {
        return Some(dir);
    }

    let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let home = std::env::var("USERPROFILE").unwrap_or_default();

    // `C:\RAGEMP` is the folder the official installer actually creates, and it
    // was missing from this list — which is why auto-find never located it.
    let search_dirs: Vec<PathBuf> = vec![
        PathBuf::from("C:\\RAGEMP"),
        PathBuf::from("D:\\RAGEMP"),
        PathBuf::from("E:\\RAGEMP"),
        PathBuf::from(&local_appdata).join("RAGEMP"),
        PathBuf::from(&local_appdata).join("RageMP"),
        PathBuf::from(&local_appdata).join("Rage"),
        PathBuf::from(&home).join("Desktop").join("RAGEMP"),
        PathBuf::from(&home).join("Desktop").join("Rage"),
        PathBuf::from("C:\\Rage"),
        PathBuf::from("D:\\Rage"),
        PathBuf::from("E:\\Rage"),
    ];

    for dir in &search_dirs {
        if dir.exists() && is_rage_dir(dir) {
            return Some(dir.clone());
        }
    }

    // 3. Search .lnk shortcuts on Desktop
    let desktop = PathBuf::from(&home).join("Desktop");
    if let Some(found) = search_lnk_recursive(&desktop, "Rage") {
        return Some(found);
    }

    // Also common Start Menu
    let roaming = std::env::var("APPDATA").unwrap_or_default();
    if !roaming.is_empty() {
        let start_menu = PathBuf::from(&roaming)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        if start_menu.exists() {
            if let Some(found) = search_lnk_recursive(&start_menu, "Rage") {
                return Some(found);
            }
        }
    }

    let common_start = PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs");
    if common_start.exists() {
        if let Some(found) = search_lnk_recursive(&common_start, "Rage") {
            return Some(found);
        }
    }

    // 4. Recursive search in LocalAppData for Rage*
    if !local_appdata.is_empty() {
        let la = PathBuf::from(&local_appdata);
        if let Ok(entries) = fs::read_dir(&la) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.starts_with("rage") && entry.path().is_dir() {
                    if let Some(found) = search_gta5_recursive(&entry.path(), 3) {
                        return Some(found);
                    }
                }
            }
        }
    }

    None
}

fn search_lnk_recursive(dir: &Path, name_contains: &str) -> Option<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return None;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = search_lnk_recursive(&path, name_contains) {
                return Some(found);
            }
        } else {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name
                .to_lowercase()
                .contains(&name_contains.to_lowercase())
                && file_name.ends_with(".lnk")
            {
                if let Some(target) = resolve_lnk_target(&path) {
                    if target.exists() {
                        // The .lnk might point to RageMP.exe, check parent dir
                        if let Some(parent) = target.parent() {
                            if is_rage_dir(parent) {
                                return Some(parent.to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn resolve_lnk_target(lnk_path: &Path) -> Option<PathBuf> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!(
                "(New-Object -ComObject WScript.Shell).CreateShortcut('{}').TargetPath",
                lnk_path
                    .to_string_lossy()
                    .replace('\'', "''")
            ),
        ])
        .creation_flags(0x00000008) // CREATE_NO_WINDOW
        .output()
        .ok()?;

    let target = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if target.is_empty() || !target.contains(':') {
        return None;
    }
    Some(PathBuf::from(target))
}

#[cfg(not(target_os = "windows"))]
fn resolve_lnk_target(_lnk_path: &Path) -> Option<PathBuf> {
    None
}

fn search_gta5_recursive(dir: &Path, max_depth: u32) -> Option<PathBuf> {
    if max_depth == 0 {
        return None;
    }

    if is_rage_dir(dir) {
        return Some(dir.to_path_buf());
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = search_gta5_recursive(&path, max_depth - 1) {
                    return Some(found);
                }
            }
        }
    }

    None
}

/// Save the RAGE:MP client folder.
///
/// The incoming value is normalised first: if a *file* is passed (the picker
/// used to be a file dialog, so `C:\RAGEMP\uninstall.exe` was a realistic
/// value) its parent directory is stored instead.
///
/// Returns the normalised folder and whether it actually looks like a RAGE:MP
/// install, so the caller can report the truth instead of assuming success.
pub fn save_rage_dir(path: &str) -> Result<(String, bool), String> {
    let dir = normalize_rage_path(path);
    let dir_str = dir.to_string_lossy().to_string();
    let valid = dir.exists() && is_rage_dir(&dir);

    if !valid {
        log_warn!(
            "saved RAGE:MP path '{}' does not look like a RAGE:MP folder (no GTA5.exe / ragemp_v.exe)",
            dir_str
        );
    }

    let sp = settings_path();
    let mut settings: serde_json::Value = if sp.exists() {
        fs::read_to_string(&sp)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Write the current key and keep the legacy one in sync, so older builds
    // (and the pre-existing settings.json) agree on the path.
    settings["rage_dir"] = serde_json::Value::String(dir_str.clone());
    settings["rage_client_path"] = serde_json::Value::String(dir_str.clone());

    fs::write(&sp, serde_json::to_string_pretty(&settings).unwrap())
        .map_err(|e| format!("Ошибка сохранения: {}", e))?;

    Ok((dir_str, valid))
}

pub fn check_client_installed(config: &LauncherConfig) -> bool {
    if let Some(rage_dir) = find_rage_dir(config) {
        let gta5 = rage_dir.join("GTA5.exe");
        if gta5.exists() {
            if let Ok(meta) = fs::metadata(&gta5) {
                if meta.len() > 1_000_000 {
                    log_info!(
                        "Client installed: GTA5.exe found at {:?} ({} MB)",
                        rage_dir,
                        meta.len() / 1_048_576
                    );
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("astra-client-test-{}-{}", label, nonce))
    }

    #[test]
    fn recognizes_install_with_gta5_exe() {
        let root = test_dir("gta");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("GTA5.exe"), b"test").unwrap();
        assert!(is_rage_dir(&root));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recognizes_install_with_ragemp_v_exe() {
        let root = test_dir("client");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("ragemp_v.exe"), b"test").unwrap();
        assert!(is_rage_dir(&root));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_unrelated_folder() {
        let root = test_dir("unrelated");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("uninstall.exe"), b"test").unwrap();
        assert!(!is_rage_dir(&root));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn normalizes_exe_path_to_parent_folder() {
        let root = test_dir("normalize");
        fs::create_dir_all(&root).unwrap();
        let exe = root.join("uninstall.exe");
        fs::write(&exe, b"test").unwrap();

        let normalized = normalize_rage_path(&exe.to_string_lossy());
        assert_eq!(normalized, root);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn leaves_directory_path_unchanged() {
        let root = test_dir("directory");
        fs::create_dir_all(&root).unwrap();
        let normalized = normalize_rage_path(&root.to_string_lossy());
        assert_eq!(normalized, root);
        fs::remove_dir_all(root).unwrap();
    }
}
